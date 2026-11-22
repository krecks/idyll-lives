#!/bin/sh
# A 30-minute memory soak: the gameplay baseline played at 1x, its per-second JS heap and
# simMemoryBytes series in bench-results/gameplay-<time>/. Pass a shorter --secs for a smoke run.
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
secs=${1:-1800}
if [ "$#" -gt 0 ]; then
    shift
fi
exec bun "$here/gameplay.mjs" --secs "$secs" "$@"
