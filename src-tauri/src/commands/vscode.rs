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

/// 直连：必要时把主机写入 ~/.ssh/config，用 VS Code 默认方式打开不带文件夹的已连接远端窗口。
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

/// 用 VS Code 打开指定的远端目录（绝对路径原样；~/相对路径相对家目录解析）。
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
