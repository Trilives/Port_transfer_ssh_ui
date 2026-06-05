use std::process::Command;

use crate::model::Host;
use crate::ssh::command::build_terminal_args;

/// 打开一个外部 PowerShell 窗口，运行交互式 `ssh` 连接到该主机。
/// 该窗口独立于本应用，提供完整交互（Tab 补全由远端 shell 完成）。
pub fn open_terminal(host: &Host) -> Result<(), String> {
    let args = build_terminal_args(host)?;
    // 每个参数单引号包裹（内部单引号双写转义），用 PowerShell 调用运算符执行。
    let quoted: Vec<String> = args
        .iter()
        .map(|part| format!("'{}'", part.replace('\'', "''")))
        .collect();
    let script = format!("& {}", quoted.join(" "));

    let mut command = Command::new("powershell");
    command.args(["-NoExit", "-Command", &script]);
    open_in_new_console(&mut command);
    command.spawn().map_err(|err| err.to_string())?;
    Ok(())
}

/// 在新的控制台窗口中启动（可见），而不是隐藏窗口。
#[cfg(target_os = "windows")]
fn open_in_new_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x00000010;
    command.creation_flags(CREATE_NEW_CONSOLE);
}

#[cfg(not(target_os = "windows"))]
fn open_in_new_console(_command: &mut Command) {}
