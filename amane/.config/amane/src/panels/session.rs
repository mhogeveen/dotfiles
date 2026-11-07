/*
 * session menu: lock, suspend, logout, reboot, shutdown; press a number to run
 * one, or pick with up/down or tab and press enter, or click
 *
 * logout, reboot and shutdown ask again: the first choice arms the row, choosing
 * the same row again runs it, anything else disarms it
 *
 * the actions themselves are in src/session.rs
 */

use amane::{LayerWindow, Monitor};

use super::SESSION;
use super::menu::{self, Menu};
use crate::session::{ACTIONS, Action};

pub struct Session;

impl Menu for Session {
    const PANEL: &'static str = SESSION;
    const NAMESPACE: &'static str = "amane-session";

    type Item = Action;

    fn items() -> &'static [Action] {
        &ACTIONS
    }

    fn label(action: Action) -> &'static str {
        action.label()
    }

    fn confirms(action: Action) -> bool {
        action.confirms()
    }

    fn run(action: Action) {
        action.run();
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    menu::view::<Session>(monitor)
}

pub fn reset() {
    menu::reset::<Session>();
}
