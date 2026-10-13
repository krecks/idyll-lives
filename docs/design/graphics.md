# Graphics settings and performance

*Status: 0.2.0. Three presets (Low, Medium, High), Auto resolution and the paced frame rate cap are in. The targets for Intel laptops below are goals estimated from Mac measurements, not yet measured on Intel hardware. The release review has the findings behind this page ([release-0.2.0-review.md](../release-0.2.0-review.md), §2.5.1 and §3.5).*

Code: `web/src/render/quality.ts` (the presets), `web/src/render/babylon/resolution.ts` (Auto resolution), `web/src/render/babylon/pacing.ts` (the FramePacer), `web/src/settings/settings.svelte.ts` (settings and their migration), `web/src/debug/bench.ts` and `benchFlags.ts` (the benchmark), `tools/bench/` (its runner).

## The presets

Settings › Performance › Graphics. Medium is everyone's default and the game's intended look. High is Medium with sharper sun shadows and live mirrors. Low is Medium with modest trims for older laptops (integrated Intel graphics). A preset applies the next time a game starts or loads. `?quality=low|medium|high` overrides the setting for testing.

Every knob is in one table, `QUALITY` in `quality.ts`; a new knob gets a field there with what it does and what it costs, and a value in each preset.

| Knob | Low | Medium (default) | High | What it does, and what it costs on Medium |
|---|---|---|---|---|
| `shadowMapSize` | 1024 | 2048 | 4096 | The sun's shadow map, texels a side. Shadows cost 1.2 ms of GPU by day and 1.5 ms at night at 2048 (WebGL2: 1.5 and 2.3 ms of the frame) and half the frame's draw calls. Each step up is four times the texels. |
| `msaaSamples` | 1 (FXAA) | 4 | 4 | 4× multisampling of the HDR scene: 3.0 ms of Medium's 10.6 ms by day, about 100 MB of memory traffic a frame at 1080p (about 4.5 ms on an Intel HD 620 by P1's bandwidth model). 1 means FXAA instead, a cheap filter pass with slightly softer edges. |
| `anisotropy` | 4 | 8 | 8 | Anisotropic filtering of floor, wall, road and ground textures. Not measured on its own. |
| `clusteredLamps` | false | true | true | Every lamp lights (clustered lighting, up to 32), rather than the four nearest the biggest rooms. 3.1 ms by day (every lamp dark) and 4.3 ms at night on WebGPU; 0.8 and 1.6 ms on WebGL2. |
| `mirrors` | 0 | 0 | 2 | How many mirrors (the nearest the camera) reflect the room live. Each draws the world once more, about 50 MB of memory traffic a frame; the live cost wasn't measured. |
| `lightingStep` | 10 | 0.5 | 0.5 | Game minutes between lighting updates at speed 3 and up and in quiet hours (slower speeds: every half minute). Each update re-colours the sun, sky and lamps and thaws the materials for a few frames: at speed 5 that was 20 updates a second, CPU p95 4.7–6 ms against 0.3–0.4 ms. |
| `snapshotRendering` | true | true | true | WebGPU only: the menus' town records its draw commands once and replays them each frame. The game itself draws live. `?snapshot=0` turns it off. |
| *(pending)* <!-- G2 --> | | | | *(pending: G2's new knobs, e.g. lawn tufts and far woods on Low, lamps by day.)* |

Costs are what leaving the feature out saved on Medium: GPU time on an M3 Pro at 1920×1080, WebGPU, no vsync, median of 3 runs (P1's benchmark, `?bench=village`). "Intel" figures come from P1's bandwidth model, not from measurements.

The same on every preset:
- **No canvas MSAA.** Only the last full-screen pass draws to the canvas, so its 4× MSAA smoothed nothing: 66 MB of memory traffic a frame at 1080p, about 3 ms on an Intel HD 620 by P1's model (P1-06).
- **The shadow map's colour target is R16F**, not the RGBA16F target that nothing read (P1-05).
- **Auto resolution** with a budget of 1920×1080 (below).
- **The FramePacer** for the frame rate cap (below).
- **The menus' town** is drawn at 30 fps at most.
- *(pending: G2's savings on every preset, e.g. nature culled in chunks, dark lamps out of the cluster by day, the shadow map redrawn only when something moves, the lighting throttle, drawing held behind full-screen menus.)* <!-- G2 -->

Removed in 0.2.0: Ultra, Custom and the fine-tune toggles, and ambient occlusion with them (SSAO2 and the multiple-render-target code left the main chunk).

**Settings from 0.1** (`migrateGraphics`): a stored Low stays Low, whether it was the old quality preset or its parts as the old Low set them. Anything else becomes Medium. The old defaults were High's, with live mirrors, so players who never changed them now get Medium without live mirrors. The old resolution slider isn't carried over: everyone starts on Auto. The old graphics keys are dropped.

## Targets

The owner's targets are laptops with an i7-7500U, i7-8550U or i5-6500 and Intel HD 620, UHD 620 or HD 530 graphics, at 1920×1080:

| Preset | Goal | P1's estimate before 0.2.0 (§2.5.1) | Status |
|---|---|---|---|
| Medium | 30 fps or more | about 6–10 fps | A goal, not a release gate |
| Low | 60 fps or more | about 9–27 fps at 75 % scale | A goal, not a release gate |

- **The estimates aren't measurements.** No Intel hardware was available, so P1 scaled the M3 Pro's GPU time by two models that differ by about 3×. The owner confirms the targets on real hardware with the benchmark (the release review's §5).
- **Not a priority for 0.2.0** (the owner's decision; best effort, not promised): detecting a weak GPU and picking Low or WebGL2 on its own, Auto resolution stepping down when the game runs slowly, a guaranteed 60 fps for Low on an HD 620, and generated LOD models. They are future work (the release review's §4, and PLAN.md's roadmap).
- **The GPU is the limit on those laptops.** At 3× CPU slow-down on the M3 Pro, standing in for the target CPUs, WebGPU spends 21–31 ms a frame of CPU (32–47 fps) submitting the same 493 draws, and WebGL2 10–19 ms (54–98 fps). So on WebGPU the CPU alone also keeps Low from 60 fps there (P1-07).
- *(pending: the estimate with G2's savings.)* <!-- G2 -->

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
- **The menus' town** is paced at 30 fps at most (`MENU_FPS`, which the rule makes 36 on a 144 Hz display), and the flight into a game at the game's cap.

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

**The runner.** `node tools/bench/render.mjs` opens the benchmark in headed Chromium on the real GPU at 1920×1080, device pixel ratio 1, for every preset × backend × scenario (`orbit-day`, `orbit-night`, `dusk`, `build`, `wide`) × CPU slow-down × diagnostic, and writes JSON and a markdown table (medians with `--runs`). `--preview --build` measures the production build, which is what players get (compressed textures and models, minified code); `--throughput` turns vsync and the frame-rate limit off; `--throttle 1,3` slows the CPU through CDP; `--diags none,noshadows` adds diagnostics. `--shots <dir>` takes screenshots of 9 fixed poses with time frozen, and `--compare <dirA> <dirB>` counts the pixels that differ. Two runs of the same build differ by at most 1 of 255 in a few pixels, apart from the far road (P1-17). `node tools/bench/scene.mjs --name village` makes a scene. Both need `pnpm wasm` first, and Playwright (`PLAYWRIGHT=<path to playwright/index.mjs>` if it isn't installed). Use headed Chromium: headless falls back to a software GPU.

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

**G2** *(pending: G2's measurements.)* <!-- G2 -->
