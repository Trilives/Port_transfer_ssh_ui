//! 解析与写入本机 `~/.ssh/config`：导入时取出主机连接参数，导出时把主机写入一个
//! 受本程序托管的区块（不触碰用户已有的其它条目）。

use std::path::PathBuf;

use uuid::Uuid;

use crate::model::Host;
use crate::util::now_millis;

const BLOCK_BEGIN: &str = "# >>> ssh-port-forwarder managed >>>";
const BLOCK_END: &str = "# <<< ssh-port-forwarder managed <<<";

/// `~/.ssh/config` 路径。
pub fn ssh_config_path() -> Result<PathBuf, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Cannot locate user home directory.".to_string())?;
    Ok(PathBuf::from(home).join(".ssh").join("config"))
}

/// 解析 ssh config 文本为主机列表。按 `Host <alias>` 分块，跳过通配（含 `*`/`?`）。
pub fn parse_ssh_config(content: &str) -> Vec<Host> {
    let mut hosts: Vec<Host> = Vec::new();
    let mut current: Option<Host> = None;
    let mut alias = String::new();

    let flush = |hosts: &mut Vec<Host>, host: Option<Host>, alias: &str| {
        if let Some(mut host) = host {
            // HostName 缺省时回退到别名本身。
            if host.ssh_host.trim().is_empty() {
                host.ssh_host = alias.to_string();
            }
            hosts.push(host);
        }
    };

    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = split_key_value(line);
        let key_lower = key.to_lowercase();
        if key_lower == "host" {
            // 收尾上一个块。
            flush(&mut hosts, current.take(), &alias);
            // 只取第一个别名；含通配的整块跳过。
            let first = value.split_whitespace().next().unwrap_or("");
            if first.is_empty() || first.contains('*') || first.contains('?') {
                current = None;
                alias = String::new();
                continue;
            }
            alias = first.to_string();
            current = Some(Host {
                id: Uuid::new_v4().to_string(),
                name: alias.clone(),
                ssh_host: String::new(),
                ssh_port: "22".to_string(),
                ssh_user: String::new(),
                identity_file: String::new(),
                extra_options: String::new(),
                proxy_jump: String::new(),
                forwards: Vec::new(),
                pinned: false,
                updated_at: now_millis(),
            });
            continue;
        }
        let Some(host) = current.as_mut() else {
            continue;
        };
        match key_lower.as_str() {
            "hostname" => host.ssh_host = value.to_string(),
            "user" => host.ssh_user = value.to_string(),
            "port" => host.ssh_port = value.to_string(),
            "identityfile" => host.identity_file = value.to_string(),
            "proxyjump" => host.proxy_jump = value.to_string(),
            _ => {}
        }
    }
    flush(&mut hosts, current.take(), &alias);
    hosts
}

/// 在 config 中找到 `HostName` 等于该 IP 的第一个别名（用于 VS Code 按 IP 复用已有条目）。
pub fn find_alias_for_ip(content: &str, ip: &str) -> Option<String> {
    let ip = ip.trim();
    parse_ssh_config(content)
        .into_iter()
        .find(|host| host.ssh_host.trim() == ip && !host.name.trim().is_empty())
        .map(|host| host.name)
}

/// 在 config 末尾追加一个主机条目（仅写非空、ssh 可解析的字段），返回新的完整文本。
/// 不触碰已有内容，与托管区块互不影响。
pub fn append_host_stanza(content: &str, alias: &str, host: &Host) -> String {
    let mut out = content.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str("# added by ssh-port-forwarder\n");
    out.push_str(&format!("Host {alias}\n"));
    if !host.ssh_host.trim().is_empty() {
        out.push_str(&format!("    HostName {}\n", host.ssh_host.trim()));
    }
    if !host.ssh_user.trim().is_empty() {
        out.push_str(&format!("    User {}\n", host.ssh_user.trim()));
    }
    if !host.ssh_port.trim().is_empty() && host.ssh_port.trim() != "22" {
        out.push_str(&format!("    Port {}\n", host.ssh_port.trim()));
    }
    if !host.identity_file.trim().is_empty() {
        out.push_str(&format!("    IdentityFile {}\n", host.identity_file.trim()));
    }
    if !host.proxy_jump.trim().is_empty() {
        out.push_str(&format!("    ProxyJump {}\n", host.proxy_jump.trim()));
    }
    out
}

/// `Key value` 或 `Key=value`，返回去引号的值。
fn split_key_value(line: &str) -> (String, String) {
    let (key, rest) = match line.find(|c: char| c.is_whitespace() || c == '=') {
        Some(idx) => (line[..idx].to_string(), line[idx..].to_string()),
        None => (line.to_string(), String::new()),
    };
    let value = rest.trim_start_matches([' ', '\t', '=']).trim();
    let value = value.trim_matches('"');
    (key, value.to_string())
}

/// 把主机列表渲染成 ssh config 条目（只写非空、ssh 可解析的字段）。
pub fn render_managed_block(hosts: &[Host]) -> String {
    let mut out = String::new();
    out.push_str(BLOCK_BEGIN);
    out.push('\n');
    for host in hosts {
        let alias = if host.name.trim().is_empty() {
            host.ssh_host.trim()
        } else {
            host.name.trim()
        };
        if alias.is_empty() {
            continue;
        }
        out.push_str(&format!("Host {alias}\n"));
        if !host.ssh_host.trim().is_empty() {
            out.push_str(&format!("    HostName {}\n", host.ssh_host.trim()));
        }
        if !host.ssh_user.trim().is_empty() {
            out.push_str(&format!("    User {}\n", host.ssh_user.trim()));
        }
        if !host.ssh_port.trim().is_empty() && host.ssh_port.trim() != "22" {
            out.push_str(&format!("    Port {}\n", host.ssh_port.trim()));
        }
        if !host.identity_file.trim().is_empty() {
            out.push_str(&format!("    IdentityFile {}\n", host.identity_file.trim()));
        }
        if !host.proxy_jump.trim().is_empty() {
            out.push_str(&format!("    ProxyJump {}\n", host.proxy_jump.trim()));
        }
        out.push('\n');
    }
    out.push_str(BLOCK_END);
    out.push('\n');
    out
}

/// 取出托管区块以外的内容（用户自己写的条目），用于查重时排除本程序托管的部分。
pub fn strip_managed_block(content: &str) -> String {
    let begin = content.find(BLOCK_BEGIN);
    let end = content.find(BLOCK_END);
    if let (Some(begin), Some(end)) = (begin, end) {
        if end > begin {
            let after = end + BLOCK_END.len();
            let tail = content[after..].strip_prefix('\n').unwrap_or(&content[after..]);
            let mut result = String::new();
            result.push_str(&content[..begin]);
            result.push_str(tail);
            return result;
        }
    }
    content.to_string()
}

/// 用新的托管区块替换已有区块；没有则追加到文件末尾。区块外内容原样保留。
pub fn upsert_managed_block(existing: &str, block: &str) -> String {
    let begin = existing.find(BLOCK_BEGIN);
    let end = existing.find(BLOCK_END);
    if let (Some(begin), Some(end)) = (begin, end) {
        if end > begin {
            let after = end + BLOCK_END.len();
            // 连同区块结束行后的一个换行一起替换掉。
            let tail = existing[after..].strip_prefix('\n').unwrap_or(&existing[after..]);
            let mut result = String::new();
            result.push_str(&existing[..begin]);
            result.push_str(block);
            result.push_str(tail);
            return result;
        }
    }
    let mut result = existing.to_string();
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    if !result.is_empty() {
        result.push('\n');
    }
    result.push_str(block);
    result
}
