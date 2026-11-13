# Where the time goes

## The PlayCanvas port

After 0.3.2 the renderer moved from Babylon.js 9.30 to PlayCanvas 2.23 (how it is built: [graphics.md](graphics.md#the-renderer)).

### Babylon.js against PlayCanvas: the engine comparison (2026-10-23)

`engines.html` (dev only, `web/src/debug/engines/`, removed after the port; in git history up to f15b839b, first in da009b41) drew the same scene with both engines, each run in a fresh frame of its own so they never shared a page or a GPU device: the game's furniture as glTF (20 models a room, about 220 mesh parts and 95 k triangles, laid out on a grid, without the game's finishes or soft look), a sun and a flat ambient, an orbiting camera. It measured the main thread's time for a whole engine frame (Babylon's `beginFrame`/`render`/`endFrame`, PlayCanvas's `update`/`render`), the time between frames, draw calls, and the load; not the GPU's own time. 1600×900 drawn pixels, 8 s measured after 3 s, on a Mac (Apple GPU, Metal 3, 12 cores; Chrome 156).

The cases: `furniture`, 8 rooms of the models each its own mesh; `instanced`, each model's copies as hardware instances of one mesh (Babylon's thin instances); `lamps`, 32 clustered spot lamps over the rooms; `shadows`, the sun's shadows (one 2048² map, 3×3 PCF); `game`, lamps and shadows together. The game-sized cases are one house's furniture with a garden, 300 instanced trees, 8 of the game's own residents walking (skinned on the GPU by each engine from `rig.json` and the `.bin` bodies), 8 lamps, sun shadows and the game's post-processing (MSAA into an HDR target, tone mapping, vignette, colour grading) at the game's camera distance: `lot`; `lot-nopost` without the post-processing; `lot-bloom` with bloom added; `lot-crowd` with 32 residents.

Chromium, vsync at 120 Hz (main-thread ms a frame, average, and p95):

| Case | Draws | WebGPU CPU ms, Babylon → PlayCanvas | WebGL2 CPU ms, Babylon → PlayCanvas | WebGPU fps | WebGL2 fps |
|---|---|---|---|---|---|
| `furniture` | 1609 | 6.61 → 0.96 (p95 7.63 → 1.67) | 3.73 → 1.07 (p95 4.57 → 1.29) | 120.0 → 120.0 | 120.0 → 120.0 |
| `instanced` | 202 | 1.15 → 0.39 (p95 1.40 → 0.52) | 0.90 → 0.49 (p95 1.52 → 0.82) | 120.0 → 120.0 | 120.0 → 120.0 |
| `lamps` | 1610 | 8.56 → 0.86 (p95 9.39 → 1.35) | 4.95 → 1.00 (p95 5.47 → 1.27) | 115.3 → 120.0 | 120.0 → 120.0 |
| `shadows` | 3217 | 12.57 → 1.12 (p95 14.38 → 1.46) | 8.72 → 1.88 (p95 10.34 → 2.49) | 78.8 → 120.0 | 113.3 → 120.0 |
| `game` | 3218 | 14.77 → 1.44 (p95 15.93 → 1.83) | 8.84 → 1.97 (p95 10.75 → 2.46) | 67.2 → 120.0 | 111.4 → 120.0 |

| Case | Draws | WebGPU CPU ms, Babylon → PlayCanvas | WebGL2 CPU ms, Babylon → PlayCanvas |
|---|---|---|---|
| `lot` | 675 | 3.23 → 1.03 (p95 3.84 → 1.61) | 2.58 → 1.00 (p95 3.20 → 1.35) |
| `lot-nopost` | 674 | 3.02 → 1.15 (p95 3.68 → 1.75) | 2.62 → 0.97 (p95 3.05 → 1.19) |
| `lot-bloom` | 679 | 3.49 → 0.90 (p95 4.13 → 1.22) | 2.75 → 1.02 (p95 3.34 → 1.31) |
| `lot-crowd` | 1059 | 6.89 → 1.82 (p95 8.11 → 2.07) | 4.63 → 1.74 (p95 5.37 → 2.17) |

The same game-sized cases without vsync (frames as fast as they come; with the display no longer the limit, fps shows the GPU too):

| Case | Draws | WebGPU CPU ms, Babylon → PlayCanvas | WebGL2 CPU ms, Babylon → PlayCanvas | WebGPU fps | WebGL2 fps |
|---|---|---|---|---|---|
| `lot` | 675 | 3.70 → 1.08 (p95 4.30 → 1.52) | 4.00 → 2.85 (p95 4.79 → 4.60) | 262.1 → 860.1 | 240.0 → 335.0 |
| `lot-nopost` | 674 | 3.44 → 0.95 (p95 4.18 → 1.44) | 3.45 → 2.68 (p95 4.16 → 4.56) | 281.6 → 958.4 | 268.5 → 356.1 |
| `lot-bloom` | 679 | 4.65 → 1.20 (p95 6.32 → 1.67) | 4.37 → 3.01 (p95 5.10 → 4.72) | 210.0 → 746.6 | 221.7 → 319.3 |
| `lot-crowd` | 1059 | 6.92 → 1.60 (p95 8.01 → 1.92) | 5.39 → 3.25 (p95 6.29 → 4.95) | 142.1 → 582.2 | 179.4 → 292.4 |

WebKit (Safari's engine, WebGPU, 60 Hz; each run in a freshly started browser, as WebKit 26.5 slows down for good after a few runs in one; the files don't record the machine):

| Case | Draws | WebGPU CPU ms, Babylon → PlayCanvas | WebGPU fps |
|---|---|---|---|
| `furniture` | 1609 | 26.59 → 1.94 (p95 27.72 → 2.66) | 29.4 → 60.0 |
| `instanced` | 202 | 3.58 → 1.11 (p95 4.52 → 1.70) | 60.0 → 60.0 |
| `lamps` | 1610 | 30.35 → 1.77 (p95 31.58 → 2.52) | 26.5 → 60.0 |
| `shadows` | 3217 | 50.62 → 1.73 (p95 52.42 → 2.06) | 16.8 → 60.0 |
| `game` | 3218 | 51.71 → 2.00 (p95 53.42 → 2.76) | 16.8 → 60.0 |

| Case | Draws | WebGPU CPU ms, Babylon → PlayCanvas | WebGPU fps |
|---|---|---|---|
| `lot` | 675 | 14.92 → 1.73 (p95 16.12 → 2.12) | 59.8 → 60.0 |
| `lot-nopost` | 674 | 12.23 → 1.58 (p95 12.92 → 1.90) | 60.0 → 60.0 |
| `lot-bloom` | 679 | 15.00 → 1.86 (p95 15.90 → 2.68) | 60.0 → 59.7 |
| `lot-crowd` | 1059 | 23.16 → 2.15 (p95 24.50 → 2.96) | 40.0 → 60.0 |

On a game-sized scene PlayCanvas took about a third of Babylon's main-thread time in Chromium (WebGPU; two fifths on WebGL2) and about a ninth in WebKit, where Babylon fell below 60 fps with 32 residents and with the furniture cases' 1,600–3,200 draws. Where draws dominate (`furniture`, `lamps`, `shadows`, `game`), Babylon's main thread took 3.5–11 times PlayCanvas's in Chromium.

### Low on WebGL2, the woods (0.5.0)

2026-10-26, the same machine, with up to ten agents sharing it (`bench-results/0.5.0/`). Under that load, with vsync, one cell's GPU time came out at about 3 ms in some runs and about 6 ms in others (Low, WebGL2, the wide view: 5.99, 3.07, 6.54 ms), flipping between rounds, the Apple GPU's clocks following how busy the machine was rather than what was drawn. So the numbers here are from runs without vsync (`--throughput`, the GPU always busy, about ±0.2 ms), each change against its base in the same round and the rounds interleaved, and from an in-page A/B (one bench page, the variants switched every 2 s along the camera path, each window's GPU time against the base window next to it, 30–40 rounds), which cancels the slow drift altogether. Triangles and draws don't depend on the load.

Nature's parts, one left out at a time (the new `diag=nowoods,nofarwoods,noflowers,nograss`; Low, WebGL2, the wide view):

| left out | triangles (M) | draws | GPU ms, without vsync (median of 3) |
|---|---|---|---|
| nothing | 1.80 | 316 | 3.3 |
| all nature (`nonature`) | 0.77 | 270 | 2.5 |
| the far woods (`nofarwoods`) | 1.00 | 280 | 2.8 |
| the woods near the town (`nowoods`) | 1.62 | 306 | 2.9 (6 rounds later: within the noise) |
| the lawn's flowers (`noflowers`) | 1.74 | 315 | not timed apart (with vsync, within the noise) |
| the grass (`nograss`) | 1.80 | 316 | nothing: none is in reach of the wide view's eye |

So the far woods (the far versions of the trees from 28 m past the town's edge to the horizon) are nearly all of nature's cost. What a far tree costs, from the in-page A/B against the far woods as they were (GPU ms, median of the paired differences): thinning them to 60 % −0.15 and to 40 % −0.26, leaving them out −0.40 to −0.42; without their normal and ARM maps −0.02, without the wind −0.10, without receiving the sun's shadows 0.00, against a copy of their materials +0.08 (the copy itself, a control). So the cost goes with the number of trees drawn, not with how each is shaded.

And most of the trees drawn weren't on screen. The game drew the woods in eight sectors round the town, each from the town's edge to the horizon, and a sector with one tree in view draws all of its trees. Counted along the wide view's path (nine moments, each far tree's foot and crown projected to the screen): about 50 of the 2291 far trees on screen, about 700 drawn (fourteen times as many). Drawn in squares they would be: 100 m, about 180 (3.6×); 80 m, about 155; 60 m, about 115.

**The change:** Low draws the woods in 100 m squares (`QualitySettings.woodsCell`; `?woodscell=<m>` sets it, 0 the sectors). The same trees in the same places: Low's 18 benchmark shots on both backends with sectors and with squares are the same to the pixel, but for a car on the far road in the two wide night shots (at most 0.018 % of the pixels). Medium and High keep the sectors (`woodsCell: 0`), so nothing about them changes. Tried and not kept: the woods near the town in their far versions (no gain to be seen), the far woods without the sun's shadows (none), and thinning the far woods (−0.26 ms at 40 %, but a visibly thinner forest; the squares save as much and change nothing on screen).

Low on WebGL2, the production build, before (sectors) and after (squares), and with `fix/rendering-open` (each lamp kept to its room on Low too) merged in, 4 interleaved rounds without vsync, GPU ms (median; the change against the base in the same round, median):

| scenario | base | squares | room mask | both | triangles (M) | draws |
|---|---|---|---|---|---|---|
| orbit-day | 3.03 | −0.11 | +0.09 | −0.25 | 1.30 → 1.04 | 275 → 266 |
| orbit-night | 3.07 | −0.13 | +0.12 | 0.00 | 1.31 → 1.05 | 281 → 272 |
| wide | 2.86 | −0.33 | +0.03 | −0.27 | 1.79 → 1.21 | 315 → 311 |

The room mask's cost on Low with WebGL2 is within the noise (0.0–0.1 ms here; the 5.2 → 4.4 ms seen at its merge was most likely the clocks flipping, as above). The squares' cost is on the main thread: 446 more meshes (the far woods' 72 group draws become 518), each culled every frame, +0.17–0.20 ms of main-thread time in every round of a separate vsync check (orbit-day, 5 rounds: 1.44 → 1.62 ms without the room mask, 1.44 → 1.63 ms with it; the room mask itself +0.00–0.03). Low is the preset for older laptops' integrated graphics, which wait on the GPU, so the trade is worth it there; Medium and High, whose main thread is the busier one on WebGPU, keep the sectors.

### Low on WebGL2 on a quiet machine, and the world boxes (0.5.0)

2026-10-29, the same machine with nothing else running (no process but the window server above 10 % of a core), PlayCanvas 2.23.3, the 0.4.1 method: the production build, windows at 120 Hz with vsync, 20 s after a 5 s warm-up, three rounds, each the squares and then the sectors (`?woodscell=0`) beside them. A first set as the branch stood (`bench-results/0.5.0/quiet-*`), a second with the world boxes below (`boxes-*`).

Even on a quiet machine the GPU time with vsync still spreads: the wide view with squares came out at 5.39, 4.55 and 3.89 ms in the first set's three rounds, 4.32, 4.22 and 4.17 in the second's. Low on WebGL2 against Babylon's single run, GPU ms (median of 3):

| scenario | Babylon | 0.4.1 | first set: squares / sectors | second set: squares / sectors |
|---|---|---|---|---|
| orbit-day | 3.83 | 4.07–4.24 | 4.42 (+15 %) / 4.39 | 4.32 (+13 %) / 4.36 |
| orbit-night | 3.96 | 4.22 | 4.59 (+16 %) / 5.32 | 4.39 (+11 %) / 4.79 |
| dusk | 4.16 | 4.25 | 4.11 (−1 %) / 4.54 | 4.24 (+2 %) / 4.29 |
| build | 3.69 | 3.77 | 3.60 (−3 %) / 3.87 | 3.77 (+2 %) / 3.85 |
| wide | 4.03 | 4.67–4.76 | 4.55 (+13 %) / 4.27 | 4.22 (+5 %) / 4.22 |

So the wide view, +16–18 % in 0.4.1 and the one gap clear of the noise then, is +5 % in the steadier second set. Dusk and Build are at Babylon's. The orbits stay 11–16 % over in both sets, 0.4–0.6 ms; that is past the plan's +10 %, but it isn't the woods (squares and sectors alike: 4.32 and 4.36 ms), and with three rounds spreading by up to 0.5 ms it is about the size of the noise. What else costs more there isn't known yet. Under vsync the squares' own GPU saving (−0.11 to −0.33 ms without vsync, above) doesn't show past the noise.

**The world boxes.** A CPU profile of the bench page (Low, WebGL2, orbit-day, 12 s, twice each way) put most of the squares' main-thread cost in one function: PlayCanvas works out a draw's world box again every frame when it has a box of its own (`setCustomAabb`, which every culled `Instances` draw sets), and the woods' 518 draws took that from 0.08 to 0.19 ms a frame. The woods, the street's things and the flowers never move. `Instances` now keeps the world box until the box or the entity's place changes (`_updateAabbFunc`, against the node's `_aabbVer`; both in `check:chunks`), and the function's share fell to under 0.001 ms. Test: `tests/instancesBox.test.ts`, on the engine's null device. Low and Medium's 36 benchmark shots on both backends are the same to the pixel before and after, but for the car on the far road in the wide night shots (at most 0.032 %).

Main thread, ms (median of 3; the squares' cost is against the sectors of the same set):

| scenario | first set: squares / sectors (cost) | second set: squares / sectors (cost) |
|---|---|---|
| orbit-day | 1.43 / 1.27 (+0.16) | 1.29 / 1.17 (+0.12) |
| orbit-night | 1.47 / 1.35 (+0.12) | 1.29 / 1.23 (+0.06) |
| dusk | 1.79 / 1.61 (+0.18) | 1.68 / 1.57 (+0.12) |
| build | 1.44 / 1.21 (+0.23) | 1.31 / 1.22 (+0.09) |
| wide | 1.55 / 1.23 (+0.31) | 1.31 / 1.19 (+0.13) |

The squares now cost the main thread about half what they did, and Low with squares is 0.06–0.24 ms lighter than before. Every preset gains a little, since all culled instanced draws keep their box (the sectors here 0.04–0.12 ms). What is left of the squares' cost is the 446 more entities walked each frame (`syncHierarchy`, about +0.02 ms) and the GL state of the draws (texture binds and uniforms).

## The open world: a street in chunks, a camera that roams (2026-11-01)

The open world's renderer work (`docs/design/open-world.md`, branch `feat/open-world-render`):
the street is planned once per town, view-independent, per 64 m chunk (`render/shared/street.ts`
`TownStreet`, `shared/chunks.ts`) and drawn merged per 128 m draw group (`render/pc/street.ts`);
the camera roams the land (`shared/focus.ts`), streaming the chunks round it in a big world, with
houses past the stream as boxes with roofs (far LOD), and the lot it settles on becomes the focus.
Measured on an M3-class Mac, Chromium (Playwright's), WebGPU, 1920 × 1080 at DPR 1, a 120 Hz
display, vsync, medians of 2 runs unless said (`tools/bench/render.mjs`).

**Today's towns, before → after** (the base built from the commit before the work; same machine,
back to back):

| scene | preset | scenario | draws | shadow draws | CPU ms / frame | GPU ms |
|---|---|---|---|---|---|---|
| village | medium | orbit-day | 369 → 369 | 60 → 59 | 1.57 → 1.69 | 4.43 → 4.37 |
| village | medium | wide | 430 → 429 | 60 → 59 | 1.64 → 1.72 | 4.55 → 4.88 |
| village | high | orbit-day | 384 → 382 | 61 → 59 | 1.64 → 1.66 | 4.76 → 4.77 |
| village | high | wide | 431 → 430 | 60 → 59 | 1.77 → 1.48 | 4.75 → 4.70 |
| forest | medium | orbit-day | 402 → 400 | 90 → 88 | 1.69 → 1.36 | 4.88 → 4.55 |

120 fps in every cell, no long tasks. Pictures (`--shots`, then `--compare` with the base's):
identical but for the flower beds' shrubs the old street lost (a model both planted as foliage
and placed on another lot had its first batch written over): at most 0.069 % of a picture's
pixels, all at those shrubs; with `residents=lot` (residents only on the viewed lot, as before)
the same. Planning the street for a focus lot: every view of both scenes gives the same vertices
and instances as the old `planStreet`; a focus moving to the next lot plans 2–4 of the village's
6 chunks again (4–30 ms) instead of the whole street.

**Roaming** (`cam=roam`: the camera's target round a loop over the town at 12 m/s, stopping
2 s every 8 s so the lot under it becomes the focus):

- The village (Medium): 119.6 fps, 288 draws, 0–1 frames over 33 ms in 20 s; a focus lot
  settled on is drawn in full 75–175 ms after the camera stops (`render.focusSwitches`).
- `?bench=bigworld` (the village tiled 5 × 5 for the renderer, ~930 × 740 m, a big world):
  see the table below (the first round, before the fixes that followed it, had 96 fps on Medium
  and long tasks of 0.6–0.9 s at each focus switch: `tableRooms` matching every seat of the town
  against every thing, and the town's grass mask made again whole; both fixed).

The big world after the fixes (two runs each, 20 s; `over 33`: frames over 33 ms in the 20 s;
streamed: 18 chunks in detail in 8 draw groups and 42 far groups while roaming):

| preset | scenario | fps | interval p95 / p99 / max ms | over 33 | draws (shadow) | CPU p50 / p95 ms | long tasks |
|---|---|---|---|---|---|---|---|
| low | roam | 117.4 / 117.3 | 9.8–10.3 / 10.3 / 66–109 | 6 / 8 | 241 (0) | 1.3–1.8 / 1.7–2.4 | 1–2, ≤ 59 ms |
| medium | roam | 115.9 / 117.1 | 10.1–10.2 / 16.4–16.7 / 82–93 | 6 / 5 | 334 (47) | 1.5 / 2.3–2.4 | 0–1, ≤ 50 ms |
| high | roam | 112.6 / 111.6 | 10.3–15.3 / 17.0 / 67–92 | 8 / 7 | 336–339 (48–51) | 2.1–2.2 / 3.0–3.1 | 1–2, ≤ 55 ms |
| low | orbit-day | 120.0 | 9.8–10.3 / 10.3 / 10.3 | 0 | 290 (2) | 1.4–2.1 / 1.7–2.6 | 0 |
| medium | orbit-day | 120.0 | 9.7–10.2 / 10.1–10.3 / 10.4 | 0 | 382 (61) | 1.7–2.3 / 2.5–3.2 | 0 |
| high | orbit-day | 115.6–116.2 | 10.3 / 16.7 / 18.7 | 0 | 395 (62) | 1.5–1.8 / 2.0–2.6 | 0 |
| low, CPU ×3 | roam | 110.0 | 10.2 / 25.2 / 191 | 20 | 241 (0) | 3.9 / 5.1 | 10, ≤ 163 ms |

No shader or pipeline is made while it roams (`diag=shaders`: 84 shaders, 134 pipelines at the
start and the end). What is left behind a focus switch, from a CPU profile of the roam (each
under 50 ms, but together the frames over 33 ms): the focus house's storeys (`HousePlanner.build`,
25–40 ms a storey, paced between storeys only), the lot layer's key (`lotKeyOf`, the town's walls
strung), the residents' and the birds' rebuilds, and the bench's own tiling of each world message
(`debug/tileWorld.ts`, ~10 ms). A cached lot layer (the focus lot plus the 2 last, built in
slices that a newer focus cancels) is what would take them off the roaming camera's frames; it is
not done.

Building the big world (the village tiled, headless, measured once): the landscape 4.7 s (its
grass field 3.4 s, a texel a metre over the land), the street 4.4 s (every chunk of the town
planned up front, so streaming only merges and uploads). The village's street: 43 ms planned
whole from fresh, against the old 24 ms per view (it is planned once now, not per view).

**A real Big world** (after merging the worldgen branch: `?bench=bigworld` draws Agent 3's Big
"small" world, 896 m, generated in the page; the roam goes from hamlet to hamlet). The land is
now chunked too: the ground in 128 m tiles of three levels, flat round everything built with
hills and woods between the hamlets, the woods in 256 m squares left out past 560 m, all of them
far versions and half as dense away from what is built. Roaming it (Medium and High, two runs
each): 0 frames over 33 ms, 68–74 fps, but GPU-bound at 11–12 ms a frame (32–35 M triangles,
the woods' most), interval p95 17.5–25 ms, and 640–870 draws (123–361 of them the sun's shadow):
over the 550 / 650 budgets. Today's scenes after all of it (village Medium/High orbit, wide and roam,
forest Medium; two runs each): 120 fps (the village orbit once 118), 0 frames over 33 ms, draws
as before (370 / 430 / 384 / 431, roam 288), the village's focus switches 76–152 ms; pictures
against the base: at most 0.052 % of a picture's pixels (the shrubs), forest 0.018 %.

### The real Big world again: its own focus lot, its zones, its warm-up (2026-11-02)

Same machine and method (`tools/bench/render.mjs`, each run on a copy of the code made for it, so
the code changing meanwhile doesn't reload the page under it). The roam is 60 s now (20 s covered
little more than the way from the main town to the first hamlet: no focus switch at all); `over 33`
is frames over 33 ms in those 60 s, `slow` in the result says when each came and what was being
built (`bench.ts`).

What the 640–870 draws and 11–12 ms of GPU time were: the bench's Big world starts with no lot in
view, and a null view meant the whole town in full detail, every house's storeys (a 68 MB vertex
buffer) and every thing in the world. On Low the things' merged shadow caster took them all too
(a 134 MB vertex and a 128 MB index buffer), and that is what lost Low's WebGPU device (Chromium's
GPU process went down: "A valid external Instance reference no longer exists", then no adapter).
A big world given no view now builds the lot nearest the camera (`setWorld`). Then:

- the lawn's flowers, one draw over the whole world (46,861 of them, 12.5 M triangles), in 128 m
  squares left out past 170 m;
- a house's rooms found over its lot's tiles, not the world's million a storey;
- the warm-up builds and draws up to 6 more lots that between them have most of what the world's
  houses are made of, and a reflection (`warmLots`): roaming made 49 shaders and pipelines in
  play before, 30 of them in the frame the first mirror showed; 0 now;
- Low's merged object shadows uploaded once for all the wall items the cutaway moves together;
- the street keeps each chunk's last three plans (a lot just left is planned already).

Land from the zones (`render/shared/zones.ts`): water (the Big worlds had none), woods, meadows,
fields and hills as the generator laid them out; everything built stays flat at y = 0 (roads,
country roads and trails flat corridors with their verges), so residents and clicks need no lift;
`meta.pads` is not read (countryside plots are built land at 0).

Roaming the real Big worlds now (60 s, one run each; before: the Big small world, 20 s, 2 runs):

| world | preset | fps | interval p95 / p99 / max ms | over 33 | draws (shadow) | triangles M | GPU ms | CPU p50 / p95 ms | focus switches ms |
|---|---|---|---|---|---|---|---|---|---|
| small, before | medium | 71.6 | 17.5 / 25.0 / – | 0 | 692 (175) | 32.7 | 11.1 | 2.2 | – |
| small, before | high | 68.0 | 17.5 / 25.0 / – | 0 | 642 (123) | 31.9 | 11.0 | 2.9 | – |
| small, before | low | device lost in 5 of 6 starts | | | | | | | |
| small | low | 119.9 | 9.0 / 9.3 / 25.0 | 0 | 132 (0.1) | 1.3 | 4.4 | 1.3 / 1.9 | 82, 96 |
| small | medium | 119.9 | 9.0 / 9.3 / 25.0 | 0 | 152 (3.6) | 3.0 | 4.7 | 1.3 / 2.0 | 79, 91 |
| small | high | 119.9 | 9.1 / 9.3 / 25.0 | 0 | 156 (2.1) | 3.1 | 4.6 | 1.3 / 2.0 | 79, 90 |
| lakeside | low | 119.9 | 9.8 / 10.3 / 33.4 | 0 | 111 (0.1) | 1.3 | 4.3 | 1.5 / 2.8 | 106, 77, 73 |
| lakeside | medium | 120.0 | 9.8 / 10.3 / 24.7 | 0 | 132 (10.5) | 2.9 | 4.5 | 1.4 / 2.7 | 83, 73, 64 |
| lakeside | high | 120.0 | 9.8 / 10.3 / 16.7 | 0 | 137 (10.4) | 2.9 | 4.4 | 1.4 / 2.6 | 78, 63, 62 |
| forest | low | 119.9 | 9.7 / 10.2 / 33.3 | 0 | 189 (0.3) | 2.1 | 4.7 | 1.6 / 2.1 | 73, 53, 80 |
| forest | medium | 119.9 | 9.6 / 10.2 / 25.1 | 0 | 206 (6.7) | 4.5 | 4.8 | 1.5 / 2.2 | 95, 65, 57 |
| forest | high | 119.9 | 9.7 / 10.2 / 33.3 | 0 | 236 (6.6) | 5.3 | 5.0 | 1.6 / 2.3 | 78, 53, 85 |
| small, CPU ×3 | low | 119.3 | 10.0 / 10.3 / 107 | 2 | 131 | 1.3 | 4.0 | 1.5 / 2.7 | 215, 252 |
| lakeside, CPU ×3 | low | 119.3 | 10.0 / 10.3 / 107 | 4 | 110 | 1.3 | 4.0 | 1.4 / 2.6 | 223, 217, 149 |
| forest, CPU ×3 | low | 119.4 | 10.0 / 10.3 / 100 | 4 | 188 | 2.1 | 4.4 | 2.0 / 3.4 | 220, 141, 199 |

Low starts every time (10 of 10 on the Big small world, and every run since). With the CPU
throttled 3× the slow frames come with no build under way (`slow`: building '' for all but one),
long tasks 83–87 ms: not found yet. The street's chunk plans (`TownStreet.chunk`) and a storey's
`HousePlanner.build` are still each one synchronous step (12–16 ms on Low at 1×).

Today's scenes, base (the commit before the open-world work) → now, back to back (2 runs):

| scene | preset | scenario | draws (shadow) | CPU ms | GPU ms |
|---|---|---|---|---|---|
| village | medium | orbit-day | 370 (60) → 369 (59) | 1.72 → 1.69 | 4.61 → 4.20 |
| village | medium | wide | 430 (60) → 430 (59) | 1.84 → 1.81 | 4.50 → 4.13 |
| village | high | orbit-day | 382 (59) → 383 (59) | 1.78 → 1.70 | 4.89 → 4.13 |
| village | high | wide | 430 (59) → 431 (59) | 1.88 → 1.84 | 4.71 → 4.21 |
| forest | medium | orbit-day | 400 (88) → 400 (88) | 1.54 → 1.53 | 4.30 → 4.23 |

120 fps in every cell; the village's roam 119.6 fps, 287 draws. Pictures against the base's: the
village at most 0.061 % of a picture's pixels (the flower beds' shrubs, as before; 51 of 54
pictures, 3 timed out loading in the headless browser), the forest 0.018 %.

### Roaming real Big games, not the bench (2026-11-02)

The bench's Big world has no catalog, no world messages and no town going to work. A real one was
played instead (`Create World → Lakeside → Big → Choose a household → Play`, headless Chromium on the
M3 Pro, 1440 × 900, no vsync), the camera flown hamlet to hamlet for 90 s, a new world each run,
the frames timed with the renderer's frame probe and the long animation frames (LoAF) with their
scripts. What held frames up, and what was done:

- **The catalog's pictures drawn ahead of time** (`render/preview/items.ts`): for about 15 s after
  a new game, a model built, its pipelines made on a device of its own (compiled in the same GPU
  process) and a read-back a picture; a 200 ms frame among them (a picture's encoding), and 14
  frames over 33 ms on Low at 3× CPU slow-down. They now wait while the game is in Live
  (`holdAhead`) and are drawn in Build and Create, where the game is paused.
- **Low's merged object shadows uploaded whole** for every chair pulled up to the table, every
  frame while someone sits down: 3.2 MB a frame on a big world's home lot, 425 uploads in one roam.
  The copies are now split over meshes of 32 768 vertices, each uploaded alone (8–9 shadow draws
  for a big lot, 5 for the forest scene's instead of 4; pictures as before).
- **A resident coming or going rebuilt the whole lot** (a world message whose sim count changed:
  the town's staff at noon, a visitor): the lot's objects, street, cars and birds, frames of 44 and
  74 ms. The residents are now built again alone (`buildSimsOnly`: 7 ms against 22–48 ms of builds).
- **Every world message compared 6 000 objects as JSON** (4–5 ms of each, several a second while
  quiet hours are skipped): field by field now, 1.5–2 ms.
- **The storey in view followed a resident upstairs** while the camera was hundreds of metres away,
  each change a build of the lot's objects and residents (20–60 ms, a 42–48 ms frame): only within
  40 m of the lot now.

Left: frames of 60–125 ms, two or three together, about 86 s into the roam (09:35–09:45 on the
first day, a few seconds after the town's residents got their jobs at 09:30) in roughly every other
run, on Low and Medium, WebGPU and WebGL2 alike. Nothing is built then and the main thread is idle:
the LoAF holds no script, and a GPU-process trace shows the renderer submitting a frame and then
nothing for 75 ms until the GPU hands the frame back; our passes' GPU time (timestamps) stays at
2–3 ms. It comes about 125–135 s after the browser started, not at a time of the game's: with
60 s spent on the main menu first, 5 roams of 5 had no frame over 33 ms (none at 86 s, none at the
26 s that would have been), against about every other one without. Something of a fresh browser
profile's GPU process then (not found which; the game can't avoid it), not of the game. A model file streamed in mid-roam is parsed on the main thread
(`XMLHttpRequest` callback, 31–45 ms) and makes a 40–60 ms frame now and then; the storey still
follows residents at home (a 42 ms frame).

Each roam, 90 s (before: the open-world branch's tip; after: this work):

| run | fps | p95 / max ms | over 33 | draws (max) | CPU mean / p95 ms | GPU ms |
|---|---|---|---|---|---|---|
| Medium, before (4 worlds) | 66–72 | 19.9–21.9 / 31–112 | 0, 0, 0, 4 | 264–317 (705) | 1.95–2.45 / 3.3–3.9 | 2.0–2.4 |
| Medium, after (3 worlds) | 55–72 | 21.6–24.6 / 32–61 | 1, 0, 5 | 291–301 (653) | 2.0–2.75 / 3.5–6.1 | 2.3–4.0 |
| Low, before (5 runs) | 70–76 | 19.7–21.5 / 31–126 | 3, 0, 7, 0, 1 | 223–242 (386) | 1.7–2.1 / 3.2–3.4 | – |
| Low, after (3 worlds) | 67–76 | 14.1–22.4 / 33–94 | 2, 1, 0 | 211–226 (551) | 1.67–1.80 / 2.9–3.9 | – |
| Low, CPU ×3, before | 58.0 | 26.6 / 111 | 31 | 226 | 5.9 / 10.5 | – |
| Low, CPU ×3, after | 68.6 | 22.7 / 67 | 9 | 244 | 4.9 / 8.3 | – |
| WebGL2 Medium, before | 74.3 | 20.4 / 39 | 1 | 265 | 1.75 / 3.1 | 3.1 |
| Medium, after, 60 s on the menu first (5 worlds) | 70–77 | 14.2–22.4 / 31–32 | 0, 0, 0, 0, 0 | 275–298 (623) | 2.0–2.1 / 3.4–3.5 | 2.3–2.5 |
| WebGL2 Medium, after (2) | 74–75 | 19.9–20.4 / 32–124 | 3, 0 | 271–286 | 1.7–1.8 / 2.9–3.2 | 3.1–3.2 |

(Low's GPU time from the timestamps spans the passes and what lies between them on Low: not a cost.
The third Medium run after came while Spotlight took a core: 55 fps.) Low started every time, from
the setting as a player sets it, with no device lost.

Today's scenes, base → after (WebGPU, orbit by day and night, 2 runs): the same draws in every
cell but Low's shadow draws in the forest (4 → 5); CPU within 0.05 ms; GPU 0.1–0.8 ms more in the village
(run second; nothing in it changed for Medium or High) and within 0.1 ms in the forest, inside the
±1 ms the M3 varies by between runs. Pictures against the base's: the village at most 0.059 % of a picture's pixels, the
forest 0.000 %. The bench's Big lakeside and forest roams: the same draws, fps and slow frames
(one 35–42 ms frame on Low at the same moment in both).

### The worker's world messages (0.6.0)

A Big world's structure changed with every door opened, crop grown or thing worn: some 250 times a
game day, several times a real second at speed 3. Each change sent the whole world without its lot
(2.2 MB of JSON: 16 000 things, every plot with its house scanned, 230 KB of `meta`), and each focus
lot the whole lot as well (11 MB, 7.2 MB of it `rooms` as a JSON array of 3.7 million numbers, 0.85%
of them not zero). The main thread parsed and cloned each, and after a long frame ran the whole
handler once per message queued. Now (`view::StructureSync`, `core/bridge.ts` `mergeWorld`):

- **Only what changed is sent**: the parts that changed, things by index while the list of things
  is the same, `meta` once. The bridge keeps everything else the same object, so the session and
  the renderer compare by reference (residents' looks are kept per resident and look the same object
  while they're made from the same seed and stage). Per change, measured in Bun on the sample Big
  farm world: 2.2 MB → 2.6 KB; parse 3.5 ms → 0.02 ms, clone 11 ms → 0.04 ms. The worker makes an
  update in about the time it made the lean world (5–6 ms in wasm, 2.5 ms native; the plots are made
  again only when the lot or a plot changes).
- **A new focus lot sends only its meshes** (the walls, rooms and floors are the whole town's, which
  the main thread has); **the rooms go as a transferred `Uint16Array`** when the lot changes. A lot
  change's JSON is 1.4 MB (walls, openings, fences, diagonals, floors) instead of 11 MB.
- **At most every 100 ms** from the worker (a command's and a new view's at once), and **applied
  once a frame** on the main thread (`sync`), the newest merged; without frames, 100 ms later.
- **World messages say which view they were built for** (`setView`'s `seq`): one built for the lot
  before is held until the new lot's comes, instead of being shown as the new lot's.

## Loading: the boot, step 2's portraits, a household's entry (2026-11-13)

What a new player waits for before playing, timed through the menus as a player goes
(`tools/bench/loading.mjs`: a new browser context a run, the page's `Math.random` seeded so every run and
build makes the same showcase, world and households; Medium/WebGPU, headless on the M3 Pro, the dev server,
medians of 3 first visits; other work ran on the machine meanwhile, so single runs vary by ±2 s). Every
`debug/perf.ts` span now also goes to the dev play log (`perf` lines), with a span per portrait batch, per
studio build and per lot the warm-up builds.

| moment | before | after |
|---|---|---|
| boot: the page to the main menu | 16.3 s | 10.8 s |
| step 2: its portraits in view shown | 1.9–4.5 s | 1.3 s |
| step 2: all 118 portraits shown | 8.1 s | 2.6 s |
| entry: Play to the game up | 17.9 s | 12.6 s |

Where the time went, and what changed:

- **Boot drew 118 portraits nobody saw** (`boot: portraits`, up to its 8 s cap; 5.3 s median). The main menu
  shows none, the showcase households are dropped when a new game starts, and their keys (`lookKey` of the
  look without its life stage) never matched what the screens ask for. Now one face, at its age
  (`lookAtAge`), warms the studio up (0.5 s), and the menu no longer queues the same 118 again.
- **A batch of portraits took 0.4–2.4 s.** Its 16 Sims were put on their rows two a frame, each time
  building all of them again (`studio: build` 1, 2, … 16 rows, eight frames before the first draw), and
  each frame's six draws waited for the last six to be read back (one target per camera). Now the batch's
  Sims are built at once (`Studio.setRows`, 80–130 ms for 16), each row draws into a target of its own so
  the next draws don't wait, and a batch holds 32 (about what step 2's list shows), 8 drawn a frame:
  0.3–0.6 s a batch. What is on screen goes first (`SimPreview` watches itself with an
  `IntersectionObserver`: `priority` `now`, the rest `idle`), a face that left or changed withdraws its
  request (`signal`), and the studio is kept from the boot on (`keep(true, 'page')`; the game kept it
  anyway) rather than made again 4 s after each screen's last face. Portraits are kept between visits in
  Cache Storage (`PortraitStore`, keyed by `lookKey` and the character files' contents), as the catalog's
  pictures are.
- **Entering a household**: its world build (`session: renderer setWorld`) ran in 4 ms slices, a frame's
  wait after each (`pace`), behind a loading screen that covers it: 8.4 s. Behind the loading screen
  (`Renderer.setCovered`, and the warm-up) the slices are 40 ms: 5.6 s. The warm-up's lots got the same
  (`session: warm game` 4.3 → 2.1 s; a lot 0.2–0.25 s, the first 0.4–0.75 s), a lot that adds fewer than
  `WARM_MIN_GAIN` (3) new kinds is no longer built (in the seeded world all six still are: they add 46–56,
  29, 21–24, 18–19, 15 and 11–12), and their things' model files come with the house's download
  (`gameAssetPlan`'s `warm`). The coverage stays: fewer lots would bring the pipelines back to the first
  roam. The town's other faces wait until the game is
  ready (drawn while it loaded, 4 at a time, they took frames from its build).

Left as they were: step 1's and step 2's town builds (which overlap
when the player goes on quickly), and the session's `SimBridge.start`
(1.7–1.9 s). The warm-up still builds the home again after its lots (0.2 s): building them before the home
would change where the camera is first placed and which build waits for the birds, for little.

### The main menu's town (2026-11-13)

Measured as above (`tools/bench/loading.mjs`, medians of 3 first visits), and one boot at a time with
the page's spans, its long tasks and a CPU profile of the main thread from the page opening to the menu
(the same seeded page, Medium/WebGPU, headless on the M3 Pro, the dev server; two other agents' builds
and benches ran on the machine meanwhile, load average 4–16). The page's dice are drawn in another order
now, so the seeded page's showcase is another town: the comparison below makes the same town in both
builds (its seed, 2032020577, set by hand for the measurement), and the menu was shot in both with
reduced motion (no camera sway): the two pictures differ in 0.001 % of their pixels by more than 8
levels and in none by more than 24 (who lives in the town is rolled at another moment, so a lot or two
can look lived in differently).

| moment | before | after |
|---|---|---|
| boot: the page to the main menu, the same town | 10.3 s (9.7 s at a load of 11–16) | 6.5 s (6.9 s at 11–16) |
| boot, the town the seeded page makes now | | 5.7 s |
| the main thread's long tasks before the menu | 5.3–5.6 s | 2.4–2.7 s |
| `boot: menu town` (spans) | 8.8 s | 4.9 s |
| `menu: showTown`, the renderer's build | 5.8 s | 3.1 s |

Where the boot went, before (one boot's spans and the CPU profile; the page's own time, from 0.2–0.3 s):

| what | time | where |
|---|---|---|
| the showcase made (`generateNeighbourhood`: a Big world, a house for every lot, its furniture, its people) | 2.4 s | the page, one long task |
| its preview (`previewWorld`) | 0.55 s | the page |
| the character set's clips baked (`buildClips`, the renderer's characters starting) | 0.9 s | the page, one long task inside the town's build |
| the landscape: terrain tiles 1.4 s, woods and flowers 0.55 s, grass field 0.9 s | 2.9 s | mostly waiting on the render data worker: its grass field 1.16 s, then its 144 terrain tiles 0.41 s, one after the other, the page's woods and flowers after both |
| the overview: its street planned 1.4 s, meshes and models 0.3 s, roads and the lots' things 0.5 s | 2.4 s | the page, only once the landscape was done |
| the first frames (the town's shaders) | 0.5 s | the page |
| the portrait studio's warm-up | 0.5 s | the page, after the town |
| waiting for a frame between slices (`pace`, 8 ms slices) | 2.1 s idle | |

What changed:

- **The showcase is laid out in the town worker** (`townJobs.ts`: a `land` job, then a `preview` job),
  sent as soon as the game data is in, and peopled on the page in between (`populate`, 15 ms, on the
  page's dice as before, so a seeded page still makes the same showcase every run). The page starts the
  renderer and warms the portrait studio up meanwhile (now before the town: a face of its own,
  `lookFromSeed`, so it takes none of the dice the showcase is peopled with; the studio is kept from then
  on, or it went in the 4 s it was idle and step 2 made it again); the character set's clips are baked
  in that time too.
- **The landscape and the overview are built side by side** (`showTown`: `both`), each giving the frames
  a turn (`pace`); the overview's planning fills the time the landscape waits on the workers. Within the
  landscape the woods and flowers are planted while the terrain's tiles come (`nature.ts` `buildAll`).
- **Two render data workers** (`renderData.ts`): one shades the terrain, the other makes the grass field,
  each with the land laid out (0.12 s); and 16 tiles are asked for ahead, not 4, so the shading keeps
  going while the page is busy. The grass field (1.16 s in its worker) no longer holds the terrain up.
- **Longer slices behind the boot's loading screen**: the town's build runs in 40 ms slices
  (`setCovered`), as a game's does behind its loading screen.
- **A chunk of the street is gathered once**: its key and its plan were each worked out from the same
  gathering (`TownStreet.gather`, 0.34 s in all); the plan now takes what the key's gathering made.

After, one boot (the same town): data in at 0.25 s; the worker's land at 2.7 s and its preview at 3.3 s,
the page meanwhile starting the renderer and warming the studio up (1.2 s, with the clips), then idle
for 1.8 s; then the town's build, 3.1 s, the page busy nearly throughout (0.35 s idle): the street
1.2 s of the main thread (planning 0.6, meshes 0.5), the whole town's road mesh 0.3 s (`roadMeshes`, one
long task), the grass's town mask 0.34 s (`withTownGrass`), the landscape's layout 0.12 s, GC 0.3 s.

Also faster since: a household's entry (Play to the game up) 12.1–13.9 s → 8.6–10.7 s. The town
worker, which made the showcase's houses, still has them when the game's town file is made
(`session: assembleTown` 1.9 → 0.6 s), and the game's own landscape builds use the same overlap.
(The households of step 2 are rolled from other dice than before, so this is not the same household.)

Left:

- **The 1.8 s the page waits for the showcase's land** (the worker's 2.5 s: its houses and their
  furniture are most of it, and each lot's size waits on its house). The landscape could start from the
  land before it is peopled (its terrain, woods and grass don't depend on who lives there), and the
  town's model files and shaders could be made ready meanwhile.
- **The page's part of the town's build**: the street's planning and meshes (1.2 s), the road mesh
  (0.3 s, could be made in the render data worker) and the grass's town mask (0.34 s, needs the town's
  walls and rooms, which stay on the page).
- **A returning visit** makes a new showcase; one kept from the last visit (Cache Storage, by build and
  seed) would show the same valley again with no layout at all.
- Step 2's portraits: all of them in 2.3 s (1.8–2.2 before; those in view 1.0 s, 1.1–1.3 before). Step 2's town
  builds on the page while they are drawn; the step-2 households differ between the two builds, so this
  is not settled.
