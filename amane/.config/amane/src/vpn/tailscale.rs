/*
 * tailscale: what `tailscale status` says, the exit node it suggests, and the
 * commands that connect, disconnect, pick an exit node or open the login page
 *
 * the status is asked for without its peer list (--peers=false): with mullvad's
 * exit nodes in the tailnet that list runs to hundreds of entries and close to a
 * megabyte, read every few seconds; without it the answer is a few kilobytes
 *
 * the commands need `sudo tailscale set --operator=$USER` once, otherwise
 * tailscaled refuses them from a normal user
 */

use std::process::{Command, Output};
use std::time::Duration;

use crate::command::{self, Failure};
use crate::json::{self, Value};

// status, whois and suggest answer in well under a second while tailscaled is fine
const QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/*
 * `up` waits until tailscaled is running; its own --timeout gives up first, and
 * COMMAND_TIMEOUT is the backstop for any command that hangs
 */
const UP_TIMEOUT: &str = "--timeout=15s";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    Stopped,
    NeedsLogin,
    Starting,
    Running,

    // anything newer tailscale versions might add
    Other(String),
}

impl Backend {
    fn from_text(text: &str) -> Self {
        match text {
            "Stopped" => Backend::Stopped,
            "NeedsLogin" | "NeedsMachineAuth" => Backend::NeedsLogin,
            "Starting" | "NoState" => Backend::Starting,
            "Running" => Backend::Running,
            other => Backend::Other(String::from(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub backend: Backend,

    // this machine on the tailnet
    pub name: String,
    pub ip: String,

    // the tailnet IPv4 of the exit node carrying all traffic, if any
    pub exit_ip: Option<String>,

    // where to log in, while tailscale needs it
    pub auth_url: String,
}

impl Status {
    pub fn connected(&self) -> bool {
        self.backend == Backend::Running
    }

    // up or on its way up, so the button offers to disconnect
    pub fn active(&self) -> bool {
        matches!(self.backend, Backend::Running | Backend::Starting)
    }
}

pub fn parse(text: &str) -> Option<Status> {
    let status = json::parse(text)?;

    let me = status.get("Self");

    // also there while the exit node itself is offline
    let exit_ip = status
        .get("ExitNodeStatus")
        .get("TailscaleIPs")
        .list()
        .iter()
        .map(|ip| ip.text().trim_end_matches("/32"))
        .find(|ip| !ip.is_empty() && !ip.contains(':'))
        .map(String::from);

    Some(Status {
        backend: Backend::from_text(status.get("BackendState").text()),
        name: name(me),
        ip: ipv4(me),
        exit_ip,
        auth_url: String::from(status.get("AuthURL").text()),
    })
}

// the name the tailnet knows it by: the first part of its dns name, like the cli shows
fn name(node: &Value) -> String {
    let dns = node.get("DNSName").text();

    match dns.split('.').next() {
        Some(first) if !first.is_empty() => String::from(first),
        _ => String::from(node.get("HostName").text()),
    }
}

fn ipv4(node: &Value) -> String {
    node.get("TailscaleIPs")
        .list()
        .iter()
        .map(Value::text)
        .find(|ip| !ip.contains(':'))
        .map(String::from)
        .unwrap_or_default()
}

// nl-ams-wg-303.mullvad.ts.net reads as nl-ams-wg-303
pub fn short_name(name: &str) -> &str {
    name.split('.').next().unwrap_or(name)
}

// `tailscale status --json --peers=false`; None when tailscale isn't installed or tailscaled isn't up
pub fn status() -> Option<Status> {
    let output = query(&["status", "--json", "--peers=false"])?;

    parse(&String::from_utf8_lossy(&output.stdout))
}

// a read-only tailscale command; None when it can't start or doesn't answer in time
fn query(arguments: &[&str]) -> Option<Output> {
    command::capture(Command::new("tailscale").args(arguments), QUERY_TIMEOUT).ok()
}

// the machine name behind a tailnet address, from `tailscale whois`
pub fn whois(ip: &str) -> Option<String> {
    let output = query(&["whois", ip])?;

    machine_name(&String::from_utf8_lossy(&output.stdout))
}

/*
 * Machine:
 *   Name:           nl-ams-wg-303.mullvad.ts.net
 *   ID:             ntThvrGABq11CNTRL
 */
fn machine_name(answer: &str) -> Option<String> {
    let line = answer
        .lines()
        .find_map(|line| line.trim().strip_prefix("Name:"))?;

    let name = line.trim().trim_end_matches('.');

    (!name.is_empty()).then(|| String::from(name))
}

// the exit node tailscale thinks best right now, from `tailscale exit-node suggest`
pub fn suggest() -> Option<String> {
    let output = query(&["exit-node", "suggest"])?;

    suggestion(&String::from_utf8_lossy(&output.stdout))
}

// Suggested exit node: nl-ams-wg-303.mullvad.ts.net.
fn suggestion(answer: &str) -> Option<String> {
    let line = answer
        .lines()
        .find_map(|line| line.trim().strip_prefix("Suggested exit node:"))?;

    let name = line.trim().trim_end_matches('.');

    (!name.is_empty()).then(|| String::from(name))
}

/*
 * only while logged in: when tailscale needs a login, `up` waits for it in the
 * browser, so the panel opens the login page instead (Vpn::toggle_tailscale)
 */
pub fn up() -> Result<(), String> {
    run(&["up", UP_TIMEOUT])
}

pub fn down() -> Result<(), String> {
    run(&["down"])
}

// a name or IP sends all traffic through that exit node, None stops that
pub fn set_exit_node(node: Option<&str>) -> Result<(), String> {
    let setting = format!("--exit-node={}", node.unwrap_or(""));

    run(&["set", &setting])
}

// tailscale's login page, in the browser
pub fn log_in(url: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(url)
        .status()
        .map_err(|error| format!("couldn't open the login page: {error}"))
        .map(|_| ())
}

fn run(arguments: &[&str]) -> Result<(), String> {
    let output = command::capture(Command::new("tailscale").args(arguments), COMMAND_TIMEOUT)
        .map_err(|failure| match failure {
            Failure::Start => String::from("Tailscale isn't installed"),
            Failure::Timeout => String::from("Tailscale didn't answer"),
        })?;

    if output.status.success() {
        return Ok(());
    }

    Err(short_error(&String::from_utf8_lossy(&output.stderr)))
}

// tailscale's errors are long; the ones that happen most get a short version
fn short_error(error: &str) -> String {
    let lowered = error.to_lowercase();

    if lowered.contains("access denied") || lowered.contains("permission denied") {
        String::from("Not allowed: run `sudo tailscale set --operator=$USER` once")
    } else if lowered.contains("tailscaled") && lowered.contains("connect") {
        String::from("Tailscale isn't running")
    } else {
        let first = error
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("failed");

        String::from(first.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // the parts the shell reads, from a `tailscale status --json --peers=false`
    const STATUS: &str = r#"{
      "Version": "1.104.4",
      "BackendState": "Running",
      "AuthURL": "",
      "Self": {
        "HostName": "prometheus",
        "DNSName": "prometheus.welara-turtle.ts.net.",
        "TailscaleIPs": ["100.67.176.75", "fd7a:115c:a1e0::ad35:b04c"],
        "Online": true
      },
      "ExitNodeStatus": {
        "ID": "ntThvrGABq11CNTRL",
        "Online": true,
        "TailscaleIPs": ["100.98.3.122/32", "fd7a:115c:a1e0::3801:3dd/128"]
      }
    }"#;

    #[test]
    fn reads_this_machine_and_the_exit_node() {
        let status = parse(STATUS).expect("a status");

        assert!(status.connected());
        assert!(status.active());
        assert_eq!(status.name, "prometheus");
        assert_eq!(status.ip, "100.67.176.75");
        assert_eq!(status.exit_ip.as_deref(), Some("100.98.3.122"));
    }

    #[test]
    fn no_exit_node_stopped_and_login() {
        let without: String =
            STATUS[..STATUS.find(",\n      \"ExitNodeStatus\"").unwrap()].to_string() + "}";

        let status = parse(&without).expect("a status");

        assert_eq!(status.exit_ip, None);

        let stopped = parse(&STATUS.replace("\"Running\"", "\"Stopped\"")).expect("a status");

        assert_eq!(stopped.backend, Backend::Stopped);
        assert!(!stopped.connected());
        assert!(!stopped.active());

        let login = r#"{"BackendState": "NeedsLogin", "AuthURL": "https://login.tailscale.com/a/x", "Self": {}}"#;

        let status = parse(login).expect("a status");

        assert_eq!(status.backend, Backend::NeedsLogin);
        assert_eq!(status.auth_url, "https://login.tailscale.com/a/x");
        assert_eq!(status.name, "");
    }

    #[test]
    fn suggestions_and_whois_names() {
        assert_eq!(
            suggestion(
                "Suggested exit node: nl-ams-wg-303.mullvad.ts.net.\nTo accept this suggestion, use `tailscale set --exit-node=nl-ams-wg-303.mullvad.ts.net.`.\n"
            )
            .as_deref(),
            Some("nl-ams-wg-303.mullvad.ts.net")
        );
        assert_eq!(suggestion("No exit node suggestion is available."), None);
        assert_eq!(suggestion(""), None);

        assert_eq!(
            machine_name("Machine:\n  Name:           nl-ams-wg-303.mullvad.ts.net\n  ID:             ntThvr\n")
                .as_deref(),
            Some("nl-ams-wg-303.mullvad.ts.net")
        );
        assert_eq!(machine_name("no such peer"), None);

        assert_eq!(short_name("nl-ams-wg-303.mullvad.ts.net"), "nl-ams-wg-303");
        assert_eq!(short_name("nas"), "nas");
    }

    #[test]
    fn errors_get_short() {
        assert_eq!(
            short_error("Access denied: checkprefs access denied\nUse 'sudo tailscale up' ..."),
            "Not allowed: run `sudo tailscale set --operator=$USER` once"
        );
        assert_eq!(
            short_error("failed to connect to local tailscaled; it doesn't appear to be running"),
            "Tailscale isn't running"
        );
        assert_eq!(short_error("\nsomething else\nmore"), "something else");
    }
}
