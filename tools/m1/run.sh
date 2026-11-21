#!/usr/bin/env bash
# The M1 estimate, in one command (tools/m1/README.md): the sim's speed on the three city worlds at
# each fill tier, measured on this machine natively, in the container, and in the container under
# the M1's CPU quota, and as wasm under Node and Bun; then each scaled to an M1 and set against
# the 600 ticks a second 10× needs (the fastest fixed speed; super speed runs as fast as the machine can). The report goes to
# bench-results/m1/<time>/report.md.
#
# Environment (and tools/bench/tiers.sh's): WATCH=320 (the camera's square, as the game runs it),
# HOURS=24, ROUNDS=1 (best of), PARALLEL=3 (runs at once: the machine may be shared), TIERS,
# WORLDS_AT, M1_QUOTA=7200 (of a 10,000 µs period), SKIP="native wasm container quota" (any of
# them, to reuse the last run's numbers: `container` is the build, the calibration and the runs in
# `m3`, `quota` the runs in `m1`).
set -euo pipefail
cd "$(dirname "$0")/../.."
root=$(pwd)
export WATCH=${WATCH:-320} HOURS=${HOURS:-24} ROUNDS=${ROUNDS:-1} PARALLEL=${PARALLEL:-3}
export TIERS=${TIERS:-full half near} WORLDS_AT=${WORLDS_AT:-0 1 2}
skip=" ${SKIP:-} "
stamp=$(date +%Y%m%d-%H%M%S)
out=$root/bench-results/m1/$stamp
mkdir -p "$out"
compose=(docker compose -f tools/m1/compose.yml)

# The worlds (tiers.sh makes any missing) and this machine's native numbers.
if [[ $skip != *" native "* ]]; then
  tools/bench/tiers.sh m1-native | tee "$out/native.log"
fi

# The wasm, built as the game ships it, under Node and Bun (Bun first: it merges the content Node reads).
if [[ $skip != *" wasm "* ]]; then
  (cd web && bun run wasm) > "$out/wasm-build.log" 2>&1
  : > "$out/wasm.log"
  # (Bun first, alone: it writes the merged content Node reads. Then PARALLEL at a time.)
  bun tools/bench/simtick.mjs "bench-results/tiers/tier-$(set -- $TIERS; echo "$1").json" --hours 0 > /dev/null 2>&1 || true
  jobs=0
  for engine in bun node; do
    for t in $TIERS; do
      for k in $WORLDS_AT; do
        for r in $(seq 1 "$ROUNDS"); do
          (
            line=$($engine tools/bench/simtick.mjs "bench-results/tiers/tier-$t.json" --world "$k" --hours "$HOURS" ${WATCH:+--watch $WATCH} 2>>"$out/wasm.err")
            echo "$t $k $line" | tee -a "$out/wasm.log"
          ) &
          jobs=$((jobs + 1))
          if [ "$jobs" -ge "$PARALLEL" ]; then
            wait
            jobs=0
          fi
        done
      done
    done
  done
  wait
fi

# The container: its build and calibration (best of 3, with and without the quota), then the same
# runs in it and under the quota (the benchmark built once, in `m3`).
if [[ $skip != *" container "* ]]; then
  "${compose[@]}" build m3 > "$out/docker-build.log" 2>&1
  "${compose[@]}" run --rm m3 tools/m1/calibrate.sh 2>"$out/calibrate-m3.err" | tee "$out/calibrate-m3.log"
  "${compose[@]}" run --rm m1 tools/m1/calibrate.sh 2>"$out/calibrate-m1.err" | tee "$out/calibrate-m1.log"
  "${compose[@]}" run --rm -e WATCH -e HOURS -e ROUNDS -e PARALLEL -e TIERS -e WORLDS_AT m3 tools/bench/tiers.sh m1-ctr-m3 | tee "$out/ctr-m3.log"
fi
# (One run at a time under the quota: it is the whole container's, so runs side by side would
# share the one core's 72 %.)
if [[ $skip != *" quota "* ]]; then
  "${compose[@]}" run --rm -e WATCH -e HOURS -e ROUNDS -e PARALLEL=1 -e TIERS -e WORLDS_AT m1 tools/bench/tiers.sh m1-ctr-m1=/repo/bench-results/tiers/bins/m1-ctr-m3 | tee "$out/ctr-m1.log"
  "${compose[@]}" run --rm m1 sh -c 'echo "memory limit $(cat /sys/fs/cgroup/memory.max) bytes, cpu.max $(cat /sys/fs/cgroup/cpu.max), $(nproc) cpus"' > "$out/limits.log" 2>/dev/null || true
fi

cargo m1-table "$out" > "$out/report.md"
cat "$out/report.md"
echo "→ $out/report.md"
