use std::process::{Command, Output, Stdio};

use tauri::{AppHandle, State};

use crate::ssh::command::build_send_command;
use crate::ssh::process::{apply_askpass, prepare_askpass_helper};
use crate::state::AppState;
use crate::terminal::open_terminal as open_terminal_window;
use crate::util::no_window;

/// 合并 ssh 输出的 stdout 与 stderr，供弹窗展示。
fn merge_output(output: &Output) -> String {
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
    combined
}

/// 通过 SSH 在主机上执行一条指令，返回输出（依赖已配置的免密登录）。
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
    let argv = build_send_command(&host, command.trim(), false)?;
    state.add_log("info", format!("[{}] exec $ {}", host.name, command.trim()), Some(&app));

    let mut process = Command::new(&argv[0]);
    process
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    no_window(&mut process);
    let output = process.output().map_err(|err| err.to_string())?;
    Ok(merge_output(&output))
}

/// 通过 SSH 在主机上执行一条指令，使用一次性密码（askpass 注入）。
#[tauri::command]
pub fn send_command_with_password(
    host_id: String,
    command: String,
    password: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<String, String> {
    if command.trim().is_empty() {
        return Err("Command is required.".to_string());
    }
    if password.is_empty() {
        return Err("Password is required.".to_string());
    }
    let host = state.find_host(&host_id)?;
    let argv = build_send_command(&host, command.trim(), true)?;
    state.add_log("info", format!("[{}] exec(pw) $ {}", host.name, command.trim()), Some(&app));

    let helper = prepare_askpass_helper(&state.data_dir)?;
    let mut process = Command::new(&argv[0]);
    process
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_askpass(&mut process, helper, &password);
    no_window(&mut process);
    let output = process.output().map_err(|err| err.to_string())?;
    Ok(merge_output(&output))
}

/// 打开外部 PowerShell 终端窗口连接到该主机。
#[tauri::command]
pub fn open_terminal(host_id: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let host = state.find_host(&host_id)?;
    open_terminal_window(&host)?;
    state.add_log("info", format!("[{}] opened terminal", host.name), Some(&app));
    Ok(())
}

/// 用系统默认浏览器打开一个 http/https 链接（端口转发的“网页打开”用）。
#[tauri::command]
pub fn open_url(url: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Only http/https URLs are allowed.".to_string());
    }
    // rundll32 的 FileProtocolHandler 会按系统默认关联打开浏览器，不弹控制台窗口。
    let mut command = Command::new("rundll32");
    command.args(["url.dll,FileProtocolHandler", url]);
    no_window(&mut command);
    command.spawn().map_err(|err| err.to_string())?;
    state.add_log("info", format!("open url {url}"), Some(&app));
    Ok(())
}
