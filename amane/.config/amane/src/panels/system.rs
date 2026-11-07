/*
 * system panel, top-right: who and where you are, the os and kernel, uptime,
 * cpu, memory, disk and battery use, the ways into the notification center and
 * the session menu, and the system settings as they're added
 *
 * the facts come from src/system.rs; cpu, memory and battery from amane
 */

use amane::{
    Align::{Center, Start},
    Battery, Column, Cpu, Horizontal, Key, LayerWindow, Margin, Memory, Monitor, Rectangle, Row,
    Service,
    Size::Parent,
    Text, Vertical, Weight, Widget, children,
};

use super::ui::{self, BUTTON};
use super::{NOTIFICATIONS, Panels, SESSION, SYSTEM, panel_window};
use crate::notifications::{self, ICON_BELL, ICON_BELL_OUTLINE};
use crate::system::{self, SystemInfo};
use crate::theme::{Theme, theme};

const WIDTH: f32 = 400.0;
// header, details and four meter rows, so a laptop's battery fits too
const HEIGHT: f32 = 224.0;
const PADDING: f32 = 12.0;
const GAP: f32 = 8.0;
const LINE: f32 = 20.0;

// a meter row: name, bar, then the numbers taking the rest
const NAME: f32 = 64.0;
const METER: f32 = 128.0;
const NUMBERS: f32 = WIDTH - PADDING * 2.0 - NAME - METER - GAP * 2.0;

// nerd font material design power glyph
const ICON_SESSION: &str = "\u{f0425}";

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();
    let info = SystemInfo::read();

    /*
     * a hidden window still runs its view whenever a service it read changes,
     * so cpu, memory and battery are only read while the panel shows
     */
    let shown = Panels::read().shown_on(SYSTEM, monitor);

    let mut parts: Vec<Box<dyn Widget>> =
        vec![Box::new(header(&t, &info)), Box::new(details(&t, &info))];

    if shown {
        parts.extend(meters(&t, &info));
    }

    let base = LayerWindow::new()
        .width(WIDTH)
        .height(HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 8,
            right: 8,
            bottom: 0,
            left: 0,
        })
        .namespace("amane-system")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(t.bg_deep)
                .border(1.0, t.bg3)
                .padding(PADDING)
                .clip()
                .child(Column::new(parts).gap(GAP)),
        );

    panel_window(monitor, SYSTEM, base, |_: Key| {})
}

// nothing to clear
pub fn reset() {}

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

    // frost while something waits unread, the same as the bar's snowflake
    let (bell, color) = if notifications::has_unread() {
        (ICON_BELL, t.frost)
    } else {
        (ICON_BELL_OUTLINE, t.fg)
    };

    // both sit in the right corner's group, so the center simply takes this panel's place
    let notifications = ui::icon_button_colored(t, bell, color, |_| {
        let _ = Panels::open(NOTIFICATIONS);
    });

    // the session menu takes over; switch waits for this panel to hide first
    let session = ui::icon_button(t, ICON_SESSION, |_| {
        let _ = Panels::switch(SYSTEM, SESSION);
    });

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
