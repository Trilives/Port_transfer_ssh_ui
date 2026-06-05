use std::process::Command;

/// Apply `CREATE_NO_WINDOW` on Windows so spawned ssh/helper processes do not
/// pop up a console window. No-op on other platforms.
#[cfg(target_os = "windows")]
pub fn no_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
pub fn no_window(_command: &mut Command) {}

/// Trimmed value, or the fallback when the trimmed value is empty.
pub fn empty_default<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value.trim()
    }
}

pub fn normalize_choice(value: &str, allowed: &[&str], fallback: &str) -> String {
    if allowed.contains(&value) {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

pub fn log_rank(level: &str) -> usize {
    match level {
        "debug" => 0,
        "info" => 1,
        "warning" => 2,
        "error" => 3,
        _ => 1,
    }
}

pub fn lock_error<T>(_: std::sync::PoisonError<T>) -> String {
    "Application state lock failed.".to_string()
}
