/*
 * network panel, top-right: the current connection with its address, the wi-fi
 * switch, and the networks in range; click one to join it, a new secured one
 * asks for its password first
 *
 * the state behind it (connect results, details, forgetting) is in src/network
 */

use std::sync::Mutex;

use amane::{
    AccessPoint,
    Align::{Center, End, Start},
    Column,
    Cursor::Pointer,
    Key, LayerWindow, Monitor, Network, Padding, Rectangle, Row, ScrollArea, Service,
    Size::Parent,
    Text, TextInput, Weight, Widget, children,
};

use super::ui::{self, BUTTON, PADDING, SIDES, button};
use super::{NETWORK, panel};
use crate::network::{self, ICON_LOCK, ICON_RESCAN, NetworkPanel, Outcome, Status};
use crate::theme::Theme;
use crate::vpn::{self, Backend, Status as Tailscale, Suggestion, Vpn, tailscale};

const HEIGHT: f32 = 560.0;
const GAP: f32 = 8.0;
const LINE: f32 = 20.0;
const ROW: f32 = 40.0;

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

// the header's buttons by their letters; a password being typed keeps the letters for itself
fn key_pressed(key: Key) {
    let Key::Character(letter) = key else {
        return;
    };

    match letter.to_ascii_lowercase() {
        KEY_WIFI => toggle_wifi(),
        KEY_RESCAN => Network::scan(),
        KEY_TAILSCALE if tailscale().0 => Vpn::toggle_tailscale(),
        KEY_LOG_IN if tailscale().1 => Vpn::log_in(),
        _ => {}
    }
}

// whether tailscale is there, and whether it waits for a login; read and let go before acting
fn tailscale() -> (bool, bool) {
    let vpn = Vpn::read();

    let status = vpn.tailscale();

    (status.is_some(), status.is_some_and(needs_login))
}

fn toggle_wifi() {
    let enabled = Network::read().wifi_enabled();

    Network::set_wifi(!enabled);
}

fn content(t: &Theme) -> Rectangle {
    let network = Network::read();
    let panel = NetworkPanel::read();

    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(header(t, &network)),
        Box::new(connection(t, &network, &panel)),
    ];

    // tailscale, and any other vpn there is
    parts.extend(vpn_parts(t));

    if let Some(result) = panel.result() {
        parts.push(Box::new(result_line(t, result)));
    }

    parts.push(Box::new(networks(t, &network, &panel)));

    ui::frame(t, GAP, parts)
}

// closes the password box and forgets what was typed in it
pub fn reset() {
    clear_password();

    NetworkPanel::collapse();
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
    let title = ui::title(t, "Network");

    let rescan = ui::key_button(t, ICON_RESCAN, KEY_RESCAN, |_| Network::scan());

    let enabled = network.wifi_enabled();

    let switch = ui::switch(t, "Wi-Fi", 92.0, enabled, KEY_WIFI, |_| toggle_wifi());

    Row::new(children![title, rescan, switch])
        .gap(GAP)
        .align(Center)
        .height(BUTTON)
}

// the current connection: what it is, how it's doing, its address, and buttons
fn connection(t: &Theme, network: &Network, panel: &NetworkPanel) -> Rectangle {
    let status = network::status(network);
    let joined = network::joined(network);

    /*
     * while joining, the name is the network being joined, so the state on the
     * right stays short and can't push the name out of the row
     */
    let name = match (panel.connecting_to(), &status, &joined) {
        (Some(ssid), _, _) => String::from(ssid),
        (None, Status::Wired, _) => String::from("Ethernet"),
        (None, _, Some(ssid)) => ssid.clone(),
        (None, Status::Other, None) => String::from("Connected"),
        _ => String::from("Not connected"),
    };

    let state = match (panel.connecting_to(), &status) {
        (Some(_), _) => String::from("Connecting…"),
        (None, Status::Wifi(_) | Status::Wired | Status::Other) => String::from("Connected"),
        (None, Status::NoInternet) => String::from("No internet"),
        (None, Status::WifiOff) => String::from("Wi-Fi is off"),
        (None, Status::Connecting) => String::from("Connecting…"),
        (None, Status::Offline) => String::new(),
    };

    let state_color = match status {
        Status::NoInternet => t.accent,
        _ => t.fg_dim,
    };

    let top = Row::new(children![
        Rectangle::new()
            .width(LINE)
            .height(LINE)
            .align_child(Center, Center)
            .child(Text::new(status.icon()).size(16.0).color(t.fg)),
        Rectangle::new()
            .width(Parent)
            .height(LINE)
            .align_child(Start, Center)
            .child(
                Text::new(name)
                    .size(14.0)
                    .weight(Weight::Bold)
                    .color(t.fg)
                    .elide()
            ),
        Rectangle::new()
            .width(STATE_WIDTH)
            .height(LINE)
            .align_child(End, Center)
            .child(Text::new(state).size(12.0).color(state_color)),
    ])
    .gap(GAP)
    .align(Center)
    .height(LINE);

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(top)];
    let mut height = PADDING * 2.0 + LINE;

    if let Some(details) = panel.details() {
        let line = [
            details.interface.clone(),
            details.address.clone(),
            if details.gateway.is_empty() {
                String::new()
            } else {
                format!("gw {}", details.gateway)
            },
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");

        if !line.is_empty() {
            height += GAP + LINE;

            parts.push(Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(LINE)
                    .align_child(Start, Center)
                    .child(Text::new(line).size(12.0).color(t.comment).elide()),
            ));
        }
    }

    // a wi-fi network you're on can be left or forgotten
    if let Some(ssid) = joined {
        height += GAP + BUTTON;

        let forgotten = ssid.clone();

        parts.push(Box::new(
            Row::new(children![
                button(t, "Disconnect", 100.0, |_| NetworkPanel::disconnect()),
                button(t, "Forget", 80.0, move |_| NetworkPanel::forget(&forgotten)),
            ])
            .gap(GAP)
            .height(BUTTON),
        ));
    }

    Rectangle::new()
        .width(Parent)
        .height(height)
        .fill(t.bg0)
        .border(1.0, t.bg3)
        .padding(PADDING)
        .child(Column::new(parts).gap(GAP))
}

// what went wrong last, with a way out when there is one
fn result_line(t: &Theme, result: &Outcome) -> Row {
    let (text, forget) = match result {
        Outcome::Failed(ssid) => (format!("Couldn't connect to {ssid}"), Some(ssid.clone())),
        Outcome::ForgetFailed(ssid, reason) => (format!("Couldn't forget {ssid}: {reason}"), None),
    };

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(
        Rectangle::new()
            .width(Parent)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new(text).size(13.0).color(t.accent).elide()),
    )];

    // forgetting a network saved with a mistyped password lets you type it again
    if let Some(ssid) = forget {
        parts.push(Box::new(button(t, "Forget", 80.0, move |_| {
            NetworkPanel::forget(&ssid)
        })));
    }

    Row::new(parts).gap(GAP).align(Center).height(BUTTON)
}

// every network in range except the one you're on
fn networks(t: &Theme, network: &Network, panel: &NetworkPanel) -> Rectangle {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    let message = if !network.wifi_enabled() {
        Some("Wi-Fi is off")
    } else if network.access_points().iter().all(AccessPoint::active) {
        Some("No networks found")
    } else {
        None
    };

    if let Some(message) = message {
        rows.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(ROW)
                .padding(SIDES)
                .align_child(Start, Center)
                .child(Text::new(message).size(13.0).color(t.comment)),
        ));
    }

    for access_point in network.access_points() {
        if access_point.active() || !network.wifi_enabled() {
            continue;
        }

        rows.push(Box::new(network_row(t, access_point, panel)));

        if panel.expanded() == Some(access_point.ssid()) {
            rows.push(Box::new(password_row(t, access_point.ssid())));
        }
    }

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(t.bg0)
        .border(1.0, t.bg3)
        .clip()
        .child(ScrollArea::new("network-list", Column::new(rows)))
}

fn network_row(t: &Theme, access_point: &AccessPoint, panel: &NetworkPanel) -> Rectangle {
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
                        .size(16.0)
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
                        .size(14.0)
                        .color(t.fg)
                        .elide(),
                ),
        ),
    ];

    if joining {
        parts.push(Box::new(Text::new("Joining…").size(12.0).color(t.fg_dim)));
    }

    if access_point.saved() {
        parts.push(Box::new(Text::new("Saved").size(12.0).color(t.comment)));
    }

    if access_point.secured() {
        parts.push(Box::new(Text::new(ICON_LOCK).size(13.0).color(t.comment)));
    }

    if access_point.saved() {
        let forgotten = ssid.clone();

        parts.push(Box::new(button(t, "Forget", 64.0, move |_| {
            NetworkPanel::forget(&forgotten)
        })));
    }

    // saved and open networks join right away, a new secured one asks first
    let needs_password = access_point.secured() && !access_point.saved();

    Rectangle::new()
        .width(Parent)
        .height(ROW)
        .fill(if expanded { t.bg2 } else { t.bg0 })
        .padding(SIDES)
        .cursor(Pointer)
        .on_click(move |_| {
            if needs_password {
                clear_password();

                NetworkPanel::toggle_expanded(&ssid);
            } else {
                NetworkPanel::connect(&ssid, None);
            }
        })
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
                .size(14.0)
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
        .height(ROW + 4.0)
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

const CARD_TITLE: f32 = 20.0;
const CARD_LINE: f32 = 16.0;
const ACTION_WIDTH: f32 = 100.0;

// the tailscale card, then a line for every other tunnel
fn vpn_parts(t: &Theme) -> Vec<Box<dyn Widget>> {
    let vpn = Vpn::read();

    let mut parts: Vec<Box<dyn Widget>> = Vec::new();

    if let Some(status) = vpn.tailscale() {
        parts.push(Box::new(tailscale_card(t, &vpn, status)));
    }

    for tunnel in vpn.others() {
        let all = vpn.all_traffic() == Some(tunnel);

        let mut line = format!("VPN {} · {}", tunnel.interface, tunnel.kind.name());

        if all {
            line.push_str(" · all traffic");
        }

        parts.push(Box::new(
            Row::new(children![
                Text::new(vpn::ICON_SHIELD)
                    .size(13.0)
                    .color(if all { t.accent } else { t.fg_dim }),
                Rectangle::new()
                    .width(Parent)
                    .height(LINE)
                    .align_child(Start, Center)
                    .child(Text::new(line).size(12.0).color(t.fg_dim).elide()),
            ])
            .gap(GAP)
            .align(Center)
            .height(LINE),
        ));
    }

    parts
}

/*
 *  Tailscale@prometheus                       Connected
 *  100.67.176.75
 *  <the last error, click to dismiss>
 *  [ Disconnect ]  [ Log in ]
 *  Exit node   [ none ] [ nas ] [ vps-ams ]
 */
fn tailscale_card(t: &Theme, vpn: &Vpn, status: &Tailscale) -> Rectangle {
    let connected = status.connected();

    let state = if vpn.busy() {
        "Working…"
    } else {
        match status.backend {
            Backend::Running => "Connected",
            Backend::Stopped => "Disconnected",
            Backend::NeedsLogin => "Log in needed",
            Backend::Starting => "Starting…",
            Backend::Other(_) => "",
        }
    };

    // before logging in there's no name yet
    let title = if status.name.is_empty() {
        String::from("Tailscale")
    } else {
        format!("Tailscale@{}", status.name)
    };

    let top = Row::new(children![
        Rectangle::new()
            .width(Parent)
            .height(CARD_TITLE)
            .align_child(Start, Center)
            .child(
                Text::new(title)
                    .size(14.0)
                    .weight(Weight::Bold)
                    .color(t.fg)
                    .elide()
            ),
        Text::new(state).size(12.0).color(t.fg_dim),
    ])
    .gap(GAP)
    .align(Center)
    .height(CARD_TITLE);

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(top)];
    let mut height = PADDING * 2.0 + CARD_TITLE;

    if connected {
        height += GAP + CARD_LINE;

        parts.push(Box::new(card_line(status.ip.clone(), t.comment)));
    }

    if let Some(error) = vpn.error() {
        height += GAP + CARD_LINE;

        parts.push(Box::new(
            card_line(String::from(error), t.accent)
                .cursor(Pointer)
                .on_click(|_| Vpn::dismiss_error()),
        ));
    }

    let label = if status.active() {
        "Disconnect"
    } else {
        "Connect"
    };

    let mut actions: Vec<Box<dyn Widget>> = vec![Box::new(ui::key_text_button(
        t,
        label,
        ACTION_WIDTH,
        KEY_TAILSCALE,
        |_| Vpn::toggle_tailscale(),
    ))];

    // tailscale hands out a login page instead of asking for a password
    if needs_login(status) {
        actions.push(Box::new(ui::key_text_button(
            t,
            "Log in",
            ACTION_WIDTH,
            KEY_LOG_IN,
            |_| Vpn::log_in(),
        )));
    }

    height += GAP + BUTTON;

    parts.push(Box::new(Row::new(actions).gap(GAP).height(BUTTON)));

    if connected {
        let (column, column_height) = exit_nodes(t, vpn);

        height += GAP + column_height;

        parts.push(Box::new(column));
    }

    Rectangle::new()
        .width(Parent)
        .height(height)
        .fill(t.bg0)
        .border(1.0, t.bg3)
        .padding(PADDING)
        .child(Column::new(parts).gap(GAP))
}

fn needs_login(status: &Tailscale) -> bool {
    status.backend == Backend::NeedsLogin && !status.auth_url.is_empty()
}

fn card_line(text: String, color: amane::Color) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(CARD_LINE)
        .align_child(Start, Center)
        .child(Text::new(text).size(12.0).color(color).elide())
}

/*
 *  Exit node
 *  [ None                              ]
 *  [ Suggested · nl-ams-wg-303         ]
 *  [ In use · se-sto-wg-005            ]   only when that isn't the suggestion
 *
 * the tailnet can offer hundreds of exit nodes (mullvad's), so instead of a list
 * there's what tailscale suggests, and the one in use; that one in the accent
 */
fn exit_nodes(t: &Theme, vpn: &Vpn) -> (Column, f32) {
    let in_use = vpn.exit_node().map(String::from);

    let label = Rectangle::new()
        .width(Parent)
        .height(CARD_LINE)
        .align_child(Start, Center)
        .child(Text::new("Exit node").size(12.0).color(t.fg_dim));

    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(label),
        Box::new(exit_button(
            t,
            String::from("None"),
            in_use.is_none(),
            Some(None),
        )),
    ];

    let suggested = match vpn.suggestion() {
        Suggestion::Node(node) => Some(node.clone()),
        _ => None,
    };

    match vpn.suggestion() {
        Suggestion::Unknown => parts.push(Box::new(exit_button(
            t,
            String::from("Suggested · …"),
            false,
            None,
        ))),

        Suggestion::Node(node) => {
            let chosen = in_use.as_deref() == Some(node.as_str());

            parts.push(Box::new(exit_button(
                t,
                format!("Suggested · {}", tailscale::short_name(node)),
                chosen,
                Some(Some(node.clone())),
            )));
        }

        // nothing to suggest, like when no exit node is allowed
        Suggestion::None => {}
    }

    // what carries the traffic is always on the card, even when it wasn't the suggestion
    if let Some(node) = &in_use
        && suggested.as_deref() != Some(node.as_str())
    {
        parts.push(Box::new(exit_button(
            t,
            format!("In use · {}", tailscale::short_name(node)),
            true,
            None,
        )));
    }

    let buttons = parts.len() - 1;

    let height = CARD_LINE + buttons as f32 * (GAP + BUTTON);

    (Column::new(parts).gap(GAP), height)
}

/*
 * a full-width exit node button; target is what clicking sets, Some(None) for
 * no exit node, None for a button that does nothing (the one in use, or a
 * suggestion still loading)
 */
fn exit_button(
    t: &Theme,
    label: String,
    chosen: bool,
    target: Option<Option<String>>,
) -> Rectangle {
    let (fill, color) = if chosen {
        (t.accent, t.on_accent)
    } else {
        (t.bg2, t.fg)
    };

    let button = Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .fill(fill)
        .padding(SIDES)
        .align_child(Start, Center)
        .child(Text::new(label).size(13.0).color(color).elide());

    match target {
        Some(node) if !chosen => button
            .cursor(Pointer)
            .on_click(move |_| Vpn::use_exit_node(node.clone())),
        _ => button,
    }
}
