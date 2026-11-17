/*
 * wallpaper picker, full width along the bottom: None, then the pictures in
 * the wallpapers folder (~/Pictures/Wallpapers unless set) as thumbnails. h and l or the arrow keys move along and
 * show the picture on the real background right away; enter or a click keeps
 * it, escape or clicking outside puts back the one in use. r reads the folder
 * again
 *
 * a second card above the strip shows the desktop designs (src/desktop.rs),
 * as small live copies on the dark page (not over the wallpaper). k or up moves
 * into that row and j or down back; h and l there show a design on the real
 * background, and enter or a click keeps it with the selected wallpaper. one
 * design is kept for every wallpaper, drawn straight on a picture as on None
 *
 * the strip shows as many as fit, kept around the selection, instead of a
 * ScrollArea, which only scrolls up and down
 *
 * the wallpaper itself is in src/wallpaper.rs
 */

use std::path::PathBuf;

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
use crate::desktop::{self, Design, Desktop, Moment, Page};
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

// one card: the header and a row of tiles
const CARD_HEIGHT: f32 = PADDING * 2.0 + BUTTON + HEADER_GAP + TILE_HEIGHT + NAME_GAP + NAME_LINE;

// the designs card over the pictures card, always both, so the window never resizes
const CARD_GAP: f32 = 8.0;
const HEIGHT: f32 = CARD_HEIGHT * 2.0 + CARD_GAP;

// a tile's border: thick for the selection, in the accent in the row the keys move
const BORDER: f32 = 1.0;
const BORDER_SELECTED: f32 = 2.0;

// nerd font material design image crossed out, for none
const ICON_NONE: &str = "\u{f082b}";

// the None tile's glyph, big as it stands in for a picture
const NONE_ICON: f32 = 28.0;

const KEY_PREVIOUS: char = 'h';
const KEY_NEXT: char = 'l';
const KEY_REREAD: char = 'r';
const KEY_UP: char = 'k';
const KEY_DOWN: char = 'j';

// the row the keys move in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Pictures,
    Designs,
}

/*
 * the strip's selection; None means the one in use, which is where it starts.
 * the designs row's selection is the design shown, kept in the Desktop service
 */
struct Strip {
    selected: Option<usize>,
    focus: Focus,
}

impl Service for Strip {
    fn new() -> Self {
        Self {
            selected: None,
            focus: Focus::Pictures,
        }
    }

    // only changes through input
    fn listen() {}
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
    {
        let mut strip = Strip::write();

        strip.selected = None;
        strip.focus = Focus::Pictures;
    }

    wallpaper::cancel();
    desktop::cancel();
    wallpaper::reread();
}

fn key_pressed(key: Key) {
    match key {
        Key::Left => move_along(-1),
        Key::Right => move_along(1),
        Key::Up => focus(Focus::Designs),
        Key::Down => focus(Focus::Pictures),
        Key::Enter => keep(),
        Key::Character(letter) => match letter.to_ascii_lowercase() {
            KEY_PREVIOUS => move_along(-1),
            KEY_NEXT => move_along(1),
            KEY_UP => focus(Focus::Designs),
            KEY_DOWN => focus(Focus::Pictures),
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

// h and l, in the row that has the keys
fn move_along(by: isize) {
    let focus = Strip::read().focus;

    match focus {
        Focus::Pictures => step(by),
        Focus::Designs => step_design(by),
    }
}

// the designs row is only there, and so only takes the keys, while None is selected
fn focus(row: Focus) {
    Strip::write().focus = row;
}

// moves along the designs and shows that one on the background, stopping at the ends
fn step_design(by: isize) {
    let shown = Desktop::read().shown();

    desktop::preview(Design::ALL[design_step(shown, by)]);
}

// the index of the design `by` along from this one, kept within the list
fn design_step(from: Design, by: isize) -> usize {
    let index = Design::ALL
        .iter()
        .position(|design| *design == from)
        .unwrap_or(0);

    index.saturating_add_signed(by).min(Design::ALL.len() - 1)
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

        // the wallpaper stays, the design shown is still kept
        None => {
            desktop::apply();

            let _ = Panels::close(WALLPAPER);
        }
    }
}

// keeps a wallpaper (a picture or None) and the design shown with it
fn pick(item: Option<PathBuf>) {
    desktop::apply();
    wallpaper::apply(item);

    let _ = Panels::close(WALLPAPER);
}

// a click on a design: that design, with the wallpaper selected below
fn pick_design(design: Design) {
    desktop::preview(design);

    keep();
}

fn content(t: &Theme, width: f32) -> Rectangle {
    wallpaper::read_if_stale();

    let items = items();
    let selected = selected(&items);
    let focus = Strip::read().focus;

    let folder = Wallpaper::read().folder().clone();

    let current = Wallpaper::read().current().map(PathBuf::from);

    let fits = fitting(width);
    let start = window_start(selected.unwrap_or(0), fits, items.len());

    let mut tiles: Vec<Box<dyn Widget>> = Vec::new();

    for (index, item) in items.iter().enumerate().skip(start).take(fits) {
        let in_use = *item == current;

        let mark = mark(selected == Some(index), focus == Focus::Pictures);

        tiles.push(Box::new(tile(t, item.clone(), mark, in_use)));
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
                .child(Text::new(note).size(ui::TEXT_NOTE).color(t.comment).elide()),
        ));
    }

    let strip = Rectangle::new()
        .width(Parent)
        .height(TILE_HEIGHT + NAME_GAP + NAME_LINE)
        .on_scroll(|scroll| step(list::scroll_delta(scroll)))
        .child(Row::new(tiles).gap(TILE_GAP));

    let pictures = Rectangle::new()
        .width(Parent)
        .height(CARD_HEIGHT)
        .child(ui::frame(
            t,
            HEADER_GAP,
            children![header(t, &folder), strip],
        ));

    let above = Rectangle::new()
        .width(Parent)
        .height(CARD_HEIGHT)
        .child(designs_card(t, width, focus == Focus::Designs));

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .child(Column::new(children![above, pictures]).gap(CARD_GAP))
}

// "Desktops", the keys into and out of the row, and a live copy of each design
fn designs_card(t: &Theme, width: f32, focused: bool) -> Rectangle {
    let (shown, current) = {
        let desktop = Desktop::read();

        (desktop.shown(), desktop.current())
    };

    let moment = desktop::moment();

    // as many as fit, kept around the one shown, like the pictures
    let fits = fitting(width);
    let selected = Design::ALL
        .iter()
        .position(|design| *design == shown)
        .unwrap_or(0);
    let start = window_start(selected, fits, Design::ALL.len());

    let tiles: Vec<Box<dyn Widget>> = Design::ALL
        .into_iter()
        .skip(start)
        .take(fits)
        .map(|design| -> Box<dyn Widget> {
            Box::new(design_tile(
                t,
                design,
                &moment,
                mark(design == shown, focused),
                design == current,
            ))
        })
        .collect();

    let title = Rectangle::new()
        .width(120.0)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            Text::new("Desktops")
                .size(ui::TEXT_TITLE)
                .weight(Weight::Bold)
                .color(t.fg),
        );

    let keys = if focused {
        format!("{KEY_DOWN} or ↓ back to the wallpapers")
    } else {
        format!("{KEY_UP} or ↑ to pick a desktop")
    };

    let hint = Rectangle::new()
        .width(Parent)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            Text::new(keys)
                .size(ui::TEXT_SMALL)
                .color(t.comment)
                .elide(),
        );

    let header = Row::new(children![title, hint])
        .gap(HEADER_GAP)
        .align(Center)
        .height(BUTTON);

    let row = Rectangle::new()
        .width(Parent)
        .height(TILE_HEIGHT + NAME_GAP + NAME_LINE)
        .on_scroll(|scroll| step_design(list::scroll_delta(scroll)))
        .child(Row::new(tiles).gap(TILE_GAP));

    ui::frame(t, HEADER_GAP, children![header, row])
}

// how many tiles fit between the panel's edges on a monitor this wide
fn fitting(width: f32) -> usize {
    let inside = width - EDGE as f32 * 2.0 - PADDING * 2.0;

    (((inside + TILE_GAP) / (TILE_WIDTH + TILE_GAP)) as usize).max(1)
}

/*
 * the first tile a row shows: the selection stays in the middle where it can,
 * and the ends stay put, so the row only slides once the selection passes the
 * middle
 */
fn window_start(selected: usize, fits: usize, count: usize) -> usize {
    selected
        .saturating_sub(fits / 2)
        .min(count.saturating_sub(fits))
}

// how a tile is marked: the selection, in the accent in the row the keys move
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Plain,
    Selected,
    SelectedElsewhere,
}

fn mark(selected: bool, focused: bool) -> Mark {
    match (selected, focused) {
        (false, _) => Mark::Plain,
        (true, true) => Mark::Selected,
        (true, false) => Mark::SelectedElsewhere,
    }
}

// the border for a mark: thickness and color
fn border(t: &Theme, mark: Mark) -> (f32, amane::Color) {
    match mark {
        Mark::Plain => (BORDER, t.bg3),
        Mark::Selected => (BORDER_SELECTED, t.accent),
        Mark::SelectedElsewhere => (BORDER_SELECTED, t.bg5),
    }
}

/*
 * a design drawn at tile size on the dark page, with its name under it;
 * clicking keeps it. the wallpaper isn't drawn behind it: the real background
 * previews that, on an empty tag
 */
fn design_tile(t: &Theme, design: Design, moment: &Moment, mark: Mark, in_use: bool) -> Column {
    let (border, border_color) = border(t, mark);

    let page = Page::new(TILE_WIDTH - border * 2.0, TILE_HEIGHT - border * 2.0);

    let thumbnail = Rectangle::new()
        .width(TILE_WIDTH)
        .height(TILE_HEIGHT)
        .fill(t.bg_deep)
        .border(border, border_color)
        .padding(border)
        .clip()
        .cursor(Pointer)
        .on_click(move |_| pick_design(design));

    // none draws nothing, so its tile shows the same crossed-out picture as the wallpapers' None
    let thumbnail = if design == Design::None {
        thumbnail
            .align_child(Center, Center)
            .child(Text::new(ICON_NONE).size(NONE_ICON).color(t.comment))
    } else {
        thumbnail.child(desktop::page(design, page, moment, t))
    };

    Column::new(children![
        thumbnail,
        caption(t, design.name(), mark, in_use)
    ])
    .width(TILE_WIDTH)
    .gap(NAME_GAP)
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
                .size(ui::TEXT_SMALL)
                .color(t.comment)
                .elide(),
        );

    let title = Rectangle::new()
        .width(120.0)
        .height(BUTTON)
        .align_child(Start, Center)
        .child(
            Text::new("Wallpapers")
                .size(ui::TEXT_TITLE)
                .weight(Weight::Bold)
                .color(t.fg),
        );

    Row::new(children![
        title,
        place,
        ui::key_button(t, ui::ICON_REFRESH, KEY_REREAD, |_| reset()),
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
fn tile(t: &Theme, item: Option<PathBuf>, mark: Mark, in_use: bool) -> Column {
    let (border, border_color) = border(t, mark);

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
            .child(Text::new(ICON_NONE).size(NONE_ICON).color(t.comment)),
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

    Column::new(children![thumbnail, caption(t, &name, mark, in_use)])
        .width(TILE_WIDTH)
        .gap(NAME_GAP)
}

// a tile's name under its left edge; the one in use says so after its name
fn caption(t: &Theme, name: &str, mark: Mark, in_use: bool) -> Rectangle {
    let color = if mark == Mark::Plain { t.fg_dim } else { t.fg };

    // under the picture's left edge, like a caption
    let label = Rectangle::new()
        .width(TILE_WIDTH)
        .height(NAME_LINE)
        .align_child(Start, Center);

    if in_use {
        // the name gives way, so the suffix always shows whole
        let name = shortened(name, LABEL_CHARS - IN_USE.chars().count());

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn designs_step_within_the_list() {
        let last = Design::ALL.len() - 1;

        assert_eq!(design_step(Design::ALL[0], -1), 0);
        assert_eq!(design_step(Design::ALL[0], 1), 1.min(last));
        assert_eq!(design_step(Design::ALL[last], 1), last);
        assert_eq!(
            design_step(Design::ALL[last], -(Design::ALL.len() as isize)),
            0
        );
    }

    #[test]
    fn rows_slide_with_the_selection() {
        // everything fits: never slides
        assert_eq!(window_start(9, 10, 10), 0);

        // 4 of 12 fit: stays put at the start, centers in the middle, stops at the end
        assert_eq!(window_start(0, 4, 12), 0);
        assert_eq!(window_start(1, 4, 12), 0);
        assert_eq!(window_start(6, 4, 12), 4);
        assert_eq!(window_start(11, 4, 12), 8);
    }

    #[test]
    fn tiles_fit_the_monitor() {
        // this monitor holds ten side by side
        assert_eq!(fitting(2560.0), 10);
        assert_eq!(fitting(100.0), 1);
    }

    #[test]
    fn marks_follow_the_row_with_the_keys() {
        assert_eq!(mark(false, true), Mark::Plain);
        assert_eq!(mark(false, false), Mark::Plain);
        assert_eq!(mark(true, true), Mark::Selected);
        assert_eq!(mark(true, false), Mark::SelectedElsewhere);
    }

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
