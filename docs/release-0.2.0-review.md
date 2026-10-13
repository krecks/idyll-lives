# Release review for v0.2.0

The pre-release review for 0.2.0 started on 2026-10-13 from `main` at 4cde174, which is 136 commits after v0.1.0. It covers animations, objects and items, bugs, performance and release readiness. Each finding has a location, a severity, a description and a proposed fix.

Severity:
- **critical**: a crash, data loss, or a release blocker.
- **high**: a clearly visible bug or a missed performance target on common hardware.
- **medium**: a real problem that is rare or mild.
- **low**: cosmetic, hygiene or hardening.

Line numbers refer to 4cde174 unless a later commit is named.

## 1. How the game is put together

### Project structure and build
- **Rust workspace** (`Cargo.toml`, edition 2024, version 0.1.0 before this release) with four crates:
  - `crates/sim-core`: every game rule (needs, utility AI, pathfinding, town, social life, life cycle, saves, geometry). No browser code; tested natively (`cargo test -p sim-core`).
  - `crates/sim-wasm`: a thin wasm-bindgen wrapper (`Game`) around `sim_core::World`.
  - `crates/voice`: phonemizer, mouth shapes and voice shaping.
  - `crates/voice-wasm`: wasm-bindgen wrapper for the voice worker.
- **Release profile:** `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`. `.cargo/config.toml` enables `simd128` for `wasm32-unknown-unknown`.
- **Web app** in `web/`: Vite 8, Svelte 5, TypeScript, Babylon.js 9.30, pnpm.
  - `npm run wasm` builds `sim-wasm` in release and runs `wasm-bindgen --target web` into `web/src/core/wasm-pkg/` (gitignored).
  - `npm run voice` does the same for the voice crate and fetches the voice model.
  - `npm run llm` fetches the optional language model.
  - `npm run build` runs all three, then `vite build`. The production build also writes compressed copies of the assets (`tools/assets/optimize.mjs`: KTX2 textures, meshopt models).
  - Vite content-hashes every JS chunk and both `.wasm` files. The wasm-bindgen glue loads its wasm through `new URL(..., import.meta.url)`.
  - Before this release, no `wasm-opt` step ran anywhere.
- **Deploy:** `.github/workflows/pages.yml` publishes to GitHub Pages (https://krecks.github.io/idyll-lives/) when a `v*` tag is pushed. `tools/release/version.mjs` keeps `web/package.json` and `Cargo.toml` on one version, which the menu and Credits show through `__APP_VERSION__`.

### How the world is drawn
- **3D on the GPU.** Babylon.js uses WebGPU and falls back to WebGL2 (`web/src/render/babylon/`). Svelte draws only the interface (HUD, panels, menus) as DOM over the canvas; nothing in the world is DOM or SVG.
- **Only the viewed lot is built in detail.** Other houses are silhouettes. The landscape (terrain, forests, lawn) is procedural and uses thin instances.
- **Materials and lighting:**
  - Materials are PBR with image-based lighting.
  - One sun casts a 2048 shadow map, and lamps use clustered lighting.
  - A post-processing pipeline renders into a half-float HDR target with 4× MSAA, followed by tone mapping and a night grade.
- **Residents** are skinned on the GPU from a pose texture that is uploaded every frame.
- **A second, small engine** (`render/preview/`) draws the household-creator preview, portraits and catalog pictures.
- **Before this release:** quality presets low / medium / high / ultra plus fine-tune toggles (`render/quality.ts`, `settings/settings.svelte.ts`). The default came from High.

### The game loop
- **The simulation drives itself in a Web Worker** (`web/src/core/sim.worker.ts`) with a `setTimeout` loop at a fixed 20 Hz (`TICKS_PER_SECOND`, `sim-core/src/lib.rs`).
  - At speed 1, one game minute passes per real second.
  - Speeds 0–5 run `[0, 1, 3, 10, 30, 60]` ticks per step (`clock.rs`).
  - With auto-fast on, a step may run up to 30 ticks while everyone in the household is asleep or away ("quiet hours", `World::advance`).
- **Pause is speed 0:** the loop keeps publishing, but no ticks run. Menus, Build mode and the planner pause the game by switching to speed 0. They restore the previous speed when they close.
- **Falling behind:** if the worker falls more than 1 s behind, it drops the backlog instead of catching up. Ticks per step never exceed 60, so 1,200 ticks a second at most.
- **The renderer runs its own loop on the main thread** (`engine.runRenderLoop`, `BabylonRenderer.ts`), capped by the FPS setting.
  - It interpolates residents between the two latest sim steps, using a smoothed frame clock that follows snapshot arrivals (`core/bridge.ts`). It draws one step behind and extrapolates at most 1.3 steps.
  - Animations use real time; the character animation rate follows the game speed up to a limit.

### How state flows from Rust to the screen
- **Render snapshot, every step (20 Hz).** A flat `f32` buffer: 9 header floats plus 64 residents × 23 floats. Its layout is defined once in Rust (`sim-core/src/snapshot.rs`) and read by TypeScript at start-up.
  - When the page is cross-origin isolated, it travels through a SharedArrayBuffer guarded by a seqlock (`core/snapshot.ts`), so no copy and no message.
  - Otherwise it is transferred as a copy.
  - The worker re-creates its view of wasm memory on every publish, so wasm memory growth never leaves a stale view.
- **UI state, about 10 times a second and after each command.** Delta JSON (`view::UiSync`): only what changed, keyed by id, with details only for the player's household and the inspected resident. The main thread keeps a mirror (`game/uiUpdates.ts`). Large lists use `$state.raw`.
- **World structure** (walls, rooms, floors, meshes, objects), only when its version changes. JSON plus copied mesh buffers; the lot and meshes only when the lot itself changed.
- **Commands from the player.** JSON over `postMessage`, applied between ticks.
- **Never serialized per frame or per tick:** the whole world.

### Saving and loading
- **What is saved:** the simulation writes the whole world as JSON (`sim-core/src/save.rs`, `SAVE_VERSION`, 22 at the start of this review). Saves from older versions are migrated on load.
  - Time is an integer tick count (`u64`).
  - The random generator is a seeded xorshift whose state is saved; the starting seed was not saved before this release.
- **Where:** saves are kept in IndexedDB (`persistence/saves.ts`), each with metadata and a JPEG thumbnail.
  - Autosave writes one slot every 5 minutes by default.
  - A typical town save is 115–143 KB.
- **Elsewhere in the browser:** settings and blueprints live in localStorage.
- **Load:** a save is handed to the worker, which builds the world with `Game.fromSave`.
- **Starting fast:** the main menu prepares the latest save in the background so Continue starts at once.

## 2. Findings

*(Filled in from the review agents' reports.)*
