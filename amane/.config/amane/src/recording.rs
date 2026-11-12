/*
 * screen recording: wf-recorder films the focused monitor or a region picked
 * with slurp into ~/Videos/Screencasts/<stamp>.mp4, h264 on the gpu through
 * vaapi, falling back to software x264 when that fails at once. no sound
 *
 * the bar shows the time recorded while it runs; clicking it, SUPER+P (`record
 * menu`) or `record stop` sends wf-recorder SIGINT, which finishes the file.
 * a notification then says where it went, or why it failed. a recorded region
 * is framed in the accent, just outside it (frame_view)
 *
 * the recording's pid, start, file and frame are kept in the runtime folder,
 * so a recording still running after a shell restart is picked up again
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

use amane::{
    Color, Full, Keyboard, Layer, LayerWindow, Monitor, Rectangle, Service, Size::Parent, Stack,
    Urgency, Widget, Zone,
};

use crate::json::{self, Value};
use crate::panels::{Panels, SCREENSHOT};
use crate::screenshot::{self, Area};
use crate::theme::theme;
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

// the frame around a recorded region, and the room between it and the region
const FRAME_THICKNESS: f32 = 2.0;
const FRAME_GAP: f32 = 1.0;

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

    // the region being recorded, framed on screen; None for a whole monitor
    frame: Option<Frame>,
}

/*
 * a recorded region in its monitor's own widget units, for the frame drawn
 * around it. slurp gives the region across all monitors, so it's moved by the
 * position of the monitor it starts on
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    monitor: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/*
 * the region the frame window draws around; its own service, since Recording
 * changes every second and the frame only at the start and end
 */
pub struct RecordedRegion {
    frame: Option<Frame>,
}

impl Service for RecordedRegion {
    fn new() -> Self {
        Self { frame: None }
    }

    // written by Recording's thread and begin
    fn listen() {}
}

fn show_frame(frame: Option<Frame>) {
    if RecordedRegion::read().frame != frame {
        RecordedRegion::write().frame = frame;
    }
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

    let frame = match &place {
        Place::Region(geometry) => region_frame(geometry),
        Place::Monitor(_) => None,
    };

    let running = Running {
        pid: child.id(),
        started: SystemTime::now(),
        file,
        child: Some(child),
        frame: frame.clone(),
    };

    remember(&running);

    *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(running);

    Recording::write().elapsed = Some(0);

    show_frame(frame);

    Ok(())
}

// where on which monitor slurp's region is; None when mango doesn't say, then nothing is framed
fn region_frame(geometry: &str) -> Option<Frame> {
    let answer = command::read(
        Command::new("mmsg").args(["get", "all-monitors"]),
        MMSG_TIMEOUT,
    )?;

    locate(&json::parse(&answer)?, parse_geometry(geometry)?)
}

// slurp's "x,y wxh"
fn parse_geometry(geometry: &str) -> Option<[f32; 4]> {
    let (position, size) = geometry.trim().split_once(' ')?;
    let (x, y) = position.split_once(',')?;
    let (width, height) = size.split_once('x')?;

    Some([
        x.parse().ok()?,
        y.parse().ok()?,
        width.parse().ok()?,
        height.parse().ok()?,
    ])
}

// the monitor the region's top-left corner is on, from mango's monitor list
fn locate(monitors: &Value, [x, y, width, height]: [f32; 4]) -> Option<Frame> {
    monitors.get("monitors").list().iter().find_map(|monitor| {
        let left = monitor.get("x").number()? as f32;
        let top = monitor.get("y").number()? as f32;
        let right = left + monitor.get("width").number()? as f32;
        let bottom = top + monitor.get("height").number()? as f32;

        let inside = (left..right).contains(&x) && (top..bottom).contains(&y);

        inside.then(|| Frame {
            monitor: String::from(monitor.get("name").text()),
            x: x - left,
            y: y - top,
            width,
            height,
        })
    })
}

/*
 * a click-through window over each monitor that draws a frame in the accent
 * just outside the region being recorded, so the frame isn't in the video.
 * it's always mapped and never resizes (see the layer-surface crash rule);
 * without a region it draws nothing
 */
pub fn frame_view(monitor: &Monitor) -> LayerWindow {
    let frame = RecordedRegion::read()
        .frame
        .clone()
        .filter(|frame| frame.monitor == monitor.name);

    let mut layers: Vec<Box<dyn Widget>> = Vec::new();

    if let Some(frame) = frame {
        // the border is drawn inside a rectangle's edge, so the rectangle reaches past the region
        let outside = FRAME_GAP + FRAME_THICKNESS;

        layers.push(Box::new(
            Rectangle::new()
                .width(frame.width + 2.0 * outside)
                .height(frame.height + 2.0 * outside)
                .fill(Color::TRANSPARENT)
                .border(FRAME_THICKNESS, theme().accent)
                .translate(frame.x - outside, frame.y - outside),
        ));
    }

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .keyboard(Keyboard::None)
        .namespace("amane-recording-frame")
        .child(Stack::new(layers).width(Parent).height(Parent))
        .click_through()
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
        show_frame(None);

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

/*
 * the runtime file: pid, start in seconds since 1970, the file, and for a
 * region its frame as "monitor x y width height", one per line
 */
fn remember(recording: &Running) {
    let Some(path) = state::runtime_path(STATE) else {
        return;
    };

    let started = recording
        .started
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut contents = format!(
        "{}\n{started}\n{}\n",
        recording.pid,
        recording.file.display()
    );

    if let Some(frame) = &recording.frame {
        contents.push_str(&format!(
            "{} {} {} {} {}\n",
            frame.monitor, frame.x, frame.y, frame.width, frame.height
        ));
    }

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

    let adopted = read_remembered(&contents).filter(|remembered| {
        let pid = remembered.pid;

        alive(pid)
            && fs::read_to_string(format!("/proc/{pid}/comm"))
                .is_ok_and(|name| name.trim() == RECORDER)
    });

    let Some(remembered) = adopted else {
        let _ = fs::remove_file(path);

        return;
    };

    show_frame(remembered.frame.clone());

    *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(Running {
        pid: remembered.pid,
        started: UNIX_EPOCH + Duration::from_secs(remembered.started),
        file: remembered.file,
        child: None,
        frame: remembered.frame,
    });
}

#[derive(Debug, PartialEq)]
struct Remembered {
    pid: u32,
    started: u64,
    file: PathBuf,
    frame: Option<Frame>,
}

fn read_remembered(contents: &str) -> Option<Remembered> {
    let mut lines = contents.lines();

    let pid = lines.next()?.trim().parse().ok()?;
    let started = lines.next()?.trim().parse().ok()?;
    let file = lines.next()?.trim();

    if file.is_empty() {
        return None;
    }

    // a line that doesn't read leaves the recording unframed rather than lost
    let frame = lines.next().and_then(read_frame);

    Some(Remembered {
        pid,
        started,
        file: PathBuf::from(file),
        frame,
    })
}

// "DP-1 100 200 640 480", as remember writes it
fn read_frame(line: &str) -> Option<Frame> {
    let mut words = line.split_whitespace();

    let monitor = String::from(words.next()?);
    let mut number = || words.next()?.parse::<f32>().ok();

    Some(Frame {
        monitor,
        x: number()?,
        y: number()?,
        width: number()?,
        height: number()?,
    })
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
            Some(Remembered {
                pid: 4242,
                started: 1762970000,
                file: PathBuf::from("/home/me/Videos/Screencasts/1.mp4"),
                frame: None,
            })
        );
        assert_eq!(read_remembered("4242\n"), None);
        assert_eq!(read_remembered("not a pid\n1\n/file\n"), None);
    }

    #[test]
    fn a_remembered_region_keeps_its_frame() {
        let frame = Frame {
            monitor: String::from("DP-1"),
            x: 100.0,
            y: 200.5,
            width: 640.0,
            height: 480.0,
        };

        let remembered = read_remembered("1\n2\n/file\nDP-1 100 200.5 640 480\n");

        assert_eq!(
            remembered.and_then(|remembered| remembered.frame),
            Some(frame)
        );

        // a broken frame line still adopts the recording, unframed
        let broken = read_remembered("1\n2\n/file\nDP-1 wide\n").expect("the recording");

        assert_eq!(broken.frame, None);
    }

    #[test]
    fn slurps_geometry_reads() {
        assert_eq!(
            parse_geometry("100,200 640x480\n"),
            Some([100.0, 200.0, 640.0, 480.0])
        );
        assert_eq!(parse_geometry("100,200"), None);
        assert_eq!(parse_geometry(""), None);
    }

    #[test]
    fn a_region_is_framed_on_the_monitor_it_starts_on() {
        let monitors = json::parse(
            r#"{"monitors": [
                {"name": "DP-1", "x": 0, "y": 0, "width": 2560, "height": 1440},
                {"name": "HDMI-A-1", "x": 2560, "y": 0, "width": 1920, "height": 1080}]}"#,
        )
        .expect("valid json");

        let frame = locate(&monitors, [2660.0, 50.0, 300.0, 200.0]).expect("a monitor");

        assert_eq!(frame.monitor, "HDMI-A-1");
        assert_eq!((frame.x, frame.y), (100.0, 50.0));

        assert!(locate(&monitors, [5000.0, 50.0, 10.0, 10.0]).is_none());
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
