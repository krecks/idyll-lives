# Idyll Lives 0.4.1 — Release Plan

- **Branch:** `fix/0.4.1`, merged into `main` by pull request; tag `v0.4.1` on the merge
- **Base:** v0.4.0 at `b64e0cb1`
- **Status:** **Released 2026-10-24**: https://github.com/krecks/idyll-lives/releases/tag/v0.4.1 (PR #26 merged as 2035d7ba; CI 40126464236; dry runs 40126439431 on the branch, 40127727579 on main; the tag's deploy 40128805829; live checks passed).
- **Release type:** fixes: the three points 0.4.0 left open (two fixed, one profiled)

## 1. What ships

Player notes: the `## 0.4.1` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.4.1-notes.md](release-0.4.1-notes.md).

| 0.4.0's open point | Outcome |
|---|---|
| The renderer's `dispose` left the WebGL2 context to the garbage collector | **fixed** (6e3e8793): `destroyApp` (`render/pc/device.ts`) destroys an app and loses its WebGL2 context, for the game's renderer and the previews' apps alike; the host gives every new renderer a fresh canvas (before, only a new backend got one, so the context couldn't be let go). Found while testing it: a renderer disposed on WebGL2 threw "Cannot read properties of null (reading 'deleteBuffer')" for every model template, freed after its device was gone; templates on a destroyed device are left alone now (`deviceDestroyed`). |
| `cargo envconv`'s HDRI download had no size cap | **fixed** (170d9f5c): `download::download_to` takes a byte limit and fails as soon as an answer passes it, leaving nothing behind; envconv passes the pinned 1,647,689 bytes. A test serves a file on localhost. |
| Low on WebGL2: the GPU does more than Babylon did | **profiled, not fixed**: not the FXAA; nature is the share that stands out (−0.75 ms in the wide view without it). Measurements and what's left in [docs/design/performance.md](../docs/design/performance.md), Low on WebGL2, profiled (0.4.1). |

- **Saves:** format 32, unchanged: 0.4.0 and 0.4.1 open each other's saves, so 0.4.0 is a safe build to redeploy.
- **Not changed:** `crates/sim-core`, `crates/sim-wasm`, `crates/voice`.

## 2. Checks of the fixes

- The renderer's restart, in Chromium through Playwright on the dev server, WebGL2 and WebGPU: the village benchmark game, then the graphics preset changed and the scene loaded again. A second renderer on a fresh canvas, the old canvas out of the page and its WebGL2 context lost, the second game drawing, no page errors (before the template fix: one error per model on WebGL2).
- The download cap: `download::tests::a_download_lands_whole_and_a_long_one_not_at_all`.

## 3. Version

`cargo release-version 0.4.1`; CHANGELOG `## 0.4.1 — 2026-10-24`; README 0.4.1; PLAN.md's status line points here.

## 4. Release gates (local, 2026-10-24)

| Gate | Result |
|---|---|
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (641) |
| `cargo test -p ll-tools --release --locked` | pass (101) |
| `cargo test -p voice --release --locked` | pass (32) |
| `cargo release-version` | 0.4.1 everywhere |
| generated houses and worlds (`--ignored`) | pass; the house sample 0 problems |
| `bun run check` (svelte-check) | 0 errors, 0 warnings |
| `bun run lint` | clean |
| `bun run test` | pass (51) |
| `bun run check:chunks` | 185 anchors and names found in PlayCanvas 2.23.0 |
| `bun run check:clips` | 168 clips, 0 new, 0 known |
| `bun run check:voice` | pass |

## 5. Release steps

All done 2026-10-24. Live: the bundle says 0.4.1, build 2035d7b; cross-origin isolated; the village benchmark plays on WebGPU and WebGL2 (60 fps headless) with no page errors; the large files unzip to their pinned SHA-256.

1. Pull request `fix/0.4.1` → `main`; CI passes on it; merge (a merge commit).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.4.1 -m "Idyll Lives 0.4.1: renderer restart and download fixes"`, `git push origin v0.4.1`; watch the tag's run.
4. Live checks: the bundle's version and build, cross-origin isolated, the benchmark game on WebGPU and WebGL2 with no page errors (with the isolation service worker in place), the large files' SHA-256.
5. `gh release create v0.4.1 --verify-tag --title "Idyll Lives 0.4.1" --notes-file plan/release-0.4.1-notes.md`.
