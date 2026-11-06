use amane::App;

mod bar;
mod clock;
mod notifications;
mod panels;
mod screenshot;
mod theme;

fn main() {
    App::new()
        .font("BlexMono Nerd Font")
        .ipc("panel", panels::ipc)
        .window_per_monitor(bar::view)
        .window_per_monitor(panels::backdrop::view)
        .window_per_monitor(notifications::popups::view)
        .window_per_monitor(panels::launcher::view)
        .window_per_monitor(panels::clipboard::view)
        .window_per_monitor(panels::screenshot::view)
        .window_per_monitor(panels::notifications::view)
        .run();
}
