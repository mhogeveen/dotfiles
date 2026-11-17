/*
 * epoch: the same minute, written other ways. a binary clock (each digit of
 * the time as its bits, lit in the accent) and a table of the moment in other
 * systems: unix time, the julian day, the iso week date and ordinal date,
 * swatch internet time, french decimal time, hexadecimal time, and the time
 * on mars (coordinated mars time and the sol number)
 *
 * all from the instant and the local time; nothing to fetch
 */

use amane::{Align::Center, Column, Rectangle, Row, Text, Widget, children};

use super::{Moment, Page, placed};
use crate::calendar::{self, Date};
use crate::theme::Theme;

const SECONDS_PER_DAY: f64 = 86_400.0;
const MINUTES_PER_DAY: f64 = 1_440.0;

// the julian day of 1970-01-01 00:00 UTC
const UNIX_EPOCH_JULIAN: f64 = 2_440_587.5;

// terrestrial time runs this many seconds ahead of UTC (32.184 + 37 leap seconds)
const TERRESTRIAL_AHEAD: f64 = 69.184;

// mars: the julian day its sol count starts from, and a sol in earth days (Allison & McEwen)
const MARS_EPOCH: f64 = 2_405_522.002_877_9;
const SOL: f64 = 1.027_491_251_7;

// swatch internet time: biel mean time is UTC+1, the day split into 1000 beats
const BIEL_AHEAD: f64 = 3_600.0;
const BEATS_PER_DAY: f64 = 1_000.0;

// hexadecimal time splits the day into 65536
const HEX_PER_DAY: f64 = 65_536.0;

// french decimal time: 10 hours of 100 minutes
const DECIMAL_MINUTES_PER_DAY: f64 = 1_000.0;

// the binary clock: the bits each digit of HH:MM can need, high bit first
const DIGIT_BITS: [u32; 4] = [2, 4, 3, 4];
const BITS: u32 = 4;

// where things sit: left edges as fractions of the width, top of the height below the bar
const CLOCK_LEFT: f32 = 0.12;
const TABLE_LEFT: f32 = 0.5;
const TOP: f32 = 0.24;

// the binary clock's cells as a fraction of the page's height, the gap between hours and minutes in cells
const CELL: f32 = 0.07;
const CELL_GAP: f32 = 0.18;
const HALF_GAP: f32 = 0.7;

// text sizes as fractions of the page's height
const TEXT_SIZE: f32 = 0.02;
const LABEL_SIZE: f32 = 0.012;

// at the reference height
const LINE_GAP: f32 = 18.0;

// the table's keys, padded to this many characters
const KEY_WIDTH: usize = 13;

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    let size = TEXT_SIZE * page.height;

    // the minute that's shown, so every row agrees with the clock
    let instant = moment.epoch_seconds - moment.epoch_seconds.rem_euclid(60);
    let local_minutes = f64::from(moment.hour * 60 + moment.minute);

    let date = moment.date;
    let (iso_year, week) = (iso_year(date), date.iso_week());
    let (sol, mars_hours) = mars(instant);

    let rows = [
        ("unix", instant.to_string()),
        ("julian day", format!("{:.4}", julian(instant))),
        (
            "iso week",
            format!("{iso_year}-W{week:02}-{}", date.weekday() + 1),
        ),
        (
            "ordinal",
            format!("{}-{:03}", date.year, date.day_of_year()),
        ),
        ("swatch .beat", format!("@{:03}", beat(instant))),
        ("decimal", decimal(local_minutes)),
        ("hexadecimal", hexadecimal(local_minutes)),
        (
            "mars",
            format!(
                "{:02}:{:02} MTC · sol {}",
                mars_hours.floor(),
                (mars_hours.fract() * 60.0).floor(),
                sol
            ),
        ),
    ];

    let mut lines: Vec<Box<dyn Widget>> = children![
        Text::new("the same minute, other ways")
            .size(size)
            .color(t.comment)
    ];

    for (key, value) in rows {
        lines.push(Box::new(Row::new(children![
            Text::new(format!("{key:KEY_WIDTH$}"))
                .size(size)
                .color(t.comment),
            Text::new(value).size(size).color(t.fg_dim)
        ])));
    }

    let top = page.top + TOP * page.body_height();

    children![
        placed(
            CLOCK_LEFT * page.width,
            top,
            binary_clock(page, t, moment.hour, moment.minute)
        ),
        placed(
            TABLE_LEFT * page.width,
            top,
            Column::new(lines).gap(page.scaled(LINE_GAP))
        )
    ]
}

/*
 * the time's four digits as columns of bits, 8 at the top and 1 at the
 * bottom, lit in the accent; the digit itself under each column
 */
fn binary_clock(page: Page, t: &Theme, hour: u32, minute: u32) -> Row {
    let cell = CELL * page.height;
    let gap = CELL_GAP * cell;
    let label_size = LABEL_SIZE * page.height;

    let digits = [hour / 10, hour % 10, minute / 10, minute % 10];

    let column = |index: usize| -> Box<dyn Widget> {
        let digit = digits[index];
        let needed = DIGIT_BITS[index];

        let cells: Vec<Box<dyn Widget>> = (0..BITS)
            .rev()
            .map(|bit| -> Box<dyn Widget> {
                let square = Rectangle::new().width(cell).height(cell);

                // bits a digit can't need are left as gaps, so each column has its own height
                if bit >= needed {
                    return Box::new(square);
                }

                Box::new(square.fill(if lit(digit, bit) { t.accent } else { t.bg2 }))
            })
            .collect();

        let mut parts = cells;

        parts.push(Box::new(
            Rectangle::new()
                .width(cell)
                .height(label_size * 2.5)
                .align_child(Center, Center)
                .child(
                    Text::new(digit.to_string())
                        .size(label_size)
                        .color(t.comment),
                ),
        ));

        Box::new(Column::new(parts).gap(gap))
    };

    // the bits' worth beside the rows
    let weights: Vec<Box<dyn Widget>> = (0..BITS)
        .rev()
        .map(|bit| -> Box<dyn Widget> {
            Box::new(
                Rectangle::new()
                    .width(label_size * 2.0)
                    .height(cell)
                    .align_child(Center, Center)
                    .child(
                        Text::new((1u32 << bit).to_string())
                            .size(label_size)
                            .color(t.comment),
                    ),
            )
        })
        .collect();

    Row::new(children![
        Column::new(weights).gap(gap),
        Row::new(vec![column(0), column(1)]).gap(gap),
        Row::new(vec![column(2), column(3)]).gap(gap)
    ])
    .gap(HALF_GAP * cell)
}

fn lit(digit: u32, bit: u32) -> bool {
    digit >> bit & 1 == 1
}

fn julian(epoch_seconds: i64) -> f64 {
    epoch_seconds as f64 / SECONDS_PER_DAY + UNIX_EPOCH_JULIAN
}

// beats since midnight in biel, 0 to 999
fn beat(epoch_seconds: i64) -> u32 {
    let seconds = (epoch_seconds as f64 + BIEL_AHEAD).rem_euclid(SECONDS_PER_DAY);

    (seconds / SECONDS_PER_DAY * BEATS_PER_DAY).floor() as u32
}

// the mars sol number and the hours into that sol, coordinated mars time
fn mars(epoch_seconds: i64) -> (i64, f64) {
    let terrestrial = julian(epoch_seconds) + TERRESTRIAL_AHEAD / SECONDS_PER_DAY;
    let sol_date = (terrestrial - MARS_EPOCH) / SOL;

    (sol_date.floor() as i64, sol_date.fract() * 24.0)
}

// like 8h 51, ten hours of a hundred minutes
fn decimal(local_minutes: f64) -> String {
    let minutes = (local_minutes / MINUTES_PER_DAY * DECIMAL_MINUTES_PER_DAY).floor() as u32;

    format!("{}h {:02}", minutes / 100, minutes % 100)
}

// like D_84_A: the day as sixteen hex hours of 256 minutes of 16 seconds
fn hexadecimal(local_minutes: f64) -> String {
    let value = (local_minutes / MINUTES_PER_DAY * HEX_PER_DAY).floor() as u32;

    format!(
        "{:X}_{:02X}_{:X}",
        value >> 12,
        (value >> 4) & 0xFF,
        value & 0xF
    )
}

// the year an iso week belongs to: the year of its thursday
fn iso_year(date: Date) -> i32 {
    let thursday = date.days_since_epoch() + 3 - i64::from(date.weekday());

    calendar::civil_from_days(thursday).year
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_of_a_digit() {
        // 5 is 0101
        assert!(lit(5, 0));
        assert!(!lit(5, 1));
        assert!(lit(5, 2));
        assert!(!lit(5, 3));
    }

    #[test]
    fn swatch_beats() {
        // midnight UTC is 01:00 in biel, beat 41
        assert_eq!(beat(0), 41);

        // 23:00 UTC is midnight in biel
        assert_eq!(beat(23 * 3_600), 0);
    }

    #[test]
    fn days_and_sols() {
        assert_eq!(julian(0), UNIX_EPOCH_JULIAN);

        // 2000-01-06 00:00 UTC is sol date 44795.9998 (Allison & McEwen), right at a sol's start
        let (sol, hours) = mars(947_116_800);
        let sol_date = sol as f64 + hours / 24.0;

        assert!((sol_date - 44_796.0).abs() < 0.01, "sol date {sol_date}");
    }

    #[test]
    fn other_clocks() {
        // noon is 5h 00 in decimal time and 8_00_0 in hex
        assert_eq!(decimal(720.0), "5h 00");
        assert_eq!(hexadecimal(720.0), "8_00_0");
        assert_eq!(hexadecimal(0.0), "0_00_0");
        assert_eq!(decimal(1_439.0), "9h 99");
    }

    #[test]
    fn iso_years_follow_thursday() {
        // 31 dec 2024 is in week 1 of 2025, 1 jan 2021 in week 53 of 2020
        assert_eq!(iso_year(Date::new(2024, 12, 31)), 2025);
        assert_eq!(iso_year(Date::new(2021, 1, 1)), 2020);
        assert_eq!(iso_year(Date::new(2026, 11, 16)), 2026);
    }
}
