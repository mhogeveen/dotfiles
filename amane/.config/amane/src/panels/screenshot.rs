/*
 * screenshot menu, the same choices as the old fuzzel menu in screenshot.sh:
 * press a choice's number to take it right away, or pick one with up/down or tab
 * and press enter, or click it
 *
 * the capture itself happens in src/screenshot.rs
 */

use std::thread;
use std::time::Duration;

use amane::{Cursor::Pointer, Key, LayerWindow, Monitor, Service, Vertical, Widget};

use super::list::{self, Cursor};
use super::{Panels, SCREENSHOT, panel_window};
use crate::screenshot::{self, Area, Target};
use crate::theme::theme;

/*
 * the menu hides before the capture starts, so it isn't in the screenshot;
 * this is enough for the compositor to have drawn a frame without it
 */
const HIDE_DELAY: Duration = Duration::from_millis(250);

struct Choice {
    label: &'static str,
    area: Area,
    target: Target,
}

static CHOICES: &[Choice] = &[
    Choice {
        label: "Fullscreen (copy)",
        area: Area::Fullscreen,
        target: Target::Clipboard,
    },
    Choice {
        label: "Fullscreen (save)",
        area: Area::Fullscreen,
        target: Target::File,
    },
    Choice {
        label: "Region (copy)",
        area: Area::Region,
        target: Target::Clipboard,
    },
    Choice {
        label: "Region (save)",
        area: Area::Region,
        target: Target::File,
    },
];

// only the highlight; there's nothing to type
struct Screenshot {
    cursor: Cursor,
}

impl Service for Screenshot {
    fn new() -> Self {
        Self {
            cursor: Cursor::default(),
        }
    }

    // only changes through input
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();
    let cursor = Screenshot::read().cursor;

    let count = CHOICES.len();
    let selected = cursor.selected(count);
    let visible = cursor.visible(count);
    let first = visible.start;

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for index in visible {
        let choice = &CHOICES[index];

        rows.push(Box::new(
            list::numbered_row(&t, index - first + 1, choice.label, index == selected)
                .cursor(Pointer)
                .on_click(move |_| take(choice)),
        ));
    }

    // no search box, so number keys reach key_pressed instead of being typed
    let base = LayerWindow::new()
        .width(list::width())
        .height(list::height(count.min(list::ROWS), false))
        .anchor_vertical(Vertical::Middle)
        .namespace("amane-screenshot")
        .child(list::frame(&t, None, list::rows(rows)));

    panel_window(monitor, SCREENSHOT, base, key_pressed)
}

// escape is handled by the panel manager
fn key_pressed(key: Key) {
    let count = CHOICES.len();
    let cursor = Screenshot::read().cursor;

    if let Some(position) = list::hotkey(key) {
        if let Some(index) = cursor.at_position(position, count) {
            take(&CHOICES[index]);
        }

        return;
    }

    match key {
        Key::Down | Key::Tab => Screenshot::write().cursor.step(1, count),
        Key::Up => Screenshot::write().cursor.step(-1, count),
        Key::Enter => take(&CHOICES[cursor.selected(count)]),
        _ => {}
    }
}

// the highlight starts at the top again the next time the menu opens
pub fn reset() {
    Screenshot::write().cursor = Cursor::default();
}

fn take(choice: &'static Choice) {
    let _ = Panels::close(SCREENSHOT);

    let (area, target) = (choice.area, choice.target);

    // slurp waits for you to drag a region, so this can take a while
    thread::spawn(move || {
        thread::sleep(HIDE_DELAY);

        screenshot::take(area, target);
    });
}
