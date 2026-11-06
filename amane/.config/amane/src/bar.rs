use amane::{
    Align::Center, Cursor::Pointer, Full, Layer, LayerWindow, Monitor, Padding, Rectangle, Row,
    Service, Size::Parent, Vertical, Widget, Workspaces, Zone, children,
};

use crate::{
    clock,
    theme::{Theme, theme},
};

pub fn view(_monitor: &Monitor) -> LayerWindow {
    let t = theme();

    LayerWindow::new()
        .width(Full)
        .height(32.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .space(Zone::Reserve)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(t.bg_deep)
                .padding(Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 8.0,
                })
                .child(
                    // the two Parent-sized sides split the leftover space evenly,
                    // which keeps the fixed-width clock exactly centered
                    Row::new(children![
                        workspaces(&t),
                        clock(&t),
                        Rectangle::new().width(Parent).height(Parent),
                    ])
                    .height(Parent)
                    .align(Center),
                ),
        )
}

fn workspaces(t: &Theme) -> Rectangle {
    let workspaces = Workspaces::read();

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for workspace in workspaces.list() {
        let (width, fill) = if workspace.focused() {
            (20.0, t.frost)
        } else if workspace.windows() > 0 {
            (8.0, t.bg4)
        } else {
            (8.0, t.bg1)
        };

        let id = workspace.id();

        buttons.push(Box::new(
            Rectangle::new()
                .width(width)
                .height(8.0)
                .align_child(Center, Center)
                .fill(fill)
                .cursor(Pointer)
                .on_click(move |_| Workspaces::focus(id)),
        ));
    }

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .child(Row::new(buttons).gap(4.0).align(Center).height(Parent))
}

fn clock(t: &Theme) -> Rectangle {
    clock::view(t)
}
