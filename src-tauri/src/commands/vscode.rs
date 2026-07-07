use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::vscode;

/// Check whether VS Code and the Remote-SSH extension are installed.
#[tauri::command]
pub fn vscode_status() -> vscode::VscodeStatus {
    vscode::status()
}

/// Read this host's IP's history of remote folders in VS Code Remote-SSH.
#[tauri::command]
pub fn vscode_ssh_history(
    host_id: String,
    state: State<AppState>,
) -> Result<Vec<vscode::VscodeHistoryEntry>, String> {
    let host = state.find_host(&host_id)?;
    Ok(vscode::ssh_history_for_ip(&host.ssh_host))
}

/// Open a history remote folder in VS Code (reopens its URI as-is).
#[tauri::command]
pub fn vscode_open(uri: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    vscode::open_folder_uri(&uri)?;
    state.add_log("info", format!("vscode open {uri}"), Some(&app));
    Ok(())
}

/// Direct connect: writes the host into ~/.ssh/config if needed, then opens a connected remote window with no folder using VS Code's default mode.
#[tauri::command]
pub fn vscode_open_direct(
    host_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<vscode::VscodeOpenRootResult, String> {
    let host = state.find_host(&host_id)?;
    let result = vscode::open_direct_for_host(&host)?;
    state.add_log(
        "info",
        format!(
            "[{}] vscode direct connect (alias {}, addedToConfig={})",
            host.name, result.alias, result.added_to_config
        ),
        Some(&app),
    );
    Ok(result)
}

/// Open the specified remote directory in VS Code (absolute paths as-is; `~`/relative paths resolved against the home directory).
#[tauri::command]
pub fn vscode_open_path(
    host_id: String,
    path: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<vscode::VscodeOpenRootResult, String> {
    let host = state.find_host(&host_id)?;
    let result = vscode::open_path_for_host(&host, &path)?;
    state.add_log(
        "info",
        format!(
            "[{}] vscode open path {} (alias {}, addedToConfig={})",
            host.name,
            path.trim(),
            result.alias,
            result.added_to_config
        ),
        Some(&app),
    );
    Ok(result)
}
