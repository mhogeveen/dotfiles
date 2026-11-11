/*
 * a search box over a list, shared by the launcher, clipboard and layout panels:
 * type to filter, up/down or tab to move, enter or click to pick, the mouse
 * wheel to scroll; escape is handled by the panel manager
 *
 * each panel implements Picker with its items, how a row looks and what picking
 * does; the query and selection live in Search<P>, one service per picker
 *
 * ? in the empty search box shows a help screen of the panel's own lines and the
 * shared keys instead of results. the search box takes every typed character, so
 * the panels' ? for key hints can't reach it; on_change spots it instead. a ? in
 * a search or command is just a character
 */

use std::marker::PhantomData;

use amane::{
    Cursor::Pointer, Key, LayerWindow, Monitor, Rectangle, Service, TextInput, Vertical, Widget,
};

use super::list::{self, Cursor};
use super::{Panels, panel};
use crate::theme::Theme;

pub trait Picker: Send + Sync + 'static {
    // the panel's name in the registry
    const PANEL: &'static str;

    // the search box's name, which amane keeps its text under
    const INPUT: &'static str;

    const PLACEHOLDER: &'static str;
    const NAMESPACE: &'static str;

    type Item: Clone + 'static;

    // what matches the query, best first; may read other services
    fn items(query: &str) -> Vec<Self::Item>;

    // one row; the monitor is the one the panel shows on
    fn row(t: &Theme, monitor: &Monitor, item: &Self::Item, selected: bool) -> Rectangle;

    // runs on the draw thread, the panel closes right after
    fn pick(item: Self::Item);

    // shown when nothing matches, like "No matches" or "Loading…"
    fn empty(query: &str) -> &'static str;

    // the help screen's own lines: what to press or type, and what it does
    fn help() -> &'static [(&'static str, &'static str)];

    // how wide an area to the right of the list showing the selected item is; 0 for none
    const PREVIEW: f32 = 0.0;

    // whether there's a row of buttons under the list
    const FOOTER: bool = false;

    // the area to the right of the list, for the selected item
    fn preview(_t: &Theme, _selected: Option<&Self::Item>) -> Option<Rectangle> {
        None
    }

    // the row under the list
    fn footer(_t: &Theme) -> Option<Rectangle> {
        None
    }

    // after the selection moved or the search changed
    fn moved() {}
}

// under every picker's own help lines
const SHARED_HELP: [(&str, &str); 2] = [("↑ ↓  Tab", "move the selection"), ("Esc", "close")];

const HELP: &str = "?";

// what's typed and what's selected, for one picker
pub struct Search<P> {
    query: String,
    cursor: Cursor,

    // the help screen shows instead of results
    help: bool,

    // a function pointer type, so Search<P> is Send and Sync whatever P is
    picker: PhantomData<fn() -> P>,
}

impl<P: Picker> Service for Search<P> {
    fn new() -> Self {
        Self {
            query: String::new(),
            cursor: Cursor::default(),
            help: false,
            picker: PhantomData,
        }
    }

    // only changes through input
    fn listen() {}
}

pub fn view<P: Picker>(monitor: &Monitor) -> LayerWindow {
    let base = LayerWindow::new()
        .width(list::width_with(P::PREVIEW))
        .height(list::height_with(list::ROWS, true, P::FOOTER))
        .anchor_vertical(Vertical::Middle)
        .namespace(P::NAMESPACE);

    panel(monitor, P::PANEL, base, key_pressed::<P>, |t| {
        content::<P>(t, monitor)
    })
}

fn content<P: Picker>(t: &Theme, monitor: &Monitor) -> Rectangle {
    let (query, cursor, help) = {
        let search = Search::<P>::read();

        (search.query.clone(), search.cursor, search.help)
    };

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // what the preview shows; nothing while the help does
    let mut chosen = None;

    if help {
        for (keys, does) in P::help().iter().chain(SHARED_HELP.iter()) {
            rows.push(Box::new(list::help_row(t, keys, does)));
        }
    } else {
        let found = P::items(&query);
        let selected = cursor.selected(found.len());

        chosen = found.get(selected).cloned();

        for index in cursor.visible(found.len()) {
            let item = found[index].clone();

            rows.push(Box::new(
                P::row(t, monitor, &item, index == selected)
                    .cursor(Pointer)
                    .on_click(move |_| choose::<P>(item.clone())),
            ));
        }

        if rows.is_empty() {
            rows.push(list::message(t, P::empty(&query)));
        }
    }

    let input = TextInput::new(P::INPUT)
        .placeholder(P::PLACEHOLDER)
        .focused()
        .on_change(changed::<P>)
        .on_submit(|_| submit::<P>());

    let rows = list::rows(rows).on_scroll(|scroll| {
        if close_help::<P>() {
            return;
        }

        let count = count::<P>();

        Search::<P>::write()
            .cursor
            .scroll(list::scroll_delta(scroll), count);

        P::moved();
    });

    let preview = P::preview(t, chosen.as_ref());

    list::frame_with(t, Some(input), rows, P::footer(t), preview, P::PREVIEW)
}

/*
 * amane stores the new text before calling this, so a ? typed into the empty
 * box can be taken back out: it toggles the help instead of searching
 */
fn changed<P: Picker>(text: String) {
    let mut search = Search::<P>::write();

    if text == HELP && search.query.is_empty() {
        TextInput::set_text(P::INPUT, "");

        search.help = !search.help;

        return;
    }

    search.help = false;
    search.query = text;
    search.cursor = Cursor::default();

    drop(search);

    P::moved();
}

// closes the help if it was showing, so the key that did it doesn't act on a list nobody saw
fn close_help<P: Picker>() -> bool {
    if !Search::<P>::read().help {
        return false;
    }

    Search::<P>::write().help = false;

    true
}

// the search box keeps letters and enter, the rest of the keys come here
fn key_pressed<P: Picker>(key: Key) {
    let delta = match key {
        Key::Down | Key::Tab => 1,
        Key::Up => -1,
        Key::Enter => return submit::<P>(),
        _ => return,
    };

    if close_help::<P>() {
        return;
    }

    let count = count::<P>();

    Search::<P>::write().cursor.step(delta, count);

    P::moved();
}

// the item the selection is on, if any
fn selected<P: Picker>() -> Option<P::Item> {
    let (query, cursor) = {
        let search = Search::<P>::read();

        (search.query.clone(), search.cursor)
    };

    let found = P::items(&query);

    found.get(cursor.selected(found.len())).cloned()
}

// clears the search and the help for the next time the panel opens
pub fn reset<P: Picker>() {
    TextInput::set_text(P::INPUT, "");

    let mut search = Search::<P>::write();

    search.query.clear();
    search.cursor = Cursor::default();
    search.help = false;
}

// starts the selection on this item, like the layout in use, before opening
pub fn select<P: Picker>(index: usize) {
    let count = count::<P>();

    Search::<P>::write().cursor.select(index, count);
}

fn count<P: Picker>() -> usize {
    let query = Search::<P>::read().query.clone();

    P::items(&query).len()
}

fn submit<P: Picker>() {
    if close_help::<P>() {
        return;
    }

    if let Some(item) = selected::<P>() {
        choose::<P>(item);
    }
}

fn choose<P: Picker>(item: P::Item) {
    P::pick(item);

    let _ = Panels::close(P::PANEL);
}

// how many rows the help takes, for the test that it fits
#[cfg(test)]
pub fn help_rows<P: Picker>() -> usize {
    P::help().len() + SHARED_HELP.len()
}

#[cfg(test)]
mod tests {
    use super::help_rows;
    use crate::panels::clipboard::Clipboard;
    use crate::panels::launcher::Launcher;
    use crate::panels::layout::Layouts;
    use crate::panels::list::ROWS;

    // the help shows in the list's rows, which don't scroll while it shows
    #[test]
    fn every_help_fits_the_list() {
        assert!(help_rows::<Launcher>() <= ROWS);
        assert!(help_rows::<Clipboard>() <= ROWS);
        assert!(help_rows::<Layouts>() <= ROWS);
    }
}
