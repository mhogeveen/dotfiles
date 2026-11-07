/*
 * facts about this machine for the system panel: who and where you are, the os,
 * kernel, core count, uptime and disk; cpu use, memory and battery come from amane's
 * own Cpu, Memory and Battery
 *
 * the fixed facts are read once; uptime and disk are read again only while
 * the panel is open, when it opens and every 30 seconds after; the network
 * speed is sampled every second while it's open
 */

use std::collections::VecDeque;
use std::env;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::panels::{Panels, SYSTEM};

// one-second ticks between reads while the panel is open
const REFRESH_EVERY: u32 = 30;

// readings kept for the network speed: four readings span the last 3 seconds
const SAMPLES: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Disk {
    pub used: u64,
    pub total: u64,
}

impl Disk {
    pub fn percent(&self) -> u8 {
        if self.total == 0 {
            return 0;
        }

        (self.used * 100 / self.total).min(100) as u8
    }
}

pub struct SystemInfo {
    pub user: String,
    pub host: String,
    pub os: String,
    pub kernel: String,
    pub cores: usize,

    // whole minutes, so the panel only redraws when the shown text changes
    uptime_minutes: u64,

    // None when / couldn't be measured
    pub disk: Option<Disk>,

    // None while there's no default route
    pub network: Option<NetworkSpeed>,

    was_open: bool,
}

impl Service for SystemInfo {
    // plain file reads, quick enough for whichever view asks first
    fn new() -> Self {
        Self {
            user: env::var("USER").unwrap_or_default(),
            host: first_line("/proc/sys/kernel/hostname"),
            os: os_name(),
            kernel: first_line("/proc/sys/kernel/osrelease"),
            cores: thread::available_parallelism().map_or(0, usize::from),
            uptime_minutes: uptime_minutes(),
            disk: disk("/"),
            network: None,
            was_open: false,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    // read without holding the lock, written only when something shown changed
    fn listen() {
        let mut ticks: u32 = 0;
        let mut sampler = Sampler::default();

        loop {
            thread::sleep(Self::interval());

            ticks = ticks.wrapping_add(1);

            let open = Panels::read().is_open(SYSTEM);
            let was_open = SystemInfo::read().was_open;

            // a closed panel reads nothing, and the speed starts over when it opens
            if !open {
                sampler = Sampler::default();
            }

            let due = open && (!was_open || ticks % REFRESH_EVERY == 0);

            let fresh = due.then(|| (uptime_minutes(), disk("/")));

            let network = open.then(|| sampler.sample());

            let changed = {
                let info = SystemInfo::read();

                open != info.was_open
                    || fresh.is_some_and(|(uptime, disk)| {
                        uptime != info.uptime_minutes || disk != info.disk
                    })
                    || network
                        .as_ref()
                        .is_some_and(|network| *network != info.network)
            };

            if !changed {
                continue;
            }

            let mut info = SystemInfo::write();

            info.was_open = open;

            if let Some((uptime, disk)) = fresh {
                info.uptime_minutes = uptime;
                info.disk = disk;
            }

            if let Some(network) = network {
                info.network = network;
            }
        }
    }
}

impl SystemInfo {
    // like 33m, 2h 14m or 3d 4h
    pub fn uptime(&self) -> String {
        let minutes = self.uptime_minutes;

        let (days, hours, minutes) = (minutes / 1440, minutes / 60 % 24, minutes % 60);

        match (days, hours) {
            (0, 0) => format!("{minutes}m"),
            (0, _) => format!("{hours}h {minutes}m"),
            _ => format!("{days}d {hours}h"),
        }
    }
}

fn first_line(path: &str) -> String {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| text.lines().next().map(String::from))
        .unwrap_or_default()
}

// PRETTY_NAME="NixOS 26.05 (Yarara)"
fn os_name() -> String {
    let release = fs::read_to_string("/etc/os-release").unwrap_or_default();

    release
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|name| name.trim_matches('"').to_string())
        .unwrap_or_else(|| String::from("Linux"))
}

// the first number in /proc/uptime is seconds since boot
fn uptime_minutes() -> u64 {
    let uptime = fs::read_to_string("/proc/uptime").unwrap_or_default();

    uptime
        .split_whitespace()
        .next()
        .and_then(|seconds| seconds.parse::<f64>().ok())
        .map_or(0, |seconds| seconds as u64 / 60)
}

fn disk(path: &str) -> Option<Disk> {
    statvfs::disk(path).or_else(|| df(path))
}

/*
 * the C library's statvfs, called directly since the config can't add crates;
 * the struct below is glibc's layout on 64-bit linux, other targets use df
 */
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
mod statvfs {
    use std::ffi::{CString, c_char, c_int};

    use super::Disk;

    #[repr(C)]
    struct StatVfs {
        f_bsize: u64,
        f_frsize: u64,
        f_blocks: u64,
        f_bfree: u64,
        f_bavail: u64,
        f_files: u64,
        f_ffree: u64,
        f_favail: u64,
        f_fsid: u64,
        f_flag: u64,
        f_namemax: u64,
        f_spare: [c_int; 6],
    }

    unsafe extern "C" {
        fn statvfs(path: *const c_char, buf: *mut StatVfs) -> c_int;
    }

    // used counts like df does: everything not free, including root's reserve
    pub fn disk(path: &str) -> Option<super::Disk> {
        let path = CString::new(path).ok()?;

        let mut stats = std::mem::MaybeUninit::<StatVfs>::uninit();

        // SAFETY: path is a valid C string and stats is big enough for glibc's struct statvfs
        let result = unsafe { statvfs(path.as_ptr(), stats.as_mut_ptr()) };

        if result != 0 {
            return None;
        }

        // SAFETY: statvfs returned 0, so it filled in the whole struct
        let stats = unsafe { stats.assume_init() };

        let total = stats.f_blocks * stats.f_frsize;
        let used = stats.f_blocks.saturating_sub(stats.f_bfree) * stats.f_frsize;

        Some(Disk { used, total })
    }
}

#[cfg(not(all(target_os = "linux", target_pointer_width = "64")))]
mod statvfs {
    pub fn disk(_: &str) -> Option<super::Disk> {
        None
    }
}

// `df -B1 --output=size,used /`, for when statvfs isn't available
fn df(path: &str) -> Option<Disk> {
    let output = Command::new("df")
        .args(["-B1", "--output=size,used", path])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);

    let mut numbers = text.lines().nth(1)?.split_whitespace();

    let total = numbers.next()?.parse().ok()?;
    let used = numbers.next()?.parse().ok()?;

    Some(Disk { used, total })
}

// GiB with one decimal, like 2.1, the way free -h and df -h count
pub fn gib(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkSpeed {
    // the interface carrying the default route, like wlp2s0
    pub interface: String,

    // bytes per second down and up; None for the first second, before two readings
    pub rate: Option<(f64, f64)>,
}

/*
 * the last few byte counts of the default-route interface; the speed is what
 * moved between the oldest and newest reading, so an average over 3 seconds
 */
#[derive(Default)]
struct Sampler {
    interface: Option<String>,
    readings: VecDeque<(Instant, u64, u64)>,
}

impl Sampler {
    // None when there's no default route, which the panel shows as offline
    fn sample(&mut self) -> Option<NetworkSpeed> {
        let Some(interface) = default_interface() else {
            *self = Sampler::default();

            return None;
        };

        // counters of different interfaces can't be compared
        if self.interface.as_deref() != Some(interface.as_str()) {
            *self = Sampler {
                interface: Some(interface.clone()),
                readings: VecDeque::new(),
            };
        }

        let (Some(down), Some(up)) = (counter(&interface, "rx"), counter(&interface, "tx")) else {
            return Some(NetworkSpeed {
                interface,
                rate: None,
            });
        };

        // a counter going back, after a driver reset, starts the average over
        if self
            .readings
            .back()
            .is_some_and(|(_, last_down, last_up)| down < *last_down || up < *last_up)
        {
            self.readings.clear();
        }

        self.readings.push_back((Instant::now(), down, up));

        while self.readings.len() > SAMPLES {
            self.readings.pop_front();
        }

        Some(NetworkSpeed {
            interface,
            rate: self.rate(),
        })
    }

    fn rate(&self) -> Option<(f64, f64)> {
        let (first, first_down, first_up) = self.readings.front()?;
        let (last, last_down, last_up) = self.readings.back()?;

        let seconds = last.duration_since(*first).as_secs_f64();

        if seconds <= 0.0 {
            return None;
        }

        Some((
            (last_down - first_down) as f64 / seconds,
            (last_up - first_up) as f64 / seconds,
        ))
    }
}

/*
 * the interface whose route has destination 00000000 in /proc/net/route:
 *
 *   Iface   Destination  Gateway   Flags ...
 *   wlp2s0  00000000     0101A8C0  0003  ...
 */
fn default_interface() -> Option<String> {
    let routes = fs::read_to_string("/proc/net/route").ok()?;

    routes.lines().skip(1).find_map(|line| {
        let mut columns = line.split_whitespace();

        let interface = columns.next()?;
        let destination = columns.next()?;

        (destination == "00000000").then(|| String::from(interface))
    })
}

// rx or tx bytes since the interface came up
fn counter(interface: &str, direction: &str) -> Option<u64> {
    let path = format!("/sys/class/net/{interface}/statistics/{direction}_bytes");

    fs::read_to_string(path).ok()?.trim().parse().ok()
}

// like browsers count: 512 B/s, 1.2 kB/s, 34 kB/s, 120 MB/s
pub fn speed(bytes_per_second: f64) -> String {
    const UNITS: [&str; 4] = ["B/s", "kB/s", "MB/s", "GB/s"];

    let mut value = bytes_per_second.max(0.0);
    let mut unit = 0;

    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }

    if value < 10.0 && unit > 0 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{value:.0} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_can_be_measured() {
        let disk = disk("/").expect("/ has a size");

        assert!(disk.total > 0);
        assert!(disk.used <= disk.total);
    }

    #[test]
    fn statvfs_agrees_with_df() {
        let (Some(direct), Some(command)) = (statvfs::disk("/"), df("/")) else {
            return;
        };

        // the disk may change a little between the two reads
        assert_eq!(direct.total, command.total);
        assert!(direct.used.abs_diff(command.used) < 64 * 1024 * 1024);
    }

    #[test]
    fn speeds_read_like_a_browser() {
        assert_eq!(speed(512.0), "512 B/s");
        assert_eq!(speed(1_234.0), "1.2 kB/s");
        assert_eq!(speed(34_000.0), "34 kB/s");
        assert_eq!(speed(120_000_000.0), "120 MB/s");
    }

    #[test]
    fn the_default_route_has_counters() {
        let Some(interface) = default_interface() else {
            return;
        };

        assert!(counter(&interface, "rx").is_some());
        assert!(counter(&interface, "tx").is_some());
    }

    #[test]
    fn uptime_reads_at_minute_precision() {
        let info = |uptime_minutes| SystemInfo {
            uptime_minutes,
            ..SystemInfo::new()
        };

        assert_eq!(info(33).uptime(), "33m");
        assert_eq!(info(134).uptime(), "2h 14m");
        assert_eq!(info(4560).uptime(), "3d 4h");
    }
}
