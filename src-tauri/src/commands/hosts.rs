use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::model::{Host, HostView, TunnelStatus};
use crate::ssh::process::{restart_forward, stop_tunnel};
use crate::state::AppState;
use crate::store::write_json;
use crate::util::{lock_error, now_millis};

#[tauri::command]
pub fn list_hosts(state: State<AppState>) -> Result<Vec<HostView>, String> {
    let mut hosts = state.hosts.lock().map_err(lock_error)?.clone();
    // Pinned first, then sorted by last-modified time newest-to-oldest; stable for equal order.
    hosts.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.updated_at.cmp(&a.updated_at)));
    Ok(hosts.into_iter().map(|host| state.host_view(host)).collect())
}

#[tauri::command]
pub fn save_host(mut host: Host, state: State<AppState>, app: AppHandle) -> Result<HostView, String> {
    // No parameter validation when creating/editing a host; usability is deferred to connect-time (probe/upload/forward).
    if host.id.trim().is_empty() {
        host.id = Uuid::new_v4().to_string();
    }
    host.updated_at = now_millis();

    // Record the old connection-relevant fields, to determine whether running forwards need a restart.
    let mut connection_changed = false;
    let mut hosts = state.hosts.lock().map_err(lock_error)?;
    if let Some(existing) = hosts.iter_mut().find(|item| item.id == host.id) {
        // Any field that changes the ssh command a running forward is using must trigger a restart.
        connection_changed = existing.ssh_host.trim() != host.ssh_host.trim()
            || existing.ssh_user.trim() != host.ssh_user.trim()
            || existing.ssh_port.trim() != host.ssh_port.trim()
            || existing.identity_file.trim() != host.identity_file.trim()
            || existing.extra_options.trim() != host.extra_options.trim()
            || existing.proxy_jump.trim() != host.proxy_jump.trim();
        // Keep the existing forward list and pinned state; only update connection parameters and modified time.
        host.forwards = existing.forwards.clone();
        host.pinned = existing.pinned;
        *existing = host.clone();
    } else {
        hosts.push(host.clone());
    }
    write_json(state.hosts_path(), &*hosts)?;
    drop(hosts);
    state.add_log("info", format!("[{}] host saved", host.name), Some(&app));

    // Host IP or user changed → restart all running forwards under this host with the new parameters.
    if connection_changed {
        for forward in &host.forwards {
            if matches!(state.status_for(&forward.id), TunnelStatus::Running) {
                if let Err(err) = restart_forward(host.clone(), forward.clone(), state.inner(), &app) {
                    state.add_log(
                        "warning",
                        format!("[{}/{}] restart after edit failed: {}", host.name, forward.name, err),
                        Some(&app),
                    );
                }
            }
        }
    }
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
    // First disconnect all running forwards under this host.
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
