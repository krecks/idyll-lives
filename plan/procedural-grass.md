# Idyll Lives — Procedural Wind-Reactive Grass Plan

- **Branch:** `grass` (from `origin/main` at `e99aeee1`, v0.4.1)
- **Base:** `main`'s lawn: `model.grassTuft` (7-blade Blender tufts, 1–2 a tile, instanced in 64 m squares) on the shared foliage wind
- **Status:** implemented and measured 2026-10-25 (phases 0–7; budget met, §6); owner sign-off pending on the look; not committed yet
- **Supersedes:** `grass-field`'s `plan/realistic-grass.md` (four passes on denser Blender tufts: still read as repeated clumps or static, with a faint sway, at +1–3 ms on Medium WebGL2)

## 1. Intent

Replace the lawn's tufts with grass that reads as a lawn of blades at every distance the camera sees it, with gusts you can see rolling across it, and that parts round the residents walking through it, at no more GPU time than `main`'s lawn.

Owner decisions (2026-10-25): start over cleanly; **stylised-realistic** (dense curved blades, dark root to light tip, per-blade variation, gust waves; within the "warm miniature diorama" of [PLAN.md](../PLAN.md)); **wind and character push**; **budget = main**: Medium WebGL2 orbit-day within +0.5 ms of `main` (throughput bench), Low within noise.

## 2. Approach

The blades are built on the GPU. Nothing is authored in Blender, and no blade is placed on the CPU.

- **Patches** (`render/pc/grass.ts`): the land round the town is cut into 2 m patches. Each frame (only when the eye moves 0.4 m or turns a third of a degree) the patches with grass within the preset's `radius` of the eye and in the frustum are sorted into seven draws by their nearest distance. Each draw is one instanced mesh of a patch's blades, whose vertices are only (t along the blade, side, blade number). A copy's matrix moves it to its patch's corner on the ground, and its `GRASS_SLOPE` stream carries the ground's tilt.
- **Blades** (`render/pc/shaders/grass.ts`, `GAME_GRASS`, GLSL and WGSL from one template):
  - A hash of the patch and the blade number gives each blade its root, turn, height, width, lean and shade.
  - It is drawn only while its number is within the share drawn at its distance: `min(1, near / d) × density`, as `grassShare`. Past the share it shrinks to nothing over a fifth of it, and the blades drawn widen by the share's inverse, so the cover stays the same near and far. All blades shrink away towards the reach.
  - The shape is a quadratic curve of 1–3 segments.
  - The normal is the curve's, turned to the eye, rounded across the blade and bent to the sky with distance. It reaches the lit shader through a new `vsNormal` slot (`shaders/chunks.ts`, inside the existing `normalVS` override; no new engine anchor).
  - Colour: the ground's own colour under it, desaturated a little, dark at the root (0.55) and lighter and yellower at the tip (1.35).
  - Matte: no specular path, about a third of the blades' cost. The sky stays on as the ambient, so the night stays dark.
- **Where it grows** (`render/shared/grass.ts`, pure, `tests/grass.test.ts`), in two textures:
  - **Colour** (RGBA, 1 texel a metre over the town and 40 m round it): the terrain's colour (`LandscapeLayout.terrainColour`, factored out of `shadeTerrain`) and, in alpha, how much of the land is lawn. There is none on worn patches, fields, rock, snow, road verges, sand or water.
  - **Town mask** (one byte, 4 texels a metre, `townGrass`): none indoors (on a split tile, the indoor half), under furniture, on patios, streets, paths and fields, nor within 0.22 m of a wall. It is made again on `refreshLawn`, so Build mode clears and regrows it.
- **Wind:**
  - A 64² tileable gust noise made in code, read in two octaves carried along `WIND.dir` at 4.5 m/s: frozen turbulence, so gusts travel. `gustAt` is its CPU twin.
  - Each blade bends with the gust over it plus a small flutter of its own, and lightens a little in a gust (the bent blades show their paler side).
  - The ground carries the same gusts as a ±few % lightening (`GAME_GRASS_GROUND`, colour only), so the waves roll on past the blades.
  - It uses the existing `game_wind` uniform, so reduce motion (`setWind(0)`) stills the grass with every other plant.
- **Ground under the grass:** darker under the blades near the eye (the shade between them, ×0.55), blending towards the reach into ×0.8 of the lawn's colour, as the blades and the shade between them average. There is no ring where the blades end.
- **Character push:** each frame the 16 residents nearest the eye, standing on the ground, push the blades within 0.5 m away and down, easing in and out over 0.2 s (`GrassPush`, `PlayCanvasRenderer.pushGrass`).
- **Flowers** stay as they were: the same draws from the same seeded stream (the tufts' random draws are still taken), the same places.

## 3. Quality

`QualitySettings.grass` (`render/quality.ts`; `docs/design/graphics.md`):

| Preset | radius | density | nearest draw |
|---|---|---|---|
| Low | 45 m | 0.5 | 1 (no 3-segment blades) |
| Medium | 60 m | 1 | 0 |
| High | 90 m | 1 | 0 |

`lawnDensity` now thins only the flowers; `streetLod` draws only their far versions.

## 4. Results

- **Budget met:** Medium WebGL2 orbit +0.29 ms (budget +0.5), Low within noise; the wide views are cheaper (§6).
- **Look:** a lawn of blades at the orbit camera, curved blades up close, gust bands rolling along the wind, the ground carrying the lawn and its gusts past the blades with no ring; the night as dark as `main`'s. Shots: `bench-results/grass-proc-shots-after/` against `grass-proc-shots-before/`.
- **Motion** was checked on running frames, the camera held (`hold=1`): wind (blades change lean frame to frame) and the push (blades part in a ring round a pusher).
- **Fixed on the way:** grass on the side roads past the town's ends (`CountryRoads.links`, which `near` doesn't know); the lawn stops 0.6 m from every road's edge (`terrainColour`, `LAWN_CLEAR`; a test in `tests/grass.test.ts`).
- **Fixed after review (the owner, 2026-10-25):** zoomed out, the far blades twinkled white: a pixel or two each, their light tips (×1.35, a gust's ×1.2) against the darker shade under them. Over `LAWN.blend` (45–85 % of the reach) the blades now take the lawn's plain colour, as the ground does over the same distances (`shaders/grass.ts`); shots `bench-results/grass-sparkle-before/` and `grass-sparkle-after/`.
- **Checks:** `bun run check` 0/0, `lint` clean, 61 tests pass, `check:chunks` 185 anchors, `cargo check-glb` 591 models, 0 failed, `cargo test -p ll-tools` passes.
- **Bench tools:** `?radius=`, `?beta=`, `?tx=`, `?tz=` (a close look at the lawn) and `?hold=1` (the camera held, time running) for `bench.ts`.

## 5. What was removed

`model.grassTuft` (its GLBs, its pack entry, `grassTuft()` and its palette in `tools/art/blender/families/ground.py`, its centre). `material.finish.foliage` stays (every plant uses it). The historical reviews in `docs/review/` are left as they were.

## 6. Measurements

Village, 1920×1080, M3 Pro, `--throughput`, 20 s after 5 s warm-up, median of 3 (`bench-results/grass-proc-before`, `grass-proc-after`). GPU frame avg, ms:

| Preset | Backend | View | `main` | Grass | Change |
|---|---|---|---:|---:|---:|
| Low | WebGL2 | orbit-day | 2.63 | 2.66 | +0.03 |
| Low | WebGL2 | wide | 2.86 | 2.65 | −0.21 |
| Medium | WebGL2 | orbit-day | 4.36 | 4.65 | +0.29 |
| Medium | WebGL2 | wide | 5.00 | 4.09 | −0.91 |
| Low | WebGPU | orbit-day | 1.65 | 1.71 | +0.06 |
| Low | WebGPU | wide | 1.84 | 1.66 | −0.18 |
| Medium | WebGPU | orbit-day | 2.07 | 2.34 | +0.27 |
| Medium | WebGPU | wide | 2.67 | 2.01 | −0.66 |

After the far-blade blend and the code cleanup, re-measured the same hour against `main` (Medium WebGL2 orbit-day, 20 s × 3; `bench-results/grass-main-recheck`, `grass-simplify-bench-full`): `main` 4.42 ms, grass 4.80 ms, **+0.38 ms**, still inside the +0.5 budget. An early-out for collapsed blades measured the same (4.80 ms both ways) and was left out. The ground reading the town's mask too (no blade shade by walls, under furniture or on patios; `bench-results/grass-mask-bench`): 4.87 ms, **+0.45 ms** against that `main`; WebGPU 2.39 ms (+0.32).

CPU (update + draw), Medium WebGL2: orbit 3.65 → 3.98 ms, wide 4.18 → 3.57. Draws within ±5. Where the blades' cost went (Medium WebGL2 orbit, 8 s runs): about 0.3 ms vertex work and 1.0 ms fragment work before the matte material took 0.35 ms off; the ground's gust shading about 0.05 ms.
