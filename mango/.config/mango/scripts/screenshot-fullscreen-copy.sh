#!/usr/bin/env sh
grim - | wl-copy
noctalia msg notification-show '{"app_name":"","summary":"Screenshots","body":"Fullscreen copied to clipboard","urgency":"low","timeout_ms":1000,"icon":"circle-check"}'
