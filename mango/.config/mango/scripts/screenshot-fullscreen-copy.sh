#!/usr/bin/env sh
grim - | wl-copy
notify-send -u low -a Screenshots "Screenshots" "Fullscreen copied to clipboard"
