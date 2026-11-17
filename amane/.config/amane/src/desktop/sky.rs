/*
 * the moon, worked out here rather than fetched: its age from a known new moon
 * and the mean length of a lunar month (within a day of the real phases, as
 * the moon's orbit isn't a circle; fine for naming the phase and counting to
 * the next)
 *
 * pure functions, for lunar
 */

use std::f64::consts::{PI, TAU};

// the mean time from new moon to new moon, in days
pub const SYNODIC_MONTH: f64 = 29.530_588_853;

// a new moon: 2000-01-06 18:14 UTC, as a Julian day
const KNOWN_NEW_MOON: f64 = 2_451_550.1;

// 1970-01-01 00:00 UTC as a Julian day
const UNIX_EPOCH_JULIAN: f64 = 2_440_587.5;

const SECONDS_PER_DAY: f64 = 86_400.0;

// the eight phases, a slice of the month each, the new moon first
const PHASES: [&str; 8] = [
    "new moon",
    "waxing crescent",
    "first quarter",
    "waxing gibbous",
    "full moon",
    "waning gibbous",
    "last quarter",
    "waning crescent",
];

// the moon's age in days at an instant: 0 at new moon, half a month at full
pub fn moon_age(epoch_seconds: i64) -> f64 {
    let julian = epoch_seconds as f64 / SECONDS_PER_DAY + UNIX_EPOCH_JULIAN;

    (julian - KNOWN_NEW_MOON).rem_euclid(SYNODIC_MONTH)
}

// how much of the disc is lit, 0 to 1
pub fn illumination(age: f64) -> f64 {
    (1.0 - (TAU * age / SYNODIC_MONTH).cos()) / 2.0
}

pub fn phase_name(age: f64) -> &'static str {
    let eighth = (age / SYNODIC_MONTH * PHASES.len() as f64).round() as usize;

    PHASES[eighth % PHASES.len()]
}

// days from this age until the moon is `target` days old again: 0 for new, half a month for full
pub fn days_until(age: f64, target: f64) -> f64 {
    (target - age).rem_euclid(SYNODIC_MONTH)
}

/*
 * where the lit part of one slice of the disc runs, as x from -1 (left) to 1
 * (right), for a slice whose half-width is 1: the northern hemisphere sees a
 * waxing moon lit on the right and a waning one on the left
 */
pub fn lit_span(age: f64) -> (f64, f64) {
    let angle = TAU * age / SYNODIC_MONTH;

    if angle <= PI {
        (angle.cos(), 1.0)
    } else {
        (-1.0, -angle.cos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_moon_keeps_its_phases() {
        // a full moon: 2024-04-23 23:49 UTC
        let full = moon_age(1_713_916_140);

        assert!((full - SYNODIC_MONTH / 2.0).abs() < 1.0, "age {full}");
        assert_eq!(phase_name(full), "full moon");
        assert!(illumination(full) > 0.97);

        // a new moon: 2024-04-08 18:21 UTC, the eclipse
        let new = moon_age(1_712_600_460);
        let from_new = new.min(SYNODIC_MONTH - new);

        assert!(from_new < 1.0, "age {new}");
        assert_eq!(phase_name(new), "new moon");
    }

    #[test]
    fn the_lit_side_follows_the_phase() {
        // first quarter: the right half
        let (from, to) = lit_span(SYNODIC_MONTH / 4.0);
        assert!(from.abs() < 1e-9 && to == 1.0);

        // full: all of it
        let (from, to) = lit_span(SYNODIC_MONTH / 2.0);
        assert!((from + 1.0).abs() < 1e-9 && to == 1.0);

        // last quarter: the left half
        let (from, to) = lit_span(SYNODIC_MONTH * 0.75);
        assert!(from == -1.0 && to.abs() < 1e-9);
    }

    #[test]
    fn counting_to_the_next_phase() {
        assert!(
            (days_until(10.0, SYNODIC_MONTH / 2.0) - (SYNODIC_MONTH / 2.0 - 10.0)).abs() < 1e-9
        );
        assert!((days_until(20.0, 0.0) - (SYNODIC_MONTH - 20.0)).abs() < 1e-9);
    }
}
