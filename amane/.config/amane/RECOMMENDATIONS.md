# amane shell: recommendations

Ideas for the shell, kept so they aren't lost between sessions. Each item has a
status (open, blocked, parked), why it's worth doing, where it would hook in and
what to settle first. Update the status as things change; once an item is
built, move it to DONE.md, keeping its number.

`amane compile` only builds `src/`, so this file doesn't affect the shell.

## Already decided

Settled earlier, no need to revisit:

- Only one panel open at a time: every panel is in one group (`PANELS` in
  `src/panels/mod.rs`). Mango never gives the keyboard back to a panel left open
  underneath, so stacked panels broke Escape. The exception is the screenshot
  menu, which is in no group, so it opens on top of other panels and they can
  be captured.
- The panel under the screenshot menu doesn't get the keyboard back when the
  menu closes (click it to type again). Doing that would mean every panel is
  `Keyboard::Exclusive` while open and drops to `None` in a commit before
  hiding, with a short forced grace period when animations are off: a missed
  path leaves an invisible panel holding the keyboard, and the extra commit
  per close risks the layer-surface crash. Weighed and declined (2026-11-12).
- amane's own `Apps` and `Network` services stay, despite their idle cost
  (~1% CPU each; see the upstream list).
- No Do Not Disturb mode.
- Font stays `BlexMono Nerd Font`, not the Mono or Propo variant.
- Every accent, warning and error uses the accent picked in the system panel;
  there are no separate warning/error colors. Five fixed status colors
  (error, warning, ok, hint, info) were tried and reverted. The one exception
  is `theme.error` (`#d25780`), used only for the bar's privacy dot (item 26),
  which must stand out whatever accent is picked (2026-11-12).
- Panels have a fixed size (showing, hiding or resizing a layer surface while
  another changes can crash the shell, see the upstream list).
- Neovim: the relative `path = 'src/main.rs'` in `Cargo.toml` is what lets
  rust-analyzer find the crate. Its `cargo check` diagnostics still fail, since
  the dependencies' build scripts need amane's `pkg-config` setup; a
  `rust-analyzer.toml` ignored the override commands, and changing the Neovim
  config was declined.
- SUPER+P runs `amane ipc call record menu`, not `panel toggle screenshot`
  like the other panel binds: it stops a running recording, else toggles the
  screenshot menu. Kept explicit rather than hiding a stop inside `panel
  toggle`, or adding a second stop key (2026-11-12).
- No interim polkit agent (like `polkit_gnome`); wait for amane instead (item
  10).

## Gaps worth closing first

Nothing open here right now (item 4, the keybinds, is in DONE.md).

## Polish on what exists

### 15. A key for the clipboard's "Clear history"
- **Status:** open
- **Why:** clearing is click-only (click twice); every other action in the
  panels has a key.
- **Hooks in:** `src/panels/clipboard.rs`, arming and confirming like the
  button does, e.g. with `!`.
- **To settle:** which key, given that the search box takes every typed
  character.

### 17. Decorations for floating windows
- **Status:** parked
- **Why:** floating windows have no title bars.
- **Hooks in:** not amane: layer surfaces can't stack between app windows
  (they're above or below all of them), and moving them along with a window
  risks the layer-surface crash. mango has `allow_csd` (apps draw their own
  title bars), borders, `border_radius` and scenefx shadows; its group and tab
  bars only decorate grouped windows.
- **To settle:** whether `allow_csd` and the border/shadow options are enough,
  after checking mango's docs.

## New features

Common desktop-shell features the shell doesn't have yet, picked from a list
on 2026-11-12. Not wanted (from the same list): a system tray, the focused
window's title in the bar, a caffeine/idle-inhibit toggle, a window switcher,
a screen color picker, calendar events, OCR of a screenshot region.

### 27. Per-app volume
- **Status:** open
- **Why:** the audio panel only sets device volume; one loud app can't be
  turned down on its own.
- **Hooks in:** `src/panels/audio.rs`: an "Apps" list under the devices, a
  slider and mute per playback stream (`wpctl status` Streams, `wpctl
  set-volume <id>` / `set-mute <id>`), parsed in `src/audio/mod.rs`. The
  panel has a fixed size, so the list scrolls like the device lists.
- **To settle:** how many rows before it scrolls; how to name a stream (app
  name, media name) and whether to show app icons.

### 28. Night light
- **Status:** open
- **Why:** no warmer screen in the evening.
- **Hooks in:** wlsunset or gammastep (wlr-gamma-control) as a home-manager
  service, or started and stopped by the shell; a checkbox in the system
  panel's Settings (`ui::checkbox`, a state file like `animations`), maybe a
  key. The weather location's lat/lon could drive sunset times.
- **To settle:** whether mango supports wlr-gamma-control (check first);
  which tool; temperature; schedule (sun-based or fixed times); whether the
  DDC brightness (`src/brightness.rs`) should drop too.

### 29. Bluetooth battery
- **Status:** open
- **Why:** headphones' and controllers' battery levels aren't shown anywhere.
- **Hooks in:** `src/panels/bluetooth.rs`, a percentage beside connected
  devices, from BlueZ's `org.bluez.Battery1.Percentage` (read in
  `src/bluetooth/mod.rs`). Many headsets only report it with `Experimental =
  true` in bluez's `main.conf` (`hardware.bluetooth.settings` in the host
  config).
- **To settle:** panel only, or also on the bar's Bluetooth icon; a
  notification when it runs low.

### 30. Emoji and glyph picker
- **Status:** open
- **Why:** no way to search for and type an emoji or Nerd Font glyph.
- **Hooks in:** a `Picker` (`src/panels/picker.rs`) with its own panel and
  mango bind, or a launcher prefix (`src/panels/launcher.rs`, like `=`);
  Enter copies with wl-copy, like the calculator. Data read at runtime (no
  crates): an emoji list (e.g. Unicode's `emoji-test.txt` from a nix
  package) and nerd-fonts' `glyphnames.json`. Recently used first through
  `src/usage.rs`.
- **To settle:** own panel or launcher prefix; emoji, glyphs or both; where
  the data files come from.

### 32. Timer and pomodoro
- **Status:** open
- **Why:** no timers or reminders.
- **Hooks in:** started from the launcher (a prefix like `t25m`, next to `=`
  and `>`) or `amane ipc call timer ...`; a countdown in `src/bar.rs`; a
  notification (`notifications::send`) when it ends. The end time kept in the
  runtime folder (`state::runtime_path`) so a shell restart keeps it.
- **To settle:** plain timers, pomodoro cycles (25/5) or both; one or several
  at once; where in the bar; cancel/pause from the bar.

### 33. Health warnings
- **Status:** open
- **Why:** an overheating CPU, a full disk or a failed service goes unnoticed.
- **Hooks in:** CPU/GPU temperature from `/sys/class/hwmon` in the system
  panel's diagnostics card (`src/system.rs`, `src/panels/system.rs`); a
  background check every few minutes for low disk space (`df`, already read
  for the disk meter) and failed systemd units (`systemctl --failed`, system
  and `--user`), sending one shell notification per new problem.
- **To settle:** thresholds (disk %, temperature); how often to check;
  which sensors to show.

### 34. Removable drives
- **Status:** open
- **Why:** plugging in a USB drive shows nothing, and mounting or ejecting
  needs a terminal.
- **Hooks in:** udisks2 over D-Bus (`org.freedesktop.UDisks2`, its
  ObjectManager's `InterfacesAdded`/`Removed`): a notification when a drive
  arrives, with Mount (and Open in the file manager); mount, unmount and eject
  in a list (system panel, or its own panel). Needs `services.udisks2.enable`
  in the host config (check).
- **To settle:** where the list lives; automount or ask; whether the
  notification's buttons are enough.

## Outside the shell

### 10. Polkit agent
- **Status:** blocked on amane. The polkit daemon is on (`security.polkit.enable
  = true` in `hosts/prometheus/default.nix`) and its helper socket
  `/run/polkit/agent-helper.socket` listens, but registering as an agent needs
  a D-Bus struct argument amane doesn't have (see the upstream list). No agent
  runs until then.
- **Why:** with no polkit agent running, anything that asks for an admin
  password fails silently (NetworkManager was hit; the `networkmanager` group
  worked around that one).
- **Decided:** an agent built into the shell, its prompt a centered dialog
  like the screenshot and session menus (what's asked, the program and
  action, a password field, Cancel and Authenticate); opening it closes other
  panels so it gets the keyboard.
- **Notes for later:**
  - register with `org.freedesktop.PolicyKit1.Authority.RegisterAuthenticationAgent`
    (subject `("unix-session", {"session-id": <"1">})`, from `XDG_SESSION_ID` or
    logind's `GetSessionByPID`), and export
    `org.freedesktop.PolicyKit1.AuthenticationAgent` with `BeginAuthentication`
    and `CancelAuthentication`, the way `src/bluetooth/agent.rs` exports BlueZ's
    agent; register again when polkitd restarts.
  - the helper (polkit 127, socket-activated): connect to
    `/run/polkit/agent-helper.socket` with `std::os::unix::net::UnixStream`,
    write `username\n` and `cookie\n`, then read lines (escaped like
    `g_strcompress`): `PAM_PROMPT_ECHO_OFF <prompt>` and `PAM_PROMPT_ECHO_ON`
    are answered with a line, `PAM_ERROR_MSG`/`PAM_TEXT_INFO` are shown, and
    `SUCCESS` or `FAILURE` ends it. On `FAILURE`, connect again with the same
    cookie to let you retry.
  - reply to `BeginAuthentication` only after the helper says `SUCCESS`;
    polkitd then checks whether the cookie was authenticated. amane can't send
    error replies, but that's fine here: cancelling is replying without
    authenticating, which callers report as "Not authorized" instead of
    "Request dismissed".
  - the identity: your own user if polkit lists it, else the first
    `unix-user`; requests arriving while one is open wait their turn.

### 12. Leftovers
- **Status:** open
- **Why:** no longer used since the shell took over.
- **What:**
  - `mango/.config/mango/scripts/screenshot*.sh` (the screenshot panel does
    this itself)
  - `fuzzel` in `~/.nixos/home/packages.nix`, its config symlink there, and
    the `fuzzel` config folder
  - the `wlogout` config (points at `uwsm stop`, which isn't installed)
  - the `hypr` folder: unused here, since hypridle's config is now generated
    by home-manager (`~/.nixos/home/hypridle.nix`)
  - the `rofi`, `waybar` and `uwsm` folders in `~/.nixos/config`. That folder
    is a git submodule that also holds macOS configs (hammerspoon, karabiner
    and others), so these may be used on another machine; check before
    deleting.

### 18. Dimming before the lock
- **Status:** open
- **Why:** the old Hyprland setup dimmed the monitor with `ddcutil` before
  locking; hypridle (item 1 in DONE.md) only locks, turns the screen off and
  suspends.
- **Hooks in:** a listener in `~/.nixos/home/hypridle.nix`. DDC/CI works now
  (`hardware.i2c.enable` and `ddcutil` are in the host config, the monitor
  answers on `/dev/i2c-5`, no `i2c` group needed), so the listener could call
  `ddcutil --bus 5 setvcp 10 <low>` and restore on resume, or go through the
  shell (`src/brightness.rs`) so the slider stays in step.
- **To settle:** whether it's wanted, how dim, and when (e.g. at 4.5 min,
  before the 5 min lock).

## Worth reporting to amane upstream

Behavior the shell works around, better fixed in amane itself. Could be written
up as issue texts.

- **Hide/show crash** (confirmed 2026-11-11 with `WAYLAND_DEBUG=1`).
  - **Mechanism:** hiding a layer window attaches a null buffer
    (`hide()` in `wayland/update.rs`), which unmaps it; per the layer-shell
    protocol it returns to its just-created state and wlroots discards every
    configure it sent that surface. mango re-arranges the monitor on any
    map, unmap or resize and sends a configure to *every* layer surface on
    it, size changed or not. If a surface unmaps while such a configure is
    in flight, smithay-client-toolkit acks it anyway (it acks every
    configure unconditionally before amane sees it,
    `shell/wlr_layer/dispatch.rs:69`), and wlroots answers with
    `wrong configure serial` (error 0), dropping the connection.
  - **Log** (network panel and screenshot menu closed by one close-all):
    ```
    -> wl_surface#62.attach(nil, 0, 0); commit()    panel A hides
    -> wl_surface#78.attach(nil, 0, 0); commit()    panel B hides, same pass
    <- zwlr_layer_surface_v1#79.configure(25617, 400, 630)  sent before B's unmap
    -> zwlr_layer_surface_v1#79.ack_configure(25617)
    <- wl_display.error(zwlr_layer_surface_v1#79, 0, "wrong configure serial: 25617")
    ```
  - **Triggers:** any unmap while another layer surface maps, unmaps or
    resizes: a panel hiding while another shows (why `SWAP_DELAY` exists),
    or two panels hiding together. The latter still breaks today: close-all
    with the screenshot menu open on top of a panel.
  - **Symptoms:** the process keeps running (IPC still answers) but every
    window is gone; amane doesn't exit or log on a lost connection, and
    `amane run` sends the shell's stderr to `/dev/null`, so nothing records
    the error.
  - **Upstream fix:** after a null-buffer commit, don't ack that surface's
    configures until a `wl_display.sync` sent after the commit returns
    (events arrive in order, so stale configures come first). This needs
    amane's own handling of `zwlr_layer_surface_v1` events instead of
    sctk's automatic ack. Also: exit (or log) on a protocol error, and keep
    the shell's stderr somewhere.
  - **Related:** a configure arriving while hidden makes
    `OpenWindow::resize` draw and attach a buffer, showing the window again.
  - **Worked around** by keeping popups mapped and delaying panel swaps
    (`SWAP_DELAY` in `src/panels/mod.rs`). A shell-side alternative: never
    unmap panels, hide them like the popups (transparent, click-through, no
    keyboard).
  - **Testing it:** a live process proves nothing. Check that
    `mmsg get all-layers` still lists the bar's `"name":"amane"`.
- **Keyboard mode of hidden windows.** Settings aren't sent while hidden, so a
  hidden window keeps `Exclusive` in the compositor, and mango then gives it
  focus. Worked around with `Keyboard::OnDemand` for every panel.
- **`Apps` rescans every icon theme every 30 s** once started (~1% CPU idle on
  this machine), and never stops.
- **`Network` re-reads every saved profile each second** (`GetSettings` per
  profile on every poll); `connected()` only means full internet
  (`CONNECTED_GLOBAL`).
- **No modifiers and no key repeat** in `KeyboardHandler`, so no Ctrl+n/p/y in
  lists and no held arrow keys.
- **Delete, Insert, Page Up/Down, F-keys and modifier keys pressed alone are all
  `Key::Other`** (`character()` in `wayland/keyboard.rs`: Delete types the DEL
  control character, a lone modifier types nothing), so none of them can be a
  keybind; the clipboard panel wanted Delete for removing an entry.
- **No D-Bus error replies** (`Method` can only `reply`), which the Bluetooth
  agent works around by cancelling pairings instead of rejecting them.
- **No struct arguments for D-Bus calls.** `Argument` builds basic values,
  lists, `a{sv}` maps, groups and variants, but no struct, so methods taking
  one can't be called. Example: polkit's `RegisterAuthenticationAgent(subject
  (sa{sv}), locale s, object_path s)`, needed for a polkit agent, where the
  subject is `("unix-session", {"session-id": <"1">})`. Calling it through
  `busctl` doesn't help, since polkit sends its requests to the connection that
  registered. Suggested fix: `Argument::Struct(Vec<Argument>)`, converted in
  `dbus/convert.rs` with the `StructureBuilder` that `body()` already uses
  (`Variant::Structure(...)`). Incoming structs already arrive as
  `Value::List`, so only sending is missing. Blocks item 10.
- **No error from connect/pair/forget** in `Network` and `Bluetooth`; the shell
  makes its own calls to report failures.
- **Patches to `~/.cache/amane/library` are overwritten** by `amane compile`,
  so library fixes can't be tried locally.
- **`ScrollArea` only scrolls up and down**, so the wallpaper strip slides its
  window along with the selection instead.
- **`Monitor` has no scale factor**, so the wallpaper is decoded at twice the
  monitor's size in widget units, a guess that covers scales up to 2.
- **The image decoder reads PNG and JPEG only**, not WebP.
- **Every image asked for starts its own decode thread**, with no limit: many
  full-size pictures requested at once all decode in parallel.
