use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::model::{Host, HostView, TunnelStatus};
use crate::ssh::process::stop_tunnel;
use crate::state::AppState;
use crate::store::write_json;
use crate::util::lock_error;
use crate::validate::validate_host;

#[tauri::command]
pub fn list_hosts(state: State<AppState>) -> Result<Vec<HostView>, String> {
    let hosts = state.hosts.lock().map_err(lock_error)?.clone();
    Ok(hosts.into_iter().map(|host| state.host_view(host)).collect())
}

#[tauri::command]
pub fn save_host(mut host: Host, state: State<AppState>, app: AppHandle) -> Result<HostView, String> {
    validate_host(&host)?;
    if host.id.trim().is_empty() {
        host.id = Uuid::new_v4().to_string();
    }

    let mut hosts = state.hosts.lock().map_err(lock_error)?;
    if let Some(existing) = hosts.iter_mut().find(|item| item.id == host.id) {
        // 保留已有转发列表，仅更新连接参数。
        host.forwards = existing.forwards.clone();
        *existing = host.clone();
    } else {
        hosts.push(host.clone());
    }
    write_json(state.hosts_path(), &*hosts)?;
    drop(hosts);
    state.add_log("info", format!("[{}] host saved", host.name), Some(&app));
    Ok(state.host_view(host))
}

#[tauri::command]
pub fn delete_host(id: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    // 先断开该主机下所有运行中的转发。
    let host = state.find_host(&id)?;
    for forward in &host.forwards {
        if matches!(state.status_for(&forward.id), TunnelStatus::Running) {
            stop_tunnel(&forward.id, state.inner(), Some(&app))?;
        }
    }

    let mut hosts = state.hosts.lock().map_err(lock_error)?;
    hosts.retain(|item| item.id != id);
    write_json(state.hosts_path(), &*hosts)?;
    drop(hosts);
    state.add_log("info", format!("[{}] host deleted", host.name), Some(&app));
    Ok(())
}
