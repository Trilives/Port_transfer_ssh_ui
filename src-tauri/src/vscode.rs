//! 通过 VS Code 的 Remote-SSH 打开远端主机：检测安装、读取历史连接、按需写入
//! `~/.ssh/config` 并用 `code --folder-uri` 打开。
//!
//! 历史连接来自 VS Code 的 `storage.json` → `profileAssociations.workspaces`，其 key 是
//! 形如 `vscode-remote://ssh-remote%2B<authority>/<path>` 的文件夹 URI。authority 有两种：
//! 裸 IP（按 IP 直连过），或 hex 编码的 `{"hostName":"<别名>"}`（按 ssh config 别名连过）。
//! 别名通过本机 `~/.ssh/config` 的 `HostName` 解析回 IP，再与主机 IP 比对。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::Serialize;

use crate::model::Host;
use crate::ssh::command::build_send_command;
use crate::sshconfig::{append_host_stanza, find_alias_for_ip, parse_ssh_config, ssh_config_path};
use crate::util::no_window;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VscodeStatus {
    /// 是否能找到 VS Code 可执行文件。
    pub installed: bool,
    /// 是否安装了 Remote-SSH 扩展（ms-vscode-remote.remote-ssh）。
    pub remote_ssh: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VscodeHistoryEntry {
    /// 原始文件夹 URI，用于 `code --folder-uri` 原样重开。
    pub uri: String,
    /// 解码后的远端路径，用于界面展示。
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VscodeOpenRootResult {
    /// 是否本次新写入了 ~/.ssh/config。
    pub added_to_config: bool,
    /// 连接所用的 ssh config 别名。
    pub alias: String,
}

/// 启动 VS Code 的方式：直接调用 `Code.exe`，或回退到 `cmd /c code.cmd`。
struct CodeLauncher {
    program: PathBuf,
    prefix: Vec<String>,
}

/// 探测 VS Code 与 Remote-SSH 扩展的安装情况。
pub fn status() -> VscodeStatus {
    VscodeStatus {
        installed: code_launcher().is_some(),
        remote_ssh: remote_ssh_installed(),
    }
}

/// 读取与指定 IP 对应的所有 VS Code 历史远端文件夹。
pub fn ssh_history_for_ip(ip: &str) -> Vec<VscodeHistoryEntry> {
    let ip = ip.trim();
    if ip.is_empty() {
        return Vec::new();
    }
    let alias_map = alias_to_ip_map();
    let Some(path) = storage_json_path() else {
        return Vec::new();
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    let Some(workspaces) = json
        .get("profileAssociations")
        .and_then(|value| value.get("workspaces"))
        .and_then(|value| value.as_object())
    else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for key in workspaces.keys() {
        let Some((authority, rest)) = parse_remote_uri(key) else {
            continue;
        };
        if resolve_authority_ip(&authority, &alias_map).as_deref() == Some(ip) && seen.insert(key.clone()) {
            out.push(VscodeHistoryEntry {
                uri: key.clone(),
                path: percent_decode(&rest),
            });
        }
    }
    out
}

/// 用 `code --folder-uri <uri>` 打开一个历史远端文件夹。
pub fn open_folder_uri(uri: &str) -> Result<(), String> {
    if !uri.starts_with("vscode-remote://") {
        return Err("Invalid VS Code folder URI.".to_string());
    }
    let launcher = code_launcher().ok_or_else(|| "VS Code not found.".to_string())?;
    let mut command = Command::new(&launcher.program);
    command.args(&launcher.prefix);
    command.arg("--folder-uri").arg(uri);
    no_window(&mut command);
    command.spawn().map_err(|err| err.to_string())?;
    Ok(())
}

/// 直连/打开家目录：找到（或写入）该 IP 对应的 ssh config 别名，探测远端 `$HOME` 并用别名打开它；
/// 探测不到（需密码/不可达）时回退到打开一个已连接但不带文件夹的远端窗口（仍落在家目录）。
pub fn open_home_for_host(host: &Host) -> Result<VscodeOpenRootResult, String> {
    let ip = host.ssh_host.trim();
    if ip.is_empty() {
        return Err("Host has no SSH host/IP.".to_string());
    }
    let config_path = ssh_config_path()?;
    let content = fs::read_to_string(&config_path).unwrap_or_default();

    let (alias, added_to_config) = match find_alias_for_ip(&content, ip) {
        Some(alias) => (alias, false),
        None => {
            let alias = pick_alias(&content, host, ip);
            let updated = append_host_stanza(&content, &alias, host);
            if let Some(parent) = config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(&config_path, updated).map_err(|err| err.to_string())?;
            (alias, true)
        }
    };

    match query_remote_home(host) {
        Some(home) => {
            let uri = format!("vscode-remote://ssh-remote+{}{}", alias, encode_path(&home));
            open_folder_uri(&uri)?;
        }
        None => open_remote_window(&alias)?,
    }
    Ok(VscodeOpenRootResult { added_to_config, alias })
}

/// 用一次性 SSH（免密、8s 超时、不弹密码）探测远端 `$HOME`；失败/非绝对路径时返回 None。
fn query_remote_home(host: &Host) -> Option<String> {
    let argv = build_send_command(host, "printf '%s' \"$HOME\"", false).ok()?;
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    no_window(&mut command);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
    home.starts_with('/').then_some(home)
}

/// 打开一个已连接但不带文件夹的远端窗口（`code --remote ssh-remote+<authority>`）。
fn open_remote_window(authority: &str) -> Result<(), String> {
    let launcher = code_launcher().ok_or_else(|| "VS Code not found.".to_string())?;
    let mut command = Command::new(&launcher.program);
    command.args(&launcher.prefix);
    command.arg("--remote").arg(format!("ssh-remote+{authority}"));
    no_window(&mut command);
    command.spawn().map_err(|err| err.to_string())?;
    Ok(())
}

/// 仅对 folderUri 路径里的空格做最小转义（家目录几乎都是 ASCII，无空格时原样返回）。
fn encode_path(path: &str) -> String {
    path.replace(' ', "%20")
}

// ---- 内部辅助 ----

/// `~/.ssh/config` 中 `别名 -> HostName` 的映射，用于把 VS Code 历史里的别名解析回 IP。
fn alias_to_ip_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(path) = ssh_config_path() {
        if let Ok(text) = fs::read_to_string(path) {
            for host in parse_ssh_config(&text) {
                if !host.name.trim().is_empty() && !host.ssh_host.trim().is_empty() {
                    map.insert(host.name.clone(), host.ssh_host.clone());
                }
            }
        }
    }
    map
}

/// 为待写入 config 的主机挑一个不与现有条目冲突的别名（优先用主机名，其次用 IP）。
fn pick_alias(content: &str, host: &Host, ip: &str) -> String {
    let existing: HashSet<String> = parse_ssh_config(content)
        .into_iter()
        .map(|host| host.name)
        .collect();
    let mut candidate = host.name.trim().to_string();
    if candidate.is_empty() {
        candidate = ip.to_string();
    }
    if !existing.contains(&candidate) {
        return candidate;
    }
    if !existing.contains(ip) {
        return ip.to_string();
    }
    format!("{ip}-pf")
}

/// 拆出 `vscode-remote://ssh-remote(+|%2B)<authority>/<path>` 的 authority 与 path 部分。
fn parse_remote_uri(uri: &str) -> Option<(String, String)> {
    let rest = uri.strip_prefix("vscode-remote://ssh-remote")?;
    let rest = rest.strip_prefix("%2B").or_else(|| rest.strip_prefix('+'))?;
    let slash = rest.find('/').unwrap_or(rest.len());
    let authority = rest[..slash].to_string();
    let path = &rest[slash..];
    let path = if path.is_empty() { "/".to_string() } else { path.to_string() };
    Some((authority, path))
}

/// 把 authority 解析为目标 IP：hex 的 `{"hostName":..}` → 别名 → config 解析；裸串当作 IP/别名。
fn resolve_authority_ip(authority: &str, alias_map: &HashMap<String, String>) -> Option<String> {
    if !authority.is_empty()
        && authority.len() % 2 == 0
        && authority.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        if let Some(decoded) = hex_decode(authority) {
            if let Ok(text) = String::from_utf8(decoded) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(name) = json.get("hostName").and_then(|value| value.as_str()) {
                        return Some(alias_map.get(name).cloned().unwrap_or_else(|| name.to_string()));
                    }
                }
            }
        }
    }
    Some(alias_map.get(authority).cloned().unwrap_or_else(|| authority.to_string()))
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let hi = (bytes[index] as char).to_digit(16)?;
        let lo = (bytes[index + 1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
        index += 2;
    }
    Some(out)
}

/// 解码 URI 路径中的 %XX 转义（再按 UTF-8 还原），用于界面展示。
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hi = (bytes[index + 1] as char).to_digit(16);
            let lo = (bytes[index + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi * 16 + lo) as u8);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// VS Code 用户数据目录下的 `storage.json` 路径。
fn storage_json_path() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    Some(
        PathBuf::from(appdata)
            .join("Code")
            .join("User")
            .join("globalStorage")
            .join("storage.json"),
    )
}

/// `~/.vscode/extensions` 下是否存在 Remote-SSH 主扩展目录（排除 remote-ssh-edit）。
fn remote_ssh_installed() -> bool {
    let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) else {
        return false;
    };
    let dir = PathBuf::from(home).join(".vscode").join("extensions");
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    const PREFIX: &str = "ms-vscode-remote.remote-ssh-";
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        // 紧跟前缀的应是版本号数字，借此排除 ms-vscode-remote.remote-ssh-edit。
        if name.starts_with(PREFIX)
            && name.as_bytes().get(PREFIX.len()).is_some_and(u8::is_ascii_digit)
        {
            return true;
        }
    }
    false
}

/// 定位 VS Code 的启动方式：优先 `Code.exe`（含注册表探测，支持非标准盘符安装），
/// 回退 `cmd /c code.cmd`。
fn code_launcher() -> Option<CodeLauncher> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    for var in ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(base) = std::env::var_os(var) {
            candidates.push(PathBuf::from(&base).join("Programs").join("Microsoft VS Code").join("Code.exe"));
            candidates.push(PathBuf::from(&base).join("Microsoft VS Code").join("Code.exe"));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir.join("code.exe").exists() {
                candidates.push(dir.join("code.exe"));
            }
            // bin\code(.cmd) 的上级目录即安装根，根下是 Code.exe。
            if dir.join("code.cmd").exists() || dir.join("code").exists() {
                if let Some(parent) = dir.parent() {
                    candidates.push(parent.join("Code.exe"));
                }
            }
        }
    }
    // 注册表探测：不依赖 PATH，覆盖装在任意盘符（如 E:\）的情况。
    if let Some(exe) = find_code_via_registry() {
        candidates.push(exe);
    }
    for candidate in &candidates {
        if candidate.exists() {
            return Some(CodeLauncher { program: candidate.clone(), prefix: Vec::new() });
        }
    }
    // 回退：找不到 Code.exe，但 PATH 里有 code.cmd 时，用 cmd /c 调用。
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            for name in ["code.cmd", "code.exe"] {
                let candidate = dir.join(name);
                if candidate.exists() {
                    return Some(CodeLauncher {
                        program: PathBuf::from("cmd"),
                        prefix: vec!["/c".to_string(), candidate.to_string_lossy().into_owned()],
                    });
                }
            }
        }
    }
    None
}

/// 从 Windows 注册表里探测 `Code.exe` 路径：先看 `vscode://` 协议处理器，再看卸载项的 DisplayIcon。
fn find_code_via_registry() -> Option<PathBuf> {
    // vscode 协议命令：默认值形如 "<盘符>\...\Code.exe" --open-url -- "%1"
    let protocol_keys = [
        "HKEY_CLASSES_ROOT\\vscode\\shell\\open\\command",
        "HKEY_CURRENT_USER\\Software\\Classes\\vscode\\shell\\open\\command",
    ];
    for key in protocol_keys {
        if let Some(exe) = reg_query_code_exe(&[key, "/ve"]) {
            return Some(exe);
        }
    }
    // 卸载项 DisplayIcon：形如 <盘符>\...\Code.exe,0（用户版 HKCU、系统版 HKLM）。
    let uninstall_keys = [
        "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{771FD6B0-FA20-440A-A002-3B3BAC16DC50}_is1",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{EA457B21-F73E-494C-ACAB-524FDE069978}_is1",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{EA457B21-F73E-494C-ACAB-524FDE069978}_is1",
    ];
    for key in uninstall_keys {
        if let Some(exe) = reg_query_code_exe(&[key, "/v", "DisplayIcon"]) {
            return Some(exe);
        }
    }
    None
}

/// 运行 `reg query <args>` 并从输出里抽出存在的 `Code.exe` 路径。
fn reg_query_code_exe(args: &[&str]) -> Option<PathBuf> {
    let mut command = Command::new("reg");
    command.arg("query").args(args);
    no_window(&mut command);
    let output = command.output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    extract_code_exe(&text)
}

/// 从一行 `reg` 输出里抽取以 `Code.exe` 结尾的路径（兼容带引号的协议命令与裸 DisplayIcon）。
fn extract_code_exe(text: &str) -> Option<PathBuf> {
    for line in text.lines() {
        let lower = line.to_lowercase();
        let Some(hit) = lower.find("code.exe") else {
            continue;
        };
        let end = hit + "code.exe".len();
        let prefix = &line[..end];
        let start = if let Some(quote) = prefix.rfind('"') {
            // 协议命令："<path>\Code.exe" --open-url ...
            quote + 1
        } else if let Some(sz) = lower[..end].rfind("reg_sz") {
            // DisplayIcon：<path>\Code.exe,0，路径在 REG_SZ 与若干空白之后。
            let after = sz + "reg_sz".len();
            after + (line[after..end].len() - line[after..end].trim_start().len())
        } else {
            0
        };
        let candidate = PathBuf::from(line[start..end].trim());
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
