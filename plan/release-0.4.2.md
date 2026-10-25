# Idyll Lives 0.4.2 — Release Plan

- **Branch:** `release/0.4.2` (the version and notes), merged into `main` by pull request; tag `v0.4.2` on the merge. The feature came first in `feat/procedural-grass` (PR #27, merged as efe1da33).
- **Base:** v0.4.1 at `2035d7ba`
- **Status:** **Released 2026-10-25**: https://github.com/krecks/idyll-lives/releases/tag/v0.4.2 (PR #28 merged as c9066c66; CI 40219885966; dry run 40220615649 on main; the tag's deploy 40221704610; live checks passed).
- **Release type:** a feature: the lawn's grass built on the GPU

## 1. What ships

Player notes: the `## 0.4.2` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.4.2-notes.md](release-0.4.2-notes.md). Design, decisions and measurements: [procedural-grass.md](procedural-grass.md).

| Change | Where |
|---|---|
| The lawn's tufts replaced by blades built in the vertex shader: seven instanced patch draws chosen by distance, thinned and widened with it, shrinking away at the preset's reach | `web/src/render/pc/grass.ts`, `web/src/render/pc/shaders/grass.ts` |
| Where the grass grows (the terrain's colour and grassiness, the town's mask), the gusts' noise and its CPU twin, the residents' push | `web/src/render/shared/grass.ts` |
| Gusts rolling along the wind over the blades and the ground; residents push the blades aside | the same |
| `QualitySettings.grass`; `lawnDensity` thins only the flowers | `web/src/render/quality.ts`, `docs/design/graphics.md` |
| `model.grassTuft` removed (GLBs, pack entry, Blender builder) | `tools/art/blender/families/ground.py` |

- **Saves:** format 32, unchanged: 0.4.1 and 0.4.2 open each other's saves, so 0.4.1 is a safe build to redeploy.
- **Not changed:** `crates/sim-core`, `crates/sim-wasm`, `crates/voice`.
- **Cost** (village, M3 Pro, 1080p, throughput, 20 s × 3, against `main` the same hour): Medium WebGL2 orbit +0.45 ms (budget +0.5), WebGPU +0.32 ms; the wide views 0.6–0.9 ms cheaper; Low within noise.

## 2. Checks of the feature

PR #27's checks: `bun run check`, `lint`, `test` (61, with `tests/grass.test.ts`), `check:chunks`, `cargo check-glb`, `cargo test -p ll-tools`; shots on WebGL2 and WebGPU by day and night, near and far; the wind and the push on running frames; CI passed (run 40218483181).

## 3. Version

`cargo release-version 0.4.2`; CHANGELOG `## 0.4.2 — 2026-10-25`; README 0.4.2; PLAN.md's status line points here.

## 4. Release gates (local, 2026-10-25)

| Gate | Result |
|---|---|
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (641) |
| `cargo test -p ll-tools --release --locked` | pass (101) |
| `cargo test -p voice --release --locked` | pass (32) |
| `cargo release-version` | 0.4.2 everywhere |
| `bun run check` (svelte-check) | 0 errors, 0 warnings |
| `bun run lint` | clean |
| `bun run test` | pass (61) |
| `bun run check:chunks` | 185 anchors and names found in PlayCanvas 2.23.0 |
| `bun run check:clips` | 168 clips, 0 new, 0 known |
| `bun run check:voice` | pass |

## 5. Release steps

All done 2026-10-25. Live: the build is c9066c6 (the release merge); the village benchmark plays on WebGPU and WebGL2 with the new grass and no page errors (`bench-results/live-0.4.2`).

1. Pull request `release/0.4.2` → `main`; CI passes on it; merge (a merge commit).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.4.2 -m "Idyll Lives 0.4.2: grass that sways and parts"`, `git push origin v0.4.2`; watch the tag's run.
4. Live checks: the bundle's version and build, cross-origin isolated, the benchmark game on WebGPU and WebGL2 with no page errors.
5. `gh release create v0.4.2 --verify-tag --title "Idyll Lives 0.4.2" --notes-file plan/release-0.4.2-notes.md`.
6. Clean up: the merged `feat/procedural-grass` and `release/0.4.2`, and `grass-field` (the earlier tuft passes, superseded), locally and on GitHub.
