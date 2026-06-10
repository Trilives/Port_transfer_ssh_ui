use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use tauri::{AppHandle, Emitter, Manager};

use crate::model::{CriticalErrorPayload, Forward, Host};
use crate::ssh::command::build_ssh_command;
use crate::ssh::diagnose::{classify_ssh_failure, SshFailureKind};
use crate::state::{AppState, ManagedTunnel};
use crate::util::{lock_error, no_window};

/// 启动一条转发的 ssh 子进程并登记进运行态。
pub fn start_tunnel(
    host: Host,
    forward: Forward,
    state: &AppState,
    app: AppHandle,
    password: Option<String>,
) -> Result<(), String> {
    let mut tunnels = state.tunnels.lock().map_err(lock_error)?;
    if let Some(existing) = tunnels.get_mut(&forward.id) {
        if let Some(child) = existing.child.as_mut() {
            if child.try_wait().map_err(|err| err.to_string())?.is_none() {
                return Ok(());
            }
        }
    }

    let command = build_ssh_command(&host, &forward)?;
    state.add_log(
        "debug",
        format!("[{}/{}] $ {}", host.name, forward.name, command.join(" ")),
        Some(&app),
    );
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    no_window(&mut process);
    if let Some(password_value) = password.as_ref() {
        let helper = prepare_askpass_helper(&state.data_dir)?;
        process
            .env("SSH_ASKPASS", helper)
            .env("SSH_ASKPASS_REQUIRE", "force")
            .env("DISPLAY", "ssh-port-forwarder")
            .env("SSH_PORT_FORWARDER_PASSWORD", password_value);
    }
    let child = process.spawn().map_err(|err| err.to_string())?;

    let forward_id = forward.id.clone();
    tunnels.insert(
        forward_id.clone(),
        ManagedTunnel {
            host: host.clone(),
            forward: forward.clone(),
            child: Some(child),
            stop_requested: false,
            password,
        },
    );
    drop(tunnels);
    state.add_log("info", format!("[{}/{}] connected", host.name, forward.name), Some(&app));
    watch_tunnel(forward_id, state.data_dir.clone(), app);
    Ok(())
}

/// 用新的 host/forward 参数重启一条运行中的转发：沿用其暂存密码（若有），先停后起。
pub fn restart_forward(
    host: Host,
    forward: Forward,
    state: &AppState,
    app: &AppHandle,
) -> Result<(), String> {
    let password = {
        let tunnels = state.tunnels.lock().map_err(lock_error)?;
        tunnels.get(&forward.id).and_then(|tunnel| tunnel.password.clone())
    };
    stop_tunnel(&forward.id, state, Some(app))?;
    start_tunnel(host, forward, state, app.clone(), password)
}

pub fn stop_tunnel(forward_id: &str, state: &AppState, app: Option<&AppHandle>) -> Result<(), String> {
    let mut tunnels = state.tunnels.lock().map_err(lock_error)?;
    if let Some(tunnel) = tunnels.get_mut(forward_id) {
        tunnel.stop_requested = true;
        if let Some(child) = tunnel.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        state.add_log(
            "info",
            format!("[{}/{}] disconnected", tunnel.host.name, tunnel.forward.name),
            app,
        );
        tunnels.remove(forward_id);
    }
    Ok(())
}

pub fn cleanup_tunnels(state: &AppState, app: Option<&AppHandle>) {
    let tunnels = {
        let Ok(mut tunnels) = state.tunnels.lock() else {
            return;
        };
        tunnels.drain().map(|(_, tunnel)| tunnel).collect::<Vec<_>>()
    };

    for mut tunnel in tunnels {
        tunnel.stop_requested = true;
        if let Some(child) = tunnel.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        state.add_log(
            "info",
            format!("[{}/{}] cleaned up on exit", tunnel.host.name, tunnel.forward.name),
            app,
        );
    }
}

fn watch_tunnel(forward_id: String, data_dir: PathBuf, app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));
        let state = app.state::<AppState>();
        let exit_info = {
            let mut tunnels = match state.tunnels.lock() {
                Ok(tunnels) => tunnels,
                Err(_) => return,
            };
            let Some(tunnel) = tunnels.get_mut(&forward_id) else {
                return;
            };
            let Some(child) = tunnel.child.as_mut() else {
                return;
            };
            match child.try_wait() {
                Ok(None) => continue,
                Ok(Some(status)) => {
                    let host = tunnel.host.clone();
                    let forward = tunnel.forward.clone();
                    let stopped_by_user = tunnel.stop_requested;
                    let mut stderr = String::new();
                    if let Some(mut stderr_pipe) = child.stderr.take() {
                        let _ = stderr_pipe.read_to_string(&mut stderr);
                    }
                    tunnel.child = None;
                    let password = tunnel.password.clone();
                    (host, forward, password, stopped_by_user, status.code(), stderr)
                }
                Err(_) => return,
            }
        };

        let (host, forward, password, stopped_by_user, code, stderr) = exit_info;
        let detail = stderr.trim();
        let label = format!("{}/{}", host.name, forward.name);
        let message = if detail.is_empty() {
            format!("[{}] exited with code {:?}", label, code)
        } else {
            format!("[{}] exited with code {:?}: {}", label, code, detail)
        };
        if code == Some(255) {
            // 给关键错误标注具体原因（不可达 / 认证失败等），而不是一律按密码处理。
            let kind = classify_ssh_failure(&stderr);
            let critical_message = if kind == SshFailureKind::Unknown {
                message.clone()
            } else {
                format!("{}\n\n{}", kind.reason(state.is_zh()), message)
            };
            state.add_log("error", critical_message.clone(), Some(&app));
            let _ = app.emit(
                "critical-error",
                CriticalErrorPayload {
                    host_id: host.id.clone(),
                    forward_id: forward.id.clone(),
                    name: label,
                    message: critical_message,
                },
            );
            // 致命错误：移除运行态，停止重连。
            if let Ok(mut tunnels) = state.tunnels.lock() {
                tunnels.remove(&forward.id);
            }
            return;
        }
        state.add_log("warning", message, Some(&app));
        if stopped_by_user || !forward.keep_connected {
            if let Ok(mut tunnels) = state.tunnels.lock() {
                tunnels.remove(&forward.id);
            }
            return;
        }
        thread::sleep(Duration::from_secs(3));
        let _ = fs::create_dir_all(&data_dir);
        if start_tunnel(host, forward, state.inner(), app.clone(), password).is_err() {
            return;
        }
        return;
    });
}

pub fn prepare_askpass_helper(data_dir: &PathBuf) -> Result<PathBuf, String> {
    let helper_dir = data_dir.join("helpers");
    fs::create_dir_all(&helper_dir).map_err(|err| err.to_string())?;
    let helper = helper_dir.join("ssh-port-forwarder-askpass.exe");
    let current_exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let should_copy = match (fs::metadata(&current_exe), fs::metadata(&helper)) {
        (Ok(current), Ok(existing)) => current.len() != existing.len(),
        (Ok(_), Err(_)) => true,
        _ => true,
    };
    if should_copy {
        fs::copy(current_exe, &helper).map_err(|err| err.to_string())?;
    }
    Ok(helper)
}

/// 上传公钥时用到的密码注入辅助：在已有进程上设置 askpass 环境。
pub fn apply_askpass(process: &mut Command, helper: PathBuf, password: &str) {
    process
        .env("SSH_ASKPASS", helper)
        .env("SSH_ASKPASS_REQUIRE", "force")
        .env("DISPLAY", "ssh-port-forwarder")
        .env("SSH_PORT_FORWARDER_PASSWORD", password);
}

/// 把多个写操作合并：写入 stdin。
pub fn write_stdin(child: &mut std::process::Child, data: &[u8]) -> Result<(), String> {
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(data).map_err(|err| err.to_string())?;
    }
    Ok(())
}
