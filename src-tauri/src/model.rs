use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::empty_default;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelMode {
    Local,
    Remote,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Running,
    Stopped,
}

/// 二级目录：挂在某个主机下的一条端口转发，只保存转发参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Forward {
    pub id: String,
    pub name: String,
    pub mode: TunnelMode,
    pub bind_host: String,
    pub bind_port: String,
    pub target_host: String,
    pub target_port: String,
    pub keep_connected: bool,
}

impl Default for Forward {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "new forward".to_string(),
            mode: TunnelMode::Local,
            bind_host: "127.0.0.1".to_string(),
            bind_port: String::new(),
            target_host: "127.0.0.1".to_string(),
            target_port: String::new(),
            keep_connected: true,
        }
    }
}

impl Forward {
    pub fn bind_display(&self) -> String {
        format!("{}:{}", empty_default(&self.bind_host, "127.0.0.1"), self.bind_port.trim())
    }

    pub fn target_display(&self) -> String {
        if self.mode == TunnelMode::Dynamic {
            "SOCKS proxy".to_string()
        } else {
            format!("{}:{}", empty_default(&self.target_host, "127.0.0.1"), self.target_port.trim())
        }
    }

    /// 仅 local / dynamic 在本机监听端口；remote 监听在远端。
    pub fn binds_local_port(&self) -> bool {
        matches!(self.mode, TunnelMode::Local | TunnelMode::Dynamic)
    }
}

/// 一级目录：一台 SSH 服务器，保存连接参数与其下的转发列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Host {
    pub id: String,
    pub name: String,
    pub ssh_host: String,
    pub ssh_port: String,
    pub ssh_user: String,
    pub identity_file: String,
    pub extra_options: String,
    /// 跳板机（ProxyJump）；可空。形如 `user@jump-host:port`，多级用逗号分隔。
    #[serde(default)]
    pub proxy_jump: String,
    #[serde(default)]
    pub forwards: Vec<Forward>,
    /// 是否置顶；置顶的主机排在列表最前。
    #[serde(default)]
    pub pinned: bool,
    /// 最后修改时间（Unix 毫秒）；列表按其从新到旧排序。
    #[serde(default)]
    pub updated_at: i64,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "new host".to_string(),
            ssh_host: String::new(),
            ssh_port: "22".to_string(),
            ssh_user: String::new(),
            identity_file: String::new(),
            extra_options: String::new(),
            proxy_jump: String::new(),
            forwards: Vec::new(),
            pinned: false,
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardView {
    #[serde(flatten)]
    pub forward: Forward,
    pub status: TunnelStatus,
    pub bind_display: String,
    pub target_display: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostView {
    pub id: String,
    pub name: String,
    pub ssh_host: String,
    pub ssh_port: String,
    pub ssh_user: String,
    pub identity_file: String,
    pub extra_options: String,
    pub proxy_jump: String,
    pub forwards: Vec<ForwardView>,
    pub pinned: bool,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub log_level: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            language: "zh-CN".to_string(),
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CriticalErrorPayload {
    pub host_id: String,
    pub forward_id: String,
    pub name: String,
    pub message: String,
}
