use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::vscode;

/// Check whether VS Code and the Remote-SSH extension are installed.
#[tauri::command]
pub fn vscode_status() -> vscode::VscodeStatus {
    vscode::status()
}

/// Open a history remote folder in VS Code (reopens its URI as-is). Bumps the entry to the top of the
/// local history and rescans VS Code's own history for anything new.
#[tauri::command]
pub fn vscode_open(
    uri: String,
    host_id: String,
    label: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let host = state.find_host(&host_id)?;
    vscode::open_folder_uri(&uri)?;
    state.record_open(&host.id, "vscode", &label, &uri, "");
    state.merge_vscode_history(&host.id, &host.ssh_host);
    state.add_log("info", format!("[{}] vscode open {uri}", host.name), Some(&app));
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
    // A direct connect has no folder URI to record, but launching VS Code is a good moment to rescan history.
    state.merge_vscode_history(&host.id, &host.ssh_host);
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
    // Record the opened folder (when a concrete URI was produced) and rescan VS Code's own history.
    if !result.uri.is_empty() {
        state.record_open(&host.id, "vscode", path.trim(), &result.uri, "");
    }
    state.merge_vscode_history(&host.id, &host.ssh_host);
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
