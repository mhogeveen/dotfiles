#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Previous
# @raycast.mode silent

# Optional parameters:
# @raycast.icon ./cliamp.png
# @raycast.packageName cliamp

# Documentation:
# @raycast.author mhogeveen
# @raycast.authorURL https://raycast.com/mhogeveen

cliamp prev
status=$(cliamp status --json)
track_title=$(echo $status | jq .track.title)
track_artist=$(echo $status | jq .track.artist)
echo Playing: $track_title by $track_artist
