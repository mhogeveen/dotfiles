/*
 * app launcher: type to fuzzy-search the installed programs, up/down or tab to pick,
 * enter or click to start one
 */

use std::time::Duration;

use amane::{
    Apps, Cursor::Pointer, DesktopApp, Key, LayerWindow, Monitor, Service, TextInput, Vertical,
    Widget,
};

use super::list::{self, Cursor, fuzzy};
use super::{LAUNCHER, Panels, panel_window};
use crate::theme::theme;

const INPUT: &str = "launcher-search";

struct Launcher {
    query: String,
    cursor: Cursor,
}

impl Service for Launcher {
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
    let apps = Apps::read();
    let launcher = Launcher::read();

    let found = matches(apps.list(), &launcher.query);
    let selected = launcher.cursor.selected(found.len());

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for index in launcher.cursor.visible(found.len()) {
        let app = found[index];
        let launched = app.clone();

        rows.push(Box::new(
            list::row(&t, app.name(), index == selected)
                .cursor(Pointer)
                .on_click(move |_| launch(&launched)),
        ));
    }

    if rows.is_empty() {
        // the first scan walks every icon theme, which takes a few seconds
        let text = if apps.list().is_empty() {
            "Loading applications…"
        } else {
            "No matches"
        };

        rows.push(list::message(&t, text));
    }

    let input = TextInput::new(INPUT)
        .placeholder("Search applications")
        .focused()
        .on_change(|text| {
            let mut launcher = Launcher::write();

            launcher.query = text;
            launcher.cursor = Cursor::default();
        })
        .on_submit(|_| submit());

    let rows = list::rows(rows).on_scroll(|scroll| {
        let count = count();

        Launcher::write()
            .cursor
            .scroll(list::scroll_delta(scroll), count);
    });

    let base = LayerWindow::new()
        .width(list::width())
        .height(list::height(list::ROWS, true))
        .anchor_vertical(Vertical::Middle)
        .namespace("amane-launcher")
        .child(list::frame(&t, Some(input), rows));

    panel_window(monitor, LAUNCHER, base, key_pressed)
}

// escape is handled by the panel manager; the search box keeps letters and enter
fn key_pressed(key: Key) {
    let delta = match key {
        Key::Down | Key::Tab => 1,
        Key::Up => -1,
        Key::Enter => return submit(),
        _ => return,
    };

    let count = count();

    Launcher::write().cursor.step(delta, count);
}

// clears the search for the next time the launcher opens
pub fn reset() {
    TextInput::set_text(INPUT, "");

    let mut launcher = Launcher::write();

    launcher.query.clear();
    launcher.cursor = Cursor::default();
}

fn count() -> usize {
    let apps = Apps::read();
    let launcher = Launcher::read();

    matches(apps.list(), &launcher.query).len()
}

fn submit() {
    let app = {
        let apps = Apps::read();
        let launcher = Launcher::read();

        let found = matches(apps.list(), &launcher.query);

        found
            .get(launcher.cursor.selected(found.len()))
            .map(|app| (*app).clone())
    };

    if let Some(app) = app {
        launch(&app);
    }
}

fn launch(app: &DesktopApp) {
    app.launch();

    let _ = Panels::close(LAUNCHER);
}

// best match first; an empty query keeps the alphabetical order
fn matches<'a>(apps: &'a [DesktopApp], query: &str) -> Vec<&'a DesktopApp> {
    let query = query.trim();

    if query.is_empty() {
        return apps.iter().collect();
    }

    let mut scored: Vec<(i32, &DesktopApp)> = apps
        .iter()
        .filter_map(|app| score_app(app, query).map(|score| (score, app)))
        .collect();

    // stable, so equal scores stay alphabetical
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    scored.into_iter().map(|(_, app)| app).collect()
}

// the name counts most, the command it runs helps find things like "code"
fn score_app(app: &DesktopApp, query: &str) -> Option<i32> {
    let name = fuzzy(query, app.name());

    let program = app
        .exec()
        .split_whitespace()
        .next()
        .and_then(|command| command.rsplit('/').next())
        .and_then(|program| fuzzy(query, program))
        .map(|score| score - 10);

    name.max(program)
}
