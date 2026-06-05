use tauri::{AppHandle, State};

use crate::model::HostView;
use crate::ssh::keys::{ensure_public_key, upload_key_to_remote};
use crate::ssh::probe::{
    get_host_fingerprint as fingerprint, probe_connection as probe, remove_known_host as remove_host,
};
use crate::state::AppState;
use crate::validate::validate_host_connection;

#[tauri::command]
pub fn upload_public_key(
    host_id: String,
    password: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<HostView, String> {
    if password.is_empty() {
        return Err("Password is required.".to_string());
    }
    let host = state.find_host(&host_id)?;
    validate_host_connection(&host)?;
    let public_key = ensure_public_key(&host, &app)?;
    upload_key_to_remote(&host, &public_key, &password, state.inner(), &app)?;
    state.add_log("info", format!("[{}] public key uploaded", host.name), Some(&app));
    Ok(state.host_view(host))
}

#[tauri::command]
pub fn probe_connection(host_id: String, state: State<AppState>, app: AppHandle) -> Result<String, String> {
    let host = state.find_host(&host_id)?;
    validate_host_connection(&host)?;
    probe(&host, state.inner(), &app)
}

#[tauri::command]
pub fn get_host_fingerprint(host_id: String, state: State<AppState>, _app: AppHandle) -> Result<String, String> {
    let host = state.find_host(&host_id)?;
    validate_host_connection(&host)?;
    fingerprint(&host)
}

#[tauri::command]
pub fn remove_known_host(host_id: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let host = state.find_host(&host_id)?;
    remove_host(&host, state.inner(), &app)
}
