# Release review for v0.2.0

The pre-release review for 0.2.0 started on 2026-10-13 from `main` at 4cde174, which is 136 commits after v0.1.0. It covers animations, objects and items, bugs, performance and release readiness. Each finding has a location, a severity, a description and a proposed fix.

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
- **The renderer runs its own loop on the main thread** (`engine.runRenderLoop`, `BabylonRenderer.ts`), capped by the FPS setting.
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
  - A new 9-house town saves to about 150 KB and grows to 200–220 KB over weeks of play (about 20 KB gzipped). A full town of 64 residents is about 1 MB (83 KB gzipped).
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
- R2 and R4 used a stricter scale than the one above: critical only for a crash or data loss likely in ordinary play, and high for a crash or corruption in plausible play. Their grades are kept, which is why nothing is graded critical.

| Area | Critical | High | Medium | Low | Rows |
|---|---|---|---|---|---|
| 2.1 Animations and frame pacing (R1) | 0 | 2 | 4 | 11 | 17 |
| 2.2 Objects and items (R2) | 0 | 1 | 4 | 14 | 19 |
| 2.3 Simulation bugs and lints (R3) | 0 | 2 | 5 | 7 | 14 |
| 2.4 Web robustness (R4) | 0 | 6 | 7 | 11 | 24 |
| 2.5.1 GPU and rendering (P1) | – | – | – | – | – |
| 2.5.2 Main thread (P2) | 0 | 2 | 4 | 5 | 11 |
| 2.5.3 Simulation, build and load (P3) | 0 | 0 | 3 | 10 | 13 |
| 2.6 Release readiness (R5) | 0 | 4 | 12 | 11 | 27 |
| **Total without P1** | **0** | **17** | **39** | **69** | **125** |

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
| R2-01 | high | `crates/sim-core/src/home.rs:1179` | Selling or moving an object shifts the ids in residents' `current` and `queue` but not in `follow_up` or `stuck`. A pending follow-up then panics at `content.rs:210` when its id now names an object with fewer interactions, or sends the resident to the wrong object, possibly a neighbour's. NPC purchases also shift ids, mid-tick. Measured: about 1 crash per 1,200 Build-mode visits that move or sell something. | In `remove_object`, shift `follow_up` too (drop it if it used the removed object), and shift or clear `stuck`. In `start_task`, use `interactions.get()` and check `access`. Tests: the two R2-01 proofs. |
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
| R4-01 | high | `web/src/core/sim.worker.ts:98` | A panic in the sim's `step` freezes the clock and every resident with no message. It only reaches an uncaught worker `error` event (not the F8 log), and every later action toasts "recursive use of an object detected…". With `panic = abort` the message is just `unreachable`. R2-01 is one real path to it. | Guard `step` and every handler: on a trap, drop the game, stop the loop and post a fatal error. Keep `worker.onerror` after start and show a blocking panel ("The game stopped because of an error. Your last save is safe.") with Reload and Download debug report. A panic hook in `sim-wasm` passes the message and location. |
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

*(GPU numbers from the benchmark follow.)*

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
| P2-07 | high | `web/src/render/babylon/BabylonRenderer.ts:1819` | The main thread is saturated at `r = 3`, and Low doesn't help. `scene.render` takes 2.9 ms a frame at `r = 1` and 8.5–10 ms at `r = 3`, 75–80 % of main-thread time; Low is 90–95 % busy, like Medium. The code comment that live drawing costs only ~0.2 ms more (`:3093`) is an M3 number. | For P1 and the owner: Low needs CPU-side cuts (fewer draw calls or shadow casters, merged static meshes, culling the near nature instances, or snapshot rendering in play). The app-side fixes in this table recover 10–20 % at speed 5. |
| P2-02 | medium | `web/src/render/babylon/BabylonRenderer.ts:793` | `project()` reads the canvas's `clientWidth` and `clientHeight` for every bubble every frame, after DOM writes, which forces style and layout inside the frame (26.5 ms/s at speed 5, `r = 3`). With a cached size, `project` went from 14.8 to 0.6 ms/s. | Cache the canvas's CSS size (a ResizeObserver or the existing `onResize`) and use it in `project()` and `townFrame`. Do it together with R1-07. |
| P2-03 | medium | `web/src/ui/theme.css:73` | The `backdrop-filter` blur on the 29 glass panels is redone every frame, because the canvas under it changes. Removing it leaves the main thread unchanged but lowers the GPU process's main thread by about 20 points of busy share (speed 5, `r = 3`, SimPanel open). On HD 620-class GPUs it is extra GPU time. | On Low only, replace the blur with a more opaque background (a `.low-gfx` class). Medium keeps the look, as decided. |
| P2-04 | medium | `web/src/render/babylon/BabylonRenderer.ts:639` | At speed 5 every snapshot moves the clock 3 game minutes, so `applyLighting` runs 20 times a second (1.65 ms each at `r = 3`) and keeps the lit materials thawed through dawn and dusk. At most 4 times a second: `applyLighting` 33.1 → 8.4 ms/s, about 25–55 ms/s saved in all. | Rate-limit by real time too (at most every 250 ms), or scale `LIGHT_STEP` with the ticks per step. 15 game minutes per update doesn't show as a step. |
| P2-08 | medium | `web/src/game/host.ts:198` | The menu town is drawn at full rate: 87 % main-thread busy at `r = 3` (25 % at `r = 1`), as much as play. | Cap the menu town at 30 fps with the FramePacer (estimated 87 → ~45 %), or stop drawing while a full-screen menu covers it (owner question). Cache the label obstacle rectangles. |
| P2-05 | low | `web/src/game/session.ts:738` | Idea toasts are rebuilt on every UI message (7.7 a second) even when no resident changed: 2.1–2.9 ms/s at `r = 3`, about a quarter of all UI-message work. | `if (changed.sims) ideas.update(game.sims)`. |
| P2-06 | low | `web/src/render/babylon/models.ts:622` | When `buildAhead`'s idle callback fires on its timeout, one model job still runs: 38–55 ms at `r = 3`. It is the only long task seen in play (one per 30 s at speed 5), and a visible hitch on target CPUs. | Run a job on timeout only if it fits, or re-arm after a frame. Better: split model building into smaller jobs. |
| P2-11 | low | JS heap (production build) | Over 10 minutes at speed 5 (about 25 game days) the heap after forced GC grew from 153.5 to 160.2 MB, and DOM nodes from 960 to 1,129. Likely portraits and lists for new residents; not a leak by itself. | If time allows, a heap-snapshot diff over a 1-hour speed-5 run before release. |
| P2-13 | low | `web/src/audio/world.ts:577` | Bird songs are made on the main thread inside the 10 Hz sound tick: 1–2 MB and a few ms per new song (cached afterwards). | Make them in the water-sound worker or in idle time. |
| P2-14 | low | `web/src/debug/threads.ts:55` | F3 "Busy" counts only the frame callback, not style, layout, paint, UI messages or GC: it shows 70 % where the trace measures 80.5 %. | Use a long-animation-frame based busy share, or label it "Frame work share". |

Measured savings at speed 5, `r = 3`:

| Fix | Finding | Saving |
|---|---|---|
| Bars as `transform: scaleX` | P2-01 | 59 ms/s with the panel closed, about 140 ms/s with the SimPanel open |
| Lighting at most 4 times a second | P2-04 | about 25 ms/s directly, plus about 30 ms/s from fewer thaws (noisy) |
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
- Not tested: Safari and Firefox with the service worker, the Pages CDN with the 255 MB and 120 MB files, and the Linux CI build. All need the dry run (R5-02).

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
- **P2-07:** which CPU-side rendering cuts for Low at 60 fps: fewer draw calls or shadow casters, merged static meshes, culling the near nature instances, or snapshot rendering in play?
- **P2-08:** cap the menu town at 30 fps, or stop drawing it behind full-screen menus?
- **P2-03:** lower the glass blur everywhere, or only replace it on Low as decided?
- **P3-01:** show the menu before the latest save is ready, with Continue showing the remaining progress?
