/*
 * the time and date in the bar, then the weather now (icon and temperature,
 * once a forecast is in), and today's date for the calendar; click the clock to
 * open the calendar
 *
 * the C library's localtime_r gives the local time, timezone and daylight
 * saving included (no `date` to start, which could fail). it's read once a
 * minute, right after the minute turns, instead of every second: the thread
 * sleeps until the next whole minute by the wall clock, so it stays in step
 * after a suspend or when the time is set (see sleep_until_next_minute)
 */

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amane::{
    Align::Center, Cursor::Pointer, Rectangle, Row, Service, Size::Parent, Text, Widget, children,
};

use crate::calendar::{self, Date};
use crate::panels::{CALENDAR, Panels, ui};
use crate::theme::Theme;
use crate::weather::{self, Weather, forecast};

// read it in a view to redraw once a minute, like the notification center's "5m ago"
#[derive(PartialEq)]
pub struct Clock {
    time: String,
    date: String,
    today: Date,
}

// in english, like the calendar panel, whatever LC_TIME says
const DAY_NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

// how many letters of a month's name the bar shows: Nov
const MONTH_LETTERS: usize = 3;

// the clock in the bar: a fixed width, so it stays centered as the text changes
const WIDTH: f32 = 300.0;
const GAP: f32 = 8.0;

// between the date and the weather
const SEPARATOR: &str = "|";

// nerd font icons draw wider than they measure, so the weather's icon gets a box of its own
const WEATHER_ICON: f32 = 16.0;

// the local time, timezone and daylight saving included, read from the C library
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    pub date: Date,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl LocalTime {
    // like 17:05
    pub fn time(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    // like Wed 11 Nov
    fn short_date(&self) -> String {
        let month: String = calendar::MONTHS[self.date.month as usize - 1]
            .chars()
            .take(MONTH_LETTERS)
            .collect();

        format!(
            "{} {:02} {month}",
            DAY_NAMES[self.date.weekday() as usize],
            self.date.day
        )
    }

    // like 20261111171503, for file names that sort by time
    pub fn stamp(&self) -> String {
        format!(
            "{:04}{:02}{:02}{:02}{:02}{:02}",
            self.date.year, self.date.month, self.date.day, self.hour, self.minute, self.second
        )
    }
}

pub fn local_now() -> LocalTime {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    wall::local(since_epoch.as_secs() as i64)
}

fn now() -> Clock {
    let local = local_now();

    Clock {
        time: local.time(),
        date: local.short_date(),
        today: local.date,
    }
}

impl Clock {
    // like 17:05
    pub fn time(&self) -> &str {
        &self.time
    }

    // like Fri 06 Nov
    pub fn date(&self) -> &str {
        &self.date
    }

    // changes at midnight, which is a minute turning, so the calendar keeps up
    pub fn today(&self) -> Date {
        self.today
    }
}

impl Service for Clock {
    fn new() -> Self {
        now()
    }

    // asked again only as each minute starts, and written only when it shows something new
    fn listen() {
        loop {
            sleep_until_next_minute();

            let fresh = now();

            if *Clock::read() != fresh {
                *Clock::write() = fresh;
            }
        }
    }
}

pub fn view(t: &Theme) -> Rectangle {
    // only starts a thread; this view redraws every minute, so that's how often it's asked
    weather::refresh_for_bar();

    // one service at a time: the weather is copied out before the clock is read
    let now = Weather::read().forecast().map(|forecast| {
        let (icon, _) = forecast::describe(forecast.now.code, forecast.now.daytime);

        (icon, forecast::degrees(forecast.now.temperature))
    });

    let (time, date) = {
        let clock = Clock::read();

        (clock.time.clone(), clock.date.clone())
    };

    let mut parts: Vec<Box<dyn Widget>> = children![
        Text::new(time).color(t.fg_dim).size(ui::TEXT_BODY),
        Text::new(date).color(t.fg_dim).size(ui::TEXT_SMALL)
    ];

    // nothing until the first forecast is in, so the clock looks as it did
    if let Some((icon, temperature)) = now {
        parts.push(Box::new(
            Text::new(SEPARATOR).color(t.bg3).size(ui::TEXT_SMALL),
        ));

        parts.push(Box::new(
            Rectangle::new()
                .width(WEATHER_ICON)
                .height(Parent)
                .align_child(Center, Center)
                .child(Text::new(icon).color(t.fg_dim).size(ui::TEXT_BODY)),
        ));

        parts.push(Box::new(
            Text::new(temperature).color(t.fg_dim).size(ui::TEXT_SMALL),
        ));
    }

    Rectangle::new()
        .width(WIDTH)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(CALENDAR);
        })
        .child(Row::new(parts).align(Center).gap(GAP).height(Parent))
}

// a little past the minute, so the new one is surely read
const MARGIN: Duration = Duration::from_millis(20);

/*
 * an ordinary sleep counts time the machine is running, so after a suspend it
 * would wake late and show an old time for up to a minute. this sleeps until a
 * moment on the wall clock (CLOCK_REALTIME with TIMER_ABSTIME), which the kernel
 * checks again after a resume and when the time is set, so it wakes on time
 *
 * minutes turn at the same moment in local time and UTC, as no timezone in use
 * is offset by part of a minute
 */
fn sleep_until_next_minute() {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let next = (since_epoch.as_secs() / 60 + 1) * 60;
    let wake = Duration::from_secs(next) + MARGIN;

    wall::sleep_until(wake);
}

// without the C library's answer: the time in UTC, still a valid time
fn utc(since_epoch: i64) -> LocalTime {
    let seconds_of_day = since_epoch.rem_euclid(SECONDS_PER_DAY) as u32;

    LocalTime {
        date: calendar::civil_from_days(since_epoch.div_euclid(SECONDS_PER_DAY)),
        hour: seconds_of_day / 3600,
        minute: seconds_of_day / 60 % 60,
        second: seconds_of_day % 60,
    }
}

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
mod wall {
    use std::ffi::{c_char, c_int, c_long};
    use std::time::Duration;

    use super::LocalTime;
    use crate::calendar::Date;

    // glibc's struct timespec on 64-bit linux
    #[repr(C)]
    struct Timespec {
        seconds: i64,
        nanoseconds: i64,
    }

    const CLOCK_REALTIME: c_int = 0;
    const TIMER_ABSTIME: c_int = 1;

    // glibc's struct tm on 64-bit linux
    #[repr(C)]
    struct Tm {
        second: c_int,
        minute: c_int,
        hour: c_int,
        day: c_int,

        // 0 to 11
        month: c_int,

        // since 1900
        year: c_int,
        weekday: c_int,
        yearday: c_int,
        daylight_saving: c_int,
        offset: c_long,
        zone: *const c_char,
    }

    // the C library's own, which the config may call since it can't add crates
    unsafe extern "C" {
        fn clock_nanosleep(
            clock: c_int,
            flags: c_int,
            request: *const Timespec,
            remain: *mut Timespec,
        ) -> c_int;

        fn tzset();

        fn localtime_r(time: *const i64, result: *mut Tm) -> *mut Tm;
    }

    // this many seconds after 1970 in the local timezone
    pub fn local(since_epoch: i64) -> LocalTime {
        let mut tm = Tm {
            second: 0,
            minute: 0,
            hour: 0,
            day: 1,
            month: 0,
            year: 70,
            weekday: 0,
            yearday: 0,
            daylight_saving: 0,
            offset: 0,
            zone: std::ptr::null(),
        };

        /*
         * SAFETY: tzset only reads TZ and /etc/localtime; it's asked each time so
         * a changed timezone is followed, like `date` did. localtime_r gets a valid
         * time and a struct tm to fill, and touches nothing else
         */
        let filled = unsafe {
            tzset();

            !localtime_r(&since_epoch, &mut tm).is_null()
        };

        if !filled {
            return super::utc(since_epoch);
        }

        LocalTime {
            date: Date::new(1900 + tm.year, (tm.month + 1) as u32, tm.day as u32),
            hour: tm.hour as u32,
            minute: tm.minute as u32,
            second: tm.second as u32,
        }
    }

    // until this long after 1970 by the wall clock
    pub fn sleep_until(since_epoch: Duration) {
        let request = Timespec {
            seconds: since_epoch.as_secs() as i64,
            nanoseconds: i64::from(since_epoch.subsec_nanos()),
        };

        // a signal can cut it short, an absolute time is simply asked for again
        loop {
            // SAFETY: request is a valid timespec, and remain may be null with TIMER_ABSTIME
            let result = unsafe {
                clock_nanosleep(
                    CLOCK_REALTIME,
                    TIMER_ABSTIME,
                    &request,
                    std::ptr::null_mut(),
                )
            };

            // 4 is EINTR
            if result != 4 {
                return;
            }
        }
    }
}

// elsewhere the wall clock isn't reachable, so check every second like before
#[cfg(not(all(target_os = "linux", target_pointer_width = "64")))]
mod wall {
    use std::time::Duration;

    use super::LocalTime;

    pub fn sleep_until(_: Duration) {
        std::thread::sleep(Duration::from_secs(1));
    }

    pub fn local(since_epoch: i64) -> LocalTime {
        super::utc(since_epoch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_are_written_for_the_bar_and_file_names() {
        let local = LocalTime {
            date: Date::new(2026, 11, 11),
            hour: 9,
            minute: 5,
            second: 3,
        };

        assert_eq!(local.time(), "09:05");
        assert_eq!(local.short_date(), "Wed 11 Nov");
        assert_eq!(local.stamp(), "20261111090503");
    }

    #[test]
    fn utc_is_counted_from_1970() {
        // 2026-11-11 08:05:03 UTC
        let utc = utc(1_794_384_303);

        assert_eq!(utc.date, Date::new(2026, 11, 11));
        assert_eq!((utc.hour, utc.minute, utc.second), (8, 5, 3));
    }

    #[test]
    fn the_local_time_matches_date() {
        let date = || {
            let output = std::process::Command::new("date")
                .arg("+%Y %m %d %H %M")
                .output()
                .expect("date runs");

            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        // asked before and after, so a minute turning in between can't fail it
        let before = date();
        let local = local_now();
        let after = date();

        let ours = format!(
            "{:04} {:02} {:02} {:02} {:02}",
            local.date.year, local.date.month, local.date.day, local.hour, local.minute
        );

        assert!(
            ours == before || ours == after,
            "{ours} is neither {before} nor {after}"
        );
    }

    #[test]
    fn sleeps_until_the_wall_clock_time() {
        let start = SystemTime::now();
        let target = start.duration_since(UNIX_EPOCH).unwrap() + Duration::from_millis(200);

        wall::sleep_until(target);

        let woke = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        assert!(woke >= target);
        assert!(woke - target < Duration::from_millis(100));
    }
}
