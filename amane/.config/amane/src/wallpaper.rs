/*
 * the wallpaper: a picture from ~/Pictures/Wallpapers drawn on the background
 * layer of every monitor, or none, which leaves the background see-through so
 * mango's own root_color shows
 *
 * the picker (src/panels/wallpaper.rs) previews by changing what's shown and
 * keeps it with apply; closing it any other way puts back the one in use. the
 * choice is kept in a state file, like the accent
 */

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use amane::{
    Color, Full, Image, Keyboard, Layer, LayerWindow, Monitor, Rectangle, Service, Size::Parent,
    Zone,
};

use crate::state;

// the file in the state folder the choice is kept in: a path, or "none"
const STATE: &str = "wallpaper";
const NONE: &str = "none";

// what amane can decode, told apart by the first bytes like amane does
const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF];

// what's in the folder, once it has been read
#[derive(Debug, Clone, PartialEq)]
pub enum Folder {
    Reading,
    Missing,
    Pictures(Vec<PathBuf>),
}

pub struct Wallpaper {
    // the one picked; None for no picture
    current: Option<PathBuf>,

    // what the background draws now, the same as current unless the picker previews another
    shown: Option<PathBuf>,

    folder: Folder,

    // the folder is read again the next time the picker opens
    stale: bool,
}

impl Service for Wallpaper {
    // a file of one line, quick enough for the first view that asks
    fn new() -> Self {
        let current = fs::read_to_string(state::path(STATE))
            .ok()
            .and_then(|text| saved(&text));

        Self {
            shown: current.clone(),
            current,
            folder: Folder::Reading,
            stale: true,
        }
    }

    // read when the picker opens, nothing to poll
    fn listen() {}
}

// reading, safe inside views
impl Wallpaper {
    pub fn current(&self) -> Option<&Path> {
        self.current.as_deref()
    }

    pub fn folder(&self) -> &Folder {
        &self.folder
    }
}

// one read at a time; not part of the service, so starting one redraws nothing
static READING: AtomicBool = AtomicBool::new(false);

/*
 * reads the folder when the picker opened since the last read; safe to call from a
 * view, as it only starts a thread, which writes the pictures when it's done
 */
pub fn read_if_stale() {
    if !Wallpaper::read().stale || READING.swap(true, Ordering::AcqRel) {
        return;
    }

    thread::spawn(|| {
        let folder = match pictures(&folder()) {
            Some(pictures) => Folder::Pictures(pictures),
            None => Folder::Missing,
        };

        {
            let mut wallpaper = Wallpaper::write();

            wallpaper.folder = folder;
            wallpaper.stale = false;
        }

        READING.store(false, Ordering::Release);
    });
}

// the folder is read again on the next draw; from a key, or when the picker closes
pub fn reread() {
    Wallpaper::write().stale = true;
}

// shows this one without keeping it; from the picker's keys
pub fn preview(picture: Option<PathBuf>) {
    let mut wallpaper = Wallpaper::write();

    if wallpaper.shown != picture {
        wallpaper.shown = picture;
    }
}

// keeps this one, also after a restart
pub fn apply(picture: Option<PathBuf>) {
    let contents = match &picture {
        Some(path) => path.to_string_lossy().into_owned(),
        None => String::from(NONE),
    };

    {
        let mut wallpaper = Wallpaper::write();

        wallpaper.current = picture.clone();
        wallpaper.shown = picture;
    }

    if let Err(error) = state::save(STATE, &contents) {
        eprintln!("wallpaper: couldn't save the choice: {error}");
    }
}

// back to the one in use, after a preview that wasn't kept
pub fn cancel() {
    let mut wallpaper = Wallpaper::write();

    if wallpaper.shown != wallpaper.current {
        wallpaper.shown = wallpaper.current.clone();
    }
}

// ~/Pictures/Wallpapers
pub fn folder() -> PathBuf {
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();

    home.join("Pictures").join("Wallpapers")
}

// like ~/Pictures/Wallpapers, for showing
pub fn folder_name() -> String {
    let folder = folder();

    match env::var_os("HOME") {
        Some(home) => match folder.strip_prefix(home) {
            Ok(rest) => format!("~/{}", rest.display()),
            Err(_) => folder.display().to_string(),
        },
        None => folder.display().to_string(),
    }
}

// a picture's name without its folder and extension, like blossom
pub fn name(picture: &Path) -> String {
    picture
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/*
 * the background of every monitor: the picture shown, decoded at no more than
 * twice the monitor's size in widget units (enough for a scale of 2, and a big
 * file costs no more memory than that), or see-through for mango's root_color
 *
 * the window is always there and only its fill changes, so the picker's preview
 * doesn't show or hide a layer surface
 */
pub fn view(monitor: &Monitor) -> LayerWindow {
    let shown = Wallpaper::read().shown.clone();

    let background = Rectangle::new().width(Parent).height(Parent);

    let background = match shown {
        Some(picture) => {
            background.fill(Image::cover(picture).thumbnail(monitor.width * 2, monitor.height * 2))
        }
        None => background.fill(Color::TRANSPARENT),
    };

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Background)
        .space(Zone::Ignore)
        .keyboard(Keyboard::None)
        .namespace("amane-wallpaper")
        .child(background)
        .click_through()
}

// a saved choice; a picture that's gone since counts as none
fn saved(text: &str) -> Option<PathBuf> {
    let text = text.trim();

    if text.is_empty() || text == NONE {
        return None;
    }

    let path = PathBuf::from(text);

    path.is_file().then_some(path)
}

// the pictures in a folder by name, None when the folder can't be read
fn pictures(folder: &Path) -> Option<Vec<PathBuf>> {
    let entries = fs::read_dir(folder).ok()?;

    let mut pictures: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_picture(path))
        .collect();

    pictures.sort_by_key(|path| name(path).to_lowercase());

    Some(pictures)
}

// a png or jpeg by its first bytes, whatever its name ends in; also for cover art
pub fn is_picture(path: &Path) -> bool {
    let mut head = [0u8; 8];

    let Ok(mut file) = File::open(path) else {
        return false;
    };

    let read = file.read(&mut head).unwrap_or(0);

    known_format(&head[..read])
}

fn known_format(head: &[u8]) -> bool {
    head.starts_with(PNG) || head.starts_with(JPEG)
}

#[cfg(test)]
mod tests {
    use super::*;

    // a fresh folder per test, removed again at the end
    fn scratch(name: &str) -> PathBuf {
        let folder = env::temp_dir().join(format!("amane-wallpaper-{name}-{}", std::process::id()));

        let _ = fs::remove_dir_all(&folder);

        fs::create_dir_all(&folder).unwrap();

        folder
    }

    #[test]
    fn formats_are_told_by_their_first_bytes() {
        assert!(known_format(PNG));
        assert!(known_format(&[0xFF, 0xD8, 0xFF, 0xE0]));
        assert!(!known_format(b"RIFF\0\0\0\0WEBP"));
        assert!(!known_format(b"GIF89a"));
        assert!(!known_format(&[]));
    }

    #[test]
    fn only_pictures_are_listed_by_name() {
        let folder = scratch("list");

        fs::write(folder.join("b-dunes.jpg"), [0xFF, 0xD8, 0xFF, 0xE0]).unwrap();
        fs::write(folder.join("A-forest.png"), PNG).unwrap();
        fs::write(folder.join("c-renamed.webp"), PNG).unwrap();
        fs::write(folder.join("notes.txt"), "not a picture").unwrap();
        fs::write(folder.join("fake.png"), "not a png either").unwrap();
        fs::create_dir(folder.join("sub")).unwrap();

        let names: Vec<String> = pictures(&folder)
            .unwrap()
            .iter()
            .map(|path| name(path))
            .collect();

        assert_eq!(names, ["A-forest", "b-dunes", "c-renamed"]);

        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_missing_folder_is_none() {
        let folder = scratch("missing");

        fs::remove_dir_all(&folder).unwrap();

        assert_eq!(pictures(&folder), None);
    }

    #[test]
    fn saved_choices() {
        let folder = scratch("saved");
        let picture = folder.join("kept.png");

        fs::write(&picture, PNG).unwrap();

        assert_eq!(saved(&format!("{}\n", picture.display())), Some(picture));
        assert_eq!(saved("none"), None);
        assert_eq!(saved(""), None);
        assert_eq!(saved(&folder.join("gone.png").display().to_string()), None);

        fs::remove_dir_all(&folder).unwrap();
    }
}
