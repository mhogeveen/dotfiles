/*
 * the weather beside the calendar, from Open-Meteo: no account or key, and it
 * answers in csv (forecast.rs reads it)
 *
 * opening the calendar fetches when the forecast is over 30 minutes old, and the
 * bar's clock, which shows the weather now, when it's over 2 hours old; the
 * place is set with `amane ipc call weather location <latitude> <longitude> <name>`
 * and kept in ~/.local/state/amane-shell/weather, Hilversum until then
 */

pub mod forecast;

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::command::{self, Failure};
use crate::{clock, state};

pub use forecast::Forecast;

const STATE: &str = "weather";

// how old a forecast may get before opening the calendar fetches a new one
const STALE: Duration = Duration::from_secs(30 * 60);

// how long after a failed fetch opening the calendar tries again
const RETRY: Duration = Duration::from_secs(60);

/*
 * the same for the bar, which asks every minute without anyone looking: the
 * weather doesn't change much in 2 hours, and offline curl runs every 10 minutes, not every minute
 */
const BAR_STALE: Duration = Duration::from_secs(2 * 60 * 60);
const BAR_RETRY: Duration = Duration::from_secs(10 * 60);

const API: &str = "https://api.open-meteo.com/v1/forecast";

// what's asked for; forecast.rs finds each by name in the answer
const QUERY: &str = "current=temperature_2m,apparent_temperature,weather_code,wind_speed_10m,wind_direction_10m,is_day\
&hourly=temperature_2m,weather_code,precipitation_probability,is_day&forecast_hours=24\
&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&forecast_days=7\
&timezone=auto&format=csv";

#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub latitude: f64,
    pub longitude: f64,
    pub name: String,
}

impl Default for Place {
    fn default() -> Self {
        Self {
            latitude: 52.223,
            longitude: 5.176,
            name: String::from("Hilversum"),
        }
    }
}

pub struct Weather {
    place: Place,
    forecast: Option<Forecast>,

    // when the forecast shown was fetched, as the clock on the wall says it, like 19:15
    fetched_at: Option<String>,
    fetched: Option<Instant>,

    // why and when the last fetch failed; the older forecast, if any, stays
    error: Option<String>,
    failed: Option<Instant>,

    // a fetch is due on the next check even if the forecast is fresh
    forced: bool,

    // a fetch is running; kept here, unlike FETCHING, so the panel redraws when it changes
    fetching: bool,
}

impl Service for Weather {
    fn new() -> Self {
        let place = fs::read_to_string(state::path(STATE))
            .ok()
            .and_then(|text| parse_place(&text))
            .unwrap_or_default();

        Self {
            place,
            forecast: None,
            fetched_at: None,
            fetched: None,
            error: None,
            failed: None,
            forced: false,
            fetching: false,
        }
    }

    // fetched on demand, nothing to poll
    fn listen() {}
}

// reading, safe inside views
impl Weather {
    pub fn place(&self) -> &Place {
        &self.place
    }

    pub fn forecast(&self) -> Option<&Forecast> {
        self.forecast.as_ref()
    }

    pub fn fetched_at(&self) -> Option<&str> {
        self.fetched_at.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    // a fetch is running or asked for, so the panel can say so
    pub fn refreshing(&self) -> bool {
        self.fetching || self.forced
    }

    fn stale(&self, window: Window) -> bool {
        due(
            window,
            self.forced,
            self.fetched.map(|fetched| fetched.elapsed()),
            self.failed.map(|failed| failed.elapsed()),
        )
    }
}

// how old a forecast may get, and how soon a failed fetch is tried again
#[derive(Debug, Clone, Copy)]
struct Window {
    stale: Duration,
    retry: Duration,
}

const CALENDAR: Window = Window {
    stale: STALE,
    retry: RETRY,
};

const BAR: Window = Window {
    stale: BAR_STALE,
    retry: BAR_RETRY,
};

/*
 * whether to fetch, from how long ago the last good and the last failed fetch
 * were: a failure is tried again after the retry rather than the stale time, so
 * a calendar opened while offline doesn't keep saying so for half an hour
 */
fn due(
    window: Window,
    forced: bool,
    since_fetched: Option<Duration>,
    since_failed: Option<Duration>,
) -> bool {
    if forced {
        return true;
    }

    if let Some(since_failed) = since_failed {
        return since_failed > window.retry;
    }

    since_fetched.is_none_or(|since_fetched| since_fetched > window.stale)
}

// one fetch at a time; not part of the service, so starting one redraws nothing
static FETCHING: AtomicBool = AtomicBool::new(false);

// for the calendar, when it opens
pub fn refresh_if_stale() {
    refresh_within(CALENDAR);
}

// for the bar's clock, asked every minute
pub fn refresh_for_bar() {
    refresh_within(BAR);
}

/*
 * fetches when the forecast is old or missing; safe to call from a view, as it
 * only starts a thread, which writes the result when it's done
 */
fn refresh_within(window: Window) {
    let (stale, place) = {
        let weather = Weather::read();

        (weather.stale(window), weather.place.clone())
    };

    if !stale || FETCHING.swap(true, Ordering::AcqRel) {
        return;
    }

    thread::spawn(move || {
        // cleared as the fetch starts, so a refresh asked for during it makes another one
        {
            let mut weather = Weather::write();

            weather.forced = false;
            weather.fetching = true;
        }

        let result = fetch(&place);

        {
            let mut weather = Weather::write();

            weather.fetching = false;

            // the place changed while fetching, the answer is for the old one
            if weather.place == place {
                match result {
                    Ok(forecast) => {
                        weather.forecast = Some(forecast);
                        weather.fetched = Some(Instant::now());
                        weather.fetched_at = Some(wall_clock());
                        weather.error = None;
                        weather.failed = None;
                    }
                    Err(error) => {
                        weather.error = Some(error);
                        weather.failed = Some(Instant::now());
                    }
                }
            }
        }

        FETCHING.store(false, Ordering::Release);
    });
}

// the next check fetches, fresh or not (the bar asks every minute); from a key or ipc
pub fn refresh() {
    Weather::write().forced = true;
}

fn fetch(place: &Place) -> Result<Forecast, String> {
    let url = format!(
        "{API}?latitude={}&longitude={}&{QUERY}",
        place.latitude, place.longitude
    );

    let output = command::capture(
        Command::new("curl")
            .args(["--silent", "--show-error", "--fail"])
            .args(["--max-time", command::CURL_MAX_TIME])
            .arg(&url),
        command::CURL_TIMEOUT,
    )
    .map_err(|failure| match failure {
        Failure::Start => String::from("couldn't run curl"),
        Failure::Timeout => String::from("no answer in time"),
    })?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);

        return Err(curl_error(error.trim()));
    }

    forecast::parse(&String::from_utf8_lossy(&output.stdout))
}

// curl's errors are long; the ones that happen most get a short version
fn curl_error(error: &str) -> String {
    if error.contains("Could not resolve host") || error.contains("Couldn't connect") {
        String::from("no connection")
    } else if error.contains("timed out") {
        String::from("no answer in time")
    } else if error.is_empty() {
        String::from("fetching failed")
    } else {
        String::from(error)
    }
}

// like 19:15
fn wall_clock() -> String {
    clock::local_now().time()
}

// latitude<TAB>longitude<TAB>name
fn parse_place(text: &str) -> Option<Place> {
    let mut fields = text.trim().splitn(3, '\t');

    let latitude = fields.next()?.parse().ok()?;
    let longitude = fields.next()?.parse().ok()?;
    let name = fields.next().unwrap_or("").trim();

    valid(latitude, longitude).then(|| Place {
        latitude,
        longitude,
        name: String::from(name),
    })
}

fn valid(latitude: f64, longitude: f64) -> bool {
    (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude)
}

const USAGE: &str =
    "usage: weather location <latitude> <longitude> <name> | weather refresh | weather";

/*
 * `amane ipc call weather location 52.22 5.18 Hilversum` sets the place,
 * `weather refresh` fetches within a minute, `weather` says where it's for
 */
pub fn ipc(arguments: &[String]) -> String {
    match arguments.first().map(String::as_str) {
        None => {
            let place = Weather::read().place.clone();

            format!("{} ({}, {})", place.name, place.latitude, place.longitude)
        }

        Some("refresh") => {
            refresh();

            String::from("fetching within a minute")
        }

        Some("location") => set_location(&arguments[1..]),

        Some(_) => String::from(USAGE),
    }
}

fn set_location(arguments: &[String]) -> String {
    let (Some(latitude), Some(longitude)) = (arguments.first(), arguments.get(1)) else {
        return String::from(USAGE);
    };

    let (Ok(latitude), Ok(longitude)) = (latitude.parse::<f64>(), longitude.parse::<f64>()) else {
        return String::from("latitude and longitude should be numbers, like 52.22 5.18");
    };

    if !valid(latitude, longitude) {
        return String::from("latitude goes from -90 to 90, longitude from -180 to 180");
    }

    let name = arguments[2..].join(" ");

    let place = Place {
        latitude,
        longitude,
        name: name.clone(),
    };

    if let Err(error) = state::save(
        STATE,
        &format!("{}\t{}\t{}\n", place.latitude, place.longitude, place.name),
    ) {
        eprintln!("weather: couldn't save the location: {error}");
    }

    {
        let mut weather = Weather::write();

        weather.place = place;
        weather.forecast = None;
        weather.fetched = None;
        weather.failed = None;
        weather.fetched_at = None;
        weather.error = None;
    }

    let name = if name.is_empty() {
        String::from("there")
    } else {
        name
    };

    format!("weather for {name} within a minute")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: Duration = Duration::from_secs(60);

    #[test]
    fn a_fetch_is_due_when_old_missing_or_forced() {
        assert!(due(CALENDAR, false, None, None));
        assert!(due(CALENDAR, false, Some(STALE + MINUTE), None));
        assert!(!due(CALENDAR, false, Some(MINUTE), None));
        assert!(due(CALENDAR, true, Some(MINUTE), None));
    }

    #[test]
    fn a_failure_is_tried_again_soon() {
        // offline at the first open: not again at once, but well before STALE
        assert!(!due(CALENDAR, false, None, Some(Duration::from_secs(10))));
        assert!(due(CALENDAR, false, None, Some(RETRY + MINUTE)));

        // a failure after a good fetch retries too, the old forecast staying
        assert!(due(CALENDAR, false, Some(MINUTE), Some(RETRY + MINUTE)));
    }

    #[test]
    fn the_bar_waits_longer() {
        // a forecast the calendar would fetch again is still fine for the bar
        assert!(!due(BAR, false, Some(STALE + MINUTE), None));
        assert!(due(BAR, false, Some(BAR_STALE + MINUTE), None));
        assert!(due(BAR, false, None, None));

        // offline, it doesn't try every minute
        assert!(!due(BAR, false, None, Some(RETRY + MINUTE)));
        assert!(due(BAR, false, None, Some(BAR_RETRY + MINUTE)));
    }

    #[test]
    fn places_read_back() {
        assert_eq!(
            parse_place("52.223\t5.176\tHilversum\n"),
            Some(Place::default())
        );
        assert_eq!(
            parse_place("51.92\t4.48\tRotterdam Centrum"),
            Some(Place {
                latitude: 51.92,
                longitude: 4.48,
                name: String::from("Rotterdam Centrum"),
            })
        );
        assert_eq!(parse_place("91\t0\tNowhere"), None);
        assert_eq!(parse_place("not a place"), None);
    }

    #[test]
    fn curl_errors_get_short() {
        assert_eq!(
            curl_error("curl: (6) Could not resolve host: api.open-meteo.com"),
            "no connection"
        );
        assert_eq!(
            curl_error("curl: (28) Connection timed out after 10001 milliseconds"),
            "no answer in time"
        );
    }
}
