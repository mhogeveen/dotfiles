/*
 * the network panel's vpn part: the tailscale card (state, address, the last
 * error, connect or log in, and the exit nodes) and a line for every other
 * tunnel; each part comes with its height, as the panel adds them up to know
 * how much room its wi-fi list has
 */

use amane::{
    Align::{Center, Start},
    Color, Column,
    Cursor::Pointer,
    Rectangle, Row, Service,
    Size::Parent,
    Text, Weight, Widget, children,
};

use super::{GAP, KEY_LOG_IN, KEY_TAILSCALE, LINE};
use crate::panels::ui::{self, BUTTON, PADDING, SIDES};
use crate::theme::Theme;
use crate::vpn::{self, Backend, Status as Tailscale, Suggestion, Vpn, tailscale};

const CARD_TITLE: f32 = 20.0;
const CARD_LINE: f32 = 16.0;
const ACTION_WIDTH: f32 = 100.0;

// title, address, an error, the actions, and three exit node buttons under their label
pub(super) const TAILSCALE_AT_MOST: f32 = PADDING * 2.0
    + CARD_TITLE
    + 2.0 * (GAP + CARD_LINE)
    + GAP
    + BUTTON
    + GAP
    + CARD_LINE
    + 3.0 * (GAP + BUTTON);

/*
 * the tailscale card, then a line for every other tunnel; with their height,
 * the gap after each included
 */
pub(super) fn vpn_parts(t: &Theme) -> (Vec<Box<dyn Widget>>, f32) {
    let vpn = Vpn::read();

    let mut parts: Vec<Box<dyn Widget>> = Vec::new();
    let mut height = 0.0;

    if let Some(status) = vpn.tailscale() {
        let (card, card_height) = tailscale_card(t, &vpn, status);

        height += card_height + GAP;

        parts.push(Box::new(card));
    }

    for tunnel in vpn.others() {
        height += LINE + GAP;

        let all = vpn.all_traffic() == Some(tunnel);

        let mut line = format!("VPN {} · {}", tunnel.interface, tunnel.kind.name());

        if all {
            line.push_str(" · all traffic");
        }

        parts.push(Box::new(
            Row::new(children![
                Text::new(vpn::ICON_SHIELD)
                    .size(ui::TEXT_NOTE)
                    .color(if all { t.accent } else { t.fg_dim }),
                Rectangle::new()
                    .width(Parent)
                    .height(LINE)
                    .align_child(Start, Center)
                    .child(Text::new(line).size(ui::TEXT_SMALL).color(t.fg_dim).elide()),
            ])
            .gap(GAP)
            .align(Center)
            .height(LINE),
        ));
    }

    (parts, height)
}

/*
 *  Tailscale@prometheus                       Connected
 *  100.67.176.75
 *  <the last error, click to dismiss>
 *  [ Disconnect ]  [ Log in ]
 *  Exit node   [ none ] [ nas ] [ vps-ams ]
 *
 * the card and its height
 */
fn tailscale_card(t: &Theme, vpn: &Vpn, status: &Tailscale) -> (Rectangle, f32) {
    let connected = status.connected();

    let top = Row::new(children![
        ui::line(
            CARD_TITLE,
            Text::new(tailscale_title(status))
                .size(ui::TEXT_BODY)
                .weight(Weight::Bold)
                .color(t.fg)
                .elide()
        ),
        Text::new(tailscale_state(status, vpn.busy()))
            .size(ui::TEXT_SMALL)
            .color(t.fg_dim),
    ])
    .gap(GAP)
    .align(Center)
    .height(CARD_TITLE);

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(top)];
    let mut height = PADDING * 2.0 + CARD_TITLE;

    if connected {
        height += GAP + CARD_LINE;

        parts.push(Box::new(card_line(status.ip.clone(), t.comment)));
    }

    if let Some(error) = vpn.error() {
        height += GAP + CARD_LINE;

        parts.push(Box::new(
            card_line(String::from(error), t.accent)
                .cursor(Pointer)
                .on_click(|_| Vpn::dismiss_error()),
        ));
    }

    height += GAP + BUTTON;

    parts.push(Box::new(tailscale_actions(t, status)));

    if connected {
        let (column, column_height) = exit_nodes(t, vpn);

        height += GAP + column_height;

        parts.push(Box::new(column));
    }

    (ui::card(t, height, parts), height)
}

// before logging in there's no name yet
fn tailscale_title(status: &Tailscale) -> String {
    if status.name.is_empty() {
        String::from("Tailscale")
    } else {
        format!("Tailscale@{}", status.name)
    }
}

fn tailscale_state(status: &Tailscale, busy: bool) -> &'static str {
    if busy {
        return "Working…";
    }

    match status.backend {
        Backend::Running => "Connected",
        Backend::Stopped => "Disconnected",
        Backend::NeedsLogin => "Log in needed",
        Backend::Starting => "Starting…",
        Backend::Other(_) => "",
    }
}

// connect or disconnect, and the way to log in while that's needed
fn tailscale_actions(t: &Theme, status: &Tailscale) -> Row {
    let label = if status.active() {
        "Disconnect"
    } else {
        "Connect"
    };

    let mut actions: Vec<Box<dyn Widget>> = vec![Box::new(ui::key_text_button(
        t,
        label,
        ACTION_WIDTH,
        KEY_TAILSCALE,
        |_| Vpn::toggle_tailscale(),
    ))];

    // tailscale hands out a login page instead of asking for a password
    if needs_login(status) {
        actions.push(Box::new(ui::key_text_button(
            t,
            "Log in",
            ACTION_WIDTH,
            KEY_LOG_IN,
            |_| Vpn::log_in(),
        )));
    }

    Row::new(actions).gap(GAP).height(BUTTON)
}

pub(super) fn needs_login(status: &Tailscale) -> bool {
    status.backend == Backend::NeedsLogin && !status.auth_url.is_empty()
}

fn card_line(text: String, color: Color) -> Rectangle {
    ui::line(
        CARD_LINE,
        Text::new(text).size(ui::TEXT_SMALL).color(color).elide(),
    )
}

/*
 *  Exit node
 *  [ None                              ]
 *  [ Suggested · nl-ams-wg-303         ]
 *  [ In use · se-sto-wg-005            ]   only when that isn't the suggestion
 *
 * the tailnet can offer hundreds of exit nodes (mullvad's), so instead of a list
 * there's what tailscale suggests, and the one in use; that one in the accent
 */
fn exit_nodes(t: &Theme, vpn: &Vpn) -> (Column, f32) {
    let in_use = vpn.exit_node().map(String::from);

    let label = Rectangle::new()
        .width(Parent)
        .height(CARD_LINE)
        .align_child(Start, Center)
        .child(Text::new("Exit node").size(ui::TEXT_SMALL).color(t.fg_dim));

    let mut parts: Vec<Box<dyn Widget>> = vec![
        Box::new(label),
        Box::new(exit_button(
            t,
            String::from("None"),
            in_use.is_none(),
            Some(None),
        )),
    ];

    let suggested = match vpn.suggestion() {
        Suggestion::Node(node) => Some(node.clone()),
        _ => None,
    };

    match vpn.suggestion() {
        Suggestion::Unknown => parts.push(Box::new(exit_button(
            t,
            String::from("Suggested · …"),
            false,
            None,
        ))),

        Suggestion::Node(node) => {
            let chosen = in_use.as_deref() == Some(node.as_str());

            parts.push(Box::new(exit_button(
                t,
                format!("Suggested · {}", tailscale::short_name(node)),
                chosen,
                Some(Some(node.clone())),
            )));
        }

        // nothing to suggest, like when no exit node is allowed
        Suggestion::None => {}
    }

    // what carries the traffic is always on the card, even when it wasn't the suggestion
    if let Some(node) = &in_use
        && suggested.as_deref() != Some(node.as_str())
    {
        parts.push(Box::new(exit_button(
            t,
            format!("In use · {}", tailscale::short_name(node)),
            true,
            None,
        )));
    }

    let buttons = parts.len() - 1;

    let height = CARD_LINE + buttons as f32 * (GAP + BUTTON);

    (Column::new(parts).gap(GAP), height)
}

/*
 * a full-width exit node button; target is what clicking sets, Some(None) for
 * no exit node, None for a button that does nothing (the one in use, or a
 * suggestion still loading)
 */
fn exit_button(
    t: &Theme,
    label: String,
    chosen: bool,
    target: Option<Option<String>>,
) -> Rectangle {
    let (fill, color) = if chosen {
        (t.accent, t.on_accent)
    } else {
        (t.bg2, t.fg)
    };

    let button = Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .fill(fill)
        .padding(SIDES)
        .align_child(Start, Center)
        .child(Text::new(label).size(ui::TEXT_NOTE).color(color).elide());

    match target {
        Some(node) if !chosen => button
            .cursor(Pointer)
            .on_click(move |_| Vpn::use_exit_node(node.clone())),
        _ => button,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(backend: Backend, name: &str) -> Tailscale {
        Tailscale {
            backend,
            name: String::from(name),
            ip: String::new(),
            exit_ip: None,
            auth_url: String::new(),
        }
    }

    #[test]
    fn the_card_is_named_after_the_machine_once_known() {
        assert_eq!(
            tailscale_title(&status(Backend::NeedsLogin, "")),
            "Tailscale"
        );
        assert_eq!(
            tailscale_title(&status(Backend::Running, "prometheus")),
            "Tailscale@prometheus"
        );
    }

    #[test]
    fn the_state_says_working_while_a_command_runs() {
        let running = status(Backend::Running, "prometheus");

        assert_eq!(tailscale_state(&running, false), "Connected");
        assert_eq!(tailscale_state(&running, true), "Working…");
        assert_eq!(
            tailscale_state(&status(Backend::NeedsLogin, ""), false),
            "Log in needed"
        );
    }
}
