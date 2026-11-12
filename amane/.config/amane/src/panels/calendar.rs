/*
 * calendar panel, dropping down from the clock, in two cards:
 *
 * the month on top, with ISO week numbers, weeks starting on Monday and today in
 * the accent; h and l or the arrow buttons move a month, t or clicking the month
 * name goes back to today
 *
 * the weather below it (src/weather): now, the next 24 hours in three-hour steps
 * and the next 7 days; it's fetched when the panel opens, at most every 30
 * minutes, and r fetches it again
 *
 * both cards share one grid: a gutter on the left (the week numbers) and seven
 * columns, so the hours and days line up under the weekdays; every cell looks
 * the same, and now and today are in the accent wherever they show
 */

use amane::{
    Align::{Center, Start},
    Color, Column,
    Cursor::Pointer,
    Key, LayerWindow, Margin, Monitor, Padding, Rectangle, Row, Service,
    Size::Parent,
    Text, Vertical, Weight, Widget, children,
};

use super::ui::{self, BUTTON, HEADING, PADDING};
use super::{CALENDAR, hints, panel};
use crate::calendar::{self, Date, Day, MONTHS, WEEKDAYS};
use crate::clock::Clock;
use crate::theme::Theme;
use crate::weather::forecast::{self, DayForecast, Now, Slot};
use crate::weather::{self, Weather};

// nerd font material design chevrons
const ICON_PREVIOUS: &str = "\u{f0141}";
const ICON_NEXT: &str = "\u{f0142}";

const KEY_PREVIOUS: char = 'h';
const KEY_NEXT: char = 'l';
const KEY_TODAY: char = 't';
const KEY_REFRESH: char = 'r';

/*
 * the grid both cards share: the gutter, then seven columns, all GAP apart;
 * GAP also sits between the rows of a grid
 */
const GUTTER: f32 = 24.0;
const COLUMN: f32 = 50.0;
const GAP: f32 = 6.0;
const GRID_WIDTH: f32 = GUTTER + GAP + 7.0 * COLUMN + 6.0 * GAP;

// under a card's header or a heading, between the weather's parts, between the cards
const HEADER_GAP: f32 = 8.0;
const SECTION_GAP: f32 = 16.0;
const CARD_GAP: f32 = 12.0;

/*
 * text: labels over or in a column (weekdays, hours, day names), values (dates,
 * temperatures), and the quieter extras (week numbers, rain, lows)
 */
const LABEL: f32 = 12.0;
const VALUE: f32 = 13.0;
const EXTRA: f32 = 11.0;

// the calendar: weekday names over six weeks, always six so the card never changes size
const LABEL_ROW: f32 = 16.0;
const DAY: f32 = 34.0;
const MONTH: f32 = LABEL_ROW + GAP + 6.0 * DAY + 5.0 * GAP;

// a forecast cell's four lines, when, icon, temperature and rain, and the room around them
const LABEL_LINE: f32 = 16.0;
const ICON_LINE: f32 = 24.0;
const FORECAST_ICON: f32 = 18.0;
const VALUE_LINE: f32 = 18.0;
const EXTRA_LINE: f32 = 14.0;
const LINE_GAP: f32 = 2.0;
const SLOT_PADDING: f32 = 4.0;
const SLOT: f32 =
    SLOT_PADDING * 2.0 + LABEL_LINE + ICON_LINE + VALUE_LINE + EXTRA_LINE + 3.0 * LINE_GAP;

// the weather now: icon, temperature and description, then wind and when it was fetched
const NOW_ROW: f32 = 28.0;
const NOW_ICON: f32 = 24.0;
const NOW_LINE: f32 = 16.0;

// the icon and temperature now, bigger than the rest of the card
const NOW_ICON_SIZE: f32 = 20.0;
const NOW_TEMPERATURE: f32 = 22.0;
const NOW_GAP: f32 = 4.0;
const NOW: f32 = NOW_ROW + NOW_GAP + NOW_LINE;

// a heading and its row of seven cells
const FORECAST: f32 = HEADING + HEADER_GAP + SLOT;

const CALENDAR_CARD: f32 = PADDING * 2.0 + BUTTON + HEADER_GAP + MONTH;
const WEATHER_BODY: f32 = NOW + 2.0 * (SECTION_GAP + FORECAST);
const WEATHER_CARD: f32 = PADDING * 2.0 + BUTTON + HEADER_GAP + WEATHER_BODY;

const WIDTH: f32 = PADDING * 4.0 + GRID_WIDTH;
const HEIGHT: f32 = PADDING * 2.0 + CALENDAR_CARD + CARD_GAP + WEATHER_CARD;

// which month is shown; None follows today
struct Shown {
    month: Option<(i32, u32)>,
}

impl Service for Shown {
    fn new() -> Self {
        Self { month: None }
    }

    // only changes through input
    fn listen() {}
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

    ui::frame(
        t,
        CARD_GAP,
        children![calendar_card(t, today, year, month), weather_card(t, today)],
    )
}

// back to today's month the next time it opens
pub fn reset() {
    Shown::write().month = None;
}

fn key_pressed(key: Key) {
    match super::letter(&key) {
        Some(KEY_PREVIOUS) => go(-1),
        Some(KEY_NEXT) => go(1),
        Some(KEY_TODAY) => reset(),
        Some(KEY_REFRESH) => weather::refresh(),
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

// the pieces both cards are made of

// a raised card with a header on top and its body under it
fn card(t: &Theme, height: f32, header: Row, body: impl Widget + 'static) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(height)
        .fill(t.bg0)
        .border(1.0, t.bg3)
        .padding(PADDING)
        .child(Column::new(children![header, body]).gap(HEADER_GAP))
}

// a title on the left taking the room the buttons on the right leave
fn header(title: Rectangle, buttons: Vec<Box<dyn Widget>>) -> Row {
    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(title)];

    parts.extend(buttons);

    Row::new(parts).gap(GAP).align(Center).height(BUTTON)
}

// the gutter, then seven cells
fn grid_row(gutter: Rectangle, cells: Vec<Box<dyn Widget>>, height: f32) -> Row {
    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(gutter)];

    parts.extend(cells);

    Row::new(parts).gap(GAP).height(height)
}

fn gutter(height: f32) -> Rectangle {
    Rectangle::new()
        .width(GUTTER)
        .height(height)
        .align_child(Center, Center)
}

// every cell in both cards; the accent one is today or now
fn cell(t: &Theme, height: f32, accent: bool) -> Rectangle {
    let fill = if accent { t.accent } else { t.bg1 };

    Rectangle::new()
        .width(COLUMN)
        .height(height)
        .fill(fill)
        .align_child(Center, Center)
}

// the calendar card

fn calendar_card(t: &Theme, today: Date, year: i32, month: u32) -> Rectangle {
    /*
     * clicking the month name goes back to today, so while ? hints show it gives
     * way to that key, in the accent like every button's key
     */
    let title = if hints::shown() {
        Rectangle::new()
            .width(Parent)
            .height(BUTTON)
            .align_child(Start, Center)
            .child(
                Text::new(KEY_TODAY.to_string())
                    .size(ui::TEXT_TITLE)
                    .weight(Weight::Bold)
                    .color(t.accent),
            )
    } else {
        ui::title(t, &format!("{} {year}", MONTHS[month as usize - 1]))
    };

    let title = title.cursor(Pointer).on_click(|_| reset());

    let header = header(
        title,
        children![
            ui::key_button(t, ICON_PREVIOUS, KEY_PREVIOUS, |_| go(-1)),
            ui::key_button(t, ICON_NEXT, KEY_NEXT, |_| go(1)),
        ],
    );

    let mut weekdays: Vec<Box<dyn Widget>> = Vec::new();

    for name in WEEKDAYS {
        weekdays.push(Box::new(
            Rectangle::new()
                .width(COLUMN)
                .height(LABEL_ROW)
                .align_child(Center, Center)
                .child(Text::new(name).size(LABEL).color(t.comment)),
        ));
    }

    let mut rows: Vec<Box<dyn Widget>> =
        vec![Box::new(grid_row(gutter(LABEL_ROW), weekdays, LABEL_ROW))];

    for week in calendar::grid(year, month) {
        // a week with no day of this month stays empty, its number included
        let number = if week.days.iter().any(|day| day.in_month) {
            gutter(DAY).child(
                Text::new(week.number.to_string())
                    .size(EXTRA)
                    .color(t.comment),
            )
        } else {
            gutter(DAY)
        };

        let days = week
            .days
            .into_iter()
            .map(|day| Box::new(day_cell(t, day, today)) as Box<dyn Widget>)
            .collect();

        rows.push(Box::new(grid_row(number, days, DAY)));
    }

    card(t, CALENDAR_CARD, header, Column::new(rows).gap(GAP))
}

// a day of the shown month, today in the accent; days around the month are left empty
fn day_cell(t: &Theme, day: Day, today: Date) -> Rectangle {
    if !day.in_month {
        return Rectangle::new().width(COLUMN).height(DAY);
    }

    let is_today = day.date == today;

    let color = if is_today { t.on_accent } else { t.fg };

    cell(t, DAY, is_today).child(Text::new(day.date.day.to_string()).size(VALUE).color(color))
}

// the weather card

/*
 * now, the next 24 hours and the next 7 days; reading the weather starts a fetch
 * when the last one is old, which redraws the panel once it's in
 */
fn weather_card(t: &Theme, today: Date) -> Rectangle {
    weather::refresh_if_stale();

    let weather = Weather::read();

    // like the bluetooth panel's "Scanning…", left of the button
    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    if weather.refreshing() {
        buttons.push(Box::new(
            Text::new("Refreshing…")
                .size(ui::TEXT_SMALL)
                .color(t.comment),
        ));
    }

    buttons.push(Box::new(ui::key_button(
        t,
        ui::ICON_REFRESH,
        KEY_REFRESH,
        |_| weather::refresh(),
    )));

    let header = header(
        ui::title(t, &format!("Weather · {}", weather.place().name)),
        buttons,
    );

    let Some(forecast) = weather.forecast() else {
        let text = match weather.error() {
            Some(error) => format!("Weather unavailable: {error}"),
            None => String::from("Loading weather…"),
        };

        let note = Rectangle::new()
            .width(Parent)
            .height(WEATHER_BODY)
            .align_child(Center, Center)
            .child(Text::new(text).size(VALUE).color(t.comment).elide());

        return card(t, WEATHER_CARD, header, note);
    };

    let now = &forecast.now;

    let when = when_text(weather.error().is_some(), weather.fetched_at());

    let slots = forecast
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| Box::new(slot_cell(t, slot, index == 0)) as Box<dyn Widget>)
        .collect();

    let days = forecast
        .days
        .iter()
        .map(|day| Box::new(day_forecast_cell(t, day, today)) as Box<dyn Widget>)
        .collect();

    let body = Column::new(children![
        Column::new(children![
            now_conditions(t, now),
            now_details(t, now, &when)
        ])
        .gap(NOW_GAP),
        forecast_row(t, "Next 24 hours", slots),
        forecast_row(t, "Next 7 days", days),
    ])
    .gap(SECTION_GAP);

    card(t, WEATHER_CARD, header, body)
}

// the icon, the temperature, and what it's like: Mainly clear · feels 1°
fn now_conditions(t: &Theme, now: &Now) -> Row {
    let (icon, description) = forecast::describe(now.code, now.daytime);

    // nerd font icons draw wider than they measure, so the icon gets a box of its own
    let icon = Rectangle::new()
        .width(NOW_ICON)
        .height(NOW_ROW)
        .align_child(Start, Center)
        .child(Text::new(icon).size(NOW_ICON_SIZE).color(t.fg));

    Row::new(children![
        icon,
        Text::new(forecast::degrees(now.temperature))
            .size(NOW_TEMPERATURE)
            .weight(Weight::Bold)
            .color(t.fg),
        ui::line(
            NOW_ROW,
            Text::new(format!(
                "{description} · feels {}",
                forecast::degrees(now.feels_like)
            ))
            .size(VALUE)
            .color(t.fg_dim)
            .elide()
        ),
    ])
    .gap(HEADER_GAP)
    .align(Center)
    .height(NOW_ROW)
}

// the wind, and when the forecast was fetched: 7 km/h NE · updated 17:43
fn now_details(t: &Theme, now: &Now, when: &str) -> Rectangle {
    let details = format!(
        "{:.0} km/h {}{when}",
        now.wind,
        forecast::compass(now.direction)
    );

    ui::line(
        NOW_LINE,
        Text::new(details).size(LABEL).color(t.comment).elide(),
    )
}

// " · updated 17:43", or " · couldn't update · from 17:43" after a failed refetch
fn when_text(failed: bool, fetched_at: Option<&str>) -> String {
    match (failed, fetched_at) {
        (true, Some(at)) => format!(" · couldn't update · from {at}"),
        (false, Some(at)) => format!(" · updated {at}"),
        _ => String::new(),
    }
}

// a heading over seven cells on the grid
fn forecast_row(t: &Theme, title: &str, cells: Vec<Box<dyn Widget>>) -> Column {
    Column::new(children![
        ui::heading(t, title, HEADING),
        grid_row(gutter(SLOT), cells, SLOT)
    ])
    .gap(HEADER_GAP)
}

/*
 * a forecast cell: when, an icon, the temperature and the chance of rain (0%
 * too, that's worth knowing), all in on_accent when the cell is in the accent
 */
fn forecast_cell(
    t: &Theme,
    accent: bool,
    when: String,
    icon: &str,
    temperature: impl Widget + 'static,
    rain: u32,
) -> Rectangle {
    let (strong, quiet) = if accent {
        (t.on_accent, t.on_accent)
    } else {
        (t.fg, t.comment)
    };

    cell(t, SLOT, accent)
        .padding(Padding {
            top: SLOT_PADDING,
            right: 0.0,
            bottom: SLOT_PADDING,
            left: 0.0,
        })
        .align_child(Center, Start)
        .child(
            Column::new(children![
                line(LABEL_LINE, Text::new(when).size(LABEL).color(quiet)),
                line(ICON_LINE, Text::new(icon).size(FORECAST_ICON).color(strong)),
                line(VALUE_LINE, temperature),
                line(
                    EXTRA_LINE,
                    Text::new(format!("{rain}%")).size(EXTRA).color(quiet)
                ),
            ])
            .gap(LINE_GAP),
        )
}

// one centered line of a forecast cell
fn line(height: f32, content: impl Widget + 'static) -> Rectangle {
    Rectangle::new()
        .width(COLUMN)
        .height(height)
        .align_child(Center, Center)
        .child(content)
}

// three hours; the first is now
fn slot_cell(t: &Theme, slot: &Slot, now: bool) -> Rectangle {
    let (icon, _) = forecast::describe(slot.code, slot.daytime);

    let when = if now {
        String::from("Now")
    } else {
        format!("{:02}", slot.hour)
    };

    let color = if now { t.on_accent } else { t.fg };

    let temperature = Text::new(forecast::degrees(slot.temperature))
        .size(VALUE)
        .color(color);

    forecast_cell(t, now, when, icon, temperature, slot.rain)
}

// a day, with the high and the quieter low on one line
fn day_forecast_cell(t: &Theme, day: &DayForecast, today: Date) -> Rectangle {
    let (icon, _) = forecast::describe(day.code, true);

    let is_today = day.date == today;

    let name = if is_today {
        String::from("Today")
    } else {
        String::from(WEEKDAYS[day.date.weekday() as usize])
    };

    let (high, low): (Color, Color) = if is_today {
        (t.on_accent, t.on_accent)
    } else {
        (t.fg, t.comment)
    };

    let temperatures = Row::new(children![
        Text::new(forecast::degrees(day.high))
            .size(VALUE)
            .color(high),
        Text::new(format!("/{}", forecast::degrees(day.low)))
            .size(EXTRA)
            .color(low),
    ])
    .align(Center);

    forecast_cell(t, is_today, name, icon, temperatures, day.rain)
}

#[cfg(test)]
mod tests {
    use super::when_text;

    #[test]
    fn the_weather_says_when_it_was_fetched() {
        assert_eq!(when_text(false, Some("17:43")), " · updated 17:43");
        assert_eq!(
            when_text(true, Some("17:43")),
            " · couldn't update · from 17:43"
        );
        assert_eq!(when_text(false, None), "");
    }
}
