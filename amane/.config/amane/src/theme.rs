/*
 * the shell's colors, and the accent picked in the system panel
 *
 * the five accent colors themselves are in AccentColor; every view calls
 * theme(), which reads the Accent service, so picking a new accent redraws
 * every window by itself: the bar, panels, popups, lock screen
 */

use std::env;
use std::fs;
use std::path::PathBuf;

use amane::{Color, Service};

pub struct Theme {
    // the one picked in the system panel, used for every accent in the shell
    pub accent: Color,

    // text on an accent fill; dark reads well on all five, light ones included
    pub on_accent: Color,

    pub fg: Color,
    pub fg_dim: Color,
    pub comment: Color,

    // part of the palette, not used anywhere yet
    #[allow(dead_code)]
    pub gutter: Color,
    pub bg_deep: Color,
    pub bg0: Color,
    pub bg1: Color,
    pub bg2: Color,
    pub bg3: Color,
    pub bg4: Color,
    pub bg5: Color,
}

pub fn theme() -> Theme {
    let accent = Accent::read().color;

    Theme {
        accent: accent.color(),
        on_accent: Color::from("#0f0c0a"),
        fg: Color::from("#e6d5c2"),
        fg_dim: Color::from("#a09384"),
        comment: Color::from("#73665b"),
        gutter: Color::from("#4e4641"),
        bg_deep: Color::from("#0f0c0a"),
        bg0: Color::from("#171311"),
        bg1: Color::from("#201b19"),
        bg2: Color::from("#2a2422"),
        bg3: Color::from("#362f2c"),
        bg4: Color::from("#463e3a"),
        bg5: Color::from("#5a504c"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentColor {
    Cinder,
    Ember,
    Brass,
    Sap,
    Frost,
}

// in the order the system panel shows them and `next` cycles through
pub const ACCENTS: [AccentColor; 5] = [
    AccentColor::Cinder,
    AccentColor::Ember,
    AccentColor::Brass,
    AccentColor::Sap,
    AccentColor::Frost,
];

impl AccentColor {
    pub fn name(self) -> &'static str {
        match self {
            AccentColor::Cinder => "cinder",
            AccentColor::Ember => "ember",
            AccentColor::Brass => "brass",
            AccentColor::Sap => "sap",
            AccentColor::Frost => "frost",
        }
    }

    pub fn color(self) -> Color {
        Color::from(match self {
            AccentColor::Cinder => "#d1766e",
            AccentColor::Ember => "#ea9875",
            AccentColor::Brass => "#fcba81",
            AccentColor::Sap => "#99af6b",
            AccentColor::Frost => "#4e89a2",
        })
    }

    pub fn from_name(name: &str) -> Option<Self> {
        ACCENTS
            .into_iter()
            .find(|accent| accent.name().eq_ignore_ascii_case(name.trim()))
    }

    fn next(self) -> Self {
        let index = ACCENTS
            .iter()
            .position(|accent| *accent == self)
            .unwrap_or(0);

        ACCENTS[(index + 1) % ACCENTS.len()]
    }
}

// the accent in use, kept in a state file so it survives restarts
pub struct Accent {
    color: AccentColor,
}

impl Service for Accent {
    // a file of a few bytes, quick enough for the first view that asks
    fn new() -> Self {
        let color = fs::read_to_string(state_file())
            .ok()
            .and_then(|name| AccentColor::from_name(&name))
            .unwrap_or(AccentColor::Frost);

        Self { color }
    }

    // only changes when picked, nothing to poll
    fn listen() {}
}

impl Accent {
    pub fn color(&self) -> AccentColor {
        self.color
    }

    // from a click or ipc; a file that can't be written only costs remembering it
    pub fn set(color: AccentColor) {
        Accent::write().color = color;

        let path = state_file();

        let saved = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::write(&path, color.name()));

        if let Err(error) = saved {
            eprintln!(
                "theme: couldn't save the accent to {}: {error}",
                path.display()
            );
        }
    }

    pub fn next() {
        let next = Accent::read().color.next();

        Accent::set(next);
    }
}

// $XDG_STATE_HOME/amane-shell/accent, outside the config repo so picking doesn't dirty git
fn state_file() -> PathBuf {
    let state = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| {
            let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();

            home.join(".local").join("state")
        });

    state.join("amane-shell").join("accent")
}

/*
 * `amane ipc call accent sap` sets it, `next` cycles, nothing prints the one in
 * use; runs on the draw thread like every ipc handler
 */
pub fn accent_ipc(arguments: &[String]) -> String {
    let Some(argument) = arguments.first() else {
        return String::from(Accent::read().color.name());
    };

    if argument == "next" {
        Accent::next();

        return String::from(Accent::read().color.name());
    }

    match AccentColor::from_name(argument) {
        Some(color) => {
            Accent::set(color);

            String::from(color.name())
        }
        None => {
            let names: Vec<&str> = ACCENTS.iter().map(|accent| accent.name()).collect();

            format!(
                "unknown accent '{argument}', pick one of: {}",
                names.join(", ")
            )
        }
    }
}
