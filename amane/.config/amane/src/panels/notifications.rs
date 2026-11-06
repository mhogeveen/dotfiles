/*
 * notification center: every notification still around, newest first, with
 * a dismiss button on each and a "Clear all" for the lot
 */

use std::time::SystemTime;

use amane::{
    Align::{Center, Start},
    Apps, Column,
    Cursor::Pointer,
    Horizontal, Key, LayerWindow, Margin, Monitor, Notifications, Rectangle, Row, ScrollArea,
    Service,
    Size::Parent,
    Text, Vertical, Weight, Widget, children,
};

use super::{NOTIFICATIONS, panel_window};
use crate::clock::Clock;
use crate::notifications::{
    Seen,
    card::{self, Options},
};
use crate::theme::theme;

const WIDTH: f32 = 400.0;
const HEIGHT: f32 = 600.0;
const PADDING: f32 = 12.0;
const GAP: f32 = 8.0;
const HEADER: f32 = 28.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();
    let notifications = Notifications::read();
    let apps = Apps::read();

    // redraws once a minute, so "5m ago" keeps up
    let _clock = Clock::read();

    let now = SystemTime::now();
    let card_width = WIDTH - PADDING * 2.0;

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    for notification in notifications.list().iter().rev() {
        let id = notification.id();

        let options = Options {
            width: card_width,
            body_lines: 6,
            now: Some(now),
        };

        let (card, _) = card::card(&t, &apps, notification, options, move |_| {
            Notifications::dismiss(id)
        });

        cards.push(Box::new(card));
    }

    if cards.is_empty() {
        let text = if notifications.running() {
            "No notifications"
        } else {
            "Another notification daemon is running"
        };

        cards.push(Box::new(card::message(&t, card_width, text)));
    }

    let clear = Rectangle::new()
        .width(80.0)
        .height(HEADER)
        .fill(t.bg2)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| Notifications::clear())
        .child(Text::new("Clear all").size(13.0).color(t.fg));

    let title = Rectangle::new()
        .width(Parent)
        .height(HEADER)
        .align_child(Start, Center)
        .child(
            Text::new("Notifications")
                .size(15.0)
                .weight(Weight::Bold)
                .color(t.fg),
        );

    let header = Row::new(children![title, clear]).height(HEADER);

    let list = ScrollArea::new("notification-center", Column::new(cards).gap(GAP));

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
        .namespace("amane-notification-center")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(t.bg_deep)
                .border(1.0, t.bg3)
                .padding(PADDING)
                .child(Column::new(children![header, list]).gap(GAP)),
        );

    panel_window(monitor, NOTIFICATIONS, base, |_: Key| {})
}

// everything shown while it was open counts as read
pub fn reset() {
    Seen::mark_all();
}
