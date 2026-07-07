use std::process::Command;

use crate::model::Host;
use crate::ssh::command::build_terminal_args;

/// Open an external terminal window running an interactive `ssh` connected to the host.
/// The window is independent of the app and fully interactive (Tab completion handled by the remote shell).
pub fn open_terminal(host: &Host) -> Result<(), String> {
    let args = build_terminal_args(host)?;
    open_terminal_platform(&args)
}

/// Windows: a new, visible PowerShell console running the ssh command via the call operator.
#[cfg(target_os = "windows")]
fn open_terminal_platform(args: &[String]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x00000010;

    // Wrap each argument in single quotes (doubling inner single quotes to escape them).
    let quoted: Vec<String> = args
        .iter()
        .map(|part| format!("'{}'", part.replace('\'', "''")))
        .collect();
    let script = format!("& {}", quoted.join(" "));

    let mut command = Command::new("powershell");
    command.args(["-NoExit", "-Command", &script]);
    command.creation_flags(CREATE_NEW_CONSOLE);
    command.spawn().map_err(|err| err.to_string())?;
    Ok(())
}

/// macOS: hand the ssh command to Terminal.app via AppleScript.
#[cfg(target_os = "macos")]
fn open_terminal_platform(args: &[String]) -> Result<(), String> {
    let joined = args
        .iter()
        .map(|part| format!("'{}'", part.replace('\'', "'\\''")))
        .collect::<Vec<_>>()
        .join(" ");
    // Escape for the AppleScript string literal.
    let escaped = joined.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("tell application \"Terminal\" to do script \"{escaped}\"");
    let mut command = Command::new("osascript");
    command.args(["-e", &script]);
    command.spawn().map_err(|err| err.to_string())?;
    Ok(())
}

/// Linux: probe common terminal emulators until one launches.
#[cfg(target_os = "linux")]
fn open_terminal_platform(args: &[String]) -> Result<(), String> {
    // Each emulator differs in how it takes the command to run:
    //   gnome-terminal wants `-- cmd args`; most others take `-e cmd args`; xfce4-terminal uses `-x cmd args`.
    enum ExecFlag {
        DashE,
        DashDash,
        DashX,
    }
    let candidates = [
        ("x-terminal-emulator", ExecFlag::DashE),
        ("gnome-terminal", ExecFlag::DashDash),
        ("konsole", ExecFlag::DashE),
        ("xfce4-terminal", ExecFlag::DashX),
        ("xterm", ExecFlag::DashE),
    ];

    for (term, flag) in candidates {
        let mut command = Command::new(term);
        match flag {
            ExecFlag::DashE => {
                command.arg("-e").args(args);
            }
            ExecFlag::DashDash => {
                command.arg("--").args(args);
            }
            ExecFlag::DashX => {
                command.arg("-x").args(args);
            }
        }
        match command.spawn() {
            Ok(_) => return Ok(()),
            // Not installed: try the next emulator.
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err.to_string()),
        }
    }
    Err("No supported terminal emulator found. Install one of: gnome-terminal, konsole, xfce4-terminal, or xterm."
        .to_string())
}
