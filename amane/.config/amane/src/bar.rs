/*
 * the top bar, one per monitor: that monitor's workspaces and mango layout on
 * the left, the clock in the middle, and status icons on the right (audio,
 * network with a vpn badge, bluetooth, the nixos snowflake); each icon opens
 * its panel. it reserves its height, so windows start below it
 */

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use amane::{
    Align::{Center, End},
    Animation, Audio, Bluetooth, Color,
    Cursor::Pointer,
    Full, Layer, LayerWindow, Monitor, Network, Padding, Rectangle, Row, Service,
    Size::Parent,
    Stack, Text, Vertical, Widget, Workspaces, Zone, children,
};

use crate::animations::{self, Animations};
use crate::audio::privacy::Privacy;
use crate::bluetooth::{
    self,
    agent::{Agent, Request},
};
use crate::layout::MangoLayout;
use crate::network::{self, Status};
use crate::panels::{AUDIO, BLUETOOTH, NETWORK, Panels, SYSTEM, audio::volume_icon, layout, ui};
use crate::theme::{Theme, theme};
use crate::vpn::{self, Vpn};
use crate::{clock, notifications};

// the bar's height; the backdrop starts below it so the bar stays clickable
pub const HEIGHT: f32 = 32.0;

// room at the bar's left and right ends
const END_PADDING: f32 = 8.0;

// between the bar's groups, and between its status icons
const GAP: f32 = 8.0;

// workspaces: the shown one a wide block, the others dots
const WORKSPACE_SHOWN: f32 = 20.0;
const WORKSPACE_DOT: f32 = 8.0;
const WORKSPACE_GAP: f32 = 4.0;

/*
 * each workspace's width and fill as drawn, moving towards what its state asks
 * for; the view is built anew on every redraw, so they're kept here, by
 * monitor and workspace id since every monitor has its own bar
 */
static INDICATORS: Mutex<Option<HashMap<(String, i64), Indicator>>> = Mutex::new(None);

struct Indicator {
    width: Animation<f32>,
    fill: Animation<Color>,
}

impl Indicator {
    // at rest where it should be, so the first bar and a new workspace don't animate in
    fn new(width: f32, fill: Color) -> Self {
        Self {
            width: Animation::new(width).duration(animations::DURATION),
            fill: Animation::new(fill).duration(animations::DURATION),
        }
    }
}

const LAYOUT_SIZE: f32 = 12.0;

// the box every status icon sits in, and its glyph's size
const ICON_WIDTH: f32 = 24.0;
const ICON_SIZE: f32 = 14.0;

// the network glyph's box, and the vpn badge on its top-right corner
const GLYPH: f32 = 16.0;
const BADGE: f32 = 10.0;
const BADGE_GLYPH: f32 = 9.0;
const BADGE_X: f32 = 10.0;
const BADGE_Y: f32 = -4.0;

// the privacy dot's diameter, like a workspace dot
const PRIVACY_DOT: f32 = WORKSPACE_DOT;

// nerd font linux-nixos
const ICON_NIXOS: &str = "\u{f313}";

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();

    LayerWindow::new()
        .width(Full)
        .height(HEIGHT)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .space(Zone::Reserve)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(t.bg_deep)
                .padding(Padding {
                    top: 0.0,
                    right: END_PADDING,
                    bottom: 0.0,
                    left: END_PADDING,
                })
                .on_click(|_| Panels::close_all())
                .child(
                    /*
                     * the two Parent-sized sides split the leftover space evenly,
                     * which keeps the fixed-width clock exactly centered
                     */
                    Row::new(children![
                        workspaces(&t, monitor),
                        clock::view(&t),
                        status(&t)
                    ])
                    .height(Parent)
                    .align(Center),
                ),
        )
}

fn workspaces(t: &Theme, monitor: &Monitor) -> Rectangle {
    let workspaces = Workspaces::read();

    let animate = Animations::read().enabled();

    // a plain lock, not a service, so holding it in the view is fine
    let mut indicators = INDICATORS.lock().unwrap_or_else(PoisonError::into_inner);
    let indicators = indicators.get_or_insert_with(HashMap::new);

    // this monitor's workspaces in this view; animations of ones gone are dropped after
    let mut seen = Vec::new();

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    /*
     * only this monitor's own workspaces; one the compositor doesn't place on a
     * monitor shows everywhere rather than nowhere
     */
    let own = workspaces.list().iter().filter(|workspace| {
        workspace
            .output()
            .is_none_or(|output| output == monitor.name)
    });

    for workspace in own {
        /*
         * the one shown on this monitor is wide: in the accent when this monitor
         * has focus, dimmer when another one does; an urgent one is a dot in the accent
         */
        let (width, fill) = if workspace.active() {
            let fill = if workspace.focused() {
                t.accent
            } else {
                t.fg_dim
            };

            (WORKSPACE_SHOWN, fill)
        } else if workspace.urgent() {
            (WORKSPACE_DOT, t.accent)
        } else if workspace.windows() > 0 {
            (WORKSPACE_DOT, t.bg4)
        } else {
            (WORKSPACE_DOT, t.bg1)
        };

        let id = workspace.id();

        seen.push(id);

        let indicator = indicators
            .entry((monitor.name.clone(), id))
            .or_insert_with(|| Indicator::new(width, fill));

        let width = animations::follow(&mut indicator.width, width, animate);
        let fill = animations::follow(&mut indicator.fill, fill, animate);

        buttons.push(Box::new(
            Rectangle::new()
                .width(width)
                .height(WORKSPACE_DOT)
                .align_child(Center, Center)
                .fill(fill)
                .cursor(Pointer)
                .on_click(move |_| Workspaces::focus(id)),
        ));
    }

    indicators.retain(|(name, id), _| *name != monitor.name || seen.contains(id));

    Rectangle::new().width(Parent).height(Parent).child(
        Row::new(children![
            Row::new(buttons)
                .gap(WORKSPACE_GAP)
                .align(Center)
                .height(Parent),
            layout(t, monitor),
        ])
        .gap(GAP)
        .align(Center)
        .height(Parent),
    )
}

// this monitor's mango layout like [S]; opens the layout picker on it
fn layout(t: &Theme, monitor: &Monitor) -> Rectangle {
    let symbol = MangoLayout::read().symbol(&monitor.name).map(String::from);

    let text = symbol
        .as_deref()
        .map(|symbol| format!("[{symbol}]"))
        .unwrap_or_default();

    Rectangle::new()
        .width(ui::text_width(&text, LAYOUT_SIZE))
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(move |_| layout::toggle_from(symbol.as_deref()))
        .child(Text::new(text).size(LAYOUT_SIZE).color(t.fg_dim))
}

// the right side; Parent-wide like the workspaces, so the clock stays centered
fn status(t: &Theme) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(End, Center)
        .child(Row::new(status_icons(t)).gap(GAP).align(Center))
}

/*
 * left to right: the privacy dot (only while recording or sharing), audio,
 * network, bluetooth (only on machines that have it), system
 */
fn status_icons(t: &Theme) -> Vec<Box<dyn Widget>> {
    let mut icons: Vec<Box<dyn Widget>> = Vec::new();

    if Privacy::read().active() {
        icons.push(Box::new(privacy_dot(t)));
    }

    icons.push(Box::new(audio(t)));
    icons.push(Box::new(network(t)));

    if let Some(bluetooth) = bluetooth_icon(t) {
        icons.push(Box::new(bluetooth));
    }

    icons.push(Box::new(system(t)));

    icons
}

// a status icon's box, opening its panel when clicked
fn status_button(panel: &'static str, icon: impl Widget + 'static) -> Rectangle {
    Rectangle::new()
        .width(ICON_WIDTH)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(move |_| {
            let _ = Panels::toggle(panel);
        })
        .child(icon)
}

/*
 * the nixos snowflake, in the accent while notifications wait unread; opens the
 * system panel, which has the ways into the wallpaper picker, the notification
 * center and the session menu
 */
fn system(t: &Theme) -> Rectangle {
    let color = if notifications::has_unread() {
        t.accent
    } else {
        t.fg_dim
    };

    status_button(SYSTEM, Text::new(ICON_NIXOS).size(ICON_SIZE).color(color))
}

// wi-fi strength, ethernet, no internet, wi-fi off or offline; opens the network panel
fn network(t: &Theme) -> Rectangle {
    let status = network::status(&Network::read());

    let color = match status {
        Status::NoInternet => t.accent,
        Status::Offline | Status::WifiOff | Status::Connecting => t.comment,
        _ => t.fg_dim,
    };

    let icon = Rectangle::new()
        .width(GLYPH)
        .height(GLYPH)
        .align_child(Center, Center)
        .child(Text::new(status.icon()).size(ICON_SIZE).color(color));

    let mut layers: Vec<Box<dyn Widget>> = vec![Box::new(icon)];

    /*
     * a badge on the icon's top-right corner while a vpn is up: dim while it only
     * reaches its own network, in the accent while all traffic goes through it,
     * like a tailscale exit node. a disc of the bar's own color under the shield
     * cuts it loose from the wifi bars it overlaps
     */
    if let Some(all_traffic) = Vpn::read().shield() {
        let shield = if all_traffic { t.accent } else { t.fg_dim };

        layers.push(Box::new(
            Rectangle::new()
                .width(BADGE)
                .height(BADGE)
                .radius(Full)
                .fill(t.bg_deep)
                .align_child(Center, Center)
                .translate(BADGE_X, BADGE_Y)
                .child(Text::new(vpn::ICON_SHIELD).size(BADGE_GLYPH).color(shield)),
        ));
    }

    status_button(NETWORK, Stack::new(layers))
}

// off, on, or a device connected; the accent while a pairing code waits for an answer
fn bluetooth_icon(t: &Theme) -> Option<Rectangle> {
    let (available, powered, connected) = {
        let bluetooth = Bluetooth::read();

        (
            bluetooth.available(),
            bluetooth.powered(),
            bluetooth.devices().iter().any(|device| device.connected()),
        )
    };

    if !available {
        return None;
    }

    let waiting = Agent::read().request().is_some_and(Request::answerable);

    let (icon, color) = if waiting {
        (bluetooth::ICON_ON, t.accent)
    } else if !powered {
        (bluetooth::ICON_OFF, t.comment)
    } else if connected {
        (bluetooth::ICON_CONNECTED, t.fg_dim)
    } else {
        (bluetooth::ICON_ON, t.fg_dim)
    };

    Some(status_button(
        BLUETOOTH,
        Text::new(icon).size(ICON_SIZE).color(color),
    ))
}

/*
 * a red dot while an app records from a microphone or the screen is shared;
 * red rather than the accent so it can't be missed. opens the audio panel
 */
fn privacy_dot(t: &Theme) -> Rectangle {
    status_button(
        AUDIO,
        Rectangle::new()
            .width(PRIVACY_DOT)
            .height(PRIVACY_DOT)
            .radius(Full)
            .fill(t.error),
    )
}

// the speaker by volume, dimmed while muted; opens the audio panel
fn audio(t: &Theme) -> Rectangle {
    let (volume, muted) = {
        let audio = Audio::read();

        (audio.volume(), audio.muted())
    };

    let dimmed = muted || volume == 0;

    status_button(
        AUDIO,
        Text::new(volume_icon(volume, muted))
            .size(ICON_SIZE)
            .color(if dimmed { t.comment } else { t.fg_dim }),
    )
}
