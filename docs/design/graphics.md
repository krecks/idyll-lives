# Graphics settings and performance

*Status: 0.2.0. Three presets (Low, Medium, High), Auto resolution and the paced frame rate cap are in. On the Intel laptops below, Medium's target is 30 fps. It hasn't been measured on Intel hardware: scaled from Mac measurements, 0.2.0's Medium is estimated at about 10–21 fps there by day (8–13 at night), so it probably falls short until someone measures it on a real laptop. Low is best effort. The release review has the findings behind this page ([release-0.2.0-review.md](../release-0.2.0-review.md), §2.5.1 and §3.5).*

Code: `web/src/render/quality.ts` (the presets), `web/src/render/babylon/resolution.ts` (Auto resolution), `web/src/render/babylon/pacing.ts` (the FramePacer), `web/src/settings/settings.svelte.ts` (settings and their migration), `web/src/debug/bench.ts` and `benchFlags.ts` (the benchmark), `tools/bench/` (its runner).

## The presets

Settings › Performance › Graphics. Medium is everyone's default and the game's intended look. High is Medium with sharper sun shadows and live mirrors. Low is Medium with modest trims for older laptops (integrated Intel graphics). A preset applies the next time a game starts or loads. `?quality=low|medium|high` overrides the setting for testing.

Every knob is in one table, `QUALITY` in `quality.ts`; a new knob gets a field there with what it does and what it costs, and a value in each preset.

| Knob | Low | Medium (default) | High | What it does, and what it costs on Medium |
|---|---|---|---|---|
| `shadowMapSize` | 1024 | 2048 | 4096 | The sun's shadow map, texels a side. P1 measured shadows at 1.2 ms of GPU by day and 1.5 ms at night at 2048 (WebGL2: 1.5 and 2.3 ms of the frame), and half the frame's draw calls (242 of 493). Since 0.2.0 culls the casters to the map, they are about a third (146–163 of about 430). Each step up is four times the texels. |
| `msaaSamples` | 1 (FXAA) | 4 | 4 | 4× multisampling of the HDR scene: 3.0 ms of Medium's 10.6 ms by day, about 100 MB of memory traffic a frame at 1080p (about 4.5 ms on an Intel HD 620 by P1's bandwidth model). 1 means FXAA instead, a cheap filter pass with slightly softer edges. |
| `anisotropy` | 4 | 8 | 8 | Anisotropic filtering of floor, wall, road and ground textures. Not measured on its own. |
| `clusteredLamps` | false | true | true | Every lamp lights (clustered lighting, up to 32), rather than the four nearest the biggest rooms. P1 measured 3.1 ms by day (every lamp dark) and 4.3 ms at night on WebGPU, 0.8 and 1.6 ms on WebGL2. Since a dark lamp leaves the cluster, the cost by day is mostly gone (G2: 8.87 → 6.49 ms by day; the night is unchanged). Low's four lamps stay in the shader by day, dark. |
| `mirrors` | 0 | 0 | 2 | How many mirrors (the nearest the camera) reflect the room live. Each draws the world once more, about 50 MB of memory traffic a frame. In the village with the benchmark's default 20 s window, High's mirror averages 50–64 extra draws and 0.5 M triangles a frame; its GPU cost was within noise except +3 ms on WebGL2 at night (it reflects for only part of the window, so inconclusive). Before 0.2.0 Medium had 1 live mirror: A/B runs against that base need `diag=nomirrors`. Each reflection target, about 56 MB of GPU memory at 1080p (0.75 of the screen, 4× MSAA), exists only while a mirror reflects, and for 10 s after. |
| `lightingStep` | 10 | 0.5 | 0.5 | Game minutes between lighting updates at speed 3 and up and in quiet hours (slower speeds: every half minute). Each update re-colours the sun, sky and lamps and can thaw the materials for a few frames: at speed 5 that was 20 updates a second, CPU p95 4.7–6 ms against 0.3–0.4 ms. On every preset the lighting also changes at most every 100 ms of real time, 10 times a second. |
| `lawnDensity` | 0.5 | 1 | 1 | Share of the lawn's grass tufts and flowers drawn: 0.5 keeps every other one, always the same ones, standing where they did (24,245 of the landscape's 44,655 plants). With `landscapeShadows`, on Low by day: −0.29 M triangles, −6 shadow draws, −0.5 ms of GPU (WebGPU, orbit view). |
| `landscapeShadows` | false | true | true | The woods round the town cast shadows. The lot's own garden trees, the street and the house always do. |
| `ambientRange` | 0 | 40 | 60 | Metres from the camera within which things' ambient clips play (a ceiling fan, clock hands, a fountain's water: a model's `motions.idle`); further off, and on Low, they rest. Each moving thing rewrites its model's part buffers each frame (one upload per model, however many move); the sun's shadows aren't redrawn for them, their culling boxes stay the rest pose's, and no draw calls are added. 94 at once on a lot (a stress case: 94 scratch rocking chairs, each with a fan turning; WebGPU, M3 Pro, headless, 2026-10-14): 0.09 ms of CPU a frame for the clips and +0.15–0.3 ms for the frame's CPU; with Chrome's CPU throttled 4× (a stand-in for an older laptop's CPU): 0.2–0.35 ms and +0.4–0.6 ms. Not measured on Intel graphics hardware; Low keeps them off. |
| `snapshotRendering` | true | true | true | WebGPU only: the menus' town records its draw commands once and replays them each frame. The game itself draws live. `?snapshot=0` turns it off. |

Costs are what leaving the feature out saved on Medium: GPU time on an M3 Pro at 1920×1080, WebGPU, no vsync, median of 3 runs (P1's benchmark, `?bench=village`). "Intel" figures come from P1's bandwidth model, not from measurements.

The same on every preset:
- **No canvas MSAA.** Only the last full-screen pass draws to the canvas, so its 4× MSAA smoothed nothing: 66 MB of memory traffic a frame at 1080p, about 3 ms on an Intel HD 620 by P1's model (P1-06).
- **The shadow map's colour target is R16F**, not the RGBA16F target that nothing read (P1-05).
- **Auto resolution** with a budget of 1920×1080 (below).
- **The FramePacer** for the frame rate cap (below).
- **The menus' town** is drawn at about 30 fps (the cap's rounding rule for 30, below).
- **Dark lamps leave the cluster:** a lamp that is off gets a range of 1 cm, so it covers next to no tiles (P1-02).
- **Nature is culled in groups, in the game:** the landscape's plants are grouped by where they stand (lawn tufts in about 64 m squares, the woods in 8 sectors round the town; a part with fewer than 36,000 triangles in all, such as bark or flowers, stays whole), so the camera leaves out those out of view (P1-01). Each group is a mesh of its own, culled by its box, and every mesh costs the main thread a little each frame, so the squares are wide. The menus' town, with the whole landscape in view, and the game's sun shadows (the near woods) draw the plants whole (`nature.ts`).
- **The sun's shadow map** draws only the casters that can reach its square round the lot, and is drawn again only when something in it changed: the light, a thing placed or moved, walls, roofs or storeys, or a resident's pose. Paused, in Build mode and behind menus it isn't drawn at all; in live play, with residents in view, it still is every frame (P1-04).
- **Only the residents in view** have their pose rows uploaded each frame (P1-14).
- **Nothing is drawn behind full-screen menus:** the pause menu, Settings, Load game, Credits, the planner and the crash screen get 2 more frames, then none until they close; the frosted glass keeps blurring the last frame.
- **Mirrors' memory on demand** (High), the **lighting at most 10 times a second**, and a **100 ms limit on every frame step**, Babylon's particle clock included.

Removed in 0.2.0: Ultra, Custom and the fine-tune toggles, and ambient occlusion with them (SSAO2 and the multiple-render-target code left the main chunk).

**Settings from 0.1** (`migrateGraphics`): a stored Low stays Low, whether it was the old quality preset or its parts as the old Low set them. Anything else becomes Medium. The old defaults were High's, with live mirrors, so players who never changed them now get Medium without live mirrors. The old resolution slider isn't carried over: everyone starts on Auto. The old graphics keys are dropped.

## Targets

The owner's target machines are laptops with an i7-7500U, i7-8550U or i5-6500 and Intel HD 620, UHD 620 or HD 530 graphics, at 1920×1080:

| Preset | 0.2.0 target | Estimate for 0.2.0, by day / at night | Estimate before 0.2.0 (P1, §2.5.1) | Status |
|---|---|---|---|---|
| Medium | 30 fps or more | about 10–21 / 8–13 fps | about 6–10 fps | Probably falls short; not measured on Intel hardware; not a release gate |
| Low | none: best effort | about 14–41 / 13–39 fps | about 9–27 fps | The owner made Low at 60 fps not a 0.2.0 target |
| High | none | about 9–17 / 7–11 fps | about 6 fps | – |

- **The estimates aren't measurements.** No Intel hardware was available, so P1 scaled the M3 Pro's GPU time by two models that differ by 2–3×; at sign-off it applied the same models to 0.2.0 (on AC power, `render.mjs --throughput`, median of 3). Only a run on a real Intel laptop settles it; the owner does that with the benchmark (the release review's §5).
- **Medium misses 30 fps on both models.** At night it barely improved (8.84 → 7.39 ms of GPU on the M3), because lit lamps still cost: the night range cap for the lamp cluster (P1-02) wasn't done.
- **Not a priority for 0.2.0** (the owner's decision; best effort, not promised): detecting a weak GPU and picking Low or WebGL2 on its own, Auto resolution stepping down when the game runs slowly, 60 fps for Low on an HD 620, and generated LOD models. They are future work (the release review's §4, and PLAN.md's roadmap).
- **The GPU is the limit on those laptops.** At 3× CPU slow-down on the M3 Pro, standing in for the target CPUs, 0.2.0 allows 47–57 fps on WebGPU and 78–98 fps on WebGL2, above every GPU estimate (P1's sign-off). Before 0.2.0 WebGPU spent 21–31 ms a frame of CPU (32–47 fps) submitting the same 493 draws, so on WebGPU the CPU alone also kept Low from 60 fps there (P1-07).

## Resolution

Settings › Performance › Resolution: Auto (the default) or Custom.

- **Auto** draws every pixel of the screen up to a budget of 1920×1080 (2,073,600 pixels). A screen with more device pixels is drawn at `sqrt(budget / pixels)` of its resolution along each side, and the browser scales the picture up. The interface is still drawn at the screen's own resolution, so text stays sharp.
- **Custom** draws 50–100 % of the screen's own resolution along each side, whatever the screen.
- **Rounding:** the drawn size is CSS size × device pixel ratio × scale, and Babylon cuts it down to whole pixels.
- **When it's applied:** on start, on a settings change, when the window resizes, and when the device pixel ratio changes. Browser zoom and moving the window to a screen with another density change the ratio, which doesn't always resize the window, so a `matchMedia` resolution query watches it. Babylon's own `adaptToDeviceRatio` is off; it would apply the ratio a second time.
- **There is no step-down:** Auto doesn't lower the resolution when the game runs slowly.

Examples with Auto:

| Screen (CSS size × device pixel ratio) | Device pixels | Drawn | Share along each side |
|---|---|---|---|
| 1366×768 × 1 | 1366×768 | 1366×768 | 100 % |
| 1920×1080 × 1 | 1920×1080 | 1920×1080 | 100 % |
| 2560×1440 × 1 | 2560×1440 | 1920×1080 | 75 % |
| 2560×1600 × 1 | 2560×1600 | 1821×1138 | 71 % |
| 1512×982 × 2 (a MacBook window, measured by G1) | 3024×1964 | 1786×1160 | 59 % |
| 1920×1080 × 2 | 3840×2160 | 1920×1080 | 50 % |
| 3840×2160 × 1 | 3840×2160 | 1920×1080 | 50 % |

The F3 overlay shows the drawn size and the share ("Resolution 1786×1160 · 59 %").

## The frame rate cap

Settings › Performance › Frame rate cap: 30, 60 (the default), 120, No cap, or a custom number from 10 to 500.

Babylon's own cap (`engine.maxFPS`) drew once enough time had piled up since the last frame, so after a late frame the next came a refresh early, and frames stayed on screen for 1, 2 or 3 refreshes: motion juddered (R1-02). The FramePacer replaces it as Babylon's frame requester (`engine.customAnimationFrameRequester`):

- It measures the display's refresh rate from the `requestAnimationFrame` timestamps: the usual short gap between refreshes, worked out again every 32 refreshes. A drift under 0.5 % doesn't change it.
- It draws every N-th refresh, counted from the last frame drawn.
- **Rounding rule:** N is the smallest divisor whose rate isn't more than a quarter above the cap (with 2 % leeway for a measured rate a little off the nominal one): `N = max(1, ceil(Hz / (cap × 1.25 × 1.02)))`. So a cap the refresh rate doesn't divide becomes a rate it does, rounded up when that's within a quarter, and down otherwise.
- **No cap** draws every refresh.
- **The game's clocks** run on the timestamp of the refresh a frame is drawn for, not the moment its code runs. The residents' interpolation, the step counter and the director get that timestamp, and Babylon's frame time (camera glides and inertia, clouds, wind) is measured between those timestamps, so motion advances in the same even steps the frames are shown in. The frame time is clamped to 100 ms.
- **The menus' town** is paced at about 30 fps (`MENU_FPS` through the same rule, as in the Cap 30 column below: 36 on a 144 Hz display, 37.5 on 75 Hz), and the flight into a game at the game's cap.

Frame rates the pacer gives:

| Display | Cap 30 | Cap 60 | Cap 120 | No cap |
|---|---|---|---|---|
| 60 Hz | 30 | 60 | 60 | 60 |
| 75 Hz | 37.5 | 75 | 75 | 75 |
| 90 Hz | 30 | 45 | 90 | 90 |
| 100 Hz | 33.3 | 50 | 100 | 100 |
| 120 Hz | 30 | 60 | 120 | 120 |
| 144 Hz | 36 | 72 | 144 | 144 |
| 165 Hz | 33 | 55 | 82.5 | 165 |
| 240 Hz | 34.3 | 60 | 120 | 240 |

The F3 overlay shows the measured rate and N ("Display 120 Hz ÷ 2"). Measured on a 120 Hz MacBook Pro display (production build, Medium, cap 60, 2× CPU slow-down, 8 s): frame lengths in refreshes went from 1×60, 2×327, 3×78, 4×3 to 2×471, 3×6. The other rates in the table are worked out from the rule; only 120 Hz was measured.

## Troubleshooting

Settings › Performance › Troubleshooting › Graphics API: Auto uses WebGPU where there is one, WebGL2 otherwise; WebGL2 forces the fallback. `?renderer=webgl` does the same for one page. When WebGPU has an adapter but fails to start, the game draws with WebGL2 for the session and says so. Voices and the language model need WebGPU either way.

## The benchmark

Opening the game with `?bench=<scene>` plays a saved game state, `web/public/bench/<scene>.json.gz` (`village`: the menus' valley as a new game makes it, with a household of four at home at 17:55 on day 1; `forest`), with the camera on a path fixed in time. It warms up for 5 s, measures for 20 s, and then shows a summary with a "Copy result" button; the result is also in `window.__benchResult`, and with the dev server in `bench-results/`. It works on the deployed site too, so it can run on any laptop. Nothing changes for players without the flag: the settings it changes are for its page alone and never saved, its save slot is deleted, and the renderer's hooks are null checks.

| Flag | Values | What it does |
|---|---|---|
| `bench` | `village`, `forest` | The scene to play. |
| `quality` | `low`, `medium`, `high` | The preset; without it, the player's setting. Works without `bench` too. |
| `hour` | e.g. `13`, `21` | Holds the lighting at that hour; without it, the game's clock. Works without `bench` too. |
| `cam` | `orbit` (default), `wide`, `build`, `dusk` | `orbit`: 24 m round the home lot, a turn in 40 s. `wide`: 80 m, 90 s. `build`: the orbit in Build mode, paused. `dusk`: the orbit at speed 5 from the scene's 18:00, the light changing every frame. |
| `res` | `auto` (default), `1`, `0.5`–`1` | The render scale; `auto` is what the game picks. |
| `secs`, `warmup` | 20, 5 | Seconds measured, after the warm-up. |
| `cap` | 0 (default) | Frames per second at most; 0 draws every refresh. |
| `speed` | 1 (`dusk`: 5) | The game speed while measuring. |
| `hud`, `gpu` | `0` | `hud=0` hides the interface and the summary; `gpu=0` doesn't time the GPU. |
| `label` | text | Kept with the result. |
| `diag` | `noshadows`, `noaa`, `nonature`, `nolamps`, `nomirrors`, `noglass`, `nonight`, `freeze` | Leaves features out, to measure what each costs (a comma list). `freeze` stops time for screenshots that compare pixel for pixel, at `at=<s>` along the path. Works without `bench` too. |
| `renderer` | `webgl` | WebGL2 instead of WebGPU. |

The result has the frame interval (average, p50, p95, p99), the main thread's work split into the game's update and Babylon's `scene.render`, the GPU time (WebGPU timestamp queries: from the start of a frame's first pass to the end of its last; WebGL2 timer queries where offered), draw calls and triangles (all passes, shadow map, mirrors), the render passes with their pixels and bytes, the residents' posing time, lighting updates a second, long tasks, the adapter, and a fingerprint of scene counts that shows two runs drew the same scene. Its `fps` counts its own `requestAnimationFrame` ticks, which is right without a cap and misleading with one.

Other debug flags: `?lights=clustered|forward` forces the lamps' mode, and `?snapshot=0` turns off WebGPU snapshot rendering.

**The runner.** `node tools/bench/render.mjs` opens the benchmark in Chromium on the real GPU at 1920×1080, device pixel ratio 1, for every preset × backend × scenario (`orbit-day`, `orbit-night`, `dusk`, `build`, `wide`) × CPU slow-down × diagnostic, and writes JSON and a markdown table (medians with `--runs`). `--preview --build` measures the production build, which is what players get (compressed textures and models, minified code); `--throughput` turns vsync and the frame-rate limit off; `--throttle 1,3` slows the CPU through CDP; `--diags none,noshadows` adds diagnostics. `--shots <dir>` takes screenshots of 9 fixed poses with time frozen, and `--compare <dirA> <dirB>` counts the pixels that differ. Two runs of the same build in a window differ by at most 1 of 255 in a few pixels, apart from the far road (P1-17); two headless runs, in at most 0.12 % of their pixels. `node tools/bench/scene.mjs --name village` makes a scene. Both need `pnpm wasm` first, and Playwright (`PLAYWRIGHT=<path to playwright/index.mjs>` if it isn't installed). Measurements run in a window, for the real display's timing; shots and scenes run headless on the real GPU (Chromium's full build; `chrome-headless-shell` draws in software). Compare shots taken the same way: a window's come out about 2 of 255 brighter (the screen's colour profile).

## Measured costs

All on a MacBook Pro M3 Pro (18-core GPU), Chrome for Testing 153 headed, 1920×1080 at device pixel ratio 1, production build, the `village` scene. GPU time on WebGPU is the span from a frame's first pass to its last; on WebGL2, whose timer reads too high, the frame time without vsync is used instead. Runs vary by about ±1 ms; the tables are medians of 3, without vsync.

**Before 0.2.0 (P1, at 4cde174):**
- Medium: 10.3–10.5 ms of GPU a frame. Low: 6.6–7.0 ms, and 6.7 ms at 75 % scale. On the M3 Low is limited by geometry, not pixels.
- Every preset drew the same scene: 6.0 M triangles a frame (2.26 M nature, 1.70 M shadow pass, 2.2 M lot, street and furniture) in 493 draw calls, 242 of them for the shadow map.
- The interface costs about 4 ms a frame (65 → 88 fps without the HUD).

What each feature costs on Medium (GPU time saved by leaving it out):

| Feature left out | WebGPU GPU, day | WebGPU GPU, night | WebGL2 frame, day | WebGL2 frame, night | Draws | Triangles |
|---|---|---|---|---|---|---|
| (Medium as it was) | 10.58 ms | 12.16 ms | 12.10 ms | 12.19 ms | 493 | 6.01 M |
| Clustered lamps (all 32 dark by day) | −3.07 | −4.28 | −0.80 | −1.64 | 0 | 0 |
| Scene 4× MSAA | −3.03 | – | −0.30 | – | 0 | 0 |
| Sun shadows | −1.20 | −1.50 | −1.53 | −2.26 | −242 | −1.70 M |
| Nature (grass, flowers, forest) | −1.08 | – | −2.12 | – | −38 | −2.26 M |
| Night grade | – | +0.01 | – | −0.80 | −1 | 0 |
| Canvas MSAA, frosted glass, mirrors | within noise | – | within noise | – | 0 | 0 |

The HD 620 estimate scales these: T_gpu ≈ k × T_mac + 0.5 × B / 22 GB/s, with k = 16 (an M3 Pro GPU of about 6.4 TFLOPS) and B twice the render-target bytes a frame; a second model uses k = 5 for geometry and fixed costs and k = 16 for pixel work. Details in the release review's §2.5.1.

**G1, 0.2.0 presets** (A/B against a428263, `render.mjs --preview --throughput`):

| Preset, backend, light | fps | GPU ms | CPU ms | Render targets a frame |
|---|---|---|---|---|
| Low, WebGL2, day | 145.3 → 157.5 | 8.95 → 8.13 | 6.34 → 5.86 | 120 → 64 MB |
| Low, WebGPU, day | 124.5 → 132.7 | 5.45 → 5.37 | 7.45 → 6.99 | 120 → 64 MB |
| Medium, WebGL2, day | 129.5 → 139.0 | 11.87 → 10.52 | 7.30 → 6.71 | 216 → 141 MB |
| Medium, WebGPU, day | 90.9 → 93.0 | 9.06 → 8.98 | 10.30 → 10.22 | 216 → 141 MB |
| High, WebGL2, day | 121.2 → 138.1 | 12.35 → 11.02 | 7.75 → 6.83 | 216 → 217 MB |
| High, WebGPU, day | 89.8 → 89.6 | 8.98 → 8.77 | 10.52 → 10.69 | 216 → 217 MB |

- WebGPU's GPU times are within its ±1 ms noise; the night rows are in the release review's §3.5.1.
- The render-target savings come from the canvas MSAA and the shadow map's colour target. Apple's tile memory hides them; by P1's bandwidth model an Intel HD 620 pays about 3 ms for the canvas MSAA and 0.7 ms for the shadow target on Medium.
- Medium's 18 reference shots differ from P1's baseline in under 0.15 % of their pixels: only the far road's lane markings (P1-17) and one mirror that Medium no longer reflects live.
- The menus' town at 30 fps: main thread 20 → 14 % busy, 45 → 24 % at 3× CPU slow-down.

**0.2.0 against the review's start** (P1's sign-off, on AC power: base 44809e7, which is 4cde174 plus the benchmark tooling, with `diag=nomirrors` on Medium, against 5f72a7c; `render.mjs --throughput`, median of 3 alternating runs; WebGPU GPU is the pass span, WebGL2 the frame p50):

| Preset, light | WebGPU fps | WebGPU GPU ms | WebGL2 fps | WebGL2 frame ms | Draws: main / shadow / mirror | Triangles M | Render targets MB |
|---|---|---|---|---|---|---|---|
| Low, day | 113 → 155 | 6.02 → 4.32 | 161 → 198 | 6.19 → 4.74 | 252/242/0 → 276/146/0 | 6.03 → 3.43 | 120 → 64 |
| Low, night | 113 → 154 | 6.16 → 4.53 | 153 → 179 | 6.31 → 5.19 | 253/242/0 → 278/157/0 | 6.03 → 3.49 | 129 → 73 |
| Medium, day | 89 → 144 | 8.73 → 5.58 | 122 → 160 | 8.11 → 5.61 | 252/242/0 → 277/152/0 | 6.03 → 3.71 | 216 → 141 |
| Medium, night | 82 → 107 | 8.84 → 7.39 | 113 → 157 | 8.38 → 5.59 | 253/242/0 → 278/163/0 | 6.03 → 3.77 | 225 → 150 |
| High, day | 78 → 132 | 10.23 → 6.06 | 107 → 157 | 8.46 → 5.40 | 252/242/49 → 276/152/50 | 6.75 → 4.22 | 216 → 217 |
| High, night | 80 → 91 | 8.62 → 8.24 | 116 → 108 | 8.02 → 8.11 | 253/242/48 → 277/163/54 | 6.73 → 4.31 | 225 → 225 |

- High isn't like for like: the base had 2048 shadows. G2's own whole-branch table may include runs on battery; these numbers take precedence.
- **The processor's side:** at 3× CPU slow-down P1's sign-off found no CPU gain in play on 5f72a7c (8 of 12 cells 0.6–3.9 ms a frame worse): G2's plant groups added about 300 meshes (cloned, which slowed Babylon's per-frame mesh loop) and main-pass draws, and every visible change of the sky's light at dusk walked every mesh once per material (41 of them). G3 (5ef8dbf) kept the groups to the game in wider squares, made them new meshes, and changes the light by flags. With P2's method at 3× CPU (speed 5, on AC power, median of 3), a frame's main-thread work now averages 7.21 ms (p95 9.39) against 7.94 (12.40) at 4cde174, 8.19 (12.61) before G2 and 10.18 (20.45) after it, with no long tasks.

**After G3** (benchmark, village orbit by day, `--preview --throughput`, median of 3), dee213f (before G2) → G2 → G3, GPU ms and the share of G2's saving G3 keeps:

| Preset, backend | GPU ms | Kept | Draws (shadow) | Triangles M |
|---|---|---|---|---|
| Medium, WebGPU | 8.60 → 5.18 → 5.39 | 94 % | 494 (242) → 429 (152) → 409 (156) | 6.03 → 3.71 → 4.14 |
| Medium, WebGL2 | 10.52 → 6.52 → 6.92 | 90 % | as above | as above |
| Low, WebGPU | 4.91 → 3.81 → 3.61 | all of it | 494 (242) → 423 (146) → 399 (146) | 6.03 → 3.44 → 3.58 |
| Low, WebGL2 | 6.78 → 4.47 → 4.58 | 95 % | 494 (242) → 422 (146) → 399 (146) | 6.03 → 3.44 → 3.58 |

The table above this one (P1's sign-off) is 5f72a7c, before G3; G3 gives back about 0.2–0.4 ms of Medium's GPU saving on the M3 for the CPU it saves. Shots against dee213f, headless on both sides: Medium at most 0.095 % of pixels off by more than 2 (the far road, P1-17); Low and High against `main` at most 0.136 %.

**G2, step by step** (each change against the one before, production build, median of 3 interleaved runs; each change's own tables are in its commit body, and in the release review's §3.5.1):

| Change | What it saved (Medium unless said; against the step before) |
|---|---|
| Dark lamps out of the cluster | WebGPU orbit by day: GPU 8.87 → 6.49 ms, 92 → 119 fps; at night unchanged |
| Nature in cells (G2's first version; G3 widened them) | WebGPU orbit by day: GPU 6.08 → 5.17 ms, triangles 6.01 → 4.56 M (draws 493 → 538) |
| Shadow casters culled to the map | Draws 540 → 429 (shadow 262 → 152), shadow triangles 1.70 → 0.81 M, 140 → 148 fps |
| Shadow map drawn only when something moved | Build mode (paused): GPU 8.54 → 4.96 ms, shadow draws 152 → 0; live play with residents in view unchanged |
| Lighting at most 10 times a second | Dusk at speed 5: lighting updates 20.1 → 9.7 a second |
| Pose rows of the residents in view | Residents' update 0.17 → 0.14 ms; frame CPU within noise |
| Nothing drawn behind full-screen menus | The pause menu: every frame → 0; the menus' town behind Settings: 60 → 0 frames a second. Each held frame saves about 6 ms of main thread and 5 ms of GPU (orbit by day) |
| Low's half lawn and no woods' shadows | Low, WebGPU orbit by day: GPU 4.51 → 4.03 ms, triangles 3.73 → 3.44 M |

- Many runs were noisy (load average 5–6); the wide view's culling A/B was inconclusive. The baselines' WebGPU runs stalled often (P1-16), so their fps is pessimistic.
- Medium still looks the same: every change's shots differ from the build before in at most 0.12 % of pixels by more than 2 of 255, all on the far road (P1-17).
- In live play the shadow map is still drawn every frame while residents are in view, from 152 draws after culling (the release review's §4).
