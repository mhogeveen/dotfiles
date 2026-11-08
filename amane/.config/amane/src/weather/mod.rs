/*
 * the weather beside the calendar, from Open-Meteo: no account or key, and it
 * answers in csv (forecast.rs reads it)
 *
 * it's fetched only while the calendar shows, and at most every 30 minutes; the
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

use crate::state;

pub use forecast::Forecast;

const STATE: &str = "weather";

// how old a forecast may get before opening the calendar fetches a new one
const STALE: Duration = Duration::from_secs(30 * 60);

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

    // why the last fetch failed; the older forecast, if any, stays
    error: Option<String>,

    // a fetch is due on the next open even if the forecast is fresh
    forced: bool,
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
            forced: false,
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

    fn stale(&self) -> bool {
        self.forced || self.fetched.is_none_or(|fetched| fetched.elapsed() > STALE)
    }
}

// one fetch at a time; not part of the service, so starting one redraws nothing
static FETCHING: AtomicBool = AtomicBool::new(false);

/*
 * fetches when the forecast is old or missing; safe to call from a view, as it
 * only starts a thread, which writes the result when it's done
 */
pub fn refresh_if_stale() {
    let (stale, place) = {
        let weather = Weather::read();

        (weather.stale(), weather.place.clone())
    };

    if !stale || FETCHING.swap(true, Ordering::AcqRel) {
        return;
    }

    thread::spawn(move || {
        let result = fetch(&place);

        {
            let mut weather = Weather::write();

            // the place changed while fetching, the answer is for the old one
            if weather.place == place {
                weather.fetched = Some(Instant::now());
                weather.forced = false;

                match result {
                    Ok(forecast) => {
                        weather.forecast = Some(forecast);
                        weather.fetched_at = Some(wall_clock());
                        weather.error = None;
                    }
                    Err(error) => weather.error = Some(error),
                }
            }
        }

        FETCHING.store(false, Ordering::Release);
    });
}

// the next open fetches, fresh or not; from a key or ipc
pub fn refresh() {
    Weather::write().forced = true;
}

fn fetch(place: &Place) -> Result<Forecast, String> {
    let url = format!(
        "{API}?latitude={}&longitude={}&{QUERY}",
        place.latitude, place.longitude
    );

    let output = Command::new("curl")
        .args(["--silent", "--show-error", "--fail", "--max-time", "10"])
        .arg(&url)
        .output()
        .map_err(|error| format!("couldn't run curl: {error}"))?;

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
    amane::output("date +%H:%M")
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
 * `weather refresh` fetches on the next open, `weather` says where it's for
 */
pub fn ipc(arguments: &[String]) -> String {
    match arguments.first().map(String::as_str) {
        None => {
            let place = Weather::read().place.clone();

            format!("{} ({}, {})", place.name, place.latitude, place.longitude)
        }

        Some("refresh") => {
            refresh();

            String::from("fetching when the calendar opens")
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
        weather.fetched_at = None;
        weather.error = None;
    }

    let name = if name.is_empty() {
        String::from("there")
    } else {
        name
    };

    format!("weather for {name} from the next time the calendar opens")
}

#[cfg(test)]
mod tests {
    use super::*;

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
