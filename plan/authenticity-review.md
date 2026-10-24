# Idyll Lives — Authenticity Review Plan (models, animations, interactions)

- **Base:** `main` at `e725166` (v0.3.0)
- **Status:** Done (2026-10-22): review, triage and Phase 4 fixes, signed off by the owner and merged into `main` (`docs/authenticity-review.md`)
- **Output:** `docs/authenticity-review.md` (findings, fixes, leftovers, laid out like `docs/release-0.2.0-review.md`) plus one ledger per category in `docs/review/`

## 1. Intent

Review every model in the game on three questions:

1. **Model authenticity.** Does it look like the real thing at real size, with the right parts, proportions and materials, and is it recognisable from the game camera?
2. **Animation authenticity.** Do residents and moving parts move the way people and real mechanisms move: weight, contact, timing, hinge side and angle, transitions?
3. **Interaction authenticity.** Does each object offer the uses a real one has, used from the right place, in the right pose, for a believable time and with believable effects? Do social interactions look right?

"Authentic" is measured against the game's art style ([PLAN.md](../PLAN.md): modern, minimalist, stylised). Simplified shapes are fine. Wrong sizes, missing parts that make a thing recognisable, impossible motion and nonsensical uses are not.

**Success criteria**

- Every item in §2 has a ledger row with a verdict on each of the three questions that applies to it.
- Each finding has an id, a severity, a fix class and a file to fix (§4).
- The automated checks in §5.1 exist and run in CI, so the measurable part does not regress.
- The findings are triaged into fix now, a later release or won't fix, and the owner decisions in §9 are followed.

**Non-goals**

- No fixing during the review passes (Phases 1–3). Fixes come in Phase 4, after triage.
- No balance rework. A wrong duration or effect is recorded, but content numbers change only with `balance.rs` re-run (§7).
- No new art direction or realism pass. The style stays.
- No performance work, beyond keeping the existing budgets (triangles, draws) for any rebuilt model.

---

## 2. Scope: what gets reviewed

Counted on `e725166`.

| Group | Count | Source of truth | Built by |
|---|---|---|---|
| Buyable objects | 541 (577 `.glb` including non-content props, plus 105 `.far.glb`) | `web/public/content/{base,furniture,garden,community}.json`, `catalog2/*.json` (11), `packs/*.json` (16); per-category pages in `docs/objects/*.md` | `tools/art/blender/families/*.py` |
| Held items and tools | 20 models (17 `item.*` for 25 items, 3 `tool.*`) | `base.json` `items`; `families/held.py` | Blender |
| Props no content object names | 16 (9 `model.street.*`, 4 ground, 3 hedges and bushes) | `street.ts`, `nature.ts` | Blender |
| Houses | walls, 7 door styles, 6 window styles, 25 wall and 13 floor coverings, roofs, porches, chimneys, garages, neighbour shells | `base.json` style lists | code: `web/src/render/babylon/house.ts` |
| Town, built in code | fences, roads and pavements, 4 car bodies, streetlights, mailboxes, hydrants, bins, shop silhouettes | `fences.ts`, `roads.ts`, `street.ts` (`CARS`) | code |
| Residents | 2 bodies × 6 life stages, 11 hairstyles plus bald and beard, painted clothes (7/3 male, 6/4 female tops and bottoms, 4 shoes), garment shells | `docs/characters-format.md`, `docs/design/look.md` | `cargo characters` (`crates/tools/src/characters/`) |
| Resident clips | about 55 clips (locomotion, seated, use and cooking, social, garden, lying, baby, street) plus procedural layers | `web/src/render/shared/characters/clips/index.ts` | TypeScript key poses |
| Object motions | 316 of 577 models (uses, loops, idle, ride, open, night, growth, clock) | `web/public/assets/packs/*/manifest.json` `motions` | Blender clips, `objectMotion.ts` |
| Code-driven motion | front and bathroom door swing, lamp fades, blankets, held grips | `BabylonRenderer.swingDoors`, `characters.ts` | code |
| Object interactions | 599 interactions on 541 objects; 159 objects have none | content `interactions` | content and `characters.ts` `ACTIONS` |
| Social interactions | 19 socials, 5 phone actions, group conversations up to 4 | `base.json` `socials`, `phone.actions`; `conversation.rs` | sim and `characters.ts` |
| Special systems | cooking (9 recipes over stations), baby care, phone, lamps, objects that open, crops, visitors knocking | `cooking.rs`, `babies`, `phone.rs`, `lights.rs`, `when`/`leaves`, `grows` | sim and renderer |

---

## 3. Rubrics

Each reviewer works from these lists and records a verdict per line: **OK**, **finding** (with an id) or **n/a**.

### 3.1 Model authenticity (M)

**Size and proportion**
- M1. Overall W × H × D is inside a real-world range for the thing (§3.4 reference table). Built size comes from the `cargo check-glb` summary line, not from the family notes.
- M2. Functional heights match people: seat, table top, counter, desk, bed mattress, sink rim, toilet seat, shelf reach, light switch, handle heights. They also agree three ways: content (`seat.height`, `surface.height`, `light.height`), the renderer tables (`web/src/render/shared/characters/furniture.json`) and the mesh. A height up to 1 cm out (5 mm on table tops) counts as inside: the modelling grid and bevels move it that much.
- M3. Parts are in proportion to each other: a fridge handle against the door, a drawer against the carcass, a tap against the basin.
- M4. The model fits its footprint and layer: floor, surface, wall back on the wall face, ceiling hanging from 2.8 m. A model may be smaller than its tile only where a real one would be.

**Recognisability**
- M5. You can name the object from the default Live camera at Medium without a label, and from Build-mode zoom.
- M6. It has the parts that define the thing: a toilet's cistern and seat, a stove's hob and controls, a washer's porthole, a bike's chain line, a piano's keys.
- M7. Nothing is plainly wrong: a door handle on the hinge side, a tap that can't reach the basin, a lamp with no visible source, legs that don't reach the floor, floating parts.
- M8. Its front faces the way residents use it (front is +Z in game).

**Material and finish**
- M9. Each finish suits its material: wood is wood, hardware is metal or chrome, fabric is fabric, glass is glass. The `body` paint part is the part a buyer would choose the colour of.
- M10. It reads right in the day and night lighting and with the night glow (shades, screens). Nothing glows through a shut lid (known issue).

**Consistency**
- M11. Models of one kind agree: every dining chair has a similar seat height, every counter is 0.90–0.95 m, every single bed the same mattress height unless a variant says otherwise.
- M12. The far model keeps the silhouette and colours of the full model.
- M13. Quality is about the same as its neighbours in the same room. Note outliers in detail level, both too plain and too busy.

**Residents (in addition)**
- M14. Height and head-to-body ratio per life stage are in a believable range for the style, which draws heads about half a head bigger than life (baby about 1:3.5, child 1:4.5–5, teen 1:6–6.5, adult 1:6.5–7, elder a little shorter with a stoop). Check `rig.rs` `proportions` and `target_height`.
- M15. Hair sits on the head without clipping at the usual head turns. Garments are plausible as clothes (collar, hem, sole), and skirts and jackets don't clip at sitting or walking.
- M16. The `build` slider extremes (0.85, 1.2) still read as people.

**Houses and town (in addition)**
- M17. Doors are about 2.0–2.1 × 0.8–0.9 m, window sills sit at a sensible height, wall height and storeys suit the doors, stairs are walkable (rise and run), roofs read as roofs.
- M18. Cars, streetlights and street props are at real scale next to residents.

### 3.2 Animation authenticity (A)

**Residents**
- A1. Contact: feet don't slide, the seat carries the pelvis, a lying resident touches the mattress, hands touch what they use (handle, tap, keyboard, pan) and don't hover or pass through it. A resident's pose at an object (reach, eye line, contact with its parts) is filed under I3 with A1 in its Rubric field; the A column names the same id.
- A2. Weight and balance: the centre of mass sits over the support, sitting down lowers into the seat, standing up pushes off, and carrying changes the posture.
- A3. Timing: walk and jog cadence fit the ground speed at each game speed, gestures last a believable time, and loops have no visible seam.
- A4. Transitions: no pops between clips. Approach, turn, sit and lie happen in a believable order: no gliding back to the tile, no spinning on the spot without stepping.
- A5. Age: children, teens and elders share adult clips rescaled. Check that each clip still reads right per stage (short legs, child reach, elder speed and stoop). Babies have their own clips.
- A6. Emotion: body language matches the feeling in the snapshot. Today only the face does (a known gap).
- A7. Pairs and groups: a hug, kiss, fight or conversation lines up for every pair of heights (adult and child, tall and short). Groups face each other and keep a real spacing.

**Objects**
- A8. The hinge or pivot is where a real one is, on the right side, through the right angle (fridge door about 110–130°, oven door drops down, a laptop lid about 100–110°, a wardrobe door about 90–110°).
- A9. Speed and easing suit the mechanism: doors swing, drawers glide, lids fall faster than they rise, a washing drum spins at plausible speeds.
- A10. In time with the resident: a part opens when the hand gets there and closes when it leaves (`delay`, `hold`, `out`), not on use start alone. A ride part carries the rider with no gap.
- A11. Ambient clips (fan, fountain, lava lamp) loop with no seam and at a believable rate.
- A12. Code-driven motion: house doors (proximity only today, no hand), lamp fades, blankets, and held grips during instrument and weights loops.
- A13. Sound, where there is one, starts and stops with the motion (`web/src/audio/world.ts`).

### 3.3 Interaction authenticity (I)

**Per object**
- I1. Coverage: the object offers the uses a real one has, and none it shouldn't. Each of the 159 objects with no use is sorted into "decor, no use needed" or "missing use" (a lamp you can switch counts as a use).
- I2. Place: the resident uses the object from the side and slot a real person would (`approach`, `slots`, `slot_position`, `inside`, `seatNear`). Two-person objects seat both people correctly.
- I3. Pose and clip: the `pose` and `anim` fit the use (sitting on a toilet, standing at a counter, kneeling to weed), and the `ACTIONS` entry for that anim gives a fitting clip, hand pose and layer at this object.
- I4. Duration: `minutes` is believable in game time (a shower 10–15, the toilet 3–5, a meal 15–30, a night's sleep 7–9 h) or is a deliberate game shortening noted in the ledger.
- I5. Effects: needs, skills and feelings point the right way and are in sensible proportion to similar uses (a bath relaxes more than a shower, a nap rests less than a night's sleep).
- I6. Who may use it: age (nothing stops a child using a stove today), the phone from teens up, care uses for adults, homework for pupils. Privacy (no private flag exists; the bathroom door is renderer-only). Visitors use only non-private objects.
- I7. Moving part and prop: the right part moves for the right use (fridge for snacks and ingredients, freezer for frozen things), and the thing in hand is shown or plausibly mimed. No phone, book, cup or cutlery prop exists today.
- I8. Open, closed and ripe states make sense (`when`, `leaves`, `closeAfter`): who opens and closes, and what happens to an occupant.
- I9. Sound: uses that make noise in life have it, or are listed as missing.

- I16. Conditions: a use whose real thing needs a time, a place or a state (night for stargazing, outdoors, someone to play with, a baby in the crib) has it, or is filed.

**Social**
- I10. Each social has a fitting clip, distance and facial expression, and its success and failure look different.
- I11. Group conversations: joining, roles, glances and spacing.
- I12. Phone actions look like phone use.

**Systems**
- I13. Cooking: the station order is a believable kitchen flow (fridge → prep → hob or oven → serve → eat → wash up), carried items appear at each step, and props sit on the right part of the station (known: drawn at the middle of the grill).
- I14. Baby care: feed, change, play, cuddle and sleep look and last right.
- I15. Lamps, doors, visitors knocking, crops (plant, water, harvest) and objects that open.

### 3.4 Reference table

Before Phase 1 starts, a reviewer writes `docs/review/reference.md`: real-world size ranges for every kind in the catalog (seat 42–48 cm, dining table 72–76 cm, kitchen counter 90–95 cm, desk 72–76 cm, bar stool 65–80 cm, single bed 90 × 190–200 cm, double 135–160 × 190–200 cm, mattress top 45–60 cm, toilet seat 40–43 cm, basin rim 80–90 cm, bath 170 × 70–80 cm, fridge 55–70 cm wide, interior door 2.0–2.1 m, and so on), the uses of each kind, typical durations, and the hinge angles of common mechanisms. Each range cites its source (standards, catalogue sizes, ergonomics tables). The automated check in §5.1 reads its machine-readable twin (`docs/review/reference.json`).

---

## 4. Findings: format and severity

**Ids:** `M-<cat>-NN`, `A-<cat>-NN`, `I-<cat>-NN` (for example `M-kitchen-07`), where the category is the `docs/objects/` page or `residents`, `houses`, `town`, `social`, `systems`.

**Severity**
- **S1 — breaks belief:** wrong at a glance in normal play (a resident floats above the bed, a door opens into the wall, a child-sized adult chair, a use from the back of the object, an adult using a child's or a baby's thing, or a child an adult's, as if it fitted).
- **S2 — noticeably wrong:** a careful player sees it (hands hover, a counter 10 cm too low, a 2-minute shower, a missing obvious use).
- **S3 — polish:** only on close inspection or a side-by-side.

A `decision` finding takes the severity of its worse outcome.

**Fix class:** `model` (Blender rebuild), `content` (numbers or fields), `renderer-table` (`SEATS`, `BEDS`, `ACTIONS`, `STATION_PROPS`), `clip` (resident clip), `motion` (object clip or manifest `motions`), `code` (new behaviour), `decision` (needs the owner).

**Ledger row** (`docs/review/<cat>.md`, one table per group, in the order of `docs/objects/<cat>.md`):

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `fridge` | OK | A-kitchen-03 | OK | A-kitchen-03 S2 motion: freezer door opens at use start, not at the hand | `review/kitchen/fridge-*.png` |

Shots stay in the gitignored `bench-results/review/<cat>/`, not in git. The Shots cell names the object's contact sheet (`sheets/obj-<id>.png`, from `check.mjs sheet`) and at most two more shots.

**Status values:** `open`, `seeded (from <source>)`, `confirmed`, `confirmed and widened`, `confirmed fixed on <item>`, `not checked (<why>)`, `closed (<why>)`, and in Phase 4 `fixed (<commit>)`, `later`, `won't fix (<why>)`.

**Causes shared by many objects:** a finding whose cause is in the sim or the renderer and not in one object (a `Command::Use` rule, an `ACTIONS` entry, a clip) goes in the ledger of the first object it was seen on, naming every other affected item; the other ledgers' rows point to it. The coordinator moves it to `systems.md` or `residents.md` when the wave is merged. Each object wave judges the `ACTIONS` entries its objects use, at those objects; the cross-cutting wave takes the rest.

**Real uses (I1):** under each group, a table "Real uses against the content's" lists the real-world uses of each item next to the content's.

---

## 5. Phase 0 — Tooling and baseline

Do this before any reviewing, so every reviewer works from the same evidence. All new tools are Rust in `crates/tools` with a cargo alias; Python only inside Blender.

### 5.1 Automated checks (new or extended)

- **T1. Functional-height check** in `cargo check-glb`. For every model with `seat`, `surface` or `light` in content, measure the mesh at that point and warn when the mesh and content differ by more than 2 cm. Today only four family scripts assert heights.
- **T2. Three-way seat and bed check.** Content `seat.height`, `SEATS`/`BEDS` in `characters.ts` and the mesh must agree (the known sofa bed is 0.44 against 0.42 m). Export the two tables as JSON at build time, or read them with a small parser, so the Rust check doesn't depend on TypeScript.
- **T3. Reference-range check.** `cargo review sizes` compares each object's measured W × H × D and functional heights with `docs/review/reference.json` by kind and prints the outliers as a table. This is the first pass of M1, M2 and M11 for all 541 objects at once.
- **T4. Interaction audit.** `cargo review uses` lists, per object, uses, pose, anim, minutes, effects, slots, approach, motions and sounds in one table, and flags: objects with no use, uses whose anim has no `ACTIONS` entry for that object kind, `motions` keys that name no use, uses that change an open state with no `motions.open`, and durations or effects far from the median for their tag. This feeds I1–I5 and I7.
- **T5. Clip checker in CI.** `tools/characters/check_clips.ts` already checks seams, foot slide and contact. Run it for every clip at each life stage (A5) and record today's failures (`jogAlong`) as the baseline.

### 5.2 Fresh evidence

- **E1. Previews.** The renders in `tools/art/blender/previews/` are old and 161 models have none. Ids used in two families (bench, flowers, hedge, arcadeCabinet, puzzleTable, recliner) overwrite each other. Name previews `<family>/<id>.png`, then run `cargo art --all --preview`.
- **E2. In-game gallery.** `tools/bench/check.mjs gallery` per category, at Medium, headless (full Chrome for Testing for the real GPU, [tools/bench/README.md](../tools/bench/README.md)). Two views per object: the default Live camera distance, and Build-mode close. A resident stands next to each for scale; add that to `gallery.html` if it isn't there.
- **E3. Clips.** `check.mjs clips` for every model with motions: shots at rest, mid-way and open (`t=99`).
- **E4. Poses.** `poses.html` strips for each object that has a seat or bed, with every life stage that may use it, and `characters.html?stage=all` for every resident clip.
- **E5. Play.** `check.mjs play` for every use: buy in `?state=bathroom-check` (free walls at `[23,14]` and `[24,17]`), free will off, `use`, timed shots and a log of pose, clip and `__renderer.motion.states`. Outdoor, community and town things use `town-things`, `shops-town` and `stalls-town`. New states needed: a family with every life stage (extend `family-check`), and a kitchen with every station for cooking. Write them with `DUMP_STATES=1 cargo test -p sim-core --test check_states`.
- **E6. Known issues as seeds.** Copy into the ledgers, already filed, the open items from `docs/objects/open.md` ("Still open"), `plan/release-0.3.0.md` §1.4 and §2.5, and `docs/release-0.2.0-review.md` §4 (R1-04, R1-05, R1-12, R1-15). That way reviewers confirm or close them rather than rediscover them.

**Phase 0 done when:** T1–T5 run, E1–E5 have produced shots for every item, `reference.md` exists, and the seeds are in the ledgers.

---

## 6. Phases 1–3 — The review passes

### 6.1 Order

The objects every home has come first, because players see them most:

1. **Core home:** the 28 objects in `base.json` (bed, toilet, shower, fridge, stove, sofa, TV and so on) and the residents. This is the pilot: it tests the rubric and the ledger before the rest.
2. **Rooms:** kitchen (72), bathroom (26), bedroom (27), living (50), office (14), kids (20).
3. **Activities:** hobbies (19), fitness (5), tabletop (39), decor (22), lighting (21).
4. **Outside:** outdoor (43), garden plants and trees (53), community and farm (36), street props, houses and town built in code.
5. **Personality packs** (94).
6. **Cross-cutting:** social interactions, group conversations, phone, cooking, baby care, lamps and doors, life stages over all seats and beds.

### 6.2 Per category

One agent per category, given the rubric and that category's evidence from Phase 0. It works in three steps:

1. **Model (M).** Start from the T3 table, then go through the gallery shots and previews against the reference. File M findings.
2. **Animation (A).** Clip shots and `poses.html` strips, then the resident clips used at these objects in play. File A findings.
3. **Interaction (I).** Start from the T4 table, then the play logs and shots. For each object, list its real-world uses next to the content's uses. File I findings.

Each finding gets a shot and the file and line to change.

**Pilot (step 1 of §6.1) is reviewed by the owner** before the other categories start. The rubric, severity levels and ledger format get one round of changes there.

### 6.3 How agents run

These follow the project's standing rules:
- At most 11 agents at once, one CPU core each: one browser page or one Blender job at a time.
- Each agent starts its own vite from its own worktree, with copied (not symlinked) wasm packages. Worktrees branched from `origin/main` merge `main` first.
- Browser checks are headless, at 1 % volume (`addInitScript` from `tools/bench/quiet.mjs`). Headed only for anything timed.
- Every agent keeps a `log.md`: steps, commands, durations, failures, retries, tooling problems and ideas for the tools.
- Reviewers only read and file findings. They don't edit models, content or code.
- Remove each worktree and branch when its ledger is merged.

A suggested wave: pilot (1 agent) → rooms (6) → activities (5) → outside (4) → packs (2–3) → cross-cutting (3), with one coordinator merging the ledgers into `docs/authenticity-review.md` after each wave.

---

## 7. Phase 4 — Triage and fixes

1. **Dedupe and group.** Many findings share a cause (one `SEATS` row, one `ACTIONS` entry, one clip used by 40 objects). Group them by fix, not by object.
2. **Triage** each group: fix now, a later release, or won't fix (with a reason), decided with the owner. S1 is fixed now unless the owner says otherwise.
3. **Fix in this order:** content and renderer tables (cheap, broad), object clips and `motions`, resident clips, Blender rebuilds, then new code (props in hand, emotional body language, turning on the spot, hand-timed doors, age rules, privacy), each as its own planned item.
4. **Rules for fixes:**
   - A Blender rebuild goes through `cargo art <family>` and passes `cargo check-glb`, T1 and T3, inside the triangle and draw budgets in `docs/design/objects.md`. The family `.md` notes are updated.
   - A content change to `minutes` or `effects` re-runs `cargo test --release --test balance -- --ignored --nocapture` and the soak test, and the result goes in the finding.
   - A changed clip passes `check_clips.ts` at every life stage.
   - Each fix updates its ledger row and `docs/objects/<cat>.md` (uses, clip, "In play" note), so the docs keep matching the code.
5. **Re-check:** the Phase 0 evidence is generated again for the changed items, and each finding is closed with a before and after shot.

---

## 8. Deliverables

- `docs/review/reference.md` and `reference.json`: real-world ranges with sources.
- `docs/review/<cat>.md`: one ledger per category.
- `docs/authenticity-review.md`: summary, findings by category and severity, owner decisions, fixed, found but not fixed, a manual test checklist.
- `cargo review sizes`, `cargo review uses`, the extended `cargo check-glb` (T1, T2) and `check_clips.ts` in CI.
- Preview naming by family, and new debug states for life stages and a full kitchen.
- After Phase 4: the fixes, with `PLAN.md`, `CHANGELOG.md` and `docs/objects/` brought up to date.

---

## 9. Owner decisions (2026-10-21)

1. **Sizes:** functional heights (seat, counter, table, desk, mattress, basin, toilet seat) must be inside the real-world range. Overall size and parts may stray about ±10 % for readability from the camera.
2. **Durations:** today's short durations are by design. The ledger records them as "by design" and flags only outliers against similar uses (more than 3× from the median for the tag).
3. **New behaviour** (props in hand, emotional body language, a turn-on-the-spot clip, doors opened by hand, hard age rules, a privacy flag): the review files it as `code` findings only. It is planned separately later.
4. **Decor without uses** (default): decor stays decor. A clear missing use is an I1 finding, decided at triage.
5. **Residents' proportions** (default): changes to `rig.rs` are filed only and decided at triage.
6. **Pace** (default): the review runs as its own workstream now, on branch `review/authenticity`, through Phase 4.
