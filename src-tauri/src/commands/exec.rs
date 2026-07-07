use std::process::{Command, Output, Stdio};

use tauri::{AppHandle, State};

use crate::ssh::command::build_send_command;
use crate::ssh::process::{apply_askpass, prepare_askpass_helper};
use crate::state::AppState;
use crate::terminal::open_terminal as open_terminal_window;
use crate::util::no_window;

/// Merge ssh's stdout and stderr for display in a dialog.
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

/// Run a single command on the host over SSH, returning the output (relies on passwordless login being configured).
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

/// Run a single command on the host over SSH, using a one-time password (injected via askpass).
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

/// Open an external PowerShell terminal window connected to the host.
#[tauri::command]
pub fn open_terminal(host_id: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let host = state.find_host(&host_id)?;
    open_terminal_window(&host)?;
    state.add_log("info", format!("[{}] opened terminal", host.name), Some(&app));
    Ok(())
}

/// Open an http/https link with the system default browser (used by the forward's "Open in browser").
#[tauri::command]
pub fn open_url(url: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Only http/https URLs are allowed.".to_string());
    }
    // rundll32's FileProtocolHandler opens the browser via the system's default association, with no console window.
    let mut command = Command::new("rundll32");
    command.args(["url.dll,FileProtocolHandler", url]);
    no_window(&mut command);
    command.spawn().map_err(|err| err.to_string())?;
    state.add_log("info", format!("open url {url}"), Some(&app));
    Ok(())
}
