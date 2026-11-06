use amane::App;

mod bar;
mod clock;
mod launcher;
mod theme;

fn main() {
    App::new()
        .font("BlexMono Nerd Font")
        .window_per_monitor(bar::view)
        .run();
}
