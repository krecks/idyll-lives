# Idyll Lives 0.3.2 — Release Plan

- **Branch:** `main`
- **Base:** v0.3.1 at `d4caf7b7`
- **Status:** Prepared (2026-10-22): version, notes and docs set; gates in §3.
- **Release type:** a feature: birds (renderer, audio and models only; the simulation and saves are as in 0.3.1)

## 1. What ships

Birds: blue tits, chaffinches, willow warblers, blackbirds and collared doves in the trees, bushes and hedges near the viewed lot (`web/src/render/babylon/birds.ts`, the Blender family `tools/art/blender/families/birds.py`, the plant models' `perches` and `habitat`, the visible birds' song in `web/src/audio/world.ts` and a blackbird song in `birdsong.ts`). Preset knob `birds`: Low 6, Medium 24, High 40. Player notes: the `## 0.3.2` section of [CHANGELOG.md](../CHANGELOG.md), also in [release-0.3.2-notes.md](release-0.3.2-notes.md).

- **Saves:** format 32, unchanged: 0.3.1 and 0.3.2 open each other's saves, so 0.3.1 is a safe build to redeploy if this one goes wrong.
- **Not changed:** `crates/sim-core` and `crates/sim-wasm` (no diff since v0.3.1), so the simulation's long runs (`century`, `soak`, `balance`) and tickbench are as at 0.3.1 and weren't run again.
- **Known to be imperfect** (from the birds' build): at the usual camera distance a bird is a few pixels even drawn 1.25–1.6× life size; flights pass through tree crowns (only houses are obstacles); doves seldom use the roof, which is only drawn with walls up and zoomed out.

## 2. Version

`node tools/release/version.mjs 0.3.2`; CHANGELOG `## 0.3.2 — 2026-10-22`; README 0.3.2; PLAN.md's status line points here.

## 3. Release gates (local, 2026-10-22)

| Gate | Result |
|---|---|
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test -p sim-core --locked` | pass (641) |
| `cargo test -p ll-tools --release --locked` | pass (73); `cargo check-glb` 592 models, 0 failed |
| `cargo test -p voice --release --locked` | pass (32) |
| `node tools/release/version.mjs` | 0.3.2 everywhere |
| generated houses and worlds (`--ignored`) | pass |
| `pnpm check:voice` | pass |
| `pnpm check:clips` | 168 clips, 0 new, 0 known |
| `pnpm check` (svelte-check) | 0 errors, 0 warnings |
| `pnpm lint` | clean |
| `pnpm build` | pass; `dist` 627 MB (CI fails over 900) |

The birds in play (headless, the village scene): 24 on Medium, 0.02–0.05 ms of main-thread time a frame; from 18:00 at speed 4 every bird was roosting by 21:18, none ever inside a house, no page errors.

## 4. Release steps

1. `git push origin main`.
2. Dry run: `gh workflow run pages.yml --ref main -f deploy=false`, watched to the end.
3. Tag and push: `git tag -a v0.3.2 -m "Idyll Lives 0.3.2: birds"`, `git push origin v0.3.2`; watch the tag's run.
4. Live checks: menu and Credits show 0.3.2 (build `<tagged commit>`), Shared memory on, a game runs, the large files' sizes (as for 0.3.1), birds in the trees.
5. `gh release create v0.3.2 --verify-tag --title "Idyll Lives 0.3.2" --notes-file plan/release-0.3.2-notes.md`.
