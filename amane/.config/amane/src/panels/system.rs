/*
 * system panel, top-right: who and where you are, the os and kernel, uptime,
 * cpu, memory, disk and battery use, the ways into the notification center and
 * the session menu, and the system settings as they're added
 *
 * the facts come from src/system.rs; cpu, memory and battery from amane
 */

use amane::{
    Align::{Center, End, Start},
    Battery, Cpu,
    Cursor::Pointer,
    Key, LayerWindow, Memory, Monitor, Rectangle, Row, Service,
    Size::Parent,
    Text, Weight, Widget, children,
};

use super::ui::{self, BUTTON, CORNER_WIDTH, PADDING};
use super::{NOTIFICATIONS, Panels, SESSION, SYSTEM, panel};
use crate::notifications::{self, ICON_BELL, ICON_BELL_OUTLINE};
use crate::system::{self, NetworkSpeed, SystemInfo};
use crate::theme::{ACCENTS, Accent, Theme};
use crate::vpn::{self, Vpn};

// header, details, five diagnostics rows (a laptop's battery included) and the settings
const HEIGHT: f32 = 324.0;
const GAP: f32 = 8.0;
const LINE: f32 = 20.0;

// a meter row: name, bar, then the numbers taking the rest
const NAME: f32 = 64.0;
const METER: f32 = 128.0;
const NUMBERS: f32 = CORNER_WIDTH - PADDING * 2.0 - NAME - METER - GAP * 2.0;

// the interface name at the end of the network row
const INTERFACE: f32 = 72.0;

// nerd font material design power glyph
const ICON_SESSION: &str = "\u{f0425}";

const KEY_NOTIFICATIONS: char = 'n';
const KEY_SESSION: char = 's';

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = ui::corner(HEIGHT, "amane-system");

    panel(monitor, SYSTEM, base, key_pressed, content)
}

fn content(t: &Theme) -> Rectangle {
    let info = SystemInfo::read();

    let mut parts: Vec<Box<dyn Widget>> =
        vec![Box::new(header(t, &info)), Box::new(details(t, &info))];

    parts.extend(meters(t, &info));

    parts.push(Box::new(ui::heading(t, "Settings", LINE)));
    parts.push(Box::new(accent_setting(t)));

    ui::frame(t, GAP, parts)
}

// the header buttons by their letters, uppercase too; escape is handled by the panel manager
fn key_pressed(key: Key) {
    let Key::Character(letter) = key else {
        return;
    };

    match letter.to_ascii_lowercase() {
        KEY_NOTIFICATIONS => open_notifications(),
        KEY_SESSION => open_session(),
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

// user@host, and the ways into the notification center and the session menu
fn header(t: &Theme, info: &SystemInfo) -> Row {
    let title = Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            Text::new(format!("{}@{}", info.user, info.host))
                .size(15.0)
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

    let session = ui::key_button(t, ICON_SESSION, KEY_SESSION, |_| open_session());

    Row::new(children![title, notifications, session])
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
        .child(Text::new(line).size(12.0).color(t.fg_dim).elide())
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
            .child(Text::new(name).size(13.0).color(t.fg)),
        ui::meter(t, METER, percent),
        Rectangle::new()
            .width(NUMBERS)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new(numbers).size(12.0).color(t.fg_dim).elide()),
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
            .child(Text::new("Network").size(13.0).color(t.fg)),
        Rectangle::new()
            .width(Parent)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new(speeds).size(12.0).color(t.fg).elide()),
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
            .child(Text::new("Accent").size(13.0).color(t.fg)),
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
 * the interface name, with the vpn shield after it while one is up: dim for
 * a vpn that only reaches its own network, the accent while all traffic goes
 * through it (the speed is still measured on the interface underneath)
 */
fn interface_with_shield(t: &Theme, interface: String) -> Rectangle {
    let mut parts: Vec<Box<dyn Widget>> =
        vec![Box::new(Text::new(interface).size(12.0).color(t.comment))];

    if let Some(all_traffic) = Vpn::read().shield() {
        let color = if all_traffic { t.accent } else { t.fg_dim };

        parts.push(Box::new(
            Text::new(vpn::ICON_SHIELD).size(11.0).color(color),
        ));
    }

    Rectangle::new()
        .width(INTERFACE)
        .height(BUTTON)
        .align_child(End, Center)
        .child(Row::new(parts).gap(4.0).align(Center))
}
