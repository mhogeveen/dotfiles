/*
 * app launcher: type to fuzzy-search the installed programs, the ones you use
 * most first; up/down or tab to pick, enter or click to start one
 *
 * what the search starts with switches what it does:
 *
 *   =12*7      calculates, enter copies the answer
 *   >command   runs it in the background, like a keybind
 *   >>command  runs it in a foot terminal, then leaves your shell there
 */

use std::process::Command;
use std::thread;

use amane::{Apps, DesktopApp, LayerWindow, Monitor, Rectangle, Service, spawn};

use super::LAUNCHER;
use super::list::{self, fuzzy};
use super::picker::{self, Picker};
use crate::calculator;
use crate::theme::Theme;
use crate::usage::Usage;

const CALCULATE: &str = "=";
const RUN: &str = ">";
const RUN_IN_TERMINAL: &str = ">>";

// an app counts as used once its score is above this, for the empty search's top
const USED: f64 = 0.1;

#[derive(Clone)]
pub enum Entry {
    App(DesktopApp),

    // a calculator's answer, ready to copy
    Answer(String),

    Run { command: String, terminal: bool },
}

// what the search asks for, by how it starts
enum Mode<'a> {
    Apps(&'a str),
    Calculate(&'a str),
    Run { command: &'a str, terminal: bool },
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
        }
    }

    fn row(t: &Theme, _: &Monitor, entry: &Entry, selected: bool) -> Rectangle {
        match entry {
            Entry::App(app) => list::row(t, app.name(), selected),

            Entry::Answer(answer) => {
                list::detail_row(t, &format!("= {answer}"), "Enter copies", selected, false)
            }

            Entry::Run { command, terminal } => {
                let place = if *terminal { "in foot" } else { "background" };

                list::detail_row(t, &format!("Run {command}"), place, selected, false)
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
            } => run_in_foot(command),
        }
    }

    fn help() -> &'static [(&'static str, &'static str)] {
        &[
            ("firefox", "search apps, most used first"),
            ("=12*7", "calculate"),
            (">command", "run in the background"),
            (">>command", "run in foot, then your shell"),
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
 * runs the command in foot, then hands the terminal to your own shell, so the
 * output stays readable above the prompt and the window closes like any other.
 * the command is passed as $1 rather than pasted into the script, so its quotes
 * and $ can't break it; the wrapper ignores ctrl+c, which stops the command but
 * would otherwise close the window too
 */
const IN_FOOT: &str = r#"trap : INT; eval "$1"; exec "${SHELL:-/bin/sh}""#;

fn run_in_foot(command: String) {
    thread::spawn(move || {
        let started = Command::new("foot")
            .args(["sh", "-c", IN_FOOT, "sh"])
            .arg(&command)
            .spawn();

        match started {
            Ok(mut foot) => {
                // a finished child nobody waits for stays behind as a zombie
                let _ = foot.wait();
            }
            Err(error) => eprintln!("launcher: couldn't start foot: {error}"),
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
    fn use_bonus_grows_slowly_and_stops() {
        assert_eq!(use_bonus(0.0), 0);
        assert!(use_bonus(1.0) > 0);
        assert!(use_bonus(10.0) > use_bonus(1.0));
        assert_eq!(use_bonus(1_000_000.0), 24);
    }
}
