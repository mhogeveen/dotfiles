/*
 * amane-shell: a desktop shell for mango on the amane library. this starts the
 * parts that must run from launch, registers the ipc handlers, and makes every
 * window once per monitor; the order of the windows is their stacking order,
 * bottom first: wallpaper, bar, backdrop, popups, every panel, the screenshot
 * menu (over the panels it can capture), the lock screen
 */

use amane::App;

mod animations;
mod audio;
mod bar;
mod bluetooth;
mod brightness;
mod calculator;
mod calendar;
mod clipboard_preview;
mod clock;
mod command;
mod covers;
mod json;
mod keybinds;
mod layout;
mod lock;
mod network;
mod notifications;
mod osd;
mod panels;
mod recording;
mod screenshot;
mod session;
mod state;
mod system;
mod theme;
mod usage;
mod vpn;
mod wallpaper;
mod weather;

fn main() {
    // locks on suspend and `loginctl lock-session`
    session::watch_logind();

    /*
     * answers BlueZ when a device asks to confirm a pairing code, and registers
     * again when bluetoothd restarts
     */
    bluetooth::agent::start();
    bluetooth::start();

    // records the clipboard from the start, the panel only reads it when open
    panels::clipboard::start();

    // restores the notifications from before a restart, and keeps saving them
    notifications::history::start();

    // the volume popup follows every change from the start
    osd::start();

    App::new()
        .font("BlexMono Nerd Font")
        .ipc("panel", panels::ipc)
        .ipc("lock", session::lock_ipc)
        .ipc("accent", theme::accent_ipc)
        .ipc("weather", weather::ipc)
        .ipc("media", osd::media_ipc)
        .ipc("brightness", brightness::ipc)
        .ipc("record", recording::ipc)
        // first, so it is the bottom-most window
        .window_per_monitor(wallpaper::view)
        .window_per_monitor(bar::view)
        .window_per_monitor(panels::backdrop::view)
        .window_per_monitor(notifications::popups::view)
        .window_per_monitor(osd::view)
        .window_per_monitor(panels::launcher::view)
        .window_per_monitor(panels::clipboard::view)
        .window_per_monitor(panels::session::view)
        .window_per_monitor(panels::layout::view)
        .window_per_monitor(panels::notifications::view)
        .window_per_monitor(panels::network::view)
        .window_per_monitor(panels::bluetooth::view)
        .window_per_monitor(panels::audio::view)
        .window_per_monitor(panels::system::view)
        .window_per_monitor(panels::calendar::view)
        .window_per_monitor(panels::wallpaper::view)
        .window_per_monitor(panels::keybinds::view)
        // after every panel, so it opens over one left open underneath
        .window_per_monitor(panels::screenshot::view)
        .lock(lock::view)
        .run();
}
