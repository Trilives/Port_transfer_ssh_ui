use tauri::State;

use crate::model::HistoryEntry;
use crate::state::AppState;

/// This host's local open-history (ports / VS Code / terminal), most recent first.
/// Opening the history window is also a good moment to pull in any new VS Code entries.
#[tauri::command]
pub fn list_history(host_id: String, state: State<AppState>) -> Result<Vec<HistoryEntry>, String> {
    let host = state.find_host(&host_id)?;
    state.merge_vscode_history(&host.id, &host.ssh_host);
    Ok(state.history_for_host(&host_id))
}
