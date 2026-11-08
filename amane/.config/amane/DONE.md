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
