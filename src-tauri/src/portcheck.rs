use std::collections::HashSet;
use std::process::{Command, Stdio};

use crate::state::AppState;
use crate::util::no_window;

pub enum PortConflict {
    /// Port is used by another running forward in this app; carries that forward's name.
    Internal { forward_name: String },
    /// Port is used by another local process; carries the process name and PID.
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

/// Increment from `start_port`, returning the first port that doesn't conflict with another forward in this app or a system process (up to 200 tries).
/// Returns None if `start_port` is invalid (not a number), leaving the caller to handle the original value.
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

/// Check whether a listening port conflicts. `exclude_forward_id` is the forward currently being operated on; it doesn't count as a conflict with itself.
pub fn detect_conflict(
    state: &AppState,
    port: &str,
    exclude_forward_id: &str,
) -> Option<PortConflict> {
    let port = port.trim();
    if port.is_empty() {
        return None;
    }

    // 1) Collect the ssh process PIDs of all running forwards in this app, while also checking for an in-app port conflict.
    //    Note: this holds the tunnels lock and checks child process state directly; don't call status_for here, which would re-lock.
    let mut own_pids: HashSet<u32> = HashSet::new();
    if let Ok(mut tunnels) = state.tunnels.lock() {
        for (id, tunnel) in tunnels.iter_mut() {
            // Only count child processes that are still running (disconnected/exited forwards are already removed from the map, but double-check here).
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

    // 2) OS level: skip this app's own ssh processes; only other processes count as occupying the port.
    if let Some((pid, process)) = port_listener(port, &own_pids) {
        return Some(PortConflict::External { process, pid });
    }

    None
}

/// Return the (pid, process_name) listening on the given port, ignoring this app's own ssh processes.
fn port_listener(port: &str, own_pids: &HashSet<u32>) -> Option<(String, String)> {
    for (pid, process) in find_listeners(port) {
        // A process started by this app doesn't count as occupying the port.
        if own_pids.contains(&pid) {
            continue;
        }
        let name = if process.trim().is_empty() {
            "unknown".to_string()
        } else {
            process
        };
        return Some((pid.to_string(), name));
    }
    None
}

/// Capture a helper command's stdout, returning None if the program can't be launched.
fn capture(program: &str, args: &[&str]) -> Option<String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    no_window(&mut command);
    let output = command.output().ok()?;
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Windows: `netstat -ano` for the PID, then `tasklist` for the process name.
#[cfg(target_os = "windows")]
fn find_listeners(port: &str) -> Vec<(u32, String)> {
    let Some(text) = capture("netstat", &["-ano", "-p", "tcp"]) else {
        return Vec::new();
    };
    parse_netstat_pids(&text, port)
        .into_iter()
        .map(|pid| {
            let name = process_name(&pid.to_string()).unwrap_or_else(|| "unknown".to_string());
            (pid, name)
        })
        .collect()
}

/// Linux: `ss` reports the listening socket plus its owning process(es); fall back to `lsof` when `ss` is absent.
#[cfg(target_os = "linux")]
fn find_listeners(port: &str) -> Vec<(u32, String)> {
    if let Some(text) = capture("ss", &["-H", "-tlnp"]) {
        return parse_ss_listeners(&text, port);
    }
    let filter = format!("-iTCP:{port}");
    match capture("lsof", &["-nP", filter.as_str(), "-sTCP:LISTEN", "-Fpc"]) {
        Some(text) => parse_lsof(&text),
        None => Vec::new(),
    }
}

/// macOS: `lsof` in field mode lists the pid (`p`) and command (`c`) of each listener.
#[cfg(target_os = "macos")]
fn find_listeners(port: &str) -> Vec<(u32, String)> {
    let filter = format!("-iTCP:{port}");
    match capture("lsof", &["-nP", filter.as_str(), "-sTCP:LISTEN", "-Fpc"]) {
        Some(text) => parse_lsof(&text),
        None => Vec::new(),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn find_listeners(_port: &str) -> Vec<(u32, String)> {
    Vec::new()
}

/// Windows: look up the process name for a PID via `tasklist`.
#[cfg(target_os = "windows")]
fn process_name(pid: &str) -> Option<String> {
    let filter = format!("PID eq {pid}");
    let text = capture("tasklist", &["/FI", filter.as_str(), "/FO", "CSV", "/NH"])?;
    let first = text.lines().next()?.trim();
    // Looks like: "chrome.exe","1234","Console","1","123,456 K"
    let name = first.split(',').next()?.trim().trim_matches('"');
    if name.is_empty() || name.eq_ignore_ascii_case("INFO:") {
        None
    } else {
        Some(name.to_string())
    }
}

/// Parse the PIDs listening on `port` from `netstat -ano -p tcp` output.
/// Line shape: `TCP  0.0.0.0:8000  0.0.0.0:0  LISTENING  1234`.
#[allow(dead_code)]
fn parse_netstat_pids(text: &str, port: &str) -> Vec<u32> {
    let needle = format!(":{port}");
    let mut pids = Vec::new();
    for line in text.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 5 || !cols[3].eq_ignore_ascii_case("LISTENING") {
            continue;
        }
        if let Some(idx) = cols[1].rfind(':') {
            if cols[1][idx..] == needle {
                if let Ok(pid) = cols[4].parse::<u32>() {
                    pids.push(pid);
                }
            }
        }
    }
    pids
}

/// Parse `(pid, process)` pairs listening on `port` from `ss -H -tlnp` output.
/// Line shape: `LISTEN 0 128 0.0.0.0:8000 0.0.0.0:* users:(("python3",pid=4321,fd=3))`.
#[allow(dead_code)]
fn parse_ss_listeners(text: &str, port: &str) -> Vec<(u32, String)> {
    let needle = format!(":{port}");
    let mut result = Vec::new();
    for line in text.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        // Column 3 is the local address (State, Recv-Q, Send-Q, Local, ...).
        let local = cols[3];
        let matches = match local.rfind(':') {
            Some(idx) => local[idx..] == needle,
            None => false,
        };
        if matches {
            result.extend(parse_ss_users(line));
        }
    }
    result
}

/// Extract `(pid, name)` pairs from an `ss` line's `users:(("name",pid=N,fd=M),...)` field.
#[allow(dead_code)]
fn parse_ss_users(line: &str) -> Vec<(u32, String)> {
    let mut result = Vec::new();
    // Each process entry begins with `("` — the name follows, then `pid=<num>`.
    for chunk in line.split("(\"").skip(1) {
        let name = chunk.split('"').next().unwrap_or("").to_string();
        if let Some(after) = chunk.split("pid=").nth(1) {
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(pid) = digits.parse::<u32>() {
                result.push((pid, name));
            }
        }
    }
    result
}

/// Parse `lsof -Fpc` field output: `p<pid>` lines followed by `c<command>` lines.
#[allow(dead_code)]
fn parse_lsof(text: &str) -> Vec<(u32, String)> {
    let mut result = Vec::new();
    let mut current: Option<u32> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix('p') {
            current = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix('c') {
            if let Some(pid) = current {
                result.push((pid, rest.trim().to_string()));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ss_parses_pid_and_name() {
        let text = "LISTEN 0 128 0.0.0.0:8000 0.0.0.0:* users:((\"python3\",pid=4321,fd=3))\n\
                    LISTEN 0 128 [::]:22 [::]:* users:((\"sshd\",pid=1000,fd=3))";
        assert_eq!(parse_ss_listeners(text, "8000"), vec![(4321, "python3".to_string())]);
    }

    #[test]
    fn ss_ignores_non_matching_ports() {
        let text = "LISTEN 0 128 0.0.0.0:8000 0.0.0.0:* users:((\"python3\",pid=4321,fd=3))";
        assert!(parse_ss_listeners(text, "9000").is_empty());
    }

    #[test]
    fn ss_matches_ipv6_and_multiple_processes() {
        let text = "LISTEN 0 128 [::]:80 [::]:* users:((\"nginx\",pid=1,fd=6),(\"nginx\",pid=2,fd=6))";
        assert_eq!(
            parse_ss_listeners(text, "80"),
            vec![(1, "nginx".to_string()), (2, "nginx".to_string())]
        );
    }

    #[test]
    fn lsof_pairs_pid_with_command() {
        let text = "p4321\ncpython3\np5000\ncnode\n";
        assert_eq!(
            parse_lsof(text),
            vec![(4321, "python3".to_string()), (5000, "node".to_string())]
        );
    }

    #[test]
    fn netstat_extracts_listening_pid() {
        let text = "  TCP    0.0.0.0:8000     0.0.0.0:0   LISTENING   1234\n\
                      TCP    0.0.0.0:9000     0.0.0.0:0   LISTENING   5678";
        assert_eq!(parse_netstat_pids(text, "8000"), vec![1234]);
    }
}
