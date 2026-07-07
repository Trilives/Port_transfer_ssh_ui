use tauri::{AppHandle, State};

use crate::state::AppState;

/// Check whether an OpenSSH client (`ssh`) is available on the system.
#[tauri::command]
pub fn check_ssh() -> bool {
    use std::process::{Command, Stdio};

    use crate::util::no_window;

    let mut command = Command::new("ssh");
    command
        .arg("-V")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    no_window(&mut command);
    // output() returns Err when the executable can't be found; being able to launch it counts as installed.
    command.output().is_ok()
}

/// Install the OpenSSH client.
///
/// On Windows this runs `Add-WindowsCapability` in an elevated PowerShell (downloads from Windows Update;
/// the user confirms the UAC prompt). On other platforms OpenSSH ships by default or is installed via the
/// system package manager, so this returns guidance instead.
#[tauri::command]
pub fn install_openssh(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::{Command, Stdio};

        use crate::util::no_window;

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
    #[cfg(not(target_os = "windows"))]
    {
        let _ = &app;
        let message = if state.is_zh() {
            "自动安装 OpenSSH 仅支持 Windows。请用系统包管理器安装 OpenSSH 客户端（Debian/Ubuntu：sudo apt install openssh-client；Fedora：sudo dnf install openssh-clients；macOS 已内置）。"
        } else {
            "Automatic OpenSSH installation is only supported on Windows. Install the OpenSSH client with your package manager (Debian/Ubuntu: sudo apt install openssh-client; Fedora: sudo dnf install openssh-clients; macOS ships it by default)."
        };
        Err(message.to_string())
    }
}
