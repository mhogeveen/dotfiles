/*
 * mango's layout on each monitor, for the bar's [S] and the layout panel
 *
 * amane already follows mango for the workspaces, but doesn't pass the layout
 * on, so this opens its own `watch all-monitors` on mango's socket; mango sends
 * the whole state as one json line on every change
 *
 * amane's json library isn't available to the config, so a small scanner reads
 * just the two fields needed (see parse)
 */

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

use amane::Service;

// how long to wait before trying mango's socket again
const RETRY: Duration = Duration::from_secs(5);

pub struct Layout {
    pub symbol: &'static str,

    // what mango's setlayout takes
    pub name: &'static str,
}

impl Layout {
    // vertical_scroller reads better as "vertical scroller"
    pub fn label(&self) -> String {
        self.name.replace('_', " ")
    }
}

// in mango's own order, the order switch_layout cycles through (src/layout/arrange.c)
pub static LAYOUTS: &[Layout] = &[
    Layout {
        symbol: "T",
        name: "tile",
    },
    Layout {
        symbol: "S",
        name: "scroller",
    },
    Layout {
        symbol: "G",
        name: "grid",
    },
    Layout {
        symbol: "M",
        name: "monocle",
    },
    Layout {
        symbol: "K",
        name: "deck",
    },
    Layout {
        symbol: "CT",
        name: "center_tile",
    },
    Layout {
        symbol: "RT",
        name: "right_tile",
    },
    Layout {
        symbol: "VS",
        name: "vertical_scroller",
    },
    Layout {
        symbol: "VT",
        name: "vertical_tile",
    },
    Layout {
        symbol: "VG",
        name: "vertical_grid",
    },
    Layout {
        symbol: "VK",
        name: "vertical_deck",
    },
    Layout {
        symbol: "DW",
        name: "dwindle",
    },
    Layout {
        symbol: "F",
        name: "fair",
    },
    Layout {
        symbol: "VF",
        name: "vertical_fair",
    },
];

#[derive(Debug, Clone, PartialEq, Default)]
struct Monitor {
    name: String,
    symbol: String,
}

// each monitor's layout symbol, by monitor name
pub struct MangoLayout {
    monitors: Vec<Monitor>,
}

impl Service for MangoLayout {
    fn new() -> Self {
        Self {
            monitors: Vec::new(),
        }
    }

    // mango sends every change, nothing to poll
    fn listen() {
        // not running under mango, there's nothing to follow
        if env::var_os("MANGO_INSTANCE_SIGNATURE").is_none() {
            return;
        }

        loop {
            if let Some(stream) = send("watch all-monitors") {
                for line in BufReader::new(stream).lines() {
                    let Ok(line) = line else {
                        break;
                    };

                    let Some(monitors) = parse(&line) else {
                        continue;
                    };

                    // a window title changing sends everything again, without a new layout
                    if MangoLayout::read().monitors != monitors {
                        MangoLayout::write().monitors = monitors;
                    }
                }
            }

            thread::sleep(RETRY);
        }
    }
}

impl MangoLayout {
    // like "S", or the overview's own glyph while it's open
    pub fn symbol(&self, monitor: &str) -> Option<&str> {
        self.monitors
            .iter()
            .find(|candidate| candidate.name == monitor)
            .map(|candidate| candidate.symbol.as_str())
    }
}

// the focused monitor's current tag gets it, like a keybind would; waits on mango, so on a thread
pub fn set_layout(name: &'static str) {
    thread::spawn(move || {
        let Some(stream) = send(&format!("dispatch setlayout,{name}")) else {
            return;
        };

        // waiting for mango's answer keeps the request from being cut off
        let mut reply = String::new();

        let _ = BufReader::new(stream).read_line(&mut reply);
    });
}

fn send(command: &str) -> Option<UnixStream> {
    let path = env::var_os("MANGO_INSTANCE_SIGNATURE")?;

    let mut stream = UnixStream::connect(path).ok()?;

    stream.write_all(format!("{command}\n").as_bytes()).ok()?;

    Some(stream)
}

/*
 * the name and layout_symbol of each object in the top-level "monitors" array:
 *
 *   {"monitors":[{"name":"DP-1",...,"layout_symbol":"S","tags":[{..."layout":"S"}],...}]}
 *
 * keys only count directly inside a monitor object, so the tags' "layout" and
 * nested objects never get mixed in; None when there's no monitors array
 */
fn parse(line: &str) -> Option<Vec<Monitor>> {
    let mut characters = line.chars();

    // how many objects and arrays the scanner is inside of
    let mut depth = 0usize;

    // the depth just inside the monitors array, once it opened
    let mut monitors_at: Option<usize> = None;
    let mut found_monitors = false;

    let mut key: Option<String> = None;
    let mut wants_value = false;

    let mut current: Option<Monitor> = None;
    let mut monitors = Vec::new();

    while let Some(character) = characters.next() {
        match character {
            '"' => {
                let text = read_string(&mut characters)?;

                if !wants_value {
                    key = Some(text);

                    continue;
                }

                let in_monitor = monitors_at.is_some_and(|at| depth == at + 1);

                if let (true, Some(monitor), Some(key)) = (in_monitor, &mut current, &key) {
                    match key.as_str() {
                        "name" => monitor.name = text,
                        "layout_symbol" => monitor.symbol = text,
                        _ => {}
                    }
                }

                key = None;
                wants_value = false;
            }

            ':' => wants_value = true,

            ',' => {
                key = None;
                wants_value = false;
            }

            '{' | '[' => {
                if character == '[' && depth == 1 && key.as_deref() == Some("monitors") {
                    monitors_at = Some(depth + 1);
                    found_monitors = true;
                }

                if character == '{' && monitors_at == Some(depth) {
                    current = Some(Monitor::default());
                }

                depth += 1;

                key = None;
                wants_value = false;
            }

            '}' | ']' => {
                if character == '}' && monitors_at.is_some_and(|at| depth == at + 1) {
                    if let Some(monitor) = current.take() {
                        if !monitor.name.is_empty() {
                            monitors.push(monitor);
                        }
                    }
                }

                if character == ']' && monitors_at == Some(depth) {
                    monitors_at = None;
                }

                depth = depth.checked_sub(1)?;

                key = None;
                wants_value = false;
            }

            _ => {}
        }
    }

    found_monitors.then_some(monitors)
}

// the rest of a string whose opening quote was just read, escapes undone
fn read_string(characters: &mut std::str::Chars) -> Option<String> {
    let mut text = String::new();

    loop {
        match characters.next()? {
            '"' => return Some(text),

            '\\' => match characters.next()? {
                'n' => text.push('\n'),
                't' => text.push('\t'),
                'r' => text.push('\r'),
                'b' => text.push('\u{8}'),
                'f' => text.push('\u{c}'),
                'u' => {
                    let hex: String = characters.by_ref().take(4).collect();
                    let code = u32::from_str_radix(&hex, 16).ok()?;

                    // half of a surrogate pair can't stand alone, it only shows up in emoji titles
                    text.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                }
                other => text.push(other),
            },

            character => text.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // from `mmsg get all-monitors` on this machine
    const STATUS: &str = r#"{"monitors":[{"name":"DP-1","active":true,"is_hdr":false,"is_vrr":false,"x":0,"y":0,"width":2560,"height":1440,"scale":1.5,"transform":"normal","layout_index":1,"layout_symbol":"S","last_open_surface":"amane-launcher","tag_num":9,"hide_clients":0,"tags":[{"index":1,"is_active":true,"is_urgent":false,"layout":"T","client_count":2},{"index":2,"is_active":false,"is_urgent":false,"layout":"S","client_count":1}],"active_tags":[1],"active_client":{"id":3,"title":"OC | \"Amane\" shell {x}","appid":"foot"},"keymode":"default","keyboardlayout":"English (US)"}]}"#;

    #[test]
    fn reads_each_monitors_layout_symbol() {
        let monitors = parse(STATUS).expect("a monitors array");

        assert_eq!(
            monitors,
            vec![Monitor {
                name: String::from("DP-1"),
                symbol: String::from("S"),
            }]
        );
    }

    #[test]
    fn key_order_and_nested_objects_dont_matter() {
        let line = r#"{"monitors":[
            {"tags":[{"layout":"T","name":"not a monitor"}],"layout_symbol":"VS","name":"eDP-1"},
            {"name":"HDMI-A-1","active_client":{"name":"x","layout_symbol":"no"},"layout_symbol":"\u00f3"}
        ]}"#;

        let monitors = parse(line).expect("a monitors array");

        assert_eq!(monitors.len(), 2);
        assert_eq!(monitors[0].name, "eDP-1");
        assert_eq!(monitors[0].symbol, "VS");
        assert_eq!(monitors[1].name, "HDMI-A-1");
        assert_eq!(monitors[1].symbol, "\u{f3}");
    }

    #[test]
    fn anything_else_is_not_read() {
        assert!(parse("ok").is_none());
        assert!(parse(r#"{"other":[]}"#).is_none());
    }
}
