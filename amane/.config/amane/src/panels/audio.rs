/*
 * audio panel, top-right: volume and mute for the default output and
 * microphone, and the devices to pick the default from
 *
 * volume and mute are amane's Audio, live; the device lists come from wpctl,
 * read while the panel is open (see src/audio)
 */

use amane::{
    Align::{Center, Start},
    Audio, Column,
    Cursor::Pointer,
    Key, LayerWindow, Monitor, Rectangle, Row, Service,
    Size::Parent,
    Text, Weight, Widget, children,
};

use super::ui::{self, BUTTON, CORNER_WIDTH, PADDING, SIDES};
use super::{AUDIO, hints, panel};
use crate::audio::{Device, Devices, State};
use crate::theme::Theme;

const GAP: f32 = 8.0;
const ROW: f32 = 36.0;
const ICON: f32 = 20.0;
const PERCENT: f32 = 40.0;
const MUTE: f32 = 64.0;

// fits one device list of 4 and one of 2 without scrolling
const HEIGHT: f32 = 460.0;

// the slider takes what's left of the row
const SLIDER: f32 =
    CORNER_WIDTH - PADDING * 2.0 - ICON - PERCENT - MUTE - GAP * 3.0 - PADDING * 2.0;

// nerd font material design glyphs
const ICON_MUTED: &str = "\u{f075f}";
const ICON_LOW: &str = "\u{f057f}";
const ICON_MEDIUM: &str = "\u{f0580}";
const ICON_HIGH: &str = "\u{f057e}";
const ICON_MIC: &str = "\u{f036c}";
const ICON_MIC_MUTED: &str = "\u{f036d}";

// the speaker glyph for this volume, shared with the bar
pub fn volume_icon(volume: u8, muted: bool) -> &'static str {
    match (muted, volume) {
        (true, _) | (_, 0) => ICON_MUTED,
        (_, 1..=33) => ICON_LOW,
        (_, 34..=66) => ICON_MEDIUM,
        _ => ICON_HIGH,
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = ui::corner(HEIGHT, "amane-audio");

    panel(monitor, AUDIO, base, key_pressed, content)
}

const KEY_MUTE: char = 'm';
const KEY_MIC_MUTE: char = 'n';
const KEY_QUIETER: char = 'j';
const KEY_LOUDER: char = 'k';

// how far j and k move the volume
const STEP: i32 = 5;

fn key_pressed(key: Key) {
    let Key::Character(letter) = key else {
        return;
    };

    match letter.to_ascii_lowercase() {
        KEY_MUTE => Audio::toggle_mute(),
        KEY_MIC_MUTE => Audio::toggle_microphone_mute(),
        KEY_QUIETER => step(-STEP),
        KEY_LOUDER => step(STEP),
        _ => {}
    }
}

// the output volume, kept between 0 and 100
fn step(by: i32) {
    let volume = i32::from(Audio::read().volume());

    Audio::set_volume((volume + by).clamp(0, 100) as u8);
}

fn content(t: &Theme) -> Rectangle {
    let audio = Audio::read();
    let devices = Devices::read();

    let output = Level {
        icon: volume_icon(audio.volume(), audio.muted()),
        volume: audio.volume(),
        muted: audio.muted(),
        set_volume: Audio::set_volume,
        toggle_mute: Audio::toggle_mute,
        mute_key: KEY_MUTE,
        volume_keys: Some((KEY_QUIETER, KEY_LOUDER)),
    };

    let input = Level {
        icon: if audio.microphone_muted() {
            ICON_MIC_MUTED
        } else {
            ICON_MIC
        },
        volume: audio.microphone_volume(),
        muted: audio.microphone_muted(),
        set_volume: Audio::set_microphone_volume,
        toggle_mute: Audio::toggle_microphone_mute,
        mute_key: KEY_MIC_MUTE,
        volume_keys: None,
    };

    let parts = children![
        ui::title(t, "Audio"),
        section(t, "Output", &output, devices.outputs(), devices.state()),
        section(t, "Input", &input, devices.inputs(), devices.state()),
    ];

    ui::frame(t, GAP * 2.0, parts)
}

// one side of the panel: the output or the microphone
struct Level {
    icon: &'static str,
    volume: u8,
    muted: bool,
    set_volume: fn(u8),
    toggle_mute: fn(),

    // what ? shows: the mute key, and the volume keys where there are any
    mute_key: char,
    volume_keys: Option<(char, char)>,
}

fn section(t: &Theme, title: &str, level: &Level, devices: &[Device], state: &State) -> Column {
    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Text::new(title)
                .size(12.0)
                .weight(Weight::Bold)
                .color(t.comment),
        ),
        Box::new(level_row(t, level)),
    ];

    let mut list: Vec<Box<dyn Widget>> = Vec::new();

    match state {
        State::Loading => list.push(note(t, "Loading…")),
        State::Failed => list.push(note(t, "Couldn't read devices from wpctl")),
        State::Ready if devices.is_empty() => list.push(note(t, "No devices")),
        State::Ready => {
            // with nothing to pick between, the one device is just named
            let choosable = devices.len() > 1;

            for device in devices {
                list.push(Box::new(device_row(t, device, choosable)));
            }
        }
    }

    parts.push(Box::new(
        Rectangle::new()
            .width(Parent)
            .height(ROW * list.len() as f32)
            .fill(t.bg0)
            .border(1.0, t.bg3)
            .child(Column::new(list)),
    ));

    Column::new(parts).gap(GAP)
}

// icon, slider, percentage and mute
fn level_row(t: &Theme, level: &Level) -> Rectangle {
    let color = if level.muted { t.comment } else { t.fg };

    let toggle_mute = level.toggle_mute;

    let mute = ui::key_text_button(
        t,
        if level.muted { "Unmute" } else { "Mute" },
        MUTE,
        level.mute_key,
        move |_| toggle_mute(),
    );

    // while ? hints show, the percentage says which keys change the volume
    let percent = match (hints::shown(), level.volume_keys) {
        (true, Some((quieter, louder))) => Text::new(format!("{quieter} {louder}"))
            .size(13.0)
            .weight(Weight::Bold)
            .color(t.accent),
        _ => Text::new(format!("{}%", level.volume))
            .size(13.0)
            .color(color),
    };

    let row = Row::new(children![
        Rectangle::new()
            .width(ICON)
            .height(BUTTON)
            .align_child(Center, Center)
            .child(Text::new(level.icon).size(16.0).color(color)),
        ui::slider(t, SLIDER, level.volume, level.muted, level.set_volume),
        Rectangle::new()
            .width(PERCENT)
            .height(BUTTON)
            .align_child(Center, Center)
            .child(percent),
        mute,
    ])
    .gap(GAP)
    .align(Center)
    .height(BUTTON);

    Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .padding(SIDES)
        .child(row)
}

fn device_row(t: &Theme, device: &Device, choosable: bool) -> Rectangle {
    let (mark, color) = if device.default {
        ("●", t.fg)
    } else {
        ("○", t.fg_dim)
    };

    let mut parts: Vec<Box<dyn Widget>> = Vec::new();

    if choosable {
        parts.push(Box::new(
            Rectangle::new()
                .width(ICON)
                .height(ROW)
                .align_child(Center, Center)
                .child(Text::new(mark).size(12.0).color(color)),
        ));
    }

    parts.push(Box::new(
        Rectangle::new()
            .width(ICON)
            .height(ROW)
            .align_child(Center, Center)
            .child(Text::new(device.kind.icon()).size(15.0).color(color)),
    ));

    parts.push(Box::new(
        Rectangle::new()
            .width(Parent)
            .height(ROW)
            .align_child(Start, Center)
            .child(
                Text::new(device.label.as_str())
                    .size(13.0)
                    .color(color)
                    .elide(),
            ),
    ));

    let row = Rectangle::new()
        .width(Parent)
        .height(ROW)
        .fill(if device.default && choosable {
            t.bg2
        } else {
            t.bg0
        })
        .padding(SIDES)
        .child(Row::new(parts).gap(GAP).align(Center).height(ROW));

    if !choosable || device.default {
        return row;
    }

    let id = device.id;

    row.cursor(Pointer)
        .on_click(move |_| Devices::set_default(id))
}

fn note(t: &Theme, text: &str) -> Box<dyn Widget> {
    Box::new(
        Rectangle::new()
            .width(Parent)
            .height(ROW)
            .padding(SIDES)
            .align_child(Start, Center)
            .child(Text::new(text).size(13.0).color(t.comment)),
    )
}
