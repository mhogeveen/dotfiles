/*
 * screenshot menu, the same choices as the old fuzzel menu in screenshot.sh:
 * type to filter, up/down or tab to pick, enter or click to take it
 *
 * the capture itself happens in src/screenshot.rs
 */

use std::thread;
use std::time::Duration;

use amane::{Cursor::Pointer, Key, LayerWindow, Monitor, Service, TextInput, Vertical, Widget};

use super::list::{self, Cursor, fuzzy};
use super::{Panels, SCREENSHOT, panel_window};
use crate::screenshot::{self, Area, Target};
use crate::theme::theme;

const INPUT: &str = "screenshot-search";

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

struct Screenshot {
    query: String,
    cursor: Cursor,
}

impl Service for Screenshot {
    fn new() -> Self {
        Self {
            query: String::new(),
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
    let screenshot = Screenshot::read();

    let found = matches(&screenshot.query);
    let selected = screenshot.cursor.selected(found.len());

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for (index, choice) in found.into_iter().enumerate() {
        rows.push(Box::new(
            list::row(&t, choice.label, index == selected)
                .cursor(Pointer)
                .on_click(move |_| take(choice)),
        ));
    }

    if rows.is_empty() {
        rows.push(list::message(&t, "No matches"));
    }

    let input = TextInput::new(INPUT)
        .placeholder("Screenshot")
        .focused()
        .on_change(|text| {
            let mut screenshot = Screenshot::write();

            screenshot.query = text;
            screenshot.cursor = Cursor::default();
        })
        .on_submit(|_| submit());

    // every choice fits, so the list never scrolls
    let base = LayerWindow::new()
        .width(list::width())
        .height(list::height_with(CHOICES.len()))
        .anchor_vertical(Vertical::Middle)
        .namespace("amane-screenshot")
        .child(list::frame(&t, input, list::rows(rows)));

    panel_window(monitor, SCREENSHOT, base, key_pressed)
}

// escape is handled by the panel manager; the search box keeps letters and enter
fn key_pressed(key: Key) {
    let delta = match key {
        Key::Down | Key::Tab => 1,
        Key::Up => -1,
        Key::Enter => return submit(),
        _ => return,
    };

    let count = matches(&Screenshot::read().query).len();

    Screenshot::write().cursor.step(delta, count);
}

// clears the filter for the next time the menu opens
pub fn reset() {
    TextInput::set_text(INPUT, "");

    let mut screenshot = Screenshot::write();

    screenshot.query.clear();
    screenshot.cursor = Cursor::default();
}

fn submit() {
    let choice = {
        let screenshot = Screenshot::read();

        let found = matches(&screenshot.query);

        found.get(screenshot.cursor.selected(found.len())).copied()
    };

    if let Some(choice) = choice {
        take(choice);
    }
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

// best match first; an empty query keeps the menu's own order
fn matches(query: &str) -> Vec<&'static Choice> {
    let query = query.trim();

    if query.is_empty() {
        return CHOICES.iter().collect();
    }

    let mut scored: Vec<(i32, &'static Choice)> = CHOICES
        .iter()
        .filter_map(|choice| fuzzy(query, choice.label).map(|score| (score, choice)))
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));

    scored.into_iter().map(|(_, choice)| choice).collect()
}
