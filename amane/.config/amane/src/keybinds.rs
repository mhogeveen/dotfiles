/*
 * mango's keybinds, read from its config for the keybind sheet: every bind and
 * mousebind line, under the first line of the comment block above it (cut at a
 * , : ; or (), with what it does in words
 *
 * binds in a row that differ only in their last word fold into one line, like
 * Super+H L K J "focus left/right/up/down" or Super+1–9 "workspace 1–9"; spawn
 * binds never fold, their commands are too different to read as one
 *
 * keys the keyboard here doesn't have (XF86 media keys) are left out
 */

use std::env;
use std::path::PathBuf;

// a line of the sheet: the keys to press and what they do
#[derive(Clone, Debug, PartialEq)]
pub struct Bind {
    pub keys: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub title: String,
    pub binds: Vec<Bind>,
}

// for binds that come before any comment
const UNTITLED: &str = "general";

// a heading is cut at the first of these, so "tag: move client…" reads "tag"
const HEADING_ENDS: [char; 4] = [',', ':', ';', '('];

// where mango reads its config; ~/.config/mango links to the dotfiles, so edits show at once
pub fn config_path() -> PathBuf {
    let config = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();

            home.join(".config")
        });

    config.join("mango").join("config.conf")
}

// one bind as read, before folding
struct Raw {
    modifiers: Vec<&'static str>,
    key: String,
    dispatcher: String,
    action: String,
}

pub fn parse(config: &str) -> Vec<Section> {
    let mut sections: Vec<(String, Vec<Raw>)> = Vec::new();

    // the heading the next bind starts a section under
    let mut pending: Option<String> = None;

    // whether the next bind goes into the last section
    let mut appending = false;

    let mut previous_was_comment = false;

    for line in config.lines() {
        let line = line.trim();

        if let Some(comment) = line.strip_prefix('#') {
            // only a block's first line names it
            if !previous_was_comment {
                pending = heading(comment);
                appending = false;
            }

            previous_was_comment = true;

            continue;
        }

        previous_was_comment = false;

        if line.is_empty() {
            continue;
        }

        let Some((name, value)) = line.split_once('=') else {
            continue;
        };

        let name = name.trim();

        if !is_bind(name) {
            // an option between binds ends their section
            pending = None;
            appending = false;

            continue;
        }

        let Some(raw) = read_bind(value) else {
            continue;
        };

        if !appending || sections.is_empty() {
            let title = pending.take().unwrap_or_else(|| String::from(UNTITLED));

            sections.push((title, Vec::new()));
            appending = true;
        }

        if let Some((_, binds)) = sections.last_mut() {
            binds.push(raw);
        }
    }

    sections
        .into_iter()
        .map(|(title, raws)| Section {
            title,
            binds: fold(&raws),
        })
        .filter(|section| !section.binds.is_empty())
        .collect()
}

// bind, bindl, bindr and the like, and mousebind; not axis or gesture binds
fn is_bind(name: &str) -> bool {
    name.starts_with("bind") || name == "mousebind"
}

/*
 * the first line of a comment block as a heading; None for a commented-out
 * setting like "bind=Alt,1,tagsilent,1", whose first word has an =
 */
fn heading(comment: &str) -> Option<String> {
    let comment = comment.trim();
    let first_word = comment.split_whitespace().next()?;

    if first_word.contains('=') {
        return None;
    }

    let cut = comment.find(HEADING_ENDS).unwrap_or(comment.len());
    let title = comment[..cut].trim();

    (!title.is_empty()).then(|| String::from(title))
}

// "SUPER+SHIFT,h,move_client,left"; None for keys this keyboard doesn't have
fn read_bind(value: &str) -> Option<Raw> {
    let mut parts = value.splitn(4, ',');

    let modifiers = parts.next()?.trim();
    let key = parts.next()?.trim();
    let dispatcher = parts.next()?.trim();

    // a spawned command keeps its own commas
    let arguments = parts.next().unwrap_or("").trim();

    if key.is_empty() || key.starts_with("XF86") {
        return None;
    }

    Some(Raw {
        modifiers: modifier_names(modifiers),
        key: key_name(key),
        dispatcher: String::from(dispatcher),
        action: describe(dispatcher, arguments),
    })
}

// in the same order whichever way the config writes them
const MODIFIER_ORDER: [&str; 4] = ["Super", "Ctrl", "Alt", "Shift"];

fn modifier_names(modifiers: &str) -> Vec<&'static str> {
    let named: Vec<&'static str> = modifiers
        .split('+')
        .filter_map(
            |modifier| match modifier.trim().to_ascii_lowercase().as_str() {
                "super" | "logo" | "mod4" => Some("Super"),
                "ctrl" | "control" => Some("Ctrl"),
                "alt" | "mod1" => Some("Alt"),
                "shift" => Some("Shift"),
                _ => None,
            },
        )
        .collect();

    MODIFIER_ORDER
        .into_iter()
        .filter(|modifier| named.contains(modifier))
        .collect()
}

fn key_name(key: &str) -> String {
    let name = match key {
        "equal" => "=",
        "minus" => "-",
        "plus" => "+",
        "backslash" => "\\",
        "slash" => "/",
        "bracketleft" => "[",
        "bracketright" => "]",
        "comma" => ",",
        "period" => ".",
        "semicolon" => ";",
        "apostrophe" => "'",
        "grave" => "`",
        "space" => "Space",
        "Return" => "Enter",
        "Escape" => "Esc",
        "Left" => "←",
        "Right" => "→",
        "Up" => "↑",
        "Down" => "↓",
        "btn_left" => "Left button",
        "btn_right" => "Right button",
        "btn_middle" => "Middle button",
        _ if key.chars().count() == 1 => return key.to_uppercase(),
        _ => key,
    };

    String::from(name)
}

const PANEL_TOGGLE: &str = "amane ipc call panel toggle ";
const MEDIA: &str = "amane ipc call media ";
const BRIGHTNESS: &str = "amane ipc call brightness ";

fn describe(dispatcher: &str, arguments: &str) -> String {
    if dispatcher == "spawn" || dispatcher == "spawn_shell" {
        return describe_command(arguments);
    }

    let arguments: Vec<&str> = arguments
        .split(',')
        .map(str::trim)
        .filter(|argument| !argument.is_empty())
        .collect();

    let first = arguments.first().copied().unwrap_or("");

    let fixed = match dispatcher {
        "reload_config" => "reload mango's config",
        "quit" => "quit mango",
        "killclient" => "close window",
        "viewtoleft" => "workspace left",
        "viewtoright" => "workspace right",
        "viewtoleft_have_client" => "workspace with windows left",
        "viewtoright_have_client" => "workspace with windows right",
        "tagtoleft" => "window to workspace left",
        "tagtoright" => "window to workspace right",
        "toggleglobal" => "show window on every workspace",
        "togglejump" => "jump to a window",
        "togglefloating" => "float window",
        "togglemaximizescreen" => "maximize window",
        "togglefullscreen" => "fullscreen",
        "togglefakefullscreen" => "fake fullscreen",
        "minimized" => "minimize window",
        "restore_minimized" => "restore minimized window",
        "toggleoverlay" => "keep window on top",
        "toggle_scratchpad" => "scratchpad",
        "switch_proportion_preset" => "cycle window width",
        _ => "",
    };

    if !fixed.is_empty() {
        return String::from(fixed);
    }

    match dispatcher {
        "focusdir" => format!("focus {first}"),
        "move_client" | "exchange_client" => format!("move window {first}"),
        "scroller_stack" => format!("stack window {first}"),
        "view" => format!("workspace {first}"),
        "tag" => format!("window to workspace {first}"),
        "tagsilent" => format!("send window to workspace {first}"),
        "set_proportion" => match first.parse::<f32>() {
            Ok(proportion) => format!("window width {}%", (proportion * 100.0).round()),
            Err(_) => format!("window width {first}"),
        },
        "resizewin" => format!("resize {}", resize_direction(&arguments)),
        "moveresize" if first == "curmove" => String::from("drag to move window"),
        "moveresize" if first == "curresize" => String::from("drag to resize window"),
        _ => {
            let name = dispatcher.replace('_', " ");

            if arguments.is_empty() {
                name
            } else {
                format!("{name} {}", arguments.join(" "))
            }
        }
    }
}

// resizewin's "-50,+0": which way the window grows or shrinks
fn resize_direction(arguments: &[&str]) -> &'static str {
    let amount = |index: usize| {
        arguments
            .get(index)
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0)
    };

    match (amount(0), amount(1)) {
        (x, _) if x < 0 => "narrower",
        (x, _) if x > 0 => "wider",
        (_, y) if y < 0 => "shorter",
        (_, y) if y > 0 => "taller",
        _ => "",
    }
}

// a spawned command: the shell's own ipc and wpctl in words, anything else as the program
fn describe_command(command: &str) -> String {
    let command = command.trim();

    if let Some(panel) = command.strip_prefix(PANEL_TOGGLE) {
        let named = match panel.trim() {
            "launcher" => "app launcher",
            "layout" => "layout picker",
            "screenshot" => "screenshot menu",
            "session" => "session menu (log out, …)",
            "keybinds" => "this keybind sheet",
            other => return format!("{other} panel"),
        };

        return String::from(named);
    }

    if let Some(media) = command.strip_prefix(MEDIA) {
        return match media.trim() {
            "play-pause" => String::from("play / pause"),
            "next" => String::from("next track"),
            "previous" => String::from("previous track"),
            other => format!("media {other}"),
        };
    }

    if let Some(direction) = command.strip_prefix(BRIGHTNESS) {
        return format!("brightness {}", direction.trim());
    }

    if command.starts_with("wpctl") {
        let microphone = command.contains("SOURCE");

        if command.contains("set-mute") {
            return String::from(if microphone {
                "mute microphone"
            } else {
                "mute"
            });
        }

        if command.contains("set-volume") {
            let direction = if command.ends_with('-') { "down" } else { "up" };
            let what = if microphone { "mic volume" } else { "volume" };

            return format!("{what} {direction}");
        }
    }

    let program = command
        .split_whitespace()
        .next()
        .and_then(|program| program.rsplit('/').next())
        .unwrap_or(command);

    format!("run {program}")
}

/*
 * folds runs of binds with the same modifiers and dispatcher whose actions
 * differ only in their last word; spawn binds stay one per line
 */
fn fold(raws: &[Raw]) -> Vec<Bind> {
    let mut binds = Vec::new();
    let mut start = 0;

    while start < raws.len() {
        let mut end = start + 1;

        while end < raws.len() && folds_with(&raws[start], &raws[end]) {
            end += 1;
        }

        binds.push(join(&raws[start..end]));

        start = end;
    }

    binds
}

fn folds_with(first: &Raw, other: &Raw) -> bool {
    let spawns = first.dispatcher.starts_with("spawn");

    !spawns
        && first.modifiers == other.modifiers
        && first.dispatcher == other.dispatcher
        && first.action != other.action
        && matches!(
            (split_last_word(&first.action), split_last_word(&other.action)),
            (Some((a, _)), Some((b, _))) if a == b
        )
}

// "focus left" into ("focus", "left"); None for one word
fn split_last_word(action: &str) -> Option<(&str, &str)> {
    action.rsplit_once(' ')
}

fn join(raws: &[Raw]) -> Bind {
    let first = &raws[0];

    if raws.len() == 1 {
        return Bind {
            keys: keys(&first.modifiers, &first.key),
            action: first.action.clone(),
        };
    }

    let (prefix, _) = split_last_word(&first.action).unwrap_or(("", ""));

    let words: Vec<&str> = raws
        .iter()
        .filter_map(|raw| split_last_word(&raw.action).map(|(_, word)| word))
        .collect();

    let pressed: Vec<&str> = raws.iter().map(|raw| raw.key.as_str()).collect();

    // keys 1 to 9 for workspaces 1 to 9 read as a range
    if let Some(range) = number_range(&words)
        && number_range(&pressed).as_deref() == Some(range.as_str())
    {
        return Bind {
            keys: keys(&first.modifiers, &range),
            action: format!("{prefix} {range}"),
        };
    }

    Bind {
        keys: keys(&first.modifiers, &pressed.join(" ")),
        action: format!("{prefix} {}", words.join("/")),
    }
}

// "1", "2", … "9" as "1–9", when they count up one at a time
fn number_range(words: &[&str]) -> Option<String> {
    let numbers: Vec<u32> = words
        .iter()
        .map(|word| word.parse().ok())
        .collect::<Option<_>>()?;

    let counts_up = numbers.windows(2).all(|pair| pair[1] == pair[0] + 1);

    match (numbers.first(), numbers.last()) {
        (Some(first), Some(last)) if counts_up && numbers.len() > 2 => {
            Some(format!("{first}–{last}"))
        }
        _ => None,
    }
}

fn keys(modifiers: &[&str], key: &str) -> String {
    if modifiers.is_empty() {
        return String::from(key);
    }

    format!("{}+{key}", modifiers.join("+"))
}

/*
 * whether every word of the search is somewhere in the bind's keys, its
 * action or its section's title, ignoring case
 */
pub fn matches(query: &str, title: &str, bind: &Bind) -> bool {
    let text = format!("{title} {} {}", bind.keys, bind.action).to_lowercase();

    query
        .to_lowercase()
        .split_whitespace()
        .all(|word| text.contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bind(keys: &str, action: &str) -> Bind {
        Bind {
            keys: String::from(keys),
            action: String::from(action),
        }
    }

    #[test]
    fn binds_go_under_the_first_line_of_the_comment_above() {
        let config = "\
# switch window focus
# more about it
bind=SUPER,h,focusdir,left

# resize window
bind=CTRL+ALT,h,resizewin,-50,+0
";
        let sections = parse(config);

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "switch window focus");
        assert_eq!(sections[0].binds, vec![bind("Super+H", "focus left")]);
        assert_eq!(
            sections[1].binds,
            vec![bind("Ctrl+Alt+H", "resize narrower")]
        );
    }

    #[test]
    fn headings_are_cut_and_commented_settings_are_not_headings() {
        assert_eq!(
            heading(" brightness, through the shell"),
            Some(String::from("brightness"))
        );
        assert_eq!(heading(" tag: move client"), Some(String::from("tag")));
        assert_eq!(heading(" bind=Alt,1,tagsilent,1"), None);
    }

    #[test]
    fn an_option_between_binds_ends_the_section() {
        let config = "\
# layouts
tag_rule=id:1,layout_name:scroller

bind=SUPER,q,killclient,
";
        let sections = parse(config);

        assert_eq!(sections[0].title, UNTITLED);
        assert_eq!(sections[0].binds, vec![bind("Super+Q", "close window")]);
    }

    #[test]
    fn missing_keys_are_left_out_and_empty_sections_dropped() {
        let config = "\
# media keys
bindl=NONE,XF86AudioMute,spawn,wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
";
        assert!(parse(config).is_empty());
    }

    #[test]
    fn directions_and_numbers_fold() {
        let config = "\
# move
bind=SUPER,h,focusdir,left
bind=SUPER,l,focusdir,right
bind=SUPER,1,view,1,0
bind=SUPER,2,view,2,0
bind=SUPER,3,view,3,0
bind=SUPER+SHIFT,1,tag,1,0
";
        let sections = parse(config);

        assert_eq!(
            sections[0].binds,
            vec![
                bind("Super+H L", "focus left/right"),
                bind("Super+1–3", "workspace 1–3"),
                bind("Super+Shift+1", "window to workspace 1"),
            ]
        );
    }

    #[test]
    fn spawns_never_fold() {
        let config = "\
bind=SUPER,b,spawn,qutebrowser
bind=SUPER,t,spawn,foot
";
        assert_eq!(
            parse(config)[0].binds,
            vec![
                bind("Super+B", "run qutebrowser"),
                bind("Super+T", "run foot")
            ]
        );
    }

    #[test]
    fn the_shell_and_wpctl_read_as_words() {
        assert_eq!(
            describe_command("amane ipc call panel toggle wallpaper"),
            "wallpaper panel"
        );
        assert_eq!(describe_command("amane ipc call media next"), "next track");
        assert_eq!(
            describe_command("wpctl set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ 5%+"),
            "volume up"
        );
        assert_eq!(
            describe_command("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"),
            "mute microphone"
        );
        assert_eq!(describe_command("/usr/bin/foot -e htop"), "run foot");
    }

    #[test]
    fn modifiers_and_keys_read_the_same_whichever_way_they_are_written() {
        assert_eq!(
            modifier_names("alt+super+ctrl"),
            vec!["Super", "Ctrl", "Alt"]
        );
        assert_eq!(modifier_names("NONE"), Vec::<&str>::new());
        assert_eq!(key_name("bracketleft"), "[");
        assert_eq!(key_name("I"), "I");
        assert_eq!(key_name("Left"), "←");
    }

    #[test]
    fn a_search_needs_every_word_somewhere() {
        let volume = bind("Super+=", "volume up");

        assert!(matches("vol super", "volume and media", &volume));
        assert!(matches("MEDIA", "volume and media", &volume));
        assert!(!matches("volume down", "volume and media", &volume));
        assert!(matches("", "anything", &volume));
    }
}
