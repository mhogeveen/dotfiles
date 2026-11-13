/*
 * whether something is recording from a microphone or sharing the screen, for
 * the red dot in the bar
 *
 * pipewire knows both: a recording app is a running Stream/Input/Audio node, a
 * screen share is the Video/Source node xdg-desktop-portal-wlr makes while a
 * cast is open. polling `pw-dump` costs about 1% cpu all the time, so a
 * long-running `pw-mon` is the signal instead: it prints its own dump once,
 * then only what changes. each settled burst of changes to nodes is answered
 * with one `pw-dump Node` (see NodeEvents for what counts)
 *
 * the same dump lists the apps playing sound, handed to streams.rs for the
 * audio panel's per-app volume
 *
 * when pw-mon exits (pipewire restarted, or it isn't installed) the dot goes
 * away and pw-mon is started again after a pause
 */

use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use amane::Service;

use super::streams;
use crate::command;
use crate::json::{self, Value};

// how long pw-mon stays quiet before a change counts as finished and pipewire is read
const SETTLE: Duration = Duration::from_millis(300);

// between pw-mon exiting and starting it again, so a missing pw-mon doesn't spin
const RESTART_DELAY: Duration = Duration::from_secs(2);

// pw-dump answers in about 30 ms; past this pipewire is stuck and the dot stays as it was
const PW_DUMP_TIMEOUT: Duration = Duration::from_secs(3);

// xdg-desktop-portal-wlr names its screen cast nodes xdpw_stream
const SCREEN_CAST_PREFIX: &str = "xdpw";

pub struct Privacy {
    microphone: bool,
    screen: bool,
}

impl Service for Privacy {
    fn new() -> Self {
        Self {
            microphone: false,
            screen: false,
        }
    }

    fn listen() {
        loop {
            if let Some(child) = start_monitor() {
                follow(child);
            }

            // nothing watches pipewire now, so nothing can be claimed about it
            store(Detected::default());

            thread::sleep(RESTART_DELAY);
        }
    }
}

// reading, safe inside views
impl Privacy {
    // the microphone or the screen is in use
    pub fn active(&self) -> bool {
        self.microphone || self.screen
    }
}

fn start_monitor() -> Option<Child> {
    Command::new("pw-mon")
        .arg("-N")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

/*
 * reads pipewire after every settled burst of pw-mon output, until pw-mon
 * exits. its lines arrive through a channel from their own thread, so waiting
 * for the burst to settle can time out
 */
fn follow(mut child: Child) {
    let Some(stdout) = child.stdout.take() else {
        stop(child);

        return;
    };

    let (sender, receiver) = mpsc::channel::<()>();

    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = Vec::new();
        let mut nodes = NodeEvents::default();

        // read as bytes: a property that isn't utf-8 mustn't end the watch
        while reader
            .read_until(b'\n', &mut line)
            .is_ok_and(|read| read > 0)
        {
            if nodes.line(&String::from_utf8_lossy(&line)) && sender.send(()).is_err() {
                break;
            }

            line.clear();
        }
    });

    // the nodes in the dump pw-mon starts with are changes too, so the first read follows it
    let mut changed = false;

    loop {
        match receiver.recv_timeout(SETTLE) {
            Ok(()) => changed = true,
            Err(RecvTimeoutError::Timeout) => {
                if changed {
                    refresh();

                    changed = false;
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    stop(child);
}

/*
 * picks the events about nodes out of pw-mon's output. everything else is
 * ignored, above all clients: every pw-dump joins pipewire as a client, so
 * reacting to clients would read pipewire again and again. an event is a line
 * like `added:`, `changed:` or `removed:` at the left edge, then a tab-indented
 * `id: 75` and, except on `removed:`, `type: PipeWire:Interface:Node (version 3)`
 * a few lines on. a removal only names the id, so the ids of nodes seen are kept
 */
#[derive(Default)]
struct NodeEvents {
    removing: bool,
    id: Option<u32>,
    nodes: HashSet<u32>,
}

impl NodeEvents {
    // takes one line of pw-mon's output; true when it completes an event about a node
    fn line(&mut self, line: &str) -> bool {
        if !line.starts_with([' ', '\t']) {
            self.removing = line.trim_end() == "removed:";
            self.id = None;

            return false;
        }

        let line = line.trim();

        if let Some(id) = line.strip_prefix("id:") {
            self.id = id.trim().parse().ok();

            return self.removing && self.id.is_some_and(|id| self.nodes.remove(&id));
        }

        if let Some(kind) = line.strip_prefix("type:")
            && kind.trim_start().starts_with("PipeWire:Interface:Node")
            && let Some(id) = self.id.take()
        {
            self.nodes.insert(id);

            return true;
        }

        false
    }
}

// pw-mon may still run when its output broke off; never leave it or a zombie behind
fn stop(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

// asks pipewire, waits on it, so only from listen's thread
fn refresh() {
    let read = command::read(Command::new("pw-dump").arg("Node"), PW_DUMP_TIMEOUT)
        .and_then(|dump| json::parse(&dump));

    // a failed read keeps what was shown, rather than hiding a recording
    if let Some(dump) = read {
        store(detect(&dump));
        streams::store(streams::from_dump(&dump));
    }
}

// writes only on a change, so the bar doesn't redraw for every pipewire event
fn store(detected: Detected) {
    let changed = {
        let privacy = Privacy::read();

        privacy.microphone != detected.microphone || privacy.screen != detected.screen
    };

    if changed {
        let mut privacy = Privacy::write();

        privacy.microphone = detected.microphone;
        privacy.screen = detected.screen;
    }
}

#[derive(Debug, Default, PartialEq)]
struct Detected {
    microphone: bool,
    screen: bool,
}

/*
 * what `pw-dump Node` says is in use: a list of nodes, each with info.state
 * and info.props. streams recording a speaker's output (visualisers, peak
 * meters, desktop audio) aren't a microphone, and an idle or suspended
 * recording stream isn't recording
 */
fn detect(dump: &Value) -> Detected {
    let mut detected = Detected::default();

    for node in dump.list() {
        let info = node.get("info");
        let props = info.get("props");
        let property = |key: &str| props.get(key);

        // pipewire writes these flags as booleans, but a "true" string is accepted too
        let flag = |key: &str| property(key).bool() || property(key).text() == "true";

        match property("media.class").text() {
            "Stream/Input/Audio" => {
                let running = info.get("state").text() == "running";
                let monitors_output = flag("stream.capture.sink") || flag("stream.monitor");

                if running && !monitors_output {
                    detected.microphone = true;
                }
            }
            "Video/Source" => {
                if property("node.name").text().starts_with(SCREEN_CAST_PREFIX) {
                    detected.screen = true;
                }
            }
            _ => {}
        }
    }

    detected
}

#[cfg(test)]
mod tests {
    use super::*;

    // one pw-dump node, trimmed to what detect reads
    fn node(class: &str, state: &str, extra: &str) -> String {
        named(class, state, "node", extra)
    }

    fn named(class: &str, state: &str, name: &str, extra: &str) -> String {
        format!(
            r#"{{"id": 1, "type": "PipeWire:Interface:Node", "info": {{"state": "{state}",
            "props": {{"media.class": "{class}", "node.name": "{name}"{extra}}}}}}}"#
        )
    }

    fn detect_in(nodes: &[String]) -> Detected {
        let dump = json::parse(&format!("[{}]", nodes.join(","))).expect("valid json");

        detect(&dump)
    }

    #[test]
    fn a_running_recording_is_the_microphone() {
        let detected = detect_in(&[node("Stream/Input/Audio", "running", "")]);

        assert!(detected.microphone);
        assert!(!detected.screen);
    }

    #[test]
    fn an_idle_recording_stream_is_not() {
        for state in ["idle", "suspended"] {
            assert!(!detect_in(&[node("Stream/Input/Audio", state, "")]).microphone);
        }
    }

    #[test]
    fn recording_a_speakers_output_is_not_the_microphone() {
        let sink = node(
            "Stream/Input/Audio",
            "running",
            r#", "stream.capture.sink": true"#,
        );
        let monitor = node(
            "Stream/Input/Audio",
            "running",
            r#", "stream.monitor": "true""#,
        );

        assert!(!detect_in(&[sink, monitor]).microphone);
    }

    #[test]
    fn playback_is_not_the_microphone() {
        assert!(!detect_in(&[node("Stream/Output/Audio", "running", "")]).microphone);
        assert!(!detect_in(&[node("Audio/Source", "running", "")]).microphone);
    }

    #[test]
    fn a_portal_cast_is_screen_sharing_but_a_camera_is_not() {
        let cast = named("Video/Source", "running", "xdpw_stream_1", "");

        assert!(detect_in(&[cast]).screen);
        assert!(!detect_in(&[node("Video/Source", "running", "")]).screen);
    }

    // the lines pw-mon prints, trimmed to what NodeEvents reads
    fn events(lines: &str) -> Vec<bool> {
        let mut events = NodeEvents::default();

        lines.lines().map(|line| events.line(line)).collect()
    }

    #[test]
    fn node_events_count_and_client_events_do_not() {
        let output = "added:\n\tid: 75\n\tpermissions: r-xm\n\ttype: PipeWire:Interface:Client (version 3)\n\
                      removed:\n\tid: 75\n\
                      added:\n\tid: 80\n\ttype: PipeWire:Interface:Node (version 3)\n\
                      changed:\n\tid: 80\n\ttype: PipeWire:Interface:Node (version 3)\n\
                      removed:\n\tid: 80";

        let counted = events(output).iter().filter(|counted| **counted).count();

        // the node's addition, change and removal; nothing for the client
        assert_eq!(counted, 3);
    }

    #[test]
    fn a_nested_type_line_is_not_another_event() {
        let output = "added:\n\tid: 80\n\ttype: PipeWire:Interface:Node (version 3)\n\
                      \t\ttype: PipeWire:Interface:Node";

        assert_eq!(events(output), vec![false, false, true, false]);
    }

    #[test]
    fn nothing_listed_is_nothing_in_use() {
        assert_eq!(detect_in(&[]), Detected::default());
        assert_eq!(detect(&Value::Null), Detected::default());
    }
}
