/*
 * small pieces the panels share: sizes, the top-right window and frame,
 * buttons, switches, checkboxes, sliders and meters
 */

use amane::{
    Align::{Center, Start},
    Button, Color, Column,
    Cursor::Pointer,
    Horizontal, LayerWindow, Margin, Padding, Rectangle, Row,
    Size::Parent,
    Text, Vertical, Weight, Widget,
};

use super::hints;
use crate::theme::Theme;

pub const BUTTON: f32 = 28.0;

// nerd font material design refresh, on every button that reads or scans again
pub const ICON_REFRESH: &str = "\u{f0450}";

// BlexMono is monospaced: every character is this much of the text size wide
const CHARACTER_WIDTH: f32 = 0.6;

// how wide a text is at this size, for boxes that fit their text exactly
pub fn text_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * CHARACTER_WIDTH
}

// room on either side of a text button's label
const BUTTON_SIDES: f32 = 12.0;

/*
 * how wide a text button is: its label plus the same room on either side, so
 * every button has the same look whatever its label. a button whose label
 * changes (Mute, Unmute) is sized for the longest, so it doesn't shift. labels
 * are plain ascii, so bytes count as characters, which a const fn can count
 */
pub const fn fit(longest: &str) -> f32 {
    longest.len() as f32 * TEXT_NOTE * CHARACTER_WIDTH + BUTTON_SIDES * 2.0
}

// the room every panel keeps inside its border and on either side of a row
pub const PADDING: f32 = 12.0;

// room on the left and right only, so a row's content centers vertically
pub const SIDES: Padding = Padding {
    top: 0.0,
    right: PADDING,
    bottom: 0.0,
    left: PADDING,
};

// what every button is: this wide, a button tall, its content centered, clickable
fn base(width: f32, fill: Color, on_click: impl Fn(Button) + 'static) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(BUTTON)
        .fill(fill)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(on_click)
}

pub fn button(
    t: &Theme,
    label: &str,
    width: f32,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    base(width, t.bg2, on_click).child(Text::new(label).size(TEXT_NOTE).color(t.fg))
}

// a text button with a key that does the same; while ? hints show, the key replaces the label
pub fn key_text_button(
    t: &Theme,
    label: &str,
    width: f32,
    key: char,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    hint_text_button(t, label, width, &key.to_string(), on_click)
}

// the same with a key that has a name rather than a letter, like Enter
pub fn hint_text_button(
    t: &Theme,
    label: &str,
    width: f32,
    hint: &str,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    if !hints::shown() {
        return button(t, label, width, on_click);
    }

    base(width, t.bg2, on_click).child(key_text(hint, t.accent))
}

// a square button with a glyph in its own color, like a bell that lights up
pub fn icon_button_colored(
    t: &Theme,
    glyph: &str,
    color: Color,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    base(BUTTON, t.bg2, on_click).child(Text::new(glyph).size(TEXT_BODY).color(color))
}

// the text sizes the panels share
pub const TEXT_SMALL: f32 = 12.0;
pub const TEXT_NOTE: f32 = 13.0;
pub const TEXT_BODY: f32 = 14.0;
pub const TEXT_TITLE: f32 = 15.0;

// a glyph at the start of a row, like a device or wi-fi strength
pub const ICON_SIZE: f32 = 16.0;

// between the parts of a header, a card, or an error line
const GAP: f32 = 8.0;

// a card's border, around a group of lines like the network you're on
const CARD_BORDER: f32 = 1.0;

/*
 * a panel's header: the title (taking the room the rest leaves), then its
 * buttons; as tall as a button
 */
pub fn header(title: impl Widget + 'static, buttons: Vec<Box<dyn Widget>>) -> Row {
    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(title)];

    parts.extend(buttons);

    Row::new(parts).gap(GAP).align(Center).height(BUTTON)
}

// a dim line saying why a list is empty, like "Bluetooth is off"
pub fn note(t: &Theme, text: &str, height: f32) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(height)
        .padding(SIDES)
        .align_child(Start, Center)
        .child(Text::new(text).size(TEXT_NOTE).color(t.comment))
}

// one line of text in a box this tall, taking the width it's given
pub fn line(height: f32, text: Text) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(height)
        .align_child(Start, Center)
        .child(text)
}

// a bordered group of lines, this tall (cards add their parts' heights up)
pub fn card(t: &Theme, height: f32, parts: Vec<Box<dyn Widget>>) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(height)
        .fill(t.bg0)
        .border(CARD_BORDER, t.bg3)
        .padding(PADDING)
        .child(Column::new(parts).gap(GAP))
}

// what went wrong, in the accent, with a button to act on it when there is one
pub fn error_line(t: &Theme, text: &str, button: Option<Rectangle>) -> Row {
    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(line(
        BUTTON,
        Text::new(text).size(TEXT_NOTE).color(t.accent).elide(),
    ))];

    if let Some(button) = button {
        parts.push(Box::new(button));
    }

    Row::new(parts).gap(GAP).align(Center).height(BUTTON)
}

// a section heading's usual height
pub const HEADING: f32 = 16.0;

// a small bold dim title over a part of a panel, like "Settings" or "Next 7 days"
pub fn heading(t: &Theme, text: &str, height: f32) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(height)
        .align_child(Start, Center)
        .child(
            Text::new(text)
                .size(TEXT_SMALL)
                .weight(Weight::Bold)
                .color(t.comment),
        )
}

// the bold panel name on the left of a header row, taking the room the buttons leave
pub fn title(t: &Theme, text: &str) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            // shortened rather than running under the buttons beside it (a long weather place)
            Text::new(text)
                .size(TEXT_TITLE)
                .weight(Weight::Bold)
                .color(t.fg)
                .elide(),
        )
}

// an on/off switch like "Wi-Fi on" with a key that flips it too; click flips it
// sized for its "off" text, the longer, so flipping it doesn't move what's beside it
pub fn switch(
    t: &Theme,
    label: &str,
    on: bool,
    key: char,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    let (state, fill, color) = if on {
        ("on", t.accent, t.on_accent)
    } else {
        ("off", t.bg2, t.fg_dim)
    };

    let width = text_width(&format!("{label} off"), TEXT_NOTE) + BUTTON_SIDES * 2.0;

    base(width, fill, on_click).child(if hints::shown() {
        // the switch's own text color, the accent wouldn't show on an accent fill
        key_text(&key.to_string(), color)
    } else {
        Text::new(format!("{label} {state}"))
            .size(TEXT_NOTE)
            .color(color)
    })
}

// nerd font material design checkbox-marked and checkbox-blank-outline
const ICON_CHECKED: &str = "\u{f0132}";
const ICON_UNCHECKED: &str = "\u{f0131}";

/*
 * a checkbox for a setting that's on or off, filled in the accent when on,
 * with a key that flips it too; while ? hints show, the key replaces the box
 */
pub fn checkbox(
    t: &Theme,
    checked: bool,
    key: char,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    let content = if hints::shown() {
        key_text(&key.to_string(), t.accent)
    } else if checked {
        Text::new(ICON_CHECKED).size(ICON_SIZE).color(t.accent)
    } else {
        Text::new(ICON_UNCHECKED).size(ICON_SIZE).color(t.fg_dim)
    };

    Rectangle::new()
        .width(BUTTON)
        .height(BUTTON)
        .align_child(Start, Center)
        .cursor(Pointer)
        .on_click(on_click)
        .child(content)
}

const SLIDER_HEIGHT: f32 = 8.0;

/*
 * a 0 to 100 bar; pressing or dragging on it sets the value from where the
 * pointer is. the fill shows value, what the sound server says, so it follows
 * a drag as fast as the server answers
 */
pub fn slider(
    t: &Theme,
    width: f32,
    value: u8,
    dimmed: bool,
    on_change: impl Fn(u8) + 'static,
) -> Rectangle {
    let filled = width * f32::from(value.min(100)) / 100.0;

    let fill = Rectangle::new()
        .width(filled)
        .height(SLIDER_HEIGHT)
        .fill(if dimmed { t.bg5 } else { t.accent });

    // a taller invisible area around the thin bar, so it's easy to grab
    Rectangle::new()
        .width(width)
        .height(BUTTON)
        .align_child(Start, Center)
        .cursor(Pointer)
        .on_drag(move |point| {
            let picked = (point.x / width * 100.0).round().clamp(0.0, 100.0) as u8;

            // a drag moves in steps smaller than 1%, only real changes go out
            if picked != value {
                on_change(picked);
            }
        })
        .child(
            Rectangle::new()
                .width(width)
                .height(SLIDER_HEIGHT)
                .fill(t.bg2)
                .clip()
                .child(fill),
        )
}

// a 0 to 100 bar that only shows a level, like cpu or disk use, in the accent; dimmed like a muted slider
pub fn meter(t: &Theme, width: f32, percent: u8, dimmed: bool) -> Rectangle {
    let percent = percent.min(100);

    let fill = Rectangle::new()
        .width(width * f32::from(percent) / 100.0)
        .height(SLIDER_HEIGHT)
        .fill(if dimmed { t.bg5 } else { t.accent });

    Rectangle::new()
        .width(width)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            Rectangle::new()
                .width(width)
                .height(SLIDER_HEIGHT)
                .fill(t.bg2)
                .clip()
                .child(fill),
        )
}

// the line around a list box
pub const LIST_BORDER: f32 = 1.0;

/*
 * the box a corner panel's list goes in, taking the room left. amane clips a
 * rectangle at its outer edge, border included, so the rows are clipped by an
 * inner box inside the border instead: a row cut off at the bottom, or one
 * with a fill of its own (a highlight), can't paint over it
 */
pub fn list_box(t: &Theme, rows: impl Widget + 'static) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(t.bg0)
        .border(LIST_BORDER, t.bg3)
        .padding(LIST_BORDER)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .clip()
                .child(rows),
        )
}

// how wide the top-right panels are
pub const CORNER_WIDTH: f32 = 400.0;

// a top-right panel's window: below the bar, a little in from the edge
pub fn corner(height: f32, namespace: &'static str) -> LayerWindow {
    LayerWindow::new()
        .width(CORNER_WIDTH)
        .height(height)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 8,
            right: 8,
            bottom: 0,
            left: 0,
        })
        .namespace(namespace)
}

// a panel's background, border and padding around its parts, top to bottom
pub fn frame(t: &Theme, gap: f32, parts: Vec<Box<dyn Widget>>) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(t.bg_deep)
        .border(1.0, t.bg3)
        .padding(PADDING)
        .clip()
        .child(Column::new(parts).gap(gap))
}

// a square button with a key that does the same; while ? hints show, the key replaces the glyph
pub fn key_button(
    t: &Theme,
    glyph: &str,
    key: char,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    key_button_colored(t, glyph, t.fg, key, on_click)
}

// a square button with a glyph in its own color, like a bell that lights up
pub fn key_button_colored(
    t: &Theme,
    glyph: &str,
    color: Color,
    key: char,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    if !hints::shown() {
        return icon_button_colored(t, glyph, color, on_click);
    }

    base(BUTTON, t.bg2, on_click).child(key_text(&key.to_string(), t.accent))
}

// a key as the ? hints show it: a letter, or a name like Enter
fn key_text(key: &str, color: Color) -> Text {
    Text::new(key)
        .size(TEXT_BODY)
        .weight(Weight::Bold)
        .color(color)
}
