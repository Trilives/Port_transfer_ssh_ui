//! Confirm that a spawned SSH tunnel is usable before the UI reports it as connected.

use std::{
    io::Read,
    net::{TcpStream, ToSocketAddrs},
    process::Child,
    thread,
    time::{Duration, Instant},
};

use crate::model::Forward;

/// Local/dynamic forwards are ready once their listener accepts TCP. Remote forwards have no local listener,
/// so only guard against an immediate ssh failure before returning.
pub fn wait_for_tunnel_ready(child: &mut Child, forward: &Forward) -> Result<(), String> {
    let timeout = if forward.binds_local_port() {
        Duration::from_secs(10)
    } else {
        Duration::from_millis(400)
    };
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            let mut stderr = String::new();
            if let Some(stream) = child.stderr.as_mut() {
                let _ = stream.read_to_string(&mut stderr);
            }
            let detail = stderr.trim();
            return Err(if detail.is_empty() {
                format!("SSH tunnel exited before becoming ready ({status}).")
            } else {
                detail.to_string()
            });
        }
        if forward.binds_local_port() && local_listener_ready(forward) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return if forward.binds_local_port() {
                Err(format!(
                    "SSH tunnel did not start listening on {}:{} within {} seconds.",
                    listener_probe_host(&forward.bind_host),
                    forward.bind_port.trim(),
                    timeout.as_secs()
                ))
            } else {
                Ok(())
            };
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn local_listener_ready(forward: &Forward) -> bool {
    let Ok(port) = forward.bind_port.trim().parse::<u16>() else {
        return false;
    };
    let Ok(addresses) = (listener_probe_host(&forward.bind_host), port).to_socket_addrs() else {
        return false;
    };
    addresses
        .into_iter()
        .any(|address| TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok())
}

fn listener_probe_host(bind_host: &str) -> &str {
    match bind_host.trim() {
        "" | "0.0.0.0" => "127.0.0.1",
        "::" => "::1",
        host => host,
    }
}

#[cfg(test)]
mod tests {
    use super::listener_probe_host;

    #[test]
    fn wildcard_listeners_are_probed_via_loopback() {
        assert_eq!(listener_probe_host("0.0.0.0"), "127.0.0.1");
        assert_eq!(listener_probe_host("::"), "::1");
        assert_eq!(listener_probe_host("192.0.2.8"), "192.0.2.8");
    }
}
