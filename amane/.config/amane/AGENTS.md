# amane shell: context for agents

A desktop shell for the MangoWM Wayland compositor, written in Rust on the
[amane](https://github.com/MystiaFin/amane) shell library. This file is loaded
into every session. Read it before changing anything; `RECOMMENDATIONS.md` has
what's open, `DONE.md` what was built (by item number), `HISTORY.md` how and why
things came to be the way they are.

## 1. Where things live

- **This folder** is `~/.nixos/config/amane/.config/amane`, inside the user's
  NixOS dotfiles repo (`~/.nixos`, with `config/` as a git submodule that also
  holds macOS configs). `~/.config/amane` links to it through the
  home-manager store (an out-of-store symlink set in `~/.nixos/home/packages.nix`).
  Open files by the real path.
- **The build:** `amane compile` writes its own cargo project to
  `~/.cache/amane/project` (its manifest points at `~/.config/amane/src/main.rs`)
  and the amane library to `~/.cache/amane/library`, overwritten on every
  compile, so patching the library is pointless. The binary is
  `~/.cache/amane/amane-shell`. No crates can be added: only `std`, `amane`
  and libc through `unsafe extern "C"` (see `src/clock.rs`).
- **`Cargo.toml` here** is only for the editor. Its `[[bin]] path` must stay
  the relative `'src/main.rs'`: an absolute path through the `~/.config`
  symlink makes rust-analyzer report every file as "not included in any
  crates". rust-analyzer's `cargo check` diagnostics still fail (the
  dependencies' build scripts need `pkg-config` from amane's nix wrapper); a
  `rust-analyzer.toml` was tried and ignored, and changing the Neovim config
  was declined. Navigation and completion work.
- **State the shell keeps:**
  - `~/.local/state/amane-shell/` (survives reboots, `src/state.rs`):
    `accent`, `launches` (launcher frecency), `weather` (the place),
    `wallpaper` (a path or `none`)
  - `$XDG_RUNTIME_DIR/amane-shell/` (until logout): `clipboard/` (image
    previews), `notifications.json` (the history), `covers/` (cover art)
  - `$XDG_RUNTIME_DIR/amane-notifications/` is amane's own (notification
    pictures sent as raw pixels)
- **Wallpapers** come from `~/Pictures/Wallpapers` (PNG/JPEG).
- **The mango config** is `~/.nixos/config/mango/.config/mango/config.conf`
  (linked like this folder). `exec_once=amane run` starts the shell. NixOS
  host config: `~/.nixos/hosts/prometheus/default.nix`; home-manager modules:
  `~/.nixos/home/`.

## 2. Workflow

- **Build:** `amane compile`. Zero warnings is the rule; fix every one.
- **Format:** `rustfmt --edition 2024 src/main.rs` (formats every module).
- **Test:** `./test.sh` runs `cargo test` on the cache project with the same
  toolchain and libraries as `amane compile` (it sources amane's nix wrapper
  minus its `exec` line). `./test.sh system::` filters. 74 tests at last count;
  add tests for pure logic (parsers, formatting, state files).
- **Restart the running shell:** `amane compile --restart`, or `amane restart`
  after a compile. A restart loses in-memory state (panel state, popups); the
  notification center comes back under "Earlier" (`src/notifications/history.rs`).
  The user is fine with restarts for checking work.
- **IPC:** `amane ipc call <name> ...`. Handlers: `panel` (`open|close|toggle
  <name>`, `close-all`, `list`), `lock`, `accent` (`<name>|next`), `weather`
  (`location <lat> <lon> <name>`, `refresh`), `media`
  (`play-pause|next|previous`), `brightness` (`up|down`).
- **Mango:** `mmsg get all-monitors`, `mmsg get all-clients`,
  `mmsg dispatch setlayout,tile` and so on. `SUPER+r` reloads mango's config.
- **Screenshots for checking UI:**
  - `grim -g "x,y wxh" file.png` takes **physical** pixel coordinates (screen
    is 2560×1440 at scale 1.5). Top-right panels: `"1940,0 620x700"`;
    the bottom-center popup: `"900,1200 760x240"`; full screen: `grim -s 1`.
  - The read tool sees new files late and caches by name: use a fresh file
    name each time, and wait (`sleep 10`) and check with `ls` before reading.
  - Key presses into a panel: `wt=$(nix build --no-link --print-out-paths
    nixpkgs#wtype)/bin/wtype; $wt '?'` (or `$wt -k Escape`, `-k Return`).
    Nothing gets installed.
- **Leave the user's machine as it was:** put volume, mute and brightness back
  after testing (`wpctl get-volume @DEFAULT_AUDIO_SINK@`,
  `ddcutil --bus 5 getvcp 10 --brief`). Ask before locking the session,
  toggling the microphone, starting media, or touching their files (like
  `~/Pictures/Wallpapers`). `media play-pause` toggles whatever player is
  active, so don't use it to "test" the no-player case.

## 3. Working agreements

- **Plan mode:** when it's on, only read and plan. Ask clarifying questions with
  options (recommended one first), present the plan, and wait for "Go ahead";
  if plan mode is still on at that point, say so and wait.
- **Scope:** don't edit outside this folder without asking. The mango config
  and the NixOS/home-manager files are edited only when the user asked for it
  or agreed to a plan that includes it; the user runs `nixos-rebuild` (sudo).
  For a new file a flake must see, `git add --intent-to-add` it (that's all).
- **Git:** the user commits. Don't commit, push or stage beyond intent-to-add.
  The user also edits code themselves between sessions; check `git status`
  / `git log` before assuming the tree is as you left it, and don't revert
  their changes.
- **Recommendations:** keep `RECOMMENDATIONS.md` statuses current. When an item
  is built, move it to `DONE.md` keeping its number, with a status line saying
  what was built. New ideas get the next free number (19 was the last).
- **Code style:**
  - lowercase explanatory comments, `/* */` for blocks, `//` for lines; say
    why, not what. Every module starts with a block comment on what it is.
  - descriptive names, no abbreviations; constants for every size and timing.
  - `rustfmt` edition 2024.
- **UI rules (decided, don't revisit):**
  - one panel open at a time (all in the `PANELS` group), except the
    screenshot menu, which opens on top so open panels can be captured
  - panels have a fixed size; reserve space rather than resize
  - the accent picked in the system panel is used for every highlight,
    warning and error. Five fixed status colors (error, warning, ok, hint,
    info) were tried and reverted on request.
  - font `BlexMono Nerd Font` (not Mono or Propo); it's monospaced, so a
    text's width is `chars × size × 0.6`
  - no Do Not Disturb mode
- **Nerd Font glyphs:** never guess a codepoint. Look it up in
  `https://raw.githubusercontent.com/ryanoasis/nerd-fonts/master/glyphnames.json`
  (e.g. `md-play` is `f040a`). A guessed one once rendered a phone.

## 4. Module map

`src/main.rs` declares the modules, starts the background pieces
(`session::watch_logind`, the bluetooth agent, clipboard recording, notification
history, the osd watcher) and registers the IPC handlers and windows. Windows
are made per monitor; the order matters for stacking: wallpaper first, then the
bar, backdrop, notification popups, osd, every panel, and the lock screen.

**Bar and always-there windows**
- `bar.rs`: the top bar (32 px, reserves its space). Workspaces of its own
  monitor (wide accent block for the shown one, dots for urgent), the layout
  symbol `[S]` (opens the layout picker), the clock (opens the calendar),
  and icons: audio, network with a VPN shield badge, bluetooth, and the NixOS
  snowflake (opens the system panel, accent while notifications are unread).
- `clock.rs`: the `Clock` service, waking at each minute with
  `clock_nanosleep` on `CLOCK_REALTIME`; the bar's clock text.
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
  `meter(t, width, percent, dimmed)`.
- `list.rs`: list rows and windowing for picker/menu panels (`Cursor`,
  `ROWS` = 8, rows, footers, `help_row`, `hotkey` for 1–9, `scroll_delta`).
- `picker.rs`: the `Picker` trait for search panels (search box, list,
  optional preview and footer, `?` in an empty search shows a help screen).
- `menu.rs`: the `Menu` trait for numbered menus with confirm-twice rows.
- `launcher.rs` (Picker): apps by frecency (`usage.rs`), `=sum` calculator
  (`calculator.rs`, Enter copies), `>cmd` runs, `>>cmd` runs in foot.
- `clipboard.rs` (Picker): cliphist history, × deletes, "Clear history"
  clicked twice, preview pane (`clipboard_preview.rs`).
- `layout.rs` (Picker): mango layouts, the one in use striped.
- `screenshot.rs` (Menu): fullscreen/region × copy/save (`../screenshot.rs`).
- `session.rs` (Menu): lock, suspend, logout, reboot, shutdown.
- `notifications.rs`: the notification center (live, then "Earlier").
- `network.rs`: Wi-Fi (`w` toggles, `r` rescans, password box), the
  Tailscale card (`v` connect, `i` log in, exit nodes), other VPN tunnels.
- `bluetooth.rs`: power `b`, rescan `r`, devices, pairing requests.
- `audio.rs`: "Now playing" (`h` `p` `l`), output and input levels (`j`/`k`
  volume, `m` mute, `n` mic mute), device lists; `volume_icon`, mic icons.
- `system.rs`: user@host, OS/kernel/uptime, CPU/memory/disk meters,
  network speed, battery on laptops, settings: accent swatches and the
  brightness slider; buttons for wallpaper (`w`), notifications (`n`),
  session (`s`).
- `calendar.rs`: month + weather as two cards on one grid (see 5.5).
- `wallpaper.rs`: the wallpaper strip along the bottom (see 5.6).

**Services and helpers**
- `audio/mod.rs`: `Devices` from `wpctl status`/`inspect` (amane's `Audio`
  only knows the defaults).
- `bluetooth/mod.rs`, `bluetooth/agent.rs`: BlueZ jobs and failures; the
  pairing agent (exports `org.bluez.Agent1`).
- `network/mod.rs`, `details.rs`, `profiles.rs`: NetworkManager over D-Bus on
  top of amane's `Network` (connect attempts, details, forgetting).
- `vpn/mod.rs`, `tunnels.rs`, `tailscale.rs`: tunnels in `/sys/class/net`,
  `ip route get`, `tailscale status --peers=false`, exit nodes.
- `weather/mod.rs`, `forecast.rs`: Open-Meteo CSV via curl.
- `notifications/mod.rs`, `card.rs`, `history.rs`: shared card, unread
  tracking (`Seen`), `send()` for the shell's own notifications, history.
- `brightness.rs`: brightness over DDC/CI or a laptop backlight (see 5.9).
- `covers.rs`: cover art paths/downloads for "Now playing".
- `layout.rs`: mango's layout per monitor (own `watch all-monitors` socket).
- `session.rs`: session actions, the `lock` IPC, logind watcher.
- `screenshot.rs`: grim/slurp/wl-copy, notification after capture.
- `system.rs`: `SystemInfo` (os, kernel, uptime, disk, network speed).
- `theme.rs`: `Theme` colors, `Accent` service and the `accent` IPC.
- `json.rs`: small JSON reader (`parse`, `Value` getters) and `quote()`.
- `state.rs`: `path(name)`/`save(name, text)` in the state folder.
- `calendar.rs`, `calculator.rs`, `usage.rs`, `clipboard_preview.rs`, `clock.rs`.

## 5. Subsystems

### 5.1 Services and threads
amane `Service`s are global, lazily created state (`X::read()`, `X::write()`).
A view that reads a service redraws when it changes. A service's `listen()` runs
on its own thread, **starting the first time anything reads it**, so services
that must run from launch have a `start()` that does `drop(X::read())` (the
`let _ = X::read()` form is rejected by the `let_underscore_lock` lint).
Never write a service from inside a view (the view holds a read lock, so it
deadlocks); views start work by spawning a thread or setting a static
(`AtomicBool`, `Mutex<Option<…>>`), and the thread writes when done (see
`weather::refresh_if_stale`, `wallpaper::read_if_stale`, `covers::path`,
`brightness::follow`). Take one service lock at a time in loops, never nested
writes. `Write` has `quiet()` for updates that shouldn't redraw.

### 5.2 Panels
Every panel window always exists (amane can't open windows at runtime) and is
shown or hidden through the `Panels` service. `panels::panel()` wraps each:
overlay layer, `Keyboard::OnDemand` (a hidden exclusive window would keep
focus), Escape closes, `?` shows key hints, content built only while shown.
The registry (`REGISTRY` in `panels/mod.rs`) names each panel, its groups and
an `on_close` hook (resets search, selection, previews). Opening a panel closes
those sharing a group; the replacement shows after `SWAP_DELAY` (100 ms) so the
old surface unmaps first. The backdrop (`backdrop.rs`) is a full-screen
transparent top-layer window **starting below the bar** (32 px margin, so the
bar stays clickable while a panel is open); it's always mapped and only
toggles click-through, and a click on it closes all panels. Corner panels use
`ui::corner(height, namespace)` + `ui::frame`.

### 5.3 Layer surfaces: the crash rule
Showing, hiding or resizing a layer surface while another changes makes mango
reconfigure surfaces; a configure acked after wlroots dropped it kills the
shell (`Protocol error 0 on zwlr_layer_surface_v1`). So: panels have fixed
sizes; popups (notifications, osd) are always mapped, never resize, are
click-through outside their content (`input_region`/`click_through`), and
only draw while showing; the wallpaper preview only changes a fill. Don't
introduce windows that show/hide quickly.

### 5.4 Notifications
amane's `Notifications` service is the daemon. `popups.rs` shows cards
top-right on the focused monitor (hidden while a right-side panel is open),
5 s each, critical ones until handled, hover holds. The center lists live
ones newest first, then restored ones under "Earlier". `history.rs` writes the
newest 50 (live + restored) to `$XDG_RUNTIME_DIR/amane-shell/notifications.json`
within a second of a change (temp file + rename); a restarted shell reads them
back as read-only `Kept` copies: no action buttons (amane's ids restart at 1,
senders can't be reached), clicking does nothing, × and Clear all remove them
(and amane's saved picture). amane's list can't be seeded (crate-private
fields), hence `card::Content::live`/`::kept`. Unread = received after the
center was last closed (`Seen`); restored ones count as read.

### 5.5 Calendar and weather
One panel under the clock, two cards in a column on a shared grid: a 24 px
gutter (week numbers) plus 7 columns of 50 px, 6 px apart, so hourly and daily
forecast slots line up under the weekdays. Cells are `bg1`; today, "Now" and
"Today" get the accent fill. Headers: title left, key buttons right (`h`/`l`
months, `t` today, `r` refetch). Weather: Open-Meteo CSV (no key), fetched on
open at most every 30 min, place via the `weather` IPC (Hilversum default).
Rain chance is always shown, 0% included.

### 5.6 Wallpaper
`wallpaper.rs`: `current` (kept) and `shown` (previewed) as `Option<PathBuf>`.
The background window per monitor fills with `Image::cover(path)
.thumbnail(monitor.width*2, monitor.height*2)` (bounded memory, enough for
scale 2), or `Color::TRANSPARENT` for None so mango's `root_color` (equal to
`bg_deep`) shows. The picker (`panels/wallpaper.rs`, from the system panel's
`w` or `panel toggle wallpaper`) is a full-width bottom strip: None, then the
folder's pictures by name (recognized by their first bytes, `is_picture`),
read each time it opens or on `r`. h/l/arrows/wheel move and preview on the
real background (only visible where no window covers it, by choice),
Enter/click keeps, Escape/outside restores. Selected tile: 2 px accent border;
in use: " · in use" after its name (names shortened to fit; left-aligned).

### 5.7 Lock and session
`session::watch_logind` locks on logind's `PrepareForSleep` and on this
session's `Lock` signal (needs `XDG_SESSION_ID` in the shell's environment,
which mango's `exec_once` passes). hypridle (home-manager,
`~/.nixos/home/hypridle.nix`) only calls `loginctl lock-session` (5 min), turns
the screen off with `wlopm` (5.5 min) and suspends (30 min); `before_sleep_cmd`
locks then waits 1 s (mango lacks the lock-notify protocol, so
`inhibit_sleep = 3` can't work). The session menu waits `LOCK_BEFORE_SUSPEND`
(1 s) too. mango also has a `sleep_monitor` dispatch, unused.

### 5.8 Volume/brightness popup and media
`osd.rs`: the `Osd` service polls amane's `Audio` 10×/s and
`brightness::changes()` (a counter bumped by every shell-made change). Any
volume/mute/mic change, from any source, shows the popup (bottom center of the
focused monitor, 300×48, 1.5 s, icon + `ui::meter` + percent, "Muted" dimmed);
brightness only for shell-made changes. Not at startup, not for volume while
the audio panel is open, not for brightness while the system panel is open.
Media: amane's `Media` (MPRIS, polled every second once started, "active"
player = playing one, else last shown). The `media` IPC and the audio panel's
"Now playing" card use it; covers via `covers.rs` (`file://` decoded,
`https://` fetched once with curl, PNG/JPEG only).

### 5.9 Brightness
amane's `Brightness` only drives laptop backlights (`/sys/class/backlight`,
empty on this desktop), so `brightness.rs` uses DDC/CI: `ddcutil detect
--brief` once (bus per connector, `card1-DP-1` → `DP-1`), `getvcp 10 --brief`
when the system panel opens (≤ every 10 s, ~0.4 s per call), and writes with
`setvcp 10 N --noverify`, newest value only, one call at a time (slider drags
don't pile up). The slider controls the panel's monitor; the shortcuts the
focused one (5% steps; the first press after a restart detects and reads,
~1 s). A laptop backlight, when present, is used instead.

### 5.10 VPN and Tailscale
Tunnels from `/sys/class/net` (wireguard/tun). Tailscale status via
`tailscale status --json --peers=false` (3 KB, not 850 KB with 531 Mullvad
peers). Exit nodes: only None, what `tailscale exit-node suggest` suggests
(asked on panel open and after changes) and the one in use if different
(named via `tailscale whois`, cached by IP). Polls every 5 s (2 s with the
network or system panel open); shield badge on the bar's network icon, accent
when all traffic goes through the VPN.

## 6. amane gotchas

- **Keys:** no modifiers and no key repeat in panels. Delete, Insert, Page
  Up/Down, F-keys and lone modifiers all arrive as `Key::Other`. A focused
  `TextInput` takes every character (so search panels can't use letter keys).
- **D-Bus:** `Argument` can't build structs (blocks the polkit agent, item 10)
  and `Method` can't send error replies. Incoming structs arrive as
  `Value::List`.
- **Images:** PNG and JPEG only (detected by first bytes), each request decodes
  on its own thread with no limit; `Image::thumbnail(w, h)` keeps only a small
  copy (cached forever), `.blurred(r)` blurs once; full-size images are freed
  when no window shows them.
- **Widgets:** `ScrollArea` scrolls vertically only and can't be scrolled from
  code (lists window their rows instead). Calling `.height(x)` on a value
  returned as a built `Rectangle` can resolve to the `Widget` getter; pass
  sizes into helpers instead (why `ui::heading` takes a height).
  `Rectangle::fill` takes a `Color` or an `Image`.
- **Monitors:** `Monitor` has `name`, `width`, `height` in widget units, but no
  scale factor.
- **Layers:** `Layer::Background|Bottom|Top|Overlay`, `Zone::Reserve|Respect|
  Ignore`, `Keyboard::None|Exclusive|OnDemand`; mango gives exclusive focus to
  any exclusive surface, even hidden ones.
- **Idle cost:** amane's `Apps` rescans icon themes every 30 s and `Network`
  re-reads saved profiles each second once started; accepted.
- The upstream list in `RECOMMENDATIONS.md` has these written up as issue texts.

## 7. Machine facts

- Host `prometheus`, NixOS 26.05, user `maarten`, home-manager as a NixOS module.
- One monitor: `DP-1`, AOC U27B3AF, 2560×1440 at scale 1.5 (1706×960 widget
  units), DDC/CI on `/dev/i2c-5` (works without the `i2c` group).
  `root_color=0x0f0c0aff` in mango (= `bg_deep`).
- Keyboard: NuPhy Halo65 (65%, no media or brightness keys; its Fn layer
  could be remapped with NuPhy's configurator). Caps is ctrl/esc via keyd.
- Audio: PipeWire/WirePlumber (`wpctl`), output "Built-in Audio Analog
  Stereo", an HDMI output on the monitor.
- Tailscale on (`prometheus`, 100.67.176.75), Mullvad exit nodes.
- polkit daemon on, no agent (item 10). hypridle and `wlopm` via
  home-manager. `ddcutil` and `hardware.i2c.enable` in the host config.
- **Mango binds the shell relies on:** `SUPER+space` launcher, `SUPER+c`
  clipboard, `SUPER+s` system, `SUPER+n` layout, `SUPER+p` screenshot;
  `SUPER+=`/`-` volume ±5% (`wpctl`), `SUPER+0` mute, `SUPER+SHIFT+0` mic
  mute; `SUPER+SHIFT+=`/`-` brightness ±5%; `SUPER+\` play/pause,
  `SUPER+]`/`[` next/previous; plus `XF86Audio*`/`XF86MonBrightness*`
  equivalents. All volume/media/brightness binds are `bindl` (work while
  locked). Taken `SUPER+letter`s: h j k l t b q r f g i o v c s n p m;
  `SUPER+w` is free. `SUPER+m` still runs mango's `quit` (item 4).

## 8. Where to pick up

`RECOMMENDATIONS.md`: open are 4 (missing keybinds), 12 (leftover configs),
15 (a key for the clipboard's Clear history), 18 (dimming before the lock);
10 (polkit agent) is blocked on amane, 17 (floating window decorations) is
parked. Its "Already decided" list and the upstream list are worth reading
before proposing anything. `DONE.md` holds items 1, 2, 3, 5–9, 11, 13, 14,
16, 19. `HISTORY.md` is the session-by-session log.
