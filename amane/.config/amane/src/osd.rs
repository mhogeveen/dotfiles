/*
 * the volume popup at the bottom of the focused monitor: the speaker or the
 * microphone, a level bar and the percentage, for a moment after either changes,
 * whatever changed it (the volume keys run wpctl, an app, a headset becoming the
 * default). not for the first reading at startup, and not while the audio panel
 * is open, which shows the levels already
 *
 * the window is always mapped, never changes size and lets every click through;
 * it only draws while showing, like the notification popups (see popups.rs)
 *
 * also the `media` ipc for the play, next and previous keys
 */

use std::thread;
use std::time::{Duration, Instant};

use amane::{
    Align::{Center, Start},
    Audio, Color, Full, Keyboard, Layer, LayerWindow, Margin, Media, Monitor, Rectangle, Row,
    Service,
    Size::Parent,
    Text, Vertical, children,
};

use crate::panels::audio::{ICON_MIC, ICON_MIC_MUTED, volume_icon};
use crate::panels::ui;
use crate::panels::{self, AUDIO, Panels};
use crate::theme::{Theme, theme};

// how long it stays after the last change
const SHOWN_FOR: Duration = Duration::from_millis(1500);

// how often the levels are looked at; amane already keeps them up to date from pipewire
const LOOK_EVERY: Duration = Duration::from_millis(100);

const WIDTH: f32 = 300.0;
const HEIGHT: f32 = 48.0;
const BOTTOM: i32 = 32;

const ICON: f32 = 24.0;
const PERCENT: f32 = 56.0;
const GAP: f32 = 12.0;
const PADDING: f32 = 12.0;
const BAR: f32 = WIDTH - PADDING * 2.0 - ICON - PERCENT - GAP * 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Device {
    Output,
    Microphone,
}

// what the audio service says, the parts the popup shows
#[derive(Debug, Clone, Copy, PartialEq)]
struct Levels {
    volume: u8,
    muted: bool,
    microphone_volume: u8,
    microphone_muted: bool,
}

impl Levels {
    fn read() -> Self {
        let audio = Audio::read();

        Self {
            volume: audio.volume(),
            muted: audio.muted(),
            microphone_volume: audio.microphone_volume(),
            microphone_muted: audio.microphone_muted(),
        }
    }
}

// which one changed between two readings, the output first when both did
fn changed(before: Levels, now: Levels) -> Option<Device> {
    if (before.volume, before.muted) != (now.volume, now.muted) {
        Some(Device::Output)
    } else if (before.microphone_volume, before.microphone_muted)
        != (now.microphone_volume, now.microphone_muted)
    {
        Some(Device::Microphone)
    } else {
        None
    }
}

pub struct Osd {
    // what the popup shows, and until when
    showing: Option<(Device, Instant)>,
}

impl Service for Osd {
    fn new() -> Self {
        Self { showing: None }
    }

    fn interval() -> Duration {
        LOOK_EVERY
    }

    fn listen() {
        // the first reading is what the volume already was, nothing changed yet
        let mut last = Levels::read();

        loop {
            thread::sleep(Self::interval());

            // one lock at a time: the levels, then the panels, then this service
            let now = Levels::read();
            let change = changed(last, now);

            last = now;

            let audio_panel = Panels::read().is_open(AUDIO);

            let showing = Osd::read().showing;

            match (change, showing) {
                (Some(device), _) if !audio_panel => {
                    Osd::write().showing = Some((device, Instant::now() + SHOWN_FOR));
                }

                // over, or the audio panel opened meanwhile
                (_, Some((_, until))) if Instant::now() >= until || audio_panel => {
                    Osd::write().showing = None;
                }

                _ => {}
            }
        }
    }
}

// starts watching from the start; a service only listens once something reads it
pub fn start() {
    drop(Osd::read());
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();

    let showing = Osd::read().showing.map(|(device, _)| device);

    let content = match showing {
        Some(device) if panels::is_focused(monitor) => popup(&t, device),
        _ => Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(Color::TRANSPARENT),
    };

    LayerWindow::new()
        .width(Full)
        .height(HEIGHT)
        .anchor_vertical(Vertical::Bottom)
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: BOTTOM,
            left: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::None)
        .namespace("amane-osd")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Center, Center)
                .child(content),
        )
        .click_through()
}

// [icon] ━━━━━━━━──── 45%
fn popup(t: &Theme, device: Device) -> Rectangle {
    let audio = Audio::read();

    let (icon, volume, muted) = match device {
        Device::Output => (
            volume_icon(audio.volume(), audio.muted()),
            audio.volume(),
            audio.muted(),
        ),
        Device::Microphone => {
            let icon = if audio.microphone_muted() {
                ICON_MIC_MUTED
            } else {
                ICON_MIC
            };

            (icon, audio.microphone_volume(), audio.microphone_muted())
        }
    };

    let (percent, color) = if muted {
        (String::from("Muted"), t.comment)
    } else {
        (format!("{volume}%"), t.fg)
    };

    Rectangle::new()
        .width(WIDTH)
        .height(HEIGHT)
        .fill(t.bg_deep)
        .border(1.0, t.bg3)
        .padding(PADDING)
        .child(
            Row::new(children![
                Rectangle::new()
                    .width(ICON)
                    .height(Parent)
                    .align_child(Center, Center)
                    .child(Text::new(icon).size(18.0).color(color)),
                ui::meter(t, BAR, volume, muted),
                Rectangle::new()
                    .width(PERCENT)
                    .height(Parent)
                    .align_child(Start, Center)
                    .child(Text::new(percent).size(13.0).color(color)),
            ])
            .gap(GAP)
            .align(Center)
            .height(Parent),
        )
}

// `amane ipc call media play-pause|next|previous`, for the media keys
pub fn media_ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str).unwrap_or_default();

    if !matches!(command, "play-pause" | "next" | "previous") {
        return String::from("usage: media play-pause | media next | media previous");
    }

    if Media::read().active().is_none() {
        return String::from("no player");
    }

    match command {
        "play-pause" => Media::play_pause(),
        "next" => Media::next(),
        _ => Media::previous(),
    }

    String::from(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUIET: Levels = Levels {
        volume: 40,
        muted: false,
        microphone_volume: 80,
        microphone_muted: false,
    };

    #[test]
    fn only_a_change_shows_and_says_what() {
        assert_eq!(changed(QUIET, QUIET), None);

        let louder = Levels {
            volume: 45,
            ..QUIET
        };
        let muted = Levels {
            muted: true,
            ..QUIET
        };
        let mic_muted = Levels {
            microphone_muted: true,
            ..QUIET
        };
        let both = Levels {
            volume: 45,
            microphone_volume: 50,
            ..QUIET
        };

        assert_eq!(changed(QUIET, louder), Some(Device::Output));
        assert_eq!(changed(QUIET, muted), Some(Device::Output));
        assert_eq!(changed(QUIET, mic_muted), Some(Device::Microphone));
        assert_eq!(changed(QUIET, both), Some(Device::Output));
    }
}
