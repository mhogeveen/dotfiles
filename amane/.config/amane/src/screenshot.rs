/*
 * taking screenshots: slurp picks a region, grim captures it, and the result goes
 * to the clipboard through wl-copy or to ~/Pictures/Screenshots as a png; a
 * notification says how it went
 *
 * everything here waits on other programs, slurp for as long as you take to drag
 * a region, so call take() from its own thread
 */

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use amane::{Urgency, output};

use crate::notifications;

const APP_NAME: &str = "Screenshots";

#[derive(Clone, Copy)]
pub enum Area {
    Fullscreen,
    Region,
}

#[derive(Clone, Copy)]
pub enum Target {
    Clipboard,
    File,
}

pub fn take(area: Area, target: Target) {
    match capture(area, target) {
        // slurp was cancelled with escape or a right click, nothing to say
        Ok(None) => {}

        Ok(Some(done)) => {
            notifications::send(
                APP_NAME,
                APP_NAME,
                &done.message,
                Urgency::Low,
                done.file.as_deref(),
            );
        }

        Err(error) => {
            notifications::send(
                APP_NAME,
                "Screenshot failed",
                &error,
                Urgency::Critical,
                None,
            );
        }
    }
}

struct Done {
    message: String,

    // the saved png, shown as the notification's picture
    file: Option<PathBuf>,
}

fn capture(area: Area, target: Target) -> Result<Option<Done>, String> {
    let mut grim = Command::new("grim");

    let area_name = match area {
        Area::Fullscreen => "Fullscreen",
        Area::Region => {
            let Some(geometry) = pick_region()? else {
                return Ok(None);
            };

            grim.arg("-g").arg(geometry);

            "Region"
        }
    };

    match target {
        Target::Clipboard => {
            let png = run(grim.arg("-"))?;

            copy(&png)?;

            Ok(Some(Done {
                message: format!("{area_name} copied to clipboard"),
                file: None,
            }))
        }

        Target::File => {
            let file = new_file()?;

            run(grim.arg(&file))?;

            Ok(Some(Done {
                message: format!("{area_name} saved to {}", file.display()),
                file: Some(file),
            }))
        }
    }
}

// None when the selection was cancelled
fn pick_region() -> Result<Option<String>, String> {
    let picked = Command::new("slurp")
        .arg("-d")
        .output()
        .map_err(|error| format!("couldn't start slurp: {error}"))?;

    if !picked.status.success() {
        return Ok(None);
    }

    let geometry = String::from_utf8_lossy(&picked.stdout).trim().to_string();

    Ok((!geometry.is_empty()).then_some(geometry))
}

// runs a capture and gives back what it printed, the png when writing to "-"
fn run(command: &mut Command) -> Result<Vec<u8>, String> {
    let name = command.get_program().to_string_lossy().into_owned();

    let result = command
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("couldn't start {name}: {error}"))?;

    if !result.status.success() {
        let error = String::from_utf8_lossy(&result.stderr);

        return Err(format!("{name}: {}", error.trim()));
    }

    Ok(result.stdout)
}

fn copy(png: &[u8]) -> Result<(), String> {
    let mut wl_copy = Command::new("wl-copy")
        .args(["--type", "image/png"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| format!("couldn't start wl-copy: {error}"))?;

    wl_copy
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(png)
        .map_err(|error| format!("wl-copy: {error}"))?;

    // wl-copy forks to keep serving the clipboard, so this only waits for the handover
    let status = wl_copy
        .wait()
        .map_err(|error| format!("wl-copy: {error}"))?;

    if !status.success() {
        return Err(String::from("wl-copy failed"));
    }

    Ok(())
}

// ~/Pictures/Screenshots/20261106171503.png, the folder made when missing
fn new_file() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|_| String::from("HOME is not set"))?;

    let folder = PathBuf::from(home).join("Pictures").join("Screenshots");

    fs::create_dir_all(&folder)
        .map_err(|error| format!("couldn't create {}: {error}", folder.display()))?;

    let stamp = output("date +%Y%m%d%H%M%S");

    Ok(folder.join(format!("{stamp}.png")))
}
