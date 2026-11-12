/*
 * the keybind sheet (SUPER+/): every mango bind, read from its config each
 * time it changed (see ../keybinds.rs), in sections over a few columns, and
 * the keys every panel shares; type to narrow it down, escape closes
 *
 * the sheet has a fixed size like every panel; the columns are filled as
 * evenly as the sections allow, and what doesn't fit says how much is left
 */

use std::fs;
use std::ops::Add;
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

use amane::{
    Align::{Center, Start},
    Column, Key, LayerWindow, Monitor, Rectangle, Row, Service,
    Size::Parent,
    Text, TextInput, Vertical, Widget, children,
};

use super::list;
use super::ui::{self, TEXT_NOTE};
use super::{KEYBINDS, panel};
use crate::keybinds::{self, Bind, Section};
use crate::theme::Theme;

const INPUT: &str = "keybinds-search";
const PLACEHOLDER: &str = "Search keybinds  ·  from mango's config.conf";
const NAMESPACE: &str = "amane-keybinds";

const WIDTH: f32 = 1500.0;
const COLUMNS: usize = 3;

// how many lines a column holds; the window's height follows from it
const LINES: usize = 28;
const LINE_HEIGHT: f32 = 22.0;

const COLUMN_GAP: f32 = 24.0;

// between a bind's keys and what it does
const KEY_GAP: f32 = 16.0;

/*
 * room added to the widest keys in a column: the arrows come from a wider
 * fallback font, and the text elides a little before its estimated width
 */
const KEY_SLACK: f32 = 2.0 * TEXT_NOTE;

// the line around the frame, inside the window's width
const BORDER: f32 = 1.0;

// the keys every panel knows, after mango's
const IN_PANELS: &str = "in panels";
const PANEL_KEYS: [(&str, &str); 2] = [
    ("?", "button keys, or help in a search"),
    ("Esc", "close the panel"),
];

pub struct Sheet {
    query: String,
}

impl Service for Sheet {
    fn new() -> Self {
        Self {
            query: String::new(),
        }
    }

    // only changes through the search box
    fn listen() {}
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let base = LayerWindow::new()
        .width(WIDTH)
        .height(list::height(0, true) + LINES as f32 * LINE_HEIGHT)
        .anchor_vertical(Vertical::Middle)
        .namespace(NAMESPACE);

    panel(monitor, KEYBINDS, base, key_pressed, content)
}

// the search box takes every typed key, and there's nothing to pick
fn key_pressed(_: Key) {}

pub fn reset() {
    TextInput::set_text(INPUT, "");

    Sheet::write().query.clear();
}

fn changed(text: String) {
    Sheet::write().query = text;
}

fn content(t: &Theme) -> Rectangle {
    let query = Sheet::read().query.clone();

    let input = super::focused(TextInput::new(INPUT).placeholder(PLACEHOLDER)).on_change(changed);

    let sections = match sections() {
        Ok(sections) => sections,
        Err(error) => {
            return list::frame(t, Some(input), list::rows(vec![list::message(t, &error)]));
        }
    };

    let found = filtered(&sections, &query);

    if found.is_empty() {
        return list::frame(
            t,
            Some(input),
            list::rows(vec![list::message(t, "No keybinds match")]),
        );
    }

    let (columns, hidden) = layout(&found, LINES, COLUMNS);

    let inner = WIDTH - 2.0 * (ui::PADDING + BORDER);
    let column_width = (inner - (COLUMNS - 1) as f32 * COLUMN_GAP) / COLUMNS as f32;

    let last = columns.len().saturating_sub(1);

    let columns: Vec<Box<dyn Widget>> = columns
        .iter()
        .enumerate()
        .map(|(index, lines)| {
            let more = if index == last { hidden } else { 0 };

            Box::new(column(t, lines, column_width, more)) as Box<dyn Widget>
        })
        .collect();

    let rows = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .clip()
        .child(Row::new(columns).gap(COLUMN_GAP));

    list::frame(t, Some(input), rows)
}

/*
 * mango's sections and the panels' own keys; the config is read again only
 * when it changed, since the view runs on every typed letter
 */
static CACHE: Mutex<Option<(SystemTime, Vec<Section>)>> = Mutex::new(None);

fn sections() -> Result<Vec<Section>, String> {
    let path = keybinds::config_path();

    let unreadable = |error: std::io::Error| format!("Couldn't read {}: {error}", path.display());

    let modified = fs::metadata(&path)
        .and_then(|metadata| metadata.modified())
        .map_err(unreadable)?;

    let mut cache = CACHE.lock().unwrap_or_else(PoisonError::into_inner);

    let fresh = matches!(&*cache, Some((read_at, _)) if *read_at == modified);

    if !fresh {
        let config = fs::read_to_string(&path).map_err(unreadable)?;

        *cache = Some((modified, keybinds::parse(&config)));
    }

    let mut sections = cache
        .as_ref()
        .map(|(_, sections)| sections.clone())
        .unwrap_or_default();

    sections.push(Section {
        title: String::from(IN_PANELS),
        binds: PANEL_KEYS
            .iter()
            .map(|(keys, action)| Bind {
                keys: String::from(*keys),
                action: String::from(*action),
            })
            .collect(),
    });

    Ok(sections)
}

// the binds that match, in their sections; sections with none left go
fn filtered(sections: &[Section], query: &str) -> Vec<Section> {
    sections
        .iter()
        .map(|section| Section {
            title: section.title.clone(),
            binds: section
                .binds
                .iter()
                .filter(|bind| keybinds::matches(query, &section.title, bind))
                .cloned()
                .collect(),
        })
        .filter(|section| !section.binds.is_empty())
        .collect()
}

#[derive(Debug, PartialEq)]
enum Line<'a> {
    Heading(&'a str),
    Bind(&'a Bind),

    // between two sections in one column
    Gap,
}

/*
 * spreads the sections over the columns as evenly as they go: the shortest
 * column height that holds everything, up to lines; with more than fits, the
 * last line is kept for saying how many binds are left out
 */
fn layout(sections: &[Section], lines: usize, columns: usize) -> (Vec<Vec<Line<'_>>>, usize) {
    let total: usize = sections.iter().map(|section| section.binds.len() + 2).sum();

    let shortest = total.div_ceil(columns.max(1)).min(lines);

    for height in shortest..=lines {
        let (placed, hidden) = place(sections, height, columns);

        if hidden == 0 {
            return (placed, 0);
        }
    }

    place(sections, lines.saturating_sub(1), columns)
}

/*
 * fills columns of this many lines in order: a section that doesn't fit what's
 * left of a column starts the next one, and one longer than a whole column
 * carries on in the next under its heading again. returns the binds left over
 */
fn place(sections: &[Section], height: usize, columns: usize) -> (Vec<Vec<Line<'_>>>, usize) {
    let mut placed: Vec<Vec<Line>> = vec![Vec::new()];
    let mut hidden = 0;

    for section in sections {
        let mut binds = section.binds.iter().peekable();

        while binds.peek().is_some() {
            let used = placed.last().map_or(0, Vec::len);
            let gap = usize::from(used > 0);
            let room = height.saturating_sub(used + gap + 1);

            let whole_fits = room >= binds.len();
            let fits_alone = section.binds.len() + 1 <= height;

            // a heading needs at least one bind under it, and a short section isn't split
            if room == 0 || (!whole_fits && fits_alone && used > 0) {
                if placed.len() == columns {
                    hidden += binds.len();

                    break;
                }

                placed.push(Vec::new());

                continue;
            }

            let Some(column) = placed.last_mut() else {
                break;
            };

            if gap > 0 {
                column.push(Line::Gap);
            }

            column.push(Line::Heading(&section.title));

            for bind in binds.by_ref().take(room) {
                column.push(Line::Bind(bind));
            }
        }
    }

    (placed, hidden)
}

fn column(t: &Theme, lines: &[Line], width: f32, hidden: usize) -> Rectangle {
    // the widest keys in this column, so what they do lines up
    let key_width = lines
        .iter()
        .filter_map(|line| match line {
            Line::Bind(bind) => Some(ui::text_width(&bind.keys, TEXT_NOTE)),
            _ => None,
        })
        .fold(0.0, f32::max)
        .add(KEY_SLACK)
        .min(width / 2.0);

    let mut rows: Vec<Box<dyn Widget>> = lines
        .iter()
        .map(|line| -> Box<dyn Widget> {
            match line {
                Line::Heading(title) => Box::new(ui::heading(t, title, LINE_HEIGHT)),
                Line::Bind(bind) => Box::new(bind_line(t, bind, key_width)),
                Line::Gap => Box::new(Rectangle::new().width(Parent).height(LINE_HEIGHT)),
            }
        })
        .collect();

    if hidden > 0 {
        rows.push(Box::new(ui::line(
            LINE_HEIGHT,
            Text::new(format!("… {hidden} more, search to find them"))
                .size(TEXT_NOTE)
                .color(t.comment),
        )));
    }

    Rectangle::new()
        .width(width)
        .height(Parent)
        .child(Column::new(rows))
}

fn bind_line(t: &Theme, bind: &Bind, key_width: f32) -> Row {
    let keys = Rectangle::new()
        .width(key_width)
        .height(LINE_HEIGHT)
        .align_child(Start, Center)
        .child(
            Text::new(&bind.keys)
                .size(TEXT_NOTE)
                .color(t.accent)
                .elide(),
        );

    let action = Rectangle::new()
        .width(Parent)
        .height(LINE_HEIGHT)
        .align_child(Start, Center)
        .child(
            Text::new(&bind.action)
                .size(TEXT_NOTE)
                .color(t.fg_dim)
                .elide(),
        );

    Row::new(children![keys, action])
        .width(Parent)
        .height(LINE_HEIGHT)
        .gap(KEY_GAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(title: &str, count: usize) -> Section {
        Section {
            title: String::from(title),
            binds: (0..count)
                .map(|index| Bind {
                    keys: index.to_string(),
                    action: String::new(),
                })
                .collect(),
        }
    }

    fn heights(columns: &[Vec<Line>]) -> Vec<usize> {
        columns.iter().map(Vec::len).collect()
    }

    #[test]
    fn short_sections_spread_evenly() {
        // three sections of 3 binds, 4 lines each, in three columns of up to 28
        let sections = [section("a", 3), section("b", 3), section("c", 3)];
        let (columns, hidden) = layout(&sections, 28, 3);

        assert_eq!(hidden, 0);
        assert_eq!(heights(&columns), vec![4, 4, 4]);
    }

    #[test]
    fn a_section_that_does_not_fit_moves_to_the_next_column() {
        let sections = [section("a", 3), section("b", 3)];
        let (columns, hidden) = place(&sections, 6, 3);

        assert_eq!(hidden, 0);
        assert_eq!(columns[1][0], Line::Heading("b"));
    }

    #[test]
    fn a_long_section_carries_on_under_its_heading() {
        let sections = [section("long", 7)];
        let (columns, hidden) = place(&sections, 5, 3);

        assert_eq!(hidden, 0);
        assert_eq!(heights(&columns), vec![5, 4]);
        assert_eq!(columns[1][0], Line::Heading("long"));
    }

    #[test]
    fn what_does_not_fit_is_counted_and_leaves_room_to_say_so() {
        let sections = [section("a", 10), section("b", 10)];
        let (columns, hidden) = layout(&sections, 6, 2);

        // columns of 5 lines: a heading and 4 binds each, the rest left out
        assert_eq!(heights(&columns), vec![5, 5]);
        assert_eq!(hidden, 12);
    }
}
