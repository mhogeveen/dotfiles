/*
 * night light for the system panel: wlsunset warms the screen from sunset to
 * sunrise, set up in the NixOS config (home/wlsunset.nix) as a user service
 * that starts with every login. the shell only starts and stops that unit, so
 * switching it off lasts for this session and needs no state file; a stopped
 * wlsunset gives the normal colors back at once
 *
 * systemctl runs on threads only: the state is read once at start and when
 * the system panel opens (at most every READ_EVERY), and a switch shows on
 * the checkbox right away, then is read back so a failed start shows too
 */

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::command;

// the home-manager unit
const UNIT: &str = "wlsunset.service";

// the panel asks systemd again when it's been open this long
const READ_EVERY: Duration = Duration::from_secs(10);

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

pub struct NightLight {
    state: State,
}

impl Service for NightLight {
    fn new() -> Self {
        Self {
            state: State::Reading,
        }
    }

    // read once at start, then when the panel opens
    fn listen() {
        follow();
    }
}

// reading, safe inside views
pub fn state() -> State {
    NightLight::read().state
}

// when the state was last read; outside the service, so starting a read redraws nothing
static READ_AT: Mutex<Option<Instant>> = Mutex::new(None);
static READING: AtomicBool = AtomicBool::new(false);
static SWITCHING: AtomicBool = AtomicBool::new(false);

/*
 * reads the state on a thread when that's due; safe to call from a view, it
 * only starts a thread
 */
pub fn follow() {
    {
        let mut read_at = READ_AT.lock().unwrap_or_else(PoisonError::into_inner);

        if read_at.is_some_and(|at| at.elapsed() < READ_EVERY) {
            return;
        }

        if READING.swap(true, Ordering::AcqRel) {
            return;
        }

        *read_at = Some(Instant::now());
    }

    thread::spawn(|| {
        let state = read_state();

        // a switch still on its way wins over what systemd said before it
        if !SWITCHING.load(Ordering::Acquire) {
            NightLight::write().state = state;
        }

        READING.store(false, Ordering::Release);
    });
}

/*
 * from a click or key, never inside a view (it writes the service); one
 * switch at a time, and nothing while the state is unknown
 */
pub fn toggle() {
    let State::Active(active) = state() else {
        return;
    };

    if SWITCHING.swap(true, Ordering::AcqRel) {
        return;
    }

    let wanted = !active;

    NightLight::write().state = State::Active(wanted);

    thread::spawn(move || {
        let verb = if wanted { "start" } else { "stop" };

        let switched = command::capture(
            Command::new("systemctl").args(["--user", verb, UNIT]),
            CALL_TIMEOUT,
        );

        if !matches!(&switched, Ok(output) if output.status.success()) {
            eprintln!("night light: systemctl --user {verb} {UNIT} failed");
        }

        let state = read_state();

        NightLight::write().state = state;

        SWITCHING.store(false, Ordering::Release);
    });
}

fn read_state() -> State {
    let answer = command::read(
        Command::new("systemctl").args([
            "--user",
            "show",
            "--property=LoadState,ActiveState",
            UNIT,
        ]),
        CALL_TIMEOUT,
    );

    answer.as_deref().map_or(State::Unavailable, state_from)
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
