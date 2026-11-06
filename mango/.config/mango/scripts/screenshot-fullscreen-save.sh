#!/usr/bin/env sh
grim $HOME/Pictures/Screenshots/$(date +%Y%m%d%H%M%S).png
notify-send -u low -a Screenshots "Screenshots" "Fullscreen saved"
