/*
 * ledger: the day, measured. left of centre, on one edge: the time, an accent
 * rule under the hour, a line with the date, the week, the day of the year and
 * the weather, and a strip of 24 hour cells that fill with the accent as the
 * day goes (the one in progress fills by the minute)
 */

use amane::{
    Align::{End, Start},
    Color, Column, Rectangle, Row, Stack, Text, Widget, children,
};

use super::{Moment, Page, date_line, placed, text_width};
use crate::theme::Theme;

const HOURS: u32 = 24;
const MINUTES_PER_HOUR: f32 = 60.0;

// a label under the strip every this many hours: 00 06 12 18, and 24 at the end
const LABEL_EVERY: u32 = 6;

// the column: its left edge as a fraction of the width, its top of the height below the bar
const COLUMN_LEFT: f32 = 0.12;
const COLUMN_TOP: f32 = 0.36;

/*
 * text sizes as fractions of the page's height. amane keeps every letter in
 * one 1024 px texture and leaves out what doesn't fit, so the time stays well
 * under that (a three-letter numeral at 0.42 lost its last letter)
 */
const TIME_SIZE: f32 = 0.13;
const LINE_SIZE: f32 = 0.016;
const LABEL_SIZE: f32 = 0.009;

// the accent rule under the hour, at the reference height
const RULE_THICKNESS: f32 = 3.0;

// space between the parts of the column, and between the parts of the date line, in text sizes
const COLUMN_GAP: f32 = 1.2;
const LINE_GAP: f32 = 3.0;

// the strip: a fraction of the width, its cells a fraction of the height; the rest at the reference height
const STRIP_WIDTH: f32 = 0.5;
const CELL_HEIGHT: f32 = 0.035;
const CELL_GAP: f32 = 4.0;
const CELL_BORDER: f32 = 1.0;
const LABEL_ROOM: f32 = 6.0;

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    children![placed(
        COLUMN_LEFT * page.width,
        page.top + COLUMN_TOP * page.body_height(),
        column(page, moment, t),
    )]
}

// the time, its rule, the date line and the hour strip, on one left edge
fn column(page: Page, moment: &Moment, t: &Theme) -> Column {
    let time_size = TIME_SIZE * page.height;
    let line_size = LINE_SIZE * page.height;

    // as wide as the hour, the part of the time that changes least
    let time = moment.time.as_str();
    let hour_text = time.split(':').next().unwrap_or(time);

    let rule = Rectangle::new()
        .width(text_width(hour_text, time_size))
        .height(page.scaled(RULE_THICKNESS))
        .fill(t.accent);

    let date = moment.date;

    let mut line: Vec<Box<dyn Widget>> = children![
        Text::new(date_line(date)).size(line_size).color(t.fg_dim),
        Text::new(format!(
            "week {} · day {}",
            date.iso_week(),
            date.day_of_year()
        ))
        .size(line_size)
        .color(t.comment)
    ];

    // nothing until the first forecast is in
    if let Some(weather) = &moment.weather {
        line.push(Box::new(
            Text::new(weather.clone()).size(line_size).color(t.fg_dim),
        ));
    }

    let line = Row::new(line).gap(LINE_GAP * line_size);

    Column::new(children![
        Text::new(time).size(time_size).color(t.fg),
        rule,
        line,
        hour_strip(page, t, moment.hour, moment.minute)
    ])
    .gap(COLUMN_GAP * line_size)
}

// 24 cells, past hours in the accent, the hour in progress filled by its minutes, labels under them
fn hour_strip(page: Page, t: &Theme, hour: u32, minute: u32) -> Column {
    let gap = page.scaled(CELL_GAP);
    let border = page.scaled(CELL_BORDER);

    let strip_width = STRIP_WIDTH * page.width;
    let cell_width = (strip_width - gap * (HOURS - 1) as f32) / HOURS as f32;
    let cell_height = CELL_HEIGHT * page.height;

    let cells: Vec<Box<dyn Widget>> = (0..HOURS)
        .map(|cell| -> Box<dyn Widget> {
            let fill = hour_fill(cell, hour, minute);

            let empty = Rectangle::new()
                .width(cell_width)
                .height(cell_height)
                .fill(t.bg1)
                .border(border, t.bg3);

            if fill >= 1.0 {
                Box::new(
                    Rectangle::new()
                        .width(cell_width)
                        .height(cell_height)
                        .fill(t.accent),
                )
            } else if fill > 0.0 {
                Box::new(
                    empty.align_child(Start, Start).child(
                        Rectangle::new()
                            .width(cell_width * fill)
                            .height(cell_height)
                            .fill(t.accent),
                    ),
                )
            } else {
                Box::new(empty)
            }
        })
        .collect();

    let label_size = LABEL_SIZE * page.height;

    // each label starts where its hour's cell does
    let every = (cell_width + gap) * LABEL_EVERY as f32;

    let starts: Vec<Box<dyn Widget>> = (0..HOURS / LABEL_EVERY)
        .map(|index| -> Box<dyn Widget> {
            Box::new(
                Rectangle::new()
                    .width(every)
                    .height(label_size * 2.0)
                    .align_child(Start, Start)
                    .child(label(index * LABEL_EVERY, label_size, t.comment)),
            )
        })
        .collect();

    let end = Rectangle::new()
        .width(strip_width)
        .height(label_size * 2.0)
        .align_child(End, Start)
        .child(label(HOURS, label_size, t.comment));

    Column::new(children![
        Row::new(cells).gap(gap),
        Stack::new(children![Row::new(starts), end])
    ])
    .gap(page.scaled(LABEL_ROOM))
}

fn label(hour: u32, size: f32, color: Color) -> Text {
    Text::new(format!("{hour:02}")).size(size).color(color)
}

// how full an hour's cell is: 1 for a past hour, part for the one in progress, 0 for the rest
fn hour_fill(cell: u32, hour: u32, minute: u32) -> f32 {
    if cell < hour {
        1.0
    } else if cell == hour {
        minute as f32 / MINUTES_PER_HOUR
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hours_fill_as_the_day_goes() {
        assert_eq!(hour_fill(0, 17, 5), 1.0);
        assert_eq!(hour_fill(16, 17, 5), 1.0);
        assert_eq!(hour_fill(17, 17, 30), 0.5);
        assert_eq!(hour_fill(17, 17, 0), 0.0);
        assert_eq!(hour_fill(18, 17, 59), 0.0);
        assert_eq!(hour_fill(23, 23, 45), 0.75);
    }
}
