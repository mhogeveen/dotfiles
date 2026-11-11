/*
 * the parts the list panels share: an optional search box over a list that's
 * moved through with the arrow keys, or picked from with number keys
 *
 * the list shows a window of ROWS rows instead of using a ScrollArea, because a
 * ScrollArea can't be scrolled from code, so it couldn't follow the selection
 */

use std::ops::Range;

use amane::{
    Align::{Center, Start},
    Color, Column, Key, Padding, Rectangle, Row, Scroll,
    Size::Parent,
    Text, TextInput, Widget, children,
};

use super::ui::{PADDING, SIDES};
use crate::theme::Theme;

pub const ROWS: usize = 8;

const WIDTH: f32 = 600.0;
const GAP: f32 = 8.0;
const SEARCH_HEIGHT: f32 = 44.0;
const ROW_HEIGHT: f32 = 40.0;
const TEXT_SIZE: f32 = 15.0;
const NUMBER_WIDTH: f32 = 24.0;
const FOOTER_HEIGHT: f32 = 28.0;
const STRIPE_WIDTH: f32 = 2.0;

pub fn width() -> f32 {
    WIDTH
}

// rows is how many fit, ROWS for long lists; search adds the search box above them
pub fn height(rows: usize, search: bool) -> f32 {
    let search = if search { SEARCH_HEIGHT + GAP } else { 0.0 };

    PADDING * 2.0 + search + rows as f32 * ROW_HEIGHT
}

// with an area beside the list this wide, like a preview; 0 for none
pub fn width_with(beside: f32) -> f32 {
    if beside > 0.0 {
        WIDTH + GAP + beside
    } else {
        WIDTH
    }
}

// with a row of buttons under the list
pub fn height_with(rows: usize, search: bool, footer: bool) -> f32 {
    let footer = if footer { GAP + FOOTER_HEIGHT } else { 0.0 };

    height(rows, search) + footer
}

/*
 * number keys 1 to 9 pick the row at that position on screen; only for lists
 * without a search box, since a focused search box takes every typed digit
 */
pub fn hotkey(key: Key) -> Option<usize> {
    match key {
        Key::Character(digit @ '1'..='9') => Some(digit as usize - '1' as usize),
        _ => None,
    }
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

    // jumps straight to a row, like one picked with the mouse or a number key
    pub fn select(&mut self, index: usize, count: usize) {
        if count == 0 {
            return;
        }

        self.selected = index.min(count - 1);

        self.follow();
    }

    // the list index of the row at this position on screen, counted from 0
    pub fn at_position(&self, position: usize, count: usize) -> Option<usize> {
        let index = self.visible(count).start + position;

        self.visible(count).contains(&index).then_some(index)
    }

    fn follow(&mut self) {
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + ROWS {
            self.offset = self.selected + 1 - ROWS;
        }
    }
}

/*
 * a highlight for the lists in the corner panels (networks, bluetooth devices),
 * which scroll with the mouse in a ScrollArea and have rows of more than one
 * kind. None until a key moves it, so the mouse alone sees the list as before;
 * once it's there, the list is drawn from a first row that follow_window keeps
 * the highlight in sight with, instead of through the ScrollArea
 */
pub fn step_highlight(current: Option<usize>, delta: isize, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }

    let last = count - 1;

    Some(match current {
        // the first press lands on the first row, whichever way it goes
        None => 0,
        Some(index) => index.min(last).saturating_add_signed(delta).min(last),
    })
}

/*
 * the first row to draw so rows anchor..=row are in a list this tall, moving
 * the last first row only as far as needed: the list stays put while the
 * highlight moves inside it, like a picker's. anchor is the highlighted row,
 * or the heading right above it so its group's name shows too
 */
pub fn follow_window(
    first: usize,
    anchor: usize,
    row: usize,
    heights: &[f32],
    visible: f32,
) -> usize {
    let row = row.min(heights.len().saturating_sub(1));

    let mut first = first.min(anchor).min(row);

    while first < row && heights[first..=row].iter().sum::<f32>() > visible {
        first += 1;
    }

    /*
     * no empty room under the last row while rows above the window would fit
     * in it, like after a password box closes or the list gets shorter
     */
    while first > 0 && heights[first - 1..].iter().sum::<f32>() <= visible {
        first -= 1;
    }

    first
}

// the movement keys of a highlight list: arrows and j/k
pub fn highlight_delta(key: &Key) -> Option<isize> {
    match key {
        Key::Down | Key::Character('j') => Some(1),
        Key::Up | Key::Character('k') => Some(-1),
        _ => None,
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

// the whole panel: the search box on top when there is one, rows under it
pub fn frame(t: &Theme, search: Option<TextInput>, rows: Rectangle) -> Rectangle {
    frame_with(t, search, rows, None, None, 0.0)
}

/*
 * the same, with a row under the list (footer) and an area of this width beside
 * it (like a preview of the selected item)
 */
pub fn frame_with(
    t: &Theme,
    search: Option<TextInput>,
    rows: Rectangle,
    footer: Option<Rectangle>,
    beside: Option<Rectangle>,
    beside_width: f32,
) -> Rectangle {
    let mut parts: Vec<Box<dyn Widget>> = Vec::new();

    if let Some(input) = search {
        parts.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(SEARCH_HEIGHT)
                .fill(t.bg1)
                .padding(SIDES)
                .align_child(Start, Center)
                .child(input.size(16.0).color(t.fg)),
        ));
    }

    parts.push(Box::new(rows));

    if let Some(footer) = footer {
        parts.push(Box::new(footer));
    }

    let list = Column::new(parts).width(Parent).gap(GAP);

    let frame = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(t.bg0)
        .border(1.0, t.bg3)
        .padding(PADDING);

    let Some(beside) = beside else {
        return frame.child(list);
    };

    let beside = Rectangle::new()
        .width(beside_width)
        .height(Parent)
        .child(beside);

    frame.child(Row::new(children![list, beside]).gap(GAP).height(Parent))
}

// the row under a list: its parts from the left; a Parent-wide part pushes the rest right
pub fn footer(parts: Vec<Box<dyn Widget>>) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(FOOTER_HEIGHT)
        .child(Row::new(parts).gap(GAP).align(Center).height(FOOTER_HEIGHT))
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
    row_with(t, None, label, selected, None)
}

// a row with the number key that picks it in front, see hotkey()
pub fn numbered_row(t: &Theme, number: usize, label: &str, selected: bool) -> Rectangle {
    row_with(t, Some(number), label, selected, None)
}

/*
 * a row with a button at its end, like a delete button on the selected row
 */
pub fn button_row(t: &Theme, label: &str, selected: bool, button: Option<Rectangle>) -> Rectangle {
    let (fill, color) = if selected {
        (t.bg3, t.fg)
    } else {
        (t.bg0, t.fg_dim)
    };

    let label = Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .align_child(Start, Center)
        .child(Text::new(label).size(TEXT_SIZE).color(color).elide());

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(label)];

    if let Some(button) = button {
        parts.push(Box::new(button));
    }

    Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .fill(fill)
        .padding(SIDES)
        .align_child(Start, Center)
        .child(Row::new(parts).width(Parent).gap(GAP).align(Center))
}

// a selected numbered row with its label in the accent, for an action waiting to be confirmed
pub fn armed_row(t: &Theme, number: usize, label: &str) -> Rectangle {
    row_with(t, Some(number), label, true, Some(t.accent))
}

/*
 * a row with a dimmed detail on the right, like a layout's symbol; marked puts
 * a thin accent stripe on its left edge, like for the layout in use. every row
 * keeps room for the stripe, so the labels line up either way
 */
pub fn detail_row(t: &Theme, label: &str, detail: &str, selected: bool, marked: bool) -> Rectangle {
    let (fill, color) = if selected {
        (t.bg3, t.fg)
    } else {
        (t.bg0, t.fg_dim)
    };

    let stripe = Rectangle::new()
        .width(STRIPE_WIDTH)
        .height(ROW_HEIGHT)
        .fill(if marked { t.accent } else { fill });

    let label = Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .align_child(Start, Center)
        .child(Text::new(label).size(TEXT_SIZE).color(color).elide());

    let detail = Text::new(detail).size(TEXT_SIZE).color(t.comment);

    // the stripe takes its width out of the left padding, so the text stays put
    let content = Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .padding(Padding {
            top: 0.0,
            right: PADDING,
            bottom: 0.0,
            left: PADDING - STRIPE_WIDTH,
        })
        .align_child(Start, Center)
        .child(
            Row::new(children![label, detail])
                .width(Parent)
                .gap(GAP)
                .align(Center),
        );

    Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .fill(fill)
        .child(Row::new(children![stripe, content]).width(Parent))
}

fn row_with(
    t: &Theme,
    number: Option<usize>,
    label: &str,
    selected: bool,
    label_color: Option<Color>,
) -> Rectangle {
    let (fill, color) = if selected {
        (t.bg3, t.fg)
    } else {
        (t.bg0, t.fg_dim)
    };

    let color = label_color.unwrap_or(color);

    let label = Text::new(label).size(TEXT_SIZE).color(color).elide();

    let row = Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .fill(fill)
        .padding(SIDES)
        .align_child(Start, Center);

    let Some(number) = number else {
        return row.child(label);
    };

    // a fixed-width number column, so the labels line up
    let number = Rectangle::new()
        .width(NUMBER_WIDTH)
        .height(ROW_HEIGHT)
        .align_child(Start, Center)
        .child(
            Text::new(number.to_string())
                .size(TEXT_SIZE)
                .color(t.comment),
        );

    let label = Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .align_child(Start, Center)
        .child(label);

    row.child(Row::new(children![number, label]).width(Parent))
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

// the width of a help row's first column, what to press or type
const HELP_KEYS: f32 = 140.0;

// a line of a help screen: what to press or type in the accent, then what it does
pub fn help_row(t: &Theme, keys: &str, does: &str) -> Rectangle {
    let keys = Rectangle::new()
        .width(HELP_KEYS)
        .height(ROW_HEIGHT)
        .align_child(Start, Center)
        .child(Text::new(keys).size(TEXT_SIZE).color(t.accent));

    let does = Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .align_child(Start, Center)
        .child(Text::new(does).size(TEXT_SIZE).color(t.fg_dim).elide());

    Rectangle::new()
        .width(Parent)
        .height(ROW_HEIGHT)
        .padding(SIDES)
        .align_child(Start, Center)
        .child(Row::new(children![keys, does]).width(Parent))
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
    use super::{Cursor, Key, follow_window, fuzzy, hotkey, step_highlight};

    #[test]
    fn a_highlight_starts_at_the_top_and_stays_in_the_list() {
        assert_eq!(step_highlight(None, 1, 3), Some(0));
        assert_eq!(step_highlight(None, -1, 3), Some(0));
        assert_eq!(step_highlight(Some(0), -1, 3), Some(0));
        assert_eq!(step_highlight(Some(2), 1, 3), Some(2));

        // the list shrank under it
        assert_eq!(step_highlight(Some(7), -1, 3), Some(1));
        assert_eq!(step_highlight(Some(1), 1, 0), None);
    }

    #[test]
    fn the_window_only_moves_when_the_highlight_would_leave_it() {
        // ten rows of 40 in room for four
        let heights = [40.0; 10];

        // everything up to row 3 fits, so nothing moves
        assert_eq!(follow_window(0, 3, 3, &heights, 160.0), 0);

        // row 4 needs the window one down, row 9 six down
        assert_eq!(follow_window(0, 4, 4, &heights, 160.0), 1);
        assert_eq!(follow_window(1, 9, 9, &heights, 160.0), 6);

        // going back up inside the window leaves it where it is
        assert_eq!(follow_window(6, 8, 8, &heights, 160.0), 6);

        // above it, the window starts at the highlight, or its heading
        assert_eq!(follow_window(6, 4, 5, &heights, 160.0), 4);
    }

    #[test]
    fn a_shorter_list_leaves_no_room_at_the_bottom() {
        // scrolled to the end with a password box open: six rows and a 44 row
        let open = [40.0, 40.0, 40.0, 40.0, 40.0, 40.0, 44.0];

        // rows 4 to 6 take 124, row 3 too would be 164, more than there's room for
        assert_eq!(follow_window(0, 5, 6, &open, 160.0), 4);

        // the box closed: starting at row 4 would leave 40 empty, rows 2 to 5 fill it
        let closed = [40.0; 6];

        assert_eq!(follow_window(4, 5, 5, &closed, 160.0), 2);
    }

    #[test]
    fn a_list_that_fits_never_moves() {
        let heights = [28.0, 40.0, 40.0, 28.0, 40.0];

        assert_eq!(follow_window(0, 4, 4, &heights, 400.0), 0);
    }

    #[test]
    fn digits_one_to_nine_are_hotkeys() {
        assert_eq!(hotkey(Key::Character('1')), Some(0));
        assert_eq!(hotkey(Key::Character('9')), Some(8));
        assert_eq!(hotkey(Key::Character('0')), None);
        assert_eq!(hotkey(Key::Character('a')), None);
    }

    #[test]
    fn positions_count_from_the_top_visible_row() {
        let mut cursor = Cursor::default();

        assert_eq!(cursor.at_position(3, 4), Some(3));
        assert_eq!(cursor.at_position(4, 4), None);

        cursor.scroll(2, 20);

        assert_eq!(cursor.at_position(0, 20), Some(2));
    }

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
