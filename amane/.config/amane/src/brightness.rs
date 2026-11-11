/*
 * screen brightness for the system panel and the shortcuts: a laptop's backlight through amane's
 * Brightness, otherwise the monitor's own setting over DDC/CI with ddcutil
 * (VCP code 10). ddcutil takes a few tenths of a second per call, so it only
 * runs on threads:
 *
 *   - which i2c bus drives which monitor is found once (`ddcutil detect`)
 *   - the level is read when the system panel opens, at most every READ_EVERY
 *   - a slider drag sends many values, but only the newest is written, one
 *     call at a time, which also spares the monitor's settings memory
 *
 * the slider shows the value asked for right away, not when the monitor took it.
 * the shortcuts (`amane ipc call brightness up|down`) step 5% on the focused
 * monitor, and every change made here shows the popup (osd.rs)
 */

use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use amane::{Brightness, Service};

use crate::{command, panels};

// the panel reads the monitor again when it's been open this long
const READ_EVERY: Duration = Duration::from_secs(10);

/*
 * a read or write takes ~0.4 s, a detect about a second; past these a hung
 * ddcutil is killed, so the one-at-a-time flags are cleared and later changes go through
 */
const CALL_TIMEOUT: Duration = Duration::from_secs(5);
const DETECT_TIMEOUT: Duration = Duration::from_secs(15);

// VCP feature code for brightness
const VCP_BRIGHTNESS: &str = "10";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
    // being read for the first time
    Reading,

    // no backlight, and no monitor answering over DDC/CI
    Unavailable,

    Percent(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Bus {
    number: u32,

    // the monitor's highest value, 100 for most
    max: u32,
}

pub struct Display {
    // None until ddcutil detect ran; then the bus per monitor, like DP-1
    buses: Option<HashMap<String, Bus>>,

    // per monitor, as read or as asked for since
    percent: HashMap<String, u8>,

    // monitors whose reading failed
    failed: Vec<String>,
}

impl Service for Display {
    fn new() -> Self {
        Self {
            buses: None,
            percent: HashMap::new(),
            failed: Vec::new(),
        }
    }

    // read when the panel opens, nothing to poll
    fn listen() {}
}

// reading, safe inside views
pub fn level(monitor: &str) -> Level {
    if let Some(percent) = backlight() {
        return Level::Percent(percent);
    }

    let display = Display::read();

    if let Some(percent) = display.percent.get(monitor) {
        return Level::Percent(*percent);
    }

    let unknown_monitor = display
        .buses
        .as_ref()
        .is_some_and(|buses| !buses.contains_key(monitor));

    if unknown_monitor || display.failed.iter().any(|failed| failed == monitor) {
        Level::Unavailable
    } else {
        Level::Reading
    }
}

// a laptop's backlight, None on a desktop
fn backlight() -> Option<u8> {
    let brightness = Brightness::read();

    brightness.present().then(|| brightness.percent())
}

// the monitor the system panel shows, which the slider then sets
static TARGET: Mutex<Option<String>> = Mutex::new(None);

// when each monitor was last read; not part of the service, so starting one redraws nothing
static READ_AT: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);
static READING: AtomicBool = AtomicBool::new(false);

/*
 * remembers which monitor the slider sets, and reads its level when that's due;
 * safe to call from a view, it only starts a thread
 */
pub fn follow(monitor: &str) {
    *TARGET.lock().unwrap_or_else(PoisonError::into_inner) = Some(String::from(monitor));

    if Brightness::read().present() {
        return;
    }

    {
        let mut read_at = READ_AT.lock().unwrap_or_else(PoisonError::into_inner);
        let read_at = read_at.get_or_insert_with(HashMap::new);

        if read_at
            .get(monitor)
            .is_some_and(|at| at.elapsed() < READ_EVERY)
        {
            return;
        }

        if READING.swap(true, Ordering::AcqRel) {
            return;
        }

        read_at.insert(String::from(monitor), Instant::now());
    }

    let monitor = String::from(monitor);

    thread::spawn(move || {
        read_level(&monitor);

        READING.store(false, Ordering::Release);
    });
}

fn read_level(monitor: &str) {
    if Display::read().buses.is_none() {
        let buses = detect();

        Display::write().buses = Some(buses);
    }

    let Some(bus) = Display::read()
        .buses
        .as_ref()
        .and_then(|buses| buses.get(monitor).copied())
    else {
        return;
    };

    let answer = ddcutil(bus.number, &["getvcp", VCP_BRIGHTNESS, "--brief"]);

    let mut display = Display::write();

    match answer.as_deref().and_then(parse_level) {
        Some((current, max)) => {
            if let Some(buses) = display.buses.as_mut()
                && let Some(bus) = buses.get_mut(monitor)
            {
                bus.max = max;
            }

            // a write still on its way wins over what the monitor said before it
            if !WRITING.load(Ordering::Acquire) {
                display
                    .percent
                    .insert(String::from(monitor), percent_of(current, max));
            }

            display.failed.retain(|failed| failed != monitor);
        }
        None => {
            if !display.failed.iter().any(|failed| failed == monitor) {
                display.failed.push(String::from(monitor));
            }
        }
    }
}

// the newest value asked for, as (bus, value); only this one gets written
static WANTED: Mutex<Option<(u32, u32)>> = Mutex::new(None);
static WRITING: AtomicBool = AtomicBool::new(false);

// bumped on every change made through the shell, so the popup (osd.rs) can tell
static CHANGES: AtomicU64 = AtomicU64::new(0);

pub fn changes() -> u64 {
    CHANGES.load(Ordering::Acquire)
}

// from the slider, so a plain fn; the monitor the system panel is on
pub fn set(percent: u8) {
    let Some(monitor) = TARGET
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
    else {
        return;
    };

    set_on(&monitor, percent);
}

fn set_on(monitor: &str, percent: u8) {
    if Brightness::read().present() {
        Brightness::set(percent);

        CHANGES.fetch_add(1, Ordering::AcqRel);

        return;
    }

    let bus = {
        let mut display = Display::write();

        let Some(bus) = display
            .buses
            .as_ref()
            .and_then(|buses| buses.get(monitor).copied())
        else {
            return;
        };

        display.percent.insert(String::from(monitor), percent);

        bus
    };

    CHANGES.fetch_add(1, Ordering::AcqRel);

    *WANTED.lock().unwrap_or_else(PoisonError::into_inner) =
        Some((bus.number, value_of(percent, bus.max)));

    if !WRITING.swap(true, Ordering::AcqRel) {
        thread::spawn(write_newest);
    }
}

// one step at a time, so quick presses build on each other
static STEPPING: Mutex<()> = Mutex::new(());

/*
 * up or down from where it is, on the focused monitor; from the shortcuts.
 * the first time it may have to find and read the monitor, so on a thread
 */
pub fn step(by: i32) {
    thread::spawn(move || {
        let _one_at_a_time = STEPPING.lock().unwrap_or_else(PoisonError::into_inner);

        if let Some(percent) = backlight() {
            set_on("", stepped(percent, by));

            return;
        }

        let Some(monitor) = panels::focused_output().or_else(first_monitor) else {
            return;
        };

        if !Display::read().percent.contains_key(&monitor) {
            read_level(&monitor);
        }

        let current = Display::read().percent.get(&monitor).copied();

        if let Some(current) = current {
            set_on(&monitor, stepped(current, by));
        }
    });
}

// without mango to say which monitor has focus, the first one ddcutil found
fn first_monitor() -> Option<String> {
    if Display::read().buses.is_none() {
        let buses = detect();

        Display::write().buses = Some(buses);
    }

    let display = Display::read();

    let mut names: Vec<&String> = display.buses.as_ref()?.keys().collect();

    names.sort();

    names.first().map(|name| String::from(name.as_str()))
}

fn stepped(percent: u8, by: i32) -> u8 {
    (i32::from(percent) + by).clamp(0, 100) as u8
}

// the shortcuts' step
const STEP: i32 = 5;

// `amane ipc call brightness up|down`
pub fn ipc(arguments: &[String]) -> String {
    match arguments.first().map(String::as_str) {
        Some("up") => step(STEP),
        Some("down") => step(-STEP),
        _ => return String::from("usage: brightness up | brightness down"),
    }

    String::from(arguments[0].as_str())
}

// writes until nothing newer was asked for
fn write_newest() {
    loop {
        let next = WANTED.lock().unwrap_or_else(PoisonError::into_inner).take();

        match next {
            Some((bus, value)) => {
                ddcutil(
                    bus,
                    &["setvcp", VCP_BRIGHTNESS, &value.to_string(), "--noverify"],
                );
            }
            None => {
                WRITING.store(false, Ordering::Release);

                // asked for again just as this one stopped: carry on, unless another took over
                let more = WANTED
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .is_some();

                if more && !WRITING.swap(true, Ordering::AcqRel) {
                    continue;
                }

                return;
            }
        }
    }
}

// ddcutil's answer, None when it isn't installed, failed or hung
fn ddcutil(bus: u32, arguments: &[&str]) -> Option<String> {
    let output = command::capture(
        Command::new("ddcutil")
            .arg("--bus")
            .arg(bus.to_string())
            .args(arguments),
        CALL_TIMEOUT,
    )
    .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

// every monitor ddcutil can talk to, by connector name like DP-1
fn detect() -> HashMap<String, Bus> {
    let Ok(output) = command::capture(
        Command::new("ddcutil").args(["detect", "--brief"]),
        DETECT_TIMEOUT,
    ) else {
        return HashMap::new();
    };

    parse_detect(&String::from_utf8_lossy(&output.stdout))
}

/*
 *   Display 1
 *      I2C bus:          /dev/i2c-5
 *      DRM connector:    card1-DP-1
 *
 * "Invalid display" blocks are monitors that don't answer DDC/CI, left out
 */
fn parse_detect(text: &str) -> HashMap<String, Bus> {
    let mut buses = HashMap::new();

    let mut valid = false;
    let mut bus: Option<u32> = None;
    let mut connector: Option<String> = None;

    let mut finish = |valid: bool, bus: &mut Option<u32>, connector: &mut Option<String>| {
        if let (true, Some(number), Some(name)) = (valid, bus.take(), connector.take()) {
            buses.insert(name, Bus { number, max: 100 });
        }

        *bus = None;
        *connector = None;
    };

    for line in text.lines() {
        if !line.starts_with(char::is_whitespace) {
            finish(valid, &mut bus, &mut connector);

            valid = line.starts_with("Display ");

            continue;
        }

        let line = line.trim();

        if let Some(path) = line.strip_prefix("I2C bus:") {
            bus = path
                .trim()
                .strip_prefix("/dev/i2c-")
                .and_then(|n| n.parse().ok());
        } else if let Some(name) = line.strip_prefix("DRM connector:") {
            // card1-DP-1 is DP-1, the name mango and amane use
            let name = name.trim();

            connector = Some(String::from(
                name.split_once('-').map_or(name, |(_, rest)| rest),
            ));
        }
    }

    finish(valid, &mut bus, &mut connector);

    buses
}

// "VCP 10 C 65 100" is 65 of 100
fn parse_level(text: &str) -> Option<(u32, u32)> {
    let mut words = text.split_whitespace();

    if words.next()? != "VCP" || words.next()? != VCP_BRIGHTNESS || words.next()? != "C" {
        return None;
    }

    let current: u32 = words.next()?.parse().ok()?;
    let max: u32 = words.next()?.parse().ok()?;

    (max > 0).then_some((current.min(max), max))
}

fn percent_of(current: u32, max: u32) -> u8 {
    (current * 100 / max.max(1)).min(100) as u8
}

fn value_of(percent: u8, max: u32) -> u32 {
    u32::from(percent.min(100)) * max / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    // from this machine, with an extra display that doesn't answer
    const DETECT: &str = "Display 1
   I2C bus:          /dev/i2c-5
   DRM connector:    card1-DP-1
   drm_connector_id: 60
   Monitor:          AOC:U27B3AF:ZXLR3HA000596

Invalid display
   I2C bus:          /dev/i2c-7
   DRM connector:    card1-HDMI-A-1
   Monitor:          ACME:Nope

Display 2
   I2C bus:          /dev/i2c-8
   DRM connector:    card0-HDMI-A-2
   Monitor:          DEL:U2720Q
";

    #[test]
    fn detect_maps_connectors_to_buses() {
        let buses = parse_detect(DETECT);

        assert_eq!(buses.len(), 2);
        assert_eq!(buses["DP-1"].number, 5);
        assert_eq!(buses["HDMI-A-2"].number, 8);
        assert!(!buses.contains_key("HDMI-A-1"));
    }

    #[test]
    fn no_displays_is_empty() {
        assert!(parse_detect("").is_empty());
        assert!(parse_detect("No displays found.\n").is_empty());
    }

    #[test]
    fn levels_are_read_and_scaled() {
        assert_eq!(parse_level("VCP 10 C 65 100\n"), Some((65, 100)));
        assert_eq!(parse_level("VCP 10 C 30 60"), Some((30, 60)));
        assert_eq!(parse_level("VCP 10 C 70 50"), Some((50, 50)));
        assert_eq!(parse_level("VCP 10 C 5 0"), None);
        assert_eq!(parse_level("VCP 12 C 65 100"), None);
        assert_eq!(parse_level("garbage"), None);
        assert_eq!(parse_level(""), None);

        assert_eq!(percent_of(30, 60), 50);
        assert_eq!(value_of(50, 60), 30);
        assert_eq!(value_of(65, 100), 65);

        assert_eq!(stepped(65, 5), 70);
        assert_eq!(stepped(98, 5), 100);
        assert_eq!(stepped(3, -5), 0);
    }
}
