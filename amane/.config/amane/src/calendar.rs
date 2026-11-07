/*
 * date arithmetic for the calendar panel: weekdays, ISO week numbers and the
 * six-week grid a month is shown in; weeks start on Monday
 *
 * dates are counted as days since 1970-01-01, using Howard Hinnant's
 * days_from_civil and civil_from_days, which hold for any year
 */

pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: i32,

    // 1 to 12
    pub month: u32,

    // 1 to 31
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    fn days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    // Monday is 0, Sunday is 6
    pub fn weekday(self) -> u32 {
        // 1970-01-01 was a Thursday
        (self.days() + 3).rem_euclid(7) as u32
    }

    /*
     * the ISO 8601 week: the week that holds its Thursday, counted in that
     * Thursday's year, so 31 Dec can be week 1 and 1 Jan week 52 or 53
     */
    pub fn iso_week(self) -> u32 {
        let thursday = self.days() + 3 - i64::from(self.weekday());

        let year = civil_from_days(thursday).year;
        let first = days_from_civil(year, 1, 1);

        ((thursday - first) / 7 + 1) as u32
    }
}

// a month moved by some number of months, across years: (2026, 12) + 1 is (2027, 1)
pub fn shift(year: i32, month: u32, by: i32) -> (i32, u32) {
    let index = i64::from(year) * 12 + i64::from(month) - 1 + i64::from(by);

    (index.div_euclid(12) as i32, index.rem_euclid(12) as u32 + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day {
    pub date: Date,

    // false for the days of the months around the one shown
    pub in_month: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Week {
    pub number: u32,
    pub days: [Day; 7],
}

/*
 * always six weeks, Monday first, so the panel never changes size: the month
 * with the end of the one before and the start of the one after around it
 */
pub fn grid(year: i32, month: u32) -> Vec<Week> {
    let first = Date::new(year, month, 1);
    let start = first.days() - i64::from(first.weekday());

    (0..6)
        .map(|week| {
            let days = std::array::from_fn(|weekday| {
                let date = civil_from_days(start + week * 7 + weekday as i64);

                Day {
                    date,
                    in_month: date.year == year && date.month == month,
                }
            });

            Week {
                number: days[0].date.iso_week(),
                days,
            }
        })
        .collect()
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;

    // March is month 0, so a leap day falls at the end of the year
    let month = (i64::from(month) + 9) % 12;
    let day_of_year = (153 * month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> Date {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month + 2) / 5 + 1;
    let month = if month < 10 { month + 3 } else { month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);

    Date::new(year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_leap(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            2 if is_leap(year) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    #[test]
    fn weekdays_of_known_dates() {
        assert_eq!(Date::new(1970, 1, 1).weekday(), 3);
        assert_eq!(Date::new(2026, 1, 1).weekday(), 3);
        assert_eq!(Date::new(2026, 11, 7).weekday(), 5);
        assert_eq!(Date::new(2000, 2, 29).weekday(), 1);
    }

    #[test]
    fn iso_weeks_cross_years() {
        assert_eq!(Date::new(2026, 11, 7).iso_week(), 45);
        assert_eq!(Date::new(2024, 12, 31).iso_week(), 1);
        assert_eq!(Date::new(2021, 1, 1).iso_week(), 53);
        assert_eq!(Date::new(2026, 1, 1).iso_week(), 1);
        assert_eq!(Date::new(2027, 1, 3).iso_week(), 53);
    }

    #[test]
    fn leap_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2100, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(2026, 11), 30);
    }

    #[test]
    fn days_round_trip() {
        for days in -800_000..800_000_i64 {
            if days % 997 != 0 {
                continue;
            }

            assert_eq!(civil_from_days(days).days(), days);
        }
    }

    #[test]
    fn grid_starts_on_monday_and_holds_the_month() {
        let weeks = grid(2026, 11);

        assert_eq!(weeks.len(), 6);
        assert_eq!(weeks[0].days[0].date, Date::new(2026, 10, 26));
        assert!(!weeks[0].days[0].in_month);
        assert_eq!(weeks[0].days[6].date, Date::new(2026, 11, 1));
        assert!(weeks[0].days[6].in_month);
        assert_eq!(weeks[1].number, 45);

        let in_month = weeks
            .iter()
            .flat_map(|week| week.days)
            .filter(|day| day.in_month)
            .count();

        assert_eq!(in_month, days_in_month(2026, 11) as usize);

        // a leap february, and one that starts on a monday
        for (year, month) in [(2024, 2), (2027, 2), (2026, 6)] {
            let in_month = grid(year, month)
                .iter()
                .flat_map(|week| week.days)
                .filter(|day| day.in_month)
                .count();

            assert_eq!(in_month, days_in_month(year, month) as usize);
        }
    }

    #[test]
    fn months_shift_across_years() {
        assert_eq!(shift(2026, 12, 1), (2027, 1));
        assert_eq!(shift(2026, 1, -1), (2025, 12));
        assert_eq!(shift(2026, 11, 0), (2026, 11));
        assert_eq!(shift(2026, 3, -27), (2023, 12));
    }
}
