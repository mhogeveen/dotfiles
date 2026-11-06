/*
 * session menu: lock, suspend, logout, reboot, shutdown; press a number to run
 * one, or pick with up/down or tab and press enter, or click
 *
 * logout, reboot and shutdown ask again: the first choice arms the row, choosing
 * the same row again runs it, anything else disarms it
 *
 * the actions themselves are in src/session.rs
 */

use std::time::Duration;

use amane::{Cursor::Pointer, Key, LayerWindow, Monitor, Service, Vertical, Widget};

use super::list::{self, Cursor};
use super::{SESSION, panel_window};
use crate::session::{ACTIONS, Action};
use crate::theme::theme;

struct Session {
    cursor: Cursor,

    // the action waiting to be chosen again, as an index into ACTIONS
    armed: Option<usize>,
}

impl Service for Session {
    fn new() -> Self {
        Self {
            cursor: Cursor::default(),
            armed: None,
        }
    }

    // only changes through input
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();

    let (cursor, armed) = {
        let session = Session::read();

        (session.cursor, session.armed)
    };

    let count = ACTIONS.len();
    let selected = cursor.selected(count);
    let visible = cursor.visible(count);
    let first = visible.start;

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for index in visible {
        let action = ACTIONS[index];
        let number = index - first + 1;

        let row = if armed == Some(index) {
            let label = format!("{} — press {number} or Enter again", action.label());

            list::armed_row(&t, number, &label)
        } else {
            list::numbered_row(&t, number, action.label(), index == selected)
        };

        rows.push(Box::new(
            row.cursor(Pointer).on_click(move |_| choose(index)),
        ));
    }

    // no search box, so number keys reach key_pressed instead of being typed
    let base = LayerWindow::new()
        .width(list::width())
        .height(list::height(count.min(list::ROWS), false))
        .anchor_vertical(Vertical::Middle)
        .namespace("amane-session")
        .child(list::frame(&t, None, list::rows(rows)));

    panel_window(monitor, SESSION, base, key_pressed)
}

// escape is handled by the panel manager
fn key_pressed(key: Key) {
    let count = ACTIONS.len();
    let cursor = Session::read().cursor;

    if let Some(position) = list::hotkey(key) {
        if let Some(index) = cursor.at_position(position, count) {
            choose(index);
        }

        return;
    }

    let delta = match key {
        Key::Down | Key::Tab => 1,
        Key::Up => -1,
        Key::Enter => return choose(cursor.selected(count)),
        _ => return,
    };

    // moving away disarms, so a stray enter later can't shut down
    let mut session = Session::write();

    session.cursor.step(delta, count);
    session.armed = None;
}

/*
 * runs the action, or arms it first when it asks for confirmation;
 * the highlight follows a click or number key too
 */
fn choose(index: usize) {
    let action: Action = ACTIONS[index];

    let run = {
        let mut session = Session::write();

        let confirmed = session.armed == Some(index);

        if action.confirms() && !confirmed {
            session.armed = Some(index);
            session.cursor.select(index, ACTIONS.len());

            false
        } else {
            session.armed = None;

            true
        }
    };

    // the Session lock is released first: running closes the panel, whose hook writes it
    if run {
        action.run();
    }
}

// starts at the top and disarmed the next time the menu opens
pub fn reset() {
    let mut session = Session::write();

    session.cursor = Cursor::default();
    session.armed = None;
}
