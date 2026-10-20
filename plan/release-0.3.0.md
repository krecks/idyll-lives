# Idyll Lives 0.3.0 — Release Plan

- **Branch:** `release/0.3.0`
- **Base:** `main` at `a6b7924`
- **Status:** Draft
- **Release type:** stability, performance and architecture; no new headline feature

## 1. Release intent

0.3.0 should make the game more reliable and cheaper to run, not broader. The release is done when the known player-visible bugs are fixed, stable object ids have landed with a save migration, Low has been measured on weak hardware and its policy decided, and the carry-over from the 0.2.0 review has been triaged rather than silently inherited.

**Success criteria**

- No known release-blocking player-visible bug remains.
- Stable object ids ship, with old saves migrating and no id remapping needed for normal edits.
- Low is measured on at least one weak-GPU target, and the WebGPU/WebGL2 policy is recorded.
- Every 0.2.0 review carry-over item is fixed, scheduled, or explicitly deferred with a reason.
- `PLAN.md`, `docs/objects/open.md`, the roadmap and the release notes agree with the code.
- CI, the dry run and the live-site checks pass.

**Non-goals**

- No renderer rewrite or engine bake-off.
- No new voice engine; Auto mode only.
- Smarter thoughts stays research, not shipping behaviour.
- Not every low-severity review item is fixed in this release.
- No new large content category unless it is needed to close an existing use gap.

---

## 2. Workstream 1 — Player-visible fixes

These are the first priority because they affect play directly. Workstream 1 must be complete before the release candidate.

### 1.1 Sofa bed can open under a sitter

**Problem:** `ObjectInstance::allows` prevents a use from closing an object someone else is on, but not from opening it. A second resident's doze can unfold the sofa bed while another resident is relaxing on it, leaving the sitter on an open bed and `relax` unavailable.

**Files:** `crates/sim-core/src/world.rs`, sofa-bed content, `crates/sim-core/tests/open_objects.rs`.

**Acceptance**

- Opening is refused only when another occupant's use requires the object to stay closed.
- A couple can still open a wall bed or futon together.
- The sitter is not left on an invalid open/closed state.
- Regression test: doze is refused while someone relaxes on the sofa bed; wall bed/futon still allow the shared opening use.

**Priority:** Must ship.

### 1.2 Urgent need cuts every activity in a loop

**Problem:** An urgent need the resident never fixes keeps cutting short whatever else they choose. The resident then chooses the same high-scoring activity again, is cut off again, and can repeat until they fall asleep on the floor.

**Files:** `crates/sim-core/src/ai.rs`, `planner.rs`, `world.rs`; long-run tests.

**Acceptance**

- An urgent need's own fix outranks, or prevents choosing, an activity the urgent need would immediately abort.
- No repeated choose/abort loop in the reported case.
- Regression test with a high-scoring relaxing activity and an urgent need.
- `century` seed 11 and a soak run show no recurrence of the floor-sleep loop.

**Priority:** Must ship.

### 1.3 Too-large household in a too-small house

**Problem:** The new generated-house furnishing only furnishes the bedrooms that exist. `ChooseHome` does not check `bedroomsNeeded`, so a four-person household can move into a one-bedroom house and leave extra sleepers with no bed.

**Files:** `web/src/game/household.ts`, `web/src/game/housegen.ts`, `web/src/game/town.ts`, `web/src/ui/screens/ChooseHome.svelte`.

**Decision:** warn and require confirmation. `ChooseHome` keeps the move available when a house has fewer bed places than sleepers, but shows the shortage and requires an explicit confirm. The simulation keeps a defined fallback: extra sleepers can share a bed or sleep on the floor, and the shortage is reported as a `noBed` event rather than silently leaving a resident without a place.

**Acceptance**

- A household cannot silently end up with fewer bed places than sleepers.
- If overcrowding is allowed, the UI says so and the simulation has a defined fallback (shared bed, floor sleep, or a `noBed` event).
- Test: four-person household in a one-bedroom house follows the chosen policy.

**Priority:** Must ship.

### 1.4 Model and animation polish

These are visible but mostly cosmetic. Fix the cheap ones; explicitly defer the rest with a note in `docs/objects/open.md`.

| Item | Where | Ship? |
|---|---|---|
| Bean bag snooze lies flat at bed height, legs in the air | `docs/objects/open.md`, characters/poses | If cheap |
| Weight-bench lie stretches legs past the bench | `docs/objects/open.md` | If cheap |
| Different-height kiss meets forehead, not mouth | `docs/objects/open.md` | If cheap |
| Getting up from a chair glides back to the tile | `docs/objects/open.md` | Defer unless trivial |
| Chair tuck misses standing desk / high top | `docs/objects/open.md`, `tableTuck.ts` | If cheap |
| Freezer door has a clip but no use | kitchen models/content | Defer or add a frozen-food use |
| Sofa-bed mattress 0.44 m vs sleeper 0.42 m | `families/living.py`, `characters.ts` | Fix with the sofa-bed bug |
| Floating mirrors / party-bar colour / heart-tub shins | `docs/objects/open.md` | Defer unless touched |

**Priority:** Should ship; only the sofa-bed mattress is coupled to 1.1.

---

## 3. Workstream 2 — v0.3 architecture and performance

These are the release's defining changes. They are larger and should start in parallel with Workstream 1 where possible.

### 2.1 Stable object ids

**Why:** Objects are still named by their position in the list. Edits remap follow-ups, queues and unreachable targets, and commands use a `seen` guard. This is the largest remaining refactor and the main save-format risk.

**Scope**

- Give every object an id of its own for its lifetime.
- Persist ids in saves; migrate older saves.
- Remove position-based remapping and the `seen` guard where stable ids replace them.
- Keep ids valid across move, sell, reload, undo/redo, blueprints and generated houses.

**Acceptance**

- Moving or selling an object does not change another object's id.
- A queued action, follow-up or unreachable target still points at the same object after an edit and after save/reload.
- Old saves load and play; a save from a newer version is still refused safely.
- Save version bumped and documented if the format changes.
- Tests: move/sell/reload, queue survival, undo/redo, old-save migration.

**Priority:** Must ship. This is the critical path.

### 2.2 Low-end rendering

**Why:** Draw submission is the largest known performance risk. About 493 draws cost 5.1–6.7 ms of CPU a frame on WebGPU on an M3 Pro; Low still draws the same lot, street and furniture geometry as Medium, with no LODs. On weak GPUs, WebGPU can also stall when the GPU is saturated.

**Scope, in order**

1. **Measure first.** Run `?bench=village&quality=low` on an Intel HD 620-class laptop, WebGPU and `&renderer=webgl`, with vsync and cap off. Record frame times, long tasks and draw counts.
2. **Halve the draws.** Merge static shadow casters per lot; trim Low; add furniture and street LODs; draw neighbours as overview shells on Low; retry snapshot rendering in play with the pose texture updated outside the recorded bundles.
3. **Decide the weak-GPU policy.** If WebGPU still stalls under saturation, choose WebGL2 for weak GPUs or add an explicit fallback message.
4. **Record the result** in `docs/design/graphics.md` and the release notes.

**Acceptance**

- Before/after numbers on the weak-GPU target.
- Draw count and triangle count for Low, day and night.
- A documented decision: Low target, WebGL2 policy, and which trims shipped.
- No regression on the M3 Pro Medium baseline.

**Priority:** Must ship. This is the other critical path.

### 2.3 Voices III: Auto mode

**Why:** Auto is the remaining voice roadmap item. It must pick the right mode without frame drops and step down on slow machines with a clear message.

**Files:** `web/src/voice/*`, Settings › Voices, `docs/design/voices.md`.

**Acceptance**

- Auto selects the appropriate mode at start and after a preset or device change.
- A slow machine steps down with a visible, non-blocking message.
- No frame drops or long tasks introduced by the switch.
- Setting persists across reloads.
- Manual mode still works and is not overridden.

**Priority:** Should ship; can slip to 0.3.1 if stable ids or Low overrun.

### 2.4 Smarter thoughts — research only

**Why:** The roadmap lists this as research. The utility AI must keep deciding; a model may at most re-rank a top-3 shortlist, and any model input must enter the sim as a recorded command so saves and replays stay deterministic.

**Deliverable**

- A short design note: where the model sits, what it sees, how determinism is preserved, cost and fallback.
- Optional prototype behind a debug flag, not shipped behaviour.
- No save-format change.

**Priority:** Research; not a release blocker.

### 2.5 Uses for parts that still have none

Close or explicitly defer the known dead parts:

- Executive desk drawer.
- Nesting tables' small table.
- High-chair tray.
- Laundry-basket lid.
- Display-cabinet door.
- Kettle, bread bin, laptop (blocked by the rule against uses on things on a top).

**Acceptance:** Each item has a use, a documented decision to leave it decorative, or a follow-up task with a reason. No clip exists that no use can play.

**Priority:** Should ship; small.

---

## 4. Workstream 3 — 0.2.0 review carry-over

Triage the review's “Found but not fixed” table into: fix in 0.3.0, defer to 0.3.x, or never. Do not carry the whole list silently.

### 3.1 Must fix or decide before release

| Item | Acceptance |
|---|---|
| Graphics-preset switch leaks ~40–55 MB per switch | Heap snapshot identifies the retained renderer; six switches do not grow live renderers or heap monotonically. |
| Build benchmark stuck around 60 fps | Cause identified; fixed, or documented with a follow-up. |
| Night clustered lamps still cost | Cap the cluster's `maxRange` at the largest lamp's range and measure; record before/after. |
| WebGPU stalls on a saturated GPU | Weak-GPU measurement from 2.2 decides WebGL2 fallback or a documented best-effort policy. |
| Live Pages large-file check | On the live site, verify the LLM file is 255,530,880 bytes and the voice file 119,568,968 bytes; split or rehost only if the check fails. |
| `generated_*` tests pass vacuously without `HOUSES`/`WORLDS` | Tests fail or generate samples when the variables are missing; CI runs them. |
| CI has no push/PR job | Add a push/PR job with clippy and `cargo test`; keep the release-only heavy jobs. |
| `check:llm` is manual | Document the trigger (prompt, cleaning rules or model change) and keep it out of every release unless it gets cheaper. |

### 3.2 Small 0.2.1-style batch

Fix if time allows; otherwise defer explicitly:

- Isolation service worker's reload flag is never cleared (`R5-24`).
- Deploy chunk failure leaves a stale tab hanging (`R4-13`).
- Walk start/end jump at speeds 4–5 (`R1-04`).
- Start has no time limit (`Game.fromSave` hang).
- Crash “Save and reload” writes a new manual save instead of autosave slots.
- Graphics API switch without reload fails to start.
- Licence links download instead of displaying.
- Named WASM CI artifact does not match shipped function numbers.
- Cache `wasm-opt` between CI runs.
- Width transitions remain in Skills/Career/Home/Goals panels.
- Retirees still see “between jobs” and can be suggested “get a job”.

### 3.3 Defer / never

Record the reason for each deferred item. Likely defer:

- P1-02 Low lamps by day (freeze on dawn/dusk switch).
- P1-09 High mirror resolution/MSAA.
- P1-13 Low RGBA16F intermediate.
- P1-17 far-road lane marking brightness.
- Running CSS animations (look change).
- Menu snapshot bundles / redeploy cache / Pages caching.
- Background-tab sim slowdown.
- Per-phase sim timings.

---

## 5. Workstream 4 — Docs and repository cleanup

Do this before the release candidate, not after.

### 4.1 Reconcile the roadmap

- Update `PLAN.md` §8: mark done rows as done, remove rows for work already on `main`, and fix the stale `v0.3/garden-decor` reference.
- Update milestone 0.18: the current sample covers four household variants, not the old 450-house figure.
- Make the “Not yet” row match the actual remaining voice/thoughts work.

### 4.2 Resolve stashes and `plan/future.md`

- `plan/future.md` is absent from the working tree but exists in the stash's parent commit. Decide: restore it, move its still-relevant items into `plan/release-0.3.0.md` / `PLAN.md`, or drop it.
- Two stashes remain: an old merged-work stash and a WIP that only removes the `partner_of` debug assertion. Decide keep or drop; do not leave both unexplained.

### 4.3 Clean `docs/objects/open.md`

- Remove the duplicated line.
- Re-verify “Still open” items that may already be fixed.
- Make “Family notes out of date” match the current family docs.
- Ensure every deferred Workstream 1 item is listed with a reason.

### 4.4 Release paperwork

- Add the `0.3.0` section to `CHANGELOG.md` with the release date.
- Run `node tools/release/version.mjs 0.3.0` at release time.
- Update README's current version only in the release commit.
- Keep this plan linked from the release notes or `PLAN.md` if useful.

---

## 6. Sequence and milestones

No dates; use the order to keep the two critical paths moving.

### M0 — Decisions and baseline

- Decide the too-large-household policy.
- Decide the Low target and weak-GPU policy after the first measurement.
- Confirm the stable-id save-version plan.
- Confirm Auto voice scope and whether it can slip.
- Record the decisions in this file.

### M1 — Player-visible fixes

- Sofa-bed opening.
- Urgent-need loop.
- Too-large-household guard.
- Cheap model/pose fixes.
- Regression tests for each.

### M2 — Stable object ids

- Id allocator and persistence.
- Migration.
- Remove remapping/`seen` where replaced.
- Tests and save-version bump.

### M3 — Low-end rendering

- Weak-GPU measurement.
- Draw reduction, LODs, Low trims.
- Snapshot retry.
- Policy and documentation.

### M4 — Hardening and docs

- Workstream 3 must-fix items.
- Small batch as time allows.
- Roadmap, stashes, `open.md`, changelog.
- Auto voice if it did not run in parallel.

### M5 — Release candidate

- Full CI and dry run.
- Manual checklist.
- Tag and live-site verification.

---

## 7. Release gates

Run from the repository root unless noted.

```sh
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p sim-core --locked
cargo test -p ll-tools --release --locked
cargo run -p sim-core --example tickbench --locked
cargo test -p voice --release --locked

cd web
pnpm check:voice
pnpm check
pnpm lint
pnpm build
```

Long runs, by hand:

```sh
cargo test --release -p sim-core --test century -- --ignored
cargo test --release -p sim-core --test soak -- --ignored
HOUSES=... cargo test -p sim-core --test generated_houses -- --ignored
HOUSES=... WORLDS=... cargo test -p sim-core --test generated_worlds -- --ignored
```

Rendering:

```sh
node tools/bench/render.mjs
```

Dry run:

```sh
gh workflow run pages.yml --ref release/0.3.0 -f deploy=false
```

Live-site checks after tagging:

- Menu shows `0.3.0`; Credits shows the tagged commit.
- F3 shows shared memory on and a game runs.
- LLM file is 255,530,880 bytes; voice file is 119,568,968 bytes.
- The “new version” path works after a redeploy.

---

## 8. Open decisions

- **Too-large household:** decided — warn and require confirmation, with shared/floor sleep and a `noBed` event.
- **Low target:** what frame rate counts as acceptable on the weak-GPU target, and does WebGPU stay the default there?
- **Stable ids:** does the save format bump to a new version, and what is the migration/rollback story?
- **Auto voice:** must it ship in 0.3.0, or can it slip to 0.3.1?
- **Polish:** which model/pose items are release-blocking, if any?
- **`plan/future.md`:** restore, merge, or drop?
- **Stashes:** keep or drop each, with a reason.

---

## 9. Risks

| Risk | Mitigation |
|---|---|
| Stable ids regress saves or queued actions | Land early, migrate old saves, test move/sell/reload and undo/redo, keep a save-version fallback. |
| Low-end work has no available weak GPU | Use the owner's Intel laptop measurement; if unavailable, ship the draw/LOD work with the policy explicitly unverified. |
| Scope creep from the 0.2.0 review list | Triage every row; only the must-fix table blocks the release. |
| Auto voice destabilises the release | Treat it as should-ship, not must-ship; keep manual modes unchanged. |
| Docs drift again | Make Workstream 4 a release gate, not a post-release cleanup. |
| Large model files fail on Pages | Verify the live files before announcing; keep the split/rehost option ready. |
