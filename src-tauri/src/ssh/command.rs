use crate::model::{Forward, Host, TunnelMode};
use crate::util::empty_default;

/// `user@host` 或仅 `host`（未填用户时）。
pub fn destination(host: &Host) -> String {
    if host.ssh_user.trim().is_empty() {
        host.ssh_host.trim().to_string()
    } else {
        format!("{}@{}", host.ssh_user.trim(), host.ssh_host.trim())
    }
}

fn push_identity_and_extra(host: &Host, command: &mut Vec<String>) -> Result<(), String> {
    if !host.identity_file.trim().is_empty() {
        command.extend(["-i".to_string(), host.identity_file.trim().to_string()]);
    }
    if !host.extra_options.trim().is_empty() {
        command.extend(shell_words::split(&host.extra_options).map_err(|err| err.to_string())?);
    }
    Ok(())
}

/// 后台端口转发命令：`ssh -N -T … -L/-R/-D … user@host`。
pub fn build_ssh_command(host: &Host, forward: &Forward) -> Result<Vec<String>, String> {
    let mut command = vec![
        "ssh".to_string(),
        "-N".to_string(),
        "-T".to_string(),
        "-o".to_string(),
        "ExitOnForwardFailure=yes".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=30".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(),
        empty_default(&host.ssh_port, "22").to_string(),
    ];
    push_identity_and_extra(host, &mut command)?;

    let bind = empty_default(&forward.bind_host, "127.0.0.1");
    match forward.mode {
        TunnelMode::Local => command.extend([
            "-L".to_string(),
            format!(
                "{}:{}:{}:{}",
                bind,
                forward.bind_port.trim(),
                empty_default(&forward.target_host, "127.0.0.1"),
                forward.target_port.trim()
            ),
        ]),
        TunnelMode::Remote => command.extend([
            "-R".to_string(),
            format!(
                "{}:{}:{}:{}",
                bind,
                forward.bind_port.trim(),
                empty_default(&forward.target_host, "127.0.0.1"),
                forward.target_port.trim()
            ),
        ]),
        TunnelMode::Dynamic => {
            command.extend(["-D".to_string(), format!("{}:{}", bind, forward.bind_port.trim())])
        }
    }
    command.push(destination(host));
    Ok(command)
}

/// 探测命令：`BatchMode=yes`，区分免密直连 / 需要密码 / 指纹变化 / 不可达。
pub fn build_probe_command(host: &Host) -> Result<Vec<String>, String> {
    let mut command = vec![
        "ssh".to_string(),
        "-T".to_string(),
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "ConnectTimeout=8".to_string(),
        "-o".to_string(),
        "NumberOfPasswordPrompts=0".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(),
        empty_default(&host.ssh_port, "22").to_string(),
    ];
    push_identity_and_extra(host, &mut command)?;
    command.push(destination(host));
    command.push("exit 0".to_string());
    Ok(command)
}

/// 上传公钥命令（允许交互式输入密码）。
pub fn build_key_upload_command(host: &Host) -> Result<Vec<String>, String> {
    let mut command = vec![
        "ssh".to_string(),
        "-T".to_string(),
        "-o".to_string(),
        "BatchMode=no".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(),
        empty_default(&host.ssh_port, "22").to_string(),
    ];
    if !host.identity_file.trim().is_empty() {
        command.extend(["-i".to_string(), host.identity_file.trim().to_string()]);
    }
    command.push(destination(host));
    command.push(
        "umask 077; mkdir -p ~/.ssh && touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && key=$(cat) && { grep -qxF \"$key\" ~/.ssh/authorized_keys || printf '%s\\n' \"$key\" >> ~/.ssh/authorized_keys; }"
            .to_string(),
    );
    Ok(command)
}

/// 发送一次性指令命令。`with_password=false` 时用 `BatchMode=yes` 依赖免密登录；
/// `with_password=true` 时允许密码（配合 askpass 注入）。
pub fn build_send_command(
    host: &Host,
    remote_command: &str,
    with_password: bool,
) -> Result<Vec<String>, String> {
    let batch_mode = if with_password { "BatchMode=no" } else { "BatchMode=yes" };
    let prompts = if with_password {
        "NumberOfPasswordPrompts=1"
    } else {
        "NumberOfPasswordPrompts=0"
    };
    let mut command = vec![
        "ssh".to_string(),
        "-T".to_string(),
        "-o".to_string(),
        batch_mode.to_string(),
        "-o".to_string(),
        "ConnectTimeout=8".to_string(),
        "-o".to_string(),
        prompts.to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(),
        empty_default(&host.ssh_port, "22").to_string(),
    ];
    push_identity_and_extra(host, &mut command)?;
    command.push(destination(host));
    command.push(remote_command.to_string());
    Ok(command)
}

/// 交互式终端用的 ssh 参数（不含 `ssh` 本身后续会单独拼装）。
pub fn build_terminal_args(host: &Host) -> Result<Vec<String>, String> {
    let mut command = vec![
        "ssh".to_string(),
        "-p".to_string(),
        empty_default(&host.ssh_port, "22").to_string(),
    ];
    push_identity_and_extra(host, &mut command)?;
    command.push(destination(host));
    Ok(command)
}
