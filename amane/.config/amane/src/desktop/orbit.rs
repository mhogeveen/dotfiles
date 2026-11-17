/*
 * orbit: where the earth is on its way around the sun today. how far from the
 * sun it is, how long the sunlight took to arrive, how fast the earth moves
 * and how far it went since new year, and when it's next closest or farthest.
 * two lines show it: the distance between closest and farthest, and the year's
 * way around the sun with the seasons' turning points on it
 *
 * a kepler orbit from the mean elements, perihelion taken as 3 january and
 * aphelion as 4 july (a day either way in some years); good to a few hundred
 * thousand km, more than a desktop needs
 */

use std::f64::consts::TAU;

use amane::{Column, Rectangle, Text, Widget, children};

use super::{Moment, Page, in_days, placed, short_date, text_width};
use crate::calendar::Date;
use crate::theme::Theme;

// the orbit's size and shape: semi-major axis in km, eccentricity, and the year it takes
const SEMI_MAJOR_AXIS: f64 = 149_598_023.0;
const ECCENTRICITY: f64 = 0.016_71;
const ANOMALISTIC_YEAR: f64 = 365.259_6;

// the earth's mean speed around the sun, km/s
const MEAN_SPEED: f64 = 29.78;

const LIGHT_SPEED: f64 = 299_792.458;
const SECONDS_PER_DAY: f64 = 86_400.0;
const MILLION: f64 = 1_000_000.0;

// when the earth is closest and farthest, as month and day
const PERIHELION: (u32, u32) = (1, 3);
const APHELION: (u32, u32) = (7, 4);

// the seasons' turning points on the way around, as month, day and name
const TURNS: [(u32, u32, &str); 4] = [
    (3, 20, "equinox"),
    (6, 21, "solstice"),
    (9, 22, "equinox"),
    (12, 21, "solstice"),
];

// the text and the lines: left edge and widths as fractions of the width, tops of the height below the bar
const LEFT: f32 = 0.12;
const LINE_WIDTH: f32 = 0.6;
const TEXT_TOP: f32 = 0.18;
const DISTANCE_TOP: f32 = 0.58;
const YEAR_TOP: f32 = 0.78;

// text sizes as fractions of the page's height
const HEADLINE_SIZE: f32 = 0.045;
const LINE_SIZE: f32 = 0.016;
const LABEL_SIZE: f32 = 0.011;

// at the reference height: lines, marks and gaps
const TRACK: f32 = 2.0;
const MARK: f32 = 16.0;
const TICK: f32 = 14.0;
const LINE_GAP: f32 = 12.0;
const LABEL_GAP: f32 = 14.0;

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    let today = moment.date;
    let day_fraction = f64::from(moment.hour * 60 + moment.minute) / 1_440.0;

    let since_perihelion = days_since(today, PERIHELION) as f64 + day_fraction;
    let distance = distance(since_perihelion);
    let speed = speed(distance);
    let light = distance / LIGHT_SPEED;

    let into_year = (today.day_of_year() - 1) as f64 + day_fraction;
    let travelled = TAU * SEMI_MAJOR_AXIS * into_year / ANOMALISTIC_YEAR;

    // whichever comes first: closest or farthest
    let to_perihelion = days_until(today, PERIHELION);
    let to_aphelion = days_until(today, APHELION);

    let next = if to_perihelion < to_aphelion {
        format!(
            "closest to the sun {} ({})",
            in_days(to_perihelion),
            short_date(after(today, to_perihelion))
        )
    } else {
        format!(
            "farthest from the sun {} ({})",
            in_days(to_aphelion),
            short_date(after(today, to_aphelion))
        )
    };

    let line_size = LINE_SIZE * page.height;

    let text = Column::new(children![
        Text::new("the earth today")
            .size(line_size)
            .color(t.comment),
        Text::new(format!("{:.1} million km from the sun", distance / MILLION))
            .size(HEADLINE_SIZE * page.height)
            .color(t.fg),
        Text::new(format!(
            "the sunlight you see left it {} ago",
            minutes_seconds(light)
        ))
        .size(line_size)
        .color(t.fg_dim),
        Text::new(format!(
            "moving at {speed:.2} km/s, {:.1} million km a day",
            speed * SECONDS_PER_DAY / MILLION
        ))
        .size(line_size)
        .color(t.fg_dim),
        Text::new(format!(
            "about {:.0} million km travelled since 1 january",
            travelled / MILLION
        ))
        .size(line_size)
        .color(t.fg_dim),
        Text::new(next).size(line_size).color(t.fg_dim)
    ])
    .gap(page.scaled(LINE_GAP));

    let left = LEFT * page.width;
    let width = LINE_WIDTH * page.width;

    let closest = SEMI_MAJOR_AXIS * (1.0 - ECCENTRICITY);
    let farthest = SEMI_MAJOR_AXIS * (1.0 + ECCENTRICITY);

    let mut layers: Vec<Box<dyn Widget>> =
        children![placed(left, page.top + TEXT_TOP * page.body_height(), text)];

    // the distance, between closest and farthest
    layers.extend(line(
        page,
        t,
        (left, page.top + DISTANCE_TOP * page.body_height()),
        width,
        ((distance - closest) / (farthest - closest)) as f32,
        &[
            (0.0, format!("closest {:.1}", closest / MILLION)),
            (1.0, format!("farthest {:.1}", farthest / MILLION)),
        ],
        "million km from the sun",
    ));

    // the year around the sun, from 1 january, with the seasons' turns
    let year_days = Date::new(today.year, 12, 31).day_of_year() as f64;

    let turns: Vec<(f32, String)> = TURNS
        .iter()
        .map(|&(month, day, name)| {
            let at = (Date::new(today.year, month, day).day_of_year() - 1) as f64 / year_days;

            (at as f32, name.to_string())
        })
        .collect();

    layers.extend(line(
        page,
        t,
        (left, page.top + YEAR_TOP * page.body_height()),
        width,
        (into_year / year_days) as f32,
        &turns,
        "around the sun since 1 january",
    ));

    layers
}

/*
 * a track with ticks and labels under it at fractions of its length, the
 * earth on it as an accent square, and a caption above at the left
 */
fn line(
    page: Page,
    t: &Theme,
    (left, top): (f32, f32),
    width: f32,
    at: f32,
    ticks: &[(f32, String)],
    caption: &str,
) -> Vec<Box<dyn Widget>> {
    let track = page.scaled(TRACK);
    let mark = page.scaled(MARK);
    let tick = page.scaled(TICK);
    let label_size = LABEL_SIZE * page.height;
    let label_gap = page.scaled(LABEL_GAP);

    let mut layers: Vec<Box<dyn Widget>> = children![
        placed(
            left,
            top - label_gap - 2.0 * label_size,
            Text::new(caption.to_string())
                .size(label_size)
                .color(t.comment)
        ),
        placed(
            left,
            top,
            Rectangle::new().width(width).height(track).fill(t.bg3)
        )
    ];

    for (fraction, label) in ticks {
        let x = left + fraction * width;

        // labels at the very ends line up with them instead of hanging over
        let label_x = if *fraction >= 1.0 {
            x - text_width(label, label_size)
        } else if *fraction <= 0.0 {
            x
        } else {
            x - text_width(label, label_size) / 2.0
        };

        layers.push(Box::new(placed(
            x,
            top,
            Rectangle::new().width(track).height(tick).fill(t.bg4),
        )));

        layers.push(Box::new(placed(
            label_x,
            top + tick + label_gap / 2.0,
            Text::new(label.clone()).size(label_size).color(t.comment),
        )));
    }

    layers.push(Box::new(placed(
        left + at.clamp(0.0, 1.0) * width - mark / 2.0,
        top + track / 2.0 - mark / 2.0,
        Rectangle::new().width(mark).height(mark).fill(t.accent),
    )));

    layers
}

// km from the sun, some days after perihelion: kepler's equation, solved by a few steps
fn distance(days_since_perihelion: f64) -> f64 {
    let mean_anomaly = TAU * days_since_perihelion / ANOMALISTIC_YEAR;

    let mut eccentric = mean_anomaly;

    for _ in 0..5 {
        eccentric = mean_anomaly + ECCENTRICITY * eccentric.sin();
    }

    SEMI_MAJOR_AXIS * (1.0 - ECCENTRICITY * eccentric.cos())
}

// km/s at a distance, by vis-viva: faster when closer
fn speed(distance: f64) -> f64 {
    MEAN_SPEED * (2.0 * SEMI_MAJOR_AXIS / distance - 1.0).sqrt()
}

// days since the last time it was this month and day
fn days_since(today: Date, (month, day): (u32, u32)) -> i64 {
    let this_year = Date::new(today.year, month, day);

    let last = if this_year.days_since_epoch() <= today.days_since_epoch() {
        this_year
    } else {
        Date::new(today.year - 1, month, day)
    };

    today.days_since_epoch() - last.days_since_epoch()
}

// days until the next time it's this month and day, 0 on the day itself
fn days_until(today: Date, (month, day): (u32, u32)) -> i64 {
    let this_year = Date::new(today.year, month, day);

    let next = if this_year.days_since_epoch() >= today.days_since_epoch() {
        this_year
    } else {
        Date::new(today.year + 1, month, day)
    };

    next.days_since_epoch() - today.days_since_epoch()
}

fn after(date: Date, days: i64) -> Date {
    crate::calendar::civil_from_days(date.days_since_epoch() + days)
}

// like 8m 13s
fn minutes_seconds(seconds: f64) -> String {
    let seconds = seconds.round() as u32;

    format!("{}m {:02}s", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_in_january_farthest_in_july() {
        let closest = distance(0.0);
        let farthest = distance(ANOMALISTIC_YEAR / 2.0);

        assert!((closest / MILLION - 147.1).abs() < 0.1, "closest {closest}");
        assert!(
            (farthest / MILLION - 152.1).abs() < 0.1,
            "farthest {farthest}"
        );

        // faster when closer
        assert!(speed(closest) > 30.2 && speed(farthest) < 29.4);
    }

    #[test]
    fn sunlight_takes_about_eight_minutes() {
        let light = SEMI_MAJOR_AXIS / LIGHT_SPEED;

        assert_eq!(minutes_seconds(light), "8m 19s");
    }

    #[test]
    fn counting_to_and_from_dates() {
        let today = Date::new(2026, 11, 16);

        assert_eq!(days_until(today, PERIHELION), 48);
        assert_eq!(days_since(today, PERIHELION), 317);
        assert_eq!(days_until(today, (11, 16)), 0);
        assert_eq!(days_since(Date::new(2026, 1, 2), PERIHELION), 364);
    }
}
