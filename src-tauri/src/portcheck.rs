use std::collections::HashSet;
use std::process::{Command, Stdio};

use crate::state::AppState;
use crate::util::no_window;

pub enum PortConflict {
    /// 端口被本应用另一条运行中的转发占用，携带该转发名称。
    Internal { forward_name: String },
    /// 端口被本机其他进程占用，携带进程名与 PID。
    External { process: String, pid: String },
}

impl PortConflict {
    pub fn message(&self, zh: bool, port: &str) -> String {
        match self {
            PortConflict::Internal { forward_name } => {
                if zh {
                    format!("端口 {port} 已被本应用的转发「{forward_name}」占用。")
                } else {
                    format!("Port {port} is already used by this app's forward \"{forward_name}\".")
                }
            }
            PortConflict::External { process, pid } => {
                if zh {
                    format!("端口 {port} 已被进程 {process} (PID {pid}) 占用。")
                } else {
                    format!("Port {port} is in use by {process} (PID {pid}).")
                }
            }
        }
    }
}

/// 从 `start_port` 起递增，返回第一个不与本应用其他转发或系统进程冲突的端口（最多尝试 200 个）。
/// `start_port` 非法（非数字）时返回 None，交由上层按原值处理。
pub fn find_free_port(state: &AppState, start_port: &str, exclude_forward_id: &str) -> Option<String> {
    let mut port: u16 = start_port.trim().parse().ok()?;
    for _ in 0..200 {
        let candidate = port.to_string();
        if detect_conflict(state, &candidate, exclude_forward_id).is_none() {
            return Some(candidate);
        }
        port = port.checked_add(1)?;
    }
    None
}

/// 检测某监听端口是否冲突。`exclude_forward_id` 是当前正在操作的转发，自身不算冲突。
pub fn detect_conflict(
    state: &AppState,
    port: &str,
    exclude_forward_id: &str,
) -> Option<PortConflict> {
    let port = port.trim();
    if port.is_empty() {
        return None;
    }

    // 1) 收集本应用所有运行中转发的 ssh 进程 PID，同时检查应用内端口冲突。
    //    注意：这里持有 tunnels 锁，直接检查子进程状态，不要再调用会二次加锁的 status_for。
    let mut own_pids: HashSet<u32> = HashSet::new();
    if let Ok(mut tunnels) = state.tunnels.lock() {
        for (id, tunnel) in tunnels.iter_mut() {
            // 仅统计仍在运行的子进程（断开/退出的转发已从 map 移除，但这里再保险一次）。
            let pid = match tunnel.child.as_mut() {
                Some(child) => {
                    if child.try_wait().ok().flatten().is_none() {
                        child.id()
                    } else {
                        continue;
                    }
                }
                None => continue,
            };
            own_pids.insert(pid);
            if id != exclude_forward_id
                && tunnel.forward.binds_local_port()
                && tunnel.forward.bind_port.trim() == port
            {
                return Some(PortConflict::Internal {
                    forward_name: tunnel.forward.name.clone(),
                });
            }
        }
    }

    // 2) 操作系统层：跳过本应用自己的 ssh 进程，只有其他进程才算占用。
    if let Some((pid, process)) = port_listener(port, &own_pids) {
        return Some(PortConflict::External { process, pid });
    }

    None
}

/// 返回监听指定端口的 (pid, process_name)，忽略本应用自己的 ssh 进程。
fn port_listener(port: &str, own_pids: &HashSet<u32>) -> Option<(String, String)> {
    let mut netstat = Command::new("netstat");
    netstat
        .args(["-ano", "-p", "tcp"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    no_window(&mut netstat);
    let output = netstat.output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);

    let needle = format!(":{port}");
    for line in text.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        // 形如: TCP  0.0.0.0:8000  0.0.0.0:0  LISTENING  1234
        if cols.len() < 5 {
            continue;
        }
        if !cols[3].eq_ignore_ascii_case("LISTENING") {
            continue;
        }
        let local = cols[1];
        // 精确匹配端口：按最后一个冒号拆分。
        if let Some(idx) = local.rfind(':') {
            if &local[idx..] == needle {
                let pid = cols[4].to_string();
                // 本应用自己启动的 ssh 进程不算占用。
                if let Ok(pid_num) = pid.parse::<u32>() {
                    if own_pids.contains(&pid_num) {
                        continue;
                    }
                }
                let process = process_name(&pid).unwrap_or_else(|| "unknown".to_string());
                return Some((pid, process));
            }
        }
    }
    None
}

fn process_name(pid: &str) -> Option<String> {
    let mut tasklist = Command::new("tasklist");
    tasklist
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    no_window(&mut tasklist);
    let output = tasklist.output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let first = text.lines().next()?.trim();
    // 形如: "chrome.exe","1234","Console","1","123,456 K"
    let name = first.split(',').next()?.trim().trim_matches('"');
    if name.is_empty() || name.eq_ignore_ascii_case("INFO:") {
        None
    } else {
        Some(name.to_string())
    }
}
