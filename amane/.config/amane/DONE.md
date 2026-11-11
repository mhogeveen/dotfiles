# amane shell: done

Recommendations that were built, moved here from RECOMMENDATIONS.md. Numbers
are kept, so references between the two files still work.

### 1. Idle lock, screen off, suspend
- **Status:** done outside the shell: hypridle as a home-manager service in
  `~/.nixos/home/hypridle.nix` (a systemd user service on
  `graphical-session.target`, which mango's session target brings up). Lock
  after 5 min, screen off after 5.5, suspend after 30.
- **How it fits:** hypridle only calls `loginctl lock-session`; the shell's
  `session::watch_logind` puts up amane's lock screen on logind's `Lock` (and
  on `PrepareForSleep`), so there's no `lock_cmd`. The screen goes off with
  `wlopm` (mango has wlr-output-power-management), not `mmsg dispatch
  disable_monitor`, which removes the output and reshuffles layer surfaces.
- **Limits:** mango has no lock-notify protocol, so hypridle can't hold sleep
  until the lock screen is up (`inhibit_sleep = 3`); `before_sleep_cmd` waits
  a second after locking instead, like the session menu's
  `LOCK_BEFORE_SUSPEND`. No dimming: the old Hyprland setup used ddcutil,
  which needs I2C (`hardware.i2c.enable`) and isn't set up. Idle locking
  needs `XDG_SESSION_ID` in the shell's environment (mango's `exec_once`
  passes it); without it, `lock_cmd = "amane ipc call lock"` would be the
  fallback.

### 5. Workspaces per monitor
- **Status:** done: each bar shows only its monitor's workspaces; the one shown
  on a monitor is wide (accent on the focused monitor, dimmed on others), urgent
  ones are accent dots. No scrolling (decided against).
- **Why:** every bar shows the workspaces of all monitors; with a second
  screen each bar would show both. Urgent workspaces aren't marked.
- **Hooks in:** `workspaces()` in `src/bar.rs`: filter `Workspaces::read()` on
  `workspace.output() == monitor.name`; give urgent ones the accent; an
  `on_scroll` on the dots to move between workspaces.
- **To settle:** what scrolling should do (next/previous workspace, or
  nothing).

### 6. Keys in the corner panels
- **Status:** done: network `w` Wi-Fi, `r` rescan; Bluetooth `b` power, `r`
  rescan; audio `m` mute, `n` mic mute, `j`/`k` volume -/+5%; notifications
  `c` clear all. `?` shows them in the buttons, the switches and (for `j k`) the
  volume percentage. Helpers: `ui::key_text_button`, `ui::switch` with a key.
- **Why:** network, Bluetooth, audio and the notification center have no keys,
  so `?` has nothing to show there.
- **Hooks in:** each panel's `view` passes a key handler to `panel()` and uses
  `ui::key_button` (`src/panels/ui.rs`) for the buttons with a key. Ideas:
  network `w` Wi-Fi on/off, `r` rescan; Bluetooth `b` power, `r` rescan;
  audio `m` mute, `j`/`k` volume; notifications `c` clear all.
- **To settle:** the exact letters. The on/off switches aren't square, so
  they'd need a key variant of `ui::switch`.

### 7. Launcher
- **Status:** done: frecency ranking (score +1 per launch, halves every 7 days,
  kept in `~/.local/state/amane-shell/launches`, `src/usage.rs`); empty search
  shows used apps first; `=sum` calculator (`src/calculator.rs`), Enter copies
  the answer; `>cmd` runs in the background, `>>cmd` in foot, then your shell.
  `?` in an empty search box shows a help screen in every search panel
  (launcher, clipboard, layout); `Picker::help()` in `src/panels/picker.rs`.
- **Why:** results are ranked by match quality only.
- **Hooks in:** `src/panels/launcher.rs` (a `Picker`): remember launch counts
  in a state file like the accent (`~/.local/state/amane-shell/`) and use them
  as a tie-breaker; optionally a `=` prefix for a calculator or `>` to run a
  command.
- **To settle:** which of the three.

### 8. Clipboard
- **Status:** done: the × on the selected row removes an entry (`cliphist
  delete`); no delete key, as amane reports Delete like Shift or Ctrl (see the
  upstream list); a "Clear history" button under the list, clicked
  twice, wipes it all; the selected entry shows in full beside the list (text,
  or the picture decoded to `$XDG_RUNTIME_DIR/amane-shell/clipboard`,
  `src/clipboard_preview.rs`). Picker hooks for this: `moved`, `footer`,
  `preview`.
- **Why:** entries can't be deleted, and the history can't be cleared.
- **Hooks in:** `src/panels/clipboard.rs` (a `Picker`): `cliphist delete` for
  the selected entry and `cliphist wipe` for all. The search box takes letter
  keys, so these need buttons or non-letter keys (amane has no modifiers or
  Delete key). Optionally a preview of image entries via `cliphist decode` to a
  temporary file.
- **To settle:** how to trigger delete and clear without modifier keys.

### 9. Weather in the calendar
- **Status:** done: beside the month in the calendar panel, from Open-Meteo
  (csv, no key; `src/weather/`): now, 7 three-hour slots for the next 24
  hours (the first labelled Now) and 7 day slots, under "Next 24 hours" and
  "Next 7 days". Fetched when the panel opens, at most every 30 min; `r`
  refetches. Place set with `amane ipc call weather location <lat> <lon>
  <name>`, kept in `~/.local/state/amane-shell/weather` (Hilversum by
  default).
- **Why:** asked for when the calendar was built; the panel was laid out as a
  list of sections for this.
- **Hooks in:** a new section after `calendar_section` in
  `src/panels/calendar.rs`, a weather service fetched only when the panel opens
  and kept for a while, and a `HEIGHT` increase.
- **To settle:** source (wttr.in or Open-Meteo, both need no API key), the
  location, and what to show (now only, or a few days).
- **Update:** the calendar and weather are now two cards in a column, on one
  shared grid.

### 11. Wallpaper
- **Status:** done (`src/wallpaper.rs`, `src/panels/wallpaper.rs`): a window
  per monitor on the background layer draws the picture (decoded at most at
  twice the monitor's size), or stays see-through for None so mango's
  `root_color` shows. The picker is a full-width strip along the bottom,
  opened from the system panel (`w`) or `amane ipc call panel toggle
  wallpaper`: None, then the PNG/JPEG files in `~/Pictures/Wallpapers` (told
  apart by their first bytes, read again each time it opens or on `r`).
  h/l or the arrows preview on the real background, Enter or a click keeps
  it, Escape or clicking outside puts back the one in use. The choice is in
  the state file `wallpaper` (a path or `none`). The lock screen shows the
  wallpaper blurred and darkened, plain `bg_deep` with None.
- **Limits:** the preview only shows where no window covers the background
  (decided against drawing it over windows). WebP isn't supported by amane's
  decoder. The wallpaper doesn't follow the accent (decided against).

### 13. VPN and Tailscale
- **Status:** done in the shell (`src/vpn/`, `src/json.rs`): a small shield badge
  on the network icon's corner while a vpn is up (accent when all traffic goes
  through it, found with `ip route get`); a Tailscale card in the network panel
  (Tailscale@name, its IP, Connect/Disconnect on `v`, Log in on `i`) and
  lines for other tunnels (WireGuard, tun); the shield in the system panel's
  network row. Exit nodes: with hundreds on the tailnet (Mullvad), the card
  shows only None, what `tailscale exit-node suggest` suggests (asked when the
  panel opens and after each change), and the one in use if that's another
  (named via `tailscale whois`, asked once per change). Status is read with
  `--peers=false` (3 KB instead of 850 KB). Checks every 5 s (2 s with the
  network or system panel open); `ip` and `tailscale` only run while a tunnel
  exists.
- **Your part:** `services.tailscale.enable = true;` in
  `~/.nixos/hosts/prometheus/default.nix`, then once `sudo tailscale up` and
  `sudo tailscale set --operator=maarten`.
- **Later, maybe:** `src/layout.rs` has its own small reader for mango's json;
  it could use `src/json.rs` now.

### 16. `src/layout.rs` on `src/json.rs`
- **Status:** done: `parse` in `src/layout.rs` reads mango's `watch
  all-monitors` line with `json::parse` and takes each monitor's `name` and
  `layout_symbol`; the scanner (about 100 lines) is gone, its tests kept, plus
  one for an empty monitors array. A line that isn't valid json is skipped,
  where the scanner read what it could.
- **Why:** `src/layout.rs` has its own small reader for mango's json; the
  shared reader in `src/json.rs` came later (noted under item 13 in DONE.md).
- **Hooks in:** `src/layout.rs` only, with its tests kept.
- **To settle:** nothing, it's a cleanup.

### 14. Notification history across restarts
- **Status:** done (`src/notifications/history.rs`): the notifications in the
  center are saved to `$XDG_RUNTIME_DIR/amane-shell/notifications.json` (the
  newest 50, written within a second of a change, through a temporary file),
  so they survive shell restarts but not logout or reboot. A restarted shell
  shows them under "Earlier" below the live ones: read-only, since their
  senders can't be reached (amane's ids start over), so no buttons and
  clicking does nothing; × and Clear all remove them, along with pictures
  amane saved for them. They never pop up again and count as read. amane's
  list can't be seeded (crate-private fields), so `card::Content` takes a live
  or a restored one; `json::quote` writes the file.
- **Why:** every shell restart (`amane compile --restart`, `amane restart`)
  empties the notification center.
- **Hooks in:** keep the history in the state folder like the accent
  (`src/state.rs`) and show it again after a restart (`src/notifications/`).
- **To settle:** whether amane's `Notifications` service can be seeded, or the
  shell keeps its own copy; how many to keep; whether a reboot drops them.

### 2. Volume and media keys, with an on-screen popup
- **Status:** done (`src/osd.rs`, mango config): `SUPER+=` / `SUPER+-` change
  the volume 5% (capped at 100%), `SUPER+0` mutes, `SUPER+SHIFT+0` mutes the
  mic, all through `wpctl` and `bindl` (also while locked); the same
  `XF86Audio*` binds are there for a keyboard that sends media keys (the
  NuPhy Halo65 doesn't, its Fn layer could be remapped). A popup at the
  bottom center of the focused monitor shows the speaker or microphone, a
  bar and the percentage (dimmed, "Muted" when muted) for 1.5 s after any
  change, whatever made it; not at startup and not while the audio panel is
  open. It watches amane's `Audio` 10 times a second, in an always-mapped,
  click-through window that only draws while showing. Playback keys:
  `SUPER+\` play/pause, `SUPER+]` next, `SUPER+[` previous, through
  `amane ipc call media play-pause|next|previous` (amane's `Media`).
- **Why:** `mango/.config/mango/config.conf` binds no `XF86Audio*` keys, so the
  volume keys do nothing.
- **Hooks in:** new IPC handlers in `src/main.rs`, e.g. `amane ipc call volume
  up|down|mute`, using amane's `Audio` service (as `src/panels/audio.rs` does);
  mango binds for the keys; a small popup near the top that shows the volume
  for a moment after a change. The popup would be an always-mapped,
  click-through window like `src/notifications/popups.rs`, since showing and
  hiding windows quickly risks the crash.
- **To settle:** step size (5%?), whether the popup also shows for mic mute,
  and where it sits.

### 3. Media controls
- **Status:** done: a "Now playing" card at the top of the audio panel (which
  grew to fit, still a fixed size) for amane's active player: cover art,
  title, artist · player, a progress bar with times (none for streams without
  a length) and previous / play-pause / next, also on `h` `p` `l`. "Nothing
  playing" in the same space without a player. Covers: `file://` paths as
  they are, links fetched once with curl into
  `$XDG_RUNTIME_DIR/amane-shell/covers` (`src/covers.rs`). The playback keys
  were bound with item 2. Left out on purpose: a bar indicator, the title in
  the bar, the lock screen, and picking between several players.
- **Why:** amane has `Media`/`MediaPlayer` services (MPRIS: qutebrowser,
  Spotify, mpv and so on) that nothing in the shell uses yet.
- **Hooks in:** play/pause/next and the track title in `src/panels/audio.rs`;
  optionally a small bar indicator in `src/bar.rs` while something plays, and
  the track on the lock screen (`src/lock.rs`).
- **To settle:** panel only, or also bar and lock screen; what to show when
  several players are open.

### 19. Brightness popup and shortcuts
- **Status:** done: `SUPER+SHIFT+=` / `SUPER+SHIFT+-` (and
  `XF86MonBrightnessUp/Down`) run `amane ipc call brightness up|down`, 5% on
  the focused monitor through `brightness::set` (a laptop backlight through
  amane's `Brightness`); the first press after a restart finds and reads the
  monitor first, about a second. The volume popup shows brightness too (a sun
  that brightens with the level), for changes made through the shell only:
  the shortcuts and the slider, not the monitor's own buttons (no DDC
  polling). Not while the system panel is open.
- **Why:** brightness is only in the system panel (a slider over DDC/CI,
  `src/brightness.rs`); volume has shortcuts and a popup (item 2 in DONE.md).
- **Hooks in:** a brightness version of the volume popup in `src/osd.rs`,
  shown when it changes; `amane ipc call brightness up|down` going through
  `brightness::set` (newest value only, one ddcutil call at a time), with mango
  binds like `SUPER+SHIFT+=` / `SUPER+SHIFT+-` and `XF86MonBrightnessUp/Down`
  for keyboards that send them.
- **To settle:** the step size, and whether the popup should also show for
  changes made with the monitor's own buttons, which can only be noticed by
  polling DDC (about 0.4 s per read).

### 20. Audit findings (2026-11-11)
- **Status:** done in four steps (see HISTORY.md, session "audit"):
  - the most important bugs: Tailscale `up` hanging on a needed login,
    time limits for the external calls that read output
    (`src/command.rs`), weather and cover retries, popups held after a
    hover, the wallpaper picker applying None, an empty lock-screen password
  - round A, smaller bugs: `%20` icon paths, a bare `<` in bodies, atomic
    state saves, lost launch counts, zombies (`command::feed`), overflowing
    text, history save logging, reused PipeWire ids, `date` replaced by
    `clock::local_now` (libc `localtime_r`), nested service reads
  - round B, UX gaps: pairing keys, keyboard highlight in the network and
    Bluetooth lists, mic volume keys, scrolling audio device lists, the
    network panel's reserved list height, "No Wi-Fi adapter", errors
    cleared on close, wording
  - round C + D, structure: `ui` helpers (`header`, `note`, `line`, `card`,
    `error_line`, `list_box`, `text_width`, a button base, text size
    constants), `list::highlight_list`, `panels::letter`,
    `state::runtime_path`, `bar::HEIGHT`, a `corner` flag in the panel
    registry, shared glyphs, `command::read`/`capture` for wpctl, ip, df,
    curl and grim, `panels/network/` split into `mod.rs` and
    `vpn_cards.rs`, long functions split, module comments, no `bt`/`net`
    aliases, `was_open` kept in the poll loops, empty `listen()` for
    input-only services, tests (74 at the start, 107 after)
- **Left on purpose:** see item 21.

### 21. Audit leftovers
- **Status:** done. Button widths now fit their label: `ui::fit(longest
  label)` is the label's width plus the same room on each side
  (`BUTTON_SIDES`, 12), so Forget, Dismiss and Disconnect look alike
  everywhere; a button whose label changes is sized for the longest (Mute
  for Unmute, Tailscale's actions for Disconnect), switches for their "off"
  text. Popup timing became `Toasts::shows_at` on plain values, with tests.
  `now_playing` (audio) and `device_row` (Bluetooth) were split. `slurp`
  keeps no time limit on purpose: it waits for a region to be dragged.
