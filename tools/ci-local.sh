#!/usr/bin/env bash
# The CI jobs' steps, in one place: .github/workflows/ci.yml's four jobs run each as
# `tools/ci-local.sh <step>`, and run here with no step they all go, before a push or a pull request,
# so what goes up arrives green.
#
# The Bun steps run on the Bun CI pins (ci.yml's `bun-version`), fetched with bunx where the one
# installed differs: another Bun can pass what CI's fails.
#
#   tools/ci-local.sh            every step, and `types`, the slow tests too (about 20 minutes)
#   tools/ci-local.sh --quick    pins fmt clippy sim clips web chunks types
#   tools/ci-local.sh web sim    those steps
#   SEED_BASE=3 tools/ci-local.sh generated
#                                the generated-world checks over other sample worlds (default 0:
#                                CI's). The same seed base makes the same worlds, so a failure
#                                replays; the release gate runs `generated` over five seed bases.
#   SLOW_TESTS=1 tools/ci-local.sh sim web
#                                with the slow tests: the simulation's long runs (`slow-tests`
#                                feature) and the renderer's town-building files (`*.slow.test.ts`).
#                                A pull request's CI leaves them out; a tag's and a run by hand
#                                have them, as does this script run with no step.
#
# Steps: pins fmt clippy sim generated clips web tools chunks sizes uses models built art, and types (the web's types
# and lint over fresh wasm builds of the sim, the sounds and the voice: not in CI's job, which builds no wasm, so run
# only here).
set -euo pipefail
cd "$(dirname "$0")/.."

all=(pins fmt clippy sim generated clips web tools chunks sizes uses models built art types)
quick=(pins fmt clippy sim clips web chunks types)

pinned=$(sed -nE 's/^ *bun-version: *([0-9.]+).*/\1/p' .github/workflows/ci.yml | head -1)
[[ -n "$pinned" ]] || { echo "No bun-version in .github/workflows/ci.yml" >&2; exit 1; }
bun_ci=(bun)
# (CI runs the Rust steps before it sets Bun up.)
if command -v bun >/dev/null && [[ "$(bun --version)" != "$pinned" ]]; then
  echo "Bun $(bun --version) here, $pinned in CI: the Bun steps run on $pinned (bunx)."
  bun_ci=(bunx "bun@$pinned")
fi

step() {
  printf '\n== %s\n' "$1"
  case "$1" in
    # Accepted parts held by their sha256 (tools/pins/pins.json).
    pins) cargo pins ;;
    fmt) cargo fmt --all --check ;;
    clippy) cargo clippy --workspace --all-targets --locked -- -D warnings ;;
    # (The long runs only with SLOW_TESTS: `#[cfg_attr(not(feature = "slow-tests"), ignore)]`.) Then
    # tickbench, which `cargo test` only compiles, run once (the tests' build, so its timings are rough).
    sim)
      cargo test -p sim-core --locked ${SLOW_TESTS:+--features slow-tests}
      cargo run -p sim-core --example tickbench --locked ${SLOW_TESTS:+--features slow-tests}
      ;;
    # The ignored generated_* tests need sample files made first. In the simulation tests' own build
    # (already optimised): a release build of the whole test binary would cost more than it saves.
    generated)
      local out
      out=$(mktemp -d)
      # (Gone even when a test fails, which ends the script; the path taken now, as `out` is this
      # step's own.)
      trap "rm -rf '$out'" EXIT
      # (`--no-timing`: their time budgets were measured on an M3 Pro, so a slower or busy machine
      # doesn't fail the gate; the timings are still printed.)
      "${bun_ci[@]}" tools/housegen/sample.ts "$out/houses.json" --no-timing
      # Worlds three per kind (every one Big), the same seed base making the same worlds.
      local base=${SEED_BASE:-0}
      echo "Sample worlds of seed base $base."
      "${bun_ci[@]}" tools/worlds/sample.ts "$out/worlds.json" 3 --seed-base "$base" --no-timing
      if ! HOUSES="$out/houses.json" WORLDS="$out/worlds.json" cargo test -p sim-core --locked --test sim -- --ignored generated_houses:: generated_worlds::; then
        echo "The generated-world checks failed on seed base $base (again: SEED_BASE=$base tools/ci-local.sh generated)." >&2
        exit 1
      fi
      rm -rf "$out"
      ;;
    # The residents' clips on both bodies at every life stage (known failures: tools/characters/clip-baseline.json).
    clips) (cd web && "${bun_ci[@]}" run check:clips) ;;
    # The renderer's engine-neutral tests.
    # (The town-building files, `*.slow.test.ts`, only with SLOW_TESTS.)
    # (The tests that make sounds load the sounds' wasm: `bun run sound` builds it.)
    web) (cd web && "${bun_ci[@]}" install --frozen-lockfile && "${bun_ci[@]}" run sound && "${bun_ci[@]}" run test && if [[ -n "${SLOW_TESTS:-}" ]]; then "${bun_ci[@]}" run test:slow; fi) ;;
    # The character build, the object gate, the models' kernel, runner and families, and the content and
    # texture tools, in the build the aliases above and below run (Cargo.toml's `tools` profile).
    tools) cargo test -p ll-tools -p sound -p artkit -p ll-art -p art-nature -p art-wet -p art-rooms -p art-packs -p art-outside -p art-things --profile tools --locked ;;
    # The PlayCanvas shader chunks' anchors and private names the renderer relies on (tools/render/chunks.json),
    # in the release build the game imports: its package, then the tools' check.
    chunks) (cd web && "${bun_ci[@]}" install --frozen-lockfile) && cargo check-chunks ;;
    # Every shipped model against the reference's kinds (what is outside held by tools/art/gate-baseline.json);
    # every use the renderer can play; every model at the gate, with the Khronos glTF validator (web/'s package,
    # about 10 s).
    sizes) cargo review sizes --check ;;
    uses) cargo review uses --check ;;
    models) (cd web && "${bun_ci[@]}" install --frozen-lockfile) && cargo check-glb --validate ;;
    # What the game builds as it runs (houses near and far, the lots' buildings closed, a street, fences, cars) at the models'
    # gate: a fixed set of samples dumped by the game's own generators (about 2 s; no browser, no wasm, no
    # packages), then held to the same rules, what they find today by tools/art/gate-baseline.json (about 12 s).
    built) "${bun_ci[@]}" tools/built/dump.ts target/built && cargo check-glb --built target/built ;;
    # Every family of item models built again in memory and held to the committed files, packs and centres, byte
    # for byte (crates/art; about 10 s on twelve cores on top of the tools' build): a model changed in code and not rebuilt, or a
    # build that gives other bytes on another machine, fails here.
    art) cargo art --all --check ;;
    types) (cd web && bun run wasm && bun run sound && bun run voice:wasm && "${bun_ci[@]}" run check && "${bun_ci[@]}" run lint) ;;
    *)
      echo "unknown step: $1 (steps: ${all[*]})" >&2
      return 2
      ;;
  esac
}

if [[ $# -eq 0 ]]; then
  set -- "${all[@]}"
  export SLOW_TESTS=${SLOW_TESTS-1}
elif [[ "$1" == "--quick" ]]; then
  set -- "${quick[@]}"
fi
for s in "$@"; do step "$s"; done
[[ $# -gt 1 ]] && printf '\n== All green (%s)\n' "$*"
exit 0
