/*
 * `~/.cache/amane/amane-shell doctor`: what this host is missing for the
 * shell, for setting it up on a new machine (NixOS or not). it runs instead
 * of the shell (main() checks the argument before starting the app), so it
 * works before the shell has ever started and never blocks a running one
 *
 * every check reads files or asks a program with a time limit, and changes
 * nothing. the programs, units and render node come from the settings
 * (src/settings.rs), so it checks what this host actually uses. the report
 * lists each finding with the feature it breaks and how to fix it; the exit
 * status is 1 when something the shell can't start without is missing
 */

use std::env;
use std::ffi::{CString, c_char, c_int};
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::command;
use crate::settings::{RenderNode, settings};
use crate::user_unit::{self, State};

// busctl, fc-list and tailscale answer at once; past this they're stuck
const CALL_TIMEOUT: Duration = Duration::from_secs(5);

const FONT: &str = "BlexMono Nerd Font";
const NOTIFICATIONS_NAME: &str = "org.freedesktop.Notifications";
const SHELL_PROCESS: &str = "amane-shell";

const BACKLIGHTS: &str = "/sys/class/backlight";
const DEVICES: &str = "/dev";
const I2C_PREFIX: &str = "i2c-";

// access(2): may this user read and write it
const READ_WRITE: c_int = 4 | 2;

// any execute bit
const EXECUTABLE: u32 = 0o111;

// the report's columns: status, then what was checked
const STATUS_WIDTH: usize = 9;
const WHAT_WIDTH: usize = 28;

// programs the shell can't do without, and what each is for
const REQUIRED_PROGRAMS: [(&str, &str); 4] = [
    ("mmsg", "compositor state (MangoWM)"),
    ("systemctl", "suspend, reboot, failed units"),
    ("df", "disk use in the system panel"),
    ("ip", "vpn detection"),
];

// programs a single feature needs
const FEATURE_PROGRAMS: [(&str, &str); 13] = [
    ("wpctl", "audio panel, volume popup"),
    ("pw-dump", "privacy dot"),
    ("pw-mon", "privacy dot"),
    ("cliphist", "clipboard history"),
    ("wl-copy", "clipboard, screenshots to the clipboard"),
    ("wl-paste", "clipboard history"),
    ("grim", "screenshots"),
    ("slurp", "screenshots and recordings of a region"),
    ("curl", "weather, album art"),
    ("rfkill", "bluetooth on and off"),
    ("tailscale", "vpn card"),
    ("xdg-open", "tailscale login page"),
    ("ddcutil", "brightness of an external monitor"),
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    Ok,
    Missing,
    // couldn't tell, like without fc-list to ask
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
struct Finding {
    status: Status,
    // the shell doesn't start, or starts broken, without it
    required: bool,
    what: String,
    feature: String,
    // how to fix it; empty when it's fine
    hint: String,
}

impl Finding {
    fn new(status: Status, required: bool, what: &str, feature: &str, hint: &str) -> Self {
        Self {
            status,
            required,
            what: String::from(what),
            feature: String::from(feature),
            hint: String::from(hint),
        }
    }
}

// runs every check, prints the report, and gives the exit status
pub fn run() -> i32 {
    let (text, healthy) = report(&checks());

    print!("{text}");

    if healthy { 0 } else { 1 }
}

fn checks() -> Vec<Finding> {
    let path = env::var_os("PATH").unwrap_or_default();
    let on_path = |program: &str| find_in_path(program, &path.to_string_lossy(), is_executable);

    let mut findings = Vec::new();

    for (program, feature) in REQUIRED_PROGRAMS {
        findings.push(program_finding(
            program,
            true,
            feature,
            "",
            on_path(program),
        ));
    }

    /*
     * these ask systemd and d-bus; without the program to ask, the answer is
     * unknown, not missing
     */
    let can_ask_systemd = on_path("systemctl").is_some();
    let can_ask_bus = on_path("busctl").is_some();

    findings.push(match can_ask_bus {
        true => notification_daemon(),
        false => cannot_ask("busctl", true, "notification daemon", "notifications"),
    });

    let settings = settings();
    let configured = [
        (settings.terminal, "terminal", "launcher's >>command"),
        (settings.annotator, "annotator", "screenshot annotation"),
        (settings.recorder, "recorder", "screen recording"),
    ];

    for (value, setting, feature) in configured {
        let program = first_word(value);

        findings.push(program_finding(
            program,
            false,
            feature,
            setting,
            on_path(program),
        ));
    }

    for (program, feature) in FEATURE_PROGRAMS {
        findings.push(program_finding(
            program,
            false,
            feature,
            "",
            on_path(program),
        ));
    }

    let units = [
        (settings.idle_unit, "idle_unit", "idle switch"),
        (
            settings.night_light_unit,
            "night_light_unit",
            "night light switch",
        ),
    ];

    for (unit, setting, feature) in units {
        findings.push(match can_ask_systemd {
            true => unit_finding(unit, setting, feature),
            false => cannot_ask("systemctl", false, unit, feature),
        });
    }

    findings.push(render_node(&settings.render_node));
    findings.push(font());
    findings.push(brightness());

    let services = [
        (
            "org.freedesktop.NetworkManager",
            "NetworkManager",
            "network panel",
        ),
        ("org.bluez", "BlueZ", "bluetooth panel"),
    ];

    for (name, what, feature) in services {
        findings.push(match can_ask_bus {
            true => system_service(name, what, feature),
            false => cannot_ask("busctl", false, what, feature),
        });
    }

    if on_path("tailscale").is_some() {
        findings.push(tailscale_operator());
    }

    findings
}

/*
 * a program's finding; setting names the settings field it came from, empty
 * for a fixed one, so the hint can say where to change it
 */
fn program_finding(
    program: &str,
    required: bool,
    feature: &str,
    setting: &str,
    found: Option<PathBuf>,
) -> Finding {
    let what = if setting.is_empty() {
        String::from(program)
    } else {
        format!("{program} ({setting})")
    };

    let (status, hint) = match found {
        Some(_) => (Status::Ok, String::new()),
        None if setting.is_empty() => (Status::Missing, format!("install {program}")),
        None => (
            Status::Missing,
            format!("install {program}, or set {setting} in src/settings/<host>.rs"),
        ),
    };

    Finding::new(status, required, &what, feature, &hint)
}

// a check that couldn't run, since the program that answers it isn't installed
fn cannot_ask(program: &str, required: bool, what: &str, feature: &str) -> Finding {
    Finding::new(
        Status::Unknown,
        required,
        what,
        feature,
        &format!("couldn't check without {program} (systemd)"),
    )
}

fn unit_finding(unit: &str, setting: &str, feature: &str) -> Finding {
    match user_unit::read_state(unit) {
        State::Active(_) => Finding::new(Status::Ok, false, unit, feature, ""),
        _ => Finding::new(
            Status::Missing,
            false,
            unit,
            feature,
            &format!("no such user unit; add it, or set {setting} in src/settings/<host>.rs"),
        ),
    }
}

fn render_node(node: &RenderNode) -> Finding {
    let feature = "recording on the gpu";

    match (node, node.path()) {
        (RenderNode::None, _) => Finding::new(Status::Ok, false, "software", feature, ""),
        (_, Some(path)) if path.exists() => {
            Finding::new(Status::Ok, false, &path.to_string_lossy(), feature, "")
        }
        (_, path) => Finding::new(
            Status::Missing,
            false,
            &path.map_or(String::from("render node"), |path| {
                path.to_string_lossy().into_owned()
            }),
            feature,
            "no gpu render node; records in software (check render_node)",
        ),
    }
}

fn font() -> Finding {
    let feature = "every text in the shell";
    let answer = command::read(Command::new("fc-list").args([FONT, "family"]), CALL_TIMEOUT);

    match answer {
        Some(families) if !families.trim().is_empty() => {
            Finding::new(Status::Ok, true, FONT, feature, "")
        }
        Some(_) => Finding::new(
            Status::Missing,
            true,
            FONT,
            feature,
            "install it (nixpkgs: nerd-fonts.blex-mono)",
        ),
        None => Finding::new(
            Status::Unknown,
            true,
            FONT,
            feature,
            "couldn't ask fc-list (fontconfig)",
        ),
    }
}

// a laptop's backlight needs nothing; a monitor needs an i2c device this user may use
fn brightness() -> Finding {
    let feature = "brightness slider and keys";

    let backlight = fs::read_dir(BACKLIGHTS).is_ok_and(|mut entries| entries.next().is_some());

    if backlight {
        return Finding::new(Status::Ok, false, "backlight", feature, "");
    }

    let devices: Vec<PathBuf> = fs::read_dir(DEVICES)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.file_name().to_string_lossy().starts_with(I2C_PREFIX))
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default();

    match (
        devices.is_empty(),
        devices.iter().any(|device| may_read_write(device)),
    ) {
        (_, true) => Finding::new(Status::Ok, false, "i2c (DDC/CI)", feature, ""),
        (true, _) => Finding::new(
            Status::Missing,
            false,
            "i2c (DDC/CI)",
            feature,
            "no /dev/i2c-*: enable hardware.i2c (NixOS) or load i2c-dev",
        ),
        (false, _) => Finding::new(
            Status::Missing,
            false,
            "i2c (DDC/CI)",
            feature,
            "no access to /dev/i2c-*: join the i2c group",
        ),
    }
}

fn may_read_write(path: &Path) -> bool {
    unsafe extern "C" {
        fn access(path: *const c_char, mode: c_int) -> c_int;
    }

    let Ok(path) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };

    // SAFETY: a valid nul-terminated path; access only reads it
    unsafe { access(path.as_ptr(), READ_WRITE) == 0 }
}

/*
 * the shell is the notification daemon; another one holding the name keeps it
 * from starting its own. no owner (the shell isn't running) is fine
 */
fn notification_daemon() -> Finding {
    let feature = "notifications";
    let answer = command::read(
        Command::new("busctl").args(["--user", "status", NOTIFICATIONS_NAME]),
        CALL_TIMEOUT,
    );

    match notification_owner(answer.as_deref().unwrap_or("")) {
        None => Finding::new(Status::Ok, true, "notification daemon", feature, ""),
        Some(owner) if owner == SHELL_PROCESS => {
            Finding::new(Status::Ok, true, "notification daemon", feature, "")
        }
        Some(owner) => Finding::new(
            Status::Missing,
            true,
            "notification daemon",
            feature,
            &format!("{owner} holds the name: stop and disable it"),
        ),
    }
}

// the program holding the name in busctl status' answer, from its Comm= line
fn notification_owner(answer: &str) -> Option<String> {
    answer
        .lines()
        .find_map(|line| line.strip_prefix("Comm="))
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
}

fn system_service(name: &str, what: &str, feature: &str) -> Finding {
    let answer = command::read(
        Command::new("busctl").args(["--system", "status", name]),
        CALL_TIMEOUT,
    );

    match answer {
        Some(_) => Finding::new(Status::Ok, false, what, feature, ""),
        None => Finding::new(
            Status::Missing,
            false,
            what,
            feature,
            &format!("{what} isn't running"),
        ),
    }
}

// the vpn card connects and picks exit nodes as this user, which tailscale only allows an operator
fn tailscale_operator() -> Finding {
    let feature = "vpn card actions";
    let user = env::var("USER").unwrap_or_default();
    let answer = command::read(
        Command::new("tailscale").args(["debug", "prefs"]),
        CALL_TIMEOUT,
    );

    match answer {
        Some(prefs) if operator_is(&prefs, &user) => {
            Finding::new(Status::Ok, false, "tailscale operator", feature, "")
        }
        Some(_) => Finding::new(
            Status::Missing,
            false,
            "tailscale operator",
            feature,
            "run `sudo tailscale set --operator=$USER` once",
        ),
        None => Finding::new(
            Status::Unknown,
            false,
            "tailscale operator",
            feature,
            "tailscaled isn't answering",
        ),
    }
}

// whether `tailscale debug prefs` names this user as the operator
fn operator_is(prefs: &str, user: &str) -> bool {
    !user.is_empty()
        && prefs.lines().any(|line| {
            line.trim()
                .strip_prefix("\"OperatorUser\":")
                .is_some_and(|value| value.trim().trim_end_matches(',').trim_matches('"') == user)
        })
}

// the first folder in a PATH-style list holding the program as an executable file
fn find_in_path(
    program: &str,
    path: &str,
    is_executable: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if program.is_empty() {
        return None;
    }

    // a path of its own isn't looked up
    if program.contains('/') {
        let program = PathBuf::from(program);

        return is_executable(&program).then_some(program);
    }

    path.split(':')
        .filter(|folder| !folder.is_empty())
        .map(|folder| Path::new(folder).join(program))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & EXECUTABLE != 0)
}

// `alacritty -e` runs alacritty
fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap_or("")
}

/*
 * the findings under "required" and "features", then a count; healthy when
 * nothing required is missing
 */
fn report(findings: &[Finding]) -> (String, bool) {
    let mut text = String::new();

    for (heading, required) in [("required", true), ("features", false)] {
        text.push_str(heading);
        text.push('\n');

        for finding in findings
            .iter()
            .filter(|finding| finding.required == required)
        {
            let status = match finding.status {
                Status::Ok => "ok",
                Status::Missing => "missing",
                Status::Unknown => "unknown",
            };

            let mut line = format!(
                "  {status:<STATUS_WIDTH$}{:<WHAT_WIDTH$} {}",
                finding.what, finding.feature
            );

            if !finding.hint.is_empty() {
                line.push_str(" · ");
                line.push_str(&finding.hint);
            }

            text.push_str(line.trim_end());
            text.push('\n');
        }
    }

    let missing = |required: bool| {
        findings
            .iter()
            .filter(|finding| finding.status == Status::Missing && (finding.required || !required))
            .count()
    };

    let (all, required) = (missing(false), missing(true));

    text.push_str(&match all {
        0 => String::from("nothing missing\n"),
        _ => format!("{all} missing ({required} required)\n"),
    });

    (text, required == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // a fake file system: only these paths are executables
    fn executables(paths: &'static [&'static str]) -> impl Fn(&Path) -> bool {
        move |path| paths.iter().any(|known| Path::new(known) == path)
    }

    #[test]
    fn a_program_is_found_in_a_later_folder() {
        let found = find_in_path("grim", "/a:/b", executables(&["/b/grim"]));

        assert_eq!(found, Some(PathBuf::from("/b/grim")));
    }

    #[test]
    fn a_program_that_isnt_there_or_runnable_is_missing() {
        assert_eq!(find_in_path("grim", "/a:/b", executables(&[])), None);
        assert_eq!(find_in_path("grim", "", executables(&["/grim"])), None);
        assert_eq!(find_in_path("", "/a", executables(&["/a"])), None);
    }

    #[test]
    fn a_path_of_its_own_is_checked_as_is() {
        let found = find_in_path("/opt/foot", "/a", executables(&["/opt/foot"]));

        assert_eq!(found, Some(PathBuf::from("/opt/foot")));
    }

    #[test]
    fn a_setting_with_arguments_checks_the_program() {
        assert_eq!(first_word("alacritty -e"), "alacritty");
        assert_eq!(first_word(""), "");
    }

    #[test]
    fn the_notification_owner_comes_from_comm() {
        let answer = "PID=35224\nPPID=1\nComm=amane-shell\nCommandLine=amane-shell\n";

        assert_eq!(notification_owner(answer).as_deref(), Some("amane-shell"));
        assert_eq!(notification_owner("Comm=mako\n").as_deref(), Some("mako"));
        assert_eq!(notification_owner(""), None);
    }

    #[test]
    fn the_operator_is_matched_by_name() {
        let prefs = "{\n\t\"ControlURL\": \"https://controlplane.tailscale.com\",\n\t\"OperatorUser\": \"maarten\",\n}";

        assert!(operator_is(prefs, "maarten"));
        assert!(!operator_is(prefs, "someone"));
        assert!(!operator_is("{}", "maarten"));
        assert!(!operator_is(prefs, ""));
    }

    #[test]
    fn the_report_groups_and_counts() {
        let findings = [
            Finding::new(Status::Ok, true, "mmsg", "compositor state", ""),
            Finding::new(
                Status::Missing,
                false,
                "satty",
                "annotation",
                "install satty",
            ),
            Finding::new(
                Status::Unknown,
                false,
                "tailscale operator",
                "vpn",
                "no answer",
            ),
        ];

        let (text, healthy) = report(&findings);

        assert!(healthy);
        assert_eq!(
            text,
            "required\n  ok       mmsg                         compositor state\n\
             features\n  missing  satty                        annotation · install satty\n  \
             unknown  tailscale operator           vpn · no answer\n\
             1 missing (0 required)\n"
        );
    }

    #[test]
    fn a_missing_requirement_fails() {
        let findings = [Finding::new(
            Status::Missing,
            true,
            "df",
            "disk use",
            "install coreutils",
        )];

        let (text, healthy) = report(&findings);

        assert!(!healthy);
        assert!(text.ends_with("1 missing (1 required)\n"));
    }

    #[test]
    fn nothing_missing_says_so() {
        let findings = [Finding::new(Status::Ok, true, "df", "disk use", "")];

        assert_eq!(
            report(&findings),
            (
                String::from(
                    "required\n  ok       df                           disk use\nfeatures\nnothing missing\n"
                ),
                true
            )
        );
    }
}
