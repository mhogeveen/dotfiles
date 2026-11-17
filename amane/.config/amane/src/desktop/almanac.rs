/*
 * almanac: the whole year as one sheet, a row of day cells per month. days
 * gone are filled, weekends a step lighter so the weeks show as stripes, and
 * today is the one cell in the accent. the time and the date sit to its left,
 * on the sheet's top edge
 */

use amane::{
    Align::{Center, End},
    Color, Column, Rectangle, Row, Text, Widget, children,
};

use super::{Moment, Page, date_line, placed};
use crate::calendar::{self, Date};
use crate::theme::Theme;

const MONTHS: u32 = 12;
const LONGEST_MONTH: u32 = 31;
const MONTH_LETTERS: usize = 3;

// Saturday and Sunday, as Date::weekday counts them
const FIRST_WEEKEND_DAY: u32 = 5;

// the sheet: where it starts and how wide its cells run, as fractions of the width
const SHEET_LEFT: f32 = 0.42;
const SHEET_WIDTH: f32 = 0.46;

// the time and date: their left edge as a fraction of the width
const TEXT_LEFT: f32 = 0.1;

// text sizes as fractions of the page's height
const TIME_SIZE: f32 = 0.1;
const LINE_SIZE: f32 = 0.016;
const LABEL_SIZE: f32 = 0.011;

// at the reference height: between cells, between the labels and the sheet, between lines
const CELL_GAP: f32 = 4.0;
const LABEL_GAP: f32 = 12.0;
const LINE_GAP: f32 = 10.0;

// a month label is this many characters wide, plus room
const LABEL_WIDTH: f32 = 4.0;
const CHARACTER_WIDTH: f32 = 0.6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Gone,
    GoneWeekend,
    Today,
    Ahead,
    AheadWeekend,
}

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    let gap = page.scaled(CELL_GAP);
    let pitch = SHEET_WIDTH * page.width / LONGEST_MONTH as f32;
    let cell = pitch - gap;

    let label_size = LABEL_SIZE * page.height;
    let label_width = LABEL_WIDTH * label_size * CHARACTER_WIDTH;
    let label_gap = page.scaled(LABEL_GAP);

    let today = moment.date;

    let rows: Vec<Box<dyn Widget>> = (1..=MONTHS)
        .map(|month| -> Box<dyn Widget> {
            let name: String = calendar::MONTHS[month as usize - 1]
                .chars()
                .take(MONTH_LETTERS)
                .collect();

            // this month's name stands out a little, the accent stays with today
            let label_color = if month == today.month {
                t.fg
            } else {
                t.comment
            };

            let label = Rectangle::new()
                .width(label_width)
                .height(cell)
                .align_child(End, Center)
                .child(Text::new(name).size(label_size).color(label_color));

            let cells: Vec<Box<dyn Widget>> = (1..=month_length(today.year, month))
                .map(|day| -> Box<dyn Widget> {
                    let state = cell_state(Date::new(today.year, month, day), today);

                    Box::new(
                        Rectangle::new()
                            .width(cell)
                            .height(cell)
                            .fill(cell_color(t, state)),
                    )
                })
                .collect();

            Box::new(Row::new(children![label, Row::new(cells).gap(gap)]).gap(label_gap))
        })
        .collect();

    let sheet_height = MONTHS as f32 * pitch - gap;
    let top = page.top + (page.body_height() - sheet_height) / 2.0;

    let sheet = Column::new(rows).gap(gap);

    let line_size = LINE_SIZE * page.height;
    let year_length = Date::new(today.year, 12, 31).day_of_year();

    let text = Column::new(children![
        Text::new(moment.time.clone())
            .size(TIME_SIZE * page.height)
            .color(t.fg),
        Text::new(date_line(today)).size(line_size).color(t.fg_dim),
        Text::new(format!(
            "day {} · {} to go",
            today.day_of_year(),
            year_length - today.day_of_year()
        ))
        .size(line_size)
        .color(t.comment)
    ])
    .gap(page.scaled(LINE_GAP));

    children![
        placed(
            SHEET_LEFT * page.width - label_width - label_gap,
            top,
            sheet
        ),
        placed(TEXT_LEFT * page.width, top, text)
    ]
}

fn cell_color(t: &Theme, cell: Cell) -> Color {
    match cell {
        Cell::Gone => t.bg3,
        Cell::GoneWeekend => t.bg4,
        Cell::Today => t.accent,
        Cell::Ahead => t.bg1,
        Cell::AheadWeekend => t.bg2,
    }
}

fn cell_state(date: Date, today: Date) -> Cell {
    let weekend = date.weekday() >= FIRST_WEEKEND_DAY;

    match (date.day_of_year().cmp(&today.day_of_year()), weekend) {
        (std::cmp::Ordering::Equal, _) => Cell::Today,
        (std::cmp::Ordering::Less, false) => Cell::Gone,
        (std::cmp::Ordering::Less, true) => Cell::GoneWeekend,
        (std::cmp::Ordering::Greater, false) => Cell::Ahead,
        (std::cmp::Ordering::Greater, true) => Cell::AheadWeekend,
    }
}

// the number of days in a month: the distance to the first of the next
fn month_length(year: i32, month: u32) -> u32 {
    if month == 12 {
        return LONGEST_MONTH;
    }

    Date::new(year, month + 1, 1).day_of_year() - Date::new(year, month, 1).day_of_year()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_have_their_lengths() {
        assert_eq!(month_length(2026, 1), 31);
        assert_eq!(month_length(2026, 2), 28);
        assert_eq!(month_length(2024, 2), 29);
        assert_eq!(month_length(2026, 11), 30);
        assert_eq!(month_length(2026, 12), 31);

        let year: u32 = (1..=12).map(|month| month_length(2026, month)).sum();

        assert_eq!(year, 365);
    }

    #[test]
    fn cells_know_their_place() {
        // Monday 16 November 2026
        let today = Date::new(2026, 11, 16);

        assert_eq!(cell_state(today, today), Cell::Today);
        assert_eq!(cell_state(Date::new(2026, 11, 13), today), Cell::Gone);
        assert_eq!(
            cell_state(Date::new(2026, 11, 15), today),
            Cell::GoneWeekend
        );
        assert_eq!(cell_state(Date::new(2026, 11, 17), today), Cell::Ahead);
        assert_eq!(
            cell_state(Date::new(2026, 11, 21), today),
            Cell::AheadWeekend
        );
    }
}
