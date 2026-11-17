/*
 * ridges: hills one behind the other, cut from thin vertical strokes like a
 * woodcut, lightest far away and darkest close by, with a sun in the accent
 * sinking behind the farthest. the hills' shapes are waves of a few sizes laid
 * over each other, seeded by the day, so the view holds all day and changes
 * overnight
 *
 * every size is a fraction of the page, so the thumbnail is the same view
 */

use amane::{Align::Start, Color, Full, Rectangle, Row, Widget};

use super::{Moment, Page, Random, placed, signature};
use crate::theme::Theme;

// mixed into the day's seed, so each generative design gets its own numbers
const SALT: u64 = 0x41D6;

// strokes across the page per hill, and the gap between them at the reference height
const STROKES: u32 = 220;
const STROKE_GAP: f32 = 3.0;

// how many hills, far to near
const HILLS: usize = 5;

// the farthest hill's middle height and the step to the next, as fractions of the height below the bar
const FARTHEST: f32 = 0.42;
const STEP: f32 = 0.1;

// how far a hill rises and falls, the farthest the most, as fractions of the height below the bar
const SWELL: f32 = 0.1;
const SWELL_STEP: f32 = 0.012;

// the waves each hill is made of: (how much of the swell, fewest and most waves across the page)
const WAVES: [(f32, f32, f32); 3] = [(0.6, 1.0, 2.5), (0.3, 3.0, 6.0), (0.1, 9.0, 16.0)];

// the sun: its size as a fraction of the height, where it may stand across the width
const SUN: f32 = 0.17;
const SUN_FROM: f32 = 0.2;
const SUN_TO: f32 = 0.8;

// how far the sun sits into the farthest hill, in suns
const SUN_SINK: f32 = 0.35;

struct Wave {
    share: f32,
    count: f32,
    phase: f32,
}

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    let mut random = Random::for_day(moment.date, SALT);

    let body = page.body_height();

    // far to near: lightest to darkest
    let colors: [Color; HILLS] = [t.bg4, t.bg3, t.bg2, t.bg1, t.bg0];

    let hills: Vec<Vec<Wave>> = (0..HILLS)
        .map(|_| {
            WAVES
                .iter()
                .map(|&(share, fewest, most)| Wave {
                    share,
                    count: random.between(fewest, most),
                    phase: random.between(0.0, std::f32::consts::TAU),
                })
                .collect()
        })
        .collect();

    let sun_size = SUN * page.height;
    let sun_x = random.between(SUN_FROM, SUN_TO);

    let farthest_at_sun = page.top + body * (FARTHEST - SWELL * height(&hills[0], sun_x));

    let mut layers: Vec<Box<dyn Widget>> = vec![Box::new(placed(
        sun_x * page.width - sun_size / 2.0,
        farthest_at_sun - sun_size * (1.0 - SUN_SINK),
        Rectangle::new()
            .width(sun_size)
            .height(sun_size)
            .radius(Full)
            .fill(t.accent),
    ))];

    let pitch = page.width / STROKES as f32;
    let stroke = (pitch - page.scaled(STROKE_GAP)).max(1.0);

    // every hill's edge, stroke by stroke, far to near
    let tops: Vec<Vec<f32>> = hills
        .iter()
        .enumerate()
        .map(|(index, waves)| {
            let middle = FARTHEST + STEP * index as f32;
            let swell = SWELL - SWELL_STEP * index as f32;

            (0..STROKES)
                .map(|stroke| {
                    let x = (stroke as f32 + 0.5) / STROKES as f32;

                    page.top + body * (middle - swell * height(waves, x))
                })
                .collect()
        })
        .collect();

    for (index, hill) in tops.iter().enumerate() {
        let spans = visible_spans(&tops, index, page.height);
        let highest = hill.iter().copied().fold(f32::INFINITY, f32::min);

        /*
         * each stroke only runs down to where a nearer hill starts, so no two
         * hills draw over the same pixels: the strokes sit between whole
         * pixels, and stacked soft edges would show as slivers in the gaps
         */
        let strokes: Vec<Box<dyn Widget>> = spans
            .iter()
            .map(|&(top, bottom)| -> Box<dyn Widget> {
                Box::new(
                    Rectangle::new()
                        .width(stroke)
                        .height(page.height - highest)
                        .align_child(Start, Start)
                        .child(
                            Rectangle::new()
                                .width(stroke)
                                .height((bottom - top).max(0.0))
                                .fill(colors[index])
                                .translate(0.0, top - highest),
                        ),
                )
            })
            .collect();

        layers.push(Box::new(placed(
            0.0,
            highest,
            Row::new(strokes).gap(pitch - stroke),
        )));
    }

    layers.push(Box::new(signature(page, moment, t)));

    layers
}

/*
 * where each of a hill's strokes shows, as (top, bottom): from its own edge
 * down to the highest edge of any hill in front of it, or the page's bottom
 * for the nearest. a stroke wholly behind a nearer hill gets bottom = top
 */
fn visible_spans(tops: &[Vec<f32>], hill: usize, page_bottom: f32) -> Vec<(f32, f32)> {
    tops[hill]
        .iter()
        .enumerate()
        .map(|(stroke, &top)| {
            let nearer = tops[hill + 1..]
                .iter()
                .map(|nearer| nearer[stroke])
                .fold(page_bottom, f32::min);

            (top, nearer.max(top))
        })
        .collect()
}

// a hill's height at x (0 to 1 across the page), from -1 to 1
fn height(waves: &[Wave], x: f32) -> f32 {
    waves
        .iter()
        .map(|wave| wave.share * (std::f32::consts::TAU * wave.count * x + wave.phase).sin())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hills_stay_within_their_swell() {
        let waves: Vec<Wave> = WAVES
            .iter()
            .map(|&(share, count, _)| Wave {
                share,
                count,
                phase: 0.3,
            })
            .collect();

        for step in 0..=100 {
            let value = height(&waves, step as f32 / 100.0);

            assert!((-1.0..=1.0).contains(&value), "height {value}");
        }
    }

    #[test]
    fn strokes_stop_where_a_nearer_hill_starts() {
        // three hills, two strokes: far, middle, near
        let tops = vec![vec![100.0, 300.0], vec![200.0, 250.0], vec![400.0, 260.0]];

        // the far hill shows down to the middle one, and not at all where the middle is higher
        assert_eq!(
            visible_spans(&tops, 0, 1000.0),
            [(100.0, 200.0), (300.0, 300.0)]
        );

        // the middle one down to the near one
        assert_eq!(
            visible_spans(&tops, 1, 1000.0),
            [(200.0, 400.0), (250.0, 260.0)]
        );

        // the nearest runs off the page
        assert_eq!(
            visible_spans(&tops, 2, 1000.0),
            [(400.0, 1000.0), (260.0, 1000.0)]
        );
    }

    #[test]
    fn the_waves_share_the_whole_swell() {
        let total: f32 = WAVES.iter().map(|(share, _, _)| share).sum();

        assert!((total - 1.0).abs() < 1e-6);
    }
}
