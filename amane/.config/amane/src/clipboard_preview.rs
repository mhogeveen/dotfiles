/*
 * previews for the clipboard panel: the full text of a text entry, or the picture
 * of an image entry; the list only has cliphist's first 100 characters, or a
 * label like [[ binary data 259 KiB png 1771x1408 ]]
 *
 * an entry is decoded with `cliphist decode` on a thread the first time it's
 * selected, never the whole history; pictures are written to
 * $XDG_RUNTIME_DIR/amane-shell/clipboard/<id>, which lives in memory and is gone
 * after logging out, and amane's image loader tells png from jpeg by the bytes
 */

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, PoisonError};
use std::thread;

use amane::Service;

// a huge paste is cut off here, the preview has no room for more anyway
const MAX_TEXT: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub enum Preview {
    Text(String),
    Image(PathBuf),
    Failed,
}

pub struct Previews {
    ready: HashMap<String, Preview>,
}

impl Service for Previews {
    fn new() -> Self {
        Self {
            ready: HashMap::new(),
        }
    }

    // changes when a decode finishes, nothing to poll
    fn listen() {}
}

impl Previews {
    // None while it's still being decoded, or before anyone asked
    pub fn get(&self, id: &str) -> Option<&Preview> {
        self.ready.get(id)
    }
}

/*
 * entries already being decoded or done, so a view asking on every redraw
 * starts only one decode; not part of the service, it changes nothing shown
 */
static ASKED: Mutex<Option<HashSet<String>>> = Mutex::new(None);

fn asked<T>(work: impl FnOnce(&mut HashSet<String>) -> T) -> T {
    let mut asked = ASKED.lock().unwrap_or_else(PoisonError::into_inner);

    work(asked.get_or_insert_with(HashSet::new))
}

/*
 * starts decoding an entry unless that already happened; safe to call from a
 * view, it never writes the service itself, the thread does once it's done
 */
pub fn request(id: &str, line: &str, image: bool) {
    let first = asked(|asked| asked.insert(String::from(id)));

    if !first {
        return;
    }

    let id = String::from(id);
    let line = String::from(line);

    thread::spawn(move || {
        let preview = decode(&id, &line, image).unwrap_or(Preview::Failed);

        // forgotten while decoding, like deleted: the result isn't wanted any more
        if !asked(|asked| asked.contains(&id)) {
            return;
        }

        Previews::write().ready.insert(id, preview);
    });
}

fn decode(id: &str, line: &str, image: bool) -> Option<Preview> {
    let mut decode = Command::new("cliphist")
        .arg("decode")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;

    decode.stdin.take()?.write_all(line.as_bytes()).ok()?;

    let decoded = decode.wait_with_output().ok()?;

    if !decoded.status.success() {
        return None;
    }

    if image {
        let path = folder().join(id);

        fs::create_dir_all(folder()).ok()?;
        fs::write(&path, &decoded.stdout).ok()?;

        return Some(Preview::Image(path));
    }

    let bytes = &decoded.stdout[..decoded.stdout.len().min(MAX_TEXT)];

    Some(Preview::Text(String::from_utf8_lossy(bytes).into_owned()))
}

// after deleting an entry
pub fn forget(id: &str) {
    asked(|asked| asked.remove(id));

    let removed = Previews::write().ready.remove(id);

    if let Some(Preview::Image(path)) = removed {
        let _ = fs::remove_file(path);
    }
}

// after wiping the history, and at startup for pictures left from an earlier run
pub fn forget_all() {
    asked(|asked| asked.clear());

    let had_any = !Previews::read().ready.is_empty();

    if had_any {
        Previews::write().ready.clear();
    }

    let _ = fs::remove_dir_all(folder());
}

fn folder() -> PathBuf {
    let runtime = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);

    runtime.join("amane-shell").join("clipboard")
}

/*
 * what cliphist's label says about an image: [[ binary data 259 KiB png 1771x1408 ]]
 * reads as "259 KiB · 1771×1408"; None for a text entry
 */
pub fn image_info(preview: &str) -> Option<String> {
    let inside = preview
        .trim()
        .strip_prefix("[[ binary data ")?
        .strip_suffix("]]")?
        .trim();

    let mut words = inside.split_whitespace();

    let size = words.next()?;
    let unit = words.next()?;

    // the format, like png, isn't shown
    let _format = words.next()?;

    let mut info = format!("{size} {unit}");

    if let Some(dimensions) = words.next() {
        info.push_str(&format!(" · {}", dimensions.replace('x', "×")));
    }

    Some(info)
}

#[cfg(test)]
mod tests {
    use super::image_info;

    #[test]
    fn image_labels_read_as_size_and_dimensions() {
        assert_eq!(
            image_info("[[ binary data 259 KiB png 1771x1408 ]]").as_deref(),
            Some("259 KiB · 1771×1408")
        );
        assert_eq!(
            image_info("[[ binary data 12 KiB jpeg ]]").as_deref(),
            Some("12 KiB")
        );
        assert_eq!(image_info("amane compile --restart"), None);
        assert_eq!(image_info("[[ not quite ]]"), None);
    }
}
