/*
 * notification center: every notification still around, newest first, with
 * a dismiss button on each and a "Clear all" for the lot
 *
 * under "Earlier", the ones from before the shell last restarted
 * (src/notifications/history.rs): read-only, their senders can't be reached
 */

use std::time::SystemTime;

use amane::{
    Align::Center, Apps, Column, Key, LayerWindow, Monitor, Notifications, Rectangle, Row,
    ScrollArea, Service, Widget, children,
};

use super::ui::{self, BUTTON, CORNER_WIDTH, PADDING};
use super::{NOTIFICATIONS, panel};
use crate::clock::Clock;
use crate::notifications::{
    Seen,
    card::{self, Content, Options},
    history::{self, History},
};
use crate::theme::Theme;

const HEIGHT: f32 = 600.0;
const GAP: f32 = 8.0;

// the "Earlier" heading over the restored ones
const HEADING: f32 = 20.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = ui::corner(HEIGHT, "amane-notification-center");

    panel(monitor, NOTIFICATIONS, base, key_pressed, content)
}

const KEY_CLEAR: char = 'c';

fn key_pressed(key: Key) {
    if super::letter(&key) == Some(KEY_CLEAR) {
        clear_all();
    }
}

// the live ones and the restored ones alike
fn clear_all() {
    Notifications::clear();
    history::clear();
}

fn content(t: &Theme) -> Rectangle {
    let notifications = Notifications::read();
    let history = History::read();
    let apps = Apps::read();

    // redraws once a minute, so "5m ago" keeps up
    let _clock = Clock::read();

    let now = SystemTime::now();
    let card_width = CORNER_WIDTH - PADDING * 2.0;

    let options = || Options {
        width: card_width,
        body_lines: 6,
        now: Some(now),
    };

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    for notification in notifications.list().iter().rev() {
        let id = notification.id();

        let (card, _) = card::card(
            t,
            Content::live(notification, &apps),
            options(),
            move |_| Notifications::dismiss(id),
        );

        cards.push(Box::new(card));
    }

    let restored = history.restored();

    if !restored.is_empty() {
        cards.push(Box::new(ui::heading(t, "Earlier", HEADING)));
    }

    for (index, kept) in restored.iter().enumerate().rev() {
        let (card, _) = card::card(t, Content::kept(kept, &apps), options(), move |_| {
            history::dismiss(index)
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
        ui::key_text_button(t, "Clear all", ui::fit("Clear all"), KEY_CLEAR, |_| {
            clear_all()
        }),
    ])
    .align(Center)
    .height(BUTTON);

    let list = ScrollArea::new("notification-center", Column::new(cards).gap(GAP));

    ui::frame(t, GAP, children![header, list])
}

// everything shown while it was open counts as read
pub fn reset() {
    Seen::mark_all();
}
