# Idyll Lives: Technical Plan

*Status: **v0.2.0** (notes in [CHANGELOG.md](CHANGELOG.md)), after v0.1.0 (2026-10-10), the first release. 0.2.0 brought babies, cooking, the phone, basements, generated worlds, Supertonic 3 voices and an optional language model for residents' lines, and went through a release review whose findings, fixes and leftovers are in [docs/release-0.2.0-review.md](docs/release-0.2.0-review.md). Version numbers 0.2–0.20 in the milestone table below are the development builds that led up to 0.1.0; their tags and releases were retired when 0.1.0 shipped. Direction changed on 2026-10-09: see "Game direction" below.*

A home-building game with a living simulation that runs entirely in the browser: Rust/WASM simulation core, a GPU renderer, a Svelte 5 + CSS interface, and saves stored in the browser.

## Game direction: build your home, then watch it live

**Building and designing your home is the main activity. The residents live in it on their own, and you watch them, like an aquarium or an ant farm.** You build everything; the simulation does the rest.

- **Watching is the default.** Left alone, Live mode directs its own camera: it glides to conversations, fights, first kisses, someone trying out the thing you just built, guests arriving, and otherwise slowly orbits the house or looks into a room. Any input hands the camera back.
- **You steer, you don't command.** Each resident has a **planner**: a weekly calendar of routines ("train 1 h at 18:00 Mon/Wed/Fri", "sleep 23–7", "cook dinner") and life goals ("get a job", "get promoted", "find love"). Routines are a strong nudge, not an order: urgent needs and traits can win, and mood and discipline decide how well residents stick to them. Residents also suggest goals of their own. Direct control (click an object, talk to someone) stays available from the Inspect panel.
- **The house matters.** Room size, light, decor, cleanliness and what a room is for change how residents feel and what they choose. A missing bed or toilet has visible consequences. Residents form opinions about rooms and wish for things, and wishes and planner gaps link straight to the catalog. Watching tells you what to build next.
- **Lives move on.** Residents find and lose jobs on their own, age, fall in love, move in together, have children, grow old, move out or pass away; newcomers arrive in empty houses. The house has to grow with the family.
- **One home, a living town.** You build one home; the rest of the neighbourhood keeps simulating and neighbours visit.
- **Creative or Living.** Each game is either Creative (building is free) or Living (the residents earn the money). Build and Buy mode pause time.
- **Start from an empty lot** or a ready-made house, with a family you design, a random one, or none yet ("build first, they move in later").

**Ground rules (from the project owner):**
- **Performance is the top priority.** The CPU runs the game logic; the GPU does everything visual.
- **Desktop only.** No mobile or tablet targets.
- **Modular and easy to maintain.** The project will be heavily adapted; every concern lives in its own module behind a small interface.
- **Art will be replaced.** Code and content refer to assets only by key; files are swapped through the asset manifest or override packs.
- **Art style:** modern, minimalist and stylised.

## Status

| Area | State |
|---|---|
| Sim core (Rust) | Needs, utility AI with smart objects, A* pathfinding, lot/rooms, mesh generation, clock. **Town** of plots and households with object ownership. **Social life**: directional relationships (friendship, romance, chemistry), 17 data-driven social interactions with success chances, feelings and emotions that steer behaviour, jealousy, partners and break-ups, fights, story events. **Gender and attraction**. **Daily life**: data-driven careers with shifts, pay, performance and promotions; time-of-day schedules; household funds; neighbours visiting each other (guests stay on the host's lot and use only non-private objects). **Careers and skills**: 1000 positions (25 categories × 4 tracks × grades A–J) expanded from `careers.json`; 13 skills (12 in the base game, Gardening with the garden) trained at work and with home objects; skill fit decides hiring (probation), the workweek length and promotions. **Money**: dollars, weekly rent by lot size plus bills by household value, per-use costs only for consumables, debt. **Day/night rhythm**: sleep through the night, early risers for early shifts, slower need decay asleep, practice fatigue. **Build/buy** (`home.rs`): buy (auto or exact placement), move, sell, recolour (visual only), upgrade quality (instant Buy-mode purchase), walls (straight and diagonal), doors and windows with reachability checks; multi-slot objects (double beds). **0.2.0:** cooking (recipes as chains of stations, with food and dishes as items), babies cared for in the crib, a phone for everyone from teens up, basements, guests keeping their hosts company, up to four in a conversation, residents' own wishes; an integer clock (20 ticks a game minute); content checked when it loads; object ids remapped on every edit and commands guarded by `seen`. Save format version 23 (the starting seed; older saves, 0.1.0's version 17 included, are migrated on load). |
| WASM bridge | Sim in a Web Worker at 20 Hz; seqlock snapshot over SharedArrayBuffer (with conversation, emotion and animation fields); requests for saves and social options. |
| Renderer | **Only the viewed lot is drawn** (its walls/floors are generated per region; other houses appear as silhouettes; residents elsewhere or at work are hidden); the view follows the selected resident. Babylon.js 9.30 on WebGPU (WebGL2 fallback), thin instances everywhere, time-of-day lighting with a gradient sky dome and distance fog, procedural landscape (terrain hills, clustered forests, lawn detail), roads, swappable hair models, procedural conversation body language. The game draws live; WebGPU snapshot rendering replays the menus' town. **0.2.0:** three graphics presets in one table (Low, Medium, High), Auto resolution with a pixel budget of about 1920×1080, and a frame rate cap paced in whole display refreshes ([docs/design/graphics.md](docs/design/graphics.md)). The release review's benchmark (`?bench=village`, Medium, 1080p, M3 Pro, at its start): 493 draw calls and 6.0 M triangles a frame, 10.3–10.5 ms of GPU time. |
| UI | Main menu, settings, credits, load/save with thumbnails, pause menu. **Menus over a live 3D town** (`render/babylon/overview.ts`, `game/menuScene.ts`): the shared renderer draws a whole-neighbourhood overview (every lot as a dressed shell, street, gardens, park; golden hour on the title screen) behind every menu screen; the neighbourhood and home steps use it as a clickable map (lot glow, tags pinned to lots, camera glides), and Move in flies the camera into the lot. New game: **neighbourhood creator → household creator (gender, attraction, skin, hair, clothes, traits, perks, bonds) → home picker**. The household creator shows the in-game 3D resident (`render/preview`: a small second engine, only alive while the stage is shown or portraits are drawn) with drag-to-turn, zoom to the face and little reactions; avatars everywhere are 3D head-and-shoulders portraits, drawn in batches and cached by look. HUD: resident panel (Now / People / Feelings), social menu with chances, story feed, speech bubbles. |
| Not yet | See the roadmap in section 8 (our own voice engine, smarter thoughts). Dropped: the renderer bake-off (M0); Babylon has carried every build since 0.2. Characters are skinned on the GPU (`render/babylon/characters.ts`). Lamps give light (0.10): with clustered lighting (Babylon's `ClusteredLightContainer`, quality `clusteredLamps`, Medium and High) every lamp lights, up to 32; otherwise the four in the largest rooms. Measured on an M3 Pro at 2880×1800, uncapped, 20 lamps lit: WebGPU 160 → 146 fps (+0.6 ms), WebGL2 90 → 72 fps (+2.8 ms); with four lamps no difference. Clustered lights need Babylon's default falloff, and Babylon 9.29's clustered PBR shader names a missing `normalW` for sheen (our fabrics): patched at start-up in `render/babylon/shaderFixes.ts`. (Cutaway walls and floor coverings are done.) **Simulation scale (0.19):** objects are indexed by lot (`sim-core/src/index.rs`), so decisions only see what a resident may use (home, public places, the lot they're visiting); lots nobody watches run at lower detail (`sim-core/src/lod.rs`): residents away from the viewed lot, the player's home and the player's household take a turn every game minute and catch up (`World::lod` switches it off). Big town (34 houses, 1,835 objects, 64 residents, `tests/big_town.rs`): about 67k → 160k ticks/s at full detail (results identical) and about 345k with lower detail, whose outcomes match full detail within noise. A town holds at most 64 residents (`MAX_SIMS`); raising it means a bigger snapshot and two hard-coded limits in the renderer. |

### Startup performance
- The renderer (engine, compiled shaders and pipelines, cached object meshes, landscape) is owned by a long-lived **game host** and reused across sessions; sessions only swap world contents.
- Sessions are **prepared in the background**, paused and without input: the latest save once the main menu has settled (since 0.2.0 the menu no longer waits for it), and the chosen house on the home screen. Their world builds hidden behind the menus' town overview (camera layer masks, `render/babylon/layers.ts`) and is drawn invisibly for a few frames so its shaders and pipelines are ready. Starting is then a reveal: a fade for a saved game, a camera flight into the lot for a new one. The overview and a new game of the same town share one landscape.
- **Boot set (0.19):** only content, manifests, the sky and the overview's art load before the menu (222 requests, 12.8 MB, about 8 MB gzip; was 304 / 23.7 / 15.0). Residents' data loads after the menu appears; the viewed lot loads first and other lots' furniture streams in the background (`assets/stream.ts`); the game screen, the glTF loader and particles are separate chunks loaded on demand. Menu at 50 Mbit/s: 5.2–6.5 s → 3.3–3.8 s; Continue and Move in unchanged. Move in has one 120–190 ms long task since 0.16–0.18, still to find.
- Measured (production build, headless Chrome, M-series Mac): Continue 286 ms with 110 + 165 ms main-thread stalls → **48 ms, no long tasks**; Move in 397 ms → **98 ms, no long tasks**. The remaining cost of a cold start is WebGPU shader/pipeline compilation, which now happens while the player is still in a menu.
- **0.2.0:** the content, house templates and manifests arrive in one `bundle.json` instead of 56 requests in three waves; the sky's lighting downloads with the page; the isolation service worker keeps the build's hashed files for returning players; the menu shows before the latest save is prepared. Production build on an M3 Pro, Chrome 153, Pages-like server, median of 3: a cold menu at 10 Mbit/s 3.44 → 3.14 s (219 → 164 requests); a returning player after 10 minutes 1.54 → 1.34 s; the menu with a save, warm cache, 2.83 → 1.42 s (the release review's §3.5.3).

### Languages
The game is **English only for now; other languages will follow, for text and for the resident voices.** Until then, new work keeps translation possible:
- **The simulation speaks in ids and numbers, never sentences.** Story events carry kinds, residents, numbers, skill/career/goal indices; planner outcomes carry reason codes; goal text is formatted in the UI from the content's template (`goalText`). Remaining exceptions to clean up: command errors are English strings shown as toasts (they should become codes with parameters), and the journal's event templates and content labels are English text inside the content files.
- **Text comes from content by id.** Labels (needs, skills, activities, goals, objects, feelings, story templates in `events`) stay in content; translations will be locale files that override them by id (e.g. `content/locales/<lang>.json`), loaded like content packs. Templates use named placeholders (`{a}`, `{skill}`, `{n}`), not word order.
- **Interface strings** are still written in the Svelte components; they move to a string table when the first second language starts. Numbers, money and times go through formatting helpers (`ui/format.ts`) so they can switch to `Intl` per locale.
- **Voices** are recorded or synthesised per language (see `docs/design/voices.md`); spoken lines are keyed, not looked up by English text.

### Balance targets (Living mode)
`crates/sim-core/tests/balance.rs` states what Living mode should feel like: a working household saves about a room's worth or less per week (median $100 to two rooms, about $2,000), nobody holds more than $30k after 60 days, residents climb at most 4 grades (2 on average) in 60 days (`cargo test --release --test balance -- --ignored --nocapture`, 180 game days).
- Before the rebalance: median weekly savings $8,025, richest household $190k, 6.3 grades on average, up to 9.
- After: median $1,099, richest $28.8k, 1.7–1.8 grades on average, at most 3 (also on other town seeds). Pay grades $12–52/h (were $15–300/h); promotions about a third as fast (`performancePerShift` 2.5, `performancePerFit` 1, new `performancePerMood` 4, was a fixed 15); rent base $60 + $0.25 per tile; new living costs of $150 per resident in the weekly bills.
- Slower careers left the player's household with little news in a quiet week; goal ideas nobody answers are now taken on after two days (`planner.suggestionDays`), so an idle household still has something to work toward.

### Findings from the town soak test (0.7)
`crates/sim-core/tests/soak.rs` runs a 9-house town with free will on and no orders (14 days in every test run, 60 days with `--release -- --ignored`), checking world invariants every game hour. About 600k ticks/s for 26 residents natively at 0.7; at 0.19 (with storeys, life cycle and visitors) about 500k at full detail and about 970k with lower detail on (`LOD=1`), so even the fastest speed (1200 ticks/s) is far from the CPU budget. With `LOD=1`, soak needs run about 0.01 lower on one seed (not seen across eight towns); worth watching.
- **Careers climb too fast and pay too much:** in 60 days residents reach the top grades and households hold $100–200k. Living mode (0.9) needs a pay and promotion rebalance, or building is never limited by money.
- **Crowded houses wear people down:** households of four with one bathroom have the lowest needs (residents wait for the toilet with bladder near zero). That's the house mattering; 0.11 should make it visible (thoughts, wishes) rather than hide it. (Since residents go at about half rather than bursting, a busy toilet now mostly means a short wait; see *Needs before they run out*.)
- **The story is thin on romance and conflict** over weeks (few first kisses, almost no fights); everyone ends up friends. Planner goals ("find love") and life events should give it more arcs.
- Planner (0.8): residents were woken by any urgent need, so a lonely resident spent nights walking between bed and nowhere. Only `dayRhythm.wakeFor` needs (bladder, hunger) end a night's sleep now. Practice that fits a planned block runs its full time; otherwise activities still end once the needs they fill are full (running practice to the end for everyone wore needs down across the town).
- House matters (0.11): accidents first fired for every need left at 0 (half an accident per resident per day in the soak). Residents routinely let needs sit at 0 for hours: a long activity (a gourmet meal) isn't interrupted by a non-draining urgent need, and energy runs out before bed after work. Accidents now only happen when the home has nothing for the need (no toilet, bed or fridge, or it's broken); content can opt in to a "crowded" grace (`crowdedGraceMinutes`). Residents managing their needs better came later (*Needs before they run out*, below). The test town also gave one-bedroom houses up to four residents; it now sizes households like the game (bedrooms, maybe one more).
- House matters (0.11), wishes: the first "another room" wish formed for any essential that was busy (a partner asleep in the bed, the fridge in use), so nearly every household wished for a second bedroom and kitchen. It also took the room from the first object that fills the need, and Oakridge House's bathroom wall stopped halfway (bathroom, living room and kitchen were one "mixed" room), so a busy fridge there counted as a busy bathroom: most "waits" were hunger. Now the accident names the room it asks for (`anotherRoom`, only the toilet's), it takes two different days within a week, Oakridge's bathroom is closed, and a content test keeps every template's toilet in a bathroom of its own. Over 30 days with one bathroom: 0 of 38 single residents wish for a second one, 8 of 43 couples, 16 of 24 households of three, 2 of 3 of four.
- Life cycle (0.12): with aging on, a starving and exhausted resident started a sleep that hunger ended the same minute, every minute (the AI now deals with waking needs first), and each such start wore the bed as a full use (beds lasted about 30 days; wear now counts the share of a full use, with base wear doubled). Moving first waited for residents to be at home at 11:00, so working grown children never moved. In families of three a broken toilet was left too long (repairs now weigh how much the resident needs what it gives). Retirees on a 40% pension can lose about $100 a week; left as a consequence of retiring.
- Life cycle II (0.13): *Try for a baby* at a quarter of a kiss's weight was almost never chosen (no births in 60 days); at 1.5 couples have about one child per 10–20 days until the house is full. Romance between singles never started: the flirting preference was romance/50 + chemistry, 0 for people with no romance yet, so all flirting went to existing partners, and *Ask to be partners* (weight 0.6) never made the top three. A spark for single, attracted grown-ups who've met (`socialRules.singleSpark` 1.0) and weight 2.5 for asking: couples form (9 in 300 days, was 0). Births are now limited by couples forming late (the age limit is 50; a higher pregnancy chance didn't change the count); singles meeting new people across households is the next lever. Neighbours adopt (`adoption.neighbours`). The AI's own social choices allowed romance between relatives and minors (only the player's orders checked); fixed.
- Needs before they run out: `crates/sim-core/tests/routines.rs` (a week of a town; now also how low needs go, for how long and what residents were doing then, and hours a day walking, eating, asleep...) showed residents using the toilet twice a day, at the last moment, and 62–66% of resident-days with some need under 0.05 (test town and a generated "medium" town, seeds 7/11/23). Causes: a sofa that fills two half-empty needs outscored an empty one; long activities (a two-hour film, cooking with hunger at 0.07) were started and never left for anything but a draining or planned one; a draining activity stopped at once whenever *any* need was low, and was picked again (a park flower bed, over and over); a plate or bed nobody could get to was chosen every minute; people were pulled into chats while bursting; workers left hungry and came back drained (a shift cost half of fun and hunger); and fun and social drained faster than a work-day evening could fill them, so needs fell day by day from Monday to Friday. Now needs have a `comfortable` level (seen to when convenient below it), the essentials can become `urgent` (damping everything else, stopping what residents chose themselves if it would run out, keeping them out of chats), choices look ahead (activity, the walk there and on to the toilet, and the time until leaving for work), unreachable choices wait `retryMinutes`, `dayRhythm.habits` send them to the toilet on getting up, before bed, before leaving (and a snack), on getting home and after a meal, an alarm gets workers up in time, and fun, comfort and social decay 0.06/h with gentler work decay. After: toilet about 4 a day (chosen at bladder about 0.5, plus habits), resident-days with a need under 0.05 15–25% (test town) and 24–30% (medium), one-block nights 88–96% (were 78–86%), naps 0.03–0.12 a day. What's left: loners' social (no one at home to talk to; the phone will help), long early shifts, two-storey homes where the toilet is a long walk away, and a generated home whose bed can't be reached (`fellAsleep` now happens after 120 minutes when no bed can be had: about 25 in the 60-day soak, residents in households with one more resident than bed places).
- Stable resident ids were moved from 0.7 to 0.12: nobody is removed before then, so the saved story log can keep naming residents by index and is converted when 0.12 loads it. Thought bubbles moved to 0.8 (thoughts come with the planner).

### Known issues and findings from the first build
- **Babylon SSAO2 on WebGPU failed to bind its sampler on about 1 in 4 startups** (`randomSampler not found`, then `createBindGroup` errors and a frozen frame), so ambient occlusion was off by default. It was removed in 0.2.0 with the Ultra preset.
- **Standard-mode snapshot rendering bakes light uniforms and the clear colour into the recording.** The renderer re-records whenever lighting changes, throttled to every 5 game minutes. Moving residents (thin-instance buffers) and helper meshes update without re-recording. Only the menus' town uses it; the game draws live (retrying it in play is P1-07 in the release review).
- **Bundle size (0.19):** Babylon is imported by module through `render/babylon/core.ts` (never from the `@babylonjs/core` root): main chunk 1.76 MB (478 KB gzip), was 6.6 MB.
- **Diagonal walls (limits):** movement stays on whole tiles, so the tile a diagonal wall crosses is closed to walking and furniture (its two half-triangles are dead space; a diagonal door tile is walked through). Residents do walk the corridor between two parallel diagonals. Roofs: a convex indoor outline with diagonal sides gets an exact hip roof over that outline; other shapes (e.g. an L with a diagonal corner) get the usual rectangles over their tiles, with split tiles covered whole (a deeper eave over the outdoor half). Neighbours' houses with diagonals are drawn as their bounding box. Landscape trees aren't removed when a room is built over them (straight walls too).

---

## 1. Research summary: which renderer?

The renderer is the decision that matters most for performance, so I compared the options against **what a life-sim actually draws**:

- **Hundreds to thousands of unique furniture/wall pieces.** Many separate meshes, each with its own material.
- **Many small lights.** Lamps, TVs and fireplaces mean dozens of point lights per lot.
- **Sun shadows plus post-processing**, for the "stunning" look.
- **5–30 skinned characters**, each with blended animations.
- Mostly static scenes. The house changes only in build mode.

### Benchmark data (three.js r186, PlayCanvas 2.22, Babylon.js 9.26; Chrome/Firefox/Safari, M1 Pro)

| Workload (relevant to us) | PlayCanvas | three.js | Babylon.js |
|---|---|---|---|
| 5k separate animated meshes (CPU) | **12 ms** | 29 ms | 24 ms |
| 100 materials (CPU) | **13 ms** | 30 ms | 41 ms |
| 32 forward point lights (GPU) | **1.3 ms** | 53 ms | n/a |
| Sun + spot shadows + bloom/ACES (CPU) | **21 ms** | 52 ms | 78 ms |
| 50 complex skinned characters | **6.5 ms** | 18–22 ms | 18–22 ms |
| 50k instanced meshes | tie (1–2 ms) | tie | tie |
| Bundle / first frame | ~0.6–1 MB | **~170–300 KB, ~100 ms** | ~1.8 MB |

Sources: [webgpu-webgl-benchmarks](https://github.com/mvaligursky/webgpu-webgl-benchmarks), [web-engines-compare](https://github.com/mvaligursky/web-engines-compare) (per-draw CPU cost: PlayCanvas 0.75 µs vs three.js 0.89 µs vs Babylon 2.35 µs on macOS WebGL2).

**Caveat:** both benchmark repos are maintained by Martin Valigursky, the second-largest contributor to the PlayCanvas engine. The methodology is open and reproducible, but we **re-measure on our own scene** before committing (see Milestone 0).

### Other findings

- **three.js WebGPURenderer has a known slowdown on scenes with many separate meshes.** It does a lot of per-object bind-group and buffer work, about 2× the CPU of its own WebGLRenderer, and no fix has landed ([forum thread](https://discourse.threejs.org/t/webgpurenderer-2x-slower-cpu-and-5-10x-slower-first-frame-than-webglrenderer-on-many-mesh-scenes-r183-same-on-both-backends/91904)). A house full of furniture is exactly that kind of scene.
- **WebGPU is not automatically faster.** Without instancing, the WebGL2 backends beat their own WebGPU backends in the benchmarks. Instancing and batching cut CPU cost 10–100× and **matter more than which engine we pick**.
- **Babylon.js** has *snapshot rendering* (it records a frame's GPU commands and replays them), which is great for static scenes, and clustered lighting since 9.0. But it was the slowest in Safari and has the heaviest bundle.
- **Custom Rust renderer (wgpu)** gives one language and maximum control. However, every WebGPU call from WASM still goes through JS glue code, so there is no free speedup. It also means building shadows, post-processing, skinning, a material system and a WebGL2 fallback ourselves. That is many months of work before the game looks good. Rejected for now.
- **Bevy (full Rust engine on the web)** means large WASM bundles, slow iteration, and in-engine UI that is weaker than Svelte/CSS. Rejected.
- **Browser support:** WebGPU ships in Chrome/Edge, Safari 26 (macOS and iOS), and Firefox on Windows and Apple-Silicon Macs. Firefox on Linux and Android is still missing, so **a WebGL2 fallback is required**.
- **Rust WASM threads still need nightly Rust** (`+atomics`, `-Zbuild-std`) plus cross-origin isolation. We avoid this. The simulation is cheap enough for a single thread, and we get parallelism from **Web Workers** instead.

### Decision

**PlayCanvas engine (npm `playcanvas`), used code-first with no cloud editor.** *(Superseded, see the update below: the M1 skeleton is built on Babylon.js because of its WebGPU snapshot rendering.)* It leads on every workload a life-sim stresses: many meshes, many materials, many lights, shadows plus post-processing, and skinned characters. It falls back to WebGL2 automatically.

**Update:** because everything visual should run on the GPU (section 2), **Babylon.js with snapshot rendering is now a third contender.** Its replay of recorded GPU commands fits our mostly static house, while PlayCanvas is still mainly a WebGL2 engine. Milestone 0 decides between the three.

The renderer stays behind a thin `Renderer` interface fed by plain data from the simulation. That keeps the engine choice reversible: if the Milestone 0 bake-off disagrees, swapping to three.js costs days, not months.

---

## 2. Architecture

```
┌──────────────────────── Main thread ────────────────────────┐
│  Svelte 5 UI (HUD, needs, build/buy catalog)  ◄─ 10 Hz store │
│  Input → commands                                            │
│  Babylon.js renderer (60–120 fps, interpolates sim state)    │
└───────▲─────────────────────────────┬────────────────────────┘
        │ SharedArrayBuffer            │ SharedArrayBuffer
        │ (triple-buffered snapshots)  │ (command ring buffer)
┌───────┴─────────────────────────────▼────────────────────────┐
│  Sim Worker: Rust → WASM                                     │
│  fixed 20 Hz tick · needs · utility AI · pathfinding ·       │
│  lot/room graph · build-mode geometry generation · saving    │
└──────────────────────────────────────────────────────────────┘
        │ asset loads / saves
   Asset Worker (glTF + KTX2 decode)          OPFS / IndexedDB
```

**Core rule: the CPU runs the game, the GPU draws it.**

| CPU (Rust/WASM, sim worker) | GPU (WebGPU, WebGL2 fallback) |
|---|---|
| Game clock, needs, mood, AI decisions | Visibility culling (compute shader) |
| Pathfinding, interactions, relationships | Character skinning and animation blending |
| Lot/room logic, build-mode rules, picking | Shadows, lighting (clustered), ambient light |
| Saving and loading | Ambient occlusion, bloom, tonemapping, tilt-shift, colour grading |
| Writes **one compact snapshot per tick**; nothing else | Particles, cutaway walls, selection outlines, placement ghosts |
| | Time-of-day sky, weather, foliage sway (shaders) |

The main thread's only rendering job is to upload the snapshot and **replay pre-recorded draw commands**. It does no per-object work.

**Key principles**

1. **Sim and render are fully separate.** The sim runs at a fixed 20 Hz in a Worker, and the renderer interpolates between the last two snapshots. A slow frame never slows the sim, and a sim spike never drops a frame.
2. **No per-object JS↔WASM calls.** The sim writes a compact binary snapshot (positions, rotations, animation IDs, state flags) into a SharedArrayBuffer. The renderer reads it with typed arrays, with no copies, no serialization and no garbage-collection churn. Commands flow back through a lock-free ring buffer.
3. **The UI is not in the hot path.** Svelte receives changes at about 10 Hz (`view::UiSync` in Rust, `game/uiUpdates.ts`): keyed by id, drifting values rounded to what the HUD shows, details only for the player's household and the inspected resident; a full update at start and on request (`resync`). Needs bars animate with CSS transitions, so the UI never forces per-frame DOM work.
4. **No garbage in the frame loop.** All per-frame JS buffers are pre-allocated. The Rust side allocates nothing in steady state.
5. **Cross-origin isolation** (`COOP: same-origin`, `COEP: require-corp`) is needed for SharedArrayBuffer. Fine for self-hosting, and Vite dev and Cloudflare/Netlify all support it. Fallback when unavailable: `postMessage` with transferable buffers.

**Later option:** if main-thread jank appears, move rendering to its own worker via `OffscreenCanvas`. The interface makes this a contained change.

**As built (0.2.0)**, where it differs from the plan above:
- **Snapshots** are one buffer guarded by a seqlock (`core/snapshot.ts`), not triple-buffered, plus a step counter so the renderer clocks by sim steps. Without cross-origin isolation they're transferred as a copy.
- **Commands** are JSON over `postMessage`, applied between ticks; no ring buffer was built.
- **Workers:** the game's own are the simulation, the voices, the language model and the water sounds; no asset worker was built. Saves are kept in IndexedDB, not OPFS.
- **Drawing:** the game draws live. Snapshot rendering only replays the menus' town (above).
- **Hosting:** GitHub Pages can't send the isolation headers, so a service worker (`public/isolation-sw.js`) adds them.
- **The UI's transitions:** the release review found that CSS transitions of the bars' width forced style, layout and paint on every display refresh (P2-01). Since 0.2.0 the bars move by transform, which the compositor animates: layouts went from about 108 to 11 a second at speed 5. Chrome still recalculates style each frame for running CSS animations, about 0.13 ms a frame on an M3 Pro (the release review's §4).

---

## 3. Simulation core (Rust)

Pure Rust crate `sim-core` with **no browser dependencies**. It is tested and benchmarked natively with `cargo test` / `cargo bench`; a thin `sim-wasm` crate holds the bindings.

| System | Design |
|---|---|
| **Data model** | Data-oriented: structure-of-arrays storage with generational IDs. No ECS framework, to keep the WASM small and the code fast. |
| **Time** | Fixed 20 Hz tick; game clock with pause / 1× / 2× / 3× (3× = more ticks per real second, capped). Deterministic, seeded RNG. |
| **Needs and mood** | Hunger, energy, bladder, hygiene, fun, social, comfort, environment. Each decays along a curve; mood = weighted sum + feelings (temporary modifiers). |
| **AI ("smart objects")** | Objects *advertise* interactions with need deltas. Each resident scores them with need-weighted utility curves, distance cost and personality modifiers, then picks from the top-N with weighted randomness. Player commands go to a queue and override autonomy. This is the proven smart-object model of classic life sims. |
| **Interactions** | Small state machines: route to slot → animate → apply effects → exit. Multi-resident interactions (talking, sharing a sofa) reserve object slots. |
| **Pathfinding** | Tile grid (1 m tiles, 4 sub-positions); walls block tile edges; doors are portals. A* with Jump Point Search, a cached room-level graph for long routes, string-pulling for smooth paths. Slot reservation plus simple local avoidance between residents. |
| **Lot / build** | Multi-level tile grid; walls on edges, plus diagonal walls corner to corner across a tile (one per tile; rooms are found per half tile, so the two halves can be inside and outside); rooms found by flood fill (used by the AI, room lighting and cutaway). **Rust generates wall/floor meshes** into WASM memory; the renderer uploads them directly. |
| **Picking** | Planned: a ray against the tile grid and object bounding boxes, done in Rust. As built: the renderer picks (`BabylonRenderer.pick`, `pickPlane`, `pickOnWall`, in TypeScript), by what's drawn of an object; no GPU readback. |
| **Saving** | JSON with a versioned schema (`sim-core/src/save.rs`, version 23 since 0.2.0; older saves are migrated on load, and `tests/old_saves.rs` keeps 0.1.0's loading), stored in IndexedDB (`web/src/persistence/saves.ts`, with a `meta` store for the lists). The worker writes the save, so the frame never stalls. |

Performance budget: a sim tick under **1 ms for 8 residents** and under **3 ms for 50 residents** (neighbourhood scale later). WASM SIMD (`+simd128`) on: the wasm crates refuse to build for wasm32 without it, since a `RUSTFLAGS` in the environment would replace `.cargo/config.toml`'s. `lto = "fat"`, `opt-level = 3`, `panic = "abort"`. Since 0.2.0 production builds run binaryen's `wasm-opt -O3` and strip the names (sim wasm 2.85 → 2.25 MB, 835 → 796 KB gzipped; no change in speed), never with `-tnh`, as every panic is a trap. The release review measured a speed-5 step (60 ticks) of the 64-resident town at 0.86 ms on average in Chrome on an M3 Pro, worst 9–17 ms.

---

## 4. Rendering and look

**Art direction: "warm miniature diorama".** Stylised low-to-mid-poly models, soft PBR materials, pastel palette, a toy-like scale feel. It ages well, scales across GPUs, and is achievable without a AAA art team.

**Quality features**

- Physically based materials with image-based lighting; a sky gradient and ambient light that **change with time of day** (golden hour, blue hour, warm lamp-lit nights).
- Sun: 2-cascade soft shadows; shadow updates throttled while nothing moves.
- Interior lamps: clustered lighting so dozens of lights stay cheap; shadow casting only on a budget of about 4 nearby lights.
- Ambient occlusion (SSAO), bloom, ACES/neutral tonemapping, subtle **tilt-shift depth of field** (the dollhouse look), vignette, a colour-grade lookup table per time of day.
- Selection outline, soft placement ghosts in build mode, glowing interaction targets.
- **Cutaway walls** (walls up / cutaway / down), done in the shader so it costs no CPU.

**GPU-first rendering (WebGPU path)**

- **Render bundles:** the draw commands for the house are recorded once and replayed every frame, so the CPU cost is close to zero. They are re-recorded only when build mode changes the house. (Babylon calls this "snapshot rendering".)
- **GPU culling:** a compute shader tests object bounds against the camera and writes `drawIndirect` arguments. The CPU never loops over objects.
- **GPU animation:** skinning in the vertex shader. Background/neighbourhood residents use baked vertex-animation textures, so hundreds cost almost nothing.
- **GPU particles:** compute-driven steam, fire, sparkles and mood effects.
- **All per-object data lives in GPU storage buffers** (transforms, material IDs, highlight state). Each tick, the snapshot from the sim is uploaded in **one** buffer write.
- **Limit:** multi-draw-indirect (one call for *everything*) is still experimental in Chrome (~0.25% of devices), so each draw is still one cheap call. Merged meshes, instancing and shared materials keep the count low (about 100–300).
- **WebGL2 fallback** (no compute shaders): CPU culling plus instancing, otherwise the same scene. Slower, but within budget.

**How we stay fast**

- Static walls and floors are merged into **one mesh per room chunk**, rebuilt only on build-mode edits.
- Furniture is **instanced by model** (every identical chair is one draw call).
- Texture atlases plus a small set of shared materials, which keeps material switches low.
- Characters: GPU skinning, 2–3 LODs, animation sampling skipped for off-screen residents.
- Frustum culling plus per-level culling (upper floors hidden in cutaway).
- Adaptive resolution: render scale drops before frame rate does.

**As built (0.2.0)**, where it differs from the plan above:
- **Shadows:** one sun shadow map over a fixed area round the lot, 1024, 2048 or 4096 by preset. It draws only the casters that can reach that area (about 150 shadow draws by day in the village, from 242), and only when something in it changed: the light, the casters, a thing placed or moved, roofs and storeys, or a resident's pose. Paused, in Build mode and behind menus it isn't drawn at all; in live play, with residents in view, it still is every frame, as they breathe (residents' shadows in a path of their own are a v0.3 candidate).
- **Lamps:** clustered, up to 32, on Medium and High; the four nearest the biggest rooms on Low.
- **Effects:** no ambient occlusion (removed in 0.2.0), bloom or tilt-shift. There is ACES tone mapping, a vignette and a night grade ([docs/design/look.md](docs/design/look.md)).
- **No GPU culling:** there are no compute shaders or `drawIndirect`. Culling is Babylon's frustum culling on the CPU, and build effects use CPU particles.
- **Snapshot rendering** replays only the menus' town; the game draws live.
- **Residents** are skinned on the GPU from a pose texture, with a low-detail copy for far residents and shadows. Only the viewed lot's residents are drawn, so no vertex-animation textures were needed.
- **The draw count is higher than planned:** P1 measured 493 draw calls a frame on Medium in the village scene, 242 of them for the shadow map.
- **Resolution:** Auto draws native pixels up to about 1920×1080 and doesn't step down when frames run slow (the owner put that off for 0.2.0). A Custom resolution draws 50–100 %.
- **Presets:** the three graphics presets and their costs are in [docs/design/graphics.md](docs/design/graphics.md).

**No hardware target** (the owner, 2026-10-15): there is no frame-rate goal for particular machines. Low is for older laptops, best effort.

*(The plan's first targets were 60 fps at 1080p on integrated GPUs such as Apple M1 and Intel Iris Xe, with about 300 draw calls or fewer a frame and main-thread CPU at 6 ms or less, and 120 fps on discrete GPUs.)*

---

## 5. UI (Svelte 5 + CSS)

- Svelte 5 runes; plain CSS with design tokens (colour, radius, blur, motion) and **no UI framework**.
- Visual language: frosted glass panels (`backdrop-filter`), soft shadows, rounded geometry, one accent colour per mode (Live / Build / Buy), spring-based micro-animations, variable fonts.
- Panels: resident portrait plus mood ring, needs (circular gauges), interaction pie menu (radial, at the cursor), action queue, clock and speed control, build/buy catalog with 3D thumbnails rendered once and cached.
- Accessibility: keyboard shortcuts, reduced-motion support, scalable UI.
- Performance rules: no per-frame reactive updates; the catalog uses virtual scrolling; thumbnails are lazy-loaded.

---

## 6. Assets (replaceable by design)

- **Everything is referenced by key.** Content and code ask for `model.fridge`, `icon.need.hunger`, `material.wall` or `look.coral`, never for a file path.
- **`web/public/assets/manifest.json`** maps keys to files (`url`). Entry types: `model`, `material`, `icon`, `image`, `sprite`, `look`.
- **Override packs:** manifests listed in `packs` (or passed as `?pack=…`) override keys, so a new art set is a folder plus one line, with no code changes.
- **Graceful fallback:** a missing model draws nothing and a missing icon a neutral glyph, so broken art never breaks the game.
- Format for real art: **glTF 2.0**. The loader bakes transforms so thin instancing keeps working.
- **Compression is automatic (0.19):** production builds run `tools/assets/optimize.mjs` (KTX2 ETC1S textures, meshopt geometry, gzipped character data; cached in `web/.assets`, index `assets/optimized.json`), and loaders fall back to the source file if a copy fails. Sources stay editable; the dev server serves them, and `LL_ASSETS=source` builds without copies. Decoders are self-hosted in `public/decoders/` (`pnpm decoders` after a Babylon upgrade), because cross-origin isolation blocks Babylon's CDN. Packs outside `public/assets` load as they are.
- Starting library: CC0 packs (Quaternius, Kenney, Poly Haven); a **modular character system** (body + head + hair + outfits on one skeleton) for the household creator.

---

## 7. Repository layout (as built)

```
idyll-lives/
├─ crates/
│  ├─ sim-core/        # pure Rust simulation, no browser deps, native tests
│  ├─ sim-wasm/        # thin wasm-bindgen layer
│  ├─ voice/           # resident voices: phonemizer (mouth shapes), voice shaping (PSOLA), resampling
│  └─ voice-wasm/      # thin wasm-bindgen layer for the voice worker
├─ web/
│  ├─ public/content/  # gameplay data: needs, objects, careers, houses, voice lines (JSON)
│  ├─ public/assets/   # asset manifest, models, textures, icons, packs, CREDITS.md
│  └─ src/
│     ├─ core/         # sim worker, bridge, protocol, shared-memory snapshot
│     ├─ render/       # Renderer interface + babylon/ implementation, 3D previews
│     ├─ assets/       # manifest schema, registry, streaming
│     ├─ content/      # gameplay content loader (for UI labels)
│     ├─ game/         # composition root, input, build mode, generated houses
│     ├─ audio/        # mixer and the sound of the world
│     ├─ voice/        # voice worker, director, voices per resident
│     ├─ persistence/  # saves and blueprints (IndexedDB)
│     ├─ settings/     # player settings
│     ├─ debug/        # debug reports
│     └─ ui/           # Svelte components, design tokens, UI state
├─ tools/              # asset, voice, house-generation and release scripts
├─ docs/               # content packs, design notes, personalities
└─ PLAN.md, README.md, CHANGELOG.md
```

The snapshot memory layout is **defined once in Rust** (`sim-core/src/snapshot.rs`) and read by TypeScript at startup, so the two sides can't drift apart.

---

## 8. Milestones

**0.1.0 (2026-10-10) is the first release.** It is everything the development builds below brought: M1 skeleton, living residents, look pass (lighting, shadows, tilt-shift, cutaway walls), Build and Buy (walls, diagonals, rooms, paint, floors, doors, windows, undo), household and town, persistence, household creator, careers and skills (builds 0.2–0.6), then "build your home, then watch it live" (builds 0.7–0.20).

**Built before 0.1.0** (one development build each; each that changed the save format bumped the save version and migrates older saves):

| Build | Theme | What it brought |
|---|---|---|
| 0.7 | **Watch mode** + autonomy foundations | Stable resident ids; free will per household; the player's household visits on its own; residents find, quit and lose jobs on their own; saved story log with home and town scope; faster speeds and auto-fast at night; wall-straddle fix; town soak tests. Director camera, household strip, Inspect panel, Journal, thought bubbles, camera dock, "Watching" settings. |
| 0.8 | **Planner** | Per-resident weekly routines (strong nudge) and household templates; life goals with progress; goals suggested by residents; thoughts and wishes; calendar UI with drag/resize, today's timeline, outcomes with reasons, catalog links. |
| 0.9 | **Builder's start** | Creative/Living per game; empty lot; household optional ("move a family in" later); build and buy addressed to the household. |
| 0.10 | Build depth I | Incremental world updates; redo; eyedropper; roof style and colour; lamps that give light; fences and gates; free rotation for decor. *(Incremental updates: the renderer keeps the lot layer (streets, house, fences) unless what it is built from changes (`lotKeyOf`), so furniture edits cost about 2.5 ms instead of 13 ms; the worker sends the lot (walls, rooms, floors, meshes) only when `World::lot_version` changes, otherwise a lean structure (17 KB instead of 56 KB + 107 KB of meshes; worker 0.6 → 0.15 ms, main-thread handling 2 → 0.6 ms, M3 Pro). Also Watch/Live as modes, Watch by default.)* |
| 0.11 | **The house matters** | Room scores (size, light, decor, cleanliness, function); environment need; dirt, wear and repairs; essentials with fallbacks and accidents; room opinions and wishes; room-score overlay and "Our home" panel. *(design in `docs/design/house-matters.md`. 60-day soak: rooms stay clean (0.76–1.0), 5 breakdowns all repaired, 2 accidents; wishes for a second bathroom grow with the household (see "Findings from the town soak test"); balance unchanged.)* |
| 0.12 | Life cycle I | Aging, elders, death or retirement, partners moving in, grown children moving out, newcomers, family relations. *(design in `docs/design/life-cycle.md`. Residents keep their slot when they die or leave (ids, snapshot rows and the story stay valid); newcomers reuse it and the story then names the former resident. 150-day soak (75 years at the normal pace): the town holds steady at 12–16 residents in 19 slots, with 11–19 deaths, 7–8 newcomer households and the player's empty home kept.)* |
| 0.13 | Life cycle II | Pregnancy and adoption, babies, children, teens, school; child and elder bodies. *(design in `docs/design/life-cycle-2.md`. Bodies are the adult rigs scaled per stage with a head factor in skinning; real child models can replace them through the manifest. 300-day run (150 years): after the starting families pass, the town holds steady at 14–18 residents in 9 houses, with 9 couples formed, 9 births, 15 adoptions, 38 deaths.)* |
| 0.14 | Voices II: **Babble** | Babble, the made-up language: no download, a Settings choice next to English, and what residents say while English loads or after it fails; voices by life stage (children higher, elders lower); the voice worker unloads after 5 minutes of quiet; a live watchdog steps English down to Babble for the session when lines keep coming late or frames suffer; Babble in the F3 voice line. Presentation only, no save change. Plan: `docs/design/voices.md`, phase 3. *(Babble renders about 1 ms per second of speech on the main thread; checked in the game in headless Chrome: Babble lines play with no engine loaded, English still loads and speaks.)* **Babble removed in 0.20:** residents speak English or nothing; quiet while the voice loads, babies silent, and the watchdog pauses voices instead. |
| 0.15 | **Visitors and sound** | Visitors walk to the front door and knock; someone at home answers, greets them and asks them in or sends them away; with nobody home they leave a note in the journal and go. Sound of the world: one audio context for effects, voices and the world; day and night ambience (birds, street, crickets) by time of day; object sounds (stove, shower, TV, radio) placed like voices, only on the viewed lot. *(Knocking runs in `life::doors` (`visits.door` in content; without it visitors walk straight in, as before); answering isn't saved, a visitor knocking at save time knocks again. 60-day soak: 61 visitors let in, 16 found nobody to answer (hosts asleep or out), none turned away (residents only visit friends). Sounds are synthesised in `audio/world.ts`, one loop per kind of sound, through the shared mixer `audio/mixer.ts`.)* |
| 0.16 | Bigger homes I | Lot sizes (small, medium and large lots in the neighbourhood creator); blueprints (save a house's walls, floors, doors, windows and furniture; build one on an empty lot, paying for it in Living); move a room with what stands in it, with undo. *(Lot sizes live in the town draft (`PlotSlot.lot`, `houses.json` `lotSizes`) and are chosen on the home screen; layouts are drawn for a medium lot and centred on others. Blueprints and moving rooms are `sim-core/src/blueprint.rs`: a blueprint is stored by content id with the street side in front, so it's turned round for the other side of the street; building one is priced with a dry run and kept as one undo step. Blueprints are kept in the browser (`persistence/blueprints.svelte.ts`), not in saves, so the save format is unchanged.)* |
| 0.17 | **Storeys** | Multiple storeys and stairs: one tile grid per storey, stairs as a buildable object that joins storeys for pathfinding, rooms and room scores per storey, roofs over the top storey, a storey switcher in Build and Live, cutaway per storey. *(`sim-core/src/storeys.rs`. The lot's rows hold every storey (storey `k` is rows `k × depth ..`), so walls, rooms, room scores, furniture, saving, blueprints and moving rooms work upstairs unchanged; above the ground there is floor only inside rooms, and a room or wall upstairs needs a room under it. Stairs (`stairs` objects, 1×4) are walked step by step and link their top step to the landing above (`path::StairMap`); the snapshot carries each resident's height. The renderer builds each storey's house at its height (no roof under a room above, no floor over a stairwell) and hides storeys above the one in view; build tools and picking work on the storey in view, which follows the resident you're looking at. Content `build.storeys` (3 in the base game); save version 17, older saves grow the storeys. Town files may bring storeys (`storeys`, rows as above).)* |
| 0.18 | **Generated houses** | Neighbour houses generated from a config (`content/housegen.json`) instead of three fixed layouts: every home has a bedroom, a kitchen and a bathroom, bigger ones a living room and more bedrooms; varied footprints, room arrangements, looks and furniture; some with one storey, others with two (stairs, bedrooms upstairs). *(`web/src/game/housegen.ts`, config `web/public/content/housegen.json`. A house class is picked by lot size; its footprint is cut into rooms (a guillotine plan), doors form a tree from the front door through the living room and kitchen (sometimes one open space), windows go along outside walls, furniture against walls with doors, stairs and every piece kept reachable. Two storeys: stairs along an outside wall of the living room, a hall over them upstairs, bedrooms and a bathroom off it. Towns keep only `gen:<lot size>:<seed>` per lot, so menus and game build the same house; neighbours' silhouettes are as tall as their storeys. Checked: `tools/housegen/sample.ts` makes houses for every lot size and `tests/generated_houses.rs` (ignored; `HOUSES=…`) loads each into the simulation: 450 of 450 made, 137 with two storeys, every one with a bed, fridge and toilet in rooms of their own and everything reachable from the street.)* |
| 0.19 | **Performance and scale** | Thread stats in F3; objects indexed by lot and lower detail for unwatched lots; the UI gets changes only; compressed assets, a small boot set and streaming; Babylon imported by module. *(numbers under Status, Startup performance and Known issues. Save format unchanged.)* |
| 0.20 | **Voices and things** | Babble removed; resident voices on ONNX Runtime Web (about twice as fast); KittenTTS nano for teens and grown-ups (eight real voices, a mix per resident), Paradee for children and as the fallback; voice size (depth); a clip cache; voice options in the household creator; voice lines per item and interaction (about 2,350); 130 new things to buy (Kids and Lighting categories, things on walls and ceilings); birds by species, singing in bouts. *(Save format unchanged; a chosen voice is an optional field in the appearance.)* After 0.20, before 0.1.0: Paradee removed, KittenTTS speaks for everyone. |

**Resident voices in 0.1.0** were an experimental setting: the selected resident, and conversations the player starts, were spoken aloud by **KittenTTS nano** (Apache-2.0, 15.5 MB as int8 weights, downloaded on request), run by ONNX Runtime Web in a worker: eight real men's and women's voices, each resident a mix of two (children a mix of the women's, raised and made smaller), phonemized by our own Rust/WASM code (`crates/voice`) mapped to espeak-style IPA, avoiding GPL espeak-ng. English lines per item and interaction, a clip cache, voices by gender and life stage with a size (depth, by resampling in the worker), voices pausing for the session when lines keep coming late or slow the game, voice options in the household creator, and Settings → Audio with a speed test. Presentation only, no sim changes. Plan: [docs/design/voices.md](docs/design/voices.md).

**0.2.0** brought babies cared for in the crib, cooking from fetching the ingredients to washing up, a phone for everyone from teens up, basements, generated worlds of eight kinds, residents made in code, 196 more things to buy, resizable furniture, wide windows and double doors, and Live and Build as the two modes (full list in [CHANGELOG.md](CHANGELOG.md)).
- **Voices:** residents speak with Supertone's **Supertonic 3** on the graphics card, WebGPU only. It is under the BigScience OpenRAIL-M licence, whose use restrictions bind every player; its weights are stored as 16-bit floats and gzipped, 185 MB. It is about 28× faster than real time on an M3 Pro, with ten bundled voices, each resident with their own pitch, depth and speed. It replaced KittenTTS nano; Kyutai's Pocket TTS was tried in between, on 2026-10-12, and never released.
- **Lines:** an optional small language model, Liquid AI's **LFM2.5 350M** (LFM Open License v1.0, its 4-bit ONNX build as published, WebGPU only), writes residents' spoken lines from who they are and what's going on. The utility AI still decides everything.
- **The release review:** a review of animations, objects, bugs, performance and release readiness came before the release ([docs/release-0.2.0-review.md](docs/release-0.2.0-review.md)). It fixed most of its 140 findings, among them the crash when furniture is sold or moved (R2-01) and the silent freeze on a sim error (R4-01). It also brought century-long and busy-player test runs, a crash screen, per-game autosaves with export and import, three graphics presets with Auto resolution, wasm-opt, and CI gates with a dry run.

**Roadmap after 0.2.0** (one release each; a release that changes the save format bumps the save version and migrates older saves). Most rows come from what the release review found and left (its §4). `v0.3/garden-decor` already holds v0.3 work that is not in 0.2.0 (garden decor built in Blender). A dev-only study of cel and soft shading led to the soft look, now on `main` (the Look row below), and found a possible bug with plain finishes, which the release review's §4 lists for checking in play (the Plain finishes row); its branches, `v0.3/look-study` and `v0.3/look-study-soft`, were deleted once it was merged (2026-10-19). The unreachable-goals row is done and on `main` (2026-10-19): reach labels refuse goals in another walkable region, people in the unreachable memory, walkers re-routed when the house changes (including residents nobody watches, on their next turn), `nav_version` split from `structure_version`, and versioned settings. So is the calling-out row (2026-10-19): chats only within earshot, the name called and the two meeting halfway.

| Release | Theme | Done when |
|---|---|---|
| v0.3 | **Objects built in Blender** (the owner's decision, 2026-10-13) | New objects are made in Blender and exported as glTF, as the standard. The pipeline is `tools/art/blender`; the standard is in `docs/design/objects.md`. |
| v0.3 | **Stable object ids** | Every object keeps an id of its own for life, so selling or moving one shifts nothing. That replaces the remapping of follow-ups, queues and unreachable things and the `seen` guard on commands that 0.2.0 uses, and saves keep their ids across a reload. |
| v0.3 | **Unreachable goals** ✅ | A goal in an area the resident can't reach is refused before A* floods the town, instead of capping the search (P3-08). People join the unreachable memory (Social, Join). Walkers re-route when walls change. `structure_version` is split so a change refreshes only what it touched. Settings get a version and ordered migrations. *(Done: merged to `main` 2026-10-19.)* |
| v0.3 | **Calling out** ✅ | Nobody crosses the house or the lot for a chat: a resident only starts one with someone within earshot (the same room, the next one near a doorway, the garden; walls and windows stop a call). They call the other's name (a bubble, and the voice for the inspected resident), the other turns to them, and the two meet halfway. A chat the player orders from further off walks only until the other is in earshot. A target too busy to come is walked up to and waited for, as before. *(Done: merged to `main` 2026-10-19.)* |
| v0.3 | **Look: soft shading** ✅ (the owner's decision, 2026-10-18) | The soft look from the dev-only shading study is the game's shading: every lit PBR surface (furniture and objects, walls and floors, the ground, residents and their clothes) draws its light as a wide, soft ramp in the light's own colour, with a cool fill from the sky in the shade, a soft rim and a little extra saturation; no outlines. It lives in `web/src/render/babylon/softLook.ts` (GLSL and WGSL, applied by `MaterialLibrary`); water, the sky and reflective glass keep their own materials, and unlit materials are untouched. The light's own colour comes through, so golden hour stays golden, lamps stay warm and night stays dark. The previous PBR shading is kept as Settings, Performance, **Shading: Classic**, which applies live (no restart); `?look=soft\|classic` on the dev server overrides it. Done when it is on `main`, `docs/design/look.md` describes it (done), and the release passes the frame-time budget (about 0.4 ms a frame more on an M3 Pro at 1600×900, 60 fps either way). *(Done: `main`; examples `debug-reports/soft-look-objects.png`, `soft-look-characters.png`, `soft-look-game.png`.)* |
| v0.3 | **Plain finishes** (bug) | Placeholder models' plain finishes show as what they are. Their finish is in the vertex colour's alpha (`encodeFinish`), but the meshes have `hasVertexAlpha` false, so the shader gets an alpha of 1 and `PlainFinishPlugin` reads every plain finish (matte, satin, gloss, metal, chrome) as fully rough metal: the cozy fridge's cream `#EFE2BF` shows as khaki. Found in `/gallery.html` by the look study (2026-10-13); the game builds items with the same `buildModel` but wasn't checked in play. The street sets `hasVertexAlpha = false` on its plain meshes on purpose, so check it too. Done when the alpha reaches the shader without making the meshes transparent, the five finishes look different in the gallery and in a game, and the look is checked again (furniture will turn lighter). |
| next | Voices III: Auto mode | Auto picks the right mode without frame drops, and a slow machine steps down with a clear message (phase 6 of `docs/design/voices.md`). ONNX Runtime Web stays. |
| later | Smarter thoughts (research) | The language model also writes thoughts for actions the utility AI already chose, feeding Resident voices. The utility AI keeps deciding; a model may at most re-rank its top-3 shortlist, entering the sim as a recorded command so saves and replays stay deterministic. |

Every release passes CI's gates: clippy with `-D warnings`, the simulation tests, tickbench, the voice tests, `pnpm check:voice`, `pnpm check` and `lint`, and a build under 900 MB. It has a dry run on `main` before its tag (README, "Versions and releasing"). The long runs (`cargo test --release -p sim-core --test century -- --ignored`, and the same with `--test soak`) and the frame-time budget are checked by hand.

---

## 9. Risks

| Risk | Mitigation |
|---|---|
| Art and animation volume (the real bottleneck) | CC0 base packs, one consistent stylised art direction, modular characters. |
| Safari quirks (large shaders rejected, 60 fps cap) | Test Safari from M0 onward; keep shader permutations small. |
| No Firefox WebGPU on Linux | WebGL2 fallback is built in and tested (`?renderer=webgl`); budgets are measured on WebGL2 too. |
| Cross-origin isolation blocks third-party embeds | Self-host all assets; `postMessage` fallback path. |
| Benchmark bias toward PlayCanvas | M0 bake-off on our own scene decides. |
| Scope creep (life sims are huge) | Strict releases (section 8). Building and watching come first; direct control stays as it is. |
| Trademark ✅ | Done: the project is called Idyll Lives (see "Naming" below). Use only original or licensed art. |

---

## 10. Decisions made

1. **Devices:** desktop only.
2. **Art direction:** modern, minimalist and stylised; all art is replaceable through the manifest.
3. **Name:** Idyll Lives (`idyll-lives` in slugs).
4. **Licence:** Idyll Lives is free software, released under the GNU General Public License v3.0 (or later). You may use, study, share, and modify it; if you distribute Idyll Lives or a modified version, you must make the source available under the same licence. (MIT until 0.1.0.) Third-party parts keep their own licences: the art is CC0; the voice model, Supertone's Supertonic 3, is under the BigScience OpenRAIL-M licence, with use restrictions that bind every player; the dictionary (Misaki) is Apache-2.0; the optional language model, Liquid AI's LFM2.5 350M, is under the LFM Open License v1.0. See `web/public/assets/CREDITS.md`, and `THIRD-PARTY-LICENSES.txt` in every build for the bundled code.

### Naming

The project was first called `open-sims-wasm`. It was renamed to stay clear of Electronic Arts'
trademarks, and for 0.1.0, the first release, it became **Idyll Lives** ("idyll" sounds like
"idle": lives you leave alone and watch):

- Display text says *Idyll Lives*; slugs (package name, repository, file names) use `idyll-lives`.
  The game is at https://krecks.github.io/idyll-lives/.
- Characters are called **residents** in everything a player or pack author reads.
- The temporary mood modifiers are called **feelings**. Content packs and saves written with the
  old key names still load (serde aliases in sim-core, `LEGACY_KEYS` in both content mergers).
- Kept on purpose: the repository folder, the crate names `sim-core` / `sim-wasm` ("sim" for
  simulation), code identifiers such as `sim` and `SimView`, and the browser storage keys
  (the IndexedDB database, settings, blueprints, the voice speed test and the voice and
  item-picture caches keep the names of earlier versions) so existing saves, settings, blueprints
  and downloads survive.
- Credits and the README keep a short nominative disclaimer naming EA's trademark.
