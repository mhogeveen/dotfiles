/*
 * switches for systemd user units the system panel turns on and off (night
 * light's wlsunset, the idle timers' hypridle): reading whether one runs and
 * starting or stopping it. both block on systemctl, so they're for threads
 * only; each switch keeps its own service and statics (src/night_light.rs,
 * src/idle.rs), since every amane service is its own type
 *
 * a start or stop lasts for this session: home-manager starts the units again
 * with every login, so no state file is kept
 */

use std::process::Command;
use std::time::Duration;

use crate::command;

// systemctl answers in milliseconds; past this a stuck call is killed
const CALL_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    // systemd hasn't answered yet
    Reading,

    // no such unit (not rebuilt yet), or systemctl failed
    Unavailable,

    Active(bool),
}

// blocks: on a thread only
pub fn read_state(unit: &str) -> State {
    let answer = command::read(
        Command::new("systemctl").args([
            "--user",
            "show",
            "--property=LoadState,ActiveState",
            unit,
        ]),
        CALL_TIMEOUT,
    );

    answer.as_deref().map_or(State::Unavailable, state_from)
}

/*
 * starts or stops the unit and gives back what systemd says afterwards, so a
 * failed start shows too; blocks: on a thread only
 */
pub fn switch(unit: &str, wanted: bool) -> State {
    let verb = if wanted { "start" } else { "stop" };

    let switched = command::capture(
        Command::new("systemctl").args(["--user", verb, unit]),
        CALL_TIMEOUT,
    );

    if !matches!(&switched, Ok(output) if output.status.success()) {
        eprintln!("systemctl --user {verb} {unit} failed");
    }

    read_state(unit)
}

/*
 * from `systemctl show`'s KEY=value lines; a unit that's starting counts as
 * on, so the checkbox doesn't flicker right after a switch
 */
fn state_from(answer: &str) -> State {
    let value = |key: &str| {
        answer.lines().find_map(|line| {
            line.trim()
                .strip_prefix(key)
                .and_then(|rest| rest.strip_prefix('='))
        })
    };

    match (value("LoadState"), value("ActiveState")) {
        (Some("loaded"), Some(active)) => {
            State::Active(matches!(active, "active" | "activating" | "reloading"))
        }
        _ => State::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_running_unit_is_on() {
        assert_eq!(
            state_from("LoadState=loaded\nActiveState=active\n"),
            State::Active(true)
        );
        assert_eq!(
            state_from("ActiveState=activating\nLoadState=loaded\n"),
            State::Active(true)
        );
    }

    #[test]
    fn a_stopped_or_failed_unit_is_off() {
        assert_eq!(
            state_from("LoadState=loaded\nActiveState=inactive\n"),
            State::Active(false)
        );
        assert_eq!(
            state_from("LoadState=loaded\nActiveState=failed\n"),
            State::Active(false)
        );
    }

    #[test]
    fn a_missing_unit_is_unavailable() {
        assert_eq!(
            state_from("LoadState=not-found\nActiveState=inactive\n"),
            State::Unavailable
        );
        assert_eq!(state_from(""), State::Unavailable);
    }
}
