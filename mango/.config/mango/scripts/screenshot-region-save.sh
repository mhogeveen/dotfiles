#!/usr/bin/env sh
grim -g "$(slurp -d)" $HOME/Pictures/Screenshots/$(date +%Y%m%d%H%M%S).png
noctalia msg notification-show '{"app_name":"","summary":"Screenshots","body":"Region saved","urgency":"low","timeout_ms":1000,"icon":"circle-check"}'
