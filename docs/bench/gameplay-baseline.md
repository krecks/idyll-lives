# Gameplay baseline: 2 runs, medium/webgpu, seed 7, 300 s steady

MacBookPro18,1, Apple M1 Pro, 10 cores, 32 GB RAM, Apple M1 Pro, macOS 27.2.1 (26B131)

commit `89b684a6388f565186a96ca1aa3ca177a9232d1a` (dirty working tree) · Chromium 156.0.8078.4 · dev server started (ready 273 ms after process start) · cold · 1280x720 @1 · quality medium · HUD 1

## Command

```
bun tools/bench/gameplay.mjs --runs 2 --secs 300 --seed 7 --preset medium --renderer webgpu --hud 1
```

Expected runtime: ~11 min (300 s steady + ~40 s startup/entry/shutdown per run).

## Stage timeline

| run | launch | startup (launch → menu) | entry (Play → game) | steady | frames | samples |
| --- | --- | --- | --- | --- | --- | --- |
| run 1 | 315 ms | 8354 ms | 11549 ms (stage 17686 ms) | 300.0 s | 17993 | 300 |
| run 2 | 311 ms | 7625 ms | 11122 ms (stage 17112 ms) | 300.0 s | 17985 | 300 |

The game is up and interactive when `.hud-root` exists and the loading screen (`.loading`) is gone — the
same signal `loading.mjs` and `profile.mjs` wait on — not a fixed sleep. The steady state must cover
all 300 s: the run fails if it lasts less, overruns by more than 30 s, samples fewer than 300 seconds, or the frames span under 95% of it.

## Per-metric summary

| metric | stage | run | min | mean | median | p95 | p99 | max | n |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| frame gap (ms) | startup | run 1 | 16.66 | 31.39 | 16.66 | 83.33 | 399.99 | 1116.63 | 249 |
| frame gap (ms) | startup | run 2 | 16.66 | 34.38 | 16.66 | 83.33 | 383.32 | 1116.63 | 206 |
| frames/s (counted) | startup | run 1 | 13 | 31.13 | 30 | 60 | 60 | 60 | 8 |
| frames/s (counted) | startup | run 2 | 5 | 25.75 | 22 | 60 | 60 | 60 | 8 |
| frame gap (ms) | entry | run 1 | 16.66 | 48.76 | 33.33 | 116.67 | 633.3 | 733.31 | 364 |
| frame gap (ms) | entry | run 2 | 16.66 | 46.44 | 16.67 | 133.32 | 633.3 | 733.3 | 370 |
| frames/s (counted) | entry | run 1 | 3 | 20.22 | 19 | 48 | 48 | 48 | 18 |
| frames/s (counted) | entry | run 2 | 3 | 21.76 | 20 | 53 | 53 | 53 | 17 |
| frame gap (ms) | steady | run 1 | 16.66 | 16.67 | 16.66 | 16.67 | 16.67 | 66.67 | 17993 |
| frame gap (ms) | steady | run 2 | 16.66 | 16.68 | 16.66 | 16.67 | 16.67 | 66.67 | 17985 |
| frames/s (counted) | steady | run 1 | 56 | 59.98 | 60 | 60 | 60 | 61 | 300 |
| frames/s (counted) | steady | run 2 | 56 | 59.95 | 60 | 60 | 60 | 61 | 300 |
| renderer fps | steady | run 1 | 47.18 | 59.91 | 60 | 60 | 60 | 60 | 300 |
| renderer fps | steady | run 2 | 50.34 | 59.93 | 60 | 60 | 60 | 60 | 300 |
| renderer frame (ms) | steady | run 1 | 16.67 | 16.7 | 16.67 | 16.67 | 17.71 | 21.2 | 300 |
| renderer frame (ms) | steady | run 2 | 16.67 | 16.69 | 16.67 | 16.67 | 17.88 | 19.87 | 300 |
| main-thread frame work (ms) | steady | run 1 | 2.62 | 3.48 | 3.37 | 4.61 | 6.72 | 6.99 | 300 |
| main-thread frame work (ms) | steady | run 2 | 2.36 | 3.44 | 3.25 | 4.45 | 6.63 | 8.18 | 300 |
| GPU frame (ms) | steady | run 1 | 7.1 | 8.4 | 8.24 | 10.05 | 10.41 | 15.66 | 300 |
| GPU frame (ms) | steady | run 2 | 7.24 | 8.76 | 8.59 | 10.55 | 11.24 | 16.12 | 300 |
| draw calls | steady | run 1 | 404 | 621.84 | 566 | 928 | 1034 | 1050 | 300 |
| draw calls | steady | run 2 | 294 | 583.08 | 558 | 835 | 995 | 1016 | 300 |
| main-thread busy (%) | steady | run 1 | 20.39 | 25.84 | 25.25 | 34.72 | 44.11 | 49.09 | 299 |
| main-thread busy (%) | steady | run 2 | 19.96 | 26.05 | 24.65 | 35.07 | 48.11 | 54.01 | 299 |
| script (%) | steady | run 1 | 16.3 | 20.64 | 20.15 | 25.8 | 31.53 | 33.25 | 299 |
| script (%) | steady | run 2 | 14.2 | 20.4 | 19.62 | 27.73 | 31.12 | 39.6 | 299 |
| layout (%) | steady | run 1 | 0.33 | 0.36 | 0.36 | 0.38 | 0.42 | 0.45 | 299 |
| layout (%) | steady | run 2 | 0.33 | 0.36 | 0.36 | 0.39 | 0.41 | 0.43 | 299 |
| browser tree CPU (%) | steady | run 1 | 63.9 | 77.48 | 73.9 | 99 | 172 | 255.9 | 299 |
| browser tree CPU (%) | steady | run 2 | 63.9 | 79.39 | 74.9 | 102 | 166 | 258.8 | 299 |
| GPU process CPU (%) | steady | run 1 | 32 | 36.24 | 36 | 39 | 41.2 | 43 | 299 |
| GPU process CPU (%) | steady | run 2 | 31 | 36.58 | 36 | 41 | 43 | 46 | 299 |
| renderer RSS (MB) | steady | run 1 | 3047.2 | 3498.79 | 3181.2 | 4283.2 | 4396.2 | 4398.8 | 300 |
| renderer RSS (MB) | steady | run 2 | 2815.9 | 3018.24 | 2977.9 | 3399.6 | 4488.6 | 4532 | 300 |
| tree RSS (sum of per-process) (MB) | steady | run 1 | 4779.1 | 5219.32 | 4913.1 | 5983 | 6055.5 | 6056.6 | 300 |
| tree RSS (sum of per-process) (MB) | steady | run 2 | 4551.1 | 4740.32 | 4683.8 | 5056.2 | 6160.3 | 6190.7 | 300 |
| GPU process RSS (MB) | steady | run 1 | 746.7 | 808.83 | 818.3 | 825.1 | 832.8 | 836.8 | 300 |
| GPU process RSS (MB) | steady | run 2 | 744.7 | 807.79 | 820.7 | 830.8 | 831.3 | 834.8 | 300 |
| JS heap (MB) | steady | run 1 | 410.14 | 704.38 | 470.68 | 1283.67 | 1305.12 | 1311.48 | 300 |
| JS heap (MB) | steady | run 2 | 413.32 | 526.79 | 508.67 | 656.83 | 1163.41 | 1168.84 | 300 |
| DOM nodes | steady | run 1 | 1476 | 1557.31 | 1554 | 1697 | 1713 | 1713 | 300 |
| DOM nodes | steady | run 2 | 1313 | 1526.22 | 1564 | 1627 | 1660 | 1660 | 300 |

`frame gap` and `frames/s` come from the page's own `requestAnimationFrame` timestamps (raw kept);
`renderer *` and `GPU frame` from `window.__gameStats()` (`?gpustats`, the renderer's own smoothed numbers);
`main-thread busy`/`script`/`layout` from CDP `Performance.getMetrics` deltas; `browser tree CPU` and
`GPU process CPU` from `ps` CPU-time deltas across the whole process tree (may exceed 100% across cores).
Startup and entry have no 1 Hz samples: their frames and perf spans carry them.

## Memory

At each stage boundary (the menu up, the household entered, the window over):

| run | boundary | renderer RSS (MB) | tree RSS (MB) | GPU proc RSS (MB) | JS heap (MB) | DOM nodes |
| --- | --- | --- | --- | --- | --- | --- |
| run 1 | startup | 2005.7 | 3096 | 331.7 | 589.5 | 533 |
| run 1 | entry | 4243.5 | 5912.3 | 760.4 | 1091.1 | 1483 |
| run 1 | steadyEnd | 3166.3 | 4915.5 | 833 | 590 | 1753 |
| run 2 | startup | 2037 | 3122.4 | 321.3 | 614.1 | 613 |
| run 2 | entry | 4380.7 | 6049.5 | 758.6 | 1158.4 | 1481 |
| run 2 | steadyEnd | 2899.9 | 4647.3 | 827.5 | 442.1 | 1763 |

Through the steady state (every 30 s):

| run | t (s) | renderer RSS (MB) | tree RSS (MB) | GPU proc RSS (MB) | JS heap (MB) | DOM nodes |
| --- | --- | --- | --- | --- | --- | --- |
| run 1 | 0 | 4283.2 | 5954.5 | 762.6 | 1105.99 | 1493 |
| run 1 | 30 | 4132.8 | 5807.1 | 766.6 | 1162.86 | 1554 |
| run 1 | 60 | 4226.7 | 5931.7 | 794 | 1230.86 | 1574 |
| run 1 | 90 | 4273.8 | 5994.9 | 809.3 | 1301.93 | 1701 |
| run 1 | 120 | 3357.2 | 5087.5 | 818.1 | 440.96 | 1501 |
| run 1 | 150 | 3182.9 | 4911.8 | 816.6 | 485.55 | 1513 |
| run 1 | 180 | 3133.9 | 4864.4 | 818.3 | 426.91 | 1551 |
| run 1 | 210 | 3082.5 | 4818.8 | 823.9 | 452.16 | 1543 |
| run 1 | 240 | 3049.5 | 4786.5 | 824.4 | 442.61 | 1564 |
| run 1 | 270 | 3049.9 | 4781.9 | 819.4 | 499.99 | 1568 |
| run 1 | 299 (end) | 3163.3 | 4910.8 | 832.4 | 578.03 | 1592 |
| run 2 | 0 | 4425.2 | 6097.8 | 760.1 | 1152.11 | 1489 |
| run 2 | 30 | 2973.4 | 4640.5 | 756.7 | 494.27 | 1366 |
| run 2 | 60 | 3048.1 | 4744.5 | 782.7 | 551 | 1382 |
| run 2 | 90 | 3100 | 4826.6 | 812.6 | 588.81 | 1497 |
| run 2 | 120 | 2878 | 4618.1 | 825.7 | 423.55 | 1521 |
| run 2 | 150 | 2918.9 | 4653.6 | 819.9 | 509.92 | 1564 |
| run 2 | 180 | 2992.8 | 4728.3 | 820.6 | 572.45 | 1598 |
| run 2 | 210 | 3146.7 | 4882.5 | 820.9 | 638.01 | 1627 |
| run 2 | 240 | 2928.7 | 4674.6 | 830.8 | 430.23 | 1609 |
| run 2 | 270 | 2831.1 | 4571.2 | 825 | 454.38 | 1606 |
| run 2 | 299 (end) | 2881.9 | 4626.6 | 826.4 | 449.23 | 1619 |

run 1: renderer 4283.2 → 3163.3 MB (peak 4398.8, -1119.9 MB), tree 5954.5 → 4910.8 MB (peak 6056.6), GPU proc 762.6 → 832.4 MB, heap 1105.99 → 578.03 MB (peak 1311.48), DOM nodes 1493 → 1592.
run 2: renderer 4425.2 → 2881.9 MB (peak 4532, -1543.3 MB), tree 6097.8 → 4626.6 MB (peak 6190.7), GPU proc 760.1 → 826.4 MB, heap 1152.11 → 449.23 MB (peak 1168.84), DOM nodes 1489 → 1619.

**How to read it.** The renderer's RSS is the primary number: the tab that runs the game. The *tree RSS*
column is the plain sum of every Chromium process's RSS, so shared pages (the framework, the code, the
GPU buffers) are counted once per process and the figure is a ceiling, not a footprint: do not compare it
with Activity Monitor's total. `ps` RSS also over-counts a process's own shared and file-backed pages.
The JS heap is V8's `JSHeapUsedSize` as CDP reports it, unforced (no GC before the read), so it includes
garbage waiting for a collection. `phys_footprint` (from `/usr/bin/footprint`, which suspends the process
for a few hundred ms while it reads it, hence only at the edges of the window) is the number Activity
Monitor would show for the renderer:

run 1: renderer phys_footprint 3819 → 2320 MB.
run 2: renderer phys_footprint 3984 → 1978 MB.

## Disk and network

| run | stage | requests | encoded | decoded | from cache | external |
| --- | --- | --- | --- | --- | --- | --- |
| run 1 | startup | 781 | 53868 kB | 57248 kB | 0 | 0 |
| run 1 | entry | 1191 | 117854 kB | 131756 kB | 0 | 0 |
| run 1 | steady | 119 | 1463 kB | 1648 kB | 0 | 0 |
| run 2 | startup | 910 | 55970 kB | 64904 kB | 0 | 0 |
| run 2 | entry | 1056 | 116327 kB | 123920 kB | 0 | 0 |
| run 2 | steady | 129 | 2970 kB | 3207 kB | 0 | 0 |

External (non-localhost) requests: run 1: 0, run 2: 0. The game made none at runtime.

Disk bytes read/written by the processes are **not measured**: macOS has no per-process I/O counters
without root (`fs_usage`/`iotop` need sudo). What is reported instead: the browser's wire bytes per
request (CDP `encodedDataLength`), the page's resource timings (`decodedBodySize`, i.e. server response
bytes after decoding), and cache hits.

## Stability: run 1 vs run 2

| metric | run 1 mean | run 2 mean | Δ | run 1 p99 | run 2 p99 | Δ p99 |
| --- | --- | --- | --- | --- | --- | --- |
| frame gap (ms) | 16.67 | 16.68 | +0.1% | 16.67 | 16.67 | +0.0% |
| frames/s (counted) | 59.98 | 59.95 | -0.1% | 60 | 60 | +0.0% |
| renderer fps | 59.91 | 59.93 | +0.0% | 60 | 60 | +0.0% |
| renderer frame (ms) | 16.7 | 16.69 | -0.1% | 17.71 | 17.88 | +1.0% |
| main-thread frame work (ms) | 3.48 | 3.44 | -1.1% | 6.72 | 6.63 | -1.3% |
| GPU frame (ms) | 8.4 | 8.76 | +4.3% | 10.41 | 11.24 | +8.0% |
| draw calls | 621.84 | 583.08 | -6.2% | 1034 | 995 | -3.8% |
| main-thread busy (%) | 25.84 | 26.05 | +0.8% | 44.11 | 48.11 | +9.1% |
| script (%) | 20.64 | 20.4 | -1.2% | 31.53 | 31.12 | -1.3% |
| layout (%) | 0.36 | 0.36 | +0.0% | 0.42 | 0.41 | -2.4% |
| browser tree CPU (%) | 77.48 | 79.39 | +2.5% | 172 | 166 | -3.5% |
| GPU process CPU (%) | 36.24 | 36.58 | +0.9% | 41.2 | 43 | +4.4% |
| renderer RSS (MB) | 3498.79 | 3018.24 | -13.7% | 4396.2 | 4488.6 | +2.1% | **unstable**
| tree RSS (sum of per-process) (MB) | 5219.32 | 4740.32 | -9.2% | 6055.5 | 6160.3 | +1.7% |
| GPU process RSS (MB) | 808.83 | 807.79 | -0.1% | 832.8 | 831.3 | -0.2% |
| JS heap (MB) | 704.38 | 526.79 | -25.2% | 1305.12 | 1163.41 | -10.9% | **unstable**
| DOM nodes | 1557.31 | 1526.22 | -2.0% | 1713 | 1660 | -3.1% |

Marked **unstable** where the means differ by more than 10% or the p99s by more than 20%.

## What could not be fixed or measured

- GPU utilisation (%): not exposed by the browser. GPU frame time (`RenderStats.gpuMs`) and the GPU
  process's CPU time are reported instead.
- Disk I/O per process: needs root on macOS (above).
- The display: the host's refresh rate and compositor still schedule frames; a 60 Hz display caps at
  60 fps and the game paces to it (`refreshDivisor`). Headless uses the same GPU but no vsync of a
  visible window, so frame pacing can differ from a headed run (`--headed`).
- Other load on the machine (other browsers, builds) is not controlled; the two runs are sequential,
  not isolated.
- Chromium version and flags are recorded; the page's own settings are seeded and fixed, but
  `Math.random` seeding covers the game's generation only, not the browser's internals.

## Raw output

- run 1: `/Users/virtual/Documents/Workspace/idyll-lives/bench-results/gameplay-2026-11-16-20-39-27/run1/raw.jsonl`, `/Users/virtual/Documents/Workspace/idyll-lives/bench-results/gameplay-2026-11-16-20-39-27/run1/summary.json`
- run 2: `/Users/virtual/Documents/Workspace/idyll-lives/bench-results/gameplay-2026-11-16-20-39-27/run2/raw.jsonl`, `/Users/virtual/Documents/Workspace/idyll-lives/bench-results/gameplay-2026-11-16-20-39-27/run2/summary.json`

## Production gating (verified, not assumed)

A production build was made from this tree (`cd web && bunx vite build --logLevel warn`, 2026-11-16 21:52)
and every emitted script was searched:

```
grep -rho "__gameStats\|gpustats\|GAMEPLAY_STATS" web/dist/assets/*.js   # 0 hits
grep -rho "measureGpu([^)]*)" web/dist/assets/*.js
   measureGpu(!1,`bench`)          # bench.html, bench mode: off
   measureGpu(a.gpu,`bench`)       # bench.html, bench mode: on for its own overlay
   measureGpu(e,t=`overlay`)       # renderer default
   measureGpu(o.perfOpen||!1)      # session.ts: only ?profile turns it on
```

`import.meta.env.DEV` is constant-folded to `false`, so `GAMEPLAY_STATS` is `false`, the
`window.__gameStats` assignment is eliminated, and the session's call becomes
`measureGpu(perfOpen || false)`. A release build has no hook, no `?gpustats` handling and no GPU-timer
change unless `?profile` is used (which is existing, documented behaviour). When the flag is off in dev,
the only added work is one `params.has('gpustats')` at startup and one boolean in the 500 ms stats timer.

## Files added or changed

| file | state | what |
| --- | --- | --- |
| `tools/bench/gameplay.mjs` | new | the harness: stages, 1 Hz sampling, rAF frames, CDP network/perf, `ps`/`footprint`, summary, report, fail-loud checks |
| `web/src/debug/gameplayStats.ts` | new | dev-only `?gpustats` hook exposing `window.__gameStats()` (renderer stats + per-pass GPU times) |
| `web/src/game/session.ts` | modified | installs the hook in dev and turns the GPU timer on for `?gpustats` (`measureGpu(game.perfOpen \|\| GAMEPLAY_STATS)`); no other change |
| `tools/bench/README.md` | modified | "The gameplay baseline (cold, from process start)" usage section |
| `docs/bench/gameplay-baseline.md` | new | this report (the generated copy stays in `bench-results/`, which is gitignored) |

No gameplay, simulation or rendering behaviour was changed; `web/dist` is gitignored and was rebuilt only
to check the gating above.
