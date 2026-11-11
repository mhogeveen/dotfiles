/*
 * system panel, top-right: who and where you are, the os and kernel, uptime,
 * cpu, memory, disk and battery use, the ways into the notification center and
 * the session menu and the wallpaper picker, and the system settings as they're added;
 * the facts in one card, the settings in another under their heading
 *
 * the facts come from src/system.rs; cpu, memory and battery from amane
 */

use amane::{
    Align::{Center, Start},
    Battery, Cpu,
    Cursor::Pointer,
    Key, LayerWindow, Memory, Monitor, Rectangle, Row, Service,
    Size::Parent,
    Text, Weight, Widget, children,
};

use super::ui::{self, BUTTON, CORNER_WIDTH, PADDING};
use super::{NOTIFICATIONS, Panels, SESSION, SYSTEM, WALLPAPER, panel};
use crate::animations::Animations;
use crate::brightness::{self, Level};
use crate::notifications::{self, ICON_BELL, ICON_BELL_OUTLINE};
use crate::system::{self, NetworkSpeed, SystemInfo};
use crate::theme::{ACCENTS, Accent, Theme};
use crate::vpn::{self, Vpn};

const GAP: f32 = 8.0;
const LINE: f32 = 20.0;

// the diagnostics rows at most: cpu, memory, disk, network and a laptop's battery
const DIAGNOSTICS_MOST: f32 = 5.0;

// the settings card: accent, brightness and animations
const SETTINGS_ROWS: f32 = 3.0;

/*
 * the header and the details line, the diagnostics card, and the settings
 * card under its heading; fixed, so room is kept for a battery row on a
 * desktop too
 */
const HEIGHT: f32 = PADDING * 2.0
    + BUTTON
    + GAP
    + LINE
    + GAP
    + card_height(DIAGNOSTICS_MOST)
    + GAP
    + LINE
    + GAP
    + card_height(SETTINGS_ROWS);

/*
 * a meter row: name, bar, then the numbers taking the rest of a card's
 * inside (the panel's padding and the card's own)
 */
const NAME: f32 = 96.0;
const METER: f32 = 104.0;
const NUMBERS: f32 = CORNER_WIDTH - PADDING * 4.0 - NAME - METER - GAP * 2.0;

// the interface name at the end of the network row
const INTERFACE: f32 = 72.0;

/*
 * the vpn shield after it; a box wider than the glyph measures, since nerd
 * font glyphs draw wider than their size says
 */
const SHIELD_WIDTH: f32 = 12.0;
const SHIELD_GAP: f32 = 4.0;
const SHIELD_SIZE: f32 = 11.0;

// nerd font material design power and wallpaper glyphs
const ICON_SESSION: &str = "\u{f0425}";
const ICON_WALLPAPER: &str = "\u{f0e09}";

const KEY_NOTIFICATIONS: char = 'n';
const KEY_SESSION: char = 's';
const KEY_WALLPAPER: char = 'w';
const KEY_ANIMATIONS: char = 'a';

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = ui::corner(HEIGHT, "amane-system");

    let name = monitor.name.clone();

    panel(monitor, SYSTEM, base, key_pressed, move |t| {
        content(t, &name)
    })
}

fn content(t: &Theme, monitor: &str) -> Rectangle {
    let info = SystemInfo::read();

    let diagnostics = meters(t, &info);
    let diagnostics_height = card_height(diagnostics.len() as f32);

    let settings: Vec<Box<dyn Widget>> = vec![
        Box::new(accent_setting(t)),
        Box::new(brightness_setting(t, monitor)),
        Box::new(animations_setting(t)),
    ];

    /*
     * the details line stays outside the cards, where it has the panel's
     * full width and isn't cut short
     */
    ui::frame(
        t,
        GAP,
        children![
            header(t, &info),
            details(t, &info),
            ui::card(t, diagnostics_height, diagnostics),
            ui::heading(t, "Settings", LINE),
            ui::card(t, card_height(SETTINGS_ROWS), settings),
        ],
    )
}

// a card of this many button-tall rows: its padding, the rows and the gaps between them
const fn card_height(rows: f32) -> f32 {
    PADDING * 2.0 + BUTTON * rows + GAP * (rows - 1.0)
}

/*
 * the header buttons and the animations switch by their letters, uppercase
 * too; escape is handled by the panel manager
 */
fn key_pressed(key: Key) {
    match super::letter(&key) {
        Some(KEY_NOTIFICATIONS) => open_notifications(),
        Some(KEY_SESSION) => open_session(),
        Some(KEY_WALLPAPER) => open_wallpaper(),
        Some(KEY_ANIMATIONS) => Animations::toggle(),
        _ => {}
    }
}

// the notification center takes this panel's place
fn open_notifications() {
    let _ = Panels::open(NOTIFICATIONS);
}

// the session menu takes this panel's place, like any other panel would
fn open_session() {
    let _ = Panels::open(SESSION);
}

// the wallpaper picker along the bottom takes this panel's place too
fn open_wallpaper() {
    let _ = Panels::open(WALLPAPER);
}

// user@host, and the ways into the notification center and the session menu
fn header(t: &Theme, info: &SystemInfo) -> Row {
    let title = Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            Text::new(format!("{}@{}", info.user, info.host))
                .size(ui::TEXT_TITLE)
                .weight(Weight::Bold)
                .color(t.fg)
                .elide(),
        );

    // the accent while something waits unread, the same as the bar's snowflake
    let (bell, color) = if notifications::has_unread() {
        (ICON_BELL, t.accent)
    } else {
        (ICON_BELL_OUTLINE, t.fg)
    };

    let notifications =
        ui::key_button_colored(t, bell, color, KEY_NOTIFICATIONS, |_| open_notifications());

    let wallpaper = ui::key_button(t, ICON_WALLPAPER, KEY_WALLPAPER, |_| open_wallpaper());

    let session = ui::key_button(t, ICON_SESSION, KEY_SESSION, |_| open_session());

    Row::new(children![title, wallpaper, notifications, session])
        .gap(GAP)
        .align(Center)
        .height(BUTTON)
}

// NixOS 26.05 (Yarara) · kernel 6.18.60 · up 2h 14m
fn details(t: &Theme, info: &SystemInfo) -> Rectangle {
    let line = format!(
        "{} · kernel {} · up {}",
        info.os,
        info.kernel,
        info.uptime()
    );

    Rectangle::new()
        .width(Parent)
        .height(LINE)
        .align_child(Start, Center)
        .child(Text::new(line).size(ui::TEXT_SMALL).color(t.fg_dim).elide())
}

fn meters(t: &Theme, info: &SystemInfo) -> Vec<Box<dyn Widget>> {
    let cpu = Cpu::read();
    let memory = Memory::read();
    let battery = Battery::read();

    let mut rows: Vec<Box<dyn Widget>> = vec![
        Box::new(meter_row(
            t,
            "CPU",
            cpu.percent(),
            format!("{}% · {} cores", cpu.percent(), info.cores),
        )),
        Box::new(meter_row(
            t,
            "Memory",
            memory.percent(),
            format!(
                "{} / {} GiB",
                system::gib(memory.used_kib() * 1024),
                system::gib(memory.total_kib() * 1024)
            ),
        )),
    ];

    if let Some(disk) = info.disk {
        rows.push(Box::new(meter_row(
            t,
            "Disk /",
            disk.percent(),
            format!(
                "{} / {} GiB",
                system::gib(disk.used),
                system::gib(disk.total)
            ),
        )));
    }

    rows.push(Box::new(network_row(t, info.network.as_ref())));

    // desktops have none, a mouse's battery doesn't count
    if battery.present() {
        let state = if battery.charging() {
            " charging"
        } else if battery.full() {
            " full"
        } else {
            ""
        };

        rows.push(Box::new(meter_row(
            t,
            "Battery",
            battery.percent(),
            format!("{}%{state}", battery.percent()),
        )));
    }

    rows
}

fn meter_row(t: &Theme, name: &str, percent: u8, numbers: String) -> Row {
    Row::new(children![
        Rectangle::new()
            .width(NAME)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new(name).size(ui::TEXT_NOTE).color(t.fg)),
        ui::meter(t, METER, percent, false),
        Rectangle::new()
            .width(NUMBERS)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(
                Text::new(numbers)
                    .size(ui::TEXT_SMALL)
                    .color(t.fg_dim)
                    .elide()
            ),
    ])
    .gap(GAP)
    .align(Center)
    .height(BUTTON)
}

// down and up speed of the default-route interface, no bar since a speed has no maximum
fn network_row(t: &Theme, network: Option<&NetworkSpeed>) -> Row {
    let (speeds, interface) = match network {
        None => (String::from("Offline"), String::new()),
        Some(network) => {
            let speeds = match network.rate {
                Some((down, up)) => {
                    format!("↓ {}   ↑ {}", system::speed(down), system::speed(up))
                }

                // the first second has only one reading to go on
                None => String::from("…"),
            };

            (speeds, network.interface.clone())
        }
    };

    Row::new(children![
        Rectangle::new()
            .width(NAME)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new("Network").size(ui::TEXT_NOTE).color(t.fg)),
        Rectangle::new()
            .width(Parent)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new(speeds).size(ui::TEXT_SMALL).color(t.fg).elide()),
        interface_with_shield(t, interface),
    ])
    .width(Parent)
    .gap(GAP)
    .align(Center)
    .height(BUTTON)
}

const SWATCH: f32 = 20.0;
const SWATCH_GAP: f32 = 6.0;

// the accent used throughout the shell: one square per color, the one in use outlined
fn accent_setting(t: &Theme) -> Row {
    let current = Accent::read().color();

    let mut swatches: Vec<Box<dyn Widget>> = Vec::new();

    for accent in ACCENTS {
        let swatch = Rectangle::new()
            .width(SWATCH)
            .height(SWATCH)
            .fill(accent.color())
            .cursor(Pointer)
            .on_click(move |_| Accent::set(accent));

        let swatch = if accent == current {
            swatch.border(2.0, t.fg)
        } else {
            swatch
        };

        swatches.push(Box::new(swatch));
    }

    Row::new(children![
        Rectangle::new()
            .width(NAME)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new("Accent").size(ui::TEXT_NOTE).color(t.fg)),
        Row::new(swatches)
            .gap(SWATCH_GAP)
            .align(Center)
            .height(BUTTON),
    ])
    .gap(GAP)
    .align(Center)
    .height(BUTTON)
}

/*
 * the screen's brightness: a laptop's backlight, or the monitor's own setting
 * over DDC/CI (src/brightness.rs); the monitor is the one this panel is on
 */
fn brightness_setting(t: &Theme, monitor: &str) -> Row {
    brightness::follow(monitor);

    let name = Rectangle::new()
        .width(NAME)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(Text::new("Brightness").size(ui::TEXT_NOTE).color(t.fg));

    let numbers = |text: &str, color| {
        Rectangle::new()
            .width(NUMBERS)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new(text).size(ui::TEXT_SMALL).color(color))
    };

    let row = match brightness::level(monitor) {
        Level::Percent(percent) => children![
            name,
            ui::slider(t, METER, percent, false, brightness::set),
            numbers(&format!("{percent}%"), t.fg_dim),
        ],

        // ddcutil takes a moment the first time
        Level::Reading => children![
            name,
            ui::slider(t, METER, 0, true, |_| {}),
            numbers("…", t.comment),
        ],

        // no backlight, and the monitor doesn't answer DDC/CI (it can be off in its menu)
        Level::Unavailable => children![name, numbers("Not available", t.comment)],
    };

    Row::new(row).gap(GAP).align(Center).height(BUTTON)
}

/*
 * whether the shell animates (src/animations.rs), for every window at once;
 * the name flips it too, like a checkbox's label
 */
fn animations_setting(t: &Theme) -> Row {
    let enabled = Animations::read().enabled();

    Row::new(children![
        Rectangle::new()
            .width(NAME)
            .height(BUTTON)
            .align_child(Start, Center)
            .cursor(Pointer)
            .on_click(|_| Animations::toggle())
            .child(Text::new("Animations").size(ui::TEXT_NOTE).color(t.fg)),
        ui::checkbox(t, enabled, KEY_ANIMATIONS, |_| Animations::toggle()),
    ])
    .gap(GAP)
    .align(Center)
    .height(BUTTON)
}

/*
 * the interface name, with the vpn shield after it while one is up: dim for
 * a vpn that only reaches its own network, the accent while all traffic goes
 * through it (the speed is still measured on the interface underneath)
 */
fn interface_with_shield(t: &Theme, interface: String) -> Rectangle {
    let shield = Vpn::read().shield();

    // a long name (wlp0s20f3, tailscale0) is shortened to leave the shield its room
    let name_width = match shield {
        Some(_) => INTERFACE - SHIELD_WIDTH - SHIELD_GAP,
        None => INTERFACE,
    };

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(
        Rectangle::new()
            .width(name_width)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(
                Text::new(interface)
                    .size(ui::TEXT_SMALL)
                    .color(t.comment)
                    .elide(),
            ),
    )];

    if let Some(all_traffic) = shield {
        let color = if all_traffic { t.accent } else { t.fg_dim };

        parts.push(Box::new(
            Rectangle::new()
                .width(SHIELD_WIDTH)
                .height(BUTTON)
                .align_child(Start, Center)
                .child(Text::new(vpn::ICON_SHIELD).size(SHIELD_SIZE).color(color)),
        ));
    }

    Rectangle::new()
        .width(INTERFACE)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(Row::new(parts).gap(SHIELD_GAP).align(Center))
}
