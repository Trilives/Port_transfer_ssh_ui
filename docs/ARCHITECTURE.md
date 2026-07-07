# SSH Port Forwarder Architecture

Read this to get oriented before touching the code. For features and getting started as a user, see [../README.md](../README.md);
for when/how to split a file into a new module, see [MODULARITY.md](MODULARITY.md).

## 1. Overview

The core object is a **host-centric, two-level structure**:

- **Level 1 = Host**: an SSH server, storing connection parameters (host, port, user, key).
- **Level 2 = Forward**: a port forward nested under a host, storing only the forwarding parameters (mode, bind, target).

The host is a UI/data "object"; there is no reusable, persistent primary SSH connection (Windows' bundled OpenSSH doesn't
support ControlMaster multiplexing). What actually persists is each forward's own `ssh.exe` child process. "Send command",
"Open terminal", and "Upload key" each make a one-off `ssh` call as needed, reusing the host's connection parameters.

```mermaid
flowchart TB
  subgraph FE[React Frontend]
    Dash[Home/Dashboard] --> Cur[Current connections + key events]
    Cfg[Config page] --> HostCard --> ForwardRow
    HostCard --> Dlg[Dialogs: Host/Forward/SendCmd/Password/KeyUpload/HostKey]
    LogsPage[Logs page]
    SetPage[Settings page]
    GuidePage[Guide page]
    api[api.ts] -. invoke / listen .- FE
  end
  subgraph BE[Tauri 2 + Rust]
    cmds[commands/*] --> model[model + store: hosts.json]
    cmds --> ssh[ssh: command/process/probe/keys]
    cmds --> port[portcheck]
    cmds --> term[terminal: external PowerShell]
    ssh --> sshexe[(ssh.exe forward child process)]
    term --> ps[(powershell -NoExit ssh ...)]
  end
  api == Tauri IPC ==> cmds
  cmds -. emit log-entry / critical-error .-> api
```

## 2. Tech Stack

- Desktop app shell and native backend: Tauri 2 + Rust
- Frontend UI: React + TypeScript + Tailwind CSS, local shadcn/ui-style components
- Forward execution: Rust calls the system `ssh`; the external terminal opens a native console (PowerShell on Windows,
  a detected terminal emulator on Linux) running interactive `ssh`
- Packaging targets: Windows (NSIS/MSI) and Linux (deb/AppImage/rpm), built by `.github/workflows/build.yml`.
  Platform-specific backend paths are gated with `#[cfg(target_os = ...)]`

## 3. Getting Started

Requires Node.js, npm, the Rust stable toolchain, and the Windows OpenSSH Client. If PowerShell doesn't recognize `cargo`/`rustc`,
add `%USERPROFILE%\.cargo\bin` to PATH.

```powershell
npm.cmd --prefix frontend install   # install deps (from project root)
npm.cmd run tauri:dev               # run the dev build
cargo check --manifest-path src-tauri\Cargo.toml   # rust check
npm.cmd run build                   # build the frontend only
```

Packaging must run from the **project root** (so the Tauri CLI finds `src-tauri/tauri.conf.json`); it builds the frontend
automatically via `beforeBuildCommand`:

```powershell
.\frontend\node_modules\.bin\tauri.cmd build
```

Output: `src-tauri\target\release\ssh-port-forwarder.exe` and an NSIS installer under `src-tauri\target\release\bundle\nsis\`.

## 4. Data Model

```text
Host  (level 1 = SSH server)
├─ id, name, sshHost, sshPort (default 22), sshUser
├─ identityFile     // private key path; empty uses default id_ed25519
├─ extraOptions     // extra args applied to every ssh operation for this host
├─ proxyJump        // jump host (ProxyJump), optional; adds -J on connect
├─ pinned           // pinned hosts sort first
├─ updatedAt        // Unix ms; list sorts by this, newest first
└─ forwards: Vec<Forward>

Forward (level 2 = a single port forward)
├─ id, name, mode: local | remote | dynamic
├─ bindHost (default 127.0.0.1), bindPort
├─ targetHost (default 127.0.0.1), targetPort   // not needed for dynamic
└─ keepConnected    // default true; auto-reconnect after disconnect
```

- Persistence file: `hosts.json` (forwards nested inside), in the system app-data directory.
- The v0.1.x `profiles.json` **is not migrated**; if present, it's backed up to `profiles.json.v0.1.bak` on startup.
- Runtime state: `hosts: Mutex<Vec<Host>>`, `tunnels: Mutex<HashMap<ForwardId, ManagedTunnel>>` (one child process per running
  forward, with a snapshot of the parent host for auto-reconnect), plus `settings` and `logs`.

## 5. Module Layout

### Backend `src-tauri/src/`

```text
main.rs            entry point + askpass helper
lib.rs             run(), Tauri Builder, command registration, AppState definition and wiring
model.rs           Host / Forward / enums / AppSettings / LogEntry / View / Default
store.rs           read_json / write_json, data directory path, AppState persistence and log writes
validate.rs        validate_host / validate_forward
portcheck.rs       local port-in-use detection + owning process name/PID (netstat + tasklist)
sshconfig.rs       parse/write ~/.ssh/config (import hosts, export to a managed block, look up alias by IP)
vscode.rs          VS Code Remote-SSH: detect install, read history, match by IP, write config, open folder-uri
terminal.rs        open_terminal: launches an external PowerShell window running ssh
ssh/
  command.rs       build_ssh / build_probe / build_key_upload / build_send command construction
  diagnose.rs      classify_ssh_failure: classify unreachable/auth/etc. failures from stderr, with localized reasons
  process.rs       start/stop/watch/cleanup tunnel, CREATE_NO_WINDOW, askpass helper
  probe.rs         probe_connection / get_host_fingerprint / remove_known_host
  keys.rs          ensure_public_key / resolve_identity_file / upload_key_to_remote
commands/          thin #[tauri::command] wrappers, one file per domain:
  hosts.rs forwards.rs exec.rs keys.rs settings.rs transfer.rs vscode.rs system.rs
```

### Frontend `frontend/src/`

```text
App.tsx              shell: navigation, page routing, data load/refresh, all action/dialog orchestration, theme
types.ts  api.ts  i18n.ts    shared types, IPC calls, UI copy (zh-CN/en-US)
pages/               DashboardPage ConfigPage LogsPage SettingsPage GuidePage
components/          HostCard ForwardRow StatusBadge LogTable dialogs.tsx ui/*
```

Data loading, polling, and all dialog state are orchestrated centrally in `App.tsx`; pages and components stay purely
presentational (receiving data and callbacks via props). See [MODULARITY.md](MODULARITY.md) for the layering rules behind this split.

## 6. Tauri Commands

| Domain | Commands | Notes |
| --- | --- | --- |
| Host | `list_hosts` `save_host` `set_host_pinned` `delete_host` | no parameter validation on save; usability is checked at connect time; `delete_host` disconnects its forwards first |
| Import/export | `read_import_file` `read_import_ssh_config` `import_hosts` `export_hosts_to_file` `export_hosts_to_ssh_config` | dedup by sshHost/IP; strategy `""` detects conflicts, `overwrite`, or `skip` |
| Forward | `save_forward` `delete_forward` `connect_forward[_with_password]` `disconnect_forward` `disconnect_host` `disconnect_all` | connect probes first; port-in-use is only checked at connect time |
| Command/Terminal | `send_command` `open_terminal` `open_url` | `open_url` opens http/https in the default browser (the "Open in browser" button) |
| Keys/Probe | `upload_public_key` `probe_connection` `get_host_fingerprint` `remove_known_host` | `probe_connection` returns `ready｜password_required｜host_key_changed`, or an Err with a reason for unreachable hosts |
| Settings/Logs | `get_settings` `save_settings_cmd` `list_logs` | default theme is light |
| System | `check_ssh` `install_openssh` | installs OpenSSH via elevated `Add-WindowsCapability` |
| VS Code | `vscode_status` `vscode_ssh_history` `vscode_open` `vscode_open_direct` `vscode_open_path` | history matched by host IP via `~/.ssh/config`; see `vscode.rs` for the state.vscdb/storage.json merge logic |

Events: `log-entry` (one log line) and `critical-error` (forward exited with code 255; payload has `hostId/forwardId/name/message`, frontend shows one dialog and stops auto-reconnect).

## 7. Key Flows

- **Automatic local port fallback** (local/dynamic only): `start_tunnel` calls `find_free_port`, incrementing from the
  configured `bindPort` up to 200 times. `detect_conflict` checks both this app's other running forwards and the OS
  (`netstat` + `tasklist`, ignoring this app's own `ssh.exe` PIDs). The actual port used overrides `bindPort`/`bindDisplay` in
  the view, so Home, the forward row, and "Open in browser" always point at what's really listening. Remote mode listens on
  the remote side, so no local check applies.
- **Connect / auto-reconnect**: `connect_forward` builds `ssh -N -T -L/-R/-D … user@host` and runs it with `CREATE_NO_WINDOW`;
  `watch_tunnel` polls the child and reconnects after a delay if `keepConnected` is on and the exit wasn't fatal (exit code 255
  → log + `critical-error`, no retry). Editing a running host/forward (`restart_forward`) reuses the cached password and does
  a stop-then-start.
- **One-time password connect**: frontend calls `probe_connection` first — `ready` connects directly, `password_required`
  prompts and calls `connect_forward_with_password` (password injected via `SSH_ASKPASS`, never written to disk, kept in
  memory only if `keepConnected`), `host_key_changed` shows the fingerprint and offers `remove_known_host` + retry.
- **External terminal**: `open_terminal` opens a visible, independent PowerShell window running interactive `ssh`; it's not
  tracked or cleaned up by the app.
- **Open in VS Code**: `vscode_ssh_history` merges Remote-SSH's `state.vscdb` (preferred, freshest) with `storage.json`
  (fallback) and matches folders to the host by IP. "Direct" (`vscode_open_direct`) reuses or writes a `~/.ssh/config` alias
  and opens a connected window with no folder; a specific path (`vscode_open_path`) resolves `~`/relative paths against the
  remote `$HOME` via a one-time passwordless probe.

On app exit, all forward child processes started by this app are cleaned up.

## 8. Data Storage

The backend uses `directories::ProjectDirs` for the system app-data directory, writing `hosts.json`, `settings.json`, and
`logs/` there — never into the source tree or the program's own folder.
