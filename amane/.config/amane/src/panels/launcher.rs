/*
 * app launcher: type to fuzzy-search the installed programs, the ones you use
 * most first; up/down or tab to pick, enter or click to start one
 *
 * what the search starts with switches what it does:
 *
 *   =12*7      calculates, enter copies the answer
 *   >command   runs it in the background, like a keybind
 *   >>command  runs it in a terminal, then leaves your shell there
 *   :wallpaper opens that panel, by name or what it's for
 */

use std::process::Command;
use std::thread;

use amane::{Apps, DesktopApp, LayerWindow, Monitor, Rectangle, Service, spawn};

use super::list::{self, fuzzy};
use super::picker::{self, Picker};
use super::{LAUNCHER, PanelDef, Panels};
use crate::calculator;
use crate::settings::settings;
use crate::theme::Theme;
use crate::usage::Usage;

const CALCULATE: &str = "=";
const RUN: &str = ">";
const RUN_IN_TERMINAL: &str = ">>";
const PANEL: &str = ":";

// how far a match on a panel's description ranks below the same match on its name
const DESCRIPTION_PENALTY: i32 = 10;

// an app counts as used once its score is above this, for the empty search's top
const USED: f64 = 0.1;

#[derive(Clone)]
pub enum Entry {
    App(DesktopApp),

    // a calculator's answer, ready to copy
    Answer(String),

    Run { command: String, terminal: bool },

    Panel(&'static PanelDef),
}

// what the search asks for, by how it starts
enum Mode<'a> {
    Apps(&'a str),
    Calculate(&'a str),
    Run { command: &'a str, terminal: bool },
    Panels(&'a str),
}

fn mode(query: &str) -> Mode<'_> {
    let query = query.trim_start();

    // >> first, it also starts with >
    if let Some(command) = query.strip_prefix(RUN_IN_TERMINAL) {
        Mode::Run {
            command: command.trim(),
            terminal: true,
        }
    } else if let Some(command) = query.strip_prefix(RUN) {
        Mode::Run {
            command: command.trim(),
            terminal: false,
        }
    } else if let Some(sum) = query.strip_prefix(CALCULATE) {
        Mode::Calculate(sum)
    } else if let Some(name) = query.strip_prefix(PANEL) {
        Mode::Panels(name.trim())
    } else {
        Mode::Apps(query)
    }
}

pub struct Launcher;

impl Picker for Launcher {
    const PANEL: &'static str = LAUNCHER;
    const INPUT: &'static str = "launcher-search";
    const PLACEHOLDER: &'static str = "Search apps  ·  ? for help";
    const NAMESPACE: &'static str = "amane-launcher";

    type Item = Entry;

    fn items(query: &str) -> Vec<Entry> {
        match mode(query) {
            Mode::Apps(query) => {
                let apps = Apps::read();
                let usage = Usage::read();

                ranked(apps.list(), &usage, query)
                    .into_iter()
                    .map(|app| Entry::App(app.clone()))
                    .collect()
            }

            Mode::Calculate(sum) => calculator::calculate(sum)
                .map(|answer| vec![Entry::Answer(answer)])
                .unwrap_or_default(),

            Mode::Run { command, terminal } if !command.is_empty() => vec![Entry::Run {
                command: String::from(command),
                terminal,
            }],

            Mode::Run { .. } => Vec::new(),

            Mode::Panels(query) => panels(query).into_iter().map(Entry::Panel).collect(),
        }
    }

    fn row(t: &Theme, _: &Monitor, entry: &Entry, selected: bool) -> Rectangle {
        match entry {
            Entry::App(app) => list::row(t, app.name(), selected),

            Entry::Answer(answer) => {
                list::detail_row(t, &format!("= {answer}"), "Enter copies", selected, false)
            }

            Entry::Run { command, terminal } => {
                let place = if *terminal {
                    "in a terminal"
                } else {
                    "background"
                };

                list::detail_row(t, &format!("Run {command}"), place, selected, false)
            }

            Entry::Panel(panel) => {
                list::detail_row(t, panel.name, panel.description, selected, false)
            }
        }
    }

    fn pick(entry: Entry) {
        match entry {
            Entry::App(app) => {
                Usage::record(app.name());

                app.launch();
            }

            Entry::Answer(answer) => copy(answer),

            Entry::Run {
                command,
                terminal: false,
            } => spawn(&command),

            Entry::Run {
                command,
                terminal: true,
            } => run_in_terminal(command),

            // replaces the launcher, which then needs no closing (see picker::choose)
            Entry::Panel(panel) => {
                if let Err(error) = Panels::open(panel.name) {
                    eprintln!("launcher: {error}");
                }
            }
        }
    }

    fn help() -> &'static [(&'static str, &'static str)] {
        &[
            ("firefox", "search apps, most used first"),
            ("=12*7", "calculate"),
            (">command", "run in the background"),
            (">>command", "run in a terminal, then your shell"),
            (":wallpaper", "open a panel"),
            ("Enter", "start, copy the answer or run"),
        ]
    }

    fn empty(query: &str) -> &'static str {
        match mode(query) {
            // the first scan walks every icon theme, which takes a few seconds
            Mode::Apps(_) if Apps::read().list().is_empty() => "Loading applications…",
            Mode::Apps(_) => "No matches",

            Mode::Calculate(sum) if sum.trim().is_empty() => "Type a sum, like =12*7",
            Mode::Calculate(sum) => match calculator::calculate(sum) {
                Err(error) => error.message(),
                Ok(_) => "",
            },

            Mode::Run { .. } => "Type a command to run",
            Mode::Panels(_) => "No panel by that name",
        }
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    picker::view::<Launcher>(monitor)
}

pub fn reset() {
    picker::reset::<Launcher>();
}

/*
 * an empty search puts the apps you use first, most used on top, then the rest
 * alphabetically; a search ranks by match, with use as a bonus that settles
 * close matches but never lifts a poor match above a good one
 */
fn ranked<'a>(apps: &'a [DesktopApp], usage: &Usage, query: &str) -> Vec<&'a DesktopApp> {
    let query = query.trim();

    if query.is_empty() {
        let mut used: Vec<(f64, &DesktopApp)> = apps
            .iter()
            .map(|app| (usage.score(app.name()), app))
            .filter(|(score, _)| *score > USED)
            .collect();

        used.sort_by(|a, b| b.0.total_cmp(&a.0));

        let rest = apps.iter().filter(|app| usage.score(app.name()) <= USED);

        return used.into_iter().map(|(_, app)| app).chain(rest).collect();
    }

    let mut scored: Vec<(i32, &DesktopApp)> = apps
        .iter()
        .filter_map(|app| {
            let bonus = use_bonus(usage.score(app.name()));

            score_app(app, query).map(|score| (score + bonus, app))
        })
        .collect();

    // stable, so equal scores stay alphabetical
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    scored.into_iter().map(|(_, app)| app).collect()
}

/*
 * an empty search lists every panel in the registry's order; a search ranks by
 * match on the name, or on what it's for, below a name match
 */
fn panels(query: &str) -> Vec<&'static PanelDef> {
    if query.is_empty() {
        return super::launchable().collect();
    }

    let mut scored: Vec<(i32, &'static PanelDef)> = super::launchable()
        .filter_map(|panel| {
            let name = fuzzy(query, panel.name);
            let description =
                fuzzy(query, panel.description).map(|score| score - DESCRIPTION_PENALTY);

            name.max(description).map(|score| (score, panel))
        })
        .collect();

    // stable, so equal scores keep the registry's order
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    scored.into_iter().map(|(_, panel)| panel).collect()
}

// grows with use but slows down, and stops at 24 (about two letters' worth of match)
fn use_bonus(score: f64) -> i32 {
    (8.0 * score.ln_1p()).min(24.0) as i32
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

// the clipboard history watcher picks it up from there too
fn copy(answer: String) {
    thread::spawn(move || {
        if let Err(error) = Command::new("wl-copy").arg(answer).status() {
            eprintln!("launcher: couldn't copy the answer: {error}");
        }
    });
}

/*
 * runs the command in the terminal from the settings (foot unless set; its
 * words come before `sh -c ...`, so `alacritty -e` works too), then hands the
 * terminal to your own shell, so the output stays readable above the prompt
 * and the window closes like any other. the command is passed as $1 rather
 * than pasted into the script, so its quotes and $ can't break it; the wrapper
 * ignores ctrl+c, which stops the command but would otherwise close the window
 * too
 */
const IN_TERMINAL: &str = r#"trap : INT; eval "$1"; exec "${SHELL:-/bin/sh}""#;

fn run_in_terminal(command: String) {
    thread::spawn(move || {
        let terminal = settings().terminal;
        let mut words = terminal.split_whitespace();
        let program = words.next().unwrap_or_default();

        let started = Command::new(program)
            .args(words)
            .args(["sh", "-c", IN_TERMINAL, "sh"])
            .arg(&command)
            .spawn();

        match started {
            Ok(mut child) => {
                // a finished child nobody waits for stays behind as a zombie
                let _ = child.wait();
            }
            Err(error) => eprintln!("launcher: couldn't start {terminal}: {error}"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_search_start_picks_the_mode() {
        assert!(matches!(mode("firefox"), Mode::Apps("firefox")));
        assert!(matches!(mode("=1+1"), Mode::Calculate("1+1")));
        assert!(matches!(
            mode(">htop"),
            Mode::Run {
                command: "htop",
                terminal: false
            }
        ));
        assert!(matches!(
            mode(">> uname -a"),
            Mode::Run {
                command: "uname -a",
                terminal: true
            }
        ));
    }

    #[test]
    fn colon_lists_panels() {
        assert!(matches!(mode(":wall"), Mode::Panels("wall")));
        assert!(matches!(mode(": wall "), Mode::Panels("wall")));
    }

    #[test]
    fn panels_rank_by_name_then_description() {
        assert_eq!(panels("wall")[0].name, super::super::WALLPAPER);
        assert_eq!(panels("wallpapers")[0].name, super::super::WALLPAPER);
        assert_eq!(panels("wifi")[0].name, super::super::NETWORK);
    }

    #[test]
    fn panels_never_list_the_launcher_or_screenshot_menu() {
        let names: Vec<&str> = panels("").iter().map(|panel| panel.name).collect();

        assert!(!names.contains(&LAUNCHER));
        assert!(!names.contains(&super::super::SCREENSHOT));
        assert!(names.contains(&super::super::WALLPAPER));
    }

    #[test]
    fn use_bonus_grows_slowly_and_stops() {
        assert_eq!(use_bonus(0.0), 0);
        assert!(use_bonus(1.0) > 0);
        assert!(use_bonus(10.0) > use_bonus(1.0));
        assert_eq!(use_bonus(1_000_000.0), 24);
    }
}
