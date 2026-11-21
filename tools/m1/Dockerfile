# The M1 estimate's container (tools/m1/README.md): Linux on arm64 with the game's own toolchain,
# so the sim builds and runs here as it does natively, and under a CPU quota as a slower machine.
FROM debian:trixie-slim

# The pins: rust-toolchain.toml's Rust, the wasm-bindgen the project uses (Cargo.lock), CI's Bun
# (.github/workflows/ci.yml), and a Node of the line the dev machine runs.
ARG RUST=1.100.0
ARG WASM_BINDGEN=0.2.130
ARG BUN=1.4.4
ARG NODE=26.13.0

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl build-essential pkg-config unzip xz-utils 7zip \
  && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL "https://nodejs.org/dist/v${NODE}/node-v${NODE}-linux-arm64.tar.xz" | tar -xJ -C /usr/local --strip-components=1

ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "${RUST}" --target wasm32-unknown-unknown \
  && cargo install wasm-bindgen-cli --version "${WASM_BINDGEN}" --locked \
  && rm -rf /usr/local/cargo/registry

RUN curl -fsSL https://bun.sh/install | BUN_INSTALL=/usr/local bash -s "bun-v${BUN}"

WORKDIR /repo
