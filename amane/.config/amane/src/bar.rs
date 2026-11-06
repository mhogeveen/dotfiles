use amane::{
    Align::{Center, End},
    Audio, Bluetooth,
    Cursor::Pointer,
    Full, Layer, LayerWindow, Monitor, Network, Notifications, Padding, Rectangle, Row, Service,
    Size::Parent,
    Text, Vertical, Widget, Workspaces, Zone, children,
};

use crate::{
    bluetooth::{
        self as bt,
        agent::{Agent, Request},
    },
    clock, network as net,
    notifications::Seen,
    panels::{AUDIO, BLUETOOTH, NETWORK, NOTIFICATIONS, Panels, SESSION, audio::volume_icon},
    theme::{Theme, theme},
};

pub fn view(_monitor: &Monitor) -> LayerWindow {
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
                    Row::new(children![workspaces(&t), clock(&t), status(&t)])
                        .height(Parent)
                        .align(Center),
                ),
        )
}

fn workspaces(t: &Theme) -> Rectangle {
    let workspaces = Workspaces::read();

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for workspace in workspaces.list() {
        let (width, fill) = if workspace.focused() {
            (20.0, t.frost)
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

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .child(Row::new(buttons).gap(4.0).align(Center).height(Parent))
}

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

// left to right: audio, network, bluetooth (only on machines that have it), notifications, session
fn status_icons(t: &Theme) -> Vec<Box<dyn Widget>> {
    let mut icons: Vec<Box<dyn Widget>> = vec![Box::new(audio(t)), Box::new(network(t))];

    if let Some(bluetooth) = bluetooth(t) {
        icons.push(Box::new(bluetooth));
    }

    icons.push(Box::new(notifications(t)));
    icons.push(Box::new(session(t)));

    icons
}

/*
 * a bell with the number of notifications since the center was last closed;
 * reading Notifications here, in a window open from the start, is also what
 * makes the shell the notification daemon
 */
fn notifications(t: &Theme) -> Rectangle {
    let notifications = Notifications::read();
    let open = Panels::read().is_open(NOTIFICATIONS);

    let unread = if open {
        0
    } else {
        Seen::read().unread(&notifications)
    };

    let (icon, color) = if unread > 0 {
        ("\u{f009a}", t.brass)
    } else {
        ("\u{f009c}", t.fg_dim)
    };

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(Text::new(icon).size(14.0).color(color))];

    // as wide as the other icons, plus room for the count when there is one
    let mut width = ICON_WIDTH;

    if unread > 0 {
        let count = unread.to_string();

        width += COUNT_GAP + count.len() as f32 * COUNT_DIGIT;

        parts.push(Box::new(Text::new(count).size(COUNT_SIZE).color(t.fg)));
    }

    Rectangle::new()
        .width(width)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(NOTIFICATIONS);
        })
        .child(Row::new(parts).gap(COUNT_GAP).align(Center))
}

// the box every status icon sits in
const ICON_WIDTH: f32 = 24.0;

const COUNT_GAP: f32 = 4.0;
const COUNT_SIZE: f32 = 12.0;

// BlexMono is monospaced, every digit advances 0.6 of the text size
const COUNT_DIGIT: f32 = COUNT_SIZE * 0.6;

// opens the session menu: lock, suspend, logout, reboot, shutdown
fn session(t: &Theme) -> Rectangle {
    Rectangle::new()
        .width(ICON_WIDTH)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(SESSION);
        })
        .child(Text::new("\u{f0425}").size(14.0).color(t.fg_dim))
}

// wi-fi strength, ethernet, no internet, wi-fi off or offline; opens the network panel
fn network(t: &Theme) -> Rectangle {
    let network = Network::read();

    let status = net::status(&network);

    let color = match status {
        net::Status::NoInternet => t.brass,
        net::Status::Offline | net::Status::WifiOff | net::Status::Connecting => t.comment,
        _ => t.fg_dim,
    };

    Rectangle::new()
        .width(ICON_WIDTH)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(NETWORK);
        })
        .child(Text::new(status.icon()).size(14.0).color(color))
}

// off, on, or a device connected; brass while a pairing code waits for an answer
fn bluetooth(t: &Theme) -> Option<Rectangle> {
    let bluetooth = Bluetooth::read();

    if !bluetooth.available() {
        return None;
    }

    let waiting = Agent::read().request().is_some_and(Request::answerable);

    let connected = bluetooth.devices().iter().any(|device| device.connected());

    let (icon, color) = if waiting {
        (bt::ICON_ON, t.brass)
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
