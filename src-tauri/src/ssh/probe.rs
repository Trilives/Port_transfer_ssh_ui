use std::{
    io::Write,
    process::{Command, Stdio},
};

use tauri::AppHandle;

use crate::model::Host;
use crate::ssh::command::build_probe_command;
use crate::ssh::diagnose::{classify_ssh_failure, format_failure, SshFailureKind};
use crate::state::AppState;
use crate::util::{empty_default, no_window};

/// 探测连接，返回 "ready" / "password_required" / "host_key_changed"，或不可达错误。
pub fn probe_connection(host: &Host, state: &AppState, app: &AppHandle) -> Result<String, String> {
    let command = build_probe_command(host)?;
    state.add_log("debug", format!("[{}] probe $ {}", host.name, command.join(" ")), Some(app));
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    no_window(&mut process);
    let output = process.output().map_err(|err| err.to_string())?;
    if output.status.success() {
        return Ok("ready".to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    match classify_ssh_failure(&stderr) {
        SshFailureKind::HostKeyChanged => Ok("host_key_changed".to_string()),
        // 已连上主机但免密不可用 / 认证失败 → 弹密码框。
        SshFailureKind::AuthRequired => Ok("password_required".to_string()),
        // 主机不可达 / IP / 端口 / 网络等问题 → 直接报错并说明原因，不要当成需要密码。
        other => Err(format_failure(other.reason(state.is_zh()), &stderr)),
    }
}

pub fn get_host_fingerprint(host: &Host) -> Result<String, String> {
    let port = empty_default(&host.ssh_port, "22");
    let target = host.ssh_host.trim();

    let mut scan = Command::new("ssh-keyscan");
    scan.args(["-p", port, target])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    no_window(&mut scan);
    let scan_output = scan.output().map_err(|err| err.to_string())?;
    if scan_output.stdout.is_empty() {
        return Err("Cannot fetch host key.".to_string());
    }

    let mut keygen = Command::new("ssh-keygen");
    keygen
        .args(["-l", "-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    no_window(&mut keygen);
    let mut child = keygen.spawn().map_err(|err| err.to_string())?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(&scan_output.stdout).map_err(|err| err.to_string())?;
    }
    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    let fingerprint = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fingerprint.is_empty() {
        return Err("Cannot compute host key fingerprint.".to_string());
    }
    Ok(fingerprint)
}

pub fn remove_known_host(host: &Host, state: &AppState, app: &AppHandle) -> Result<(), String> {
    let target = host.ssh_host.trim().to_string();
    if target.is_empty() {
        return Err("SSH host is required.".to_string());
    }
    let port = empty_default(&host.ssh_port, "22").to_string();
    let mut targets = vec![target.clone()];
    if port != "22" {
        targets.push(format!("[{target}]:{port}"));
    }
    for entry in targets {
        let mut keygen = Command::new("ssh-keygen");
        keygen
            .args(["-R", &entry])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        no_window(&mut keygen);
        match keygen.output() {
            Ok(_) => state.add_log(
                "info",
                format!("[{}] removed old host key for {}", host.name, entry),
                Some(app),
            ),
            Err(err) => state.add_log(
                "warning",
                format!("[{}] ssh-keygen -R {} failed: {}", host.name, entry, err),
                Some(app),
            ),
        }
    }
    Ok(())
}
