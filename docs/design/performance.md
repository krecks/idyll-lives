# Where the time goes (measured 2026-10-22)

Why the game sometimes stutters, measured on the branch `perf/stutter-measurements`. The short
answer: **steady rendering is not the problem** — the renderer holds 120 fps with no long tasks.
The hitches come from *rebuilding the whole rendered world* whenever the simulation says the
structure changed, and from the background asset stream doing the same when a file arrives after
Play. Both scale with how fast the game is played.

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

## What would fix it, in order of payoff

1. **Do not rebuild the world for a structure message that only changes a few objects.** Either diff
   the message in the session (objects by id: rebuild only the changed templates/meshes, keep the
   rest), or have the simulation say *what* changed (opened object ids, grown crop ids) so the
   renderer can patch those meshes instead of the lot.
2. **Patch streamed models into the meshes that stand in for them** instead of `setWorld`. The
   renderer already knows which template keys each arrived url invalidates; swapping the meshes for
   those keys avoids the street/object rebuild and most of the 1 s warm.
3. **Throttle the stream rebuilds** to one per few seconds while the player is not looking at the
   affected lot (the town's street does not need to be rebuilt while the camera is inside a house).
4. Lower `DROP_MS` to about 20 ms so `?profile` records what players feel.

None of these are done on this branch: it carries the measurements, the profiler tool and this
write-up.
