#!/usr/bin/env bash
# Runs inside the container (tools/m1/run.sh calls it in `m3` and in `m1`): one core's speed, best
# of RUNS (default 3), so a run that landed on an efficiency core or beside a busy neighbour is
# thrown out. Two measures:
# - 7-Zip's single-threaded benchmark (`7z b -mmt1`), its total rating (MIPS);
# - the sim's tickbench (`crates/sim-core/examples/tickbench.rs`: the starter lot over a game day),
#   its time a tick with socials (µs).
# Prints `calibrate: 7z <MIPS> tick_us <µs>`; the quota's ratio is m1's over m3's.
set -euo pipefail
runs=${RUNS:-3}
cd /repo
cargo build --release --locked -p sim-core --example tickbench >&2
bench=$CARGO_TARGET_DIR/release/examples/tickbench
best7z=0
best_us=
for _ in $(seq 1 "$runs"); do
  # (7-Zip's last line: "Tot:  <usage> <R/U MIPS> <rating MIPS>".)
  mips=$(7z b -mmt1 | awk '/^Tot:/ { print $NF }')
  [ "${mips:-0}" -gt "$best7z" ] && best7z=$mips
  # ("with socials: 123.456µs/tick, ..." — in µs whatever unit Debug printed.)
  us=$("$bench" | awk -F'[:,]' '/^with socials/ {
    t = $2; gsub(/ /, "", t); sub(/\/tick/, "", t)
    if (t ~ /ns$/) { sub(/ns$/, "", t); print t / 1000 }
    else if (t ~ /µs$/) { sub(/µs$/, "", t); print t }
    else if (t ~ /ms$/) { sub(/ms$/, "", t); print t * 1000 }
  }')
  if [ -z "$best_us" ] || awk -v a="$us" -v b="$best_us" 'BEGIN { exit !(a < b) }'; then best_us=$us; fi
done
echo "calibrate: 7z $best7z tick_us $best_us"
