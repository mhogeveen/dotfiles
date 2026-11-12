/*
 * screen recording: wf-recorder films the focused monitor or a region picked
 * with slurp into ~/Videos/Screencasts/<stamp>.mp4, h264 on the gpu through
 * vaapi, falling back to software x264 when that fails at once. no sound
 *
 * the bar shows the time recorded while it runs; clicking it, SUPER+P (`record
 * menu`) or `record stop` sends wf-recorder SIGINT, which finishes the file.
 * a notification then says where it went, or why it failed
 *
 * the recording's pid, start and file are kept in the runtime folder, so a
 * recording still running after a shell restart is picked up again
 */

use std::env;
use std::fs::{self, File};
use std::os::raw::c_int;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amane::{Service, Urgency};

use crate::json::{self, Value};
use crate::panels::{Panels, SCREENSHOT};
use crate::screenshot::{self, Area};
use crate::{clock, command, notifications, state};

const APP_NAME: &str = "Screen recording";

const RECORDER: &str = "wf-recorder";

// the gpu's render node, which vaapi encodes on; this machine has one gpu
const RENDER_NODE: &str = "/dev/dri/renderD128";
const GPU_CODEC: &str = "h264_vaapi";
const SOFTWARE_CODEC: &str = "libx264";

/*
 * a recorder that can't encode (no vaapi) exits right away; one still running
 * after this is recording
 */
const START_GRACE: Duration = Duration::from_secs(1);

// how often the time in the bar moves on and a finished recorder is noticed
const TICK: Duration = Duration::from_secs(1);

// mmsg answers at once; past this mango is stuck and the recording takes every monitor
const MMSG_TIMEOUT: Duration = Duration::from_secs(3);

// in the runtime folder: the running recording, and what wf-recorder printed
const STATE: &str = "recording";
const LOG: &str = "recording.log";

// how much of wf-recorder's output a failure notification quotes
const LOG_LINES: usize = 3;

// sent to wf-recorder to stop; it finishes the file before exiting
const SIGINT: c_int = 2;

// nerd font md-record_rec
pub const ICON_RECORD: &str = "\u{f044b}";

// what the bar shows: the whole seconds recorded, None while not recording
pub struct Recording {
    elapsed: Option<u64>,
}

impl Service for Recording {
    fn new() -> Self {
        Self { elapsed: None }
    }

    fn interval() -> Duration {
        TICK
    }

    fn listen() {
        adopt();
        tick();

        loop {
            thread::sleep(TICK);

            tick();
        }
    }
}

// reading, safe inside views
impl Recording {
    pub fn elapsed(&self) -> Option<u64> {
        self.elapsed
    }
}

// the recorder running now, shared by the threads that start, stop and watch it
struct Running {
    pid: u32,
    started: SystemTime,
    file: PathBuf,

    // None for a recorder picked up after a restart, which another process reaps
    child: Option<Child>,
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);

// set while slurp waits for a region, so a second start doesn't open another
static STARTING: AtomicBool = AtomicBool::new(false);

pub fn active() -> bool {
    RUNNING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_some()
}

/*
 * starts recording the focused monitor or a dragged region; waits on slurp and
 * on the recorder starting, so only from its own thread
 */
pub fn start(area: Area) {
    if active() || STARTING.swap(true, Ordering::SeqCst) {
        return;
    }

    if let Err(error) = begin(area) {
        notifications::send(
            APP_NAME,
            "Recording failed",
            &error,
            Urgency::Critical,
            None,
        );
    }

    STARTING.store(false, Ordering::SeqCst);
}

fn begin(area: Area) -> Result<(), String> {
    let place = match area {
        Area::Fullscreen => Place::Monitor(focused_monitor()),
        Area::Region => match screenshot::pick_region()? {
            Some(geometry) => Place::Region(geometry),
            // cancelled with escape or a right click, nothing to say
            None => return Ok(()),
        },
    };

    let file = new_file()?;
    let log = state::runtime_path(LOG).ok_or("XDG_RUNTIME_DIR is not set")?;

    if let Some(folder) = log.parent() {
        fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    }

    let mut child = None;

    for codec in [GPU_CODEC, SOFTWARE_CODEC] {
        let mut recorder = spawn(&place, codec, &file, &log)?;

        thread::sleep(START_GRACE);

        if matches!(recorder.try_wait(), Ok(None)) {
            child = Some(recorder);

            break;
        }

        // what the failed try wrote is no use to the next one
        let _ = fs::remove_file(&file);
    }

    let Some(child) = child else {
        return Err(log_tail(&log));
    };

    let running = Running {
        pid: child.id(),
        started: SystemTime::now(),
        file,
        child: Some(child),
    };

    remember(&running);

    *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(running);

    Recording::write().elapsed = Some(0);

    Ok(())
}

enum Place {
    // a monitor's name, None to leave the choice to wf-recorder (fine with one monitor)
    Monitor(Option<String>),

    // slurp's "x,y wxh", in the same logical units wf-recorder takes
    Region(String),
}

fn spawn(place: &Place, codec: &str, file: &Path, log: &Path) -> Result<Child, String> {
    let mut recorder = Command::new(RECORDER);

    match place {
        Place::Monitor(Some(name)) => {
            recorder.args(["-o", name]);
        }
        Place::Monitor(None) => {}
        Place::Region(geometry) => {
            recorder.args(["-g", geometry]);
        }
    }

    recorder.args(["-c", codec]);

    if codec == GPU_CODEC {
        recorder.args(["-d", RENDER_NODE]);
    }

    let output = File::create(log).map_err(|error| format!("{}: {error}", log.display()))?;
    let errors = output
        .try_clone()
        .map_err(|error| format!("{}: {error}", log.display()))?;

    recorder
        .arg("-f")
        .arg(file)
        /*
         * the shell's own LD_LIBRARY_PATH holds libraries built against another
         * glibc (alsa-lib), which wf-recorder can't load
         */
        .env_remove("LD_LIBRARY_PATH")
        // wf-recorder asks on stdin which monitor when it isn't told; nobody answers here
        .stdin(Stdio::null())
        .stdout(output)
        .stderr(errors)
        .spawn()
        .map_err(|error| format!("couldn't start {RECORDER}: {error}"))
}

// the monitor mango has focus on; None when mango doesn't say
fn focused_monitor() -> Option<String> {
    let answer = command::read(
        Command::new("mmsg").args(["get", "all-monitors"]),
        MMSG_TIMEOUT,
    )?;

    focused_in(&json::parse(&answer)?)
}

fn focused_in(monitors: &Value) -> Option<String> {
    monitors
        .get("monitors")
        .list()
        .iter()
        .find(|monitor| monitor.get("active").bool())
        .map(|monitor| String::from(monitor.get("name").text()))
        .filter(|name| !name.is_empty())
}

// sends the recorder SIGINT; tick notices it's gone and says where the file went
pub fn stop() {
    let pid = RUNNING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .map(|running| running.pid);

    if let Some(pid) = pid {
        signal(pid, SIGINT);
    }
}

// what SUPER+P does: stops a recording, and otherwise opens or closes the screenshot menu
pub fn ipc(arguments: &[String]) -> String {
    match arguments.first().map(String::as_str) {
        Some("stop") if active() => {
            stop();

            String::from("stopping")
        }
        Some("stop") => String::from("not recording"),
        Some("menu") if active() => {
            stop();

            String::from("stopping")
        }
        Some("menu") => match Panels::toggle(SCREENSHOT) {
            Ok(true) => String::from("opened"),
            Ok(false) => String::from("closed"),
            Err(error) => error,
        },
        _ => String::from("usage: record stop | record menu"),
    }
}

/*
 * moves the time on, and notices a recorder that has exited: stopped, or
 * failed partway. from listen's thread only
 */
fn tick() {
    let (elapsed, finished) = {
        let mut running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);

        let still = running.as_mut().map(|recording| {
            let alive = match &mut recording.child {
                Some(child) => matches!(child.try_wait(), Ok(None)),
                None => alive(recording.pid),
            };

            let elapsed = SystemTime::now()
                .duration_since(recording.started)
                .unwrap_or_default()
                .as_secs();

            (alive, elapsed)
        });

        match still {
            Some((true, elapsed)) => (Some(elapsed), None),
            Some((false, _)) => (None, running.take()),
            None => (None, None),
        }
    };

    if Recording::read().elapsed != elapsed {
        Recording::write().elapsed = elapsed;
    }

    if let Some(recording) = finished {
        finish(&recording);
    }
}

fn finish(recording: &Running) {
    if let Some(path) = state::runtime_path(STATE) {
        let _ = fs::remove_file(path);
    }

    let written = fs::metadata(&recording.file).is_ok_and(|metadata| metadata.len() > 0);

    if written {
        notifications::send(
            APP_NAME,
            APP_NAME,
            &format!("Saved to {}", recording.file.display()),
            Urgency::Low,
            None,
        );
    } else {
        let log = state::runtime_path(LOG).map(|log| log_tail(&log));

        notifications::send(
            APP_NAME,
            "Recording failed",
            log.as_deref().unwrap_or("nothing was written"),
            Urgency::Critical,
            None,
        );
    }
}

// the last lines wf-recorder printed, for a failure notification
fn log_tail(log: &Path) -> String {
    let text = fs::read_to_string(log).unwrap_or_default();

    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if lines.is_empty() {
        return format!("{RECORDER} stopped without saying why");
    }

    lines[lines.len().saturating_sub(LOG_LINES)..].join("\n")
}

// the runtime file: pid, start in seconds since 1970, and the file, one per line
fn remember(recording: &Running) {
    let Some(path) = state::runtime_path(STATE) else {
        return;
    };

    let started = recording
        .started
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let contents = format!(
        "{}\n{started}\n{}\n",
        recording.pid,
        recording.file.display()
    );

    let _ = state::write_atomically(&path, &contents);
}

// picks up a recording started before a shell restart, if its recorder still runs
fn adopt() {
    let Some(path) = state::runtime_path(STATE) else {
        return;
    };

    let Ok(contents) = fs::read_to_string(&path) else {
        return;
    };

    let adopted = read_remembered(&contents).filter(|(pid, _, _)| {
        alive(*pid)
            && fs::read_to_string(format!("/proc/{pid}/comm"))
                .is_ok_and(|name| name.trim() == RECORDER)
    });

    let Some((pid, started, file)) = adopted else {
        let _ = fs::remove_file(path);

        return;
    };

    *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(Running {
        pid,
        started: UNIX_EPOCH + Duration::from_secs(started),
        file,
        child: None,
    });
}

fn read_remembered(contents: &str) -> Option<(u32, u64, PathBuf)> {
    let mut lines = contents.lines();

    let pid = lines.next()?.trim().parse().ok()?;
    let started = lines.next()?.trim().parse().ok()?;
    let file = lines.next()?.trim();

    (!file.is_empty()).then(|| (pid, started, PathBuf::from(file)))
}

// running, and not a zombie waiting to be reaped
fn alive(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| {
        // the state follows the name in parentheses, which may itself hold spaces
        stat.rsplit_once(')')
            .and_then(|(_, rest)| rest.trim_start().chars().next())
            .is_some_and(|state| state != 'Z')
    })
}

fn signal(pid: u32, signal: c_int) {
    unsafe extern "C" {
        fn kill(pid: c_int, signal: c_int) -> c_int;
    }

    let Ok(pid) = c_int::try_from(pid) else {
        return;
    };

    // a pid of 0 or below would signal a whole process group
    if pid <= 0 {
        return;
    }

    // SAFETY: kill takes plain integers and touches no memory of ours
    unsafe {
        kill(pid, signal);
    }
}

// ~/Videos/Screencasts/20261112190503.mp4, the folder made when missing
fn new_file() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|_| String::from("HOME is not set"))?;

    let folder = PathBuf::from(home).join("Videos").join("Screencasts");

    fs::create_dir_all(&folder)
        .map_err(|error| format!("couldn't create {}: {error}", folder.display()))?;

    Ok(folder.join(format!("{}.mp4", clock::local_now().stamp())))
}

// the bar's time: 0:42, 12:05, 1:02:03
pub fn elapsed_text(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = seconds / 60 % 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_reads_like_a_clock() {
        assert_eq!(elapsed_text(0), "0:00");
        assert_eq!(elapsed_text(42), "0:42");
        assert_eq!(elapsed_text(725), "12:05");
        assert_eq!(elapsed_text(3723), "1:02:03");
    }

    #[test]
    fn the_remembered_recording_reads_back() {
        assert_eq!(
            read_remembered("4242\n1762970000\n/home/me/Videos/Screencasts/1.mp4\n"),
            Some((
                4242,
                1762970000,
                PathBuf::from("/home/me/Videos/Screencasts/1.mp4")
            ))
        );
        assert_eq!(read_remembered("4242\n"), None);
        assert_eq!(read_remembered("not a pid\n1\n/file\n"), None);
    }

    #[test]
    fn the_focused_monitor_is_the_active_one() {
        let monitors = json::parse(
            r#"{"monitors": [{"name": "HDMI-A-1", "active": false},
                             {"name": "DP-1", "active": true}]}"#,
        )
        .expect("valid json");

        assert_eq!(focused_in(&monitors), Some(String::from("DP-1")));
        assert_eq!(focused_in(&Value::Null), None);
    }

    #[test]
    fn this_process_is_alive() {
        assert!(alive(std::process::id()));
        assert!(!alive(u32::MAX));
    }
}
