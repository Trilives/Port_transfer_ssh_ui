mod commands;
mod model;
mod portcheck;
mod ssh;
mod state;
mod store;
mod terminal;
mod util;
mod validate;

use tauri::Manager;

use crate::ssh::process::cleanup_tunnels;
use crate::state::AppState;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::hosts::list_hosts,
            commands::hosts::save_host,
            commands::hosts::delete_host,
            commands::forwards::save_forward,
            commands::forwards::delete_forward,
            commands::forwards::connect_forward,
            commands::forwards::connect_forward_with_password,
            commands::forwards::disconnect_forward,
            commands::forwards::disconnect_host,
            commands::forwards::disconnect_all,
            commands::exec::send_command,
            commands::exec::send_command_with_password,
            commands::exec::open_terminal,
            commands::keys::upload_public_key,
            commands::keys::probe_connection,
            commands::keys::get_host_fingerprint,
            commands::keys::remove_known_host,
            commands::settings::get_settings,
            commands::settings::save_settings_cmd,
            commands::settings::list_logs,
            commands::system::check_ssh,
            commands::system::install_openssh
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
