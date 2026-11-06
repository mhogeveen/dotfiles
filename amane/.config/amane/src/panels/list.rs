/*
 * the parts the launcher and clipboard panels share: a search box over a list
 * that's moved through with the arrow keys
 *
 * the list shows a window of ROWS rows instead of using a ScrollArea, because a
 * ScrollArea can't be scrolled from code, so it couldn't follow the selection
 */

use std::ops::Range;

use amane::{
    Align::{Center, Start},
    Column, Padding, Rectangle, Scroll,
    Size::Parent,
    Text, TextInput, Widget, children,
};

use crate::theme::Theme;

pub const ROWS: usize = 8;

const WIDTH: f32 = 600.0;
const PADDING: f32 = 12.0;
const GAP: f32 = 8.0;
const SEARCH_HEIGHT: f32 = 44.0;
const ROW_HEIGHT: f32 = 40.0;
const TEXT_SIZE: f32 = 15.0;

// room on the left and right only, so rows and the search box center vertically
const SIDES: Padding = Padding {
    top: 0.0,
    right: PADDING,
    bottom: 0.0,
    left: PADDING,
};

pub fn width() -> f32 {
    WIDTH
}

pub fn height() -> f32 {
    height_with(ROWS)
}

// for a short, fixed list that never needs all ROWS rows
pub fn height_with(rows: usize) -> f32 {
    PADDING * 2.0 + SEARCH_HEIGHT + GAP + rows as f32 * ROW_HEIGHT
}

// which row is picked, and which row is at the top of the visible window
#[derive(Default, Clone, Copy)]
pub struct Cursor {
    selected: usize,
    offset: usize,
}

impl Cursor {
    // the list can shrink under the cursor, a new clipboard entry or a narrower search
    pub fn selected(&self, count: usize) -> usize {
        self.selected.min(count.saturating_sub(1))
    }

    pub fn visible(&self, count: usize) -> Range<usize> {
        let start = self.offset.min(count.saturating_sub(ROWS));

        start..(start + ROWS).min(count)
    }

    pub fn step(&mut self, delta: isize, count: usize) {
        if count == 0 {
            return;
        }

        let selected = self.selected(count).saturating_add_signed(delta);

        self.selected = selected.min(count - 1);

        self.follow();
    }

    // moves the window, and drags the selection along when it would leave it
    pub fn scroll(&mut self, delta: isize, count: usize) {
        let last_offset = count.saturating_sub(ROWS);

        self.offset = self
            .offset
            .min(last_offset)
            .saturating_add_signed(delta)
            .min(last_offset);

        let selected = self.selected(count);

        self.selected = selected.clamp(self.offset, self.offset + ROWS - 1);
    }

    fn follow(&mut self) {
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + ROWS {
            self.offset = self.selected + 1 - ROWS;
        }
    }
}

// one wheel notch or touchpad flick moves one row
pub fn scroll_delta(scroll: Scroll) -> isize {
    if scroll.y > 0.0 {
        1
    } else if scroll.y < 0.0 {
        -1
    } else {
        0
    }
}

// the whole panel: search box on top, rows under it
pub fn frame(t: &Theme, input: TextInput, rows: Rectangle) -> Rectangle {
    let search = Rectangle::new()
        .width(Parent)
        .height(SEARCH_HEIGHT)
        .fill(t.bg1)
        .padding(SIDES)
        .align_child(Start, Center)
        .child(input.size(16.0).color(t.fg));

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(t.bg0)
        .border(1.0, t.bg3)
        .padding(PADDING)
        .child(Column::new(children![search, rows]).gap(GAP))
}

// the area the rows go in, whatever room the window leaves; add on_scroll to it to move the window
pub fn rows(rows: Vec<Box<dyn Widget>>) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .clip()
        .child(Column::new(rows))
}

pub fn row(t: &Theme, label: &str, selected: bool) -> Rectangle {
    let (fill, color) = if selected {
        (t.bg3, t.fg)
    } else {
        (t.bg0, t.fg_dim)
    };

    Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .fill(fill)
        .padding(SIDES)
        .align_child(Start, Center)
        .child(Text::new(label).size(TEXT_SIZE).color(color).elide())
}

/*
 * every query letter has to appear in the text in order, ignoring case;
 * letters right after each other, at the start of a word or at the very start
 * score higher, and gaps between them cost a little
 */
pub fn fuzzy(query: &str, text: &str) -> Option<i32> {
    let text: Vec<char> = text.chars().flat_map(char::to_lowercase).collect();

    let mut score = 0;
    let mut from = 0;
    let mut previous: Option<usize> = None;

    for letter in query.chars().flat_map(char::to_lowercase) {
        if letter.is_whitespace() {
            continue;
        }

        let found = (from..text.len()).find(|&index| text[index] == letter)?;

        score += 10;

        let word_start = found == 0 || !text[found - 1].is_alphanumeric();

        if word_start {
            score += 8;
        }

        match previous {
            Some(previous) if found == previous + 1 => score += 6,
            Some(previous) => score -= (found - previous - 1).min(6) as i32,
            None => score -= found.min(12) as i32,
        }

        previous = Some(found);
        from = found + 1;
    }

    // between two equal matches, the shorter text is the closer one
    score -= (text.len() / 4) as i32;

    Some(score)
}

// shown instead of rows when there are none
pub fn message(t: &Theme, text: &str) -> Box<dyn Widget> {
    Box::new(
        Rectangle::new()
            .width(Parent)
            .height(ROW_HEIGHT)
            .padding(SIDES)
            .align_child(Start, Center)
            .child(Text::new(text).size(TEXT_SIZE).color(t.comment)),
    )
}

#[cfg(test)]
mod tests {
    use super::fuzzy;

    #[test]
    fn letters_must_appear_in_order() {
        assert!(fuzzy("ffx", "Firefox").is_some());
        assert!(fuzzy("xff", "Firefox").is_none());
    }

    #[test]
    fn prefixes_beat_scattered_letters() {
        assert!(fuzzy("fi", "Firefox") > fuzzy("fi", "Profile Manager"));
        assert!(fuzzy("term", "Terminal") > fuzzy("term", "Kitty Terminal Emulator"));
    }
}
