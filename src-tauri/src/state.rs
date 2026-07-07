use chrono::Local;
use directories::ProjectDirs;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::Child,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};
use tauri::{AppHandle, Emitter};

use crate::model::{
    AppSettings, Forward, ForwardView, Host, HostView, LogEntry, TunnelStatus,
};
use crate::store::read_json;
use crate::util::lock_error;

/// A running forward: keeps a snapshot of the parent host and forward, used for auto-reconnect after disconnect.
pub struct ManagedTunnel {
    pub host: Host,
    pub forward: Forward,
    pub child: Option<Child>,
    pub stop_requested: bool,
    pub password: Option<String>,
    /// Unique per (re)start. A reconnect watcher only resurrects an entry whose generation still matches its own,
    /// so a tunnel that was disconnected/deleted (and possibly reconnected) in the meantime can't be revived.
    pub generation: u64,
}

impl Drop for ManagedTunnel {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub struct AppState {
    pub hosts: Mutex<Vec<Host>>,
    pub settings: Mutex<AppSettings>,
    /// key = forward id
    pub tunnels: Mutex<HashMap<String, ManagedTunnel>>,
    pub logs: Mutex<Vec<LogEntry>>,
    pub data_dir: PathBuf,
    /// Monotonic source of `ManagedTunnel::generation` values.
    tunnel_generation: AtomicU64,
}

impl AppState {
    pub fn new() -> Self {
        let data_dir = ProjectDirs::from("com", "codex", "ssh-port-forwarder")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".data"));
        let _ = fs::create_dir_all(&data_dir);

        // The v0.1.x flat profiles.json is no longer used: back it up once if present, to avoid misreading the old structure.
        let legacy = data_dir.join("profiles.json");
        if legacy.exists() {
            let backup = data_dir.join("profiles.json.v0.1.bak");
            if !backup.exists() {
                let _ = fs::rename(&legacy, &backup);
            }
        }

        Self {
            hosts: Mutex::new(read_json(data_dir.join("hosts.json")).unwrap_or_default()),
            settings: Mutex::new(read_json(data_dir.join("settings.json")).unwrap_or_default()),
            tunnels: Mutex::new(HashMap::new()),
            logs: Mutex::new(Vec::new()),
            data_dir,
            tunnel_generation: AtomicU64::new(1),
        }
    }

    /// Allocate a fresh, unique tunnel generation.
    pub fn next_tunnel_generation(&self) -> u64 {
        self.tunnel_generation.fetch_add(1, Ordering::Relaxed)
    }

    pub fn hosts_path(&self) -> PathBuf {
        self.data_dir.join("hosts.json")
    }

    pub fn settings_path(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    pub fn add_log(&self, level: &str, message: impl Into<String>, app: Option<&AppHandle>) {
        let entry = LogEntry {
            level: level.to_string(),
            message: message.into(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        let _ = fs::create_dir_all(self.logs_dir());
        let file_path = self
            .logs_dir()
            .join(format!("{}.log", Local::now().format("%Y-%m-%d")));
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .and_then(|mut file| {
                use std::io::Write;
                writeln!(
                    file,
                    "{} [{}] {}",
                    entry.timestamp,
                    entry.level.to_uppercase(),
                    entry.message
                )
            });

        if let Ok(mut logs) = self.logs.lock() {
            logs.push(entry.clone());
            if logs.len() > 500 {
                let drain_count = logs.len() - 500;
                logs.drain(0..drain_count);
            }
        }
        if let Some(app) = app {
            let _ = app.emit("log-entry", entry);
        }
    }

    /// Whether the current UI language is Simplified Chinese (used to localize backend error messages); defaults to Chinese if the lock fails.
    pub fn is_zh(&self) -> bool {
        self.settings
            .lock()
            .map(|settings| settings.language == "zh-CN")
            .unwrap_or(true)
    }

    pub fn status_for(&self, forward_id: &str) -> TunnelStatus {
        let mut tunnels = self.tunnels.lock().unwrap();
        if let Some(tunnel) = tunnels.get_mut(forward_id) {
            if let Some(child) = tunnel.child.as_mut() {
                if child.try_wait().ok().flatten().is_none() {
                    return TunnelStatus::Running;
                }
            }
        }
        TunnelStatus::Stopped
    }

    pub fn forward_view(&self, mut forward: Forward) -> ForwardView {
        let status = self.status_for(&forward.id);
        // While running: override the displayed value with the actual listening port (may differ from the configured
        // port after auto-fallback), so Home, the forward row, and "Open in browser" all point at the port really being listened on.
        if status == TunnelStatus::Running {
            if let Ok(tunnels) = self.tunnels.lock() {
                if let Some(tunnel) = tunnels.get(&forward.id) {
                    forward.bind_port = tunnel.forward.bind_port.clone();
                }
            }
        }
        ForwardView {
            status,
            bind_display: forward.bind_display(),
            target_display: forward.target_display(),
            forward,
        }
    }

    pub fn host_view(&self, host: Host) -> HostView {
        let forwards = host
            .forwards
            .into_iter()
            .map(|forward| self.forward_view(forward))
            .collect();
        HostView {
            id: host.id,
            name: host.name,
            ssh_host: host.ssh_host,
            ssh_port: host.ssh_port,
            ssh_user: host.ssh_user,
            identity_file: host.identity_file,
            extra_options: host.extra_options,
            proxy_jump: host.proxy_jump,
            forwards,
            pinned: host.pinned,
            updated_at: host.updated_at,
        }
    }

    pub fn find_host(&self, host_id: &str) -> Result<Host, String> {
        self.hosts
            .lock()
            .map_err(lock_error)?
            .iter()
            .find(|item| item.id == host_id)
            .cloned()
            .ok_or_else(|| "Host not found.".to_string())
    }

    /// Get a clone of the host together with one of its forwards (connect/operate needs the parent host's connection parameters).
    pub fn find_forward(&self, host_id: &str, forward_id: &str) -> Result<(Host, Forward), String> {
        let host = self.find_host(host_id)?;
        let forward = host
            .forwards
            .iter()
            .find(|item| item.id == forward_id)
            .cloned()
            .ok_or_else(|| "Forward not found.".to_string())?;
        Ok((host, forward))
    }
}
