/*
 * mango's layout on each monitor, for the bar's [S] and the layout panel
 *
 * amane already follows mango for the workspaces, but doesn't pass the layout
 * on, so this opens its own `watch all-monitors` on mango's socket; mango sends
 * the whole state as one json line on every change, read with src/json.rs
 */

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

use amane::Service;

use crate::json::{self, Value};

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
 * only a monitor's own keys count, so the tags' "layout" and nested objects never
 * get mixed in; None when there's no monitors array, like mango's "ok"
 */
fn parse(line: &str) -> Option<Vec<Monitor>> {
    let status = json::parse(line)?;

    let Value::List(monitors) = status.get("monitors") else {
        return None;
    };

    Some(
        monitors
            .iter()
            .filter(|monitor| !monitor.get("name").text().is_empty())
            .map(|monitor| Monitor {
                name: String::from(monitor.get("name").text()),
                symbol: String::from(monitor.get("layout_symbol").text()),
            })
            .collect(),
    )
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

    #[test]
    fn no_monitors_is_an_empty_list() {
        assert_eq!(parse(r#"{"monitors":[]}"#), Some(Vec::new()));
    }
}
