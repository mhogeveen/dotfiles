/*
 * notifications: amane's Notifications service is the daemon, this module decides
 * how they're shown; popups in the top-right corner, a history in the notification
 * center panel (panels/notifications.rs) and an unread count in the bar
 */

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use amane::{Apps, Argument, Bus, Notification, Notifications, Service, Urgency};

use crate::panels::{NOTIFICATIONS, Panels};

pub mod card;
pub mod popups;

// when the notification center was last closed; anything newer is unread
pub struct Seen {
    until: SystemTime,
}

impl Service for Seen {
    fn new() -> Self {
        Self {
            until: SystemTime::now(),
        }
    }

    // only changes when the center closes
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

impl Seen {
    pub fn unread(&self, notifications: &Notifications) -> usize {
        notifications
            .list()
            .iter()
            .filter(|notification| notification.received() > self.until)
            .count()
    }

    pub fn mark_all() {
        Seen::write().until = SystemTime::now();
    }
}

/*
 * whether anything arrived since the notification center was last closed;
 * nothing counts while it's open. reads Notifications, so a view that asks
 * redraws when one arrives or goes away
 */
pub fn has_unread() -> bool {
    let notifications = Notifications::read();

    if Panels::read().is_open(NOTIFICATIONS) {
        return false;
    }

    Seen::read().unread(&notifications) > 0
}

// nerd font material design glyphs
pub const ICON_BELL: &str = "\u{f009a}";
pub const ICON_BELL_OUTLINE: &str = "\u{f009c}";

/*
 * sends a notification the way any program would, over d-bus, so it reaches the
 * shell's own daemon (or whichever one runs) and lands in the history like the rest.
 * it waits for the daemon's answer, so call it from a thread, never from a view or
 * an input handler: the daemon answers on its own thread, but a view would hold up
 * drawing while it does
 */
pub fn send(app_name: &str, summary: &str, body: &str, urgency: Urgency, image: Option<&Path>) {
    let level = match urgency {
        Urgency::Low => 0,
        Urgency::Normal => 1,
        Urgency::Critical => 2,
    };

    let mut hints = BTreeMap::new();

    // the spec sends this as a byte, which Argument has no kind for; amane reads any number
    hints.insert(String::from("urgency"), Argument::Unsigned(level));

    if let Some(image) = image {
        hints.insert(
            String::from("image-path"),
            Argument::from(image.to_string_lossy().into_owned()),
        );
    }

    let arguments = [
        Argument::from(app_name),
        // replaces no earlier notification
        Argument::from(0u32),
        // no icon
        Argument::from(""),
        Argument::from(summary),
        Argument::from(body),
        // no action buttons
        Argument::TextList(Vec::new()),
        Argument::Map(hints),
        // let the daemon pick the timeout
        Argument::from(-1i32),
    ];

    Bus::session().call(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
        &arguments,
    );
}

/*
 * the picture to show: the notification's own image, then its icon as a file,
 * then the icon of the installed app it names or comes from
 */
pub fn icon_path(notification: &Notification, apps: &Apps) -> Option<PathBuf> {
    let file = |value: &str| {
        let path = PathBuf::from(value.strip_prefix("file://").unwrap_or(value));

        (path.is_absolute() && path.exists()).then_some(path)
    };

    if let Some(path) = file(notification.image()) {
        return Some(path);
    }

    if let Some(path) = file(notification.icon()) {
        return Some(path);
    }

    let icon = notification.icon();
    let sender = notification.app_name();

    apps.list()
        .iter()
        .find(|app| {
            (!icon.is_empty() && app.icon() == Some(icon))
                || (!sender.is_empty() && app.name().eq_ignore_ascii_case(sender))
        })
        .and_then(|app| app.icon_path())
        .map(PathBuf::from)
}

// bodies may hold simple markup like <b> and &amp;, shown here as plain text
pub fn plain(body: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;

    for character in body.chars() {
        match character {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }

    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

pub fn time_ago(received: SystemTime, now: SystemTime) -> String {
    let seconds = now.duration_since(received).unwrap_or_default().as_secs();

    match seconds {
        0..60 => String::from("now"),
        60..3600 => format!("{}m ago", seconds / 60),
        3600..86400 => format!("{}h ago", seconds / 3600),
        _ => format!("{}d ago", seconds / 86400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_becomes_plain_text() {
        assert_eq!(plain("<b>bold</b> &amp; <i>more</i>"), "bold & more");
        assert_eq!(plain("a &lt;tag&gt;"), "a <tag>");
        assert_eq!(plain("&amp;lt;"), "&lt;");
    }

    #[test]
    fn times_round_down() {
        let now = SystemTime::now();

        assert_eq!(time_ago(now, now), "now");
        assert_eq!(time_ago(now - Duration::from_secs(150), now), "2m ago");
        assert_eq!(time_ago(now - Duration::from_secs(7300), now), "2h ago");
    }
}
