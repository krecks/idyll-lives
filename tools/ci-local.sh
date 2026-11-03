#!/usr/bin/env bash
# The CI `test` job's steps, in one place: .github/workflows/ci.yml runs each as
# `tools/ci-local.sh <step>`, and run here with no step they all go, before a push or a pull request,
# so what goes up arrives green.
#
# The Bun steps run on the Bun CI pins (ci.yml's `bun-version`), fetched with bunx where the one
# installed differs: another Bun can pass what CI's fails.
#
#   tools/ci-local.sh            every step, and `types` (about 20 minutes)
#   tools/ci-local.sh --quick    fmt clippy sim clips web types
#   tools/ci-local.sh web sim    those steps
#
# Steps: fmt clippy sim generated clips web tools sizes uses models, and types (the web's types and
# lint over fresh wasm builds of the sim and the voice: not in CI's job, which builds no wasm, so run
# only here).
set -euo pipefail
cd "$(dirname "$0")/.."

all=(fmt clippy sim generated clips web tools sizes uses models types)
quick=(fmt clippy sim clips web types)

pinned=$(sed -nE 's/^ *bun-version: *([0-9.]+).*/\1/p' .github/workflows/ci.yml | head -1)
[[ -n "$pinned" ]] || { echo "No bun-version in .github/workflows/ci.yml" >&2; exit 1; }
bun_ci=(bun)
# (CI runs the Rust steps before it sets Bun up.)
if command -v bun >/dev/null && [[ "$(bun --version)" != "$pinned" ]]; then
  echo "Bun $(bun --version) here, $pinned in CI: the Bun steps run on $pinned (bunx)."
  bun_ci=(bunx "bun@$pinned")
fi

# The tools' release build without fat LTO, its tests and the checks the same build (slow to link;
# the answers are the same).
tools_cargo() { CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 cargo "$@"; }

step() {
  printf '\n== %s\n' "$1"
  case "$1" in
    fmt) cargo fmt --all --check ;;
    clippy) cargo clippy --workspace --all-targets --locked -- -D warnings ;;
    sim) cargo test -p sim-core --locked ;;
    # The ignored generated_* tests need sample files made first. In the simulation tests' own build
    # (already optimised): a release build of the whole test binary would cost more than it saves.
    generated)
      local out
      out=$(mktemp -d)
      "${bun_ci[@]}" tools/housegen/sample.ts "$out/houses.json"
      # Cosy and Normal worlds three per kind, Big ones (up to 1024 m) one per kind in a file of their own.
      "${bun_ci[@]}" tools/worlds/sample.ts "$out/worlds.json" 3 --sizes cosy,normal,big --big-per-kind 1 --big-out "$out/worlds-big.json"
      HOUSES="$out/houses.json" WORLDS="$out/worlds.json" cargo test -p sim-core --locked --test sim -- --ignored generated_houses:: generated_worlds::
      WORLDS="$out/worlds-big.json" cargo test -p sim-core --locked --test sim -- --ignored generated_worlds::
      rm -rf "$out"
      ;;
    # The residents' clips on both bodies at every life stage (known failures: tools/characters/clip-baseline.json).
    clips) (cd web && "${bun_ci[@]}" run check:clips) ;;
    # The PlayCanvas shader chunks' anchors and private names the renderer relies on, and its tests.
    web) (cd web && "${bun_ci[@]}" install --frozen-lockfile && "${bun_ci[@]}" run check:chunks && "${bun_ci[@]}" run test) ;;
    # The character build, the object gate and the content and texture tools.
    tools) tools_cargo test -p ll-tools --release --locked ;;
    # Every shipped model against the reference's kinds; every use the renderer can play; every model at the gate.
    sizes) tools_cargo review sizes ;;
    uses) tools_cargo review uses --check ;;
    models) tools_cargo check-glb ;;
    types) (cd web && bun run wasm && bun run voice:wasm && "${bun_ci[@]}" run check && "${bun_ci[@]}" run lint) ;;
    *)
      echo "unknown step: $1 (steps: ${all[*]})" >&2
      return 2
      ;;
  esac
}

if [[ $# -eq 0 ]]; then
  set -- "${all[@]}"
elif [[ "$1" == "--quick" ]]; then
  set -- "${quick[@]}"
fi
for s in "$@"; do step "$s"; done
[[ $# -gt 1 ]] && printf '\n== All green (%s)\n' "$*"
exit 0
