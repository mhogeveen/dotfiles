use amane::App;

mod audio;
mod bar;
mod bluetooth;
mod calculator;
mod calendar;
mod clipboard_preview;
mod clock;
mod json;
mod layout;
mod lock;
mod network;
mod notifications;
mod panels;
mod screenshot;
mod session;
mod state;
mod system;
mod theme;
mod usage;
mod vpn;
mod weather;

fn main() {
    // locks on suspend and `loginctl lock-session`
    session::watch_logind();

    // answers BlueZ when a device asks to confirm a pairing code, and registers again
    // when bluetoothd restarts
    bluetooth::agent::start();
    bluetooth::start();

    // records the clipboard from the start, the panel only reads it when open
    panels::clipboard::start();

    App::new()
        .font("BlexMono Nerd Font")
        .ipc("panel", panels::ipc)
        .ipc("lock", session::lock_ipc)
        .ipc("accent", theme::accent_ipc)
        .ipc("weather", weather::ipc)
        .window_per_monitor(bar::view)
        .window_per_monitor(panels::backdrop::view)
        .window_per_monitor(notifications::popups::view)
        .window_per_monitor(panels::launcher::view)
        .window_per_monitor(panels::clipboard::view)
        .window_per_monitor(panels::screenshot::view)
        .window_per_monitor(panels::session::view)
        .window_per_monitor(panels::layout::view)
        .window_per_monitor(panels::notifications::view)
        .window_per_monitor(panels::network::view)
        .window_per_monitor(panels::bluetooth::view)
        .window_per_monitor(panels::audio::view)
        .window_per_monitor(panels::system::view)
        .window_per_monitor(panels::calendar::view)
        .lock(lock::view)
        .run();
}
