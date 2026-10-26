# Where the time goes (measured 2026-10-22)

Why the game stuttered, measured and then fixed on the branch `perf/stutter-measurements`. The short
answer: **steady rendering was not the problem** — the renderer holds 120 fps with no long tasks. The
hitches came from *rebuilding the whole rendered world* whenever the simulation said the structure
changed, from the background asset stream doing the same when a file arrived after Play, and from
Babylon making reflection pipelines in the first frame that needed them. All three scaled with how
fast the game was played.

Measured first (sections below), then fixed; see [What was fixed](#what-was-fixed) for the changes and
the after numbers.

*All of this was measured on the Babylon.js 9.30 renderer, which drew the game until 0.3.2. The
files and lines named (`BabylonRenderer.ts` and the rest of `web/src/render/babylon/`) were removed
by the move to PlayCanvas; they are in git history up to f15b839b. The engine comparison that chose
PlayCanvas, and the port's own measurements, are in [The PlayCanvas port](#the-playcanvas-port).*

## How it was measured

Two instruments, both in a real browser on the real GPU, 1920×1080 at device pixel ratio 1:

- `tools/bench/render.mjs` — the existing `?bench=<scene>` matrix (fixed camera, no UI, no
  streaming). This is the floor: what a frame costs when nothing changes.
- `tools/bench/profile.mjs` (new) — opens the *real game* on the dev server (`?state=<name>&profile&debug`),
  presses a speed key, records its own `requestAnimationFrame` gaps for `--secs`, then reads
  `window.__perfReport()` (`web/src/debug/perf.ts`). It writes `frames.json`, `perf.json` and a
  `report.md` (gap percentiles, counts over 20/33/50/100 ms, dropped frames and long tasks by
  phase, worst frames, slowest spans) to `bench-results/profile-<time>/`.

State used: `debug-states/readme-village.json` — the full town, a household of six, the whole town
streamed in the background. Preset Medium, WebGPU, `?profile&debug` on, HUD on.

```sh
PLAYWRIGHT=/private/tmp/idyll-pw/node_modules/playwright/index.mjs \
  bun tools/bench/profile.mjs --state readme-village --speed 1 --secs 90
```

## The floor: fixed-camera benchmark

Production build, village scene, no vsync, medians of 3 (`bench-results/2026-10-22-09-35-31`):

| preset / backend | fps | p50 / p95 / p99 ms | CPU ms (update + draw) | GPU avg / p95 ms | long tasks |
|---|---|---|---|---|---|
| low WebGPU | 240.0 | 4.1 / 5.1 / 5.9 | 3.94 (0.20 + 3.75) | 2.53 / 3.01 | 0 |
| medium WebGPU | 158.1 | 6.1 / 8.0 / 9.9 | 6.02 (0.26 + 5.76) | 4.68 / 6.00 | 0 |
| high WebGPU | 167.2 | 5.7 / 7.7 / 8.5 | 5.69 (0.26 + 5.43) | 4.45 / 5.67 | 0 |
| low WebGL2 | 335.3 | 2.9 / 4.1 / 4.7 | 2.80 (0.24 + 2.56) | 3.11 / 3.84 | 0 |
| medium WebGL2 | 226.1 | 4.3 / 6.1 / 7.4 | 4.19 (0.35 + 3.85) | 5.03 / 6.36 | 1 |
| high WebGL2 | 190.4 | 4.9 / 7.5 / 8.6 | 4.96 (0.41 + 4.52) | 6.01 / 8.08 | 0 |

p99 is about 1.4× p50 and there are no long tasks: the draw path itself is smooth at every preset.

## The real game: hitches scale with sim speed

Same state, Medium/WebGPU, 60 s each after Play (`bench-results/profile-2026-10-22-10-*`):

| speed | fps | p50 | p95 | p99 | max | frames > 20 ms | > 33 ms | > 50 ms |
|---|---|---|---|---|---|---|---|---|
| 1× | 117.3 | 9.3 | 18.1 | 23.1 | 154 | 167 | 4 | 2 |
| 3× | 110.6 | 10.2 | 19.4 | 31.6 | 123 | 299 | 45 | 2 |
| 5× | 94.5 | 11.2 | 29.4 | 40.8 | 99 | 701 | 188 | 10 |

At 1× a hitch over 33 ms is rare (4 in 7037 frames). At 5× it is 3.3 % of frames. A 90 s run at 5×
(`bench-results/profile-2026-10-22-09-47-56`) had 142 frames over 33 ms and a **925 ms freeze**:

```
- 925 ms at 74.52 s (session: warm game (deferred))
- 400 ms at 57.55 s (session: Ready)
- 133 ms at 74.33 s (render: object shadows)
- 117 ms at 54.87 s (render: slow frame)
```

## Cause 1 — a structural change rebuilds the whole rendered world

The simulation bumps `structure_version` for anything that changes how the lot looks or is walked:
objects opened/closed or clothes changed (`crates/sim-core/src/world.rs:2492`, `reshaped`), crops
changing growth stage (`world.rs:3077`), walkability (`world.rs:2706`, `nav_changed`), residents
added or removed (`world.rs:2272`, `world.rs:2357`). The worker posts a `world` message on any
bump (`web/src/core/sim.worker.ts:249-262`), the bridge hands it to the game
(`web/src/core/bridge.ts:403-409`), and the session's `onWorld` handler (`web/src/game/session.ts:979`)
ends in:

```ts
const build = homeAssets.then(async () => {
  const buildSpan = perfStart('session: renderer setWorld');
  try { return await renderer.setWorld(w, view); } finally { perfEnd(buildSpan); }
});
```

That expression is built on **every** world message. `setWorld` coalesces concurrent calls into a
`nextWorld` loop (`web/src/render/babylon/BabylonRenderer.ts:923-936`), but each build still runs
`buildOne` (`:947`) which rebuilds the street, the object layer and the residents. Cost measured
in play (after the initial world is up), one minute each:

| speed | rebuilds in play | durations |
|---|---|---|
| 1× | 0 | — |
| 3× | 3 | 35, 59, 16 ms |
| 5× | 14 | 11–129 ms |

So in a busy household at 5× a fridge door or a cupboard opening costs 11–129 ms of main thread,
several times a minute; the cost is the *whole* lot, not the one object that changed. At 1× it is
rare, which matches "sometimes" — the hitch comes with a burst of household activity.

## Cause 2 — the background asset stream rebuilds the same world again

After Play the session prefetches the rest of the town (`web/src/game/session.ts:1534-1545`), two
files at a time (`web/src/assets/stream.ts`). Each arrived model calls `fileArrived`
(`BabylonRenderer.ts:4491`), debounced 1.5 s with a 5 s cap, then `refreshModels` (`:4514`) invalidates
the templates and, if the world draws them, calls `setWorld` again. The worst case measured, at
74.18 s of the 5× run:

```
session: renderer setWorld   356.9 ms
  render: buildWorld         356.3 ms
    render: street furniture 161.2 ms
    render: objects          187.7 ms   (object templates 123.3 ms)
  session: warm frames      1004.8 ms   → the 925 ms dropped frame
```

The street's models arrive last, so the last rebuild is the most expensive: it builds the street
furniture and every object template with their real models and then warms the new shader/pipeline
variants. That is the "freeze a minute into play" a player sees.

A/B with the post-reveal `.glb` requests aborted: the stalls shrink but do not vanish — a 144 ms
`render: street furniture` rebuild still happened at 71 s from a *structural* change (Cause 1). Both
paths rebuild everything.

## Cause 3 — the first draws after a rebuild are slow

`render: scene.render` (Babylon's `scene.render()`, `BabylonRenderer.ts:3069-3079`) normally costs
~6 ms in game. After a rebuild that brought new models it ran 30–45 ms for 2–3 s stretches
(61.8–64.2 s and 72.0–74.2 s of the 5× run), i.e. 22–28 fps for seconds after a streamed batch. A V8
CPU profile of the same scene (`/tmp/cpu.cpuprofile`, 45 s at 5×) puts the main-thread time in
Babylon's per-draw path rather than in game code:

```
_renderForCamera 1004 ms   _draw 1513 ms   _isReady (clustered lights) 981 ms
render (mesh) 844 ms       _evaluateActiveMeshes 841 ms
_setVertexState 734 ms     _buffersEqual 686 ms   updateUniform 348 ms   setFloat4 387 ms
```

New materials mean new uniform buffers and bind groups per draw (`getBindGroups`, `_updateOwnerKeyed`),
and the pipeline variants compile around the same time. Treat the exact attribution as probable, not
proven: it did not reproduce in every run (a second 5× session stayed at ~180 ms/s in `render`).

## Cause 4 — boot (not play)

The 2–2.4 s of long tasks in every run are all before Play, in the loading screens: drawing portraits
807–954 ms, starting the simulation 358 ms, `warm game` 383–400 ms, town nature 236 ms, town overview
156 ms, world build 130 ms. They are what the loading screen is for; they do not stutter gameplay.

## Blind spot in the existing instrumentation

`web/src/debug/perf.ts:49` records a dropped frame only at `DROP_MS = 50`. On a 120 Hz display a
dropped frame is 25–40 ms, so most of what a player feels is invisible to `?profile` and to the F3
overlay — the benchmark can look clean while the game hitches. Counts over 20 ms are the useful
signal (the profiler does report them).

## What was fixed

The four causes and the blind spot, on this branch:

1. **Patch the world instead of rebuilding it.** `BabylonRenderer.setWorld` now compares the incoming
   world with the one it built (`unpatchable`) and, when only objects opened/closed or crops grew,
   applies it in place: `patchWorld` recomputes the picking box of every placed object
   (`pickBox`, now shared with the build) and `objectMotion.patch` moves `open`/`stage` on the live
   motions — the clips are read each frame, so doors swing and crops advance with no rebuild. The
   comparison ignores what does not shape the mesh (`wear`, `quality`, `sellValue`, `repairCost`) and
   the lot by `lotVersion` only (lean world messages re-parse `plots`/`meta`, so references never
   match). A patch costs **0.2–0.6 ms**; rebuilds in play at 5× went from 14 a minute to 0.
2. **Hold the stream's rebuilds until the town has streamed**, then build once
   (`renderer.holdModelRefresh` from the reveal tail in `web/src/game/session.ts`), and **pre-build the
   arrived templates a frame at a time** before that rebuild, so the models that came with the stream
   are not all built in its first frame.
3. **Warm the reflections where the pipelines are made.** The menu warmed them through `warmGame`,
   which the in-game session never runs; `buildOne` now sets `warmMirrors`, and the frame loop calls
   `mirrors.warm(0)` once after each build, drawing every caster into the reflection (with
   `warmAll`) so no material's pipeline is created mid-play. Build helpers are no longer reflected
   (`HELPER_MESH`), and Medium's `mirrorRate` is 10 instead of 30 (it was 30 at 30 fps; at 120 fps
   that was a reflection every 4 frames for no visible gain).
4. **Warm the save thumbnail at boot** (`session: warm thumbnail`, 336 ms inside the loading screen)
   instead of on the first autosave, and capture it from a kept render target rather than a fresh
   screenshot of the whole scene.
5. **`DROP_MS` 50 → 20** in `web/src/debug/perf.ts`, so `?profile` records the 25–40 ms hitches a
   player feels on a 120 Hz display.
6. Boot portraits are posed two a frame (`POSE_PER_FRAME`) and the batch draws one portrait on its
   first frame, which spreads the work; the remaining boot long task is the first draw of the
   portrait target (cold pipeline), not the posing.

After, same state, Medium/WebGPU, 60 s after Play (`bench-results/profile-final*`):

| speed | fps | p50 | p95 | p99 | max | > 20 ms | > 33 ms | > 50 ms |
|---|---|---|---|---|---|---|---|---|
| 1× (90 s) | 119.9 | 8.3 | 8.8 | 9.3 | 16.8 | 2 | 0 | 0 |
| 5× | 114.2–116.7 | 8.3 | 9.0–9.2 | 16.7–25.0 | 108–141 | 88–101 | 36–49 | 2 |

The 925 ms freeze is gone, the once-a-minute autosave hitch is gone (it is a 336 ms warm at boot
now), and the mirror no longer makes a pipeline in play.

### What is left

- **The one stream-settle rebuild**, once per load (~25 s in, when the rest of the town has streamed):
  118 ms, of which `mirror` pipeline creation for the newly streamed models is 94 ms. It happens in the
  same frame because the new world is drawn the moment the build finishes. Spreading the reflection
  warm across frames would need a hidden warm target, so the reflection pipelines are made for the
  target format without drawing into the visible mirror.
- **Night variants at 5×** (`glow`, `simBlanket`, `gardenPools`, `simShadows` are enabled only later in
  the day): 50–82 ms frames as their reflection pipelines are made, three or four times in a 5× minute.
  At 1× they are spread over the day and did not show in a 90 s run (max 16.8 ms).
- **Boot**: portraits 819 ms, `warm game` 503 ms, world build 1421 ms, town overview 442 ms — all in
  the loading screen, which is what it is for.
- **Structural rebuilds still rebuild the lot** when the structure really changes (storey switched,
  residents in or out, furniture added): 11–45 ms, rare.

The remaining big win would be patching streamed models into the stand-in meshes instead of building
again; the patch path above makes that less urgent.

## Steady state: the shadow map and the mirror picking (2026-10-22)

The floor above is draw-call bound: 476 draws at about 8.5 µs each (5.8 ms of the frame), while 1.5 M
triangles cost 0.30 ms (`diag=nonature`) and MSAA 0.18 ms (`diag=noaa`). Two of those costs were paid
for nothing.

**The sun's map was drawn every frame.** `SunShadows` draws it again when something in it moved, and
a Sim's every pose counts as a move; the pose redraws were capped only by `?shadowrate`, which
defaulted to 0 (every change). So 181 casters, 0.9 ms of CPU and 0.25–0.6 ms of the GPU, every frame
of play. The presets now carry `shadowRate` (Low 15 Hz, Medium and High 30 Hz); `?shadowrate` still
overrides it. The sun's step, the walls, a thing placed or hidden still draw it at once, and nothing
draws it while the game is paused.

**The mirrors picked their glasses every frame.** `Mirrors.update` placed every mirror in the world,
ranked them, worked out which the camera could see and which reflections to redraw — on every frame,
though a reflection is drawn at most `rate` times a second (10 on Medium) and none of that work is
needed on the frames between. It now returns at once unless a reflection is due and the camera (or a
resident in its view) moved since it was drawn; `changed` (the light, lamps, things placed, a door
opening) is never held back, so a glass still follows its door. A held reflection is looked up
through the camera it was drawn for, so it stays on the glass.

Interleaved A/B, three runs each, production build, village orbit by day, no vsync:

| | fps | p50 ms | CPU draw ms | GPU ms | draws (shadow) |
|---|---|---|---|---|---|
| before | 162.9 / 167.2 / 175.6 | 5.6–6.0 | 5.18 / 5.44 / 5.55 | 4.33 / 4.43 / 4.72 | 476 (181) |
| after | 183.9 / 205.3 / 205.9 | 4.7–5.4 | 4.37 / 4.41 / 4.91 | 3.68 / 3.76 / 4.23 | 328–331 (33–37) |

About a millisecond of the frame and 22 fps back, and the picture at rest is unchanged: the mirror
camera's frame differs in 337 of 2 073 600 pixels (0.016 %), none of them on the glass. The mirror
scene (`cam=mirror`) went from 145.7 fps to 161–180.

## The PlayCanvas port

After 0.3.2 the renderer moved from Babylon.js 9.30 to PlayCanvas 2.23 (how it is built: [graphics.md](graphics.md#the-renderer)). Two sets of numbers led there and followed it: the engine comparison that chose PlayCanvas, and the checkpoints the port was held to (main-thread time at or below Babylon's, GPU time within +10 %).

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

### Checkpoints during the port

`render.mjs` on a production build, the `village` scene at 1920×1080 and device pixel ratio 1, vsync, 20 s after a 5 s warm-up, one run per cell, on the M3 Pro (Chrome 156); Medium and High on both backends. CPU is the main thread's frame (the game's update and the renderer's draw), GPU the frame's GPU time as each renderer measures it (Babylon: its timestamp span from the first pass to the last; PlayCanvas: its GPU profiler), draws all passes. Every cell ran at 116–120 fps, held by vsync. The PlayCanvas runs' split of draws and triangles by pass (shadow, mirror) wasn't reported yet.

**Stages 5 and 6, without residents** (`diag=nosims`): Babylon (measured alongside Stage 5) against Stage 5 (33cf3230: the lot without its residents; no lamps, post-processing or night grade yet, the sun's shadows as Stage 3 had them) and Stage 6 (eed0cdfb: lighting, shadows and the look). High's two mirrors are in Babylon's numbers (13 draws) and in neither stage's (mirrors came in Stage 9).

| Preset, backend, scenario | CPU ms: Babylon → Stage 5 → Stage 6 | GPU ms: Babylon → Stage 5 → Stage 6 | Draws: Babylon → Stage 5 → Stage 6 |
|---|---|---|---|
| Medium, WebGPU, orbit-day | 4.32 → 1.03 → 1.31 | 7.47 → 4.01 → 3.87 | 302 → 191 → 191 |
| Medium, WebGPU, orbit-night | 4.40 → 1.07 → 1.37 | 7.31 → 4.00 → 3.62 | 310 → 197 → 197 |
| Medium, WebGPU, dusk | 4.80 → 1.69 → 1.63 | 7.05 → 3.31 → 3.13 | 354 → 227 → 229 |
| Medium, WebGPU, build | 4.27 → 1.05 → 1.29 | 7.51 → 3.92 → 3.40 | 288 → 179 → 180 |
| Medium, WebGPU, wide | 4.79 → 1.22 → 1.33 | 7.22 → 4.41 → 4.01 | 352 → 241 → 242 |
| High, WebGPU, orbit-day | 4.68 → 1.08 → 1.34 | 7.48 → 4.05 → 3.91 | 314 → 190 → 192 |
| High, WebGPU, orbit-night | 5.02 → 1.16 → 1.34 | 7.38 → 4.07 → 3.64 | 322 → 197 → 197 |
| High, WebGPU, dusk | 5.38 → 1.68 → 1.68 | 7.02 → 3.19 → 3.42 | 365 → 229 → 230 |
| High, WebGPU, build | 4.29 → 1.11 → 1.30 | 7.53 → 3.97 → 3.69 | 288 → 179 → 180 |
| High, WebGPU, wide | 4.77 → 1.10 → 1.35 | 7.54 → 4.14 → 4.15 | 353 → 241 → 242 |
| Medium, WebGL2, orbit-day | 2.50 → 0.82 → 0.82 | 5.29 → 4.51 → 4.84 | 302 → 190 → 191 |
| Medium, WebGL2, orbit-night | 2.78 → 0.81 → 0.90 | 5.82 → 4.44 → 4.91 | 310 → 197 → 198 |
| Medium, WebGL2, dusk | 3.22 → 1.36 → 1.13 | 5.23 → 4.24 → 4.58 | 353 → 227 → 228 |
| Medium, WebGL2, build | 2.11 → 0.76 → 0.73 | 4.90 → 4.60 → 5.13 | 288 → 179 → 180 |
| Medium, WebGL2, wide | 2.93 → 0.79 → 0.92 | 5.98 → 4.66 → 4.73 | 353 → 241 → 242 |
| High, WebGL2, orbit-day | 2.53 → 0.76 → 0.79 | 5.30 → 4.59 → 4.99 | 314 → 190 → 192 |
| High, WebGL2, orbit-night | 3.14 → 0.81 → 0.80 | 5.82 → 4.43 → 4.90 | 323 → 197 → 198 |
| High, WebGL2, dusk | 3.41 → 1.34 → 1.22 | 5.61 → 3.81 → 4.58 | 365 → 229 → 230 |
| High, WebGL2, build | 2.00 → 0.80 → 0.76 | 4.94 → 4.53 → 4.89 | 288 → 179 → 180 |
| High, WebGL2, wide | 2.95 → 0.78 → 0.93 | 6.00 → 4.62 → 4.62 | 353 → 241 → 242 |

**Stage 7, with the residents** (6439e874): Babylon's baseline taken before the port (`diag` none; its run log wasn't kept, its matrix and flags are the same) against Stage 7. Again no mirrors on PlayCanvas yet, so High's reflections are only in Babylon's.

| Preset, backend, scenario | CPU ms: Babylon → Stage 7 | GPU ms: Babylon → Stage 7 | Draws: Babylon → Stage 7 |
|---|---|---|---|
| Medium, WebGPU, orbit-day | 4.74 → 1.70 | 7.33 → 4.60 | 346 → 307 |
| Medium, WebGPU, orbit-night | 4.90 → 1.54 | 7.18 → 4.54 | 355 → 313 |
| Medium, WebGPU, dusk | 5.36 → 2.08 | 6.97 → 3.71 | 370 → 326 |
| Medium, WebGPU, build | 4.29 → 1.40 | 7.51 → 4.20 | 295 → 256 |
| Medium, WebGPU, wide | 4.89 → 1.56 | 7.38 → 4.52 | 392 → 352 |
| High, WebGPU, orbit-day | 5.01 → 1.68 | 7.28 → 4.39 | 357 → 307 |
| High, WebGPU, orbit-night | 5.03 → 1.58 | 6.90 → 4.20 | 367 → 313 |
| High, WebGPU, dusk | 5.59 → 2.07 | 7.03 → 3.44 | 381 → 325 |
| High, WebGPU, build | 4.27 → 1.47 | 7.51 → 3.78 | 295 → 256 |
| High, WebGPU, wide | 4.86 → 1.61 | 7.39 → 3.72 | 392 → 352 |
| Medium, WebGL2, orbit-day | 2.48 → 1.14 | 5.10 → 4.57 | 345 → 307 |
| Medium, WebGL2, orbit-night | 2.91 → 0.98 | 5.89 → 4.78 | 355 → 313 |
| Medium, WebGL2, dusk | 3.36 → 1.73 | 5.40 → 4.40 | 368 → 324 |
| Medium, WebGL2, build | 1.99 → 1.03 | 4.67 → 4.29 | 295 → 256 |
| Medium, WebGL2, wide | 2.99 → 0.97 | 5.93 → 4.68 | 389 → 352 |
| High, WebGL2, orbit-day | 2.65 → 1.15 | 5.26 → 4.56 | 356 → 307 |
| High, WebGL2, orbit-night | 3.09 → 0.97 | 6.08 → 4.77 | 366 → 313 |
| High, WebGL2, dusk | 3.68 → 1.73 | 5.57 → 4.34 | 380 → 321 |
| High, WebGL2, build | 1.88 → 1.00 | 4.56 → 4.30 | 295 → 256 |
| High, WebGL2, wide | 3.00 → 1.01 | 5.95 → 4.71 | 390 → 352 |

So by Stage 7 the main thread did about a third of Babylon's work on WebGPU and a third to a half on WebGL2, with 39–56 fewer draws (High's count Babylon's reflections), and the GPU time was lower in every cell (3.4–4.6 ms against 6.9–7.5 on WebGPU, 4.3–4.8 against 4.6–6.1 on WebGL2). Stage 6, with the lamps, shadows and post-processing it added, put up to 0.3 ms back on WebGPU's main thread and up to 0.8 ms on WebGL2's GPU (High at dusk) over Stage 5.

### Final checkpoint (Stage 11)

2026-10-24, the finished port (no Babylon left) against the Babylon renderer: `render.mjs --preview --build --runs 1`, the village with its 41 residents, 1920×1080 at DPR 1, vsync at 120 Hz, 20 s after a 5 s warm-up, M3 Pro (Apple GPU, Metal 3). Babylon's Medium and High are the Stage 0 baseline (`bench-results/baseline-babylon/perf`); Babylon's Low was measured the same day from the Babylon reference checkout (the tree as of Stage 2, `bench-results/stage11/perf-babylon-low`). Main thread is the whole frame (update + draw), ms; GPU is WebGPU's pass span or WebGL2's frame timer, ms; draws are all passes; p99 is the frame interval's 99th percentile, ms. One run each, so single cells move by a few tenths of a millisecond between runs.

| preset | backend | scenario | main thread Babylon | main thread PlayCanvas | GPU Babylon | GPU PlayCanvas | draws Babylon | draws PlayCanvas | p99 Babylon | p99 PlayCanvas |
|---|---|---|---|---|---|---|---|---|---|---|
| low | WebGPU | orbit-day | 3.80 | 1.50 | 4.92 | 3.72 | 286 | 274 | 10.0 | 10.3 |
| low | WebGPU | orbit-night | 3.97 | 1.53 | 4.86 | 3.63 | 293 | 280 | 10.2 | 10.3 |
| low | WebGPU | dusk | 4.27 | 1.84 | 4.80 | 3.37 | 285 | 276 | 10.1 | 10.3 |
| low | WebGPU | build | 3.62 | 1.46 | 3.38 | 3.81 | 284 | 273 | 10.1 | 10.3 |
| low | WebGPU | wide | 4.24 | 1.53 | 5.23 | 3.78 | 331 | 320 | 10.1 | 10.3 |
| low | WebGL2 | orbit-day | 2.44 | 1.25 | 3.83 | 4.42 | 286 | 274 | 10.2 | 10.3 |
| low | WebGL2 | orbit-night | 2.46 | 1.23 | 3.96 | 4.34 | 293 | 280 | 10.2 | 10.3 |
| low | WebGL2 | dusk | 3.03 | 1.55 | 4.16 | 4.23 | 286 | 276 | 10.3 | 10.2 |
| low | WebGL2 | build | 2.35 | 1.21 | 3.69 | 3.88 | 284 | 273 | 10.2 | 10.3 |
| low | WebGL2 | wide | 2.29 | 1.21 | 4.03 | 4.34 | 331 | 320 | 10.2 | 10.3 |
| medium | WebGPU | orbit-day | 4.74 | 1.59 | 7.33 | 3.54 | 346 | 334 | 9.3 | 10.3 |
| medium | WebGPU | orbit-night | 4.90 | 1.56 | 7.18 | 3.88 | 355 | 339 | 9.3 | 10.2 |
| medium | WebGPU | dusk | 5.36 | 2.06 | 6.97 | 3.50 | 370 | 352 | 9.3 | 10.2 |
| medium | WebGPU | build | 4.29 | 1.51 | 7.51 | 3.81 | 295 | 282 | 9.1 | 10.3 |
| medium | WebGPU | wide | 4.89 | 1.58 | 7.38 | 3.92 | 392 | 379 | 16.7 | 10.3 |
| medium | WebGL2 | orbit-day | 2.48 | 1.29 | 5.10 | 4.77 | 345 | 333 | 9.3 | 10.3 |
| medium | WebGL2 | orbit-night | 2.91 | 1.08 | 5.89 | 4.77 | 355 | 339 | 9.1 | 10.3 |
| medium | WebGL2 | dusk | 3.36 | 1.95 | 5.40 | 4.49 | 368 | 350 | 9.3 | 10.3 |
| medium | WebGL2 | build | 1.99 | 1.01 | 4.67 | 4.27 | 295 | 282 | 9.1 | 10.3 |
| medium | WebGL2 | wide | 2.99 | 1.13 | 5.93 | 4.88 | 389 | 379 | 10.2 | 10.2 |
| high | WebGPU | orbit-day | 5.01 | 1.61 | 7.28 | 3.69 | 357 | 342 | 8.9 | 10.3 |
| high | WebGPU | orbit-night | 5.03 | 1.57 | 6.90 | 3.97 | 367 | 348 | 9.2 | 10.3 |
| high | WebGPU | dusk | 5.59 | 2.13 | 7.03 | 4.28 | 381 | 361 | 16.6 | 10.3 |
| high | WebGPU | build | 4.27 | 1.47 | 7.51 | 3.95 | 295 | 282 | 9.3 | 10.2 |
| high | WebGPU | wide | 4.86 | 1.58 | 7.39 | 4.05 | 392 | 379 | 16.7 | 10.3 |
| high | WebGL2 | orbit-day | 2.65 | 1.34 | 5.26 | 5.24 | 356 | 342 | 10.2 | 10.3 |
| high | WebGL2 | orbit-night | 3.09 | 1.12 | 6.08 | 4.94 | 366 | 348 | 10.2 | 10.2 |
| high | WebGL2 | dusk | 3.68 | 1.92 | 5.57 | 4.70 | 380 | 361 | 10.3 | 10.2 |
| high | WebGL2 | build | 1.88 | 1.07 | 4.56 | 4.44 | 295 | 282 | 10.2 | 10.2 |
| high | WebGL2 | wide | 3.00 | 1.30 | 5.95 | 5.44 | 390 | 379 | 10.0 | 10.3 |

- **Main thread:** PlayCanvas needs 2.3–3.2× less on WebGPU and 1.7–2.8× less on WebGL2, on every preset and scenario.
- **GPU:** on WebGPU, 1.6–2.1× less on Medium and High and 24–30 % less on Low (except Build, 3.4 → 3.8 ms). On WebGL2, Medium and High are equal or lower; Low is 2–15 % higher (orbit-day 3.83 → 4.42 ms), the one place past the plan's +10 % bound. Profiled for 0.4.1 (below): not the FXAA; nature's share is the one that stands out.
- **Frames:** both hold 120 fps everywhere. PlayCanvas drops none (p99 at most 10.3 ms), where Babylon dropped a few in the wide view on High and Medium (p99 16.7 ms); its frame intervals do jitter more around 8.33 ms (p95 about 10 ms against Babylon's 8.4–9.9), with the same average.
- **Shader compiles in play:** after `warmGame`, a scripted session (play, night, a mirror, walls, Build mode, the catalog, buying) compiles 1 shader and 1 pipeline, for the model bought for the first time (before the warm-up was ported: 32 shaders and 340 pipelines on WebGPU, which stalled the first seconds of play). `?diag=shaders` logs them.
- The pictures, against Babylon's: the 54 lit benchmark shots differ by a median 0.74 % of pixels by more than 16 (2.5 of 255 on average), the albedo and normals views by 0.08 %, the mirror poses by 1.1 %, and the 15 Build-mode states replayed with Babylon's pointer by 0.06–1.6 % (`bench-results/stage11/`).

### Low on WebGL2, profiled (0.4.1)

2026-10-24, the same machine and method (`render.mjs --preview`, windows at 120 Hz, median of 3 runs; `bench-results/0.4.1/`). Low on WebGL2 against Babylon's single run: orbit-day 4.07–4.24 ms against 3.83 (+6–11 %), orbit-night 4.22 against 3.96, dusk 4.25 against 4.16, build 3.77 against 3.69, wide 4.67–4.76 against 4.03 (+16–18 %). Runs of one build differ by 0.2–0.3 ms, so only the wide view's gap is well clear of the noise. Passes, pixel samples, draws and triangles are Babylon's (4 passes, 7.3 M samples; 320 against 331 draws, 1.97 against 2.01 M triangles in the wide view): the difference is in what a pixel or a vertex costs, not in how much is drawn.

What each part costs, one left out at a time (`diag=`, and on the dev server the preset's own knobs one by one):

- **Nature** (`nonature`): −0.75 ms in the wide view (4.76 → 4.01), −0.06 ms in orbit-day. The only part whose cost stands out, and the view with the largest gap.
- **Within the noise** (under 0.3 ms either way): the FXAA (`noaa`, −0.1 ms: not the suspect it was), the sun's shadows (`noshadows`) and its map's size (1024 or 2048), the lamps (`nolamps`; clustered or forward, `lights=forward`), anisotropy (4 or 1), merged shadow casters, the soft look against classic shading, the night grade and glass.

Left for later (done for 0.5.0, below): nature has no switches of its own yet (the woods, the lawn's tufts, the terrain), so which of them costs more than on Babylon isn't known. The Babylon reference checkout is gone, so Babylon can't be measured again beside it; the comparison is with the recorded runs.

### Low on WebGL2, the woods (0.5.0)

2026-10-26, the same machine, with up to ten agents sharing it (`bench-results/0.5.0/`). Under that load the method above stopped measuring: with vsync, one cell's GPU time came out at about 3 ms in some runs and about 6 ms in others (Low, WebGL2, the wide view: 5.99, 3.07, 6.54 ms), flipping between rounds, the Apple GPU's clocks following how busy the machine was rather than what was drawn. So the numbers here are from runs without vsync (`--throughput`, the GPU always busy, about ±0.2 ms), each change against its base in the same round and the rounds interleaved, and from an in-page A/B (one bench page, the variants switched every 2 s along the camera path, each window's GPU time against the base window next to it, 30–40 rounds), which cancels the slow drift altogether. Triangles and draws don't depend on the load.

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

Against Babylon: its runs were with vsync, at whatever clocks the GPU chose then, so they can't be set beside these. The 0.4.1 gap (wide view +0.6–0.7 ms) was about nature's cost then (−0.75 ms); the squares take about 40 % of nature's cost away (−0.33 of −0.8 ms without vsync), so the wide view may now be within the plan's +10 %, but that can only be checked on a quiet machine with the 0.4.1 method.
