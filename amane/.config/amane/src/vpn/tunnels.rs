/*
 * finding vpn tunnels: an interface the kernel calls a wireguard device, or one
 * with a tun_flags file (a tun or tap device, like openvpn or tailscale0)
 *
 * and finding out whether all traffic goes through one: `ip route get` answers
 * which interface a packet to the internet would really leave by, following the
 * routing rules wg-quick, tailscale exit nodes and vpn apps set up, which the
 * main routing table alone doesn't show
 */

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::command;

const INTERFACES: &str = "/sys/class/net";

// any address on the internet will do, nothing is sent to it
const SOMEWHERE: &str = "1.1.1.1";

// ip answers from the kernel's tables at once
const IP_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    WireGuard,

    // tun or tap: openvpn, tailscale and most vpn apps that aren't wireguard
    Tunnel,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::WireGuard => "WireGuard",
            Kind::Tunnel => "tunnel",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tunnel {
    pub interface: String,
    pub kind: Kind,
}

// every tunnel interface there is right now; plain file reads, cheap enough every few seconds
pub fn scan() -> Vec<Tunnel> {
    let Ok(entries) = fs::read_dir(INTERFACES) else {
        return Vec::new();
    };

    let mut tunnels: Vec<Tunnel> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let interface = entry.file_name().to_string_lossy().into_owned();

            let kind = kind(&entry.path())?;

            Some(Tunnel { interface, kind })
        })
        .collect();

    tunnels.sort_by(|a, b| a.interface.cmp(&b.interface));

    tunnels
}

fn kind(path: &Path) -> Option<Kind> {
    let uevent = fs::read_to_string(path.join("uevent")).unwrap_or_default();

    kind_of(&uevent, path.join("tun_flags").exists())
}

fn kind_of(uevent: &str, tun_flags: bool) -> Option<Kind> {
    if uevent
        .lines()
        .any(|line| line.trim() == "DEVTYPE=wireguard")
    {
        return Some(Kind::WireGuard);
    }

    tun_flags.then_some(Kind::Tunnel)
}

// the interface traffic to the internet leaves by; waits on `ip`, so from a thread
pub fn route() -> Option<String> {
    let answer = command::read(
        Command::new("ip").args(["route", "get", SOMEWHERE]),
        IP_TIMEOUT,
    )?;

    device(&answer)
}

// 1.1.1.1 via 192.168.1.1 dev wlp2s0 src 192.168.1.204 uid 1000
fn device(answer: &str) -> Option<String> {
    let mut words = answer.split_whitespace();

    words.find(|word| *word == "dev")?;

    words.next().map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wireguard_tun_and_ordinary_interfaces() {
        assert_eq!(
            kind_of("DEVTYPE=wireguard\nINTERFACE=wg0\nIFINDEX=7\n", false),
            Some(Kind::WireGuard)
        );
        assert_eq!(
            kind_of("INTERFACE=tun0\nIFINDEX=8\n", true),
            Some(Kind::Tunnel)
        );
        assert_eq!(kind_of("DEVTYPE=wlan\nINTERFACE=wlp2s0\n", false), None);
        assert_eq!(kind_of("INTERFACE=enp1s0\n", false), None);
    }

    #[test]
    fn the_device_traffic_leaves_by() {
        assert_eq!(
            device("1.1.1.1 via 192.168.1.1 dev wlp2s0 src 192.168.1.204 uid 1000 \n    cache \n")
                .as_deref(),
            Some("wlp2s0")
        );
        assert_eq!(
            device("1.1.1.1 dev wg0 table 51820 src 10.64.0.2 uid 1000\n    cache\n").as_deref(),
            Some("wg0")
        );
        assert_eq!(device("RTNETLINK answers: Network is unreachable"), None);
    }

    #[test]
    fn this_machine_scans_without_trouble() {
        // whatever is up right now; ordinary interfaces must never count as tunnels
        for tunnel in scan() {
            assert_ne!(tunnel.interface, "lo");
        }
    }
}
