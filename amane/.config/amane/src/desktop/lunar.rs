/*
 * lunar: the moon as it is tonight, a disc of horizontal slices (the shell's
 * blocks, rounded), lit on the side the sun lights it. beside it the phase,
 * how much is lit and how old the moon is, when the next full and new moons
 * are, and a strip of the lunar month's days with today in the accent
 *
 * the phase is worked out in sky.rs from the mean lunar month, so it can be a
 * day off the almanac at times; fine for a desktop
 */

use amane::{Column, Full, Rectangle, Row, Text, Widget, children};

use super::sky::{self, SYNODIC_MONTH};
use super::{Moment, Page, date_after, in_days, placed, short_date};
use crate::theme::Theme;

// the disc: its middle as a fraction of the width, its radius of the height below the bar
const DISC_CENTER: f32 = 0.31;
const DISC_RADIUS: f32 = 0.27;

// how many slices make the disc, and the gap between them at the reference height
const SLICES: u32 = 34;
const SLICE_GAP: f32 = 5.0;

// lit parts narrower than this, at the reference height, aren't drawn
const THINNEST_LIGHT: f32 = 2.0;

// the text: its left edge as a fraction of the width
const TEXT_LEFT: f32 = 0.58;

// text sizes as fractions of the page's height
const HEADLINE_SIZE: f32 = 0.05;
const LINE_SIZE: f32 = 0.016;
const LABEL_SIZE: f32 = 0.011;

// at the reference height: between lines, before the table and before the strip
const LINE_GAP: f32 = 12.0;
const SECTION_GAP: f32 = 36.0;

// the month strip: a cell per day of the lunar month, sized at the reference height
const MONTH_DAYS: u32 = 30;
const CELL: f32 = 22.0;
const CELL_GAP: f32 = 4.0;

// the table's keys, padded to this many characters
const KEY_WIDTH: usize = 11;

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    let age = sky::moon_age(moment.epoch_seconds);

    let mut layers = disc(page, t, age);

    let line_size = LINE_SIZE * page.height;
    let gap = page.scaled(LINE_GAP);

    let next = |target: f64, name: &str| -> Box<dyn Widget> {
        let days = sky::days_until(age, target);
        let date = date_after(moment, days);
        let count = date.days_since_epoch() - moment.date.days_since_epoch();

        Box::new(Row::new(children![
            Text::new(format!("{name:KEY_WIDTH$}"))
                .size(line_size)
                .color(t.comment),
            Text::new(format!("{} · {}", short_date(date), in_days(count)))
                .size(line_size)
                .color(t.fg_dim)
        ]))
    };

    // the nearer of the two first
    let full = sky::days_until(age, SYNODIC_MONTH / 2.0);
    let new = sky::days_until(age, 0.0);

    let table = if full < new {
        Column::new(vec![
            next(SYNODIC_MONTH / 2.0, "full moon"),
            next(0.0, "new moon"),
        ])
    } else {
        Column::new(vec![
            next(0.0, "new moon"),
            next(SYNODIC_MONTH / 2.0, "full moon"),
        ])
    }
    .gap(gap);

    let text = Column::new(children![
        Text::new("the moon tonight")
            .size(line_size)
            .color(t.comment),
        Text::new(sky::phase_name(age))
            .size(HEADLINE_SIZE * page.height)
            .color(t.fg),
        Text::new(format!(
            "{}% lit · {:.1} days old",
            (sky::illumination(age) * 100.0).round(),
            age
        ))
        .size(line_size)
        .color(t.fg_dim),
        Rectangle::new().width(1.0).height(page.scaled(SECTION_GAP)),
        table,
        Rectangle::new().width(1.0).height(page.scaled(SECTION_GAP)),
        month_strip(page, t, age)
    ])
    .gap(gap);

    let radius = DISC_RADIUS * page.body_height();
    let top = page.top + page.body_height() / 2.0 - radius;

    layers.push(Box::new(placed(TEXT_LEFT * page.width, top, text)));

    layers
}

// the disc, slice by slice: the whole slice dark, its lit part over it
fn disc(page: Page, t: &Theme, age: f64) -> Vec<Box<dyn Widget>> {
    let radius = DISC_RADIUS * page.body_height();
    let center_x = DISC_CENTER * page.width;
    let center_y = page.top + page.body_height() / 2.0;

    let pitch = 2.0 * radius / SLICES as f32;
    let thickness = (pitch - page.scaled(SLICE_GAP)).max(1.0);
    let thinnest = page.scaled(THINNEST_LIGHT);

    let (from, to) = sky::lit_span(age);

    let mut layers: Vec<Box<dyn Widget>> = Vec::new();

    for slice in 0..SLICES {
        // the slice's middle, from the top of the disc
        let y = -radius + pitch * (slice as f32 + 0.5);
        let half = (radius * radius - y * y).max(0.0).sqrt();

        let top = center_y + y - thickness / 2.0;

        layers.push(Box::new(placed(
            center_x - half,
            top,
            Rectangle::new()
                .width(2.0 * half)
                .height(thickness)
                .radius(Full)
                .fill(t.bg1),
        )));

        let left = center_x + half * from as f32;
        let right = center_x + half * to as f32;

        if right - left >= thinnest {
            layers.push(Box::new(placed(
                left,
                top,
                Rectangle::new()
                    .width(right - left)
                    .height(thickness)
                    .radius(Full)
                    .fill(t.fg),
            )));
        }
    }

    layers
}

// a cell per day of the lunar month: gone, today in the accent, ahead; new, full and new under it
fn month_strip(page: Page, t: &Theme, age: f64) -> Column {
    let cell = page.scaled(CELL);
    let gap = page.scaled(CELL_GAP);
    let today = age.floor() as u32;

    let cells: Vec<Box<dyn Widget>> = (0..MONTH_DAYS)
        .map(|day| -> Box<dyn Widget> {
            let color = match day.cmp(&today) {
                std::cmp::Ordering::Less => t.bg3,
                std::cmp::Ordering::Equal => t.accent,
                std::cmp::Ordering::Greater => t.bg1,
            };

            Box::new(Rectangle::new().width(cell).height(cell).fill(color))
        })
        .collect();

    let label_size = LABEL_SIZE * page.height;
    let half = (cell + gap) * (MONTH_DAYS / 2) as f32;

    let label = |text: &str, width: f32| -> Box<dyn Widget> {
        Box::new(
            Rectangle::new()
                .width(width)
                .height(2.0 * label_size)
                .child(
                    Text::new(text.to_string())
                        .size(label_size)
                        .color(t.comment),
                ),
        )
    };

    Column::new(children![
        Row::new(cells).gap(gap),
        Row::new(vec![
            label("new", half),
            label("full", half),
            label("new", half)
        ])
    ])
    .gap(gap)
}
