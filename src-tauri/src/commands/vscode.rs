use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::vscode;

/// 检测 VS Code 与 Remote-SSH 扩展是否已安装。
#[tauri::command]
pub fn vscode_status() -> vscode::VscodeStatus {
    vscode::status()
}

/// 读取该主机 IP 在 VS Code Remote-SSH 中的历史远端文件夹。
#[tauri::command]
pub fn vscode_ssh_history(
    host_id: String,
    state: State<AppState>,
) -> Result<Vec<vscode::VscodeHistoryEntry>, String> {
    let host = state.find_host(&host_id)?;
    Ok(vscode::ssh_history_for_ip(&host.ssh_host))
}

/// 用 VS Code 打开一个历史远端文件夹（原样重开其 URI）。
#[tauri::command]
pub fn vscode_open(uri: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    vscode::open_folder_uri(&uri)?;
    state.add_log("info", format!("vscode open {uri}"), Some(&app));
    Ok(())
}

/// 直连/打开远端家目录：必要时把主机写入 ~/.ssh/config，探测 $HOME 并用别名打开它。
#[tauri::command]
pub fn vscode_open_home(
    host_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<vscode::VscodeOpenRootResult, String> {
    let host = state.find_host(&host_id)?;
    let result = vscode::open_home_for_host(&host)?;
    state.add_log(
        "info",
        format!(
            "[{}] vscode open home (alias {}, addedToConfig={})",
            host.name, result.alias, result.added_to_config
        ),
        Some(&app),
    );
    Ok(result)
}
