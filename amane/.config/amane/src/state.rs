/*
 * small files the shell remembers things in between restarts, like the accent
 * and how often apps are launched: $XDG_STATE_HOME/amane-shell/<name>, or
 * ~/.local/state/amane-shell/<name>, outside the config repo so they never
 * dirty git
 */

use std::env;
use std::fs;
use std::path::PathBuf;

pub fn path(name: &str) -> PathBuf {
    let state = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| {
            let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();

            home.join(".local").join("state")
        });

    state.join("amane-shell").join(name)
}

// writes the file, making the folder first when it's missing
pub fn save(name: &str, contents: &str) -> std::io::Result<()> {
    let path = path(name);

    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }

    fs::write(path, contents)
}
