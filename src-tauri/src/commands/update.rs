//! Channel-aware in-app updater. The JS `check()` can only read the single static endpoint from
//! `tauri.conf.json`, so channel selection is done here in Rust by building the updater with the
//! endpoint for the requested channel. Signatures are still verified against the configured `pubkey`.

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// Stable: GitHub's "latest" (non-prerelease) release manifest. Keep in sync with `tauri.conf.json`.
const STABLE_ENDPOINT: &str =
    "https://github.com/Trilives/sshdeck/releases/latest/download/latest.json";
/// Preview: a rolling `preview` pre-release whose `latest.json` CI refreshes on every beta tag.
const PREVIEW_ENDPOINT: &str =
    "https://github.com/Trilives/sshdeck/releases/download/preview/latest.json";

/// The update available on a channel (mirrors the JS updater's `version`/`body`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
}

fn endpoint_for(channel: &str) -> &'static str {
    if channel == "preview" {
        PREVIEW_ENDPOINT
    } else {
        STABLE_ENDPOINT
    }
}

/// Build the updater pointed at the channel's endpoint and check for an update.
async fn check(app: &AppHandle, channel: &str) -> Result<Option<tauri_plugin_updater::Update>, String> {
    let endpoint = endpoint_for(channel);
    app.updater_builder()
        .endpoints(vec![endpoint.parse().map_err(|e| format!("Invalid update endpoint: {e}"))?])
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())
}

/// Check the given channel for a newer signed release, returning its version/notes when one exists.
#[tauri::command]
pub async fn check_update(channel: String, app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    Ok(check(&app, &channel).await?.map(|update| UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone().unwrap_or_default(),
    }))
}

/// Download and install the newest release on the given channel, then relaunch. No-op if none is available.
#[tauri::command]
pub async fn install_update(channel: String, app: AppHandle) -> Result<(), String> {
    let Some(update) = check(&app, &channel).await? else {
        return Ok(());
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    app.restart();
}
