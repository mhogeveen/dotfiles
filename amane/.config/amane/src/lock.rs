/*
 * the lock screen, shown on every monitor while the session is locked: the time,
 * the date, who's logged in and a password box; the password is checked through
 * PAM's login service, the same as logging in on a tty
 *
 * lock from the session panel, `amane ipc call lock`, `loginctl lock-session` or
 * by suspending (see session.rs)
 *
 * test it with another tty logged in: a locked session stays locked even when
 * the shell dies, `loginctl terminate-session <id>` from there gets you out
 */

use std::env;
use std::path::PathBuf;

use amane::{
    Align::{Center, Start},
    Column, Image, LayerWindow, Lock, Monitor, Padding, Rectangle, Service,
    Size::Parent,
    Stack, Text, TextInput, Weight, Widget, children,
};

use crate::clock::Clock;
use crate::theme::{Theme, theme};
use crate::wallpaper::Wallpaper;

// the wallpaper behind the lock screen: a small copy, blurred once, then darkened
const BLURRED: (u32, u32) = (320, 180);
const BLUR: u32 = 6;
const DARKEN: f32 = 0.6;

const INPUT: &str = "lock-password";

pub fn view(_monitor: &Monitor) -> LayerWindow {
    let t = theme();
    let clock = Clock::read();
    let lock = Lock::read();

    let (status, status_color) = if lock.checking() {
        ("Checking…", t.fg_dim)
    } else if lock.failed() {
        ("Wrong password", t.accent)
    } else {
        ("", t.fg_dim)
    };

    let user = env::var("USER").unwrap_or_default();

    /*
     * every monitor shows its own box under the same name, so they hold the same
     * text and typing works on whichever monitor the compositor sends keys to
     */
    let password = Rectangle::new()
        .width(320.0)
        .height(44.0)
        .fill(t.bg1)
        .border(1.0, if lock.failed() { t.accent } else { t.bg3 })
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .align_child(Start, Center)
        .child(
            TextInput::new(INPUT)
                .placeholder("Password")
                .password()
                .focused()
                .size(16.0)
                .color(t.fg)
                .on_submit(|password| {
                    TextInput::set_text(INPUT, "");

                    Lock::unlock(&password);
                }),
        );

    let content = Column::new(children![
        Text::new(clock.time())
            .size(72.0)
            .weight(Weight::Bold)
            .color(t.fg),
        Text::new(clock.date()).size(18.0).color(t.fg_dim),
        // room between the clock and the login part
        Rectangle::new().width(1.0).height(32.0),
        Text::new(user).size(16.0).color(t.fg),
        password,
        Text::new(status).size(14.0).color(status_color),
    ])
    .gap(12.0)
    .align(Center);

    let mut layers = background(&t);

    layers.push(Box::new(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .align_child(Center, Center)
            .child(content),
    ));

    // the compositor sizes lock screens to the monitor, so this size is never used
    LayerWindow::new()
        .width(1.0)
        .height(1.0)
        .child(Stack::new(layers).width(Parent).height(Parent))
}

/*
 * the wallpaper in use, blurred and darkened so the clock and password box read
 * well on any picture; plain bg_deep without one, the same as mango's root_color.
 * a lock surface can't show what's under it, so none can't be see-through here
 */
fn background(t: &Theme) -> Vec<Box<dyn Widget>> {
    let plain = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(t.bg_deep);

    let Some(picture) = Wallpaper::read().current().map(PathBuf::from) else {
        return vec![Box::new(plain)];
    };

    let blurred = Rectangle::new().width(Parent).height(Parent).fill(
        Image::cover(picture)
            .thumbnail(BLURRED.0, BLURRED.1)
            .blurred(BLUR),
    );

    let dark = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(t.bg_deep)
        .opacity(DARKEN);

    vec![Box::new(plain), Box::new(blurred), Box::new(dark)]
}
