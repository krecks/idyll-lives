# Idyll Lives: Technical Plan

*Status: **v0.9.0** is the latest release; what each release brought is in [CHANGELOG.md](CHANGELOG.md).*

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
- **No minimum machine** (set by the owner, 2026-11-23): nothing in the game is held back for a particular machine. (The Apple M1 (2020) the owner set as the floor on 2026-11-03 is dropped as a reference.)
- **Modular and easy to maintain.** The project will be heavily adapted; every concern lives in its own module behind a small interface.
- **Art will be replaced.** Code and content refer to assets only by key; files are swapped through the asset manifest or override packs.
- **Art style:** modern, minimalist and stylised.

## Status

| Area | State |
|---|---|
| Sim core (Rust) | Needs, utility AI with smart objects, A* pathfinding, lot/rooms, mesh generation, clock. **Town** of plots and households with object ownership. **Social life**: directional relationships (friendship, romance, chemistry), 17 data-driven social interactions with success chances, feelings and emotions that steer behaviour, jealousy, partners and break-ups, fights, story events. **Gender and attraction**. **Daily life**: data-driven careers with shifts, pay, performance and promotions; time-of-day schedules; household funds; neighbours visiting each other (guests stay on the host's lot and use only non-private objects). **Careers and skills**: 1000 positions (25 categories × 4 tracks × grades A–J) expanded from `careers.json`; 13 skills (12 in the base game, Gardening with the garden) trained at work and with home objects; skill fit decides hiring (probation), the workweek length and promotions. **Money**: dollars, weekly rent by lot size plus bills by household value, per-use costs only for consumables, debt. **Day/night rhythm**: sleep through the night, early risers for early shifts, slower need decay asleep, practice fatigue. **Build/buy** (`home.rs`): buy (auto or exact placement), move, sell, recolour (visual only), upgrade quality (instant Buy-mode purchase), walls (straight and diagonal), doors and windows with reachability checks; multi-slot objects (double beds). **0.2.0:** cooking (recipes as chains of stations, with food and dishes as items), babies cared for in the crib, a phone for everyone from teens up, basements, guests keeping their hosts company, up to four in a conversation, residents' own wishes; an integer clock (20 ticks a game minute); content checked when it loads. **0.3.0:** every object keeps a stable id for life (`ObjectInstance::stable`, never reused): commands, views, the renderer's snapshot and saves name objects by it, and the `seen` guard on commands is gone (inside the sim, objects are still indexed by their place in the list and remapped on an edit); a player-chosen colour per part instead of styles (`recolor`); lamps switched on and off by the player and by residents as they come and go after dark; objects that open and stay open (a wall bed, a sofa bed, a futon) and parts that open while used; goals in another walkable region refused at once (reach labels); the player is told of residents without a bed (`EventKind::NoBed`), and Choose Home warns before a household moves into a home with too few. Save format version 29 (0.2.0 wrote 23, 0.1.0 17; older saves are migrated on load and get object ids). **Authenticity review**: age rules (content `stages`) for free will and orders alike; a place and a facing per slot (`places`); games for two fetch a partner (`partners.rs`); things on tops used where they stand; uses that need an hour, open sky or something to work on, and laundry that piles up; each standing user at their own slot, beds and desks come to from the side or behind; the dish carried to the table and town food fetched and eaten sitting down; babies carried to the high chair (`carry.rs`), harvests to the fridge; outfits; talk from seats and over meals. Save format version 32 (30: laundry, 31: outfits, 32: the school day kept). **0.6.0:** households' cars (`cars.rs`) and open worlds of three sizes with a road graph, travel times and trips (`travel.rs`); save format 35. **0.7.0:** the way to work or school chosen once among a car seat, a neighbour's lift (`cars.rs`), the household's bike or e-scooter (`ride.rs`, content `vehicle`), the town's bus (`bus.rs`, content `bus`) and the walk; a host's offer (`hosting.rs`); a purchase set up at home (`setup.rs`); careers paced (`life::shift_performance`); a game day of 30 real minutes at normal speed and a 60 Hz tick (`TICKS_PER_SECOND` 60 for real time: walking, driving, doors, lights; `TICKS_PER_MINUTE` 75 for game time), save format 36 (older saves' ticks rescaled on load). |
| WASM bridge | Sim in a Web Worker at 60 Hz; triple-buffered snapshot over SharedArrayBuffer (with conversation, emotion and animation fields); requests for saves and social options. |
| Renderer | **Only the viewed lot is drawn** (its walls/floors are generated per region; other houses appear as silhouettes; residents elsewhere or at work are hidden); the view follows the selected resident. PlayCanvas 2.23 on WebGPU (WebGL2 fallback; Babylon.js 9.30 until 0.3.2), instanced draws everywhere, time-of-day lighting with a gradient sky dome and distance fog, procedural landscape (terrain hills, clustered forests, lawn detail), roads, swappable hair models, procedural conversation body language. The game draws live while it runs; paused, its picture is held as the menus' town's is while it stands still, drawn again only when something in it changes (`heldFrames.ts`). **0.2.0:** three graphics presets in one table (Low, Medium, High), Auto resolution with a pixel budget of about 1920×1080, and a frame rate cap paced in whole display refreshes. **0.3.0:** every object is a glTF built in code (by Blender scripts until 0.8.3, by Rust since: `crates/art`, `cargo art`; the code-built primitive models are gone, and a file that fails to load is a stand-in box, tried again in the background), with moving parts played by uses (`objectMotion.ts`) and ambient clips within `ambientRange`; Classic, physically based, is the game's shading since 2026-11-10 (the soft look a setting); the sun comes in only through windows and doorways; the front and bathroom doors swing; neighbours' windows go dark when they go to bed; mirrors reflect on every preset (each reflection held and drawn again when something in it changes, on Medium and Low at most 10 times a second). Low draws the street with far models, leaves the street and residents without sun shadows, merges the lot's furniture shadow casters, and draws only live quilt rows: the village orbit by day falls from 447 draws and 5.1 M triangles a frame to 293 and 1.2 M, with no long tasks (M3 Pro proxy, 1920×1080). **Authenticity review:** 134 models rebuilt or new at real sizes and working heights; 168 resident clips checked on both bodies at every life stage (the slide, the bunk's ladder, rides, games and sport, reaching up, social gestures, seated legs fitted per stage); a lamp's light aimed by its shade and, on Medium and High, kept to its room (`roomLights.ts`); glow no longer through the parts in front of it; doors and gates open only for the one going through; a use sounds like its thing; quilts in each bed's colours, and pyjamas, towels or swimwear by what is done; the daybed is lain on its side by day (`SIDE_BEDS` `bedDaybed`, I-bedroom-03). |
| UI | Main menu, settings, credits, load/save with thumbnails, pause menu. **Menus over a live 3D town** (`render/pc/overview.ts`, `game/menuScene.ts`): the shared renderer draws a whole-neighbourhood overview (every lot as a dressed shell, street, gardens, park; golden hour on the title screen until 0.8.3) behind New game's steps, which use it as a clickable map (lot glow, tags pinned to lots, camera glides), and Move in flies the camera into the lot. New game (0.8.3): **Land → Streets & zones → Build → Household (the town's own, or the household creator: gender, attraction, skin, hair, clothes, traits, perks, bonds) → Home**, on the town stage with a free camera; the main menu shows a picture of the last world made (`menuStill.ts`). The household creator shows the in-game 3D resident (`render/preview`: a second PlayCanvas app, only alive while the stage is shown or portraits are drawn) with drag-to-turn, zoom to the face and little reactions; avatars everywhere are 3D head-and-shoulders portraits, drawn in batches and cached by look. HUD: resident panel (Now / People / Feelings), social menu with chances, story feed, speech bubbles. Since the authenticity review the pie menu greys out what the selected resident's age doesn't allow. |
| Not yet | See the Roadmap under Open points: furniture resizing back for the objects it suits, more languages for text and voices (see Languages), and smarter thoughts if the research's numbers ever justify it (research done 2026-11-01). Dropped: the renderer bake-off (M0); Babylon carried every build from 0.2 to 0.3.2, and after 0.3.2 an engine comparison moved the renderer to PlayCanvas. Characters are skinned on the GPU from a pose texture (`render/pc/characters.ts`). Lamps give light (0.10): with clustered lighting (quality `clusteredLamps`, Medium and High; Babylon's `ClusteredLightContainer` when it was measured) every lamp lights, up to 32; otherwise the four in the largest rooms. Measured on an M3 Pro at 2880×1800, uncapped, 20 lamps lit: WebGPU 160 → 146 fps (+0.6 ms), WebGL2 90 → 72 fps (+2.8 ms); with four lamps no difference. (On Babylon, clustered lights needed its default falloff, and Babylon 9.29's clustered PBR shader named a missing `normalW` for sheen, our fabrics, patched at start-up; the PlayCanvas renderer needs neither.) (Cutaway walls and floor coverings are done.) **Simulation scale (0.19):** objects are indexed by lot (`sim-core/src/index.rs`), so decisions only see what a resident may use (home, public places, the lot they're visiting); lots nobody watches run at lower detail (`sim-core/src/lod.rs`): residents away from the viewed lot, the player's home and the player's household take a turn every game minute and catch up (`World::lod` switches it off). Big town (34 houses, 1,835 objects, 64 residents, `tests/big_town.rs`): about 67k → 160k ticks/s at full detail (results identical) and about 345k with lower detail, whose outcomes match full detail within noise. A town holds at most 64 residents (`MAX_SIMS`); raising it means a bigger snapshot and two hard-coded limits in the renderer. |

### Startup performance
- The renderer (engine, compiled shaders and pipelines, cached object meshes, landscape) is owned by a long-lived **game host** and reused across sessions; sessions only swap world contents.
- Sessions are **prepared in the background**, paused and without input: the latest save once the main menu has settled (since 0.2.0 the menu no longer waits for it). Their world builds hidden behind the menus' town overview (camera layer masks, `render/shared/layers.ts`) and is drawn invisibly for a few frames so its shaders and pipelines are ready. Starting a saved game is then a fade. A new game or a house picked on the home screen starts its own loading screen instead: it fetches every file the viewed house needs (furniture, garden, textures, residents) before the world is built, so the first frame after Play does not stream. The overview and a new game of the same town share one landscape.
- **Boot set:** content, manifests, the sky and the overview's art, plus every menu icon and the character set (so nothing reachable before Play is left to stream). The viewed house's files are fetched on the game's loading screen; the rest of the town and the catalog then stream in the background (`assets/stream.ts`) behind the small “Game Data is streamed and cached” indicator. The game screen, the glTF loader and particles are separate chunks loaded on demand. Menu at 50 Mbit/s: 5.2–6.5 s → 3.3–3.8 s before the menu assets were added; Continue is still instant, and Move in's 120–190 ms long task since 0.16–0.18 is still to find.
- Measured (production build, headless Chrome, M-series Mac): Continue 286 ms with 110 + 165 ms main-thread stalls → **48 ms, no long tasks**; Move in 397 ms → **98 ms, no long tasks**. The remaining cost of a cold start is WebGPU shader/pipeline compilation, which now happens while the player is still in a menu.
- **0.2.0:** the content, house templates and manifests arrive in one `bundle.json` instead of 56 requests in three waves; the sky's lighting downloads with the page; the isolation service worker keeps the build's hashed files for returning players; the menu shows before the latest save is prepared. Production build on an M3 Pro, Chrome 153, Pages-like server, median of 3: a cold menu at 10 Mbit/s 3.44 → 3.14 s (219 → 164 requests); a returning player after 10 minutes 1.54 → 1.34 s; the menu with a save, warm cache, 2.83 → 1.42 s (the release review's §3.5.3).

### Languages
The game is **English only for now; other languages will follow, for text and for the resident voices.** Until then, new work keeps translation possible:
- **The simulation speaks in ids and numbers, never sentences.** Story events carry kinds, residents, numbers, skill/career/goal indices; planner outcomes carry reason codes; goal text is formatted in the UI from the content's template (`goalText`). Remaining exceptions to clean up: command errors are English strings shown as toasts (they should become codes with parameters), and the journal's event templates and content labels are English text inside the content files.
- **Text comes from content by id.** Labels (needs, skills, activities, goals, objects, feelings, story templates in `events`) stay in content; translations will be locale files that override them by id (e.g. `content/locales/<lang>.json`), loaded like content packs. Templates use named placeholders (`{a}`, `{skill}`, `{n}`), not word order.
- **Interface strings** are still written in the Svelte components; they move to a string table when the first second language starts. Numbers, money and times go through formatting helpers (`ui/format.ts`) so they can switch to `Intl` per locale.
- **Voices** are recorded or synthesised per language; spoken lines are keyed, not looked up by English text.

### Balance targets (Living mode)
`crates/sim-core/tests/sim/balance.rs` states what Living mode should feel like: a working household saves about a room's worth or less per week (median $100 to two rooms, about $2,000), nobody holds more than $50k after 60 days, residents climb at most 8 grades (6 on average) in 60 days: nobody climbs from A to J (`cargo test --release -p sim-core --test sim balance:: -- --ignored --nocapture`, 180 game days).
- Before the rebalance: median weekly savings $8,025, richest household $190k, 6.3 grades on average, up to 9.
- After: median $1,099, richest $28.8k, 1.7–1.8 grades on average, at most 3 (also on other town seeds). Pay grades $12–52/h (were $15–300/h); promotions about a third as fast (`performancePerShift` 2.5, `performancePerFit` 1, new `performancePerMood` 4, was a fixed 15); rent base $60 + $0.25 per tile; new living costs of $150 per resident in the weekly bills.
- Pay by track (after 0.5.1): grades $12–60/h, each track's `payFactor` (0.5–1.83) bending pay up to grade J, and a promotion bonus of half a week at the new grade: median $621, richest $14.8k, 1.2 grades on average, at most 3 (seeds 7, 11 and 19).
- Careers paced (0.7.0): promotions came after 16 to 60 shifts at every grade (`performancePerShift` 2.5), so a resident barely climbed two grades in a working life. A shift now fills `performancePerShift` 18 at the entry grade in a middling mood, ±`performancePerMood` 20 by mood and `performancePerFit` 2 per spare skill level, divided by 1 + grade × `performanceSlowdown` 0.15 (more than 30 minutes late, half): the first promotion after 4–5 workdays (fine to happy), the top grade's after 7–10, A to J in about 50 workdays happy. Median $1,640–1,975, richest $37–43k, 4.4–4.8 grades on average, at most 8 (seeds 7, 11, 19 and 3, 5, 23).
- Slower careers left the player's household with little news in a quiet week; goal ideas nobody answers are now taken on after two days (`planner.suggestionDays`), so an idle household still has something to work toward.

### Open points
- **Left open by 0.9.0** (walk mode, the traffic round taxis and junctions, lighter drawing): seed bases 1, 3, 4 and 5 still fail at world generation (only 0 and 2 run their towns).
- **Left open by 0.8.4** (the audits and the model gate at zero): a church's tower stands at the seed's end far off and over its door near, so it can jump ends as its lot streams in (`townBuildings.ts` `towerOf`); a storey with walls but no closed room yet drops out of the far versions (`house.ts` `stacks`); a bird species the web knows and `crates/sound` doesn't throws in the page fallback of `WorkerSounds.song` (no shipped species does); artkit's `uv::along` still shears a tapering sweep's texture (dropping the ring scale stretched the trees' bark: reverted); a taxi booked for a visit dropped at home (the place shut) still comes and waits; a faint white strip runs from a house onto empty grass in the menu still (Elmview, right of centre); no toddler stage: a 2-year-old is a child and may shower alone (a stage 2–4 washed in a bath would need homes with baths).
- **Cars that fit the job:** done (0.8.3): the career and grade of the household's best earner pick the car (`careers.json` `carBodies` and `car`, `cars/looks.rs`), saved (format 41).
- **Cars for more than work and school:** residents walk to visits, the park and the shops; trips there could go by car too.
- **The bus, dropped (`fix/car-guard`):** 0.7.0's timetabled bus is gone, with its stops (a town file's or a save's stops are no ways out: dropped as they load); a shared work taxi takes its place. What the cars learnt with it stays: one queue of claims where their ways cross or meet (`traffic.rs`), pulling over on 6 m lanes (the kerbside furniture keeps back there, `WorldStructure.pullOverStreets`), the last-resort guard.
- **Tree impostors: not perfect, good enough for now** (`fix/far-woods`, 2026-11-19, `render/pc/impostors.ts`): the far woods in the town view are pictures of their trees on cards, baked at load from 8 sides at 2 heights (colour by day and by night, normals, occlusion, glossiness, how much tint each leaf takes) and blended between the nearest. Measured tree for tree against the real ones at 330 m they agree within a few levels by day and at dawn; the 15–20 % blue by night came from the cards' gloss read from coarse mipmaps (`fix/impostor-night` reads the finest level); the crowns are a little softer, and zooming across the switch (250 m from the eye) still shows a small change. Left: a night look while orbiting (halfway through a side or height blend the gloss still drops, so a tree may turn a little bluer as it passes through), the town view's other shaders (ground, buildings) that still compile on the first zoom out, and a fade at the switch if the change still shows.
- **Left open by the real town** (0.8.3, `feat/real-town`):
  - Choose Home names a flat's floor but can't open its storey: the menus' town never opens a building (it carries ground storeys only, `game/townPreview.ts`, `pc/overview.ts`).
  - In the cutaway a closed part's lid lies at the walls' stub; the back walls and partitions the cutaway leaves standing round it show their inner faces over it.
  - Build's high street is as dense as the generator's (90 % of its shops) but not in its continuous rows: Build's named shops don't go into parades along the high street first (`macro.ts` `namedLots`).
  - A car that gave up its turn round in a ring (`cars/driving.rs` `give_up_turn`) is placed after a load as one turned round twice is, from its plain way.
  - A paving outline that crosses a tile without any of its corners inside (a sharp spike) leaves that tile unpaved; partial tiles are painted cell by cell on the far map, where anti-aliasing could leave a faint seam (none seen).
  - Mountain towns' small, steep centres get no buildings of flats; the inn gets no flats over it and a restaurant only 8 times in 10; the leisure centre and the church have no storey above, and the hotel, school and town hall no third.
  - The built gate measures a building of flats' far box a little lighter than its dressed walls (30 of 255 against 28) and the restaurant's mansard at 19° near against 45° far; both were within the limits when the samples stood turned (baseline in `tools/art/gate-baseline.json`).
  - The dense pass's approaches along country roads into town keep their own corner rule (`dense.ts` `COUNTRY_CORNER`), apart from `macro.ts` `roadMeets`.
  - The childcare check counts using the building's shared things as leaving home; `lifecycle::arrive` doesn't check `is_home` itself.
  - Villages with lots out along the country road into them can't be moved in step 2 (their outskirts reach the town only through them).
  - The bench's speed-ups (2026-11-20, `tools/bench/README.md`): one page per picture set (`window.__bench`), pages kept loaded (`keep.mjs`), the wasm packages kept by content (`tools/wasm-cache.sh`). Left: the town view's spots are stepped home first, as the game opens on the home (centre first, the lots round the home came back dressed otherwise, 0.2 % of the picture); `create.mjs`, `check.mjs` and the `/src`-importing tools still load a page per job and sleep fixed times; an `ll-tools` binary kept by content (each worktree's first production build or `cargo` tool compiles it, about 4 minutes) is not done; `--prod` measured no gain for the benches and is for checks that need what players get.
  - Bench time (2026-11-20, `tools/bench/README.md` "Bench time"): states made without a browser at an exact tick (`state.mjs`), swapped into an open page (`__bench.load`, about 6 to 8 s against 15 s for the first game a page opens), played in fixed steps of the game's ticks (`__bench.run`, 600 frames in about 5 s). Two runs from the same state count the same frame for frame, and WebGL and WebGPU the same as each other; the pictures are the same to the pixel but for the garden birds' poses (up to 0.01 % of a picture over the home). The play camera's comparison on this branch, one saved moment (the town-day scene, 12:55, Medium) and 600 frames of 1 tick each, draws a frame (mean): over the home 548.2 at 24 m and 695.7 at 60 m, over the centre 556.6 and 746.6; the sun's picture of the still casters drawn again 36 times over the home (33 as the sun moved, 3 for its depth) against 15 to 16 over the centre. A state made without a browser has the page's town and households (its generator under Node, V8 as in Chrome), its residents elsewhere than a page played in real time puts them. Left: 0.8.3 can't play a state in fixed steps, so the comparison against it is still real time (each build makes its own state from the same code and tick: 0.8.3 can't read save 42); why the still picture is drawn again twice a sun step over the home (`moved`) is not looked into; the birds' poses still differ run to run; a camera moving with the wall clock (the bench's orbits) doesn't repeat in fixed steps.
  - The renderer's memory against 0.8.3, measured again with the collected reads (2026-11-20, `memory-ab.mjs`, 3 runs a build in turn, Medium, WebGPU, 150 s each, the world `country-lrstm-7` on both): the real town's renderer footprint is 1,944 MB against 0.8.3's 2,028 (−4 %), the GPU process's 1,380 against 1,488 (−7 %), PlayCanvas's VRAM 553 against 602 MB, in a bigger world (486 plots, 682 residents and 33,083 objects against 412, 633 and 26,820); the sim's WASM memory is 107 against 87 MB and the page's JS heap 279 against 274 MB. The 8 to 10 % more the run's gate measured earlier was reads taken as they stood (up to 2 GB not yet collected). The two builds play other households of the same world (0.8.3's household card 0 is the Nguyens' house, the real town's the Johanssons'), so the numbers compare the towns, not one household.
  - Measured against 0.8.3 before the run (2026-11-20, M3 Pro, back to back): the river city's step p99 at 10× is 22 % over it (the budget was 20 %), with 155 residents at full detail round the home on average against 136 (the town is denser there). The town view's +12 % at 400 m (the budget was as many) was one frame each, the base's a plain one and the real town's one where the sun's kept picture of the still casters was drawn again (about 130 draws); over 600 frames at 1× (the town-day scene, Medium, medians of 3, 2026-11-20) the real town drew 1 to 2 % more than 0.8.3 (more parked cars and bikes, more residents' clothes, the kept picture drawn again about twice as often), and after the draws round (a group's trails in the garden paths' draw, the residents' shadow-only copies only where they may cast, a parked bike ridden off across town no longer drawing the kept picture again) 1,145 / 1,146 draws a frame at 400 m against 0.8.3's 1,153 / 1,149 (WebGL / WebGPU) and 1,443 / 1,435 at 783 m against 1,440 / 1,438. Near the centre (24 and 60 m) the kept picture was drawn again more often than in 0.8.3, mostly because PlayCanvas fits the light's view again at every redraw: its depth to every caster's box (a car or a town-wide instanced draw moving anywhere along the sun's way moved it by 1e-6 to 1e-5) and its place from points turned into the camera's space and back (float noise as the camera turns). The shadows round (2026-11-20) keeps the view the picture was drawn with while PlayCanvas's differs from it only by that noise and its casters lie within its depth (`sunShadows.ts` `fitSquare`; a redraw that draws the picture anyway takes PlayCanvas's own, so the pictures are the same to the pixel), keys the parked cars' draws as the bikes' are (`Instances.castKey`), and leaves the map as it is at a step of the light that doesn't turn the sun (the night's). The bench counts why the picture is drawn again (`sunShadowWhy`). Over 10 s at 1× in the town-day scene (Medium, medians of 2 runs after and 3 of 0.8.3, both backends alike) it is drawn 16 to 17 times at 24, 60, 400 and 783 m, once a step of the sun's, against 0.8.3's 21 to 29 and 17 to 30 before the round. The near draws are still over 0.8.3, for what stands and moves there, not the shadows: 539 against 469 a frame at 24 m over the centre (+15 %), 759 against 675 at 60 m (+12 %), the moving pass 5.5 against 1.3 a frame at 24 m (more residents and cars about the centre); over the home at 24 m 686 against 556, but the scenes differ there (the real town's household is at home and moving, 0.8.3's out), so the play camera's gate (T1: 559 against 482) isn't settled by it; at 400 and 783 m 1,093 and 1,354 against 1,099 and 1,368. The parked cars were one draw per body, paint and age; they are one per body and age now (`feat/real-town-carpaint`: the body painted white, each car's copy tinted to its paint where the body is white, `GAME_TINT_WHITE`; the pictures the same but for single pixels in the shade, where the sun's map is fitted to the casters' boxes as they are grouped). Measured over 600 frames at 1× (2026-11-20, this round's town-day scene, Medium, medians of 3, WebGL / WebGPU): parked cars' draws 10 / 10 at 400 m and 11 / 11 at 783 m against 18 / 19 and 32 / 31 before (0.8.3: 18 / 18 and 25 / 25); the frame 1,186 / 1,182 at 400 m (median frame 1,187 / 1,189) against 1,289 / 1,303 (1,206 / 1,205) before and 0.8.3's 1,162 / 1,199 (1,158 / 1,161), and 1,582 / 1,583 at 783 m (1,555 / 1,553) against 1,599 / 1,616 (1,607 / 1,613) and 0.8.3's 1,563 / 1,569 (1,568 / 1,568). Neither of the plan's levers helps the river city's p99: flats at two storeys left the river city as it was, and the homes kept for sale made shells slowed it 18 %. That was not what shells add to the world: the town's day ran differently, and in it a baby was home with no grown-up for much of the day, and both the babysitter's care (`life.rs` `childcare`) and a crying baby's call (`World::woken_for`) looked through every object in town for the crib each tick (31,000 objects, twice). They now look at what the baby holds a place at and the home plot's objects. Measured again after that (2026-11-20, node, medians of 3, back to back): the shells lever costs about 2 % (1,296 against 1,329 ticks a second, step p99 30.4 against 30.3 ms; the rest spread over path finding as the day goes otherwise, no one pass), where before the fix it cost 7 % (1,226 against 1,318); and the river city's step p99 is 18 % over 0.8.3 (30.3 against 25.7 ms; 5 to 20 % round by round), throughput 5.5 % under (1,329 against 1,407).
- **Left open by 0.7.0** (for the owner): the money balance after faster promotions, the 30-minute late grace, only four idea toasts at once, nothing to drink for a host to serve, the Firefox model-load fix unverified in Firefox, Big towns' rush-hour traffic (a preview), and coarse parked-car tiles.

### Catalog gaps left to plan
Not built: no catalog entry and no art. Recorded so the gaps aren't lost.
- **Heating and cooling:** a radiator, a space heater, an air-conditioning unit and a thermostat. Heating is implicit today: rooms are warm with nothing in them, so a radiator or a thermostat would need the sim to know a room's temperature first.
- **Aquarium and fish tank:** no pet animals of any kind: only `dogHouse` and `statueCat` stand in for them. A tank would be decor with a feeding use, not a pet.
- **Toddler stage:** no cot, cradle, potty, changing table, stair gate, toddler bed or toddler toys, and none is added. Kept as decided: "no toddler stage: a 2-year-old is a child" (Open points above), and `kids.json` starts at `kidsTable`, `kidsChair` and `kidsBedCar`; the furniture alone would contradict the stage.

### Roadmap

| Release | Theme | Done when |
|---|---|---|
| 0.9.1 | Big towns' traffic finished | Rush-hour jams in view of a big town gone. 0.8.0 shipped the traffic's own clock and real junctions, and with them fewer cars held up in view and for far less long (a sweep of seed bases 16–35: 10 cars in 7 worlds, the longest 100 game minutes, from 25 and 927); the rest is left. Parked for it (the owner's call, 2026-11-08), each on its own local branch, not merged: **faster cars and bikes** (`fix/vehicle-speeds`: cars 36 km/h in town and 80 on country roads, the city bike 20 km/h, the e-scooter 25; it tips big towns into jams in view, and cyclists wait hours behind them: generated seed bases 0, 17, 23 and others failed), superseded by **the traffic's own clock** (shipped in 0.8.0; `feat/drive-clock`, 2026-11-09: every vehicle, cars, the taxi, bikes and e-scooters, goes `DRIVE_SCALE` times as fast as real time with its look-aheads, claims, jams and the lights timed alike, so junctions let as many more through as the cars go faster; a drive across a big world takes about half an hour of game time, not two to four hours; save 37), and **leaving for work at varied times** (`feat/work-jitter`, also on `fix/vehicle-speeds`: each household ±30 game minutes a day, a taxi run one offset, lateness judged from the household's own aim; seed base 24's big-town taxi riders came 6.6 min late against the limit of 6). The jams in view are nearly all cars driving home after work, which the morning spread doesn't touch: spread the end of the shift too, then fix the jams themselves. 0.8.3 took on what the full fill (up to 680 neighbours) brought out: the rider-and-car deadlock, a car turning round backing into a claim given to another, a car pulled over where it couldn't pull back in, a driver sent off to work from the wheel, and visits cut short by a taxi booking; and it tried a departure spread in the sim (`life::early`, up to 20 game minutes by household), which made the full fill worse (seed bases 1, 3 and 4 failed, work taxis no more on time) and is left at 0 (`EARLY_MOST`). Also: the generated-world sampler on seed bases 6, 10 and 34 (roads side by side) and 36 and 44 (a 46 m court); a car pulling over slides sideways; a driveway opening onto a junction. |
| later | Furniture resizing again | Buy mode's resize arrows come back (taken out of `web/src/game/buildmode.ts`; a door's or window's arrows there show how) for the objects it suits, once moving parts follow a resize well (a fridge stretched wide, a wall bed); see [docs/design/objects.md](docs/design/objects.md). |
| later | Smarter thoughts (research done 2026-11-01) | The language model also writes thoughts for actions the utility AI already chose, feeding Resident voices. The utility AI keeps deciding; a model may at most re-rank its top-3 shortlist, entering the sim as a recorded command so saves and replays stay deterministic. *(No behaviour or save change. Measured 2026-11-01: 1.21 decisions per resident per game hour, 0.75 of them with a shortlist, the top candidate holding 45 % of the top three's weight and the second within 10 % in a fifth of those — so the re-rank has little to reorder, and 38 % of decisions have nothing to choose at all.)* |

### Known issues and findings from the first build
- **Babylon SSAO2 on WebGPU failed to bind its sampler on about 1 in 4 startups** (`randomSampler not found`, then `createBindGroup` errors and a frozen frame), so ambient occlusion was off by default. It was removed in 0.2.0 with the Ultra preset.
- **Standard-mode snapshot rendering bakes light uniforms and the clear colour into the recording.** The renderer re-records whenever lighting changes, throttled to every 5 game minutes. Moving residents (thin-instance buffers) and helper meshes update without re-recording. Only the menus' town uses it; the game draws live (retrying it in play is P1-07 in the release review).
- **Bundle size (0.19):** Babylon was imported by module (never from the `@babylonjs/core` root): main chunk 1.76 MB (478 KB gzip), was 6.6 MB; 1.80 MB (630 KB gzip) on 2026-11-14, PlayCanvas 1.27 MB in a chunk of its own. Since the port PlayCanvas comes in through `render/pc/engine.ts`, its release build by path.
- **Diagonal walls (limits):** movement stays on whole tiles, so the tile a diagonal wall crosses is closed to walking and furniture (its two half-triangles are dead space; a diagonal door tile is walked through). Residents do walk the corridor between two parallel diagonals. Roofs: a convex indoor outline with diagonal sides gets an exact hip roof over that outline; other shapes (e.g. an L with a diagonal corner) get the usual rectangles over their tiles, with split tiles covered whole (a deeper eave over the outdoor half). Neighbours' houses with diagonals are drawn as their bounding box. Landscape trees aren't removed when a room is built over them (straight walls too).


## Architecture

```
┌──────────────────────── Main thread ────────────────────────┐
│  Svelte 5 UI (HUD, needs, build/buy catalog)  ◄─ 10 Hz store │
│  Input → commands                                            │
│  PlayCanvas renderer (60–120 fps, interpolates sim state)    │
└───────▲─────────────────────────────┬────────────────────────┘
        │ SharedArrayBuffer            │ SharedArrayBuffer
        │ (triple-buffered snapshots)  │ (command ring buffer)
┌───────┴─────────────────────────────▼────────────────────────┐
│  Sim Worker: Rust → WASM                                     │
│  fixed 60 Hz tick · needs · utility AI · pathfinding ·       │
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

1. **Sim and render are fully separate.** The sim runs at a fixed 60 Hz in a Worker, and the renderer interpolates between the last two snapshots. A slow frame never slows the sim, and a sim spike never drops a frame.
2. **No per-object JS↔WASM calls.** The sim writes a compact binary snapshot (positions, rotations, animation IDs, state flags) into a SharedArrayBuffer. The renderer reads it with typed arrays, with no copies, no serialization and no garbage-collection churn. Commands flow back through a lock-free ring buffer.
3. **The UI is not in the hot path.** Svelte receives changes at about 10 Hz (`view::UiSync` in Rust, `game/uiUpdates.ts`): keyed by id, drifting values rounded to what the HUD shows, details only for the player's household and the inspected resident; a full update at start and on request (`resync`). Needs bars animate with CSS transitions, so the UI never forces per-frame DOM work.
4. **No garbage in the frame loop.** All per-frame JS buffers are pre-allocated. The Rust side allocates nothing in steady state.
5. **Cross-origin isolation** (`COOP: same-origin`, `COEP: require-corp`) is needed for SharedArrayBuffer. Fine for self-hosting, and Vite dev and Cloudflare/Netlify all support it. Fallback when unavailable: `postMessage` with transferable buffers.

**Later option:** if main-thread jank appears, move rendering to its own worker via `OffscreenCanvas`. The interface makes this a contained change.

**As built (0.2.0)**, where it differs from the plan above:
- **Snapshots** are triple-buffered (`core/snapshot.ts`; until 0.6.0 one buffer guarded by a seqlock, which JavaScript's memory model doesn't make safe), each with the sim step it shows so the renderer clocks by sim steps. Without cross-origin isolation they're transferred as a copy.
- **Commands** are JSON over `postMessage`, applied between ticks; no ring buffer was built.
- **Workers:** the game's own are the simulation, the voices, the language model and the water sounds; no asset worker was built. Saves are kept in IndexedDB, not OPFS.
- **Drawing:** the game draws live. Snapshot rendering only replays the menus' town (above).
- **Hosting:** GitHub Pages can't send the isolation headers, so a service worker (`public/isolation-sw.js`) adds them.
- **The UI's transitions:** the release review found that CSS transitions of the bars' width forced style, layout and paint on every display refresh (P2-01). Since 0.2.0 the bars move by transform, which the compositor animates: layouts went from about 108 to 11 a second at speed 5. Chrome still recalculates style each frame for running CSS animations, about 0.13 ms a frame on an M3 Pro (the release review's §4).

---

## Simulation core (Rust)

Pure Rust crate `sim-core` with **no browser dependencies**. It is tested and benchmarked natively with `cargo test` / `cargo bench`; a thin `sim-wasm` crate holds the bindings.

| System | Design |
|---|---|
| **Data model** | Data-oriented: structure-of-arrays storage with generational IDs. No ECS framework, to keep the WASM small and the code fast. |
| **Time** | Fixed 60 Hz tick, 75 ticks a game minute (a game day is 30 real minutes at normal speed); game clock with pause / 1× / 2× / 3× (3× = more ticks per real second, capped). Deterministic, seeded RNG. |
| **Needs and mood** | Hunger, energy, bladder, hygiene, fun, social, comfort, environment. Each decays along a curve; mood = weighted sum + feelings (temporary modifiers). |
| **AI ("smart objects")** | Objects *advertise* interactions with need deltas. Each resident scores them with need-weighted utility curves, distance cost and personality modifiers, then picks from the top-N with weighted randomness. Player commands go to a queue and override autonomy. This is the proven smart-object model of classic life sims. |
| **Interactions** | Small state machines: route to slot → animate → apply effects → exit. Multi-resident interactions (talking, sharing a sofa) reserve object slots. |
| **Pathfinding** | Tile grid (1 m tiles, 4 sub-positions); walls block tile edges; doors are portals. A* with Jump Point Search, a cached room-level graph for long routes, string-pulling for smooth paths. Slot reservation plus simple local avoidance between residents. |
| **Lot / build** | Multi-level tile grid; walls on edges, plus diagonal walls corner to corner across a tile (one per tile; rooms are found per half tile, so the two halves can be inside and outside); rooms found by flood fill (used by the AI, room lighting and cutaway). **Rust generates wall/floor meshes** into WASM memory; the renderer uploads them directly. |
| **Picking** | Planned: a ray against the tile grid and object bounding boxes, done in Rust. As built: the renderer picks (`render/pc/picking.ts`: `pick`, `pickPlane`, `pickOnWall`, in TypeScript), by what's drawn of an object; no GPU readback. |
| **Saving** | JSON with a versioned schema (`sim-core/src/save.rs`, version 43 since 0.8.4, 42 in 0.8.3, 39 in 0.8.1 and 0.8.2, 37 in 0.8.0, 36 in 0.7.0, 35 in 0.6.x, 33 in 0.5.x, 32 in 0.3.1 to 0.4.2, 29 in 0.3.0, 23 in 0.2.0; no older save opens, and none is migrated: until 1.0 a change of format leaves the saves before it behind), stored in IndexedDB (`web/src/persistence/saves.ts`, with a `meta` store for the lists). The worker writes the save, so the frame never stalls. |

Performance budget: a sim tick under **1 ms for 8 residents** and under **3 ms for 50 residents** (neighbourhood scale later). WASM SIMD (`+simd128`) on: the wasm crates refuse to build for wasm32 without it, since a `RUSTFLAGS` in the environment would replace `.cargo/config.toml`'s. `lto = "fat"`, `opt-level = 3`, `panic = "abort"`. Since 0.2.0 production builds run binaryen's `wasm-opt -O3` and strip the names (sim wasm 2.85 → 2.25 MB, 835 → 796 KB gzipped; no change in speed; 4.03 MB, 1.42 MB gzipped by 2026-11-14), never with `-tnh`, as every panic is a trap. The release review measured a speed-5 step (60 ticks) of the 64-resident town at 0.86 ms on average in Chrome on an M3 Pro, worst 9–17 ms. At 60 Hz (0.7.0) a speed-5 step has 16.7 ms: a Big world's measured 4.2–7.2 ms on average natively on the M3 Pro over six game hours from 08:00 (2026-11-14, seed base 7: Snowdale the slowest, worst 1 % of steps 32–62 ms; see Measured hot spots), after the planning redone every tick for what hadn't changed was kept (`taxi` kerbs, `life::firm_leave`).

---

## Rendering and look

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

**As built (0.3.0)**, where it differs from the plan above:
- **Shadows:** one sun shadow map over a fixed area round the lot, 1024, 2048 or 4096 by preset. It draws only the casters that can reach that area (about 150 shadow draws by day in the village, from 242), and only when something in it changed: the light, the casters, a thing placed or moved, roofs and storeys, or a resident's pose. Paused, in Build mode and behind menus it isn't drawn at all; in live play, with residents in view, it still is every frame, as they breathe. On Low (0.3.0) the residents, the street and the woods cast no sun shadows, and the lot's furniture casts from one merged mesh: one shadow draw instead of about a hundred.
- **Lamps:** clustered, up to 32, on Medium and High; the four nearest the biggest rooms on Low. They are spot lights: a shade sends its light down, up, out from its wall or off the ceiling (content `light.aim`), the rest light all round. The lamps don't light the landscape; the garden's lamps cast pools of light on the lawn instead (`gardenPools.ts`), as the street's do. On Medium and High a clustered lamp lights only its own room (`roomLights.ts`: a room mask per tile; about 0.1 ms on Medium); Low's four forward lamps aren't masked.
- **Effects:** contact shading on Medium and High (screen-space occlusion in the lighting; 0.2.0 had removed ambient occlusion), after 0.7.0; no bloom (tried, and it showed too little) and no tilt-shift (a miniature focus was tried after 0.7.0 and taken out). There is ACES tone mapping, a vignette and a night grade.
- **No GPU culling:** there are no compute shaders or `drawIndirect`. Culling is the engine's frustum culling on the CPU, and build effects use CPU particles (`render/shared/buildFx.ts`).
- **Snapshot rendering** replays only the menus' town; the game draws live.
- **Residents** are skinned on the GPU from a pose texture, with a low-detail copy for far residents and shadows. Only the viewed lot's residents are drawn, so no vertex-animation textures were needed.
- **The draw count is higher than planned:** P1 measured 493 draw calls a frame on Medium in the village scene, 242 of them for the shadow map.
- **Resolution:** Auto draws native pixels up to about 1920×1080 and doesn't step down when frames run slow (the owner put that off for 0.2.0). A Custom resolution draws 50–100 %.
- **Presets:** the three graphics presets and their costs are in `web/src/render/quality.ts`.

**No hardware target** (the owner, 2026-10-15): there is no frame-rate goal for particular machines. Low is for older laptops, best effort.

*(The plan's first targets were 60 fps at 1080p on integrated GPUs such as Apple M1 and Intel Iris Xe, with about 300 draw calls or fewer a frame and main-thread CPU at 6 ms or less, and 120 fps on discrete GPUs.)*

---

## UI (Svelte 5 + CSS)

- Svelte 5 runes; plain CSS with design tokens (colour, radius, blur, motion) and **no UI framework**.
- Visual language: frosted glass panels (`backdrop-filter`), soft shadows, rounded geometry, one accent colour per mode (Live / Build / Buy), spring-based micro-animations, variable fonts.
- Panels: resident portrait plus mood ring, needs (circular gauges), interaction pie menu (radial, at the cursor), action queue, clock and speed control, build/buy catalog with 3D thumbnails rendered once and cached.
- Accessibility: keyboard shortcuts, reduced-motion support, scalable UI.
- Performance rules: no per-frame reactive updates; the catalog uses virtual scrolling; thumbnails are lazy-loaded.

---

## Assets (replaceable by design)

- **Everything is referenced by key.** Content and code ask for `model.fridge`, `icon.need.hunger`, `material.wall` or `look.coral`, never for a file path.
- **`web/public/assets/manifest.json`** maps keys to files (`url`). Entry types: `model`, `material`, `icon`, `image`, `sprite`, `look`.
- **Override packs:** manifests listed in `packs` (or passed as `?pack=…`) override keys, so a new art set is a folder plus one line, with no code changes.
- **Graceful fallback:** a missing model, or one whose file fails, is drawn as a plain box in its place (placed objects, so they can still be clicked, buy mode's copy in hand, the town's and public places' things, food and carried things, catalog pictures; the landscape and foliage are left out), a failed file is tried again in the background and drawn as soon as it comes (the landscape's plants and the selection marker too, built again on their own, without the world), and a missing icon is a neutral glyph, so broken art never breaks the game.
- Format for real art: **glTF 2.0**. Every object's model is generated in Rust (`crates/art` on the kernel `crates/artkit`, `cargo art`, checked by `cargo check-glb`; the standard is [docs/design/objects.md](docs/design/objects.md), the guide [crates/art/README.md](crates/art/README.md); built in Blender from 0.3.0 until 0.8.3), its materials mapped to the game's finishes. The loader bakes transforms so thin instancing keeps working.
- **Compression is automatic (0.19):** production builds run `cargo assets` (`crates/tools/src/assets.rs`, in JavaScript until 2026-11: KTX2 ETC1S textures, quantised meshopt geometry, gzipped character data; cached in `web/.assets`, index `assets/optimized.json`), and loaders fall back to the source file if a copy fails. Sources stay editable; the dev server serves them, and `LL_ASSETS=source` builds without copies. The texture decoder is self-hosted in `public/decoders/` (`bun run decoders` after a PlayCanvas upgrade), because cross-origin isolation blocks a CDN. Packs outside `public/assets` load as they are.
- Library: CC0 textures and sky lighting (Poly Haven, ambientCG), models generated in Rust for the game (`cargo art`), and residents generated in code (`cargo characters`): a **modular character system** (body + head + hair + outfits on one skeleton) for the household creator. No downloaded models are used (`web/public/assets/CREDITS.md`).

---

## Repository layout (as built)

```
idyll-lives/
├─ crates/
│  ├─ art/             # the item models: cargo art and the families (art-nature, art-wet, art-rooms, art-packs, art-outside, art-things)
│  ├─ artkit/          # the models' kernel: shapes, forms, parts, clips, the kit, botany, shading, the .glb writer
│  ├─ sim-core/        # pure Rust simulation, no browser deps, native tests
│  ├─ sim-wasm/        # thin wasm-bindgen layer
│  ├─ tools/           # ll-tools: the Rust tools behind the cargo aliases (cargo art, check-glb, characters, …)
│  ├─ voice/           # resident voices: phonemizer (mouth shapes), voice shaping (PSOLA), resampling
│  └─ voice-wasm/      # thin wasm-bindgen layer for the voice worker
├─ web/
│  ├─ public/content/  # gameplay data: needs, objects, careers, houses, voice lines (JSON)
│  ├─ public/assets/   # asset manifest, models, textures, icons, packs, CREDITS.md
│  └─ src/
│     ├─ core/         # sim worker, bridge, protocol, shared-memory snapshot
│     ├─ render/       # Renderer interface; pc/ (PlayCanvas), shared/ (engine-neutral), preview/
│     ├─ assets/       # manifest schema, registry, streaming
│     ├─ content/      # gameplay content loader (for UI labels)
│     ├─ game/         # composition root, input, build mode, generated houses
│     ├─ audio/        # mixer and the sound of the world
│     ├─ voice/        # voice worker, director, voices per resident
│     ├─ persistence/  # saves and blueprints (IndexedDB)
│     ├─ settings/     # player settings
│     ├─ debug/        # debug reports
│     └─ ui/           # Svelte components, design tokens, UI state
├─ tools/              # asset, voice, house-generation, benchmark and release scripts; the models' gate files (tools/art)
├─ docs/               # content packs, design notes, personalities
└─ PLAN.md, README.md, CHANGELOG.md
```

The snapshot memory layout is **defined once in Rust** (`sim-core/src/snapshot.rs`) and read by TypeScript at startup, so the two sides can't drift apart.

---
### Names

- Characters are called **residents** in everything a player or pack author reads.
- The temporary mood modifiers are called **feelings**. Content packs and saves written with the
  old key names still load (serde aliases in sim-core, `LEGACY_KEYS` in both content mergers).
- Kept on purpose: the repository folder, the crate names `sim-core` / `sim-wasm` ("sim" for
  simulation), code identifiers such as `sim` and `SimView`, and the browser storage keys
  (the IndexedDB database, settings, blueprints, the voice speed test and the voice and
  item-picture caches keep the names of earlier versions) so existing saves, settings, blueprints
  and downloads survive.
- Credits and the README keep a short nominative disclaimer naming EA's trademark.
