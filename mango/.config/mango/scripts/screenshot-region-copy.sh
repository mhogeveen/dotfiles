#!/usr/bin/env sh
grim -g "$(slurp -d)" - | wl-copy
noctalia msg notification-show '{"app_name":"","summary":"Screenshots","body":"Region copied to clipboard","urgency":"low","timeout_ms":1000,"icon":"circle-check"}'
