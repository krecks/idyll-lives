#!/usr/bin/env bash
# The game's three wasm packages (web/src/<pkg>/wasm-pkg: the simulation's, the sounds' and the voice's), built
# once per content and kept: `bun run wasm`, `bun run sound` and `bun run voice:wasm` call it.
#
#   tools/wasm-cache.sh sim|sound|voice     the package made ready in this checkout
#   tools/wasm-cache.sh key sim|sound|voice the key it is kept under
#
# The key is a hash of everything the package is built from: the git trees of its crates and of the workspace's
# Cargo.toml, Cargo.lock, .cargo/ and rust-toolchain.toml, the uncommitted changes to them and the untracked files
# among them, and the versions of rustc and wasm-bindgen. A package already made with that key (by this checkout or
# by any other worktree of the repo) is copied from ~/.cache/idyll-lives/wasm-pkg/<pkg>/<key>/; otherwise it is
# built here (cargo in this checkout's target/, then wasm-bindgen), kept there and copied. A build holds the key's
# lock, so two worktrees asking for the same package at once build it once. LL_WASM_CACHE=0 builds without the
# cache, as it always did, and so does CI ($CI set: Pages' `bun run wasm`) unless
# LL_WASM_CACHE=1 says otherwise. LL_WASM_CACHE_DIR keeps the packages elsewhere.
set -euo pipefail
cd "$(dirname "$0")/.."

mode=build
if [ "${1:-}" = key ]; then mode=key; shift; fi
pkg=${1:?usage: tools/wasm-cache.sh [key] sim|sound|voice}
case "$pkg" in
  sim) crate=sim-wasm out=web/src/core/wasm-pkg inputs="crates/sim-core/src crates/sim-core/Cargo.toml crates/sim-wasm" ;;
  sound) crate=sound-wasm out=web/src/audio/wasm-pkg inputs="crates/sound crates/sound-wasm" ;;
  voice) crate=voice-wasm out=web/src/voice/wasm-pkg inputs="crates/voice crates/voice-wasm" ;;
  *) echo "tools/wasm-cache.sh: no package $pkg (sim, sound or voice)" >&2; exit 2 ;;
esac
# (What every package is built with besides its crates: the profiles, the locked versions, the simd128 flag, the toolchain.)
inputs="$inputs Cargo.toml Cargo.lock .cargo rust-toolchain.toml"
lib=${crate//-/_}

build() {
  cargo build -p "$crate" --target wasm32-unknown-unknown --release --locked
  wasm-bindgen "target/wasm32-unknown-unknown/release/$lib.wasm" --target web --out-dir "$1"
}

if [ "${LL_WASM_CACHE:-$([ -n "${CI:-}" ] && echo 0 || echo 1)}" = 0 ] && [ "$mode" = build ]; then
  build "$out"
  exit 0
fi

sha256() { if command -v sha256sum >/dev/null; then sha256sum; else shasum -a 256; fi | cut -d' ' -f1; }

# shellcheck disable=SC2086
key=$(
  {
    for p in $inputs; do echo "$p $(git rev-parse "HEAD:$p" 2>/dev/null || echo none)"; done
    git diff HEAD --binary -- $inputs
    git ls-files --others --exclude-standard -- $inputs | while read -r f; do echo "$f $(git hash-object "$f")"; done
    rustc -V
    wasm-bindgen -V
  } | sha256 | cut -c1-24
)
if [ "$mode" = key ]; then
  echo "$key"
  exit 0
fi

store=${LL_WASM_CACHE_DIR:-$HOME/.cache/idyll-lives/wasm-pkg}/$pkg
kept=$store/$key
mkdir -p "$store"

# The key's lock: a directory (mkdir is atomic), with the holder's pid; one whose holder is gone is taken over.
lock=$kept.lock
until mkdir "$lock" 2>/dev/null; do
  holder=$(cat "$lock/pid" 2>/dev/null || true)
  if [ -n "$holder" ] && ! kill -0 "$holder" 2>/dev/null; then
    rm -rf "$lock"
    continue
  fi
  echo "wasm-cache: $pkg $key is being built elsewhere; waiting" >&2
  sleep 2
done
echo $$ >"$lock/pid"
trap 'rm -rf "$lock"' EXIT

if [ -d "$kept" ]; then
  echo "wasm-cache: $pkg $key kept, copied" >&2
  touch "$kept"
else
  echo "wasm-cache: $pkg $key not kept, building" >&2
  rm -rf "$kept.new"
  build "$kept.new"
  mv "$kept.new" "$kept"
fi
# (A copy, not a link: Vite refuses files outside the checkout.)
rm -rf "$out"
mkdir -p "$out"
cp "$kept"/* "$out"/
# The newest 12 kept of each package (about 5 MB each for the simulation's), the rest let go.
find "$store" -mindepth 1 -maxdepth 1 -type d ! -name '*.lock' ! -name '*.new' -print0 | xargs -0 ls -dt | tail -n +13 | while read -r old; do rm -rf "$old"; done
