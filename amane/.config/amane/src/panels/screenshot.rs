/*
 * screenshot menu, the same choices as the old fuzzel menu in screenshot.sh:
 * press a choice's number to take it right away, or pick one with up/down or tab
 * and press enter, or click it
 *
 * the capture itself happens in src/screenshot.rs
 */

use std::thread;
use std::time::Duration;

use amane::{LayerWindow, Monitor};

use super::SCREENSHOT;
use super::menu::{self, Menu};
use crate::screenshot::{self, Area, Target};

/*
 * the menu hides before the capture starts, so it isn't in the screenshot;
 * this is enough for the compositor to have drawn a frame without it
 */
const HIDE_DELAY: Duration = Duration::from_millis(250);

#[derive(Clone, Copy)]
pub struct Choice {
    label: &'static str,
    area: Area,
    target: Target,
}

static CHOICES: &[Choice] = &[
    Choice {
        label: "Fullscreen (copy)",
        area: Area::Fullscreen,
        target: Target::Clipboard,
    },
    Choice {
        label: "Fullscreen (save)",
        area: Area::Fullscreen,
        target: Target::File,
    },
    Choice {
        label: "Region (copy)",
        area: Area::Region,
        target: Target::Clipboard,
    },
    Choice {
        label: "Region (save)",
        area: Area::Region,
        target: Target::File,
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

            screenshot::take(choice.area, choice.target);
        });
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    menu::view::<Screenshot>(monitor)
}

pub fn reset() {
    menu::reset::<Screenshot>();
}
