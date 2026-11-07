/*
 * clipboard history, kept by cliphist: type to filter, up/down or tab to pick,
 * enter or click to copy an entry back to the clipboard
 *
 * the shell records the history itself, by running `wl-paste --watch cliphist store`
 * for as long as it runs, so nothing has to be added to the compositor's autostart
 */

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use amane::{LayerWindow, Monitor, Rectangle, Service, lines, output};

use super::CLIPBOARD;
use super::list;
use super::picker::{self, Picker};
use crate::theme::Theme;

// stores every new selection, then prints a line so the history knows to reload
const WATCH: &str = "wl-paste --watch sh -c 'cliphist store; echo'";

pub struct Entry {
    // the line exactly as `cliphist list` printed it, which `cliphist decode` takes back
    line: String,

    preview: String,
    lowercase: String,
}

enum State {
    Loading,
    Missing,
    Ready(Vec<Arc<Entry>>),
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
    let entries = output("cliphist list")
        .lines()
        .filter_map(parse)
        .map(Arc::new)
        .collect();

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

/*
 * records the clipboard from startup on, not only once the panel first opens;
 * the history starts when something first reads it
 */
pub fn start() {
    drop(History::read());
}

pub struct Clipboard;

impl Picker for Clipboard {
    const PANEL: &'static str = CLIPBOARD;
    const INPUT: &'static str = "clipboard-search";
    const PLACEHOLDER: &'static str = "Search clipboard history";
    const NAMESPACE: &'static str = "amane-clipboard";

    // shared, so listing hundreds of entries copies no text
    type Item = Arc<Entry>;

    fn items(query: &str) -> Vec<Arc<Entry>> {
        let history = History::read();

        let State::Ready(entries) = &history.state else {
            return Vec::new();
        };

        let query = query.trim().to_lowercase();

        entries
            .iter()
            .filter(|entry| entry.lowercase.contains(&query))
            .cloned()
            .collect()
    }

    fn row(t: &Theme, _: &Monitor, entry: &Arc<Entry>, selected: bool) -> Rectangle {
        list::row(t, &entry.preview, selected)
    }

    fn pick(entry: Arc<Entry>) {
        copy(entry.line.clone());
    }

    fn empty(_: &str) -> &'static str {
        match &History::read().state {
            State::Loading => "Loading history…",
            State::Missing => "cliphist is not installed",
            State::Ready(entries) if entries.is_empty() => "Nothing copied yet",
            State::Ready(_) => "No matches",
        }
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    picker::view::<Clipboard>(monitor)
}

pub fn reset() {
    picker::reset::<Clipboard>();
}

/*
 * `cliphist decode` turns the listed line back into the original bytes, text or
 * image, and wl-copy puts them on the clipboard; on its own thread so a big image
 * never holds up drawing. the watcher then moves the entry to the top of the history
 */
fn copy(line: String) {
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
