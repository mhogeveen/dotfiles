use amane::{
    Color, Full, Keyboard, Layer, LayerWindow, Monitor, Rectangle, Service, Size::Parent, Zone,
};

use super::Panels;

/*
 * a transparent window covering every monitor; amane has no focus-lost event,
 * so clicking here is how clicking outside closes panels. it sits on the top layer,
 * below the panels on the overlay layer
 *
 * it never hides, only lets clicks through while no panel is open: unmapping it
 * at the same moment a panel unmaps races mango's reconfigure of the other
 * surfaces, which kills the shell with a protocol error. the input region is
 * plain surface state, so changing it doesn't make mango reconfigure anything
 */
pub fn view(_monitor: &Monitor) -> LayerWindow {
    let any_open = Panels::read().any_open();

    let window = LayerWindow::new()
        .width(Full)
        .height(Full)
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
