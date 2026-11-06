/*
 * the output and input devices, for the audio panel to switch between; amane's
 * Audio only knows the default ones, so these come from wpctl
 *
 * `wpctl status` is written for people, not programs: if a WirePlumber update
 * changes its layout, the panel says it couldn't read the devices, and the
 * volume sliders (amane's Audio) keep working
 *
 * polled only while the panel is open: when it opens, every 2 seconds after
 * that, and right after switching
 */

use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use amane::Service;

use crate::panels::{AUDIO, Panels};

// how many one-second ticks between reads while the panel is open
const POLL_EVERY: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Speaker,
    Headphones,
    Display,
    Bluetooth,
    Microphone,
}

impl Kind {
    // nerd font material design glyphs
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Speaker => "\u{f04c3}",
            Kind::Headphones | Kind::Bluetooth => "\u{f02cb}",
            Kind::Display => "\u{f0379}",
            Kind::Microphone => "\u{f036c}",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    // wpctl's id, which set-default takes
    pub id: u32,
    pub label: String,
    pub kind: Kind,
    pub default: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Loading,
    Ready,
    Failed,
}

pub struct Devices {
    outputs: Vec<Device>,
    inputs: Vec<Device>,
    state: State,
    was_open: bool,
}

impl Service for Devices {
    fn new() -> Self {
        Self {
            outputs: Vec::new(),
            inputs: Vec::new(),
            state: State::Loading,
            was_open: false,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn listen() {
        let mut ticks: u32 = 0;

        loop {
            thread::sleep(Self::interval());

            ticks = ticks.wrapping_add(1);

            let open = Panels::read().is_open(AUDIO);
            let was_open = Devices::read().was_open;

            if open && (!was_open || ticks % POLL_EVERY == 0) {
                refresh();
            }

            if open != was_open {
                Devices::write().was_open = open;
            }
        }
    }
}

// reading, safe inside views
impl Devices {
    pub fn outputs(&self) -> &[Device] {
        &self.outputs
    }

    pub fn inputs(&self) -> &[Device] {
        &self.inputs
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}

impl Devices {
    /*
     * makes this the default output or input; marked right away so the click
     * shows, then read back. amane's Audio follows the new default by itself
     */
    pub fn set_default(id: u32) {
        {
            let mut devices = Devices::write();

            let Devices {
                outputs, inputs, ..
            } = &mut *devices;

            for list in [outputs, inputs] {
                if list.iter().any(|device| device.id == id) {
                    for device in list.iter_mut() {
                        device.default = device.id == id;
                    }
                }
            }
        }

        thread::spawn(move || {
            let _ = Command::new("wpctl")
                .args(["set-default", &id.to_string()])
                .status();

            refresh();
        });
    }
}

// asks wpctl, waits on it, so only from a thread
fn refresh() {
    let read = Command::new("wpctl")
        .arg("status")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .and_then(|status| parse(&status));

    let (state, outputs, inputs) = match read {
        Some((outputs, inputs)) => (
            State::Ready,
            describe(outputs, Kind::Speaker),
            describe(inputs, Kind::Microphone),
        ),
        None => (State::Failed, Vec::new(), Vec::new()),
    };

    let changed = {
        let devices = Devices::read();

        devices.state != state || devices.outputs != outputs || devices.inputs != inputs
    };

    if changed {
        let mut devices = Devices::write();

        devices.state = state;
        devices.outputs = outputs;
        devices.inputs = inputs;
    }
}

// a line of `wpctl status`, before wpctl inspect gave it a better name
#[derive(Debug, PartialEq)]
struct Listed {
    id: u32,
    name: String,
    default: bool,
}

/*
 * the Sinks and Sources blocks of the Audio section:
 *
 *   Audio
 *    ├─ Sinks:
 *    │      53. Built-in Audio Digital Stereo (HDMI) [vol: 0.40]
 *    │  *   54. Built-in Audio Analog Stereo        [vol: 0.40]
 *
 * None when there's no Audio section at all, which means the layout changed
 */
fn parse(status: &str) -> Option<(Vec<Listed>, Vec<Listed>)> {
    enum Block {
        Sinks,
        Sources,
        Other,
    }

    let mut in_audio = false;
    let mut block = Block::Other;
    let mut sinks = Vec::new();
    let mut sources = Vec::new();

    for line in status.lines() {
        if line.trim() == "Audio" {
            in_audio = true;

            continue;
        }

        if !in_audio {
            continue;
        }

        // the next section, like Video, starts at the left edge
        if !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }

        let content = line
            .trim_start_matches(|character: char| " │├└─".contains(character))
            .trim_end();

        if content.is_empty() {
            continue;
        }

        if content.ends_with(':') {
            block = match content {
                "Sinks:" => Block::Sinks,
                "Sources:" => Block::Sources,
                _ => Block::Other,
            };

            continue;
        }

        let list = match block {
            Block::Sinks => &mut sinks,
            Block::Sources => &mut sources,
            Block::Other => continue,
        };

        let (default, rest) = match content.strip_prefix('*') {
            Some(rest) => (true, rest.trim_start()),
            None => (false, content),
        };

        let Some((id, rest)) = rest.split_once('.') else {
            continue;
        };

        let Ok(id) = id.trim().parse() else {
            continue;
        };

        let name = rest.split(" [vol:").next().unwrap_or(rest).trim();

        list.push(Listed {
            id,
            name: String::from(name),
            default,
        });
    }

    in_audio.then_some((sinks, sources))
}

// what wpctl inspect said about each id, asked once per id
static NAMES: Mutex<Option<HashMap<u32, (String, Kind)>>> = Mutex::new(None);

fn describe(listed: Vec<Listed>, fallback: Kind) -> Vec<Device> {
    listed
        .into_iter()
        .map(|listed| {
            let (label, kind) = {
                let mut names = NAMES.lock().unwrap_or_else(|error| error.into_inner());

                names
                    .get_or_insert_with(HashMap::new)
                    .entry(listed.id)
                    .or_insert_with(|| inspect(listed.id, &listed.name, fallback))
                    .clone()
            };

            Device {
                id: listed.id,
                label,
                kind,
                default: listed.default,
            }
        })
        .collect()
}

/*
 * a nicer name and the kind of device: a monitor's speakers go by the monitor's
 * name, bluetooth by the device's; falls back to wpctl status's name
 */
fn inspect(id: u32, name: &str, fallback: Kind) -> (String, Kind) {
    let output = Command::new("wpctl")
        .args(["inspect", &id.to_string()])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();

    let properties: HashMap<&str, &str> = output
        .lines()
        .filter_map(|line| {
            let line = line.trim_start().trim_start_matches('*').trim_start();
            let (key, value) = line.split_once(" = ")?;

            Some((key, value.trim_matches('"')))
        })
        .collect();

    let property = |key: &str| properties.get(key).copied().unwrap_or("");

    let description = match property("node.description") {
        "" => name,
        description => description,
    };

    if fallback == Kind::Microphone {
        return (String::from(description), Kind::Microphone);
    }

    let path = property("api.alsa.path");
    let nick = property("node.nick");

    if property("node.name").starts_with("bluez_") {
        return (String::from(description), Kind::Bluetooth);
    }

    let display = if path.starts_with("hdmi") || description.contains("HDMI") {
        Some("HDMI")
    } else if description.contains("DisplayPort") {
        Some("DisplayPort")
    } else {
        None
    };

    if let Some(connector) = display {
        let label = if nick.is_empty() || nick == description {
            String::from(description)
        } else {
            format!("{nick} ({connector})")
        };

        return (label, Kind::Display);
    }

    let form = property("device.form-factor");

    let headphones =
        matches!(form, "headphone" | "headset") || property("device.icon-name").contains("head");

    let kind = if headphones {
        Kind::Headphones
    } else {
        Kind::Speaker
    };

    (String::from(description), kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS: &str = "PipeWire 'pipewire-0' [1.6.6, maarten@prometheus, cookie:1]
 └─ Clients:
        36. WirePlumber                         [1.6.6, maarten@prometheus, pid:4521]

Audio
 ├─ Devices:
 │      45. Built-in Audio                      [alsa]
 │  
 ├─ Sinks:
 │      53. Built-in Audio Digital Stereo (HDMI) [vol: 0.40]
 │  *   54. Built-in Audio Analog Stereo        [vol: 0.40]
 │  
 ├─ Sources:
 │  *   55. Built-in Audio Analog Stereo        [vol: 1.00 MUTED]
 │  
 ├─ Filters:
 │  
 └─ Streams:

Video
 ├─ Sinks:
 │      70. Not audio                           [vol: 1.00]
";

    #[test]
    fn reads_sinks_and_sources_of_the_audio_section_only() {
        let (sinks, sources) = parse(STATUS).expect("an audio section");

        assert_eq!(
            sinks,
            vec![
                Listed {
                    id: 53,
                    name: String::from("Built-in Audio Digital Stereo (HDMI)"),
                    default: false,
                },
                Listed {
                    id: 54,
                    name: String::from("Built-in Audio Analog Stereo"),
                    default: true,
                },
            ]
        );

        assert_eq!(sources.len(), 1);
        assert!(sources[0].default);
        assert_eq!(sources[0].name, "Built-in Audio Analog Stereo");
    }

    #[test]
    fn no_audio_section_is_a_failure() {
        assert!(parse("something else entirely").is_none());
    }
}
