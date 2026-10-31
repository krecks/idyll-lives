# Idyll Lives 0.5.1 — Release Plan

- **Branch:** `fix/0.5.1`, merged into `main` by pull request; tag `v0.5.1` on the merge
- **Base:** v0.5.0 at `efd47428`
- **Status:** **Released 2026-10-29**: https://github.com/krecks/idyll-lives/releases/tag/v0.5.1 (PR #34 merged as 7aa616f9; CI 40896648736; dry run 40897960646 on main; the tag's deploy 40900084711; live checks passed).
- **Release type:** fixes: three things the owner found in 0.5.0 (none new in it: 0.4.2 did the same)

## 1. What ships

Player notes: the `## 0.5.1` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.5.1-notes.md](release-0.5.1-notes.md).

| Found | Cause | Fix |
|---|---|---|
| Green patches on the streets of the menus' town | The roads lie 1.2 cm over the ground; with the near plane at 0.3 m the depth buffer's steps pass a centimetre from 250 m, and the overview looks from 200–300 m. Raised to 8 cm (a test) the patches went, and the road covered the sidewalks. | The near plane a fiftieth of the camera's distance, never under 0.3 m (`Lens.nearShare`, `render/pc/camera.ts`): unchanged up to 15 m, 1.9 m at the game's farthest, about 6 m from the overview. |
| The world behind the menus stutters | Measured in a window on a 120 Hz display (and on the live site at device pixel ratio 2): no hitch, an even 30 fps; the menus had a cap of their own (`MENU_FPS` 30), so a gliding or dragged camera held each picture four refreshes. A still town is held anyway (`heldFrames.ts`). | The menus draw at the player's cap (30, 60, 120 or none), as the game does: 30/60/119/118 fps measured on the opening glide. |
| Residents walk through the hedges between gardens | The hedges and fences are the renderer's (`shared/street.ts`); the simulation kept residents off others' gardens but opened the one they went to, and went straight over the boundary. | A step from one plot straight into another beside a garden costs three hundred tiles (`OVER_THE_FENCE`, `Grounds::fenced`, `path.rs`): next door is by the street; plots with no street between them are still reachable. |

Commits: c7376d96 (the near plane and the menus' cap), 8f947339 (next door by the street), 5ce8a0dd and 0d4ea25d (the review's cleanups: the fence only beside gardens, one plot lookup a tile, the rule in the lens).

- **Saves:** format 33, unchanged: 0.5.0 and 0.5.1 open each other's saves, so 0.5.0 is a safe build to redeploy.
- **Left as it was:** choosing another world on the new-world screen takes one 60–120 ms task on the main thread (the world made), as in 0.4.2.

## 2. Checks of the fixes

- The streets: the new-world screen (Medium town, `Math.random` seeded) at device pixel ratio 2 before and after: no green on any road; 0.4.2 and 0.5.0 drew the same picture before. Low and Medium's 36 benchmark shots on both backends differ only where surfaces lie millimetres apart (a road's edge, now straight; window panes): 0.04–0.16 % of the pixels.
- The frame rate: the opening glide in a window on the 120 Hz display at caps 30, 60, 120 and none: 30, 60, 119, 118 fps; settled, 150 of 150 frames held.
- Next door: `path::tests::into_the_garden_next_door_by_the_street_not_over_the_hedge` (it failed before: straight from (9, 3) to (10, 3)), and unfenced plots crossed straight. In the first 8 generated towns a resident of each sent two tiles into the garden beside their own went over the boundary 8 times before, 0 after, and all arrived.

## 3. Version

`cargo release-version 0.5.1`; CHANGELOG `## 0.5.1 — 2026-10-29`; README 0.5.1 and the saves; PLAN.md's status line points here; 0.5.0's plan marked released.

## 4. Release gates (local, 2026-10-29)

| Gate | Result |
|---|---|
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (687) |
| generated houses and worlds (`--ignored`, as CI) | pass |
| `cargo test -p ll-tools --release --locked` | pass (101) |
| `cargo test -p voice --release --locked` | pass (32) |
| `cargo release-version` | 0.5.1 everywhere |
| `bun run check` | 0 errors, 0 warnings |
| `bun run lint` | clean |
| `bun run test` | pass (122) |
| `bun run check:chunks` | 195 anchors and names found in PlayCanvas 2.23.3 |
| `bun run check:clips` | 311 clips, 0 new, 0 known |
| `bun run check:voice`, `check:llm` | pass |
| `bun run build` | pass |

## 5. Release steps

All done 2026-10-29. Live: the menu says v0.5.1, the bundle's build is 7aa616f (the release merge); cross-origin isolated; the village benchmark reports 0.5.1 on WebGPU and WebGL2 at 60 fps with no page errors.

1. Pull request `fix/0.5.1` → `main`; CI passes on it; merge (a merge commit).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.5.1 -m "Idyll Lives 0.5.1: the menus' town and next door by the street"`, `git push origin v0.5.1`; watch the tag's run.
4. Live checks: the bundle's version and build, cross-origin isolated, the benchmark game on WebGPU and WebGL2 with no page errors.
5. `gh release create v0.5.1 --verify-tag --title "Idyll Lives 0.5.1" --notes-file plan/release-0.5.1-notes.md`.
6. Clean up `fix/0.5.1`.
