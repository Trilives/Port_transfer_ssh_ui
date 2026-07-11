//! Open a remote host via VS Code's Remote-SSH: detect the install, read connection history, write to
//! `~/.ssh/config` as needed, and open with `code --folder-uri`.
//!
//! Connection history has two sources, merged and deduplicated by IP match (only one entry kept per path):
//! 1. **Preferred**: the Remote-SSH extension's own `folder.history.v1`, stored in `state.vscdb` (SQLite)'s
//!    `ItemTable` under key=`ms-vscode-remote.remote-ssh`. This is the newest and most complete "recently opened
//!    remote folders" list, updated live on every open, avoiding stale entries from a lagging `storage.json`.
//! 2. **Fallback**: `storage.json` → `profileAssociations.workspaces`, whose keys are folder URIs shaped like
//!    `vscode-remote://ssh-remote%2B<authority>/<path>`.
//!
//! Both sources' authority comes in two forms: a bare IP (connected directly by IP), or a hex-encoded
//! `{"hostName":"<alias>"}` (connected via an ssh config alias). The alias is resolved back to an IP via the local
//! `~/.ssh/config`'s `HostName`, then compared against the host's IP.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::Serialize;

use crate::model::Host;
use crate::ssh::command::build_send_command;
use crate::sshconfig::{append_host_stanza, find_alias_for_ip, parse_ssh_config, sanitize_alias, ssh_config_path};
use crate::util::no_window;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VscodeStatus {
    /// Whether the VS Code executable can be found.
    pub installed: bool,
    /// Whether the Remote-SSH extension (ms-vscode-remote.remote-ssh) is installed.
    pub remote_ssh: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VscodeHistoryEntry {
    /// The raw folder URI, used to reopen it as-is via `code --folder-uri`.
    pub uri: String,
    /// The decoded remote path, used for display in the UI.
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VscodeOpenRootResult {
    /// Whether ~/.ssh/config was newly written to this time.
    pub added_to_config: bool,
    /// The ssh config alias used for the connection.
    pub alias: String,
    /// The folder URI opened (for a specific path), so it can be recorded in history and reopened. Empty for a direct connect.
    pub uri: String,
}

/// How to launch VS Code: call `Code.exe` directly, or fall back to `cmd /c code.cmd`.
struct CodeLauncher {
    program: PathBuf,
    prefix: Vec<String>,
}

/// Detect whether VS Code and the Remote-SSH extension are installed.
pub fn status() -> VscodeStatus {
    VscodeStatus {
        installed: code_launcher().is_some(),
        remote_ssh: remote_ssh_installed(),
    }
}

/// Read all VS Code history remote folders matching the given IP.
/// Takes the Remote-SSH extension's latest `folder.history.v1` first, falls back to `storage.json`, and dedups by path.
pub fn ssh_history_for_ip(ip: &str) -> Vec<VscodeHistoryEntry> {
    let ip = ip.trim();
    if ip.is_empty() {
        return Vec::new();
    }
    let alias_map = alias_to_ip_map();
    let mut out = Vec::new();
    let mut seen_paths = HashSet::new();

    // 1. Preferred: the Remote-SSH extension's own recently-opened folders (newest, most complete).
    for (authority, paths) in remote_ssh_folder_history() {
        if resolve_authority_ip(&authority, &alias_map).as_deref() != Some(ip) {
            continue;
        }
        for path in paths {
            if seen_paths.insert(path.clone()) {
                let uri = format!("vscode-remote://ssh-remote+{}{}", authority, percent_encode_path(&path));
                out.push(VscodeHistoryEntry { uri, path });
            }
        }
    }

    // 2. Fallback: storage.json's profileAssociations.workspaces (may lag, but fills in entries missing from the extension).
    if let Some(workspaces) = read_profile_association_workspaces() {
        for key in workspaces {
            let Some((authority, rest)) = parse_remote_uri(&key) else {
                continue;
            };
            if resolve_authority_ip(&authority, &alias_map).as_deref() != Some(ip) {
                continue;
            }
            let path = percent_decode(&rest);
            if seen_paths.insert(path.clone()) {
                out.push(VscodeHistoryEntry { uri: key, path });
            }
        }
    }
    out
}

/// Read all the keys (folder URIs) of `storage.json`'s `profileAssociations.workspaces`.
fn read_profile_association_workspaces() -> Option<Vec<String>> {
    let path = storage_json_path()?;
    let text = fs::read_to_string(&path).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&text).ok()?;
    let workspaces = json
        .get("profileAssociations")
        .and_then(|value| value.get("workspaces"))
        .and_then(|value| value.as_object())?;
    Some(workspaces.keys().cloned().collect())
}

/// Read the Remote-SSH extension's `folder.history.v1` (authority → list of recently opened remote directories).
/// Comes from `state.vscdb` (SQLite)'s `ItemTable`, key=`ms-vscode-remote.remote-ssh`.
/// Any read failure (missing file, locked, format change) returns an empty map, falling back to storage.json.
fn remote_ssh_folder_history() -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    let Some(value) = read_state_db_item("ms-vscode-remote.remote-ssh") else {
        return map;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&value) else {
        return map;
    };
    let Some(history) = json.get("folder.history.v1").and_then(|value| value.as_object()) else {
        return map;
    };
    for (authority, paths) in history {
        let list: Vec<String> = paths
            .as_array()
            .map(|array| array.iter().filter_map(|item| item.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        if !list.is_empty() {
            map.insert(authority.clone(), list);
        }
    }
    map
}

/// Read a key's value (JSON text) from `state.vscdb`'s `ItemTable`, read-only.
fn read_state_db_item(key: &str) -> Option<String> {
    let path = state_vscdb_path()?;
    if !path.exists() {
        return None;
    }
    let conn = rusqlite::Connection::open_with_flags(
        &path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .ok()?;
    // VS Code may briefly hold the lock while running; wait a bit before giving up, falling back to storage.json on failure.
    let _ = conn.busy_timeout(Duration::from_millis(800));
    conn.query_row("SELECT value FROM ItemTable WHERE key = ?1", [key], |row| {
        // value is usually TEXT; some versions store it as BLOB, so try both.
        row.get::<_, String>(0)
            .or_else(|_| row.get::<_, Vec<u8>>(0).map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
    })
    .ok()
}

/// Open a history remote folder with `code --folder-uri <uri>`.
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

/// Direct connect: find (or write) the ssh config alias for this IP, then open a connected remote window with no
/// folder using VS Code's default mode (`code --remote ssh-remote+<alias>`, without any history path).
pub fn open_direct_for_host(host: &Host) -> Result<VscodeOpenRootResult, String> {
    let (alias, added_to_config) = ensure_alias(host)?;
    open_remote_window(&alias)?;
    Ok(VscodeOpenRootResult { added_to_config, alias, uri: String::new() })
}

/// Open the specified remote directory. Absolute paths (starting with `/`) open as-is; `~` or relative paths resolve against the home directory.
pub fn open_path_for_host(host: &Host, path: &str) -> Result<VscodeOpenRootResult, String> {
    let path = path.trim();
    if path.is_empty() {
        return open_direct_for_host(host);
    }
    let (alias, added_to_config) = ensure_alias(host)?;

    let absolute = if path.starts_with('/') {
        path.to_string()
    } else {
        // ~ or relative path: resolve against the remote home directory.
        let rel = path.strip_prefix('~').unwrap_or(path).trim_start_matches('/');
        let home = query_remote_home(host)
            .ok_or_else(|| "Cannot resolve the remote home directory. Use an absolute path that starts with /.".to_string())?;
        if rel.is_empty() {
            home
        } else {
            format!("{}/{}", home.trim_end_matches('/'), rel)
        }
    };

    let uri = format!("vscode-remote://ssh-remote+{}{}", alias, encode_path(&absolute));
    open_folder_uri(&uri)?;
    Ok(VscodeOpenRootResult { added_to_config, alias, uri })
}

/// Find the ssh config alias for this host's IP; if none exists, append one using the host name (falling back to the IP on conflict).
/// Returns (alias, whether it was newly written this time).
fn ensure_alias(host: &Host) -> Result<(String, bool), String> {
    let ip = host.ssh_host.trim();
    if ip.is_empty() {
        return Err("Host has no SSH host/IP.".to_string());
    }
    let config_path = ssh_config_path()?;
    let content = fs::read_to_string(&config_path).unwrap_or_default();
    match find_alias_for_ip(&content, ip) {
        Some(alias) => Ok((alias, false)),
        None => {
            let alias = pick_alias(&content, host, ip);
            let updated = append_host_stanza(&content, &alias, host);
            if let Some(parent) = config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(&config_path, updated).map_err(|err| err.to_string())?;
            Ok((alias, true))
        }
    }
}

/// Probe the remote `$HOME` with a one-off SSH call (passwordless, 8s timeout, no password prompt); returns None on failure or a non-absolute result.
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

/// Open a connected remote window with no folder (`code --remote ssh-remote+<authority>`).
fn open_remote_window(authority: &str) -> Result<(), String> {
    let launcher = code_launcher().ok_or_else(|| "VS Code not found.".to_string())?;
    let mut command = Command::new(&launcher.program);
    command.args(&launcher.prefix);
    command.arg("--remote").arg(format!("ssh-remote+{authority}"));
    no_window(&mut command);
    command.spawn().map_err(|err| err.to_string())?;
    Ok(())
}

/// Minimally escape only spaces in a folderUri path (home directories are almost always ASCII; returned as-is when there are no spaces).
fn encode_path(path: &str) -> String {
    path.replace(' ', "%20")
}

/// Percent-encode a remote path into the path portion of a folderUri: keeps unreserved characters and `/`, escapes everything else as UTF-8 bytes.
/// Used to build a URI directly openable via `--folder-uri` from `folder.history.v1`'s raw path (which may contain spaces or non-ASCII characters).
fn percent_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for &byte in path.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

// ---- Internal helpers ----

/// A map of `alias -> HostName` from `~/.ssh/config`, used to resolve aliases in VS Code history back to an IP.
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

/// Pick an alias for a host to be written into the config that doesn't conflict with existing entries (prefers the host name, then the IP).
fn pick_alias(content: &str, host: &Host, ip: &str) -> String {
    let existing: HashSet<String> = parse_ssh_config(content)
        .into_iter()
        .map(|host| host.name)
        .collect();
    // The alias can't contain whitespace (ssh / VS Code would split it into multiple patterns); replace with underscores.
    let mut candidate = sanitize_alias(host.name.trim());
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

/// Split `vscode-remote://ssh-remote(+|%2B)<authority>/<path>` into its authority and path parts.
fn parse_remote_uri(uri: &str) -> Option<(String, String)> {
    let rest = uri.strip_prefix("vscode-remote://ssh-remote")?;
    let rest = rest.strip_prefix("%2B").or_else(|| rest.strip_prefix('+'))?;
    let slash = rest.find('/').unwrap_or(rest.len());
    let authority = rest[..slash].to_string();
    let path = &rest[slash..];
    let path = if path.is_empty() { "/".to_string() } else { path.to_string() };
    Some((authority, path))
}

/// Resolve authority to a target IP: hex-encoded `{"hostName":..}` → alias → resolved via config; a bare string is treated as an IP/alias.
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

/// Decode %XX escapes in a URI path (then restore as UTF-8), used for display in the UI.
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

/// Path to `storage.json` under the VS Code user data directory.
fn storage_json_path() -> Option<PathBuf> {
    Some(global_storage_dir()?.join("storage.json"))
}

/// Path to `state.vscdb` (SQLite) under the VS Code user data directory.
fn state_vscdb_path() -> Option<PathBuf> {
    Some(global_storage_dir()?.join("state.vscdb"))
}

/// The `%APPDATA%\Code\User\globalStorage` directory.
fn global_storage_dir() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    Some(PathBuf::from(appdata).join("Code").join("User").join("globalStorage"))
}

/// Whether the main Remote-SSH extension directory exists under `~/.vscode/extensions` (excludes remote-ssh-edit).
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
        // What follows the prefix should be a version number digit; this excludes ms-vscode-remote.remote-ssh-edit.
        if name.starts_with(PREFIX)
            && name.as_bytes().get(PREFIX.len()).is_some_and(u8::is_ascii_digit)
        {
            return true;
        }
    }
    false
}

/// Locate how to launch VS Code: prefer `Code.exe` (including registry probing, so non-standard drive installs work),
/// falling back to `cmd /c code.cmd`.
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
            // bin\code(.cmd)'s parent directory is the install root, which contains Code.exe.
            if dir.join("code.cmd").exists() || dir.join("code").exists() {
                if let Some(parent) = dir.parent() {
                    candidates.push(parent.join("Code.exe"));
                }
            }
        }
    }
    // Registry probing: doesn't rely on PATH, covers installs on any drive letter (e.g. E:\).
    if let Some(exe) = find_code_via_registry() {
        candidates.push(exe);
    }
    for candidate in &candidates {
        if candidate.exists() {
            return Some(CodeLauncher { program: candidate.clone(), prefix: Vec::new() });
        }
    }
    // Fallback: Code.exe not found, but code.cmd is on PATH, so invoke it via cmd /c.
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

/// Probe the Windows registry for `Code.exe`'s path: check the `vscode://` protocol handler first, then the uninstall entry's DisplayIcon.
fn find_code_via_registry() -> Option<PathBuf> {
    // vscode protocol command: the default value looks like "<drive>\...\Code.exe" --open-url -- "%1"
    let protocol_keys = [
        "HKEY_CLASSES_ROOT\\vscode\\shell\\open\\command",
        "HKEY_CURRENT_USER\\Software\\Classes\\vscode\\shell\\open\\command",
    ];
    for key in protocol_keys {
        if let Some(exe) = reg_query_code_exe(&[key, "/ve"]) {
            return Some(exe);
        }
    }
    // Uninstall entry DisplayIcon: looks like <drive>\...\Code.exe,0 (user install under HKCU, system install under HKLM).
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

/// Run `reg query <args>` and extract an existing `Code.exe` path from the output.
fn reg_query_code_exe(args: &[&str]) -> Option<PathBuf> {
    let mut command = Command::new("reg");
    command.arg("query").args(args);
    no_window(&mut command);
    let output = command.output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    extract_code_exe(&text)
}

/// Extract a path ending in `Code.exe` from a line of `reg` output (handles both quoted protocol commands and bare DisplayIcon values).
fn extract_code_exe(text: &str) -> Option<PathBuf> {
    for line in text.lines() {
        let lower = line.to_lowercase();
        let Some(hit) = lower.find("code.exe") else {
            continue;
        };
        let end = hit + "code.exe".len();
        let prefix = &line[..end];
        let start = if let Some(quote) = prefix.rfind('"') {
            // Protocol command: "<path>\Code.exe" --open-url ...
            quote + 1
        } else if let Some(sz) = lower[..end].rfind("reg_sz") {
            // DisplayIcon: <path>\Code.exe,0, with the path following REG_SZ and some whitespace.
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
