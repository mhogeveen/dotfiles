/*
 * cover art for the audio panel's "Now playing": players give a file:// path or
 * an https:// link. a path is used as it is; a link is fetched once with curl
 * into $XDG_RUNTIME_DIR/amane-shell/covers, named after the link, so the same
 * cover is never fetched twice (a failed one is tried again after RETRY).
 * logging out empties the folder
 *
 * a view asks with path(), which starts the fetch on a thread the first time;
 * the panel redraws once it's there
 */

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::wallpaper;

// a cover bigger than this isn't a cover
const MAX_SIZE: &str = "5M";

// curl's --max-time, in seconds
const TIMEOUT: &str = "10";

// a link that failed (offline, say) is fetched again after this, not never
const RETRY: Duration = Duration::from_secs(60);

// curl writes here first, so a fetch cut short by a crash isn't taken for a cover
const PARTIAL: &str = "part";

pub struct Covers {
    // fetched links and the picture each ended up as
    fetched: HashMap<String, PathBuf>,

    // links whose fetch failed, and when
    failed: HashMap<String, Instant>,
}

impl Service for Covers {
    fn new() -> Self {
        Self {
            fetched: HashMap::new(),
            failed: HashMap::new(),
        }
    }

    // changes when a fetch finishes, nothing to poll
    fn listen() {}
}

// links being fetched; not part of the service, so starting one redraws nothing
static FETCHING: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/*
 * the picture for a player's art url, None until it's there or when there is
 * none; safe to call from a view
 */
pub fn path(url: &str) -> Option<PathBuf> {
    if url.is_empty() {
        return None;
    }

    if let Some(path) = file_path(url) {
        return path.is_file().then_some(path);
    }

    if !url.starts_with("https://") && !url.starts_with("http://") {
        return None;
    }

    {
        let covers = Covers::read();

        if let Some(path) = covers.fetched.get(url) {
            return Some(path.clone());
        }

        if covers
            .failed
            .get(url)
            .is_some_and(|failed| failed.elapsed() < RETRY)
        {
            return None;
        }
    }

    fetch(url);

    None
}

// starts one fetch per link
fn fetch(url: &str) {
    {
        let mut fetching = FETCHING.lock().unwrap_or_else(PoisonError::into_inner);

        if !fetching
            .get_or_insert_with(HashSet::new)
            .insert(String::from(url))
        {
            return;
        }
    }

    let url = String::from(url);

    thread::spawn(move || {
        let done = download(&url);

        {
            let mut covers = Covers::write();

            match done {
                Some(path) => {
                    covers.failed.remove(&url);
                    covers.fetched.insert(url.clone(), path);
                }
                None => {
                    covers.failed.insert(url.clone(), Instant::now());
                }
            }
        }

        if let Some(fetching) = FETCHING
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_mut()
        {
            fetching.remove(&url);
        }
    });
}

fn download(url: &str) -> Option<PathBuf> {
    let folder = PathBuf::from(env::var_os("XDG_RUNTIME_DIR")?)
        .join("amane-shell")
        .join("covers");

    fs::create_dir_all(&folder).ok()?;

    let path = folder.join(file_name(url));

    if !path.is_file() {
        let partial = path.with_extension(PARTIAL);

        let fetched = Command::new("curl")
            .args(["--silent", "--fail", "--location"])
            .args(["--max-time", TIMEOUT, "--max-filesize", MAX_SIZE])
            .arg("--output")
            .arg(&partial)
            .arg(url)
            .status()
            .is_ok_and(|status| status.success());

        if !fetched || fs::rename(&partial, &path).is_err() {
            let _ = fs::remove_file(&partial);

            return None;
        }
    }

    // only what amane can draw
    if wallpaper::is_picture(&path) {
        Some(path)
    } else {
        let _ = fs::remove_file(&path);

        None
    }
}

// the same link always gives the same name
fn file_name(url: &str) -> String {
    let mut hasher = DefaultHasher::new();

    url.hash(&mut hasher);

    format!("{:016x}", hasher.finish())
}

// file:///home/me/My%20Music/cover.jpg is /home/me/My Music/cover.jpg; None for any other link
pub fn file_path(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix("file://")?;

    let bytes = rest.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        let escaped = (bytes[index] == b'%')
            .then(|| bytes.get(index + 1..index + 3))
            .flatten()
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());

        match escaped {
            Some(byte) => {
                decoded.push(byte);
                index += 3;
            }
            None => {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
    }

    let path = PathBuf::from(String::from_utf8(decoded).ok()?);

    path.is_absolute().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_links_are_decoded() {
        assert_eq!(
            file_path("file:///home/me/My%20Music/caf%C3%A9.jpg"),
            Some(PathBuf::from("/home/me/My Music/café.jpg"))
        );
        assert_eq!(
            file_path("file:///tmp/plain.png"),
            Some(PathBuf::from("/tmp/plain.png"))
        );

        // a stray % stays as it is
        assert_eq!(
            file_path("file:///tmp/100%.png"),
            Some(PathBuf::from("/tmp/100%.png"))
        );

        assert_eq!(file_path("https://example.org/cover.jpg"), None);
        assert_eq!(file_path("file://relative/cover.jpg"), None);
    }

    #[test]
    fn names_follow_the_link() {
        let first = file_name("https://example.org/a.jpg");

        assert_eq!(first, file_name("https://example.org/a.jpg"));
        assert_ne!(first, file_name("https://example.org/b.jpg"));
        assert_eq!(first.len(), 16);
    }
}
