# Rendering benchmark

The game has a rendering benchmark built in: open it with `?bench=<scene>` (`web/src/debug/bench.ts`, flags in `benchFlags.ts`). These scripts run it over a matrix in a real browser and make its scenes.

| Script | Does |
|---|---|
| `render.mjs` | Starts Vite (or uses `--url`), opens `?bench=` in Chromium on the real GPU at 1920×1080 and device pixel ratio 1, once per preset × backend × scenario × CPU slow-down × diagnostic, and writes JSON and a markdown table to `bench-results/<time>/` (medians with `--runs`). `--shots <dir>` takes reference screenshots of 9 fixed poses (`--pages` the dev pages', held still: the gallery, clips, residents, poses and previews); `--compare <dirA> <dirB>` counts the pixels that differ (`cargo shots compare`). `looks.mjs` shoots the pages (and poses) once in the soft look and once in classic PBR and tiles each pair into one sheet (`--pair <dirA> <dirB>` tiles any two shot folders: before and after). `play.mjs` shoots the play set (`--poses play`: the home lot from the play camera by day with the sun behind and ahead, at dusk and by night, the same with the walls up, from 80 m, from across its street, and the menus' town) on medium/webgpu and low/webgl, the pictures a change to the look is judged by; `play.mjs --poses day` shoots the day set (the play camera over three homes of the scene in the morning, at midday and in the late afternoon, and the home with its walls up: where a change to the daytime look has to show); `--poses close` the close set (the play camera zoomed all the way in, 6 m, over the home from two sides by day, at dusk and by night, over a fuller house, and the garden by night from 12 m: where a finish's texture, an edge and a soft thing's shape show; the pages `gallery-soft`, `gallery-drape`, `gallery-wood`, `gallery-plain`, `gallery-shrubs`, `gallery-conifers` and `previews-catalogue` are the same kinds of thing on their own and as Buy mode pictures them); `cargo shots stats <dirA> [<dirB>]` (crates/tools/src/shots.rs) measures shot folders (brightness percentiles, clipping, saturation, how much of the frame is one tone, contrast at four sizes), and `cargo shots steady <dirA> <dirB>` how much pictures change when the camera moves a quarter of a pixel (`--extra 'tx=0.004&tz=0.004'`), which is what shimmers in play. `play.mjs --sheet <group>` shoots one build in the variants of `play-variants.json` (each a `?tune=…`, dev server only: `web/src/render/tune.ts`) and tiles them side by side under their names. `--jobs <file.json>` runs a list of command lines in one invocation (see below). |
| `gameplay.mjs` | The end-to-end gameplay baseline: from process start (it starts its own Vite, `--port 5222`, or uses `--url`) it launches Chromium, goes boot → New game → the streets laid out → the town built → a household's Play → step 5's Move in, then plays at 1× for `--secs` (default 150) and shuts down cleanly. Every second it records CDP `Performance.getMetrics` deltas (main-thread task/script/layout busy %), the browser process tree's RSS, CPU % and GPU-process CPU % (`ps`), the page's JS heap and DOM nodes, and the renderer's own `stats()` through the dev-only `?gpustats` hook (`window.__gameStats()`: GPU ms, draw calls, renderer frame ms); every animation frame's gap is kept raw. CDP network events give every request's bytes, cache and timing, and the resource timing entries the asset bytes per stage. The world is pinned (`--code`, default `country-lrstm-7`: the countryside as a new player gets it at seed 7; or `--world` and `--options`), and so is the kind of home of the household played (`--home house`, `flat` or `any`). At fixed seconds of play (`--reads 30,70,110,140`) it makes a collected read: the game paused, the page's and every worker's heap collected and a critical memory-pressure notification sent, then the renderer's and the GPU process's `phys_footprint`, a memory-infra dump of their allocators, the sim's WASM, the renderer's census (`__memCensus`) and the world's plots, households, residents and objects; the game plays on after, and the read's time is left out of the window. Writes `raw.jsonl`, `summary.json` (with each key's median and spread over the reads, `memory.collected`) and `report.md` (stage timeline, per-metric min/mean/median/p95/p99/max, memory over time, network/disk, run 1 vs run 2 stability) to `bench-results/gameplay-<time>/`. A stage timeout, a page error or a short steady window fails the run with a non-zero exit. |
| `keep.mjs` | Keeps a bench page per build loaded in one browser for scripts to attach to (`render.mjs --attach`): see "Keep it loaded" below. |
| `soak.sh` | A 30-minute memory soak: runs `gameplay.mjs` once with `--secs 1800` and collected reads at 60, 600, 1200 and 1740 s (flags given win: `--secs 150 --reads 30,140` is a smoke run), so the per-second JS heap and `simMemoryBytes` series and the collected reads land in `bench-results/gameplay-<time>/`. Extra flags are passed through. |
| `memory-ab.mjs` | The memory of two builds against each other: `gameplay.mjs` on build A and build B in turn (A B, B A, …) `--runs` times each (default 3), each a tree served by its own Vite (`--prod`: its production build for the benchmark, `vite preview`) or a dev server's URL, both measured by this tree's harness in the same world and kind of home. Prints and writes (`compare.md`, `compare.json` in `bench-results/memory-ab-<time>/`) each key's median over the runs (a run's number is the median of its collected reads), least and most, and B − A: the footprints, each process's allocators, the sim's WASM, the workers' heaps, the census and the counts. See "Memory, two builds" below. |
| `scene.mjs` | Makes a scene, `web/public/bench/<name>.json.gz`: a new game as the menus make one (in the place `--world`, default the countryside, with `--options town=village,…`), with a household of four in the first home for sale, played at full speed to `--hour` on day 1, paused and saved. |
| `state.mjs` | Makes a game state without a browser: the new game a world's code starts (`--code`, the five steps as the screens run them, Play on an offered household by `--home house\|flat\|any` and `--household`), scene.mjs's recipe (`--scene --world … --options … --seed … --home …`) or a save (`--from`), run by the sim's wasm under Bun to `--hour`, `--at d:hh:mm`, `--tick` or `--ticks`, saved paused. Kept in `bench-results/.states/` by what made it (the town file, the sim's wasm, the dice, the watch, the moment); `--name` copies it to `debug-states/` (`?state=`), `--bench` also to `web/public/bench/` (`?bench=`). See "Bench time" below. |
| `steps.mjs` | Counts a state in fixed steps: one page, `__bench.load` and `run` per pose (`--poses home:24,centre:60`) and backend, `--frames` frames of `--ticks` ticks, each pose warmed up once first; `--runs 2` proves the counts repeat, `--shots` takes each run's last frame. See "Bench time" below. |
| `profile.mjs` | Profiles the **real game** instead of the bench scene: opens `?state=<name>&profile&debug&quality=<preset>` on the dev server (only there: a production build opens no `?state=`), presses `--speed`, records its own animation-frame gaps for `--secs`, then reads `window.__perfReport()`. Writes `frames.json`, `perf.json` and `report.md` (gap p50/p95/p99/max, counts over 20/33/50/100 ms, dropped frames and long tasks by phase, worst frames, slowest spans) to `bench-results/profile-<time>/`. A server already answering on `--port` is reused. |
| `loading.mjs` | Times what a new player waits for, through the menus as a player goes, in a new browser context per run: the page to the main menu and its picture (`boot`), New game to step 1's land made and built (`world`; the countryside as a new player gets it, or `--world <place> --options town=city,villages=many`, or `--code <world code>`), "Lay out the streets" to step 2's layout proposed and shown (`layout`), "Build the town" to step 3's town built and shown (`town`), step 4 opened to the portraits in view and to every card's portrait shown (`inView`, `portraits`), and step 5's Move in to the game up (`entry`; step 5 reached by Play on a household's card, `--household <n>`, or through Creation, `--create`). The page's `Math.random` is seeded (`--seed`), so step 1's world and its households are the same in every run and every build measured; `--again` opens the page a second time in the same context (what the browser kept between visits); `--back` then goes back to the menu and in again (Save and quit to the menu's picture on screen, Continue to the game up, the heap in that game, Quit without saving to the menu's picture again, and the long tasks after each). Medians of `--runs`, and each run's `?profile` spans (the dev play log's `perf` lines), to `bench-results/loading-<time>/`. |
| `menu-still.mjs` | Takes the main menu's picture shipped with the game (`web/public/menu/still.jpg`, a first visit's menu): a world made as step 1 makes one (`--world country --seed 2098028765`, Elmview; `--options town=city,…` for other options than the place's own), taken as the menu takes its own, at 1920 × 1080 once every model and texture has come. Prints the town's name (`FIRST_STILL.name` in `web/src/game/menuStill.ts`). |
| `roadShots.mjs` | The road hierarchy's fixture (`?bench=bigworld&big=roads`: `web/src/debug/roadFixture.ts`, the motorway, its ramps, an avenue and a one-way loop set in a flat countryside) from about 40 m at the overpass, a ramp merging, the motorway, its bridge, the avenue's junctions, the one-way loop and a ramp's T, then whole from 400 m and 783 m; each picture's draws (`__bench.counts`) to `counts.json` beside them. `--renderers`, `--quality`, `--only`, `--follow` (on a land the town follows: the built surface's decks), `--port`/`--url`, to `--out` (default `bench-results/road-shots/`). `--big <place> --options town:city --spots <file>`: a generated Big world's motorway instead (its junction and overpass near, the junction from 250 m, the town from 783 m), where `tools/worlds/map.ts --seed 7 --spots <file>` found them. |
| `zoneShots.mjs` | Step 2's painted zones (`menuScene.setUses`, `render/pc/useOverlay.ts`) and the free camera on the menus' town, without going through the screens: a generated town shown and painted as its own lots and buildings are, the zoned cells more than 36 m from a road hatched, shot from the whole world, the main town and close up; then the time a brush stamp's change takes against the whole layer drawn again; then the camera tilted low, come in to 10 m (`OVERVIEW_CLOSE`), turned and panned, each shot. `--renderers`, `--size`, `--world`/`--options`, `--seed`, `--port`/`--url`, to `--out` (default `/tmp/zone-shots/`). |
| `create.mjs` | Pictures of a new game's steps 1 (Land), 2 (Streets & zones), 3 (Build) and 4 (Household) as a player gets there, through the main menu's New game: at 2380 × 1546, 1920 × 1080 and 740 × 1000 (`--sizes`), on WebGPU and WebGL (`--renderers`), the narrowest also with its panel scrolled to the end; in step 1 then an arrow key in the place grid and the making panel while that place's land is made; then "Lay out the streets" and step 2's proposed layout the same way, its making panel too; then "Build the town" and step 3's built town the same way, the main town's high street close (its compact shops, most with flats over them), and "Build again" and its high street close from two sides; then "Choose a household" and step 4's cards the same way once their portraits in view are drawn, then a card chosen and "Show others" (it says how many cards and portraits, and how long they took). `--shots 4` shoots step 4 alone (the steps before only gone through). `--shots …,layout` adds step 2's session (`--layout zones,streets`: a stroke of homes beside a street as one undo step, zones far from a street hatched, Erase, Clear zones; a street drawn showing as a road, a refused bend); `--shots …,edit` step 3's (`--edit lots,buildup,villages`: lots (a building of flats put down with the Lot tool among them, close), build-up, a civic place, villages), then back to step 2 and on again with the tweaks kept. The stream pill is shown throughout; it says if the side panel, the buttons or the figures meet the pill or a lot tag, if the arrow key chose the wrong place, if a step doesn't say it is that step of 5, if step 4 offers fewer than 10 or more than 30 households, or if "Show others" offers ones shown already. `Math.random` seeded (`--seed`); `--world`, `--options`, `--code` as `loading.mjs`. To `bench-results/create-<time>/` (`--out`). |
| `room.mjs` | The furnished living room from the play camera, where a change to a room's look is judged: the `review-stage` state (below) furnished as a living room (a sofa, an armchair, a rug, tables, lamps, a bookshelf, a plant, a TV bench: `ROOM` in the script), shot at 9.5 m and at 6 m, by day (11:00) and at dusk (19:30, the lamps on; `--hours 11,16,19.5` adds the afternoon, when the sun comes through the window), on medium/webgpu and low/webgl, each picture cut to the furniture (`--clip`). `--sheet <group>` shoots it in the variants of `play-variants.json`'s group and tiles them side by side under their names, as `play.mjs --sheet` does for the bench scene; `--gpu <secs>` measures the GPU's time per frame there instead (`?gpustats`), the variants in turn and the round repeated (`--runs`); `--floor <id>` lays the room's floor in one of content's floor coverings first (`wood.honey`); `--nudge <m>` moves the camera's target, for the pair of pictures `cargo shots steady` compares. Its own Vite on `--port` (`--strictPort`), or `--url`. |
| `townShots.mjs` | The new game's camera shots and lot tints on the menus' town, without going through the screens: a bare land, then the town on it from the `world` and `area` shots, the lots for sale lit up (the dearer ones in their own tint) and a pan past the land's edge, on each renderer; then the time the town view takes to build again after a change to the draft (a lot removed, a street added, a home chosen). Pictures to `/tmp/ng-render/` (`--out`); exits non-zero on a fault. |
| `land.mjs` | The land brushes as a player uses them (`window.__land`): step 1's land shaped (a plateau, a pond, a river, woods, a peak, the brush's 1 m and 120 m ends), each stroke's frame times and pictures before and after, then the streets, the town, a household played and Move in (the pond still water to the sim), Build mode's Land tab (paid, undone, a bridge), and saved and continued with the land kept. Writes to `bench-results/land-<time>/`; exits non-zero on a fault. |
| `simtick.mjs` | The sim's speed without a browser: the wasm build ticking a world of `tools/worlds/sample.ts`'s under Node or Bun step by step as the worker does (10×, speed 3), and one line of ticks a second, the slowest hour, a step's p99 and ticks a second of CPU time. |
| `tiers.sh` | The sim's speed on the three city worlds at each fill tier over a game day: the big-world benchmark (`crates/sim-core/tests/big_world.rs`) built per label (or a binary given), best of `ROUNDS`, and a table to `bench-results/tiers/`. |

Both need `bun run wasm` and `bun run sound` first, and Playwright (`PLAYWRIGHT=<path to playwright/index.mjs>` if it isn't installed, `CHROME=<executable>` for a browser of your own: Chrome or Chrome for Testing, not `chrome-headless-shell`, which draws in software). Measurements run in a window, so their timing is the real display's. Shots and `scene.mjs`, which time nothing, run headless: no window takes the focus, and it's still the real GPU. `--headless` and `--headed` choose for either. A window's shots come out about 2 of 255 brighter than headless ones (the screen's colour profile), so compare shots taken the same way: two headless runs of the same build differ in at most 0.12 % of their pixels. Both play the game at 1 % volume (`quiet.mjs`).

**The town view** (the play camera zoomed out past 120 m, to 900 m). `render.mjs --shots <dir> --poses town` shoots it over the main town's centre and over the home from 150 m (just into the town view), 400 m (the full town view) and 900 m (as far out as it goes), at 13 h, at dusk (19.5 h) and at 23 h: 18 pictures, `town-<centre|home>-<150|400|900>-<day|dusk|night>`, each `cam=over&over=…&radius=…&hour=…`. They are the game as a player has it there: a Big town with its residents going about that hour, a household of the player's, and the interface with the town tags (`hud=game`: the game's interface without the benchmark's summary). Each is taken in a scene saved at its hour, `town-day`, `town-dusk` and `town-night`, which are not in git (half a megabyte each, and they age with the save format); make them once, and again after a change to the simulation or the generator:

```sh
bun tools/bench/scene.mjs --name town-day --world country --options town=village --seed 7 --home town --hour 13
bun tools/bench/scene.mjs --name town-dusk --world country --options town=village --seed 7 --home town --hour 19.5
bun tools/bench/scene.mjs --name town-night --world country --options town=village --seed 7 --home town --hour 23
```

(`--seed`: the same town, neighbours and household each time; `--home town`: a home for sale in the main town; `--home flat`: a flat for sale, so the game starts in a flat.) With `--scene bigworld` the set is the drawing alone, as every other set is: the interface hidden, a Big town with nobody in it (`web/src/debug/bigTown.ts`: the game plays the village while the renderer draws it; the countryside unless `--world <place>` and `--options` say otherwise), where a lot with a house three quarters of the way out from the main town's middle stands in for the home. `--scenarios town-zoom` times the zoom itself (`cam=zoom`): from the home at the game's opening 24 m out to 900 m and back in, once in 60 s, the distance changing by the same share each second, in `--scene bigworld` for 60 s unless the command line gives a scene or `--secs`. The camera first stays at 24 m over the place for 2.5 s, so the lot there is the focus (drawn in full), as the home is when a player zooms out of it: from the town view no lot becomes one. The camera's poses are the game's own: its tilt is held in the town view's range, its target over land (`web/src/render/shared/townCamera.ts`), so `over=home` at 900 m is held nearer the world's middle than the home is.

**One page, many poses.** A bench page (`?bench=<scene>`) answers `window.__bench` once `__benchReady` is set (`web/src/debug/bench.ts` `BenchApi`; a `?debug` game page, `?state=…&debug`, has it too, its camera the player's until the first `pose`): `pose({cam, at, radius, beta, alpha, tx, tz, over, walls, townCentre})` holds the camera there (a whole pose: what it leaves out is the URL's default), `hour(h | null)` lights it at that hour at once (the sun's map told, the birds settled again), `freeze(on)` stops time or lets it go on, `settle({limitMs})` waits until the picture is ready (the town view's 2.5 s near over its spot whenever its pose changes, so the lot there is the focus as on a page opened there; the files and the street streamed in; nothing building; the textures; the birds; 1.6 s after a new hour or a town view come out to, for the windows that follow who is up; three frames), `counts(frames)` sums the frame probe's draws and triangles over the next frames (means, the median frame's draws, and the sun's map's redraws and why: `sunShadowWhy`), `walls(mode)`, `view()`, `release()` (the camera the player's again), `reset()` (the page's own pose, hour and freeze) and `idle(on)` (nothing drawn). `render.mjs --shots` (so `looks.mjs` and `play.mjs`) opens one page per scene × preset × backend × diagnostic and steps it through its poses and hours, the town view's poses over the home first (the game opens on the home, so it is the focus as on a page opened there, and the camera then goes once to the centre, as a page opened there does; the other way round the lots round the home come back dressed otherwise, 0.2 % of the picture); the menus' town and Build mode keep a page each, and `--reload-per-pose` opens a page for every picture as before. On 2026-11-20 (M3 Pro, WebGL, Medium, 1920 × 1080, a loaded machine) the town set of `--scene town-day` (18 poses) took 81 s on one page against 503 s reloaded, 55 s attached to a page `keep.mjs` keeps (its load, 26 s, paid once), and the play set 67 s against 268 s. The pictures were the reloaded ones to the pixel but for a few: in the town set at most 9 pixels a picture (in 5 of 10 compared), on WebGPU the same, and in the play set up to 101 pixels in 5 of 12 (a bird at night, a few far trees); two runs of the town scene's own sets differ by up to 0.09 % from each other (its cars stand elsewhere: the frozen moment differs run to run).

**Bench time: states and fixed steps.** To count what a moment of the game draws without waiting for it, and the same every time:

```sh
# The town-day moment, made without a browser (about 50 s the first time, at once from then on)
bun tools/bench/state.mjs --scene --world country --options town=town --seed 7 --home town --hour 13 --name town-day
# Or a world's code at a moment of its first day, its household played as gameplay.mjs plays it
bun tools/bench/state.mjs --code country-lrstm-7 --home house --at 1:12:55 --name lrstm-1255
# Its counts over the home and the centre, 600 frames a tick apart, twice each, on both backends
bun tools/bench/steps.mjs --state town-day --poses home:24,home:60,centre:24,centre:60 --runs 2 --shots
```

In a page (`?bench=…` or the dev server's `?debug`): `await __bench.load('town-day')` swaps the game for that state as Continue opens a save (no page load: 6 to 8 s on 2026-11-20 against about 15 s for the first game a page opens), `__bench.pose({cam: 'over', over: 'home', radius: 24})`, then `await __bench.run(600)` draws exactly 600 frames, each after exactly one tick (`{ticks: n}`: n), as fast as the machine draws them (600 in about 5 s), and gives each frame's draws, sun-map draws and triangles, the frames whose sun picture of the still casters was drawn again and why (`sunShadowWhy`), and a fingerprint (`hash`). `__bench.step(n)` holds the page in fixed steps, `step(null)` lets it go, `__bench.state()` saves the moment as it is. On a `?debug` page `__bench` is up from the main menu on, and `run(frames, {pose: {over: 'home' | 'centre', radius}})` lets the bench's camera go and holds the game's over the home or the centre itself (what `steps.mjs` does). What makes it repeat: the sim's worker runs ticks only as frames ask (`SimBridge.pace`, `advance`), a frame shows the newest snapshot as it is, and what the renderer draws at a rate of its own (the sun's and the lamps' maps, the reflections, the lit windows) goes by the frame's time, not the wall clock's; before the first counted frame the camera is settled with the game paused and the clock held, the residents and the birds are set up afresh and the clocks of what moves only for the eye (clips, wind, clouds) start from 0. A frame that asks for a model not built yet (a prop first held) counts in `lateBuilds`: the first run of a pose in a page is a warm-up (`steps.mjs` does one). On 2026-11-20 two runs from the same state gave the same counts frame for frame, and WebGL the same as WebGPU (the town-day moment: over the home 548.2 draws a frame at 24 m, 695.7 at 60 m; over the centre 556.6 and 746.6); the last frames' pictures were the same to the pixel over the centre and differed only in the garden birds' poses over the home (73 to 230 pixels, 0.01 %).

A state made by `state.mjs` has the town and the households the page makes from the same code: its generator runs under Node, V8 as in Chrome (under Bun's engine the same code made a town with other lots). Against scene.mjs's town-day scene saved at tick 22,217 (2026-11-20), the state made for that tick had every lot (559) and the 421 households it started with the same, in the same homes; the residents stood where the page's did for 254 of 696 (the sim moves those out of full detail coarsely round what the camera streams, and scene.mjs's page played in real time at its speeds and its camera's watch), and the households that moved in during the day differed. So a state stands in for a scene of the same moment, not for the same frames: count a state against a state.

Use states and fixed steps for what is counted or pictured (draws, triangles, the sun map's redraws, pictures before and after a change, A against B in one build); the real-time bench and `gameplay.mjs` for what the wall clock measures (frame times, the GPU's time, loading, memory, how the sim keeps up). Against 0.8.3 a state can't be shared (it can't read save 42, nor play in fixed steps): each build makes its own state from the same code at the same tick with its own `state.mjs`, and the base runs in real time. A camera moving with the wall clock (the bench's `orbit`, `roam`, `zoom` paths) doesn't repeat in fixed steps: hold it (`cam=over`, `hold=1`).

Every script launches Chromium with `lib/browser.mjs`'s `launchArgs` (since 2026-11-25 `create`, `land`, `loading`, `profile`, `scene`, `menu-still`, `room` and `gameplay` too, which had their own): WebGPU with its developer features (full-precision GPU timestamps) and, on macOS, ANGLE on Metal, Chromium's default there, so their results from before should compare with later ones (not measured: if an A/B against an older run looks off, set the flag back in that script).

A script of one's own (`lib/browser.mjs`; no reload per pose, no 15–45 s sleeps; builds measured one after another, not side by side, as each wants every core):

```js
import { launch, openBench, serve, step } from './tools/bench/lib/browser.mjs';
const { url, stop } = await serve({ port: 5301 });                 // or serve({ mode: 'preview', outDir }) of lib/build.mjs; serveBuild(tree, { prod }) for another tree's
const { context } = await launch({ profileDir: 'bench-results/.browser/mine' });
const { page, done } = await openBench(context, url, 'bench=town-day&quality=medium&renderer=webgl&res=1&hud=0&secs=0&warmup=6&diag=freeze');
await step(page, [{ name: 'c400', query: 'cam=over&over=centre&radius=400&hour=13' }, { name: 'c900', query: 'cam=over&over=centre&radius=900&hour=13' }], async (page, p) => {
  await page.evaluate(() => window.__bench.freeze(false));          // counts at 1×, then frozen again for the next pose
  console.log(p.name, await page.evaluate(() => window.__bench.counts(600)));
  await page.evaluate(() => window.__bench.freeze(true));
});
await done(); await context.close(); stop();
```

**Production bench build.** `--prod` (render.mjs, gameplay.mjs, memory-ab.mjs) measures a production build made for the benchmark (`tools/bench/lib/build.mjs`): `VITE_BENCH=1` keeps the hooks a bench needs (`web/src/debug/hooks.ts` `DEBUG_HOOKS`: `__osw`, `__places`, `?tune=`, `?look=`, `__townEditor`, `__land`, `?hud=0`, `?gpustats`; the dev play log, `?state=` and the dev server's `/__debug` endpoints stay with the dev server), copies `public/` without the voice and language models and writes no licence file. It is built once per content into `bench-results/.dist/<key>/` (the key: web/'s git tree, its uncommitted changes and untracked files, the wasm packages and the bench scenes; about 190 MB each), the newest four kept and none older than a week, with the textures as their sources (`LL_ASSETS=source`, so its pictures compare with the dev server's; `--assets prod` for what players get), one build at a time. `bun tools/bench/lib/build.mjs [--tree <dir>]` makes or finds one and prints its folder. A worktree's first build also compiles `ll-tools` for `cargo wasm-opt` (281 s in all on 2026-11-20; one already made 0.1 s). It is never the default: measured against the dev server on 2026-11-20 it gained nothing a bench needs (the town set's pictures 95 s against 99 s, the gameplay run's start 2.1 s against 3.3 s and its entry 17.7 s against 18.5 s, the collected memory the same within a run's spread), so use it only where a check needs what players get (minified code, the release wasm, `--assets prod`'s compressed textures).

**Keep it loaded.** `keep.mjs` starts one browser (headless, its profile kept, DevTools on `--port`, default 9333) and opens a bench page per build (`--build <tree|url>`, several for an A/B; `--prod` their bench builds), the scene loaded and settled, then keeps them, drawing nothing, until Ctrl-C; it writes `bench-results/.keep.json`. `render.mjs --attach` (or `lib/browser.mjs` `attach('keep')` and `openBench(…, { attached: true })`) uses a kept page with the same scene and settings as it is and resets it after (`__bench.reset()`, `idle(true)`): a probe or an A/B pays the game's load once per build per session. Reload a kept page when its code changed (a dev server's page reloads on its own as `web/src` changes: wait for `__benchReady` again) or when a measurement needs a fresh start: load and entry times, memory (`gameplay.mjs`, `memory-ab.mjs` always start a new browser), the first frames' shader compiles.

**A lean load for counting runs.** `diag=nowarm` (`--diags nowarm`, `--extra diag=…` for a measured run) leaves out the load's warm-up of a Big world's lots (a few lots built and drawn so their pipelines are made before play): a bench page was ready about 5 s sooner in `--scene bigworld` (25 and 24 s against 31 and 28) and 2.5 s sooner in `town-day` (20 and 19 s against 23 and 21; 2026-11-20, WebGL, loaded machine). Its first frames compile what the warm-up would have, so leave it out of timings of the first seconds, and its pictures are not the warmed page's (0.01 to 0.07 % of the pixels differ at 400 m): use it for draw counts and probes, not for pictures compared with ones taken without it.

**A room's look.** `room.mjs --sheet <group>` and the `room-…` groups of `play-variants.json` are for the look indoors: `room-shading` (the contact shading's reach and strength, Medium and High only), `room-shadows` (the shadows shaped like what casts them, every preset), `room-light` (how a room takes the sky's light by day), `room-colour` (the floor's and the walls' tone, the grade), `room-floor` (the floorboards' texture) and `room-pick` (the room before 2026-11-18 against the room as it is). Each group's own poses are the bench scene's day poses: `play.mjs --sheet <group> --low` shows a variant out of doors. The variant "as it is" is the game's own values (picked from these sheets on 2026-11-18: contact shading radius 0.5 and strength 1.5, contact shadows' opacity 0.8, a room's daylight a little warm and as bright, oak boards wide, calm and a tone darker); "before 2026-11-18" sets the values the game had. What they tune (`web/src/render/tune.ts`):

| `?tune=` | What |
|---|---|
| `contact.radius`, `.intensity`, `.power`, `.samples`, `.minAngle`, `.scale` | The contact shading's pass (`render/pc/post.ts` `CONTACT`). |
| `contactLight.direct` | The share of it on the lamps' and the sun's light (`render/pc/shaders/softLook.ts`). |
| `contactShadow.peak`, `.half`, `.blur`, `.margin` | The shadow under a thing (`render/shared/contactShadow.ts`). |
| `room.dayAmbient`, `room.dayFill`, `room.dayTint` | By day, a room's share of the sky's ambient and of its fill from above, and the colour both take there (`render/shared/roomLight.ts`; back to the garden's as the lamps come on). |
| `floor.boards` | The boards of a room with no floor laid (`render/shared/house.ts`; content's `wood.oak` covering has the same colour, in `base.json`). |
| `<manifest key>.<field>` | A surface of the asset manifest: `material.floor.wood.color:D8D4CC`, `material.wall.interior.color:C5D3BC`, `material.floor.wood.uvScale:0.36`, `material.floor.wood.texture:../floor_wood.pine/albedo.jpg` (a file named from the entry's own). |

The floorboards' variants are texture sets of their own, made when named and not in git: `cargo tex synth floor_wood.pine floor_wood.calm floor_wood.wide floor_wood.oak` writes `web/public/assets/textures/floor_wood.<variant>/` (`crates/tools/src/tex/synth/built/` `variants`; `pine` is the set as it was before 2026-11-18, byte for byte). Make them before shooting `room-floor`, `room-colour` or `room-pick`, and delete them before a production build (`rm -r web/public/assets/textures/floor_wood.*`): a build ships whatever stands under `web/public`.

**Memory, two builds.** Compare builds' memory by `gameplay.mjs`'s collected reads, never by a footprint read as it stands: one read before collecting and one after, at the same moment, differ by up to 2 GB (the garbage of the way in, and what Chrome's allocators keep while nothing asks for it back), and the renderer's RSS swings between 1 and 3.7 GB over a run. Collected, three runs of this tree at 1× (2026-11-20, the countryside at `country-lrstm-7`, a house, Medium on WebGPU, 300 s) gave a renderer footprint of 2,088, 2,092 and 2,051 MB (each the median of its five reads; its reads 2,023 to 2,332 MB, the one at 30 s always the highest, the way in still being given back), and the same world each time: 486 plots, 40 households, 682 residents, 33,083 objects. The GPU process's footprint moves more (1,529 to 1,639 MB): read it, but judge the renderer's. `memory-ab.mjs` runs two builds in turn; the base in a worktree of its own, its wasm built and `node_modules` linked:

```sh
git worktree add --detach ../base 85936d1d0
ln -s "$(pwd)/web/node_modules" ../base/web/node_modules
(cd ../base/web && bun run wasm && bun run sound)
bun tools/bench/memory-ab.mjs --a ../base --b .
```

A run takes about 4 minutes (the way in, 150 s of play, four reads of 5 to 15 s), so three each (the default) is under half an hour; until 2026-11-20 a run played 300 s with five reads, but a run's collected reads agree within about 3 % from the first (1,912 to 1,969 MB in one run, prod and dev alike), so 150 s and four reads say as much, and runs are what a comparison needs more of (`--secs 300 --reads 30,90,150,210,270` as before); keep the machine quiet meanwhile (the load at each run's start is in `compare.md`). Options after `--` go to `gameplay.mjs` (`-- --renderer webgl --home flat`). A read step that fails or gets no answer in its time (a page call 15 s, the memory dump 60 s) is named in the read's `missed` in `summary.json` and on the console, and what it would have given is left out of the medians. Read the renderer's footprint first, the allocators for where a difference is (`partition_alloc/partitions/array_buffer`: typed arrays, the geometry kept on the CPU; `v8`: the JS heaps; `gpu`: the GPU's buffers seen from the page; `malloc`), and the counts for how much more world there is: two builds can read the same code into different towns.

**One browser, kept.** `render.mjs` launches its browser once per invocation, with a profile under `bench-results/.browser/` (named after the port, so two runs on their own ports don't share one), and opens each page in a window of its own. The HTTP cache and the compiled GPU shaders carry over from page to page and to later invocations, which shortens the loads and shader compiles before a warm-up, not what is measured; the site's own storage is cleared before every page, so each starts as a new player's would. `--fresh` gives every page a new context with nothing kept. `--jobs <file.json>` takes a JSON list of command lines (each an array of the options above) and runs them in one go, sharing one Vite server per mode and one browser per kind:

```json
[
  ["--shots", "bench-results/new/shots", "--headless", "--presets", "low,medium,high", "--backends", "webgpu,webgl"],
  ["--shots", "bench-results/new/pages", "--pages", "gallery-objects,clips", "--diags", "none,albedo,normals", "--headless"],
  ["--compare", "bench-results/base/shots", "bench-results/new/shots", "--out", "bench-results/new/shots-diff"]
]
```

```sh
# The production build (what players get), presets × both backends, no vsync, median of 3
bun tools/bench/render.mjs --preview --build --presets low,medium,high --backends webgpu,webgl --scenarios orbit-day,orbit-night --throughput --runs 3

# What a feature costs: Medium with and without shadows and nature
bun tools/bench/render.mjs --preview --presets medium --diags none,noshadows,nonature --throughput --runs 3

# At 3x CPU slow-down (CDP), standing in for slower laptops
bun tools/bench/render.mjs --preview --throttle 1,3

# Does Medium still look the same? Shots of a change against shots of the base
bun tools/bench/render.mjs --preview --shots /tmp/shots-new --presets medium
bun tools/bench/render.mjs --compare /tmp/shots-base /tmp/shots-new --out /tmp/shots-diff

# A new scene (the bench's village, as made for save format 42)
bun tools/bench/scene.mjs --name village --world country --hour 18 --seed 7

# The real game (a debug state), 90 s at super speed (4: 30×, 60× in quiet hours), Medium
bun tools/bench/profile.mjs --state readme-village --speed 4 --secs 90 --preset medium
```

The options are documented at the top of each script. To compare two builds, measure them back to back on the same machine, with nothing else running, and take medians of at least 3 runs; GPU times on Apple hardware vary by about ±1 ms between runs. A GPU time marked `(overlapped)` is not one: the frame's stamps ran longer than the time between frames (on Metal the town view's bloom chain at night read 62 ms a frame at 95 fps uncapped, 2026-11-27), so judge that run by its frame times and by `diag=` runs against each other. Shots of two runs of the same build taken in a window differed by at most 1 of 255 in a few pixels, apart from a far road's lane markings (P1-17 in the release review); headless ones in at most 0.12 % of their pixels (above).

On a laptop without the repo, open the deployed game instead, e.g. `https://krecks.github.io/idyll-lives/?bench=village&quality=low&hour=13&res=1` (add `&renderer=webgl` for WebGL2), and press "Copy result" when it's done.

# The gameplay baseline (cold, from process start)

`gameplay.mjs` measures what a player actually gets: the menus, the load into a household, and five
minutes of play at 1× speed, with the frame times, CPU, GPU, memory and I/O of that session.

```sh
# Two cold runs of five minutes each (about 12 minutes in all), WebGPU, Medium
bun tools/bench/gameplay.mjs --runs 2 --secs 300

# One warm run: the same browser context and its cache kept from the run before
bun tools/bench/gameplay.mjs --warm

# WebGL2, Low, a fixed seed and household, a shorter window
bun tools/bench/gameplay.mjs --renderer webgl --preset low --seed 7 --household 2 --secs 60
```

The stages are timed from the process starting: the dev server (started here unless one answers on the
port), the browser launch, the main menu (`.main-menu` up, `.loading` gone), step 5's Move in
(`.hud-root` up), then the steady window, which fails unless it covers `--secs` and the samples do too.
Memory is reported per process as well as summed: the renderer (the tab running the game) is the number
to read, the tree sum is a ceiling (shared pages count once per process), and the renderer's
`phys_footprint` at the edges of the window is what Activity Monitor would show. Every renderer process
is counted (`renderers RSS`), because a worker can be a process of its own. Two more numbers come from the
page itself: `GPU memory` (PlayCanvas's own counter for what the device holds, textures and buffers) and
`sim WASM` (the simulation worker's linear memory, as the worker reports it once a second — the main
thread cannot read another thread's memory). `--heapprofile` samples the JS heap through the steady
window and reports the 20 call sites that kept the most (writes `heapprofile.json`). `--help` lists every
option.
The hook it reads, `window.__gameStats()`, exists only under `?gpustats` on a dev build
(`web/src/debug/gameplayStats.ts`): it hands back the renderer's own `stats()` with the GPU timer on, and
is absent from a production build. `--profile` isn't needed for it and the interface stays off
(`--hud 0` hides it) if you want a quieter frame.

# Checking things in the browser

`check.mjs` takes the screenshots an object task needs, headless on the real GPU and at 1 % volume, with
the camera never taking over on its own (the director's delay 0) and free will off (`--free-will` keeps
it). It starts its own Vite from this checkout on the port you give (`--port`), or uses `--url` of a server
you started yourself. Pictures, and `<what>-<run>.json` with what each one shows (and, per job, every HTTP
error, failed request, page error and console error), go to `bench-results/check-<time>/` or `--out <dir>`;
`<run>` is `--name <run>`, else the time the run started, so runs into one `--out` keep their logs.
Before any page it fetches the sim's wasm and the asset manifest through the server and stops unless they
answer 200 (a missing or linked wasm package otherwise gives pictures of an empty scene).

| Check | Does |
|---|---|
| `gallery` | `/gallery.html?keys=…` from the front right. `--alpha --beta --radius --target --cols`; `--each`: one picture per key, `gallery-<key>.png`, on one page; `--scale adult\|child\|all` stands residents beside each model; `--camera game` takes one picture per model with the game's Live camera aimed at it |
| `clips` | `/clips.html?keys=…` held at each of `--times` (default `0,0.3,0.6,1,99`; 99 is open), one picture each. `--each`: each key on its own, `clips-<key>-<clip>-t<t>.png`, one page for all. `--clip --colors --target` |
| `play` | Opens `?state=<--state, default bathroom-check>&debug`, adds money (and `--skills`), then for each of `--use-existing a,b` and `--objects a,b@x,z,rot,…` (or `--objects-file <path>`, below) on the same page: buys its `--with` helpers and it (or takes the state's own), runs each `--use` (`all`, the default, an interaction id, an index, or `ripe` for the uses offered only when a crop is ripe) for `--sim`, takes `--shots` (3) pictures `--every` (1000) ms once the resident is using it, framing the object (or with `--shot-idle` a picture of it as bought first, and `--top` the camera above; `--views` 2–4 takes that many angles per picture), then sells what it bought and the resident walks back to where they started. One JSON line per use in `<out>/play-<run>.jsonl`. |
| `places` | The named places' lots (`?state=<--state, default visitable>&debug`, paused): for one place per layout, or `--places a,b` by name, or `--all`, pictures of it closed from outside (`<layout>-closed-<tag>`, drawn as its kind's building), looked inside (`-inside-`) and, for the hotel, the school and the town hall, upstairs (`-upstairs-`); `--views far` adds it looked inside from the town view at 150 m. Each from its entry side, framed on its walls' box. `--tag` ends every name (default `day`, `night` on a `*-night` state); run once per state and renderer. The state is a scene unzipped into `debug-states/` (`scene.mjs --name visitable --world country --options town=town --seed 7 --home town --hour 13`, then `gunzip -c web/public/bench/visitable.json.gz > debug-states/visitable.json`). `places-<run>.json` lists every place found. |
| `sheet` | Contact sheets of an evidence folder, for reading it a sheet at a time rather than shot by shot: `--dir bench-results/review/<cat>` writes `<dir>/sheets/obj-<id>.png` per object (up to 6 of its pictures, 2 × 2 or 3 × 2: gallery, game camera, a play shot in use, the clip held open, then a close-up, a night shot, other clip framings and the other uses) and `age-<id>.png` (`-2`, `-3`, …) with its `poses.html` shots, and play shots from folders named for a life stage, side by side by stage. Each picture is labelled with its path. Pictures are found by the names `gallery`, `clips`, `play` and the pose shots give them (`gallery-<model>`, `gallery-game-<model>`, `clips-<model>-<clip>-t<t>`, `<id>-<use>-NN`, `poses-…-<stage>-…-<id>`, or ending in one of `poses.html`'s own station labels, `poses-only-child-4-feed_baby`: the crib's), anywhere in the folder, the newest of a name taken twice; the objects from `--ids` or `--ids-file`, else `../lists/<cat>.txt` (or `.models.txt`), else the ids in backticks that open `index.md`'s rows. Composed as a page of `<img>` tiles in the headless browser and screenshotted (no ffmpeg, no server: no `--port`); 1600 wide, about 1000 high for 2 × 2 and 700 for 3 × 2. About 0.2 s a sheet. |

`play` in more detail:
- **Where.** `def@x,z,rot`, else `--at`, else on a `review-*` state (`review-stage`, `-night`,
  `review-garden`) a free spot by the object's layer and footprint (below), else wherever the game
  finds room (things on a surface always go there: the stage's counter).
- **Objects file.** One thing a line, `#` comments:
  `def[@x,z,rot | @existing] [with helper[@x,z,rot] helper…]`, e.g.
  `officeChair with deskSimple@10,13,2`, `cafeChair with cafeTable@10,13,0`,
  `stoolBarHigh with counter@10,13,0` (a stool needs a counter or a bar, not a table; the counter's
  back to it, as a counter facing the stool would have its front tile blocked), `crib@existing`.
- **Helpers.** `--with deskSimple@10,13,2,…` (for every object) and a line's `with …` (added to them):
  bought before the object, where given or wherever the game finds room, kept while its uses run, sold
  after it (a helper that can't be bought fails the object, `helper not bought`). On the stage the
  object stands at `(10 − ⌊w/2⌋, 12 − ⌊d/2⌋, 0)` facing +z, so a table or desk in front of a chair goes
  at z = 13, facing it (rot 2) for a desk. Records name them (`with`). The stage's own kitchen (a
  fridge, a coffee machine, a counter) needs no helpers for cooking or coffee.
- **The state's own.** `--use-existing crib,garden.tomatoes` (or `def@existing` in the file): the first
  object of that kind the state has (the stage's crib with the baby in it, `review-garden`'s ripe
  crops), run before the `--objects`, never sold (records: `existing: true`).
- **Skills.** `--skills cooking=5,creativity=3` sets the resident's skills each time the state opens
  (`Command::SetSkill`, debug only, as `addFunds`; an id the content doesn't have stops the run).
  Records carry them (`skills`).
- **Needs and bonds.** `--needs bladder=0.1,all=0.6` sets needs (`Command::SetNeed`; `all` first, then the
  rest) and `--bond 0:1:friendship=60/romance=40/chemistry=0.5/partners` how two residents stand with each
  other, both ways (`Command::SetBond`; several with commas), each time the state opens, for the residents
  of `--sims`. Both debug only, as `setSkill`. Only grown-ups who aren't family can be set in love or partners
  (the game lets nobody else court): in `review-stage` everyone is family; `crowded-garden`'s Ada and Ben aren't.
- **Two or more residents.** `--sims 0,1` (instead of `--sim`): each does the use, a moment after the one
  before, and the pictures wait until they all are at it (the seesaw, ping-pong, two at a pool table, the
  climbing frame). The first is the one watched and framed (`before`, `after`); the others are in each
  picture's info and the record's `others`, and `together` says whether they all got to it. All go back to
  where they started.
- **Socials and the phone.** `--social chat,joke --sims 0,1`: no object; the first starts each social
  (content `socials` id) with the second, the state opened afresh for each; `--phone call,text --sims 0,1`
  each phone action (`phone.actions`) with the second as the contact (`browse` with nobody). Under way when
  they're talking with the other (the snapshot's `partner`) or the phone's animation plays; then `--shots`
  pictures following the first, the camera on them before it starts. One record each (`social` or `phone`,
  `outcome`, `startedAfter` in s, `with`, `others`); a refusal is `refused: <the game's toast>` at once. Nobody phones someone
  on the same lot ("right here"): in `bathroom-check` resident 1 lives in another house.
- **Waiting.** For every loading screen ("Moving in", "Getting ready", "Drawing portraits", "Building the
  world", "Warming up") before each buy and each picture; `goto` times out at 90 s. A use's cap (`--wait`,
  60 s) counts from when it is sent; a use nobody takes on (nothing in the resident's queue for 4 s: a
  baby's use, one needing a skill or something near) fails at once. The game's toasts meanwhile (why
  a buy or a use was refused: "needs a table", "would hang over the edge") go into the record
  (`toasts`) and a failed buy's error.
- **Speed.** `--walk` (3 = 10×, never more) on the way, `--speed` (1) from 3 tiles off the object once
  the resident is going to it, as short uses are over between frames at 30× (a cook's steps at the
  fridge and the counter go fast too).
- **Logging.** In the page, every animation frame: each clip on the object (`__renderer.objectClips(id)`: its kind,
  length, furthest time, and when it moved, was open and back at rest) and each change in the
  resident's pose, `using`, animation and use. A record holds the object, the use (and its pose and anim),
  where it stands, `outcome` (`using`, `not taken`, `timeout`, `skipped`, `not bought`, `no uses`), when
  they were using it, `played` (clips that moved, not the ambient, growth, night or clock ones), each
  picture's file with the pose, how far they stand from its nearest face (`distance`), the clips and the object's place then, the needs before and after, timings,
  fps and errors.
- **Open things.** A use that needs the object open or closed (`when`) is set up with the use that
  leaves it so (fold the wall bed down first), and anything left open is shut before it is sold. The
  helper uses are records too (`helper: open|close`).
- **Failures.** Each object gets one retry of what failed (afresh from the state unless all were uses not
  taken); then the failure is recorded and the run goes on. A crop's "when ripe" uses are skipped unless
  it is ripe (bought new, it is just planted: `--use-existing garden.vegPatch --use ripe` on
  `review-garden`, whose crops are ripe); asked for by name they run anyway and the game says why not
  ("the vegetable patch isn't ripe yet"). Each record says the crop's stage it found (`growth`: 0
  planted, 1 growing, 2 ripe, 3 picked). The state is opened again when the resident's lowest need is
  below 0.25.
- **Camera.** `__controls.follow(null)`, aimed at the footprint's middle from in front and 0.7 rad to the
  side, set again before each picture; `--alpha --beta --radius --y` override, and `--top` looks down
  from above (who sits where at a table; the game camera stops at 1.35 rad). `--views 2`–`4` takes that
  many angles of each use-loop picture off the aimed one — the front, then round the back (a use from
  behind the thing), above (a seat, a top, what the hands are at) and low (a door's foot, a drawer's
  runner) — named `<object>-v2`… and each record's `angle` says which; the first is unchanged. The radius is the
  larger of what the footprint needs and what the model as drawn needs to be whole in the picture
  inside the HUD (`__check.fit` over its vertices: a tree's crown, a wardrobe, a shower); a model over
  1.6 m is looked at halfway up. Not drawn yet: its height from `__renderer.objectBox`, else a guess.
  The game camera comes no nearer than 6. Records give the view (`view`: the pose, the model's height
  and bounds, and where they came from). The old single object (`--object deskSimple --at 23,14,0
  --use sit`, 6 shots 500 ms apart) still follows the resident, as does `--follow`.
- **Light.** `--hour 12` fixes the renderer's light (the state sets the clock); on a `*-night` state
  there's no default, so the light is the sim's clock's; `--hour state` for that anywhere.
- **On the way.** `--walk-shots <n>` takes up to `n` pictures before the use begins
  (`<def>-<use>-walk-00.png`), `--every` ms apart, stopping as the resident starts using it: the walk
  itself, and a door's swing as their hand takes the handle (`0.45 s`, over by the first picture of a
  use). With `--every 0` they are as fast as the page draws, a frame every ~0.07 s on `review-stage`.
- **Names.** Pictures are `<def>-<use>-<n>.png` (a kind used twice in a run: with its place; its
  `--with` helpers after it, `chair-with-coffeeTable-sit-01.png`); with
  `--name <run>` they start with it, so runs (an adult's, then a child's) into one `--out` keep theirs.
  A picture written over from an earlier run is said once.

Common options: `--quality low|medium|high` (`?quality=`), `--renderer webgl`, `--size 1280x800`,
`--extra 'k=v&k=v'`, `--headed` (a window, only to measure timing).

```sh
bun tools/bench/check.mjs gallery --port 5227 --keys model.dartboard,model.tableClock
bun tools/bench/check.mjs gallery --port 5227 --keys model.fridge,model.diningChair --scale all --camera game
bun tools/bench/check.mjs gallery --port 5227 --each --keys model.fridge,model.toilet,model.bed
bun tools/bench/check.mjs clips --port 5227 --keys model.fridge --clip door_open --times 0,0.2,0.4,99
bun tools/bench/check.mjs clips --port 5227 --each --keys model.fridge,model.toilet,model.bed --times 0,0.5,99
bun tools/bench/check.mjs play --port 5227 --state review-stage --objects toilet,fridge,bed,dartboard@15,12,3 --use all
bun tools/bench/check.mjs play --port 5227 --state review-stage --objects-file /path/to/kitchen.txt
bun tools/bench/check.mjs play --port 5227 --state review-stage --objects officeChair --with deskSimple@10,13,2 --name chair
bun tools/bench/check.mjs play --port 5227 --state review-stage --objects cafeChair --use coffee --with cafeTable@10,13,0
bun tools/bench/check.mjs play --port 5227 --state review-stage --use-existing crib --sim 4 --name crib-elder
bun tools/bench/check.mjs play --port 5227 --state review-stage --objects chefStove --skills cooking=5
bun tools/bench/check.mjs play --port 5227 --state crowded-garden --use-existing seesaw --sims 2,3 --name seesaw   # Dee and Fin (9, 7): a seesaw is a child's
bun tools/bench/check.mjs play --port 5227 --state crowded-garden --social chat,hug,kiss --sims 0,1 --bond 0:1:friendship=80/romance=70/partners
bun tools/bench/check.mjs play --port 5227 --state bathroom-check --phone call,text --sims 0,1 --bond 0:1:friendship=50 --needs all=0.6
bun tools/bench/check.mjs play --port 5227 --state review-garden --use-existing garden.tomatoes,raisedBed --use harvest
bun tools/bench/check.mjs play --port 5227 --state review-garden --use-existing garden.vegPatch --use ripe   # every use offered only when ripe
bun tools/bench/check.mjs play --port 5227 --state review-stage-night --objects telescope@10,22,0,lamp
bun tools/bench/check.mjs play --port 5227 --object toiletClassic --at 23,14,0 --use use --shots 6 --every 500
bun tools/bench/check.mjs play --port 5227 --object toiletClassic --at 24,17,1 --state office-night --renderer webgl --quality low
```

On the test run of 2026-10-21 (headless, Medium, one core) an object took 9–26 s with all its uses (a
use 3–9 s: walking there 3–5 s, then 3 pictures), the state 8 s to open; `gallery --each` and `clips
--each` about 2 s a key.

## Debug pages

Dev-only pages on the dev server (not in the build); each sets `window.__ready` once it's drawn, and the
flags are at the top of its file in `web/src/debug/`. The ones the review uses:

| Page | Flags |
|---|---|
| `/gallery.html` | `keys=model.a,model.b` (or `filter=`), `cols`, `gap`, `labels=0`, `alpha`, `beta`, `radius`, `target`. `scale=adult` stands a man and a woman on each model's own left (the right of the picture, away from the camera), facing the camera (the game's `Characters`, at the idle), `scale=child` a child, `scale=all` a woman, a teen, a child and an elder (at most 64 residents: the first models get them). `camera=game` is the game's Live camera as a game starts (radius 24, beta 0.86, its 0.7 lens) aimed at the floor under the first model, or under `at=<key or index>`; `__gallery.aim(n)` aims it at model n. |
| `/poses.html` | The objects as the game loads them (`base.json` and all its includes, footprints and slots). `only=bed,water` (the stations whose label, object id or tag is one of these, exactly), `focus=<index, label or object id>` (close up; one that isn't there says so on the page and in `__poses.error`, and the page still sets `__ready`), `seats=sofa,chessTable` (one resident a slot, a man, a woman, …, sitting; a futon or wall bed, used only open, open), `seats=<def>:<use or anim>` (doing that use: its pose and clip, the object's clips for it held open, and open if the use opens it: `seats=sofaBed:doze`), `beds=wallBed,sofaBed:doze` (the same, the first use lying down unless one is named), `labels=0` (labels are on the floor in front of each station), `clip=`, `strip=N` (the clip held at N times on a man, a woman, a child, a teen and an elder), `bottom=`, `manual`. `stage=baby\|child\|teen\|youngAdult\|adult\|elder` puts everyone placed at that stage where sim-core lets the stage take the place (a baby only lies in a crib; children and teens don't care for a baby, flirt or kiss), with the game's look for it (an elder's stoop and grey hair); with `strip`, the rows are a man and a woman at that stage. |
| `/characters.html` | `stage=child` or `stage=all`, `hair=all`, `tops=all`, `face=moods`, `clip=walk&t=0.4`, `view=side` (more in `characters.ts`). |
| `/clips.html` | `keys=`, `clip=`, `t=` (99: open), `colors=`, `target=`, `radius=` (see `check.mjs clips`); `still` (no wind, the stage's clock held: `--extra still=1` for pictures of plants that compare pixel for pixel). Models that don't fit the default view at rest or open (a wardrobe, a shower, a tree) are framed whole: aimed at their middle height, from as far as it takes. |

```
/gallery.html?keys=model.fridge,model.diningChair,model.bed&scale=all
/gallery.html?keys=model.fridge,model.diningChair,model.bed&scale=adult&camera=game&at=model.bed
/poses.html?stage=child
/poses.html?strip=8&clip=sitDown
/poses.html?seats=sofa,diningChair&stage=elder
/poses.html?only=bed&stage=child&focus=sleep
/poses.html?beds=wallBed:sleep,sofaBed,bedBunk&stage=elder
/clips.html?keys=model.wardrobe&t=99
```

## States for checks

The simulation's tests write these with `DUMP_STATES=1 cargo test -p sim-core --test sim check_states::`
(`crates/sim-core/tests/sim/check_states.rs`) into `debug-states/` (gitignored); open one with
`?state=<name>&debug` on the dev server. Each test checks its state is still fit for use as content changes.

| State | What's in it |
|---|---|
| `bathroom-check` | The house of the town `wants.rs` makes (seed 21), 10:00 on day 1. Resident 0 at home, doing nothing, no job, free will off, every need at 0.8: nothing pulls them away from a `use`, and a use that fills a need lasts (a full need ends it at once). $1,000,000. Free wall in the living room at `[23, 14, rot]` and `[24, 17, rot]`. |
| `needy-check` | The same with free will on and low needs (bladder 0.05, hunger 0.2, energy 0.25, hygiene 0.3, the rest 0.5): they go to the toilet, then eat, wash and sleep on their own. |
| `office-night` | `bathroom-check` at 21:30: lamps lit, night lighting. |
| `town-things` | A public lot ("Town things", plot 1: `__showPlot(1)`) with every community thing in rows by group (street, playground, stalls, farm; the shops against a wall at the back), all facing the street, beside a home with a small house. Resident 0 stands at the lot's front, free will off, needs at 0.8. |
| `town-street`, `town-street-night` | `town-things` with a street along the front of both lots (`world.meta.streets`, as a generated town's: the road, the pavements and their props), the lots 6 tiles back; `-night` at 22:00, the street lamps and windows lit. |
| `vendors-park`, `vendors-market`, `vendors-cafe`, `vendors-grocer`, `vendors-bakery` | A town place (`houses.json` `parks`) on plot 1 (`__showPlot(1)`) beside an empty home lot, 11:30 on day 1: everyone serving there at work (sim-core `staff.rs`; the hot dog stand from 11:00). Ada (0) and Bo (1) at the home's front, free will off, $500. `-night`: the same at 21:30, everyone gone home and the posts closed (`public_places.rs` `vendor_states_for_play`). |
| `review-stage` | For looking at one thing after another (`play --objects`): an empty 32 × 32 home lot, the street at z = 0, with one walled room from `(4, 6)` to `(16, 18)` (12 × 12 tiles; the door at `x = 5` in the front wall, windows at `z = 14, 15` in the left wall; a crib at `(13, 6)`; along the right wall, facing −x, a counter at `(15, 7)` for things on a top, a fridge at `(15, 8)` and a coffee machine at `(15, 9)`, so cooking at a stove bought and coffee at a table need nothing more) and a garden behind it. One resident per life stage, ids 0–5: adult (40), young adult (22), teen (15), child (8), elder (70), baby (0.5, in the crib); all but the baby in the front yard from `(20.5, 3.5)`. Saturday 10:00, free will off, every need at 0.5 (a use that fills one lasts), $1,000,000. Free spots for a thing of footprint `w × d` (as `play` places them by default, on every `review-*` state; all checked with the kitchen in, 2026-10-21): **floor** in the middle `(10 − ⌊w/2⌋, 12 − ⌊d/2⌋, 0)` (also rugs and ceiling things); **back wall** inside `(10 − ⌊w/2⌋, 18 − d, 2)` (wall things, and floor things that fill a wall or stand against one: fridges, wall beds, wardrobes, a wall-hung toilet); **windows** `(4, 14, 1)` (curtains); **left wall** `(4, 8–12, 1)`; **right wall** `(16 − d, 10–16, 3)`; **front wall** `(7–12, 6, 0)`; **back wall outside** `(4–15, 18, 0)`; **garden** `(10 − ⌊w/2⌋, 22, 0)` (outdoor things, up to 6 × 6; free from z = 19 to 31, and x = 18 to 31 beside the room). The kitchen at `(15, 7–9, 3)` and the crib are outside them all. |
| `review-stage-night` | `review-stage` at 22:00 on the same Saturday (the clock set, not run, so nobody ages): night lighting by the sim's own clock (`play` sets no `--hour` on a `*-night` state). The lamps as the game leaves them: with free will off nobody switches one, and a lamp bought comes on. For stargazing, lamps, night clips. |
| `review-baby` | `review-stage` with a dining table for six (`tableDining4`) at `(8, 12)` and the high chair pulled up to its front at `(8, 13, 2)`, Eve's (5) hunger and bladder at 0.25: Ada's (0) *Feed the baby* at the crib carries Eve to the high chair and feeds her there (by day), *Change the baby* is at the crib's changing table (G-66). |
| `review-garden` | `review-stage` with every crop kind (content `grows`, in content order: the apple tree, the lemon tree, `garden.vegPatch`, `garden.tomatoes`, `garden.strawberries`, `garden.pumpkins`, `raisedBed`, `natureLover.veggieBed`) in a row at the back of the garden, `(2, 29)`, `(5, 29)`, … every 3 tiles, facing the house, all ripe (a crop comes ripe with a lot; one bought is just planted). For their "when ripe" uses: `--use-existing <def>`. A harvested crop lies fallow until the state is opened again. The garden's free spot holds up to 6 × 6 (z 22–27). |
| `crowded-garden` | A 22 × 26 home lot with one walled room at the front (doors front and back) and its back garden from `(5, 14)` to `(15, 22)` packed with garden things at random, every way round (crops, plants, beds, trees, seats, the fire pit, a picnic table, swings: only what buying allowed). Ada (0), Ben (1), Dee (2, child) and Fin (3, child) in the front yard, Saturday 12:00, free will off, needs at 0.5. The first `sunLounger`, at `(10, 16)` facing −z, has both sides shut in and only its foot free (I-garden-01): `--use-existing sunLounger`. |
| `review-resized` | `crowded-garden`'s house and household with things that have moving parts bought resized (`Command::Buy` with `size` and `tall`), each against a wall of the room: a `fridge` at `(4, 10, 2)` 200 × 100 cm and 30 cm taller, a `toilet` at `(7, 10, 2)` 25 cm taller, a `filingCabinet` at `(13, 10, 2)` 40 cm taller, a `laundryWasher` at `(16, 10, 2)` 200 × 100 cm and 20 cm taller, a `wallBed` at `(12, 3, 0)` 300 × 300 cm and 30 cm taller; needs at 0.5. For their clips with a resize (A-systems-01; resizing is off in Buy mode): `--use-existing fridge,toilet,filingCabinet,laundryWasher,wallBed`. `review-resized-base`: the same at their own size. |

A state keeps its own free will as the page opens (`?state=`: the game's Free will setting is sent only once it's
changed). Needs and how two residents stand with each other: `__send({type: 'setNeed', sim, need, value})` (`need`
`all` for every one) and `__send({type: 'setBond', a, b, friendship?, romance?, chemistry?, partners?})`. Other
states the tests write: `wants-asking` (its resident is asking to buy something and doesn't take a `use`),
`group-talk`, `basement`, `water-sounds` and more (`grep -rn dump_state crates/sim-core/tests`).

## Hooks under `?debug`

| Hook | Does |
|---|---|
| `__sims()` | Every resident as the newest snapshot shows them: `x`, `z`, `storey`, `yaw`, `pose` (`stand`, `sit`, `lie`), `moving`, the `object` they use or walk to (`__world().objects` id), its `def` and `interaction` ids, `using` (at it, not walking), `action` (the animation), `held`, `recipe`, `partner`, `ride` (the bike or e-scooter they ride, by object id), `needs` by id (the player's household and the inspected), `doing` and `queue` (as the resident's panel). Wait on it rather than timing screenshots: `page.waitForFunction(() => __sims()[0].using && __sims()[0].pose === 'sit')`. |
| `__world()` | The lot as drawn: objects (a lamp's or a fire's with `off`), walls, plots, `meta` (the streets). |
| `__send(command)` | Any simulation command (`web/src/core/protocol.ts`): `buy`, `use`, `setAutonomy`, `setSpeed`, `cancel`, `adopt`… |
| `__addFunds(amount)` | Money for the player's household (negative takes it away); for long sessions, where buys start failing ("not bought"). |
| `__camera(pose?)` | Reads the camera's pose; with a (partial) pose `{alpha, beta, radius, target: [x, y, z]}` or `{y}` (the height it looks at) puts it there. Stops the camera watching on its own. The radius goes from 6 m out to 900 m (`cameraInfo().radius`); past 120 m it is the town view, which holds the tilt between 0.6 and 1.0 and the target over land, so read the pose back to see where the camera is. |
| `__controls.follow(id, view?)` | The camera follows a resident; `view` `{radius, beta, alpha, y}` also zooms and aims (a dartboard above the resident: `y: 1.6`). `follow(null)` stops, the view staying on the lot it shows (the game's own Follow goes back home). The rest of `__controls` is the HUD's controls. |
| `__content` | The game's content: `__content.object('<id>').interactions` for interaction indices. |
| `__renderer`, `__scene` | The renderer, whose `RendererDebugHooks` (web/src/render/types.ts) the tools use: `objectClips(id)` (the clips playing on an object), `objectMotions(id)`, `objectPoints(id)` (its vertices as drawn), `cameraInfo()` (lens and limits), `keepRoof(true)` (the roof stays on however close the camera comes: the game hides it under 15 m; it shows with walls up only, and `__camera` hands back the walls the watching camera found, so put the camera first and the walls up after); and the engine's scene, for the console only. |
| `__showPlot(i)`, `__items()`, `__crash(what)` | Look at another lot; the food and dishes about; fail on purpose. |
| `__places()`, `__lookInside(name, storey?)` | Every named place with a lot of its own (name, kind, plot, the layout it was stamped from, its lot, storeys and hours; `web/src/debug/places.ts`, dev server only), and one looked inside by name as its card's Look inside does, on that storey (0: the ground). |
| `hud=0` (URL) | The interface hidden (the top bar, the household strip, what's over the world, toasts), on any page: pictures of the world alone. |

## Sharing one machine with other sessions

Several sessions (and agents) often work at once, each in its own worktree:

- **Ports.** Start your own dev server from your own checkout, on a port of your own, with `--strictPort`;
  check it's free first (`lsof -iTCP:<port> -sTCP:LISTEN`). Never use 5199 (the main checkout's) or a
  server another session started: with the port taken, `--strictPort` fails, and a `curl` or browser
  would answer from someone else's code. A port given up is soon taken by someone else: pick a fresh one
  each run, and stop your server when you're done. `check.mjs --port` refuses a port in use.
- **Worktrees.** Copy (don't link) the git-ignored wasm packages into a worktree
  (`web/src/core/wasm-pkg`, `web/src/voice/wasm-pkg`; a link is refused with 403), and `debug-states/`
  too (a run can change a shared state). `web/node_modules` may be a link to the main checkout's:
  `vite.config.ts` then lets the server read the linked folder and keeps its pre-bundled packages in
  `web/.vite-cache` (git-ignored), not in the shared `node_modules/.vite`, which every other worktree's
  server would re-optimise under it. A checkout with its own `node_modules` is unchanged.
- **Reloads.** Other sessions' edits in a shared checkout reload the dev server's page mid-run, and a
  content file being written can be invalid for a moment ("the game data failed to load"). Work in a
  worktree; if a run fails that way, run it again.
- **Scratch.** Give every session or agent its own folder in the scratchpad (as with ports): parallel
  agents writing helper scripts into one folder overwrote each other's. Keep screenshots, copies of
  content and probes there, never in the repo.
- **Shared files.** Write a shared file only after its new text is built and parsed (`t = fix(s);
  json.loads(t); open(p, 'w').write(t)`: opening it for writing first empties it, and once emptied
  `furniture.json`). Moving items into Validate and back: `cargo content`, which does that.
