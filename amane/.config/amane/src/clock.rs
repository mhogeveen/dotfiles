use amane::{Align::Center, Rectangle, Row, Service, Size::Parent, Text, children};

use crate::theme::Theme;

// read it in a view to redraw once a minute, like the notification center's "5m ago"
pub struct Clock {
    time: String,
    date: String,
}

fn now() -> Clock {
    let line = amane::output("date '+%H:%M|%a %d %b'");

    let mut parts = line.trim().split('|');
    let time = parts.next().unwrap_or("").to_string();
    let date = parts.next().unwrap_or("").to_string();

    Clock {
        time: time,
        date: date,
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
}

impl Service for Clock {
    fn new() -> Self {
        now()
    }

    // polled every second (the default interval), but only redraws when the minute changes
    fn update(&mut self) -> bool {
        let fresh = now();

        if fresh.time == self.time {
            return false;
        }

        self.time = fresh.time;
        self.date = fresh.date;

        true
    }
}

pub fn view(t: &Theme) -> Rectangle {
    let clock = Clock::read();

    Rectangle::new()
        .width(300.0)
        .height(Parent)
        .align_child(Center, Center)
        .child(
            Row::new(children![
                Text::new(&clock.time).color(t.fg_dim).size(14.0),
                Text::new(&clock.date).color(t.fg_dim).size(12.0)
            ])
            .align(Center)
            .gap(8.0),
        )
}
