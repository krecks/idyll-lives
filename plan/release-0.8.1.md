# Idyll Lives 0.8.1 — Release Plan

- **Branch:** `release/0.8.1`, cut from `main` at `3884d602` on 2026-11-15; to be merged into `main` by pull request; tag `v0.8.1` on the merge
- **Base:** v0.8.0 at `9401649f`
- **Release type:** everything merged into `main` since 0.8.0 (the living town, the porch, the junctions' follow-ups, the quality pass, the asset prune, the CI and dependency work) and the 13 pull requests that took on PLAN.md's measured hot spots of 2026-11-14; save format 39

## 1. What ships

The owner's call (2026-11-15): bring the hot-spot pull requests together, review them and release them as 0.8.1.

- **#92** `ci/faster-runs`: Pages no longer repeats CI's tests and waits for CI on its commit; only main saves caches.
- **#93** `perf/resident-draws-tags`: town tags laid out only when something they read changed (residents' draws left as they are).
- **#94** `perf/overview-release`: the menus' town let go during a game.
- **#95** `perf/mesh-cpu-copy`: no CPU copy of meshes that never change.
- **#96** `perf/paused-frame-hold`: a paused game standing still is held.
- **#97** `perf/worker-messages`: the worker sends what changed; the lawn refreshed only around an edit.
- **#98** `perf/medium-webgl2-gpu`: the sun's shadows redraw only what moves over a kept picture of the rest; WebGL2's MSAA as a preset setting (Medium keeps 4×).
- **#99** `chore/ll-tools-build`: a `tools` profile for the tool aliases.
- **#100** `perf/sim-smaller`, **#102** `perf/walk-commute`, **#104** `perf/route-nearest`: the simulation's hot spots, results bit-identical.
- **#101** `perf/street-plans`: a Big world's street keeps only what it draws.
- **#103** `perf/download-hash`: data files by content hash, kept across builds.

### The merge

- `pages.yml`: #92 removed Pages' own test steps; #99's change to its tool-test line goes with them.
- `house.ts`: #95's `MeshUse` on `draw`/`drawData` with #101's part without geometry and entity return.
- `PlayCanvasRenderer.ts`: #96's `pictureChanged()` kept in the lamps' callback and `setWorld` with #98's `this.samples` and #103's lazy `ltFixture`.
- CHANGELOG: every side's entries kept.
- Fixes CI had found on the pull requests: `rooms_print` with `as_chunks` (clippy, #97); the `sim-partners` pin re-taken for `free_now`'s split (#100, no behaviour change).

### The review

`/code-review high` over the whole merge found no high-confidence bugs; three of low-to-medium likelihood were fixed (`50f536d7`): a town tag whose element the keyed `each` made again wasn't placed or shown; a sun-shadow redraw whose still picture wasn't copied in drew only the movers over the last map; a Pages deploy by hand took a pull request's CI run (which tested the merge with main) as its commit's.

## 2. Version

`cargo release-version 0.8.1`; CHANGELOG `## 0.8.1 — 2026-11-15`; README 0.8.1, save format 39 and the way back to 0.8.0; PLAN.md's status line, the 0.8.1 paragraph, Measured hot spots with what 0.8.1 did, the roadmap (the traffic's row as 0.8.2, the 0.9.0 memory work partly done).

## 3. Release gates (local)

On `release/0.8.1`, Bun 1.4.4, the M3 Pro:

| Gate | Result |
|---|---|
| `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `SLOW_TESTS=1 tools/ci-local.sh sim` | pass (807 in the sim binary, 22 ignored), tickbench |
| `tools/ci-local.sh generated` (seed base 0) | pass (3) |
| `cargo test -p ll-tools --profile tools --locked` (pins included) | pass |
| `bun run check`, `bun run lint` | 0 errors, 0 warnings; clean |
| `bun run test` | pass (573) |
| `bun run test:slow` | pass (41 in 9 files) |
| `bun run check:chunks` | 266 anchors and names found in PlayCanvas 2.23.3 |

The tag's CI runs the generated worlds and the slow tests again on Linux, and the deploy waits for it.

## 4. Release steps

1. Push `release/0.8.1`; pull request into `main`; CI passes on it; merge (a merge commit, so #92–#104 show as merged).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.8.1 -m "Idyll Lives 0.8.1: a living town, lighter"`, `git push origin v0.8.1`; watch the tag's CI and Pages runs.
4. Live checks: the bundle's version and build, cross-origin isolated, the large files' sizes.
5. `gh release create v0.8.1 --verify-tag --title "Idyll Lives 0.8.1" --notes-file plan/release-0.8.1-notes.md`.

## 5. Open points (for the owner)

- **Choices left as the pull requests made them:** quitting a continued game to the menu waits about 3 s (#94; keeping the menu's landscape would save about 1 s for 240 MB); a held paused frame stands the wind, clouds and marker still (#96, `?snapshot=0` turns holding off); Medium keeps 4× MSAA and contact shading (#98's screenshots of 2× and FXAA in `bench-results/msaa-options/` of its worktree); the stale Actions caches (about 8 GB) are left to expire (#92).
- **Not done:** Snowdale's road walks failing on a waypoint on a blocked tile (#102; fixing it changes walks); chunks not drawn keep their vertex arrays (#101); model and instancing-buffer copies (#95); the lamps' shadow maps redrawn whole (#98).
- **Times measured under load:** every before/after time of #92–#104 was taken while a dozen of them built at once; measure again on a quiet machine (memory, draws, tiles and bytes are exact).
- **The review's cleanups, not done:** holding a paused frame relies on hand-placed `pictureChanged()` calls; `leaves_by` repeats `shift_today`/`school_today` (a `debug_assert` keeps them together); `rooms_changed` hashes the rooms on every lot change (a version on the lot would be exact); `Graph::seek` repeats `project_segment`; three FNV fingerprints; `Near::enough` sorts per ring; two copies of the caster-list swap in `sunShadows.ts`.
