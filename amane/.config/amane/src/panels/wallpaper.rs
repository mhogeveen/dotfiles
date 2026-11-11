/*
 * wallpaper picker, full width along the bottom: None, then the pictures in
 * ~/Pictures/Wallpapers as thumbnails. h and l or the arrow keys move along and
 * show the picture on the real background right away; enter or a click keeps
 * it, escape or clicking outside puts back the one in use. r reads the folder
 * again
 *
 * the strip shows as many as fit, kept around the selection, instead of a
 * ScrollArea, which only scrolls up and down
 *
 * the wallpaper itself is in src/wallpaper.rs
 */

use std::path::PathBuf;
use std::time::Duration;

use amane::{
    Align::{Center, Start},
    Column,
    Cursor::Pointer,
    Full, Image, Key, LayerWindow, Margin, Monitor, Rectangle, Row, Service,
    Size::Parent,
    Text, Vertical, Weight, Widget, children,
};

use super::list;
use super::ui::{self, BUTTON, PADDING};
use super::{Panels, WALLPAPER, panel};
use crate::theme::Theme;
use crate::wallpaper::{self, Folder, Wallpaper};

// a thumbnail is 16:9 like the screen, with its name under it
const TILE_WIDTH: f32 = 240.0;
const TILE_HEIGHT: f32 = 135.0;
const NAME_GAP: f32 = 4.0;
const NAME_LINE: f32 = 16.0;
const TILE_GAP: f32 = 12.0;

/*
 * the name line; BlexMono is monospaced, so a character is 0.6 of the size wide
 * and the line holds a known number of them
 */
const NAME_SIZE: f32 = 12.0;
const CHAR_WIDTH: f32 = NAME_SIZE * 0.6;
const LABEL_CHARS: usize = (TILE_WIDTH / CHAR_WIDTH) as usize;
const IN_USE: &str = " · in use";

// thumbnails are decoded at twice their size, sharp on a scaled screen
const THUMBNAIL: (u32, u32) = (TILE_WIDTH as u32 * 2, TILE_HEIGHT as u32 * 2);

const HEADER_GAP: f32 = 8.0;
const EDGE: i32 = 8;

const HEIGHT: f32 = PADDING * 2.0 + BUTTON + HEADER_GAP + TILE_HEIGHT + NAME_GAP + NAME_LINE;

// nerd font material design glyphs: refresh, and an image crossed out for none
const ICON_REFRESH: &str = "\u{f0450}";
const ICON_NONE: &str = "\u{f082b}";

const KEY_PREVIOUS: char = 'h';
const KEY_NEXT: char = 'l';
const KEY_REREAD: char = 'r';

// the strip's selection; None means the one in use, which is where it starts
struct Strip {
    selected: Option<usize>,
}

impl Service for Strip {
    fn new() -> Self {
        Self { selected: None }
    }

    // only changes through input
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let width = monitor.width as f32;

    let base = LayerWindow::new()
        .width(Full)
        .height(HEIGHT)
        .anchor_vertical(Vertical::Bottom)
        .margin(Margin {
            top: 0,
            right: EDGE,
            bottom: EDGE,
            left: EDGE,
        })
        .namespace("amane-wallpaper-picker");

    panel(monitor, WALLPAPER, base, key_pressed, move |t| {
        content(t, width)
    })
}

// when the picker closes however it closes: back to the one in use, and a fresh read next time
pub fn reset() {
    Strip::write().selected = None;

    wallpaper::cancel();
    wallpaper::reread();
}

fn key_pressed(key: Key) {
    match key {
        Key::Left => step(-1),
        Key::Right => step(1),
        Key::Enter => keep(),
        Key::Character(letter) => match letter.to_ascii_lowercase() {
            KEY_PREVIOUS => step(-1),
            KEY_NEXT => step(1),
            /*
             * the list can change under the selection, so it starts over from the
             * one in use, the preview with it, or the two would disagree
             */
            KEY_REREAD => reset(),
            _ => {}
        },
        _ => {}
    }
}

/*
 * None first, then the pictures; Some(None) is the None tile, so an index past
 * the end gives None
 */
fn items() -> Vec<Option<PathBuf>> {
    let wallpaper = Wallpaper::read();

    let mut items = vec![None];

    if let Folder::Pictures(pictures) = wallpaper.folder() {
        items.extend(pictures.iter().cloned().map(Some));
    }

    items
}

/*
 * where the selection is: as moved, or else on the one in use. nothing is
 * selected while the one in use isn't in the list (moved or deleted, or the
 * folder still being read), so Enter can't swap it for the None tile unasked
 */
fn selected(items: &[Option<PathBuf>]) -> Option<usize> {
    if let Some(index) = Strip::read().selected {
        return Some(index.min(items.len().saturating_sub(1)));
    }

    let current = Wallpaper::read().current().map(PathBuf::from);

    items.iter().position(|item| *item == current)
}

// moves along and shows that one on the background; from no selection it starts at the first tile
fn step(by: isize) {
    let items = items();

    let index = selected(&items)
        .unwrap_or(0)
        .saturating_add_signed(by)
        .min(items.len() - 1);

    Strip::write().selected = Some(index);

    wallpaper::preview(items[index].clone());
}

// keeps the selected one and closes; with nothing selected, the one in use stays
fn keep() {
    let items = items();

    match selected(&items) {
        Some(index) => pick(items[index].clone()),
        None => {
            let _ = Panels::close(WALLPAPER);
        }
    }
}

fn pick(item: Option<PathBuf>) {
    wallpaper::apply(item);

    let _ = Panels::close(WALLPAPER);
}

fn content(t: &Theme, width: f32) -> Rectangle {
    wallpaper::read_if_stale();

    let items = items();
    let selected = selected(&items);

    let folder = Wallpaper::read().folder().clone();

    let current = Wallpaper::read().current().map(PathBuf::from);

    // how many tiles fit between the panel's edges
    let inside = width - EDGE as f32 * 2.0 - PADDING * 2.0;
    let fits = (((inside + TILE_GAP) / (TILE_WIDTH + TILE_GAP)) as usize).max(1);

    // the selection stays in the middle where it can, the ends stay put
    let start = selected
        .unwrap_or(0)
        .saturating_sub(fits / 2)
        .min(items.len().saturating_sub(fits));

    let mut tiles: Vec<Box<dyn Widget>> = Vec::new();

    for (index, item) in items.iter().enumerate().skip(start).take(fits) {
        let in_use = *item == current;

        tiles.push(Box::new(tile(
            t,
            item.clone(),
            selected == Some(index),
            in_use,
        )));
    }

    // with nothing but None to pick, the room beside it says why
    let note = match &folder {
        Folder::Reading => Some(format!("Reading {}…", wallpaper::folder_name())),
        Folder::Missing => Some(format!("{} doesn't exist", wallpaper::folder_name())),
        Folder::Pictures(pictures) if pictures.is_empty() => Some(format!(
            "No pictures yet: put PNG or JPEG files in {}",
            wallpaper::folder_name()
        )),
        Folder::Pictures(_) => None,
    };

    if let Some(note) = note {
        tiles.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(TILE_HEIGHT)
                .align_child(Center, Center)
                .child(Text::new(note).size(13.0).color(t.comment).elide()),
        ));
    }

    let strip = Rectangle::new()
        .width(Parent)
        .height(TILE_HEIGHT + NAME_GAP + NAME_LINE)
        .on_scroll(|scroll| step(list::scroll_delta(scroll)))
        .child(Row::new(tiles).gap(TILE_GAP));

    ui::frame(t, HEADER_GAP, children![header(t, &folder), strip])
}

// "Wallpapers", where they come from and how many, and the key that reads the folder again
fn header(t: &Theme, folder: &Folder) -> Row {
    let count = match folder {
        Folder::Pictures(pictures) => format!(" · {}", pictures.len()),
        _ => String::new(),
    };

    let place = Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            Text::new(format!("{}{count}", wallpaper::folder_name()))
                .size(12.0)
                .color(t.comment)
                .elide(),
        );

    let title = Rectangle::new()
        .width(120.0)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            Text::new("Wallpapers")
                .size(15.0)
                .weight(Weight::Bold)
                .color(t.fg),
        );

    Row::new(children![
        title,
        place,
        ui::key_button(t, ICON_REFRESH, KEY_REREAD, |_| reset()),
    ])
    .gap(HEADER_GAP)
    .align(Center)
    .height(BUTTON)
}

// a name cut to this many characters, the last one an ellipsis when it had to be cut
fn shortened(name: &str, chars: usize) -> String {
    if name.chars().count() <= chars {
        return String::from(name);
    }

    let mut short: String = name.chars().take(chars.saturating_sub(1)).collect();

    short.push('…');

    short
}

/*
 * a thumbnail and its name; the selected one has an accent border, the one in
 * use says so after its name. clicking keeps it
 */
fn tile(t: &Theme, item: Option<PathBuf>, selected: bool, in_use: bool) -> Column {
    let (border, border_color) = if selected {
        (2.0, t.accent)
    } else {
        (1.0, t.bg3)
    };

    let name = match &item {
        Some(picture) => wallpaper::name(picture),
        None => String::from("None"),
    };

    // the picture fills the tile once decoded, with a plain tile under it until then
    let picture = match &item {
        Some(picture) => Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(Image::cover(picture.clone()).thumbnail(THUMBNAIL.0, THUMBNAIL.1)),
        None => Rectangle::new()
            .width(Parent)
            .height(Parent)
            .align_child(Center, Center)
            .child(Text::new(ICON_NONE).size(28.0).color(t.comment)),
    };

    let fill = if item.is_some() { t.bg1 } else { t.bg_deep };

    let picked = item.clone();

    let thumbnail = Rectangle::new()
        .width(TILE_WIDTH)
        .height(TILE_HEIGHT)
        .fill(fill)
        .border(border, border_color)
        // the picture inside the border, not over it
        .padding(border)
        .clip()
        .cursor(Pointer)
        .on_click(move |_| pick(picked.clone()))
        .child(picture);

    let color = if selected { t.fg } else { t.fg_dim };

    // under the picture's left edge, like a caption
    let label = Rectangle::new()
        .width(TILE_WIDTH)
        .height(NAME_LINE)
        .align_child(Start, Center);

    let label = if in_use {
        // the name gives way, so the suffix always shows whole
        let name = shortened(&name, LABEL_CHARS - IN_USE.chars().count());

        let width = (name.chars().count() + IN_USE.chars().count()) as f32 * CHAR_WIDTH;

        label.child(
            Row::new(children![
                Text::new(name).size(NAME_SIZE).color(color),
                Text::new(IN_USE).size(NAME_SIZE).color(t.comment),
            ])
            .width(width)
            .align(Center),
        )
    } else {
        label.child(Text::new(name).size(NAME_SIZE).color(color).elide())
    };

    Column::new(children![thumbnail, label])
        .width(TILE_WIDTH)
        .gap(NAME_GAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_give_way_to_the_suffix() {
        assert_eq!(shortened("blossom", 24), "blossom");
        assert_eq!(shortened("abcdef", 6), "abcdef");
        assert_eq!(shortened("abcdefg", 6), "abcde…");
        assert_eq!(shortened("ab", 0), "…");

        // the name and suffix together never outgrow the line
        let long = "a".repeat(100);
        let short = shortened(&long, LABEL_CHARS - IN_USE.chars().count());

        assert_eq!(short.chars().count() + IN_USE.chars().count(), LABEL_CHARS);
    }
}
