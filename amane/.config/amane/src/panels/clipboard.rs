/*
 * clipboard history, kept by cliphist: type to filter, up/down or tab to pick,
 * enter or click to copy an entry back to the clipboard
 *
 * the shell records the history itself, by running `wl-paste --watch cliphist store`
 * for as long as it runs, so nothing has to be added to the compositor's autostart
 */

use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use amane::{
    Cursor::Pointer, Key, LayerWindow, Monitor, Service, TextInput, Vertical, Widget, lines, output,
};

use super::list::{self, Cursor};
use super::{CLIPBOARD, Panels, panel_window};
use crate::theme::theme;

const INPUT: &str = "clipboard-search";

// stores every new selection, then prints a line so the history knows to reload
const WATCH: &str = "wl-paste --watch sh -c 'cliphist store; echo'";

struct Entry {
    // the line exactly as `cliphist list` printed it, which `cliphist decode` takes back
    line: String,

    preview: String,
    lowercase: String,
}

enum State {
    Loading,
    Missing,
    Ready(Vec<Entry>),
}

// newest first
struct History {
    state: State,
}

impl Service for History {
    fn new() -> Self {
        Self {
            state: State::Loading,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(30)
    }

    fn listen() {
        loop {
            if output("command -v cliphist").is_empty() {
                History::write().state = State::Missing;

                thread::sleep(Self::interval());

                continue;
            }

            reload();

            // blocks until wl-paste exits, which only happens when the compositor goes
            for _ in lines(WATCH) {
                reload();
            }

            thread::sleep(Self::interval());
        }
    }
}

fn reload() {
    let entries = output("cliphist list").lines().filter_map(parse).collect();

    History::write().state = State::Ready(entries);
}

// `<id>\t<preview>`
fn parse(line: &str) -> Option<Entry> {
    let (_, preview) = line.split_once('\t')?;

    let preview = preview.split_whitespace().collect::<Vec<_>>().join(" ");

    Some(Entry {
        line: String::from(line),
        lowercase: preview.to_lowercase(),
        preview,
    })
}

struct Clipboard {
    query: String,
    cursor: Cursor,
}

impl Service for Clipboard {
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
    let history = History::read();
    let clipboard = Clipboard::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    match &history.state {
        State::Loading => rows.push(list::message(&t, "Loading history…")),
        State::Missing => rows.push(list::message(&t, "cliphist is not installed")),
        State::Ready(entries) => {
            let found = matches(entries, &clipboard.query);
            let selected = clipboard.cursor.selected(found.len());

            for index in clipboard.cursor.visible(found.len()) {
                let entry = found[index];
                let line = entry.line.clone();

                rows.push(Box::new(
                    list::row(&t, &entry.preview, index == selected)
                        .cursor(Pointer)
                        .on_click(move |_| copy(line.clone())),
                ));
            }

            if rows.is_empty() {
                let text = if entries.is_empty() {
                    "Nothing copied yet"
                } else {
                    "No matches"
                };

                rows.push(list::message(&t, text));
            }
        }
    }

    let input = TextInput::new(INPUT)
        .placeholder("Search clipboard history")
        .focused()
        .on_change(|text| {
            let mut clipboard = Clipboard::write();

            clipboard.query = text;
            clipboard.cursor = Cursor::default();
        })
        .on_submit(|_| submit());

    let rows = list::rows(rows).on_scroll(|scroll| {
        let count = count();

        Clipboard::write()
            .cursor
            .scroll(list::scroll_delta(scroll), count);
    });

    let base = LayerWindow::new()
        .width(list::width())
        .height(list::height(list::ROWS, true))
        .anchor_vertical(Vertical::Middle)
        .namespace("amane-clipboard")
        .child(list::frame(&t, Some(input), rows));

    panel_window(monitor, CLIPBOARD, base, key_pressed)
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

    Clipboard::write().cursor.step(delta, count);
}

// clears the search for the next time the panel opens
pub fn reset() {
    TextInput::set_text(INPUT, "");

    let mut clipboard = Clipboard::write();

    clipboard.query.clear();
    clipboard.cursor = Cursor::default();
}

fn count() -> usize {
    let history = History::read();
    let clipboard = Clipboard::read();

    match &history.state {
        State::Ready(entries) => matches(entries, &clipboard.query).len(),
        _ => 0,
    }
}

fn submit() {
    let line = {
        let history = History::read();
        let clipboard = Clipboard::read();

        let State::Ready(entries) = &history.state else {
            return;
        };

        let found = matches(entries, &clipboard.query);

        found
            .get(clipboard.cursor.selected(found.len()))
            .map(|entry| entry.line.clone())
    };

    if let Some(line) = line {
        copy(line);
    }
}

fn matches<'a>(entries: &'a [Entry], query: &str) -> Vec<&'a Entry> {
    let query = query.trim().to_lowercase();

    entries
        .iter()
        .filter(|entry| entry.lowercase.contains(&query))
        .collect()
}

/*
 * `cliphist decode` turns the listed line back into the original bytes, text or
 * image, and wl-copy puts them on the clipboard; on its own thread so a big image
 * never holds up drawing. the watcher then moves the entry to the top of the history
 */
fn copy(line: String) {
    let _ = Panels::close(CLIPBOARD);

    thread::spawn(move || {
        if let Err(error) = decode_and_copy(&line) {
            eprintln!("clipboard: failed to copy entry: {error}");
        }
    });
}

fn decode_and_copy(line: &str) -> std::io::Result<()> {
    let mut decode = Command::new("cliphist")
        .arg("decode")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    decode
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(line.as_bytes())?;

    let decoded = decode.wait_with_output()?;

    let mut copy = Command::new("wl-copy").stdin(Stdio::piped()).spawn()?;

    copy.stdin
        .take()
        .expect("stdin is piped")
        .write_all(&decoded.stdout)?;

    copy.wait()?;

    Ok(())
}
