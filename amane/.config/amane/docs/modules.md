# amane shell: module map

What each file in `src/` does. Section numbers like "see 5.6" refer to
`subsystems.md`.

`src/main.rs` declares the modules, starts the background pieces
(`session::watch_logind`, the bluetooth agent, clipboard recording, notification
history, the osd watcher) and registers the IPC handlers and windows. Windows
are made per monitor; the order matters for stacking: wallpaper first, then the
bar, backdrop, notification popups, osd, every panel, and the lock screen.

**Bar and always-there windows**
- `bar.rs`: the top bar (`bar::HEIGHT`, 32 px, reserves its space; the
  backdrop starts below it). Workspaces of its own
  monitor (wide accent block for the shown one, dots for urgent; width and
  color animate on a switch, see 5.11), the layout
  symbol `[S]` (opens the layout picker), the clock (opens the calendar),
  and icons: a red privacy dot while recording or sharing the screen, audio, network with a VPN shield badge, bluetooth, and the NixOS
  snowflake (opens the system panel, accent while notifications are unread).
- `clock.rs`: the `Clock` service, waking at each minute with
  `clock_nanosleep` on `CLOCK_REALTIME`; the bar's clock text ("17:05 Wed 11
  Nov | 󰖙 4°", the weather once a forecast is in; English whatever `LC_TIME`
  says, like the calendar). `local_now()`
  reads the local time through libc `localtime_r` (`time()`, `stamp()` for
  file names); use it instead of running `date`.
- `lock.rs`: the lock screen (amane's session lock + PAM): clock, date, user,
  password box; the wallpaper blurred and darkened behind it.
- `wallpaper.rs`: the `Wallpaper` service and the background-layer window per
  monitor (see 5.6).
- `osd.rs`: the volume/brightness popup and the `media` IPC (see 5.8).
- `notifications/popups.rs`: notification popups, top-right.

**Panels** (`src/panels/`)
- `mod.rs`: the panel registry, `Panels` service, `panel()` wrapper for every
  panel window, the `panel` IPC, `is_focused`/`focused_output`.
- `backdrop.rs`: the transparent click-outside-to-close window.
- `hints.rs`: `?` shows each button's key for 2 s.
- `ui.rs`: shared pieces: `BUTTON` (28), `PADDING` (12), `SIDES`,
  `CORNER_WIDTH` (400), `corner()` (top-right panel window), `frame()`,
  `title()`, `heading(t, text, height)`, `button`, `key_button`,
  `key_button_colored`, `key_text_button`, `icon_button_colored`, `switch`,
  `slider(t, width, value, dimmed, on_change)`,
  `meter(t, width, percent, dimmed)`; and for building panels: `header(title,
  buttons)`, `note(t, text, height)`, `line(height, text)`, `card(t, height,
  parts)`, `error_line(t, text, button)`, `list_box(t, rows)`,
  `text_width(text, size)`, `fit(longest label)` for every text button's
  width (switches size themselves), text sizes `TEXT_SMALL`/`NOTE`/`BODY`/`TITLE`
  (12/13/14/15), `ICON_SIZE` (16), `ICON_REFRESH`. Panels use these rather
  than their own copies.
- `list.rs`: list rows and windowing for picker/menu panels (`Cursor`,
  `ROWS` = 8, rows, footers, `help_row`, `hotkey` for 1–9, `scroll_delta`),
  and the corner panels' highlight lists (`step_highlight`, `follow_window`,
  `highlight_list`, `highlight_delta`). `panels::letter(key)` gives a typed
  letter, lowercased.
- `picker.rs`: the `Picker` trait for search panels (search box, list,
  optional preview and footer, `?` in an empty search shows a help screen).
- `menu.rs`: the `Menu` trait for numbered menus with confirm-twice rows.
- `launcher.rs` (Picker): apps by frecency (`usage.rs`), `=sum` calculator
  (`calculator.rs`, Enter copies), `>cmd` runs, `>>cmd` runs in foot.
- `clipboard.rs` (Picker): cliphist history, × deletes, "Clear history"
  clicked twice, preview pane (`clipboard_preview.rs`).
- `layout.rs` (Picker): mango layouts, the one in use striped.
- `screenshot.rs` (Menu): fullscreen/region × copy/save, region annotate in
  satty (`../screenshot.rs`), and fullscreen/region record
  (`../recording.rs`).
- `session.rs` (Menu): lock, suspend, logout, reboot, shutdown.
- `notifications.rs`: the notification center (live, then "Earlier").
- `network/mod.rs` (+ `vpn_cards.rs`, the Tailscale card and tunnel lines): Wi-Fi (`w` toggles, `r` rescans while on), the list (j/k or
  arrows highlight, Enter clicks), `d` and `f` leave and forget the joined
  network, or `f` the highlighted one once there is a highlight; the
  password box (Escape closes just the box, `back_out`), "No Wi-Fi adapter",
  the Tailscale card (`v` connect, `i` log in, exit nodes), other VPN
  tunnels. Its fixed height is the most the cards above can take plus three
  list rows (`ABOVE_LIST_AT_MOST`).
- `bluetooth.rs`: power `b`, rescan `r`, devices (j/k or arrows, Enter
  clicks, `d` disconnects, `f` forgets), pairing requests (Enter confirms,
  `c` cancels, `x` dismisses a notice or failure).
- `audio.rs`: "Now playing" (`h` `p` `l`), output and input levels (`j`/`k`
  output volume, `J`/`K` (Shift) mic volume, `m` mute, `n` mic mute), device
  lists (4 outputs, 2 inputs, then they scroll), an "Apps" list (3 two-line
  rows, then it scrolls: slider, percent, Mute per playback stream; idle or
  muted dimmed). `Tab` selects the next app, then the output again; `j`/`k`/`m`
  act on the selection, which resets on close (`reset`). `volume_icon`, mic
  icons.
- The network and Bluetooth highlight (`list::step_highlight`,
  `follow_window`): None until a key moves it, then the list is drawn from a
  first row (a `FIRST_ROW` static per panel) instead of through its
  ScrollArea (which can't scroll from code). The window only moves when the
  highlight would leave it, so each panel adds up the heights above its
  list to know how much shows; the wheel moves the highlight. Cleared on
  close. Lists sit in `ui::list_box`, which clips the rows inside its border
  (amane clips at the outer edge, border included), so rows can't cover it.
- `system.rs`: user@host, OS/kernel/uptime, then two cards like the network
  panel: CPU/memory/disk meters, network speed and battery on laptops; and
  under a "Settings" heading the accent swatches, the brightness slider and
  the animations checkbox (`a`, `ui::checkbox`). Buttons for wallpaper
  (`w`), notifications (`n`), session (`s`).
- `calendar.rs`: month + weather as two cards on one grid (see 5.5).
- `wallpaper.rs`: the wallpaper strip along the bottom (see 5.6).
- `keybinds.rs`: the keybind sheet (`SUPER+/`): a search box over mango's
  binds in three columns (`layout`/`place` spread the sections evenly), the
  config re-read when its mtime changes, plus the keys every panel shares.

**Services and helpers**
- `audio/mod.rs`: `Devices` from `wpctl status`/`inspect` (amane's `Audio`
  only knows the defaults).
- `audio/privacy.rs`: `Privacy`, whether a microphone is recording or the
  screen is shared (a long-running `pw-mon` as the signal, `pw-dump Node` to
  read; see 5.12).
- `audio/streams.rs`: `Streams`, the playback streams (`Stream/Output/Audio`)
  and the audio panel's selection; filled from privacy.rs's `pw-dump Node`
  (no polling of its own), changed through `wpctl set-volume`/`set-mute` on
  one worker thread that sends only the newest change per stream.
- `bluetooth/mod.rs`, `bluetooth/agent.rs`: BlueZ jobs and failures; the
  pairing agent (exports `org.bluez.Agent1`).
- `network/mod.rs`, `details.rs`, `profiles.rs`: NetworkManager over D-Bus on
  top of amane's `Network` (connect attempts, details, forgetting).
- `vpn/mod.rs`, `tunnels.rs`, `tailscale.rs`: tunnels in `/sys/class/net`,
  `ip route get`, `tailscale status --peers=false`, exit nodes.
- `weather/mod.rs`, `forecast.rs`: Open-Meteo CSV via curl; fetched by the
  calendar (30 min) and the bar's clock (2 h).
- `notifications/mod.rs`, `card.rs`, `history.rs`: shared card, unread
  tracking (`Seen`), `send()` for the shell's own notifications, history.
- `brightness.rs`: brightness over DDC/CI or a laptop backlight (see 5.9).
- `covers.rs`: cover art paths/downloads for "Now playing".
- `layout.rs`: mango's layout per monitor (own `watch all-monitors` socket).
- `session.rs`: session actions, the `lock` IPC, logind watcher.
- `screenshot.rs`: grim/slurp/wl-copy, notification after capture; annotate
  pipes the capture into satty (configured in `~/.nixos/home/satty.nix`).
- `recording.rs`: `Recording` (seconds recorded, for the bar), wf-recorder
  start/stop (SIGINT), the `record` IPC; picks up a recording still running
  after a restart from `$XDG_RUNTIME_DIR/amane-shell/recording`.
  `frame_view`: an always-mapped, click-through overlay window per monitor
  that frames a recorded region (`RecordedRegion`), just outside it.
- `system.rs`: `SystemInfo` (os, kernel, uptime, disk, network speed).
- `theme.rs`: `Theme` colors, `Accent` service and the `accent` IPC.
- `animations.rs`: the `Animations` on/off setting (state file
  `animations`), `DURATION` and `follow()` (see 5.11).
- `json.rs`: small JSON reader (`parse`, `Value` getters) and `quote()`.
- `command.rs`: `capture(command, timeout)`, a program's output with a time
  limit (kills and waits on timeout), `read` (its output on success),
  `feed(child, input)` (writes stdin, always waits), and curl's limits; used
  for tailscale, ddcutil, wpctl, ip, df, curl and grim, so a hung call can't
  leave a busy flag set. Use it for new external calls.
  `feed(child, input)` writes a piped stdin, closes it and always waits (no
  zombies when the write fails).
- `keybinds.rs`: mango's binds from `~/.config/mango/config.conf` as
  sections of keys and actions in words (`parse`, folding, `matches` for
  the search; see DONE.md item 23).
- `state.rs`: `path(name)`/`save(name, text)` in the state folder, saved
  through `write_atomically` (temp file + rename, also used by the
  notification history); `runtime_dir()` and `runtime_path(name)` for
  `$XDG_RUNTIME_DIR/amane-shell`.
- `calendar.rs`, `calculator.rs`, `usage.rs`, `clipboard_preview.rs`, `clock.rs`.

