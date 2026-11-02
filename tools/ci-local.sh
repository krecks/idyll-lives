#!/usr/bin/env bash
# Every step of the CI `test` job (.github/workflows/ci.yml), run here before a push or a pull
# request, so what goes up arrives green. Keep it in step with ci.yml when a step is added there.
#
# The web steps run on the Bun that CI pins (its `bun-version`), fetched with bunx when the one
# installed differs: another Bun can pass what CI's fails.
#
#   tools/ci-local.sh          everything (about 20 minutes)
#   tools/ci-local.sh --quick  without the generated worlds, the tools' tests and the model checks
set -euo pipefail
cd "$(dirname "$0")/.."

quick=false
[[ "${1:-}" == "--quick" ]] && quick=true
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT

step() { printf '\n== %s\n' "$*"; }

pinned=$(sed -nE 's/^ *bun-version: *([0-9.]+).*/\1/p' .github/workflows/ci.yml | head -1)
bun_ci=(bun)
if [[ "$(bun --version)" != "$pinned" ]]; then
  echo "Bun $(bun --version) here, $pinned in CI: the web steps run on $pinned (bunx)."
  bun_ci=(bunx "bun@$pinned")
fi

step "Formatting"
cargo fmt --all --check

step "Clippy"
cargo clippy --workspace --all-targets --locked -- -D warnings

step "Simulation tests"
cargo test -p sim-core --locked

if ! $quick; then
  step "Generated houses and worlds"
  bun tools/housegen/sample.ts "$out/houses.json"
  bun tools/worlds/sample.ts "$out/worlds.json" 3 --sizes cosy,normal,big --big-per-kind 1 --big-out "$out/worlds-big.json"
  HOUSES="$out/houses.json" WORLDS="$out/worlds.json" cargo test -p sim-core --locked --test sim -- --ignored generated_houses:: generated_worlds::
  WORLDS="$out/worlds-big.json" cargo test -p sim-core --locked --test sim -- --ignored generated_worlds::
fi

(
  cd web
  step "Clip check"
  "${bun_ci[@]}" run check:clips
  step "Web packages"
  "${bun_ci[@]}" install --frozen-lockfile
  step "Shader chunks and renderer tests"
  "${bun_ci[@]}" run check:chunks
  "${bun_ci[@]}" run test
  # (Not in CI's test job, which needs no wasm build; the release work runs them. A type error
  # or a lint left here only fails later.)
  step "Types and lint"
  bun run wasm
  "${bun_ci[@]}" run check
  "${bun_ci[@]}" run lint
)

if ! $quick; then
  step "Tool tests"
  CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 cargo test -p ll-tools --release --locked
  step "Object sizes"
  cargo review sizes > "$out/review-sizes.md"
  step "Object uses"
  cargo review uses --check > "$out/review-uses.md"
  step "Models"
  cargo check-glb > "$out/check-glb.txt"
fi

step "All green"
