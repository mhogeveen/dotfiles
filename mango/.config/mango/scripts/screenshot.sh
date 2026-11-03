#!/usr/bin/env sh

choice=$(printf '%s\n' \
  "Fullscreen (copy)" \
  "Fullscreen (save)" \
  "Region (copy)" \
  "Region (save)" \
  | fuzzel --dmenu --prompt "Screenshot: ")

# Exit if the user pressed Esc (empty output)
[ -z "$choice" ] && exit 0

case "$choice" in
  "Fullscreen (copy)") "$HOME/.config/mango/scripts/screenshot-fullscreen-copy.sh" ;;
  "Fullscreen (save)") "$HOME/.config/mango/scripts/screenshot-fullscreen-save.sh" ;;
  "Region (copy)") "$HOME/.config/mango/scripts/screenshot-region-copy.sh" ;;
  "Region (save)") "$HOME/.config/mango/scripts/screenshot-region-save.sh" ;;
esac
