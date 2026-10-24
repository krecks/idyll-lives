# The repo's tools

One binary each, under `src/bin/`, run through the cargo aliases in
[`.cargo/config.toml`](../../.cargo/config.toml). None of them is shipped: the game is `crates/sim-core`
(and `crates/voice`), and these build, fetch, check and release what surrounds it. Their Python and
JavaScript predecessors are gone; `tools/` keeps only what needs Playwright, Blender, sharp or the
browser's own runtime (see [What is still not Rust](#what-is-still-not-rust)).

Every tool's `src/<name>.rs` opens with what it does and its arguments; this page is the map.

| Alias | Does |
|---|---|
| `cargo art` | Builds the Blender families and checks what they wrote (was `tools/art/blender/build.py`). `--all`, `--only id`, `--preview`, `--no-pack`, `--centres`. |
| `cargo characters` | The residents generator: rig, body, face, hair and clothes from code, plus its checks (was `tools/characters/prim/`). |
| `cargo check-glb` | The gate for the game's glTF models: reads every `.glb` a model entry uses and checks what the game needs of it (was `tools/art/blender/check_glb.py`). |
| `cargo content` | Moves catalog items into the Validate category and back (was `tools/content/validate.py`). |
| `cargo decoders` | Refreshes `web/public/decoders/basis/`, PlayCanvas's Basis transcoder, which the cross-origin isolated page can't fetch from a CDN (was `tools/assets/decoders.mjs`). |
| `cargo envconv` | The sky's lighting for PlayCanvas: converts the Babylon `.env` in `web/public/assets/env/` into a `.pcenv` (cube faces turned and mirrored into the PlayCanvas renderer's space, and the ambient as spherical harmonics). The output is committed; run it when the `.env` changes. |
| `cargo fetch-voice` | The voice model and the pronunciation dictionary into `web/public/voice/` (was `tools/voice/fetch.mjs`, `fp16.mjs`). |
| `cargo fetch-llm` | The language model into `web/public/llm/` (was `tools/llm/fetch.mjs`). |
| `cargo review` | The authenticity review's first pass over every buyable object: `sizes`, `uses`. |
| `cargo tex` | The texture scripts: `clouds`, `process`, `house`, `materials` (was `tools/art/*.py`). |
| `cargo release-version` | One version for the whole game. Prints it and fails if `web/package.json`, `Cargo.toml` and `Cargo.lock` differ (CI runs this); with an argument, sets all of them. Not `cargo version`: that is cargo's own. |
| `cargo licenses` | `dist/THIRD-PARTY-LICENSES.txt`, written by production builds (`web/vite.config.ts`). |
| `cargo wasm-opt` | Optimises the simulation's and the voice's WebAssembly for release, with the pinned binaryen. |

The release tools and the three fetchers are unoptimised aliases (no `--release`): they wait on the
network or on binaryen, so a release build would only cost minutes of compiling. The rest are
release builds, as the character build is slow otherwise.

## Release

How to make a release is in the [README](../../README.md#versions-and-releasing): the version and the
CHANGELOG section (its date set by hand), `git push origin main`, the dry run (watched by its run id),
the tag, the live checks of the large files, and `gh release create … --verify-tag`. The workflow is
`.github/workflows/pages.yml`. If a release goes wrong, fix forward or redeploy the last good 0.3.x:
0.3.0 can't open 0.3.1's saves (format 32), 0.2.0 can't open 0.3.0's (format 29), and v0.1.0 can't
open 0.2.0's saves database.

What each release tool does, in more words than the table above:

- **`cargo release-version`** — one version for the whole game: `web/package.json` (shown in the menu
  and Credits), the Rust workspace in `Cargo.toml`, and the five workspace crates' entries in
  `Cargo.lock`. It prints it and fails if they differ (CI runs this; with `--locked` everywhere, a
  stale lock would fail the build anyway). `cargo release-version 0.3.0` sets all of them.
- **`cargo wasm-opt`** — binaryen's `wasm-opt -O3` (the exact-pinned `binaryen` devDependency), the
  name and producers sections stripped (`--names` keeps the name section, for CI's crash-stack
  artifact). The features are passed explicitly; never `-tnh` or `-iit`, because with
  `panic = "abort"` every panic is a trap. The production build runs it on the built files
  (`vite.config.ts`, the `release-wasm` plugin), about a minute the first time and then cached in
  `web/.assets/wasm-opt/`; the dev server keeps the WASM with its names.
- **`cargo licenses`** — `dist/THIRD-PARTY-LICENSES.txt` in production builds: the npm packages found
  in the bundles (the page's and every worker's), the Rust crates in both WASM files (`cargo metadata
  --locked --offline`), files shipped as they are (Inter), the texts kept in [`texts/`](texts/) (the
  Basis Universal transcoder's, shipped as a file; Babylon.js's licence and notice, for the parts of
  the renderer ported from it, each of which says so in a header; TalkingHead's, whose expressions
  are adapted in code) and the licences of the voice model, the dictionary and the language model,
  each text once. The build fails if a licence text is missing; its test checks the kept texts and
  that the files with Babylon.js code and their headers match its list (`BABYLON_PORTED`).

Numbers to know:
- `wasm-opt`: on an M3 Pro, the simulation's WASM took 48–55 s from cold and the voice's 3 s. Expect
  1.5–3 minutes on a CI runner from cold; CI keeps `web/.assets/wasm-opt/` between runs
  (`actions/cache`), so an unchanged WASM is instant. It made the two files 620 KB smaller raw and
  44 KB smaller gzipped, with no change in speed.
- The site (`web/dist`) was 489 MB in the first CI dry run (assets 56 MB, voice 181 MB, language
  model 248 MB; the largest file the language model's weights, 255,530,880 bytes). CI fails over
  900 MB, below Pages' limit of 1 GB.
- The live checks after a release: `llm/lfm2.5-350m/model_q4.onnx_dataz` is 254,923,309 bytes and
  `voice/supertonic-3/vector_estimator.onnxz` 119,556,677 bytes. (Both are gzipped by Rust's
  `flate2`, whose output is a little smaller than Node's zlib was, and the same on every platform.)

CI keeps a WASM with function names as an artifact (`wasm-with-names-<commit>`) for 90 days: the
site's own optimisations with the name section kept (`cargo wasm-opt --names`), so its function
numbers match a crash stack from the site.

## What is still not Rust

| Tool | Why |
|---|---|
| `tools/voice/check.mjs`, `samples.mjs`, `tools/llm/check.mjs`, `devices.mjs` (and the `engine.mjs` they share) | They run the models the way the browser does: ONNX Runtime Web, plus the worker's own `web/src/voice/supertonic.ts` and `voices.ts`. Porting them means an ONNX runtime in Rust (`ort` or `tract`) and `supertonic.ts` in Rust. |
| `tools/characters/check_clips.ts` | Reads the clip data that is generated into `web/src/render/shared/characters/clips/`. |
| `tools/bench/*.mjs` | Playwright: they drive a real browser and time the render. |
| `tools/assets/optimize.mjs` | ktx2-encoder, sharp and gltf-transform: image and texture work with no Rust equivalent worth writing. |
| `tools/housegen/sample.ts`, `tools/worlds/sample.ts` | Bun runs them; they are samples, not build steps. |
