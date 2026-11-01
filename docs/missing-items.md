# Missing items

*Checked 2026-10-30 against `main` (58f4b1a1) + `feat/new-items`. Sources: content JSON, asset
manifests, `docs/objects/*.md`, [objects/open.md](objects/open.md), [authenticity-review.md](authenticity-review.md)
§7 and §9, and [PLAN.md](../PLAN.md) §8.*

## Verdict: no catalog item is missing

Every buyable object the docs describe exists in content and has a model:

| Check | Result |
|---|---|
| Object rows in `docs/objects/*.md` | 543 |
| Objects in `web/public/content/**/*.json` | 543 |
| Doc ids with no content entry | 0 |
| Content objects with no doc row | 0 |
| Content objects with no buy category | 0 |
| `model` keys referenced by content | 578 |
| Model keys missing from the manifests | 0 |
| Manifest entries whose file is missing | 0 |
| `item` ids in content | 44 |
| Unresolved `item` references | 0 |

**Doc bug (fixed 2026-10-30):** [objects/README.md](objects/README.md) said "541 of them", but its own
category table sums to **542** (72+26+27+20+50+14+5+19+23+39+21+43+53+36+94), and content had 542.
Cause: the prose was written on 2026-10-17, when there were 541; `curtainsWide` was added on
2026-10-21 and the table was updated, the prose was not. The count was then 542, and the wide curtains
were named as the one object not in the 2026-10-17 play check.

The empty vase, added 2026-10-30, brings both counts to **543** (tabletop 39 → 40); `item` ids to **43**
(the bunch of cut flowers), and the daisy chain, added 2026-10-31, to **44**.

So what is missing is not objects — it is **uses, clips, behaviours and polish** around them.

## 1. Modelled parts with no use or clip

| Object | Part | What is missing | Source | Effort |
|---|---|---|---|---|
| `deskExecutive` Executive desk | drawer | Drawer modelled but static — **won't fix** | I-office-02, Q-19 | — |
| `grandfatherClock` Grandfather clock | trunk door | Not a part of its own; clock is wound at the dial — **won't fix** | A-decor-01 | — |

No modelled part is left unbuilt. The laundry basket has no lid by design (bathroom review, 2026-10-21):
clean washing is heaped above its rim, and the lidded one is the bedroom's `laundryHamper`. The 2026-10-30 check
that listed a basket lid here was reading a stale line in `open.md`, now corrected.

That is the only remaining "(+use)" part, and it stays decorative. Kettle lid, bread-bin roll-top, laptop lid and
the high chair's tray were closed in 0.5.0 (I-tabletop-01, I-kids-01); the nesting tables' small table and the
display cabinet's door were built on 2026-10-31 (below).

**Done 2026-10-31 (e14b4874: the last two decorative parts get their uses).** Both were closed as decor under owner decision
4; that decision was reopened for these two parts only.
- `nestingTables`: `small_out` slides the oak table 42 cm out from under the white one (`G.slide` after `G.settle`,
  `living.py` `nesting_tables()`), with content uses `pullOut` ("Pull out the small table", 1 min) and `pushIn`
  ("Push the small table back", 1 min, `motions.uses` `{'clip': 'small_out', 'over': -1}`), `leaves: open`/`closed`,
  `motions.open`. Both are the owner's (`autonomous: false`).
- `displayCabinet`: `door_open` swings the brass-framed glass door 100° on its left hinge (`G.swing` about
  `DOOR_HINGE`, already `G.hinged`), with content uses `openDoor` (1 min) and `shutDoor` (1 min, `over: -1`).
- `cargo check-glb`: `model.nestingTables` 1,128 tris, 3 draws, 1 moving part, 1 clip; `model.displayCabinet`
  3,536 tris, 1 moving part, 1 clip — and 4 draws, one more than the still door's 3 (an animated node is exported as
  its own primitives), which is the gate's limit, so the trunk door's "fifth draw" still holds for the clock. `cargo review uses`: no `motions key` and no
  "opens, no motions.open" flags, living's "no use" count 10 → 8.
- In play: `bench-results/check-parts/parts-nestingTables-{pullOut,pushIn}-01.png`,
  `parts-displayCabinet-{openDoor,shutDoor}-01.png` (`play --state review-stage --objects nestingTables,displayCabinet
  --use all`); each use played its clip (`played small_out`, `played door_open`).
- Docs: `docs/objects/living.md` (both rows, in-play bullets), `docs/review/living.md` (I1 rows, I-living-01 and
  I-living-02 statuses, both "After" paragraphs), `docs/objects/open.md`, `tools/art/blender/families/living.md`,
  `docs/authenticity-review.md` §9.1.

## 2. Model work still accepted as incomplete

Nothing is left. The two rows that stood here (the hot tub's cover, the grandfather clock's trunk door) are
done, below.

**Done 2026-10-31 (a moving part that costs a draw):** the draw-call gate allowed 3 draws a model, 4 with a
warning, so a piece of furniture that wanted a fourth moving part could not have one. `G.Model(draws=5)` now
writes `"draws": 5` into the pack entry (`gamekit/family.py`), and `checkglb::draws_allowed` reads it: 4 unless
the entry says otherwise, a whole number from 3 to 8, anything else an error (`a_model_may_allow_itself_more_draws`).
- **Grandfather clock** (A-decor-01): the trunk door was joined into the body and never opened. It is a moving
  part now, hinged on its left edge and swung open 100° in 0.8 s (`door_open`), on "Wind the clock"
  (`motions.uses.wind`). 6,394 triangles, 5 draws, 6 materials, 4 moving parts, 4 clips
  (`previews/decor/grandfatherClock@door_open.png`); the only model with a `draws` allowance.
  Docs: `docs/objects/decor.md`, `docs/review/decor.md`, `tools/art/blender/families/decor.md`,
  `docs/design/objects.md` (draw calls).
- **Hot tub** (A-outdoor-01): the cover folds in two halves — the back half hinged on the tub's rim and the front
  flap on their seam, turned down and hung on the folded edge — and had been since `2bb30dba`, the row being
  stale: 4,368 triangles, 4 draws, 3 moving parts, 2 clips (`previews/outdoor/hotTubCedar@cover_off.png`).

The rest of §9.2 of the authenticity review (chess table for two, low apples, food wrappers, arc-lamp
overhang, washing line piece by piece, high-chair tray, kettle/bread-bin/laptop lid clips, bakery-case
door) shipped in 0.5.0 — see open.md, "Fixed in 0.5.0".

**Done 2026-10-31 (a model's own size):** what a model only carries is drawn but no longer counted in its
size. `G.aside(parts, name)` (`tools/art/blender/gamekit/parts.py`) joins those parts into one node named
`<name>_aside`; `G.Model(size=…)` measures the rest (`gamekit.parts.own`), and so does `cargo check-glb`
(`model()` skips `*_aside` nodes, `crates/tools/src/checkglb.rs`). Seven families rebuilt: the dressing
table's pouf out in front (depth 0.858 → 0.418 m), the chest of drawers' picture, vase and jewellery box
(height 1.349 → 0.89 m), the toy box's blocks (depth 0.802 → 0.518 m), the yoga mat's block and bottle
(0.919 × 0.233 × 0.27 → 0.623 × 0.132 × 0.132 m), the three desks' chairs (height 1.215/1.114/1.009 →
0.752/0.752/0.773 m) and the climbing frame's flag (height 3.45 → 2.808 m). The content's heights follow the
furniture itself (the dresser's `surface.height` stays 0.89 m). Test: `cargo check-glb`'s unit test
`what_a_model_carries_is_drawn_but_not_its_size`; `cargo check-glb` 629 models, 0 failed; `cargo test -p
ll-tools --release` passes. Docs: `docs/design/objects.md` ("Scale, axes, origin and footprint").

## 3. Behaviours not in the game yet

| Item | What is missing | Source | Effort |
|---|---|---|---|
| Furniture resizing | Resize arrows stay off (`RESIZING`, `buildmode.ts`); moving parts do not follow a resize yet | PLAN.md roadmap "later" | L |
| Smarter thoughts | No LLM re-rank of the utility AI's top-3; research only | PLAN.md roadmap "later", [design/smarter-thoughts.md](design/smarter-thoughts.md) | L (research) |

The servers' apron left this list on 2026-10-31 (below): `top.apron` is a garment of the character set, and the
staff wear it.

Flowers cut for a vase left this list on 2026-10-30 (below), on the raised bed and then on the five beds cut for a
vase (tulips, daffodils, lilies, cosmos, daisies). The daisies' "make a daisy chain" followed on 2026-10-31, so no
bed is missing a real use any more.

**Done 2026-10-31 (the servers' apron):** `top.apron` is a garment of the character set — a white shirt painted on
the body (its cut the shirt's: long sleeves, `crew -0.012`, over the trousers) under an apron shell over the front
of the trousers, from the waistband to just above the knee. The shell is clipped from the body's own torso and leg
surface inside a ±1.28 rad span round the front, arms excluded, and turned back on every open edge
(`apron_layout`/`apron`, `crates/tools/src/characters/clothes.rs`), in the top's colour with the hem band the
skirt's flag (`fields.y = 3`) gives it, and with the skirt's seated corrective (`seated_tone`, 8 mm clear of the
skin). One garment, two parts: `top.apron` (267 vertices) and `lod.top.apron` (112). In `WORKWEAR` (`outfit.ts`),
a list of its own so it is never picked for a resident but counts as a garment of both bodies (`check.rs`); the
town's staff wear it with dark trousers and dark shoes (`web/src/game/staff.ts`, `UNIFORM`), and it is drawn in
play (`bench-results/review/fixes/apron/`). `cargo characters build` → check ok (2 bodies, 58 channels).
Docs: `docs/characters-format.md` (shells, the hem band, garment ids), `docs/objects/open.md`,
`docs/authenticity-review.md`, `docs/review/community.md`.

## 4. Not yet checked (not known broken)

| Item | Why it is open |
|---|---|
| Sounds by ear (G-53) | All 45 were checked by spectrum, timing and level, never listened to. `bun run sounds` now writes every one to a WAV at the level the game plays it, with its content and numbers (`tools/audio/listen.ts`) — the listening itself is the reviewer's |
| Low on WebGL2 orbits | 11–16 % over Babylon's GPU time (0.4–0.6 ms; budget 10 %), about noise level; a headless run's GPU time is SwiftShader's, so this needs a real GPU |

**Done 2026-10-31 (`lampFloorArc`'s `at` check):** the lamp build does check `light.at`, not only the bulb's
height: `tools/art/blender/families/lighting.py` `_check_at` (called from `_m` for every lighting model,
the bulb's faces in its material against the content's `light.at` within `AT_TOLERANCE`) — and the arc
lamp passes it: `cargo art lighting --only lampFloorArc` → `ok model.lampFloorArc … 0.61 x 2.05 x 1.61 m`,
1 model, 0 failed. The line that said otherwise was left over from before 2026-10-26.

**Done 2026-10-31 (`bun run sounds`: every sound to a WAV, at the level the game plays it):**
`tools/audio/listen.ts` (also `web/package.json`'s `sounds`) renders all 45 synthesised sounds to
`bench-results/sounds/<kind>.wav` with `index.md`: its length, whether it loops or is played once, the
mixer's level, peak and RMS at that level (dBFS), its spectral centroid, and the objects and uses of the
content that sound it (a dash: no content names it — it comes from the animation's tag). The levels come
from `world.ts`'s `LOOPS` and `ONCE_LEVEL` read as `web/tests/sounds.test.ts` reads them, so one sound
against another sounds as it does in the game; the peaks land in the ledger's range (-18.0 to -11.3 dBFS,
0.13–0.27 linear). Nothing was changed by it — it is the ear's part made a five-minute listen instead of an
afternoon's. Two to hear first, from the numbers: `smash` (crockery smashed in a bin) sits at a 454 Hz
centroid, low for shards, and `tools` at 217 Hz (hammer and saw thuds, as expected).

**Done 2026-10-30:** `curtainsWide` checked in play on the stage's two-wide window: closed over both edges at
22:00, stacked at each end by day at 10:00, WebGPU and WebGL2. Its "In play" line is in
[objects/decor.md](objects/decor.md) D3.

**Done 2026-10-31 (the six held foods):** `crates/sim-core/tests/held_foods.rs` cooks each dish in a
one-room house with the shipped appliances, a dining table with two chairs pulled up and a picnic table in
the yard, and reads the hand off the snapshot (`sim::HELD`): a tasting board, a pizza, a pie, a tray of baked
treats, a barbecue platter and a sandwich each end up in the hand, and the dish is then on the table the
household eats at (the picnic table for the two cooked outdoors), with the sandwich eaten where it is made
(`eatNow`) and its dirty plate left there. A serving taken from the pizza is a `pizzaSlice` in the hand
(`a_serving_taken_from_a_pizza_is_a_slice_in_the_hand`). A pot of food from the fridge is the full path —
`ingredients`, `prepBowl`, `mealPot`, `mealPlate` — and the plate is left on the table the household sits at
(`a_meal_from_the_fridge_is_carried_to_the_table`). In play (`bench-results/check-foods/`): Ada bakes a
wood-fired pizza at a bought oven in the garden and carries it across the grass to the table
(`pizza-walk-00.png` … `pizza-walk-07.png`), a slice on a plate once it is down (`pizza-slice.png`), and a
sandwich carried from the counter to a chair (`sandwich-walk-04.png`). The dish is only carried when the
household's table is one they can sit at: with no chair pulled up the dish is put down on a free surface
within `cooking.serveRange` (6 tiles) instead, which is why the test buys chairs.

## 5. Accepted / won't fix

| Item | Why |
|---|---|
| Feet slide at game speeds 4 and 5 | 45–90 m/s; no gait steps without strobing |
| Uses end once every need they raise is full | By design (I-systems-02) |
| Plates pile up with neither sink nor bin | Every generated home has a sink (I-systems-03) |
| Breakfast at a dining set from nowhere | Kept as a game shortcut (I-kitchen-10) |
| Executive desk drawer static | Q-19 |
| Large sofas, chaise and long coffee table fill their tiles | Q-08 |
| Cactus trio and succulent bowl are floor pots | Q-08 |
| Flat kerb | Q-13 |

## 6. Tooling not there yet

Full list: [authenticity-review.md](authenticity-review.md) §7. Highest-value gaps:

**Done 2026-10-30 (content):**

- Range-hood light: the hood comes on while the hob under it cooks (`light.while`,
  `["stir", "bake"]`), by day as at night, and goes off when the cooking is done. It has no switch of
  its own; the simulation decides (`lights.rs`). Any task counts, not only a plain use: a recipe's
  Cook step is `TaskKind::Cook`, so the light watches the step's animation at the station
  (`crates/sim-core/tests/hood.rs`). Evidence: `bench-results/check-hood/` — `hood-night-final`
  (idle 22:02: no light; cooking 22:34: the wall, the hob and the floor under the hood lit) and
  `hood-day-final` (cooking 10:40: the wall under the hood lit by day too).

- `cargo review sizes` takes the highest up-face near the middle that doesn't stand on another, so a canopy
  or a rail over open air wins (`marketStall`, `crib` read "no top to measure") and a vessel basin's rim is
  not the face it takes; its sizes also include non-object geometry.
- `check.mjs play`: no whole-recipe following (it records clips only on the object used, not on the
  helpers a recipe walks to); needs carry over between uses. (`--use ripe` and the crop stage in the
  record came 2026-10-31, `--views` 2026-10-31.)
- `check.mjs sheet`, `poses.html`, gallery and previews each have known blind spots (see §7).
- CI runs clips, the GLB gate, `cargo check-glb` (since 2026-10-31), `review sizes` (2026-10-30) and
  `review uses --check` (2026-10-31); still not the browser checks.

**Done 2026-10-31 (`check.mjs play --views`: several angles of the same picture):**

- One camera per run meant a use that is only wrong from behind, from above or from the floor was a
  second run. `--views 2`–`4` takes that many angles of each use-loop picture off the aimed one: the
  front as before, then round the back (a use from behind the thing), above at 1.35 rad (a seat, a top,
  what the hands are at) and low at 0.35 rad (a door's foot, a drawer's runner, a chair's legs).
- The first picture keeps its name, so an existing folder's `<object>-00.png` is still the front; the
  others are `<object>-00-v2.png`… and each record's `angle` says which. `--views 1` (the default) is
  exactly the old run.
- `bench-results/check-views/chairWood-sit-0{0,1}{,-v2,-v3}.png` is the run that proves it: six pictures,
  three angles, one page.

**Done 2026-10-31 (`review uses --check`: a use the renderer can't play fails CI):**

- `cargo review uses` exited 0 whatever it found, so nothing could be gated on it. It has `--check` now:
  exit 1 if any object's flags include one the renderer can't play — an animation `ACTIONS` has no entry
  for (`anim not in ACTIONS`), a `motions.uses`/`motions.loops` key that is no use or animation of the
  object, or a use that opens the thing on a model with no `motions.open`.
- The other three flags stay the reviewer's: a missing use (a table with no use of its own is still a table
  the sim uses as a station — 50 of the 100 are flagged today, and that is the review's business), a
  duration out of its median, and an odd sound.
- `Flag::hard` is the rule, `all_flags` is the list the gate and the table both read, and the CI step
  **Object uses** sits after **Object sizes**. `the_shipped_content_reads` asserts the shipped content has
  no hard flag, so the gate can't quietly start failing on a change that only touched content.

**Done 2026-10-31 (`review sizes`: the faces it measured, and the kinds that were wrong):**

- The grill and the bakery case were not the probe's fault. The kettle grill's grate is a `surface` in its
  content now (0.77 m, `web/public/content/furniture.json`), so `cargo review sizes` reads the grate (0.77)
  instead of the coals under it (0.69); a display case has its own kind, `displayCase` (deck 70–85 cm), since
  a case's deck sits below a shop counter's 90–110 cm worktop (`bakeryCase` 0.78, was "out").
- Five rows of `docs/review/reference.json` were the wrong way round or too narrow, and each showed up as an
  "out" that no model change could fix: `seesaw` and `pingPongTable` keep their long axis in `w` (as
  `poolTable` and `foosball` already did), `chessTable` allows a 0.45 m depth (its seats are at the two ends
  of its length), `recoveryLounger` a 0.75 m depth (a pod is sat in, not stretched out on).
- `energetic.spinBike` and `energetic.recoveryPod` had no seat in `web/src/render/shared/characters/
  furniture.json`, so neither was in the gate at all — the spin bike never appeared in the table, the
  recovery pod read "no seat to measure". Both have one now (0.44 / 0.49 m, inside their kinds' ranges), and
  a resident on either sits at the right height.
- 125 out rows → 119, and the six that went were all the ones above. What is left is real or recorded: the
  closed lid of `toiletClassic` (0.47 m, inside the range by 1.5 cm), `barStool` 2.4 cm over, `highChair`
  5 mm over, `natureLover.pottingBench` 3 cm, `pingPongTable` 7 cm deep (the frame's splay), `climbingFrame`
  3.45 m tall, the WC's boxed cistern, and the "no top to measure" cases where a canopy or a rail hangs over
  open air (`marketStall`, `crib`) — the probe's remaining blind spot, filed here.
- The review ledgers' own notes said as much (`docs/review/community.md:29`, `:125`, `:198`: "an artefact"),
  so this was the reference catching up with them, not a new measurement.

**Done 2026-10-31 (`review uses`: the odd sound, and what the decor list silences):**

- A use's sound is now weighed against the kind that owns it: a sound that `SOUND_OWNERS` (3) objects of one
  kind make and nothing else does belongs to that kind, and any other kind making it is flagged `odd sound`
  (`Flag::OddSound`, `sound_owners`, `crates/tools/src/review.rs`) — the check that would have found
  I-packs-19 (a toaster sizzling like a pan) at once. It is a sixth column in the summary table.
  Today's content raises none: the only sounds one kind owns are the arcade cabinets', the dishwasher's
  crockery, the drum kits', the microwave cart's and karaoke's. The counters' sizzle is not a miss but a
  cooktop: `cooktopCounter` maps to the kind `baseCabinet`, as every counter does, so the hobs do not own
  `sizzle` alone and a pan on a counter is not flagged.
- The ladder shelf is not a blind spot: `docs/review/reference.json`'s `decor` list is the review's decision
  record, and the living ledger's real-uses table already reads `ladderShelf | show plants and books | none |
  OK (decor)` (`docs/review/living.md:158`, beside the industrial shelving and the cube shelf, which at least
  take things on top). The gate stays silent on purpose; what it did not do was say so, and now it does:
  `cargo review uses` ends with how many objects have no use and how many of those are decor by the
  reference's list (100 have no use, 50 are decor by the list, so the 50 flagged are the ones nobody decided).

**Done 2026-10-31 (the height gate):**

- `cargo check-glb` no longer probes a single point for a functional height: it takes the face nearest the
  expected height over a 5 cm ring round the point (`PROBE`, `Sampler::top_around`), so a gap between a
  bench's slats or a table's planks is no longer "no model under it" (`crates/tools/src/checkglb.rs`).
- A thing that stands on a top (content `layer: "surface"`) has no seat: a `sit` use of the mugs, the
  books, the laptop or the toy blocks is the resident sitting at the table they stand on, and the gate no
  longer reads a seat 45 cm lower there — eight false warnings (the tabletop things) went.
- A baby's place is the object's middle, as sim-core puts it (`place_at`), so a crib's mattress is measured
  once at its middle (0.558 m, was the changing pad under slot 1 at 0.78 m) and the high chair's seat is its
  own (0.605 m, was read at a leg at 0.21 m). `crates/tools/tests/checkglb.rs` says so.
- `outgoing.partyBar`'s seat was 0.5 m forward of its middle where its stools stand at 0.28 m: the content's
  `seat.forward` fixed, so `review sizes` and the gate measure the 0.74 m stool seat (it read "no model there") —
  and in play the sitter sat in the air behind the stools at 0.5, on a stool at 0.28
  (`bench-results/check-bar/bar28-outgoing.partyBar-barDrink-05.png`, `bar50-…-05.png`).
- In CI: a new **Models** step runs `cargo check-glb` after **Object sizes** (0.4 s; a broken model fails,
  a measure out of range is the review's to weigh) — `.github/workflows/ci.yml`.
- Warnings from the whole gate: 105 → 94, and every one of the eleven that went was a false one.

**Done 2026-10-31 (a ripe use):**

- `play --use ripe` runs every use offered only when a crop is ripe (the veggie bed's `pickSnack`, the
  trees' `pick`), and each record says the stage it found (`growth`: 0 planted, 1 growing, 2 ripe, 3
  picked) — `--use-existing garden.vegPatch --use ripe` on `review-garden` harvests a ripe bed
  (`bench-results/check-ripe/ripe3-garden.vegPatch-harvest-01.png`). Asked for by name on a crop that
  isn't ripe, the use runs and the game says why not ("the vegetable patch isn't ripe yet"), instead of
  the harness skipping it silently (`tools/bench/check.mjs`, `tools/bench/README.md`).

**Done 2026-10-30 (tooling):**

- `play --shot-idle`: a picture of each thing as bought, before any use moves it (`<object>-idle.png`).
- `play --top`: the camera above, looking down (who sits where at a table; the game camera stops at 1.35 rad).
- `play` picture names carry the `--with` helpers, so a helper run no longer overwrites the plain one
  (`chairWood-with-coffeeTable-sit-00.png`).
- Each `play` picture's `sim` records `distance`: how far the resident stands from the object's nearest
  face (a use from the back or the side is visible in the log).
- CI's **Object sizes** step runs `cargo review sizes` and fails when an object maps to no kind.
- `play --walk-shots <n>`: pictures on the way to a use, before it begins (`<object>-walk-00.png`), for
  the walk itself and for a door's swing, which is over by the time the resident is using anything.

**Done 2026-10-30 (doors by hand):**

- A house door's leaf is timed by the walker: the lead follows the game's speed
  (`doorLead(rate)` = 0.75 m, or 1.5 m/s × rate × 0.45 s), so at 1× it starts a hand's reach out and
  stands open as they arrive, and at 10× it is still open 0.6 m out. The old fixed 1.5 m lead left the
  leaf a quarter open at 3× (`web/src/render/shared/doors.ts`).
- The resident's right arm reaches for the handle as the leaf swings (`doorHand`, 0..1 over the last
  0.75 m, not while carrying a baby or holding something), and only for a door that actually swings:
  one standing open has no handle to reach.
- Tests: `web/tests/doors.test.ts` (7: the lead at each game speed, the swing finishing before the
  walker arrives, the hand out as the leaf starts and back once through, a shut leaf clear of anyone
  within 1.1 m, a wall along z). Pictures: `bench-results/check-doors/door-front-*.png` (shut, the hand
  on the leaf mid-swing, through, shut behind, and the same on WebGL2).

**Done 2026-10-30 (cut flowers for a vase):**

- The raised flower bed gains **"Cut some flowers"** (`cut`, 6 min, `anim: tend`, the bunch in the hand):
  `web/public/content/base.json`.
- **`vaseEmpty`** ("Empty vase", bought, price 15, tabletop / plants) takes the bunch: "Put the flowers in"
  (3 min, `brings: cutFlowers`, `leaves: open`) grows the vase's own bunch up out of the mouth over 0.9 s
  (`flowers_open`) and it stays up while the vase is open, saved that way; "Take the flowers out" (2 min) runs
  the same clip back along the use (`{'clip': 'flowers_open', 'over': -1}`), so the flowers go down as they are
  lifted out.
- The bunch is one new held item (`item.cutFlowers`, `hold: hand`, 0.37 m, 624 tris, one draw) built from the
  same six heads as the vase of flowers (`VASE_HEADS` / `BUNCH`), so a bunch cut in the garden and one bought
  arranged look alike (G-10); the empty vase shares the filled one's body (`VASE_BODY`).
- Tests: `crates/sim-core/tests/flowers.rs` (the bed gives a bunch in hand, the vase refuses `takeOut` while
  closed, carries the bunch in the arms on the way, opens, keeps it open through a save and load, closes when
  they are taken out). In play: `bench-results/check-flowers/` (`cut3-flowerbed-cut-00.png`,
  `vase2-vaseEmpty-{idle,putFlowers-01}.png`; `flowers3-*` the take-out run).
- The five beds cut for a vase gained the same use the same day: "Cut some tulips" / daffodils / lilies /
  cosmos / daisies (6 min, `anim: tend`, `holds: cutFlowers`, never chosen on their own) — `garden.json`;
  `crates/sim-core/tests/flowers.rs` cuts a bed of tulips and checks that every bed of flowers the garden sells
  has the use and holds the same bunch. In play: `bench-results/check-flowers/beds-*.png`.
- Docs: `docs/objects/tabletop.md` (40 things, `vaseEmpty` in TT3), `docs/review/tabletop.md`,
  `docs/objects/garden.md` and `docs/review/garden.md` (`flowerbed`'s `cut`, the five beds'), `docs/authenticity-review.md`
  (Q-18, G-10, G-68, §9.1 I-garden-08).

**Done 2026-10-31 (the daisies' daisy chain):**

- The daisies' bed gains **"Make a daisy chain"** (`daisyChain`, 12 min, `anim: tend`, `holds: daisyChain`,
  never chosen on its own) — `web/public/content/garden.json`. It was the one real use of a flower bed the
  garden still lacked (`docs/review/garden.md`, I-garden-08's After).
- One new held item: **`item.daisyChain`** (`hold: hand`, 16 cm across, 1,034 tris, one draw, three materials) —
  a ring of plaited green stems with eleven white daisies round it, heads up, the stem ends tucked back under
  where the chain was closed; the daisies are the beds' own white and amber (`tools/art/blender/families/held.py`,
  `daisyChain()`). `item` ids: 43 → **44**.
- Test: `crates/sim-core/tests/flowers.rs` `the_daisies_make_a_daisy_chain` (the chain in the hand while it is
  made, and only then).
- In play: `bench-results/check-chain/chain-garden.daisies-daisyChain-01.png` (Ada kneeling at the daisies, the
  chain coiled in her hand), `play-chain.json` (used after 3.9 s; `tend` has no clip, as the other cuts).
- Docs: `tools/art/blender/families/held.md` (the item), `docs/objects/garden.md` (daisies row, in play),
  `docs/review/garden.md` (I1 row, I-garden-08's After), `docs/authenticity-review.md` (Q-18, I-garden-08),
  `docs/objects/open.md`.

**Done 2026-10-30 (servers walk in):**

- A server now comes in to work instead of standing at their post from the first minute: `Server.home` is the
  lot's door (`Plot::arrival_point()`, the same door a visitor uses), they are placed there off shift, and at
  opening they walk to their post at the game's walking speed (`crates/sim-core/src/staff.rs`, `walk`). At
  closing they walk back out and are gone once they reach the door, so the cart stands unstaffed afterwards.
- Tests: `crates/sim-core/tests/public_places.rs` — `a_vendor_walks_in_at_opening_and_out_at_closing` (at the
  door before ten, drawn walking in, not at the post a minute later, there within ten minutes; back at the door
  and still after 21:30) and `the_barista_steps_over_to_the_machine_and_the_cafe_closes` (on the way from the
  door at a minute past ten, then at the machine).
- They wear a uniform: a white shirt under a white apron (`top.apron`), dark trousers and dark shoes over their
  own face, hair and skin (`web/src/game/staff.ts`, `UNIFORM`). The apron is a garment of the character set since
  2026-10-31 (below).
- In play: `bench-results/check-staff/staff-cart-before-opening.png` (the ice cream cart unstaffed at 09:55),
  `staff-cart-arrived.png` (the vendor at it in the uniform), `staff-posts.png` (three posts staffed at once).
  The walk itself is 18 seconds of game time, a frame or two at a bench's clock, so the stills show the cart
  empty and then staffed; the path between is the sim's (the test above), drawn with the residents' own walk.
- Docs: `docs/objects/open.md` ("Content and sim"), `docs/review/community.md` (I-community-09's After).

## Suggested next items

Nothing from the review ledgers is left unbuilt: the remaining work is the not-yet-checked list in §4 and the
tooling in §6. Every flower bed now has its real uses.
