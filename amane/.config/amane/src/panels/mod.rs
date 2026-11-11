/*
 * panel manager: tracks which panels are open and enforces the relationships between them
 *
 * panels are layer windows that always exist and are shown or hidden through `.visible`,
 * since amane can't open or close layer windows at runtime
 *
 * from a keybind: `amane ipc call panel toggle launcher`
 */

use std::thread;
use std::time::Duration;

use amane::{Key, Keyboard, Layer, LayerWindow, Monitor, Rectangle, Service, Workspaces};

use crate::theme::{Theme, theme};

pub mod audio;
pub mod backdrop;
pub mod bluetooth;
pub mod calendar;
pub mod clipboard;
mod hints;
pub mod launcher;
pub mod layout;
mod list;
mod menu;
pub mod network;
pub mod notifications;
mod picker;
pub mod screenshot;
pub mod session;
pub mod system;
pub mod ui;
pub mod wallpaper;

pub const LAUNCHER: &str = "launcher";
pub const CLIPBOARD: &str = "clipboard";
pub const NOTIFICATIONS: &str = "notifications";
pub const SCREENSHOT: &str = "screenshot";
pub const SESSION: &str = "session";
pub const NETWORK: &str = "network";
pub const BLUETOOTH: &str = "bluetooth";
pub const AUDIO: &str = "audio";
pub const LAYOUT: &str = "layout";
pub const SYSTEM: &str = "system";
pub const CALENDAR: &str = "calendar";
pub const WALLPAPER: &str = "wallpaper";

pub struct PanelDef {
    pub name: &'static str,

    // opening a panel closes every open panel that shares at least one group with it
    pub groups: &'static [&'static str],

    /*
     * runs whenever the panel closes, however that happens, to clear its search and
     * selection; always on the draw thread, so it may touch TextInput. None when
     * there is nothing to clear
     */
    pub on_close: Option<fn()>,

    // sits in the top-right corner, where notification popups show, which hide while it's open
    pub corner: bool,
}

/*
 * the group every panel is in, so only one is ever open: when a panel that has
 * the keyboard closes, mango gives the keyboard to a window, never to another
 * panel, so escape could no longer reach a panel left open underneath
 *
 * except the screenshot menu, which opens on top so open panels can be captured;
 * the panel underneath loses the keyboard once the menu (or slurp) closes, and
 * gets it back when clicked
 */
const PANELS: &str = "panels";

// every panel and its relationships, the single place to configure them
static REGISTRY: &[PanelDef] = &[
    PanelDef {
        name: LAUNCHER,
        groups: &[PANELS],
        on_close: Some(launcher::reset),
        corner: false,
    },
    PanelDef {
        name: CLIPBOARD,
        groups: &[PANELS],
        on_close: Some(clipboard::reset),
        corner: false,
    },
    PanelDef {
        name: SCREENSHOT,
        groups: &[],
        on_close: Some(screenshot::reset),
        corner: false,
    },
    PanelDef {
        name: SESSION,
        groups: &[PANELS],
        on_close: Some(session::reset),
        corner: false,
    },
    PanelDef {
        name: LAYOUT,
        groups: &[PANELS],
        on_close: Some(layout::reset),
        corner: false,
    },
    PanelDef {
        name: NOTIFICATIONS,
        groups: &[PANELS],
        on_close: Some(notifications::reset),
        corner: true,
    },
    PanelDef {
        name: NETWORK,
        groups: &[PANELS],
        on_close: Some(network::reset),
        corner: true,
    },
    PanelDef {
        name: BLUETOOTH,
        groups: &[PANELS],
        on_close: Some(bluetooth::reset),
        corner: true,
    },
    PanelDef {
        name: AUDIO,
        groups: &[PANELS],
        on_close: None,
        corner: true,
    },
    PanelDef {
        name: SYSTEM,
        groups: &[PANELS],
        on_close: None,
        corner: true,
    },
    PanelDef {
        name: CALENDAR,
        groups: &[PANELS],
        on_close: Some(calendar::reset),
        corner: false,
    },
    PanelDef {
        name: WALLPAPER,
        groups: &[PANELS],
        on_close: Some(wallpaper::reset),
        corner: false,
    },
];

fn find(name: &str) -> Option<&'static PanelDef> {
    REGISTRY.iter().find(|panel| panel.name == name)
}

fn conflicts(a: &PanelDef, b: &PanelDef) -> bool {
    a.name != b.name && a.groups.iter().any(|group| b.groups.contains(group))
}

struct OpenPanel {
    def: &'static PanelDef,

    // the monitor that was focused when the panel opened; None shows it everywhere
    output: Option<String>,
}

pub struct Panels {
    // oldest first
    open: Vec<OpenPanel>,

    // bumped on every change, so a delayed open can tell it was overtaken
    generation: u64,
}

/*
 * when opening a panel replaces another, the new one waits this long so the old one
 * is unmapped first; showing a layer surface makes mango reconfigure every surface on
 * the monitor, and a configure that crosses the old panel's unmap gets acked after
 * wlroots has dropped it, which kills the shell with a protocol error
 */
const SWAP_DELAY: Duration = Duration::from_millis(100);

impl Service for Panels {
    fn new() -> Self {
        Self {
            open: Vec::new(),
            generation: 0,
        }
    }

    // only changes through input and ipc
    fn listen() {}
}

// reading, safe inside views
impl Panels {
    pub fn is_open(&self, name: &str) -> bool {
        self.open.iter().any(|panel| panel.def.name == name)
    }

    pub fn any_open(&self) -> bool {
        !self.open.is_empty()
    }

    // whether a panel in the top-right corner is open
    pub fn corner_open(&self) -> bool {
        self.open.iter().any(|panel| panel.def.corner)
    }

    pub fn open_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.open.iter().map(|panel| panel.def.name)
    }

    pub fn shown_on(&self, name: &str, monitor: &Monitor) -> bool {
        self.open.iter().any(|panel| {
            panel.def.name == name
                && panel
                    .output
                    .as_deref()
                    .is_none_or(|output| output == monitor.name)
        })
    }
}

/*
 * writing; these take the write lock, so never call them from inside a view,
 * only from input handlers and ipc
 */
impl Panels {
    pub fn open(name: &str) -> Result<(), String> {
        let def = find(name).ok_or_else(|| unknown(name))?;
        let output = focused_output();

        let mut panels = Panels::write();

        let replaced = panels.remove(|other| conflicts(def, other));

        panels.generation += 1;

        if replaced.is_empty() {
            panels.show(def, output);

            return Ok(());
        }

        // the replaced panels hide now, this one shows once they are gone
        let generation = panels.generation;

        drop(panels);

        run_close_hooks(&replaced);

        thread::spawn(move || {
            thread::sleep(SWAP_DELAY);

            let mut panels = Panels::write();

            // anything that happened in between wins
            if panels.generation == generation {
                panels.show(def, output);
            }
        });

        Ok(())
    }

    pub fn close(name: &str) -> Result<(), String> {
        find(name).ok_or_else(|| unknown(name))?;

        let closed = Panels::write().remove(|other| other.name == name);

        run_close_hooks(&closed);

        Ok(())
    }

    // returns whether the panel is open afterwards
    pub fn toggle(name: &str) -> Result<bool, String> {
        if Panels::read().is_open(name) {
            Panels::close(name)?;

            Ok(false)
        } else {
            Panels::open(name)?;

            Ok(true)
        }
    }

    pub fn close_all() {
        let closed = Panels::write().remove(|_| true);

        run_close_hooks(&closed);
    }

    // takes out the open panels that match, and bumps the generation
    fn remove(&mut self, matches: impl Fn(&PanelDef) -> bool) -> Vec<&'static PanelDef> {
        let mut removed = Vec::new();

        self.open.retain(|panel| {
            if matches(panel.def) {
                removed.push(panel.def);

                false
            } else {
                true
            }
        });

        self.generation += 1;

        removed
    }

    fn show(&mut self, def: &'static PanelDef, output: Option<String>) {
        if let Some(panel) = self
            .open
            .iter_mut()
            .find(|panel| panel.def.name == def.name)
        {
            panel.output = output;
        } else {
            self.open.push(OpenPanel { def, output });
        }
    }
}

// called without the Panels lock held, so hooks may read or write any service
fn run_close_hooks(closed: &[&'static PanelDef]) {
    if !closed.is_empty() {
        hints::hide();
    }

    for def in closed {
        if let Some(on_close) = def.on_close {
            on_close();
        }
    }
}

fn unknown(name: &str) -> String {
    let known: Vec<&str> = REGISTRY.iter().map(|panel| panel.name).collect();

    format!("unknown panel '{name}', known panels: {}", known.join(", "))
}

// whether this monitor has focus; true everywhere when the compositor doesn't say
pub fn is_focused(monitor: &Monitor) -> bool {
    focused_output().is_none_or(|output| output == monitor.name)
}

pub fn focused_output() -> Option<String> {
    Workspaces::read()
        .list()
        .iter()
        .find(|workspace| workspace.focused())
        .and_then(|workspace| workspace.output())
        .map(String::from)
}

const USAGE: &str = "usage: panel open|close|toggle <name> | panel close-all | panel list";

// `amane ipc call panel <command> [name]`
pub fn ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str);
    let name = arguments.get(1).map(String::as_str);

    let result = match (command, name) {
        (Some("open"), Some(name)) => Panels::open(name).map(|_| "opened"),
        (Some("close"), Some(name)) => Panels::close(name).map(|_| "closed"),
        (Some("toggle"), Some(name)) => {
            Panels::toggle(name).map(|open| if open { "opened" } else { "closed" })
        }
        (Some("close-all"), None) => {
            Panels::close_all();

            Ok("closed")
        }
        (Some("list"), None) => {
            let names: Vec<&str> = Panels::read().open_names().collect();

            return names.join("\n");
        }
        _ => Err(String::from(USAGE)),
    };

    match result {
        Ok(reply) => String::from(reply),
        Err(error) => error,
    }
}

/*
 * every panel's window: shown on the monitor it was opened on, above the
 * backdrop, taking the keyboard and closing on escape; every other key that
 * reaches the window goes to the panel's own handler, except ? which shows the
 * key hints (see hints.rs)
 *
 * content is only built while the panel shows. a hidden window still runs its
 * view whenever a service it read changes, so a panel that read the network or
 * the volume while hidden would run again on every signal-strength or volume
 * change; built lazily, a hidden panel reads nothing but Panels
 *
 * on-demand rather than exclusive: amane keeps a hidden window's last keyboard mode,
 * and mango hands exclusive focus to any exclusive layer surface, even unmapped ones,
 * so a hidden exclusive panel would keep stealing focus from your windows.
 * mango still focuses an on-demand surface as soon as it shows, and the backdrop
 * covers everything else, so the panel keeps the keyboard while it's open
 */
// the letter a key typed, lowercased, so a held Shift doesn't change what it does; None for other keys
pub fn letter(key: &Key) -> Option<char> {
    match key {
        Key::Character(letter) => Some(letter.to_ascii_lowercase()),
        _ => None,
    }
}

/*
 * for Escape: whether the panel closed something inside it instead, like the
 * network panel's password box (a text input lets go of Escape and passes it
 * on, so it would close the whole panel and lose your place in the list)
 */
fn backs_out(name: &str) -> bool {
    match name {
        NETWORK => network::back_out(),
        _ => false,
    }
}

pub fn panel(
    monitor: &Monitor,
    name: &'static str,
    base: LayerWindow,
    on_key: fn(Key),
    content: impl FnOnce(&Theme) -> Rectangle,
) -> LayerWindow {
    let shown = Panels::read().shown_on(name, monitor);

    // a window needs a child even while hidden, though nothing draws it
    let child = if shown {
        content(&theme())
    } else {
        Rectangle::new().width(1.0).height(1.0)
    };

    base.layer(Layer::Overlay)
        .visible(shown)
        .keyboard(Keyboard::OnDemand)
        .on_key(move |key| {
            match key {
                // something open inside the panel closes first, the panel on the next press
                Key::Escape if backs_out(name) => {}

                Key::Escape => {
                    let _ = Panels::close(name);
                }

                // every panel shows its buttons' keys for a moment
                Key::Character('?') => hints::show(),

                _ => on_key(key),
            }
        })
        .child(child)
}
