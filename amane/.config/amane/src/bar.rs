use amane::{
    Align::{Center, End},
    Audio, Bluetooth,
    Cursor::Pointer,
    Full, Layer, LayerWindow, Monitor, Network, Padding, Rectangle, Row, Service,
    Size::Parent,
    Stack, Text, Vertical, Widget, Workspaces, Zone, children,
};

use crate::{
    bluetooth::{
        self as bt,
        agent::{Agent, Request},
    },
    clock,
    layout::MangoLayout,
    network as net, notifications,
    panels::{AUDIO, BLUETOOTH, NETWORK, Panels, SYSTEM, audio::volume_icon, layout},
    theme::{Theme, theme},
    vpn::{self, Vpn},
};

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();

    LayerWindow::new()
        .width(Full)
        .height(32.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .space(Zone::Reserve)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(t.bg_deep)
                .padding(Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 8.0,
                })
                .child(
                    // the two Parent-sized sides split the leftover space evenly,
                    // which keeps the fixed-width clock exactly centered
                    Row::new(children![workspaces(&t, monitor), clock(&t), status(&t)])
                        .height(Parent)
                        .align(Center),
                ),
        )
}

fn workspaces(t: &Theme, monitor: &Monitor) -> Rectangle {
    let workspaces = Workspaces::read();

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    /*
     * only this monitor's own workspaces; one the compositor doesn't place on a
     * monitor shows everywhere rather than nowhere
     */
    let own = workspaces.list().iter().filter(|workspace| {
        workspace
            .output()
            .is_none_or(|output| output == monitor.name)
    });

    for workspace in own {
        /*
         * the one shown on this monitor is wide: in the accent when this monitor
         * has focus, dimmer when another one does; an urgent one is a dot in the accent
         */
        let (width, fill) = if workspace.active() {
            let fill = if workspace.focused() {
                t.accent
            } else {
                t.fg_dim
            };

            (20.0, fill)
        } else if workspace.urgent() {
            (8.0, t.accent)
        } else if workspace.windows() > 0 {
            (8.0, t.bg4)
        } else {
            (8.0, t.bg1)
        };

        let id = workspace.id();

        buttons.push(Box::new(
            Rectangle::new()
                .width(width)
                .height(8.0)
                .align_child(Center, Center)
                .fill(fill)
                .cursor(Pointer)
                .on_click(move |_| Workspaces::focus(id)),
        ));
    }

    Rectangle::new().width(Parent).height(Parent).child(
        Row::new(children![
            Row::new(buttons).gap(4.0).align(Center).height(Parent),
            layout(t, monitor),
        ])
        .gap(8.0)
        .align(Center)
        .height(Parent),
    )
}

// this monitor's mango layout like [S]; opens the layout picker on it
fn layout(t: &Theme, monitor: &Monitor) -> Rectangle {
    let symbol = MangoLayout::read().symbol(&monitor.name).map(String::from);

    let text = symbol
        .as_deref()
        .map(|symbol| format!("[{symbol}]"))
        .unwrap_or_default();

    // BlexMono is monospaced, so the width follows from the number of characters
    let width = text.chars().count() as f32 * LAYOUT_SIZE * 0.6;

    Rectangle::new()
        .width(width)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(move |_| layout::toggle_from(symbol.as_deref()))
        .child(Text::new(text).size(LAYOUT_SIZE).color(t.fg_dim))
}

const LAYOUT_SIZE: f32 = 12.0;

fn clock(t: &Theme) -> Rectangle {
    clock::view(t)
}

// the right side; Parent-wide like the workspaces, so the clock stays centered
fn status(t: &Theme) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(End, Center)
        .child(Row::new(status_icons(t)).gap(8.0).align(Center))
}

// left to right: audio, network, bluetooth (only on machines that have it), system
fn status_icons(t: &Theme) -> Vec<Box<dyn Widget>> {
    let mut icons: Vec<Box<dyn Widget>> = vec![Box::new(audio(t)), Box::new(network(t))];

    if let Some(bluetooth) = bluetooth(t) {
        icons.push(Box::new(bluetooth));
    }

    icons.push(Box::new(system(t)));

    icons
}

// the box every status icon sits in
const ICON_WIDTH: f32 = 24.0;

/*
 * the nixos snowflake, in the accent while notifications wait unread; opens the system
 * panel, which has the way into the notification center and the session menu
 */
fn system(t: &Theme) -> Rectangle {
    let color = if notifications::has_unread() {
        t.accent
    } else {
        t.fg_dim
    };

    Rectangle::new()
        .width(ICON_WIDTH)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(SYSTEM);
        })
        .child(Text::new("\u{f313}").size(14.0).color(color))
}

// wi-fi strength, ethernet, no internet, wi-fi off or offline; opens the network panel
fn network(t: &Theme) -> Rectangle {
    let network = Network::read();

    let status = net::status(&network);

    let color = match status {
        net::Status::NoInternet => t.accent,
        net::Status::Offline | net::Status::WifiOff | net::Status::Connecting => t.comment,
        _ => t.fg_dim,
    };

    let icon = Rectangle::new()
        .width(GLYPH)
        .height(GLYPH)
        .align_child(Center, Center)
        .child(Text::new(status.icon()).size(14.0).color(color));

    let mut layers: Vec<Box<dyn Widget>> = vec![Box::new(icon)];

    /*
     * a badge on the icon's top-right corner while a vpn is up: dim while it only
     * reaches its own network, in the accent while all traffic goes through it,
     * like a tailscale exit node. a disc of the bar's own color under the shield
     * cuts it loose from the wifi bars it overlaps
     */
    if let Some(all_traffic) = Vpn::read().shield() {
        let shield = if all_traffic { t.accent } else { t.fg_dim };

        layers.push(Box::new(
            Rectangle::new()
                .width(BADGE)
                .height(BADGE)
                .radius(Full)
                .fill(t.bg_deep)
                .align_child(Center, Center)
                .translate(BADGE_X, BADGE_Y)
                .child(Text::new(vpn::ICON_SHIELD).size(BADGE_GLYPH).color(shield)),
        ));
    }

    Rectangle::new()
        .width(ICON_WIDTH)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(NETWORK);
        })
        .child(Stack::new(layers))
}

// the network glyph's box, and the vpn badge on its top-right corner
const GLYPH: f32 = 16.0;
const BADGE: f32 = 10.0;
const BADGE_GLYPH: f32 = 9.0;
const BADGE_X: f32 = 10.0;
const BADGE_Y: f32 = -4.0;

// off, on, or a device connected; the accent while a pairing code waits for an answer
fn bluetooth(t: &Theme) -> Option<Rectangle> {
    let bluetooth = Bluetooth::read();

    if !bluetooth.available() {
        return None;
    }

    let waiting = Agent::read().request().is_some_and(Request::answerable);

    let connected = bluetooth.devices().iter().any(|device| device.connected());

    let (icon, color) = if waiting {
        (bt::ICON_ON, t.accent)
    } else if !bluetooth.powered() {
        (bt::ICON_OFF, t.comment)
    } else if connected {
        (bt::ICON_CONNECTED, t.fg_dim)
    } else {
        (bt::ICON_ON, t.fg_dim)
    };

    Some(
        Rectangle::new()
            .width(ICON_WIDTH)
            .height(Parent)
            .align_child(Center, Center)
            .cursor(Pointer)
            .on_click(|_| {
                let _ = Panels::toggle(BLUETOOTH);
            })
            .child(Text::new(icon).size(14.0).color(color)),
    )
}

// the speaker by volume, dimmed while muted; opens the audio panel
fn audio(t: &Theme) -> Rectangle {
    let audio = Audio::read();

    let muted = audio.muted() || audio.volume() == 0;

    Rectangle::new()
        .width(ICON_WIDTH)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(AUDIO);
        })
        .child(
            Text::new(volume_icon(audio.volume(), audio.muted()))
                .size(14.0)
                .color(if muted { t.comment } else { t.fg_dim }),
        )
}
