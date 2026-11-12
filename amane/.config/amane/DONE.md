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

### 22. Animated workspace indicator, animations checkbox
- **Status:** done: switching workspace eases the indicator's width (dot ↔
  wide block) and fill over 200 ms; the dots and `[S]` beside it slide
  along, and focus moving between monitors or a new accent fades too. The
  system panel's Settings has an Animations checkbox (`a`), kept in the state
  file `animations`, that turns all shell motion off at once (see 5.11).
- **Hooks in:** `src/animations.rs` (`Animations`, `follow`, `DURATION`),
  `INDICATORS` in `src/bar.rs`, `animations_setting` in
  `src/panels/system.rs`, `ui::checkbox`. The system panel is split into
  cards like the network panel: diagnostics, then Settings under its heading.

### 4. Missing keybinds
- **Status:** done (2026-11-11). Every panel has a mango bind now:
  `SUPER+w` wallpaper, `SUPER+d` calendar, `SUPER+u` notifications,
  `SUPER+SHIFT+a` audio, `SUPER+SHIFT+n` network, `SUPER+SHIFT+b`
  bluetooth, `SUPER+/` the keybind sheet (item 23). `SUPER+m` opens the
  session menu instead of mango's `quit`, so logging out asks twice.
- **Hooks in:** `mango/.config/mango/config.conf` only. The binds were
  regrouped under short headings for the sheet (same binds, reordered):
  shell panels, apps, mango, focus window (with `SUPER+Tab` jump), move
  window, resize window, window state (with close), minimize and
  scratchpad, switch workspace, move window to workspace (with
  `CTRL+SUPER+←/→`), scroller layout, mouse, volume, brightness, playback;
  a leftover empty `#dwindle layout` comment went.
- **Note:** binds can't be tested with `wtype`: mango ignores its virtual
  keyboard for binds (an existing `SUPER+c` didn't fire either).

### 23. Keybind sheet
- **Status:** done (2026-11-11). `SUPER+/` opens a centered panel listing
  every mango bind in three columns, with a search box (every word must be
  in the keys, the action or the section title). Under mango's sections
  it adds the keys every panel shares (`?`, `Esc`).
- **Hooks in:** `src/keybinds.rs` reads `~/.config/mango/config.conf`
  (a link to the dotfiles, so edits show without a rebuild): binds go
  under the first line of the comment block above them, cut at `, : ; (`;
  an option line between binds ends a section; actions are put into words
  (shell IPC, wpctl, mango dispatchers; anything unknown shows the
  dispatcher), runs of non-spawn binds that differ in their last word fold
  (`Super+H L K J`, `Super+1–9`); `XF86` keys are left out.
  `src/panels/keybinds.rs` re-reads the config only when its mtime changes,
  and spreads the sections over the columns as evenly as they go (a long
  section carries on under its heading again; overflow says how many are
  left). Tests for both.

### 24. Screenshot menu on top
- **Status:** done (2026-11-12). The menu now draws over every panel left
  open underneath (before, it was made third, so ten panels covered it).
- **Hooks in:** `src/main.rs`: its window is made after the last panel
  (window order is stacking order). Mango gives the menu the keyboard, but
  amane has one focused text input for the whole shell, which takes every
  typed character whichever window has the keyboard, so number keys landed
  in the keybind sheet's search below. `panels::focused()` (used by the
  keybind sheet, the pickers and the network password) only marks an input
  focused while the screenshot menu is closed; mango's keyboard leave clears
  the focus it already had. Checked over the keybind sheet: arrows and
  letters reach the menu, Escape closes only the menu.

### 25. Weather in the bar's clock
- **Status:** done (2026-11-12). The clock ends with ` | <icon> <temp>`
  (like `08:59 Thu 12 Nov | 󰖙 3°`), the weather now, from the same forecast
  as the calendar; nothing after the date until the first forecast is in.
- **Hooks in:** `src/clock.rs` `view` calls `weather::refresh_for_bar()` on
  every redraw (once a minute), which fetches when the forecast is over 2 h
  old (`BAR_STALE`), or 10 min after a failed fetch (`BAR_RETRY`), so curl
  doesn't run every minute while offline. The calendar keeps its 30 min / 1
  min (`due()` takes a `Window`). `weather refresh` and `weather location`
  now take effect within a minute, without opening the calendar. Test:
  `the_bar_waits_longer`.

### 26. Privacy indicator
- **Status:** done (2026-11-12). A red dot (`theme.error`, `#d25780`, the one
  color besides the accent) is the leftmost item on the bar's right side
  while an app records from a microphone or the screen is shared; clicking
  it opens the audio panel. Camera use isn't shown (not asked for).
- **Hooks in:** `src/audio/privacy.rs`, the `Privacy` service. A
  long-running `pw-mon -N` is the change signal (restarted 2 s after it
  exits, the dot cleared meanwhile); `NodeEvents` keeps only events about
  nodes, since each `pw-dump` joins as a client and reacting to clients
  looped (~5% CPU). A burst settled for 300 ms is read with `pw-dump Node`
  and `detect()`: a running `Stream/Input/Audio` that isn't recording a
  speaker (`stream.capture.sink`, `stream.monitor`) is the microphone; a
  `Video/Source` named `xdpw*` (xdg-desktop-portal-wlr's cast) is the
  screen. No pw-dump runs while nothing changes. `src/bar.rs`:
  `privacy_dot`. Tested live with a recording from a temporary virtual
  source; the screen share is untested live. Tests: `audio::privacy::`.

### 31. Screen recording
- **Status:** done (2026-11-12). The screenshot menu's rows 6 and 7,
  "Fullscreen (record)" (the focused monitor) and "Region (record)" (slurp),
  record with wf-recorder to `~/Videos/Screencasts/<stamp>.mp4`, no sound,
  h264 on the AMD gpu (`h264_vaapi`, `/dev/dri/renderD128`), falling back to
  `libx264` when the gpu try exits within a second. The bar shows the record
  glyph and the time (`1:09`) in the accent at the left of its right side;
  clicking it, SUPER+P or `amane ipc call record stop` stops it (SIGINT, so
  the file is finished). A notification then says where it was saved, or
  quotes wf-recorder's last lines. Sound, webm/gif and copying the path were
  offered and not wanted. A recorded region gets a 2 px accent frame 1 px
  outside it, so the frame isn't in the video (slurp itself can't stay up:
  it holds the pointer until you drag). Dimming outside it, like slurp, was
  offered and not wanted.
- **Hooks in:** `src/recording.rs` (`Recording` service ticking each second,
  `start`, `stop`, the `record` IPC: `stop`, and `menu`, which stops a
  recording or toggles the screenshot menu); `src/panels/screenshot.rs`
  (`Action::Record`); `src/bar.rs` `recording_time`;
  `src/keybinds.rs` names the new SUPER+P command. The frame is
  `recording::frame_view`, a full-screen overlay window per monitor that is
  always mapped and click-through and draws nothing without a region (the
  layer-surface crash rule); it reads `RecordedRegion`, a separate service
  so it doesn't redraw with the bar's time each second. The region is moved
  into its monitor's own units with `mmsg get all-monitors`. The pid,
  start, file and frame are kept in
  `$XDG_RUNTIME_DIR/amane-shell/recording`, so a recording
  survives a shell restart (checked against `/proc/<pid>/comm`).
  wf-recorder is started without `LD_LIBRARY_PATH`: the shell's holds an
  alsa-lib built against a newer glibc, which wf-recorder fails to load.
  Tests: `recording::`.
- **Outside the shell:** `wf-recorder` in `~/.nixos/home/packages.nix`;
  mango's SUPER+P now runs `amane ipc call record menu`.

### 35. Screenshot annotation
- **Status:** done (2026-11-12). The screenshot menu's fifth row, "Region
  (annotate)", opens the dragged region in satty (arrows, boxes, text, blur,
  markers, crop). Enter copies and closes; satty's save button writes
  `~/Pictures/Screenshots/<stamp>.png`, like the menu's saves. satty sends
  its own notifications, so the shell sends none. Region only (satty can
  crop a full capture anyway); swappy and OCR were offered and not wanted.
- **Hooks in:** `src/screenshot.rs`: `Target::Annotate` captures with grim
  to stdout and hands the png to `satty --filename -` through
  `command::feed`, which waits for satty to close (on `take()`'s own
  thread); the screenshots folder is made first (`screenshots_folder()`),
  since satty doesn't make it. A missing satty gives the usual "Screenshot
  failed" notification. `src/panels/screenshot.rs`: the extra `Choice`.
- **Outside the shell:** satty and its settings come from home-manager's
  `programs.satty` in `~/.nixos/home/satty.nix` (imported in
  `home/default.nix`): output file, `wl-copy`, Enter copies, early exit,
  font `BlexMono Nerd Font`. Kept there rather than as shell arguments, so
  satty behaves the same when started elsewhere.
