# Idyll Lives 0.5.0 — the open points

- **Branch:** `release/0.5.0` (the ten streams below, merged in turn, then the follow-ups, the version and notes), merged into `main` by pull request; tag `v0.5.0` on the merge.
- **Base:** v0.4.2 at `c9066c66`, with `main`'s PRs #29–#32 merged in.
- **Status:** releasing 2026-10-29.
- **Release type:** features: everything left open after 0.4.2.

What was left open after 0.4.2 (`docs/objects/open.md` "Still open", `docs/authenticity-review.md` §9, PR #31's
"Checked and left as it is", `docs/design/performance.md` "Low on WebGL2"), done now. The owner asked for all of it
(2026-10-26), including the new behaviour that decision 3 had put off.

## Workstreams

Each runs in its own worktree and branch off `main`, and is merged by the coordinator in the order below. Ids are the
review's (`docs/review/*.md`).

| # | Branch | What | Ids |
|---|---|---|---|
| A | `feat/props-in-hand` | Props in hand: a book, a cup, cutlery, the phone, the cue and balls, the bats, the shaker, a coin, bird seed; the food from town in its own wrapping | G-70 (I-held-01, I-social-01, I-hobbies-05, I-kitchen-13, I-community-11), M-held-03 |
| B | `feat/body-language` | A body layer per mood (posture, pace, arms); a step-round turn on the spot; children's run at speed 3; the bed override on a held pose | G-71 (A-residents-07, A-residents-08), A-residents-03 (speed 3), PR #31 |
| C | `feat/privacy` | A private flag on rooms and things; a shut bathroom door the sim knows of; *Try for a baby* at a double bed behind a shut door | G-72 (I-systems-05, I-social-05), Q-21, §11.3 a |
| D | `feat/vendors` | Vendors and shopkeepers serve in town; the bakery case's door on the baker's side | I-community-09, Q-17 |
| E | `feat/laundry` | The washer and dryer as timed machines (load, run on their own, unload), a fold at the basket; the washing hung piece by piece | I-bathroom-07, A-outdoor-02 |
| F1 | `feat/garden-chess-places` | Low apples and the crown at 2.6 m, picked by children too; the home chess table for two; the bonsai, arbour roses, lectern and hoop used at their part | I-garden-11, I-garden-12, M-hobbies-11, I-packs-12 |
| F2 | `feat/moving-parts` | The arc lamp over the next tile; the hot tub's cover in two halves; the clock's trunk door; clips for the kettle's lid, the bread bin, the laptop's lid and the high chair's tray | M-lighting-06, A-outdoor-01, A-decor-01, I-tabletop-01, I-kids-01 |
| G1 | `fix/rendering-open` | Low's forward lamps kept to their room and to split tiles; the town map's backdrop; stairs from the basement; wall things over lowered walls; the mirrors' `glGetProgramiv` log; `partBox` bounds | M-lighting-08, M-living-02, M-houses-01 |
| G2 | `perf/low-webgl2` | Nature's switches for Low on WebGL2; the production build's frame time, the town overview at full night and far versions measured; the remaining casts in `web/src` | performance.md "Low on WebGL2" |
| H | `feat/sounds` | The ~25 uses still silent get their sounds | I-bathroom-04, G-53 |

Speeds 4 and 5 keep their foot slide (A-residents-03: won't fix, no gait steps at 45–90 m/s without strobing).

## Rules

As in the review (`plan/authenticity-review.md` §6.3 and §7.4): one browser page or one Blender job per agent, its own
vite on its own port, copied (not symlinked) wasm packages, headless at 1 % volume except for timed checks. A rebuild
passes `cargo art`, `cargo check-glb` and the budgets in `docs/design/objects.md`; a content change to `minutes` or
`effects` re-runs balance and soak; a changed clip passes `check:clips` at every stage; each fix updates its ledger row,
`docs/objects/<cat>.md` and `docs/objects/open.md`.

**Saves:** a stream that changes what is saved bumps `SAVE_VERSION` to 33 and migrates 32; the coordinator folds the
bumps into one at the merge.

## Merge order

G2, G1, H, F2, F1, A, B, E, C, D — the contained ones first, the sim's new behaviour last, so each conflicts with
as little as possible.

## Release

### What ships

Player notes: the `## 0.5.0` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.5.0-notes.md](release-0.5.0-notes.md). What each stream did and what it left: [docs/objects/open.md](../docs/objects/open.md) and the review ledgers.

After the streams: the props still mimed (a pen, the marker, the receiver, the ball in play), the birds at the seed, the camera keeping up at speeds 3–5, the play tool for several residents, socials and the phone, the debug states' gaps (authenticity review §7), the street's flower beds with no door, and the instanced draws' cached world box (Low's main thread, `docs/design/performance.md`).

- **Saves:** format 33 (the home chess table's 2 × 1 footprint, `migrate_32_grown`). 0.4.2's saves load; 0.4.2 can't open 0.5.0's (it keeps and skips them), so going back to 0.4.2 hides saves made since until 0.5.0 is back.
- **Dependencies:** PlayCanvas 2.23.3, ONNX Runtime Web 1.31.1 (PR #32).

### Version

`cargo release-version 0.5.0`; CHANGELOG `## 0.5.0 — 2026-10-29`; README 0.5.0 and the saves; PLAN.md's status line points here.

### Release gates (local, 2026-10-29)

| Gate | Result |
|---|---|
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (684, 21 ignored) |
| generated houses and worlds (`--ignored`, as CI) | pass |
| `cargo test -p ll-tools --release --locked` | pass (101) |
| `cargo test -p voice --release --locked` | pass (32) |
| `cargo release-version` | 0.5.0 everywhere |
| `bun run check` | 0 errors, 0 warnings |
| `bun run lint` | clean |
| `bun run test` | pass (122) |
| `bun run check:chunks` | 195 anchors and names found in PlayCanvas 2.23.3 |
| `bun run check:clips` | 311 clips, 0 new, 0 known |
| `bun run check:voice`, `check:llm` | pass |
| `bun run build` | pass |

### Release steps

1. Pull request `release/0.5.0` → `main`; CI passes on it; review; merge (a merge commit).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.5.0 -m "Idyll Lives 0.5.0: the open points"`, `git push origin v0.5.0`; watch the tag's run.
4. Live checks: the bundle's version and build, cross-origin isolated, the benchmark game on WebGPU and WebGL2 with no page errors.
5. `gh release create v0.5.0 --verify-tag --title "Idyll Lives 0.5.0" --notes-file plan/release-0.5.0-notes.md`.
6. Clean up the merged stream branches and `release/0.5.0`.
