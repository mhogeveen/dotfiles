/*
 * reading Open-Meteo's answer: asked for format=csv, it sends one block per part,
 * separated by blank lines: the place, now, the next 24 hours and the next 7 days
 *
 *   latitude,longitude,elevation,…
 *   52.222,5.162,10.0,…
 *
 *   time,temperature_2m (°C),apparent_temperature (°C),weather_code (wmo code),…
 *   2026-11-07T19:15,4.1,1.3,1,…
 *
 * columns are found by their name, not their place, so asking for another one
 * can't shift the rest
 */

use crate::calendar::Date;

#[derive(Debug, Clone, PartialEq)]
pub struct Now {
    pub temperature: f64,
    pub feels_like: f64,
    pub code: u32,

    // km/h
    pub wind: f64,

    // where the wind comes from, in degrees
    pub direction: f64,

    pub daytime: bool,
}

// three hours of the next 24
#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    pub hour: u32,
    pub temperature: f64,
    pub code: u32,

    // the highest chance of rain in its three hours, 0 to 100
    pub rain: u32,

    pub daytime: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DayForecast {
    pub date: Date,
    pub code: u32,
    pub high: f64,
    pub low: f64,
    pub rain: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Forecast {
    pub now: Now,
    pub slots: Vec<Slot>,
    pub days: Vec<DayForecast>,
}

const SLOT_HOURS: usize = 3;
const SLOTS: usize = 7;

pub fn parse(csv: &str) -> Result<Forecast, String> {
    let blocks = blocks(csv);

    // the place, now, the hours and the days
    let [_, now, hours, days] = blocks.as_slice() else {
        return Err(String::from("unexpected answer"));
    };

    let now = read_now(now).ok_or("no current weather")?;
    let slots = read_slots(hours).ok_or("no hourly forecast")?;
    let days = read_days(days).ok_or("no daily forecast")?;

    Ok(Forecast { now, slots, days })
}

struct Block {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Block {
    // a column by its name without the unit: "temperature_2m" finds "temperature_2m (°C)"
    fn column(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|column| {
            column == name
                || column
                    .strip_prefix(name)
                    .is_some_and(|rest| rest.starts_with(" ("))
        })
    }

    fn text<'a>(&'a self, row: &'a [String], name: &str) -> Option<&'a str> {
        row.get(self.column(name)?).map(String::as_str)
    }

    fn number(&self, row: &[String], name: &str) -> Option<f64> {
        self.text(row, name)?.parse().ok()
    }
}

fn blocks(csv: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut lines: Vec<&str> = Vec::new();

    for line in csv.lines().chain(std::iter::once("")) {
        if !line.trim().is_empty() {
            lines.push(line.trim());

            continue;
        }

        let Some((header, rows)) = lines.split_first() else {
            continue;
        };

        let split = |line: &str| line.split(',').map(String::from).collect::<Vec<_>>();

        blocks.push(Block {
            columns: split(header),
            rows: rows.iter().map(|row| split(row)).collect(),
        });

        lines.clear();
    }

    blocks
}

fn read_now(block: &Block) -> Option<Now> {
    let row = block.rows.first()?;

    Some(Now {
        temperature: block.number(row, "temperature_2m")?,
        feels_like: block.number(row, "apparent_temperature")?,
        code: block.number(row, "weather_code")? as u32,
        wind: block.number(row, "wind_speed_10m")?,
        direction: block.number(row, "wind_direction_10m")?,
        daytime: block.number(row, "is_day").is_none_or(|day| day != 0.0),
    })
}

// every third hour from now, each with the highest chance of rain in its three hours
fn read_slots(block: &Block) -> Option<Vec<Slot>> {
    let slots: Option<Vec<Slot>> = block
        .rows
        .chunks(SLOT_HOURS)
        .take(SLOTS)
        .map(|hours| {
            let first = hours.first()?;

            // 2026-11-07T19:00 is hour 19
            let hour = block.text(first, "time")?.get(11..13)?.parse().ok()?;

            let rain = hours
                .iter()
                .filter_map(|hour| block.number(hour, "precipitation_probability"))
                .fold(0.0_f64, f64::max);

            Some(Slot {
                hour,
                temperature: block.number(first, "temperature_2m")?,
                code: block.number(first, "weather_code")? as u32,
                rain: rain as u32,
                daytime: block.number(first, "is_day").is_none_or(|day| day != 0.0),
            })
        })
        .collect();

    slots.filter(|slots| !slots.is_empty())
}

fn read_days(block: &Block) -> Option<Vec<DayForecast>> {
    let days: Option<Vec<DayForecast>> = block
        .rows
        .iter()
        .map(|row| {
            Some(DayForecast {
                date: date(block.text(row, "time")?)?,
                code: block.number(row, "weather_code")? as u32,
                high: block.number(row, "temperature_2m_max")?,
                low: block.number(row, "temperature_2m_min")?,
                rain: block
                    .number(row, "precipitation_probability_max")
                    .unwrap_or(0.0) as u32,
            })
        })
        .collect();

    days.filter(|days| !days.is_empty())
}

// 2026-11-07
fn date(text: &str) -> Option<Date> {
    let mut parts = text.splitn(3, '-');

    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.get(..2)?.parse().ok()?;

    Some(Date::new(year, month, day))
}

/*
 * the WMO code Open-Meteo gives, as a nerd font material design glyph and a few
 * words; clear and partly cloudy skies show the moon at night
 */
pub fn describe(code: u32, daytime: bool) -> (&'static str, &'static str) {
    const SUNNY: &str = "\u{f0599}";
    const NIGHT: &str = "\u{f0594}";
    const PARTLY: &str = "\u{f0595}";
    const PARTLY_NIGHT: &str = "\u{f0f31}";
    const CLOUDY: &str = "\u{f0590}";
    const FOG: &str = "\u{f0591}";
    const RAINY: &str = "\u{f0597}";
    const POURING: &str = "\u{f0596}";
    const SNOWY: &str = "\u{f0598}";
    const SLEET: &str = "\u{f067f}";
    const THUNDER: &str = "\u{f067e}";
    const HAIL: &str = "\u{f0592}";

    let clear = if daytime { SUNNY } else { NIGHT };
    let partly = if daytime { PARTLY } else { PARTLY_NIGHT };

    match code {
        0 => (clear, "Clear"),
        1 => (clear, "Mainly clear"),
        2 => (partly, "Partly cloudy"),
        3 => (CLOUDY, "Overcast"),
        45 | 48 => (FOG, "Fog"),
        51 | 53 | 55 => (RAINY, "Drizzle"),
        56 | 57 => (SLEET, "Freezing drizzle"),
        61 | 63 => (RAINY, "Rain"),
        65 => (POURING, "Heavy rain"),
        66 | 67 => (SLEET, "Freezing rain"),
        71 | 73 | 75 | 77 => (SNOWY, "Snow"),
        80 | 81 => (RAINY, "Showers"),
        82 => (POURING, "Heavy showers"),
        85 | 86 => (SNOWY, "Snow showers"),
        95 => (THUNDER, "Thunderstorm"),
        96 | 99 => (HAIL, "Thunderstorm with hail"),
        _ => (CLOUDY, "Unknown"),
    }
}

// where the wind comes from, as one of 16 points: 140° is SE
pub fn compass(degrees: f64) -> &'static str {
    const POINTS: [&str; 16] = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];

    let index = (degrees.rem_euclid(360.0) / 22.5 + 0.5).floor() as usize % POINTS.len();

    POINTS[index]
}

// 4.1 reads as 4°, and -0.3 as 0° rather than -0°
pub fn degrees(celsius: f64) -> String {
    let rounded = celsius.round();

    let rounded = if rounded == 0.0 { 0.0 } else { rounded };

    format!("{rounded:.0}°")
}

#[cfg(test)]
mod tests {
    use super::*;

    // trimmed from a real answer for Hilversum
    const ANSWER: &str = "latitude,longitude,elevation,utc_offset_seconds,timezone,timezone_abbreviation
52.222,5.1619997,10.0,3600,Europe/Amsterdam,GMT+1

time,temperature_2m (°C),apparent_temperature (°C),weather_code (wmo code),wind_speed_10m (km/h),wind_direction_10m (°),is_day ()
2026-11-07T19:15,4.1,1.3,1,6.8,140,0

time,temperature_2m (°C),weather_code (wmo code),precipitation_probability (%),is_day ()
2026-11-07T19:00,4.1,0,0,0
2026-11-07T20:00,4.7,3,0,0
2026-11-07T21:00,5.0,3,0,0
2026-11-07T22:00,5.0,3,0,0
2026-11-07T23:00,5.4,3,5,0
2026-11-08T00:00,5.7,51,11,0
2026-11-08T01:00,6.0,45,18,0
2026-11-08T02:00,6.2,45,24,0
2026-11-08T03:00,6.1,45,31,0

time,weather_code (wmo code),temperature_2m_max (°C),temperature_2m_min (°C),precipitation_probability_max (%)
2026-11-07,3,7.9,0.2,5
2026-11-08,51,8.4,5.7,36
";

    #[test]
    fn reads_now_hours_and_days() {
        let forecast = parse(ANSWER).expect("a forecast");

        assert_eq!(forecast.now.temperature, 4.1);
        assert_eq!(forecast.now.feels_like, 1.3);
        assert_eq!(forecast.now.code, 1);
        assert!(!forecast.now.daytime);

        assert_eq!(forecast.days.len(), 2);
        assert_eq!(forecast.days[1].date, Date::new(2026, 11, 8));
        assert_eq!(forecast.days[1].high, 8.4);
        assert_eq!(forecast.days[1].rain, 36);
    }

    #[test]
    fn hours_go_in_threes_with_their_highest_rain_chance() {
        let slots = parse(ANSWER).expect("a forecast").slots;

        assert_eq!(slots.len(), 3);
        assert_eq!(slots[0].hour, 19);
        assert_eq!(slots[0].code, 0);
        assert_eq!(slots[0].rain, 0);
        assert_eq!(slots[1].hour, 22);
        assert_eq!(slots[1].rain, 11);
        assert_eq!(slots[2].hour, 1);
        assert_eq!(slots[2].rain, 31);
    }

    #[test]
    fn a_full_day_gives_seven_slots_three_hours_apart() {
        let hours: String = (0..24)
            .map(|hour| {
                let (day, hour) = if 19 + hour < 24 {
                    (7, 19 + hour)
                } else {
                    (8, 19 + hour - 24)
                };

                format!("2026-11-{day:02}T{hour:02}:00,5.0,3,0,1\n")
            })
            .collect();

        let answer = ANSWER.replace(
            &ANSWER[ANSWER.find("2026-11-07T19:00").unwrap()
                ..ANSWER.find("\ntime,weather_code").unwrap()],
            &hours,
        );

        let slots = parse(&answer).expect("a forecast").slots;

        let hours: Vec<u32> = slots.iter().map(|slot| slot.hour).collect();

        assert_eq!(hours, [19, 22, 1, 4, 7, 10, 13]);
    }

    #[test]
    fn columns_are_found_by_name() {
        let answer = ANSWER.replace(
            "time,weather_code (wmo code),temperature_2m_max",
            "time,extra (x),weather_code (wmo code),temperature_2m_max",
        );
        let answer = answer
            .replace("2026-11-07,3,7.9", "2026-11-07,9,3,7.9")
            .replace("2026-11-08,51,8.4", "2026-11-08,9,51,8.4");

        let days = parse(&answer).expect("a forecast").days;

        assert_eq!(days[0].code, 3);
        assert_eq!(days[1].code, 51);
    }

    #[test]
    fn broken_answers_are_errors() {
        assert!(parse("").is_err());
        assert!(parse("{\"error\":true,\"reason\":\"Invalid\"}").is_err());
        assert!(parse(&ANSWER.replace("4.1,1.3,1", "x,y,z")).is_err());
    }

    #[test]
    fn every_code_has_a_description() {
        for code in [
            0, 1, 2, 3, 45, 48, 51, 56, 61, 65, 66, 71, 80, 82, 85, 95, 96, 99,
        ] {
            assert_ne!(describe(code, true).1, "Unknown");
        }

        assert_ne!(describe(0, true).0, describe(0, false).0);
        assert_eq!(describe(3, true).0, describe(3, false).0);
    }

    #[test]
    fn compass_points_and_degrees() {
        assert_eq!(compass(0.0), "N");
        assert_eq!(compass(11.0), "N");
        assert_eq!(compass(12.0), "NNE");
        assert_eq!(compass(140.0), "SE");
        assert_eq!(compass(359.0), "N");
        assert_eq!(degrees(4.1), "4°");
        assert_eq!(degrees(-0.3), "0°");
        assert_eq!(degrees(-2.6), "-3°");
    }
}
