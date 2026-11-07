/*
 * facts about this machine for the system panel: who and where you are, the os,
 * kernel, core count, uptime and disk; cpu use, memory and battery come from amane's
 * own Cpu, Memory and Battery
 *
 * the fixed facts are read once; uptime and disk are read again only while
 * the panel is open, when it opens and every 30 seconds after
 */

use std::env;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::Service;

use crate::panels::{Panels, SYSTEM};

// one-second ticks between reads while the panel is open
const REFRESH_EVERY: u32 = 30;

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
            was_open: false,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    // read without holding the lock, written only when something shown changed
    fn listen() {
        let mut ticks: u32 = 0;

        loop {
            thread::sleep(Self::interval());

            ticks = ticks.wrapping_add(1);

            let open = Panels::read().is_open(SYSTEM);
            let was_open = SystemInfo::read().was_open;

            let due = open && (!was_open || ticks % REFRESH_EVERY == 0);

            let fresh = due.then(|| (uptime_minutes(), disk("/")));

            let changed = {
                let info = SystemInfo::read();

                open != info.was_open
                    || fresh.is_some_and(|(uptime, disk)| {
                        uptime != info.uptime_minutes || disk != info.disk
                    })
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
