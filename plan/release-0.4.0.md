# Idyll Lives 0.4.0 — Release Plan

- **Branch:** `playcanvas`, merged into `main` by pull request; tag `v0.4.0` on the merge
- **Base:** v0.3.2 at `be0dbdf0`
- **Status:** **Released 2026-10-24**: https://github.com/krecks/idyll-lives/releases/tag/v0.4.0 (PR #25 merged as b64e0cb1; CI 40115975935 and on main; dry run on main 40117190904; the tag's deploy 40118751867; live checks passed).
- **Release type:** a new renderer: PlayCanvas instead of Babylon.js (renderer only; the simulation and saves are as in 0.3.2)

## 1. What ships

The game draws with PlayCanvas 2.23 on WebGPU, with the WebGL2 fallback, instead of Babylon.js 9.30. The port went in stages 0–11, each checked against pictures of the Babylon renderer ([docs/design/graphics.md](../docs/design/graphics.md), The renderer; [docs/design/performance.md](../docs/design/performance.md), The PlayCanvas port). Player notes: the `## 0.4.0` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.4.0-notes.md](release-0.4.0-notes.md).

- **Saves:** format 32, unchanged: 0.3.2 and 0.4.0 open each other's saves, so 0.3.2 is a safe build to redeploy if this one goes wrong.
- **Not changed:** `crates/sim-core`, `crates/sim-wasm` and `crates/voice` (no diff since v0.3.2), so the simulation's long runs (`century`, `soak`, `balance`) and tickbench are as at 0.3.2 and weren't run again.
- **Measured** (final checkpoint, village with 41 residents, M3 Pro, 1920×1080): the main thread does 2.3–3.2× less work on WebGPU and 1.7–2.8× less on WebGL2; after the warm-up a scripted session compiles 1 shader and 1 pipeline in play (32 and 340 before); the 54 lit benchmark shots differ from Babylon's by a median 0.74 % of pixels by more than 16.
- **Generated houses, fixed here:** CI's generated-houses check had failed on `main` since PR #24 (303bafa0, 2026-10-22, the tools moved from Node to Bun): `housegen.ts` shuffled with `sort(() => rng() - 0.5)` (20 places), and JavaScriptCore's sort gave one seed other houses from call to call (4 layouts in 40 identical calls), so a house could come out otherwise for another household or another time. It shuffles with Fisher-Yates now (1 layout in 40; 0 problems in the sample). Every seed gives another house than before, so new towns get new houses; a saved town keeps its built houses.
- **Known to be imperfect:** on Low with WebGL2 the GPU does 2–15 % more than Babylon did (not yet profiled; Low's FXAA, a pass of its own on PlayCanvas, is the first suspect). Frame intervals jitter a little more around the refresh (p95 about 10 ms at 120 Hz against Babylon's 8.4–9.9), with the same average.

## 2. Review

A code review of the branch against `main` (the whole port, read closely in the last three commits and spot-checked in the riskiest new files) found seven things:

| Finding | Outcome |
|---|---|
| A failed `.pcenv` load stayed cached as null for the device | fixed: the next request tries again |
| `loadEnvironment` kept its timeout in one shared field | fixed: each call has its own |
| The catalog's stored pictures could be Babylon's | not a problem: the store is named per app version, and 0.4.0 is a new one |
| `Instances.reserve` draws the whole capacity until `update()` | not a problem in practice: every caller calls `update()` in the same synchronous run |
| The renderer's `dispose` leaves the WebGL2 context to the garbage collector | left: the canvas is reused when the backend stays, so losing the context there would break the next renderer; only many backend switches in one session could reach the browser's context limit |
| The birds' day-and-night test hashed matrices bit for bit (found by the first dry run) | fixed: the birds' sin, cos and exp differ in their last bits between macOS arm64 and Linux x86-64; the record keeps what is the same everywhere |
| `cargo envconv` no longer caps the HDRI download's size | left (a developer tool): the SHA-256 still refuses a wrong file; a cap in `download_to` is a follow-up |
| `read_hdr` reads a few unusual local `.hdr` files differently from the old reader | left (a developer tool): Poly Haven's file reads exactly as before |

## 3. Version

`cargo release-version 0.4.0`; CHANGELOG `## 0.4.0 — 2026-10-24`; README 0.4.0; PLAN.md's status line points here.

## 4. Release gates (local, 2026-10-24)

| Gate | Result |
|---|---|
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (641) |
| `cargo test -p ll-tools --release --locked` | pass (100) |
| `cargo test -p voice --release --locked` | pass (32) |
| `cargo release-version` | 0.4.0 everywhere |
| `bun run check:voice` | pass |
| `bun run check:clips` | 168 clips, 0 new, 0 known |
| `bun run check` (svelte-check) | 0 errors, 0 warnings |
| `bun run lint` | clean |
| `bun run test` | pass (51) |
| `bun run check:chunks` | 185 anchors and names found in PlayCanvas 2.23.0 |
| Dry run of the branch (`pages.yml`, run 40112707629) | pass; `dist` 619 MB (CI fails over 900) |
| CI: generated houses and worlds (`--ignored`) | pass, after the fix below; two sample runs give the same houses byte for byte |

## 5. Release steps

All done 2026-10-24. Live: the bundle says 0.4.0, build b64e0cb; cross-origin isolated (shared memory) once `isolation-sw.js` has reloaded the first visit; the village benchmark plays on WebGPU (55.6 fps headless) and WebGL2 (60 fps) with no page errors. The large files return 200 and unzip to their pinned SHA-256 (`llmfetch.rs`, `voicefetch.rs`): `model_q4.onnx_dataz` is 255,530,880 bytes and `vector_estimator.onnxz` 119,568,968, as at 0.3.2, not the README's 254,923,309 and 119,556,677. Those are what the Rust fetch tools' gzip writes; CI still serves the files cached from the earlier tools, so either size is right when the SHA-256 is.

`tools/bench/render.mjs --url` on the live site fails with "sim worker failed to start": it clears the site's storage before each page, which takes the isolation service worker with it, and GitHub Pages sends no isolation headers of its own. Opened as a player would, the site works.


1. Pull request `playcanvas` → `main`; CI passes on it; merge (a merge commit, as before).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.4.0 -m "Idyll Lives 0.4.0: PlayCanvas"`, `git push origin v0.4.0`; watch the tag's run.
4. Live checks: menu and Credits show 0.4.0 (build `<tagged commit>`), Shared memory on, a game runs, the large files' sizes (as in the README).
5. `gh release create v0.4.0 --verify-tag --title "Idyll Lives 0.4.0" --notes-file plan/release-0.4.0-notes.md`.
