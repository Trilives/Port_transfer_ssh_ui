#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|name| name.to_string_lossy().to_string()))
        .unwrap_or_default();
    if exe_name.contains("askpass") || std::env::args().any(|arg| arg == "--askpass") {
        if let Ok(password) = std::env::var("SSHDECK_PASSWORD") {
            println!("{password}");
        }
        return;
    }
    sshdeck_lib::run();
}
