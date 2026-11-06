#!/usr/bin/env sh
grim -g "$(slurp -d)" - | wl-copy
notify-send -u low -a Screenshots "Screenshots" "Region copied to clipboard"
