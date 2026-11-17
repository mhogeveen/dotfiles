/*
 * the desktop drawn on the background when the wallpaper is none: one of a few
 * designs, each a page about the day in the shell's own colours and font. the
 * wallpaper picker shows them as a second row while None is selected
 *
 * a design is a function from a page (its size) and a moment (the time, date
 * and weather, read once) to widgets. every size is a fraction of the page or
 * scaled from a 1440 high reference, so the same function draws the full
 * monitor and the picker's thumbnails
 *
 * the design picked is kept in a state file, like the wallpaper; the picker
 * previews by changing the one shown and keeps it with apply. the clock service
 * wakes the page once a minute and nothing animates, so it costs nothing between
 * minutes
 *
 * adding a design: a file in src/desktop/, a variant in Design, its name, and a
 * line in page()
 */

mod almanac;
mod epoch;
mod forecast;
mod ledger;
mod lunar;
mod orbit;
mod pines;
mod prompt;
mod ridges;
mod sky;

use std::fs;

use amane::{Align::Start, Monitor, Rectangle, Service, Size::Parent, Stack, Text, Widget};

use crate::bar;
use crate::calendar::{self, Date};
use crate::clock::{self, Clock};
use crate::state;
use crate::theme::{Theme, theme};
use crate::weather::forecast::{self as weather_forecast, Forecast};
use crate::weather::{Place, Weather};

// the file in the state folder the design is kept in, by name
const STATE: &str = "desktop";

// the height the designs' fixed sizes (gaps, rules, hairlines) are given for
const REFERENCE_HEIGHT: f32 = 1440.0;

// a scaled line or gap never gets thinner than this, so thumbnails keep their hairlines
const THINNEST: f32 = 1.0;

// in english, like the bar and the calendar panel
const DAY_NAMES: [&str; 7] = ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"];
const MONTH_LETTERS: usize = 3;

// monospaced: a character is this much of the text size wide
const CHARACTER_WIDTH: f32 = 0.6;

// the corner ticks every design shares: inset and length as fractions of the height
const CORNER_INSET: f32 = 0.04;
const CORNER_TICK: f32 = 0.02;
const HAIRLINE: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Design {
    // nothing drawn: mango's root_color, the bare background
    None,

    // the time on a left edge, a date line and 24 hour cells
    Ledger,

    // the year as a sheet of day cells, a row per month
    Almanac,

    // a terminal session: the time and what's left of the day, as output
    Prompt,

    // the moon tonight as a disc of slices, and the next full and new moon
    Lunar,

    // the weather now, the next 24 hours as bars, the week as ranges
    Forecast,

    // the minute in binary and in other systems: unix, julian, swatch, mars
    Epoch,

    // the earth's distance from the sun, speed and way around it today
    Orbit,

    // art: hills of vertical strokes, far to near, a sun in the accent behind them
    Ridges,

    // art: a pine forest at night in rows, far to near, a moon in the accent low behind them
    Pines,
}

impl Design {
    // in the order the picker shows them: none, the informative ones, then the art
    pub const ALL: [Design; 10] = [
        Design::None,
        Design::Ledger,
        Design::Almanac,
        Design::Prompt,
        Design::Forecast,
        Design::Lunar,
        Design::Orbit,
        Design::Epoch,
        Design::Ridges,
        Design::Pines,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Design::None => "none",
            Design::Ledger => "ledger",
            Design::Almanac => "almanac",
            Design::Prompt => "prompt",
            Design::Lunar => "lunar",
            Design::Forecast => "forecast",
            Design::Epoch => "epoch",
            Design::Orbit => "orbit",
            Design::Ridges => "ridges",
            Design::Pines => "pines",
        }
    }

    fn from_name(name: &str) -> Option<Design> {
        Design::ALL
            .into_iter()
            .find(|design| design.name() == name.trim())
    }
}

pub struct Desktop {
    // the one picked
    current: Design,

    // what the background draws now, the same as current unless the picker previews another
    shown: Design,
}

impl Service for Desktop {
    // a file of one word, quick enough for the first view that asks
    fn new() -> Self {
        let current = fs::read_to_string(state::path(STATE))
            .ok()
            .and_then(|text| Design::from_name(&text))
            .unwrap_or(Design::Ledger);

        Self {
            current,
            shown: current,
        }
    }

    // changes only through the picker
    fn listen() {}
}

// reading, safe inside views
impl Desktop {
    pub fn current(&self) -> Design {
        self.current
    }

    pub fn shown(&self) -> Design {
        self.shown
    }
}

// shows this one without keeping it; from the picker's keys
pub fn preview(design: Design) {
    let mut desktop = Desktop::write();

    if desktop.shown != design {
        desktop.shown = design;
    }
}

// keeps the one shown, also after a restart
pub fn apply() {
    let design = {
        let mut desktop = Desktop::write();

        desktop.current = desktop.shown;
        desktop.current
    };

    if let Err(error) = state::save(STATE, design.name()) {
        eprintln!("desktop: couldn't save the choice: {error}");
    }
}

// back to the one in use, after a preview that wasn't kept
pub fn cancel() {
    let mut desktop = Desktop::write();

    if desktop.shown != desktop.current {
        desktop.shown = desktop.current;
    }
}

// what a design shows, read once per draw so thumbnails and the page agree
pub struct Moment {
    pub time: String,
    pub date: Date,
    pub hour: u32,
    pub minute: u32,

    // icon and temperature, once a forecast is in
    pub weather: Option<String>,

    // the instant in UTC, and how far local time is ahead of it, for the sun and the moon
    pub epoch_seconds: i64,
    pub utc_offset_minutes: i64,

    // where the weather is for, so also where the sun rises
    pub place: Place,

    // the whole forecast, once one is in
    pub forecast: Option<Forecast>,
}

// one service at a time, each copied out before the next is read
pub fn moment() -> Moment {
    let (time, date, hour, minute) = {
        let clock = Clock::read();

        (
            clock.time().to_string(),
            clock.today(),
            clock.hour(),
            clock.minute(),
        )
    };

    let (weather, place, full) = {
        let weather = Weather::read();

        let now = weather.forecast().map(|forecast| {
            let (icon, _) = weather_forecast::describe(forecast.now.code, forecast.now.daytime);

            format!(
                "{icon} {}",
                weather_forecast::degrees(forecast.now.temperature)
            )
        });

        (now, weather.place().clone(), weather.forecast().cloned())
    };

    Moment {
        time,
        date,
        hour,
        minute,
        weather,
        epoch_seconds: clock::epoch_seconds(),
        utc_offset_minutes: clock::utc_offset_minutes(),
        place,
        forecast: full,
    }
}

// the area a design draws in: the monitor, or a thumbnail of it
#[derive(Debug, Clone, Copy)]
pub struct Page {
    pub width: f32,
    pub height: f32,

    // the bar's height at this size; designs start below it
    pub top: f32,

    // reference sizes times this give sizes on this page
    unit: f32,
}

impl Page {
    pub fn new(width: f32, height: f32) -> Self {
        let unit = height / REFERENCE_HEIGHT;

        Self {
            width,
            height,
            top: bar::HEIGHT * unit,
            unit,
        }
    }

    // a fixed size from the reference, like a gap or a rule, at this page's size
    pub fn scaled(&self, size: f32) -> f32 {
        (size * self.unit).max(THINNEST)
    }

    // the part below the bar
    pub fn body_height(&self) -> f32 {
        self.height - self.top
    }
}

// the background of every monitor while the wallpaper is none, in the design shown
pub fn view(monitor: &Monitor) -> Stack {
    let design = Desktop::read().shown();

    // before the clock is read, so the background (a picture too) doesn't redraw every minute for nothing
    if design == Design::None {
        return Stack::new(Vec::new()).width(Parent).height(Parent);
    }

    let moment = moment();

    page(
        design,
        Page::new(monitor.width as f32, monitor.height as f32),
        &moment,
        &theme(),
    )
}

// a design on a page; the picker calls this at thumbnail size
pub fn page(design: Design, page: Page, moment: &Moment, t: &Theme) -> Stack {
    let mut layers = match design {
        // the bare background: no ticks either
        Design::None => return Stack::new(Vec::new()).width(Parent).height(Parent),

        Design::Ledger => ledger::layers(page, moment, t),
        Design::Almanac => almanac::layers(page, moment, t),
        Design::Prompt => prompt::layers(page, moment, t),
        Design::Lunar => lunar::layers(page, moment, t),
        Design::Forecast => forecast::layers(page, moment, t),
        Design::Epoch => epoch::layers(page, moment, t),
        Design::Orbit => orbit::layers(page, moment, t),
        Design::Ridges => ridges::layers(page, moment, t),
        Design::Pines => pines::layers(page, moment, t),
    };

    // last, so the ticks frame the page over whatever a design draws in its corners
    layers.extend(corner_ticks(page, t));

    Stack::new(layers).width(Parent).height(Parent)
}

// a short line at the top left and bottom right corner of the page, like crop marks
fn corner_ticks(page: Page, t: &Theme) -> Vec<Box<dyn Widget>> {
    let inset = CORNER_INSET * page.height;
    let tick = CORNER_TICK * page.height;
    let hairline = page.scaled(HAIRLINE);

    let left = inset;
    let top = page.top + inset;
    let right = page.width - inset;
    let bottom = page.height - inset;

    let line = |x: f32, y: f32, line_width: f32, line_height: f32| -> Box<dyn Widget> {
        Box::new(placed(
            x,
            y,
            Rectangle::new()
                .width(line_width)
                .height(line_height)
                .fill(t.bg4),
        ))
    };

    vec![
        line(left, top, tick, hairline),
        line(left, top, hairline, tick),
        line(right - tick, bottom - hairline, tick, hairline),
        line(right - hairline, bottom - tick, hairline, tick),
    ]
}

// a widget moved to x, y of the page, as big as the page so it never gets squeezed
fn placed(x: f32, y: f32, child: impl Widget + 'static) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(Start, Start)
        .translate(x, y)
        .child(child)
}

fn text_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * CHARACTER_WIDTH
}

// like fri 20 nov
fn short_date(date: Date) -> String {
    let month: String = calendar::MONTHS[date.month as usize - 1]
        .chars()
        .take(MONTH_LETTERS)
        .collect();

    format!(
        "{} {:02} {}",
        DAY_NAMES[date.weekday() as usize].to_lowercase(),
        date.day,
        month.to_lowercase()
    )
}

// like today, tomorrow or in 4 days
fn in_days(days: i64) -> String {
    match days {
        0 => String::from("today"),
        1 => String::from("tomorrow"),
        _ => format!("in {days} days"),
    }
}

// the local date some (part of a) days after the moment
fn date_after(moment: &Moment, days: f64) -> Date {
    let seconds = moment.epoch_seconds as f64
        + moment.utc_offset_minutes as f64 * SECONDS_PER_MINUTE
        + days * SECONDS_PER_DAY;

    calendar::civil_from_days((seconds / SECONDS_PER_DAY).floor() as i64)
}

/*
 * random numbers for the generative designs, seeded by the day and the
 * design, so a piece stays the same through every redraw of a day (and in its
 * thumbnail) and is new the next morning. xorshift64*, plenty for drawing
 */
struct Random(u64);

impl Random {
    fn for_day(date: Date, salt: u64) -> Self {
        let day = date.days_since_epoch() as u64;

        // never zero, which xorshift can't leave
        let seed = (day ^ salt.rotate_left(32)).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;

        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;

        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    // 0 up to but not including 1
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    // somewhere from `from` up to `to`
    fn between(&mut self, from: f32, to: f32) -> f32 {
        from + (to - from) * self.unit()
    }
}

// the art designs' quiet clock: the time, small, at the bottom left corner tick
fn signature(page: Page, moment: &Moment, t: &Theme) -> Rectangle {
    let size = SIGNATURE_SIZE * page.height;
    let inset = CORNER_INSET * page.height;

    placed(
        inset,
        page.height - inset - 2.0 * size,
        Text::new(moment.time.clone()).size(size).color(t.comment),
    )
}

const SIGNATURE_SIZE: f32 = 0.012;

const SECONDS_PER_MINUTE: f64 = 60.0;
const SECONDS_PER_DAY: f64 = 86_400.0;

// like MON 16 NOV 2026
fn date_line(date: Date) -> String {
    let month: String = calendar::MONTHS[date.month as usize - 1]
        .chars()
        .take(MONTH_LETTERS)
        .collect();

    format!(
        "{} {:02} {} {}",
        DAY_NAMES[date.weekday() as usize],
        date.day,
        month.to_uppercase(),
        date.year
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_lines() {
        assert_eq!(date_line(Date::new(2026, 11, 16)), "MON 16 NOV 2026");
        assert_eq!(date_line(Date::new(2027, 1, 3)), "SUN 03 JAN 2027");
        assert_eq!(short_date(Date::new(2026, 11, 20)), "fri 20 nov");
    }

    #[test]
    fn days_ahead() {
        assert_eq!(in_days(0), "today");
        assert_eq!(in_days(1), "tomorrow");
        assert_eq!(in_days(35), "in 35 days");
    }

    #[test]
    fn dates_after_a_moment_are_local() {
        let moment = Moment {
            time: String::from("23:30"),
            date: Date::new(2026, 11, 16),
            hour: 23,
            minute: 30,
            weather: None,
            // 2026-11-16 22:30 UTC, 23:30 here
            epoch_seconds: 1_794_868_200,
            utc_offset_minutes: 60,
            place: Place::default(),
            forecast: None,
        };

        assert_eq!(date_after(&moment, 0.0), Date::new(2026, 11, 16));
        assert_eq!(date_after(&moment, 0.25), Date::new(2026, 11, 17));
        assert_eq!(date_after(&moment, 4.0), Date::new(2026, 11, 20));
    }

    #[test]
    fn text_widths_are_monospaced() {
        assert_eq!(text_width("320", 100.0), 180.0);
        assert_eq!(text_width("", 100.0), 0.0);
    }

    #[test]
    fn designs_by_name() {
        for design in Design::ALL {
            assert_eq!(Design::from_name(design.name()), Some(design));
        }

        assert_eq!(Design::from_name("lunar\n"), Some(Design::Lunar));

        // a design that was removed falls back to the default
        assert_eq!(Design::from_name("rings"), None);
        assert_eq!(Design::from_name("gone"), None);
    }

    #[test]
    fn pages_scale_from_the_reference() {
        let full = Page::new(2560.0, 1440.0);

        assert_eq!(full.scaled(4.0), 4.0);
        assert_eq!(full.top, bar::HEIGHT);

        // a thumbnail keeps its hairlines
        let thumbnail = Page::new(240.0, 135.0);

        assert_eq!(thumbnail.scaled(1.0), THINNEST);
        assert!(thumbnail.scaled(100.0) < 10.0);
    }
}
