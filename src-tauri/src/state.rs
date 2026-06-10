use chrono::Local;
use directories::ProjectDirs;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::Child,
    sync::Mutex,
};
use tauri::{AppHandle, Emitter};

use crate::model::{
    AppSettings, Forward, ForwardView, Host, HostView, LogEntry, TunnelStatus,
};
use crate::store::read_json;
use crate::util::lock_error;

/// 运行中的一条转发：记录父主机与转发快照，用于断线自动重连。
pub struct ManagedTunnel {
    pub host: Host,
    pub forward: Forward,
    pub child: Option<Child>,
    pub stop_requested: bool,
    pub password: Option<String>,
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
}

impl AppState {
    pub fn new() -> Self {
        let data_dir = ProjectDirs::from("com", "codex", "ssh-port-forwarder")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".data"));
        let _ = fs::create_dir_all(&data_dir);

        // v0.1.x 的扁平 profiles.json 不再使用：若存在则备份一次，避免误读旧结构。
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
        }
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

    /// 当前界面语言是否为简体中文（用于本地化后端错误提示）；锁失败时默认中文。
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

    pub fn forward_view(&self, forward: Forward) -> ForwardView {
        ForwardView {
            status: self.status_for(&forward.id),
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

    /// 取出主机及其下某条转发的克隆（连接/操作时需要父主机的连接参数）。
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
