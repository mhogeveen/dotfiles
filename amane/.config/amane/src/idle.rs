/*
 * the idle timers for the system panel: hypridle dims the monitor, locks,
 * turns the screen off and suspends after a while without input, set up in
 * the NixOS config (home/hypridle.nix) as a user service that starts with
 * every login. the shell only starts and stops that unit (src/user_unit.rs),
 * so switching it off keeps the screen on for this session, for a long task;
 * the shell still locks before a suspend by itself (src/session.rs)
 *
 * systemctl runs on threads only: the state is read once at start and when
 * the system panel opens (at most every READ_EVERY), and a switch shows on
 * the checkbox right away, then is read back so a failed start shows too
 */
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::settings::settings;
use crate::user_unit::{self, State};

// the panel asks systemd again when it's been open this long
const READ_EVERY: Duration = Duration::from_secs(10);

pub struct Idle {
    state: State,
}

impl Service for Idle {
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
    Idle::read().state
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
        let state = user_unit::read_state(settings().idle_unit);

        // a switch still on its way wins over what systemd said before it
        if !SWITCHING.load(Ordering::Acquire) {
            Idle::write().state = state;
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

    Idle::write().state = State::Active(wanted);

    thread::spawn(move || {
        let state = user_unit::switch(settings().idle_unit, wanted);

        Idle::write().state = state;

        SWITCHING.store(false, Ordering::Release);
    });
}
