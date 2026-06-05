use std::process::{Command, Stdio};

use tauri::{AppHandle, State};

use crate::ssh::command::build_send_command;
use crate::state::AppState;
use crate::terminal::open_terminal as open_terminal_window;
use crate::util::no_window;

/// 通过 SSH 在主机上执行一条指令，返回合并后的输出（依赖已配置的免密登录）。
#[tauri::command]
pub fn send_command(
    host_id: String,
    command: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<String, String> {
    if command.trim().is_empty() {
        return Err("Command is required.".to_string());
    }
    let host = state.find_host(&host_id)?;
    let argv = build_send_command(&host, command.trim())?;
    state.add_log("info", format!("[{}] exec $ {}", host.name, command.trim()), Some(&app));

    let mut process = Command::new(&argv[0]);
    process
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    no_window(&mut process);
    let output = process.output().map_err(|err| err.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut combined = String::new();
    if !stdout.trim().is_empty() {
        combined.push_str(stdout.trim_end());
    }
    if !stderr.trim().is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(stderr.trim_end());
    }
    if !output.status.success() && combined.trim().is_empty() {
        combined = format!("exit code {:?}", output.status.code());
    }
    Ok(combined)
}

/// 打开外部 PowerShell 终端窗口连接到该主机。
#[tauri::command]
pub fn open_terminal(host_id: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let host = state.find_host(&host_id)?;
    open_terminal_window(&host)?;
    state.add_log("info", format!("[{}] opened terminal", host.name), Some(&app));
    Ok(())
}
