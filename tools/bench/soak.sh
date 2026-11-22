#!/bin/sh
# A 30-minute memory soak: the gameplay baseline played once at 1x, its per-second JS heap and
# simMemoryBytes series and collected reads spread over the half hour in
# bench-results/gameplay-<time>/. Flags are passed through and win over the ones here
# (gameplay.mjs reads the first), so `--secs 150 --reads 30,140` is a smoke run.
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec bun "$here/gameplay.mjs" "$@" --secs 1800 --runs 1 --reads 60,600,1200,1740
