# Release review for v0.2.0

The pre-release review for 0.2.0 started on 2026-10-13 from `main` at 4cde174, which is 136 commits after v0.1.0. It covers animations, objects and items, bugs, performance and release readiness. Each finding has a location, a severity, a description and a proposed fix.

**Status (2026-10-13):** the review is done and its fixes are merged into `main`.
- §1 describes the game as it was at 4cde174, and §2 lists the 140 findings with the owner's decisions.
- §3 lists what was fixed, with the commit, the merge and the numbers before and after.
- §4 lists what was found but not fixed, with a recommendation and a target release.
- §5 is the checklist for the owner's own tests: the hardware, browsers and long sessions no agent could cover.
- §6 lists the questions left for the owner, with the default that shipped.

Fixes came in two waves of agents, each on its own branch:
- **Wave A:** F1a and F1b (simulation), F2a (crash safety), F2b (input, audio and small web fixes), F2c (saves and start-up), F4 (animation), F5 (build, CI and licences), T1 (long-run tests).
- **Wave B:** G1 (presets, resolution, pacing), L1 (load time and stepping), G2 (GPU savings nobody can see) and M1 (main thread).

Every agent's branch is merged. The release workflow's first dry run passed on Linux (§3.6, R5-02), and on `main` clippy is clean and the tests pass (§3.8).

Severity:
- **critical**: a crash, data loss, or a release blocker.
- **high**: a clearly visible bug or a missed performance target on common hardware.
- **medium**: a real problem that is rare or mild.
- **low**: cosmetic, hygiene or hardening.

Line numbers refer to 4cde174 unless a later commit is named.

## 1. How the game is put together

### Project structure and build
- **Rust workspace** (`Cargo.toml`, edition 2024, version 0.1.0 before this release) with four crates:
  - `crates/sim-core`: every game rule (needs, utility AI, pathfinding, town, social life, life cycle, saves, geometry). No browser code; tested natively (`cargo test -p sim-core`).
  - `crates/sim-wasm`: a thin wasm-bindgen wrapper (`Game`) around `sim_core::World`.
  - `crates/voice`: phonemizer, mouth shapes and voice shaping.
  - `crates/voice-wasm`: wasm-bindgen wrapper for the voice worker.
- **Release profile:** `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`. `.cargo/config.toml` enables `simd128` for `wasm32-unknown-unknown`.
- **Web app** in `web/`: Vite 8, Svelte 5, TypeScript, Babylon.js 9.30, pnpm.
  - `npm run wasm` builds `sim-wasm` in release and runs `wasm-bindgen --target web` into `web/src/core/wasm-pkg/` (gitignored).
  - `npm run voice` does the same for the voice crate and fetches the voice model.
  - `npm run llm` fetches the optional language model.
  - `npm run build` runs all three, then `vite build`. The production build also writes compressed copies of the assets (`tools/assets/optimize.mjs`: KTX2 textures, meshopt models).
  - Vite content-hashes every JS chunk and both `.wasm` files. The wasm-bindgen glue loads its wasm through `new URL(..., import.meta.url)`.
  - Before this release, no `wasm-opt` step ran anywhere.
- **Deploy:** `.github/workflows/pages.yml` publishes to GitHub Pages (https://krecks.github.io/idyll-lives/) when a `v*` tag is pushed. `tools/release/version.mjs` keeps `web/package.json` and `Cargo.toml` on one version, which the menu and Credits show through `__APP_VERSION__`.

### How the world is drawn
- **3D on the GPU.** Babylon.js uses WebGPU and falls back to WebGL2 (`web/src/render/babylon/`). Svelte draws only the interface (HUD, panels, menus) as DOM over the canvas; nothing in the world is DOM or SVG.
- **Only the viewed lot is built in detail.** Other houses are silhouettes. The landscape (terrain, forests, lawn) is procedural and uses thin instances.
- **Materials and lighting:**
  - Materials are PBR with image-based lighting.
  - One sun casts a 2048 shadow map, and lamps use clustered lighting.
  - A post-processing pipeline renders into a half-float HDR target with 4× MSAA, followed by tone mapping and a night grade.
- **Residents** are skinned on the GPU from a pose texture that is uploaded every frame.
- **A second, small engine** (`render/preview/`) draws the household-creator preview, portraits and catalog pictures.
- **Before this release:** quality presets low / medium / high / ultra plus fine-tune toggles (`render/quality.ts`, `settings/settings.svelte.ts`). The default came from High.

### The game loop
- **The simulation drives itself in a Web Worker** (`web/src/core/sim.worker.ts`) with a `setTimeout` loop at a fixed 20 Hz (`TICKS_PER_SECOND`, `sim-core/src/lib.rs`).
  - At speed 1, one game minute passes per real second.
  - Speeds 0–5 run `[0, 1, 3, 10, 30, 60]` ticks per step (`clock.rs`).
  - With auto-fast on, a step may run up to 30 ticks while everyone in the household is asleep or away ("quiet hours", `World::advance`).
- **Pause is speed 0:** the loop keeps publishing, but no ticks run. Menus, Build mode and the planner pause the game by switching to speed 0. They restore the previous speed when they close.
- **Falling behind:** if the worker falls more than 1 s behind, it drops the backlog instead of catching up. Ticks per step never exceed 60, so 1,200 ticks a second at most.
- **The renderer runs its own loop on the main thread** (`engine.runRenderLoop`, `BabylonRenderer.ts`), capped by the FPS setting through Babylon's `engine.maxFPS`. (In 0.2.0 the FramePacer paces it instead; see R1-02 in §3.1.)
  - It interpolates residents between the two latest sim steps, using a smoothed frame clock that follows snapshot arrivals (`core/bridge.ts`). It draws one step behind and extrapolates at most 1.3 steps.
  - Animations use real time; the character animation rate follows the game speed up to a limit.

### How state flows from Rust to the screen
- **Render snapshot, every step (20 Hz).** A flat `f32` buffer: 9 header floats plus 64 residents × 23 floats. Its layout is defined once in Rust (`sim-core/src/snapshot.rs`) and read by TypeScript at start-up.
  - When the page is cross-origin isolated, it travels through a SharedArrayBuffer guarded by a seqlock (`core/snapshot.ts`), so no copy and no message.
  - Otherwise it is transferred as a copy.
  - The worker re-creates its view of wasm memory on every publish, so wasm memory growth never leaves a stale view.
- **UI state, about 10 times a second and after each command.** Delta JSON (`view::UiSync`): only what changed, keyed by id, with details only for the player's household and the inspected resident. The main thread keeps a mirror (`game/uiUpdates.ts`). Large lists use `$state.raw`.
- **World structure** (walls, rooms, floors, meshes, objects), only when its version changes. JSON plus copied mesh buffers; the lot and meshes only when the lot itself changed.
- **Commands from the player.** JSON over `postMessage`, applied between ticks.
- **Never serialized per frame or per tick:** the whole world.

### Saving and loading
- **What is saved:** the simulation writes the whole world as JSON (`sim-core/src/save.rs`, `SAVE_VERSION`, 22 at the start of this review). Saves from older versions are migrated on load.
  - Time is an integer tick count (`u64`).
  - The random generator is a seeded xorshift whose state is saved; the starting seed was not saved before this release.
- **Where:** saves are kept in IndexedDB (`persistence/saves.ts`), each with metadata and a JPEG thumbnail.
  - Autosave writes one slot every 5 minutes by default.
  - A new 9-house town saves to about 150 KB and grows to 200–220 KB over weeks of play (about 20 KB gzipped). A full town of 64 residents is about 1 MB (83 KB gzipped). A default new game from the menus (14 households, 41 residents) is about 0.6 MB (43 KB gzipped).
- **Elsewhere in the browser:** settings and blueprints live in localStorage.
- **Load:** a save is handed to the worker, which builds the world with `Game.fromSave`.
- **Starting fast:** the main menu prepares the latest save in the background so Continue starts at once.

## 2. Findings

Eight reviewers worked in parallel, each in their own worktree of 4cde174:
- **R1:** animations and frame pacing.
- **R2:** objects and items.
- **R3:** simulation bugs and Rust lints.
- **R4:** web robustness: panics, errors, input, resolution, audio and memory.
- **P1:** GPU and rendering.
- **P2:** main-thread CPU.
- **P3:** simulation speed, the wasm build and load time.
- **R5:** release readiness: build, deploy, saves, licences and docs.

How they worked:
- **No game code was changed.** Sim bugs were shown with proving tests that fail on 4cde174. They are `#[ignore]`d and kept on the reviewers' branches: R2 in `crates/sim-core/tests/review_r2_objects.rs` (commit 90c66c1, 6 tests), R3 in `crates/sim-core/tests/r3_probe.rs` (commit 734d922, 10 tests, 6 of them proofs). R4 showed panics with temporary hooks that were reverted.
- **Browser checks** ran in headed Playwright Chromium 1243 (Chrome for Testing) on WebGPU, on an Apple Silicon Mac (an M3 Pro for P2 and P3). Test situations were dumped from sim tests and opened with `?state=`.
- **Timings** were taken under a shared bench lock, A/B against 4cde174, as the median of 3 runs.
- **Target laptops.** The i7-7500U, i7-8550U and i5-6500 (Intel HD 620/530) are estimated at ×3 the Mac's CPU time. Single-core benchmarks give 2.4–2.7, and 15 W laptops hold lower clocks. P2 used Chrome's CPU throttling at 3× (`r = 3`), which slows only the page's main thread.
- Before the review, a first pass collected what was already known. Where a reviewer corrected it, the section says so under "Corrections to earlier assumptions".

How to read the tables:
- Rows are sorted by severity and keep the reviewer's order within a severity. The IDs are the reviewers' own.
- A finding reported by two reviewers is listed once, in the earlier area, with both IDs and the higher severity. A few closely related low findings from one reviewer share a row.
- Items that need no action are left out of the tables; the important ones are under "Checked and fine". Information items with a proposed change are graded low.
- R2 and R4 used a stricter scale than the one above: critical only for a crash or data loss likely in ordinary play, and high for a crash or corruption in plausible play. Their grades are kept, except that R2-01 and R4-01 are raised to critical by this document's scale: a crash or silent freeze reachable in ordinary play.
- P1 graded against the HD 620 targets (Medium 30 fps, Low 60 fps). Its grades are kept, including P1-01 as critical.

| Area | Critical | High | Medium | Low | Rows |
|---|---|---|---|---|---|
| 2.1 Animations and frame pacing (R1) | 0 | 2 | 4 | 11 | 17 |
| 2.2 Objects and items (R2) | 1 | 0 | 4 | 14 | 19 |
| 2.3 Simulation bugs and lints (R3) | 0 | 2 | 5 | 7 | 14 |
| 2.4 Web robustness (R4) | 1 | 5 | 7 | 11 | 24 |
| 2.5.1 GPU and rendering (P1) | 1 | 7 | 6 | 3 | 17 |
| 2.5.2 Main thread (P2) | 0 | 1 | 3 | 5 | 9 |
| 2.5.3 Simulation, build and load (P3) | 0 | 0 | 3 | 10 | 13 |
| 2.6 Release readiness (R5) | 0 | 4 | 12 | 11 | 27 |
| **Total** | **3** | **21** | **44** | **72** | **140** |

### 2.1 Animations and frame pacing

R1 measured motion and frame pacing in the running game (MacBook Pro with a ProMotion display, Chrome's rAF at 120 Hz, 1600×900) and traced the interpolation clock frame by frame. It simulated the frame cap for refresh rates it couldn't measure. Two findings are high: every world update resets every resident's animation, and the frame cap judders on displays whose refresh rate isn't a multiple of the cap.

| ID | Severity | Where | Problem | Proposed fix |
|---|---|---|---|---|
| R1-01 | high | `web/src/render/babylon/BabylonRenderer.ts:2797` | Every world update (an object breaks or is mended, any Buy or Build edit, arrivals, births, age-ups, lot switches) rebuilds every resident's rig. Clips restart without a crossfade, lip sync stops, and a held baby drops back into the crib. While paused, sleepers lose their blanket and pyjamas until the game runs again (measured after a `sell`). | Call `characters.build(world, true)`, and keep an old rig only when its row still holds the same resident with the same body (name, stage, outfit; slots are reused). Add a check that rig identity and clip time survive a `sell` while paused. |
| R1-02 | high | `web/src/render/babylon/BabylonRenderer.ts:847` | `engine.maxFPS` can only skip whole vsyncs, so a cap that doesn't divide the refresh rate alternates 1-, 2- and 3-vsync gaps. Simulated: cap 60 on 144 Hz gives 13.9 / 20.8 ms gaps (60 / 40 %); on 75 Hz a 26.7 ms hole every 4th frame. Measured at 120 Hz: 236 of 240 gaps exact, one hiccup pair every ~2 s; ProMotion once ran the window at 102.7 Hz. | As decided: a `FramePacer` as `engine.customAnimationFrameRequester` that counts vsyncs and renders every N-th, N = round(refresh / cap) within +25 %. Drop `engine.maxFPS`. |
| R1-03 | medium | `web/src/core/bridge.ts:182` | The interpolation clock counts snapshots read, not sim steps. After any frame over ~50 ms it runs a step late for about 1 s (after one 75 ms frame: 2–6 of every ~14 frames extrapolated, 4 pinned at the 1.3 cap), so walkers stutter at 20 Hz. Each command also publishes an extra snapshot of the same tick, which freezes all motion for one step (38 ms after a `sell`). | Publish a step counter with each snapshot (next to the seqlock sequence and in the postMessage path) and set `arrivals` from it. A command's snapshot with the same step replaces the newest one instead of shifting the ring. |
| R1-04 | medium | `web/src/render/babylon/characters.ts:1665` | Positions are interpolated only while a step moves less than 1.6 m. A step is 2.25 m at speed 4 (also in quiet hours) and 4.5 m at speed 5, so every walker teleports 20 times a second. | Snap only when the jump is longer than walking could cover in the snapshot's ticks (`max(1.6, walk × trait max × ticks per step × 1.2)`) and the resident moves in both snapshots. The trade-off is cutting corners through walls (owner question). |
| R1-05 | medium | `web/src/render/babylon/characters.ts:2008` | Walk playback is clamped at 3×, so feet slide from speed 2: adults 27–33 %, teens 37–40 %, children 58–59 %; at speed 3, 78–88 %. Children walk at the adults' 1.5 m/s on short legs, so they scurry (2.1–2.9×) even at speed 1, and a short Energetic Speed Walker child slides at 1×. | Owner question: above ~2×, switch to the jog clip or accept a capped slide. For children, a slower walk speed (a gameplay change) or a longer-stride child clip. |
| R1-06 | medium | `web/src/render/babylon/characters.ts:1760` | Turning is capped at 9 rad/s while walking (6 otherwise) in real time. At speed 3 a 180° turn at a path corner takes 0.35 s, during which the walker moves about 5 m sideways or backwards. | Scale the turn limit and the smoothing with `min(gameRate, 10)` while moving, or face the travel direction directly when a step is longer than about 0.5 m. |
| R1-07 | low | `web/src/render/babylon/BabylonRenderer.ts:794` | Bubbles, subtitles, Build-mode room tags and the selection marker are placed before `scene.render()` applies this frame's camera input. They trail the heads by one frame while the camera moves (about 8 px at 60 fps for a 500 px/s drag). | Place them from `scene.onBeforeCameraRenderObservable`, or project with the updated view matrix. Don't call `camera.update()` twice. Do it together with P2-02. |
| R1-08 | low | `web/src/render/babylon/BabylonRenderer.ts:1841` | Babylon's default keyboard camera input is on: arrows rotate, Ctrl+arrows pan, +/- zoom once the canvas has focus. Each held key adds a fixed push per frame, so speed follows the frame rate (ArrowLeft for 1.5 s: about 5 rad at 60 fps, 2.5 rad at 30). | Remove `ArcRotateCameraKeyboardMoveInput` if the arrows aren't meant to move the camera (owner question), or scale its speeds by `dt × 60` each frame. |
| R1-09 | low | `web/src/render/babylon/BabylonRenderer.ts:646` | The first frame after a hidden tab gets the whole hidden time from `getDeltaTime()`. The watching camera's drift swings 0.9–1.8 rad after 60 s hidden, and the clouds jump. `idleOrbit` is never switched on. | Clamp once (`Math.min(engine.getDeltaTime(), 100)`) and pass that everywhere. Delete `idleOrbit`. |
| R1-10 | low | `web/src/render/babylon/BabylonRenderer.ts:1823` | `onFrame` gets `performance.now()` instead of the rAF timestamp, so motion timing includes earlier work in the frame (p95 3.2 ms, max 8.3 ms). Small. | Pass the rAF timestamp; the FramePacer of R1-02 has it anyway. |
| R1-11 | low | `web/src/render/babylon/characters.ts:2154` | Hand easing adds a fixed 0.02 per frame, so hands keep moving while paused and ease 2.4× faster at 144 fps than at 60. | `1 - Math.exp(-(step * 5 + dt * 1.2))`, which is 0 while paused. |
| R1-12 | low | `web/src/render/babylon/characters.ts:1710` | A resident who becomes visible again (camera, storey, back from away or from another lot) snaps into place but keeps the clip state from when last seen, so they appear mid-crossfade from a stale clip. | When the rig was hidden, start the chosen clip at full weight and set the arm, approach and hand weights to their targets. |
| R1-13 | low | `web/src/render/babylon/characters.ts:1370` | When a cuddle ends because the holder walks off, the baby stays posed in the walking holder's arms for about 0.6 s, then flies back into the crib. A baby who ages up mid-cuddle lies in the arms for that time. | End the lift at once when the holder moves or the baby is no longer a lying baby. |
| R1-14 | low | `web/src/render/preview/stage.ts:185` | Create-a-Resident framing eases by a fixed 0.2 per frame on an uncapped engine, so it settles twice as fast at 120 Hz as at 60. | `1 - Math.exp(-dt * 12)` with the stage's `dt`. |
| R1-15 | low | `web/src/render/babylon/nature.ts:807` | Reduce motion covers CSS and the camera glides, but not the 3D decorative loops (wind sway, clouds, the marker's bob and ring spin, the Build ghost's bob, build effects), the Svelte `out:fade` or smooth `scrollIntoView`. The `Busy` sweep freezes as a static bar that looks like progress. | Pass `reducedMotion` to the renderer and zero the decorative amplitudes. Use instant scrolling and a 0 ms fade when reduced, and a static or opacity-only `Busy`. |
| R1-16 | low | `web/src/ui/screens/LoadingScreen.svelte:18` | The loading screen fades out over 420 ms as a full-screen layer that still takes pointer events, so the first clicks on the game are swallowed. The fade also runs with reduce motion. | `pointer-events: none` once loading is done (or when the outro starts), and a 0 ms fade when reduced. |
| R1-17 | low | `web/src/render/babylon/BabylonRenderer.ts:491` | Two world messages in quick succession start two `buildWorld` runs that interleave at every `await`. That may leave duplicate or missing furniture until the next rebuild. Found by reading the code; not reproduced. | Serialise world builds: keep the latest pending world and build one at a time (owner question). |

**Checked and fine**
- Characters use a wall-clock `dt`, clamped to 0.1 s and 0 while paused. Pausing during quiet hours does freeze them.
- After a hidden tab, the interpolation clock resets and far walkers snap.
- A new action crossfades (0.2–0.9 s), and a conversation that ends mid-gesture fades out.
- Particles are frame-rate independent, and build effects clamp to 50 ms.
- The scene veil, the hidden game layer and the HUD container don't block input; only the loading screen does (R1-16).

**Corrections to earlier assumptions**
- Camera drag and wheel glide are frame-rate independent in Babylon 9.30 (`CameraMovement` decays with `pow(inertia, dt/16.67)`). Only the keyboard input works per frame (R1-08).
- Feet slide from speed 2, not 3, and children can slide at speed 1 (R1-05).
- The frame-cap judder is structural at 75, 90, 100, 144 and 165 Hz (simulated). At 120 Hz it is mostly absent on this Mac (measured).
- Of the unclamped `getDeltaTime()` uses, only the camera drift and the clouds misbehave.

### 2.2 Objects and items

R2 read the object and item code, scanned all shipped content (539 objects, 24 items, 9 recipes, 15 traits) and wrote six proving tests. A soak that moved one home object every game hour crashed 2 of 6 towns within 20 game days (R2-01).

| ID | Severity | Where | Problem | Proposed fix |
|---|---|---|---|---|
| R2-01 | critical | `crates/sim-core/src/home.rs:1179` | Selling or moving an object shifts the ids in residents' `current` and `queue` but not in `follow_up` or `stuck`. A pending follow-up then panics at `content.rs:210` when its id now names an object with fewer interactions, or sends the resident to the wrong object, possibly a neighbour's. NPC purchases also shift ids, mid-tick. Measured: about 1 crash per 1,200 Build-mode visits that move or sell something. | In `remove_object`, shift `follow_up` too (drop it if it used the removed object), and shift or clear `stuck`. In `start_task`, use `interactions.get()` and check `access`. Tests: the two R2-01 proofs. |
| R2-02 | medium | `crates/sim-core/src/cooking.rs:1682` | Items go with any object that shares their tile, not with their holder. Selling a fruit bowl or a rug deletes the dinner on the table, moving the fruit bowl carries the meal along, and a wall cabinet over the fridge carries off the leftovers. | Only touch items whose `items::holder(...)` is the object, worked out before the object is removed or lifted. Tests: the fruit-bowl and rug proofs. |
| R2-03 | medium | `web/src/game/session.ts:541` | A move renumbers the object last, but Rotate (R, Shift+R, the Rotate and Turn 15° buttons) and Reset size don't reselect it. After one R the selection belongs to the next object: holding R rotates the home's objects one after another, and Delete or Sell then sells the wrong one. Found by reading; not checked in a browser. | Set `reselect` with the new rotation in `rotatePlacing` and `resetSize`. Better: have `moveObject` answer with the new id. |
| R2-04 | medium | `crates/sim-core/src/world.rs:2460` | Commands name objects by position id, so a second command sent before the world update arrives hits whatever has that id now. Two quick rotates (key repeat is about 30 ms) turn the rug and then move the lamp onto it. In Live mode a pie-menu Use can hit another object after an NPC purchase. | The planned optional `seen {def, x, z}` check in `World::apply`. Test: `a_repeated_rotate_does_not_move_another_object`. |
| R2-05 | medium | `crates/sim-core/src/content.rs:2090` | `TraitRaw` has no camelCase rename, so `startingSkills` is silently dropped. All 14 traits that set starting skills give 0, and pack trait patches do nothing. | `#[serde(default, rename = "startingSkills")]` and keep the test. New residents would then start with skills (owner question). |
| R2-06 | low | `web/src/game/session.ts:560` | Undo and Redo don't drop the object in hand. After the restore its id can name another object, and putting the ghost down moves that one. | Call `buildBuy.cancel()` at the start of `undo` and `redo`. |
| R2-07 | low | `crates/sim-core/src/world.rs:3845` | A resident still uses something that broke on the way. The same holds for cooking stations and dining seats. | As decided: check `broken()` on arrival in `world::arrive` and `cooking::arrive`. |
| R2-08 | low | `crates/sim-core/src/world.rs:3413` | Repair and Clean reserve nothing, so two housemates can walk to the same job; the second one ends at once. | Reserve a slot for Repair, and skip Clean tiles someone is already heading for. |
| R2-09 | low | `crates/sim-core/src/world.rs:2281` | A directed Use has no access check, so the player can send a resident into another household's home to use their bed, shower or fridge. | Owner question. If not intended, refuse in `Command::Use` when `access` is none, and check again in `start_task`. |
| R2-10 | low | `crates/sim-core/src/world.rs:2563` | The queue cap of 6 applies only to player orders. Internal tasks can go past it, and `wake_for_crying_babies` then truncates to 6, which can drop a player order or a recipe already paid for. | Truncate only undirected, unpaid tasks, or let internal tasks go over the cap. |
| R2-11 | low | `crates/sim-core/src/home.rs:175` | Undo doesn't take back the story events an edit made: an `Upgraded` event stays after the upgrade is undone. | Remember the events length in `HomeSnapshot` and truncate on restore. |
| R2-12 | low | `crates/sim-core/src/world.rs:2341` | Without a sink or dishwasher, dirty plates and spoiled food are never cleared. Nothing caps the item list, so items pile up on the floor and are scanned on every eating decision. Selling the sink is enough. | Let Clear use a bin when there is no sink, or cap or expire floor items per household. |
| R2-13 | low | `crates/sim-core/src/cooking.rs:846` | Ingredients are paid at the first step. If a later station stays busy past `station_wait_minutes`, the recipe is dropped and the money is gone. | Owner question: refund, or keep waiting once paid. |
| R2-14 | low | `crates/sim-core/src/items.rs:125` | A hand-edited save with item id `u32::MAX` wraps the next id, which breaks the sorted lookup. Normal play can't get there. | `checked_add`: refuse the save, or saturate. |
| R2-15, R2-16 | low | `crates/sim-core/src/items.rs:127` | Version counters use plain `+= 1` (harmless, they're only compared for equality), and `Items::get_mut` bumps even when nothing is found. `idle_ticks` grows without limit for a resident with free will off and wraps after ~149k game days. | `wrapping_add` for the counters and `saturating_add` for `idle_ticks`, to state the intent. |
| R2-17, R2-19, R2-25 | low | `crates/sim-core/src/content.rs:2371` | Shape checks are missing from content loading: a recipe with more than 255 steps makes the cook loop forever, duplicate interaction ids in one object are accepted (a save reloads the first), and a footprint over 6 is accepted but can't be bought and breaks lot loading. Shipped content is within range. | Reject these in `cooking::parse` and the object parser. |
| R2-18 | low | `crates/sim-core/src/content.rs:3152` | `resale` and `upgradeCost` aren't range-checked. A resale above 1 is a buy-and-sell money loop, below 0 selling costs money, and a negative `upgradeCost` pays the player. Shipped values are 0.6 and 0.3. | Require `0 ≤ resale ≤ 1` and `upgradeCost ≥ 0`. |
| R2-20, R2-21, R2-22, R2-23, R3-13, R3-14, R3-16 | low | `crates/sim-core/src/content.rs:2159` | Content numbers aren't range-checked. A `walkSpeed` of 0 or less gives inf/NaN and a walker who never arrives; a negative decay pushes needs above 1; more than 32 genders overflow the attraction mask; bond presets, and relationships in a loaded save, aren't clamped to ±100. Shipped content is fine, and R3's 60-day probe saw no value out of range. | At load, reject a non-finite or non-positive `walkSpeed`, negative decay and more than 32 genders. Clamp bonds in `bond` and on save load. |
| R2-24 | low | `crates/sim-core/src/content.rs:2939` | Unknown tags, animations and skills in day rhythms, accident rests and repair skills are dropped silently, so a typo turns a feature off (nobody counts as asleep; Repair trains nothing). Shipped content has none. | Use the erroring `lookup`. |

R2 graded R2-01 high. It is raised to critical here: it is a crash reachable in ordinary play, through Build-mode edits and NPC purchases.

**Checked and fine**
- Seats reserve one slot each, and residents step one after another, so nothing is double-booked within a tick.
- Undo and redo can't duplicate money: redo uses the snapshot taken at undo time, and any tick clears both stacks.
- Money is `i64` and can't realistically overflow. The only money hazards are content values (R2-18).
- The content scan of everything shipped found no duplicate ids, no dangling references, a station for every recipe step, valid object ids in every house and lot file, and all 45 animations in the renderer. The only defect is R2-05.
- Content comes only from the fixed `content/base.json` and its includes, so bad content is a developer and CI risk, not a player one.
- Without Build-mode edits, 8 towns × 30 days saw 1–4 autonomous id shifts each and no panic.

**Corrections to earlier assumptions**
- Items don't follow their host. They follow whatever object shares their tile (R2-02).
- Id shifts don't need the player: NPC households sell the object they replace during the tick (R2-01).
- The stale id after Rotate is worse than noted: the selection itself moves to another object (R2-03). Undo or Redo with an object in hand has the same problem (R2-06).
- Duplicate tag ids are not a gap; they are merged. Duplicate need, emotion and feeling ids are rejected by the pack merge.

### 2.3 Simulation bugs and lints

R3 ran the Rust builds, tests and clippy, and a 60-day full-detail probe on two towns (seeds 7 and 11) that tracked every float, range, activity start, walk and conversation. The probe found no NaN and no value out of range. The visible problems are behaviour loops and the clock.

| ID | Severity | Where | Problem | Proposed fix |
|---|---|---|---|---|
| R3-01 | high | `crates/sim-core/src/clock.rs:14` | `MINUTES_PER_TICK` is an f32, so `tick_at(d, 0)` lands one tick before day `d` starts on days 1167–2331, 3497–4661 and from 5827. On those days the midnight gates never fire: no births, ageing, retirement or death. Day 1167 is 7.8 hours of play at top speed. | An integer clock (20 ticks a minute; day and minute by integer division), or at least exact f64 arithmetic in `clock.rs`. Proofs: `every_days_gates_fire`, `residents_still_age_after_day_1167`, `clock_edges`. |
| R3-02 | high | `crates/sim-core/src/world.rs:3079` | A resident with an urgent need nothing at home can meet keeps choosing chats. Each one ends the target's activity, breaks off the next tick because the actor is pressed, and repeats after 21 idle ticks. An exhausted resident with no free bed pulled a housemate into 78 chats between 01:35 and 03:48; seed 7 had 242 such chats in 60 days. | In `pick_autonomous`, skip socials and joins (and likely calls) while the resident is pressed. In the Waiting branch, end the task instead of engaging. Proof: `someone_too_tired_to_talk_leaves_others_be`. |
| R3-03 | medium | `crates/sim-core/src/conversation.rs:211` | The 10-minute chat wait counts only Waiting ticks in a row, and every chase resets it. A resident followed a departing guest for 45+ minutes, 103 chases and about 95 m across town, into the guest's home. | Keep a total wait across chases. Give up when the target isn't available for a chat, or when the chase would leave the plots the actor could chat on. Proof: `nobody_follows_someone_across_town_for_a_chat`. |
| R3-04 | medium | `crates/sim-core/src/world.rs:3827` | With the need full, skill interest or a plan still picks an activity, but the use ends as soon as its needs are full. Residents start, stop after 1–9 ticks and restart a minute later: 2,210 and 1,318 quick restarts in 60 days (flowerbed, climbing frame, reading nook), up to 19 in a row. | Make the stop rule match the reason for the choice: don't end on "satisfied" while the interaction trains a skill the resident still wants. This lengthens practice (owner question). Proof: `practising_with_nothing_else_to_do_doesnt_stutter`. |
| R3-05 | medium | `crates/sim-core/src/lifecycle.rs:95` | Children who leave the crib get no bed, and NPC households never buy one. By day 60, 2–3 of 9 households had fewer bed places than people. The child without a bed wanders at night (toilet, sink, sofa, even the park at 01:00) and falls asleep on the floor at energy 0: 17 and 10 such accidents in 60 days. | Owner question: buy a bed as `cribs_for_babies` does for cribs, or make births and adoption need a bed place. Short term: sleep on a sofa or nap sooner when no bed is free. |
| R3-06 | medium | `crates/sim-core/src/world.rs:3966` | `home_needs` counts a bed even when every bed is taken. An exhausted resident without a bed then has energy as the pressing need, and anything else they pick ends on arrival with no gain. Seen as toilet → sink → toilet → sink loops (123 + 109 on seed 7) and a 57-minute walk home to a toilet that gave nothing. | Count only objects with a free slot (or held by this resident) that they can reach. |
| R3-15 | medium | `crates/sim-core/src/save.rs:916` | A save's household `plot` and `expecting.parents` are copied without range checks and cause a panic later. `yaw` is loaded raw. NaN can't arrive through JSON. | Range-check them on load. |
| R3-07 | low | `crates/sim-core/src/world.rs:3729` | The hand-washing follow-up runs even when the toilet use was aborted at once. | Set the follow-up only when the use did something. |
| R3-08 | low | `crates/sim-core/src/lifecycle.rs:49` | Births and ageing both run on the midnight tick, births first, so newborns are born half a year old and the baby stage lasts 3.5 days instead of 4. | Age before `births(w)`, or skip residents added this tick. Proof: `newborns_are_born_at_age_zero`. |
| R3-09 | low | `crates/sim-core/src/clock.rs:40` | `minute_of_day` can return 1440 (first on day 2329), so the UI could show 24:00. Nothing panics. | Fixed by R3-01's integer clock; otherwise clamp below 1440. |
| R3-10 | low | `crates/sim-core/src/world.rs:3436` | A visit whose route fails still sets the visit, so the resident counts as away while at home and can't use home things for 30 minutes. | Set the transition only after `walk_route` succeeds. |
| R3-11 | low | `crates/sim-core/src/world.rs:3316` | `stuck` remembers one failed choice, and only for Use and Eat. Unreachable Clean, Repair, Dishes, Visit, Social and Join targets are retried with a full A* every idle cycle. It costs CPU; nobody was seen stuck. | As planned (Social and Join in the memory are deferred). See also P3-08. |
| R3-12 | low | `crates/sim-core/src/ai.rs:117` | If any score is NaN, `choose` returns the last of the top three, and a `+inf` score is never chosen unless it is last. Shipped content gives finite scores. | Drop non-finite candidates before `choose`, with a `debug_assert!`. |
| R3-17 | low | `crates/sim-core/src/world.rs` (distance score) | Residents choose things 57–115 game minutes' walk away, even at night with almost no energy: an elder walked 57 minutes home to the toilet at 04:50. | Owner question: a stronger distance penalty at night, or a cap on autonomous trips. |

**Checked and fine**
- `cargo build --workspace` has 0 warnings. `cargo test -p sim-core`: 390 passed, 9 ignored. `cargo test -p voice --release`: 32 passed, and they really run once the lexicon is there.
- The 60-day probe saw 0 non-finite floats and 0 values out of range (needs, skills, grades, wear, dirt, relationships, chemistry), and nobody walking without moving.
- Two or three residents choosing each other for a chat resolve without a deadlock (R3-18).
- Clippy gives 14 warnings on the workspace (1 on wasm32). None is a bug, and all are fine to fix for the `-D warnings` gate; two are deliberate and want an `allow` (`voice/src/rules.rs:69`, `voice/src/resample.rs:22`). The pedantic set gives 1,618 warnings, and the only real issue among them is the clock cast (R3-01).

**Corrections to earlier assumptions**
- The clock bug only stops the midnight gates: births, ageing, retirement and death. Moving (11:00), newcomers (12:00), the job market (9:30), rent (12:00) and the planner review (07:00) keep working with the shipped hours.
- Flips between two activities don't come from `ai::choose`. They come from the stop rule (R3-04) and the bed shortage (R3-05, R3-06), so hysteresis or a commitment bonus isn't the fix.
- The `walkSpeed`, decay and relationship-range items (R3-16, R3-13, R3-14) are listed with R2's content checks in 2.2.

### 2.4 Web robustness

R4 tested how the web app fails: real wasm traps in the sim (through temporary, reverted hooks), a thrown frame, a destroyed GPU device, blocked files, pixel-ratio changes, stuck keys, and memory over a 10-minute speed-5 run and repeated lot switches. It used headed Chromium on WebGPU at 1280×800, DPR 2. A single panic, render exception or GPU reset freezes the game with no message, and Save then hangs.

| ID | Severity | Where | Problem | Proposed fix |
|---|---|---|---|---|
| R4-01 | critical | `web/src/core/sim.worker.ts:98` | A panic in the sim's `step` freezes the clock and every resident with no message. It only reaches an uncaught worker `error` event (not the F8 log), and every later action toasts "recursive use of an object detected…". With `panic = abort` the message is just `unreachable`. R2-01 is one real path to it. | Guard `step` and every handler: on a trap, drop the game, stop the loop and post a fatal error. Keep `worker.onerror` after start and show a blocking panel ("The game stopped because of an error. Your last save is safe.") with Reload and Download debug report. A panic hook in `sim-wasm` passes the message and location. |
| R4-02 | high | `web/src/core/sim.worker.ts:193` | A panic in a request handler (save, events, social options, commands) shows a 3.5 s `unreachable` toast, and the next tick freezes the game as in R4-01. After a trap the wasm instance can't be trusted. | The same guard: any trap ends the game instance, and nothing calls into it again. |
| R4-03 | high | `web/src/core/bridge.ts:131` | After a sim error, Save never finishes. The pause menu stays at "Saving…" with only "Quit without saving" left, each autosave adds a hung promise, and F8 hangs too and then stays disabled for the session. | Keep `resolve` and `reject` per request. Reject them all on a fatal error, a worker error or dispose, and after a ~10 s timeout. Build the F8 report without a save when the worker is dead, and always clear `debugSaving`. |
| R4-04 | high | `web/src/render/babylon/BabylonRenderer.ts:1819` | One exception in a frame stops drawing for good, because Babylon queues the next frame only after the current one returns. The HUD keeps changing over a frozen picture, and Save hangs (R4-06). GPU resets (R4-05) and NaN positions (R4-07) cause it. | try/catch around `renderFrame`, including `onFrame`: log the first error and keep the loop alive; after N errors in a row, show the R4-01 panel. Give frame waiters a timeout. |
| R4-05 | high | `web/src/render/babylon/BabylonRenderer.ts:431` | A GPU device loss (driver reset, Windows TDR, GPU switch, GPU-process crash) isn't survivable. Babylon reports the context restored, but the app's thin-instance and storage buffers belong to the old device: in 3 of 4 runs the next frame threw and drawing stopped, and in 1 the whole scene vanished. | The sim is unaffected. On `onContextLostObservable`, either rebuild the renderer and re-send the world, or show "Graphics were reset" with Save & Reload, which needs R4-06 (owner question). |
| R4-08, R5-04 | high | `web/src/App.svelte:68` | A latest save that can't load (a trap in `fromSave`, too new, version 0, truncated JSON) stays the newest. Continue fails every time with raw text ("unreachable", "unsupported save version 23", "invalid JSON: EOF…"), and every boot and menu visit prepares it again. A NaN autosave would break Continue for good. R4 graded it medium, R5 high. | As decided: classify the error in the worker (too new, too old, corrupt). Delete corrupt and too-old saves with a short notice, keep and skip too-new ones, and remember failed saves so they aren't prepared again. Use plain words. Test each kind in a sim test and in Playwright. |
| R4-06 | medium | `web/src/render/babylon/BabylonRenderer.ts:1421` | Saving waits for a screenshot of the next drawn frame. With no frames (drawing died, the renderer inactive, or a hidden tab) there is no save: with the renderer inactive, a save hadn't finished after 5 s and finished 92 ms after drawing resumed. So autosave never runs in a background tab, where Chrome may discard the page. | Race the thumbnail against about 1 s and save with the last thumbnail or none, or skip it while hidden. Autosave on `visibilitychange` → hidden and on `pagehide`. |
| R4-07 | medium | `web/src/audio/world.ts:207` | `project()` returns true for NaN, and a NaN pan throws in Web Audio. A resident at a NaN position therefore throws inside the frame and stops drawing (R4-04). | `project` returns false unless both outputs are finite; `panAt` and `panFor` use 0 for a non-finite value. |
| R4-09 | medium | `web/src/core/bridge.ts:83` | A failed start (`init` or `fromSave`) never ends its worker. After one boot and one Continue on a trapping save, 3 sim workers were alive, and each background re-prepare adds one. | `worker.terminate()` before rejecting, in both error paths. |
| R4-10 | medium | `web/src/render/babylon/BabylonRenderer.ts:2191` | A pixel-ratio change without a window resize (moving the window between a Retina and a 1× monitor) keeps the old resolution: 2× → 1× renders 4× the pixels needed. Any settings change then applies the ratio twice, and the canvas drops to 640×356 for a 1280×713 view, visibly blurry. | Own the scaling: `adaptToDeviceRatio: false` and one `applyScaling()` that sets `1/(devicePixelRatio × scale)`, called on resize and from a re-armed `matchMedia('(resolution: …dppx)')` listener. |
| R4-11 | medium | `web/src/game/buildmode.ts:167` | Alt stays "held" after Alt+Tab (Windows) or after Option-clicking another window (macOS), so Build mode's next click is the eyedropper: it picks up the floor and switches the tool. Shift sticks the same way (snapping off). | Reset `alt`, `shift` and the pick hint on window `blur` and `visibilitychange`, or read `e.altKey` and `e.shiftKey` from the pointer events. |
| R4-12 | medium | `web/src/audio/world.ts:98` | The hidden-tab fade runs in the render loop, which stops in hidden tabs, so ambience and loops keep playing at their last volume (also when drawing has died). | As decided: suspend the audio context when hidden and resume when visible. Stop voices first, because a clip's `ended` doesn't fire while suspended. |
| R4-13 | medium | `web/src/App.svelte:31` | A tab left open across a deploy breaks at its next lazy chunk, because Pages removes the old hashed files. A missing GameView chunk leaves "Moving in…" forever, a missing PBR shader chunk makes every PBR material invisible with no message, and a missing sim wasm leaks its worker. | Handle `vite:preloadError` and catch the GameView import: show "A new version is available — Reload" and allow a retry. Optionally check an unhashed `version.json` at session start. See also R5-16. |
| R4-14 | low | `web/src/game/input.ts:43` | A missed pointerup (`pointercancel`, pen or touch, a release over another window) leaves a gesture open. A wall or room keeps following the mouse and the next click commits it; planner chips and calendar blocks keep dragging. | Treat `pointercancel`, `lostpointercapture` and window `blur` as a cancel, and end drags when `e.buttons === 0`. |
| R4-15 | low | `web/src/game/session.ts:1045` | Held keys auto-repeat into toggles: Space flickers pause, B flips Build mode with a sound each time, and so do Escape, J, M and P. Shortcuts also fire while a `<select>` has focus, so in the Planner's dropdowns P closes the planner and B opens Build mode. | Ignore `e.repeat` for toggles (keep it for R and PageUp/PageDown). Skip selects, editable elements, and focused buttons for Space and Enter. |
| R4-16 | low | `web/src/audio/mixer.ts:25` | `ctx.resume()` runs on every sound request, about 10 a second. Where a resume without a gesture is refused (Safari), each is an unhandled rejection, and Safari's `interrupted` state is never resumed. Not tested on Safari. | Resume once from a capture-phase `pointerdown` or `keydown`, catch the rejection, and treat `interrupted` like `suspended`. |
| R4-17, R5-11 | low | `web/src/persistence/saves.ts:42` | `writeSave` resolves when the `put` succeeds, not when the transaction commits, so a quota failure still shows "Game saved" and "Save and quit" leaves. A failed database open stays cached, and a closed connection isn't reopened. `listSaves` reads every save's full JSON (about 1 MB each in a big town) just to list them, twice at boot. Not reproduced. | Resolve writes on `tx.oncomplete` and reject on error or abort. Reset the cached database on failure and on close. Keep the metadata in its own store and read `data` only when loading. |
| R4-18 | low | `web/src/debug/log.ts:45` | Uncaught errors only go to the debug log, and worker errors not even there. "Couldn't load game data" has no Retry or Reload button. With no Svelte error boundary, an exception in the HUD leaves the interface half-updated. | Wrap `GameView` in `<svelte:boundary>` with a fallback that offers Save and Reload. Add Reload to the fatal screen, and forward worker errors to the log. |
| R4-19 | low | `web/src/render/babylon/BabylonRenderer.ts:431` | When WebGPU has an adapter but fails to start (a blocklisted driver, a device lost at start), the menu shows without its town and every game start fails. The player has to find the WebGL switch. | Catch the failure in `init` and create a WebGL2 engine instead. Remember it for the session and say so in a toast. |
| R4-20, P2-10 | low | `web/src/game/session.ts:876` | Once opened, the journal gets every later event appended and copied in full, and `Journal` rebuilds its days over all of it. With the journal open at speed 5: 473 entries after 10 minutes (+43 a minute), and DOM nodes 1,022 → 2,372. The sim keeps only 300. | Cap it at `EVENT_LOG_CAPACITY` (or clear it on close and fetch again on open), and render only the newest days. |
| R4-21 | low | `web/src/game/director.ts:83` | The director's event maps are pruned only while it runs, so with "watch on its own" off they keep every notable event for the session. The entries are small. | Prune or cap in `noteEvents` and `noteBuilt` too. |
| R4-22, R5-26 | low | `web/src/render/preview/items.ts:215` | Catalog pictures are cached per app version, but older versions' entries are never deleted, so each release adds a full set to the player's disk. | Put the version in the cache name and delete older caches, as `ml/assets.ts` does, or prune other versions' keys once per session. |
| R4-23 | low | `web/src/voice/tts.worker.ts:75` | A trap in the voice wasm is caught per line, and the worker keeps using an instance that may be corrupt. | On a `WebAssembly.RuntimeError`, post a fatal error so the voice service starts a new worker. |
| R4-24 | low | `web/src/game/session.ts:1269` | Small known items, confirmed: any settings change closes the F3 overlay and, when the autosave interval changes, clears `prefetchTimer`; an abort listener in `items.ts:160` lacks `{once: true}`; the long-task list is trimmed only when read (negligible). | Change `perfOpen` only when `showFps` changes, move `clearTimeout(prefetchTimer)` to `dispose`, and use `{once: true}`. |

R4 graded R4-01 high. It is raised to critical here: it is a silent freeze reachable in ordinary play, for example through R2-01.

**Checked and fine**
- No stale wasm memory views: the worker makes a new view per publish, mesh buffers are copied and freed, and the voice glue copies in and out.
- Babylon clears camera keys on canvas blur, so camera keys don't stick.
- Without WebGPU, voices and the language model download nothing, and the water-sound worker falls back to the main thread.
- Caches and lists are bounded (debug log, voice clips, portraits, toasts, story feed).
- Memory over 10 minutes at speed 5: JS heap +15.5 MB while the town grew from 19 to 24 residents; materials, textures and listeners flat; sim wasm memory 4.9 → 5.2 MB, then flat. Nothing grew without limit except the journal (R4-20).
- Lot switching returns the scene to its baseline, and each session's sim worker ends with it. About 17 Mesh and 16 Geometry JS objects (≈0.7 MB) survive each session restart; not traced further.
- Not tested: Safari and Firefox, a real monitor move, a real hidden tab, a real GPU-process crash and a real quota failure.

**Corrections to earlier assumptions**
- A panic in a request handler is fatal, not a harmless toast (R4-02).
- Browser zoom, window resize and fullscreen rescale correctly, and picking stays consistent. The resolution bug is a pixel-ratio change without a resize, plus the ratio applied twice after any settings change (R4-10).

### 2.5 Performance

Three reviewers measured performance on an Apple M3 Pro and scaled to the target laptops by ×3. P2's `r = 3` is Chrome's CPU throttling, which slows only the page's main thread: the sim worker, the compositor and the GPU still ran at M3 speed, and the M3's GPU is far faster than an HD 620. So the main-thread and sim numbers are estimates for the target CPUs, not whole-system frame rates.

#### 2.5.1 GPU and rendering

P1 built a rendering benchmark (`?bench=`, now on `main`) and measured every preset on WebGPU and WebGL2, with and without vsync, at 1× and 3× CPU. All numbers come from the benchmark:
- **Machine:** MacBook Pro M3 Pro (18-core GPU), Chrome for Testing 153 headed, 1920×1080 at DPR 1, production build.
- **Scene:** `village`, the default medium world (14 households, 41 residents), with 4 residents at home at 17:55. The camera orbits the home lot, flies wide, or sits in Build mode. Day runs hold the light at 13:00, night runs at 21:00.
- **GPU time:** on WebGPU, the span from the start of a frame's first pass to the end of its last, from timestamp queries. WebGL2's timer reads too high, so its frame time without vsync is used instead.
- **Spread:** with vsync the Apple GPU lowers its clock and GPU times read higher, so the Intel estimate uses runs without vsync. Runs vary by about ±1 ms, and the A/B tables are medians of 3.

The GPU is what keeps both targets out of reach, and WebGPU's CPU cost alone also rules out Low at 60 fps on the target CPUs.

Summary:
- **GPU per frame on the M3** (WebGPU, no vsync): Medium 10.3–10.5 ms, Low 6.6–7.0 ms. Low at 75 % scale costs the same 6.7 ms: on this GPU, Low is limited by geometry, not by pixels.
- **Every preset draws the same scene:** 6.0 M triangles a frame (2.26 M nature, 1.70 M shadow pass, 2.2 M lot, street and furniture) in 493 draw calls, 242 of them for the shadow map.
- **CPU at 3×:** WebGPU spends 21–31 ms a frame (32–47 fps), 20–28 ms of it submitting draws. WebGL2 spends 10–19 ms (54–98 fps).
- **HD 620 at 1080p:** Medium about 6–10 fps (target 30), Low about 9–27 fps (target 60), depending on the model. Both presets miss by 3–10×.
- **Savings with no visible change:** dark lamps (3.1 ms by day on the M3), the canvas's 4× MSAA (66 MB a frame) and the shadow map's unused colour target (32 MB a frame). The last two are bandwidth that Apple's tile memory hides but Intel pays for.
- **Low with trims:** without nature, shadows and the canvas MSAA, Low at 75 % drops from 5.2 to 2.45 ms on the M3 and from 491 to 224 draws. (That session measured Low at 75 % at 5.2 ms rather than 6.7; sessions differ by about ±20 %.) What remains is the lot, street and furniture geometry, so 60 fps also needs LODs (P1-12).
- **The interface costs about 4 ms a frame on the M3** (65 → 88 fps without the HUD). The frosted blur isn't the cause.

What each feature costs on Medium (GPU time saved by leaving it out; median of 3, no vsync):

| Feature left out | WebGPU GPU, day | WebGPU GPU, night | WebGL2 frame, day | WebGL2 frame, night | Draws | Triangles |
|---|---|---|---|---|---|---|
| (Medium as is) | 10.58 ms | 12.16 ms | 12.10 ms | 12.19 ms | 493 | 6.01 M |
| Clustered lamps (all 32 dark by day) | −3.07 | −4.28 | −0.80 | −1.64 | 0 | 0 |
| Scene 4× MSAA | −3.03 | – | −0.30 | – | 0 | 0 |
| Sun shadows | −1.20 | −1.50 | −1.53 | −2.26 | −242 | −1.70 M |
| Nature (grass, flowers, forest) | −1.08 | – | −2.12 | – | −38 | −2.26 M |
| Night grade | – | +0.01 | – | −0.80 | −1 | 0 |
| Canvas MSAA, frosted glass, mirrors | within noise | – | within noise | – | 0 | 0 |

Changes under about 1 ms are noise. The canvas MSAA isn't measurable on Apple but costs about 3 ms on Intel by the bandwidth model. No mirror faced the camera in the village, so mirrors weren't really measured. (P1's sign-off found that one does within the runner's default 20 s window, which P1's 10 s runs missed: §3.5.1.)

**Estimate for Intel HD 620 at 1080p.** No Intel hardware was available, so two models scale the M3's GPU time:
- **Formula:** T_gpu ≈ k × T_mac + 0.5 × B / 22 GB/s, with k = 16 and B = twice the render-target bytes touched a frame. k assumes an M3 Pro GPU of about 6.4 TFLOPS; 7.4 TFLOPS would give k = 18.5.
- **Split model:** k = 5 for the time that doesn't depend on resolution (geometry and fixed costs; Gen9 sets up about 0.3–0.5 G triangles a second), k = 16 for pixel work, plus the same bandwidth term.

| Preset, by day | M3 GPU | Render targets a frame | HD 620, formula | HD 620, split model | CPU at 3×, WebGPU / WebGL2 |
|---|---|---|---|---|---|
| Low at 100 % | 6.56 ms | 120 MB | 110 ms, 9 fps | ≈ 38 ms, ≈ 26 fps | 47 / 98 fps |
| Low at 75 % | 6.74 ms | 73 MB | 111 ms, 9 fps | ≈ 37 ms, ≈ 27 fps | 47 / 98 fps |
| Medium | 10.32 ms | 216 MB | 175 ms, 6 fps | ≈ 104 ms, ≈ 10 fps | 36 / 60 fps |

- A bottom-up count for Low (6 M triangles at 0.4 G/s, PBR with overdraw, 120 MB read and written at 22 GB/s) gives 25–35 ms, close to the split model.
- UHD 620 is about 5–10 % better, and HD 530 is similar.
- Low at 75 % without nature, shadows and the canvas MSAA: 41 ms (24 fps) by the formula, about 14 ms (70+ fps) with k = 5. The CPU at 3× then drops to an estimated 10–12 ms on WebGPU.
- With only the invisible savings, Medium stays under 30 fps on either model: about 8–9 fps by the formula, about 19–22 fps by the split model.
- The two models differ by about 3×. One run on a real HD 620 laptop would settle it.

**Running the benchmark.** Open the game with `?bench=village&quality=medium&hour=13&cam=orbit&res=1`; this works on the deployed site, so it can run on a real HD 620 laptop, and `&renderer=webgl` compares WebGL2. It warms up for 5 s, measures for 20 s, and then shows a summary with a "Copy result" button (also in `window.__benchResult`; with the dev server it is written to `bench-results/`). `diag=` leaves features out (`noshadows`, `noaa`, `nocanvasaa`, `nonature`, `nolamps`, `nomirrors`, `noglass`, `nonight`; `nocanvasaa` went with the canvas MSAA in e750d68), and `diag=freeze` stops time for screenshots; the flags are documented at the top of `web/src/debug/bench.ts`. `node tools/bench/render.mjs --preview --build` builds the game and runs presets × backends × scenarios (`--throughput` turns vsync off, `--throttle 1,3`, `--diags none,noshadows,…`, `--runs 3` for medians). `--shots <dir>` takes 9 fixed reference poses, and `--compare <dirA> <dirB>` counts the pixels that differ. `node tools/bench/scene.mjs --name village` makes the scenes in `web/public/bench/`.

| ID | Severity | Where | Problem | Proposed fix |
|---|---|---|---|---|
| P1-01 | critical | `web/src/render/babylon/nature.ts:580` | The 44,655 nature instances are 35 thin-instance meshes, one per model for the whole landscape, so their bounds are always in view and nothing is culled. They are 2.26 M of the 6.0 M triangles a frame in every preset, Low included. Leaving them out saves 1.1 ms GPU on the M3 and 2.1 ms a frame on WebGL2; on an HD 620, triangle setup makes them about 5–7 ms on their own. | Invisible, all presets: split each model's instances into spatial cells of about 32 m as separate thin-instance meshes, so frustum culling drops the cells behind the camera. Low: lawn tufts only on the viewed lot at half density, the far ring only from `.far` models, bushes and rocks only near. |
| P1-02 | high | `web/src/render/babylon/BabylonRenderer.ts:1884` | Clustered lamps cost 3.1 ms of 10.6 ms GPU by day with every lamp dark, and 4.3 of 12.2 ms at night (WebGPU; on WebGL2 0.8 and 1.6 ms). Babylon bins all 32 lamps into tiles by range whatever their intensity, every PBR fragment loops over its tile's lamps, and the tile pass runs every frame. | Invisible (Medium, High): while a lamp's intensity is 0, give it a range of about 0 or move it out of view, which needs no shader recompile. At night, cap the cluster's `maxRange` at the largest lamp range (7 m). Low: today's 4 forward lamps, none by day. |
| P1-03 | high | `web/src/render/babylon/BabylonRenderer.ts:1923` | The 4× MSAA HDR scene target (RGBA16F plus depth) touches 99.5 MB a frame and costs 3.0 ms of 10.6 ms GPU on the M3. On Intel its bandwidth alone is about 4.5 ms. P1 graded it high for Medium. | Medium keeps it (the look). Low: off, with FXAA as today; weak GPUs on Medium rely on Auto resolution. An LDR target for Low is document-only, as decided. |
| P1-04 | high | `web/src/render/babylon/BabylonRenderer.ts:1872` | The sun's shadow map is redrawn every frame from 251 casters: 242 of the frame's 493 draws and 1.70 M triangles, 1.54 M of them lot, street and furniture. It costs 1.2 ms GPU by day and 1.5 ms at night, and half of the draw submission: about 2.8 ms a frame of CPU on the M3, 10 ms at 3×. | Merge the static casters into a few shadow-only meshes per lot (house and fences, the street's houses and props, garden trees): about 10–20 shadow draws instead of 242. Low: 1024 and only the viewed lot and residents as casters; optionally redraw every second frame. |
| P1-06 | high | `web/src/render/babylon/BabylonRenderer.ts:433` | The canvas is 4× MSAA with a 4× depth-stencil (66 MB a frame at 1080p), but only the final full-screen pass draws to it: 31 % of Medium's render-target bytes and 55 % of Low's. On Apple it isn't measurable; on Intel it is about 3 ms by the bandwidth model. Screenshots without it are pixel-identical. P1 graded it high on Intel. | Invisible: `antialias: false` for both engines. |
| P1-07, P2-07 | high | `web/src/render/babylon/BabylonRenderer.ts:3097` | Submitting the same 493 draws costs 5.1–6.7 ms a frame on WebGPU and 3.2–4.6 ms on WebGL2 (M3). At 3× CPU, WebGPU frames take 21–31 ms (32–47 fps) and WebGL2 10–19 ms (54–98 fps), so WebGPU alone can't reach Low's 60 fps on the target CPUs. P2 saw the same from the page: `scene.render` is 75–80 % of main-thread time, and Low is as busy as Medium (90–95 % at `r = 3`). | Halve the draws (P1-04 and the Low trims: 491 → 224 draws, about 10–12 ms at 3×). Retry snapshot rendering in play with the pose texture updated outside the recorded bundles, and measure it. Whether Low on Intel should default to WebGL2 is an owner question. |
| P1-12 | high | `web/src/render/babylon/house.ts` | Without nature, the main pass still draws 2.2 M triangles and the shadow pass 1.54 M. Low draws exactly the same geometry as Medium, and at 75 % its GPU time doesn't drop (6.74 against 6.56 ms). This floor of about 2.5 ms is what remains after the Low trims. | LODs for furniture and the street's houses (the overview already has shells), and low-detail shadow copies for furniture, as the residents have. Low: neighbours' houses as overview shells. |
| P1-15 | high | `web/src/render/babylon/BabylonRenderer.ts:2191` | Known: there is no pixel ceiling. On a 4K or DPR 2 screen Medium touches about 4× the bytes and pixels of these 1080p numbers. | The planned Auto resolution with a pixel budget. |
| P1-05 | medium | `web/src/render/babylon/BabylonRenderer.ts:1872` | The shadow map has an RGBA16F colour target beside its depth texture, and PCF never reads it: 32 MB a frame written at 2048² (Medium) and 128 MB at 4096² (High). On Intel that is about 0.7 and 2.9 ms of write bandwidth. | Invisible: `new ShadowGenerator(size, sun, false, undefined, true)` (R16F) cuts the colour writes 4×. |
| P1-08 | medium | Build mode (`?bench=…&cam=build`) | The benchmark's Build scenario runs at about 60 fps in every preset on both backends, against 88–120 fps for the same orbit in Live (Low on WebGPU: 119 → 66 fps, GPU 5.8 → 9.3 ms). The frosted glass and the scene counts don't explain it, and switching a running game into Build mode didn't reproduce it. The benchmark's game was paused from the moment it appeared. Cause not found. | Find what differs in a game revealed paused straight into Build mode. Start with the catalog pictures' second engine (`web/src/render/preview/items.ts`), which may keep drawing, and the paused session's frame clock. |
| P1-09 | medium | `web/src/render/babylon/mirrors.ts:23` | Each live mirror draws the whole world again, nature included, into a 0.75-ratio RGBA16F target with 4× MSAA: about 50 MB a frame per mirror. The target is allocated even when nothing reflects. `QUALITY.medium` has 1 mirror while the interface's Medium has none, and the defaults come from High (2). The live cost wasn't measured. | High: reflection ratio 0.5, no MSAA (or 2×), no landscape in the reflection, and the target created on first use. Medium and Low: no mirrors, as decided. |
| P1-10 | medium | `web/src/ui/theme.css:73` | The HUD as a whole costs about 4 ms a frame on the M3 (Live 65 → 88 fps without it, Build 39 → 52). On the GPU the frosted blur is within noise, which points at the HUD's per-frame style, layout and compositing (P2-01). On Intel the compositor's blur of large panels over a moving canvas should cost more; not measured (see P2-03). | Low: no blur, a plain tint (as P2-03). Medium and High keep the blur, but no full-screen blur over the moving 3D view (the Pause menu and Modal). Profile the HUD's per-frame work with P2-01. |
| P1-11, P2-04 | medium | `web/src/render/babylon/BabylonRenderer.ts:639` | At speed 5 the lighting updates 20 times a second, and each update thaws the PBR materials for 3 frames. On the dusk path the update's CPU p95 is 4.7–6 ms against 0.3–0.4 ms otherwise; at 3× CPU there are 9–12 long tasks in 12 s and frame p99 is 67–71 ms. P2 measured 1.65 ms per call at `r = 3`, and limiting it to 4 Hz saved about 25–55 ms/s. | Rate-limit by real time as well as game time: P2 suggests at most every 250 ms, P1 every 2–3 s while only the sun's angle changes. Move environment intensity to a scene uniform (a material plugin is document-only, as decided). Low: lighting steps 10 game minutes apart. |
| P1-16 | medium | Chrome's WebGPU under a saturated GPU | With vsync and the frame cap off, WebGPU frames stall (35–43 long tasks in 10 s, frame p95 60–137 ms) while WebGL2 stays smooth (p95 13–17 ms). A GPU-bound M3 with vsync doesn't stall, but a weak GPU is always saturated. P1 left the severity open until it is checked. | Run `?bench=village&quality=low` on an Intel laptop and compare WebGPU with `&renderer=webgl`. |
| P1-13 | low | `web/src/render/babylon/BabylonRenderer.ts:1923` | After tone mapping, image processing still writes an RGBA16F target before FXAA: 16.6 MB a frame on Low. | Do FXAA inside the image-processing pass, or use an 8-bit intermediate; only if cheap in Babylon. |
| P1-14 | low | `web/src/render/babylon/characters.ts:795` | The whole 64-row RGBA32F pose texture (about 250 KB) is uploaded every frame, though 4–5 residents are drawn. | Upload only the drawn residents' rows (`texture.update` with a sub-rectangle). |
| P1-17 | low | Landscape roads (shared by the menu overview and the game) | Between page loads, a far road's lane markings render brighter or darker at the same frozen pose (0.03–0.12 % of pixels, both backends). Probably a frozen material that keeps the lighting of the moment it froze. Not verified. | Thaw or refreeze the shared landscape materials on the switch from the overview to the game, as `setEnvironmentIntensity` does for lit materials. |

P1-07 and P1-11 include P2's matching findings (P2-07, P2-04), which are no longer listed in 2.5.2.

**Checked and fine**
- The benchmark changes nothing for players without its URL flags: the renderer hooks are null checks, its settings are never saved, its save slot is deleted, and GPU timing runs only while F3 or the benchmark is on.
- The night grade is one full-screen pass, about 0 ms on the M3.
- Two runs of the same build differ by at most 1/255 in a handful of pixels, apart from the far road (P1-17), so the reference shots can check that Medium still looks the same.
- Babylon's own WebGPU frame timer reads about 0.4 ms on Apple GPUs and misses the drawing. The benchmark and F3 use the pass span instead (P1-18).
- Not verified: Intel hardware, the cost of a live mirror, and whether CPU throttling slows the sim worker (probably not).

**Corrections to earlier assumptions**
- Lamps at intensity 0 don't just occupy cluster tiles: they cost the full 3.1 ms by day (P1-02).
- The canvas MSAA is pure waste; screenshots without it are pixel-identical (P1-06).
- Mirrors render at 0.75 of the screen, not half, with 4× MSAA, and the target exists even when nothing reflects (P1-09).
- A default new game from the menus saves to about 0.6 MB, not 150 KB; §1 now says so.

#### 2.5.2 Main thread

P2 profiled the production build (Medium, 1920×1080 at DPR 1, a 20-resident town on day 3 at 17:30) as the median of 3 runs, and tried fixes as runtime patches in the dev build. At `r = 3` the main thread is 85–93 % busy at 60 fps, and 75–80 % of that is Babylon's `scene.render`. Medium at 30+ fps holds with about 2× headroom. Low at 60 fps is not met by the main thread, and the Low preset doesn't lower CPU cost.

| Scenario (production, Medium) | Main-thread busy | Frame work avg / p95 (ms) | fps | Frames over 16.7 ms per 30 s |
|---|---|---|---|---|
| Speed 1, r = 1 | 27.5 % | 3.7 / 6.3 | 60 | 0 |
| Speed 1, r = 3 | 85 % | 11.5 / 14.2 | 61 | 15 |
| Speed 5, r = 1 | 40 % | 4.6 / 8.2 | 60 | 1 |
| Speed 5, r = 3 | 93 % | 12.7 / 18.5 | 57.7 | 222 |
| Main menu, r = 3 | 87 % | 13.5 / 16.7 | 60 | 64 |
| Low preset, speed 1, r = 3 (1 run) | 90 % | 13.3 / 16.8 | 58 | 108 |

| ID | Severity | Where | Problem | Proposed fix |
|---|---|---|---|---|
| P2-01 | high | `web/src/ui/HouseholdStrip.svelte:200` | The action-progress bar in each household card is always on screen and animates `width`, as do the SimPanel's need bars. Every UI update restarts a 300 ms transition, so Chrome runs style, layout and paint on every vsync (120 Hz on ProMotion): 72–120 layouts a second. Without the transitions: −59 ms/s at speed 5, `r = 3` with the panel closed, about −140 ms/s with the SimPanel open. | Animate `transform: scaleX(v)` with `transform-origin: left` and `will-change: transform` on the inner span; no visible change. Or drop the transition, since updates arrive at 8 Hz. |
| P2-02 | medium | `web/src/render/babylon/BabylonRenderer.ts:793` | `project()` reads the canvas's `clientWidth` and `clientHeight` for every bubble every frame, after DOM writes, which forces style and layout inside the frame (26.5 ms/s at speed 5, `r = 3`). With a cached size, `project` went from 14.8 to 0.6 ms/s. | Cache the canvas's CSS size (a ResizeObserver or the existing `onResize`) and use it in `project()` and `townFrame`. Do it together with R1-07. |
| P2-03 | medium | `web/src/ui/theme.css:73` | The `backdrop-filter` blur on the 29 glass panels is redone every frame, because the canvas under it changes. Removing it leaves the main thread unchanged but lowers the GPU process's main thread by about 20 points of busy share (speed 5, `r = 3`, SimPanel open). On HD 620-class GPUs it is extra GPU time. | On Low only, replace the blur with a more opaque background (a `.low-gfx` class). Medium keeps the look, as decided. |
| P2-08 | medium | `web/src/game/host.ts:198` | The menu town is drawn at full rate: 87 % main-thread busy at `r = 3` (25 % at `r = 1`), as much as play. | Cap the menu town at 30 fps with the FramePacer (estimated 87 → ~45 %), or stop drawing while a full-screen menu covers it (owner question). Cache the label obstacle rectangles. |
| P2-05 | low | `web/src/game/session.ts:738` | Idea toasts are rebuilt on every UI message (7.7 a second) even when no resident changed: 2.1–2.9 ms/s at `r = 3`, about a quarter of all UI-message work. | `if (changed.sims) ideas.update(game.sims)`. |
| P2-06 | low | `web/src/render/babylon/models.ts:622` | When `buildAhead`'s idle callback fires on its timeout, one model job still runs: 38–55 ms at `r = 3`. It is the only long task seen in play (one per 30 s at speed 5), and a visible hitch on target CPUs. | Run a job on timeout only if it fits, or re-arm after a frame. Better: split model building into smaller jobs. |
| P2-11 | low | JS heap (production build) | Over 10 minutes at speed 5 (about 25 game days) the heap after forced GC grew from 153.5 to 160.2 MB, and DOM nodes from 960 to 1,129. Likely portraits and lists for new residents; not a leak by itself. | If time allows, a heap-snapshot diff over a 1-hour speed-5 run before release. |
| P2-13 | low | `web/src/audio/world.ts:577` | Bird songs are made on the main thread inside the 10 Hz sound tick: 1–2 MB and a few ms per new song (cached afterwards). | Make them in the water-sound worker or in idle time. |
| P2-14 | low | `web/src/debug/threads.ts:55` | F3 "Busy" counts only the frame callback, not style, layout, paint, UI messages or GC: it shows 70 % where the trace measures 80.5 %. | Use a long-animation-frame based busy share, or label it "Frame work share". |

Two of P2's findings are listed with P1's in 2.5.1: P2-07 (the main thread is saturated at `r = 3` and Low doesn't help) with P1-07, and P2-04 (lighting updates 20 times a second at speed 5) with P1-11.

Measured savings at speed 5, `r = 3`:

| Fix | Finding | Saving |
|---|---|---|
| Bars as `transform: scaleX` | P2-01 | 59 ms/s with the panel closed, about 140 ms/s with the SimPanel open |
| Lighting at most 4 times a second (0.2.0 chose at most 10 times a second, 465c287) | P2-04 | about 25 ms/s directly, plus about 30 ms/s from fewer thaws (noisy) |
| Cached canvas size in `project()` | P2-02 | about 14 ms/s, up to 39 ms/s with many bubbles |
| Smaller `buildAhead` jobs | P2-06 | removes the only long task in play |
| Idea toasts only when residents change | P2-05 | 2–3 ms/s |

Together that is about 100–200 ms/s, taking the main thread from 93 % to roughly 75–85 % busy. It is enough for 60 fps at speed 1 on Medium, but not for Low at 60 fps everywhere (P2-07).

**Checked and fine**
- The UI path is cheap: 0.28 ms per update at `r = 1` and 0.9–1.3 ms at `r = 3`. Nothing in Svelte updates per frame, and all large HUD data is `$state.raw` (P2-09). The cost is the layout and paint the UI triggers (P2-01).
- There is exactly one rAF callback per frame, Babylon's, in dev, in production and on the menu (P2-15).
- Allocation and GC are not a problem: 10–15 MB/s, 90 % of it Babylon's per-draw work, under 1 MB/s from app code, and no major GC (P2-12).
- The two autosaves in the 10-minute run caused no long task.

**Corrections to earlier assumptions**
- The main cause of per-frame layout is the always-visible household progress bar, not the SimPanel's need bars (P2-01).
- The per-frame allocation sites listed earlier are each under 50 KB/s and don't matter (P2-12).
- The menu's per-frame label layout doesn't run on the main menu, only on the neighbourhood screens, which weren't measured (P2-08).
- Live drawing is the largest main-thread cost on the target CPUs, not a 0.2 ms extra (P2-07).

#### 2.5.3 Simulation, build and load

P3 timed the sim natively and in the Chrome worker on the largest town the game can hold (64 residents, 34 houses, 2,200 objects). It also tried `wasm-opt`, and measured load time from a production build on a Pages-like server with network throttling. The sim is not a bottleneck, `wasm-opt` doesn't make it faster, and load time is where the wins are.

| Sim speed, big town | M3 Pro | Targets (×3) |
|---|---|---|
| Native, full / lower detail | 98,048 / 181,444 ticks/s | – |
| Chrome worker, speed 5 step (60 ticks), mean | 0.86 ms, 1.7 % of a core | ≈2.6 ms, 5 % of the 50 ms budget |
| Chrome worker, worst step | 9–17 ms | 25–50 ms (one late step) |
| Same step in a 17-resident town | 0.38 ms, 0.8 % of a core | ≈1.1 ms |

| Sim wasm (binaryen 132) | Raw | gzip | brotli | wasm-opt time (M3) |
|---|---|---|---|---|
| As built | 2.85 MB | 830 KB | 595 KB | – |
| Stripped only | 2.47 MB | 798 KB | 572 KB | under 1 s |
| `-O3`, stripped | 2.25 MB | 792 KB | 580 KB | 48 s |

The steady-state speed is the same within ±5 % (Chrome, big town: 0.86 ms per step as built, 0.90 ms with `-O3`).

| Time to menu (Pages-like, COOP/COEP) | Menu, no save | Play → in game | Menu with a save, cold cache | Menu with a save, warm cache | Continue → in game |
|---|---|---|---|---|---|
| Unthrottled | 1.90 s | 1.90 s | 3.17 s | 3.46 s | 0.82 s |
| 50 Mbit/s, 20 ms | 2.32 s | 2.06 s | 4.25 s | 3.22 s | 0.93 s |
| 10 Mbit/s, 20 ms | 3.58 s | 6.07 s | 10.66 s | 3.11 s | 0.83 s |

Before the menu, a cold visit loads 223 requests and 7.27 MB: KTX2 textures 4.1 MB, the IBL `.env` 1.07 MB, JS 1.0 MB in 115 requests, the character prefetch 0.63 MB, manifests 0.24 MB and content 0.13 MB. One run per cell.

| ID | Severity | Where | Problem | Proposed fix |
|---|---|---|---|---|
| P3-01 | medium | `web/src/App.svelte:68` | Boot prepares the whole latest save (read, worker and wasm, world build, warm-up, portraits) before the menu shows. With a warm cache the menu appears at 3.1–3.5 s instead of 1.9 s; cold at 10 Mbit/s at 10.7 s instead of 3.6 s; about 4–5 s later on target CPUs. Continue then takes 0.8–1.0 s. | Show the menu once the menu town is up and prepare the save afterwards, with Continue showing the remaining progress. Or read the save and start the worker before the menu, and build the world on Continue. Measure whether the background build hitches the menu town. |
| P3-02 | medium | `web/src/render/babylon/BabylonRenderer.ts:469` | The 1.07 MB IBL `.env` (already compressed) is fetched only inside renderer start-up, and awaited before the camera and lights: 0.9 s of the renderer phase at 10 Mbit/s. | Start the download at page start (`<link rel="preload" as="fetch" crossorigin>`, or a `fetch()` when the manifest arrives). About 0.4–0.9 s sooner to the menu at 10 Mbit/s. |
| P3-05 | medium | `web/public/isolation-sw.js:12` | Pages serves every file, hashed or not, with `max-age=600`, and the isolation service worker caches nothing. After 10 minutes a returning player revalidates 240–280 requests, and the first visit loads twice because of the service-worker reload (+0.18 s unthrottled). P3 graded it low to medium. | Make the service worker cache-first for hashed `/assets/*` and stale-while-revalidate for `content/` and the manifests, with a cache named per release. Leave `voice/` and `llm/`, which are SHA-pinned. |
| P3-03 | low | `web/src/render/babylon/core.ts` | About 60 shader chunks of 1–17 KB load in 3–4 dependent waves during renderer start, each wave costing a round trip. | Statically import or `modulepreload` the shaders the game always uses (default, PBR, shadow map, post-process, image processing, blur, RGBD decode). |
| P3-04 | low | `web/src/game/session.ts:102` | 55 small JSON requests in two dependent waves (`base.json` and its includes, then `manifest.json` and 20 pack manifests) come before anything else. Cheap at 20 ms latency, costly on slow links and on revalidation. | Bundle them into one content file and one manifest at build time, and keep the sources for modding. |
| P3-06, P3-07 | low | `web/package.json:8` | The sim wasm ships its 384 KB name section (13.5 % of 2.85 MB). `wasm-opt` gives no steady-state speed-up; its size gain is mostly the stripping (gzip 830 → 798 KB stripped, 792 KB with `-O3`). binaryen.js takes 48 s for `-O3` on the M3, so 1.5–3 minutes on CI. | As decided, add `wasm-opt -O3 --strip-debug --strip-producers` with the wasm features passed explicitly, for both wasm files. Not `-O4`, and never `-tnh` or `-iit`. Keep the unstripped wasm as a CI artifact to look up traps. |
| P3-08 | low | `crates/sim-core/src/path.rs:239` | A* has no expansion cap, so an unreachable or far goal floods the 30k-tile town. `start_task` peaks at 3–3.7 ms natively, and worker steps spike to 9–17 ms on the M3 (25–50 ms on targets): one late step, never a stall. | Cap expansions (a node budget or a cost bound), and remember unreachable targets (R3-11). |
| P3-09 | low | `web/src/core/sim.worker.ts:96` | A step that overruns 50 ms is followed by back-to-back catch-up steps (bursts of up to 20 after a stall), and a sustained overload silently runs the game slower than the chosen speed. There is no spiral of death, and with 20× headroom it isn't needed for v0.2.0. | Optional: if the worker stays more than 80 % busy, lower the ticks per step and show the real speed. |
| P3-10 | low | `crates/sim-core/examples/tickbench.rs:8` | The documented quick benchmark panics on today's content, because it loads `base.json` without its includes. `cargo test` only compiles examples, so nobody noticed. | Merge the includes with `sim_core::merge_content`, as the tests do. |
| P3-11 | low | `web/src/render/babylon/core.ts:14` | The SSAO2 pipeline and the geometry buffer (about 17 KB) are in the main chunk for Ultra only, which is going. | Drop the imports together with Ultra. |
| P3-15 | low | `web/public/assets/manifest.json` | The manifest is 1.46 MB pretty-printed (81 KB gzipped, 2.8 ms to parse). | Minify it at build time (16 KB less on the wire). Not worth more. |
| P3-16 | low | `.cargo/config.toml:2` | `simd128` comes from the config's `rustflags`, which an environment `RUSTFLAGS` (for example one added to CI later) would silently replace. | Note it in `pages.yml`, or set the flag in the `wasm` npm script. |
| P3-17 | low | `.github/workflows/pages.yml` | CI runs only on a tag or by hand, with no tests and no performance check, so problems like the broken tickbench surface late. | Add a push and PR job (`cargo test`, clippy) that prints the big town's ticks per second without gating. If build time hurts, use the SHA-pinned native binaryen binary. |

**Checked and fine**
- 60 ticks per step at speed 5 is safe, with about 20× headroom.
- Build config: opt-level 3, fat LTO, one codegen unit, `panic = "abort"`, simd128. No dev wasm can ship: both wasm scripts build in release, CI rebuilds them, and `wasm-pkg` is gitignored.
- Pages gzips JS, JSON and wasm (no brotli), and serves the sim wasm as `application/wasm`.
- No HashMap or SipHash in per-tick paths. `UiSync` costs 0.126 ms natively per update in the big town, about 0.25 % of a core (P3-13). Allocation happens per decision, not per tick (P3-14). Per-tick briefs are 14 % of a lower-detail tick, worth changing only if sim time ever matters (P3-12).
- At speed 5, `structure_version` changed 0 times in a game day, so whole-town room refreshes (0.6–0.76 ms) are rare.
- Voices, the ONNX Runtime wasm, the language model and the GameView chunk don't load before the menu.

**Corrections to earlier assumptions**
- `wasm-opt` shrinks the sim wasm but doesn't speed it up, and most of the transfer saving comes from stripping the name section (P3-06, P3-07).
- The documented quick benchmark doesn't run on today's content (P3-10).

### 2.6 Release readiness

R5 built the site with the Pages base path, served it like Pages and played it. It injected bad saves, measured save sizes, loaded saves from v0.1.0 and from the live site's build, and checked the live site and the GitHub settings (read-only), the licences and the docs. The site works on its subpath with the isolation service worker. The release workflow has never run in its current form, and the language model and the models' licences are missing from the deploy.

| ID | Severity | Where | Problem | Proposed fix |
|---|---|---|---|---|
| R5-01 | high | `.github/workflows/pages.yml:62` | CI never runs `pnpm llm`, so the deployed site has no `llm/` folder and the language model fails with 404s (live: `llm/lfm2.5-350m/tokenizer.json`). | Add `pnpm llm` before `vite build`, cached (R5-12), and ship the model's licence with it (R5-17). |
| R5-02 | high | `.github/workflows/pages.yml` | The release workflow has never run in its current form. The tag trigger, the version check for tags, the `v*` environment rule, and Babylon 9.30, Vite 8.3.4 and the Supertonic golden hashes on a Linux runner are all untested. Any failure would show only after `v0.2.0` is pushed. | Before tagging, run `workflow_dispatch` on `main` and check the deployed site (service worker, Supertonic, language model, the 255 MB file), or tag `v0.2.0-rc.1` with a matching version (owner question). |
| R5-17 | high | `tools/llm/fetch.mjs:25` | The LFM2.5-350M weights ship without their licence, which the LFM Open License v1.0 requires, and Credits doesn't name the model. This becomes a real redistribution once R5-01 lands. | Fetch the pinned `LICENSE` from the same Hugging Face revision (with SHA-256) into `llm/lfm2.5-350m/`, as the voice fetch does, and name it in Credits. |
| R5-18 | high | `web/src/ui/screens/Credits.svelte:9` | In-game Credits says third-party parts are CC0 only. Missing: Supertonic 3 (OpenRAIL-M, whose use restrictions must be passed on), the Misaki dictionary (Apache-2.0), LFM2.5-350M, ONNX Runtime Web (MIT), Tokenizers.js (Apache-2.0), the TalkingHead expressions (MIT) and the texture and mesh decoders. | Add a "Voices and language model" block naming each model and licence, with the OpenRAIL-M restrictions in one sentence and a link to the licence. Add an "Open-source libraries" line that links `assets/CREDITS.md`. |
| R5-03 | medium | live site | The live site is a post-0.1.0 build of `main` (d7c506e, save version 19, KittenTTS) labelled `v0.1.0`. It has no isolation service worker, so it runs without SharedArrayBuffer. Players' saves there are version 17 or 19. | Fixed by the 0.2.0 deploy. Keep old-save fixtures (R5-08). |
| R5-05 | medium | `crates/sim-core/src/save.rs:88` | The starting seed isn't saved, only the generator state, so a bug report can't say which seed a game started from. A loaded save does replay deterministically. | As decided: SAVE_VERSION 23 with `seed` (absent means unknown), a migration line, and the seed in the F8 report. |
| R5-07 | medium | `web/src/game/session.ts:1270` | Autosave is one slot shared by every game. Five minutes of game B overwrite game A's autosave, and two tabs overwrite each other. With no rotation, one bad autosave replaces the only good one. | Key the autosave by game (town seed and household, or a game id in `meta`) and keep 2 rotating autosaves per game (owner question). |
| R5-08 | medium | `crates/sim-core/src/save.rs:1934` | Save migration is tested only with synthetic saves. Saves from v0.1.0 (version 17) and the live build (version 19) load and run cleanly today, but nothing keeps them so. | Commit R5's two fixture saves (about 150 KB each) with a test that loads each, runs a day with `common::check`, and saves and reloads. Add one per release. |
| R5-09 | medium | `web/src/persistence/saves.ts:32` | Saves are best-effort IndexedDB: no `navigator.storage.persist()` and no export or import. Safari deletes script-written storage for sites unused for 7 days, and any browser may evict under storage pressure. The README lists Safari as supported. | Call `navigator.storage.persist()` on the first save. Add Export save (a `.json` download) and Import in Load game (owner question), and mention it in Credits or a FAQ. |
| R5-12 | medium | `.github/workflows/pages.yml:56` | Voices (398 MB from Hugging Face plus the fp16 conversion) and the language model (about 300 MB) are downloaded on every release, so a Hugging Face outage fails it. A tag run's caches can't be restored by the next tag, so most releases also build Rust and assets cold. | Cache `web/public/voice` and `web/public/llm`, keyed on their fetch scripts (which verify SHA-256). Run the dry run on `main` right before tagging so the caches are warm. |
| R5-13 | medium | `.github/workflows/pages.yml:16` | `cancel-in-progress: true` lets a manual run or a second tag cancel a release's deploy. It has happened (run 38051959603). | `cancel-in-progress: false`, so deployments queue. |
| R5-14 | medium | `.github/workflows/pages.yml:51` | CI lacks the clippy gate, the voice tests and `wasm-opt` (all decided). Cargo runs without `--locked`, there is no `rust-toolchain.toml` (a new stable's lints would break the `-D warnings` gate on release day), and there is no `timeout-minutes`. | Add clippy and `pnpm test:voice`, use `--locked`, pin the toolchain, set `timeout-minutes: 45`, and make the lexicon check fail rather than skip when `CI` is set. |
| R5-15 | medium | `web/dist/llm/lfm2.5-350m/model_q4.onnx_dataz` | The Pages artifact is about 523 MB (limit 1 GB), so one more model breaks deploys. A 255 MB file through the Pages CDN is unverified; nothing over 47 MB has been served from the site. A player who turns on voices and the language model downloads about 430 MB, so the 100 GB a month soft limit is about 230 such players. | Check in the dry run that the 255 MB file downloads in full, and fail CI over 900 MB. Consider splitting the file into parts of 100 MB or less. If voices become popular, host the models on Hugging Face or a CDN (owner question). |
| R5-16 | medium | `web/src/content/content.ts:917` | Unhashed runtime files (`content/*.json`, `assets/manifest.json`, `optimized.json`, art by manifest URL, `index.html`) are cached for 10 minutes. Right after a deploy a fresh load can pair new code with old content or manifests, or old HTML with deleted chunks (a white page). The open-tab case is R4-13. | Add the build id as a query on every `public/` fetch except the SHA-pinned `voice/` and `llm/`, or emit those files under `dist/<buildId>/`. Optionally carry the previous deploy's `assets/` into the new artifact. |
| R5-19 | medium | `web/dist/` | Licence notices aren't shipped with the bundled code: Babylon (Apache-2.0), onnxruntime-web and Svelte (MIT), the Inter font (OFL), the Rust crates in both wasm files, and Misaki's licence next to `voice/en-us.lexz`. Only 1 of 405 JS chunks has a licence comment. | Generate `THIRD-PARTY-LICENSES.txt` at build time (`pnpm licenses list --prod`, `cargo about`), copy the Inter and Misaki licences next to their files, and link it from Credits. |
| R5-21 | medium | `web/src/audio/mixer.ts:33` | There is no master volume (the buses go straight to the destination), no "Mute in background" and no "Pause when the window loses focus". | As decided: a master gain node (`masterVolume: 1`), `muteInBackground: true` (fade to 0 on hide or blur, suspend in hidden tabs, see R4-12) and `pauseOnBlur: false`. |
| R5-06 | low | `web/src/game/session.ts:677` | The F8 report has no build or commit id (every 0.1.0-era build says `0.1.0`), no starting seed and no command log. The README doesn't say that a downloaded report loads in production through Settings → Diagnostics → Game state. | Add `__APP_COMMIT__` (CI's `GITHUB_SHA`, else `git rev-parse --short HEAD`) to the report and to Credits, and document the paste path in the README. |
| R5-10 | low | `web/src/game/session.ts:1275` | Closing the tab loses up to the autosave interval of play. There is no autosave on hide or close and no unsaved-changes prompt, and a failed autosave only logs a warning. | Autosave on `visibilitychange` → hidden (at most once a minute; see R4-06), use `beforeunload` while unsaved for over a minute, and toast once when an autosave fails. |
| R5-20 | low | `PLAN.md:332` | The licence statements disagree. PLAN says the voice model is "CC BY 4.0 and CC0" and leaves out LFM; `CREDITS.md:3` says all third-party art is CC0, then lists OpenRAIL-M, Apache and LFM parts; CHANGELOG has no 0.2.0 licence line. The README is right. | Fix `PLAN.md:332` and `CREDITS.md:3` (the voice, dictionary, language model and decoders have their own licences), and repeat the README's licence line in the 0.2.0 changelog. |
| R5-22 | low | `web/src/settings/settings.svelte.ts:112` | Settings have no version field: migrations guess from which keys exist, and the preset migration would be the third guess. Values aren't type- or range-checked. Storage is per origin `krecks.github.io`, shared with any other Pages project of the account. | Add `settingsVersion` with ordered migrations, and clamp numbers and coerce enums after the merge. |
| R5-23 | low | `tools/release/version.mjs:1` | `version.mjs` updates `package.json` and `Cargo.toml` but not `Cargo.lock`. The CI check ignores CHANGELOG, and `github.ref_name` is put straight into the shell script. | Update the four lock entries in `version.mjs`, fail tag builds without a `## <version>` heading in CHANGELOG, and pass `REF_NAME` through `env`. |
| R5-24 | low | `web/index.html:11` | The service worker works on the subpath (verified). Its reload flag is never cleared, so if the worker is ever evicted during a long-lived tab session, the page runs without isolation and doesn't retry. Safari and Firefox weren't tested. | Clear the flag once isolated, and test Safari 18 and Firefox in the dry run. Caching hashed assets in the worker is P3-05. |
| R5-25 | low | `web/public/decoders/` | The decoders are from Babylon 9.29.0 while the engine is 9.30.0. KTX2 loading worked, but the pairing is untested. | Run `pnpm decoders` and update `CREDITS.md:104`. |
| R5-27 | low | `web/public/content/lots/starter.json` | It ships, but only tests and examples read it, and `?lot=` no longer exists. | Move it to `crates/sim-core/tests/data/`, or keep it and drop it from the README. |
| R5-28 | low | `README.md:9` | There is no written release procedure (version bump with `Cargo.lock`, CHANGELOG, dry run, tag, live check). The README's command table lacks `voice`, `llm`, `check:voice`, `check:llm`, `test:voice` and `devices:llm`, and doesn't say that `pnpm dev` downloads about 430 MB of models. | Add a "Releasing" section and complete the command table. |
| R5-29 | low | `.github/workflows/pages.yml:24` | The action versions are the ones that last ran on 2026-10-11; deprecations since then haven't been seen. | Read the dry run's annotations and bump if warned. |
| R5-30 | low | `docs/release-0.2.0-review.md` §1 | The save sizes in §1 were too low. Measured: 150 KB for a new 9-house town, 200–220 KB after weeks, about 1 MB for 64 residents (20 and 83 KB gzipped). | Done: §1 now has the measured figures. |

**Checked and fine**
- The production build on the `/idyll-lives/` subpath registers the isolation service worker, reloads once, is cross-origin isolated, and plays normally.
- Every JS chunk (including the four workers) and all three wasm files are content-hashed. `.wasm` is served as `application/wasm` on Pages, in `vite preview` and through the service worker.
- Voice and language-model files are SHA-256 checked and cached under versioned names, and old caches are pruned.
- The Pages environment allows only `main` and `v*`, and a stray `v…` tag fails the version check instead of deploying.
- Saves from v0.1.0 (version 17) and the live build (version 19) load with nothing dropped, run 2 game days through `common::check`, and save and reload. Loading the same save twice and running a quarter day gives byte-identical saves.
- Save size isn't a problem: the story log is capped at 300 events, and relationships are 45 % of the 64-resident save. Saving takes at most 1.7 ms and loading 8.2 ms natively.
- Bad saves don't slow the boot (1.2–1.4 s to the menu); the problem is what Continue does (R4-08, R5-04).
- Not tested: Safari and Firefox with the service worker, the Pages CDN with the 255 MB and 120 MB files, and the Linux CI build. All need the dry run (R5-02). (The Linux CI build has since passed in the dry run, §3.6; the large files can only be checked on the live site, §4.)

**Corrections to earlier assumptions**
- On the live site only the Supertonic voice paths return 404. The KittenTTS model and `voice/en-us.lexz` load, so voices work there today.
- A 9-house town's save is 150–220 KB, not 115–143 KB (R5-30).
- Pocket TTS was never in a release; 0.1.0 shipped KittenTTS. This affects `PLAN.md` and `docs/design/voices.md`.

**Stale docs** (R5's list, for the docs pass)
- `README.md`: what `pnpm dev` downloads, the command table, hosting through `isolation-sw.js`, the F8 paste path, `?quality=…ultra` and `?lot=starter`, lot files, and "version 0.1.0".
- `PLAN.md`: status, save version 17 (23 after the seed), Babylon 9.29, removed features and old targets, `wasm-opt`, the voice and language-model history, and the licences.
- `CHANGELOG.md`: no 0.2.0 section, and "numbered 0.3–0.20" against PLAN's "0.2–0.20".
- `docs/design/voices.md` (Pocket TTS dates), `docs/design/look.md` (the shadow setting, "up to 4" room lights), `web/public/assets/CREDITS.md:3,104`, `research/voices-multi/README.md:15`.
- This document's "capped by the FPS setting" in §1, once the FramePacer lands.

### 2.7 Questions for the owner

Decisions: see §2.8.

- **R2-01:** are stable object ids still deferred to v0.3? Remapping `follow_up` and `stuck` is a small, safe fix for v0.2.0.
- **R2-05:** restore the traits' starting skills? New residents would start with skills, as designed.
- **R2-09:** may the player send residents to use other households' things?
- **R2-13:** refund the ingredients when a recipe is given up, or keep waiting once paid?
- **R3-04:** may practice go on past a full need while the resident still wants the skill? It makes practice longer.
- **R3-05:** beds for children who leave the crib: NPC households buy one, births and adoption need a bed place, or a notice for the player?
- **R3-17:** a stronger distance penalty at night, or a cap on autonomous trips?
- **R1-04:** at speeds 4–5, continuous motion that cuts path corners, or jumps 20 times a second? A middle way interpolates along straight runs, which needs path corners in the snapshot.
- **R1-05:** above speed 2, switch to the jog clip or accept a capped slide? For children, a slower walk speed or a longer-stride child clip?
- **R1-08:** are the arrow keys meant to move the camera?
- **R1-17:** should world builds be coalesced when updates come in bursts?
- **R4-05:** after a GPU reset, rebuild the renderer silently, or show "Graphics were reset" with Save & Reload?
- **R5-02:** dry run through `workflow_dispatch` on `main`, or a `v0.2.0-rc.1` tag?
- **R5-07:** one autosave per game, with 2 rotating slots?
- **R5-09:** ask the browser for persistent storage, and add save export and import?
- **R5-15:** keep the large model files on Pages (one file is 255 MB; about 430 MB per player who uses voices and the language model), split them, or host them on Hugging Face or a CDN?
- **P1-07, P2-07:** which CPU-side rendering cuts for Low at 60 fps: fewer draw calls or shadow casters, merged static meshes, culling the near nature instances, or snapshot rendering in play?
- **P1-07:** should Low on Intel integrated GPUs default to WebGL2? The WebGL toggle stays under Troubleshooting either way.
- **P1 (HD 620 estimate):** is Medium's 30 fps on an HD 620 to be reached only through Auto resolution, or should those GPUs default to Low (adapter detection at first start)?
- **P1-01, P1-12:** may Low drop the neighbours' house interiors and the lawn beyond the viewed lot?
- **P2-08:** cap the menu town at 30 fps, or stop drawing it behind full-screen menus?
- **P2-03:** lower the glass blur everywhere, or only replace it on Low as decided?
- **P3-01:** show the menu before the latest save is ready, with Continue showing the remaining progress?

### 2.8 Owner decisions

From planning, before the review:
- **R2-01:** object ids stay position-based for 0.2.0 (the remap fix plus the `seen` guard of R2-04). Stable per-object ids are a v0.3 refactor.

From the review gate (2026-10-13):
- **R2-05:** fix the parse bug, so new residents start with the designed skills. Old saves keep theirs.
- **R3-04:** practice runs its full session. Practising a skill the resident wants isn't ended by "needs satisfied"; it runs to the interaction's normal length or until practice fatigue makes interest 0.
- **R3-05:** NPC households buy a bed when a child leaves the crib and there are fewer bed places than people, as with cribs. The player's household gets a notice that the child has no bed, and no automatic purchase.
- **R2-09:** blocked. A directed Use is refused unless the resident has access (their own home, or the guest rules while visiting), and `start_task` checks again.
- **R5-07:** one autosave per game, keeping the 2 most recent. A game is an id created at New game and carried in the save record's metadata.
- **R5-09:** both: `navigator.storage.persist()` on the first save (silently), and Export and Import of a save file in Load game.
- **R1-04, R1-05:** glide and jog, visual only. At speeds 4–5 walkers are interpolated (slight corner cutting is accepted). When the walk clip would need more than its natural rate, the existing `jog` clip plays, for children too. No change to the sim's walk speeds.
- **R2-13:** refund the ingredient cost when a recipe is given up after paying.

Defaults taken by the coordinator:
- **R3-17:** no change in 0.2.0 (balance); documented for later.
- **R4-05:** show "Graphics were reset" with Save and reload, which needs saving without a drawn frame (R4-06). A full renderer rebuild only if trivial.
- **R5-10:** autosave on `visibilitychange` → hidden and on `pagehide` (throttled); no `beforeunload` prompt.
- **R5-15:** verify the large model files in the Linux CI dry run, and split them only if it shows a problem.
- **R1-08:** keep the keyboard camera keys and make them frame-rate independent.
- **Balance:** no other gameplay balance changes in 0.2.0 beyond the decisions above.

Graphics (the owner, 2026-10-13, revised after P1's HD 620 estimate):
- **Three presets:** Low, Medium and High, each knob defined in one table (`web/src/render/quality.ts`). Ultra, Custom and the fine-tune toggles go.
  - **Medium** is everyone's default and keeps today's Medium look: pixel-identical apart from savings nobody can see.
  - **Low** has modest visible trims for older laptops: FXAA, 1024 shadows, fewer night lamps, fewer tufts. It has no 60 fps target.
  - **High** is Medium with 4096 shadows and 2 live mirrors.
- **Settings from 0.1:** an old Low stays Low, and everything else becomes Medium.
- **Resolution:** Auto draws at native resolution up to a budget of about 1920×1080 pixels, and is applied again on zoom and when the window moves to another screen. Custom draws 50–100 % of native. Auto doesn't step down when the game runs slowly.
- **Frame rate:** the FPS cap stays, paced in whole display refreshes so it doesn't judder (the FramePacer).
- **WebGL2:** the switch stays, under Troubleshooting.
- **In scope, the savings nobody can see:** these make Medium cheaper everywhere.
  - No canvas MSAA, and the pixel budget on 4K and Retina screens.
  - Daytime lamps out of the cluster.
  - The shadow map not redrawn when nothing moved.
  - Nature culled in chunks.
  - Drawing held behind full-screen menus.
  - A lighting throttle and a dt clamp.
- **Not a priority for 0.2.0:** these are best effort, documented as future work and not promised.
  - Detecting a weak GPU, and picking Low or WebGL2 on its own at first start.
  - Auto resolution stepping down on slow machines.
  - Low at 60 fps on an Intel HD 620: Low is best effort, and 60 fps on Low is not a 0.2.0 target.
  - Generated LOD models.
- **The target for Intel HD 620/530 laptops** (i7-7500U, i7-8550U, i5-6500 at 1080p) is Medium at 30 fps or more. It is not a release gate.
  - It hasn't been measured on Intel hardware. Scaled from Mac measurements, 0.2.0's Medium is estimated at about 10–21 fps there by day and 8–13 at night (P1's sign-off; before 0.2.0, 6–10 fps, §2.5.1), so it probably falls short until someone measures it on a real laptop.
  - The owner checks it on real hardware with the in-game benchmark (§5).

Later owner decisions (2026-10-13, afternoon, during sign-off):
- **Saves the loader fails on are kept:** a save whose loading traps is kept and marked for this version, not deleted, and a later version tries again. Saves the simulation's own checks find damaged are still removed (c939e8e; this amends R4-08's "corrupt deleted").
- **Tests:** browser tests run headless (still on the real GPU) unless they measure timing, and the browser tools play the game at 1 % master and world volume.
- **Pensions:** residents at or past the retirement age without a job retire, at spawn and at midnight; the job market doesn't hire them, but a job the player gives them sticks. A pension is the larger of `life.basicPension` and 40 % of the weekly pay, and the basic pension is 500 a week in the base content, about the rent and bills of a home of one.
- **v0.3:** new objects are built in Blender and exported as glTF, as the standard (on the branch `v0.3/garden-decor`). That branch and `v0.3/look-study` were moved off `main` and aren't part of 0.2.0.

## 3. Fixed

Every fix was made on an agent's branch and merged into `main` with `git merge --no-ff` (`git log 4cde174..main --first-parent` lists the merges). Each row names the fixing commit; the commit bodies hold the details. Numbers are before → after, A/B against 4cde174 unless another base is named; how each area was measured is stated under its heading.

| Agent | Merge | What it brought |
|---|---|---|
| P1 | c8370d6 | The rendering benchmark (`?bench=`, `tools/bench/`) and GPU time in F3 |
| F2b | bc5ac13 | Input, audio and small web fixes |
| F1a | 85c7e86 | Object ids, the clock, saves, content checks, undo, access |
| F5 | cc88635 | wasm-opt, CI gates and the dry run, licences, build id |
| F1b | b00e999 | Behaviour fixes, beds, refunds, unreachable things |
| T1 | e1833c4 | Century and busy-player long runs, stricter test checks |
| F2a | a8b3388 | Crash screen, render and GPU-reset recovery |
| F2c | 9394264 | Unloadable saves, per-game autosaves, Export/Import, the menu first |
| F4 | a428263 | Animation and frame-clock fixes |
| (century run, seed 19) | 2113e94 | Newcomers arrive grown up |
| (soak runs) | 67244b5 | Two corrected test expectations |
| L1 | 0c561b4 | Load time, service-worker caching, catch-up cap |
| G1 | accaaad | Presets, Auto resolution, FramePacer, menu at 30 fps |
| G2 | 5f72a7c | Dark lamps out of the cluster, nature culled in cells, shadow casters culled and the map redrawn only when something moves, nothing drawn behind full-screen menus, mirrors' memory on demand, the lighting throttle, the pose texture, Low's lawn and woods |
| M1 | dee213f | Bars by transform, bubbles without forced layout, idea toasts only on change, models built ahead in small steps, bird songs off the main thread, F3's Busy |
| V1 (verification) | e32d4e9 | Load game lists a save's own household; no autosave notice over the crash screen |
| F8 | 11b9b7d | Pensions: a basic pension, and retirement at the age without a job |
| F9 | e58a6d5 | A range cooker no longer sells the fridge; a stove alone isn't food at home; once-a-day hours clamped |

Some small fixes went straight onto `main`: f1993e5 (the bed purchase leaves a week's rent), b565f95 (T1-01, §3.7), 954506d (the long runs and the day-1167 test un-ignored), 1a01ce1 (the last structure counters wrap, §3.2), f2b1b43 (code comments on how loading orders furniture and where the benchmark's scenes are), f5f08bf (the feed, §3.7), c939e8e (saves the loader fails on are kept, §3.4) and 1a4018a (the licence texts of the Zstandard decoder and TalkingHead's expressions, §3.6).

### 3.1 Animations and frame pacing (R1)

F4 (merge a428263), except R1-02 and R1-10 (G1, accaaad). F4 measured on an Apple Silicon Mac in Chrome for Testing (headed, WebGPU, 1600×900), with test states dumped from a sim test (`f4-day`, `f4-night`), A/B against a worktree of 4cde174. The clock and speed runs were taken under the bench lock, 3 runs each, and the machine drew about 30 fps during them on both sides.

| ID | What changed | Commit | Before → after |
|---|---|---|---|
| R1-01 | Residents are rebuilt with `keep = 'residents'`: a row keeps its rig while it holds the same resident in the same body (name, slot generation, gender, life stage, body, outfit). Previews keep every row as before. | d6d6fd0 | After a `sell`, every rig is the same object and its clip runs on (before: new rigs, clips restarted). Paused at 01:03, the sleeper is `lie, w 1, cover 1, asleep true` (before: `w 0, cover 0, asleep false`, no blanket or pyjamas). |
| R1-02 | The FramePacer (`render/babylon/pacing.ts`) is Babylon's frame requester: it measures the display's refresh rate and draws every N-th refresh. `engine.maxFPS` is gone. Rules and examples: `docs/design/graphics.md`. | e2ca9d6 | Production build, 120 Hz built-in display, Medium, cap 60, 2× CPU slow-down, frame lengths in refreshes over 8 s: 1×60, 2×327, 3×78, 4×3 → 2×471, 3×6 (58.5 → 59.6 fps). Unthrottled and cap 30 were already even (all 2, all 4). 144 Hz and 75 Hz by simulation only: cap 60 gives 72 and 75 fps. |
| R1-03 | The worker counts the steps it runs and publishes the count with each snapshot; the bridge clocks by it, and a snapshot of the same step replaces the newest one. | 68981ab | Cap 60, three runs. 1.5 s after one 75 ms frame: extrapolated frames 15/30/0 → 0/0/0, walker speed spread 0.31/0.54/0.05 → 0.01/0.01/0.02 m/s. A command every 97 ms for 2.5 s: frames drawing one tick twice 27/37/49 → 0. |
| R1-04 | A resident walking in both snapshots glides as far as walking can take them between the two, `0.075 m × 1.5 × 1.2 × ticks per step × steps`, at least 1.6 m. Path corners are cut (accepted by the owner). | 675557b | Frames in which a walker visibly moves: speed 4 0.62 → 0.95, speed 5 0.58 → 0.88. The start and the end of a walk still jump at speeds 4–5 (§4). |
| R1-05 | When the walk would need more than 3× its own pace, residents jog along: a new `jogAlong` clip (the jog's pose given a 2.5 m/s stride). They walk again below 0.8 of the switch. Visual only, children too. | 76ac264 | Foot slide, height 1.0, no traits: adults at speed 2 27–31 % → 0 %, speed 3 78–79 % → 50–53 %; teens at speed 2 37–40 % → 0 %; children at speed 2 58–59 % → 4–8 %; speeds 4–5 93–98 % → 83–95 %. A short Energetic Speed Walker child at 1×: 25 % → jogging, no slide. |
| R1-06 | While moving, the turn limit and its smoothing scale with the game rate, up to 10×. | fac600f | Angle between facing and travel, 90th percentile: speed 3 14° → 0°, speed 5 57° → 0°. |
| R1-07 | Bubbles, subtitles, room tags and the marker are placed from `onBeforeCameraRenderObservable` (`run(onFrame, onView)`), after the camera took the frame's input. | fc3d3f4 | Camera turned as a drag does, 1.5 s: overlay and drawn position differed in 78 of 78 frames (mean 3.2 px, max 13.3 px) → 0 of 80. |
| R1-08 | The keys stay (decision); each frame their nudges are scaled by `dt × 60`. | af97b84 | ArrowLeft held 1.5 s, plus the 1.2 s glide, two runs: cap 30 −2.49/−2.54 rad and cap 60 −4.43/−5.27 → cap 30 −5.00/−5.22, cap 60 −5.15/−5.28, uncapped −5.02/−5.20. |
| R1-09 | The frame time is clamped once to 100 ms and passed to everything that moves; the never-used `idleOrbit` is deleted. Babylon's own clock, which runs the build effects' particles, is held to 100 ms too since G2's 969699d. | 35d381b, 969699d | One injected 3 s frame: camera swing 0.098 → 0.0015 rad, cloud turn 0.012 → 0.0004 rad. |
| R1-10 | The frame callback gets the timestamp of the refresh it draws for (from the FramePacer), and Babylon's frame time is measured between those timestamps. | e2ca9d6 | Not measured on its own. |
| R1-11, R1-14 | Hands ease by `1 − exp(−(step × 5 + dt × 1.2))`, which is 0 while paused. The creator's framing eases by `1 − exp(−dt × 12)`. | ced877d | Paused with every resident's hands at 0.5: 1 s later 0.57/0.57/0.744 → 0.5/0.5/0.5. The creator's framing wasn't checked in a browser. |
| R1-12 | On the first frame a resident is drawn again, the chosen clip starts fully faded in, and the layers, hands and blanket start at their targets. | 18f1a1b | Hidden 4 s at speed 3, then paused and shown: 8 of 8 residents frozen mid-crossfade (a sleeper uncovered) → 0 of 8. |
| R1-13 | The lift ends at once when the holder walks or the held row is no longer a baby lying in the crib. | 46b38d0 | Frames with the baby in a walking parent's arms: 37 (597 ms) → 0. |
| R1-15 | The renderer gets `reducedMotion`: plants, clouds, the marker and its ring, the Build ghost and build effects keep still. The loading screen leaves without its fade, scrolling isn't smooth, and Busy breathes instead of sweeping. | 604b110 | System reduce-motion on, 2 s: wind strength 1 → 0, cloud turn 0.008 → 0 rad, marker change 0.040 → 0, loading-screen fade 417 → 0 ms. Build effects and the ghost were only code-reviewed. |
| R1-16 | No change needed: Svelte 5.57 already makes the fading loading screen `inert`. | – | 0 clicks swallowed, before and after, through `?state` and through Menu → Continue. |
| R1-17 | `setWorld` builds one world at a time. Worlds that arrive meanwhile wait, and only the newest is built next. | ec04bd6 | Three `setWorld` calls at once: 3 builds at the same time, 51 placements for 17 objects → 2 builds one after the other, 17 placements. |

Also from F4: group conversations and still speech bubbles no longer make garbage every frame (d119bba).

### 3.2 Objects and items (R2)

F1a (merge 85c7e86), except R2-07, R2-08, R2-10, R2-12 and R2-13 (F1b, b00e999). R2's six proving tests are in `crates/sim-core/tests/objects.rs`, and none is ignored now. A new test, `tests/rearranging.rs`, sells, moves, turns and buys furniture at random about every 1.5 game minutes for two game days in two towns, with `common::check` after every edit (about 1,900 edits per town). With the R2-01 fix taken out it panicked within 1.4 s on F1a's branch; on `main` it panics late, about 7.6 s into an 8.2 s run (R2's sign-off). The reliable guard is R2-01's two proofs in `objects.rs`, which fail at once without the fix.

| ID | What changed | Commit | Before → after |
|---|---|---|---|
| R2-01 | `remove_object` shifts residents' follow-ups and their memory of unreachable things along with the ids, and drops them when they name the object that went. `start_task` bails out on an interaction the object doesn't have, and on a place the resident has no access to. | 7a9a87f | R2 measured about 1 crash per 1,200 Build-mode visits that move or sell something. Both R2-01 proofs pass. The 60-day busy-player soak (seed 7) panicked on day 6 at 02:01 on f5fc4b5; it passes now. |
| R2-02 | Sell and move first work out which items each object holds (`items::holder`), then drop or carry only those. | 71fde53 | Both proofs pass (fruit bowl, rug). |
| R2-03, R2-06 | Rotate and Reset size select the object again where it lands (`reselectAt`). Undo and Redo first put down an owned object in hand (`dropOwned`). | 9422394 | Headed Chromium, 1280×800: R pressed 4× at 30 ms intervals, then 3× back to back, turned only the armchair, which stayed selected; the other four objects never moved. |
| R2-04 | The interface sends `seen` (`{def, x, z}`: the object as the view shows it) with use, sell, move, restyle, upgrade and repair. `World::apply_seen` skips a command whose object at that id isn't the same any more. | fac2b31 | `a_repeated_rotate_does_not_move_another_object` passes. A stale move and sell sent with `seen` changed nothing. |
| R2-05 | `startingSkills` is read (decision): new residents start with their traits' skills, and old saves keep the skills they have. | ffd9ac1 | 14 traits gave no skills → their designed ones (`traits_give_their_starting_skills`). |
| R2-07 | On arrival, a broken object isn't used or paid for (decision). A cook goes on to another station, washing up finds another sink or bin, and a diner finds another seat. | 6113e66 | – |
| R2-08 | A repair takes a place at the object when the resident sets off, and tidying up skips a spot someone else is already cleaning. | 9353b98 | – |
| R2-09 | A directed Use is refused unless the thing is at home, in a public place or on the street, or allowed by the guest rules while visiting (decision). `start_task` checks again. | df4cefe | `the_neighbours_things_arent_yours_to_use`, `a_guests_order_goes_when_the_visit_does`. |
| R2-10 | Trimming the queue drops only tasks chosen for the resident, never the player's orders, a paid recipe or the care just added. If nothing else can go, the queue goes over 6. | 9f9231b | – |
| R2-11 | Undo takes back the story events of its edit, and redo tells them again. Undo and redo also keep the goals, routines, free will and purchase answers given after the edit (a planning decision). | 4b56869 | `undoing_an_upgrade_takes_back_its_story`, `undo_and_redo_keep_the_orders_given_since`. |
| R2-12 | With a bin but no sink or dishwasher, clearing away ends at the bin, both of residents' own accord and through "Clear away". With neither, plates still pile up (§4). | 1c2372a | – |
| R2-13 | Giving up a paid recipe refunds its ingredients (decision). | 783dffb | – |
| R2-14 | An item id past `u32::MAX` in a save makes the save corrupt instead of wrapping. Only the ids of items already in the save are checked: the save's next-item counter isn't, and `Items::add` still adds 1 unchecked (§4). | 2f7eb97 | – |
| R2-15, R2-16 | Version counters wrap on purpose, `Items::get_mut` no longer bumps when nothing is found, and `idle_ticks` saturates. The last four `structure_version` bumps in `lifecycle.rs` wrap too (1a01ce1). | c9d58e3, 1a01ce1 | – |
| R2-17 to R2-25, R3-13, R3-14, R3-16 | Content that would break the game is refused at load, with the id and the rule in the message (for example "lazy: walkSpeed 0 must be above 0"; the checks run on the merged content, so only merge errors name files); the rules are in `docs/content-packs.md` ("What content is refused"). At run time, needs clamp to 0–1, and relationships clamp to ±100 on load and in bonds. | 04a216d, 6d2eb3c, 2f7eb97 | `refuses_content_that_would_break_the_game` has one case per rule. The shipped content still loads. |

Also from the rearranging test: someone carrying dishes kept holding them, with nothing to do, when an edit stopped their task (the sink was moved). `World::apply` now has them put the dishes down at once (335a8ab).

### 3.3 Simulation bugs and lints (R3)

F1b (merge b00e999), except R3-01, R3-09, R3-12 and R3-15 (F1a, 85c7e86). F1b ran R3's `long_run_probe` (60 days, `--release`, Apple Silicon Mac) before and after:

| Probe, 60 days | Seed 7 | Seed 11 |
|---|---|---|
| Chats cut short | 242 → 1 | 71 → 0 |
| Quick restarts | 2,210 → 25 | 1,318 → 9 |
| A-B-A flips within 10 minutes | 1,383 → 102 | 980 → 41 |
| A-B-A-B flips within 30 minutes | 579 → 24 | 349 → 6 |
| Fell asleep on the floor | 17 → 7 | 10 → 2 |
| Most chases in one chat | 103 → 0 | 50 → 1 |
| Newborn age | 0.5 (×6) → 0.0 (×6) | 0.5 (×3) → 0.0 (×2) |
| Activity starts per waking hour | 2.57 → 2.23 | 2.69 → 2.26 |
| Activities over in 2 ticks or less | 1,906 → 267 | 807 → 130 |

The probe never saw a value that wasn't finite or was out of range, nor anyone stuck walking.

The "after" figures are from F1b's branch. On `main` at sign-off (R3's re-probe at e32d4e9), seed 7 / seed 11: quick restarts 37 / 648, A-B-A flips 98 / 117, A-B-A-B flips 34 / 28, falls asleep on the floor 12 / 6, activities over in 2 ticks or less 222 / 907, activity starts per waking hour 2.14 / 2.28. Seed 11's jump was the range cooker selling the fridge (§3.7); with F9's fix (rebased on 11b9b7d), quick restarts are 32 / 25 and short activities 177 / 173. The extra falls asleep on the floor fit f1993e5: a neighbour buys a child's bed only if a week's rent is left after.

| ID | What changed | Commit | Before → after |
|---|---|---|---|
| R3-01, R3-09 | The clock counts whole ticks (`TICKS_PER_MINUTE = 20`), and every once-a-day gate asks `clock::at_minute`. `minute_of_day` can't return 1440 any more. Saves were already in whole ticks and don't change. | 78fdf79 | `the_town_still_ages_after_day_1167`: 76 missed ageings on days 1167–1170 → 0. Clock unit tests: days 2 to 10,000 each hit midnight exactly once. |
| R3-02 | Residents about to drop don't choose socials, joining in, calls or texts, and a social that reaches its target while the actor is pressed ends. | 1dab63a | Chats cut short (probe). |
| R3-03 | Waiting and following share one 10-minute limit. The actor follows only while the target is still someone to talk to, and only onto plots they'd chat on. | 5b87d03 | Most chases in one chat (probe). In the 150-day soak on f5fc4b5, 14 residents waited an hour or more following a guest who had left; the soak passes now. |
| R3-04 | Practising a skill the resident still wants runs for the interaction's time, or until practice fatigue or a low need takes the interest away (decision). | e3b0b7d | Quick restarts (probe). |
| R3-05 | When a child leaves the crib or is adopted, and the home has fewer bed places than people past the crib, a household the player doesn't play buys the cheapest bed, if a week's rent is left after paying. The player's household gets a `noBed` story event instead, once per child (decision). | 638500d, f1993e5 | Fell asleep on the floor (probe). |
| R3-06 | The urgent needs that cut other things short count only things with a free place (or the resident's own) that weren't out of reach a moment ago. | ff69241 | A-B-A-B flips (probe). |
| R3-07 | The hand-washing follow-up follows only a use that was under way. | f8cbe10 | – |
| R3-08 | A baby born tonight starts ageing at the next midnight. | 695bde2 | Newborn age (probe), and every century run. |
| R3-10, R3-11 | Residents remember up to four things they chose and couldn't reach (uses, eating, cleaning, repairs, washing up, visits) for the content's retry time. A visit begins only once there's a way there. `Sim::unreachable` replaces `Sim::stuck`. Social and Join aren't in the memory (§4). | 1a7490c | – |
| R3-12 | Scores that aren't numbers are left out of `ai::choose`, and debug builds assert there are none. | 6d2eb3c | `a_score_that_isnt_a_number_never_wins`. |
| R3-15 | A household on an unknown plot, or an expected baby whose parent isn't in town, makes the save corrupt; a facing that isn't finite loads as 0. Saving writes a usable value for any number that isn't finite (`finite`), so one NaN can no longer make a save unloadable. | 2f7eb97 | A unit test for each range check. |
| Lints | The 14 clippy warnings are fixed, two of them with a stated `allow`. `cargo clippy --workspace --all-targets -- -D warnings` gates CI. | bc23c13, 87d8875 | 14 warnings → 0. |

### 3.4 Web robustness (R4)

F2a (merge a8b3388), F2b (bc5ac13), F2c (9394264), and G1 (accaaad) for R4-10. Checked in headed Chromium on WebGPU on an Apple Silicon Mac; R4's sign-off re-check ran headless, still on the real GPU. Hidden tabs were simulated by overriding `document.hidden`, and Alt+Tab with synthetic events.

| ID | What changed | Commit | Before → after |
|---|---|---|---|
| R4-01, R4-02, R4-03, R4-18 | A panic hook in sim-wasm passes on the message and location. The worker guards its step and every request: a trap ends the game for good, and nothing calls into it again. The bridge rejects waiting requests, gives each request 15 s, and has an 8 s watchdog. A crash screen says "The game stopped because of an error. Your last save is safe." and offers Reload, Save a debug report and Main menu. F8 works without the simulation. A `<svelte:boundary>` catches errors in the interface and offers Save and reload. | 0b8c35f, 5bff8ad | With `?debug` and `__crash(...)`: the crash screen appears 88 ms after a panic, 22 ms after a throw, and 8.4 s after a hang (the watchdog). Save is refused at once, and the debug reports save. No false alarm after a 12 s main-thread stall or a page frozen for 15 s. Sim wasm +2,493 bytes. |
| R4-04 | Each frame is caught and the first error logged; 60 failed frames in a row show the crash screen with Save and reload. Waiting for frames gives up after 15 s. | 4ddc3fc | `__crash('render')` at 30 fps: crash screen after 2.4 s, and saving still worked (21 ms). |
| R4-05 | On the engine's context-lost event, a "Graphics were reset" screen offers Save and reload, or Reload (decision). | a0e26f8 | `engine._device.destroy()`: Save and reload stored a save that Continue offered. WebGL context loss wasn't tested. |
| R4-06 | The save's picture is skipped in a hidden tab and waited for at most 1 s; otherwise the last picture is kept. | 0cccdb0 | Drawing stopped: a save that hadn't finished after 5 s → done in 1.0 s without a picture; with drawing, 0.12 s. |
| R4-07 | `project()` returns false unless both outputs are finite, and the sound pans use 0 for a value that isn't finite. | 9877f02 | – |
| R4-08, R5-04 | `Game.fromSave` says why a save fails: `tooNew`, `tooOld` or `corrupt` (decision). A too-new save is kept, marked and skipped; a too-old or damaged one is removed with a short notice; a failed save isn't prepared again in the same session. Since c939e8e (owner's decision), a save whose loading traps is kept too, marked for this version (`failedIn`): Continue skips it, Load game says "This version couldn't open it" and still exports it, and a later version tries again. Saves the simulation's own checks find damaged are still removed. | a6b8a67, c939e8e | Four bad saves newest (trapping, too new, too old, cut short): four notices, Continue lands on the good save, every failed worker closed. sim-wasm unit tests cover each kind, and the v0.1.0 (version 17) and live-site (version 19) saves load. The kept trapping save, checked headless: it and its data stay, Continue picks the next good save, Load is disabled, Export downloads it. |
| R4-09 | A failed start terminates its worker. | 7fc9e9e | Sim wasm blocked: 3 workers alive after a boot and a Continue → 1 started, 1 closed, 0 left. |
| R4-10 | Babylon's `adaptToDeviceRatio` is off. `applyResolution()` sets the scaling from the device pixel ratio and the resolution setting on start, on settings changes and resizes, and from a `matchMedia` resolution query (zoom, another screen). | fb084dc | Real browser zoom at 1.5×, 0.5× and 0.33×: applied again, about 2.07 MP each time. A move between monitors wasn't tested (one display). The automatic re-apply can't be triggered headless (CDP's pixel-ratio override fires no resize or media-query change), so it rests on G1's real-zoom test. |
| R4-11 | Build mode reads Alt and Shift from every pointer event, and forgets them when the window loses focus or the tab is hidden. | 9190394 | Synthetic Alt keydown, blur, focus, then a click: the Wall tool stays, no hint. A real Alt+click still picks. |
| R4-12, R4-16, R5-21 | A master gain node with a Master volume setting. The audio context is suspended while the tab is hidden (voices stopped first), whatever the settings, and resumed when it's visible. "Mute in background" (on) fades out while another window has focus. The context starts from the first click or key and restarts after Safari's `interrupted`; there is no `resume()` on every sound any more, and world sounds fade when updates stop for 2 s. "Pause when the window loses focus" is new too (off). | 67d6937, 5f22b1c | With drawing stopped, the world's peak level went 0.036 → 0 within 3.5 s (before: it kept playing). Safari wasn't tested. |
| R4-13, R5-16 | Every runtime fetch from `public/` carries the build id (`?v=<id>`), except the SHA-pinned `voice/` and `llm/`. A failed import of the game screen goes back to the menu, and `vite:preloadError` or a failed script fetch shows "A new version is available." with Reload. | 690105c, a44ac1a | Dev server: every `public/` request up to New game but `index.html` carries the id (350 of 589 requests). Each blocked chunk shows the banner. |
| R4-14 | `pointercancel`, `lostpointercapture` and window blur end a held press without committing it. Planner chip and calendar drags end the same way. | ed3d3a6 | A wall drag cut off by blur builds nothing, and a normal drag still builds (651 → 654 walls). |
| R4-15 | Held keys repeat only for R, PageUp/PageDown and Ctrl/Cmd+Z/Y. Dropdowns and text fields keep their keys, and Space on a keyboard-focused button presses it. | 61a4454 | 12 Space repeats after a real press stay paused; a held B enters Build once. |
| R4-17, R5-11 | Saves resolve when the transaction commits and reject on error or abort; the cached connection is dropped on failure, on close and on another tab's upgrade. Save storage is IndexedDB version 2 with a `meta` store, so lists read only the details. | 6379656, 43584dc | A refused write now rejects; a real quota failure wasn't provoked. A version-1 store with six saves opens as version 2 with all six. |
| R4-19 | When WebGPU has an adapter but no device, a WebGL2 engine draws instead for the session, and a toast says so. | 40aeaf2 | With `requestDevice` rejecting: backend "WebGL2 · clustered lamps", and the game ran. |
| R4-20, P2-10 | The journal adds events only while it is open, and keeps the last 300. | e6d3d94 | Speed 5 with the journal open: R4 saw 473 entries after 10 minutes and DOM nodes 1,022 → 2,372; now 300 entries from 6.5 minutes on, and DOM nodes 650–675. |
| R4-21 | The director prunes its events and new objects on every note too. | 35ca2fc | – |
| R4-22, R5-26 | The picture cache's name carries the version, and opening it deletes the older versions' caches. | 89ff7eb | Two old caches present → only the current one left, with 539 pictures. |
| R4-23 | A trap in the voice worker's wasm is reported as fatal, and the voice service starts a new worker (at most 3 times a session). | 060cbe7 | Simulated trap: the next line was spoken. Voice wasm +3,734 bytes. |
| R4-24 | The F3 overlay changes only with its own setting; the prefetch timer is cleared only when the session ends; the abort listener runs once; the long-task list is trimmed as tasks arrive. | b1fe004 | A volume change leaves F3 open. |

Also from F2a: debug reports include Babylon's own errors, such as a shader file that failed to load (5586340).

### 3.5 Performance

#### 3.5.1 GPU and rendering (P1)

G1 (merge accaaad) and G2 (merge 5f72a7c). G1 measured on a MacBook Pro M3 Pro in Chrome for Testing 153 (headed), 1920×1080 at DPR 1, with `tools/bench/render.mjs --preview --throughput` (production build, no vsync), median of 3, A/B against a428263 (`main` when G1 started).

| ID | What changed | Commit | Before → after |
|---|---|---|---|
| P1-03 | Low uses FXAA instead of the scene's 4× MSAA; Medium and High keep MSAA (decision). | 2a71891 | In Low's rows below. |
| P1-05 | The shadow map's colour target is R16F instead of RGBA16F, a quarter of the bytes (32 MB a frame written at 2048, 128 MB at 4096, 8 MB at 1024 before). | 0a2bdf3 | Medium's reference shots are unchanged on WebGPU and WebGL2. |
| P1-06 | Neither engine multisamples the canvas (`antialias: false`). | e750d68 | With P1-05: render-target bytes a frame on Medium 216 → 141 MB (night 225 → 150), on Low 120 → 64 MB. |
| P1-09 (part) | Low and Medium reflect in no mirror, High in 2 (decision). | 2a71891 | High's mirror cost wasn't measured: no mirror faces the camera on the benchmark's paths. |
| P1-11 (part), P2-04 (part) | Low updates the lighting every 10 game minutes at speed 3 and up and in quiet hours. | 2a71891 | – |
| P1-15 | Auto resolution draws native pixels up to a budget of 1920×1080 (2.07 MP). | fb084dc | A 1512×982 MacBook window at DPR 2: 3024×1964 → 1786×1160 (59 %). A 3840×2160 screen: 1920×1080 (50 %). |
| P1-02 | A lamp that is off gets a range of 1 cm: it covers next to no tiles of the clustered lighting and still adds nothing, with no shader change. Lit lamps keep their range. (Low's 4 forward lamps by day: not done, §4.) | cae471c | Medium, WebGPU, orbit by day: GPU 8.87 → 6.49 ms, 92 → 119 fps; at night unchanged (8.16 → 8.19 ms). WebGL2 by day: frame 7.59 → 6.89 ms, 132 → 145 fps. |
| P1-01 | The landscape's 44,655 plants are grouped by where they stand (lawn tufts in 32 m squares, flowers in 96 m squares, the woods in 8 sectors round the town), each group a mesh of its own, so the camera leaves out those out of view. Models under 36,000 triangles in all stay one mesh. The placement is generated exactly as before. | 9e801a5 | Medium, WebGPU, orbit by day: GPU 6.08 → 5.17 ms, 118 → 140 fps, triangles 6.01 → 4.56 M (draws 493 → 538). Wide view: GPU 5.98 → 5.52 ms. WebGL2: frame 6.79 → 5.99 ms (orbit), 6.94 → 6.73 ms (wide). |
| P1-14 | Only the pose texture's rows that changed since the last upload (residents drawn, or just hidden) go to the GPU, as one band; none while paused. | a02080f | Residents' update 0.17 → 0.14 ms (WebGPU, with vsync); frame CPU within noise. |
| P1-04 (part) | The game's shadow map draws only the casters that can reach it: those wholly outside its square round the lot (1 m margin) are left out; residents are always drawn. The menus' town draws all its casters as before. | 12f09fa | Medium, WebGPU, orbit by day: draws 540 → 429 (shadow 262 → 152), shadow triangles 1.70 → 0.81 M, 140 → 148 fps; Build mode 62 → 71 fps. WebGL2 orbit by day 163 → 173 fps. |
| P1-04 (part) | The shadow map is drawn only when something in it changed: the lighting step, the caster list, a thing placed, moved or hidden, roofs and storeys, and residents whose pose changed. While paused (Build mode, menus) it isn't drawn at all. In live play, with residents in view, it is still drawn every frame (they breathe), §4. | 71e6c9a | Medium, Build mode (paused): WebGPU GPU 8.54 → 4.96 ms, shadow draws 152 → 0, 71 → 78 fps; WebGL2 77 → 91 fps. Live with residents in view: unchanged (WebGPU 148 → 146 fps, within noise). Residents' shadows follow every pose (checked by eye). |
| P1-11, P2-04 | The lighting is also updated at most every 100 ms of real time: at the fastest speeds 10 times a second instead of 20. At speed 1 its half-minute steps are 500 ms apart anyway. Materials already thaw only for a change one could see. | 465c287 | Medium, dusk path at speed 5, with vsync: lighting updates 20.1 → 9.7 a second. (G2's CPU timings for this step are in its commit body; for the CPU in play see below.) |
| P1-09 (rest) | A mirror's reflection target (about 56 MB of GPU memory at 1080p: a 1024×1024 RGBA16F target with 4× MSAA and depth) is made when a mirror first reflects in its slot, and let go 10 s after no mirror used it, or at once in a world without mirrors. Resolution and MSAA stay: both cuts showed (§4). | 64e6152 | High, the village's bathroom mirror: made when it comes into view, let go 10 s after turning away, reflects as before. |
| P2-08 (rest), P1-10 (rest) | Behind the pause menu, an overlay (Settings, Load game, Credits), the planner and the crash screen, the renderer draws 2 more frames and then none until the menu closes; the frosted glass keeps blurring the last frame. A screenshot, `framesRendered` and a resized canvas still get their frames. | f2ad0c8 | Frames behind the pause menu: every frame (30 in 500 ms, dev build) → 0 in 1 s; the menus' town behind Settings 60 → 0 a second, back to 60 after closing; both backends. Each held frame saves what a frame costs: Medium in orbit by day, about 6 ms of main thread and 5 ms of GPU on the M3. |
| R1-09 (rest) | Babylon's own clock, which runs the build effects' sparkles and dust, is held to 100 ms a frame (`Scene.MaxDeltaTime`), like everything else the game moves by frame time. | 969699d | The first frame after a hidden tab or a stall no longer jumps them ahead. |
| Low's trims | Two new knobs in the preset table: `lawnDensity` (Low 0.5: every other tuft and flower, always the same ones) and `landscapeShadows` (Low false: the woods round the town cast no shadows; the lot's garden trees, the street and the house still do). Medium and High keep everything. | 4516430 | Low, WebGPU, orbit by day: GPU 4.51 → 4.03 ms, triangles 3.73 → 3.44 M, draws 429 → 423, nature instances 44,655 → 24,245. WebGL2 timer 6.05 → 5.55 ms. |

G1's A/B (fps; GPU and CPU in ms a frame; render targets in MB a frame):

| Preset, backend, light | fps | GPU ms | CPU ms | Render targets |
|---|---|---|---|---|
| Low, WebGL2, day | 145.3 → 157.5 | 8.95 → 8.13 | 6.34 → 5.86 | 120 → 64 |
| Low, WebGL2, night | 142.2 → 153.6 | 9.25 → 8.47 | 6.47 → 6.03 | 129 → 73 |
| Low, WebGPU, day | 124.5 → 132.7 | 5.45 → 5.37 | 7.45 → 6.99 | 120 → 64 |
| Low, WebGPU, night | 120.7 → 122.1 | 5.76 → 5.78 | 7.69 → 7.61 | 129 → 73 |
| Medium, WebGL2, day | 129.5 → 139.0 | 11.87 → 10.52 | 7.30 → 6.71 | 216 → 141 |
| Medium, WebGL2, night | 127.6 → 139.8 | 11.90 → 10.58 | 7.40 → 6.67 | 225 → 150 |
| Medium, WebGPU, day | 90.9 → 93.0 | 9.06 → 8.98 | 10.30 → 10.22 | 216 → 141 |
| Medium, WebGPU, night | 89.0 → 88.7 | 8.42 → 9.26 | 10.57 → 10.45 | 225 → 150 |
| High, WebGL2, day | 121.2 → 138.1 | 12.35 → 11.02 | 7.75 → 6.83 | 216 → 217 |
| High, WebGL2, night | 126.9 → 136.0 | 11.87 → 11.29 | 7.41 → 6.96 | 225 → 225 |
| High, WebGPU, day | 89.8 → 89.6 | 8.98 → 8.77 | 10.52 → 10.69 | 216 → 217 |
| High, WebGPU, night | 89.4 → 88.5 | 8.42 → 8.05 | 10.59 → 10.73 | 225 → 225 |

- WebGPU's GPU times move within its ±1 ms run-to-run noise. High now has 4096 shadows, yet costs what the old 2048 High did.
- Medium with vsync, CPU ms: WebGPU 5.72 → 5.31 by day and 6.07 → 5.40 at night; WebGL2 4.08 → 3.40 and 3.95 → 3.58.
- The baseline's `?quality=medium` had 1 live mirror, but no mirror was drawn in G1's 8 s windows, so the Medium rows compare like with like (with the runner's default 20 s window it does reflect: see below).
- **Medium still looks the same.** All 18 reference shots (9 poses × 2 backends) differ from P1's baseline in under 0.15 % of their pixels, by more than 2 of 255 (the most changed 0.095 %). The only differences are the far road's lane markings (P1-17) and one mirror, which the new Medium, like the old in-game Medium, doesn't reflect live.

G2 measured each change against the step before it, on the M3 Pro in Chrome 153, 1920×1080, DPR 1, production build, median of 3 interleaved runs, in the `village` scene, without vsync unless a row says so; the full tables are in each commit's body. The rows above give the main figures. Many runs were noisy (load average 5–6), and the wide view's culling A/B was inconclusive. G2's last measuring campaigns (about 14:40–15:20) partly fell in a window when the Mac ran on battery (14:56–16:27 by its power log), so its own whole-branch table may include battery runs; P1's sign-off re-measured on AC power, and its numbers below take precedence.

0.2.0 against the review's start, measured by P1's sign-off (P1s): MacBook Pro M3 Pro on AC power, Chrome for Testing 153.0.8010.12 headed, 1920×1080 at DPR 1, production builds via `vite preview`, `render.mjs --throughput` (no vsync), median of 3 with base and main alternating. Base is 44809e7 (4cde174 plus P1's benchmark tooling, which 4cde174 lacks), run with `diag=nomirrors` on Medium (see below); main is 5f72a7c. WebGPU GPU is the pass span (for High by day its p50: bogus timestamps spoiled two averages); WebGL2 is the frame p50, as P1 used.

| Preset, light | WebGPU fps | WebGPU GPU ms | WebGL2 fps | WebGL2 frame ms | Draws: main / shadow / mirror | Triangles M | Render targets MB |
|---|---|---|---|---|---|---|---|
| Low, day | 113 → 155 | 6.02 → 4.32 | 161 → 198 | 6.19 → 4.74 | 252/242/0 → 276/146/0 | 6.03 → 3.43 | 120 → 64 |
| Low, night | 113 → 154 | 6.16 → 4.53 | 153 → 179 | 6.31 → 5.19 | 253/242/0 → 278/157/0 | 6.03 → 3.49 | 129 → 73 |
| Medium, day | 89 → 144 | 8.73 → 5.58 | 122 → 160 | 8.11 → 5.61 | 252/242/0 → 277/152/0 | 6.03 → 3.71 | 216 → 141 |
| Medium, night | 82 → 107 | 8.84 → 7.39 | 113 → 157 | 8.38 → 5.59 | 253/242/0 → 278/163/0 | 6.03 → 3.77 | 225 → 150 |
| High, day | 78 → 132 | 10.23 → 6.06 | 107 → 157 | 8.46 → 5.40 | 252/242/49 → 276/152/50 | 6.75 → 4.22 | 216 → 217 |
| High, night | 80 → 91 | 8.62 → 8.24 | 116 → 108 | 8.02 → 8.11 | 253/242/48 → 277/163/54 | 6.73 → 4.31 | 225 → 225 |

- High isn't like for like: the base had 2048 shadows, 0.2.0 has 4096.
- At night Medium barely improved on WebGPU (8.84 → 7.39 ms): lit lamps still cost, and the night range cap for the lamp cluster (P1-02) wasn't done.
- **Mirrors:** with the runner's default 20 s window a mirror does face the camera on the orbit, so the base's Medium (1 live mirror at the time) reflected, about 45 extra draws a frame; P1's 10 s and G1's 8 s windows missed it. A/B runs against the base need `diag=nomirrors` on Medium. 0.2.0's High mirror averages 50–64 extra draws and 0.5 M triangles a frame; its GPU cost was within noise, except +3 ms on WebGL2 at night (inconclusive, as it reflects for only part of the window).
- **Medium still looks the same.** Per change, against the build before (in a window): at most 0.12 % of pixels off by more than 2 of 255, all on the far road's lane markings (P1-17). The merged branch against `main` (both headless): at most 0.115 %, the same far-road noise. V1's 18 shots against the baseline: at most 0.118 %, the far road and the bathroom mirror.
- **CPU in play:** these savings are GPU savings. At 3× CPU slow-down (with vsync) P1s found no CPU gain in play: 8 of 12 cells were 0.6–3.9 ms a frame worse. The main scene has about 970–1,370 meshes against 650; the nature cells add 24–25 main-pass draws in the orbit (+10 %) and 57 in the wide view (+21 %), while the shadow pass drops by 79–96. P2's sign-off saw the same pattern (§3.5.2). G3 is working on it. *(G3's numbers to come.)* <!-- G3 -->

#### 3.5.2 Main thread (P2)

G1 (accaaad) for P2-08, and M1 (merge dee213f) for the rest. M1 measured on the M3 Pro in headed Chrome for Testing (Playwright's Chromium 1243) on WebGPU, 1920×1080, Medium, with the CPU slowed 3× (CDP), at speed 5, with P2's method (`measure.mjs`, the `p2-evening` save). Each figure is a median of 3, the baseline (a428263) and M1's build run alternately under the bench lock, after a 30 s warm-up so the catalog pictures drawn early in a game stay out of the window. The total busy share swings by ±5 points between runs on that shared machine; the figures that isolate the changes (work outside the frame callback, layouts, interface updates) are steady.

M1's branch as a whole (production build, speed 5, 3× CPU):

| Measure | Panel closed | SimPanel open |
|---|---|---|
| Main thread busy | 84.7 → 74.0 % | 96.4 → 82.1 % |
| Work outside the frame callback | 204 → 143 ms/s | 273 → 205 ms/s |
| Frame p95 | 16.1 → 15.0 ms | 17.2 → 15.5 ms |
| Interface updates (handler and Svelte's flush) | 8.7 → 6.0 ms/s | 11.8 → 8.6 ms/s |
| Layouts a second | 108 → 11 | 82 → 10.5 |
| GPU process busy | 27 → 15 % | 35 → 15 % |
| Longest idle callback | 19.5 → 4.5 ms | 19.4 → 4.7 ms |

With the SimPanel open the frame rate went 57.8 → 60.8 fps. M1's runs saw no long task; the baseline had one of 55 ms in one of three panel-closed runs. At speed 1 with the panel closed: busy 68 → 60 %, layouts 99 → 8 a second, interface updates 5.9 → 3.7 ms/s.

**After G2's merge** (P2's sign-off, at 5f72a7c): G1's and M1's figures were measured before G2's work was merged, and G2's merge raised the CPU cost of a frame. WebGPU draw calls per frame in play went 87.5 → 102.7 at speed 1 and 89.1 → 104.7 at speed 5, and the menus' town's replayed draws 48k → 75k per 5 s (+55 %). At 3× CPU and speed 5 the frame p95 went 18 → 37 ms, with 19–60 long tasks (50–142 ms) per 30 s, and the menus' town at 30 fps took 25.7 ms of frame work instead of 14.6 (82 % busy instead of 49 %). P2 measured on battery, so these timings are only indicative; the draw counts don't depend on power. The suspect is the nature cells (9e801a5), which put every cell in view from the menu overview. G3 is fixing it, and its re-measure replaces the "no long tasks" of P2-06 and the menu's 45 → 24 % of P2-08 below. *(G3's numbers to come.)* <!-- G3 -->

| ID | What changed | Commit | Before → after |
|---|---|---|---|
| P2-08 | The town behind the menus is paced at about 30 fps, by the cap's rounding rule (as in §5's Cap 30 column: 36 on a 144 Hz display, 37.5 on 75 Hz, 33 on 165 Hz), in whole refreshes like the cap; the flight into the game runs at the game's rate. | f10d9e5 | Main menu on the M3 Pro, 120 Hz, production build, 8 s, main-thread task time from CDP: 60 fps and 20 % busy → 30 fps and 14 %; at 3× CPU slow-down, 45 % → 24 %. Every menu frame now stays 4 refreshes on screen. |
| P2-01 | The household cards' action bars and the SimPanel's need and action bars move by transform, which the compositor animates: `scaleX` for action bars, `translateX` for need bars (so a bar's rounded end stays round). They used to animate `width`, and each update (about 8 a second) restarted a 300 ms transition, so the page was laid out and repainted on every display refresh. | d9b58d1 | Measured alone, panel closed: busy 68.7 → 61.4 %, work outside the frame callback 169 → 113 ms/s, layouts 119 → 11 a second (16.5 → 3.5 ms/s), the GPU process 23 → 16 % busy. Same look: old and new bars side by side, at rest and held at 0, ¼, ½ and ¾ of a transition, differ only in the anti-aliasing of a bar's end pixel (and in the shape of the 1–3 px sliver of a need under 3 %). |
| P2-02 | The renderer keeps the canvas's size, updated by a ResizeObserver, for placing bubbles, panning sounds and the menu town's camera, instead of reading it from the page after the interface has changed it. | b70beaf | Dev server, 25 s CPU profiles, two runs each: `project` 23.7 → 0.4 ms/s, the bubbles' update 19.2 → 3.1 ms/s, the world sounds' update 8.9–11.4 → 1.0–1.3 ms/s (with P2-13). Forced style and layout inside the frame (trace, full CPU speed): 68 → 2 a second, 11.3 → 0.7 ms/s; what's left is a new bubble's pop. |
| P2-05 | Goal ideas, things to buy and wishes are worked out again only when the residents' names, the parts of their plans that hold ideas, or (for repairs) the things at home changed. A plain "a resident changed" check would have saved almost nothing: some resident changes in nearly every update. | 519dd59 | Dev server: the toasts' update 2.0–2.5 → 0.1–0.2 ms/s. |
| P2-06 | A model built ahead for another lot goes part by part, then group by group, at most 4 ms at a time, and goes on in the next idle moment (waiting at most 200 ms for one). Its pieces stay hidden until it is whole, and the next model waits for it. Models wanted on screen are still built at once. | 61f86c1 | Longest idle callback 19.5 → 4.5 ms (production build), 28–29 → 4.5 ms (dev server); long tasks in play 0 or 1 a run (55 ms) → none. A tree or park table used to take 10–15 ms on the M3 and 35–55 ms with the CPU slowed 3×. A model still being built ahead when the camera arrives may now appear up to a few hundred ms later. |
| P2-13 | The bird species and their synthesis move to `audio/birdsong.ts`, and the worker that makes the water sounds makes the songs too (made on the spot where there are no workers). A bird whose song isn't ready yet sings a quarter of a second later. | 9ebcec5 | Removes 20–26 ms bursts inside the frame with the CPU slowed 3×. Every bird's three versions at 44.1, 48 and 96 kHz are identical sample for sample, old and new (54 of 54). |
| P2-14 | F3's "Busy" is now "Busy (frames)": the frame callback's share plus the browser's style, layout and paint after it, measured only while the overlay is open. "Frame work" shows the callback's own share. (A share from long-animation-frame entries isn't possible: they only cover frames over 50 ms.) | 4fd8694 | Against a trace of the same seconds (dev server, speed 5): 29 % shown for 32 % measured, and 93 % for 96 % with the CPU slowed 3×. Interface updates between frames aren't counted, as its tooltip says. |
| P1-10 | No change of its own: P2-01 removed the per-frame layout and P2-02 the forced layouts, which were most of what the interface cost a frame. The frosted blur is unchanged (P2-03, §4). | d9b58d1, b70beaf | Benchmark without vsync, 2 runs each: the interface cost 5 fps (88 against 93 without it) → within noise (94 against 95). |

#### 3.5.3 Simulation, build and load (P3)

F2c (P3-01), F5 (P3-06, P3-07, P3-10), L1 (the rest) and G1 (P3-11). L1 measured production builds on the M3 Pro in headed Chrome 153 at 1920×1080, DPR 1, served by a Pages-like Node server (HTTP/2, COOP/COEP, `max-age=600`) with CDP network throttling: median of 3, a fresh browser profile each run, under the bench lock, A/B against a428263. L1's "menu" is the moment the loading screen is gone and New game shows, so its absolute times aren't comparable with P3's in §2.5.3.

| L1, first visit | 10 Mbit/s, 20 ms | 40 Mbit/s, 80 ms | Unthrottled |
|---|---|---|---|
| Menu, cold | 3.44 → 3.14 s | 2.39 → 2.02 s | 1.35 → 1.28 s |
| Menu, a save present, cache cleared | 3.43 → 3.17 s | 2.39 → 2.05 s | – |
| Menu, warm cache | 1.29 → 1.38 s (within the ±0.06 s spread) | 1.28 → 1.30 s | – |
| Continue → in game, cold | 6.16 → 6.18 s | 2.24 → 2.28 s | – |
| Requests before the menu | 219 → 164 | – | 262 → 207 |

| ID | What changed | Commit | Before → after |
|---|---|---|---|
| P3-01 | The menu shows as soon as its town is up, and the latest save is prepared behind it once the menu has settled. Continue clicked earlier shows the remaining progress. | 4f7d699 | M3 Pro, headed Chrome for Testing 153 on WebGPU, 1920×1080, Pages-like server with COOP/COEP, a saved town, warm cache, 3 interleaved runs against 4cde174: menu 2.83 → 1.42 s. Continue clicked as the menu appears: page start to in game 3.59 → 3.00 s. Continue 6 s after the menu: 0.54 → 0.57 s. No save: 1.69 → 1.78 s (within the spread). The cost: 2–3 frames of 100–175 ms in the first seconds on the menu (none before). |
| P3-02 | A script in `index.html` starts the sky's 1.07 MB `.env` download with the page, and the renderer takes the bytes. (A `<link rel=preload>` downloaded it twice under the isolation service worker.) | f9395df | Cold menu at 10 Mbit/s: 3.44 → 3.31 s. |
| P3-04 | The build writes `bundle.json`: content, house templates, asset manifests and the index of compressed copies, minified, in one file (3.1 MB, 308 KB gzipped). The files still ship one by one for modders, and the dev server and `?pack=` load them as before. | 16d27ef | 56 requests in three dependent waves → 1. Cold menu at 10 Mbit/s: 3.31 → 3.16 s. Parsing costs about 4 ms more. |
| P3-05 | The isolation service worker keeps the build's hashed files (scripts, styles, wasm, fonts) in a cache of the build's own, and a new build's worker deletes the old cache. | 3404e97 | Returning after Pages' 10 minutes, each response 50 ms late, HTTP/2: menu 1.54 → 1.34 s, requests reaching the server 219 → 62. With no delay: 1.22 → 1.27 s. |
| P3-06, P3-07 | Production builds run `wasm-opt -O3 --strip-debug --strip-producers` on both wasm files (binaryen 132.0.0, an exact-pinned devDependency; features passed explicitly; never `-tnh` or `-iit`). CI keeps a wasm with names as an artifact for 90 days: it is the un-optimised file (2,108 functions against 1,347 shipped), so its function numbers don't match crash stacks from the site (§4). | 73b38ab, 87d8875 | Sim wasm 2,851,080 → 2,254,731 bytes raw, 835,216 → 795,765 gzip -9, 594,890 → 580,114 brotli 11. Voice wasm 157,808 → 133,668, 63,968 → 59,254, 53,773 → 49,909. Together 620 KB smaller raw and 44 KB smaller gzipped. Speed unchanged: 110,757 vs 110,938 ticks/s (P3's V8 harness, the one interleaved pair not disturbed by other builds). `vite build` takes 60 s with wasm-opt from cold, 8–11 s cached (M3 Pro, busy with other builds). |
| P3-09 | After a late step, at most two steps catch up; a longer backlog is dropped. | 1ffa502 | Big town at speed 5, dev server, the worker's timer hooked: blocked 300 ms, 7 steps back to back → 3; blocked 2 s, 2 → 3; normal running 20.3 steps/s in both. |
| P3-10 | tickbench merges the content's includes, and CI runs it. | d4a5107 | A panic → 485 ns/tick with socials, 482 without (M3 Pro). |
| P3-11 | SSAO2 and the multiple-render-target imports left the main chunk with Ultra. | 2a71891 | Not measured on its own. |
| P3-15 | The build writes every asset manifest without its spaces; `public/` keeps them readable. | ab895d1 | Base manifest 1,464,025 → 592,947 bytes (82.9 → 66.0 KB gzipped); all 21 manifests 243 → 209 KB gzipped. |
| P3-16 | sim-wasm and voice-wasm stop with a compile error when built for wasm32 without `simd128`. | ebc9029 | Checked with a `RUSTFLAGS` in the environment. |

P3's sign-off measured 0.2.0 against the review's start: M3 Pro, Chrome for Testing 153 headed, 1920×1080 at DPR 1, two lock holds with 4cde174 and `main` (5f72a7c) alternating, on L1's Pages-like server (HTTP/2, the isolation headers, 10-minute caching, gzip, 20 ms latency) at 10 Mbit/s with a fresh profile per run, median of 3 (spreads under 0.1 s). Native timings are medians of 6 quiet runs (tickbench on 4cde174 is main's fixed copy, as the original panics).

| | 4cde174 | 0.2.0 (`main`) |
|---|---|---|
| Sim wasm in `dist`, raw / gzip -9 / brotli 11 | 2,851,035 / 830,273 / 594,880 | 2,298,877 / 806,901 / 590,303 |
| Voice wasm in `dist` | 157,808 / 63,450 / 53,792 | 135,997 / 59,953 / 50,807 |
| `tickbench`, with / without socials | 486 / 489 ns per tick | 513 / 507 ns per tick |
| `big_town_speed` (2 days), full / lower detail | 92,642 / 169,714 ticks/s | 91,035 / 164,742 ticks/s |
| First visit, no save → menu | 3.36 s | 3.17 s |
| Play (new game) → in game | 5.94 s | 5.98 s |
| A save, empty cache → menu | 10.05 s | 3.14 s |
| Continue clicked at once → in game | 0.81 s | 6.21 s (the save is still being prepared) |
| Page start → in game, empty cache | 10.88 s | 9.35 s |
| A save, warm cache → menu | 2.72 s | 1.37 s |
| Page start → in game, warm cache | 3.51 s | 2.95 s |
| Requests before the menu (a save, empty cache) | 240 | 137 |

- The sim wasm grew to 2,901,413 bytes before wasm-opt, with the new fixes and content; wasm-opt strips the name and producers sections and keeps the features.
- The sim is 2–5 % slower natively, from the new sim fixes and content. The about 20× headroom at speed 5 is unchanged in practice.
- With an empty cache the menu comes 6.9 s sooner and page start to in game is 1.5 s faster, but a Continue clicked at once now waits for the save to be prepared (P3-01's accepted cost, with the menu's stutter in §4).
- Also checked: a reload without the isolation headers sent none of the hashed files to the server (130 cached, 133 and 5.8 MB once in game); an environment `RUSTFLAGS` stops the build; the early sky download starts at 0.3 s instead of 1.3 s; SSAO is gone from the main chunk (1.76 MB, 512 KB gzipped). L1's choices on P3-03 and P3-08 were right.

### 3.6 Release readiness (R5)

F5 (merge cc88635), F1a (R5-05, R5-08), F2c (R5-07, R5-09, R5-10, R5-11), F2b (R5-21) and F2a (part of R5-16). The README, PLAN, CHANGELOG and docs items (the F8 paste path of R5-06, R5-20, R5-28 and R5's stale-docs list) are this docs pass.

| ID | What changed | Commit | Before → after |
|---|---|---|---|
| R5-01 | CI runs `pnpm llm` before the build. | 87d8875 | – |
| R5-02 | The workflow can be run by hand on any branch: `gh workflow run pages.yml --ref <branch> -f deploy=false` builds and checks everything, and publishes only with `-f deploy=true`. Release tags publish as before. Running it before tagging is part of the release. | 87d8875 | The first dry run (run 38393256436, on `release/0.2.0-ci` at f5f08bf, `deploy=false`): the build job passed in 9 min 23 s on Linux, and the deploy was skipped. Its annotations are warnings, not failures (§4, R5-29). |
| R5-03 | Fixed by the 0.2.0 deploy. | – | – |
| R5-05 | SAVE_VERSION 23 stores `seed`, the seed the game started from (absent: 0, unknown). `World::seed` and the wasm `Game.seed` expose it; nothing in the web reads it yet, but F8 reports carry the whole save. | 2f7eb97 | – |
| R5-06 | `__APP_COMMIT__` (the short `GITHUB_SHA` in CI, else `git rev-parse --short HEAD`) shows in Credits ("Version 0.1.0 (build abc1234)") and in the F8 report. | 7f5ea0d | – |
| R5-07 | Each game has an id from New game, carried by its saves, and autosaves into two slots of its own (`autosave:<game>:0` and `:1`), always overwriting the older one (decision). Saves from before ids are each a game of their own. | 42ca6f6 | Three autosaves of one game left two; a new game got its own two. |
| R5-08 | `tests/old_saves.rs` loads real saves from v0.1.0 (version 17) and from the live build after it (version 19), plays a day with `common::check`, and saves and reloads them (fixtures in `tests/saves/`). | 2f7eb97 | – |
| R5-09 | The first save of a page visit asks for persistent storage, silently. Load game has Export on every save and "Import a save…", which the simulation checks in a worker of its own (decision). Load game is always open, to import into an empty list. | ee3e71a | Export, delete, then import gives the save back and it loads; too-new, too-old, damaged and non-save files are refused with their message. |
| R5-10 | Autosave also runs when the tab is hidden and on `pagehide`: at most once a minute, not in a game's first minute, and only with autosave on. A failed autosave says so once. No `beforeunload` prompt (decision). Only the game being played autosaves. | 0cccdb0 | Hiding the tab autosaved at once, without waiting for a picture. |
| R5-12 | CI caches `web/public/voice` and `web/public/llm`, keyed on the fetch scripts (which check every SHA-256). | 87d8875 | – |
| R5-13 | `cancel-in-progress: false`: a second run waits. | 87d8875 | – |
| R5-14 | CI gates: clippy with `-D warnings`, the voice tests after `pnpm voice` (they fail instead of skipping when `CI` is set), tickbench must run, and `--locked` on every cargo call, the npm scripts included. `rust-toolchain.toml` pins 1.97.0 with clippy, rustfmt and wasm32. `timeout-minutes` is 45 for the build and 20 for the deploy. | 87d8875, 57dfe1a, 495d7e0 | – |
| R5-15 (part) | CI lists `dist` by size and fails over 900 MB. Whether Pages serves the 255 MB file in full is checked on the live site (§4). | 87d8875 | The dry run's site: 489 MB (assets 56 MB, voice 181 MB, language model 248 MB; the largest file `llm/lfm2.5-350m/model_q4.onnx_dataz`, 255.5 MB), under the 900 MB guard. |
| R5-17 | `pnpm llm` fetches the LFM Open License, pinned by SHA-256, from Liquid AI's own repository (LiquidAI/LFM2.5-350M at 9e6c6ccf; the ONNX export has no copy). | 89b6dbc | – |
| R5-18, R5-20 (part) | Credits has "Voices and language model" (each model and its licence, the OpenRAIL-M use restrictions in one sentence, a link to the licence) and "Open-source libraries" (links to `THIRD-PARTY-LICENSES.txt` and `assets/CREDITS.md`). `CREDITS.md` says only the art is CC0. | 5f6bd01 | – |
| R5-19 | Production builds write `dist/THIRD-PARTY-LICENSES.txt` (`tools/release/licenses.mjs`): the 6 npm packages found in the bundles, the 18 crates in the two wasm files, Inter, the meshopt decoder and the three model licences, each text once (79 KB). The build fails if a licence text is missing. Misaki's licence ships as `voice/misaki-LICENSE`, and Inter's as `assets/inter-LICENSE.txt`. Since 1a4018a it also prints the Zstandard decoder's (`decoders/zstddec.wasm`: MIT, with Zstandard's BSD-3-Clause notice) and TalkingHead's (MIT, the expressions adapted in `face.ts`), kept in `tools/release/texts/`. | 59d37cb, 89b6dbc, 1a4018a | – |
| R5-23 | `version.mjs` rewrites the four workspace entries in `Cargo.lock`, and its check fails on a stale lock. Tag builds need a `## <version>` heading in CHANGELOG, and `ref_name` reaches the script through `env`. | 6173fdf, 87d8875 | – |
| R5-25 | `pnpm decoders` against Babylon 9.30.0: only `babylon.ktx2Decoder.js` changed. | cafa0a7 | – |
| R5-30 | Done before the fixes (§1). | – | – |

### 3.7 Found and fixed during the work

| ID | What was wrong | Commit | Check |
|---|---|---|---|
| T1-01 | Partners in a full town (64 residents) kept trying for a baby that couldn't come: `could_conceive` didn't check the town's limit, which `room_for_one` does. Only towns of 16 or more houses reach 64. | b565f95 | The 60-day busy soak (seed 7) failed on it once R2-01 was fixed; it passes now. A test in `tests/babies.rs`. |
| Newcomer babies | A newcomer family's "grown child" could arrive as a baby, with no crib and nothing to do: the youngest age came from the first life stage, which became the baby's when the young stages were added. In the century run of seed 19, Hugo stood where he arrived from day 122 to day 126. Newcomers now start at the first grown-up stage (18), and the random draws are unchanged. | de0a880 | `newcomers_are_grown_up` (seeds 1–40); the seed-19 century runs pass. |
| Departures | Leaving town now also ends answering the door to the leaver, calls with them and waiting for them, and ends a pregnancy they're a parent of. | 1631442 | – |
| Dishes in hand | See §3.2. | 335a8ab | – |
| Range cooker (R3's sign-off) | Buying a range cooker sold the fridge: a replacement sold the cheapest thing of the same group that shared any one interaction id, and the fridge and the cooker share only `gourmet`. Without a fridge every recipe lacks its `ingredients` station, yet `home_needs` still counted the cooker's recipes as food, so hunger stayed urgent, everything else was cut short, and takeout never came. Seen in neighbours' homes on seeds 7 and 11, and for the player through an accepted request; it came in on 2026-10-12 with residents' wishes (4e7007c). Now an object is only replaced by one that has every interaction id, cooking station and leftovers it had, and a recipe counts as food at home only when the home's working stations can make it (this also covers a broken fridge). In the shipped catalog 174 of 579 allowed replacements took a use away: a gas stove sold the fridge, a wall oven or cooktop a stove, a dining table a chair. | 045ce31, 9abc764 | Probe, 60 days, release: seed 11 quick restarts 648 → 25, activities over in 2 ticks or less 907 → 173, longest run of them 39 → 3; seed 7 37 → 32, 222 → 177, 8 → 4. Every home keeps a fridge on seeds 7, 11 and 19. Tests in `wants.rs` and `cooking.rs`. |
| Once-a-day hours (R3's sign-off) | The hours for rent, the job market and the planner's morning review weren't range-checked; they are now clamped to 0–23.9, like moving and newcomers. Shipped content is in range. | 6cdcae7 | `once_a_day_hours_stay_within_the_day`. |
| Pensions (owner's decision) | Residents at or past the retirement age (65) without a job had no pension and lost about 250 a week. Now anyone created at or past the age starts retired; a working resident retires on the midnight they reach it; anyone at or past it without a job retires at midnight on the basic pension; the job market doesn't hire them, and a job the player gives sticks (quitting it brings the basic pension back). A pension is the larger of `life.basicPension` (500 a week) and 40 % of the weekly pay. No save format change. | 5114eee, cf307b1 | Base content, seed 7: rent and bills for one are 468, 492 or 516 a week by house type; a lone retiree on 500 gains 8 a week with free will off and loses about 13 with it on (before: about −250). 7 tests in `tests/retirement.rs`; the long runs pass unchanged. |
| Load game members (V1) | A save's members, shown in Load game and in a debug report, were every resident the interface knew of, the whole town (30–40 names for a household of two), since 0.1.0. Now they are the player's household. | 2caf64c | A save and an autosave list 1 of 19 residents. |
| Autosave over the crash screen (V1) | After the simulation stopped, autosave kept trying and showed "Autosave failed: … Save from the pause menu…" over the crash screen. It no longer tries once the simulation has stopped. | bbb9ee4 | With a 3 s autosave, after `__crash('panic')`: 0 tries and no notice in 8 s (before: 3–12 warnings and the notice). |
| Feed news | Found by M1: a faded entry in the story feed was only see-through, and up to three stayed clickable until the next news arrived. With reduced motion on, the 12 s fade ran in 0 ms, so entries were invisible from the start. The fade now waits 10.2 s, lasts 1.8 s and ends hidden. | f5f08bf | In Chrome: hidden and unclickable at 12.5 s; with animation durations forced to 0, an entry shows until 10.2 s. |

### 3.8 Tests added

- **Stricter checks** (T1, e1833c4): `common::check` asserts that every number is finite and in range (needs, skills, ages, mood, grades, relationships, wear, dirt, funds, items), and that every task in hand or queued, and every follow-up, names something that exists. `common::Watch` reports a resident stuck in one state for too long.
- **Century runs:** `tests/century.rs` runs the 9-house town for 200 game days (100 years) on seeds 7, 11 and 19, with lower detail on and off (6 ignored tests, `--release`).
  - It checks ageing at every midnight, newborns at age 0, 8 to 64 residents, and the story.
  - It saves and reloads at days 50, 100 and 150.
  - `the_town_still_ages_after_day_1167` runs with the suite.
- **Busy-player soaks:** `tests/soak.rs` runs the 34-house big town (64 residents, about 2,200 objects) for 60 days (seed 7) and 150 days (seed 11).
  - Every 1–4 game hours a seeded player edits the home or gives orders.
  - The town is saved and reloaded at a third and at two thirds.
- **Others:** `tests/rearranging.rs` (§3.2), `tests/old_saves.rs` (R5-08), `tests/clock.rs` (R3-01), `tests/objects.rs` (R2), and R3's probes as behaviour tests (6f4be33).
- **Results on f5fc4b5, before the fixes:** every century run failed on R3-08, the day-1167 test on R3-01 (76 missed ageings), the 60-day soak on R2-01 (a panic on day 6), and the 150-day soak on R3-03 and R2-01.
- **Results with the fixes:** all 6 century runs and all 4 soak runs pass. These are release builds on an Apple Silicon Mac, as reported by the agents that fixed the last failures. A century run takes 13–14 s with lower detail and 19–23 s without; the 150-day soak takes about 44 s.
  - Two soak expectations were wrong, not the sim: no birth is possible in the full 60-day town, and a guest keeping the host company for a whole visit isn't stuck (2d2dc17, fdcca34).
- **The suite:** `cargo test -p sim-core` reported 453 passed, 0 failed and 20 ignored on the last sim branch merged (de0a880), against 390 passed and 9 ignored at 4cde174 (§2.3).
- **On `main` before the release** (the parent of aaadacb, run by the coordinator): `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo test --workspace` without the voice crate 455 passed and 20 ignored; `cargo test -p voice --release` 32 passed. The CI dry run passed on Linux (R5-02, §3.6).

### 3.9 Verification (V1)

V1 verified `main` at 5f72a7c end to end on an Apple Silicon Mac, in Chromium (headless 60 Hz display, simulated focus loss, emulated pixel ratio). Every gate passed, and nothing blocks the release. Its two fixes (2caf64c, bbb9ee4) are in §3.7; its findings are in §4.

| Gate / check | Result |
|---|---|
| Clippy, the workspace with all targets, and sim-wasm and voice-wasm for wasm32 | pass, 0 warnings |
| `cargo test --workspace --exclude voice` | 455 passed, 0 failed, 20 ignored (66 s) |
| `cargo test -p voice --release` | 32 passed (with the dictionary, nothing skipped) |
| The ignored long runs, release | 20 of 20: century 6 (16–26 s each); soak 4 (7.7 / 15.8 / 20.6 / 41.3 s); big_town 2 (5.5 / 36.0 s); balance 14.1 s; behaviour 3 (up to 16.7 s); routines 0.6 s; ui_updates 0.3 s; generated_houses 0.5 s and generated_worlds 17.8 s, which test nothing without sample files (§4) |
| `pnpm check`, `pnpm lint`, `check:voice`, `check:llm` | pass (3,161 files, 0 errors; 16 cleaning cases and 6 prompts) |
| `npm run wasm`, `vite build`, the Pages build | pass: sim wasm 2,901,413 → 2,298,877 B after wasm-opt; `bundle.json` 3.1 MB; the build id in `isolation-sw.js`; `THIRD-PARTY-LICENSES.txt` 78,866 B (before 1a4018a) |
| First visit, second visit, redeploy | 18/18: one reload, cross-origin isolated; the second visit asks for 0 hashed files; a redeploy loads the new build, asks for 0 old files and deletes the old cache |
| New game, speeds, saves, export and import, bad saves | 33/33: speeds 1–5 run at 1.0 / 2.99 / 9.66 / 29.56 / 59.88 game minutes a second; two games keep two autosaves each; a 524 KB export imports and loads; the damaged save is removed with a notice, the version-999 save kept and flagged |
| Crash screens | 33/33: panic, throw and hang (8.4 s) offer Reload, Main menu and the report, without "Save and reload"; the render crash offers it, and it works |
| Build mode | 17/17 (dev server): no stuck Alt, `pointercancel` builds nothing, a double Sell sells only the selected object, held R turns only it |
| Audio | 10/10: a hidden tab suspends the audio, Mute in background fades, pause on focus loss resumes at the old speed |
| Graphics | 31/32: Auto at DPR 2 on 1920×1080 draws 2.07 MP; DPR 1 / 1.5 / 2 re-apply to 100 / 67 / 50 %; cap 30 gives 30.0 fps ("60 Hz ÷ 2"); nothing drawn behind full-screen menus; `?renderer=webgl` works. The miss: 25 fps in the 1.5 s after a resolution change, where the test expected 27 |
| Settings from 0.1 | 7/7 |
| Speed 5 for 3 game days | 6/6: 20 steps a second in all 111 samples; no faded feed entry clickable; walkers' largest step a frame 1.51 m, under the 1.6 m snap |
| Medium looks the same | All 18 shots within 0.118 % of pixels: the far road (P1-17) and the bathroom mirror |

## 4. Found but not fixed

What the review found and 0.2.0 doesn't fix, by severity, with a recommendation and a target: **0.2.1** (a small follow-up), **v0.3** (the next feature release) or **never** (not worth doing unless something changes). Rows marked "new" were found during the fixes and aren't in §2. Items the owner put off on purpose, and the plan's list of things left undone, follow the table.

| ID | Severity | What's left | Why not in 0.2.0 | Recommendation | Target |
|---|---|---|---|---|---|
| P1-07, P2-07 | high | Submitting about 493 draws cost 5.1–6.7 ms of CPU a frame on WebGPU (M3 Pro), 21–31 ms at 3× CPU, so on WebGPU Low can't reach 60 fps on the target CPUs. G2's culling brought the orbit by day to about 429 draws (152 of them for the shadow map), but the CPU cost of a frame in play didn't drop: at 3× CPU 0.2.0 runs at 47–57 fps on WebGPU and 78–98 on WebGL2 (P1's sign-off), and the nature cells added meshes and main-pass draws (§3.5.1). *(G3's fix to come.)* <!-- G3 --> | The owner put the low-end work off (§2.8). | Halve the draws: merge the static shadow casters per lot (P1-04) and trim Low. Retry snapshot rendering in play, with the pose texture updated outside the recorded bundles. Measure on an Intel laptop (§5) before choosing WebGL2 for weak GPUs. | v0.3 |
| P1-12 | high | Low draws the same lot, street and furniture geometry as Medium: 2.2 M triangles in the main pass and 1.54 M in the shadow pass, a floor of about 2.5 ms on the M3 after any trim. | Generated LODs were put off (§2.8). | LODs for furniture and the street's houses, low-detail shadow copies of furniture, and the neighbours' houses as overview shells on Low. | v0.3 |
| P1-04 (rest) | medium (was high) | In live play the shadow map is still drawn every frame while residents are in view, because they breathe: 146–163 shadow draws and about 0.8 M triangles a frame after G2's culling (it was 242 and 1.70 M). The static casters aren't merged either. Paused, in Build mode and behind menus it isn't drawn any more. | A per-frame path for residents alone (a cached static map plus a depth copy each frame) needs a depth-writing blit in both WGSL and GLSL; culling had already cut the map to 152 draws (§6). | Residents' shadows in their own per-frame path over a cached static map, or merge the static casters per lot (house and fences, the street's houses and props, garden trees). | v0.3 |
| P1-16 | medium | With vsync and the cap off, WebGPU frames stall under a saturated GPU (35–43 long tasks in 10 s, frame p95 60–137 ms on the M3) while WebGL2 stays smooth. A weak GPU is always saturated. G2's baseline runs on WebGPU stalled the same way. | Needs Intel hardware. | The owner's bench run on an HD 620 with and without `&renderer=webgl` (§5) decides whether weak GPUs should draw with WebGL2. | 0.2.1, after §5 |
| P1-08 | medium | The benchmark's Build scenario ran at about 60 fps in every preset, against 88–120 fps for the same orbit in Live. G2's shadow work raised it (Medium, no vsync: WebGPU 62 → 71 → 78 fps, WebGL2 77 → 91), but it is still well below the orbit in Live (about 145 fps on WebGPU). The cause wasn't found. | Not investigated. | Start with the catalog pictures' second engine (`render/preview/items.ts`) and the paused session's frame clock. | 0.2.1 |
| P2-03 | medium | The interface's frosted blur is redone every frame over the moving 3D view; Low keeps it. | Not among the Low trims in the revised scope (§2.8). | Measure it on an Intel laptop; if it costs 1 ms or more, a plain tint on Low. | v0.3 |
| Autonomy loop (new, F9) | medium (visible when it happens) | An urgent need the resident never picks keeps cutting everything else short. Zoe, alone with a working fridge at 00:30 with hunger 0 and bladder under 0.1: the night schedule scored the armchair's relax at about 28 even after the urgent ×0.25 penalty, against about 1 for a snack; relax was cut after one tick by the pressing hunger and chosen again, and she fell asleep on the floor 136 times over the following days (century run, seed 11, full detail). Older than 0.2.0: the same happened with F9's changes switched off. | Found at the end of F9's work. | Let an urgent need's own fix outscore what it keeps cutting short, or stop choosing what the pressing need would end at once. | 0.2.1 |
| Preset switch memory (new, V1) | medium | Every switch of the graphics preset keeps the old renderer alive, about 40–55 MB of heap each time (heap after 6 switches 434 → 680 MB on the Pages build; live renderers 1 → 2 → 3). Also on 4cde174. One cause: Babylon 9.30 forgets the engine's canvas before disposing its scenes, so the camera's `contextmenu` listener stays; detaching the camera first stopped that, but the renderers were still kept. | Older than 0.2.0; needs a heap-snapshot investigation. | Find what else holds the old renderer (heap snapshots: `heap.mjs presets`, `retained.mjs` in V1's notes). | 0.2.1 |
| P1-02 (night) | medium | At night the clustered lamps still cost: Medium's GPU time on the M3 barely improved (8.84 → 7.39 ms; by day 8.73 → 5.58). The proposed cap of the cluster's range at the largest lamp's (7 m) wasn't done. | Not in G2's work. | Cap the cluster's `maxRange` at night and measure. | 0.2.1 |
| Plain finishes (new, look study) | medium, if confirmed in play | Placeholder models' plain finishes (matte, satin, gloss, metal, chrome) may all show as fully rough metal: the finish is in the vertex colour's alpha (`encodeFinish`), but the meshes have `hasVertexAlpha` false, so the shader reads an alpha of 1, and the cozy fridge's cream `#EFE2BF` shows as khaki. Seen in `/gallery.html` by the look study on the branch `v0.3/look-study`; the game builds items with the same `buildModel` but wasn't checked in play. The street sets `hasVertexAlpha = false` on its plain meshes on purpose. | Found after 0.2.0's fixes, not checked in play. | Check in play; if so, get the alpha to the shader without making the meshes transparent, and check the look again (furniture would turn lighter). | 0.2.1 or v0.3 |
| R5-15 (rest) | medium | Whether the Pages CDN serves the 255 MB language-model file in full is unverified: the dry run built a 489 MB site whose largest file is that model (255.5 MB), but a dry run doesn't publish, so it can't show this; only the live check after the tag can. A player who turns on voices and the language model downloads about 476 MB (voices 216.7 MB with ONNX Runtime, the model 259 MB), against Pages' soft limit of 100 GB a month. | Only checkable once published. | Check the file on the live site after the release; split it, or host the models on Hugging Face or a CDN, only if that shows a problem (owner question). | 0.2.0 release step |
| P3-08 | low | A* has no expansion cap, so an unreachable goal floods the town: one late step, never a stall. | Dropped on purpose (L1): the largest successful search took 9,527 expansions and a failed flood about 26,800 (under 3×), 10–22 floods per 100+ town-days, so a safe cap would trim only about a third of a rare flood. | Reject goals in a disconnected area (region connectivity) and remember unreachable targets; R3-11 now does the latter for uses, cleaning, repairs, washing up and visits. | v0.3 |
| P3-03 | low | About 60 shader chunks load in 3–4 dependent waves while the renderer starts. | Dropped on purpose (L1): preloading saved 0.13 s to the menu only at 80 ms latency, nothing at 20 ms or unthrottled, and made warm loads 25–55 ms slower. Returning players now get these files from the service worker's cache. | Leave it. | never |
| R5-24 | low | The isolation service worker's reload flag (`littlelives.isolationReload` in session storage) is never cleared, so a page whose worker is evicted in a long session runs without isolation and doesn't retry. | Left untouched (L1). | Clear the flag once the page is isolated. | 0.2.1 |
| R3-11 (rest) | low | Social and Join targets aren't in the unreachable memory. | Deferred in the plan: people move. | Remember a person for a short time, or refuse a social whose target's plot can't be reached. | v0.3 |
| R3-17 | low | Residents choose things 57–115 game minutes' walk away, even at night with almost no energy. | Balance (the coordinator's default). | The owner chooses: a stronger distance penalty at night, or a cap on autonomous trips. | v0.3 |
| R2-12 (rest) | low | With neither a sink nor a bin, dirty plates and spoiled food still pile up and are scanned on every decision to eat. | A design call. | Cap or expire floor items per household. | v0.3 |
| R1-04 (rest) | low | The start and the end of a walk still jump at speeds 4–5: a walker standing in one snapshot isn't interpolated. At speed 5 on 10–14 m walks across the lot, 94 of 251 walking frames were at a walk's start or end: 36 stood still and the rest jumped, up to 4.6 m (2.35 m at speed 4) (R1's sign-off). | The decided rule glides only when walking in both snapshots (F4's question). | Glide also when walking in one snapshot and standing in the other. | 0.2.1 |
| R1-15 (rest) | low | The household creator's preview still sways its plants with reduce motion on, and a change of the system's setting reaches the renderer only at the next settings change. | Found late (F4). | Pass `reducedMotion` to the preview, and listen to the media query. | 0.2.1 |
| Start hang (new) | low | A hang inside `Game.fromSave` while a game starts isn't caught: `SimBridge.start` has no time limit, so "Moving in…" would stay. | Outside F2a's brief. | A time limit on start, as requests have. | 0.2.1 |
| Crash saves (new) | low | "Save and reload" after a crash writes a new manual save ("<household> · Day N") each time, not into the game's autosaves. | F2a and F2c ran at the same time. | Save into the game's rotating autosave slots. | 0.2.1 |
| Open 0.1 tab (new) | low | An open tab of 0.1.0 blocks the save storage's upgrade to version 2. Load game says to close it, but the main menu just shows no saves. | Rare, and only once. | Say it on the main menu too. | 0.2.1 |
| Licence links (new) | low | The model licences (`voice/supertonic-3/LICENSE`, `voice/misaki-LICENSE`, `llm/lfm2.5-350m/LICENSE`) have no file extension, so a server may send them as `application/octet-stream` and Credits' link downloads the file instead of showing it. Seen on the local server; Pages unconfirmed. | Found at the end of F5. | Ship `.txt` copies, or link the same texts in `THIRD-PARTY-LICENSES.txt`. | 0.2.1 |
| Bench fps (new) | low | The benchmark's `fps` counts its own `requestAnimationFrame` ticks, not drawn frames: right without a cap (its default), misleading with one. | Found by G1. | Count drawn frames. | 0.2.1 |
| P1-02 (Low) | low | Low keeps its 4 forward lamps in the largest rooms, dark by day, in the shader. | Dropped on purpose (G2): turning them off by day saved 0.2–0.3 ms of GPU but froze the game for 170–250 ms at the first dawn or dusk switch, and compiling both shader sets while loading didn't remove the freeze (§6). | Leave it, unless a way to switch without recompiling turns up. | v0.3 |
| P1-01 (Low) | low | On Low the lawn is thinned everywhere, not drawn only round the viewed lot; the far ring isn't drawn from `.far` models only, and bushes and rocks aren't limited to near. | The landscape is built once per town and shared with the menus' town, outliving lot switches. | Part of the low-end work. | v0.3 |
| P1-09 (High) | low | High's mirrors still reflect at 0.75 of the screen with 4× MSAA. | Both cuts showed (G2). Half resolution: 0.9 % of the mirror's pixels off by more than 16 of 255 against today (softer). Dropping MSAA as well: 1.4 % off by more than 16 against half resolution with MSAA (stair steps on dark edges). | Owner question (§6). | never |
| Running CSS animations (new) | low | Chrome recalculates style every frame for each running CSS animation, even a transform-only one, and on a 120 Hz display that's every refresh. The bar transitions and the low-need pulse are always running, and each feed entry runs its own for 12 s: about 0.13 ms a frame at full CPU speed (M1, M3 Pro). | Removing it means dropping the bar transitions or the pulse, which changes the look (§6). | The owner decides; if so, bars that jump to their value and a still low-need marker. | v0.3 |
| P2-06 (side effect, new) | low | A model still being built ahead when the camera arrives may appear up to a few hundred ms later than before. | The cost of building in 4 ms steps. | Leave it unless players notice. | never |
| R5-29 | low | The dry run's annotations: `actions/checkout@v4`, `actions/cache@v4`, `actions/setup-node@v4`, `actions/upload-artifact@v4`, `actions/configure-pages@v5` and `pnpm/action-setup@v4` target Node 20, which GitHub has deprecated (they were run on Node 24, and worked). `ubuntu-latest` moves to Ubuntu 26 from 2026-10-19. Warnings, not failures. | Seen in the first dry run. | Bump the actions to their Node 24 majors, pin `ubuntu-24.04` (or run the dry run again after the label change). | 0.2.1 |
| P3-17 | low | CI runs only on a release tag or by hand: no job on push or pull request. | Not in scope. | A push and pull-request job with `cargo test` and clippy that prints the big town's ticks per second without gating. | 0.2.1 |
| R5-22 | low | Settings have no version: migrations guess from which keys exist, and only graphics and resolution are range-checked. | Not in scope. | A `settingsVersion` with ordered migrations, and numbers clamped after the merge. | v0.3 |
| Menu stutter (new) | low | Preparing the latest save behind the menu shows as 2–3 frames of 100–175 ms in the first seconds on the menu (M3 Pro), roughly three times that on the target CPUs. | The cost of P3-01's faster menu. | Build that world in slices, as playing an existing household already does. | v0.3 |
| P2-11 | low | No heap-snapshot diff over a 1-hour run at speed 5 was taken; P2 saw the heap after GC grow 153.5 → 160.2 MB in 10 minutes, likely portraits and lists for new residents. | "If time allows" in the review, and M1 didn't take one. | §5's long session watches F3's JS heap; take the diff if it keeps growing. | 0.2.1 |
| P1-13 | low | Low's image processing still writes an RGBA16F target before FXAA (16.6 MB a frame). | Not done. | Do FXAA in the image-processing pass, or use an 8-bit intermediate, if cheap in Babylon. | v0.3 |
| P1-17 | low | A far road's lane markings render brighter or darker between page loads (0.03–0.12 % of pixels). | Not done. | Thaw or refreeze the shared landscape materials on the switch from the overview to the game. | v0.3 |
| Background tabs (new) | low | In a background tab Chrome delays the sim worker's timers to 100–150 ms, so the game runs at about 80–85 % of its speed there (L1, before and after). | Browser behaviour. | Leave it unless players notice; a catch-up rule for hidden tabs would be the fix. | never |
| Redeploy cache (new) | low | After a redeploy, a returning player's first visit puts the new hashed files into the old build's cache, which the new worker then deletes, so they're fetched again on the next visit (usually from the browser's own cache). | Found by L1. | Also serve files carrying `?v=<build>` from the cache. | v0.3 |
| Ids across a reload (new) | low | Loading puts things that stand on tops in last, so object ids can change across a save and reload (the comment in `save.rs` says so since f2b1b43). Tasks are remapped, so nothing breaks. | Position-based ids stay for 0.2.0 (decision). | Stable object ids (below). | v0.3 |
| R5-27 | low | `content/lots/starter.json` ships, though only tests and the pack checker read it. | Harmless. | Move it to `crates/sim-core/tests/data/`. | v0.3 |
| Pause on blur (new) | low | With "Pause when the window loses focus" on, a player who switches away while a game loads may come back to it paused. | Rare, and the setting is off by default. | Leave it unless reported. | never |
| `check:llm` (new) | low | The language model's checks aren't in CI: 7.3 minutes on 4 threads, against golden text made on arm64. | Too slow for every release. | Run `pnpm check:llm` by hand when the prompt, the cleaning rules or the model change. | never |
| Small allocations (M1's item 8) | low | Not done: the allocations at 10 Hz in `audio/world.ts`, `buildmode.ts` and `menuScene.ts`, and the churn of `game.sims`. | P2 measured each site under 50 KB/s, 90 % of all allocation being Babylon's own per-draw work, with no major GC (P2-12, §2.5.2). | Leave them. | never |
| Jogging after a hidden tab (new, R1) | low | For about 0.25 s after a hidden tab (10–20 s away), walkers jog on the spot: their speed is estimated from the distance covered over the 0.1 s-capped frame time (8.7–22 m → 13.2 m/s), past the jog switch, then fades back over about 0.3 s. Seen in all four hidden-tab runs. | Cosmetic, and too risky to rush for 0.2.0. | Don't update the speed on a frame after a long stall (raw frame time well over 0.1 s, say 0.5 s, so machines under 10 fps still update), or reset it as for residents coming into view. | 0.2.1 |
| R1-12 (hidden tab, new) | low | R1-12's fix doesn't cover a hidden tab: residents count as on screen while it is hidden, so one who sat down meanwhile replays the sit-down on return. It ends on its own. | Found at sign-off. | Treat a long stall like coming into view. | 0.2.1 |
| Named wasm artifact (new, P3) | low | CI's 90-day "wasm with names" artifact is the un-optimised file (2,108 functions against 1,347 shipped), so its function numbers don't match the crash stacks of the site; the comment in `pages.yml` says it is for reading them. | Found at sign-off. | Make the named copy from the same wasm-opt run (`-O3 -g`, then a strip-only pass for `dist`), or say what the artifact is for. | 0.2.1 |
| wasm-opt in CI (new, P3) | low | CI doesn't keep `web/.assets/wasm-opt` between runs, so every build pays about 1.5–3 minutes for it (within the 45-minute limit). | Optional. | Add it to `actions/cache`. | 0.2.1 |
| `generated_*` tests (new, V1) | low | The ignored `generated_houses` and `generated_worlds` tests pass in 0.000 s without testing anything unless `HOUSES` and `WORLDS` point at sample files (made by `tools/housegen/sample.ts` and `tools/worlds/sample.ts`; with them both pass, 180 houses and 24 worlds). CI may never run them. | A test gap, older than 0.2.0. | Fail, or make the samples, when the variables are missing; run them in CI. | 0.2.1 |
| Rolling back to v0.1.0 (new, R5) | low | 0.2.0 upgrades the browser's saves database to IndexedDB version 2 (`saves.ts`). v0.1.0 can't open it: after a rollback it lists no saves and can't save. Nothing is lost, and the saves come back with any 0.2.x. | Follows from R5-11's `meta` store. | If a release goes wrong, fix forward or redeploy the last good 0.2.x, not v0.1.0 (README, "Versions and releasing"). | never |
| Source code link (new, R5) | low | Credits doesn't say where the source code is, which the GPL invites. | Found at sign-off. | Link the repository from Credits. | 0.2.1 |
| R2-14 (rest) | low | `Items::add` (`items.rs`) still adds 1 to the next item id unchecked, and a save's next-item counter isn't range-checked. Normal play can't get there. | Only the ids in a save were checked. | A checked add, refusing a save whose counter is at the limit. | 0.2.1 |
| Stale-id commands (R2's sign-off) | low | A command skipped by the `seen` guard only logs to the console, and `seen` compares the object's kind and tile, not its rotation (two identical wall items on one tile could be confused; practically unreachable). | Harmless. | Leave them. | never |
| Width transitions (new, P2) | low | Width transitions remain in `SkillsTab.svelte`, `CareerTab.svelte`, `HomePanel.svelte` and `planner/GoalsPanel.svelte`: the Skills tab's bar re-lays out the page every frame while someone practises. | Outside M1's two bars. | `transform: scaleX`, as for the bars of P2-01. | 0.2.1 |
| Retirees' Career tab (new, F8) | low | Under the retired line the Career tab still says "between jobs. Pick from 1,000 positions…", and the planner can suggest "get a job" to retirees, which the job market never fulfils. Both were so before. | Outside F8's fix. | Hide the job prompt for retirees, and don't suggest the goal. | 0.2.1 |
| Fridges and cookers as one kind (F9) | low | `same_kind` still treats fridges and cookers as one kind for wants: a range cooker counts as "a better fridge" when choosing what to want, and a pricier cooker stops residents wanting a better fridge. | Changing it would change balance. | Owner's call. | v0.3 |
| Pages caching after a redeploy (V1) | low | A player who visited less than 10 minutes before a redeploy may get the old page from the browser's cache (Pages' `max-age=600`) while the new service worker deletes the old build's cache; a file the old page hadn't loaded yet could then fail. Not reproduced; V1 saw no errors. | Pages' caching. | Leave it; the "new version" banner covers a failed chunk. | never |

**Put off by the owner** (§2.8: best effort, not promised; target v0.3):
- Detecting a weak GPU and picking Low or WebGL2 on its own at first start (P1's question, P1-07).
- Auto resolution stepping down when the game runs slowly, and back up with headroom.
- Low at 60 fps on an Intel HD 620 (Low is best effort; Medium at 30 fps stays the target there, §2.8).
- Generated LOD models (P1-12).
- A smaller pixel budget for Low: P1 suggested about 1.17 MP; Auto uses 1920×1080 on every preset (G1).
- Low without the neighbours' house interiors (P1), and without the interface's blur (P2-03).

**Left undone in the plan** (a behaviour change or a large refactor, so it needs the owner's OK):
- **Stable per-object ids.** Objects are still named by their place in the list; 0.2.0 remaps follow-ups, unreachable things and queues when ids shift, and guards commands with `seen` (R2-01, R2-04). Target v0.3.
- **Re-routing walkers when walls change** (a behaviour change). Target v0.3.
- **Social and Join in the unreachable memory** (R3-11, above). Target v0.3.
- **Splitting `structure_version`:** a change refreshes the rooms of the whole town, which costs 0.6–0.76 ms and is rare (P3 saw none in a game day at speed 5). Target v0.3.
- **An LDR pipeline or a non-PBR material for Low,** with the rest of the low-end work. Target v0.3.
- **An environment-intensity material plugin,** so lighting steps don't thaw the PBR materials (P1-11). Target v0.3.
- **Merging the night grade into image processing:** one full-screen pass, about 0 ms on the M3 and about 1 ms on Intel by P1's estimate. Target never, unless Intel measurements show it matters.
- **Per-phase timings of the simulation** (an F6 agent in the plan, not run): P3 found the sim isn't a bottleneck (a speed-5 step averages 0.86 ms in the big town on the M3). Target never, unless sim time starts to matter.

## 5. Manual test checklist

Every agent tested on Apple Silicon Macs in Chromium, with one display. This list is what only the owner can check: other hardware and browsers, real tabs and windows, and long sessions. The site is https://krecks.github.io/idyll-lives/ (below: `…/`). F3 opens the performance overlay: its Main thread section shows the backend, the preset in use, the render resolution and its share of the screen, the display's measured refresh rate and every how many refreshes a frame is drawn ("120 Hz ÷ 2"), the frame rate and the GPU time.

**Intel HD 620/530 laptops** (i7-7500U, i7-8550U, i5-6500 at 1920×1080; nothing has been measured on Intel yet):
- [ ] Open `…/?bench=village&quality=medium&hour=13&cam=orbit&res=1`. The benchmark warms up for 5 s, measures for 20 s, then shows a summary: press "Copy result" and keep the JSON.
- [ ] The same with `quality=low`, then both again with `&renderer=webgl` (WebGL2), and the four again with `hour=21` (night). Add one run with `cam=wide`, where the nature cells cost the most draws.
- [ ] The target (§2.8): Medium at 30 fps or more; Low is best effort, with no frame-rate target. Scaled from Mac measurements, 0.2.0 is estimated at Medium about 10–21 fps by day and 8–13 at night, Low about 14–41 and 13–39 (before 0.2.0: Medium about 6–10, Low about 9–27, §2.5.1). The benchmark draws every refresh (`cap=0`), so the display's refresh rate caps its `fps`.
- [ ] P1-16: compare WebGPU with WebGL2 for long tasks and the frame interval's p95 and p99. If WebGPU stalls where WebGL2 doesn't, weak GPUs may be better off on WebGL2 (§4).
- [ ] Play 10 minutes on Medium (the default) and 10 on Low, with F3 open: frame rate, GPU time, Busy (frames), and whether Auto resolution shows 1920×1080 · 100 %.
- [ ] Main thread on a real slow CPU: at speed 5 with a resident's panel open, F3's "Busy (frames)" and long tasks. M1 measured the main thread 82 % busy there with the M3's CPU slowed 3× (§3.5.2), standing in for these laptops; that was from a trace, and F3 reads a few points lower than a trace. Note any long task or hitch, for example when the camera follows someone onto another lot.
- [ ] Low's look on Intel: half the lawn's tufts and flowers, and no shadows from the woods round the town. It should still look like the same place.
- [ ] Open the pause menu or Settings in a game: the blurred picture behind stays still, and the laptop's GPU use (Task Manager, or Activity Monitor's GPU History) drops until the menu closes.
- [ ] On High, a mirror that comes into view reflects the room at once (its reflection is now made only when needed).

**Safari and Firefox** (not tested by any agent):
- [ ] Safari 26 with WebGPU: the first visit reloads once for the isolation service worker, and F3 shows Shared memory: on. A new game, Save, reload the page, Continue.
- [ ] Safari: sound starts after the first click or key, and comes back after switching tabs and apps (`interrupted`). Voices and the language model run (WebGPU).
- [ ] Safari: Export a save from Load game, then Import it. (Safari may delete the site's storage after 7 days without a visit.)
- [ ] Firefox: where it has no WebGPU, the game draws with WebGL2 (F3 backend), and voices and the language model say they need WebGPU. On any browser `?renderer=webgl` forces WebGL2.

**Displays at 120/144 Hz and odd rates** (frame rates the FramePacer should give; F3 shows the measured Hz and the divisor):

| Display | Cap 30 | Cap 60 (default) | Cap 120 | No cap |
|---|---|---|---|---|
| 60 Hz | 30 | 60 | 60 | 60 |
| 75 Hz | 37.5 | 75 | 75 | 75 |
| 90 Hz | 30 | 45 | 90 | 90 |
| 100 Hz | 33.3 | 50 | 100 | 100 |
| 120 Hz | 30 | 60 | 120 | 120 |
| 144 Hz | 36 | 72 | 144 | 144 |
| 165 Hz | 33 | 55 | 82.5 | 165 |
| 240 Hz | 34.3 | 60 | 120 | 240 |

- [ ] On a 120 Hz and a 144 Hz display, each cap (Settings › Performance › Frame rate cap): F3's Display matches the table, and a walker crossing the lot moves evenly, without a hitch every second or two.
- [ ] On a 90 Hz or 165 Hz display, cap 60 gives 45 or 55 fps (rounded down, §6). Say if that feels wrong.
- [ ] The menus' town runs at about 30 fps, as in the Cap 30 column (36 on 144 Hz); the flight into a game is smooth.

**4K and Retina screens, zoom, a second monitor:**
- [ ] On a 4K or Retina screen with Resolution on Auto, F3's Resolution is about 2.07 million pixels (a 3840×2160 screen: 1920×1080 · 50 %) and the picture looks sharp.
- [ ] Browser zoom in and out (Ctrl/Cmd + and −): F3's Resolution follows and returns to about 2.07 MP.
- [ ] Drag the window from a Retina or 4K screen to a 1080p one and back (not tested: one display): F3's Resolution changes with the screen, with no blur.
- [ ] Custom resolution at 50 % and 100 %.

**Long sessions:**
- [ ] Play a few hours at mixed speeds, with the journal opened now and then: F3's JS heap and long tasks stay level, the journal stays at 300 entries or fewer, and each game keeps two autosaves (Load game).
- [ ] Speed 5 past game day 1167 (about 7.8 hours at speed 5): residents still age and have birthdays, babies are born, people move in. Before 0.2.0 these stopped on day 1167. (The sim test `the_town_still_ages_after_day_1167` checks the same natively.)

**GitHub Pages, first visit and a redeploy** (if a release goes wrong, fix forward or redeploy the last good 0.2.x, not v0.1.0, which can't open 0.2.0's saves database, §4):
- [ ] A fresh browser profile: the page loads once more for the service worker and plays; F3 shows Shared memory: on.
- [ ] A return visit: DevTools › Application › Cache Storage has `littlelives-build-<id>`, and the Network panel shows `assets/*-<hash>.js` served by the service worker.
- [ ] Leave a tab on the main menu across a redeploy, without having started a game in it, then start one: the game screen's script is gone from the site, so the tab shows "A new version is available." with Reload, over the menu. After the reload the new build runs, and the old build's cache is gone.

**Voices and the language model** (WebGPU only):
- [ ] Settings › Voices: Resident voices on (216.7 MB once), Try the voices, then What residents say › Lines › Language model (259 MB once), the device test ("On this computer") and Try the model.
- [ ] Play with both on and subtitles shown: lines are spoken, and F3's frame rate holds while lines are made.

**Saves:**
- [ ] Load game › Export a save, delete it, then Import a save… with the file: it comes back and loads.
- [ ] Import refusals: in an exported file, change `\"version\":23` inside `data` to 999 ("from a newer version"); cut the file short ("damaged"); pick any other JSON file ("isn't an Idyll Lives save"). Nothing is added.
- [ ] A save from the live 0.1-era site (version 17 or 19) still loads after the update.
- [ ] If Load game ever marks a save "This version couldn't open it", it's kept on purpose (a loader failure, c939e8e): Export it and keep the file for a bug report.
- [ ] Load game lists only each save's own household under its name.
- [ ] A resident of 65 or more with no job is retired, on a pension of at least $500 a week (Career tab).
- [ ] The crash screen: open a game with `?debug`, then in the console `__crash('panic')`, `__crash('throw')`, `__crash('hang')` (about 8 s) and `__crash('render')` (about 2 s at 30 fps). Each shows the crash screen. Only `render` offers Save and reload; the others refuse to save. Main menu, then Continue, starts again.

**Windows, tabs and settings:**
- [ ] Alt+Tab in Build mode (Windows; on macOS, Option-click another window), come back, and click with the Wall tool: it draws a wall, and the floor isn't picked up.
- [ ] Sound stops in a hidden tab and comes back when the tab is shown, whatever the settings. With "Mute in background" on, sound fades while another window has focus. Master volume works.
- [ ] Settings from 0.1: set Graphics to Low on the live site before the update (or, in DevTools on the new site, set `localStorage['open-sims-wasm.settings']` to `{"quality":"low"}` and reload). After the update Settings › Performance shows Low. With `{"quality":"high"}`, or the old defaults, it shows Medium. Resolution is Auto either way.

## 6. Open questions for the owner

Nothing waited on these: each has a default that shipped in 0.2.0, and a different answer would be a change for a later release. The §2.7 questions not listed here were answered by the decisions in §2.8, or by a fix in §3 (R1-17, P3-01, and P2-08: the menus' town is drawn at 30 fps and nothing is drawn behind full-screen menus).

| Question | What shipped | From |
|---|---|---|
| **Gameplay** | | |
| When ordered, may a guest nap or shower at the host's? | Orders follow the guest rules of free will: nothing private but the toilet, no sleep or nap, no cooking, no caring for the host's baby. | F1a |
| A visiting resident ordered to use something at home or in a park stays "visiting" until the visit times out. Should the order end the visit? | Unchanged (as before 0.2.0). | F1a |
| In the starter house, residents start a full night's sleep in the daytime at about 0.55 energy, about once a day each, for about 3 hours. Practice sessions that now run their full length make those sleeps about 7 % longer. | Unchanged. | F1b |
| A couple who are expecting and move out (`move_to`) leave the pregnancy with the parents' household. | Unchanged. | F1b |
| When someone leaves town, a pregnancy they're a parent of ends. The other choice was to keep it with one parent. | It ends. | F1b |
| A household the player doesn't play that couldn't keep a week's rent after buying a child's bed doesn't buy one, and doesn't try again later. | As implemented (f1993e5). | coordinator |
| In a full big town (64 residents), births only follow deaths and departures, because newcomers refill empty houses first. | Unchanged (balance). | soak runs |
| R3-17: a stronger distance penalty at night, or a cap on autonomous trips? | No change (balance). | R3 |
| **Animation** | | |
| Is the new `jogAlong` clip (the jog's pose with a stride) the right look for fast walking? | Shipped. | F4 |
| Should walkers also glide at the start and end of a walk at speeds 4–5 (walking in one snapshot, standing in the other)? | No: they glide only when walking in both snapshots, as the decision said; §4 suggests it for 0.2.1. | F4 |
| R1-08: are the arrow keys meant to move the camera? | Kept, and now as fast at any frame rate. | R1 |
| **Graphics** | | |
| Should Auto's pixel budget be smaller on Low? P1 suggested about 1.17 MP. | 1920×1080 (2.07 MP) on every preset. | G1 |
| A cap the refresh rate doesn't divide rounds down when the next rate up is more than 25 % above it: cap 60 gives 45 fps on 90 Hz, 50 on 100 Hz and 55 on 165 Hz. Acceptable? | Shipped. | G1 |
| The menus' camera glides run at 30 fps with the menus' town; only the flight into a game runs at the full rate. Acceptable? | Shipped. | G1 |
| On Low, the lighting steps every 10 game minutes at speed 3 and up and in the quiet-hours time-lapse, so the sun moves in visible steps about once a second. Acceptable? | Shipped. | G1 |
| Should weak GPUs (Intel HD 620-class) get Low, or WebGL2, by default at first start? | No detection: everyone starts on Medium with WebGPU where it works (put off, §2.8). §5's bench run informs it. | P1 |
| Should Low replace the interface's frosted blur with a plain tint (P2-03)? | No: Low keeps the blur. | P2 |
| Should High's mirrors reflect at half resolution, or without MSAA, to save GPU time and memory? Both showed: about 1 % of the mirror's pixels off by more than 16 of 255. | No: full resolution (0.75 of the screen) and 4× MSAA, as before. Only their memory is now taken on demand. | G2 |
| Should residents' shadows get their own per-frame path, so that live play stops redrawing the whole shadow map? It needs a depth-writing blit in both WGSL and GLSL. | Not in 0.2.0: the map is still drawn every frame while residents are in view, from 152 draws after culling. A v0.3 candidate (§4). | G2 |
| Low's 4 forward lamps stay in the shader by day, dark. Turning them off by day saved 0.2–0.3 ms of GPU but froze the game for 170–250 ms at the first dawn or dusk switch. | Kept, dark by day. | G2 |
| Should Low leave out the neighbours' house interiors, and draw the lawn only round the viewed lot (P1-01, P1-12)? | No: Low draws the neighbours as Medium does, and thins the lawn to half everywhere (the landscape is shared with the menus' town). | P1, G2 |
| Chrome recalculates style every frame for each running CSS animation (the bar transitions, the low-need pulse, a feed entry's fade): about 0.13 ms a frame at full CPU speed on the M3 Pro, every refresh on a 120 Hz display. Removing it means dropping the bar transitions or the pulse, which changes the look. Worth it? | Kept: the bar transitions and the low-need pulse are unchanged (now animated by transform, P2-01). | M1 |
| **Pensions** (decisions F8 took that the brief didn't cover) | | |
| Elders a new town places with a job (75 % of generated neighbours have one, elders up to 75 included) would, under "retire only on reaching the age", work until they die. | They start retired with that job's pension, as they used to at the first midnight, minus the story event. | F8 |
| Retiring without a job tells no story: the only text is "retired from their job as {job}". | No story event; they still get the "Retired" feeling. | F8 |
| Retirement now runs at midnight even with lifespan Off, so an elder who quits a job given by order gets the basic pension. | Shipped. | F8 |
| **Saves and crashes** | | |
| "Save and reload" after a crash writes a new manual save each time. Should it go into the game's autosaves? | A new manual save. | F2a |
| R4-05: after a GPU reset, rebuild the renderer silently, or offer Save and reload? | Save and reload. | R4 |
| **Release** | | |
| R5-15: keep the large model files on Pages, split them, or host them on Hugging Face or a CDN? | On Pages; split only if the live check after the tag shows a problem (a dry run doesn't publish, so it can't). | R5 |
| R5-02: dry run through `workflow_dispatch` on `main`, or a `v0.2.0-rc.1` tag? | `workflow_dispatch` on `main`, with `-f deploy=false`, before tagging. | R5 |
| **Process** | | |
| The agents' browser checks are Playwright scripts. The web app has no JS test runner: should they live in the repo, and where? | They stay out of the repo. | F4 |
| Should sim-core offer a read-only accessor for a resident's follow-up, so tests needn't read it from the debug text? | Tests read the debug text, and a test fails if that stops working. | T1 |
