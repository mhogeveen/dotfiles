/*
 * app launcher: type to fuzzy-search the installed programs, up/down or tab to pick,
 * enter or click to start one
 */

use amane::{Apps, DesktopApp, LayerWindow, Monitor, Rectangle, Service};

use super::LAUNCHER;
use super::list::{self, fuzzy};
use super::picker::{self, Picker};
use crate::theme::Theme;

pub struct Launcher;

impl Picker for Launcher {
    const PANEL: &'static str = LAUNCHER;
    const INPUT: &'static str = "launcher-search";
    const PLACEHOLDER: &'static str = "Search applications";
    const NAMESPACE: &'static str = "amane-launcher";

    type Item = DesktopApp;

    fn items(query: &str) -> Vec<DesktopApp> {
        let apps = Apps::read();

        matches(apps.list(), query).into_iter().cloned().collect()
    }

    fn row(t: &Theme, _: &Monitor, app: &DesktopApp, selected: bool) -> Rectangle {
        list::row(t, app.name(), selected)
    }

    fn pick(app: DesktopApp) {
        app.launch();
    }

    // the first scan walks every icon theme, which takes a few seconds
    fn empty(_: &str) -> &'static str {
        if Apps::read().list().is_empty() {
            "Loading applications…"
        } else {
            "No matches"
        }
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    picker::view::<Launcher>(monitor)
}

pub fn reset() {
    picker::reset::<Launcher>();
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
