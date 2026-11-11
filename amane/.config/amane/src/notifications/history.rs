/*
 * the notification history across shell restarts: the notifications still in the
 * center are written to $XDG_RUNTIME_DIR/amane-shell/notifications.json, and a
 * restarted shell reads them back as "Earlier" in the center. logging out or
 * rebooting empties the runtime folder, so the history is the session's
 *
 * amane's own list can't be refilled, so the restored ones live here: read-only
 * copies whose senders can't be reached any more, so they have no buttons and
 * clicking them does nothing; × and Clear all remove them
 */

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amane::{Notification, Notifications, Service, Urgency};

use crate::json::{self, Value};
use crate::state;

// the newest ones kept, live and restored together
const KEEP: usize = 50;

const FILE: &str = "notifications.json";

// where amane writes the pictures notifications send as raw pixels
const AMANE_IMAGES: &str = "amane-notifications";

// what a card needs, copied from a notification
#[derive(Debug, Clone, PartialEq)]
pub struct Kept {
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: String,
    pub image: String,
    pub urgency: Urgency,
    pub received: SystemTime,
}

impl Kept {
    fn from_live(notification: &Notification) -> Self {
        Self {
            app_name: String::from(notification.app_name()),
            summary: String::from(notification.summary()),
            body: String::from(notification.body()),
            icon: String::from(notification.icon()),
            image: String::from(notification.image()),
            urgency: notification.urgency(),
            received: notification.received(),
        }
    }
}

// the ones restored at startup and not dismissed since, oldest first
pub struct History {
    restored: Vec<Kept>,
}

impl Service for History {
    // a small file, read before the first frame so the center has them right away
    fn new() -> Self {
        let restored = file()
            .and_then(|path| fs::read_to_string(path).ok())
            .map(|text| read(&text))
            .unwrap_or_default();

        Self { restored }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    // writes the file whenever the live or restored ones changed
    fn listen() {
        let Some(path) = file() else {
            return;
        };

        let mut saved: Option<Vec<Kept>> = None;

        // a failing save is told once, not every second, until one works again
        let mut told = false;

        loop {
            // one lock at a time, never both
            let live: Vec<Kept> = Notifications::read()
                .list()
                .iter()
                .map(Kept::from_live)
                .collect();

            let mut all = History::read().restored.clone();

            all.extend(live);

            let all = newest(all);

            if saved.as_ref() != Some(&all) {
                match save(&path, &all) {
                    Ok(()) => {
                        saved = Some(all);
                        told = false;
                    }
                    Err(error) if !told => {
                        eprintln!("notifications: couldn't save the history: {error}");
                        told = true;
                    }
                    Err(_) => {}
                }
            }

            thread::sleep(Self::interval());
        }
    }
}

// reading, safe inside views
impl History {
    pub fn restored(&self) -> &[Kept] {
        &self.restored
    }
}

// reads the saved history and starts saving, from the start rather than when the center first opens
pub fn start() {
    drop(History::read());
}

// the restored one at this place in restored(); from its ×
pub fn dismiss(index: usize) {
    let mut history = History::write();

    if index < history.restored.len() {
        let kept = history.restored.remove(index);

        forget_image(&kept.image);
    }
}

// every restored one; Clear all clears these together with the live ones
pub fn clear() {
    let mut history = History::write();

    if history.restored.is_empty() {
        return;
    }

    for kept in history.restored.drain(..) {
        forget_image(&kept.image);
    }
}

// only amane's own copies are removed, like amane does for live ones, never a sender's file
fn forget_image(image: &str) {
    let path = Path::new(image);

    if runtime().is_some_and(|runtime| path.parent() == Some(&runtime.join(AMANE_IMAGES))) {
        let _ = fs::remove_file(path);
    }
}

fn runtime() -> Option<PathBuf> {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

// none without a runtime folder: then nothing is kept
fn file() -> Option<PathBuf> {
    Some(runtime()?.join("amane-shell").join(FILE))
}

// at most KEEP, the newest; the list is oldest first
fn newest(mut all: Vec<Kept>) -> Vec<Kept> {
    if all.len() > KEEP {
        all.drain(..all.len() - KEEP);
    }

    all
}

fn save(path: &Path, all: &[Kept]) -> std::io::Result<()> {
    state::write_atomically(path, &write(all))
}

// a json list, one object per line
fn write(all: &[Kept]) -> String {
    let objects: Vec<String> = all
        .iter()
        .map(|kept| {
            let received = kept
                .received
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            format!(
                "{{\"app\":{},\"summary\":{},\"body\":{},\"icon\":{},\"image\":{},\"urgency\":{},\"received\":{received}}}",
                json::quote(&kept.app_name),
                json::quote(&kept.summary),
                json::quote(&kept.body),
                json::quote(&kept.icon),
                json::quote(&kept.image),
                json::quote(urgency_name(kept.urgency)),
            )
        })
        .collect();

    format!("[\n{}\n]\n", objects.join(",\n"))
}

// a broken or foreign file reads as nothing
fn read(text: &str) -> Vec<Kept> {
    let Some(Value::List(objects)) = json::parse(text) else {
        return Vec::new();
    };

    let all = objects
        .iter()
        .map(|object| Kept {
            app_name: String::from(object.get("app").text()),
            summary: String::from(object.get("summary").text()),
            body: String::from(object.get("body").text()),
            icon: String::from(object.get("icon").text()),
            image: String::from(object.get("image").text()),
            urgency: urgency_from(object.get("urgency").text()),
            received: UNIX_EPOCH
                + Duration::from_secs(object.get("received").number().unwrap_or(0.0) as u64),
        })
        .collect();

    newest(all)
}

fn urgency_name(urgency: Urgency) -> &'static str {
    match urgency {
        Urgency::Low => "low",
        Urgency::Normal => "normal",
        Urgency::Critical => "critical",
    }
}

fn urgency_from(name: &str) -> Urgency {
    match name {
        "low" => Urgency::Low,
        "critical" => Urgency::Critical,
        _ => Urgency::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kept(summary: &str, seconds: u64) -> Kept {
        Kept {
            app_name: String::from("notify-send"),
            summary: String::from(summary),
            body: String::from("a \"body\" with\nlines, \\ and 😀"),
            icon: String::from("dialog-information"),
            image: String::from("/run/user/1000/amane-notifications/12-0.png"),
            urgency: Urgency::Critical,
            received: UNIX_EPOCH + Duration::from_secs(seconds),
        }
    }

    #[test]
    fn written_history_reads_back_the_same() {
        let all = vec![kept("first", 1_700_000_000), kept("second", 1_700_000_060)];

        assert_eq!(read(&write(&all)), all);
    }

    #[test]
    fn only_the_newest_are_kept() {
        let all: Vec<Kept> = (0..KEEP as u64 + 5)
            .map(|number| kept(&number.to_string(), number))
            .collect();

        let kept_ones = newest(all);

        assert_eq!(kept_ones.len(), KEEP);
        assert_eq!(kept_ones[0].summary, "5");
        assert_eq!(kept_ones[KEEP - 1].summary, (KEEP + 4).to_string());

        // a file somehow longer than that is cut down when read, too
        let long: Vec<Kept> = (0..KEEP as u64 + 1)
            .map(|number| kept("x", number))
            .collect();

        assert_eq!(read(&write(&long)).len(), KEEP);
    }

    #[test]
    fn broken_files_restore_nothing() {
        assert!(read("").is_empty());
        assert!(read("[{\"app\":").is_empty());
        assert!(read("{\"not\":\"a list\"}").is_empty());
        assert!(read("[]").is_empty());
    }

    #[test]
    fn unknown_urgency_is_normal() {
        assert_eq!(urgency_from("critical"), Urgency::Critical);
        assert_eq!(urgency_from("low"), Urgency::Low);
        assert_eq!(urgency_from("whatever"), Urgency::Normal);
    }
}
