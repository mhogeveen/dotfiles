#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Toggle
# @raycast.mode silent

# Optional parameters:
# @raycast.icon ./cliamp.png
# @raycast.packageName cliamp

# Documentation:
# @raycast.author mhogeveen
# @raycast.authorURL https://raycast.com/mhogeveen

cliamp toggle
state=$(cliamp status --json | jq .state)
echo Now $state
