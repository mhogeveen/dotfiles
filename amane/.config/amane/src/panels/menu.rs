/*
 * a short numbered menu, shared by the screenshot and session panels: press a
 * row's number to choose it, or move with up/down or tab and press enter, or
 * click; escape is handled by the panel manager
 *
 * there's no search box, so number keys reach the menu instead of being typed.
 * an item can ask to be confirmed: choosing it arms the row, choosing the same
 * row again runs it, moving away or choosing another row disarms it
 */

use std::marker::PhantomData;

use amane::{Cursor::Pointer, Key, LayerWindow, Monitor, Rectangle, Service, Vertical, Widget};

use super::list::{self, Cursor};
use super::{Panels, panel};
use crate::theme::Theme;

pub trait Menu: Send + Sync + 'static {
    // the panel's name in the registry
    const PANEL: &'static str;
    const NAMESPACE: &'static str;

    type Item: Copy + 'static;

    fn items() -> &'static [Self::Item];

    fn label(item: Self::Item) -> &'static str;

    // whether choosing it once only arms it
    fn confirms(_: Self::Item) -> bool {
        false
    }

    // runs on the draw thread, after the panel closed
    fn run(item: Self::Item);
}

// the highlight and the armed row, for one menu
pub struct Choice<M> {
    cursor: Cursor,

    // the item waiting to be chosen again, as an index into items()
    armed: Option<usize>,

    // a function pointer type, so Choice<M> is Send and Sync whatever M is
    menu: PhantomData<fn() -> M>,
}

impl<M: Menu> Service for Choice<M> {
    fn new() -> Self {
        Self {
            cursor: Cursor::default(),
            armed: None,
            menu: PhantomData,
        }
    }

    // only changes through input
    fn listen() {}
}

pub fn view<M: Menu>(monitor: &Monitor) -> LayerWindow {
    let count = M::items().len();

    let base = LayerWindow::new()
        .width(list::width())
        .height(list::height(count.min(list::ROWS), false))
        .anchor_vertical(Vertical::Middle)
        .namespace(M::NAMESPACE);

    panel(monitor, M::PANEL, base, key_pressed::<M>, content::<M>)
}

fn content<M: Menu>(t: &Theme) -> Rectangle {
    let (cursor, armed) = {
        let choice = Choice::<M>::read();

        (choice.cursor, choice.armed)
    };

    let items = M::items();
    let selected = cursor.selected(items.len());
    let visible = cursor.visible(items.len());
    let first = visible.start;

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for index in visible {
        let label = M::label(items[index]);
        let number = index - first + 1;

        let row = if armed == Some(index) {
            list::armed_row(
                t,
                number,
                &format!("{label} — press {number} or Enter again"),
            )
        } else {
            list::numbered_row(t, number, label, index == selected)
        };

        rows.push(Box::new(
            row.cursor(Pointer).on_click(move |_| choose::<M>(index)),
        ));
    }

    list::frame(t, None, list::rows(rows))
}

fn key_pressed<M: Menu>(key: Key) {
    let count = M::items().len();
    let cursor = Choice::<M>::read().cursor;

    if let Some(position) = list::hotkey(key) {
        if let Some(index) = cursor.at_position(position, count) {
            choose::<M>(index);
        }

        return;
    }

    let delta = match key {
        Key::Down | Key::Tab => 1,
        Key::Up => -1,
        Key::Enter => return choose::<M>(cursor.selected(count)),
        _ => return,
    };

    // moving away disarms, so a stray enter later can't run what was armed
    let mut choice = Choice::<M>::write();

    choice.cursor.step(delta, count);
    choice.armed = None;
}

// starts at the top and disarmed the next time the menu opens
pub fn reset<M: Menu>() {
    let mut choice = Choice::<M>::write();

    choice.cursor = Cursor::default();
    choice.armed = None;
}

// runs the item, or arms it first when it asks to be confirmed
fn choose<M: Menu>(index: usize) {
    let items = M::items();

    let Some(&item) = items.get(index) else {
        return;
    };

    let run = {
        let mut choice = Choice::<M>::write();

        if M::confirms(item) && choice.armed != Some(index) {
            choice.armed = Some(index);
            choice.cursor.select(index, items.len());

            false
        } else {
            choice.armed = None;

            true
        }
    };

    // the lock is let go first: closing runs reset, which writes it
    if run {
        let _ = Panels::close(M::PANEL);

        M::run(item);
    }
}
