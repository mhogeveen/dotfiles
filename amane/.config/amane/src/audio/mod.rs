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

pub mod privacy;

use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use amane::Service;

use crate::command;
use crate::panels::{AUDIO, Panels};

// how many one-second ticks between reads while the panel is open
const POLL_EVERY: u32 = 2;

// wpctl answers at once; past this the sound server is stuck, and the panel shows what it had
const WPCTL_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Speaker,
    Headphones,
    Display,
    Bluetooth,
    Microphone,
}

// nerd font material design speaker and headphones, shared with the bluetooth list
pub const ICON_SPEAKER: &str = "\u{f04c3}";
pub const ICON_HEADPHONES: &str = "\u{f02cb}";

impl Kind {
    // nerd font material design glyphs
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Speaker => ICON_SPEAKER,
            Kind::Headphones | Kind::Bluetooth => ICON_HEADPHONES,
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
}

impl Service for Devices {
    fn new() -> Self {
        Self {
            outputs: Vec::new(),
            inputs: Vec::new(),
            state: State::Loading,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    // was_open is kept here, not in the service, so opening the panel redraws nothing by itself
    fn listen() {
        let mut ticks: u32 = 0;
        let mut was_open = false;

        loop {
            thread::sleep(Self::interval());

            ticks = ticks.wrapping_add(1);

            let open = Panels::read().is_open(AUDIO);

            if open && (!was_open || ticks % POLL_EVERY == 0) {
                refresh();
            }

            was_open = open;
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
            let _ = command::capture(
                Command::new("wpctl").args(["set-default", &id.to_string()]),
                WPCTL_TIMEOUT,
            );

            refresh();
        });
    }
}

// asks wpctl, waits on it, so only from a thread
fn refresh() {
    let read = command::read(Command::new("wpctl").arg("status"), WPCTL_TIMEOUT)
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

/*
 * what wpctl inspect said about each device, asked once per device. pipewire
 * hands out ids again after a device goes, so the key is the id with the name
 * wpctl status lists: a new device under an old id is asked about anew
 */
static NAMES: Mutex<Option<HashMap<(u32, String), (String, Kind)>>> = Mutex::new(None);

fn describe(listed: Vec<Listed>, fallback: Kind) -> Vec<Device> {
    listed
        .into_iter()
        .map(|listed| {
            let (label, kind) = {
                let mut names = NAMES.lock().unwrap_or_else(|error| error.into_inner());

                names
                    .get_or_insert_with(HashMap::new)
                    .entry((listed.id, listed.name.clone()))
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
    let output = command::read(
        Command::new("wpctl").args(["inspect", &id.to_string()]),
        WPCTL_TIMEOUT,
    )
    .unwrap_or_default();

    describe_inspected(&output, name, fallback)
}

// the name and kind from what `wpctl inspect` printed, see inspect
fn describe_inspected(output: &str, name: &str, fallback: Kind) -> (String, Kind) {
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

    // trimmed from `wpctl inspect` on this machine
    const HDMI: &str = "id 53, type PipeWire:Interface:Node
    api.alsa.path = \"hdmi:0,1\"
  * node.description = \"Built-in Audio Digital Stereo (HDMI)\"
    node.nick = \"U27B3AF\"
  * node.name = \"alsa_output.pci-0000_00_1f.3.hdmi-stereo\"";

    const HEADSET: &str = "id 61, type PipeWire:Interface:Node
  * node.description = \"WH-1000XM4\"
  * node.name = \"bluez_output.88_C9_E8.1\"";

    const HEADPHONES: &str = "id 54, type PipeWire:Interface:Node
    device.form-factor = \"headphone\"
  * node.description = \"USB Headphones\"
  * node.name = \"alsa_output.usb\"";

    #[test]
    fn a_monitor_goes_by_its_name_and_connector() {
        assert_eq!(
            describe_inspected(HDMI, "fallback", Kind::Speaker),
            (String::from("U27B3AF (HDMI)"), Kind::Display)
        );
    }

    #[test]
    fn bluetooth_and_headphones_are_told_apart() {
        assert_eq!(
            describe_inspected(HEADSET, "fallback", Kind::Speaker),
            (String::from("WH-1000XM4"), Kind::Bluetooth)
        );
        assert_eq!(
            describe_inspected(HEADPHONES, "fallback", Kind::Speaker).1,
            Kind::Headphones
        );
    }

    #[test]
    fn nothing_inspected_keeps_the_listed_name() {
        assert_eq!(
            describe_inspected("", "Built-in Audio", Kind::Speaker),
            (String::from("Built-in Audio"), Kind::Speaker)
        );
        assert_eq!(
            describe_inspected(HEADSET, "fallback", Kind::Microphone).1,
            Kind::Microphone
        );
    }

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
