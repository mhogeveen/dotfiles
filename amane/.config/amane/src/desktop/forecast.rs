/*
 * forecast: the weather the shell already fetches, given the whole page. the
 * weather now in words, the next 24 hours as temperature bars with the chance
 * of rain under each in the accent, and the week as rows with a bar from each
 * day's low to its high, on one scale for the whole week
 *
 * until the first forecast is in it says so, and nothing else
 */

use amane::{
    Align::{Center, End, Start},
    Column, Rectangle, Row, Text, Widget, children,
};

use super::{Moment, Page, placed};
use crate::theme::Theme;
use crate::weather::forecast::{self, Forecast};

// in english, like the bar
const DAY_NAMES: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

const COMPASS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
const DEGREES_PER_POINT: f64 = 45.0;

// the left column and the week: left edges as fractions of the width, tops of the height below the bar
const LEFT: f32 = 0.12;
const WEEK_LEFT: f32 = 0.58;
const TOP: f32 = 0.18;
const HOURS_TOP: f32 = 0.52;

// text sizes as fractions of the page's height
const ICON_SIZE: f32 = 0.08;

// the weather icon's box, in icon sizes
const ICON_BOX: f32 = 1.4;
const HEADLINE_SIZE: f32 = 0.06;
const LINE_SIZE: f32 = 0.016;
const LABEL_SIZE: f32 = 0.012;

// the hours: the chart's width as a fraction of the width, its bars' tallest height of the height
const CHART_WIDTH: f32 = 0.36;
const CHART_HEIGHT: f32 = 0.2;

// the lowest bar still shows this much of the tallest, so a cold hour isn't nothing
const LOWEST_BAR: f32 = 0.2;

// the week: its range bars' width as a fraction of the width
const RANGE_WIDTH: f32 = 0.14;

// at the reference height: gaps, the rain bars, the range bars and the week's rows
const LINE_GAP: f32 = 12.0;
const BAR_GAP: f32 = 14.0;
const RAIN_BAR: f32 = 4.0;
const RANGE_BAR: f32 = 6.0;
const ROW_HEIGHT: f32 = 56.0;
const COLUMN_GAP: f32 = 24.0;

// the week's columns, in characters
const NAME_COLUMN: f32 = 4.0;
const DEGREE_COLUMN: f32 = 5.0;
const RAIN_COLUMN: f32 = 5.0;
const CHARACTER_WIDTH: f32 = 0.6;

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    let line_size = LINE_SIZE * page.height;

    let Some(forecast) = &moment.forecast else {
        return children![placed(
            LEFT * page.width,
            page.top + TOP * page.body_height(),
            Text::new(format!(
                "no forecast for {} yet",
                moment.place.name.to_lowercase()
            ))
            .size(line_size)
            .color(t.comment)
        )];
    };

    let now = &forecast.now;
    let (icon, description) = forecast::describe(now.code, now.daytime);

    // nerd font icons draw wider than they measure, so the icon gets a box of its own
    let icon_size = ICON_SIZE * page.height;

    let headline = Row::new(children![
        Rectangle::new()
            .width(ICON_BOX * icon_size)
            .height(icon_size * ICON_BOX)
            .align_child(Start, Center)
            .child(Text::new(icon).size(icon_size).color(t.fg)),
        Text::new(forecast::degrees(now.temperature))
            .size(HEADLINE_SIZE * page.height)
            .color(t.fg)
    ])
    .gap(page.scaled(COLUMN_GAP))
    .align(Center);

    let text = Column::new(children![
        Text::new(moment.place.name.to_lowercase())
            .size(line_size)
            .color(t.comment),
        headline,
        Text::new(description.to_lowercase())
            .size(line_size)
            .color(t.fg),
        Text::new(format!(
            "feels like {} · wind {} km/h from the {}",
            forecast::degrees(now.feels_like),
            now.wind.round(),
            compass(now.direction)
        ))
        .size(line_size)
        .color(t.fg_dim)
    ])
    .gap(page.scaled(LINE_GAP));

    children![
        placed(LEFT * page.width, page.top + TOP * page.body_height(), text),
        placed(
            LEFT * page.width,
            page.top + HOURS_TOP * page.body_height(),
            hours(page, t, forecast)
        ),
        placed(
            WEEK_LEFT * page.width,
            page.top + TOP * page.body_height(),
            week(page, t, forecast, moment)
        )
    ]
}

// the next 24 hours, three at a time: temperature on top, a bar, the rain, the hour
fn hours(page: Page, t: &Theme, forecast: &Forecast) -> Column {
    let label_size = LABEL_SIZE * page.height;
    let slots = &forecast.slots;

    let gap = page.scaled(BAR_GAP);
    let width = (CHART_WIDTH * page.width - gap * (slots.len().max(1) - 1) as f32)
        / slots.len().max(1) as f32;
    let tallest = CHART_HEIGHT * page.height;

    let temperatures: Vec<f64> = slots.iter().map(|slot| slot.temperature).collect();

    let columns: Vec<Box<dyn Widget>> = slots
        .iter()
        .map(|slot| -> Box<dyn Widget> {
            let height = tallest * bar_height(slot.temperature, &temperatures);
            let (icon, _) = forecast::describe(slot.code, slot.daytime);

            let bar = Rectangle::new()
                .width(width)
                .height(tallest)
                .align_child(Start, End)
                .child(Rectangle::new().width(width).height(height).fill(t.bg3));

            let rain = Rectangle::new()
                .width(width)
                .height(page.scaled(RAIN_BAR))
                .fill(t.bg1)
                .align_child(Start, Start)
                .child(
                    Rectangle::new()
                        .width(width * slot.rain as f32 / 100.0)
                        .height(page.scaled(RAIN_BAR))
                        .fill(t.accent),
                );

            let label = |text: String, color| {
                Rectangle::new()
                    .width(width)
                    .height(label_size * 1.6)
                    .align_child(Center, Center)
                    .child(Text::new(text).size(label_size).color(color))
            };

            Box::new(
                Column::new(children![
                    label(forecast::degrees(slot.temperature), t.fg_dim),
                    bar,
                    rain,
                    label(format!("{}%", slot.rain), t.comment),
                    label(icon.to_string(), t.fg_dim),
                    label(format!("{:02}", slot.hour), t.comment)
                ])
                .gap(page.scaled(LINE_GAP) / 2.0),
            )
        })
        .collect();

    Column::new(children![
        Text::new("the next 24 hours")
            .size(LINE_SIZE * page.height)
            .color(t.comment),
        Row::new(columns).gap(gap)
    ])
    .gap(page.scaled(LINE_GAP) * 2.0)
}

// a row per day: name, icon, low, a bar from low to high on the week's scale, high, rain
fn week(page: Page, t: &Theme, forecast: &Forecast, moment: &Moment) -> Column {
    let size = LINE_SIZE * page.height;
    let character = size * CHARACTER_WIDTH;
    let column_gap = page.scaled(COLUMN_GAP);

    let coldest = forecast
        .days
        .iter()
        .map(|day| day.low)
        .fold(f64::INFINITY, f64::min);
    let warmest = forecast
        .days
        .iter()
        .map(|day| day.high)
        .fold(f64::NEG_INFINITY, f64::max);

    let range_width = RANGE_WIDTH * page.width;
    let row_height = page.scaled(ROW_HEIGHT);

    let rows: Vec<Box<dyn Widget>> = forecast
        .days
        .iter()
        .map(|day| -> Box<dyn Widget> {
            let today = day.date == moment.date;
            let color = if today { t.fg } else { t.fg_dim };

            let (from, to) = span(day.low, day.high, coldest, warmest);
            let (icon, _) = forecast::describe(day.code, true);

            let cell = |text: String, characters: f32, color, end: bool| {
                Rectangle::new()
                    .width(characters * character)
                    .height(row_height)
                    .align_child(if end { End } else { Start }, Center)
                    .child(Text::new(text).size(size).color(color))
            };

            let range = Rectangle::new()
                .width(range_width)
                .height(page.scaled(RANGE_BAR))
                .fill(t.bg1)
                .align_child(Start, Start)
                .child(
                    Rectangle::new()
                        .width((to - from) * range_width)
                        .height(page.scaled(RANGE_BAR))
                        .fill(if today { t.accent } else { t.bg4 })
                        .translate(from * range_width, 0.0),
                );

            let name = if today {
                String::from("today")
            } else {
                DAY_NAMES[day.date.weekday() as usize].to_string()
            };

            Box::new(
                Row::new(children![
                    cell(name, NAME_COLUMN + 1.0, color, false),
                    cell(icon.to_string(), 2.0, color, false),
                    cell(forecast::degrees(day.low), DEGREE_COLUMN, t.comment, true),
                    range,
                    cell(forecast::degrees(day.high), DEGREE_COLUMN, color, false),
                    cell(format!("{}%", day.rain), RAIN_COLUMN, t.comment, true)
                ])
                .gap(column_gap)
                .align(Center),
            )
        })
        .collect();

    let mut parts: Vec<Box<dyn Widget>> = children![
        Text::new("the week")
            .size(LINE_SIZE * page.height)
            .color(t.comment)
    ];

    parts.extend(rows);

    Column::new(parts)
}

// how tall a temperature's bar is, from LOWEST_BAR for the coldest to 1 for the warmest
fn bar_height(temperature: f64, all: &[f64]) -> f32 {
    let coldest = all.iter().copied().fold(f64::INFINITY, f64::min);
    let warmest = all.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if warmest - coldest < f64::EPSILON {
        return 1.0;
    }

    let share = ((temperature - coldest) / (warmest - coldest)) as f32;

    LOWEST_BAR + (1.0 - LOWEST_BAR) * share
}

// where a day's low and high sit on the week's scale, 0 to 1
fn span(low: f64, high: f64, coldest: f64, warmest: f64) -> (f32, f32) {
    if warmest - coldest < f64::EPSILON {
        return (0.0, 1.0);
    }

    let at = |value: f64| ((value - coldest) / (warmest - coldest)).clamp(0.0, 1.0) as f32;

    (at(low), at(high))
}

// where the wind comes from, as one of eight points
fn compass(degrees: f64) -> &'static str {
    let point = (degrees.rem_euclid(360.0) / DEGREES_PER_POINT).round() as usize;

    COMPASS[point % COMPASS.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winds_have_a_direction() {
        assert_eq!(compass(0.0), "N");
        assert_eq!(compass(350.0), "N");
        assert_eq!(compass(225.0), "SW");
        assert_eq!(compass(100.0), "E");
        assert_eq!(compass(-45.0), "NW");
    }

    #[test]
    fn bars_grow_with_the_temperature() {
        let all = [2.0, 6.0, 10.0];

        assert_eq!(bar_height(2.0, &all), LOWEST_BAR);
        assert_eq!(bar_height(10.0, &all), 1.0);
        assert_eq!(bar_height(6.0, &[6.0, 6.0]), 1.0);
    }

    #[test]
    fn days_sit_on_the_weeks_scale() {
        assert_eq!(span(2.0, 6.0, 2.0, 10.0), (0.0, 0.5));
        assert_eq!(span(5.0, 5.0, 5.0, 5.0), (0.0, 1.0));
    }
}
