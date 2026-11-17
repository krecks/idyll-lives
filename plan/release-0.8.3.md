# Idyll Lives 0.8.3 — Release Plan

- **Branch:** `release/0.8.3`, cut from `main` at `89b684a6` (v0.8.2) on 2026-11-15; to be merged into `main` by pull request; tag `v0.8.3` on the merge
- **Base:** v0.8.2 at `89b684a6`
- **Release type:** a minor release's worth of work under a patch number, as the owner chose: New game in five steps, places and the land shaped by hand, fuller towns, four speeds with super speed, and the traffic, simulation and memory work they needed; save format 40

## 1. What ships

Seven branches, merged into `release/0.8.3` one after another (315 commits after v0.8.2):

- **`feat/world-generation`** (`b8e7f888`): New game makes the world in step 1; nothing is built before the main menu, which shows a picture of the last world made (`menuStill.ts`; the showcase town and its cache gone).
- **`feat/world-places`** (`ade83aaf`): nine places from one `WorldSpec` (the eight kinds from before kept as the classic specs, their drafts pinned by hash), the options (town, villages, hills, woods), a relief grid under the land, river valleys with bridges, islands with causeways, the heath; step 1's panel down the left.
- **`feat/land`** (`b08ba889`): the land as a height field (`game/land.ts`, `meta.land`), the brushes and the land editor, the town graded into the land it follows (`render/shared/builtSurface.ts`: roads within their grades, pads, retaining edges, the datum), Build mode's Land tab and the simulation's `ShapeLand` command; brushes 1–120 m.
- **`feat/sim-profile`** (`45dcfa05`): the `profile` feature, `tick_once` timed phase by phase.
- **`feat/new-game`** (`b7a2de94`): New game as Land, Town, Household and Home; the land first, then the town laid on it; the world code (`worldCode.ts`); the town's people from the code's people seed in two waves, the town filled on Move in (`people.ts`); `MAX_SIMS` 240 → 768, `MAX_CARS` 128 → 384, `residentsMost`; the town editor (`ui/townedit/`, `game/edit/`: streets with bridges, lots, villages, build-up, the settle and its refusals, the Problems list); the traffic deadlocks and the visits cut short at the full fill; the quadratic costs of a full town and the noon hitch (`find_spot`); the generated-world layout fixes.
- **`bench/memory-attribution`** (`e2e4bde3`): the gameplay bench's memory report, texture sources let go after upload. It didn't build as pushed (`Geo.grow` missing) and kept its notes in `docs/bench/`: fixed (`08d7081f`), the notes taken out of `docs/`.
- **`feat/083-polish`** (`c6b04b10`), the open points after New game, in streams: the simulation (household index, claims by key, step masks, rooms kept shut), the M1 estimate (`tools/m1/`), the editor's speed (an edit on screen in 0.7–1.4 s on a big city, from 2–5 s), the look (the menus' town's shadows and haze, far houses on hills, WebGL2's ground on Medium), the generated worlds' faults at seed bases 5–7, memory (`Geo` in typed arrays, model meshes GPU-only, three model files at a time, ≤ 8 GB: the menus' town let go at the loading screen and the voices and the language model on demand), shaping built land in the game and bridges from water strokes; then the owner's two calls of 2026-11-17:
  - **four speeds** (`feat/083-speeds`): pause, 1×, 3×, 10× and super speed (up to 60×, only while the household is all asleep or out), 30× and 60× removed; save format 40 keeps the speed super speed drops back to. Every graphics preset fills the town in full again: 10× needs 600 ticks a second, and the M1 estimate for the slowest full city is about 880 (756–952);
  - **five steps** (`feat/five-steps`): Streets & zones (the proposed layout, the zone brush on a painted `uses` layer, the Street tool's magnet and joins) and Build (the town built from the zones, Build again) as their own steps, and the free camera in every step.

## 2. Version

`cargo release-version 0.8.3`; CHANGELOG `## 0.8.3 — 2026-11-17`; README 0.8.3, the five steps in Playing, save format 40 and the way back to 0.8.2; PLAN.md's status line, the 0.8.3 paragraph, the UI row and the roadmap (the traffic's row as 0.8.4 with what 0.8.3 did for it, the 0.9.0 memory work partly done); `docs/playing.md`'s first step.

## 3. Release gates (local)

On `release/0.8.3` at `c6b04b10`, Bun 1.4.4, the M3 Pro:

| Gate | Result |
|---|---|
| `bun run test:slow` | pass (119 in 17 files) |
| `SEED_BASE=0…4 tools/ci-local.sh generated` | pass (3 each; 454–527 s a base) |
| `cargo release-version` | 0.8.3 |
| `bun run check` | 0 errors, 0 warnings (1,197 files) |

The stream branches ran their own tests as they merged (the touched Rust and web tests; `grid_identity` re-blessed for the rider-and-car fix and the early start, now off, its load fingerprints unchanged each time; the classic worlds' golden hashes never re-pinned). The tag's CI runs the generated worlds and the slow tests again on Linux, and the deploy waits for it.

## 4. Release steps

1. Push `release/0.8.3`; pull request into `main`; CI passes on it; merge (a merge commit).
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.8.3 -m "Idyll Lives 0.8.3: New game in five steps"`, `git push origin v0.8.3`; watch the tag's CI and Pages runs.
4. Live checks: the bundle's version and build, cross-origin isolated, the large files' sizes.
5. `gh release create v0.8.3 --verify-tag --title "Idyll Lives 0.8.3" --notes-file plan/release-0.8.3-notes.md`.

## 5. Open points (for the owner)

- **The departure spread is off:** workers setting off up to 20 game minutes early, by household, made the generated worlds worse at the full fill (seed bases 1, 3 and 4 failed where they pass without it; work taxis no more on time), so `EARLY_MOST` is 0; the way it is applied stays.
- **Generated worlds outside the gate:** the gate runs seed bases 0–4; the faults found at bases 5 to 7 (roads side by side, a lost garage with its mechanics and transport work, a 5.5 m road cut) were fixed (`a3e46154`, `f4973466`, `d4bbde06`).
- **Memory:** about 1.1 GB of the renderer's steady memory is owned by no counter (likely freed V8 pages and the allocator keeping them), and the entry's burst (1.29 GB of ArrayBuffers, 2.6 GB of committed heap at its peak) is the larger number and the next to take on. Every memory figure was taken on a shared machine and swings by hundreds of MB run to run.
- **The M1 estimate** can't see the GPU, the frame time, unified memory or thermals (`tools/m1/README.md`).
- **WebGL2 on Low** and the rest of 0.9.0 are still to do.
