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
    Align::{Center, Start},
    Column,
    Cursor::Pointer,
    Horizontal, Key, LayerWindow, Margin, Monitor, Network, Padding, Rectangle, Row, ScrollArea,
    Service,
    Size::Parent,
    Text, TextInput, Vertical, Weight, Widget, children,
};

use super::ui::{self, BUTTON, button};
use super::{NETWORK, panel_window};
use crate::network::{self, ICON_LOCK, ICON_RESCAN, NetworkPanel, Outcome, Status};
use crate::theme::{Theme, theme};

const WIDTH: f32 = 400.0;
const HEIGHT: f32 = 560.0;
const PADDING: f32 = 12.0;
const GAP: f32 = 8.0;
const LINE: f32 = 20.0;
const ROW: f32 = 40.0;

const PASSWORD_INPUT: &str = "network-password";

/*
 * what's typed in the password box, for the Connect button; the box keeps its
 * own text, this copy needs no redraw so it isn't a service
 */
static PASSWORD: Mutex<String> = Mutex::new(String::new());

const SIDES: Padding = Padding {
    top: 0.0,
    right: PADDING,
    bottom: 0.0,
    left: PADDING,
};

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();
    let network = Network::read();
    let panel = NetworkPanel::read();

    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(header(&t, &network)),
        Box::new(connection(&t, &network, &panel)),
    ];

    if let Some(result) = panel.result() {
        parts.push(Box::new(result_line(&t, result)));
    }

    parts.push(Box::new(networks(&t, &network, &panel)));

    let base = LayerWindow::new()
        .width(WIDTH)
        .height(HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 8,
            right: 8,
            bottom: 0,
            left: 0,
        })
        .namespace("amane-network")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(t.bg_deep)
                .border(1.0, t.bg3)
                .padding(PADDING)
                .child(Column::new(parts).gap(GAP)),
        );

    panel_window(monitor, NETWORK, base, |_: Key| {})
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

    let rescan = Rectangle::new()
        .width(BUTTON)
        .height(BUTTON)
        .fill(t.bg2)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| Network::scan())
        .child(Text::new(ICON_RESCAN).size(14.0).color(t.fg));

    let enabled = network.wifi_enabled();

    let switch = ui::switch(t, "Wi-Fi", enabled, move |_| Network::set_wifi(!enabled));

    Row::new(children![title, rescan, switch])
        .gap(GAP)
        .align(Center)
        .height(BUTTON)
}

// the current connection: what it is, how it's doing, its address, and buttons
fn connection(t: &Theme, network: &Network, panel: &NetworkPanel) -> Rectangle {
    let status = network::status(network);
    let joined = network::joined(network);

    let name = match (&status, &joined) {
        (Status::Wired, _) => String::from("Ethernet"),
        (_, Some(ssid)) => ssid.clone(),
        (Status::Other, None) => String::from("Connected"),
        _ => String::from("Not connected"),
    };

    let state = match (panel.connecting_to(), &status) {
        (Some(ssid), _) => format!("Connecting to {ssid}…"),
        (None, Status::Wifi(_) | Status::Wired | Status::Other) => String::from("Connected"),
        (None, Status::NoInternet) => String::from("No internet"),
        (None, Status::WifiOff) => String::from("Wi-Fi is off"),
        (None, Status::Connecting) => String::from("Connecting…"),
        (None, Status::Offline) => String::new(),
    };

    let state_color = match status {
        Status::NoInternet => t.brass,
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
        Text::new(state).size(12.0).color(state_color),
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
            .child(Text::new(text).size(13.0).color(t.cinder).elide()),
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
