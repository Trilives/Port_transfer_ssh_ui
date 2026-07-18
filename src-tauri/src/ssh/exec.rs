//! Blocking execution for one-off SSH commands.
//!
//! Tauri command handlers call this through `spawn_blocking`, so a slow remote command does not occupy
//! the IPC async runtime and freeze the rest of the interface.

use std::path::Path;
use std::process::{Command, Output, Stdio};

use crate::model::Host;
use crate::ssh::command::build_send_command;
use crate::ssh::process::{apply_askpass, prepare_askpass_helper};
use crate::util::no_window;

/// Run a one-off command and merge stdout/stderr for display in the frontend.
pub fn run_command(
    host: &Host,
    remote_command: &str,
    password: Option<&str>,
    data_dir: &Path,
) -> Result<String, String> {
    let argv = build_send_command(host, remote_command, password.is_some())?;
    let mut process = Command::new(&argv[0]);
    process
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(password) = password {
        let helper = prepare_askpass_helper(data_dir)?;
        apply_askpass(&mut process, helper, password);
    }
    no_window(&mut process);
    let output = process.output().map_err(|err| err.to_string())?;
    Ok(merge_output(&output))
}

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
