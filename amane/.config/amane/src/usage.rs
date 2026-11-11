/*
 * how often and how recently each app was launched, so the launcher can put the
 * ones you use first ("frecency", like a browser ranks its history)
 *
 * every launch adds 1 to an app's score, and a score halves every week it isn't
 * used, so an app used daily stays on top and one used a lot months ago drifts
 * down. kept in ~/.local/state/amane-shell/launches, one line per app:
 *
 *   name<TAB>score<TAB>last used, unix seconds
 *
 * amane gives no desktop file id, so apps are told apart by name
 */

use std::fs;
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use amane::Service;

use crate::state;

const STATE: &str = "launches";

// held while saving, so saves can't overlap
static SAVING: Mutex<()> = Mutex::new(());

// a score halves after this many seconds without a launch
const HALF_LIFE: f64 = 7.0 * 24.0 * 60.0 * 60.0;

#[derive(Debug, Clone, PartialEq)]
struct Launches {
    name: String,
    score: f64,
    last: u64,
}

impl Launches {
    // what the score is worth now, after fading since the last launch
    fn score_at(&self, now: u64) -> f64 {
        let elapsed = now.saturating_sub(self.last) as f64;

        self.score * 0.5_f64.powf(elapsed / HALF_LIFE)
    }
}

pub struct Usage {
    apps: Vec<Launches>,
}

impl Service for Usage {
    // a small file, read the first time the launcher opens
    fn new() -> Self {
        let text = fs::read_to_string(state::path(STATE)).unwrap_or_default();

        Self { apps: parse(&text) }
    }

    // only changes through launches
    fn listen() {}
}

impl Usage {
    // 0 for an app never launched
    pub fn score(&self, name: &str) -> f64 {
        let now = now();

        self.apps
            .iter()
            .find(|app| app.name == name)
            .map_or(0.0, |app| app.score_at(now))
    }

    // counts a launch, and saves on a thread so starting the app never waits on the disk
    pub fn record(name: &str) {
        record(&mut Usage::write().apps, name, now());

        /*
         * one save at a time, each writing the list as it is by then: with
         * the text taken before the thread, two quick launches could finish
         * out of order and the older list be written last, losing a count
         */
        thread::spawn(|| {
            let _saving = SAVING.lock().unwrap_or_else(PoisonError::into_inner);

            let text = serialize(&Usage::read().apps);

            if let Err(error) = state::save(STATE, &text) {
                eprintln!("usage: couldn't save launches: {error}");
            }
        });
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

fn record(apps: &mut Vec<Launches>, name: &str, now: u64) {
    match apps.iter_mut().find(|app| app.name == name) {
        Some(app) => {
            app.score = app.score_at(now) + 1.0;
            app.last = now;
        }
        None => apps.push(Launches {
            name: String::from(name),
            score: 1.0,
            last: now,
        }),
    }
}

// a line that doesn't read is skipped, the rest still count
fn parse(text: &str) -> Vec<Launches> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');

            let name = fields.next()?;
            let score = fields.next()?.parse().ok()?;
            let last = fields.next()?.parse().ok()?;

            (!name.is_empty()).then(|| Launches {
                name: String::from(name),
                score,
                last,
            })
        })
        .collect()
}

fn serialize(apps: &[Launches]) -> String {
    apps.iter()
        // a tab or newline in a name would break the file, there's no app like that
        .filter(|app| !app.name.contains(['\t', '\n']))
        .map(|app| format!("{}\t{}\t{}\n", app.name, app.score, app.last))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u64 = 24 * 60 * 60;

    #[test]
    fn scores_halve_every_week() {
        let app = Launches {
            name: String::from("Firefox"),
            score: 8.0,
            last: 0,
        };

        assert_eq!(app.score_at(0), 8.0);
        assert!((app.score_at(7 * DAY) - 4.0).abs() < 1e-9);
        assert!((app.score_at(14 * DAY) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn launches_add_to_the_faded_score() {
        let mut apps = Vec::new();

        record(&mut apps, "foot", 0);
        record(&mut apps, "foot", 0);

        assert_eq!(apps[0].score, 2.0);

        // a week later the 2 has faded to 1, the launch makes it 2 again
        record(&mut apps, "foot", 7 * DAY);

        assert!((apps[0].score - 2.0).abs() < 1e-9);
        assert_eq!(apps[0].last, 7 * DAY);
    }

    #[test]
    fn frequent_and_recent_beats_old_and_heavy() {
        let mut apps = Vec::new();

        // used 20 times three months ago
        for _ in 0..20 {
            record(&mut apps, "old", 0);
        }

        // used 3 times this week
        for day in 0..3 {
            record(&mut apps, "new", 90 * DAY + day * DAY);
        }

        let now = 93 * DAY;

        assert!(apps[1].score_at(now) > apps[0].score_at(now));
    }

    #[test]
    fn the_file_reads_back() {
        let mut apps = Vec::new();

        record(&mut apps, "Firefox", 100);
        record(&mut apps, "foot", 200);

        let read = parse(&serialize(&apps));

        assert_eq!(read, apps);
        assert_eq!(parse("broken line\nfoot\t1\t2\n").len(), 1);
    }
}
