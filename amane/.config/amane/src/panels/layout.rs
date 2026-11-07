/*
 * layout picker: type to fuzzy-search mango's layouts, up/down or tab to pick,
 * enter or click to switch the focused monitor's current tag to it
 *
 * opened from the [S] in the bar, which starts the selection on the current
 * layout, or with `amane ipc call panel toggle layout`
 */

use std::time::Duration;

use amane::{Cursor::Pointer, Key, LayerWindow, Monitor, Service, TextInput, Vertical, Widget};

use super::list::{self, Cursor, fuzzy};
use super::{LAYOUT, Panels, panel_window};
use crate::layout::{self, LAYOUTS, Layout, MangoLayout};
use crate::theme::theme;

const INPUT: &str = "layout-search";

// a symbol typed as is, like "vs", puts its layout first
const SYMBOL_BONUS: i32 = 100;

struct Picker {
    query: String,
    cursor: Cursor,
}

impl Service for Picker {
    fn new() -> Self {
        Self {
            query: String::new(),
            cursor: Cursor::default(),
        }
    }

    // only changes through input
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let t = theme();
    let picker = Picker::read();

    // the panel shows on the monitor it was opened on, the one whose layout changes
    let current = MangoLayout::read().symbol(&monitor.name).map(String::from);

    let found = matches(&picker.query);
    let selected = picker.cursor.selected(found.len());

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for index in picker.cursor.visible(found.len()) {
        let choice = found[index];

        // the layout in use gets the accent stripe
        let in_use = current.as_deref() == Some(choice.symbol);

        rows.push(Box::new(
            list::detail_row(
                &t,
                &choice.label(),
                choice.symbol,
                index == selected,
                in_use,
            )
            .cursor(Pointer)
            .on_click(move |_| pick(choice)),
        ));
    }

    if rows.is_empty() {
        rows.push(list::message(&t, "No matches"));
    }

    let input = TextInput::new(INPUT)
        .placeholder("Search layouts")
        .focused()
        .on_change(|text| {
            let mut picker = Picker::write();

            picker.query = text;
            picker.cursor = Cursor::default();
        })
        .on_submit(|_| submit());

    let rows = list::rows(rows).on_scroll(|scroll| {
        let count = matches(&Picker::read().query).len();

        Picker::write()
            .cursor
            .scroll(list::scroll_delta(scroll), count);
    });

    let base = LayerWindow::new()
        .width(list::width())
        .height(list::height(list::ROWS, true))
        .anchor_vertical(Vertical::Middle)
        .namespace("amane-layout")
        .child(list::frame(&t, Some(input), rows));

    panel_window(monitor, LAYOUT, base, key_pressed)
}

/*
 * for the bar: starts the selection on this layout, then opens or closes the
 * panel; a click runs on the draw thread, so the search box may be touched
 */
pub fn toggle_from(symbol: Option<&str>) {
    let index = symbol.and_then(|symbol| LAYOUTS.iter().position(|layout| layout.symbol == symbol));

    if let Some(index) = index {
        Picker::write().cursor.select(index, LAYOUTS.len());
    }

    let _ = Panels::toggle(LAYOUT);
}

// escape is handled by the panel manager; the search box keeps letters and enter
fn key_pressed(key: Key) {
    let delta = match key {
        Key::Down | Key::Tab => 1,
        Key::Up => -1,
        Key::Enter => return submit(),
        _ => return,
    };

    let count = matches(&Picker::read().query).len();

    Picker::write().cursor.step(delta, count);
}

// clears the search for the next time the picker opens
pub fn reset() {
    TextInput::set_text(INPUT, "");

    let mut picker = Picker::write();

    picker.query.clear();
    picker.cursor = Cursor::default();
}

fn submit() {
    let choice = {
        let picker = Picker::read();

        let found = matches(&picker.query);

        found.get(picker.cursor.selected(found.len())).copied()
    };

    if let Some(choice) = choice {
        pick(choice);
    }
}

fn pick(choice: &'static Layout) {
    layout::set_layout(choice.name);

    let _ = Panels::close(LAYOUT);
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
