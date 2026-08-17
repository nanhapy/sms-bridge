use crate::{
    receiver::{MAGIC_DISCOVER, MAGIC_READY, MULTICAST_GROUP, PORT},
    runtime::AppRuntime,
};
use local_ip_address::list_afinet_netifas;
use std::{
    net::{IpAddr, Ipv4Addr, UdpSocket},
    sync::Arc,
    thread,
    time::Duration,
};

const DISCOVERY_UNAVAILABLE_MESSAGE: &str = "自动发现不可用，可在手机端手动填写电脑 IP";

pub fn start(runtime: Arc<AppRuntime>) {
    let discovery_runtime = runtime.clone();
    if let Err(error) = thread::Builder::new()
        .name("sms-bridge-discovery".to_string())
        .spawn(move || run(discovery_runtime))
    {
        mark_degraded(&runtime, &error.to_string());
    }
}

fn run(runtime: Arc<AppRuntime>) {
    let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, PORT)) {
        Ok(socket) => socket,
        Err(error) => {
            mark_degraded(&runtime, &error.to_string());
            return;
        }
    };

    if let Err(error) = socket.set_broadcast(true) {
        mark_degraded(&runtime, &error.to_string());
        return;
    }

    if let Err(error) = join_multicast_interfaces(&socket) {
        mark_degraded(&runtime, &error.to_string());
    }

    if let Err(error) = socket.set_read_timeout(Some(Duration::from_secs(1))) {
        mark_degraded(&runtime, &error.to_string());
        return;
    }

    let hostname = hostname::get()
        .ok()
        .and_then(|hostname| hostname.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());
    let mut buffer = [0_u8; 65_536];

    loop {
        match socket.recv_from(&mut buffer) {
            Ok((length, sender)) => {
                let Ok(message) = std::str::from_utf8(&buffer[..length]) else {
                    continue;
                };
                if !message.contains(MAGIC_DISCOVER) {
                    continue;
                }
                let IpAddr::V4(peer) = sender.ip() else {
                    continue;
                };
                let best_ip = pick_local_ip_for(peer)
                    .map(|address| address.to_string())
                    .unwrap_or_default();
                let reply = format!("{MAGIC_READY}|{hostname}|{PORT}|{best_ip}");
                let _ = socket.send_to(reply.as_bytes(), sender);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            Err(error) => {
                mark_degraded(&runtime, &error.to_string());
                return;
            }
        }
    }
}

fn join_multicast_interfaces(socket: &UdpSocket) -> std::io::Result<()> {
    let interfaces = usable_ipv4_addresses();
    if interfaces.is_empty() {
        return socket.join_multicast_v4(&MULTICAST_GROUP, &Ipv4Addr::UNSPECIFIED);
    }

    let mut joined = false;
    for interface in interfaces {
        if socket
            .join_multicast_v4(&MULTICAST_GROUP, &interface)
            .is_ok()
        {
            joined = true;
        }
    }

    if joined {
        Ok(())
    } else {
        socket.join_multicast_v4(&MULTICAST_GROUP, &Ipv4Addr::UNSPECIFIED)
    }
}

pub fn pick_local_ip_for(peer: Ipv4Addr) -> Option<Ipv4Addr> {
    usable_ipv4_addresses()
        .into_iter()
        .find(|address| same_subnet_24(peer, *address))
}

pub fn same_subnet_24(left: Ipv4Addr, right: Ipv4Addr) -> bool {
    left.octets()[..3] == right.octets()[..3]
}

fn usable_ipv4_addresses() -> Vec<Ipv4Addr> {
    list_afinet_netifas()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(_, address)| match address {
            IpAddr::V4(address) if !address.is_loopback() && !address.is_unspecified() => Some(address),
            _ => None,
        })
        .collect()
}

fn mark_degraded(runtime: &Arc<AppRuntime>, error: &str) {
    log::error!("UDP discovery unavailable: {error}");
    tauri::async_runtime::block_on(runtime.set_degraded_unless_unavailable(
        PORT,
        DISCOVERY_UNAVAILABLE_MESSAGE.to_string(),
    ));
}
