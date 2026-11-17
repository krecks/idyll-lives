# Memory plan: what is load-bearing, what is waste

Source numbers: `docs/bench/gameplay-baseline.md` (runs 1–2, `bench-results/gameplay-2026-11-16-20-39-27/`),
macOS, medium preset, WebGPU, 300 s steady. Peak is reached **at household entry**, not in steady state.

## 1. Verdict

The game does not need 4 GB. At the peak, only **1.31 GB is JS heap** (measured, `JSHeapUsedSize`) and at
most **~0.46 GB is plausibly PlayCanvas VRAM** (derived from the assets on disk). That leaves **~2.3–2.8 GB
of the 4.40 GB peak with no owner**. The shape of the curve says where it is: entry adds **+2.2 GB of renderer
RSS but only +0.5 GB of heap** (2006 → 4244 MB vs 590 → 1091 MB), then 120 s later RSS falls 900 MB and heap
falls 860 MB — allocation that is never returned to the OS. Prime suspect is **sim-wasm linear memory**
(worldgen at entry, never shrunk) plus GPU uploads, not the asset set. Working estimate of load-bearing
memory: **~1.5–1.8 GB** (440–580 MB post-GC heap + visible-world VRAM + live sim state). Everything above
that, up to the 4.4 GB peak, is churn or un-returned allocation until measured otherwise. On 8 GB Windows
the renderer budget is ≈3 GB (OS + browser + the 830 MB GPU process), so the peak is **~1.5 GB over**.

## 2. Memory ledger (peak, run 1)

| line | MB | source | confidence |
| --- | --- | --- | --- |
| renderer RSS (peak) | 4398.8 | `ps` 1 Hz, `raw.jsonl` | measured |
| JS heap used (peak) | 1311.5 | CDP `JSHeapUsedSize` — **excludes ArrayBuffers** | measured |
| sim wasm linear memory | **60.6** | `__gameStats().simMemoryBytes` — the worker's own report (smoke run, 20 s) | measured |
| PlayCanvas VRAM (tex+vb+ib+ub+sb) | **464.5** (290.2 textures, 166.5 geometry) | `__gameStats().vram` (`app.stats.vram`) | measured |
| CPU-side texture copies | 0–340 | `Texture#releaseSourceAfterUpload` (`node_modules/playcanvas/build/playcanvas.mjs:5558`), default `false` | unknown |
| ORT (LLM 190 MB + TTS 87 MB on disk) | **0** | entry downloaded 118 MB total, steady 1.5 MB — the 277 MB of `*.onnxz` was never fetched | measured by absence |
| V8 external ArrayBuffers (GLB parse, image decode) | unknown | `Runtime.getHeapUsage`, heap sampling | unknown |
| unattributed (driver/GPU pages, fragmentation, allocator) | ~2300–2800 | remainder | unknown |

Adds up: 1311.5 + ~390 (VRAM) + ~170 (copies) + 0 (ORT) + ~2530 (remainder) ≈ 4400 MB.

## 3. Ranked plan

| # | item | evidence | saving peak / mean | stability & frame time | effort | risk | verify |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ~~**Sim-wasm memory at worldgen**~~: **falsified** — the sim's linear memory is 60.6 MB, flat through the window | measured, `simMemoryMB` | ~0 | — | — | — | closed |
| 2 | **Entry-time JS garbage**: build the household in chunks and release intermediates; force GC before steady | 1311 MB peak vs 440 MB post-GC = 870 MB of garbage | 400–800 / 300–600 | none (load time may +0.2 s) | M | low | heap sampling at entry, peak vs post-GC |
| 3 | **Free CPU texture copies**: `releaseSourceAfterUpload = true` before upload | 0–340 MB derived; no code sets it today | ≤340 / ≤340 | none — `vram.tex` unchanged, pixels identical | S | low | RSS delta with `vram.tex` flat |
| 4 | **Bench the production build** (KTX2 + meshopt, `cargo assets`) instead of dev JPEG/GLB | dev bench downloads 118 MB of uncompressed assets; prod ships compressed copies | 150–300 in the *measured* number | none — may also change load time | S | none | re-run `--runs 2 --secs 300` on `vite preview` |
| 5 | **Texture budget**: clamp 63 JPEGs to 1024² and share mips | 7.4 MB encoded → ~250–340 MB decoded, 63 files | 60–120 / 60–120 | slight quality loss; no frame cost | M | low | `vram.tex` per-asset list |
| 6 | **Geometry**: 778 GLBs / 89.1 MB, 620 mean draw calls (1050 max) | dedupe repeated models, instance the rest | 20–60 / 20–60 | helps GPU frame (8.4 ms mean) | L | med | `vram.geom`, draw calls |
| 7 | **Low-memory guard for ORT**: skip TTS/LLM preload when `navigator.deviceMemory < 8` | models are 277 MB on disk, ~300–500 MB resident when loaded | 0 at bench peak / +300–500 avoided on 8 GB | none — voice becomes on-demand | S | low | RSS with voice on, 8 GB profile |

## 4. Leave it alone

- **Post-GC heap, 440–580 MB**: the live world, parsed meshes and character data. Cutting it means cutting content.
- **Visible-world VRAM (~250–460 MB)**: it buys the 8.4 ms GPU frame and 60 fps. Compress, don't delete.
- **LLM + TTS models (277 MB)**: not in the measured peak (never fetched), lazily loaded, and only cost when used. Guard them (item 7); don't restructure them.
- **DOM (1557 nodes, flat) and layout (0.36 %)**: irrelevant to memory and frame time.

## 5. Measurement gaps (landed)

1. ~~`stats()`: `vram`, `wasmBytes`, `heapBytes`~~ — `RenderStats.vram` (`web/src/render/types.ts`), filled
   in `PlayCanvasRenderer.stats()` from `app.stats.vram`; the sim worker's bytes ride along as
   `GameplayStats.simMemoryBytes` (`web/src/debug/gameplayStats.ts`, from `bridge.threadStats`).
2. ~~New report rows~~ — `GPU memory`, `GPU memory: textures`, `sim WASM memory` in the per-metric table,
   and the `## GPU memory and the simulation` section with per-boundary rows.
3. ~~Every renderer process~~ — `rssRenderersMB`/`rendererProcs` per sample, `renderersRSS` per boundary
   (the largest renderer is still reported, but the sum is what is attributed).
4. ~~Heap profile~~ — `--heapprofile` samples through the steady window and reports the 20 sites that kept
   the most (`heapprofile.json`).
5. **Still open**: the per-asset texture list (`app.assets.filter(a => a.type === 'texture')`), to split
   asset textures from render targets inside the 290 MB `textures` figure.

## 6. First three actions

1. ~~10 minutes, no code: read VRAM + wasm at entry~~ — **done**, see §7.
2. **1 hour**: run the existing bench against the production build (item 4). If the Windows crash does not reproduce there, the dev build is the problem and the plan shrinks.
3. ~~Half a day: land gaps 1–3~~ — **done** (§5); every run now carries the ledger as columns.

## 7. What the measurements showed

The counters the plan asked for now exist and print in every report: `JS external` (`Runtime.getHeapUsage`
`backingStorageSize`, the ArrayBuffers `JSHeapUsedSize` cannot see), `GPU memory`/`textures`/`geometry`
(`app.stats.vram`), `sim WASM` (`memory.buffer.byteLength` from the worker), `renderers RSS` (every renderer
process, not only the largest), and a canvas census. Two 300 s runs on the 0.8.3 tree
(`bench-results/ab-attrib/`) put the ledger on a scale:

| at | renderers RSS | JS heap | JS external | VRAM | sim WASM |
| --- | --- | --- | --- | --- | --- |
| menu up (startup) | 748.8 | 44 | 57 | — | — |
| household entered | 3976 | 576 | 598 | 480 | 66 |
| steady end | 3414 | 497 | 480 | 473 | 86 |

- **The entry spike is ArrayBuffers, and it is transient by construction.** External memory goes 57 MB →
  1.29 GB at entry → 494 MB at steady end (run 1 of the attribution pass), while VRAM (478 → 466) and sim
  WASM (66 → 84) barely move. GLB/JPEG parse buffers, geometry built and dropped, the Vite dev bundle:
  allocated and collected inside the window. A 20 s `--heapprofile` says the same from the other side — the
  20 sites that keep the most across the window sum to ~2 MB.
- **Canvas surfaces are not the answer.** The page has exactly one `<canvas>` — 1280×720, 3.5 MB — at every
  stage boundary (`bench-results/census/`). The town map's 16.8 MB `getImageData` buffer, the portrait
  canvases, the preview studio's second `pc.AppBase`: transient or already closed.
- **The GPU backend is not the answer.** Same tree, same 90 s, `--renderer webgl` is 138 MB *higher*
  (steady end 4248.9 vs 4111.3, `bench-results/census-gl/`), so the residue is not Dawn/Metal per-resource
  overhead and no amount of texture re-encoding changes it.
- **~650 MB is fixed renderer cost** — an empty menu already sits at 748 MB RSS against 44 MB heap and
  57 MB external. The scene accrues ~1.1 GB on top of that, invisible to V8, `backingStorageSize` and
  `vram` alike.

### The change that survived: release texture sources

PlayCanvas keeps the `ImageBitmap` behind every texture level until told otherwise
(`releaseSourceAfterUpload`, false by default; `playcanvas.mjs:5557`). Those bitmaps live in the browser's
image memory — not in the JS heap, not in `stats.vram` — so they are invisible to every counter above.
`setReleaseSourceAfterUpload()` (`:5661`) closes them now or after the upload (`:11032` WebGPU, `:15745`
WebGL), and nothing on this side re-reads a level once it is uploaded.

Landed in one place, `web/src/render/pc/device.ts` `releaseTextureSources(app)`, called from `createApp`:
every texture asset that reaches the registry — the game's own and the glTF containers' — and both apps
(the previews' side app is made through `createApp` too). The town's ground map and the preview stages hold
a typed array or a canvas instead of a bitmap, which the flag does not touch (their ceiling is the 16.8 MB
map buffer).

A/B on the same tree, 2 × 300 s each (`bench-results/ab-attrib/` → `bench-results/ab-relsrc-full/`):

| metric | before | after | Δ | spread |
| --- | --- | --- | --- | --- |
| renderers RSS mean | 3825.4 | 3616.2 | −5.5 % | ±3.0 % |
| renderers RSS peak | 4890.8 | 4338.3 | **−11.3 %** | ±3.0 % |
| JS external peak | 1048.0 | 737.7 | **−29.6 %** | ±5.2 % |
| JS external mean | 610.9 | 563.7 | −7.7 % | ±5.2 % |
| JS heap peak | 1690.2 | 997.0 | −41.0 % | GC timing |
| GPU memory mean | 478.2 | 484.8 | +1.4 % | — |
| GPU frame | 8.8 | 8.6 | −1.9 % | — |
| frame gap | 16.7 | 16.7 | 0.1 % | — |

The peaks are where the mechanism predicts an effect — the bitmaps are freed as they upload instead of
accumulating until the entry burst ends — and they move by 310–553 MB, beyond the spread. The steady mean
(−209 MB) is inside the run-to-run noise (baseline runs: 4344 / 3307 MB), and the JS heap columns move by GC
timing rather than by this change, so they are reported and not claimed. Frame time is untouched: 16.67 ms
p99 in both arms, GPU frame 1.9 % faster.

### What is left

The steady residue (~1.1 GB that no counter owns) is not addressed. It is not canvases, not the GPU backend,
not VRAM, and not live JS objects; the remaining candidates are committed-but-freed V8 pages and allocator
retention, which is a different problem from the entry burst — and the entry burst (1.29 GB of ArrayBuffers,
2.6 GB of committed heap at peak) is the larger number and the one to attack next.
