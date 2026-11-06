/*
 * popups in the top-right corner of the focused monitor; each hides after a few
 * seconds but stays in the notification center, critical ones stay until handled,
 * and hovering a popup holds it
 *
 * the window is always mapped and never changes size: showing, hiding or resizing
 * a layer surface makes mango reconfigure the others on the monitor, which can
 * crash the shell when it crosses another panel showing or hiding (see panels/mod.rs).
 * it's transparent, and its input region covers only the cards, so the empty
 * part lets every click through
 *
 * lock order: Toasts before Notifications, and never Toasts::write while holding
 * another service, so the popup thread, the views and the dbus thread can't deadlock
 */

use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::{Duration, SystemTime};

use amane::{
    Apps, Column, Horizontal, InputArea, Keyboard, Layer, LayerWindow, Margin, Monitor,
    Notification, Notifications, Rectangle, Service, Urgency, Vertical, Widget,
};

use super::card::{self, Options};
use crate::panels::{self, BLUETOOTH, NETWORK, NOTIFICATIONS, Panels};
use crate::theme::theme;

const TIMEOUT: Duration = Duration::from_secs(5);

// after the pointer leaves a popup, it stays this much longer
const HOVER_GRACE: Duration = Duration::from_secs(2);

const MAX_SHOWN: usize = 4;

const WIDTH: f32 = 380.0;
const HEIGHT: f32 = 640.0;
const GAP: f32 = 8.0;
const MARGIN: i32 = 8;

pub struct Toasts {
    // closed with ×; keyed with the time received, so a replaced notification pops up again
    hidden: HashSet<(u32, SystemTime)>,

    // how long a popup stays after the pointer left it
    held_until: HashMap<u32, SystemTime>,

    hovered: Option<u32>,

    // what the last tick showed, to only redraw when that changes
    shown: Vec<u32>,
}

impl Service for Toasts {
    fn new() -> Self {
        Self {
            hidden: HashSet::new(),
            held_until: HashMap::new(),
            hovered: None,
            shown: Vec::new(),
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    /*
     * ticks to hide popups whose time ran out; only writes, and so redraws,
     * when the shown set changed. new notifications don't wait for a tick,
     * the popup window reads Notifications and redraws when one arrives
     */
    fn listen() {
        loop {
            thread::sleep(Self::interval());

            let shown = {
                let toasts = Toasts::read();
                let notifications = Notifications::read();

                let shown = shown_ids(&toasts, notifications.list());

                (shown != toasts.shown).then_some(shown)
            };

            if let Some(shown) = shown {
                let live: HashSet<u32> = Notifications::read()
                    .list()
                    .iter()
                    .map(Notification::id)
                    .collect();

                let mut toasts = Toasts::write();

                toasts.hidden.retain(|(id, _)| live.contains(id));
                toasts.held_until.retain(|id, _| live.contains(id));
                toasts.shown = shown;
            }
        }
    }
}

impl Toasts {
    fn shows(&self, notification: &Notification, now: SystemTime) -> bool {
        let id = notification.id();

        if self.hidden.contains(&(id, notification.received())) {
            return false;
        }

        if notification.urgency() == Urgency::Critical || self.hovered == Some(id) {
            return true;
        }

        let deadline = notification.received() + TIMEOUT;

        let deadline = match self.held_until.get(&id) {
            Some(held) => deadline.max(*held),
            None => deadline,
        };

        now < deadline
    }

    fn hide(id: u32, received: SystemTime) {
        Toasts::write().hidden.insert((id, received));
    }

    fn hover(id: u32, inside: bool) {
        let mut toasts = Toasts::write();

        if inside {
            toasts.hovered = Some(id);
        } else {
            if toasts.hovered == Some(id) {
                toasts.hovered = None;
            }

            toasts
                .held_until
                .insert(id, SystemTime::now() + HOVER_GRACE);
        }
    }
}

// newest first
fn shown<'a>(toasts: &Toasts, list: &'a [Notification]) -> Vec<&'a Notification> {
    let now = SystemTime::now();

    list.iter()
        .rev()
        .filter(|notification| toasts.shows(notification, now))
        .take(MAX_SHOWN)
        .collect()
}

fn shown_ids(toasts: &Toasts, list: &[Notification]) -> Vec<u32> {
    shown(toasts, list)
        .into_iter()
        .map(Notification::id)
        .collect()
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();

    // Toasts first, see the lock order at the top
    let toasts = Toasts::read();
    let notifications = Notifications::read();
    let apps = Apps::read();

    /*
     * none while a panel holds the same corner (the center lists them anyway),
     * and only the focused monitor gets popups
     */
    let right_panel = {
        let panels = Panels::read();

        [NOTIFICATIONS, NETWORK, BLUETOOTH]
            .iter()
            .any(|name| panels.is_open(name))
    };

    let hidden = right_panel || !panels::is_focused(monitor);

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();
    let mut areas = Vec::new();
    let mut top = 0.0;

    if !hidden {
        for notification in shown(&toasts, notifications.list()) {
            let id = notification.id();
            let received = notification.received();

            let options = Options {
                width: WIDTH,
                body_lines: 2,
                now: None,
            };

            let (card, height) = card::card(&t, &apps, notification, options, move |_| {
                Toasts::hide(id, received)
            });

            // a card that wouldn't fit whole waits for room
            if top + height > HEIGHT {
                break;
            }

            areas.push(InputArea {
                x: 0,
                y: top as i32,
                width: WIDTH as i32,
                height: height.ceil() as i32,
            });

            cards.push(Box::new(
                card.on_hover(move |inside| Toasts::hover(id, inside)),
            ));

            top += height + GAP;
        }
    }

    LayerWindow::new()
        .width(WIDTH)
        .height(HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: MARGIN,
            right: MARGIN,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::None)
        .namespace("amane-notifications")
        .input_region(areas)
        .child(
            Rectangle::new()
                .width(WIDTH)
                .height(HEIGHT)
                .clip()
                .child(Column::new(cards).gap(GAP)),
        )
}
