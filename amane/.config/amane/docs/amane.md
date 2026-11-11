# amane shell: amane gotchas

Limits of the amane library the shell works around. `RECOMMENDATIONS.md` has
them written up for upstream.

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

