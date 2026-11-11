# amane shell: machine facts

The machine the shell runs on (host `prometheus`).

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

