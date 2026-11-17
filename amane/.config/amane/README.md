# amane shell

A desktop shell for the [MangoWM](https://github.com/mangowm/mango) Wayland
compositor, written in Rust on the [amane](https://github.com/MystiaFin/amane)
library. It provides the bar, launcher, clipboard history, notifications,
audio, network, Bluetooth and VPN panels, a calendar with weather,
wallpapers and desktop widgets, screenshots and screen recording, a session
menu, a lock screen, and on-screen popups for volume and brightness.

This README covers installing it on a new machine and the per-machine
settings. For how the code is organised, see `AGENTS.md` and `docs/`.

## Dependencies

### Required

| What | Why |
| --- | --- |
| MangoWM | The compositor. The shell reads its state with `mmsg`, and the keybind sheet reads `~/.config/mango/config.conf`. |
| amane (CLI + library) | Builds and runs the shell (`amane compile`, `amane run`, `amane ipc`). |
| Rust toolchain (`rustup`, stable) | `amane compile` builds `src/` with cargo. The upstream amane package bundles cargo and rustc; the NixOS setup below re-wraps it to use rustup instead. |
| Build libraries | fontconfig, freetype, expat, wayland, libxkbcommon, vulkan-loader, libpulseaudio, linux-pam (found through `pkg-config`; a C compiler too). |
| A Vulkan-capable GPU | amane draws with wgpu. |
| BlexMono Nerd Font | The only font; layout assumes its monospaced width (`nerd-fonts.blex-mono`). |
| systemd + logind | Suspend, reboot and power off (`systemctl`), lock on suspend and `loginctl lock-session`, failed-unit warnings. |
| PAM | The lock screen checks your password through PAM. |
| D-Bus session bus | The shell is the notification daemon and talks to MPRIS, BlueZ and NetworkManager. **Don't run another notification daemon** (mako, dunst, swaync). |
| coreutils, iproute2 | `df` for disk use, `ip route` for VPN detection. |

### Per feature

The shell starts without these; the feature that needs one shows an
error or stays empty.

| Feature | Needs |
| --- | --- |
| Audio panel, volume popup, privacy dot | PipeWire + WirePlumber (`wpctl`, `pw-dump`, `pw-mon`) |
| Clipboard history | `cliphist`, `wl-clipboard` (`wl-copy`, `wl-paste`) |
| Screenshots | `grim`, `slurp`, and `satty` to annotate |
| Screen recording | `wf-recorder`; VA-API drivers for GPU encoding (falls back to software `libx264`) |
| Screen sharing in the privacy dot | `xdg-desktop-portal-wlr` |
| Network panel | NetworkManager |
| Bluetooth panel | BlueZ (`bluetoothd`), `rfkill` |
| VPN card | `tailscale`, and `xdg-open` for its login page |
| Brightness on external monitors | `ddcutil` and i2c access (`hardware.i2c.enable`); laptops use the backlight with nothing extra |
| Idle switch | a `hypridle.service` user unit |
| Night light switch | a `wlsunset.service` user unit |
| Album art, weather | `curl` (weather from Open-Meteo, no account needed) |
| Launcher `>>command` | a terminal, `foot` by default |
| Notifications from scripts | `libnotify` (`notify-send`), optional |

You can change the program and unit names; see [Per-machine settings](#per-machine-settings).

## Installing on NixOS (flakes + home-manager)

This is how the shell is set up on `prometheus`; the host config is in
`~/.nixos/hosts/prometheus/default.nix` and home-manager in `~/.nixos/home/`.

1. **Flake inputs.** Add `amane` (`github:MystiaFin/amane`) and `mangowm`
   (`github:mangowm/mango`), and import `mangowm.nixosModules.mango` and the
   home-manager module.

2. **System packages and services** (host config):
   ```nix
   programs.mango.enable = true;
   networking.networkmanager.enable = true;
   hardware.bluetooth.enable = true;
   hardware.i2c.enable = true;          # DDC/CI brightness on external monitors
   services.tailscale.enable = true;    # optional, for the VPN card
   fonts.packages = [ pkgs.nerd-fonts.blex-mono ];
   environment.systemPackages = [
     pkgs.wl-clipboard
     pkgs.ddcutil
     inputs.amane.packages.${pkgs.stdenv.hostPlatform.system}.default
   ];
   ```
   On `prometheus` the amane package is re-wrapped without its bundled
   cargo/rustc (about 1.6 GiB) so that it uses rustup and the libraries
   listed above. Copy that `overrideAttrs` block from the host config if you
   want the same.

3. **User packages** (home-manager): `rustup`, `cliphist`, `libnotify`,
   `grim`, `slurp`, `wf-recorder`, plus `programs.foot`, satty
   (`home/satty.nix`), hypridle (`home/hypridle.nix`) and wlsunset
   (`home/wlsunset.nix`) if you want those features.

4. **Link the config.** `~/.config/amane` must point at this folder:
   ```nix
   xdg.configFile."amane".source =
     config.lib.file.mkOutOfStoreSymlink "${flakeDir}/config/amane/.config/amane";
   ```

5. **Start it with Mango and add the binds.** In `home/mango.nix`:
   `autostart_sh = "exec amane run";`, plus the binds. The panels open over
   IPC, for example:
   ```
   bind=SUPER,space,spawn_shell,amane ipc call panel toggle launcher
   bind=SUPER,p,spawn_shell,amane ipc call record menu
   bindl=SUPER+SHIFT,equal,spawn,amane ipc call brightness up
   bindl=SUPER,backslash,spawn,amane ipc call media play-pause
   ```
   The full set is listed in `docs/machine.md`. Keep a short `#` comment
   above each group of binds: the keybind sheet (`SUPER+/`) uses it as the
   heading.

6. `nixos-rebuild switch`, then `rustup default stable` once.

7. Build, then [check the host](#check-the-host).

## Installing elsewhere

The same pieces without Nix: install MangoWM and amane from their
repositories, then the packages in the tables above. Then:

- clone or copy this folder to `~/.config/amane` (or run `amane target <folder>`)
- add `exec amane run` to Mango's autostart and the binds to its config
- give your user access to the i2c devices for DDC/CI (the `i2c` group, or a udev rule)
- build, then [check the host](#check-the-host)

## Building and running

```sh
amane compile      # builds src/ into ~/.cache/amane (the first build takes a while)
amane run          # or log in again so Mango's autostart starts it
amane restart      # after each compile
./test.sh          # the tests, with the same toolchain amane uses
```

`amane compile` must finish with zero warnings. The source may only use
`std`, `amane` and libc: you can't add crates.

## Check the host

After the first `amane compile`, ask the shell what this machine is missing:

```sh
~/.cache/amane/amane-shell doctor
```

It runs instead of the shell, so it works before the shell has ever started
and doesn't disturb a running one. It changes nothing. It checks:

- the programs in the dependency tables, including the terminal, annotator
  and recorder from this host's settings
- the idle and night-light units, the GPU render node and the font
- brightness access (a backlight, or `/dev/i2c-*`)
- that no other notification daemon holds the name
- NetworkManager, BlueZ, and the Tailscale operator

Each line says what's missing, the feature it breaks, and how to fix it:

```
features
  missing  satty (annotator)            screenshot annotation · install satty, or set annotator in src/settings/<host>.rs
  missing  wlsunset.service             night light switch · no such user unit; add it, or set night_light_unit in src/settings/<host>.rs
2 missing (0 required)
```

It exits with 1 when something under `required` is missing, so the shell
can't work. A missing feature only disables that panel or key.

## Per-machine settings

The values that differ between machines are compiled in, so a mistake fails
`amane compile` instead of showing up at run time:

- `Settings::default()` in `src/settings.rs` applies to every host.
- `src/settings/<hostname>.rs` changes only what one host needs. The shell
  picks it by `/proc/sys/kernel/hostname` at start; a host without a file
  gets the defaults.

| Setting | Default | Meaning |
| --- | --- | --- |
| `terminal` | `"foot"` | Runs the launcher's `>>command`; words after the program come before `sh -c ...`, so `"alacritty -e"` works |
| `annotator` | `"satty"` | Gets the screenshot on stdin with `--filename -` |
| `recorder` | `"wf-recorder"` | Screen recordings (wf-recorder's flags) |
| `idle_unit` | `"hypridle.service"` | User unit behind the system panel's Idle switch |
| `night_light_unit` | `"wlsunset.service"` | User unit behind the Night light switch |
| `render_node` | `RenderNode::Auto` | GPU for VA-API encoding: `Auto` (lowest `/dev/dri/renderD*`), `Path("/dev/dri/renderD129")`, or `None` for software |
| `cpu_drivers` | `k10temp zenpower coretemp` | hwmon drivers read for the CPU temperature |
| `gpu_drivers` | `amdgpu nouveau radeon i915 xe` | hwmon drivers read for the GPU temperature |
| `wallpapers` | `~/Pictures/Wallpapers` | The wallpaper picker's folder |
| `screenshots` | `~/Pictures/Screenshots` | Saved screenshots; also set satty's `output-filename` to this folder |
| `screencasts` | `~/Videos/Screencasts` | Saved recordings |
| `weather` | Hilversum (52.223, 5.176) | The forecast's place until you set one over IPC |

To add a host called `atlas`:

1. Create `src/settings/atlas.rs`:
   ```rust
   /*
    * atlas, the laptop: an intel gpu and alacritty
    */

   use super::{RenderNode, Settings};

   pub fn settings() -> Settings {
       Settings {
           terminal: "alacritty -e",
           render_node: RenderNode::Path("/dev/dri/renderD128"),
           ..Settings::default()
       }
   }
   ```
2. In `src/settings.rs`, add `mod atlas;` next to `mod prometheus;`, and an
   arm in `for_host()`: `"atlas" => atlas::settings(),`.
3. Run `amane compile` and `amane restart`.

## After installing

- **Wallpapers:** put pictures in the wallpapers folder, then open the
  picker with `SUPER+w`.
- **Weather place:** `amane ipc call weather location <latitude> <longitude> <name>`.
  It's kept in `~/.local/state/amane-shell/weather`.
- **Tailscale:** run `sudo tailscale set --operator=$USER` once, so the VPN
  card can connect, disconnect and pick exit nodes.
- **Monitors and scaling:** panels have fixed sizes, chosen for a
  2560×1440 logical screen. 1920×1080 fits; a small laptop screen at a
  high scale can be cramped. Check with `mmsg get all-monitors`.
- **Accent, animations, desktop design:** in the system panel and the
  wallpaper picker. They're kept in `~/.local/state/amane-shell/`.
- **Machine docs:** update `docs/machine.md` and the screenshot coordinates
  in `AGENTS.md` for the new monitor and keyboard.

## Troubleshooting

- **Something doesn't work on a new machine:** run
  `~/.cache/amane/amane-shell doctor` first.

- **The bar is gone but `amane-shell` is still running:** the UI can vanish
  while the process lives. Check `mmsg get all-layers` for `amane-*`
  surfaces, then run `amane restart`.
- **The shell prints nothing:** its stderr goes to `/dev/null` when Mango
  starts it. Run `amane run` from a terminal to see its output.
- **The screen freezes or the GPU resets:** the shell avoids amane's
  `Canvas` and `.rotate()`, because vello's compute shaders hang some AMD
  GPUs. Check `journalctl -k | grep -i "gfx timeout"` (see `docs/amane.md`).
- **No notifications:** check that no other notification daemon is running.
- **Recording falls back to software:** the VA-API drivers are missing, or
  `render_node` points at the wrong GPU.
