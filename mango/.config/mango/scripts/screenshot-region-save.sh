#!/usr/bin/env sh
grim -g "$(slurp -d)" $HOME/Pictures/Screenshots/$(date +%Y%m%d%H%M%S).png
notify-send -u low -a Screenshots "Screenshots" "Region saved"
