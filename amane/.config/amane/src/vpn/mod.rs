/*
 * vpns: tailscale in full, and any other tunnel (wireguard, openvpn, a vpn app)
 * just noticed; the bar's network icon, the network panel and the system panel
 * read it from here
 *
 * every 5 seconds the interfaces are listed (plain file reads, see tunnels.rs);
 * only while a tunnel exists does it also ask `ip route get` whether all traffic
 * goes through one, and only while tailscale0 exists does it ask tailscale for
 * its status. while the network or system panel is open, every 2 seconds
 */

pub mod tailscale;
pub mod tunnels;

use std::thread;
use std::time::Duration;

use amane::Service;

use crate::panels::{NETWORK, Panels, SYSTEM};

pub use tailscale::{Backend, Status};
pub use tunnels::Tunnel;

// tailscale's own interface, which gets its status instead of a plain mention
const TAILSCALE: &str = "tailscale0";

const SLOW: u32 = 5;
const FAST: u32 = 2;

// the suggested exit node: not asked yet, asked and none, or a name
#[derive(Debug, Clone, PartialEq)]
pub enum Suggestion {
    Unknown,
    None,
    Node(String),
}

#[derive(PartialEq)]
pub struct Vpn {
    tunnels: Vec<Tunnel>,

    // the interface traffic to the internet leaves by, while there's a tunnel
    route: Option<String>,

    tailscale: Option<Status>,

    // the exit node in use, as (its IP, its name), asked once each time it changes
    exit_name: Option<(String, String)>,

    suggestion: Suggestion,

    // a tailscale command is running
    busy: bool,

    // why the last tailscale command failed
    error: Option<String>,
}

impl Service for Vpn {
    fn new() -> Self {
        Self {
            tunnels: Vec::new(),
            route: None,
            tailscale: None,
            exit_name: None,
            suggestion: Suggestion::Unknown,
            busy: false,
            error: None,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn listen() {
        let mut ticks: u32 = 0;
        let mut network_was_open = false;

        loop {
            let (network, system) = {
                let panels = Panels::read();

                (panels.is_open(NETWORK), panels.is_open(SYSTEM))
            };

            // opening the network panel asks for a fresh suggestion
            let opened = network && !network_was_open;

            network_was_open = network;

            let every = if network || system { FAST } else { SLOW };

            if opened || ticks % every == 0 {
                check(opened);
            }

            ticks = ticks.wrapping_add(1);

            thread::sleep(Self::interval());
        }
    }
}

/*
 * looks again, and writes only when something shown changed; the exit node's
 * name is asked only when its IP changed, and the suggestion only when asked for
 */
fn check(suggest: bool) {
    let tunnels = tunnels::scan();

    let route = if tunnels.is_empty() {
        None
    } else {
        tunnels::route()
    };

    let tailscale = if tunnels.iter().any(|tunnel| tunnel.interface == TAILSCALE) {
        tailscale::status()
    } else {
        None
    };

    let exit_ip = tailscale.as_ref().and_then(|status| status.exit_ip.clone());

    let known = Vpn::read().exit_name.clone();

    let exit_name = match (exit_ip, known) {
        (None, _) => None,
        (Some(ip), Some(known)) if known.0 == ip => Some(known),
        (Some(ip), _) => {
            let name = tailscale::whois(&ip).unwrap_or_else(|| ip.clone());

            Some((ip, name))
        }
    };

    let connected = tailscale.as_ref().is_some_and(Status::connected);

    let suggestion = match (suggest, connected) {
        (true, true) => Some(match tailscale::suggest() {
            Some(node) => Suggestion::Node(node),
            None => Suggestion::None,
        }),
        (_, false) => Some(Suggestion::Unknown),
        (false, true) => None,
    };

    let changed = {
        let vpn = Vpn::read();

        vpn.tunnels != tunnels
            || vpn.route != route
            || vpn.tailscale != tailscale
            || vpn.exit_name != exit_name
            || suggestion
                .as_ref()
                .is_some_and(|fresh| *fresh != vpn.suggestion)
    };

    if changed {
        let mut vpn = Vpn::write();

        vpn.tunnels = tunnels;
        vpn.route = route;
        vpn.tailscale = tailscale;
        vpn.exit_name = exit_name;

        if let Some(suggestion) = suggestion {
            vpn.suggestion = suggestion;
        }
    }
}

// reading, safe inside views
impl Vpn {
    pub fn tailscale(&self) -> Option<&Status> {
        self.tailscale.as_ref()
    }

    // the exit node in use, by name (its IP until the name is known)
    pub fn exit_node(&self) -> Option<&str> {
        let ip = self.tailscale.as_ref()?.exit_ip.as_deref()?;

        match &self.exit_name {
            Some((known, name)) if known == ip => Some(name),
            _ => Some(ip),
        }
    }

    pub fn suggestion(&self) -> &Suggestion {
        &self.suggestion
    }

    // tunnels that aren't tailscale, which are only noticed
    pub fn others(&self) -> impl Iterator<Item = &Tunnel> {
        self.tunnels
            .iter()
            .filter(|tunnel| tunnel.interface != TAILSCALE)
    }

    // the tunnel all traffic goes through, if any; a tailscale exit node counts
    pub fn all_traffic(&self) -> Option<&Tunnel> {
        let route = self.route.as_deref()?;

        self.tunnels.iter().find(|tunnel| tunnel.interface == route)
    }

    /*
     * for the shield next to the network icon: None without a vpn, Some(true)
     * when all traffic goes through one; tailscale only counts while connected
     */
    pub fn shield(&self) -> Option<bool> {
        let tailscale_up = self.tailscale.as_ref().is_some_and(Status::connected);

        let any = tailscale_up || self.others().next().is_some();

        any.then(|| self.all_traffic().is_some())
    }

    pub fn busy(&self) -> bool {
        self.busy
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

// nerd font material design shield-check
pub const ICON_SHIELD: &str = "\u{f0565}";

/*
 * the tailscale commands, from clicks and keys; each runs on a thread, then the
 * state is read again at once instead of at the next check
 */
impl Vpn {
    // up or starting switches it off, anything else on
    pub fn toggle_tailscale() {
        let active = Vpn::read().tailscale.as_ref().is_some_and(Status::active);

        run(move || {
            if active {
                tailscale::down()
            } else {
                tailscale::up()
            }
        });
    }

    // a name sends all traffic through that exit node, None stops that
    pub fn use_exit_node(node: Option<String>) {
        run(move || tailscale::set_exit_node(node.as_deref()));
    }

    pub fn log_in() {
        let url = Vpn::read()
            .tailscale
            .as_ref()
            .map(|status| status.auth_url.clone())
            .unwrap_or_default();

        if url.is_empty() {
            return;
        }

        run(move || tailscale::log_in(&url));
    }

    pub fn dismiss_error() {
        if Vpn::read().error.is_some() {
            Vpn::write().error = None;
        }
    }
}

fn run(command: impl FnOnce() -> Result<(), String> + Send + 'static) {
    {
        let mut vpn = Vpn::write();

        if vpn.busy {
            return;
        }

        vpn.busy = true;
        vpn.error = None;
    }

    thread::spawn(move || {
        let result = command();

        {
            let mut vpn = Vpn::write();

            vpn.busy = false;
            vpn.error = result.err();
        }

        // a change can change what tailscale suggests too
        check(true);
    });
}
