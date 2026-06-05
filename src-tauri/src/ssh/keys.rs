use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use tauri::{AppHandle, Manager};

use crate::model::Host;
use crate::ssh::command::build_key_upload_command;
use crate::ssh::process::{apply_askpass, prepare_askpass_helper, write_stdin};
use crate::state::AppState;
use crate::util::no_window;

/// 确保本机存在私钥与对应公钥，返回公钥内容。私钥不存在时自动生成 ed25519。
pub fn ensure_public_key(host: &Host, app: &AppHandle) -> Result<String, String> {
    let private_key = resolve_identity_file(&host.identity_file)?;
    let public_key = PathBuf::from(format!("{}.pub", private_key.to_string_lossy()));
    if !private_key.exists() {
        if let Some(parent) = private_key.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let mut keygen = Command::new("ssh-keygen");
        keygen
            .args([
                "-t",
                "ed25519",
                "-N",
                "",
                "-C",
                "ssh-port-forwarder",
                "-f",
                private_key
                    .to_str()
                    .ok_or_else(|| "Identity file path contains invalid characters.".to_string())?,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        no_window(&mut keygen);
        let output = keygen.output().map_err(|err| err.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "ssh-keygen failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let state = app.state::<AppState>();
        state.add_log("info", format!("[key] generated {}", private_key.display()), Some(app));
    }

    if !public_key.exists() {
        let mut derive = Command::new("ssh-keygen");
        derive
            .args([
                "-y",
                "-f",
                private_key
                    .to_str()
                    .ok_or_else(|| "Identity file path contains invalid characters.".to_string())?,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        no_window(&mut derive);
        let output = derive.output().map_err(|err| err.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "Cannot derive public key: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        fs::write(&public_key, output.stdout).map_err(|err| err.to_string())?;
    }

    fs::read_to_string(&public_key)
        .map(|value| value.trim().to_string())
        .map_err(|err| err.to_string())
}

pub fn resolve_identity_file(identity_file: &str) -> Result<PathBuf, String> {
    let trimmed = identity_file.trim();
    if trimmed.is_empty() {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map_err(|_| "Cannot locate user home directory.".to_string())?;
        return Ok(PathBuf::from(home).join(".ssh").join("id_ed25519"));
    }

    if let Some(rest) = trimmed.strip_prefix("~/").or_else(|| trimmed.strip_prefix("~\\")) {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map_err(|_| "Cannot locate user home directory.".to_string())?;
        return Ok(PathBuf::from(home).join(rest));
    }

    Ok(PathBuf::from(trimmed))
}

pub fn upload_key_to_remote(
    host: &Host,
    public_key: &str,
    password: &str,
    state: &AppState,
    app: &AppHandle,
) -> Result<(), String> {
    let command = build_key_upload_command(host)?;
    state.add_log("debug", format!("[{}] $ {}", host.name, command.join(" ")), Some(app));
    let helper = prepare_askpass_helper(&state.data_dir)?;
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_askpass(&mut process, helper, password);
    no_window(&mut process);
    let mut child = process.spawn().map_err(|err| err.to_string())?;
    write_stdin(&mut child, format!("{public_key}\n").as_bytes())?;
    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Upload public key failed: {}", stderr.trim()));
    }
    Ok(())
}
