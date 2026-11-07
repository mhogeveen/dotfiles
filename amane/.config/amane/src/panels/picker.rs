/*
 * a search box over a list, shared by the launcher, clipboard and layout panels:
 * type to filter, up/down or tab to move, enter or click to pick, the mouse
 * wheel to scroll; escape is handled by the panel manager
 *
 * each panel implements Picker with its items, how a row looks and what picking
 * does; the query and selection live in Search<P>, one service per picker
 */

use std::marker::PhantomData;
use std::time::Duration;

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
}

// what's typed and what's selected, for one picker
pub struct Search<P> {
    query: String,
    cursor: Cursor,

    // a function pointer type, so Search<P> is Send and Sync whatever P is
    picker: PhantomData<fn() -> P>,
}

impl<P: Picker> Service for Search<P> {
    fn new() -> Self {
        Self {
            query: String::new(),
            cursor: Cursor::default(),
            picker: PhantomData,
        }
    }

    // only changes through input
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn view<P: Picker>(monitor: &Monitor) -> LayerWindow {
    let base = LayerWindow::new()
        .width(list::width())
        .height(list::height(list::ROWS, true))
        .anchor_vertical(Vertical::Middle)
        .namespace(P::NAMESPACE);

    panel(monitor, P::PANEL, base, key_pressed::<P>, |t| {
        content::<P>(t, monitor)
    })
}

fn content<P: Picker>(t: &Theme, monitor: &Monitor) -> Rectangle {
    let (query, cursor) = {
        let search = Search::<P>::read();

        (search.query.clone(), search.cursor)
    };

    let found = P::items(&query);
    let selected = cursor.selected(found.len());

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

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

    let input = TextInput::new(P::INPUT)
        .placeholder(P::PLACEHOLDER)
        .focused()
        .on_change(|text| {
            let mut search = Search::<P>::write();

            search.query = text;
            search.cursor = Cursor::default();
        })
        .on_submit(|_| submit::<P>());

    let rows = list::rows(rows).on_scroll(|scroll| {
        let count = count::<P>();

        Search::<P>::write()
            .cursor
            .scroll(list::scroll_delta(scroll), count);
    });

    list::frame(t, Some(input), rows)
}

// the search box keeps letters and enter, the rest of the keys come here
fn key_pressed<P: Picker>(key: Key) {
    let delta = match key {
        Key::Down | Key::Tab => 1,
        Key::Up => -1,
        Key::Enter => return submit::<P>(),
        _ => return,
    };

    let count = count::<P>();

    Search::<P>::write().cursor.step(delta, count);
}

// clears the search for the next time the panel opens
pub fn reset<P: Picker>() {
    TextInput::set_text(P::INPUT, "");

    let mut search = Search::<P>::write();

    search.query.clear();
    search.cursor = Cursor::default();
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
    let (query, cursor) = {
        let search = Search::<P>::read();

        (search.query.clone(), search.cursor)
    };

    let found = P::items(&query);

    if let Some(item) = found.get(cursor.selected(found.len())) {
        choose::<P>(item.clone());
    }
}

fn choose<P: Picker>(item: P::Item) {
    P::pick(item);

    let _ = Panels::close(P::PANEL);
}
