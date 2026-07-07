//! Classify connection failures from ssh stderr, distinguishing "host unreachable / bad IP-port" from "password
//! required / auth failed" so not every failure is treated as needing a password. Both probing and runtime exit reuse this classification.

/// The classification of a single ssh failure.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SshFailureKind {
    /// The remote host's fingerprint changed (mismatch with known_hosts).
    HostKeyChanged,
    /// Could not resolve the host name (SSH host/domain field is wrong).
    ResolveFailed,
    /// The target refused the connection (wrong port, or SSH isn't running on the other end).
    ConnectionRefused,
    /// Connection timed out (IP unreachable, network, or firewall blocking).
    Timeout,
    /// Network unreachable / no route to the host.
    NetworkUnreachable,
    /// Connected to the host but authentication failed (password required or key auth failed).
    AuthRequired,
    /// Any other failure that couldn't be classified.
    Unknown,
}

impl SshFailureKind {
    /// Whether this failure needs the user to fix something (auth or host-key), so auto-reconnect must NOT retry.
    /// Network-layer failures (timeout, unreachable, refused, resolve) and Unknown are transient — safe to retry.
    pub fn is_fatal(self) -> bool {
        matches!(self, Self::AuthRequired | Self::HostKeyChanged)
    }

    /// A short, localized reason shown to the user.
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

/// Classify the failure reason from ssh stderr content. Network-layer issues take priority over auth, so unreachable isn't mistaken for "password required".
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

    // Connected to the host but passwordless auth isn't available / auth failed → password required.
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

/// Combine "reason + raw stderr detail" into a multi-line message for display to the user.
pub fn format_failure(reason: &str, detail: &str) -> String {
    let detail = detail.trim();
    if detail.is_empty() {
        reason.to_string()
    } else {
        format!("{reason}\n\n{detail}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_and_host_key_are_fatal() {
        assert!(classify_ssh_failure("Permission denied (publickey,password).").is_fatal());
        assert!(classify_ssh_failure("Host key verification failed.").is_fatal());
    }

    #[test]
    fn network_failures_are_transient() {
        assert!(!classify_ssh_failure("ssh: connect to host x port 22: Connection timed out").is_fatal());
        assert!(!classify_ssh_failure("Network is unreachable").is_fatal());
        assert!(!classify_ssh_failure("connect to host x port 22: Connection refused").is_fatal());
        // A bare mid-session drop with no recognizable reason must be retried, not treated as fatal.
        assert!(!classify_ssh_failure("client_loop: send disconnect: Broken pipe").is_fatal());
        assert!(!SshFailureKind::Unknown.is_fatal());
    }
}
