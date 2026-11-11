/*
 * audio panel, top-right: what's playing with its controls, volume and mute
 * for the default output and microphone, and the devices to pick the default from
 *
 * volume and mute are amane's Audio, live; the device lists come from wpctl,
 * read while the panel is open (see src/audio). "Now playing" is amane's Media,
 * its active player: the one playing, else the one shown last; covers come
 * from src/covers.rs
 */

use std::time::Duration;

use amane::{
    Align::{Center, Start},
    Audio, Column,
    Cursor::Pointer,
    Image, Key, LayerWindow, Media, MediaPlayer, Monitor, Rectangle, Row, ScrollArea, Service,
    Size::Parent,
    Text, Weight, Widget, children,
};

use super::ui::{self, BUTTON, CORNER_WIDTH, PADDING, SIDES};
use super::{AUDIO, hints, panel};
use crate::audio::{Device, Devices, State};
use crate::covers;
use crate::theme::Theme;

const GAP: f32 = 8.0;
const ROW: f32 = 36.0;
const ICON: f32 = 20.0;
const PERCENT: f32 = 40.0;
// sized for Unmute, the longer of its two labels
const MUTE: f32 = ui::fit("Unmute");

// fits "Now playing" and the device lists at their full room (OUTPUT_LIST, INPUT_LIST)
const HEIGHT: f32 = 460.0 + PLAYING_SECTION + GAP * 2.0;

// the slider takes what's left of the row
const SLIDER: f32 =
    CORNER_WIDTH - PADDING * 2.0 - ICON - PERCENT - MUTE - GAP * 3.0 - PADDING * 2.0;

// nerd font material design glyphs
const ICON_MUTED: &str = "\u{f075f}";
const ICON_LOW: &str = "\u{f057f}";
const ICON_MEDIUM: &str = "\u{f0580}";
const ICON_HIGH: &str = "\u{f057e}";
pub const ICON_MIC: &str = "\u{f036c}";
pub const ICON_MIC_MUTED: &str = "\u{f036d}";

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
const KEY_PLAY: char = 'p';
const KEY_PREVIOUS: char = 'h';
const KEY_NEXT: char = 'l';

// the microphone's: Shift with the output's keys, which amane reports as capitals
const KEY_MIC_QUIETER: char = 'J';
const KEY_MIC_LOUDER: char = 'K';

// how far j and k move the volume
const STEP: i32 = 5;

fn key_pressed(key: Key) {
    let Key::Character(letter) = key else {
        return;
    };

    // before lowercasing, which would make them the output's keys
    match letter {
        KEY_MIC_QUIETER => return step_microphone(-STEP),
        KEY_MIC_LOUDER => return step_microphone(STEP),
        _ => {}
    }

    match letter.to_ascii_lowercase() {
        KEY_MUTE => Audio::toggle_mute(),
        KEY_MIC_MUTE => Audio::toggle_microphone_mute(),
        KEY_QUIETER => step(-STEP),
        KEY_LOUDER => step(STEP),
        KEY_PLAY => Media::play_pause(),
        KEY_PREVIOUS => Media::previous(),
        KEY_NEXT => Media::next(),
        _ => {}
    }
}

// the output volume, kept between 0 and 100
fn step(by: i32) {
    let volume = i32::from(Audio::read().volume());

    Audio::set_volume((volume + by).clamp(0, 100) as u8);
}

// the microphone volume, the same way
fn step_microphone(by: i32) {
    let volume = i32::from(Audio::read().microphone_volume());

    Audio::set_microphone_volume((volume + by).clamp(0, 100) as u8);
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
        volume_keys: Some((KEY_MIC_QUIETER, KEY_MIC_LOUDER)),
    };

    let parts = children![
        ui::title(t, "Audio"),
        now_playing(t),
        section(
            t,
            "Output",
            &output,
            devices.outputs(),
            devices.state(),
            OUTPUT_LIST
        ),
        section(
            t,
            "Input",
            &input,
            devices.inputs(),
            devices.state(),
            INPUT_LIST
        ),
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

/*
 * a device list's room in the fixed-height panel (see HEIGHT); more devices
 * than fit scroll instead of pushing the next section out of the panel
 */
struct DeviceList {
    rows: usize,
    scroll_id: &'static str,
}

const OUTPUT_LIST: DeviceList = DeviceList {
    rows: 4,
    scroll_id: "audio-outputs",
};

const INPUT_LIST: DeviceList = DeviceList {
    rows: 2,
    scroll_id: "audio-inputs",
};

fn section(
    t: &Theme,
    title: &str,
    level: &Level,
    devices: &[Device],
    state: &State,
    room: DeviceList,
) -> Column {
    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(ui::heading(t, title, ui::HEADING)),
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

    let shown_rows = list.len().min(room.rows);

    parts.push(Box::new(
        Rectangle::new()
            .width(Parent)
            .height(ROW * shown_rows as f32)
            .fill(t.bg0)
            .border(1.0, t.bg3)
            .clip()
            .child(ScrollArea::new(room.scroll_id, Column::new(list))),
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
            .size(ui::TEXT_NOTE)
            .weight(Weight::Bold)
            .color(t.accent),
        _ => Text::new(format!("{}%", level.volume))
            .size(ui::TEXT_NOTE)
            .color(color),
    };

    let row = Row::new(children![
        Rectangle::new()
            .width(ICON)
            .height(BUTTON)
            .align_child(Center, Center)
            .child(Text::new(level.icon).size(ui::ICON_SIZE).color(color)),
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
                .child(Text::new(mark).size(ui::TEXT_SMALL).color(color)),
        ));
    }

    parts.push(Box::new(
        Rectangle::new()
            .width(ICON)
            .height(ROW)
            .align_child(Center, Center)
            .child(
                Text::new(device.kind.icon())
                    .size(ui::TEXT_TITLE)
                    .color(color),
            ),
    ));

    parts.push(Box::new(
        Rectangle::new()
            .width(Parent)
            .height(ROW)
            .align_child(Start, Center)
            .child(
                Text::new(device.label.as_str())
                    .size(ui::TEXT_NOTE)
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
    Box::new(ui::note(t, text, ROW))
}

/*
 *  Now playing
 *  ┌──────┐ Song title
 *  │ art  │ Artist · Spotify
 *  │      │ ━━━━━━━━━━━━━──────  1:23 / 3:45
 *  └──────┘ [󰒮 h] [󰐊 p] [󰒭 l]
 *
 * the same height with nothing playing, since panels never change size
 */
const ART: f32 = 64.0;
const TITLE_LINE: f32 = 18.0;
const ARTIST_LINE: f32 = 16.0;
const LINE_GAP: f32 = 4.0;
const CARD_PADDING: f32 = 12.0;
const PLAYING_CARD: f32 =
    CARD_PADDING * 2.0 + TITLE_LINE + ARTIST_LINE + BUTTON * 2.0 + LINE_GAP * 3.0;
const PLAYING_SECTION: f32 = ui::HEADING + GAP + PLAYING_CARD;

// the column beside the art
const PLAYING_TEXT: f32 = CORNER_WIDTH - PADDING * 2.0 - CARD_PADDING * 2.0 - ART - GAP * 1.5;

// times are 11 wide BlexMono, monospaced, so a character is 0.6 of that
const TIME_SIZE: f32 = 11.0;

// nerd font material design glyphs
const ICON_PLAY: &str = "\u{f040a}";
const ICON_PAUSE: &str = "\u{f03e4}";
const ICON_PREVIOUS: &str = "\u{f04ae}";
const ICON_NEXT: &str = "\u{f04ad}";
const ICON_MUSIC: &str = "\u{f075a}";

// the note shown in place of a cover while nothing plays
const PLACEHOLDER_ICON: f32 = 24.0;

fn now_playing(t: &Theme) -> Column {
    let media = Media::read();

    let card = Rectangle::new()
        .width(Parent)
        .height(PLAYING_CARD)
        .fill(t.bg0)
        .border(1.0, t.bg3)
        .padding(CARD_PADDING);

    let card = match media.active() {
        None => idle_card(t, card),
        Some(player) => playing_card(t, card, player),
    };

    Column::new(children![ui::heading(t, "Now playing", ui::HEADING), card]).gap(GAP)
}

fn idle_card(t: &Theme, card: Rectangle) -> Rectangle {
    card.align_child(Start, Center).child(
        Row::new(children![
            art(t, None),
            Text::new("Nothing playing")
                .size(ui::TEXT_NOTE)
                .color(t.comment),
        ])
        .gap(GAP * 1.5)
        .align(Center),
    )
}

// the cover, then the title, who and where, the progress and the buttons
fn playing_card(t: &Theme, card: Rectangle, player: &MediaPlayer) -> Rectangle {
    // a player with nothing loaded still has a name, like qutebrowser
    let title = match player.title() {
        "" => "Nothing playing",
        title => title,
    };

    let text = Column::new(children![
        ui::line(
            TITLE_LINE,
            Text::new(title)
                .size(ui::TEXT_BODY)
                .weight(Weight::Bold)
                .color(t.fg)
                .elide()
        ),
        ui::line(
            ARTIST_LINE,
            Text::new(source_text(player.artist(), player.identity()))
                .size(ui::TEXT_SMALL)
                .color(t.fg_dim)
                .elide()
        ),
        progress(t, player.position(), player.length()),
        media_buttons(t, player.playing()),
    ])
    .width(PLAYING_TEXT)
    .gap(LINE_GAP);

    card.child(
        Row::new(children![art(t, covers::path(player.art_url())), text])
            .gap(GAP * 1.5)
            .align(Start),
    )
}

// "artist · player", or whichever of the two there is
fn source_text(artist: &str, identity: &str) -> String {
    match (artist, identity) {
        ("", identity) => String::from(identity),
        (artist, "") => String::from(artist),
        (artist, identity) => format!("{artist} · {identity}"),
    }
}

// previous, play or pause, next
fn media_buttons(t: &Theme, playing: bool) -> Row {
    let play = if playing { ICON_PAUSE } else { ICON_PLAY };

    Row::new(children![
        ui::key_button(t, ICON_PREVIOUS, KEY_PREVIOUS, |_| Media::previous()),
        ui::key_button(t, play, KEY_PLAY, |_| Media::play_pause()),
        ui::key_button(t, ICON_NEXT, KEY_NEXT, |_| Media::next()),
    ])
    .gap(GAP)
    .height(BUTTON)
}

// the cover, or a music glyph until there is one
fn art(t: &Theme, cover: Option<std::path::PathBuf>) -> Rectangle {
    let square = Rectangle::new().width(ART).height(ART).fill(t.bg2).clip();

    match cover {
        Some(path) => square.fill(Image::cover(path).thumbnail(ART as u32 * 2, ART as u32 * 2)),
        None => square.align_child(Center, Center).child(
            Text::new(ICON_MUSIC)
                .size(PLACEHOLDER_ICON)
                .color(t.comment),
        ),
    }
}

// a bar and "1:23 / 3:45"; nothing for a stream without a length
fn progress(t: &Theme, position: Duration, length: Duration) -> Row {
    if length.is_zero() {
        return Row::new(Vec::new()).height(BUTTON);
    }

    let times = format!("{} / {}", clock(position), clock(length));

    let width = times.chars().count() as f32 * TIME_SIZE * 0.6;

    let percent = (position.as_secs_f32() / length.as_secs_f32() * 100.0).clamp(0.0, 100.0) as u8;

    Row::new(children![
        ui::meter(t, PLAYING_TEXT - width - GAP, percent, false),
        Rectangle::new()
            .width(width)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new(times).size(TIME_SIZE).color(t.comment)),
    ])
    .gap(GAP)
    .align(Center)
    .height(BUTTON)
}

// 0:05, 3:45, and 1:02:03 from an hour up
fn clock(duration: Duration) -> String {
    let seconds = duration.as_secs();

    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_source_names_artist_and_player_when_known() {
        assert_eq!(source_text("Daft Punk", "Spotify"), "Daft Punk · Spotify");
        assert_eq!(source_text("", "qutebrowser"), "qutebrowser");
        assert_eq!(source_text("Daft Punk", ""), "Daft Punk");
    }

    #[test]
    fn times_read_like_a_player() {
        assert_eq!(clock(Duration::from_secs(5)), "0:05");
        assert_eq!(clock(Duration::from_secs(225)), "3:45");
        assert_eq!(clock(Duration::from_secs(3723)), "1:02:03");
        assert_eq!(clock(Duration::from_millis(59_900)), "0:59");
    }
}
