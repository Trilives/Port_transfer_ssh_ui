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

    // 1) 应用内：是否有另一条运行中的转发监听同一端口。
    //    注意：这里持有 tunnels 锁，直接检查子进程状态，不要再调用会二次加锁的 status_for。
    if let Ok(mut tunnels) = state.tunnels.lock() {
        for (id, tunnel) in tunnels.iter_mut() {
            if id == exclude_forward_id {
                continue;
            }
            if !tunnel.forward.binds_local_port() || tunnel.forward.bind_port.trim() != port {
                continue;
            }
            let running = tunnel
                .child
                .as_mut()
                .map(|child| child.try_wait().ok().flatten().is_none())
                .unwrap_or(false);
            if running {
                return Some(PortConflict::Internal {
                    forward_name: tunnel.forward.name.clone(),
                });
            }
        }
    }

    // 2) 操作系统：是否有其他进程在监听该端口。
    if let Some((pid, process)) = port_listener(port) {
        return Some(PortConflict::External { process, pid });
    }

    None
}

/// 返回监听指定端口的 (pid, process_name)。
fn port_listener(port: &str) -> Option<(String, String)> {
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
