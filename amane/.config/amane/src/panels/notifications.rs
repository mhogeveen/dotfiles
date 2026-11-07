/*
 * notification center: every notification still around, newest first, with
 * a dismiss button on each and a "Clear all" for the lot
 */

use std::time::SystemTime;

use amane::{
    Align::Center, Apps, Column, Key, LayerWindow, Monitor, Notifications, Rectangle, Row,
    ScrollArea, Service, Widget, children,
};

use super::ui::{self, BUTTON, CORNER_WIDTH, PADDING, button};
use super::{NOTIFICATIONS, panel};
use crate::clock::Clock;
use crate::notifications::{
    Seen,
    card::{self, Options},
};
use crate::theme::Theme;

const HEIGHT: f32 = 600.0;
const GAP: f32 = 8.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = ui::corner(HEIGHT, "amane-notification-center");

    panel(monitor, NOTIFICATIONS, base, |_: Key| {}, content)
}

fn content(t: &Theme) -> Rectangle {
    let notifications = Notifications::read();
    let apps = Apps::read();

    // redraws once a minute, so "5m ago" keeps up
    let _clock = Clock::read();

    let now = SystemTime::now();
    let card_width = CORNER_WIDTH - PADDING * 2.0;

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    for notification in notifications.list().iter().rev() {
        let id = notification.id();

        let options = Options {
            width: card_width,
            body_lines: 6,
            now: Some(now),
        };

        let (card, _) = card::card(t, &apps, notification, options, move |_| {
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

        cards.push(Box::new(card::message(t, card_width, text)));
    }

    let header = Row::new(children![
        ui::title(t, "Notifications"),
        button(t, "Clear all", 80.0, |_| Notifications::clear()),
    ])
    .align(Center)
    .height(BUTTON);

    let list = ScrollArea::new("notification-center", Column::new(cards).gap(GAP));

    ui::corner_frame(t, GAP, children![header, list])
}

// everything shown while it was open counts as read
pub fn reset() {
    Seen::mark_all();
}
