/*
 * small pieces the corner panels (network, bluetooth, audio, system) share
 */

use amane::{
    Align::{Center, Start},
    Button, Color,
    Cursor::Pointer,
    Rectangle,
    Size::Parent,
    Text, Weight,
};

use crate::theme::Theme;

pub const BUTTON: f32 = 28.0;

pub fn button(
    t: &Theme,
    label: &str,
    width: f32,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(BUTTON)
        .fill(t.bg2)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(on_click)
        .child(Text::new(label).size(13.0).color(t.fg))
}

// a square button with a glyph, like rescan or the way into the session menu
pub fn icon_button(t: &Theme, glyph: &str, on_click: impl Fn(Button) + 'static) -> Rectangle {
    icon_button_colored(t, glyph, t.fg, on_click)
}

// the same, with the glyph in its own color, like a bell that lights up
pub fn icon_button_colored(
    t: &Theme,
    glyph: &str,
    color: Color,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    Rectangle::new()
        .width(BUTTON)
        .height(BUTTON)
        .fill(t.bg2)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(on_click)
        .child(Text::new(glyph).size(14.0).color(color))
}

// the bold panel name on the left of a header row, taking the room the buttons leave
pub fn title(t: &Theme, text: &str) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(Text::new(text).size(15.0).weight(Weight::Bold).color(t.fg))
}

// an on/off switch like "Wi-Fi on"; click flips it
pub fn switch(t: &Theme, label: &str, on: bool, on_click: impl Fn(Button) + 'static) -> Rectangle {
    let (state, fill, color) = if on {
        ("on", t.frost, t.fg)
    } else {
        ("off", t.bg2, t.fg_dim)
    };

    Rectangle::new()
        .width(label.chars().count() as f32 * 8.0 + 52.0)
        .height(BUTTON)
        .fill(fill)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(on_click)
        .child(
            Text::new(format!("{label} {state}"))
                .size(13.0)
                .color(color),
        )
}

const SLIDER_HEIGHT: f32 = 8.0;

/*
 * a 0 to 100 bar; pressing or dragging on it sets the value from where the
 * pointer is. the fill shows value, what the sound server says, so it follows
 * a drag as fast as the server answers
 */
pub fn slider(t: &Theme, width: f32, value: u8, dimmed: bool, on_change: fn(u8)) -> Rectangle {
    let filled = width * f32::from(value.min(100)) / 100.0;

    let fill = Rectangle::new()
        .width(filled)
        .height(SLIDER_HEIGHT)
        .fill(if dimmed { t.bg5 } else { t.frost });

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

/*
 * a 0 to 100 bar that only shows a level, like cpu or disk use; it turns
 * brass when getting full and red when nearly full
 */
pub fn meter(t: &Theme, width: f32, percent: u8) -> Rectangle {
    let percent = percent.min(100);

    let color = match percent {
        95.. => t.cinder,
        80.. => t.brass,
        _ => t.frost,
    };

    let fill = Rectangle::new()
        .width(width * f32::from(percent) / 100.0)
        .height(SLIDER_HEIGHT)
        .fill(color);

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
