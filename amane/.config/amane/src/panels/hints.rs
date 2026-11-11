/*
 * key hints: pressing ? in any panel shows each button's key inside the button
 * for a moment, instead of its icon; pressing ? again starts the moment over
 *
 * amane doesn't say when a key is let go, so the hints last a set time rather
 * than for as long as ? is held
 */

use std::thread;
use std::time::Duration;

use amane::Service;

const SHOW_FOR: Duration = Duration::from_secs(2);

pub struct Hints {
    shown: bool,

    // goes up with every ?, so an older press doesn't hide a newer one's hints
    presses: u64,
}

impl Service for Hints {
    fn new() -> Self {
        Self {
            shown: false,
            presses: 0,
        }
    }

    // only changes through ? and its timer
    fn listen() {}
}

// whether buttons show their keys right now; read it while building a panel
pub fn shown() -> bool {
    Hints::read().shown
}

pub fn show() {
    let press = {
        let mut hints = Hints::write();

        hints.shown = true;
        hints.presses += 1;

        hints.presses
    };

    thread::spawn(move || {
        thread::sleep(SHOW_FOR);

        // a later ? keeps them up, and a write would redraw the panel for nothing
        let latest = {
            let hints = Hints::read();

            hints.shown && hints.presses == press
        };

        if latest {
            Hints::write().shown = false;
        }
    });
}

// when a panel closes, so the next one doesn't open with hints showing
pub fn hide() {
    if Hints::read().shown {
        Hints::write().shown = false;
    }
}
