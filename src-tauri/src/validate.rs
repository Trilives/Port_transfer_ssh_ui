use crate::model::{Forward, Host, TunnelMode};

/// 仅校验 SSH 连接可达所需的最小参数（用于探测、上传公钥等运行时操作）。
/// 新建/编辑主机时不校验，参数是否可用一律留到连接运行时判断。
pub fn validate_host_connection(host: &Host) -> Result<(), String> {
    let mut errors = Vec::new();
    if host.ssh_host.trim().is_empty() {
        errors.push("SSH host is required.");
    }
    if host.ssh_port.parse::<u16>().is_err() {
        errors.push("SSH port must be a valid port.");
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// 校验一条转发的参数。
pub fn validate_forward(forward: &Forward) -> Result<(), String> {
    let mut errors = Vec::new();
    if forward.name.trim().is_empty() {
        errors.push("Name is required.");
    }
    if forward.bind_port.parse::<u16>().is_err() {
        errors.push("Bind port must be a valid port.");
    }
    if forward.mode != TunnelMode::Dynamic {
        if forward.target_host.trim().is_empty() {
            errors.push("Target host is required.");
        }
        if forward.target_port.parse::<u16>().is_err() {
            errors.push("Target port must be a valid port.");
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
