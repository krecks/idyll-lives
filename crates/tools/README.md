# The repo's tools

One binary each, under `src/bin/`, run through the cargo aliases in
[`.cargo/config.toml`](../../.cargo/config.toml). None of them is shipped: the game is `crates/sim-core`
(and `crates/voice`), and these build, fetch, check and release what surrounds it. Their Python and
JavaScript predecessors are gone; `tools/` keeps only what needs Playwright, Blender or the
browser's own runtime (see [What is still not Rust](#what-is-still-not-rust)).

Every tool's `src/<name>.rs` opens with what it does and its arguments; this page is the map.

| Alias | Does |
|---|---|
| `cargo assets` | Compressed copies of the game's assets for the production build, which runs it (was `tools/assets/optimize.mjs`): textures to KTX2 (Basis Universal ETC1S, with mipmaps), models quantised and meshopt-compressed, the residents' binaries gzipped, kept by content hash in `web/.assets/cache`. `--force`, `--jobs N`, `--verbose`; `cargo assets check [--against <dir>]` reads every copy back and compares it with its source. |
| `cargo art` | Builds the Blender families and checks what they wrote (was `tools/art/blender/build.py`). `--all`, `--only id`, `--preview`, `--no-pack`, `--centres`. |
| `cargo characters` | The residents generator: rig, body, face, hair and clothes from code, plus its checks (was `tools/characters/prim/`). |
| `cargo check-chunks` | Checks that what the renderer patches or reads inside PlayCanvas is still there, in the release build the game imports: the anchors and private names listed in `tools/render/chunks.json` (was `tools/render/check_chunks.ts`; `bun run check:chunks` runs it). |
| `cargo check-glb` | The gate for the game's glTF models: reads every `.glb` a model entry uses and checks what the game needs of it (was `tools/art/blender/check_glb.py`). |
| `cargo content` | Moves catalog items into the Validate category and back (was `tools/content/validate.py`). |
| `cargo decoders` | Refreshes `web/public/decoders/basis/`, PlayCanvas's Basis transcoder, which the cross-origin isolated page can't fetch from a CDN (was `tools/assets/decoders.mjs`). |
| `cargo envconv` | The sky's lighting: downloads Poly Haven's CC0 HDRI "Belfast Open Field" (1k, pinned by its SHA-256) into `web/.assets/env/` and makes `web/public/assets/env/belfast_open_field.pcenv` of it (cube faces turned and mirrored into the PlayCanvas renderer's space, and the ambient as spherical harmonics, both as the game's look was calibrated on them). The output is committed; run it when the conversion changes. |
| `cargo fetch-voice` | The voice model and the pronunciation dictionary into `web/public/voice/` (was `tools/voice/fetch.mjs`, `fp16.mjs`). |
| `cargo fetch-llm` | The language model into `web/public/llm/`, repacked smaller without changing what it computes (`llmslim.rs`; was `tools/llm/fetch.mjs`). |
| `cargo review` | The authenticity review's first pass over every buyable object: `sizes`, `uses`; and `colours`, every model's colours against what real surfaces reflect (2 % to 80 % of the light, metals and glass left out): a report of those outside. |
| `cargo tex` | The texture scripts: `clouds`, `synth` (every surface map, generated in code), `materials` (was `tools/art/*.py`). |
| `cargo pins` | Accepted parts of the game held by their SHA-256 (`tools/pins/pins.json`: a file, a marked region or an `export const`), so work on something else can't change them by the way (was `tools/pins/check.ts`). `--list`, `--update`, `--add`. |
| `cargo shots` | Numbers from the bench's pictures: `stats` (brightness, clipping, saturation, how much is one tone, contrast at four sizes), `steady` (how much a picture changes when the camera moves a quarter of a pixel), `compare` (how many pixels differ between two shot folders, with pictures of where; `render.mjs --compare` runs it) and `sheet` (pictures of the same name from several folders tiled side by side under their names; `looks.mjs` and `play.mjs` run it). Was `tools/bench/stats.mjs`, part of `render.mjs` and `sheet.mjs` on sharp. |
| `cargo world-map` | Draws the sample worlds (`tools/worlds/sample.ts`'s output) from above, an SVG each and an index page (was `tools/worlds/map.ts`). |
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
`.github/workflows/pages.yml`. If a release goes wrong, fix forward or redeploy the last good build;
which builds open which saves is in the README's "If a release goes wrong" (0.6.0 and 0.7.0 share
save format 35, 0.5.x can't open it).

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
- `cargo assets`: on an M3 Pro every copy from nothing takes about 23 s (863 files: 61 textures, 778 models, 24
  residents' binaries; the textures are nearly all of it), and nothing when nothing changed. Two of its crates
  are C++ underneath (`basisu_c_sys`: Basis Universal's encoder, with bindings made by `bindgen`, so it needs
  libclang; `meshopt`: meshoptimizer): about two minutes more for a build of the tools from cold. Against the
  JavaScript tool it replaced (2026-11-13, both read back by `cargo assets check --against`): the same
  2,190,603 triangles, each within the same 0.29 mm of its source at most; models 0.9 % larger (42.8 MB),
  textures the same size (5.41 MB) with the colour and normal maps' levels the same to 0.2 of 255 on average;
  the packed occlusion and roughness maps' smaller levels are averaged as the data they are (the JavaScript
  tool averaged them as colours: up to 15 of 255 off their source's mean, now up to 4).
- `wasm-opt`: on an M3 Pro, the simulation's WASM took 48–55 s from cold and the voice's 3 s. Expect
  1.5–3 minutes on a CI runner from cold; CI keeps `web/.assets/wasm-opt/` between runs
  (`actions/cache`), so an unchanged WASM is instant. It made the two files 620 KB smaller raw and
  44 KB smaller gzipped, with no change in speed.
- The site (`web/dist`) was 489 MB in the first CI dry run (assets 56 MB, voice 181 MB, language
  model 248 MB; the largest file the language model's weights, 255,530,880 bytes). CI fails over
  900 MB, below Pages' limit of 1 GB.
- The live checks after a release: `llm/lfm2.5-350m/model_q4.onnx_dataz` is 195,523,654 bytes and
  `voice/supertonic-3/vector_estimator.onnxz` 55,258,368 bytes. (Both are gzipped by Rust's
  `flate2`, whose output is a little smaller than Node's zlib was, and the same on every platform.)

CI keeps a WASM with function names as an artifact (`wasm-with-names-<commit>`) for 90 days: the
site's own optimisations with the name section kept (`cargo wasm-opt --names`), so its function
numbers match a crash stack from the site.

## What is still not Rust

| Tool | Why |
|---|---|
| `tools/voice/check.mjs`, `samples.mjs`, `tools/llm/check.mjs`, `devices.mjs` (and the `engine.mjs` they share) | They run the models the way the browser does: ONNX Runtime Web, plus the worker's own `web/src/voice/supertonic.ts` and `voices.ts`. Porting them means an ONNX runtime in Rust (`ort` or `tract`) and `supertonic.ts` in Rust. |
| `tools/characters/check_clips.ts` | Reads the clip data that is generated into `web/src/render/shared/characters/clips/`. |
| `tools/bench/*.mjs` | Playwright: they drive a real browser and time the render. What they do with the pictures afterwards is `cargo shots`. |
| `tools/housegen/sample.ts`, `tools/worlds/sample.ts` | They run the game's own generators, which are TypeScript (`web/src/game`). |
| `tools/audio/listen.ts`, `tools/characters/blend_analysis.ts` | They run the game's own sound synthesis and clips, which are TypeScript. |
