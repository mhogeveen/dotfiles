use amane::{
    Align::{Center, End},
    Cursor::Pointer,
    Full, Layer, LayerWindow, Monitor, Notifications, Padding, Rectangle, Row, Service,
    Size::Parent,
    Text, Vertical, Widget, Workspaces, Zone, children,
};

use crate::{
    clock,
    notifications::Seen,
    panels::{NOTIFICATIONS, Panels, SESSION},
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
        .child(
            Row::new(children![notifications(t), session(t)])
                .gap(8.0)
                .align(Center),
        )
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

    if unread > 0 {
        parts.push(Box::new(
            Text::new(unread.to_string()).size(12.0).color(t.fg),
        ));
    }

    Rectangle::new()
        .width(48.0)
        .height(Parent)
        .align_child(End, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(NOTIFICATIONS);
        })
        .child(Row::new(parts).gap(4.0).align(Center))
}

// opens the session menu: lock, suspend, logout, reboot, shutdown
fn session(t: &Theme) -> Rectangle {
    Rectangle::new()
        .width(24.0)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(SESSION);
        })
        .child(Text::new("\u{f0425}").size(14.0).color(t.fg_dim))
}
