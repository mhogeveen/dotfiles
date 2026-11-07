#!/usr/bin/env bash
#
# runs the shell's tests with the same toolchain and libraries `amane compile`
# uses, so the build cache is shared and nothing is rebuilt from scratch
#
# the amane command is a nix wrapper: a list of exports (PATH with rustc, cargo
# and pkg-config, PKG_CONFIG_PATH with the libraries) and then one `exec` line;
# everything but that last line is sourced here
#
#   ./test.sh            every test
#   ./test.sh system::   only the tests whose path contains system::

set -euo pipefail

amane=$(command -v amane) || {
    echo "test.sh: amane isn't on PATH" >&2
    exit 1
}

wrapper=$(readlink -f "$amane")

if [[ $(tail -n 1 "$wrapper") != exec* ]]; then
    echo "test.sh: $wrapper doesn't end in exec, the wrapper's layout changed" >&2
    exit 1
fi

# shellcheck source=/dev/null
source <(sed '$d' "$wrapper")

# amane compile writes this project around the config's main.rs
project="${XDG_CACHE_HOME:-$HOME/.cache}/amane/project"

if [[ ! -f $project/Cargo.toml ]]; then
    echo "test.sh: no project at $project yet, run amane compile once first" >&2
    exit 1
fi

exec cargo test --manifest-path "$project/Cargo.toml" --bin amane-shell "$@"
