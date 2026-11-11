/*
 * bluetooth panel, top-right: the power switch, a pairing code to confirm when
 * one is waiting, and the devices in three groups; click a device to connect,
 * disconnect or pair it
 *
 * the state behind it (scanning, results, the pairing agent) is in src/bluetooth
 */

use std::sync::atomic::{AtomicUsize, Ordering};

use amane::{
    Align::Center, Bluetooth, BluetoothDevice, Column, Cursor::Pointer, Key, LayerWindow, Monitor,
    Rectangle, Row, Service, Size::Parent, Text, Widget, children,
};

use super::ui::{self, BUTTON, PADDING, SIDES, button};
use super::{BLUETOOTH, list, panel};
use crate::bluetooth::agent::{self, Agent, Kind, Request};
use crate::bluetooth::{self, BluetoothPanel, Job};
use crate::theme::Theme;

const HEIGHT: f32 = 560.0;
const GAP: f32 = 8.0;
const LINE: f32 = 20.0;
const ROW: f32 = 40.0;
const SECTION: f32 = 28.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = ui::corner(HEIGHT, "amane-bluetooth");

    panel(monitor, BLUETOOTH, base, key_pressed, content)
}

const KEY_POWER: char = 'b';
const KEY_RESCAN: char = 'r';
const KEY_CANCEL: char = 'c';
const KEY_DISMISS: char = 'x';
const KEY_DISCONNECT: char = 'd';
const KEY_FORGET: char = 'f';

// Enter as the ? hints name it
const HINT_ENTER: &str = "Enter";

/*
 * which device the keys act on, by its place in the list (connected, paired,
 * then available); None until a key moves it, see list::step_highlight
 */
struct Highlight {
    index: Option<usize>,
}

impl Service for Highlight {
    fn new() -> Self {
        Self { index: None }
    }

    // changes only through keys
    fn listen() {}
}

/*
 * the header's buttons by their letters, the pairing question, and the list:
 * j/k or arrows move, Enter does what a click does, d disconnects, f forgets
 */
fn key_pressed(key: Key) {
    if let Some(delta) = list::highlight_delta(&key) {
        return move_highlight(delta);
    }

    let answerable = Agent::read().request().map(Request::answerable);

    match key {
        // a pairing question comes first: Enter answers it before the list
        Key::Enter if answerable == Some(true) => agent::confirm(),
        Key::Enter => act_on_highlighted(Action::Click),
        Key::Character(letter) => match letter.to_ascii_lowercase() {
            KEY_POWER => toggle_power(),
            KEY_RESCAN => rescan(),
            KEY_CANCEL if answerable == Some(true) => agent::cancel(),
            KEY_DISMISS if answerable == Some(false) => agent::clear(),
            KEY_DISMISS => BluetoothPanel::dismiss_failure(),
            KEY_DISCONNECT => act_on_highlighted(Action::Disconnect),
            KEY_FORGET => act_on_highlighted(Action::Forget),
            _ => {}
        },
        _ => {}
    }
}

// from the keys and the wheel
fn move_highlight(delta: isize) {
    let count = listed(&Bluetooth::read()).len();
    let mut highlight = Highlight::write();

    highlight.index = list::step_highlight(highlight.index, delta, count);
}

enum Action {
    Click,
    Disconnect,
    Forget,
}

fn act_on_highlighted(action: Action) {
    let Some(index) = Highlight::read().index else {
        return;
    };

    let device = {
        let bluetooth = Bluetooth::read();
        let devices = listed(&bluetooth);

        // the list may have shrunk since the highlight moved
        let Some(device) = devices.get(index.min(devices.len().saturating_sub(1))) else {
            return;
        };

        (*device).clone()
    };

    match action {
        Action::Click => click(&device),
        Action::Disconnect if device.connected() => BluetoothPanel::disconnect(&device),
        Action::Forget if device.paired() => BluetoothPanel::forget(&device),
        _ => {}
    }
}

// a connected device disconnects, a paired one connects, a new one pairs
fn click(device: &BluetoothDevice) {
    if device.connected() {
        BluetoothPanel::disconnect(device);
    } else if device.paired() {
        BluetoothPanel::connect(device);
    } else {
        BluetoothPanel::pair(device);
    }
}

// the first row the list draws while the highlight is in use
static FIRST_ROW: AtomicUsize = AtomicUsize::new(0);

// on close: the highlight and an old failure don't come back on the next open
pub fn reset() {
    Highlight::write().index = None;
    FIRST_ROW.store(0, Ordering::Relaxed);

    BluetoothPanel::dismiss_failure();
}

// only with an adapter, like the switch
fn toggle_power() {
    let (available, powered) = {
        let bluetooth = Bluetooth::read();

        (bluetooth.available(), bluetooth.powered())
    };

    if available {
        BluetoothPanel::power(!powered);
    }
}

// only while on, like the button
fn rescan() {
    if Bluetooth::read().powered() {
        Bluetooth::start_scan();
    }
}

fn content(t: &Theme) -> Rectangle {
    let bluetooth = Bluetooth::read();
    let panel = BluetoothPanel::read();
    let agent = Agent::read();

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(header(t, &bluetooth))];

    // what's above the list, so it knows how much of itself shows
    let mut above = BUTTON + GAP;

    if let Some(request) = agent.request() {
        let (card, height) = pairing(t, request);

        above += height + GAP;

        parts.push(Box::new(card));
    }

    if let Some(failure) = panel.failure() {
        above += BUTTON + GAP;

        parts.push(Box::new(failure_line(t, failure)));
    }

    let visible = HEIGHT - PADDING * 2.0 - above - ui::LIST_BORDER * 2.0;

    parts.push(Box::new(devices(t, &bluetooth, &panel, visible)));

    ui::frame(t, GAP, parts)
}

// "Bluetooth", scanning and rescan, and the power switch
fn header(t: &Theme, bluetooth: &Bluetooth) -> Row {
    let mut parts: Vec<Box<dyn Widget>> = Vec::new();

    if bluetooth.powered() {
        if bluetooth.scanning() {
            parts.push(Box::new(
                Text::new("Scanning…").size(ui::TEXT_SMALL).color(t.comment),
            ));
        }

        parts.push(Box::new(ui::key_button(
            t,
            ui::ICON_REFRESH,
            KEY_RESCAN,
            |_| rescan(),
        )));
    }

    if bluetooth.available() {
        parts.push(Box::new(ui::switch(
            t,
            "Bluetooth",
            bluetooth.powered(),
            KEY_POWER,
            |_| toggle_power(),
        )));
    }

    ui::header(ui::title(t, "Bluetooth"), parts)
}

// the pairing agent's question, or the code to type on the device; the card and its height
fn pairing(t: &Theme, request: &Request) -> (Rectangle, f32) {
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
        ui::line(LINE, Text::new(text).size(size).color(color).elide())
    };

    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(text(title, 14.0, t.fg)),
        Box::new(text(detail, 12.0, t.fg_dim)),
    ];

    let mut height = PADDING * 2.0 + LINE * 2.0 + GAP;

    let buttons: Box<dyn Widget> = if request.answerable() {
        Box::new(
            Row::new(children![
                ui::hint_text_button(t, "Confirm", ui::fit("Confirm"), HINT_ENTER, |_| {
                    agent::confirm()
                }),
                ui::key_text_button(t, "Cancel", ui::fit("Cancel"), KEY_CANCEL, |_| {
                    agent::cancel()
                }),
            ])
            .gap(GAP)
            .height(BUTTON),
        )
    } else {
        Box::new(ui::key_text_button(
            t,
            "Dismiss",
            ui::fit("Dismiss"),
            KEY_DISMISS,
            |_| agent::clear(),
        ))
    };

    height += GAP + BUTTON;

    parts.push(buttons);

    let card = Rectangle::new()
        .width(Parent)
        .height(height)
        .fill(t.bg1)
        .border(1.0, t.accent)
        .padding(PADDING)
        .child(Column::new(parts).gap(GAP));

    (card, height)
}

fn failure_line(t: &Theme, failure: &str) -> Row {
    ui::error_line(
        t,
        failure,
        Some(ui::key_text_button(
            t,
            "Dismiss",
            ui::fit("Dismiss"),
            KEY_DISMISS,
            |_| BluetoothPanel::dismiss_failure(),
        )),
    )
}

const AVAILABLE: &str = "Available";

// connected, then paired, then the ones found by scanning (unnamed new ones left out)
fn groups(bluetooth: &Bluetooth) -> [(&'static str, Vec<&BluetoothDevice>); 3] {
    let shown: Vec<&BluetoothDevice> = bluetooth
        .devices()
        .iter()
        .filter(|device| device.paired() || !bluetooth::unnamed(device))
        .collect();

    let picked = |keep: fn(&BluetoothDevice) -> bool| {
        shown
            .iter()
            .copied()
            .filter(|device| keep(device))
            .collect()
    };

    [
        ("Connected", picked(|device| device.connected())),
        (
            "Paired",
            picked(|device| device.paired() && !device.connected()),
        ),
        (AVAILABLE, picked(|device| !device.paired())),
    ]
}

// the devices in the order they're shown, which is the order the highlight counts in
fn listed(bluetooth: &Bluetooth) -> Vec<&BluetoothDevice> {
    if !bluetooth.powered() {
        return Vec::new();
    }

    groups(bluetooth)
        .into_iter()
        .flat_map(|(_, devices)| devices)
        .collect()
}

fn devices(t: &Theme, bluetooth: &Bluetooth, panel: &BluetoothPanel, visible: f32) -> Rectangle {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // each row's height, to know which ones fit
    let mut heights: Vec<f32> = Vec::new();

    /*
     * where the highlighted device's row is among all the rows, headings
     * included, and the row to keep in sight with it: its group's heading
     * when it's the group's first device
     */
    let mut highlighted_row = None;
    let mut anchor = 0;

    let highlighted = Highlight::read()
        .index
        .map(|index| index.min(listed(bluetooth).len().saturating_sub(1)));

    let message = if !bluetooth.available() {
        Some("No Bluetooth adapter")
    } else if !bluetooth.powered() {
        Some("Bluetooth is off")
    } else {
        None
    };

    if let Some(message) = message {
        rows.push(note(t, message));
        heights.push(ROW);
    } else {
        let mut index = 0;

        for (title, devices) in groups(bluetooth) {
            // the list of new devices says when it's still empty, the others just go
            if devices.is_empty() && title != AVAILABLE {
                continue;
            }

            let heading = rows.len();

            rows.push(Box::new(ui::heading(t, title, SECTION).padding(SIDES)));
            heights.push(SECTION);

            if devices.is_empty() {
                let text = if bluetooth.scanning() {
                    "Looking for devices…"
                } else {
                    "No new devices, press r to look again"
                };

                rows.push(note(t, text));
                heights.push(ROW);
            }

            for (place, device) in devices.into_iter().enumerate() {
                let lit = highlighted == Some(index);

                if lit {
                    highlighted_row = Some(rows.len());
                    anchor = if place == 0 { heading } else { rows.len() };
                }

                rows.push(Box::new(device_row(t, device, panel, lit)));
                heights.push(ROW);

                index += 1;
            }
        }
    }

    let keep = highlighted_row.map(|row| (anchor, row));

    list::highlight_list(
        t,
        rows,
        &heights,
        keep,
        visible,
        &FIRST_ROW,
        "bluetooth-list",
        move_highlight,
    )
}

fn note(t: &Theme, text: &str) -> Box<dyn Widget> {
    Box::new(ui::note(t, text, ROW))
}

fn device_row(
    t: &Theme,
    device: &BluetoothDevice,
    panel: &BluetoothPanel,
    highlighted: bool,
) -> Rectangle {
    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Rectangle::new()
                .width(LINE)
                .height(ROW)
                .align_child(Center, Center)
                .child(
                    Text::new(bluetooth::device_icon(device))
                        .size(ui::ICON_SIZE)
                        .color(if device.connected() { t.fg } else { t.fg_dim }),
                ),
        ),
        Box::new(ui::line(
            ROW,
            Text::new(device.name())
                .size(ui::TEXT_BODY)
                .color(t.fg)
                .elide(),
        )),
    ];

    if let Some(job) = panel.job_for(device.path()) {
        parts.push(Box::new(
            Text::new(job_label(job))
                .size(ui::TEXT_SMALL)
                .color(t.fg_dim),
        ));
    }

    if let Some(battery) = device.battery() {
        parts.push(Box::new(
            Text::new(format!("{battery}%"))
                .size(ui::TEXT_SMALL)
                .color(t.fg_dim),
        ));
    }

    if let Some(button) = device_button(t, device, highlighted) {
        parts.push(Box::new(button));
    }

    // a connected device disconnects with its button, the row does the next step
    let clicked = device.clone();

    Rectangle::new()
        .width(Parent)
        .height(ROW)
        .fill(if highlighted { t.bg3 } else { t.bg0 })
        .padding(SIDES)
        .cursor(Pointer)
        .on_click(move |_| click(&clicked))
        .child(Row::new(parts).gap(GAP).align(Center).height(ROW))
}

// what a device's job shows beside its name
fn job_label(job: Job) -> &'static str {
    match job {
        Job::Connecting => "Connecting…",
        Job::Disconnecting => "Disconnecting…",
        Job::Pairing => "Pairing…",
        Job::Forgetting => "Forgetting…",
        Job::PoweringOn | Job::PoweringOff => "",
    }
}

/*
 * Disconnect for a connected device, Forget for a paired one; the keys act on
 * the highlighted row, so only its button shows its key under ?
 */
fn device_button(t: &Theme, device: &BluetoothDevice, highlighted: bool) -> Option<Rectangle> {
    let (label, key, action): (&str, char, Box<dyn Fn()>) = if device.connected() {
        let device = device.clone();

        (
            "Disconnect",
            KEY_DISCONNECT,
            Box::new(move || BluetoothPanel::disconnect(&device)),
        )
    } else if device.paired() {
        let device = device.clone();

        (
            "Forget",
            KEY_FORGET,
            Box::new(move || BluetoothPanel::forget(&device)),
        )
    } else {
        return None;
    };

    let width = ui::fit(label);

    Some(if highlighted {
        ui::key_text_button(t, label, width, key, move |_| action())
    } else {
        button(t, label, width, move |_| action())
    })
}
