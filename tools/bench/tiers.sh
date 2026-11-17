#!/usr/bin/env bash
# The sim's speed on the three city worlds (seaside, river, forest at town=city, villages=many) at
# each fill tier (`web/src/game/people.ts` FillTier), over a game day: the big-world benchmark
# (`crates/sim-core/tests/big_world.rs`) run on each, best of ROUNDS, and a table of it.
#
#   tools/bench/tiers.sh [label[=binary] ...]
#
# Each label is a build of the benchmark: `label` alone builds this checkout's and keeps a copy
# (bench-results/tiers/bins/<label>), so a later change doesn't move it; `label=path` runs a
# benchmark binary already built. With several, the rounds take them in turn, so a machine that
# gets busier halfway slows them alike. Default: one label, `now`.
#
# Environment:
#   TIERS="full half near"  the tiers to run
#   WORLDS_AT="0 1 2"       which of each tier's worlds (0 seaside, 1 river, 2 forest)
#   HOURS=24                game hours a run
#   ROUNDS=2                runs of each, the best kept (the machine is shared: the fastest run is
#                           the one least slowed by others)
#   WATCH=                  empty: the view on the player's lot, as the bench always ran; a side in
#                           tiles (320, 780): the camera's square at speed 5, as the game runs
#   FEATURES=               cargo features for the build (`profile`: where the time went, slower)
#   PARALLEL=1              runs at once (the machine is shared with others: keep it low)
#
# The worlds are made once into bench-results/tiers/tier-<tier>.json (`tools/worlds/sample.ts
# --tier`); delete them to make them again (after a change to the generator). Each run's log goes
# to bench-results/tiers/<label>/, the table to bench-results/tiers/<labels>.md.
set -euo pipefail
cd "$(dirname "$0")/../.."
root=$(pwd)
out=$root/bench-results/tiers
mkdir -p "$out/bins"

tiers=${TIERS:-full half near}
at=${WORLDS_AT:-0 1 2}
hours=${HOURS:-24}
rounds=${ROUNDS:-2}
watch=${WATCH:-}
parallel=${PARALLEL:-1}
[ $# -eq 0 ] && set -- now
labels=("$@")

for t in $tiers; do
  if [ ! -f "$out/tier-$t.json" ]; then
    echo "making the $t worlds"
    bun tools/worlds/sample.ts "$out/tier-$t.json" 1 --kinds seaside,river,forest --options town=city,villages=many --no-timing --tier "$t" > "$out/tier-$t.log"
  fi
done

names=()
bins=()
for l in "${labels[@]}"; do
  name=${l%%=*}
  if [[ $l == *=* ]]; then
    bin=${l#*=}
  else
    bin=$out/bins/$name
    echo "building the benchmark as $name"
    built=$(cargo test --release --locked -p sim-core --test big_world ${FEATURES:+--features $FEATURES} --no-run --message-format=json 2>/dev/null |
      grep -o '"executable":"[^"]*big_world[^"]*"' | tail -1 | cut -d'"' -f4)
    [ -n "$built" ] || { echo "the benchmark didn't build" >&2; exit 1; }
    cp "$built" "$bin"
  fi
  names+=("$name")
  bins+=("$bin")
  mkdir -p "$out/$name"
done

suffix=${watch:+-w$watch}
jobs=0
for round in $(seq 1 "$rounds"); do
  for i in "${!names[@]}"; do
    for t in $tiers; do
      for k in $at; do
        log=$out/${names[$i]}/$t$k$suffix-r$round.log
        (
          cd crates/sim-core
          WORLDS=$out/tier-$t.json WORLD=$k HOURS=$hours WATCH=$watch "${bins[$i]}" --ignored --nocapture big_world_benchmark > "$log" 2>&1 || echo "failed: $log" >&2
        ) &
        jobs=$((jobs + 1))
        # (macOS's bash has no `wait -n`: a batch at a time.)
        if [ "$jobs" -ge "$parallel" ]; then
          wait
          jobs=0
        fi
      done
    done
  done
done
wait

# The table: per tier and world, each label's best run (its ticks/s; the slowest hour's and its
# hour; with WATCH, a step's p99 and how many were at full detail) and its save hash.
table=$out/$(IFS=-; echo "${names[*]}")$suffix.md
kinds=(seaside river forest)
{
  echo "# Sim ticks/s, ${hours} game hours, best of $rounds${watch:+, WATCH=$watch at speed 5}"
  echo
  printf '| tier | world | residents |'
  for n in "${names[@]}"; do printf ' %s ticks/s | slowest hour |%s hash |' "$n" "${watch:+ step p99 ms | full detail |}"; done
  echo
  printf '|---|---|---:|'
  for _ in "${names[@]}"; do printf -- '---:|---:|%s---|' "${watch:+---:|---:|}"; done
  echo
  for t in $tiers; do
    for k in $at; do
      first=$(ls "$out/${names[0]}/$t$k$suffix"-r*.log 2>/dev/null | head -1)
      residents=$(grep -o '[0-9]* residents, grid' "$first" 2>/dev/null | cut -d' ' -f1)
      printf '| %s | %s | %s |' "$t" "${kinds[$k]}" "${residents:-?}"
      for n in "${names[@]}"; do
        best=$(for f in "$out/$n/$t$k$suffix"-r*.log; do grep -h '^bench: rate' "$f" 2>/dev/null; done | sort -k3 -n -r | head -1)
        hashes=$(for f in "$out/$n/$t$k$suffix"-r*.log; do grep -h '^bench: hash' "$f" 2>/dev/null | cut -d' ' -f3; done | sort -u | paste -sd/ -)
        if [ -z "$best" ]; then
          printf ' failed | | %s|' "${watch:+| | }"
          continue
        fi
        set -- $best
        # bench: rate R worst_hour W at H step_p99 P full F full_most M
        printf ' %s | %s (%s:00) |' "$3" "$5" "$7"
        [ -n "$watch" ] && printf ' %s | %s (%s) |' "$9" "${11}" "${13}"
        printf ' %s |' "${hashes:-?}"
      done
      echo
    done
  done
} > "$table"
cat "$table"
echo "→ $table"
