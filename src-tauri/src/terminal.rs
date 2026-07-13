use std::process::Command;

use crate::model::Host;
use crate::ssh::command::build_terminal_args;

/// Open an external terminal window running an interactive `ssh` connected to the host.
/// The window is independent of the app and fully interactive (Tab completion handled by the remote shell).
/// A non-empty `path` opens the shell already `cd`'d into that remote directory.
pub fn open_terminal(host: &Host, path: Option<&str>) -> Result<(), String> {
    let args = build_terminal_args(host, path)?;
    open_terminal_platform(&args)
}

/// A new, visible PowerShell console running the ssh command via the call operator.
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
