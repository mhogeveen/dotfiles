/*
 * session actions: lock, suspend, logout, reboot and shutdown, plus a watcher
 * that locks the screen when logind asks, on suspend or `loginctl lock-session`
 *
 * the panel in panels/session.rs shows these, the lock screen is in lock.rs
 */

use std::env;
use std::thread;
use std::time::Duration;

use amane::{Bus, Lock, Urgency, spawn};

use crate::notifications;
use crate::panels::Panels;

/*
 * ends this login session through logind, so it works with any compositor and
 * login manager; every process in the session gets SIGTERM and ly comes back
 */
const LOGOUT: &str = "loginctl terminate-session";

/*
 * amane doesn't say when the compositor has put the lock screen up, so suspend
 * waits this long after asking, so the machine never sleeps unlocked
 */
const LOCK_BEFORE_SUSPEND: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Lock,
    Suspend,
    Logout,
    Reboot,
    Shutdown,
}

// in the order the panel numbers them, least destructive first
pub const ACTIONS: [Action; 5] = [
    Action::Lock,
    Action::Suspend,
    Action::Logout,
    Action::Reboot,
    Action::Shutdown,
];

impl Action {
    pub fn label(self) -> &'static str {
        match self {
            Action::Lock => "Lock",
            Action::Suspend => "Suspend",
            Action::Logout => "Logout",
            Action::Reboot => "Reboot",
            Action::Shutdown => "Shutdown",
        }
    }

    // the ones that end what you're doing ask a second time
    pub fn confirms(self) -> bool {
        matches!(self, Action::Logout | Action::Reboot | Action::Shutdown)
    }

    /*
     * closes every panel first, so none comes back after unlocking;
     * call it from an input handler or ipc, it never waits itself
     */
    pub fn run(self) {
        Panels::close_all();

        match self {
            Action::Lock => Lock::start(),

            Action::Suspend => {
                Lock::start();

                thread::spawn(|| {
                    thread::sleep(LOCK_BEFORE_SUSPEND);

                    spawn("systemctl suspend");
                });
            }

            Action::Logout => logout(),
            Action::Reboot => spawn("systemctl reboot"),
            Action::Shutdown => spawn("systemctl poweroff"),
        }
    }
}

fn logout() {
    let id = env::var("XDG_SESSION_ID").unwrap_or_default();

    // only digits and letters, so it can't do anything odd in the shell
    if id.is_empty()
        || !id
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        // notifications::send waits for the daemon, so not on the draw thread
        thread::spawn(|| {
            notifications::send(
                "Session",
                "Logout failed",
                "XDG_SESSION_ID is not set, so the session to end is unknown",
                Urgency::Critical,
                None,
            );
        });

        return;
    }

    spawn(&format!("{LOGOUT} {id}"));
}

// `amane ipc call lock`, for keybinds or an idle daemon
pub fn lock_ipc(_: &[String]) -> String {
    Panels::close_all();

    Lock::start();

    String::from("locked")
}

const LOGIND: &str = "org.freedesktop.login1";
const LOGIND_PATH: &str = "/org/freedesktop/login1";
const MANAGER: &str = "org.freedesktop.login1.Manager";
const SESSION: &str = "org.freedesktop.login1.Session";

/*
 * locks when the machine is about to sleep however it was asked to (the lid,
 * systemctl suspend) and when something calls `loginctl lock-session`; runs
 * forever, so start it on its own thread
 *
 * logind doesn't wait for the lock screen: that needs a delay inhibitor, a
 * file descriptor amane's d-bus can't hold. so after a suspend from outside
 * the panel, the old screen may show for a moment on resume
 */
pub fn watch_logind() {
    thread::spawn(|| {
        for signal in Bus::system().signals(MANAGER, "PrepareForSleep") {
            let going_to_sleep = signal.arguments().first().is_some_and(|value| value.bool());

            if going_to_sleep {
                Lock::start();
            }
        }
    });

    thread::spawn(|| {
        let Some(path) = session_path() else {
            eprintln!("session: couldn't find this session in logind, lock-session won't lock");

            return;
        };

        // every session's Lock comes by, only this one's counts
        for signal in Bus::system().signals(SESSION, "Lock") {
            if signal.path() == path {
                Lock::start();
            }
        }
    });
}

fn session_path() -> Option<String> {
    let id = env::var("XDG_SESSION_ID").ok()?;

    let path = Bus::system().call(
        LOGIND,
        LOGIND_PATH,
        MANAGER,
        "GetSession",
        &[amane::Argument::from(id)],
    );

    let path = path.text();

    (!path.is_empty()).then(|| String::from(path))
}
