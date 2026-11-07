/*
 * bluetooth panel, top-right: the power switch, a pairing code to confirm when
 * one is waiting, and the devices in three groups; click a device to connect,
 * disconnect or pair it
 *
 * the state behind it (scanning, results, the pairing agent) is in src/bluetooth
 */

use amane::{
    Align::{Center, Start},
    Bluetooth, BluetoothDevice, Column,
    Cursor::Pointer,
    Key, LayerWindow, Monitor, Rectangle, Row, ScrollArea, Service,
    Size::Parent,
    Text, Weight, Widget, children,
};

use super::ui::{self, BUTTON, PADDING, SIDES, button};
use super::{BLUETOOTH, panel};
use crate::bluetooth::agent::{self, Agent, Kind, Request};
use crate::bluetooth::{self, BluetoothPanel, ICON_RESCAN, Job};
use crate::theme::Theme;

const HEIGHT: f32 = 560.0;
const GAP: f32 = 8.0;
const LINE: f32 = 20.0;
const ROW: f32 = 40.0;
const SECTION: f32 = 28.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = ui::corner(HEIGHT, "amane-bluetooth");

    panel(monitor, BLUETOOTH, base, |_: Key| {}, content)
}

fn content(t: &Theme) -> Rectangle {
    let bluetooth = Bluetooth::read();
    let panel = BluetoothPanel::read();
    let agent = Agent::read();

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(header(t, &bluetooth))];

    if let Some(request) = agent.request() {
        parts.push(Box::new(pairing(t, request)));
    }

    if let Some(failure) = panel.failure() {
        parts.push(Box::new(failure_line(t, failure)));
    }

    parts.push(Box::new(devices(t, &bluetooth, &panel)));

    ui::corner_frame(t, GAP, parts)
}

// "Bluetooth", scanning and rescan, and the power switch
fn header(t: &Theme, bluetooth: &Bluetooth) -> Row {
    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(ui::title(t, "Bluetooth"))];

    if bluetooth.powered() {
        if bluetooth.scanning() {
            parts.push(Box::new(Text::new("scanning").size(12.0).color(t.comment)));
        }

        parts.push(Box::new(ui::icon_button(t, ICON_RESCAN, |_| {
            Bluetooth::start_scan()
        })));
    }

    if bluetooth.available() {
        let powered = bluetooth.powered();

        parts.push(Box::new(ui::switch(
            t,
            "Bluetooth",
            124.0,
            powered,
            move |_| BluetoothPanel::power(!powered),
        )));
    }

    Row::new(parts).gap(GAP).align(Center).height(BUTTON)
}

// the pairing agent's question, or the code to type on the device
fn pairing(t: &Theme, request: &Request) -> Rectangle {
    let (title, detail) = match &request.kind {
        Kind::Confirm(code) => (
            format!("Pair with {}?", request.name),
            format!("Check that it shows {code}"),
        ),
        Kind::Authorize => (
            format!("Pair with {}?", request.name),
            String::from("It asked to pair with this computer"),
        ),
        Kind::Display(code) => (
            format!("Type {code} on {}", request.name),
            String::from("then press Enter on it"),
        ),
        Kind::NeedsCode => (
            format!("{} needs a code typed in", request.name),
            String::from("Pair it once with bluetoothctl, then connect here"),
        ),
    };

    let text = |text: String, size: f32, color| {
        Rectangle::new()
            .width(Parent)
            .height(LINE)
            .align_child(Start, Center)
            .child(Text::new(text).size(size).color(color).elide())
    };

    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(text(title, 14.0, t.fg)),
        Box::new(text(detail, 12.0, t.fg_dim)),
    ];

    let mut height = PADDING * 2.0 + LINE * 2.0 + GAP;

    let buttons: Box<dyn Widget> = if request.answerable() {
        Box::new(
            Row::new(children![
                button(t, "Confirm", 84.0, |_| agent::confirm()),
                button(t, "Cancel", 84.0, |_| agent::cancel()),
            ])
            .gap(GAP)
            .height(BUTTON),
        )
    } else {
        Box::new(button(t, "Dismiss", 84.0, |_| agent::clear()))
    };

    height += GAP + BUTTON;

    parts.push(buttons);

    Rectangle::new()
        .width(Parent)
        .height(height)
        .fill(t.bg1)
        .border(1.0, t.accent)
        .padding(PADDING)
        .child(Column::new(parts).gap(GAP))
}

fn failure_line(t: &Theme, failure: &str) -> Row {
    Row::new(children![
        Rectangle::new()
            .width(Parent)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(Text::new(failure).size(13.0).color(t.accent).elide()),
        button(t, "Dismiss", 72.0, |_| BluetoothPanel::dismiss_failure()),
    ])
    .gap(GAP)
    .align(Center)
    .height(BUTTON)
}

// connected, then paired, then the ones found by scanning
fn devices(t: &Theme, bluetooth: &Bluetooth, panel: &BluetoothPanel) -> Rectangle {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    let message = if !bluetooth.available() {
        Some("No Bluetooth adapter")
    } else if !bluetooth.powered() {
        Some("Bluetooth is off")
    } else {
        None
    };

    if let Some(message) = message {
        rows.push(note(t, message));
    } else {
        let shown: Vec<&BluetoothDevice> = bluetooth
            .devices()
            .iter()
            .filter(|device| device.paired() || !bluetooth::unnamed(device))
            .collect();

        let groups: [(&str, Vec<&BluetoothDevice>); 3] = [
            (
                "Connected",
                shown
                    .iter()
                    .copied()
                    .filter(|device| device.connected())
                    .collect(),
            ),
            (
                "Paired",
                shown
                    .iter()
                    .copied()
                    .filter(|device| device.paired() && !device.connected())
                    .collect(),
            ),
            (
                "Available",
                shown
                    .iter()
                    .copied()
                    .filter(|device| !device.paired())
                    .collect(),
            ),
        ];

        for (title, devices) in groups {
            // the list of new devices says when it's still empty, the others just go
            if devices.is_empty() && title != "Available" {
                continue;
            }

            rows.push(section(t, title));

            if devices.is_empty() {
                let text = if bluetooth.scanning() {
                    "Looking for devices…"
                } else {
                    "No new devices, press ↻ to look again"
                };

                rows.push(note(t, text));
            }

            for device in devices {
                rows.push(Box::new(device_row(t, device, panel)));
            }
        }
    }

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(t.bg0)
        .border(1.0, t.bg3)
        .clip()
        .child(ScrollArea::new("bluetooth-list", Column::new(rows)))
}

fn section(t: &Theme, title: &str) -> Box<dyn Widget> {
    Box::new(
        Rectangle::new()
            .width(Parent)
            .height(SECTION)
            .padding(SIDES)
            .align_child(Start, Center)
            .child(
                Text::new(title)
                    .size(12.0)
                    .weight(Weight::Bold)
                    .color(t.comment),
            ),
    )
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

fn device_row(t: &Theme, device: &BluetoothDevice, panel: &BluetoothPanel) -> Rectangle {
    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Rectangle::new()
                .width(LINE)
                .height(ROW)
                .align_child(Center, Center)
                .child(
                    Text::new(bluetooth::device_icon(device))
                        .size(16.0)
                        .color(if device.connected() { t.fg } else { t.fg_dim }),
                ),
        ),
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(ROW)
                .align_child(Start, Center)
                .child(Text::new(device.name()).size(14.0).color(t.fg).elide()),
        ),
    ];

    if let Some(job) = panel.job_for(device.path()) {
        let label = match job {
            Job::Connecting => "Connecting…",
            Job::Disconnecting => "Disconnecting…",
            Job::Pairing => "Pairing…",
            Job::Forgetting => "Forgetting…",
            Job::PoweringOn | Job::PoweringOff => "",
        };

        parts.push(Box::new(Text::new(label).size(12.0).color(t.fg_dim)));
    }

    if let Some(battery) = device.battery() {
        parts.push(Box::new(
            Text::new(format!("{battery}%")).size(12.0).color(t.fg_dim),
        ));
    }

    if device.connected() {
        let disconnected = device.clone();

        parts.push(Box::new(button(t, "Disconnect", 96.0, move |_| {
            BluetoothPanel::disconnect(&disconnected)
        })));
    } else if device.paired() {
        let forgotten = device.clone();

        parts.push(Box::new(button(t, "Forget", 64.0, move |_| {
            BluetoothPanel::forget(&forgotten)
        })));
    }

    // a connected device disconnects with its button, the row does the next step
    let clicked = device.clone();

    Rectangle::new()
        .width(Parent)
        .height(ROW)
        .padding(SIDES)
        .cursor(Pointer)
        .on_click(move |_| {
            if clicked.connected() {
                BluetoothPanel::disconnect(&clicked);
            } else if clicked.paired() {
                BluetoothPanel::connect(&clicked);
            } else {
                BluetoothPanel::pair(&clicked);
            }
        })
        .child(Row::new(parts).gap(GAP).align(Center).height(ROW))
}
