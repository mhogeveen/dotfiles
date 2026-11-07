/*
 * the shell's colors, and the accent picked in the system panel
 *
 * the five accent colors themselves are in AccentColor; every view calls
 * theme(), which reads the Accent service, so picking a new accent redraws
 * every window by itself: the bar, panels, popups, lock screen
 */

use std::fs;

use amane::{Color, Service};

use crate::state;

// the file in the state folder the accent is kept in
const STATE: &str = "accent";

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
        on_accent: BG_DEEP,
        fg: Color::rgb(0xe6, 0xd5, 0xc2),
        fg_dim: Color::rgb(0xa0, 0x93, 0x84),
        comment: Color::rgb(0x73, 0x66, 0x5b),
        gutter: Color::rgb(0x4e, 0x46, 0x41),
        bg_deep: BG_DEEP,
        bg0: Color::rgb(0x17, 0x13, 0x11),
        bg1: Color::rgb(0x20, 0x1b, 0x19),
        bg2: Color::rgb(0x2a, 0x24, 0x22),
        bg3: Color::rgb(0x36, 0x2f, 0x2c),
        bg4: Color::rgb(0x46, 0x3e, 0x3a),
        bg5: Color::rgb(0x5a, 0x50, 0x4c),
    }
}

// the darkest background, also the text on an accent fill
const BG_DEEP: Color = Color::rgb(0x0f, 0x0c, 0x0a);

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

    pub const fn color(self) -> Color {
        match self {
            AccentColor::Cinder => Color::rgb(0xd1, 0x76, 0x6e),
            AccentColor::Ember => Color::rgb(0xea, 0x98, 0x75),
            AccentColor::Brass => Color::rgb(0xfc, 0xba, 0x81),
            AccentColor::Sap => Color::rgb(0x99, 0xaf, 0x6b),
            AccentColor::Frost => Color::rgb(0x4e, 0x89, 0xa2),
        }
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
        let color = fs::read_to_string(state::path(STATE))
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

        if let Err(error) = state::save(STATE, color.name()) {
            eprintln!("theme: couldn't save the accent: {error}");
        }
    }

    pub fn next() {
        let next = Accent::read().color.next();

        Accent::set(next);
    }
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
