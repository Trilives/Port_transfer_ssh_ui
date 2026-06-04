use chrono::Local;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::Duration,
};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TunnelProfile {
    id: String,
    name: String,
    mode: TunnelMode,
    ssh_host: String,
    ssh_port: String,
    ssh_user: String,
    identity_file: String,
    bind_host: String,
    local_port: String,
    remote_host: String,
    remote_port: String,
    extra_options: String,
    keep_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TunnelProfileView {
    #[serde(flatten)]
    profile: TunnelProfile,
    status: TunnelStatus,
    bind_display: String,
    target_display: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TunnelMode {
    Local,
    Remote,
    Dynamic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TunnelStatus {
    Running,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    theme: String,
    language: String,
    log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogEntry {
    level: String,
    message: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CriticalErrorPayload {
    id: String,
    name: String,
    message: String,
}

struct ManagedTunnel {
    profile: TunnelProfile,
    child: Option<Child>,
    stop_requested: bool,
    password: Option<String>,
}

impl Drop for ManagedTunnel {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

struct AppState {
    profiles: Mutex<Vec<TunnelProfile>>,
    settings: Mutex<AppSettings>,
    tunnels: Mutex<HashMap<String, ManagedTunnel>>,
    logs: Mutex<Vec<LogEntry>>,
    data_dir: PathBuf,
}

impl Default for TunnelProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "new tunnel".to_string(),
            mode: TunnelMode::Local,
            ssh_host: String::new(),
            ssh_port: "22".to_string(),
            ssh_user: String::new(),
            identity_file: String::new(),
            bind_host: "127.0.0.1".to_string(),
            local_port: String::new(),
            remote_host: "127.0.0.1".to_string(),
            remote_port: String::new(),
            extra_options: String::new(),
            keep_connected: true,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "zh-CN".to_string(),
            log_level: "info".to_string(),
        }
    }
}

impl TunnelProfile {
    fn bind_display(&self) -> String {
        format!("{}:{}", empty_default(&self.bind_host, "127.0.0.1"), self.local_port)
    }

    fn target_display(&self) -> String {
        if self.mode == TunnelMode::Dynamic {
            "SOCKS proxy".to_string()
        } else {
            format!("{}:{}", empty_default(&self.remote_host, "127.0.0.1"), self.remote_port)
        }
    }
}

impl AppState {
    fn new() -> Self {
        let data_dir = ProjectDirs::from("com", "codex", "ssh-port-forwarder")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".data"));
        let _ = fs::create_dir_all(&data_dir);
        Self {
            profiles: Mutex::new(read_json(data_dir.join("profiles.json")).unwrap_or_default()),
            settings: Mutex::new(read_json(data_dir.join("settings.json")).unwrap_or_default()),
            tunnels: Mutex::new(HashMap::new()),
            logs: Mutex::new(Vec::new()),
            data_dir,
        }
    }

    fn profiles_path(&self) -> PathBuf {
        self.data_dir.join("profiles.json")
    }

    fn settings_path(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    fn add_log(&self, level: &str, message: impl Into<String>, app: Option<&AppHandle>) {
        let entry = LogEntry {
            level: level.to_string(),
            message: message.into(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        let _ = fs::create_dir_all(self.logs_dir());
        let file_path = self.logs_dir().join(format!("{}.log", Local::now().format("%Y-%m-%d")));
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .and_then(|mut file| {
                use std::io::Write;
                writeln!(file, "{} [{}] {}", entry.timestamp, entry.level.to_uppercase(), entry.message)
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

    fn status_for(&self, id: &str) -> TunnelStatus {
        let mut tunnels = self.tunnels.lock().unwrap();
        if let Some(tunnel) = tunnels.get_mut(id) {
            if let Some(child) = tunnel.child.as_mut() {
                if child.try_wait().ok().flatten().is_none() {
                    return TunnelStatus::Running;
                }
            }
        }
        TunnelStatus::Stopped
    }

    fn view(&self, profile: TunnelProfile) -> TunnelProfileView {
        TunnelProfileView {
            status: self.status_for(&profile.id),
            bind_display: profile.bind_display(),
            target_display: profile.target_display(),
            profile,
        }
    }
}

#[tauri::command]
fn list_profiles(state: State<AppState>) -> Result<Vec<TunnelProfileView>, String> {
    let profiles = state.profiles.lock().map_err(lock_error)?.clone();
    Ok(profiles.into_iter().map(|profile| state.view(profile)).collect())
}

#[tauri::command]
fn save_profile(mut profile: TunnelProfile, state: State<AppState>, app: AppHandle) -> Result<TunnelProfileView, String> {
    validate_profile(&profile)?;
    if profile.id.trim().is_empty() {
        profile.id = Uuid::new_v4().to_string();
    }

    let mut profiles = state.profiles.lock().map_err(lock_error)?;
    if let Some(existing) = profiles.iter_mut().find(|item| item.id == profile.id) {
        *existing = profile.clone();
    } else {
        profiles.push(profile.clone());
    }
    write_json(state.profiles_path(), &*profiles)?;
    drop(profiles);
    state.add_log("info", format!("[{}] saved", profile.name), Some(&app));
    Ok(state.view(profile))
}

#[tauri::command]
fn delete_profile(id: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    if matches!(state.status_for(&id), TunnelStatus::Running) {
        return Err("Disconnect this tunnel before deleting it.".to_string());
    }
    let mut profiles = state.profiles.lock().map_err(lock_error)?;
    let name = profiles
        .iter()
        .find(|item| item.id == id)
        .map(|item| item.name.clone())
        .unwrap_or_else(|| id.clone());
    profiles.retain(|item| item.id != id);
    write_json(state.profiles_path(), &*profiles)?;
    drop(profiles);
    state.add_log("info", format!("[{}] deleted", name), Some(&app));
    Ok(())
}

#[tauri::command]
fn connect_profile(id: String, state: State<AppState>, app: AppHandle) -> Result<TunnelProfileView, String> {
    let profile = state
        .profiles
        .lock()
        .map_err(lock_error)?
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| "Profile not found.".to_string())?;
    validate_profile(&profile)?;
    start_tunnel(profile.clone(), state.inner(), app, None)?;
    Ok(state.view(profile))
}

#[tauri::command]
fn connect_profile_with_password(
    id: String,
    password: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TunnelProfileView, String> {
    if password.is_empty() {
        return Err("Password is required.".to_string());
    }
    let profile = state
        .profiles
        .lock()
        .map_err(lock_error)?
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| "Profile not found.".to_string())?;
    validate_profile(&profile)?;
    start_tunnel(profile.clone(), state.inner(), app, Some(password))?;
    Ok(state.view(profile))
}

#[tauri::command]
fn upload_public_key(
    id: String,
    password: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TunnelProfileView, String> {
    if password.is_empty() {
        return Err("Password is required.".to_string());
    }
    let profile = state
        .profiles
        .lock()
        .map_err(lock_error)?
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| "Profile not found.".to_string())?;
    validate_ssh_profile(&profile)?;
    let public_key = ensure_public_key(&profile, &app)?;
    upload_key_to_remote(&profile, &public_key, &password, state.inner(), &app)?;
    state.add_log("info", format!("[{}] public key uploaded", profile.name), Some(&app));
    Ok(state.view(profile))
}

#[tauri::command]
fn probe_connection(id: String, state: State<AppState>, app: AppHandle) -> Result<String, String> {
    let profile = state
        .profiles
        .lock()
        .map_err(lock_error)?
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| "Profile not found.".to_string())?;
    validate_ssh_profile(&profile)?;

    let command = build_probe_command(&profile);
    state.add_log("debug", format!("[{}] probe $ {}", profile.name, command.join(" ")), Some(&app));
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    process.creation_flags(CREATE_NO_WINDOW);
    let output = process.output().map_err(|err| err.to_string())?;
    if output.status.success() {
        return Ok("ready".to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let lower = stderr.to_lowercase();
    if lower.contains("remote host identification has changed")
        || lower.contains("host key verification failed")
    {
        return Ok("host_key_changed".to_string());
    }
    if lower.contains("permission denied")
        || lower.contains("publickey")
        || lower.contains("no supported authentication")
    {
        return Ok("password_required".to_string());
    }
    Err(format!("Cannot reach host: {}", stderr.trim()))
}

#[tauri::command]
fn get_host_fingerprint(id: String, state: State<AppState>, _app: AppHandle) -> Result<String, String> {
    let profile = state
        .profiles
        .lock()
        .map_err(lock_error)?
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| "Profile not found.".to_string())?;
    validate_ssh_profile(&profile)?;
    let port = empty_default(&profile.ssh_port, "22");
    let host = profile.ssh_host.trim();

    let mut scan = Command::new("ssh-keyscan");
    scan.args(["-p", port, host])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(target_os = "windows")]
    scan.creation_flags(CREATE_NO_WINDOW);
    let scan_output = scan.output().map_err(|err| err.to_string())?;
    if scan_output.stdout.is_empty() {
        return Err("Cannot fetch host key.".to_string());
    }

    let mut keygen = Command::new("ssh-keygen");
    keygen
        .args(["-l", "-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    keygen.creation_flags(CREATE_NO_WINDOW);
    let mut child = keygen.spawn().map_err(|err| err.to_string())?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(&scan_output.stdout).map_err(|err| err.to_string())?;
    }
    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    let fingerprint = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fingerprint.is_empty() {
        return Err("Cannot compute host key fingerprint.".to_string());
    }
    Ok(fingerprint)
}

#[tauri::command]
fn remove_known_host(id: String, state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let profile = state
        .profiles
        .lock()
        .map_err(lock_error)?
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| "Profile not found.".to_string())?;
    let host = profile.ssh_host.trim().to_string();
    if host.is_empty() {
        return Err("SSH host is required.".to_string());
    }
    let port = empty_default(&profile.ssh_port, "22").to_string();
    let mut targets = vec![host.clone()];
    if port != "22" {
        targets.push(format!("[{host}]:{port}"));
    }
    for target in targets {
        let mut keygen = Command::new("ssh-keygen");
        keygen
            .args(["-R", &target])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        keygen.creation_flags(CREATE_NO_WINDOW);
        match keygen.output() {
            Ok(_) => state.add_log(
                "info",
                format!("[{}] removed old host key for {}", profile.name, target),
                Some(&app),
            ),
            Err(err) => state.add_log(
                "warning",
                format!("[{}] ssh-keygen -R {} failed: {}", profile.name, target, err),
                Some(&app),
            ),
        }
    }
    Ok(())
}

#[tauri::command]
fn disconnect_profile(id: String, state: State<AppState>, app: AppHandle) -> Result<TunnelProfileView, String> {
    let profile = state
        .profiles
        .lock()
        .map_err(lock_error)?
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| "Profile not found.".to_string())?;
    stop_tunnel(&id, state.inner(), Some(&app))?;
    Ok(state.view(profile))
}

#[tauri::command]
fn disconnect_all(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let ids: Vec<String> = state.tunnels.lock().map_err(lock_error)?.keys().cloned().collect();
    for id in ids {
        stop_tunnel(&id, state.inner(), Some(&app))?;
    }
    Ok(())
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state.settings.lock().map_err(lock_error).map(|settings| settings.clone())
}

#[tauri::command]
fn save_settings_cmd(settings: AppSettings, state: State<AppState>, app: AppHandle) -> Result<AppSettings, String> {
    let normalized = AppSettings {
        theme: normalize_choice(&settings.theme, &["dark", "light"], "dark"),
        language: normalize_choice(&settings.language, &["zh-CN", "en-US"], "zh-CN"),
        log_level: normalize_choice(&settings.log_level, &["debug", "info", "warning", "error"], "info"),
    };
    *state.settings.lock().map_err(lock_error)? = normalized.clone();
    write_json(state.settings_path(), &normalized)?;
    state.add_log(
        "info",
        format!(
            "[settings] theme={} language={} logLevel={}",
            normalized.theme, normalized.language, normalized.log_level
        ),
        Some(&app),
    );
    Ok(normalized)
}

#[tauri::command]
fn list_logs(level: String, state: State<AppState>) -> Result<Vec<LogEntry>, String> {
    let min_rank = log_rank(&level);
    let logs = state.logs.lock().map_err(lock_error)?;
    Ok(logs
        .iter()
        .filter(|entry| log_rank(&entry.level) >= min_rank)
        .cloned()
        .collect())
}

fn start_tunnel(
    profile: TunnelProfile,
    state: &AppState,
    app: AppHandle,
    password: Option<String>,
) -> Result<(), String> {
    let mut tunnels = state.tunnels.lock().map_err(lock_error)?;
    if let Some(existing) = tunnels.get_mut(&profile.id) {
        if let Some(child) = existing.child.as_mut() {
            if child.try_wait().map_err(|err| err.to_string())?.is_none() {
                return Ok(());
            }
        }
    }

    let command = build_ssh_command(&profile)?;
    state.add_log("debug", format!("[{}] $ {}", profile.name, command.join(" ")), Some(&app));
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    process.creation_flags(CREATE_NO_WINDOW);
    if let Some(password_value) = password.as_ref() {
        let helper = prepare_askpass_helper(&state.data_dir)?;
        process
            .env("SSH_ASKPASS", helper)
            .env("SSH_ASKPASS_REQUIRE", "force")
            .env("DISPLAY", "ssh-port-forwarder")
            .env("SSH_PORT_FORWARDER_PASSWORD", password_value);
    }
    let child = process.spawn().map_err(|err| err.to_string())?;

    tunnels.insert(
        profile.id.clone(),
        ManagedTunnel {
            profile: profile.clone(),
            child: Some(child),
            stop_requested: false,
            password,
        },
    );
    drop(tunnels);
    state.add_log("info", format!("[{}] connected", profile.name), Some(&app));
    watch_tunnel(profile.id.clone(), state.data_dir.clone(), app);
    Ok(())
}

fn stop_tunnel(id: &str, state: &AppState, app: Option<&AppHandle>) -> Result<(), String> {
    let mut tunnels = state.tunnels.lock().map_err(lock_error)?;
    if let Some(tunnel) = tunnels.get_mut(id) {
        tunnel.stop_requested = true;
        if let Some(child) = tunnel.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        state.add_log("info", format!("[{}] disconnected", tunnel.profile.name), app);
    }
    Ok(())
}

fn cleanup_tunnels(state: &AppState, app: Option<&AppHandle>) {
    let tunnels = {
        let Ok(mut tunnels) = state.tunnels.lock() else {
            return;
        };
        tunnels.drain().map(|(_, tunnel)| tunnel).collect::<Vec<_>>()
    };

    for mut tunnel in tunnels {
        tunnel.stop_requested = true;
        if let Some(child) = tunnel.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        state.add_log("info", format!("[{}] cleaned up on exit", tunnel.profile.name), app);
    }
}

fn watch_tunnel(id: String, data_dir: PathBuf, app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));
        let state = app.state::<AppState>();
        let should_reconnect = {
            let mut tunnels = match state.tunnels.lock() {
                Ok(tunnels) => tunnels,
                Err(_) => return,
            };
            let Some(tunnel) = tunnels.get_mut(&id) else {
                return;
            };
            let Some(child) = tunnel.child.as_mut() else {
                return;
            };
            match child.try_wait() {
                Ok(None) => continue,
                Ok(Some(status)) => {
                    let profile = tunnel.profile.clone();
                    let stopped_by_user = tunnel.stop_requested;
                    let mut stderr = String::new();
                    if let Some(mut stderr_pipe) = child.stderr.take() {
                        let _ = stderr_pipe.read_to_string(&mut stderr);
                    }
                    tunnel.child = None;
                    let password = tunnel.password.clone();
                    (profile, password, stopped_by_user, status.code(), stderr)
                }
                Err(_) => return,
            }
        };

        let (profile, password, stopped_by_user, code, stderr) = should_reconnect;
        let detail = stderr.trim();
        let message = if detail.is_empty() {
            format!("[{}] exited with code {:?}", profile.name, code)
        } else {
            format!("[{}] exited with code {:?}: {}", profile.name, code, detail)
        };
        if code == Some(255) {
            state.add_log("error", message.clone(), Some(&app));
            let _ = app.emit(
                "critical-error",
                CriticalErrorPayload {
                    id: profile.id,
                    name: profile.name,
                    message,
                },
            );
            return;
        }
        state.add_log("warning", message, Some(&app));
        if stopped_by_user || !profile.keep_connected {
            return;
        }
        thread::sleep(Duration::from_secs(3));
        let _ = fs::create_dir_all(&data_dir);
        if start_tunnel(profile, state.inner(), app.clone(), password).is_err() {
            return;
        }
        return;
    });
}

fn prepare_askpass_helper(data_dir: &PathBuf) -> Result<PathBuf, String> {
    let helper_dir = data_dir.join("helpers");
    fs::create_dir_all(&helper_dir).map_err(|err| err.to_string())?;
    let helper = helper_dir.join("ssh-port-forwarder-askpass.exe");
    let current_exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let should_copy = match (fs::metadata(&current_exe), fs::metadata(&helper)) {
        (Ok(current), Ok(existing)) => current.len() != existing.len(),
        (Ok(_), Err(_)) => true,
        _ => true,
    };
    if should_copy {
        fs::copy(current_exe, &helper).map_err(|err| err.to_string())?;
    }
    Ok(helper)
}

fn ensure_public_key(profile: &TunnelProfile, app: &AppHandle) -> Result<String, String> {
    let private_key = resolve_identity_file(&profile.identity_file)?;
    let public_key = PathBuf::from(format!("{}.pub", private_key.to_string_lossy()));
    if !private_key.exists() {
        if let Some(parent) = private_key.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let mut keygen = Command::new("ssh-keygen");
        keygen
            .args([
                "-t",
                "ed25519",
                "-N",
                "",
                "-C",
                "ssh-port-forwarder",
                "-f",
                private_key
                    .to_str()
                    .ok_or_else(|| "Identity file path contains invalid characters.".to_string())?,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        keygen.creation_flags(CREATE_NO_WINDOW);
        let output = keygen.output().map_err(|err| err.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "ssh-keygen failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let state = app.state::<AppState>();
        state.add_log(
            "info",
            format!("[key] generated {}", private_key.display()),
            Some(app),
        );
    }

    if !public_key.exists() {
        let mut derive = Command::new("ssh-keygen");
        derive
            .args([
                "-y",
                "-f",
                private_key
                    .to_str()
                    .ok_or_else(|| "Identity file path contains invalid characters.".to_string())?,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        derive.creation_flags(CREATE_NO_WINDOW);
        let output = derive.output().map_err(|err| err.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "Cannot derive public key: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        fs::write(&public_key, output.stdout).map_err(|err| err.to_string())?;
    }

    fs::read_to_string(&public_key)
        .map(|value| value.trim().to_string())
        .map_err(|err| err.to_string())
}

fn resolve_identity_file(identity_file: &str) -> Result<PathBuf, String> {
    let trimmed = identity_file.trim();
    if trimmed.is_empty() {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map_err(|_| "Cannot locate user home directory.".to_string())?;
        return Ok(PathBuf::from(home).join(".ssh").join("id_ed25519"));
    }

    if let Some(rest) = trimmed.strip_prefix("~/").or_else(|| trimmed.strip_prefix("~\\")) {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map_err(|_| "Cannot locate user home directory.".to_string())?;
        return Ok(PathBuf::from(home).join(rest));
    }

    Ok(PathBuf::from(trimmed))
}

fn upload_key_to_remote(
    profile: &TunnelProfile,
    public_key: &str,
    password: &str,
    state: &AppState,
    app: &AppHandle,
) -> Result<(), String> {
    let command = build_key_upload_command(profile)?;
    state.add_log("debug", format!("[{}] $ {}", profile.name, command.join(" ")), Some(app));
    let helper = prepare_askpass_helper(&state.data_dir)?;
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .env("SSH_ASKPASS", helper)
        .env("SSH_ASKPASS_REQUIRE", "force")
        .env("DISPLAY", "ssh-port-forwarder")
        .env("SSH_PORT_FORWARDER_PASSWORD", password)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    process.creation_flags(CREATE_NO_WINDOW);
    let mut child = process.spawn().map_err(|err| err.to_string())?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(format!("{public_key}\n").as_bytes())
            .map_err(|err| err.to_string())?;
    }
    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Upload public key failed: {}", stderr.trim()));
    }
    Ok(())
}

fn build_key_upload_command(profile: &TunnelProfile) -> Result<Vec<String>, String> {
    let destination = if profile.ssh_user.trim().is_empty() {
        profile.ssh_host.trim().to_string()
    } else {
        format!("{}@{}", profile.ssh_user.trim(), profile.ssh_host.trim())
    };
    let mut command = vec![
        "ssh".to_string(),
        "-T".to_string(),
        "-o".to_string(),
        "BatchMode=no".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(),
        empty_default(&profile.ssh_port, "22").to_string(),
    ];
    if !profile.identity_file.trim().is_empty() {
        command.extend(["-i".to_string(), profile.identity_file.trim().to_string()]);
    }
    command.push(destination);
    command.push(
        "umask 077; mkdir -p ~/.ssh && touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && key=$(cat) && { grep -qxF \"$key\" ~/.ssh/authorized_keys || printf '%s\\n' \"$key\" >> ~/.ssh/authorized_keys; }"
            .to_string(),
    );
    Ok(command)
}

fn build_probe_command(profile: &TunnelProfile) -> Vec<String> {
    let destination = if profile.ssh_user.trim().is_empty() {
        profile.ssh_host.trim().to_string()
    } else {
        format!("{}@{}", profile.ssh_user.trim(), profile.ssh_host.trim())
    };
    let mut command = vec![
        "ssh".to_string(),
        "-T".to_string(),
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "ConnectTimeout=8".to_string(),
        "-o".to_string(),
        "NumberOfPasswordPrompts=0".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(),
        empty_default(&profile.ssh_port, "22").to_string(),
    ];
    if !profile.identity_file.trim().is_empty() {
        command.extend(["-i".to_string(), profile.identity_file.trim().to_string()]);
    }
    command.push(destination);
    command.push("exit 0".to_string());
    command
}

fn build_ssh_command(profile: &TunnelProfile) -> Result<Vec<String>, String> {
    let ssh = "ssh".to_string();
    let destination = if profile.ssh_user.trim().is_empty() {
        profile.ssh_host.trim().to_string()
    } else {
        format!("{}@{}", profile.ssh_user.trim(), profile.ssh_host.trim())
    };
    let mut command = vec![
        ssh,
        "-N".to_string(),
        "-T".to_string(),
        "-o".to_string(),
        "ExitOnForwardFailure=yes".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=30".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(),
        empty_default(&profile.ssh_port, "22").to_string(),
    ];
    if !profile.identity_file.trim().is_empty() {
        command.extend(["-i".to_string(), profile.identity_file.trim().to_string()]);
    }
    if !profile.extra_options.trim().is_empty() {
        command.extend(shell_words::split(&profile.extra_options).map_err(|err| err.to_string())?);
    }
    let bind = empty_default(&profile.bind_host, "127.0.0.1");
    match profile.mode {
        TunnelMode::Local => command.extend([
            "-L".to_string(),
            format!(
                "{}:{}:{}:{}",
                bind,
                profile.local_port.trim(),
                empty_default(&profile.remote_host, "127.0.0.1"),
                profile.remote_port.trim()
            ),
        ]),
        TunnelMode::Remote => command.extend([
            "-R".to_string(),
            format!(
                "{}:{}:{}:{}",
                bind,
                profile.local_port.trim(),
                empty_default(&profile.remote_host, "127.0.0.1"),
                profile.remote_port.trim()
            ),
        ]),
        TunnelMode::Dynamic => command.extend(["-D".to_string(), format!("{}:{}", bind, profile.local_port.trim())]),
    }
    command.push(destination);
    Ok(command)
}

fn validate_ssh_profile(profile: &TunnelProfile) -> Result<(), String> {
    let mut errors = Vec::new();
    if profile.ssh_host.trim().is_empty() {
        errors.push("SSH host is required.");
    }
    if profile.ssh_port.parse::<u16>().is_err() {
        errors.push("SSH port must be a valid port.");
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(())
}

fn validate_profile(profile: &TunnelProfile) -> Result<(), String> {
    let mut errors = Vec::new();
    if profile.name.trim().is_empty() {
        errors.push("Name is required.");
    }
    if profile.ssh_host.trim().is_empty() {
        errors.push("SSH host is required.");
    }
    if profile.ssh_port.parse::<u16>().is_err() {
        errors.push("SSH port must be a valid port.");
    }
    if profile.local_port.parse::<u16>().is_err() {
        errors.push("Bind port must be a valid port.");
    }
    if profile.mode != TunnelMode::Dynamic {
        if profile.remote_host.trim().is_empty() {
            errors.push("Target host is required.");
        }
        if profile.remote_port.parse::<u16>().is_err() {
            errors.push("Target port must be a valid port.");
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: PathBuf) -> Result<T, String> {
    if !path.exists() {
        return Err("missing".to_string());
    }
    serde_json::from_str(&fs::read_to_string(path).map_err(|err| err.to_string())?).map_err(|err| err.to_string())
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, serde_json::to_string_pretty(value).map_err(|err| err.to_string())?).map_err(|err| err.to_string())
}

fn empty_default<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value.trim()
    }
}

fn normalize_choice(value: &str, allowed: &[&str], fallback: &str) -> String {
    if allowed.contains(&value) {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

fn log_rank(level: &str) -> usize {
    match level {
        "debug" => 0,
        "info" => 1,
        "warning" => 2,
        "error" => 3,
        _ => 1,
    }
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> String {
    "Application state lock failed.".to_string()
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            save_profile,
            delete_profile,
            connect_profile,
            connect_profile_with_password,
            upload_public_key,
            probe_connection,
            get_host_fingerprint,
            remove_known_host,
            disconnect_profile,
            disconnect_all,
            get_settings,
            save_settings_cmd,
            list_logs
        ])
        .setup(|app| {
            let state = app.state::<AppState>();
            let handle = app.handle().clone();
            state.add_log("info", "application started", Some(&handle));
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let state = app_handle.state::<AppState>();
                cleanup_tunnels(state.inner(), Some(app_handle));
            }
        });
}
