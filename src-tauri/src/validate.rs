use crate::model::{Forward, Host, TunnelMode};

/// Whether an SSH host string uses only characters `ssh` accepts for a hostname/FQDN or IP address:
/// ASCII letters, digits, and `. - _ :` (colon for IPv6, `%` for an IPv6 zone id). This rejects spaces
/// and non-ASCII characters (e.g. Chinese) up front, since ssh cannot resolve or connect to them.
pub fn hostname_chars_ok(host: &str) -> bool {
    let trimmed = host.trim();
    trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':' | '%'))
}

/// Validate only the minimal parameters needed for SSH connectivity (used by runtime operations like probing, uploading a key).
/// Not validated when creating/editing a host; usability is always deferred to connect time.
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

/// Validate a forward's parameters.
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
