use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::model::{Host, HostView, TunnelStatus};
use crate::ssh::process::stop_tunnel;
use crate::state::AppState;
use crate::store::write_json;
use crate::util::{lock_error, now_millis};

#[tauri::command]
pub fn list_hosts(state: State<AppState>) -> Result<Vec<HostView>, String> {
    let mut hosts = state.hosts.lock().map_err(lock_error)?.clone();
    // 置顶优先，其余按最后修改时间从新到旧；同序保持稳定。
    hosts.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.updated_at.cmp(&a.updated_at)));
    Ok(hosts.into_iter().map(|host| state.host_view(host)).collect())
}

#[tauri::command]
pub fn save_host(mut host: Host, state: State<AppState>, app: AppHandle) -> Result<HostView, String> {
    // 新建/编辑主机时不校验参数，参数是否可用留到连接（探测/上传/转发）运行时判断。
    if host.id.trim().is_empty() {
        host.id = Uuid::new_v4().to_string();
    }
    host.updated_at = now_millis();

    let mut hosts = state.hosts.lock().map_err(lock_error)?;
    if let Some(existing) = hosts.iter_mut().find(|item| item.id == host.id) {
        // 保留已有转发列表与置顶状态，仅更新连接参数与修改时间。
        host.forwards = existing.forwards.clone();
        host.pinned = existing.pinned;
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
pub fn set_host_pinned(
    id: String,
    pinned: bool,
    state: State<AppState>,
    app: AppHandle,
) -> Result<HostView, String> {
    let mut hosts = state.hosts.lock().map_err(lock_error)?;
    let host = hosts
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| "Host not found.".to_string())?;
    host.pinned = pinned;
    let updated = host.clone();
    write_json(state.hosts_path(), &*hosts)?;
    drop(hosts);
    state.add_log(
        "info",
        format!("[{}] host {}", updated.name, if pinned { "pinned" } else { "unpinned" }),
        Some(&app),
    );
    Ok(state.host_view(updated))
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
