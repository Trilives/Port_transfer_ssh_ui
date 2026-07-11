use tauri::{AppHandle, Manager, Window};

/// Hide the window to the system tray (used by the close/minimize prompt's "Minimize" choice).
#[tauri::command]
pub fn hide_to_tray(window: Window) -> Result<(), String> {
    window.hide().map_err(|err| err.to_string())
}

/// Fully quit the app (used by the close prompt's "Close" choice). Running forwards are cleaned up on exit.
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// Show and focus the main window (used by the tray).
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
