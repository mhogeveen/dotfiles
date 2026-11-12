/*
 * screenshot menu, the same choices as the old fuzzel menu in screenshot.sh plus
 * a region to annotate in satty and screen recordings: press a choice's number
 * to take it right away, or pick one with up/down or tab and press enter, or
 * click it
 *
 * the capture itself happens in src/screenshot.rs, recording in src/recording.rs
 */

use std::thread;
use std::time::Duration;

use amane::{LayerWindow, Monitor};

use super::SCREENSHOT;
use super::menu::{self, Menu};
use crate::recording;
use crate::screenshot::{self, Area, Target};

/*
 * the menu hides before the capture starts, so it isn't in the screenshot;
 * this is enough for the compositor to have drawn a frame without it
 */
const HIDE_DELAY: Duration = Duration::from_millis(250);

#[derive(Clone, Copy)]
pub struct Choice {
    label: &'static str,
    action: Action,
}

#[derive(Clone, Copy)]
enum Action {
    Shot(Area, Target),

    // fullscreen is the focused monitor
    Record(Area),
}

static CHOICES: &[Choice] = &[
    Choice {
        label: "Fullscreen (copy)",
        action: Action::Shot(Area::Fullscreen, Target::Clipboard),
    },
    Choice {
        label: "Fullscreen (save)",
        action: Action::Shot(Area::Fullscreen, Target::File),
    },
    Choice {
        label: "Region (copy)",
        action: Action::Shot(Area::Region, Target::Clipboard),
    },
    Choice {
        label: "Region (save)",
        action: Action::Shot(Area::Region, Target::File),
    },
    Choice {
        label: "Region (annotate)",
        action: Action::Shot(Area::Region, Target::Annotate),
    },
    Choice {
        label: "Fullscreen (record)",
        action: Action::Record(Area::Fullscreen),
    },
    Choice {
        label: "Region (record)",
        action: Action::Record(Area::Region),
    },
];

pub struct Screenshot;

impl Menu for Screenshot {
    const PANEL: &'static str = SCREENSHOT;
    const NAMESPACE: &'static str = "amane-screenshot";

    type Item = Choice;

    fn items() -> &'static [Choice] {
        CHOICES
    }

    fn label(choice: Choice) -> &'static str {
        choice.label
    }

    // the panel is closed by now, the capture waits for it to be gone from the screen
    fn run(choice: Choice) {
        // slurp waits for you to drag a region, so this can take a while
        thread::spawn(move || {
            thread::sleep(HIDE_DELAY);

            match choice.action {
                Action::Shot(area, target) => screenshot::take(area, target),
                Action::Record(area) => recording::start(area),
            }
        });
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    menu::view::<Screenshot>(monitor)
}

pub fn reset() {
    menu::reset::<Screenshot>();
}
