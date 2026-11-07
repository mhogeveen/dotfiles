/*
 * layout picker: type to fuzzy-search mango's layouts, up/down or tab to pick,
 * enter or click to switch the focused monitor's current tag to it
 *
 * opened from the [S] in the bar, which starts the selection on the current
 * layout, or with `amane ipc call panel toggle layout`
 */

use amane::{LayerWindow, Monitor, Rectangle, Service};

use super::list::{self, fuzzy};
use super::picker::{self, Picker};
use super::{LAYOUT, Panels};
use crate::layout::{self, LAYOUTS, Layout, MangoLayout};
use crate::theme::Theme;

// a symbol typed as is, like "vs", puts its layout first
const SYMBOL_BONUS: i32 = 100;

pub struct Layouts;

impl Picker for Layouts {
    const PANEL: &'static str = LAYOUT;
    const INPUT: &'static str = "layout-search";
    const PLACEHOLDER: &'static str = "Search layouts  ·  ? for help";
    const NAMESPACE: &'static str = "amane-layout";

    type Item = &'static Layout;

    fn items(query: &str) -> Vec<&'static Layout> {
        matches(query)
    }

    // the layout in use on the panel's monitor gets the accent stripe
    fn row(t: &Theme, monitor: &Monitor, choice: &&'static Layout, selected: bool) -> Rectangle {
        let in_use = MangoLayout::read().symbol(&monitor.name) == Some(choice.symbol);

        list::detail_row(t, &choice.label(), choice.symbol, selected, in_use)
    }

    fn pick(choice: &'static Layout) {
        layout::set_layout(choice.name);
    }

    fn empty(_: &str) -> &'static str {
        "No matches"
    }

    fn help() -> &'static [(&'static str, &'static str)] {
        &[
            ("tile, vs", "search by name or symbol"),
            ("Enter", "switch the current tag to it"),
        ]
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    picker::view::<Layouts>(monitor)
}

pub fn reset() {
    picker::reset::<Layouts>();
}

/*
 * for the bar: starts the selection on this layout, then opens or closes the
 * panel; a click runs on the draw thread, so the search box may be touched
 */
pub fn toggle_from(symbol: Option<&str>) {
    let index = symbol.and_then(|symbol| LAYOUTS.iter().position(|layout| layout.symbol == symbol));

    if let Some(index) = index {
        picker::select::<Layouts>(index);
    }

    let _ = Panels::toggle(LAYOUT);
}

// best match first; an empty query keeps mango's order, the one switch_layout cycles
fn matches(query: &str) -> Vec<&'static Layout> {
    let query = query.trim();

    if query.is_empty() {
        return LAYOUTS.iter().collect();
    }

    let mut scored: Vec<(i32, &'static Layout)> = LAYOUTS
        .iter()
        .filter_map(|choice| {
            let symbol = choice.symbol.eq_ignore_ascii_case(query);

            let score = fuzzy(query, &choice.label());

            let score = match (score, symbol) {
                (Some(score), true) => Some(score + SYMBOL_BONUS),
                (None, true) => Some(SYMBOL_BONUS),
                (score, false) => score,
            };

            score.map(|score| (score, choice))
        })
        .collect();

    // stable, so equal scores keep mango's order
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    scored.into_iter().map(|(_, choice)| choice).collect()
}
