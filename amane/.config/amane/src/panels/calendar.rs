/*
 * calendar panel, dropping down from the clock: a month with ISO week numbers,
 * weeks starting on Monday and today in the accent; h and l or the arrow
 * buttons move a month, t or clicking the month name goes back to today
 *
 * the content is a list of sections, only the calendar for now, so the weather
 * can go underneath it later
 */

use std::time::Duration;

use amane::{
    Align::Center, Column, Cursor::Pointer, Key, LayerWindow, Margin, Monitor, Rectangle, Row,
    Service, Size::Parent, Text, Vertical, Weight, Widget, children,
};

use super::ui::{self, BUTTON, PADDING};
use super::{CALENDAR, hints, panel};
use crate::calendar::{self, Date, Day, MONTHS, WEEKDAYS};
use crate::clock::Clock;
use crate::theme::Theme;

const WEEK: f32 = 28.0;

// an empty column between the week numbers and the days
const SPACER: f32 = 18.0;
const CELL_WIDTH: f32 = 36.0;
const CELL_HEIGHT: f32 = 32.0;
const CELL_GAP: f32 = 4.0;
const HEADER_GAP: f32 = 8.0;
const WEEKDAY_ROW: f32 = 20.0;

// nerd font material design chevrons
const ICON_PREVIOUS: &str = "\u{f0141}";
const ICON_NEXT: &str = "\u{f0142}";

const KEY_PREVIOUS: char = 'h';
const KEY_NEXT: char = 'l';
const KEY_TODAY: char = 't';

// the week column, the spacer and seven days, with a gap after each but the last
const WIDTH: f32 = PADDING * 2.0 + WEEK + CELL_GAP + SPACER + 7.0 * (CELL_WIDTH + CELL_GAP);

// header, weekday names and six weeks
const HEIGHT: f32 = PADDING * 2.0
    + BUTTON
    + HEADER_GAP
    + WEEKDAY_ROW
    + CELL_GAP
    + 6.0 * CELL_HEIGHT
    + 5.0 * CELL_GAP;

// which month is shown; None follows today
struct Shown {
    month: Option<(i32, u32)>,
}

impl Service for Shown {
    fn new() -> Self {
        Self { month: None }
    }

    // only changes through input
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    // under the clock: anchored to the top, centered left to right
    let base = LayerWindow::new()
        .width(WIDTH)
        .height(HEIGHT)
        .anchor_vertical(Vertical::Top)
        .margin(Margin {
            top: 8,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .namespace("amane-calendar");

    panel(monitor, CALENDAR, base, key_pressed, content)
}

fn content(t: &Theme) -> Rectangle {
    let today = Clock::read().today();

    let (year, month) = Shown::read().month.unwrap_or((today.year, today.month));

    // more sections, like the weather, go after this one
    let sections: Vec<Box<dyn Widget>> = vec![Box::new(calendar_section(t, today, year, month))];

    ui::frame(t, HEADER_GAP, sections)
}

// back to today's month the next time it opens
pub fn reset() {
    Shown::write().month = None;
}

fn key_pressed(key: Key) {
    let Key::Character(letter) = key else {
        return;
    };

    match letter.to_ascii_lowercase() {
        KEY_PREVIOUS => go(-1),
        KEY_NEXT => go(1),
        KEY_TODAY => reset(),
        _ => {}
    }
}

// a month back or forward from the one shown
fn go(by: i32) {
    let today = Clock::read().today();

    let mut shown = Shown::write();

    let (year, month) = shown.month.unwrap_or((today.year, today.month));

    shown.month = Some(calendar::shift(year, month, by));
}

fn calendar_section(t: &Theme, today: Date, year: i32, month: u32) -> Column {
    // while ? hints show, the month name says what its key does: clicking it goes back to today
    let (title, color) = if hints::shown() {
        (format!("'{KEY_TODAY}' jump to today"), t.fg_dim)
    } else {
        (format!("{} {year}", MONTHS[month as usize - 1]), t.fg)
    };

    let title = Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| reset())
        .child(
            Text::new(title)
                .size(15.0)
                .weight(Weight::Bold)
                .color(color),
        );

    let previous = ui::key_button(t, ICON_PREVIOUS, KEY_PREVIOUS, |_| go(-1));
    let next = ui::key_button(t, ICON_NEXT, KEY_NEXT, |_| go(1));

    let header = Row::new(children![previous, title, next])
        .align(Center)
        .height(BUTTON);

    let mut weekdays: Vec<Box<dyn Widget>> = vec![
        Box::new(cell(WEEK, WEEKDAY_ROW)),
        Box::new(cell(SPACER, WEEKDAY_ROW)),
    ];

    for name in WEEKDAYS {
        weekdays.push(Box::new(
            cell(CELL_WIDTH, WEEKDAY_ROW).child(Text::new(name).size(12.0).color(t.comment)),
        ));
    }

    let mut rows: Vec<Box<dyn Widget>> = vec![Box::new(
        Row::new(weekdays).gap(CELL_GAP).height(WEEKDAY_ROW),
    )];

    for week in calendar::grid(year, month) {
        // a week with no day of this month stays empty, its number included
        let any_in_month = week.days.iter().any(|day| day.in_month);

        let number = cell(WEEK, CELL_HEIGHT);

        let number = if any_in_month {
            number.child(
                Text::new(week.number.to_string())
                    .size(11.0)
                    .color(t.comment),
            )
        } else {
            number
        };

        let mut days: Vec<Box<dyn Widget>> =
            vec![Box::new(number), Box::new(cell(SPACER, CELL_HEIGHT))];

        for day in week.days {
            days.push(Box::new(day_cell(t, day, today)));
        }

        rows.push(Box::new(Row::new(days).gap(CELL_GAP).height(CELL_HEIGHT)));
    }

    Column::new(children![header, Column::new(rows).gap(CELL_GAP),]).gap(HEADER_GAP)
}

// a day of the shown month, today in the accent; days around the month are left empty
fn day_cell(t: &Theme, day: Day, today: Date) -> Rectangle {
    let cell = cell(CELL_WIDTH, CELL_HEIGHT);

    if !day.in_month {
        return cell;
    }

    let (fill, color) = if day.date == today {
        (t.accent, t.on_accent)
    } else {
        (t.bg_deep, t.fg)
    };

    cell.fill(fill)
        .child(Text::new(day.date.day.to_string()).size(13.0).color(color))
}

fn cell(width: f32, height: f32) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .align_child(Center, Center)
}
