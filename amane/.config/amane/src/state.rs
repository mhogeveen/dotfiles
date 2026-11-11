/*
 * small files the shell remembers things in between restarts, like the accent
 * and how often apps are launched: $XDG_STATE_HOME/amane-shell/<name>, or
 * ~/.local/state/amane-shell/<name>, outside the config repo so they never
 * dirty git
 */

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
pub fn save(name: &str, contents: &str) -> io::Result<()> {
    write_atomically(&path(name), contents)
}

/*
 * written beside the file and moved over it, so a shell stopped halfway leaves
 * the old file rather than an empty or cut-off one
 */
pub fn write_atomically(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }

    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".new");

    fs::write(&temporary, contents)?;
    fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_save_replaces_the_file_whole() {
        let folder = std::env::temp_dir().join("amane-shell-state-test");
        let file = folder.join("value");

        write_atomically(&file, "first").expect("written");
        write_atomically(&file, "second").expect("written again");

        assert_eq!(fs::read_to_string(&file).expect("read"), "second");
        assert!(!folder.join("value.new").exists());

        fs::remove_dir_all(&folder).expect("cleaned up");
    }
}
