/*
 * one notification, shared by the popups and the notification center:
 *
 *   [icon] app name                 5m ago  ×
 *   summary
 *   body, wrapped to a few lines…
 *   [ action ] [ action ]
 *
 * every part has a height known up front, so the popup window can make exactly
 * the cards clickable and let clicks through everywhere else
 */

use std::path::PathBuf;
use std::time::SystemTime;

use amane::{
    Action,
    Align::{Center, Start},
    Apps, Button, Column,
    Cursor::Pointer,
    Image, Notification, Notifications, Padding, Rectangle, Row,
    Size::Parent,
    Text, Urgency, Weight, Widget,
};

use crate::theme::Theme;

use super::history::Kept;
use super::{icon_path, plain, time_ago};

const PADDING: f32 = 12.0;
const GAP: f32 = 6.0;
const HEADER: f32 = 20.0;
const ACTIONS: f32 = 28.0;

pub struct Options {
    pub width: f32,

    // the body is wrapped to at most this many lines, then cut off
    pub body_lines: usize,

    // shows "5m ago" in the header when set
    pub now: Option<SystemTime>,
}

/*
 * what a card shows, from a live notification or one restored after a restart
 * (src/notifications/history.rs)
 */
pub struct Content<'a> {
    app_name: &'a str,
    summary: &'a str,
    body: &'a str,
    icon: Option<PathBuf>,
    urgency: Urgency,
    received: SystemTime,

    // the live notification's id and buttons; a restored one has neither
    live: Option<(u32, &'a [Action])>,
}

impl<'a> Content<'a> {
    pub fn live(notification: &'a Notification, apps: &Apps) -> Self {
        Self {
            app_name: notification.app_name(),
            summary: notification.summary(),
            body: notification.body(),
            icon: icon_path(
                notification.image(),
                notification.icon(),
                notification.app_name(),
                apps,
            ),
            urgency: notification.urgency(),
            received: notification.received(),
            live: Some((notification.id(), notification.actions())),
        }
    }

    // its sender can't be reached any more: no buttons, and clicking it does nothing
    pub fn kept(kept: &'a Kept, apps: &Apps) -> Self {
        Self {
            app_name: &kept.app_name,
            summary: &kept.summary,
            body: &kept.body,
            icon: icon_path(&kept.image, &kept.icon, &kept.app_name, apps),
            urgency: kept.urgency,
            received: kept.received,
            live: None,
        }
    }
}

// the card and its height
pub fn card(
    t: &Theme,
    content: Content,
    options: Options,
    on_close: impl Fn(Button) + 'static,
) -> (Rectangle, f32) {
    let inner = options.width - PADDING * 2.0;

    let mut parts: Vec<Box<dyn Widget>> = Vec::new();
    let mut height = PADDING * 2.0 + HEADER;

    parts.push(Box::new(header(t, &content, options.now, on_close)));

    let summary = content.summary.trim();

    let summary = (!summary.is_empty()).then(|| {
        Text::new(summary)
            .size(14.0)
            .weight(Weight::Bold)
            .color(t.fg)
            .elide()
    });

    let body = plain(content.body);
    let body = body.trim();

    let body = (!body.is_empty()).then(|| {
        Text::new(body)
            .size(13.0)
            .color(t.fg_dim)
            .wrap()
            .max_lines(options.body_lines)
            .elide()
    });

    // an empty summary or body gets no row at all
    for text in [summary, body].into_iter().flatten() {
        let text_height = text.height_in(inner);

        height += GAP + text_height;

        // as tall as the lines actually used, not the line limit the text reserves
        parts.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(text_height)
                .clip()
                .child(text),
        ));
    }

    if let Some((id, buttons)) = content.live
        && !buttons.is_empty()
    {
        height += GAP + ACTIONS;

        parts.push(Box::new(actions(t, id, buttons)));
    }

    let border = match content.urgency {
        Urgency::Critical => t.accent,
        _ => t.bg3,
    };

    let card = Rectangle::new()
        .width(options.width)
        .height(height)
        .fill(t.bg0)
        .border(1.0, border)
        .padding(PADDING)
        .clip();

    // only a live one does something when clicked
    let card = match content.live {
        Some((id, _)) => card
            .cursor(Pointer)
            .on_click(move |_| Notifications::click(id)),
        None => card,
    };

    (card.child(Column::new(parts).gap(GAP)), height)
}

fn header(
    t: &Theme,
    content: &Content,
    now: Option<SystemTime>,
    on_close: impl Fn(Button) + 'static,
) -> Row {
    let icon = Rectangle::new().width(HEADER).height(HEADER);

    let icon = match &content.icon {
        Some(path) => icon.fill(Image::contain(path.clone())),
        None => icon
            .align_child(Center, Center)
            .child(Text::new("\u{f009a}").size(14.0).color(t.fg_dim)),
    };

    let sender = match content.app_name {
        "" => "Notification",
        name => name,
    };

    let sender = Rectangle::new()
        .width(Parent)
        .height(HEADER)
        .align_child(Start, Center)
        .child(Text::new(sender).size(12.0).color(t.fg_dim).elide());

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(icon), Box::new(sender)];

    if let Some(now) = now {
        parts.push(Box::new(
            Text::new(time_ago(content.received, now))
                .size(12.0)
                .color(t.comment),
        ));
    }

    parts.push(Box::new(
        Rectangle::new()
            .width(HEADER)
            .height(HEADER)
            .align_child(Center, Center)
            .cursor(Pointer)
            .on_click(on_close)
            .child(Text::new("\u{f0156}").size(14.0).color(t.fg_dim)),
    ));

    Row::new(parts).gap(8.0).align(Center).height(HEADER)
}

// the buttons share the row evenly
fn actions(t: &Theme, id: u32, buttons: &[Action]) -> Row {
    let mut row: Vec<Box<dyn Widget>> = Vec::new();

    for action in buttons {
        let key = String::from(action.key());

        row.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(ACTIONS)
                .fill(t.bg2)
                .padding(Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 8.0,
                })
                .align_child(Center, Center)
                .cursor(Pointer)
                .on_click(move |_| Notifications::invoke(id, &key))
                .child(Text::new(action.label()).size(13.0).color(t.fg).elide()),
        ));
    }

    Row::new(row).gap(GAP).height(ACTIONS)
}

// a placeholder card-sized message, for the center's empty state
pub fn message(t: &Theme, width: f32, text: &str) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(PADDING * 2.0 + HEADER)
        .align_child(Center, Center)
        .child(Text::new(text).size(13.0).color(t.comment))
}
