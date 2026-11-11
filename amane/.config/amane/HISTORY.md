# amane shell: history

How the shell came to be the way it is: what was built, what was decided, and
what was tried and dropped. Newest session last. `DONE.md` has the details of
each finished recommendation by number; `AGENTS.md` the current state.

## Earlier sessions (summarized)

- **Panel system** (`src/panels/mod.rs`): a registry with groups, `panel()`
  wrapper building content only while shown, `SWAP_DELAY` between replacing
  panels, the `panel` IPC, `Keyboard::OnDemand` for every panel, Escape to
  close, `?` key hints (`hints.rs`), an always-mapped backdrop that toggles
  click-through. Stacking panels was tried and dropped: mango never gives the
  keyboard back to a panel left open underneath, so Escape broke. Hence one
  group, `PANELS`.
- **Shared UI** (`ui.rs`, `list.rs`, `picker.rs`, `menu.rs`): buttons with
  keys, switches, sliders, meters, list windowing (a ScrollArea can't follow a
  selection), the `Picker` trait for search panels with a `?` help screen,
  the `Menu` trait with confirm-twice rows.
- **Panels:** launcher (frecency via `usage.rs`, `=` calculator, `>`/`>>`
  run), clipboard (cliphist, × deletes, Clear history clicked twice, preview;
  no Delete key since amane reports it as `Key::Other`), layout picker,
  screenshot and session menus, notification center and popups, network
  (Wi-Fi), bluetooth (with a pairing agent), audio (wpctl device lists),
  system (info, meters, accent swatches), calendar with weather (Open-Meteo).
- **Bar:** per-monitor workspaces with urgent dots, `[S]` layout symbol,
  clock, status icons with a VPN shield badge, the NixOS snowflake.
- **Other:** lock screen, `clock_nanosleep` minute clock, the state folder,
  the accent theme and IPC, a small JSON reader, the VPN service and
  Tailscale card. Tailscale was installed by the user (531 Mullvad exit
  nodes on the tailnet).

## Session: exit nodes to brightness

1. **Exit nodes reworked.** With 531 Mullvad exit nodes the full peer list
   (850 KB) was unworkable. Status now comes from `tailscale status --json
   --peers=false` (3 KB); the card shows only None, what `tailscale exit-node
   suggest` suggests, and the one in use if different (named with `tailscale
   whois`, cached by IP). The JSON reader's unused methods were kept with
   `#[allow(dead_code)]`.

2. **Screenshot menu over other panels.** It got no group, so open panels
   can be captured. Accepted cost: the panel underneath loses the keyboard
   when the menu or slurp closes (click it to type again). Separate
   screenshot keybinds that skip the menu were offered and declined.

3. **Calendar redesign + `ui::heading`.** From side by side to two cards in a
   column on one shared grid (gutter + 7 columns), so forecast slots line up
   under weekdays; same cell style everywhere, today/Now/Today in the accent,
   titles left with buttons right. `ui::heading` replaced four copies of the
   section heading style (system, bluetooth, audio, calendar). It takes the
   height as an argument because `.height()` on a built `Rectangle` resolved
   to the `Widget` getter. The weather icon needed a fixed-width box: Nerd
   Font glyphs draw wider than they measure.

4. **Rain chance always shown**, 0% included (it used to hide at 0).

5. **Polkit agent: researched, blocked.** `security.polkit.enable` only
   enables the daemon (it was already running); no agent runs. A built-in
   agent was planned (centered dialog, polkit 127's socket helper protocol
   worked out from polkit's source, cancel by replying without
   authenticating), but registering needs a D-Bus struct argument amane
   can't build. An upstream issue text was written (RECOMMENDATIONS upstream
   list). An interim `polkit_gnome` was declined.

6. **rust-analyzer.** "Not included in any crates" came from `Cargo.toml`
   pointing at `main.rs` through the `~/.config` symlink while Neovim opens
   the real path; fixed with the relative `path = 'src/main.rs'`. Its `cargo
   check` also fails (build scripts need amane's `pkg-config` setup, and
   amane's rustc 1.98 differs from rustup's 1.99, so rust-analyzer can't just
   run inside amane's environment). A `ra-cargo.sh` wrapper worked by hand,
   but `rust-analyzer.toml` ignored the override commands; a Neovim config
   change was declined, so the wrapper was deleted. A later revert briefly
   undid the path fix; the user restored it.

7. **Idle lock with hypridle** (`~/.nixos/home/hypridle.nix`, home-manager):
   lock at 5 min via `loginctl lock-session` (the shell's logind watcher shows
   amane's lock screen, no `lock_cmd`), screen off at 5.5 min with `wlopm`
   (not `mmsg dispatch disable_monitor`, which removes the output), suspend at
   30 min, `before_sleep_cmd` locks and waits 1 s. `inhibit_sleep = 3` needs
   Hyprland's lock-notify protocol, which mango lacks (hypridle warns about
   it, expected). Dimming skipped (needed I2C then; item 18).

8. **Status colors tried and reverted.** error/warning/ok/hint/info from the
   theme's site were added and applied around the shell (failures, no
   internet, meters, key hints, connecting states…), then reverted by the
   user. The accent stays the color for everything.

9. **Wallpaper** (item 11): background-layer window per monitor, full-width
   bottom picker from `~/Pictures/Wallpapers`, live preview on the real
   background (drawing the preview over windows via the backdrop was offered
   and declined), Escape restores, a None option that leaves the background
   see-through so mango's `root_color` shows, the blurred wallpaper on the
   lock screen. The opener is a `w` button in the system panel (a mango bind
   wasn't wanted then). The "in use" marker went from an accent dot (disliked)
   to a " · in use" label; the labels are left-aligned. A wrong guessed glyph
   (`f082f`) rendered a phone; since then glyphs are looked up.

10. **Floating window decorations: parked** (item 17). amane can't do it well
    (layer surfaces can't stack between windows; following a window means
    moving surfaces, the crash). mango has `allow_csd`, borders and shadows;
    its group/tab bars only decorate grouped windows.

11. **Recommendations split** into `RECOMMENDATIONS.md` (open, blocked,
    parked) and `DONE.md` (built, verbatim, numbers kept). New items 14–18
    were added from things noticed during the session.

12. **`layout.rs` on `json.rs`** (item 16): the hand-written scanner replaced
    by `json::parse`; a broken line is now skipped instead of read partially.

13. **Notification history across restarts** (item 14): saved to the runtime
    folder (survives restarts, not logout), newest 50, restored ones
    read-only under "Earlier", `card::Content` for live and kept cards,
    `json::quote` for writing.

14. **Volume shortcuts and popup** (item 2). The keyboard has no media keys,
    so `SUPER+=`/`-`/`0`, `SUPER+SHIFT+0` run `wpctl` directly (atomic
    relative steps, capped at 100%), with the `XF86Audio*` binds added too
    for a future remapped Fn layer. The popup follows any volume change
    (chosen over "only for the keys"), bottom center. Playback keys
    `SUPER+\`/`]`/`[` go through a new `media` IPC. During testing,
    `media play-pause` hit qutebrowser's (stopped) player; no harm, but don't
    use it to probe the no-player case.

15. **Media controls** (item 3): a "Now playing" card at the top of the audio
    panel (cover, title, artist · player, progress with times, buttons on
    `h` `p` `l`), "Nothing playing" in the same space otherwise. Covers:
    local paths decoded, links fetched once with curl. Bar indicator, lock
    screen controls and player switching were left out. Checked with a
    YouTube video by the user.

16. **Brightness** (item 19 and the user's own start). The user began a
    system panel slider on amane's `Brightness`, which only drives laptop
    backlights; this desktop has none. Rebuilt on DDC/CI with ddcutil
    (`src/brightness.rs`): detect once, read on panel open, newest-value-only
    writes. The user's 32 px backdrop margin (bar clickable while a panel is
    open) was kept as intended. Then shortcuts `SUPER+SHIFT+=`/`-` (5%) via a
    `brightness` IPC and a brightness version of the popup, only for
    shell-made changes (no DDC polling for the monitor's own buttons).

17. **Context files.** This file and `AGENTS.md` were written so a new session
    starts with the right context.

## Session: audit (2026-11-11)

1. **Audit.** Two read-only sweeps (services, panels), with the main claims
   checked in the code. One sweep reported line numbers that couldn't exist,
   so only what was verified was kept. What wasn't fixed is item 20 in
   `RECOMMENDATIONS.md`.

2. **The most important bugs fixed:**
   - **Time limits on external calls:** new `src/command.rs` (`capture`)
     kills a program that runs too long and waits on it. tailscale (5 s for
     queries, 20 s for commands) and ddcutil (5 s, 15 s for detect) use it,
     so a hang can't leave `vpn.busy` or brightness's `WRITING` set forever.
   - **Tailscale:** `v` while a login is needed opens the login page instead
     of running `tailscale up`, which waited for the browser login with no
     end and left every Tailscale button dead. `up` also gets `--timeout=15s`.
   - **Weather:** a failed fetch is tried again after 1 min, not 30 (the time
     of failure is kept apart from the last good fetch; the check is the pure
     `due()`, tested). `forced` is cleared when a fetch starts, so a refresh
     during a fetch isn't lost.
   - **Covers:** failures are kept with their time and retried after 1 min,
     not remembered as "no cover" for the session; curl writes to `.part`
     and renames, so a cut-off download isn't taken for a cover.
   - **Popups:** amane tracks hover by a card's place in the list, sending
     the leave before the next enter. Any leave now ends the hover, whichever
     id it carries, and `listen()` drops a hover whose card can't be drawn
     (a corner panel open, the notification gone). Before, a popup could
     stay on screen for good.
   - **Wallpaper picker:** with the picture in use not in the list, nothing
     is selected and Enter only closes (it used to apply None); `r` resets
     the selection and preview so they can't disagree.
   - **Lock screen:** an empty Enter is ignored (it cost a PAM attempt and
     said "Wrong password").

   Checked live: brightness up/down through the new helper (put back at
   50%), a popup timing out with a corner panel opened and closed in
   between, the picker opening on the picture in use. The hover itself, the
   Tailscale login case and the lock screen weren't exercised (no pointer
   control, Tailscale in use, locking needs asking).

## Habits worth keeping

- Verify before claiming: check the code, the running shell (`pgrep`, its
  start time vs. the binary's), and screenshots. A rebuilt binary isn't
  running until `amane restart`.
- Screenshots: physical coordinates, fresh file names, wait before reading.
- Put the machine back: volume, mute, brightness, wallpaper choice.
- The user edits and commits between sessions; read `git log`/`git status`
  first, and fix warnings their changes leave (say so).
