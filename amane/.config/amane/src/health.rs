/*
 * health warnings: a background check every two minutes that sends one shell
 * notification when the cpu or gpu runs past its limit, / gets nearly full, or
 * a systemd unit (system or --user) fails. each problem warns once and can warn
 * again only after it clears; a temperature or the disk clears a little under
 * its limit, so a reading that hovers there doesn't warn over and over
 *
 * what has been warned about is kept in $XDG_RUNTIME_DIR, so `amane restart`
 * sends nothing again while a new login does
 *
 * the temperatures are also what the system panel's Temp row shows
 */

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::Urgency;

use crate::command;
use crate::notifications;
use crate::state;
use crate::system::{self, Disk};

const HWMON: &str = "/sys/class/hwmon";

// for a sensor that gives no tempN_max of its own, like amdgpu's edge
const FALLBACK_LIMIT_CELSIUS: u32 = 85;

// the meter's full end for a sensor without a tempN_crit
const FALLBACK_CRITICAL_CELSIUS: u32 = 100;

// a warned temperature clears only this far under its limit
const TEMPERATURE_HYSTERESIS: u32 = 5;

const DISK_WARN_PERCENT: u8 = 90;

// a warned disk clears only under DISK_WARN_PERCENT minus this
const DISK_HYSTERESIS: u8 = 2;

// so the warnings don't land among the start-up ones
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(30);

const CHECK_EVERY: Duration = Duration::from_secs(120);

// systemctl answers at once; a stuck manager shouldn't hold the check
const SYSTEMCTL_TIMEOUT: Duration = Duration::from_secs(5);

// one problem key per line, in the runtime folder
const WARNED_FILE: &str = "health-warned";

const APP_NAME: &str = "System";

const CPU_DRIVERS: [&str; 3] = ["k10temp", "zenpower", "coretemp"];
const CPU_LABELS: [&str; 2] = ["Tctl", "Package id 0"];

const GPU_DRIVERS: [&str; 3] = ["amdgpu", "nouveau", "radeon"];
const GPU_LABELS: [&str; 1] = ["edge"];

const KEY_DISK: &str = "disk:/";
const KEY_CPU: &str = "temp:cpu";
const KEY_GPU: &str = "temp:gpu";
const KEY_UNIT_SYSTEM: &str = "unit:system:";
const KEY_UNIT_USER: &str = "unit:user:";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    // whole degrees, so the panel only redraws when the shown text changes
    pub celsius: u32,

    // past this it warns: the sensor's tempN_max, else FALLBACK_LIMIT_CELSIUS
    pub limit: u32,

    // the meter's full end: tempN_crit, else FALLBACK_CRITICAL_CELSIUS
    pub critical: u32,
}

impl Temperature {
    // how far towards critical, for the panel's meter
    pub fn percent(&self) -> u8 {
        if self.critical == 0 {
            return 0;
        }

        (self.celsius * 100 / self.critical).min(100) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Temperatures {
    pub cpu: Option<Temperature>,
    pub gpu: Option<Temperature>,
}

pub fn temperatures() -> Temperatures {
    temperatures_in(Path::new(HWMON))
}

// the first cpu and gpu sensor under a hwmon folder, in hwmon number order
fn temperatures_in(root: &Path) -> Temperatures {
    let mut folders: Vec<_> = fs::read_dir(root)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();

    folders.sort();

    let mut found = Temperatures::default();

    for folder in folders {
        let name = read_trimmed(&folder.join("name")).unwrap_or_default();

        if found.cpu.is_none() && CPU_DRIVERS.contains(&name.as_str()) {
            found.cpu = sensor(&folder, &CPU_LABELS);
        } else if found.gpu.is_none() && GPU_DRIVERS.contains(&name.as_str()) {
            found.gpu = sensor(&folder, &GPU_LABELS);
        }
    }

    found
}

// the channel with one of these labels, else temp1
fn sensor(folder: &Path, labels: &[&str]) -> Option<Temperature> {
    let channel = (1..=16)
        .find(|number| {
            read_trimmed(&folder.join(format!("temp{number}_label")))
                .is_some_and(|label| labels.contains(&label.as_str()))
        })
        .unwrap_or(1);

    let degrees = |suffix: &str| {
        read_trimmed(&folder.join(format!("temp{channel}_{suffix}")))
            .and_then(|text| text.parse::<i64>().ok())
            // millidegrees; below zero reads as zero
            .map(|millidegrees| (millidegrees.max(0) / 1000) as u32)
    };

    Some(Temperature {
        celsius: degrees("input")?,
        limit: degrees("max")
            .filter(|&max| max > 0)
            .unwrap_or(FALLBACK_LIMIT_CELSIUS),
        critical: degrees("crit")
            .filter(|&crit| crit > 0)
            .unwrap_or(FALLBACK_CRITICAL_CELSIUS),
    })
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_string())
}

// the failed units in one scope; None when systemctl gave no answer
fn failed_units(user: bool) -> Option<Vec<String>> {
    let mut systemctl = Command::new("systemctl");

    if user {
        systemctl.arg("--user");
    }

    systemctl.args(["--failed", "--no-legend", "--plain"]);

    command::read(&mut systemctl, SYSTEMCTL_TIMEOUT).map(|text| parse_failed(&text))
}

// `foo.service loaded failed failed Foo`: the unit is the first column
fn parse_failed(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(String::from)
        .collect()
}

// everything one check reads; None where it couldn't be read
struct Reading {
    disk: Option<Disk>,
    temperatures: Temperatures,
    failed_system: Option<Vec<String>>,
    failed_user: Option<Vec<String>>,
}

#[derive(Debug, PartialEq)]
struct Warning {
    key: String,
    summary: String,
    body: String,
}

/*
 * the problems in a reading, each with what its warning would say; a problem
 * already warned about stays one until it clears past its hysteresis, and one
 * that couldn't be read this time stays as it was, so it doesn't warn again
 * once it can be read
 */
fn problems(reading: &Reading, warned: &BTreeSet<String>) -> Vec<Warning> {
    let mut found = Vec::new();

    match reading.disk {
        Some(disk) => {
            let percent = disk.percent();

            let threshold = if warned.contains(KEY_DISK) {
                DISK_WARN_PERCENT - DISK_HYSTERESIS
            } else {
                DISK_WARN_PERCENT
            };

            if percent >= threshold {
                found.push(Warning {
                    key: String::from(KEY_DISK),
                    summary: String::from("Disk almost full"),
                    body: format!(
                        "/ is {percent}% full, {} GiB free",
                        system::gib(disk.total.saturating_sub(disk.used))
                    ),
                });
            }
        }
        None => carry_over(&mut found, warned, KEY_DISK),
    }

    let sensors = [
        (KEY_CPU, "CPU", reading.temperatures.cpu),
        (KEY_GPU, "GPU", reading.temperatures.gpu),
    ];

    for (key, name, temperature) in sensors {
        let Some(temperature) = temperature else {
            carry_over(&mut found, warned, key);
            continue;
        };

        let hot = if warned.contains(key) {
            temperature.celsius > temperature.limit.saturating_sub(TEMPERATURE_HYSTERESIS)
        } else {
            temperature.celsius > temperature.limit
        };

        if hot {
            found.push(Warning {
                key: String::from(key),
                summary: format!("{name} is hot"),
                body: format!(
                    "{}°C, over its {}°C limit",
                    temperature.celsius, temperature.limit
                ),
            });
        }
    }

    let scopes = [
        (KEY_UNIT_SYSTEM, "system", &reading.failed_system),
        (KEY_UNIT_USER, "user", &reading.failed_user),
    ];

    for (prefix, scope, units) in scopes {
        let Some(units) = units else {
            for key in warned.iter().filter(|key| key.starts_with(prefix)) {
                carry_over(&mut found, warned, key);
            }

            continue;
        };

        for unit in units {
            found.push(Warning {
                key: format!("{prefix}{unit}"),
                summary: String::from("Unit failed"),
                body: format!("{unit} ({scope})"),
            });
        }
    }

    found
}

// an unreadable problem that was warned about stays warned, with nothing to send
fn carry_over(found: &mut Vec<Warning>, warned: &BTreeSet<String>, key: &str) {
    if warned.contains(key) {
        found.push(Warning {
            key: String::from(key),
            summary: String::new(),
            body: String::new(),
        });
    }
}

/*
 * the warnings to send (problems not warned about yet) and the warned set
 * from now on (every current problem; a cleared one drops out)
 */
fn update(warned: &BTreeSet<String>, current: Vec<Warning>) -> (Vec<Warning>, BTreeSet<String>) {
    let still: BTreeSet<String> = current.iter().map(|warning| warning.key.clone()).collect();

    let new = current
        .into_iter()
        .filter(|warning| !warned.contains(&warning.key))
        .collect();

    (new, still)
}

fn load_warned() -> BTreeSet<String> {
    state::runtime_path(WARNED_FILE)
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|text| text.lines().map(String::from).collect())
        .unwrap_or_default()
}

fn save_warned(warned: &BTreeSet<String>) {
    let Some(path) = state::runtime_path(WARNED_FILE) else {
        return;
    };

    let contents: String = warned.iter().map(|key| format!("{key}\n")).collect();

    // a lost save only means a warning may come again after a restart
    let _ = state::write_atomically(&path, &contents);
}

// starts the check on its own thread; it takes no service locks
pub fn start() {
    thread::spawn(|| {
        thread::sleep(FIRST_CHECK_DELAY);

        let mut warned = load_warned();

        loop {
            let reading = Reading {
                disk: system::disk("/"),
                temperatures: temperatures(),
                failed_system: failed_units(false),
                failed_user: failed_units(true),
            };

            let (new, still) = update(&warned, problems(&reading, &warned));

            /*
             * every health warning is critical: it stays until dismissed, with an
             * accent border. this thread may wait on the daemon's answer, never a view
             */
            for warning in &new {
                notifications::send(
                    APP_NAME,
                    &warning.summary,
                    &warning.body,
                    Urgency::Critical,
                    None,
                );
            }

            if still != warned {
                save_warned(&still);
                warned = still;
            }

            thread::sleep(CHECK_EVERY);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::path::PathBuf;

    // a fake /sys/class/hwmon: (folder, name, files)
    fn hwmon(test: &str, sensors: &[(&str, &str, &[(&str, &str)])]) -> PathBuf {
        let root = env::temp_dir().join(format!("amane-health-{test}-{}", std::process::id()));

        let _ = fs::remove_dir_all(&root);

        for (folder, name, files) in sensors {
            let folder = root.join(folder);

            fs::create_dir_all(&folder).expect("folder made");
            fs::write(folder.join("name"), format!("{name}\n")).expect("name written");

            for (file, contents) in *files {
                fs::write(folder.join(file), format!("{contents}\n")).expect("file written");
            }
        }

        root
    }

    fn reading(disk_percent: u64, cpu: u32) -> Reading {
        Reading {
            disk: Some(Disk {
                used: disk_percent,
                total: 100,
            }),
            temperatures: Temperatures {
                cpu: Some(Temperature {
                    celsius: cpu,
                    limit: 70,
                    critical: 100,
                }),
                gpu: None,
            },
            failed_system: Some(Vec::new()),
            failed_user: Some(Vec::new()),
        }
    }

    fn keys(warnings: &[Warning]) -> Vec<&str> {
        warnings
            .iter()
            .map(|warning| warning.key.as_str())
            .collect()
    }

    #[test]
    fn sensors_are_picked_by_driver_and_label() {
        let root = hwmon(
            "pick",
            &[
                ("hwmon0", "fam15h_power", &[]),
                (
                    "hwmon1",
                    "k10temp",
                    &[
                        ("temp1_input", "32000"),
                        ("temp1_max", "70000"),
                        ("temp1_crit", "100000"),
                    ],
                ),
                ("hwmon3", "iwlwifi_1", &[("temp1_input", "43000")]),
                (
                    "hwmon4",
                    "amdgpu",
                    &[
                        ("temp1_input", "1000"),
                        ("temp1_label", "junction"),
                        ("temp2_input", "45500"),
                        ("temp2_label", "edge"),
                    ],
                ),
            ],
        );

        let found = temperatures_in(&root);

        assert_eq!(
            found.cpu,
            Some(Temperature {
                celsius: 32,
                limit: 70,
                critical: 100
            })
        );

        // no max or crit of its own: the fallbacks
        assert_eq!(
            found.gpu,
            Some(Temperature {
                celsius: 45,
                limit: FALLBACK_LIMIT_CELSIUS,
                critical: FALLBACK_CRITICAL_CELSIUS
            })
        );

        fs::remove_dir_all(&root).expect("cleaned up");
    }

    #[test]
    fn no_sensors_reads_as_none() {
        let root = hwmon("none", &[("hwmon0", "acpitz", &[("temp1_input", "40000")])]);

        assert_eq!(temperatures_in(&root), Temperatures::default());
        assert_eq!(
            temperatures_in(&root.join("missing")),
            Temperatures::default()
        );

        fs::remove_dir_all(&root).expect("cleaned up");
    }

    #[test]
    fn a_meter_is_never_past_full() {
        let temperature = |celsius| Temperature {
            celsius,
            limit: 70,
            critical: 100,
        };

        assert_eq!(temperature(32).percent(), 32);
        assert_eq!(temperature(120).percent(), 100);
    }

    #[test]
    fn failed_units_are_the_first_column() {
        let text = "foo.service loaded failed failed Foo daemon\n\
                    bar.timer   loaded failed failed Bar\n";

        assert_eq!(parse_failed(text), ["foo.service", "bar.timer"]);
        assert!(parse_failed("").is_empty());
    }

    #[test]
    fn a_problem_warns_once_until_it_clears() {
        let mut warned = BTreeSet::new();

        // starts: warns
        let (new, still) = update(&warned, problems(&reading(91, 30), &warned));
        assert_eq!(keys(&new), [KEY_DISK]);
        assert_eq!(new[0].body, "/ is 91% full, 0.0 GiB free");
        warned = still;

        // lasts, and dips just under the limit: quiet, still warned
        let (new, still) = update(&warned, problems(&reading(89, 30), &warned));
        assert!(new.is_empty());
        assert!(still.contains(KEY_DISK));
        warned = still;

        // clears past the hysteresis
        let (new, still) = update(&warned, problems(&reading(87, 30), &warned));
        assert!(new.is_empty());
        assert!(still.is_empty());
        warned = still;

        // comes back: warns again
        let (new, _) = update(&warned, problems(&reading(95, 30), &warned));
        assert_eq!(keys(&new), [KEY_DISK]);
    }

    #[test]
    fn a_temperature_clears_under_its_limit_less_the_hysteresis() {
        let mut warned = BTreeSet::new();

        // at the limit is not past it
        let (new, _) = update(&warned, problems(&reading(10, 70), &warned));
        assert!(new.is_empty());

        let (new, still) = update(&warned, problems(&reading(10, 71), &warned));
        assert_eq!(keys(&new), [KEY_CPU]);
        assert_eq!(new[0].body, "71°C, over its 70°C limit");
        warned = still;

        let (_, still) = update(&warned, problems(&reading(10, 66), &warned));
        assert!(still.contains(KEY_CPU));

        let (_, still) = update(&warned, problems(&reading(10, 65), &warned));
        assert!(!still.contains(KEY_CPU));
    }

    #[test]
    fn each_failed_unit_warns_once() {
        let warned = BTreeSet::new();

        let mut first = reading(10, 30);
        first.failed_user = Some(vec![String::from("foo.service")]);

        let (new, warned) = update(&warned, problems(&first, &warned));
        assert_eq!(keys(&new), ["unit:user:foo.service"]);
        assert_eq!(new[0].body, "foo.service (user)");

        let mut second = reading(10, 30);
        second.failed_user = Some(vec![String::from("foo.service")]);
        second.failed_system = Some(vec![String::from("bar.service")]);

        let (new, _) = update(&warned, problems(&second, &warned));
        assert_eq!(keys(&new), ["unit:system:bar.service"]);
    }

    #[test]
    fn an_unreadable_problem_stays_warned_without_warning_again() {
        let warned: BTreeSet<String> = [KEY_DISK, KEY_CPU, "unit:user:foo.service"]
            .into_iter()
            .map(String::from)
            .collect();

        let unreadable = Reading {
            disk: None,
            temperatures: Temperatures::default(),
            failed_system: None,
            failed_user: None,
        };

        let (new, still) = update(&warned, problems(&unreadable, &warned));

        assert!(new.is_empty());
        assert_eq!(still, warned);
    }
}
