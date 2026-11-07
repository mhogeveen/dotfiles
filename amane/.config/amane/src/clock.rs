/*
 * the time and date in the bar, and today's date for the calendar; click the
 * clock to open the calendar
 *
 * `date` gives the local time, timezone and daylight saving included. it runs
 * once a minute, right after the minute turns, instead of every second: the
 * thread sleeps until the next whole minute by the wall clock, so it stays in
 * step after a suspend or when the time is set (see sleep_until_next_minute)
 */

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amane::{
    Align::Center, Cursor::Pointer, Rectangle, Row, Service, Size::Parent, Text, children,
};

use crate::calendar::Date;
use crate::panels::{CALENDAR, Panels};
use crate::theme::Theme;

// read it in a view to redraw once a minute, like the notification center's "5m ago"
#[derive(PartialEq)]
pub struct Clock {
    time: String,
    date: String,
    today: Date,
}

fn now() -> Clock {
    let line = amane::output("date '+%H:%M|%a %d %b|%Y %m %d'");

    let mut parts = line.trim().split('|');
    let time = parts.next().unwrap_or("").to_string();
    let date = parts.next().unwrap_or("").to_string();

    let mut numbers = parts.next().unwrap_or("").split_whitespace();
    let mut number = || numbers.next().and_then(|number| number.parse().ok());

    let today = match (number(), number(), number()) {
        (Some(year), Some(month), Some(day)) => Date::new(year as i32, month, day),
        _ => Date::new(1970, 1, 1),
    };

    Clock { time, date, today }
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
    let clock = Clock::read();

    Rectangle::new()
        .width(300.0)
        .height(Parent)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Panels::toggle(CALENDAR);
        })
        .child(
            Row::new(children![
                Text::new(&clock.time).color(t.fg_dim).size(14.0),
                Text::new(&clock.date).color(t.fg_dim).size(12.0)
            ])
            .align(Center)
            .gap(8.0),
        )
}

// a little past the minute, so `date` surely reads the new one
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

#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
mod wall {
    use std::ffi::c_int;
    use std::time::Duration;

    // glibc's struct timespec on 64-bit linux
    #[repr(C)]
    struct Timespec {
        seconds: i64,
        nanoseconds: i64,
    }

    const CLOCK_REALTIME: c_int = 0;
    const TIMER_ABSTIME: c_int = 1;

    // the C library's own, which the config may call since it can't add crates
    unsafe extern "C" {
        fn clock_nanosleep(
            clock: c_int,
            flags: c_int,
            request: *const Timespec,
            remain: *mut Timespec,
        ) -> c_int;
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

    pub fn sleep_until(_: Duration) {
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
