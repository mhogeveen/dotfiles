/*
 * the apps playing sound, for the audio panel's per-app volume: every
 * Stream/Output/Audio node, its volume and mute
 *
 * nothing is polled here: privacy.rs already reads `pw-dump Node` after every
 * settled burst of pipewire node changes, and hands that dump to store. changes
 * go out through wpctl on one worker thread, which keeps only the newest volume
 * per stream, so dragging a slider doesn't start a process for every step
 *
 * the selection says where the audio panel's j, k and m go: a stream, or the
 * output device when nothing is selected
 */

use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use amane::Service;

use crate::command;
use crate::json::Value;

// wpctl answers at once; past this the sound server is stuck, and the next change tries again
const WPCTL_TIMEOUT: Duration = Duration::from_secs(3);

// nerd font material design glyphs, picked by the program's binary name
const ICON_FIREFOX: &str = "\u{f0239}";
const ICON_CHROME: &str = "\u{f02af}";
const ICON_SPOTIFY: &str = "\u{f04c7}";
const ICON_DISCORD: &str = "\u{f066f}";
const ICON_VLC: &str = "\u{f057c}";
const ICON_MUSIC: &str = "\u{f075a}";

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    // pipewire's node id, which wpctl takes
    pub id: u32,
    pub app: String,

    // what it plays, like a tab's title; empty when it says nothing more than the app
    pub media: String,
    pub icon: &'static str,
    pub volume: u8,
    pub muted: bool,

    // running; a paused or corked stream is shown dimmed
    pub playing: bool,
}

pub struct Streams {
    streams: Vec<Stream>,
    selected: Option<u32>,
}

impl Service for Streams {
    fn new() -> Self {
        Self {
            streams: Vec::new(),
            selected: None,
        }
    }

    // privacy.rs's watcher writes the list, input writes the rest
    fn listen() {}
}

// reading, safe inside views
impl Streams {
    pub fn streams(&self) -> &[Stream] {
        &self.streams
    }

    // the selected stream, None for the output device
    pub fn selected(&self) -> Option<&Stream> {
        let id = self.selected?;

        self.streams.iter().find(|stream| stream.id == id)
    }
}

// for input handlers and the watcher thread, never inside a view
impl Streams {
    // the next stream, and after the last one the output device again
    pub fn select_next() {
        let mut streams = Streams::write();

        streams.selected = next_selection(&streams.streams, streams.selected);
    }

    // back to the output device, for when the panel closes
    pub fn select_output() {
        if Streams::read().selected.is_some() {
            Streams::write().selected = None;
        }
    }

    pub fn set_volume(id: u32, volume: u8) {
        let volume = volume.min(100);

        if change_stored(id, |stream| stream.volume = volume) {
            send(id, Change::Volume(volume));
        }
    }

    pub fn toggle_mute(id: u32) {
        let mut muted = false;

        if change_stored(id, |stream| {
            stream.muted = !stream.muted;
            muted = stream.muted;
        }) {
            send(id, Change::Mute(muted));
        }
    }
}

// marks the change right away so the slider follows; false when the stream is gone
fn change_stored(id: u32, change: impl FnOnce(&mut Stream)) -> bool {
    let mut streams = Streams::write();

    match streams.streams.iter_mut().find(|stream| stream.id == id) {
        Some(stream) => {
            change(stream);

            true
        }
        None => false,
    }
}

fn next_selection(streams: &[Stream], selected: Option<u32>) -> Option<u32> {
    let position = selected.and_then(|id| streams.iter().position(|stream| stream.id == id));

    match position {
        // a stream that went away counts as the output, so the first stream is next
        None => streams.first().map(|stream| stream.id),
        Some(index) => streams.get(index + 1).map(|stream| stream.id),
    }
}

// from privacy.rs's watcher; writes only on a change, so the panel doesn't redraw for every pipewire event
pub fn store(streams: Vec<Stream>) {
    let changed = Streams::read().streams != streams;

    if changed {
        let mut stored = Streams::write();

        if let Some(id) = stored.selected
            && !streams.iter().any(|stream| stream.id == id)
        {
            stored.selected = None;
        }

        stored.streams = streams;
    }
}

/*
 * the playback streams in a `pw-dump Node` answer: a list of nodes with id,
 * info.state, info.props and info.params.Props, where channelVolumes is linear
 * and wpctl's percent its cube root (0.125 is 50%)
 */
pub fn from_dump(dump: &Value) -> Vec<Stream> {
    let mut streams: Vec<Stream> = dump
        .list()
        .iter()
        .filter_map(|node| {
            let info = node.get("info");
            let props = info.get("props");
            let property = |key: &str| props.get(key).text();

            if property("media.class") != "Stream/Output/Audio" {
                return None;
            }

            let id = node.get("id").number()? as u32;

            let app = [
                property("application.name"),
                property("node.description"),
                property("node.name"),
            ]
            .into_iter()
            .find(|name| !name.is_empty())
            .unwrap_or("Unknown");

            let media = match property("media.name") {
                media if media == app => "",
                media => media,
            };

            // pipewire can list Props more than once; the one with channelVolumes is the stream's
            let volume_props = info
                .get("params")
                .get("Props")
                .list()
                .iter()
                .find(|props| !props.get("channelVolumes").list().is_empty());

            let (volume, muted) = match volume_props {
                Some(props) => (
                    percent(props.get("channelVolumes").list()),
                    props.get("mute").bool(),
                ),
                None => (100, false),
            };

            Some(Stream {
                id,
                app: String::from(app),
                media: String::from(media),
                icon: icon(property("application.process.binary")),
                volume,
                muted,
                playing: info.get("state").text() == "running",
            })
        })
        .collect();

    streams.sort_by_key(|stream| stream.id);

    streams
}

// the channels' mean, as wpctl shows it
fn percent(channels: &[Value]) -> u8 {
    let volumes: Vec<f64> = channels.iter().filter_map(Value::number).collect();

    if volumes.is_empty() {
        return 100;
    }

    let mean = volumes.iter().sum::<f64>() / volumes.len() as f64;

    (mean.max(0.0).cbrt() * 100.0).round().clamp(0.0, 100.0) as u8
}

fn icon(binary: &str) -> &'static str {
    let binary = binary.to_ascii_lowercase();

    if binary.contains("firefox") || binary.contains("librewolf") {
        ICON_FIREFOX
    } else if binary.contains("chrom") {
        ICON_CHROME
    } else if binary.contains("spotify") {
        ICON_SPOTIFY
    } else if binary.contains("discord") || binary.contains("vesktop") {
        ICON_DISCORD
    } else if binary.contains("vlc") {
        ICON_VLC
    } else {
        ICON_MUSIC
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Change {
    Volume(u8),
    Mute(bool),
}

// the worker's queue, started with the first change
static QUEUE: Mutex<Option<Sender<(u32, Change)>>> = Mutex::new(None);

fn send(id: u32, change: Change) {
    let mut queue = QUEUE.lock().unwrap_or_else(|error| error.into_inner());

    let sender = queue.get_or_insert_with(|| {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || work(receiver));

        sender
    });

    let _ = sender.send((id, change));
}

// waits for a change, takes whatever else queued up meanwhile, and sends only the newest of each
fn work(receiver: Receiver<(u32, Change)>) {
    while let Ok(first) = receiver.recv() {
        let mut pending = vec![first];

        pending.extend(receiver.try_iter());

        for (id, change) in newest(pending) {
            let id = id.to_string();

            let mut wpctl = Command::new("wpctl");

            match change {
                Change::Volume(volume) => wpctl.args([
                    "set-volume",
                    &id,
                    &format!("{:.2}", f32::from(volume) / 100.0),
                ]),
                Change::Mute(muted) => wpctl.args(["set-mute", &id, if muted { "1" } else { "0" }]),
            };

            let _ = command::capture(&mut wpctl, WPCTL_TIMEOUT);
        }
    }
}

// the newest volume and the newest mute of each stream, in the order they were last asked for
fn newest(pending: Vec<(u32, Change)>) -> Vec<(u32, Change)> {
    let kind = |change: &Change| matches!(change, Change::Volume(_));

    let mut last: HashMap<(u32, bool), usize> = HashMap::new();

    for (index, (id, change)) in pending.iter().enumerate() {
        last.insert((*id, kind(change)), index);
    }

    pending
        .into_iter()
        .enumerate()
        .filter(|(index, (id, change))| last.get(&(*id, kind(change))) == Some(index))
        .map(|(_, change)| change)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;

    // one pw-dump node, trimmed to what from_dump reads
    fn node(id: u32, class: &str, state: &str, props: &str, params: &str) -> String {
        format!(
            r#"{{"id": {id}, "type": "PipeWire:Interface:Node", "info": {{"state": "{state}",
            "props": {{"media.class": "{class}"{props}}}, "params": {{{params}}}}}}}"#
        )
    }

    fn volumes(channels: &str, mute: bool) -> String {
        format!(r#""Props": [{{"volume": 1.0, "mute": {mute}, "channelVolumes": [{channels}]}}]"#)
    }

    fn streams_in(nodes: &[String]) -> Vec<Stream> {
        let dump = json::parse(&format!("[{}]", nodes.join(","))).expect("valid json");

        from_dump(&dump)
    }

    fn stream(id: u32) -> Stream {
        Stream {
            id,
            app: String::from("app"),
            media: String::new(),
            icon: ICON_MUSIC,
            volume: 100,
            muted: false,
            playing: true,
        }
    }

    #[test]
    fn only_playback_streams_count() {
        let found = streams_in(&[
            node(80, "Stream/Output/Audio", "running", "", ""),
            node(81, "Stream/Input/Audio", "running", "", ""),
            node(37, "Audio/Sink", "running", "", ""),
        ]);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, 80);
    }

    #[test]
    fn the_volume_is_the_cube_root_like_wpctl() {
        let found = streams_in(&[node(
            80,
            "Stream/Output/Audio",
            "running",
            "",
            &volumes("0.125, 0.125", true),
        )]);

        assert_eq!(found[0].volume, 50);
        assert!(found[0].muted);
    }

    #[test]
    fn names_fall_back_and_repeat_nothing() {
        let named = node(
            80,
            "Stream/Output/Audio",
            "running",
            r#", "application.name": "Firefox", "media.name": "Firefox", "application.process.binary": "firefox""#,
            "",
        );
        let bare = node(
            81,
            "Stream/Output/Audio",
            "idle",
            r#", "node.name": "speech-dispatcher", "media.name": "playback""#,
            "",
        );

        let found = streams_in(&[bare, named]);

        assert_eq!(found[0].app, "Firefox");
        assert_eq!(found[0].media, "");
        assert_eq!(found[0].icon, ICON_FIREFOX);
        assert!(found[0].playing);

        assert_eq!(found[1].app, "speech-dispatcher");
        assert_eq!(found[1].media, "playback");
        assert_eq!(found[1].icon, ICON_MUSIC);
        assert!(!found[1].playing);
    }

    #[test]
    fn no_volume_props_is_full_and_unmuted() {
        let found = streams_in(&[node(80, "Stream/Output/Audio", "running", "", "")]);

        assert_eq!((found[0].volume, found[0].muted), (100, false));
    }

    #[test]
    fn selection_cycles_through_the_streams_and_back_to_the_output() {
        let streams = [stream(80), stream(90)];

        assert_eq!(next_selection(&streams, None), Some(80));
        assert_eq!(next_selection(&streams, Some(80)), Some(90));
        assert_eq!(next_selection(&streams, Some(90)), None);
        assert_eq!(next_selection(&streams, Some(70)), Some(80));
        assert_eq!(next_selection(&[], None), None);
    }

    #[test]
    fn only_the_newest_change_of_each_kind_goes_out() {
        let pending = vec![
            (80, Change::Volume(10)),
            (80, Change::Mute(true)),
            (90, Change::Volume(40)),
            (80, Change::Volume(20)),
            (80, Change::Mute(false)),
        ];

        assert_eq!(
            newest(pending),
            vec![
                (90, Change::Volume(40)),
                (80, Change::Volume(20)),
                (80, Change::Mute(false)),
            ]
        );
    }
}
