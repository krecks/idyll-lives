#!/usr/bin/env bash
# Readies the self-hosted runner (`self-hosted, Linux, X64`) for a CI job, run by
# .github/workflows/ci.yml's jobs right after the checkout (not pages.yml's build: see there):
#   - Rust on the PATH for this and the later steps (the runner's service has no login shell), rustup
#     installed once if the machine has none, and the toolchain rust-toolchain.toml pins;
#   - target/ linked to a Rust build kept on the machine between jobs (the checkout empties the
#     workspace, and it is the Actions cache's place on GitHub's runners). Every job shares it,
#     and Cargo's lock keeps two builds out of it at once. Past CI_TARGET_MAX_MB (default 40 GB) it
#     starts over. A second runner on the same machine needs a CI_TARGET_DIR of its own (in its .env):
#     one starting over would delete the build under the other's job.
# The machine needs git, curl, unzip, a C and C++ compiler and libclang (basisu's bindings):
# `apt install build-essential clang libclang-dev curl unzip git`.
set -euo pipefail
cd "$(dirname "$0")/.."

bin="${CARGO_HOME:-$HOME/.cargo}/bin"
export PATH="$bin:$PATH"
if [[ -n "${GITHUB_PATH:-}" ]]; then echo "$bin" >> "$GITHUB_PATH"; fi
# The wasm packages are kept on this machine between jobs (tools/wasm-cache.sh), which CI would otherwise skip: GITHUB_ENV carries it to the later steps.
export LL_WASM_CACHE=1; if [[ -n "${GITHUB_ENV:-}" ]]; then echo "LL_WASM_CACHE=1" >> "$GITHUB_ENV"; fi
if ! command -v rustup >/dev/null; then
  echo "No rustup here: installing it (the toolchain comes next)."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
    sh -s -- -y --no-modify-path --default-toolchain none --profile minimal
fi
rustup toolchain install
rustc --version
cargo clippy --version

kept="${CI_TARGET_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/idyll-lives/target}"
mkdir -p "$kept"
size=$(du -sm "$kept" | cut -f1)
if ((size > ${CI_TARGET_MAX_MB:-40960})); then
  echo "The kept Rust build is $size MB: it starts over."
  rm -rf "$kept"
  mkdir -p "$kept"
  size=0
fi
if [[ -e target && ! -L target ]]; then rm -rf target; fi
ln -sfn "$kept" target
echo "target -> $kept ($size MB)"
