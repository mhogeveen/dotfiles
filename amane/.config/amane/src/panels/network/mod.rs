/*
 * network panel, top-right: the current connection with its address, the wi-fi
 * switch, and the networks in range; click one to join it, a new secured one
 * asks for its password first
 *
 * the state behind it (connect results, details, forgetting) is in src/network
 */

mod vpn_cards;

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amane::{
    AccessPoint,
    Align::{Center, End, Start},
    Cursor::Pointer,
    Key, LayerWindow, Monitor, Network, Padding, Rectangle, Row, Service,
    Size::Parent,
    Text, TextInput, Weight, Widget, children,
};

use super::ui::{self, BUTTON, PADDING, SIDES, button};
use super::{NETWORK, list, panel};
use crate::network::{self, Details, ICON_LOCK, NetworkPanel, Outcome, Status};
use crate::theme::Theme;
use crate::vpn::Vpn;

const GAP: f32 = 8.0;
const LINE: f32 = 20.0;
const ROW: f32 = 40.0;

// the password box under a network, a little taller than a row for its input
const PASSWORD_ROW: f32 = ROW + 4.0;

// the first row the list draws while the highlight is in use
static FIRST_ROW: AtomicUsize = AtomicUsize::new(0);

/*
 * the panel has a fixed height (see panels/mod.rs), so it's the most the parts
 * above the wi-fi list can take, plus room for three networks: then the list
 * never shrinks under that, and with less above it simply shows more
 */
const HEIGHT: f32 = ABOVE_LIST_AT_MOST + LIST_AT_LEAST;

const LIST_AT_LEAST: f32 = 3.0 * ROW + 2.0;

// the frame, the header, then everything that can sit above the list at once
const ABOVE_LIST_AT_MOST: f32 = PADDING * 2.0
    + BUTTON
    + CONNECTION_AT_MOST
    + vpn_cards::TAILSCALE_AT_MOST
    + LINE // another tunnel
    + BUTTON // a result line
    + 5.0 * GAP;

// the joined network with its details line and its buttons
const CONNECTION_AT_MOST: f32 = PADDING * 2.0 + LINE + GAP + LINE + GAP + BUTTON;

// the connection's state beside its name; the longest, "Wi-Fi is off", is 12 characters at 12 px
const STATE_WIDTH: f32 = 88.0;

const PASSWORD_INPUT: &str = "network-password";

/*
 * what's typed in the password box, for the Connect button; the box keeps its
 * own text, this copy needs no redraw so it isn't a service
 */
static PASSWORD: Mutex<String> = Mutex::new(String::new());

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = ui::corner(HEIGHT, "amane-network");

    panel(monitor, NETWORK, base, key_pressed, content)
}

const KEY_WIFI: char = 'w';
const KEY_RESCAN: char = 'r';
const KEY_TAILSCALE: char = 'v';
const KEY_LOG_IN: char = 'i';
const KEY_DISCONNECT: char = 'd';
const KEY_FORGET: char = 'f';

/*
 * which network in the list the keys act on, by its place; None until a key
 * moves it, see list::step_highlight
 */
struct Highlight {
    index: Option<usize>,
}

impl Service for Highlight {
    fn new() -> Self {
        Self { index: None }
    }

    // changes only through keys
    fn listen() {}
}

/*
 * the header's buttons by their letters, and the list: j/k or arrows move,
 * Enter does what a click does, f forgets the highlighted network and d leaves
 * the one you're on. a password being typed keeps letters and Enter for itself
 */
fn key_pressed(key: Key) {
    if let Some(delta) = list::highlight_delta(&key) {
        return move_highlight(delta);
    }

    match key {
        Key::Enter => {
            if let Some(access_point) = highlighted() {
                click(&access_point);
            }
        }
        Key::Character(letter) => match letter.to_ascii_lowercase() {
            KEY_WIFI => toggle_wifi(),
            KEY_RESCAN => rescan(),
            KEY_TAILSCALE if tailscale().0 => Vpn::toggle_tailscale(),
            KEY_LOG_IN if tailscale().1 => Vpn::log_in(),
            KEY_DISCONNECT if network::joined(&Network::read()).is_some() => {
                NetworkPanel::disconnect()
            }
            // the highlighted network, or without a highlight the one you're on, like d
            KEY_FORGET => {
                if Highlight::read().index.is_some() {
                    if let Some(access_point) = highlighted().filter(AccessPoint::saved) {
                        NetworkPanel::forget(access_point.ssid());
                    }
                } else if let Some(ssid) = network::joined(&Network::read()) {
                    NetworkPanel::forget(&ssid);
                }
            }
            _ => {}
        },
        _ => {}
    }
}

// from the keys and the wheel
fn move_highlight(delta: isize) {
    let count = listed(&Network::read()).len();
    let mut highlight = Highlight::write();

    highlight.index = list::step_highlight(highlight.index, delta, count);
}

// only while wi-fi is on, like the button
fn rescan() {
    if Network::read().wifi_enabled() {
        Network::scan();
    }
}

// the networks in the list, in the order shown: every one in range but the one you're on
fn listed(network: &Network) -> Vec<&AccessPoint> {
    if !network.wifi_enabled() {
        return Vec::new();
    }

    network
        .access_points()
        .iter()
        .filter(|access_point| !access_point.active())
        .collect()
}

// the highlighted network, if the keys moved to one; the list may have shrunk since
fn highlighted() -> Option<AccessPoint> {
    let index = Highlight::read().index?;

    let network = Network::read();
    let shown = listed(&network);

    shown
        .get(index.min(shown.len().saturating_sub(1)))
        .map(|access_point| (*access_point).clone())
}

// saved and open networks join right away, a new secured one asks for its password first
fn click(access_point: &AccessPoint) {
    let ssid = access_point.ssid();

    if access_point.secured() && !access_point.saved() {
        clear_password();

        NetworkPanel::toggle_expanded(ssid);
    } else {
        NetworkPanel::connect(ssid, None);
    }
}

// whether tailscale is there, and whether it waits for a login; read and let go before acting
fn tailscale() -> (bool, bool) {
    let vpn = Vpn::read();

    let status = vpn.tailscale();

    (status.is_some(), status.is_some_and(vpn_cards::needs_login))
}

fn toggle_wifi() {
    let enabled = Network::read().wifi_enabled();

    Network::set_wifi(!enabled);
}

fn content(t: &Theme) -> Rectangle {
    let network = Network::read();
    let panel = NetworkPanel::read();

    let (card, card_height) = connection(t, &network, &panel);

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(header(t, &network)), Box::new(card)];

    // what's above the list, so it knows how much of itself shows
    let mut above = BUTTON + GAP + card_height + GAP;

    // tailscale, and any other vpn there is
    let (vpn, vpn_height) = vpn_cards::vpn_parts(t);

    above += vpn_height;
    parts.extend(vpn);

    if let Some(result) = panel.result() {
        above += BUTTON + GAP;

        parts.push(Box::new(result_line(t, result)));
    }

    let visible = HEIGHT - PADDING * 2.0 - above - ui::LIST_BORDER * 2.0;

    parts.push(Box::new(networks(t, &network, &panel, visible)));

    ui::frame(t, GAP, parts)
}

/*
 * Escape while the password box is open closes just the box and forgets what
 * was typed; the highlight stays, so the keys move through the list again.
 * false when there was no box, so Escape closes the panel
 */
pub fn back_out() -> bool {
    if NetworkPanel::read().expanded().is_none() {
        return false;
    }

    clear_password();

    NetworkPanel::close_password();

    true
}

/*
 * on close: the password box closes and forgets what was typed, and the
 * highlight and old errors don't come back on the next open
 */
pub fn reset() {
    clear_password();

    NetworkPanel::reset();
    Vpn::dismiss_error();

    Highlight::write().index = None;
    FIRST_ROW.store(0, Ordering::Relaxed);
}

fn clear_password() {
    TextInput::set_text(PASSWORD_INPUT, "");

    PASSWORD
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clear();
}

// "Network", rescan, and the wi-fi switch
fn header(t: &Theme, network: &Network) -> Row {
    let enabled = network.wifi_enabled();

    let mut parts: Vec<Box<dyn Widget>> = Vec::new();

    // nothing to scan with wi-fi off, like bluetooth's rescan while it's off
    if enabled {
        parts.push(Box::new(ui::key_button(
            t,
            ui::ICON_REFRESH,
            KEY_RESCAN,
            |_| rescan(),
        )));
    }

    parts.push(Box::new(ui::switch(
        t,
        "Wi-Fi",
        92.0,
        enabled,
        KEY_WIFI,
        |_| toggle_wifi(),
    )));

    ui::header(ui::title(t, "Network"), parts)
}

// the current connection: what it is, how it's doing, its address, and buttons; the card and its height
fn connection(t: &Theme, network: &Network, panel: &NetworkPanel) -> (Rectangle, f32) {
    let status = network::status(network);
    let joined = network::joined(network);

    let (name, state) = connection_text(&status, joined.as_deref(), panel.connecting_to());

    let state_color = match status {
        Status::NoInternet => t.accent,
        _ => t.fg_dim,
    };

    let top = Row::new(children![
        Rectangle::new()
            .width(LINE)
            .height(LINE)
            .align_child(Center, Center)
            .child(Text::new(status.icon()).size(ui::ICON_SIZE).color(t.fg)),
        ui::line(
            LINE,
            Text::new(name)
                .size(ui::TEXT_BODY)
                .weight(Weight::Bold)
                .color(t.fg)
                .elide()
        ),
        Rectangle::new()
            .width(STATE_WIDTH)
            .height(LINE)
            .align_child(End, Center)
            .child(Text::new(state).size(ui::TEXT_SMALL).color(state_color)),
    ])
    .gap(GAP)
    .align(Center)
    .height(LINE);

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(top)];
    let mut height = PADDING * 2.0 + LINE;

    let details = panel.details().map(details_line).unwrap_or_default();

    if !details.is_empty() {
        height += GAP + LINE;

        parts.push(Box::new(ui::line(
            LINE,
            Text::new(details)
                .size(ui::TEXT_SMALL)
                .color(t.comment)
                .elide(),
        )));
    }

    // a wi-fi network you're on can be left or forgotten
    if let Some(ssid) = joined {
        height += GAP + BUTTON;

        parts.push(Box::new(leave_buttons(t, ssid)));
    }

    (ui::card(t, height, parts), height)
}

/*
 * the card's name and state. while joining, the name is the network being
 * joined, so the state on the right stays short and can't push the name out
 */
fn connection_text(
    status: &Status,
    joined: Option<&str>,
    joining: Option<&str>,
) -> (String, String) {
    let name = match (joining, status, joined) {
        (Some(ssid), _, _) => ssid,
        (None, Status::Wired, _) => "Ethernet",
        (None, _, Some(ssid)) => ssid,
        (None, Status::Other, None) => "Connected",
        _ => "Not connected",
    };

    let state = match (joining, status) {
        (Some(_), _) => "Connecting…",
        (None, Status::Wifi(_) | Status::Wired | Status::Other) => "Connected",
        (None, Status::NoInternet) => "No internet",
        (None, Status::WifiOff) => "Wi-Fi is off",
        (None, Status::Connecting) => "Connecting…",
        (None, Status::Offline) => "",
    };

    (String::from(name), String::from(state))
}

// like wlp2s0 · 192.168.1.23/24 · gw 192.168.1.1, leaving out what isn't known
fn details_line(details: &Details) -> String {
    let gateway = if details.gateway.is_empty() {
        String::new()
    } else {
        format!("gw {}", details.gateway)
    };

    [details.interface.clone(), details.address.clone(), gateway]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

// Disconnect and Forget for the network you're on
fn leave_buttons(t: &Theme, ssid: String) -> Row {
    let forget = move |_| NetworkPanel::forget(&ssid);

    // f forgets this one until a highlight in the list takes the key over
    let forget = if Highlight::read().index.is_none() {
        ui::key_text_button(t, "Forget", 80.0, KEY_FORGET, forget)
    } else {
        button(t, "Forget", 80.0, forget)
    };

    Row::new(children![
        ui::key_text_button(t, "Disconnect", 100.0, KEY_DISCONNECT, |_| {
            NetworkPanel::disconnect()
        }),
        forget,
    ])
    .gap(GAP)
    .height(BUTTON)
}

// what went wrong last, with a way out when there is one
fn result_line(t: &Theme, result: &Outcome) -> Row {
    let (text, forget) = match result {
        Outcome::Failed(ssid) => (format!("Couldn't connect to {ssid}"), Some(ssid.clone())),
        Outcome::ForgetFailed(ssid, reason) => (format!("Couldn't forget {ssid}: {reason}"), None),
    };

    // forgetting a network saved with a mistyped password lets you type it again
    let forget = forget.map(|ssid| button(t, "Forget", 80.0, move |_| NetworkPanel::forget(&ssid)));

    ui::error_line(t, &text, forget)
}

// every network in range except the one you're on
fn networks(t: &Theme, network: &Network, panel: &NetworkPanel, visible: f32) -> Rectangle {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // each row's height, to know which ones fit
    let mut heights: Vec<f32> = Vec::new();

    let shown = listed(network);

    let highlighted = Highlight::read()
        .index
        .map(|index| index.min(shown.len().saturating_sub(1)));

    // where the highlighted network's row is among all the rows, password box included
    let mut highlighted_row = None;

    // the last row to keep in view with it: its password box, when that's open
    let mut last_in_view = None;

    let message = if !network::has_wifi_adapter() {
        Some("No Wi-Fi adapter")
    } else if !network.wifi_enabled() {
        Some("Wi-Fi is off")
    } else if shown.is_empty() {
        Some("No networks found")
    } else {
        None
    };

    if let Some(message) = message {
        rows.push(Box::new(ui::note(t, message, ROW)));
        heights.push(ROW);
    }

    for (index, access_point) in shown.iter().enumerate() {
        let lit = highlighted == Some(index);

        if lit {
            highlighted_row = Some(rows.len());
        }

        rows.push(Box::new(network_row(t, access_point, panel, lit)));
        heights.push(ROW);

        if panel.expanded() == Some(access_point.ssid()) {
            // the highlighted network's password box stays in view with it
            if lit {
                last_in_view = Some(rows.len());
            }

            rows.push(Box::new(password_row(t, access_point.ssid())));
            heights.push(PASSWORD_ROW);
        }
    }

    let keep = highlighted_row.map(|row| (row, last_in_view.unwrap_or(row)));

    list::highlight_list(
        t,
        rows,
        &heights,
        keep,
        visible,
        &FIRST_ROW,
        "network-list",
        move_highlight,
    )
}

fn network_row(
    t: &Theme,
    access_point: &AccessPoint,
    panel: &NetworkPanel,
    highlighted: bool,
) -> Rectangle {
    let ssid = String::from(access_point.ssid());

    let joining = panel.connecting_to() == Some(access_point.ssid());
    let expanded = panel.expanded() == Some(access_point.ssid());

    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Rectangle::new()
                .width(LINE)
                .height(ROW)
                .align_child(Center, Center)
                .child(
                    Text::new(network::strength_icon(access_point.strength()))
                        .size(ui::ICON_SIZE)
                        .color(t.fg_dim),
                ),
        ),
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(ROW)
                .align_child(Start, Center)
                .child(
                    Text::new(access_point.ssid())
                        .size(ui::TEXT_BODY)
                        .color(t.fg)
                        .elide(),
                ),
        ),
    ];

    if joining {
        parts.push(Box::new(
            Text::new("Joining…").size(ui::TEXT_SMALL).color(t.fg_dim),
        ));
    }

    if access_point.saved() {
        parts.push(Box::new(
            Text::new("Saved").size(ui::TEXT_SMALL).color(t.comment),
        ));
    }

    if access_point.secured() {
        parts.push(Box::new(
            Text::new(ICON_LOCK).size(ui::TEXT_NOTE).color(t.comment),
        ));
    }

    if access_point.saved() {
        let forgotten = ssid.clone();
        let forget = move |_| NetworkPanel::forget(&forgotten);

        // the keys act on the highlighted row, so only its button shows f under ?
        parts.push(Box::new(if highlighted {
            ui::key_text_button(t, "Forget", 64.0, KEY_FORGET, forget)
        } else {
            button(t, "Forget", 64.0, forget)
        }));
    }

    let fill = if highlighted {
        t.bg3
    } else if expanded {
        t.bg2
    } else {
        t.bg0
    };

    let clicked = access_point.clone();

    Rectangle::new()
        .width(Parent)
        .height(ROW)
        .fill(fill)
        .padding(SIDES)
        .cursor(Pointer)
        .on_click(move |_| click(&clicked))
        .child(Row::new(parts).gap(GAP).align(Center).height(ROW))
}

fn password_row(t: &Theme, ssid: &str) -> Rectangle {
    let submitted = String::from(ssid);
    let clicked = String::from(ssid);

    let input = Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .fill(t.bg1)
        .padding(Padding {
            top: 0.0,
            right: 8.0,
            bottom: 0.0,
            left: 8.0,
        })
        .align_child(Start, Center)
        .child(
            TextInput::new(PASSWORD_INPUT)
                .placeholder("Password")
                .password()
                .focused()
                .size(ui::TEXT_BODY)
                .color(t.fg)
                .on_change(|text| {
                    *PASSWORD.lock().unwrap_or_else(|error| error.into_inner()) = text;
                })
                .on_submit(move |password| join(&submitted, &password)),
        );

    let connect = button(t, "Connect", 84.0, move |_| {
        let password = PASSWORD
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();

        join(&clicked, &password);
    });

    Rectangle::new()
        .width(Parent)
        .height(PASSWORD_ROW)
        .fill(t.bg2)
        .padding(SIDES)
        .align_child(Start, Center)
        .child(
            Row::new(children![input, connect])
                .gap(GAP)
                .align(Center)
                .height(BUTTON),
        )
}

fn join(ssid: &str, password: &str) {
    if password.is_empty() {
        return;
    }

    clear_password();

    NetworkPanel::connect(ssid, Some(password));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(status: Status, joined: Option<&str>, joining: Option<&str>) -> (String, String) {
        connection_text(&status, joined, joining)
    }

    #[test]
    fn the_card_names_the_connection_and_its_state() {
        assert_eq!(
            text(Status::Wired, None, None),
            (String::from("Ethernet"), String::from("Connected"))
        );
        assert_eq!(
            text(Status::NoInternet, Some("home"), None),
            (String::from("home"), String::from("No internet"))
        );
        assert_eq!(
            text(Status::Offline, None, None),
            (String::from("Not connected"), String::new())
        );
    }

    #[test]
    fn while_joining_the_name_is_the_network_being_joined() {
        assert_eq!(
            text(Status::NoInternet, Some("home"), Some("cafe")),
            (String::from("cafe"), String::from("Connecting…"))
        );
    }

    #[test]
    fn the_details_line_leaves_out_what_is_unknown() {
        let mut details = Details {
            interface: String::from("wlp2s0"),
            address: String::from("192.168.1.23/24"),
            gateway: String::from("192.168.1.1"),
        };

        assert_eq!(
            details_line(&details),
            "wlp2s0 · 192.168.1.23/24 · gw 192.168.1.1"
        );

        details.address.clear();
        details.gateway.clear();

        assert_eq!(details_line(&details), "wlp2s0");
    }
}
