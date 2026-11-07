/*
 * clipboard history, kept by cliphist: type to filter, up/down or tab to pick,
 * enter or click to copy an entry back to the clipboard. the selected entry shows
 * in full beside the list (src/clipboard_preview.rs); its × button removes it,
 * and the button under the list, clicked twice, clears everything
 *
 * the shell records the history itself, by running `wl-paste --watch cliphist store`
 * for as long as it runs, so nothing has to be added to the compositor's autostart
 */

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use amane::{
    Align::{Center, Start},
    Column,
    Cursor::Pointer,
    Image, LayerWindow, Monitor, Rectangle, Service,
    Size::Parent,
    Text, children, lines, output,
};

use super::picker::{self, Picker};
use super::{CLIPBOARD, list, ui};
use crate::clipboard_preview::{self, Preview, Previews};
use crate::theme::Theme;

// stores every new selection, then prints a line so the history knows to reload
const WATCH: &str = "wl-paste --watch sh -c 'cliphist store; echo'";

pub struct Entry {
    // the line exactly as `cliphist list` printed it, which `cliphist decode` takes back
    line: String,

    // cliphist's number for it, the start of the line
    id: String,

    preview: String,
    lowercase: String,

    // for an image, its size and dimensions, like "259 KiB · 1771×1408"
    image: Option<String>,
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
    let (id, preview) = line.split_once('\t')?;

    let preview = preview.split_whitespace().collect::<Vec<_>>().join(" ");

    Some(Entry {
        line: String::from(line),
        id: String::from(id),
        lowercase: preview.to_lowercase(),
        image: clipboard_preview::image_info(&preview),
        preview,
    })
}

/*
 * records the clipboard from startup on, not only once the panel first opens;
 * the history starts when something first reads it. pictures decoded for
 * previews in an earlier run are cleared away first
 */
pub fn start() {
    clipboard_preview::forget_all();

    drop(History::read());
}

// Clear history clicked once, waiting for the second click
struct Armed {
    clear: bool,
}

impl Service for Armed {
    fn new() -> Self {
        Self { clear: false }
    }

    // only changes through input
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

// anything else done in between, like moving or typing, takes the first press back
fn disarm() {
    if Armed::read().clear {
        Armed::write().clear = false;
    }
}

// nerd font material design close glyph
const ICON_DELETE: &str = "\u{f0156}";

// how wide the preview of the selected entry is
const PREVIEW_WIDTH: f32 = 360.0;

pub struct Clipboard;

impl Picker for Clipboard {
    const PANEL: &'static str = CLIPBOARD;
    const INPUT: &'static str = "clipboard-search";
    const PLACEHOLDER: &'static str = "Search clipboard history  ·  ? for help";
    const NAMESPACE: &'static str = "amane-clipboard";

    const PREVIEW: f32 = PREVIEW_WIDTH;
    const FOOTER: bool = true;

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

    /*
     * the selected row gets a delete button; there's no delete key, amane reports
     * delete the same as shift or ctrl pressed on their own
     */
    fn row(t: &Theme, _: &Monitor, entry: &Arc<Entry>, selected: bool) -> Rectangle {
        let button = selected.then(|| {
            let deleted = entry.clone();

            ui::icon_button_colored(t, ICON_DELETE, t.fg_dim, move |_| delete(&deleted))
        });

        list::button_row(t, &entry.preview, selected, button)
    }

    fn pick(entry: Arc<Entry>) {
        copy(entry.line.clone());
    }

    fn help() -> &'static [(&'static str, &'static str)] {
        &[
            ("any text", "search the history"),
            ("Enter", "copy it back to the clipboard"),
            ("×", "remove the selected entry"),
            ("Clear history", "the button, click twice to clear it all"),
        ]
    }

    fn empty(_: &str) -> &'static str {
        match &History::read().state {
            State::Loading => "Loading history…",
            State::Missing => "cliphist is not installed",
            State::Ready(entries) if entries.is_empty() => "Nothing copied yet",
            State::Ready(_) => "No matches",
        }
    }

    // the selected entry in full: its whole text, or the picture
    fn preview(t: &Theme, selected: Option<&Arc<Entry>>) -> Option<Rectangle> {
        let area = Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(t.bg_deep)
            .padding(ui::PADDING)
            .clip();

        let Some(entry) = selected else {
            return Some(area);
        };

        // only starts a decode the first time, the result comes back as a redraw
        clipboard_preview::request(&entry.id, &entry.line, entry.image.is_some());

        let previews = Previews::read();

        let note = |text: &str| Text::new(String::from(text)).size(13.0).color(t.comment);

        let area = match previews.get(&entry.id) {
            None => area.align_child(Center, Center).child(note("…")),

            Some(Preview::Failed) => area
                .align_child(Center, Center)
                .child(note("Can't show this entry")),

            Some(Preview::Text(text)) => {
                area.child(Text::new(text.as_str()).size(13.0).color(t.fg).wrap())
            }

            Some(Preview::Image(path)) => {
                let picture = Rectangle::new()
                    .width(Parent)
                    .height(Parent)
                    .fill(Image::contain(path).thumbnail(THUMBNAIL, THUMBNAIL));

                let info = entry.image.clone().unwrap_or_default();

                area.child(
                    Column::new(children![picture, note(&info)])
                        .gap(ui::PADDING)
                        .height(Parent),
                )
            }
        };

        Some(area)
    }

    // how many entries there are, and the button that clears them all
    fn footer(t: &Theme) -> Option<Rectangle> {
        let count = match &History::read().state {
            State::Ready(entries) => entries.len(),
            _ => 0,
        };

        let counted = Rectangle::new()
            .width(Parent)
            .height(ui::BUTTON)
            .align_child(Start, Center)
            .child(
                Text::new(format!("{count} entries"))
                    .size(13.0)
                    .color(t.comment),
            );

        let button = if Armed::read().clear {
            Rectangle::new()
                .width(CLEAR_ARMED)
                .height(ui::BUTTON)
                .fill(t.accent)
                .align_child(Center, Center)
                .cursor(Pointer)
                .on_click(|_| clear_clicked())
                .child(
                    Text::new(format!("Click again to clear {count} entries"))
                        .size(13.0)
                        .color(t.on_accent),
                )
        } else {
            ui::button(t, "Clear history", CLEAR, |_| clear_clicked())
        };

        Some(list::footer(children![counted, button]))
    }

    // moving or typing takes back a first click on Clear history
    fn moved() {
        disarm();
    }
}

// a large picture is decoded at most this many pixels across, enough for the preview
const THUMBNAIL: u32 = 720;

const CLEAR: f32 = 120.0;
const CLEAR_ARMED: f32 = 260.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    picker::view::<Clipboard>(monitor)
}

pub fn reset() {
    picker::reset::<Clipboard>();

    disarm();
}

// the first click arms, the second wipes the whole history
fn clear_clicked() {
    let confirmed = Armed::read().clear;

    if !confirmed {
        Armed::write().clear = true;

        return;
    }

    disarm();

    thread::spawn(|| {
        if let Err(error) = Command::new("cliphist").arg("wipe").status() {
            eprintln!("clipboard: couldn't clear the history: {error}");
        }

        reload();

        clipboard_preview::forget_all();
    });
}

/*
 * `cliphist delete` takes the listed line back like decode does; on a thread,
 * then the history is read again so the list shows it gone
 */
fn delete(entry: &Entry) {
    disarm();

    let line = entry.line.clone();
    let id = entry.id.clone();

    thread::spawn(move || {
        if let Err(error) = cliphist_with_input("delete", &line) {
            eprintln!("clipboard: couldn't delete the entry: {error}");
        }

        reload();

        clipboard_preview::forget(&id);
    });
}

fn cliphist_with_input(command: &str, input: &str) -> std::io::Result<()> {
    let mut cliphist = Command::new("cliphist")
        .arg(command)
        .stdin(Stdio::piped())
        .spawn()?;

    cliphist
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input.as_bytes())?;

    cliphist.wait()?;

    Ok(())
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
