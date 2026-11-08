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
- amane's own `Apps` and `Network` services stay, despite their idle cost
  (~1% CPU each; see the upstream list).
- No Do Not Disturb mode.
- Font stays `BlexMono Nerd Font`, not the Mono or Propo variant.
- Every accent, warning and error uses the accent picked in the system panel;
  there are no separate warning/error colors. Five fixed status colors
  (error, warning, ok, hint, info) were tried and reverted.
- Panels have a fixed size (showing, hiding or resizing a layer surface while
  another changes can crash the shell, see the upstream list).
- Neovim: the relative `path = 'src/main.rs'` in `Cargo.toml` is what lets
  rust-analyzer find the crate. Its `cargo check` diagnostics still fail, since
  the dependencies' build scripts need amane's `pkg-config` setup; a
  `rust-analyzer.toml` ignored the override commands, and changing the Neovim
  config was declined.
- No interim polkit agent (like `polkit_gnome`); wait for amane instead (item
  10).

## Gaps worth closing first

### 2. Volume and media keys, with an on-screen popup
- **Status:** open
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
- **Status:** open
- **Why:** amane has `Media`/`MediaPlayer` services (MPRIS: qutebrowser,
  Spotify, mpv and so on) that nothing in the shell uses yet.
- **Hooks in:** play/pause/next and the track title in `src/panels/audio.rs`;
  optionally a small bar indicator in `src/bar.rs` while something plays, and
  the track on the lock screen (`src/lock.rs`).
- **To settle:** panel only, or also bar and lock screen; what to show when
  several players are open.

### 4. Missing keybinds
- **Status:** open
- **Why:** session, calendar, screenshot (has `SUPER+p`), network, Bluetooth,
  audio and wallpaper panels have no mango bind, except the ones listed in
  `config.conf`. `SUPER+m` runs mango's `quit` straight away, while the session
  menu asks twice before logging out.
- **Hooks in:** `mango/.config/mango/config.conf` only, using
  `amane ipc call panel toggle <name>`; `SUPER+m` could open the session menu
  instead.
- **To settle:** which keys (several `SUPER+letter` are taken: h j k l t b q
  r f g i o v c s n p m; `SUPER+w` is free, a fit for the wallpaper picker).

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
- **Hooks in:** a listener in `~/.nixos/home/hypridle.nix`; NixOS needs
  `hardware.i2c.enable = true`, the `i2c` group and `ddcutil`.
- **To settle:** whether it's wanted, and whether the monitor supports DDC/CI.

## Worth reporting to amane upstream

Behavior the shell works around, better fixed in amane itself. Could be written
up as issue texts.

- **Hide/show crash.** Hiding a layer window attaches a null buffer; a
  configure already on its way then gets acked after wlroots dropped it,
  killing the shell with `Protocol error 0 on zwlr_layer_surface_v1`. Also, a
  configure arriving while hidden makes `OpenWindow::resize` draw and attach a
  buffer, showing the window again. Worked around by keeping windows mapped and
  delaying panel swaps (`SWAP_DELAY` in `src/panels/mod.rs`).
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
