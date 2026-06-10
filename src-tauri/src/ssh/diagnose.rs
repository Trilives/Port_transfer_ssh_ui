//! 根据 ssh stderr 把连接失败归类，区分「主机不可达 / IP 端口错误」与「需要密码/认证失败」，
//! 避免把所有失败都当成「需要密码」。探测与运行态退出都复用这里的分类。

/// 一次 ssh 失败的归类。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SshFailureKind {
    /// 远端主机指纹变化（known_hosts 不一致）。
    HostKeyChanged,
    /// 无法解析主机名（SSH 主机/域名填写有误）。
    ResolveFailed,
    /// 目标拒绝连接（端口错误或对端未运行 SSH）。
    ConnectionRefused,
    /// 连接超时（IP 不可达、网络或防火墙拦截）。
    Timeout,
    /// 网络不可达 / 无法路由到主机。
    NetworkUnreachable,
    /// 已连到主机但认证未通过（需要密码或密钥认证失败）。
    AuthRequired,
    /// 无法归类的其他失败。
    Unknown,
}

impl SshFailureKind {
    /// 本地化的简短原因，给用户看。
    pub fn reason(self, zh: bool) -> &'static str {
        match self {
            Self::HostKeyChanged => {
                if zh {
                    "远程主机指纹已改变，连接被拒绝。"
                } else {
                    "Remote host key changed; connection refused."
                }
            }
            Self::ResolveFailed => {
                if zh {
                    "无法解析 SSH 主机名，请检查「SSH 主机」是否填写正确。"
                } else {
                    "Could not resolve the SSH host. Check the SSH host field."
                }
            }
            Self::ConnectionRefused => {
                if zh {
                    "目标拒绝连接，请检查「SSH 端口」是否正确，或远端是否已开启 SSH 服务。"
                } else {
                    "Connection refused. Check the SSH port, or whether the remote SSH service is running."
                }
            }
            Self::Timeout => {
                if zh {
                    "连接超时，主机不可达，请检查 IP、网络连通性或防火墙。"
                } else {
                    "Connection timed out; host unreachable. Check the IP, network, or firewall."
                }
            }
            Self::NetworkUnreachable => {
                if zh {
                    "网络不可达，无法路由到该主机，请检查网络与 IP。"
                } else {
                    "Network unreachable; no route to host. Check the network and IP."
                }
            }
            Self::AuthRequired => {
                if zh {
                    "认证未通过，请确认 SSH 用户、密码或密钥是否正确。"
                } else {
                    "Authentication failed. Check the SSH user, password, or key."
                }
            }
            Self::Unknown => {
                if zh {
                    "无法连接到主机。"
                } else {
                    "Cannot reach the host."
                }
            }
        }
    }
}

/// 按 ssh stderr 内容归类失败原因。网络层问题优先于认证，避免把不可达当成「需要密码」。
pub fn classify_ssh_failure(stderr: &str) -> SshFailureKind {
    let lower = stderr.to_lowercase();

    if lower.contains("remote host identification has changed")
        || lower.contains("host key verification failed")
    {
        return SshFailureKind::HostKeyChanged;
    }

    if lower.contains("could not resolve hostname")
        || lower.contains("name or service not known")
        || lower.contains("nodename nor servname")
        || lower.contains("no address associated with hostname")
    {
        return SshFailureKind::ResolveFailed;
    }
    if lower.contains("connection refused") {
        return SshFailureKind::ConnectionRefused;
    }
    if lower.contains("connection timed out")
        || lower.contains("operation timed out")
        || lower.contains("timed out")
    {
        return SshFailureKind::Timeout;
    }
    if lower.contains("network is unreachable") || lower.contains("no route to host") {
        return SshFailureKind::NetworkUnreachable;
    }

    // 已连上主机但免密认证不可用 / 认证失败 → 需要密码。
    if lower.contains("permission denied")
        || lower.contains("publickey")
        || lower.contains("no supported authentication")
        || lower.contains("too many authentication failures")
        || lower.contains("authentication failed")
        || lower.contains("keyboard-interactive")
        || lower.contains("connection closed by")
    {
        return SshFailureKind::AuthRequired;
    }

    SshFailureKind::Unknown
}

/// 把「原因 + 原始 stderr 明细」拼成给用户展示的多行文本。
pub fn format_failure(reason: &str, detail: &str) -> String {
    let detail = detail.trim();
    if detail.is_empty() {
        reason.to_string()
    } else {
        format!("{reason}\n\n{detail}")
    }
}
