use amane::App;

mod bar;
mod clock;
mod lock;
mod notifications;
mod panels;
mod screenshot;
mod session;
mod theme;

fn main() {
    // locks on suspend and `loginctl lock-session`
    session::watch_logind();

    App::new()
        .font("BlexMono Nerd Font")
        .ipc("panel", panels::ipc)
        .ipc("lock", session::lock_ipc)
        .window_per_monitor(bar::view)
        .window_per_monitor(panels::backdrop::view)
        .window_per_monitor(notifications::popups::view)
        .window_per_monitor(panels::launcher::view)
        .window_per_monitor(panels::clipboard::view)
        .window_per_monitor(panels::screenshot::view)
        .window_per_monitor(panels::session::view)
        .window_per_monitor(panels::notifications::view)
        .lock(lock::view)
        .run();
}
