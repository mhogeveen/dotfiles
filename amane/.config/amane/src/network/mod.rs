/*
 * what the network panel adds on top of amane's Network service: whether a
 * connect attempt worked, the connection's address and gateway, and forgetting
 * saved networks
 *
 * lock order: the tick copies what it needs out of Network and Panels and lets
 * go of them before it writes NetworkPanel, and nothing holds NetworkPanel's
 * write lock while reading another service, so no thread can deadlock another
 */

mod details;
mod profiles;

use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use amane::{Link, Network, Service};

use crate::panels::{NETWORK, Panels};

pub use details::Details;

// networkmanager on the system bus, for details.rs and profiles.rs
const NAME: &str = "org.freedesktop.NetworkManager";
const PATH: &str = "/org/freedesktop/NetworkManager";

// an attempt that hasn't joined by then counts as failed
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(30);

// the address can come a moment after joining, so it's read again this often
const DETAILS_EVERY: u32 = 5;

#[derive(Debug, Clone, PartialEq)]
struct Attempt {
    ssid: String,
    started: Instant,

    // networkmanager started working on it, so it stopping means it's done
    saw_connecting: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Failed(String),

    // the network, and why
    ForgetFailed(String, String),
}

pub struct NetworkPanel {
    attempt: Option<Attempt>,
    result: Option<Outcome>,

    // the network whose password box is open
    expanded: Option<String>,

    details: Option<Details>,

    // what the details were read for, to read them again when the connection changes
    details_for: Option<(Link, Option<String>)>,

    was_open: bool,
}

impl Service for NetworkPanel {
    fn new() -> Self {
        Self {
            attempt: None,
            result: None,
            expanded: None,
            details: None,
            details_for: None,
            was_open: false,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn listen() {
        let mut ticks: u32 = 0;

        loop {
            thread::sleep(Self::interval());

            ticks = ticks.wrapping_add(1);

            tick(ticks % DETAILS_EVERY == 0);
        }
    }
}

// the parts of Network and Panels a tick looks at, copied so no lock is held
struct Snapshot {
    open: bool,
    link: Link,
    connecting: bool,

    // the network the device is on, even when amane reports no internet
    joined: Option<String>,
}

impl Snapshot {
    fn take() -> Self {
        let open = Panels::read().is_open(NETWORK);
        let network = Network::read();

        Self {
            open,
            link: network.link(),
            connecting: network.connecting(),
            joined: joined(&network),
        }
    }
}

// nerd font material design glyphs
pub const ICON_ETHERNET: &str = "\u{f0200}";
pub const ICON_WIFI_OFF: &str = "\u{f05aa}";
pub const ICON_OFFLINE: &str = "\u{f092e}";
pub const ICON_NO_INTERNET: &str = "\u{f092b}";
pub const ICON_CONNECTING: &str = "\u{f092f}";
pub const ICON_LOCK: &str = "\u{f033e}";
pub const ICON_RESCAN: &str = "\u{f0450}";

// one to four bars
pub fn strength_icon(strength: u8) -> &'static str {
    match strength {
        0..=25 => "\u{f091f}",
        26..=50 => "\u{f0922}",
        51..=75 => "\u{f0925}",
        _ => "\u{f0928}",
    }
}

pub enum Status {
    WifiOff,
    Connecting,
    Wired,
    Wifi(u8),

    // joined a wifi network, but networkmanager reports no internet through it
    NoInternet,

    // a vpn, tethering and the rest
    Other,
    Offline,
}

pub fn status(network: &Network) -> Status {
    match network.link() {
        Link::Wired => Status::Wired,
        Link::Wifi => Status::Wifi(network.strength()),
        Link::Other => Status::Other,
        Link::Offline if network.connecting() => Status::Connecting,
        Link::Offline if !network.wifi_enabled() => Status::WifiOff,
        Link::Offline if joined(network).is_some() => Status::NoInternet,
        Link::Offline => Status::Offline,
    }
}

impl Status {
    pub fn icon(&self) -> &'static str {
        match self {
            Status::WifiOff => ICON_WIFI_OFF,
            Status::Connecting => ICON_CONNECTING,
            Status::Wired => ICON_ETHERNET,
            Status::Wifi(strength) => strength_icon(*strength),
            Status::NoInternet => ICON_NO_INTERNET,
            Status::Other => strength_icon(100),
            Status::Offline => ICON_OFFLINE,
        }
    }
}

// the network the wifi device is on, from the list since that works without internet
pub fn joined(network: &Network) -> Option<String> {
    network
        .access_points()
        .iter()
        .find(|access_point| access_point.active())
        .map(|access_point| String::from(access_point.ssid()))
}

fn tick(refresh_details: bool) {
    let now = Snapshot::take();

    let (attempt, details_for, was_open) = {
        let panel = NetworkPanel::read();

        (
            panel.attempt.clone(),
            panel.details_for.clone(),
            panel.was_open,
        )
    };

    let opened = now.open && !was_open;

    if opened {
        Network::scan();
    }

    let key = (now.link, now.joined.clone());

    // read while no lock is held, it waits on d-bus
    let details = (now.open && (opened || refresh_details || details_for.as_ref() != Some(&key)))
        .then(details::fetch);

    let (judged, failed) = judge(attempt.clone(), &now);

    let changed =
        details.is_some() || judged != attempt || failed.is_some() || was_open != now.open;

    if !changed {
        return;
    }

    let mut panel = NetworkPanel::write();

    panel.was_open = now.open;

    if let Some(details) = details {
        // writing anyway would redraw the panel every few seconds for nothing
        if panel.details != details || panel.details_for.as_ref() != Some(&key) {
            panel.details = details;
            panel.details_for = Some(key);
        }
    }

    // a click may have started a new attempt since the snapshot, which wins
    if panel.attempt == attempt {
        panel.attempt = judged;

        if let Some(ssid) = failed {
            panel.result = Some(Outcome::Failed(ssid));
        }
    }
}

// the attempt as it stands now, and the network it failed to join if it did
fn judge(attempt: Option<Attempt>, now: &Snapshot) -> (Option<Attempt>, Option<String>) {
    let Some(mut attempt) = attempt else {
        return (None, None);
    };

    if now.connecting {
        attempt.saw_connecting = true;
    }

    if !now.connecting && now.joined.as_deref() == Some(attempt.ssid.as_str()) {
        return (None, None);
    }

    let gave_up = attempt.saw_connecting && !now.connecting;

    if gave_up || attempt.started.elapsed() > ATTEMPT_TIMEOUT {
        return (None, Some(attempt.ssid));
    }

    (Some(attempt), None)
}

// reading, safe inside views
impl NetworkPanel {
    // the network being joined right now
    pub fn connecting_to(&self) -> Option<&str> {
        self.attempt.as_ref().map(|attempt| attempt.ssid.as_str())
    }

    pub fn result(&self) -> Option<&Outcome> {
        self.result.as_ref()
    }

    pub fn expanded(&self) -> Option<&str> {
        self.expanded.as_deref()
    }

    pub fn details(&self) -> Option<&Details> {
        self.details.as_ref()
    }
}

/*
 * writing, from clicks and key handlers; none of these wait, the d-bus work
 * happens in amane's worker or on a thread of its own
 */
impl NetworkPanel {
    // a saved or open network takes None, a new secured one its password
    pub fn connect(ssid: &str, password: Option<&str>) {
        {
            let mut panel = NetworkPanel::write();

            panel.attempt = Some(Attempt {
                ssid: String::from(ssid),
                started: Instant::now(),
                saw_connecting: false,
            });

            panel.result = None;
            panel.expanded = None;
        }

        Network::connect(ssid, password);
    }

    pub fn disconnect() {
        Network::disconnect();
    }

    pub fn forget(ssid: &str) {
        let ssid = String::from(ssid);

        thread::spawn(move || {
            let forgotten = profiles::forget(&ssid);

            {
                let mut panel = NetworkPanel::write();

                panel.result = match forgotten {
                    Err(reason) => Some(Outcome::ForgetFailed(ssid, reason)),

                    // a failure on this network is solved by forgetting it
                    Ok(()) => match panel.result.take() {
                        Some(Outcome::Failed(failed)) if failed == ssid => None,
                        other => other,
                    },
                };
            }

            // so the list drops its "Saved"
            Network::scan();
        });
    }

    // opens the password box under this network, or closes it when it's open
    pub fn toggle_expanded(ssid: &str) {
        let mut panel = NetworkPanel::write();

        panel.expanded = match panel.expanded.as_deref() {
            Some(open) if open == ssid => None,
            _ => Some(String::from(ssid)),
        };
    }

    // closes the password box, whichever network it's under
    pub fn close_password() {
        NetworkPanel::write().expanded = None;
    }

    // on close: no password box, and an old result doesn't come back on the next open
    pub fn reset() {
        let mut panel = NetworkPanel::write();

        panel.expanded = None;
        panel.result = None;
    }
}

/*
 * whether there's any wi-fi hardware: amane's Network doesn't say, and with none
 * the list would claim "No networks found". the kernel marks wireless
 * interfaces with a wireless folder; a few entries, so cheap to read on a redraw
 */
pub fn has_wifi_adapter() -> bool {
    fs::read_dir(SYSFS_INTERFACES).is_ok_and(|interfaces| {
        interfaces
            .flatten()
            .any(|interface| interface.path().join("wireless").is_dir())
    })
}

const SYSFS_INTERFACES: &str = "/sys/class/net";
