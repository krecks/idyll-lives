# Optimisation 1: geometry conversion and merge

First measured step against [`gameplay-baseline.md`](gameplay-baseline.md). One change, before and
after on the same machine, same harness settings.

**Environment:** MacBookPro18,1 · Apple M1 Pro (10 cores) · 32 GB · macOS 27.2.1 (26B131) ·
Chromium 156.0.8078.4 (Chrome for Testing) · commit `89b684a6` + this change (dirty tree) ·
1280×720 @1× · medium preset · WebGPU · seed 7 · cold runs · dev build (`vite`).

## What the profile said

`?profile` spans and a CPU profile of startup and entry put both stages on the CPU, not the network:

| stage | total | biggest spans |
| --- | --- | --- |
| startup | 8.4 s | `boot: menu town` 6.0 s (showcase land 3.0 s, street plan 2.1 s, terrain tiles 1.7 s, forests 1.5 s), `boot: portraits` 1.4 s |
| entry | 11.5 s | `session: renderer setWorld` 5.6 s (street plan 2.4 s, nature 2.1 s, meshes 1.0 s), `warm game` 2.5 s, `SimBridge.start` 2.1 s |

`src/render/shared/geometry.ts` was the largest single module in both profiles (0.96 s of startup,
2.60 s of entry; `meshData` alone 0.52 s / 1.43 s) and the largest allocator: 1.03 GB of `Geo.vertex`
allocation, 853 MB in `Geo.merge`, 890 ms of GC during entry.

## The change

`web/src/render/shared/geometry.ts` only:

1. `meshData()` converted colours with `Uint8Array.from(this.col, cb)` (a JS callback per component,
   ~4 per vertex) and then scanned the whole array again with `.some()` to decide `vertexAlpha`.
   It now does one loop: quantise, and set `vertexAlpha` when an alpha below 1 is seen.
   The rounding is `v <= 0 ? 0 : v >= 1 ? 255 : (v * 255 + 0.5) | 0`, identical to the old
   `Math.round` + clamp for every input.
2. `append()` and `merge()` copied vertices with `push`/spread, growing a plain array element by
   element (spread also builds an argument list). Both now pre-size the destination once (`grow`)
   and copy with an index loop (`copy`).

No geometry, colour, winding or draw order changed.

### Correctness

- `bunx tsc --noEmit -p tsconfig.json` — clean.
- `bun test tests/meshCpuCopy.test.ts tests/streetChunks.test.ts tests/houseCar.test.ts` — 19 pass, 0 fail.
- A throwaway check built the same `Geo` (box, poly, append, merge) at alpha 1 / 0.5 / 0.999 / 0 and
  compared positions, normals, uvs, indices, colours, cut mask and `vertexAlpha` byte for byte
  against a copy of the old implementation: identical.
- Steady state still holds 60 fps (below), which it would not if a mesh had lost its indices or colours.

## Results

The 300 s runs carry ~12 % entry variance on this machine, so the headline is an interleaved A/B:
two 20 s runs with the change stashed, two with it applied, back to back.

| metric | before (2 runs) | after (2 runs) | delta |
| --- | --- | --- | --- |
| startup (ms) | 8691 / 8372 | 7790 / 7908 | **−8.0 %** (mean 8531.5 → 7849) |
| entry (ms) | 11902 / 11945 | 9955 / 10011 | **−16.1 %** (mean 11923.5 → 9983) |
| steady frames in 20 s | 1198 / 1198 | 1196 / 1198 | unchanged, 60 fps |
| frame gap p99 (ms) | 16.67 | 16.67 | unchanged |

Full harness runs (`--runs 2 --secs 300`, cold), compared with the two baseline runs:

| metric | baseline (mean) | after (mean) | delta |
| --- | --- | --- | --- |
| startup (ms) | 7989.5 | 7511.0 | −6.0 % |
| entry (ms) | 11335.5 | 10265.5 | −9.4 % |
| JS heap peak (MB) | 1240 | 930.9 | −24.9 % |
| renderer RSS, steady mean (MB) | 3258.5 | 3275.1 | +0.5 % (unchanged) |
| tree CPU, steady mean (%) | 78.44 | 77.77 | −0.9 % (unchanged) |
| main-thread busy (%) | 25.95 | 25.15 | −3.1 % |
| GPU frame (ms) | 8.58 | 7.31 | not claimed: GPU state varies |
| draw calls | 602.5 | 582.5 | unchanged |
| steady frame gap p99 (ms) | 16.67 | 16.67 | 60 fps held |

Heap peak moved further than the other memory numbers and the baseline peaks themselves varied by
12 % (1311 / 1169 MB); treat −24.9 % as "roughly a quarter less peak heap, same direction in both
runs" rather than a precise figure. Renderer RSS did not move, which fits: the change removes
short-lived plain arrays and a second colour buffer, not resident textures or geometry.

Raw: `bench-results/gameplay-opt-geom1/` (gitignored) — `report.md`, `summary.json`,
`run1/raw.jsonl`, `run2/raw.jsonl`.

## Ranked candidates

| # | candidate | expected | status |
| --- | --- | --- | --- |
| 1 | `Geo` typed arrays / single-pass `meshData` + pre-sized merge | ~0.5 s startup, ~1.5 s entry, ~300 MB peak heap | **implemented**, measured above |
| 2 | reuse the streamed bytes instead of re-fetching every GLB (590 files / 60 MB fetched twice in dev; `Cache-Control: no-cache` makes the second request a full 200) | ~0 s entry (CPU-bound), less I/O | not done: entry is CPU-bound, so it buys bytes, not seconds |
| 3 | build the menu showcase land lazily or off the critical path (3.0 s of the 6.0 s menu town) | ~2–3 s startup | not done: changes when the menu town appears, a visual/UX change |
| 4 | portrait studio warm-up (1.1 s) | ~1 s startup | not done: risks a hitch when the first portrait opens |
| 5 | defer town nature / far chunks until after the loading screen | ~2 s entry | rejected: visible pop-in |
| 6 | IK clip baking in startup (0.73 s) | ~0.7 s | rejected: animation output would change |

## What is left

- Startup is still ~7.5 s, of which ~6 s is building the menu town on the main thread and in a
  worker; the next real win needs the showcase land built lazily (candidate 3), which is a product
  decision, not a mechanical one. Confidence it saves 2 s: high; confidence it keeps the menu looking
  the same: medium.
- Entry is still ~10 s, dominated by `setWorld` (5.6 s) and `warm game` (2.5 s). Geometry conversion
  is now cheaper; the remaining cost is building the town itself. Confidence in another 2 s from
  geometry work alone: low.
- Tree CPU at ~78 % is the GPU process (~37 %) plus the renderer; it did not move, as expected.
- Disk bytes per process remain unmeasurable without root.

## Files changed

| file | state | what |
| --- | --- | --- |
| `web/src/render/shared/geometry.ts` | modified | single-pass colour conversion + `vertexAlpha`; pre-sized `append`/`merge` copies |
| `docs/bench/optimisation-geometry.md` | new | this report |

Production gating re-checked after the change: `bunx vite build`, then
`grep -rho "__gameStats\|gpustats\|GAMEPLAY_STATS" web/dist/assets/*.js` → 0 hits; the session call
folds to `measureGpu(o.perfOpen||!1)`.
