# Load timing: page open → into a household (0.8.3)

Measured on branch `perf/load-timing` (from `release/0.8.3`, commit `97fc22c43`). Nothing in the
game was changed; only measurement code was added (`tools/bench/loading.mjs`, below).

## How it was measured

- Harness: `bun tools/bench/loading.mjs --runs 5` (the repo's own loading benchmark). It drives the
  real menus with Playwright: open the page, "New game", "Lay out the streets", "Build the town",
  a household's "Play", "Move in".
- Browser: Chromium 1248 (Playwright 1.64.1), headless, 1920×1080, device scale 1, WebGPU, fresh
  browser context per run (a first visit: no HTTP cache, no service worker, no save).
- Server: the Vite dev server the harness starts itself (`http://localhost:5222`), i.e. unbundled
  modules and a dev build of the WASM. `?profile` was on, so the page's own spans
  (`web/src/debug/perf.ts`) were collected as well.
- World: default new game, `--seed 7` (the page's `Math.random` is seeded, so every run builds the
  same land, the same 281 homes and 610 residents). Household card 0 is played.
- Machine: Apple M3 Pro, 12 cores, 36 GB, macOS 27.2.1.
- 5 runs; each run is a new context. Medians are reported with min–max and every run's value.

Instrumentation added (kept on the branch, `tools/bench/loading.mjs`): the benchmark already timed
each phase, but threw the absolute marks away. They are now returned as `marks` (page times, ms
since navigation start) and written to each run's JSON, and the report prints
`total (page load → game up)` = `marks.entered`. That is the only change; the game is untouched.

## Per-phase timings (ms, 5 runs)

| phase (what it covers) | median | min–max | share of total | runs |
|---|---|---|---|---|
| **total: page load → game up** | **30 843** | 29 462–34 420 | 100 % | 33077, 30843, 34420, 29475, 29462 |
| boot: page open → main menu shown | 1 701 | 1 597–2 465 | 5.5 % | 2465, 1657, 1701, 1830, 1597 |
| world: "New game" → land made (step 1 ready) | 1 856 | 1 807–2 620 | 6.0 % | 2620, 1856, 2030, 1835, 1807 |
| layout: "Lay out the streets" → step 2 ready | 5 338 | 5 013–5 760 | 17.3 % | 5760, 5483, 5338, 5108, 5013 |
| town: "Build the town" → step 3 ready | 4 683 | 4 518–5 678 | 15.2 % | 5106, 4683, 5678, 4518, 4614 |
| portraits: step 4 opened → all 70 faces drawn | 1 332 | 1 271–1 462 | 4.3 % | 1382, 1271, 1462, 1332, 1283 |
| home: household's "Play" → step 5 ready | 472 | 433–563 | 1.5 % | 563, 472, 536, 433, 436 |
| **entry: "Move in" → game up** | **10 213** | 9 634–12 776 | 33.1 % | 10213, 10603, 12776, 9634, 9929 |

Not in the table: 4 000 ms of deliberate settling waits the harness inserts between steps, and
1 396 ms (median) for step 4 to open before the faces are timed. Phase medians + those two account
for the total.

Spread: the end-to-end total varies by ~5 s (±8 %) and `entry` by ~3.1 s, so the ranking is stable
but single runs are not trustworthy.

## The bottleneck

**Going into the household (`entry`, 10.2 s, a third of the whole flow) is the biggest single
consumer.** Inside it (page spans, median of the 5 runs):

| span | median ms | min–max | share of entry |
|---|---|---|---|
| `host: prepare (start)` (= `session: start→ready`) | 10 130 | 9 512–12 690 | 99 % |
| └ `session: renderer setWorld` | 4 811 | 4 559–6 035 | 47 % |
| └ `session: warm game` | 3 371 | 3 087–4 023 | 33 % |
| └ `session: SimBridge.start` | 1 283 | 1 223–1 625 | 13 % |
| └ `session: assembleTown` | 287 | 273–429 | 3 % |

The second-largest cost is not a single phase but one step repeated at every step of the wizard:
`menu: showTown (renderer)` — the menus' town thrown away and rebuilt — **9 433 ms per run in total
(31 % of the whole flow)**, in 5 calls: 2 057 ms after the land is made, 2 117 ms while the layout
is proposed, 4 350 ms after the town is built, 1 145 ms when step 4 opens, 317 ms at step 5. It is
85 % of the `town` phase and 37 % of `layout`.

So: the household entry (renderer build + warm-up) and the per-step town rebuild together are
~64 % of the time from opening the page to standing in a home.

## Likely causes, with the evidence

1. **Building the game's world in the renderer (`setWorld`), not loading files.** 4.8 s of the
   entry; it is the single largest span of the whole flow. `render: nature` alone is 2 521 ms over
   6 calls (`web/src/render/pc/lotContents.ts:293`), i.e. landscape meshes and
   `refreshLawn` are rebuilt. The town is 281 homes and 610 residents (identical in every run), and
   step 5 streams the rest of it (`session: street streamed` 9 ms only, so it is not network).
   Variance (4 559–6 035 ms) points at CPU mesh building + GPU buffer/texture upload, not I/O.

2. **First-frame shader/pipeline compilation (`warmGame`).** 3.4 s for one frame drawn behind the
   loading screen (`web/src/render/pc/PlayCanvasRenderer.ts:2967`). The source says each first
   pipeline "costs its frame tens of ms"; `warm: lots` (1 867 ms over 4–5 lots, only for big
   worlds) and `render: nature` (2 521 ms over 6 calls) sit inside this work. The run that paid the
   most here (4 023 ms) is also the run with the slowest entry (12 776 ms), which fits
   compile-on-first-use.

3. **The menus rebuild the whole town for every wizard step.** `showTown` is called 5× per run for
   9.4 s total; the biggest call (4 350 ms) is right after "Build the town", when the newly built
   town is handed to the renderer. Same cause as (1) — the town's geometry, nature and textures are
   rebuilt rather than reused — and it is what makes steps 1–4 slow.

4. **Portraits are rendered on the CPU/GPU at boot and again for the cards.** `boot: portraits`
   1 189 ms of the 1 701 ms boot, made of `preview: studio` 935 ms and `portraits: batch`
   2 237 ms over 6 batches; step 4 then pays 1 332 ms for its 70 faces. Each portrait is a
   separate preview render.

5. **Simulation start (`SimBridge.start`) is comparatively cheap**: 1 283 ms, 13 % of the entry, and
   the town assembly (`assembleTown`) 287 ms. So the time is in drawing/building the world, not in
   the Rust simulation's own start-up.

## What I could not verify

- **Dev server, not the production build.** The numbers include Vite's on-demand transforms and the
  un-optimised WASM (no `wasm-opt -O3`, no bundling, no hashed caching, no service worker). A
  production `bun run build` + `preview` run would be the fair player-facing number; it was not
  measured here.
- **No GPU/CPU split.** The spans bracket JS calls; how much of `setWorld` / `warmGame` is CPU mesh
  building versus GPU upload/pipeline compile was not separated (no GPU timings, no tracing).
- **No main-thread long-task data.** The harness builds its own report (`entries` + `marks`), so
  `perf.ts`'s `longTasks` / `droppedFrames` were not carried into the JSON; blocking was only
  inferred from span durations.
- **First visit only.** Each run is a fresh context, so nothing is cached. A returning player
  (HTTP cache / service worker) was not measured.
- **One machine, one seed, one renderer.** Apple M3 Pro, WebGPU, seed 7, default world size; the
  WebGL2 fallback, other hardware and other worlds (big/`--world`) were not measured.
- `?profile` adds a `PerformanceObserver` and console logging; its cost is assumed small but was not
  measured.

## Round 2: what was changed, and what it bought

Three changes on `perf/load-timing` (game logic untouched):

1. **A town is not rebuilt when its draft is unchanged** (`web/src/game/menuScene.ts`). The menus
   hand `showTown` a fresh `NeighbourhoodDraft` object for the same draft on every call (the
   ChooseHome hover, going back a step), so the identity check `draft === this.draft` never held and
   the whole town was thrown away and built again. `show()` now keeps a `shownKey` (the home and the
   draft's JSON) and returns early when it matches the town that is up.
2. **Portraits no longer hold up the boot** (`web/src/App.svelte`, `boot()`). `boot: portraits`
   (1 321 ms: `preview: studio` + 6 batches) was awaited before the menu was shown; it now runs
   behind the menu picture (a portrait asked for meanwhile queues behind it). Boot phase 1 701 →
   1 550 ms, with 1 266 ms of that work now off the critical path.
3. **The game's world reuses the landscape the menus built** (`web/src/render/shared/nature.ts`).
   Two things stopped it: `baseKey`/`relayKey` serialised the look and the options with
   `JSON.stringify`, so the menus' draft's world and the sim's world (the same land, keys in another
   order) had different keys — now serialised canonically (keys sorted); and `takeRoads` compared
   `meta.roads` by identity, so the shared land re-solved its whole surface for roads that had not
   moved — 6 673 ms of `land: roads taken` — now compared by value (width, kind, points).
   With both, the entry builds no landscape at all: `render: nature` 2 766 → 299 ms, the rest of the
   nature work gone, and `land: roads taken` 0.

### After (3 runs, same machine, same protocol, seed 7)

| moment | median ms | runs |
|---|---|---|
| total (page load → game up) | 30 170 | 31 320, 30 170, 28 913 |
| boot | 1 550 | 2 415, 1 550, 1 452 |
| town | 5 511 | 5 511, 5 593, 5 228 |
| entry | 8 487 | 8 646, 8 487, 8 085 |

Entry, span by span (median): `setWorld` 5 029 → 2 856, `render: nature` 2 766 → 299,
`land: roads taken` 6 673 → 0, `warmGame` 3 371 → 3 391, `SimBridge.start` 1 283 → 1 310.

Caveat: the machine drifted about 8 % slower between the two sessions (`menu: showTown`, which no
change touched, went 9 603 → 10 323 ms), so totals are not directly comparable run to run; the span
comparisons above are within one session's spread.

### What is left

- `warmGame` 3 391 ms (40 % of the entry), of which `warm: lots` ~2 800 ms over 5–6 lots
  (`WARM_LOTS` in `web/src/render/shared/focus.ts`): a deliberate trade for smooth roaming.
- `SimBridge.start` 1 310 ms.
- The menus' five wizard rebuilds: `menu: showTown` 10 323 ms over 5 calls, 85 % of the town phase.
  The land → layout → town steps are genuinely different towns; only the hover case was waste.
- `street: plan` in the game, and `portraits: batch` 2 969 ms for the 70 cards at step 4.
