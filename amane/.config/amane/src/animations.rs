/*
 * whether the shell animates, switched in the system panel and kept in a
 * state file; every animated part reads it through `follow`, so turning it
 * off stops all motion at once (for now: the bar's workspace indicator)
 *
 * the animations themselves are amane's `Animation`, which asks for frames
 * only while something is still moving, so the windows rest once it arrives
 */

use std::fs;
use std::time::Duration;

use amane::{Animation, Blend, Service};

use crate::state;

// the file in the state folder the setting is kept in
const STATE: &str = "animations";

const ON: &str = "on";
const OFF: &str = "off";

// how long every animation in the shell takes, so all of them feel alike
pub const DURATION: Duration = Duration::from_millis(200);

pub struct Animations {
    enabled: bool,
}

impl Service for Animations {
    // a file of a few bytes, quick enough for the first view that asks
    fn new() -> Self {
        let saved = fs::read_to_string(state::path(STATE)).unwrap_or_default();

        Self {
            enabled: enabled_from(&saved),
        }
    }

    // only changes when switched, nothing to poll
    fn listen() {}
}

impl Animations {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    // from a click or key; a file that can't be written only costs remembering it
    pub fn set(enabled: bool) {
        Animations::write().enabled = enabled;

        let text = if enabled { ON } else { OFF };

        if let Err(error) = state::save(STATE, text) {
            eprintln!("animations: couldn't save the setting: {error}");
        }
    }

    pub fn toggle() {
        let enabled = Animations::read().enabled;

        Animations::set(!enabled);
    }
}

// only "off" turns them off, so a missing or broken file leaves them on
fn enabled_from(saved: &str) -> bool {
    !saved.trim().eq_ignore_ascii_case(OFF)
}

/*
 * moves the animation towards the target and gives the value to draw; with
 * animations off it's the target straight away, and no frames are asked for
 */
pub fn follow<T: Blend>(animation: &mut Animation<T>, target: T, enabled: bool) -> T {
    animation.to(target);

    if enabled { animation.value() } else { target }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_off_turns_animations_off() {
        assert!(!enabled_from("off"));
        assert!(!enabled_from(" OFF\n"));
        assert!(enabled_from("on"));
        assert!(enabled_from(""));
        assert!(enabled_from("garbage"));
    }

    #[test]
    fn switched_off_gives_the_target_at_once() {
        let mut animation = Animation::new(0.0_f32).duration(Duration::from_secs(60));

        assert_eq!(follow(&mut animation, 10.0, false), 10.0);
    }

    #[test]
    fn switched_on_starts_from_where_it_was() {
        let mut animation = Animation::new(0.0_f32).duration(Duration::from_secs(60));

        assert!(follow(&mut animation, 10.0, true) < 1.0);
    }
}
