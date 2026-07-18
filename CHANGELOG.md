# Changelog

## Unreleased

**Changed**
- Applied the native Windows Mica backdrop with a translucent app surface while retaining the standard title bar, resizing, and system window controls.

## 0.3.0-beta.5

**Fixed**
- Replaced the legacy development-era data path with `%LOCALAPPDATA%\SSHDeck\data`, independent of the source repository, developer name, and installation directory.

**Upgrade notice**
- There is no automatic migration from beta.4 or earlier. Export hosts before updating, then import them after launching beta.5. Settings and connection history start fresh in the new directory.

## 0.3.0-beta.4

**Added**
- Published a Windows install-free portable ZIP beside the installer from the same CI build.

**Changed**
- Moved Delete Host into each Remote Connections host item, keeping host lifecycle actions out of Port Forwarding.
- Refreshed the English and Chinese README preview composites after the final page-action layout changes.

## 0.3.0-beta.3

**Added**
- Dedicated **Remote Connections** page, grouped by host, with historical remote paths and Terminal / VS Code reopen actions.
- Dedicated **Port Forwarding** page, grouped by host, for Local, Remote, and Dynamic forwarding management.
- Automated Windows portable ZIP packaging, published beside the installer from the same CI build.

**Fixed**
- One-off SSH commands now run on a blocking worker instead of the Tauri IPC runtime, so the interface remains responsive while a remote command is running.
- Closing a running command dialog no longer lets its eventual result overwrite a newly opened host command dialog.
- Chinese layouts can shrink and wrap correctly in narrower windows without clipping right-side actions.
- The sidebar now stays fixed while only the right-hand page content scrolls.

**Changed**
- Moved host import/export, New Host, Upload Key, and Delete Host actions to Remote Connections; Port Forwarding now keeps only forwarding actions and adds one-click host disconnect.
- Replaced the outdated README screenshots with four current 1440×900 English/Chinese page previews and refreshed the architecture documentation.
- Split navigation, dialog orchestration, remote dialogs, and SSH execution into focused modules to keep files within the project's modularity limits.

## 0.3.0-beta.2

**Added**
- **Remote Connection**: each host now has a single "Remote Connection" button (and one icon on Home) opening a list of remote paths (from VS Code Remote-SSH history, matched by IP). Open any path in a terminal — SSH'd and `cd`'d into it — or in VS Code, or type a path manually. Enter opens a terminal, Esc closes.
- **Preview / Stable update channels**: pick which release channel the in-app updater follows in **Settings → Software Update**. Update checks and installs now run channel-aware in the backend.

**Changed**
- The per-host terminal split button and the separate "Open in VS Code" dialog are replaced by the unified Remote Connection window.

**Notes**
- Pre-release. The Preview channel starts resolving once a preview build has been published.

## 0.3.1-beta.1

**Added**
- In-app auto-update: **Settings → Software Update** checks GitHub Releases, shows the new version and its notes, and installs a newer signed build in place (then restarts). Powered by `tauri-plugin-updater` + `tauri-plugin-process`; releases are signed in CI and verified against a pinned public key.
- **Automatic updates** toggle in Settings. When on, an update found at startup installs silently; when off, a small "update available" notice appears on the main screen.

**Changed**
- Project renamed to **SSHDeck** (binary `sshdeck.exe`, product name `SSHDeck`).

**Notes**
- Pre-release. Existing settings and hosts are preserved (the app identifier and data directory are unchanged).
- Auto-update requires the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repository secrets to be set so CI can sign the release and publish `latest.json`.
