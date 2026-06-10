use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::model::{Forward, HostView, TunnelStatus};
use crate::portcheck::detect_conflict;
use crate::ssh::process::{start_tunnel, stop_tunnel};
use crate::state::AppState;
use crate::store::write_json;
use crate::util::{lock_error, now_millis};
use crate::validate::validate_forward;

fn language_is_zh(state: &AppState) -> bool {
    state
        .settings
        .lock()
        .map(|settings| settings.language == "zh-CN")
        .unwrap_or(true)
}

/// 端口冲突检查：仅对 local / dynamic（监听本机端口）生效。
fn ensure_port_free(state: &AppState, forward: &Forward) -> Result<(), String> {
    if !forward.binds_local_port() {
        return Ok(());
    }
    if let Some(conflict) = detect_conflict(state, &forward.bind_port, &forward.id) {
        return Err(conflict.message(language_is_zh(state), forward.bind_port.trim()));
    }
    Ok(())
}

#[tauri::command]
pub fn save_forward(
    host_id: String,
    mut forward: Forward,
    state: State<AppState>,
    app: AppHandle,
) -> Result<HostView, String> {
    // 新建/编辑转发时不校验参数，也不做端口占用检查；这些都留到「连接」运行时判断。
    if forward.id.trim().is_empty() {
        forward.id = Uuid::new_v4().to_string();
    }

    let mut hosts = state.hosts.lock().map_err(lock_error)?;
    let host = hosts
        .iter_mut()
        .find(|item| item.id == host_id)
        .ok_or_else(|| "Host not found.".to_string())?;
    if let Some(existing) = host.forwards.iter_mut().find(|item| item.id == forward.id) {
        *existing = forward.clone();
    } else {
        host.forwards.push(forward.clone());
    }
    host.updated_at = now_millis();
    let host_name = host.name.clone();
    write_json(state.hosts_path(), &*hosts)?;
    drop(hosts);
    state.add_log("info", format!("[{}/{}] forward saved", host_name, forward.name), Some(&app));
    Ok(state.host_view(state.find_host(&host_id)?))
}

#[tauri::command]
pub fn delete_forward(
    host_id: String,
    forward_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<HostView, String> {
    if matches!(state.status_for(&forward_id), TunnelStatus::Running) {
        return Err("Disconnect this forward before deleting it.".to_string());
    }
    let mut hosts = state.hosts.lock().map_err(lock_error)?;
    let host = hosts
        .iter_mut()
        .find(|item| item.id == host_id)
        .ok_or_else(|| "Host not found.".to_string())?;
    let name = host
        .forwards
        .iter()
        .find(|item| item.id == forward_id)
        .map(|item| item.name.clone())
        .unwrap_or_else(|| forward_id.clone());
    host.forwards.retain(|item| item.id != forward_id);
    host.updated_at = now_millis();
    let host_name = host.name.clone();
    write_json(state.hosts_path(), &*hosts)?;
    drop(hosts);
    state.add_log("info", format!("[{}/{}] forward deleted", host_name, name), Some(&app));
    Ok(state.host_view(state.find_host(&host_id)?))
}

#[tauri::command]
pub fn connect_forward(
    host_id: String,
    forward_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<HostView, String> {
    let (host, forward) = state.find_forward(&host_id, &forward_id)?;
    if matches!(state.status_for(&forward_id), TunnelStatus::Running) {
        return Ok(state.host_view(host));
    }
    validate_forward(&forward)?;
    ensure_port_free(state.inner(), &forward)?;
    start_tunnel(host, forward, state.inner(), app, None)?;
    Ok(state.host_view(state.find_host(&host_id)?))
}

#[tauri::command]
pub fn connect_forward_with_password(
    host_id: String,
    forward_id: String,
    password: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<HostView, String> {
    if password.is_empty() {
        return Err("Password is required.".to_string());
    }
    let (host, forward) = state.find_forward(&host_id, &forward_id)?;
    if matches!(state.status_for(&forward_id), TunnelStatus::Running) {
        return Ok(state.host_view(host));
    }
    validate_forward(&forward)?;
    ensure_port_free(state.inner(), &forward)?;
    start_tunnel(host, forward, state.inner(), app, Some(password))?;
    Ok(state.host_view(state.find_host(&host_id)?))
}

#[tauri::command]
pub fn disconnect_forward(
    host_id: String,
    forward_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<HostView, String> {
    stop_tunnel(&forward_id, state.inner(), Some(&app))?;
    Ok(state.host_view(state.find_host(&host_id)?))
}

#[tauri::command]
pub fn disconnect_host(
    host_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<HostView, String> {
    let host = state.find_host(&host_id)?;
    for forward in &host.forwards {
        if matches!(state.status_for(&forward.id), TunnelStatus::Running) {
            stop_tunnel(&forward.id, state.inner(), Some(&app))?;
        }
    }
    Ok(state.host_view(state.find_host(&host_id)?))
}

#[tauri::command]
pub fn disconnect_all(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let ids: Vec<String> = state.tunnels.lock().map_err(lock_error)?.keys().cloned().collect();
    for id in ids {
        stop_tunnel(&id, state.inner(), Some(&app))?;
    }
    Ok(())
}
