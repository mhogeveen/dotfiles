/*
 * prompt: the desktop as a terminal session, set in the shell's own font. a
 * prompt line, `$ now` with the time, date, week, weather and uptime as a
 * table, `$ left` with what's left of today, the week and the year, and an
 * accent block cursor waiting on the last line
 *
 * the user comes from $USER, the machine's name and the uptime from /proc,
 * read as the page draws (once a minute, a few bytes each)
 */

use std::env;
use std::fs;

use amane::{Column, Rectangle, Row, Text, Widget, children};

use super::{Moment, Page, date_line, placed, text_width};
use crate::calendar::Date;
use crate::theme::Theme;

const HOSTNAME: &str = "/proc/sys/kernel/hostname";
const UPTIME: &str = "/proc/uptime";

const MINUTES_PER_HOUR: u32 = 60;
const MINUTES_PER_DAY: u32 = 24 * MINUTES_PER_HOUR;
const SECONDS_PER_MINUTE: f64 = 60.0;

// Saturday, as Date::weekday counts
const SATURDAY: u32 = 5;

// where the session starts, as fractions of the width and of the height below the bar
const LEFT: f32 = 0.12;
const TOP: f32 = 0.22;

// the text size as a fraction of the page's height, and the room per line in text sizes
const TEXT_SIZE: f32 = 0.022;
const LINE_GAP: f32 = 0.45;

// the table: indented by this many characters, its keys this many wide
const INDENT: usize = 2;
const KEY_WIDTH: usize = 10;

// an empty line between commands, in text sizes
const COMMAND_GAP: f32 = 0.8;

// the cursor, in text sizes
const CURSOR_HEIGHT: f32 = 1.2;

const PROMPT: &str = "$ ";

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    let size = TEXT_SIZE * page.height;

    let user = env::var("USER").unwrap_or_else(|_| String::from("you"));
    let host = read_trimmed(HOSTNAME).unwrap_or_else(|| String::from("here"));
    let uptime = read_trimmed(UPTIME).and_then(|text| uptime_minutes(&text));

    let date = moment.date;
    let days_in_year = Date::new(date.year, 12, 31).day_of_year();

    let mut now = vec![
        ("time", moment.time.clone()),
        ("date", date_line(date)),
        (
            "week",
            format!(
                "{} · day {} of {days_in_year}",
                date.iso_week(),
                date.day_of_year()
            ),
        ),
    ];

    if let Some(weather) = &moment.weather {
        now.push(("weather", weather.clone()));
    }

    if let Some(minutes) = uptime {
        now.push(("uptime", duration(minutes)));
    }

    let minute_of_day = moment.hour * MINUTES_PER_HOUR + moment.minute;

    let left = vec![
        ("today", duration(MINUTES_PER_DAY - minute_of_day)),
        ("week", until_weekend(date.weekday())),
        ("year", plural(days_in_year - date.day_of_year(), "day")),
    ];

    let mut lines: Vec<Box<dyn Widget>> = children![prompt_line(t, size, &user, &host)];

    lines.push(Box::new(command(t, size, "now")));
    lines.extend(table(t, size, &now));
    lines.push(Box::new(blank(size)));

    lines.push(Box::new(command(t, size, "left")));
    lines.extend(table(t, size, &left));
    lines.push(Box::new(blank(size)));

    // the prompt waiting for the next command
    lines.push(Box::new(
        Row::new(children![
            Text::new(PROMPT).size(size).color(t.accent),
            Rectangle::new()
                .width(text_width(" ", size))
                .height(CURSOR_HEIGHT * size)
                .fill(t.accent)
        ])
        .align(amane::Align::Center),
    ));

    children![placed(
        LEFT * page.width,
        page.top + TOP * page.body_height(),
        Column::new(lines).gap(LINE_GAP * size)
    )]
}

// like maarten@prometheus ~
fn prompt_line(t: &Theme, size: f32, user: &str, host: &str) -> Row {
    Row::new(children![
        Text::new(user.to_string()).size(size).color(t.fg),
        Text::new(format!("@{host}")).size(size).color(t.comment),
        Text::new(" ~").size(size).color(t.fg_dim)
    ])
}

// like $ now
fn command(t: &Theme, size: f32, name: &str) -> Row {
    Row::new(children![
        Text::new(PROMPT).size(size).color(t.accent),
        Text::new(name.to_string()).size(size).color(t.fg)
    ])
}

// keys in a column of their own, values after them
fn table(t: &Theme, size: f32, rows: &[(&str, String)]) -> Vec<Box<dyn Widget>> {
    rows.iter()
        .map(|(key, value)| -> Box<dyn Widget> {
            Box::new(Row::new(children![
                Text::new(format!("{:INDENT$}{key:KEY_WIDTH$}", ""))
                    .size(size)
                    .color(t.comment),
                Text::new(value.clone()).size(size).color(t.fg_dim)
            ]))
        })
        .collect()
}

fn blank(size: f32) -> Rectangle {
    Rectangle::new().width(1.0).height(COMMAND_GAP * size)
}

fn read_trimmed(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

// /proc/uptime's first number is seconds since boot
fn uptime_minutes(text: &str) -> Option<u32> {
    let seconds: f64 = text.split_whitespace().next()?.parse().ok()?;

    Some((seconds / SECONDS_PER_MINUTE) as u32)
}

// like 45m, 3h 12m or 2d 4h
fn duration(minutes: u32) -> String {
    let days = minutes / MINUTES_PER_DAY;
    let hours = minutes % MINUTES_PER_DAY / MINUTES_PER_HOUR;
    let rest = minutes % MINUTES_PER_HOUR;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {rest}m")
    } else {
        format!("{rest}m")
    }
}

// days until Saturday, or that it's the weekend
fn until_weekend(weekday: u32) -> String {
    if weekday >= SATURDAY {
        return String::from("it's the weekend");
    }

    format!("{} to the weekend", plural(SATURDAY - weekday, "day"))
}

fn plural(count: u32, word: &str) -> String {
    if count == 1 {
        format!("{count} {word}")
    } else {
        format!("{count} {word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_read_short() {
        assert_eq!(duration(45), "45m");
        assert_eq!(duration(192), "3h 12m");
        assert_eq!(duration(MINUTES_PER_DAY * 2 + 250), "2d 4h");
        assert_eq!(duration(0), "0m");
    }

    #[test]
    fn uptime_is_read_from_proc() {
        assert_eq!(uptime_minutes("11520.42 40000.10"), Some(192));
        assert_eq!(uptime_minutes(""), None);
        assert_eq!(uptime_minutes("soon"), None);
    }

    #[test]
    fn the_weekend_comes_closer() {
        assert_eq!(until_weekend(0), "5 days to the weekend");
        assert_eq!(until_weekend(4), "1 day to the weekend");
        assert_eq!(until_weekend(5), "it's the weekend");
        assert_eq!(until_weekend(6), "it's the weekend");
    }
}
