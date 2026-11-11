/*
 * the backdrop: a transparent window covering every monitor below the bar;
 * amane has no focus-lost event, so clicking here is how clicking outside
 * closes panels. it sits on the top layer, below the panels on the overlay layer
 *
 * it starts below the bar (bar::HEIGHT), so the bar stays clickable while a
 * panel is open: clicking one of its icons goes straight to that one's panel
 *
 * it never hides, only lets clicks through while no panel is open: unmapping it
 * at the same moment a panel unmaps races mango's reconfigure of the other
 * surfaces, which kills the shell with a protocol error. the input region is
 * plain surface state, so changing it doesn't make mango reconfigure anything
 */

use amane::{
    Color, Full, Keyboard, Layer, LayerWindow, Margin, Monitor, Rectangle, Service, Size::Parent,
    Zone,
};

use super::Panels;
use crate::bar;

pub fn view(_monitor: &Monitor) -> LayerWindow {
    let any_open = Panels::read().any_open();

    let window = LayerWindow::new()
        .width(Full)
        .height(Full)
        .margin(Margin {
            top: bar::HEIGHT as i32,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .keyboard(Keyboard::None)
        .namespace("amane-backdrop")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::TRANSPARENT)
                .on_click(|_| Panels::close_all()),
        );

    if any_open {
        window
    } else {
        window.click_through()
    }
}
