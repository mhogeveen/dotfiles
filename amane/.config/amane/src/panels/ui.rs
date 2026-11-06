/*
 * small pieces the corner panels (network, bluetooth) share
 */

use amane::{
    Align::{Center, Start},
    Button,
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
