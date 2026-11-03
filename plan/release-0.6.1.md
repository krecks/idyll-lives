# Idyll Lives 0.6.1 — Release Plan

- **Branch:** `release/0.6.1`, from `main` at `70280ee9`, merged into `main` by pull request; tag `v0.6.1` on the merge
- **Base:** v0.6.0 at `f4482e7f`
- **Status:** in release
- **Release type:** fixes: three simulation bugs in 0.6.0, found when the generated-world tests failed now and then in CI; no save change

## 1. What ships

Player notes: the `## 0.6.1` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.6.1-notes.md](release-0.6.1-notes.md). The fixes are #52 (merged into `main` as `2676e678`), each with a regression test that fails on 0.6.0's code.

| Found | Cause | Fix |
|---|---|---|
| A 6-year-old frozen for 6 hours in a generated world | An adopted (or newborn) child was put down exactly where the first parent was; a parent lying on a bed or sitting on a sofa is on a tile nobody can walk off, so the child stayed there until the game was reloaded. | The child stands on a free tile beside the parent (`approach_tile`), or at the lot's entrance (`Plot::arrival_point`; `lifecycle.rs` `add_child`, 7efa2ae0). Test: `babies::an_adopted_child_stands_beside_a_parent_on_the_bed_not_on_it`. |
| A resident waiting over an hour for someone who called them over | Meeting halfway measured the route's legs in the stacked grid, where a flight of stairs is the ~220 rows between two storeys' blocks: two people in one house, one upstairs, met 115 m down the street. | Legs measured as walked (`StairMap::distance`); a middle on the stairs is the nearer end of them (`conversation.rs` `halfway`, 8bf44475). Test: `conversation::tests::meeting_halfway_over_the_stairs_is_by_the_stairs_not_across_town`. |
| A car's heading of 6.299 rad, out of the sim's range check | A reversing route turned the heading with `yaw + PI` and interpolated it unwrapped; the riders took it on. It points the same way on screen. | Wrapped when turned and when interpolated (`cars.rs` `Route::timed`/`at`, 9976a3ca). Test: `cars::tests::a_car_faces_within_a_half_turn_either_way_backing_out_too`. |

The first two were exposed by #48's change to the town's timing (a car parking changes walking only round it), found by bisecting a saved failing world; neither was caused by it.

- **Saves:** format 35, unchanged: 0.6.0 and 0.6.1 open each other's saves, so 0.6.0 is a safe build to redeploy.
- **Known:** loading a Big world holds the game once for 0.3–1.1 s while every resident's commute is worked out, as in 0.6.0.

## 2. Checks of the fixes

- Before the fixes, 2 of 18 freshly generated sets of Cosy and Normal worlds failed the generated-world tests (the frozen child; the hour's wait after meeting over the stairs). Both saved failing worlds pass now, and 15 fresh sets passed with 0 failures.

## 3. Version

`cargo release-version 0.6.1`; CHANGELOG `## 0.6.1 — 2026-11-03`; README 0.6.1 and the build to go back to; PLAN.md's status line points here.

## 4. Release gates (local, 2026-11-03)

`tools/ci-local.sh` (every CI step, on CI's Bun 1.4.3, and the web's types and lint), the voice's tests, the production build, and the game played.

| Gate | Result |
|---|---|
| `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (143 + 665) |
| generated houses and worlds (`--ignored`, as CI) | pass |
| `bun run check:clips` | 313 clips, 0 new, 0 known |
| `bun run check:chunks` | 195 anchors and names found in PlayCanvas 2.23.3 |
| `bun run test` (each file isolated) | pass (281) |
| `cargo test -p ll-tools --release --locked` | pass (112) |
| `cargo review sizes`, `cargo review uses --check`, `cargo check-glb` | pass (629 models, 0 failed) |
| `bun run check`, `bun run lint` | 0 errors, 0 warnings; clean |
| `cargo test -p voice --release --locked`, `bun run check:voice` | pass (32); matches the golden samples |
| `cargo release-version` | 0.6.1 everywhere |
| `vite build --base=/idyll-lives/` | pass, 674 MB (under 900) |
| The game | a saved game at speed 3 resumes at 3, residents act, pause and resume, birds, no console errors; the production build's benchmark (village, Medium): 0.6.1, cross-origin isolated, 60 fps on WebGPU and WebGL2, no page errors |

## 5. Release steps

1. Pull request `release/0.6.1` → `main`; CI passes on it; merge (a merge commit).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.6.1 -m "Idyll Lives 0.6.1: residents unstuck, meeting by the stairs"`, `git push origin v0.6.1`; watch the tag's run.
4. Live checks: the bundle's version and build, cross-origin isolated, the benchmark game on WebGPU and WebGL2 with no page errors.
5. `gh release create v0.6.1 --verify-tag --title "Idyll Lives 0.6.1" --notes-file plan/release-0.6.1-notes.md`.
6. Clean up `release/0.6.1`.
