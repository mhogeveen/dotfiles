/*
 * pines: a forest at night, rows of pines from far to near, each row lighter
 * than the one in front of it like ridges' hills, standing on its own band of
 * ground. a tree is a stack of bars narrowing to its tip.
 * an accent moon hangs low, half behind the farthest trees, under a scatter of
 * faint stars
 *
 * seeded by the day like the other art designs: the forest holds all day and
 * grows anew overnight. rectangles only (docs/amane.md)
 */

use amane::{Color, Full, Rectangle, Widget};

use super::{Moment, Page, Random, placed, signature};
use crate::theme::Theme;

// mixed into the day's seed, so each generative design gets its own numbers
const SALT: u64 = 0x919E;

// rows of trees, far to near
const ROWS: usize = 4;

// where the farthest row stands and the step to the next, as fractions of the height below the bar
const FIRST_GROUND: f32 = 0.52;
const GROUND_STEP: f32 = 0.13;

// a tree's height in the farthest and nearest row, as fractions of the height below the bar
const FAR_TREE: f32 = 0.11;
const NEAR_TREE: f32 = 0.34;

// how much a tree's height may differ from its row's, either way
const TREE_VARIES: f32 = 0.3;

// a tree is this much as wide as it is tall
const TREE_ASPECT: f32 = 0.4;

// the room from one tree to the next, in tree widths: dense far away, sparser close by
const FAR_SPACING: (f32, f32) = (0.35, 0.7);
const NEAR_SPACING: (f32, f32) = (0.5, 1.0);

// a tree's shape: its tiers, and how narrow the top one is as a fraction of the bottom
const TIERS: usize = 7;
const TOP_TIER: f32 = 0.12;

// how far a tree reaches into its ground, at the reference height, so they don't just touch
const OVERLAP: f32 = 2.0;

// the moon: its size as a fraction of the height, where it may stand across the width
const MOON: f32 = 0.11;
const MOON_FROM: f32 = 0.15;
const MOON_TO: f32 = 0.85;

// how deep the moon sits behind the farthest row, in far trees
const MOON_SINK: f32 = 0.5;

// stars: how many, where (fractions of the height below the bar), and their size at the reference height
const STARS: usize = 70;
const STARS_FROM: f32 = 0.04;
const STARS_TO: f32 = 0.42;
const STAR: (f32, f32) = (1.5, 3.0);

pub fn layers(page: Page, moment: &Moment, t: &Theme) -> Vec<Box<dyn Widget>> {
    let mut random = Random::for_day(moment.date, SALT);

    let body = page.body_height();

    // far to near: lightest to darkest
    let colors: [Color; ROWS] = [t.bg4, t.bg3, t.bg2, t.bg1];

    let grounds: Vec<f32> = (0..ROWS)
        .map(|row| page.top + body * (FIRST_GROUND + GROUND_STEP * row as f32))
        .collect();

    let mut layers: Vec<Box<dyn Widget>> = Vec::new();

    // the stars first, the faintest thing on the page
    for _ in 0..STARS {
        let size = page.scaled(random.between(STAR.0, STAR.1));
        let color = if random.unit() < 0.3 {
            t.fg_dim
        } else {
            t.comment
        };

        layers.push(Box::new(placed(
            random.unit() * page.width,
            page.top + body * random.between(STARS_FROM, STARS_TO),
            Rectangle::new().width(size).height(size).fill(color),
        )));
    }

    // the moon, sunk into the farthest row
    let moon = MOON * page.height;
    let far_tree = FAR_TREE * body;

    layers.push(Box::new(placed(
        random.between(MOON_FROM, MOON_TO) * page.width - moon / 2.0,
        grounds[0] - far_tree * MOON_SINK - moon / 2.0,
        Rectangle::new()
            .width(moon)
            .height(moon)
            .radius(Full)
            .fill(t.accent),
    )));

    for row in 0..ROWS {
        // 0 for the farthest row, 1 for the nearest
        let depth = row as f32 / (ROWS - 1) as f32;

        let ground = grounds[row];
        let below = grounds.get(row + 1).copied().unwrap_or(page.height);

        // the row's ground, down to where the next row's begins
        layers.push(Box::new(placed(
            0.0,
            ground,
            Rectangle::new()
                .width(page.width)
                .height(below - ground)
                .fill(colors[row]),
        )));

        let tree_height = body * lerp(FAR_TREE, NEAR_TREE, depth);
        let spacing = (
            lerp(FAR_SPACING.0, NEAR_SPACING.0, depth),
            lerp(FAR_SPACING.1, NEAR_SPACING.1, depth),
        );

        for middle in positions(&mut random, page.width, tree_height * TREE_ASPECT, spacing) {
            let height = tree_height * random.between(1.0 - TREE_VARIES, 1.0 + TREE_VARIES);
            let width = height * TREE_ASPECT;

            /*
             * same-coloured shapes overlap instead of meeting edge to edge: at
             * fractional positions two soft edges meeting leave a light seam.
             * so the trunk runs from inside the ground up into the lowest tier,
             * and every tier reaches down to the bottom of the foliage
             */
            // each tier reaches into the ground, so tiers and ground overlap rather than meet
            let bottom = ground + page.scaled(OVERLAP);
            let mut top = ground;

            for (bar_width, bar_height) in tree_bars(width, height) {
                top -= bar_height;

                layers.push(Box::new(placed(
                    middle - bar_width / 2.0,
                    top,
                    Rectangle::new()
                        .width(bar_width)
                        .height(bottom - top)
                        .fill(colors[row]),
                )));
            }
        }
    }

    layers.push(Box::new(signature(page, moment, t)));

    layers
}

/*
 * a tree's tiers from the bottom up, as (width, height), narrowing steadily
 * to the tip; the heights add up to the tree's. no trunk: at these sizes it
 * only let the sky show through in a dashed line under every row
 */
fn tree_bars(width: f32, height: f32) -> Vec<(f32, f32)> {
    let tier = height / TIERS as f32;

    (0..TIERS)
        .map(|step| {
            let narrowing = step as f32 / (TIERS - 1) as f32;

            (width * lerp(1.0, TOP_TIER, narrowing), tier)
        })
        .collect()
}

// where trees stand along a row, as their middles, a random spacing apart, across the whole width
fn positions(random: &mut Random, width: f32, tree_width: f32, spacing: (f32, f32)) -> Vec<f32> {
    let mut middles = Vec::new();
    let mut at = random.between(0.0, tree_width);

    while at < width + tree_width / 2.0 {
        middles.push(at);

        at += tree_width * random.between(spacing.0, spacing.1);
    }

    middles
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::Date;

    #[test]
    fn trees_narrow_to_their_tip() {
        let bars = tree_bars(40.0, 100.0);

        assert_eq!(bars.len(), TIERS);
        assert_eq!(bars[0].0, 40.0);

        for pair in bars.windows(2) {
            assert!(pair[1].0 < pair[0].0);
        }

        let total: f32 = bars.iter().map(|(_, height)| height).sum();

        assert!((total - 100.0).abs() < 1e-3);
    }

    #[test]
    fn rows_are_filled_across_the_width() {
        let mut random = Random::for_day(Date::new(2026, 11, 17), SALT);
        let middles = positions(&mut random, 1000.0, 40.0, FAR_SPACING);

        assert!(middles[0] >= 0.0 && middles[0] <= 40.0);
        assert!(*middles.last().unwrap() > 1000.0 - 40.0);

        for pair in middles.windows(2) {
            assert!(pair[1] > pair[0]);
        }
    }

    #[test]
    fn a_day_keeps_its_forest() {
        let row = |date| positions(&mut Random::for_day(date, SALT), 1000.0, 40.0, NEAR_SPACING);

        assert_eq!(row(Date::new(2026, 11, 17)), row(Date::new(2026, 11, 17)));
        assert_ne!(row(Date::new(2026, 11, 17)), row(Date::new(2026, 11, 18)));
    }
}
