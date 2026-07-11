# Changelog

## 0.3.0-beta.1

**Added**
- In-app auto-update: **Settings → Software Update** checks GitHub Releases, shows the new version and its notes, and installs a newer signed build in place (then restarts). Powered by `tauri-plugin-updater` + `tauri-plugin-process`; releases are signed in CI and verified against a pinned public key.
- **Automatic updates** toggle in Settings. When on, an update found at startup installs silently; when off, a small "update available" notice appears on the main screen.

**Changed**
- Project renamed to **SSHDeck** (binary `sshdeck.exe`, product name `SSHDeck`).

**Notes**
- Pre-release. Existing settings and hosts are preserved (the app identifier and data directory are unchanged).
- Auto-update requires the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repository secrets to be set so CI can sign the release and publish `latest.json`.
