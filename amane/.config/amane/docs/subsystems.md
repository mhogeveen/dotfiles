# amane shell: subsystems

How the larger parts work, numbered as `modules.md` refers to them.

## 5.1 Services and threads
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

## 5.2 Panels
Every panel window always exists (amane can't open windows at runtime) and is
shown or hidden through the `Panels` service. `panels::panel()` wraps each:
overlay layer, `Keyboard::OnDemand` (a hidden exclusive window would keep
focus), Escape closes (after first closing anything open inside, see
`backs_out`: the network password box), `?` shows key hints, content built only
while shown.
The registry (`REGISTRY` in `panels/mod.rs`) names each panel, its groups and
an `on_close` hook (resets search, selection, previews). Opening a panel closes
those sharing a group; the replacement shows after `SWAP_DELAY` (100 ms) so the
old surface unmaps first. The backdrop (`backdrop.rs`) is a full-screen
transparent top-layer window **starting below the bar** (32 px margin, so the
bar stays clickable while a panel is open); it's always mapped and only
toggles click-through, and a click on it closes all panels. Corner panels use
`ui::corner(height, namespace)` + `ui::frame`.

## 5.3 Layer surfaces: the crash rule
Showing, hiding or resizing a layer surface while another changes makes mango
reconfigure surfaces; a configure acked after wlroots dropped it kills the
shell (`Protocol error 0 on zwlr_layer_surface_v1`). So: panels have fixed
sizes; popups (notifications, osd) are always mapped, never resize, are
click-through outside their content (`input_region`/`click_through`), and
only draw while showing; the wallpaper preview only changes a fill. Don't
introduce windows that show/hide quickly.

## 5.4 Notifications
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

## 5.5 Calendar and weather
One panel under the clock, two cards in a column on a shared grid: a 24 px
gutter (week numbers) plus 7 columns of 50 px, 6 px apart, so hourly and daily
forecast slots line up under the weekdays. Cells are `bg1`; today, "Now" and
"Today" get the accent fill. Headers: title left, key buttons right (`h`/`l`
months, `t` today, `r` refetch). Weather: Open-Meteo CSV (no key), fetched on
open at most every 30 min (a failed fetch is tried again after 1 min), and by
the bar's clock, which shows icon and temperature, at most every 2 h (retry
after 10 min; it asks every minute as it redraws), place
via the `weather` IPC (Hilversum default). Rain chance is always shown, 0%
included.

## 5.6 Wallpaper
`wallpaper.rs`: `current` (kept) and `shown` (previewed) as `Option<PathBuf>`.
The background window per monitor fills with `Image::cover(path)
.thumbnail(monitor.width*2, monitor.height*2)` (bounded memory, enough for
scale 2), or `Color::TRANSPARENT` for None so mango's `root_color` (equal to
`bg_deep`) shows. The picker (`panels/wallpaper.rs`, from the system panel's
`w` or `panel toggle wallpaper`) is a full-width bottom strip: None, then the
folder's pictures by name (recognized by their first bytes, `is_picture`),
read each time it opens or on `r`. h/l/arrows/wheel move and preview on the
real background (only visible where no window covers it, by choice),
Enter/click keeps, Escape/outside restores. When the one in use isn't in the
list (moved, deleted, or the folder still being read), nothing is selected and
Enter only closes; `r` starts over from the one in use, preview included.
Selected tile: 2 px accent border; in use: " · in use" after its name (names
shortened to fit; left-aligned).

## 5.7 Lock and session
`session::watch_logind` locks on logind's `PrepareForSleep` and on this
session's `Lock` signal (needs `XDG_SESSION_ID` in the shell's environment,
which mango's `exec_once` passes). hypridle (home-manager,
`~/.nixos/home/hypridle.nix`) only calls `loginctl lock-session` (5 min), turns
the screen off with `wlopm` (5.5 min) and suspends (30 min); `before_sleep_cmd`
locks then waits 1 s (mango lacks the lock-notify protocol, so
`inhibit_sleep = 3` can't work). The session menu waits `LOCK_BEFORE_SUSPEND`
(1 s) too. mango also has a `sleep_monitor` dispatch, unused.

## 5.8 Volume/brightness popup and media
`osd.rs`: the `Osd` service polls amane's `Audio` 10×/s and
`brightness::changes()` (a counter bumped by every shell-made change). Any
volume/mute/mic change, from any source, shows the popup (bottom center of the
focused monitor, 300×48, 1.5 s, icon + `ui::meter` + percent, "Muted" dimmed);
brightness only for shell-made changes. Not while the sound server starts (the first 5 s,
and while amane still reads its empty default), not for volume while
the audio panel is open, not for brightness while the system panel is open.
Media: amane's `Media` (MPRIS, polled every second once started, "active"
player = playing one, else last shown). The `media` IPC and the audio panel's
"Now playing" card use it; covers via `covers.rs` (`file://` decoded,
`https://` fetched once with curl, PNG/JPEG only).

## 5.9 Brightness
amane's `Brightness` only drives laptop backlights (`/sys/class/backlight`,
empty on this desktop), so `brightness.rs` uses DDC/CI: `ddcutil detect
--brief` once (bus per connector, `card1-DP-1` → `DP-1`), `getvcp 10 --brief`
when the system panel opens (≤ every 10 s, ~0.4 s per call), and writes with
`setvcp 10 N --noverify`, newest value only, one call at a time (slider drags
don't pile up). The slider controls the panel's monitor; the shortcuts the
focused one (5% steps; the first press after a restart detects and reads,
~1 s). A laptop backlight, when present, is used instead.

## 5.10 VPN and Tailscale
Tunnels from `/sys/class/net` (wireguard/tun). Tailscale status via
`tailscale status --json --peers=false` (3 KB, not 850 KB with 531 Mullvad
peers). Exit nodes: only None, what `tailscale exit-node suggest` suggests
(asked on panel open and after changes) and the one in use if different
(named via `tailscale whois`, cached by IP). Polls every 5 s (2 s with the
network or system panel open); shield badge on the bar's network icon, accent
when all traffic goes through the VPN. Every tailscale call has a time limit
(`command::capture`); `up` also gets `--timeout=15s`. While tailscale needs a
login and has a login page, `v` opens that page instead of running `up`
(which would only wait for the login).


## 5.11 Animations
amane's `Animation<T>` (`f32`, `Color`) eases from where it is to a target
and, while it hasn't arrived, makes the window ask for another frame; once
it arrives the window rests, so there's no idle cost. Views are rebuilt on
every redraw, so animations live in a static `Mutex` outside the view (the
bar's `INDICATORS`, keyed by monitor and workspace id, created at rest on
their first target so nothing animates in). Every animated value goes
through `animations::follow(animation, target, enabled)`, which gives the
target at once while the system panel's Animations checkbox is off; read
`Animations::read().enabled()` once per view. One `DURATION` (200 ms) for all.

## 5.12 Privacy dot
`audio/privacy.rs` keeps the shell's one long-running child process,
`pw-mon -N`, as a signal that PipeWire changed; it's killed and waited for
when its output ends, and started again after 2 s (`RESTART_DELAY`), with
the dot cleared meanwhile. Only events about nodes count (`NodeEvents`):
every `pw-dump` is itself a PipeWire client, so reacting to client events
reads PipeWire in a loop. After 300 ms without node events (`SETTLE`) one
`pw-dump Node` is parsed with `json.rs`; a failed read keeps the dot as it
was. Microphone: a running `Stream/Input/Audio` not flagged
`stream.capture.sink`/`stream.monitor`. Screen: a `Video/Source` named
`xdpw*`. The dot is `theme.error`, the one non-accent color.
The same dump feeds `audio/streams.rs` (the audio panel's per-app volume):
every `Stream/Output/Audio`, its percent the cube root of the mean
`channelVolumes` in `info.params.Props` (as wpctl shows it), `mute`, and
`running` for not dimmed. Volume changes made elsewhere arrive as node
events too. A change from the panel is stored at once (the slider follows the
drag) and sent with wpctl by one worker thread that drops older volumes for
the same stream. WirePlumber remembers each app's volume and mute.
