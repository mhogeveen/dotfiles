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
- **Status:** open
- **Why:** the lock screen only runs when triggered by hand, so an unattended
  machine stays unlocked. No idle daemon is installed (neither hypridle nor
  swayidle); `hypr/.config/hypr/hypridle.conf` is a Hyprland leftover that
  nothing runs under mango.
- **Hooks in:** an idle daemon started from mango's `exec_once` (or a
  home-manager service) calling `amane ipc call lock`, then turning the
  screen off, then `systemctl suspend`. The shell already locks before any
  suspend (`session::watch_logind` in `src/session.rs`). The mango config has
  `idle_inhibit_*` settings, so it supports the idle protocols.
- **To settle:** hypridle or swayidle; the timeouts (e.g. lock after 5 min,
  screen off after 6, suspend after 30); how to turn the screen off under
  mango (`mmsg` dispatch or `wlopm`).

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
- **Status:** open
- **Why:** entries can't be deleted, and the history can't be cleared.
- **Hooks in:** `src/panels/clipboard.rs` (a `Picker`): `cliphist delete` for
  the selected entry and `cliphist wipe` for all. The search box takes letter
  keys, so these need buttons or non-letter keys (amane has no modifiers or
  Delete key). Optionally a preview of image entries via `cliphist decode` to a
  temporary file.
- **To settle:** how to trigger delete and clear without modifier keys.

### 9. Weather in the calendar
- **Status:** open (planned when the calendar was made)
- **Why:** asked for when the calendar was built; the panel was laid out as a
  list of sections for this.
- **Hooks in:** a new section after `calendar_section` in
  `src/panels/calendar.rs`, a weather service fetched only when the panel opens
  and kept for a while, and a `HEIGHT` increase.
- **To settle:** source (wttr.in or Open-Meteo, both need no API key), the
  location, and what to show (now only, or a few days).

## Outside the shell

### 10. Polkit agent
- **Status:** open
- **Why:** with no polkit agent running, anything that asks for an admin
  password fails silently (NetworkManager was hit; the `networkmanager` group
  worked around that one).
- **Hooks in:** start an agent such as `polkit_gnome` or `hyprpolkitagent`
  from mango's `exec_once` or as a home-manager service. Building one into
  amane is possible but sizeable.
- **To settle:** which agent, or build one.

### 11. Wallpaper
- **Status:** open
- **Why:** nothing sets a wallpaper under mango (`hyprpaper.conf` is a
  Hyprland leftover).
- **Hooks in:** either an amane window per monitor on the background layer
  drawing an image (`Image::cover`), which could follow the accent or be
  blurred behind the lock screen, or `swaybg` from `exec_once`.
- **To settle:** which image(s), and amane or swaybg.

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
- **No D-Bus error replies** (`Method` can only `reply`), which the Bluetooth
  agent works around by cancelling pairings instead of rejecting them.
- **No error from connect/pair/forget** in `Network` and `Bluetooth`; the shell
  makes its own calls to report failures.
- **Patches to `~/.cache/amane/library` are overwritten** by `amane compile`,
  so library fixes can't be tried locally.
