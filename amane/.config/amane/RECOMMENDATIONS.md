# amane shell: recommendations

Ideas for the shell, kept so they aren't lost between sessions. Each item has a
status (open, done, dropped), why it's worth doing, where it would hook in and
what to settle first. Update the status as items get done or dropped.

`amane compile` only builds `src/`, so this file doesn't affect the shell.

## Already decided

Settled earlier, no need to revisit:

- Only one panel open at a time: every panel is in one group (`PANELS` in
  `src/panels/mod.rs`). Mango never gives the keyboard back to a panel left open
  underneath, so stacked panels broke Escape.
- amane's own `Apps` and `Network` services stay, despite their idle cost
  (~1% CPU each; see the upstream list).
- No Do Not Disturb mode.
- Font stays `BlexMono Nerd Font`, not the Mono or Propo variant.
- Every accent, warning and error uses the accent picked in the system panel;
  there are no separate warning/error colors.
- Panels have a fixed size (showing, hiding or resizing a layer surface while
  another changes can crash the shell, see the upstream list).

## Gaps worth closing first

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
- **Why:** session, calendar, screenshot (has `SUPER+p`), network, Bluetooth
  and audio panels have no mango bind, except the ones listed in
  `config.conf`. `SUPER+m` runs mango's `quit` straight away, while the session
  menu asks twice before logging out.
- **Hooks in:** `mango/.config/mango/config.conf` only, using
  `amane ipc call panel toggle <name>`; `SUPER+m` could open the session menu
  instead.
- **To settle:** which keys (several `SUPER+letter` are taken: h j k l t b q
  r f g i o v c s n p m).

## Polish on what exists

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

### 12. Leftovers
- **Status:** open
- **Why:** no longer used since the shell took over.
- **What:**
  - `mango/.config/mango/scripts/screenshot*.sh` (the screenshot panel does
    this itself)
  - `fuzzel` in `~/.nixos/home/packages.nix`, its config symlink there, and
    the `fuzzel` config folder
  - the `wlogout` config (points at `uwsm stop`, which isn't installed)
  - the `hypr` folder, if Hyprland isn't used any more

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
