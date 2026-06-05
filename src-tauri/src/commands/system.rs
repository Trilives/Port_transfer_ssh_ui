use std::process::{Command, Stdio};

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::util::no_window;

/// 检测系统是否有可用的 OpenSSH 客户端（`ssh.exe`）。
#[tauri::command]
pub fn check_ssh() -> bool {
    let mut command = Command::new("ssh");
    command
        .arg("-V")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    no_window(&mut command);
    // 找不到可执行文件时 output() 返回 Err；能启动即视为已安装。
    command.output().is_ok()
}

/// 触发安装 Windows 自带的 OpenSSH 客户端。
///
/// 通过提权的 PowerShell 运行 `Add-WindowsCapability`，会从 Windows Update 下载并安装。
/// 安装在独立的提权窗口中进行，用户需在 UAC 提示中确认。
#[tauri::command]
pub fn install_openssh(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let inner = "Start-Process powershell -Verb RunAs -ArgumentList '-NoProfile','-NoExit','-Command','Write-Host \"Installing OpenSSH client...\"; Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0'";
    let mut command = Command::new("powershell");
    command
        .args(["-NoProfile", "-Command", inner])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    no_window(&mut command);
    command.spawn().map_err(|err| err.to_string())?;
    state.add_log("info", "OpenSSH client install requested", Some(&app));
    Ok(())
}
