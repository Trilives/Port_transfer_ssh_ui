use std::fs;

use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::model::Host;
use crate::sshconfig::{
    parse_ssh_config, render_managed_block, ssh_config_path, strip_managed_block,
    upsert_managed_block,
};
use crate::state::AppState;
use crate::store::write_json;
use crate::util::{lock_error, now_millis};

/// 导出文件格式：带版本号包裹主机列表（含转发与端口，可再次导入）。
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportBundle {
    version: String,
    hosts: Vec<Host>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    /// "done" 或 "conflict"。
    pub status: String,
    pub duplicates: Vec<String>,
    pub added: usize,
    pub overwritten: usize,
    pub skipped: usize,
}

fn dedup_key(host: &Host) -> String {
    host.ssh_host.trim().to_lowercase()
}

/// 选文件读取并解析为主机列表（供前端勾选）；用户取消返回空列表。
#[tauri::command]
pub fn read_import_file() -> Result<Vec<Host>, String> {
    let Some(path) = FileDialog::new()
        .add_filter("SSH Port Forwarder", &["json"])
        .add_filter("All files", &["*"])
        .pick_file()
    else {
        return Ok(Vec::new());
    };
    let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    // 兼容带版本号的包裹结构与裸数组两种格式。
    if let Ok(bundle) = serde_json::from_str::<ExportBundle>(&content) {
        return Ok(bundle.hosts);
    }
    serde_json::from_str::<Vec<Host>>(&content)
        .map_err(|err| format!("Invalid import file: {err}"))
}

/// 读取本机 `~/.ssh/config` 并解析为主机列表（供前端勾选）。
#[tauri::command]
pub fn read_import_ssh_config() -> Result<Vec<Host>, String> {
    let path = ssh_config_path()?;
    if !path.exists() {
        return Err(format!("SSH config not found: {}", path.display()));
    }
    let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    Ok(parse_ssh_config(&content))
}

/// 把选中的主机并入现有列表。strategy: "" 探测冲突、"overwrite" 覆盖、"skip" 仅导入不重复。
#[tauri::command]
pub fn import_hosts(
    hosts: Vec<Host>,
    strategy: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<ImportResult, String> {
    // 先按 IP 对入参自身去重（保留先出现的）。
    let mut incoming: Vec<Host> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for host in hosts {
        if dedup_key(&host).is_empty() {
            continue;
        }
        if seen.insert(dedup_key(&host)) {
            incoming.push(host);
        }
    }

    let mut existing = state.hosts.lock().map_err(lock_error)?;
    let existing_keys: std::collections::HashSet<String> =
        existing.iter().map(dedup_key).collect();

    let duplicates: Vec<String> = incoming
        .iter()
        .filter(|host| existing_keys.contains(&dedup_key(host)))
        .map(|host| host.name.clone())
        .collect();

    if strategy.is_empty() && !duplicates.is_empty() {
        return Ok(ImportResult {
            status: "conflict".to_string(),
            duplicates,
            added: 0,
            overwritten: 0,
            skipped: 0,
        });
    }

    let mut added = 0;
    let mut overwritten = 0;
    let mut skipped = 0;
    let now = now_millis();

    for mut host in incoming {
        let key = dedup_key(&host);
        if let Some(pos) = existing.iter().position(|item| dedup_key(item) == key) {
            // 与现有主机 IP 冲突。
            if strategy == "overwrite" {
                let target = &mut existing[pos];
                target.name = host.name.clone();
                target.ssh_host = host.ssh_host.clone();
                target.ssh_user = host.ssh_user.clone();
                target.ssh_port = host.ssh_port.clone();
                target.identity_file = host.identity_file.clone();
                target.extra_options = host.extra_options.clone();
                target.proxy_jump = host.proxy_jump.clone();
                // 入参带转发才替换（config 导入无转发 → 保留原有）。
                if !host.forwards.is_empty() {
                    target.forwards = host.forwards.clone();
                }
                target.updated_at = now;
                overwritten += 1;
            } else {
                skipped += 1;
            }
        } else {
            // 新主机：分配新 id，避免与现有/彼此撞 id。
            host.id = Uuid::new_v4().to_string();
            host.updated_at = now;
            existing.push(host);
            added += 1;
        }
    }

    write_json(state.hosts_path(), &*existing)?;
    drop(existing);
    state.add_log(
        "info",
        format!("hosts imported: +{added} ~{overwritten} skip{skipped}"),
        Some(&app),
    );
    Ok(ImportResult {
        status: "done".to_string(),
        duplicates: Vec::new(),
        added,
        overwritten,
        skipped,
    })
}

fn selected_hosts(state: &AppState, host_ids: &[String]) -> Result<Vec<Host>, String> {
    let hosts = state.hosts.lock().map_err(lock_error)?;
    Ok(hosts
        .iter()
        .filter(|host| host_ids.iter().any(|id| id == &host.id))
        .cloned()
        .collect())
}

/// 导出选中主机到指定文件（含转发与端口，可再次导入）。取消返回 false。
#[tauri::command]
pub fn export_hosts_to_file(
    host_ids: Vec<String>,
    state: State<AppState>,
    app: AppHandle,
) -> Result<bool, String> {
    let hosts = selected_hosts(state.inner(), &host_ids)?;
    let Some(path) = FileDialog::new()
        .add_filter("SSH Port Forwarder", &["json"])
        .set_file_name("ssh-port-forwarder-hosts.json")
        .save_file()
    else {
        return Ok(false);
    };
    let bundle = ExportBundle {
        version: env!("CARGO_PKG_VERSION").to_string(),
        hosts,
    };
    let json = serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?;
    fs::write(&path, json).map_err(|err| err.to_string())?;
    state.add_log("info", format!("hosts exported to {}", path.display()), Some(&app));
    Ok(true)
}

/// 导出选中主机到本机 `~/.ssh/config` 的托管区块（只写 ssh 可解析的部分）。
/// 与导入一致地按 IP 查重：重复指与用户自己写的条目（托管区块以外）撞 IP。
/// strategy: "" 探测冲突、"overwrite" 全部写入、"skip" 仅写不重复的。
#[tauri::command]
pub fn export_hosts_to_ssh_config(
    host_ids: Vec<String>,
    strategy: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<ImportResult, String> {
    let hosts = selected_hosts(state.inner(), &host_ids)?;
    let path = ssh_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let existing = fs::read_to_string(&path).unwrap_or_default();
    // 托管区块由本程序整体重写，不算冲突；只跟区块以外的用户条目查重。
    let outside_ips: std::collections::HashSet<String> = parse_ssh_config(&strip_managed_block(&existing))
        .iter()
        .map(dedup_key)
        .filter(|key| !key.is_empty())
        .collect();

    let duplicates: Vec<String> = hosts
        .iter()
        .filter(|host| outside_ips.contains(&dedup_key(host)))
        .map(|host| host.name.clone())
        .collect();

    if strategy.is_empty() && !duplicates.is_empty() {
        return Ok(ImportResult {
            status: "conflict".to_string(),
            duplicates,
            added: 0,
            overwritten: 0,
            skipped: 0,
        });
    }

    let mut to_write: Vec<Host> = Vec::new();
    let mut added = 0;
    let mut overwritten = 0;
    let mut skipped = 0;
    for host in &hosts {
        if outside_ips.contains(&dedup_key(host)) {
            if strategy == "overwrite" {
                to_write.push(host.clone());
                overwritten += 1;
            } else {
                skipped += 1;
            }
        } else {
            to_write.push(host.clone());
            added += 1;
        }
    }

    let block = render_managed_block(&to_write);
    let updated = upsert_managed_block(&existing, &block);
    fs::write(&path, updated).map_err(|err| err.to_string())?;
    state.add_log(
        "info",
        format!("hosts exported to ssh config: +{added} ~{overwritten} skip{skipped}"),
        Some(&app),
    );
    Ok(ImportResult {
        status: "done".to_string(),
        duplicates: Vec::new(),
        added,
        overwritten,
        skipped,
    })
}
