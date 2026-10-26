# Systems: authenticity ledger

What it covers: the behaviour that spans objects: cooking and its 9 recipes over stations (`cooking.rs`, `base.json` `recipes`, `cooking`), baby care at the crib, lamps (`lights.rs`), house doors, objects that open and stay open (`when`, `leaves`), crops (`grows`) and visitors knocking (`base.json` `visits`). Each object's own uses are judged in its own ledger; these rows judge the system. Rubric I13–I15 in particular.

- **Evidence:** `bench-results/review/systems/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Code references:** paths are the PlayCanvas renderer's (`web/src/render/pc/`, `PlayCanvasRenderer.ts`; both renamed when the renderer was ported from Babylon). Line numbers in `Where:` were taken before that port and no longer land on the code (some past the file's end): read the symbol named beside the number.
- **Never looked at before** (`docs/objects/open.md` "Not looked at"): a meal from the fridge eaten at a pulled-in chair (eating was checked with the café's cake). Cross-cutting wave: seen, the sandwich eaten at the dining table on a pulled-in chair with its plate on the table (`held/inhand/sandwich-04.png`).
- **Cross-cutting wave (2026-10-21):** the recipes from the held wave's runs on `family-check` (`held/inhand/<recipe>-NN.png` and `timeline.jsonl`; contact sheets `systems/sheets/cook-<recipe>-N.png`), the kitchen, outdoor and packs waves' station shots, and scenes of its own (`bench-results/review/scripts/scenes.mjs`, log in `systems/scenes/scenes.log`): baby care on `review-baby` (a copy of `review-stage` with the baby hungry and wet and free will on), lamps on `lamps-night` with free will on, the toilet and the front door on `bathroom-check` (also from above, `systems/doors-top/`), a visitor invited over on `calling-out` (a fixed camera on the lot's entry). Systems have no model of their own: M is `n/a`, the models are in their ledgers.

## Recipes (9)

Steps from `base.json` `recipes` (station, clip, minutes).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `meal` Make a meal: ingredients (fetch, 3) → prep (chop, 10, optional) → stove (stir) / oven, microwave (bake), 20, optional (minutes) | n/a | I-kitchen-04 | I-systems-04, I-kitchen-05 | Fridge (door open, the hands in it) → counter (board, chopping) → hob (pan on a lit burner) → the pot left on the hob → eaten at the island → dishes: a real kitchen flow. I-kitchen-04 S2 code (the crate and the prep bowl put down on the station's top while the step is worked; fixed 57b3a576); I-kitchen-05 S2 renderer-table (at an oven or microwave the step is the `bake` tag now, not stirring a pan; not shot at a microwave; fixed 2dcc4ea0); I-systems-04 S2 code (recipes take `stages`: a child is refused; fixed 697544c2) | `systems/sheets/cook-meal-1.png`, `systems/sheets/cook-eat.png` |
| `gourmet` Cook a gourmet meal: ingredients (fetch, 3) → prep (chop, 20) → stove (stir, 45) (minutes) | n/a | I-kitchen-04 | OK | As `meal` at the stove; the dirty pot scraped and washed after. I-kitchen-04 S2 code (fixed 57b3a576) | `systems/sheets/cook-gourmet-1.png`, `systems/sheets/cook-gourmet-2.png` |
| `tastingMenu` Cook a tasting menu: ingredients (fetch, 3) → prep (chop, 30) → stove (stir, 45) (minutes) | n/a | I-kitchen-04 | OK | Prep and cook at the one chef's island, then the tasting board served there. I-kitchen-04 S2 code (fixed 57b3a576) | `systems/sheets/cook-tastingMenu-1.png` |
| `bbq` Grill dinner: ingredients (fetch, 3) → prep (chop, 10, optional) → grill (flip, 25) (minutes) | n/a | I-kitchen-04 | I-systems-01 | Out to the kettle grill with the bowl, food turned on the grate; the platter served on the kitchen counter. I-kitchen-04 S2 code (the bowl put down to turn the food; fixed 57b3a576); I-systems-01 S2 renderer-table (confirmed: the grill station's chopping board drawn on its grate; fixed 65278fa4) | `systems/sheets/cook-bbq-1.png`, `outdoor/play/chunk5-kitchen/grillStation-cook-01.png` |
| `bbqFeast` Grill a barbecue feast: ingredients (fetch, 3) → prep (chop, 20) → grill (flip, 50) (minutes) | n/a | I-kitchen-04 | I-systems-01 | The `bbq` steps, longer (not run: the same stations and clips); at the grill station I-systems-01 (`outdoor/play/chunk5-kitchen/grillStation-feast-01.png`), and I-kitchen-04's bowl put down as there | `outdoor/sheets/obj-grillStation.png` |
| `pizza` Bake a wood-fired pizza: ingredients (fetch, 3) → prep (chop, 15, optional) → oven (bake, 25) (minutes) | n/a | I-kitchen-05 | I-kitchen-05 | The pizza slid in on a tray and the oven door shut (`bake`), not a pan stirred at its side (I-kitchen-05, fixed 2dcc4ea0); served on the kitchen counter, 6 m off, fine | `systems/sheets/cook-pizza-1.png` |
| `bakedTreats` Bake treats: ingredients (fetch, 3) → mix (mix, 15, optional) → oven (bake, 35) (minutes) | n/a | I-kitchen-05 | I-kitchen-05 | The tray in the oven and the door shut for 35 minutes (I-kitchen-05, fixed 2dcc4ea0), the batter mixed at the counter before (the Mix step's own tag); the tray served on the counter | `systems/sheets/cook-bakedTreats-1.png`, `systems/sheets/cook-ovens.png` |
| `pie` Bake a family pie: ingredients (fetch, 3) → make the pastry (mix, 15, optional) → oven (bake, 30) (minutes) | n/a | I-kitchen-05 | I-kitchen-05 | The neighbourly oven: the pastry rolled on its worktop, the pie dish in and the door shut (I-kitchen-05, fixed 2dcc4ea0) | `systems/sheets/cook-pie-1.png` |
| `sandwich` Make a sandwich: ingredients (fetch, 2) → prep (chop, 7) (minutes) | n/a | OK | OK | Fridge → counter → eaten at the dining table with the plate drawn | `systems/sheets/cook-sandwich-1.png`, `held/inhand/sandwich-04.png` |

Durations are by design (§9.2); none is an outlier against the others (the feasts and gourmet dishes 2–3× a plain meal).

## Cooking flow (7)

`crates/sim-core/src/cooking.rs`; props `STATION_PROPS` in `characters.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Props on the station (board, pan, grill food) | n/a | OK | I-systems-01 | I-systems-01 S2 renderer-table (confirmed; fixed 65278fa4); the board on a counter and the pan on a hob are right (`cook-meal-1`), the pan on an oven's or a pizza oven's top was I-kitchen-05 (fixed 2dcc4ea0) | `systems/sheets/cook-meal-1.png`, `outdoor/play/chunk5-kitchen/grillStation-cook-01.png` |
| Serving out (`serving`) | n/a | OK | OK | The dish put down on a counter by the last station (the pot on the hob, the tray, platter and pizza on the counter, the pie dish on the oven) | `systems/sheets/cook-pizza-1.png`, `systems/sheets/cook-pie-1.png` |
| Eating at a table or seat | n/a | OK | OK | At the island on a bar stool and at the dining table on a pulled-in chair, `eat` with the hands at the mouth (no cutlery: I-held-01, held ledger); the white plate on the white island barely shows, only the food on it (`cook-eat.png`) | `systems/sheets/cook-eat.png` |
| Scraping a plate | n/a | OK | OK | A minute at the bin (`clean`), then to the sink with the stack | `held/inhand/gourmet-08.png`, `held/inhand/bbq-09.png` |
| Washing up (sink, dishwasher, bin) | n/a | OK | I-systems-03 | At the sink (`wash`); I-systems-03 S3 decision (won't fix, Q-17: every generated home has a sink and a bin; owner, 2026-10-21) | `held/inhand/meal-08.png` |
| Putting clean things away (`putAwayAt`) | n/a | OK | OK | `fetch` at a cupboard or counter, two minutes | `held/inhand/gourmet-11.png` |
| Leftovers and spoiling | n/a | n/a | OK | 8 hours on a counter, 72 in the fridge (`cooking.spoilHours`, `storedSpoilHours`), spoilt food greyed (`items.ts:151`); waiting cooks: I-systems-10 S2 code (two cooks stand inside each other at the fridge; fixed 36198175) | n/a (code read) |

## Baby care (7)

The crib (`base.json`), `babyWakes` and the cuddle lift (`characters.ts`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `lie` Lie in the crib (sleep, 720 min) | n/a | OK | I-kids-03 | The baby lies on the mattress, asleep or awake (pilot); I-kids-03 S1 code (systems ledger: an adult told to lie in it) | `kids/sheets/obj-crib.png`, `systems/sheets/baby.png` |
| `feed` Feed the baby (tend, 15 min) | n/a | I-kids-04 | I-kids-04 | I-kids-04 S2 renderer-table (confirmed again: Ada, on her own, kneels at the rail for the feed); I-held-01 S2 code (no bottle) | `systems/sheets/baby.png` |
| `change` Change the baby (clean, 10 min) | n/a | I-kids-05 | I-kids-05 | I-kids-05 S2 code (confirmed again: changed over the rail, the changing chest beside it unused) | `systems/sheets/baby.png` |
| `play` Play with the baby (play, 20 min) | n/a | OK | OK | Standing at the rail, the hands working over the baby (pilot `play/crib-play-01.png`) | `kids/sheets/obj-crib.png` |
| `cuddle` Cuddle the baby (cuddle, 15 min) | n/a | OK | OK | Lifted out and held (pilot `scenes/crib-adult-cuddle-03.png`) | `kids/sheets/obj-crib.png` |
| A crying baby wakes a parent | n/a | OK | OK | By day with free will on, the nearest grown-up comes on their own, feeds and then changes (`review-baby`: Ada, 1 m off, at once; the crying bubble over the crib shows the need); at night the rules in `world.rs:2250` wake one asleep (tests `babies`; not shot at night) | `systems/sheets/baby.png` |
| The cuddle: the baby lifted into the arms | n/a | OK | OK | Up to the rail, the baby lifted, held, put back (`CUDDLE`, pilot) | `kids/sheets/obj-crib.png` |

## Lamps (4)

`crates/sim-core/src/lights.rs`; fades in the renderer.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clicking a lamp on or off | n/a | OK | OK | `switchLamp`, the lighting wave's shots | `lighting/night/extra/lamp-switched-off.png`, `lighting/night/extra/lamp-switched-on.png` |
| Residents switch a room's lamps on after dark and off as they leave | n/a | OK | OK | `lamps-night`, free will on: Ada walks into the dark room B and both its lamps come on as she comes in, room A's go off behind her; out of the house both are off; back in A, A's come on. Nobody walks to a switch (by design: "as at the switch by the door") | `systems/sheets/lamps.png` |
| Lamp fades | n/a | OK | n/a | 1.2 s (`PlayCanvasRenderer.ts:191` `LIGHTS_OUT_FADE`), the shade's glow with it | `systems/sheets/lamps.png` |
| Light coming from the shade | n/a | n/a | OK | The shade glows and the pool of light falls round the base; each lamp is judged in the lighting ledger | `lighting/night/extra/lamp-switched-on.png` |

## House doors (3)

`PlayCanvasRenderer.ts` `swingDoors` (the door models are in [houses.md](houses.md)).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Front door: who opens it and when | n/a | A-houses-01 | OK | Shut; swings in as she walks up to it, open while she passes, shut behind her (`bathroom-check`); a visitor knocking stands still, so it stays shut until the host walks out. A-houses-01 S2 code (systems ledger, confirmed: by nearness, no hand; raised from S3). Fixed (e04bc6c0): only for someone going through it | `systems/sheets/doors.png`, `fixes/town/doors/swing1-past-0.png` |
| Bathroom door: shut while in use | n/a | A-houses-01 | I-systems-05 | Opens as she walks in, shuts behind her and stays shut on the toilet and at the basin (from above, `systems/sheets/doors-top.png`). I-systems-05 S3 code (fixed 33a9ded1, 9987714b: the sim keeps a bathroom someone is in shut, others wait outside its shut door); A-houses-01 S2 code (fixed e04bc6c0) | `systems/sheets/doors-top.png`; after `fixes/privacy/1-bath-door-inside.png` |
| Garden door to the patio | n/a | A-houses-01 | OK | A door with outdoors on one side is a "front" door (`house.ts:293`, `:760`): shut, opening as someone passes, like the front door (code read; not shot apart). A-houses-01 S2 code: fixed (e04bc6c0), opens only for someone going through, as the front door | `systems/sheets/doors.png` |

## Objects that open (5)

Content `when` / `leaves` (crops below); the objects themselves are in their own ledgers.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Open state: Futon bed (`bedFuton`): sleep, nap, unfold, foldAway | n/a | A-bedroom-03 | I-bedroom-03 | Going to bed unfolds it, the last one up folds it away (tests `open_objects.rs`); nobody folds it over a sleeper. A-bedroom-03 S2 code, I-bedroom-03 S3 content (bedroom ledger: in at the foot; no sitting on it folded) | `bedroom/sheets/obj-bedFuton.png` |
| Open state: Cedar hot tub (`hotTubCedar`): soak, coverOn | n/a | OK | I-bathroom-03 | The cover off as they get in (stood up behind the tub), the last one out puts it back (`closeAfter`); I-bathroom-03 S2 code (residents ledger, widened there: soaking in clothes) | `outdoor/sheets/obj-hotTubCedar.png` |
| Open state: Sofa bed (`sofaBed`): relax, doze, pullOut, foldAway | n/a | OK | OK | Relax only folded; pulled out for a doze and put away after; nobody opens it under someone sitting (tests). The sofa bed's own shots: living ledger | `living/play/chunk2/sofaBed-pullOut-01.png`, `living/play/chunk2/sofaBed-doze-01.png` |
| Open state: Fold-down wall bed (`wallBed`): sleep, nap, foldDown, foldUp | n/a | OK | OK | Folded down from its foot, up by the last one up (tests); nobody folds it up over a sleeper | `living/sheets/obj-wallBed.png` |
| Open state: Washing line (`clothesline`): washing, takeIn | n/a | A-outdoor-02 | OK | Hung out, it stays out until someone takes it in (autonomous); A-outdoor-02 S3 motion (all the washing on the line in 1.5 s at the start of a 20-minute use; fixed in part, 7d186856) | `outdoor/sheets/obj-clothesline.png` |

**Not an object that opens** (a row of its own, judged when resizing comes back; A-systems-01 is closed, not reproduced):

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Resizing furniture with moving parts | n/a | A-systems-01 | n/a | A-systems-01 S3 code (closed, not reproduced: the moving parts follow a resize, 5866e4ab; `followup/A-systems-01/`); resizing is off in 0.3.0 for reasons of its own | n/a (no resize in 0.3.0) |

## Crops (4)

Content `grows`; `harvest` from ripe to picked.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Planting | n/a | n/a | I-systems-08 | I-systems-08 S3 decision (nobody plants: a picked bed plants itself half a day later) | `garden/sheets/obj-garden.vegPatch.png` |
| Watering | n/a | OK | I-systems-08 | "Weed and water" / "Tie up and water" (`tend`) changes nothing in the crop: I-systems-08; the clip and pose at each crop: I-garden-04 S2 renderer-table (residents ledger); no can: I-held-01 | `garden/play/c3/garden.tomatoes-tend-01.png` |
| Growing from planted to ripe | n/a | OK | OK | Planted, growing and ripe stages over 2–4 days (`grows.days`), shown by the model's growth clip; the strawberries' planted stage is M-garden-07 (garden ledger) | `garden/sheets/obj-garden.strawberries.png` |
| Harvesting and picking | n/a | I-garden-03 | I-garden-10 | I-garden-03 S2 code (systems ledger: the crop is gone as the harvest starts); I-garden-04 S2 renderer-table; I-garden-10 S3 decision (eaten on the spot) | `garden/extra/harvest/s-garden.vegPatch-harvest.png` |

## Visitors (5)

`base.json` `visits` (`door`: knock, wait 10 min, greet with `chat`; farewell).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Knocking (`knock`) | n/a | I-systems-09 | I-systems-09 | I-systems-09 S3 code (the visitor stands 1.5 m before the door, turned to the middle of the lot, and knocks on the wall beside it) | `systems/sheets/knock.png` |
| Answering the door, or turning away | n/a | OK | OK | The nearest free grown-up drops what they're doing (R2 leaves a meal in the kitchen), walks through the house and out of the door to the visitor (`life.rs:669` `door_host`); turned away at night or by someone who can't stand them (`answer`, code read; not shot) | `systems/sheets/knock.png` |
| Greeting at the door | n/a | OK | OK | A chat on the doorstep, 1 m apart, facing each other (`visits.door.greet`) | `systems/sheets/knock.png`, `social/sheets/visit.png` |
| What visitors use | n/a | n/a | I-systems-05 | I-systems-05 S3 code (fixed 33a9ded1, 9987714b: the household's own things are `private`, refused to visitors) | |
| Farewell (`goodbyeHug`, `goodbye`) | n/a | OK | OK | A hug for a friend, a wave for others, then home (`goodbye-hug`, `goodbye-wave`; social ledger rows) | `social/sheets/goodbye.png` |

## Use length (1)

Sim rules over every use.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| A use ends when every need it raises is full | n/a | n/a | I-systems-02 | I-systems-02 S3 decision (confirmed: every review state sets needs to 0.5 for this; with them full in `review-baby` care ended once the baby's needs were up) | `systems/sheets/baby.png` |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-systems-01 — A resized thing's moving parts don't follow it (closed: not reproduced)

- **Severity:** S3
- **Rubric:** A10, M4
- **Fix class:** code
- **Where:** `web/src/game/buildmode.ts` (resize handles)
- **Evidence:** Furniture resizing is off in 0.3.0. Something resized in an older save keeps its size, and its moving parts don't follow it; "Reset size" takes it back. Doors and windows still resize.
- **Status:** closed (not reproduced: the moving parts follow a resize; `review-resized`, 5866e4ab)
- **After:** five things with moving parts bought resized (`Command::Buy` with `size` and `tall`) on `review-resized` and played headless (`check.mjs play --use-existing fridge,toilet,filingCabinet,laundryWasher,wallBed`): the fridge two tiles wide and 30 cm taller swings its doors from its new edge, the toilet 25 cm taller lifts its lid on its hinge, the filing cabinet 40 cm taller slides its drawer out of its front, the washing machine two tiles wide opens its door on the stretched front, the wall bed 3 × 3 and 30 cm taller folds down out of its cabinet and up again (`bench-results/review/followup/A-systems-01/resized-*.png`, beside `base-*.png` at their own size on `review-resized-base`). Each part's pivot is moved by the resize and each clip's poses with it (`buildFromFile` `shifts`, `buildClips` in `models.ts`), since 94381476, before resizing was switched off (25519f03). What a resize still does to a moving part: the part stretches with the body (the wide fridge's one door is 2 m across), a slide goes its own distance (a deeper drawer comes out as far), and a part held in the hand (`partProp`) is drawn at the kind's size. Whether resizing comes back (`RESIZING` in `buildmode.ts`), and for which objects, is the owner's call: the moving parts don't stand in its way.
- **Source:** `CHANGELOG.md` 0.3.0 Changes ("Furniture can't be resized for now")

### I-systems-01 — Cooking props are drawn at a station's middle: on the gas grill station, on the grate

> Consolidation: the kitchen wave widened it to the hobs: the pan is drawn at the station's middle, not on the burner the clip lights (the gas range's flame at the front right and the pan at the front left, the hob counter's ring at the back and the pan at the front, the range cooker's ring in the middle and the pan to its left: `kitchen/play/c5/stoveGas-cook-01.png`, `cooktopCounter-cook-01.png`, `rangeCooker-cook-01.png`). Items: `stoveElectric`, `stoveGas`, `rangeCooker`, `cooktopCounter`, `grillStation`. A per-model hob spot fixes both.

- **Severity:** S2
- **Rubric:** I13
- **Fix class:** renderer-table
- **Where:** `web/src/render/pc/characters.ts:546` (`STATION_PROPS`) and `:2097` (`writeTool`)
- **Evidence:** The chopping board and the food are drawn at the station's middle, a little towards the cook. On `grillStation` that is the grate, so the chopping happens on the grill. Cross-cutting wave: `outdoor/play/chunk5-kitchen/grillStation-cook-01.png` and `grillStation-feast-01.png` (the outdoor wave's runs): the orange board lies over the burning grate while the station's own wooden prep board, at its right end, stays bare. The kettle grill (`bbq`) and kitchen counters are right (`systems/sheets/cook-bbq-1.png`, `cook-meal-1.png`).
- **Status:** fixed (65278fa4)
- **After:** `STATION_SPOTS` (`writeTool`): the pan on the burner each cooker's clip lights (the gas range's flame, the range cooker's middle ring, the hob counter's hot ring, measured from those parts; the electric cooker's front ring on the turning knob's side) and the grill station's chopping board on its free counter, off the grate (`fixes/tables/play-b/b-stoveGas-cook-*.png`, `b-rangeCooker-cook-*.png`, `b-cooktopCounter-cook-*.png`, `b-grillStation-cook-*.png`). Its own prep board keeps its tray of patties.
- **Source:** `docs/objects/open.md` Still open › Rendering; `plan/authenticity-review.md` §3.3 (I13); wave: cross-cutting

### I-systems-02 — Uses end once every need they raise is full

- **Severity:** S3
- **Rubric:** I4
- **Fix class:** decision
- **Where:** `crates/sim-core/src/world.rs`
- **Evidence:** By design. It makes uses (and their clips) short and easy to miss in checks: a reviewer should lower the needs first. Cross-cutting wave: every `review-*` state sets the needs to 0.5 for this reason; on `review-baby` (the grown-ups' needs full) the feed and change ended as soon as the baby's needs were up.
- **Status:** won't fix (by design, Q-22)
- **After:** Phase 4 (2026-10-21): kept as designed (owner decision Q-22, G-13); review states keep setting the needs to 0.5 so a use lasts. No code change.
- **Source:** `docs/objects/open.md` Still open › Content and sim

### I-systems-03 — With neither a sink nor a bin, dirty plates pile up on the floor

> Consolidation: the kitchen wave saw it on the bare stage: a snack's plate left on the floor with no top, sink or bin about (`kitchen/play/c1/rangeHood-no-uses.png`, bottom middle); with a counter the plate goes on the counter (`kitchen/extra/putaway-wallCabinet.json`).

- **Severity:** S3
- **Rubric:** I13
- **Fix class:** decision
- **Where:** `crates/sim-core/src/cooking.rs`
- **Evidence:** R2-12's rest: with a bin, clearing away ends at the bin; with neither, plates and spoiled food pile up and are scanned on every decision to eat.
- **Status:** won't fix (Q-17: every generated home has a sink; owner, 2026-10-21)
- **Source:** `docs/release-0.2.0-review.md` §4 (R2-12 rest), target v0.3

### I-systems-04 — Nothing stops a child using a stove

> Cross-cutting wave: confirmed and widened to the socials: any resident may start a fist fight with a child.
>
> Consolidation: widened by the object waves: every kitchen cooker (`stoveElectric`, `stoveGas`, `wallOven`, `rangeCooker`, `cooktopCounter`; kitchen ledger), the alcohol (I-kitchen-06), the upper bunk taken by an elder or a small child (A-bedroom-02), the playground (I-community-04).

- **Severity:** S2
- **Rubric:** I6
- **Fix class:** code
- **Where:** `crates/sim-core/src/content.rs` (interactions have no age rule); `crates/sim-core/src/conversation.rs:63` (`social_allowed`: the relationship, and for romance both grown up; nothing for a fight)
- **Evidence:** A known gap named in the rubric. Hard age rules are new behaviour: filed as `code`, planned separately (§9, decision 3). Cross-cutting wave: the one age rule that exists outside free will is the phone's (`phone.fromStage`: the child is refused, "too young for a phone", `social/scenes/scenes.log`). A teen starts a fight with a child (`social/sheets/fights.png`, A-social-03 for the punches), and an adult could the same (only the relationship is checked). The kids' chair takes any stage (`kids/sheets/age-kidsChair.png`), the residents ledger's seat fit row.
- **Status:** fixed (697544c2, 742572df, d2027317)
- **After:** Objects, recipes and interactions take `stages` (life-stage ids; `docs/content-packs.md`), kept by free will and the player's orders alike (`world::unfit`). Every recipe with a hot step (meal, gourmet, tasting menu, both barbecues, pizza, baked treats, pie) is teens and up wherever it is started: the child is refused at the gas range ("Dee is too young to cook dinner", "… bake treats"), the teen cooks (`fixes/age-rules/play-child.jsonl`, `teen-stoveGas-*.png`); the sandwich and reheating stay anyone's. The weight bench, dumbbells, cold plunge and sauna are teens and up; the standing desk too ("Dee is too young to browse the web", `play-child-play.jsonl`). Socials take `stages`/`targetStages`: *Start a fight* is teens and up on both sides, never offered or chosen with a child (`tests/age_rules.rs` `nobody_starts_a_fight_with_a_child`). The upper bunk (`upperStages`) is not an elder's ("Gran is too old for the top bunk"). A kids' chair is no seat for a grown-up. Paths under `bench-results/review/fixes/age-rules/`.
- **Source:** `plan/authenticity-review.md` §3.3 (I6) and §9 (decision 3); wave: cross-cutting

### I-systems-05 — No privacy: the bathroom door is renderer-only

> Cross-cutting wave: confirmed and widened. Every inside door but a bathroom's stands open all the time.

- **Severity:** S3
- **Rubric:** I6
- **Fix class:** code
- **Where:** `web/src/render/pc/PlayCanvasRenderer.ts:3778` (`swingDoors`), `web/src/render/pc/house.ts:757-760` (only front and bathroom doors move: "the others stand open"), `crates/sim-core/src/content.rs`
- **Evidence:** No private flag exists, so nothing keeps a visitor from using a private thing, and the bathroom door shuts only in the drawing. A privacy flag is new behaviour: filed as `code`, planned separately (§9, decision 3). Cross-cutting wave: `systems/sheets/doors-top.png` (`bathroom-check`, from above): the bathroom door swings open as she walks in, shuts behind her and stays shut through the toilet and the basin; nothing in the sim knows. A bedroom's door, or any other inside one, stands open day and night, a sleeper's too.
- **Status:** fixed (33a9ded1, 9987714b, a7681b17; G-72); was confirmed and widened
- **After:** a bathroom (content `roomKinds[].private`) is kept shut while someone is in it (`crates/sim-core/src/privacy.rs`), unless it is mixed with another kind, has a door to the outside or is on a public place. Someone on their way in stops 0.9 m short of its doorway and waits there, standing, and goes in once it's free; after an hour they give up. Free will doesn't choose what's in it unless a need is about to run out; kept out with the bladder empty, the accident comes after its 45 minutes, and the toilet behind the shut door counts as taken for "A second bathroom". The snapshot says who keeps a room shut (`sim::SHUTS`), and the renderer holds that room's doors shut, opening them only for the one going through, so §11.3 (a) is settled: the door stays shut to someone walking in. Visitors leave the household's own things alone (`private` on the wardrobes, dressers, the bath cabinet and the like: "Ada is a guest there, and the wardrobe is the household's own"). Tests: `tests/privacy.rs` (`someone_waits_outside_the_bathroom_in_use_and_goes_in_after`, `nobody_waits_at_a_shut_bathroom_for_ever`, `kept_out_with_an_empty_bladder_the_accident_comes`, `a_bathroom_opening_onto_the_garden_isnt_kept_shut`, `a_visitor_is_refused_the_households_own_things`). In play on `privacy-check` (headless): Ada in the shower, the door shut from inside, Ben waiting outside it, then in once she was out (`bench-results/review/fixes/privacy/1-bath-door-inside.png`, `1-bath-door.png`, `2-after-*.png`). Every other inside door but a bedroom's with a double bed still stands open.
- **Source:** `plan/authenticity-review.md` §3.3 (I6) and §9 (decision 3); wave: cross-cutting

### I-systems-06 — Every oven step is stirring a pan: pizzas, pies and cakes are "baked" on the hob

> Consolidation (2026-10-21): closed as a duplicate of I-kitchen-05 (systems ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I13, I3
- **Fix class:** renderer-table
- **Where:** `web/public/content/base.json:4396`, `:4440`, `:4483` (`pizza`, `bakedTreats`, `pie`: the oven step's `anim: stir`) and `:4176` (`meal`'s cook step at `stove`, `oven` or `microwave`, `stir`); `web/src/render/pc/characters.ts:540` (`ACTIONS.stir`: `stir`, `interact`), `:546` (`STATION_PROPS.stir`: the pan)
- **Evidence:** `systems/sheets/cook-ovens.png` (the kitchen wave's `kitchen/play/c5/stoveGas-bake-01.png`, `stoveElectric-bake-01.png`, `wallOven-bake-01.png`): baking at a stove the oven door drops open and stays open while the cook stirs a pan on the lit hob for 35 minutes, the prep bowl of batter in the other hand; at the oven tower, with no hob, she stirs at the open oven door. `systems/sheets/cook-pizza-1.png` (`held/inhand/pizza-04.png`): the pizza "baked" stirring a pan at the wood-fired oven's side; `cook-pie-1.png` the same at the neighbourly oven. Baking is putting the tray in, shutting the door and waiting (a check through the glass), then taking it out: an oven step of its own (`anim: bake` with a tray prop, the door open at the start and end).
- **Status:** closed (duplicate of I-kitchen-05)
- **Source:** wave: cross-cutting (held, kitchen and packs waves' runs)

### I-systems-07 — The prep bowl stays in the hand through the cooking step

> Consolidation (2026-10-21): closed as a duplicate of I-kitchen-04 (systems ledger), which takes its items and evidence.

- **Severity:** S3
- **Rubric:** I13, I7
- **Fix class:** code
- **Where:** `crates/sim-core/src/cooking.rs:1501` (`step_done`: `held` is set to what the step `holds` and kept until the next step ends), the renderer drawing `held` in the hand whatever the step (`web/src/render/pc/characters.ts`, `writeHeld`)
- **Evidence:** `systems/sheets/cook-meal-1.png`, `cook-bbq-1.png`, `cook-bakedTreats-1.png` (`held/inhand/meal-03.png`, `meal-04.png`, `bbq-05.png`, `bakedTreats-04.png`, `pie-04.png`, `tastingMenu-04.png`; the held wave's "Seen"): the cook stirs the pan or turns the food on the grill one-handed for 20–50 minutes with the bowl of chopped food held in the other. The bowl is tipped into the pan and put down as the step starts.
- **Status:** closed (duplicate of I-kitchen-04)
- **Source:** wave: cross-cutting (held wave's runs)

### I-systems-08 — Nobody sows: a picked bed plants itself, and tending changes nothing

- **Severity:** S3
- **Rubric:** I15, I8
- **Fix class:** decision
- **Where:** `crates/sim-core/src/world.rs:2765-2786` (`grow`: growth by time alone, the fallow ground "planted afresh" at 0), `crates/sim-core/src/content.rs:2064-2074` (`grows.fallow`, half a day by default); `web/public/content/garden.json` (the crops' `tend`: fun, gardening skill, no growth)
- **Evidence:** A harvested crop lies fallow for half a game day and then is planted again with nobody there; it ripens in its 2–4 days whether anyone weeds and waters it or not ("Weed and water", "Tie up and water" give fun and the gardening skill only). In life a bed is sown or planted out by hand, and a crop not watered does worse. For the owner: keep it (a game shortcut), or a "Sow" use on fallow ground and `tend` speeding growth, which is new behaviour (`code`).
- **Status:** fixed (9bb1c2ff): tending helps; replanting by itself stays (Q-18)
- **After:** Interactions take `tends`: a finished "Weed and water" / "Tie up and water" (the veg patch, tomatoes, strawberries, pumpkins, raised bed, the nature lover's bed: 0.15; watering the lemon tree: 0.1) brings a growing crop that share of its way on at once, so a crop tended once a day ripens hours sooner; ripe or fallow, tending does nothing for it (`tests/harvests.rs`). A picked bed still replants itself after lying fallow (the shortcut Q-18 keeps): no "Sow" use.
- **Source:** wave: cross-cutting (code read, with the garden wave's crop shots)

### I-systems-09 — Visitors knock facing the middle of the lot, 1.5 m short of the door

- **Severity:** S3
- **Rubric:** I15, A1
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:4917-4922` (`TaskKind::Visit` arriving: the yaw towards the plot's centre, at the plot's `entry`); `web/src/render/pc/characters.ts:535` (`ACTIONS.knock`: `interact`, `idle`)
- **Evidence:** `systems/sheets/knock.png` (`calling-out`, R10 invited over; `systems/scenes/scenes.log`): the visitor stops at the lot's entry (25.5, 27.5), 1.5 m in front of the front door (25.5, 26), turned 0.49 rad off it towards the middle of the lot, and knocks with the reach clip on the wall beside the doorway. A knock is at the door, an arm's length from it, facing it. The host coming out and the greeting are right.
- **Status:** fixed (36198175)
- **After:** the visitor walks to the front door (`life::knock_spot`: the door from outdoors nearest the plot's entry) and knocks 0.45 m out from it, facing it. `bench-results/review/fixes/positions/play/knock-00.png` (`calling-out`, R10 at 25.5, 26.45, yaw π, knocking) against `systems/sheets/knock.png`; sim test `tests/positions.rs` `a_visitor_knocks_at_the_front_door_facing_it`.
- **Source:** wave: cross-cutting

### I-systems-10 — A cook waiting for a busy station stands where they are, and the next one stands inside them

- **Severity:** S2
- **Rubric:** I13, A1
- **Fix class:** code
- **Where:** `crates/sim-core/src/cooking.rs:760-763` (`Found::Busy`: `Phase::Queued` where the cook stands, the tile of the station they just used)
- **Evidence:** `systems/sheets/baby.png` (`review-baby`, `systems/scenes/baby-09.png` … `baby-13.png`, positions in `systems/scenes/scenes.log`): Cal got his ingredients and, the counter busy (Ben chopping), waits at the fridge's spot (14.5, 8.5); Gus comes to the free fridge and stands on the same spot: the two bodies one inside the other for a minute and more, until the counter frees. A waiting cook steps aside (a free tile next to the next station).
- **Status:** fixed (36198175)
- **After:** a cook (or washer-up) waiting for a busy station first steps aside to a free tile beside them that no station is used from and nobody stands on (`cooking::queue`, `aside`), then waits; stations several use at once put each cook at their own spot. `bench-results/review/fixes/positions/play/fridge-cooking-011.png`: Gus waits at (13.5, 8.5), a tile off the fridge's spot (14.5, 8.5), while Ada preps at the counter; against `systems/sheets/baby.png`. Sim test `cooking.rs` `a_cook_waiting_for_a_busy_counter_steps_aside_off_the_fridge`.
- **Source:** wave: cross-cutting

### I-systems-11 — Young children make and drink coffee

- **Severity:** S3
- **Rubric:** I6
- **Fix class:** content
- **Where:** `web/public/content/furniture.json` (`coffeeMachine.espresso` "Make a coffee"), `packs/foodie.json` (`foodie.espressoBar` espresso, latte art), every *Have a coffee* at a chair or table (`requires: ["coffee"]`, about 15 objects in `base.json`, `furniture.json`, `community.json`, `catalog2/dining.json`, `catalog2/outdoor.json`)
- **Evidence:** I-kitchen-06's lesser case, left out of G-45 on purpose: with the age rules in place (`stages`), nothing yet keeps a child of 8 from a 10-minute espresso ("perked up") or a coffee at the table, on free will too. A teen with a coffee is ordinary, a young child is not. Fix: `"stages": ["teen", "youngAdult", "adult", "elder"]` on these interactions (the machine's and every *Have a coffee*), many lines across files other groups edit; or decide coffee stays anyone's (a game simplification).
- **Status:** fixed (9463d1a7)
- **After:** Every coffee use is for teens and up (`"stages": ["teen", "youngAdult", "adult", "elder"]`): the coffee machine's *Make a coffee*, the foodie espresso bar's *Pull an espresso* and *Practise latte art*, all 17 *Have a coffee* at tables and chairs (base, furniture, dining, outdoor, community), and the new *Make a coffee* at the small espresso machine on a top (G-63). A child's order is refused ("Dee is too young to make a coffee") and free will never picks one for them (`world::unfit`); `tests/tops_and_conditions.rs` `coffee_is_for_teens_and_up` checks every shipped interaction labelled coffee, espresso or latte. Tea (the kettle, the tea set) and hot drinks stay anyone's. Not shot in play (a content rule on the G-45 path, shot there).
- **Source:** fix: age-rules

### A-systems-02 — The washing appears on the line in 1.5 s at the start of a 20-minute hang-out

> Consolidation (2026-10-21): closed as a duplicate of A-outdoor-02 (systems ledger), which takes its items and evidence.

- **Severity:** S3
- **Rubric:** A10, A9
- **Fix class:** motion
- **Where:** `web/public/assets/packs/yard/manifest.json:361` (`clothesline` `motions.washing`, 1.5 s, played as the use starts; `takeIn` plays it back)
- **Evidence:** `outdoor/play/chunk3/play.jsonl` (the outdoor wave's run): the `washing` clip runs from 0.18 to its full 1.5 s by the first shot, so every piece is pegged up before the resident has lifted one, and the basket stays full beside them (`outdoor/sheets/obj-clothesline.png`); taking it in, the line is bare in the first second. Hanging washing out goes piece by piece over the use (a `hold` spread over it, or the pieces in turn).
- **Status:** closed (duplicate of A-outdoor-02)
- **Source:** wave: cross-cutting (the outdoor wave's run)

Moved here at the consolidation (2026-10-21) from the ledgers they were first seen in, ids unchanged: the sim rules (who may use what, where users stand and sit), cooking, crops, sounds, doors, lamps and placement. Pilot evidence in them stays relative to `bench-results/review/pilot/`, the waves' to `bench-results/review/`, as in the ledgers they came from.

### I-kids-03 — A direct use skips the rules free will keeps: an adult lies in the crib, a child cares for the baby, an adult does homework

> Pilot: confirmed and widened to its cause. Severity raised from S2 to S1 (an adult in pyjamas lying in a cot, legs through the rails, from the pie menu); fix class from `content` to `code`.
>
> Office/kids wave: confirmed and widened to the kids' furniture and the reverse (the owner's S1 rule: an adult using a child's or a baby's thing, or a child an adult's, as if it fitted). For these there is no rule at all, for free will either: free will checks only `baby`, care and `homework` (`world.rs:3838-3865`), so an adult on free will may ride the rocking horse too (the half I-systems-04 names for the stove).
>
> Consolidation: widened by the object waves: an adult does homework at `bookcaseWide` (`living/play/chunk2/bookcaseWide-homework-01.png`); the child's wine and cocktail (I-kitchen-06) were direct uses; the playground rides (I-community-04).
>
> Moved from kids.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S1
- **Rubric:** I1, I6
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:2981` (`Command::Use`: checks broken, `requires`, `when`/open, `whole`, `needsTable`; nothing about the user); against `world.rs:3838-3865` (free will: a `baby` use only for a baby, care only by an adult with `Access::Full` and a baby in the crib, homework only at a school stage); `web/src/game/session.ts:1160` (the pie menu offers every interaction to whoever is selected); `crates/sim-core/src/content.rs` (objects and interactions have no life-stage rule: the kids' furniture can't say it is for children)
- **Evidence:** `play/crib-lie-01.png`: the adult resident 0 lies in the crib in pyjamas, the legs out over the end rail (`sheets/crib-care.png` shows the cot at its real size beside the carers); the elder and the teen take it too (`play-elder/crib-lie-*.png`, `play-teen/crib-lie-*.png`). A newly bought, empty crib's feed, change, play and cuddle all run (`play/crib-feed-*.png` …), as Phase 0 saw. On `review-stage`'s own crib, with its baby, the child (8) and the teen (15) each feed, cuddle, change and play with the baby on a direct use (`scenes/care-child-*.png`, `scenes/care-teen-*.png`, `scenes/scenes.log`), which free will never lets them; an adult (`play/bookshelf-homework-*.png`, `play/computerDesk-homework-*.png`) does homework, which free will offers only to pupils. One cause: the command doesn't apply the user tests free will applies. Related: I-systems-04 (no age rule exists at all for a stove); this one is rules that exist but only for free will. (Pilot shots: relative to `bench-results/review/pilot/`.)
- **Evidence (office/kids wave;** paths under `bench-results/review/`**):**
  - The baby's crib: the 8-year-old lies in it, the mobile through her head and her feet through the rail (`kids/play/child-c1/crib-lie-01.png`); the child also feeds and changes at an empty crib (`kids/play/child-c1/crib-change-01.png`).
  - The race car bed (mattress 0.84 × 1.44 m, a junior bed's, `kids.py:417`): an adult lies with her feet on the bonnet (`kids/play/c1/kidsBedCar-sleep-01.png`), and an elder's blanket drapes over the whole bonnet (`kids/poses/poses-only-elder-0-car_bed.png`). The kids' bed (0.88 × 1.82 m, a single bed's size) fits an adult and isn't counted.
  - The rocking horse (seat 0.56 m, 0.29 m wide): adults, the teen and the elder ride it astride with their feet flat on the floor, the horse lost under them (`kids/play/c1/rockingHorse-ride-01.png`, `kids/poses/poses-seats-default-0-rockingHorse.png`).
  - The kids' chair (seat 0.31 m): an adult or an elder sits with the knees up and the feet sunk through the floor (`kids/poses/poses-seats-default-3-kidsChair.png`, `-elder-3-`); at the kids' table an adult draws and does homework, the knees under a 0.50 m top (`kids/play/pair-kidsChair/kidsChair-homework-01.png`).
  - The kids' desk (top 0.57, seat 0.36): an adult does homework and draws on its child-size chair (`kids/play/c1/kidsDesk-homework-01.png`); an adult desk chair also works at it, the top 9 cm over the seat (`kids/extra/side-chairTask-11-12-3-write-01.png`; `chairTask` `seatAt: ["desk"]` takes the kids' desk's `desk` kind, `furniture.json:2834`).
  - The kids' bean bag (0.62 m), the ball pit (0.93 m across, 0.47 high) and the play tent (door 0.56 m high): adults flop into the bag and "jump in the ball pit" and "play in the tent" (`kids/play/c1/beanBagKids-flop-01.png`, `kids/play/c1/ballPit-jump-01.png`, `kids/play/c1/playTent-play-01.png`; the last two are also used from outside, I-kids-07).
  - The reverse: the 8-year-old types at the standing desk's 1.05 m top, the keyboard shelf at her chin (`office/extra/child-deskStanding-browse-01.png`).
  - Homework by adults on every homework use in these two ledgers: `computerDesk`, `deskWriting`, `deskLaptop`, `chairTask`, `chairExecutive`, `kidsDesk`, `kidsChair` (`office/play/c1/deskWriting-homework-01.png`, `office/play/pair-chairTask/chairTask-homework-01.png`).
  - Toys (toy box, toy chest, doll house, train table, play mat, teddy, easels, picture books) are not counted: grown-ups play with them in life too.
- **Status:** fixed (697544c2, 742572df)
- **After:** A direct order keeps the rules free will keeps, and the new life-stage rules (`stages`), with the game's refusal toast; a refused order is never queued, and a use their age isn't for never starts however it was chosen (`start_task`). On `review-stage` (`fixes/age-rules/play-adult.jsonl`, `play-child.jsonl`, `play-teen.jsonl`): the adult is refused the crib's *Lie* ("Ada is too big for the crib") and does feed, change, play and cuddle (`adult-crib-feed-*.png`); the child and the teen are refused the care ("Dee is too young to look after a baby", "Cal is too young …") and the crib. An empty crib's care is refused ("there's no baby in the crib"); homework by a grown-up is refused ("Ada has no homework to do"; `tests/age_rules.rs`). The kids' furniture is a child's: the adult is refused the race car bed, rocking horse, kids' bean bag, ball pit, play tent, toy box and kids' desk ("Ada is too big for the rocking horse"), the child uses them (`child-kidsBedCar-*.png`, `child-rockingHorse-*.png`, `child-ballPit-*.png`, `child-kidsChair-sit-*.png`); the kids' chair is no seat for a grown-up to eat or phone on. The reverse: the child is refused the standing desk. The pie menu greys out what the selected resident's age isn't for (`Content.mayUse`; type-checked, not shot). Toys grown-ups use too (doll house, train table, play mat, teddy, easels, picture books) and the kids' bed stay anyone's. Paths under `bench-results/review/fixes/age-rules/`.
- **Source:** `check.mjs play --state review-stage --objects crib --use all` (Phase 0 and pilot, adult, teen, elder); pilot `scripts/scenes.mjs` childcare; wave: office/kids (Phase 0.9 `play/c1`, `play/child-c1`, poses; `check.mjs play --objects deskStanding --sim 3`)

### I-kitchen-06 — Children pour wine and mix cocktails

> Moved from kitchen.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S1
- **Rubric:** I6
- **Fix class:** code
- **Where:** `web/public/content/catalog2/kitchen.json:799` (`wineFridge` "Pour a glass of wine": tags `food, lounge`, autonomous), `web/public/content/catalog2/dining.json:845` (`barCart` "Mix a drink": `food, lounge, party`); `crates/sim-core/src/world.rs:3832-3856` (free will's tests on a use: baby, table, requirements, homework for pupils, care for adults; nothing on the user's stage otherwise), `:2981` (`Command::Use`, I-kids-03)
- **Evidence:** `kitchen/extra/play-child/child-wineFridge-wine-01.png`: Dee (8) "Pour a glass of wine", sipping at the open wine fridge; `child-barCart-mix-01.png`: the same child mixing a drink at the bar cart, the shaker tipping (`kitchen/extra/play-child/play-child.jsonl`: both `using`). These were direct uses, but free will has no stage rule for either, so a child picks them for comfort and fun on their own; a teen too. A lesser case of the same, by the same missing rule (not run): nothing keeps a child of 8 from a 10-minute espresso at the coffee machine (`furniture.json:464`, feeling "perked up"). Age rules are new behaviour (§9 decision 3): filed as `code`. Related: I-systems-04 (systems ledger: a child at a stove) and I-kids-03 (systems ledger: a direct use skips the rules free will keeps).
- **Status:** fixed (697544c2, 742572df)
- **After:** The wine fridge, the bar cart and the party bar's *Mix drinks for everyone* are for young adults and up (`stages`). On `review-stage` the child is refused both ("Dee is too young to pour a glass of wine", "… to mix a drink"; `fixes/age-rules/play-child.jsonl`), the teen the wine (`play-teen.jsonl`), and the adult pours and mixes (`adult-wineFridge-wine-*.png`, `adult-barCart-mix-*.png`); free will never picks them for a child or a teen (`tests/age_rules.rs` `free_will_never_picks_what_their_age_is_not_for`). The party bar's soda stays anyone's. Coffee is left to anyone: filed as I-systems-11. Paths under `bench-results/review/fixes/age-rules/`.
- **Source:** wave: kitchen (`check.mjs play --sim 3`)

### I-community-04 — Adults and elders ride the spring horse, the seesaw and the merry-go-round and climb the fort

> Moved from community.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S1
- **Rubric:** I6
- **Fix class:** code
- **Where:** no age rule on a use (`crates/sim-core/src/content.rs`, see I-systems-04); free will only weights `toys` down for teens to elders (`web/public/content/base.json:3662-3668`, `tagPreference` toys 0.4 / 0.15, multiplied in `content.rs:727`); the climbing frame isn't tagged `toys` (`web/public/content/community.json:415-419`: entertainment, fitness, outdoors)
- **Evidence:** `sheets/obj-springRider.png` (`play/springRider-ride-01.png`): the adult sits on a 1.1 m horse meant for toddlers, knees up round its head; `sheets/obj-seesaw.png`, `sheets/obj-merryGoRound.png`, `sheets/obj-slide.png`, `sheets/obj-climbingFrame.png` (`play/climbingFrame-climb-01.png`): the adult on each, ducking under the fort's roof; `sheets/age-springRider.png`, `sheets/age-seesaw.png`: an elder on them too. Free will offers the spring rider, the seesaw, the slide and the merry-go-round to adults at 15 % weight, and the climbing frame at full weight, since it carries no `toys` tag; a direct use offers them all (I-kids-03, kids ledger). An adult on a toddler's spring horse is the rubric's S1 ("an adult using a child's thing as if it fitted"). Two parts: the `toys` tag on the climbing frame (content, now), and an age rule for the toddler rides (new behaviour, `code`, §9 decision 3). Widens I-systems-04 (systems ledger) from the stove to the playground.
- **Status:** fixed (697544c2, 742572df)
- **After:** The spring rider, slide and climbing frame are a child's, the seesaw and merry-go-round a child's or a teen's (`stages`), and the climbing frame is tagged `toys`. On `town-things` the adult is refused all five ("Ada is too big for the spring rider", "… the playground slide", "… the climbing frame"; `fixes/age-rules/play-town-adult.jsonl`); on `review-stage` the child rides each (`child-play-springRider-ride-*.png`, `child-play-seesaw-*.png`, `child-play-slide-*.png`, `child-play-climbingFrame-*.png`, `child-play-merryGoRound-*.png`) and the teen takes the seesaw but is refused the spring rider (`play-teen.jsonl`). There is no toddler stage (a child is 2–12, drawn at one size), so the toddler rides are the child stage's. Paths under `bench-results/review/fixes/age-rules/`.
- **Source:** community wave (play, Phase 0)

### I-hobbies-07 — Two residents at a two-player table stand in one spot, inside each other

> Moved from hobbies.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S1
- **Rubric:** I2, A7
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:4959-4961` (a standing use keeps the resident on the tile they came to and only turns them); `world.rs:322-333` (`slot_position` spreads the slots, but only sitting and lying uses move there); `web/public/content/furniture.json:3327`, `:3377` (`poolTable`, `foosball`: `slots: 2`)
- **Evidence:** `hobbies/extra/two.mjs` sent the adult (0) and the young adult (1) to the same table: both were `using` at (9.5, 13.5), yaw 2.68, at the foosball, and at (10.5, 13.5), yaw 3.14, at the pool table (`hobbies/extra/two.jsonl`): one body inside the other, the second's sleeves showing through the first (`hobbies/extra/two.png`, front and top). At a real pool table the two move round it; at foosball they face each other across it. Wrong at a glance whenever two people play. The renderer's `STEP_IN` lanes (`characters.ts:193`) split users for the climbing frame only. The same cause reaches every other object with two or more slots and a standing use (a content scan, not checked in play): `crib`, `counterSinkDouble`, `vanityDouble`, `kitchenIsland`, `homeCinema`, `trainTable`, `pingPongTable`, `basketballHoop`, `marketStall`, `shopCounter`, `bakeryCase`, `groceryShelf`, `fountainTown`, `busStop`, `energetic.streetballHoop`, `cheerful.jukebox`, `outgoing.partyBar`, `outgoing.karaokeStage`, `outgoing.danceFloor`, `foodie.pizzaOven`, `foodie.chefsIsland`, `natureLover.glasshouse`, `romantic.roseArbourSwing`, `hotTubCedar` (its cover), and the fold uses of `sofaBed`, `wallBed`, `bedFuton`.
- **Status:** fixed (36198175, 0d20c5c9, d7b7096d, ad1c277d)
- **After:** each standing user of a thing several use at once stands at their own spot (`surface::stand_spot`): at their slot's place off the footprint (`places`: the pool players on the table's two long sides, diagonally across it, the foosball players one each side facing each other), else in front of their own slot along its front, facing straight in, walked to that point; the renderer's `STEP_IN` lane is the free one nearest them, per side for pool and foosball (`sides`). `bench-results/review/fixes/positions/play/poolTable-01.png` (10.05, 19.5 and 8.95, 22.5), `foosball-01.png`, against `hobbies/extra/two.png`, `two-foosball-v1.png`. The content scan is a sim test now (`tests/positions.rs` `shipped_things_used_standing_by_several_keep_them_apart`: 20 shipped things, nobody within 0.45 m of another); in play also the double sink and vanity (`counterSinkDouble-00.png`, `vanityDouble-00.png`: one at each basin), the shop counter, the bus stop's timetable, the hoop, the crib's carer (in front of the baby, `crib-01.png`). Places added: pool table, foosball, streetball hoop, jukebox (the basketball hoop's two step back to G-04's lanes from their own half). Still: the foosball's far player has no handles (M-hobbies-07); a partner is G-65.
- **Source:** wave: hobbies (`two.mjs`, adult and young adult on `review-stage`)

### I-community-08 — Standing at a 2-wide thing, residents stand in front of its left half

> Moved from community.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** I2
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:357-365` (`front_tile`: `x + (w - 1) / 2`, so the left of the two tiles in front of an even width)
- **Evidence:** `play/play.jsonl`: for each 2-wide thing at x 9–11 (middle 10.0) the user stands at x 9.5: the ice cream cart, the hot dog stand, the market stall, the shop counter, the bakery case, the grocery shelves, the ping-pong table, and the 2 × 2 statue. `sheets/obj-hotDogStand.png` (`play/stalls-town/hotDogStand-hotDog-01.png`): the customer stands at the cart's left end, by the sauce bottles, not at its front; `sheets/obj-townStatue.png`: the admirer at the plinth's left corner. Across the content 22 one-slot things of even width are used standing (a scan of the content: also `wardrobe`, `dresser`, `dresserLow`, `armoire`, `bookcaseWide`, `workbench`, `grillStation`, `rangeCooker`, `raisedBed`, `sandbox`, `garden.pond`, `bikeRack`, `canvasAbstract`, `tvWall`, `tvStandLow`, `shedSmall`, `greenhouse`), where it matters less beside a wardrobe's left door. Standing users of an even-width thing want its middle (the renderer could centre them, as it spreads seated slots).
- **Status:** fixed (36198175)
- **After:** the one user of a thing an even number of tiles wide stands in front of its middle (`stand_spot`), facing straight in, in every facing (`tests/positions.rs` `the_one_user_of_a_thing_two_tiles_wide_stands_at_its_middle`); two at a 2-wide stall or counter one in front of each half (`bench-results/review/fixes/positions/play/shopCounter-04.png`). Not shot alone at the hot dog cart or the statue.
- **Source:** community wave (play records)

### I-outdoor-07 — Sets for two seat their pair side by side: the bistro set takes one, the picnic table two on one bench

> Consolidation: raised from S3 to S2, as I-community-02 (community ledger, S2) is merged into it. The same cause reaches the seesaw's riders (side-on, not facing each other), the merry-go-round's four (in a row along one chord), the park chess players (shoulder to shoulder behind the board), the ping-pong player (at the long side by the net), and by the content `diningOak`, `diningGlass` and `romantic.candlelitTable` (built side by side: M-kitchen-01, M-packs-11). The renderer half: `web/src/render/pc/characters.ts:240-249` (`Seat`: one facing per object, not per slot).
>
> Moved from outdoor.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2 (raised from S3 at the consolidation: I-community-02 merged)
- **Rubric:** I2
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:321-333` (`slot_position`: slots spread across the width, all facing the object's front); `web/public/content/furniture.json:4504` (`patioSet`: no `slots`), `:4574` (`picnicTable` `slots: 2`)
- **Evidence:** The bistro set has two chairs facing each other and one slot: a second resident can't join a coffee (`sheets/obj-patioSet.png`, the near chair stays empty). The picnic table's two ("picnics for two", its description) sit on the back bench 1.5 m apart, both facing the far bench (`extra/picnic-top/top-picnicTable-picnic-01.png` for slot 0's place). A couple at a table sit across from each other. Slots with their own place and facing (or the second slot turned round) are new behaviour. The base dining table has the same side-by-side seating (M-kitchen-01, kitchen ledger).
- **Status:** fixed (aac64b84, b4dfcfc2)
- **After:** a slot can have its own place and facing (content `places`, one a slot: metres off the footprint's middle and degrees from its front; `slot_position`, `slot_yaw`); a resident takes the free place nearest them and comes to it from the tile behind it (`surface::slot_approach`), or stands at it when it's off the footprint; the renderer follows the sim's facing and `slotPlace` (`objectMotion.ts`, `poses.ts`). Given real places: the seesaw's two ride at the ends facing each other, the merry-go-round's four sit round it facing in, the park chess players face each other across the board (the table rebuilt with a stool either side), the ping-pong players stand at the two ends, the picnic table's two sit one on each bench across the cloth, the bistro set (`patioSet`) now takes two, one a chair, facing each other, the tea table's two turn 45° to each other. `bench-results/review/fixes/slots/poses/poses-default-*.png` (one sitter a slot, real footprints), `bench-results/review/fixes/slots/play/<id>-0*.png` (two to four residents in play: `picnicTable`, `seesaw`, `merryGoRound`, `parkChessTable`, `parkChessTable-side`, `pingPongTable`, `patioSet`, `kind.welcomeTeaTable`), against `outdoor/sheets/obj-patioSet.png`, `outdoor/extra/picnic-top/top-picnicTable-picnic-01.png`, `community/extra/poses/rides-child.png`, `community/extra/poses/chess-adult.png`, `sheets/obj-pingPongTable.png`. Sim tests `tests/slots.rs`. Still to do: the base dining table, the glass and farmhouse sets and the candlelit table keep their places as built, side by side (`diningOak-0*.png`), until G-39 rebuilds them (below); astride on the seesaw is G-23 (the shins go down through the plank); a partner is G-65; the chess players' knees meet on the one-tile table (I-community-14).
  G-39 must: build `diningTable`, `diningGlass`, `diningOak` and `romantic.candlelitTable` (2 × 1) with a chair at each end facing in across a top about 0.8–0.9 m long, and change their `places` to about `[-0.62, 0.1]` facing 90 and `[0.62, 0.1]` facing -90 (measured from the new chairs; the furniture.json `seats` `fwd` stays 0) and their `surface` `size`/`at` to the new top; and give the tea table its poufs either side of a lower, narrower table, so its places can face straight across (now ±45°).
- **Source:** outdoor wave

### A-hobbies-01 — Residents walk through the table, the wheel or the drum kit to a seat behind it

> Moved from hobbies.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** A4, I2
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:4962-4968` (a sitting use: the resident moves from the tile in front, where they stop, to `slot_position` in the footprint); seats behind the middle with `approach` front: `web/src/render/shared/characters/furniture.json:7`, `:44`, `:45` (`chessTable`, the digital piano, `sewingTable`: fwd −0.2, −0.2, −0.22), `web/public/content/catalog2/hobbies.json:5`, `:93`, `:130`, `:166` (`drumKit` −0.23, `potteryWheel` −0.28, `craftTable` −0.25, `puzzleTable` −0.25), `web/public/content/furniture.json:3523` (`exerciseBike` −0.12)
- **Evidence:** the first play shot of each catches the resident mid-way through the object: `hobbies/play/c1/potteryWheel-throw-00.png` (standing on the wheel head), `drumKit-drum-00.png` (in the toms), `sewingTable-sew-00.png`, `craftTable-scrapbook-00.png`, `chessTable-chess-00.png`, `puzzleTable-jigsaw-00.png` and the digital piano's `play-00` (composites `hobbies/extra/sit00.png`, `sit00b.png`); a second in, from the side, the potter still stands in the wheel's body (`hobbies/extra/side1.png`), and the rider climbs on through the bike's handlebars and flywheel (`fitness/extra/side-exerciseBike-cycle-00.png`). They come to the front tile, then glide about a metre back through the work to the seat while sitting down, and out the same way (A-residents-01, residents ledger, is the stand-up glide on its own). Real people go round to the chair, or straddle the saddle from the side. Two ways out: walk them round to the seat's side (code, for every object like this), or build these hobby objects with the seat at the front, facing back, as the piano is (`seat.facing: back`, model and content). The same layout, seat behind the middle with approach front, is in other ledgers' objects (a scan of the content, not checked in play): `computerDesk`, `diningTable` (base), `deskLaptop`, `deskWriting`, `computerGaming`, `kidsDesk`, `bistroSet`, `diningGlass`, `diningOak`, `patioSet`, `picnicTable`, `parkChessTable`, `busStop`, `chaise`.
- **Status:** fixed (36198175, 0d20c5c9, d7b7096d)
- **After:** a seat behind the work is come to from behind it, else beside it, only with both blocked from the front (content `approach: behind`, `surface::seat_approach`): the chess, sewing, craft and jigsaw tables, the desks (computer, writing, laptop, gaming, kids'), the digital piano, the drum kit, the pottery wheel, the bistro set; the bikes and the chaise from their side (`approach: side`); a seat for several (the bus stop) from the tile in front of each sitter's own slot. `bench-results/review/fixes/positions/play/computerDesk-in-200.png` (from behind the chair), `computerDesk-wall-in-200.png` (its back to a wall: from beside it, along behind the desk), `exerciseBike-in-200.png`, `busStop-wait-00.png`, against `hobbies/play/c1/*-00.png`, `fitness/extra/side-exerciseBike-cycle-00.png`. Sim tests `tests/positions.rs`. Still a slide onto the seat (A-residents-13); the dining sets' places are G-39's.
- **Source:** wave: hobbies

### I-living-05 — Watching TV takes the nearest seat even when it faces away from the screen

> Moved from living.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I2
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:4463` (`seat_near_object`: the nearest free seat within `seatNear` metres, by path distance from the TV's front tile; no facing or sight check), called at `:4299`
- **Evidence:** `check.mjs play --objects homeCinema,tvWall@9,17,2 --with sofa@9,15,2` (`living/extra/tv-seat/`): the sofa faces the home cinema (rot 2, towards −z) and its back is to the wall TV on the back wall. For the home cinema the watcher sits on it facing the screen, right; for the wall TV she takes the same sofa and watches the hour with her back to the screen (`living/extra/tv-seat/seat-tvWall-watch-01.png`, pose `sit` at 10.5, 15.5). Any sofa or chair within 4 m counts, whichever way it faces (a dining chair at the table behind, an armchair turned away). Only the five living screens use `seatNear` (`tv`, `tvWall`, `tvRetro`, `homeCinema`, `tvStandLow`); a fix (a seat whose front looks towards the screen, else the standing fallback of I-living-03) would carry over to reading if I-living-03 gives the books `seatNear`.
- **Status:** fixed (aac64b84, 1224f373)
- **After:** a `seatNear` use with `seatFacing` takes only a seat whose place looks at the thing (within 60°) and, for a screen (`front`), one in front of it (within 60° of the way it faces); the six screen uses have `seatFacing: front` (`tv`, `tvWall`, `tvRetro`, `tvStandLow`, both of `homeCinema`'s). With the wall TV at (9, 17) facing −z, a sofa nearer it with its back to it and one further off facing it, the watcher takes the one facing it (`bench-results/review/fixes/slots/play/tvWall-seat-0*.png`, against `living/extra/tv-seat/seat-tvWall-watch-01.png`); with none facing it, they stand (Q-14). Sim tests `tests/slots.rs` (`watching_tv_takes_the_seat_that_faces_the_screen`, `with_no_seat_facing_it_the_tv_is_watched_standing`). The readers' `seatNear` (G-03) can take `around` or no `seatFacing`.
- **Source:** wave: living (extra play run)

### M-bathroom-03 — Wall-backed floor things can stand in the middle of a room

> Consolidation: M-living-07 (living ledger) merged into it: the wall bed (`web/public/content/catalog2/living.json:317`, which has no `fillsWall` either, so play put it mid-room), the ladder shelf (`:956`, leaning on nothing) and the home cinema, whose screen case hangs over the media unit with no wall (`web/public/content/furniture.json:1613`). The packs' wall-hung parts on floor objects (M-packs-01, M-packs-02) would be served by the same rule.
>
> Moved from bathroom.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** M4, M7
- **Fix class:** code
- **Where:** `web/public/content/catalog2/bath.json:38` (`toiletWallHung`), `:223` (`towelLadder`); `web/public/content/furniture.json:2301` (`toiletSmart`): floor layer, nothing ties them to a wall; `crates/sim-core/src/content.rs:263` (`fillsWall` keeps the wall above clear, it doesn't require one)
- **Evidence:** `bathroom/play/chunk2/towelLadder-no-uses.png` (the ladder leans 10.7° on nothing, mid-room), `bathroom/play/chunk4-redo/toiletWallHung-use-00.png` and `bathroom/play/chunk1/toiletSmart-use-01.png` (the boxed-in cistern panel, 1.1–1.2 m tall, stands free with the bowl hung off it). Each is built with its back on the wall face (game depth −0.43) and reads right only there; the blender README's traps say no content fact says a thing stands against a wall. A placement rule (a content `againstWall`, refused away from one, like a wall thing) is new behaviour. It would serve every floor model built to the wall face in the other ledgers too (fridges, counters, wardrobes), though those read as merely pulled out; these three look broken.
- **Status:** fixed (05970ca3)
- **After:** content `againstWall` on `toiletWallHung`, `toiletSmart`, `towelLadder`, `wallBed`, `ladderShelf` and `homeCinema` (`content.rs`, `world.rs` `check_fit`): refused mid-room ("that stands against a wall: put its back to one") or against a window, and its wall can't come down or become a door or window under it (sim test `layers.rs` `what_stands_against_a_wall_needs_one`; build mode, the house generator and `check.mjs`'s stage spots follow; generated houses and worlds pass). The packs' wall-hung parts (Q-06) stay with G-36.
- **Source:** wave: bathroom (from the evidence's "Seen (not reviewed)")

### M-lighting-01 — Removing a wall can leave a ceiling lamp outdoors

> Moved from lighting.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** M4
- **Fix class:** code
- **Where:** `crates/sim-core/src/home.rs:475` (the "needs a ceiling" refusal)
- **Evidence:** Building lets a wall's removal leave a ceiling lamp outdoors (or something upstairs outside a room). It hangs there until the game is loaded again, which leaves it out and loses its value.
  Activities wave: on `review-stage`, a `ceilingPendant` at `(12, 8, 0)` inside the room, then `build` with the front wall's segments at `x = 11, 12` (`z = 6`) set `open`: no refusal and no toast, the room stops being one (its tiled floor is drawn as grass, the kitchen and the crib now stand outdoors) and the pendant hangs from nothing in the open (`lighting/extra/ceiling-before.png`, `lighting/extra/ceiling-after-wall-open.png`; `decor/extra/extra.json`). Not checked: the reload that drops it.
- **Status:** fixed (05970ca3)
- **After:** a build that would leave a ceiling lamp outdoors (or something upstairs or in the basement outside a room) is refused, "the pendant would be left outdoors: move it first", and so is moving a room that would (`home.rs` `walls_still_hold`; sim test `layers.rs` `a_wall_taken_down_leaves_no_ceiling_lamp_outdoors`). A save from before with one outdoors still loads without it (`a_lamp_left_without_a_ceiling_doesnt_spoil_the_save`).
- **Source:** `docs/release-0.2.0-review.md` §4 (new, Z1), target 0.2.1, not in the 0.3.0 notes

### I-kitchen-04 — Cooks chop with the crate in both hands and stir holding the bowl

> Consolidation: I-held-03 (held ledger) and I-systems-07 (systems ledger) merged into it: the same in every recipe run (`held/inhand/timeline.jsonl`: `held: ingredients, action: chop`, `held: prepBowl, action: stir` and `flip`), at the grill too (`held/inhand/bbq-05.png`). The sim side is `crates/sim-core/src/cooking.rs:1501` (`held` kept through the next step), the drawing `web/src/render/pc/characters.ts:2425` (`writeCarried`).
>
> Moved from kitchen.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I13, A1, A2
- **Fix class:** code
- **Where:** `web/public/content/base.json:4135` (recipes: "Get ingredients" `holds: ingredients`, "Prep" `holds: prepBowl` from `:4166`, the sandwich's "Make it" holds nothing new, so the crate stays), `web/src/render/pc/characters.ts:2331` (a carried item is drawn in both hands, with the carry clip, through every step), `:2097` (`writeTool`: the board and pan drawn on the station meanwhile)
- **Evidence:** `kitchen/extra/meal-fridge-stoveGas-01.png` (chopping at the counter with the ingredient crate held at the chest, the board and knife on the top beside it); `-06.png` (stirring the pan on the hob holding the prep bowl in both hands, for the whole 20-minute Cook step); `kitchen/play/c5/counter-sandwich-01.png`, `butcherBlock-sandwich-01.png`, `kitchenIsland-sandwich-01.png` (the sandwich made holding the crate); `stoveElectric-bake-01.png`, `rangeCooker-cook-01.png` (the bowl at the hob). A cook puts the ingredients down to chop and tips the bowl into the pan. Set the carried item down on the station's top while a step is worked (and drop the bowl once the Cook step starts); new behaviour, so `code`. Items: every `prep` station (the counters, islands, butcher block, breakfast counter), every `stove` and `oven` station.
- **Status:** fixed (57b3a576)
- **After:** what the cook carried in is put down on the station's top while the step is worked, beside its tool, and picked up to move on (`SET_DOWN` in `characters.ts`: `chop`, `mix`, `stir`, `flip`; for a `bake` it is in the oven and not drawn); the sim still holds it between steps (`held`), so a save or a crying baby keeps it. In `family-hungry` (the family-check home, the two parents hungry) the crate stands on the counter by the board while chopping (`fixes/cooking/meal-stoveGas-03.png`), the prepped bowl on the cooker's top by the pan while stirring (`-06.png`), the crate by the mixing bowl for a bake's Mix (`fixes/cooking/bake-stoveElectric-03.png`) and nothing in hand at the oven (`-07.png`); against `kitchen/extra/meal-fridge-stoveGas-01.png`, `-06.png`.
- **Source:** wave: kitchen (`scratch/scene.mjs` recipe scene, play shots)

### I-kitchen-05 — Every Cook step is stirring a frying pan, at a microwave, an oven tower or for a bake

> Consolidation: I-held-02 (held ledger) and I-systems-06 (systems ledger) merged into it: the pizza "baked" stirring a pan at the wood-fired oven, the pie at the neighbourly oven, the bake's 35 minutes stirring on the hob with the oven door open and the bowl of batter in the other hand, the Mix and Roll steps chopped with a knife, and the microwave counter's pan drawn at 0.9 m (no `surface`).
>
> Moved from kitchen.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I13, I3, I7
- **Fix class:** renderer-table
- **Where:** `web/public/content/base.json:4135` (`meal` Cook `at: [stove, oven, microwave]`, `anim: stir`; `bakedTreats` from `:4401`, `pie` and `pizza` Bake `at: [oven]`, `anim: stir`); `web/src/render/pc/characters.ts:546` (`STATION_PROPS`: `stir → tool.pan`), `:2102` (the pan at the station's `surface.height`, else `defaultSurfaceHeight` 0.9)
- **Evidence:**
  - Microwave: a meal whose only cooker is the microwave is "cooked" by stirring a frying pan of vegetables drawn on the worktop beside the shut microwave (`kitchen/extra/meal-fridge-microwave-06.png`).
  - Oven tower: the pan is drawn at 0.9 m through the shut lower oven door, the cook stirring at the tower's face (`kitchen/extra/meal-fridge-wallOven-06.png`).
  - Baking and roasting: "Bake treats" at the electric and gas cookers and the range, and "Roast dinner" and "Bake treats" at the oven tower, open the oven door for 3 s at the start and then stir a pan of vegetables on the hob or at the tower while the oven stands empty (`kitchen/play/c5/stoveElectric-bake-01.png`, `wallOven-roast-01.png`, `rangeCooker-bake-01.png`). The Mix step of baking chops on a board (`chop`).
  The steps need their own tags at an oven and a microwave (a tray slid in, a dish put in and the door shut), with props of their own (`item.bakingTray` and the pie dish exist as items) and their own clips on the station. Shared beyond this ledger: the packs' ovens use the same recipes (`web/public/content/packs/foodie.json:150`, the pizza oven's `pizza`; `packs/kind.json:100`, `:119`, the neighbourly oven's `pie` and `bakedTreats`), for the packs and cross-cutting waves.
- **Status:** fixed (2dcc4ea0)
- **After:** the steps have tags of their own: `bake` for the Bake steps (pizza, treats, pie) and for a meal's Cook step in an oven or a microwave (content `animAt`, chosen by the station's kind: a cooker still stirs on its hob), `mix` for Mix and the pie's pastry ("Make the pastry"). At an oven or the microwave the resident puts the dish in and watches it (bending to a cooker's oven), no pan drawn, and the station's door opens (`fixes/motions/recipes/meal-fridge-microwave-06.png`, `meal-fridge-wallOven-06.png`, `bake-stoveElectric-05.png`, `pie-neighbourlyOven-05.png`, `pizza-pizzaOven-*`, `practise-chefStove-*`: the logs show `bake` and the door, `.json`); mixing is stirred in a mixing bowl (new `tool.mixingBowl`) on the counter (`bake-stoveElectric-03.png`). The bowl of batter carried to the oven is G-49's (hands).
- **Source:** wave: kitchen (`scratch/scene.mjs` recipe scenes, play shots)

### A-kitchen-02 — The hob, the oven and the microwave don't work when the meal was started somewhere else

> Moved from kitchen.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** A10, I7, I13
- **Fix class:** motion
- **Where:** `web/public/assets/packs/kitchen_cooking/manifest.json:5`, `:49`, `:103`, `:141`, `:192`, `:238` (the cookers' and the microwave's `motions.uses` keyed by their own use ids: `cook`, `gourmet`, `bake`, `roast`, `reheat`); `crates/sim-core/src/snapshot.rs:140` (at a station that isn't the recipe's origin the interaction is -1, so only the step's tag, `stir`, reaches the renderer); `web/src/render/pc/objectMotion.ts:413`
- **Evidence:** `kitchen/extra/meal-fridge-stoveGas.json`: "Make a meal" started at the fridge, cooked on the gas range from 42.2 to 62.3 s with the `stir` clip and the pan drawn on its hob; no clip on the range moves (no `burner_on` flame, no knob), against `kitchen/play/c5/stoveGas-cook-01.png` where "Cook dinner" chosen at the range itself lights the ring. The same with only a microwave (`meal-fridge-microwave.json`: no `door_open`, no turntable) and only an oven tower (`meal-fridge-wallOven.json`: no `oven_open`). Free will chooses among every recipe use on every station object (the fridge's meal and gourmet, the counters' sandwich, the cookers' cook and bake: `world.rs:3894-3906`), so many meals are cooked on a cold hob. Also seen: the electric cooker's `knob_on` only turns a knob: its ring never glows, where the hob counter's does (`kitchen/clips/clips-stoveElectric-all-t99.png` against `clips-cooktopCounter-all-t99.png`). Key the hob clips by the `stir` tag as well (and the oven's by a bake step's own tag, I-kitchen-05), and give the electric cooker a glowing ring.
- **Status:** fixed (2dcc4ea0)
- **After:** every cooker's hob clip plays for `stir` too, the ovens' for `bake`, the microwave's door and plate for `bake` (the plate turning once the door has shut), the chef's stove's and the pack ovens' likewise. A meal started at the fridge lights the gas range's ring (`fixes/motions/recipes/meal-fridge-stoveGas.json`: `burner_on` 44.7–64.7 s, `-05.png`), opens the microwave and turns its plate (`meal-fridge-microwave.json`, `-06.png`) and the tower's oven (`meal-fridge-wallOven.json`); the electric cooker's front ring glows red with its knob (`burner_on`, `meal-fridge-stoveElectric-06.png`, `fixes/motions/g16/clips-stoveElectric-all-t99.png`).
- **Source:** wave: kitchen (`scratch/scene.mjs` recipe scenes)

### I-held-04 — Whole dishes never leave the station

> Moved from held.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** I13, I1
- **Fix class:** code
- **Where:** `crates/sim-core/src/cooking.rs:9-14` (the dish is served on a free spot near the stove; each eater takes a serving from there)
- **Evidence:** The six whole dishes (`mealPot`, `tastingBoard`, `grillPlatter`, `pizza`, `bakingTray`, `pieDish`) are never in a hand in play: the cook puts the dish down where it was cooked and everyone fetches a plate from there (`held/index.md` Problems; `held/inhand/bbq-06.png`, `pizza-05.png`, `bakedTreats-05.png`, `pie-07.png`); the pot is carried only dirty, to the sink. The pot's carry pose exists and works (`held/inhand/poses/poses-carry-hold.png`). For a family dinner the dish goes to the table and is served there; carrying it to the dining table, when the home has one, would use the models and poses already built.
- **Status:** fixed (57b3a576)
- **After:** the finished dish is carried to the dining table and put down there near its middle (a new serving step past the last: `cooking.tableRange` 12 m on the cook's storey, an indoor table for an indoor dish and an outdoor one for a barbecue, a table before a counter or bar with stools; `serveMinutes` 1 with the `serve` clip, "Serve it" in the action panel); "dinner's ready" comes once it is on the table, and the hungry take their plates from there and sit at it. Without a table near it is served near the station as before. A save on the way keeps the dish in hand and serves it once (`tests/cooking.rs`). On `review-stage` with a four-seat table the pot goes from the hob to the table in both hands (`fixes/cooking/meal-table-08.png`) and Ada eats at the table with the pot beside her plate (`-10.png`); in `family-hungry` it goes to the island with a stool, the nearest top with a seat (its dining set's 0.38 m top strip has no room for a pot), and she eats on the stool (`fixes/cooking/meal-stoveGas-08.png`, `-10.png`); the baked treats likewise (`fixes/cooking/bake-stoveElectric-10.png`). Against `held/inhand/bbq-06.png`, `pizza-05.png`, `bakedTreats-05.png`.
- **Source:** wave: held (shared with the systems ledger's cooking flow)

### I-garden-03 — A crop is picked the moment the harvest starts: the bed empties, then the resident picks at nothing for 18 minutes

> Moved from garden.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I8, A10
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:4978-4982` (`if inter.picks && … growth >= 0.0 { growth = -1.0 }` as the use starts); the growth clip then goes from ripe to picked at once (`G.stages`, `plants.py:1063` `_crop`)
- **Evidence:** `extra/harvest/s-garden.vegPatch-harvest.png`, `s-garden.pumpkins-harvest.png`, `s-garden.strawberries-harvest.png`, `s-garden.tomatoes-harvest.png`, `s-raisedBed-harvest.png` (three shots each, the first as the use starts): on every crop the ripe vegetables, pumpkins, berries and tomatoes are there in the first shot and gone 1–2 game minutes into the 20-minute harvest; the resident goes on with the harvest clip over bare soil or leaves to the end. The same on the outdoor ledger's `raisedBed` (and, by the same rule, on the packs ledger's `natureLover.veggieBed` `pickSnack`). Pick the crop when the use ends (or in steps through it), so the produce goes as the hands take it.
- **Status:** fixed (7d186856)
- **After:** a crop is picked when the harvest ends: the bed stays ripe through the 20 minutes and empties as the hands finish (`world.rs` `leaves_at_end`), and a harvest goes on to its end even with hunger full; the raised bed's `harvest` clip plays along with the use (`motions.uses` `over`, the snapshot's new `progress`), its tomatoes and carrots going as she picks. `review-garden`: `fixes/cooking/harvest/harvest-raisedBed-harvest-00.png` (ripe as she starts), `-03.png` (half way: the lettuces up, the crop half gone), `-06.png` (bare once done); `harvest-garden.vegPatch-harvest-01.png` (still full mid-harvest), `-04.png` (picked after). Against `extra/harvest/s-garden.vegPatch-harvest.png`, `s-raisedBed-harvest.png`.
- **Source:** wave: garden (`play --state review-garden --use-existing … --use harvest`)

### A-outdoor-02 — The washing goes out and comes in, and the crop is picked, in the first two seconds of the use

> Consolidation: A-systems-02 (systems ledger) merged into it: the `washing` clip runs its 1.5 s as the 20-minute use starts (`web/public/assets/packs/yard/manifest.json:361`), the basket full beside the resident. The crop half is I-garden-03's (the same rule, `crates/sim-core/src/world.rs:4978-4982`); one fix closes both.
>
> Moved from outdoor.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** A10, I8
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:4971-4975` (`leaves` applied as the use starts); `tools/art/blender/families/yard.py:853` (`G.grow(washing, 'washing', seconds=1.5)`), `:459` (`G.squash(crop, 'harvest', 0.001, seconds=1.0)`)
- **Evidence:** Hanging out the washing: all nine pieces appear at once as the 20-minute use starts, then the resident mimes in front of a full line (`sheets/obj-clothesline.png`); taking it in empties the line at the start of its 10 minutes (`play/chunk3/clothesline-takeIn-01.png`). Harvesting the ripe raised bed: the whole crop is gone two seconds in, with the resident's hands on the frame and nothing in them (`extra/harvest/harvest-sheet.png`). The state should change at the end of an emptying use (take in, harvest) and a filling use's clip run over its length, or in steps. The same rule opens a wall bed as the sleeper lies down, which is right; it is the slow jobs it doesn't fit.
- **Status:** fixed in part (7d186856); one piece at a time is the Blender wave's
- **After:** hanging out and taking in are `gradual` (content): the line is out, or in, once the use is done, and the `washing` clip plays along with the use (`over` 1 out, -1 in), so the washing comes down onto the line through the 20 minutes and goes up through the 10 of taking it in (`fixes/cooking/washing/line-clothesline-washing-00.png` empty as she starts, `-03.png` most of the way down; `line-clothesline-takeIn-01.png` half up, `-03.png` in). The crop half is I-garden-03's. The nine pieces are one part, so they come down together rather than piece by piece; a part per line would take the model to 5 draw calls, over the gate's 4 (`cargo check-glb`), so the pieces one at a time wait for a rebuild that keeps the still body to one finish.
- **Source:** outdoor wave (`play --state review-garden --use-existing raisedBed --use harvest`)

### I-garden-10 — Fruit and vegetables are eaten on the spot, at any time: endless apples and lemons, a lemon eaten like an apple, a raw pumpkin as a meal

> Moved from garden.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** I5, I16
- **Fix class:** decision
- **Where:** `web/public/content/garden.json:210-241` (apple: `pick`, no `grows`, hunger 0.25), `:406-437` (lemon tree: `pick`, `anim: eat`, hunger 0.1, energy 0.05), `:1494`, `:1612`, `:1718` (veg patch, strawberries, pumpkins `harvest`: hunger 0.45, 0.25, 0.5 at the bed); `catalog2/outdoor.json:342` (`raisedBed` `harvest`)
- **Evidence:** `tables/uses-garden.md`. The fruit trees have no ripeness (`grows`), so an apple can be picked and eaten every 15 minutes all year, where the crops beside them ripen in 2–4 days. The lemon is eaten for hunger like the apple ("a fresh boost"); a lemon is picked for a drink or for cooking. A harvest fills hunger at once, as if eaten raw in the bed: half a meal from a pumpkin, 0.45 from cabbages and carrots. In life the harvest goes to the kitchen (the herbs already work that way: a cooking boost, `garden.freshHerbs`). The same on the outdoor ledger's `raisedBed` and the packs ledger's `natureLover.veggieBed` (`pickSnack`, `anim: eat`). For triage: give the fruit trees `grows` (the apple model would need its growth stages), make the lemon a drink or cooking boost, and decide whether a harvest feeds directly (a game shortcut) or stocks the fridge (new behaviour, `code`).
- **Status:** fixed (9bb1c2ff, fe8cf92d)
- **After:** The fruit trees `grows` (3 days, no fallow) and their models show it (`ripen`: the fruit gone once picked bare, back when ripe; `fixes/babies-crops/garden/apple-00-in-fruit.png`, `apple-07-bare.png`, `apple-08-in-fruit-again.png` three days on). In fruit, "Pick an apple" is still eaten standing under it (10 min, hunger 0.15) and leaves the tree in fruit; "Pick the apples" picks it bare (`apple-02-picking.png`, reaching up into the crown, `pickHigh`). The lemon tree is picked bare, ripe, for Fresh lemons (a cooking boost, as the herbs), no hunger (G-21's `pickLemon*` without the eating after, `lemon-01-picking.png`). Harvests take `yields`: the veg patch, raised bed, tomatoes, strawberries, pumpkins and the apples go into the hands as `gardenProduce` (2–3 servings; `patch-00-harvest.png`), carried to the home's fridge and put away (`patch-01-carrying.png`, `apple-05-carrying.png`; the item `stored` in `garden/log.txt`), eaten later at a table like leftovers; a hungry harvester has a serving there and then; with no fridge, they sit down and eat one. The nature lover's bed keeps its nibble at the bed (the shortcut Q-18 keeps). Balance and soak: see `fixes/babies-crops/log.md`. Picked as the harvest starts, not at the hand: G-50 (I-garden-03).
- **Source:** wave: garden

### I-bathroom-04 — Noisy uses with no sound: the toilet, the treadmill, the workbench

> Consolidation: I-hobbies-09 (hobbies ledger), I-kitchen-12 (kitchen ledger) and I-packs-19 (packs ledger) merged into it: one cause, `ACTION_SOUNDS` by the clip's tag only (`web/src/audio/world.ts:34`). Silent as well: the pool table, foosball, darts, the arcade, the sewing machine, the chemistry bench, the pottery wheel, the exercise bike, the dishwasher, the coffee machine, the blender's motor, the microwave's hum and ping, the toaster, the fridge door and bin lid, both fountains, the court and table balls, the vending machines and the phone box (community ledger), and the stone fountain (A-outdoor-03). Wrong: the drum kit plays the plucked `music` tune; the toaster, blender, microwave, cheese board and oven bakes sizzle like a pan (any `cook` use away from a hob: 31 in the content); the smart speaker's story plays music.
>
> Moved from bathroom.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** I9
- **Fix class:** code
- **Where:** `web/src/audio/world.ts:34` (`ACTION_SOUNDS`: no `toilet`, `run` or `tinker`); `:193` (`clean` sounds only for a washer)
- **Evidence:** `cargo review uses` Sound column (`—` for `toilet`, `treadmill`, `workbench`) and `world.ts`: no flush after the toilet, no belt hum or footfalls on the treadmill, no tools at the workbench. The shower, tap, TV and piano have theirs. Listed as missing, as I9 asks; the sounds are synthesised in code (no sound files).
  Bathroom wave: the same for all five toilets (`toiletCompact`, `toiletSmart`, `toiletClassic`, `toiletWallHung`: Sound `—`, `tables/uses-bathroom.md`), and for the tumble dryer, whose drum turns silently while the washer's runs with the `washer` loop (`world.ts:193` matches `washer|laundryCent` only).
- **Status:** fixed in part (54139115): the sounds the repo can make; the rest are listed below as sounds to make (art). Before: confirmed and widened (bathroom wave: all five toilets and the dryer; consolidation: I-hobbies-09, I-kitchen-12 and I-packs-19 merged)
- **After:** a use's sound comes from the thing, by key (content `sound`, `endSound`, a recipe step's `sound`, the object's `sounds` by animation and `ambient`; `docs/content-packs.md`), else the old tag map; `cargo review uses` reads the same. The toilets flush once as the use ends; the dryer tumbles (`dryer`); the dishwasher is loaded with crockery and cutlery (`crockery`, also when clearing away into it); the stone and town fountains run by themselves while their lot is looked at (`fountain`); the drinking fountain runs (`tap`); the drum kits play a beat (`drums`); the smart speaker's story is a voice (`tv`); the cheese board chops; the hot tubs, whirlpool and cold plunge lap (`bathSoak`) instead of filling; karaoke plays its backing track (`music`). Toast, smoothies, the blender, the microwave and the microwave cart, and every bake step (also a cooker's oven half) are silent instead of frying. New sounds are synthesised like the rest (water.ts: flush, fountain, dryer, crockery; world.ts: drums). Evidence: `bench-results/review/fixes/sound-social/sounds/sounds.log` (the page's `soundOf` for every object named, and a live run: toilet → `flush` once, dryer, washer, dishwasher, drums heard while used; microwave and toast silent; both fountains heard with nobody at them), `sounds/uses-sound-diff.txt` (the Sound column before and after). Still to make (art, for later): the treadmill (belt, footfalls), the exercise and spin bikes, the workbench and bike-rack tools, pool, foosball and ping-pong balls, darts, the arcade cabinets, the sewing machine, the pottery wheel, the chemistry bench's bubbling, the coffee machines, the blender's motor, the microwave's hum and ping, the toaster's pop, an oven's fan, the pizza oven's fire, the fridge door, the bin lid, the vending machines, the phone box, the basketball, singing, a bowed cello (it plays the plucked tune), the punch bag and smash bin. Not done: the headed listening check (the WAVs are with the evidence).
- **Source:** pilot; wave: bathroom

### A-houses-01 — House doors open by nearness, not by hand

> Moved from houses.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2 (raised from S3: the bathroom door below)
- **Rubric:** A12, A10, I6
- **Fix class:** code
- **Where:** `web/src/render/pc/PlayCanvasRenderer.ts:3778` (`swingDoors`); the rule at `:3794` (`target = passing || !(leaf.front || engaged)`: anyone moving within `DOOR_REACH` 1.3 m, `:189`, opens any door, an occupied bathroom's too)
- **Evidence:** Front and bathroom doors swing when someone walks near them; nobody reaches for the handle. Doors opened by hand are new behaviour: filed as `code`, planned separately (§9, decision 3).
  Wave houses, on `review-stage` (`houses/extra/houses.jsonl`):
  - Walking along the front wall outside, 0.7 m off it, past the door and not through it, Ada opens the front door fully (`swing1-past-0.png`; "swing1 past": open 0 → 1 as she passes x 4.5 → 5.4).
  - A bathroom built in the back corner: with Ada in there (on the toilet, then standing by it) the door stays shut; Ben walking along the hall past it, 0.7 m off, swings it wide open on her (`sheet-bath.png`, `bath-passer-open-0.png`; "bathPass": 0 → 1 at 1 m from the door). That breaks the one privacy the renderer gives (`I-systems-05`, systems ledger) and is what raises this to S2.
  - The cheap part, short of hands on handles: open a door only for a mover heading through it (their next path step crosses the doorway), and keep an engaged bathroom's door shut for everyone but someone leaving it.
- **Status:** fixed (e04bc6c0): the cheap part (Q-03); doors opened by hand stay new behaviour (decision 3)
- **After:** the snapshot tells each walker's way ahead (`snapshot.rs` `WAY_X`, `AHEAD_X`: the waypoint they walk to and the point 1.5 m on), and `swingDoors` opens a door only when that way crosses its doorway, or someone stands in it (`goingThrough`); open, it waits to shut until nobody walking is within 1.1 m (the leaf would sweep through them). On `review-stage` (`fixes/town/doors/doors.jsonl`): Ada walking along the front wall outside 0.7 m off, past the door, leaves it shut (open 0 throughout, `fixes/town/doors/swing1-past-0.png`, against `houses/extra/swing1-past-0.png`); walking in at speed 1 it opens from 1.5 m out and stands fully open before she reaches it. A bathroom in use: Ben walking the hall 0.7 m off it leaves it shut (open 0, `fixes/town/doors/bath-passer-0.png`, against `houses/extra/bath-passer-open-0.png`); Ada going in and coming out opens it for her (`bath-in-1.png`, `bath-leave-0.png`), and it shuts behind her while she is in there (`bath-occupied-shut.png`). One way the cheap part goes past the decision's words: someone walking into an engaged bathroom (the sim lets them: no privacy, I-systems-05, G-72) still has it open for them rather than walking through the shut leaf; passers-by never do.
- **Source:** `plan/authenticity-review.md` §3.2 (A12) and §9 (decision 3); wave: houses (shared with the systems ledger's "Front door: who opens it and when" and "Bathroom door: shut while in use" rows)

### A-houses-02 — At the faster speeds residents walk through a door that has hardly opened

> Moved from houses.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** A9, A10
- **Fix class:** code
- **Where:** `web/src/render/pc/PlayCanvasRenderer.ts:188` (`DOOR_SWING` 0.45 s), `:3781` (the step is real frame time, not game time)
- **Evidence:** At speed 1 the front door is fully open 0.6 m before Ada reaches it (`houses/extra/swing1-in-1.png`; "swing1 in"). At speed 3 she crosses the 2.6 m the door watches in about 0.3 s of real time, while the door takes 0.45 s: she is through it at 19 % open and the leaf shuts behind her at once (`houses/extra/swing-in-0.png`, her body through the near-shut leaf; "swing in": open 0.19 at z 5.7, 0.11 when she is inside). Scaling the swing by the game's speed (as walkers' turning is, `characters.ts:2324`), or the reach by the walker's speed, keeps the door ahead of her.
- **Status:** fixed (e04bc6c0)
- **After:** the swing keeps pace with the game (`DOOR_SWING` over the game's rate, up to 10x, as walkers' turns), and opening for the way 1.5 m ahead gives it about a second's walk of lead at any speed. At speed 3 the front door is fully open when Ada is 0.3 m short of the doorway (open 1 at z 5.7, where it was 0.19; `fixes/town/doors/swing-in-0.png` against `houses/extra/swing-in-0.png`).
- **Source:** wave: houses

### A-town-02 — Gates never open: residents walk through the shut leaf

> Moved from town.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** A12, M7
- **Fix class:** code
- **Where:** `web/src/render/pc/fences.ts:42` and `:110` (`gate()`: one still mesh with the fences, the leaf drawn shut); `crates/sim-core/src/command.rs:389` (a gate is walked through); `web/src/render/pc/PlayCanvasRenderer.ts:3778` (`swingDoors` turns only house doors)
- **Evidence:** On `review-stage` a picket gate in a run of fences: Ada walks through it, her body passing through the shut leaf (`town/extra/gate-walk-0.png` … `gate-walk-2.png`, `town/extra/sheet-fences.png`; `houses/extra/houses.jsonl` "gate"). The house's front door swings for her on the same lot. Swinging the leaf as `swingDoors` swings a front door (hinge on a post, a quarter turn away from the walker) is new behaviour (§9, decision 3).
- **Status:** fixed (e04bc6c0)
- **After:** a gate's leaf swings a quarter turn about its hinge post, away from the one coming through, by `goingThrough` as doors do, and shuts the way it opened once they're clear (`fences.ts` `swingGate`): it moves its vertices in the fences' one mesh, so it costs no draw. On `review-stage`, a picket gate in a run of fence: open from 0.7 m out as Ada walks in, fully open as she passes (`fixes/town/doors/gate-in-1.png` against `town/extra/gate-walk-1.png`), shut behind her; the same on the way back (`gate-out-*.png`); walking along the fence past it, it stays shut (`gate-past-0.png`).
- **Source:** wave: town

### M-lighting-02 — Every lamp is a bare point light: the uplighter lights the floor, wall lights throw a disc, and light goes through walls

> Moved from lighting.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** M7, M10
- **Fix class:** code
- **Where:** `web/src/render/pc/PlayCanvasRenderer.ts:3075` (the interior lights: `PointLight`s, no shadows), `:4714` ("a warm pool around it"); `web/src/render/pc/house.ts:222` (one light per lamp at `light.height`, no direction, no room limit)
- **Evidence:** A shade decides where a real lamp's light goes; here none does, and no wall stops it.
  - `lampTorchiere`: the uplighter's bowl glows and its light is a pool on the floor round its foot, the same as a shaded floor lamp's, walls up or cut away (`lighting/extra/lampTorchiere-night-wallsup.png`, `lighting/night/c2/lampTorchiere-no-uses.png`); a real one washes the ceiling, and the ceiling isn't drawn in Live mode either.
  - `wallSconce`, `wallLampArm`, `wallLightOutdoor`: a bright round disc on the wall centred on the fitting (`lighting/night/c1/wallSconce-no-uses.png`, `…/wallLampArm-no-uses.png`), not the up-and-down wash of a sconce or the arm lamp's pool on the chair below.
  - `lampBanker` and the lamps on tops: a disc up the wall behind them (`lighting/extra/lampBanker-night-bought.png`), where a banker's lamp lights the desk under its shade.
  - `wallLightOutdoor` on the outside of the back wall, walls cut away: the fitting isn't drawn, its light lands on the inside floor by the wall (`lighting/night/extra/wallLightOutdoor-outside-on.png`; walls up, the wall is lit outside, `…-outside-on-wallsup.png`).
  Shared beyond this ledger: every content `light` is drawn so, also `nightstandLamp` (bedroom), `fireplace` (decor), `fireplaceElectric` (living), `lampPost`, `stringLights`, `firePit`, `garden.lantern` (outdoor, garden) and `streetLamp` (community). Directional lamps (a spot or a cone per kind) and lights kept to their room would be new renderer behaviour.
- **Status:** fixed (0eaa3ba7, ae10185e): light kept to its room with M-lighting-08 (Medium and High)
- **After:** the room lights are spot lights and content `light.aim` sends a shade's light one way: `down` (pendant, industrial pendant, track lights, swing-arm, arc and banker's lamps: no disc up the wall, `fixes/lighting/webgpu-medium/banker-cut.png`, `pendant-cut.png`), `up` (the sconce: a wash up the wall above it, `sconce-cut.png`), `out` (the bulkhead: nothing behind its wall, the inside floor dark, `bulkhead-inside.png`; a pool on the lawn in front, `bulkhead-cut.png`, `bulkhead-up.png`) and `ceiling` (the uplighter: from 30 cm under the ceiling, the room lit from above rather than a pool round its foot, `uplighter-cut.png`, `uplighter-up.png`). The same in `webgl-medium`, `webgpu-low`, `webgl-low`. Same draws and lights; GPU within noise on Low and Medium (`fixes/lighting/perf`). Lights passed through walls into the next room until M-lighting-08 (ae10185e) kept each clustered lamp to its own room on Medium and High; Low's four forward lamps aren't masked.
- **Source:** wave: lighting (leads "the uplighter lights the floor, not the ceiling" and "the outside bulkhead's light reaches the inside floor with walls cut away", both confirmed)

### I-lighting-01 — A lamp bought is on, at night in an empty room too

> Moved from lighting.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** I15, I8
- **Fix class:** decision
- **Where:** `crates/sim-core/src/world.rs:129` ("A lamp switched off …; bought on"), `:1553` (`off: false`); `crates/sim-core/src/lights.rs:3-10` (residents switch only on coming and going)
- **Evidence:** `lighting/extra/lamp-night-bought.png`: on `review-stage-night` (22:00 by the sim's clock), a floor lamp bought in the room while everyone is in the front yard (and the baby asleep) comes on and stays on; nobody came or left, so no rule switches it. The same by day: a lamp bought at noon in a room nobody uses is lit when dusk comes. A new real lamp is off until someone switches it on. Decide: off when bought (and residents switch it on as they come in), or on (the player sees the lamp work). Covers every lamp in this ledger and the other ledgers' (`nightstandLamp` and the rest named in M-lighting-02).
- **Status:** fixed (0eaa3ba7)
- **After:** Q-11: a lamp (or a fire) bought comes switched off (`home.rs` `buy_at`); it goes on with the room's as someone comes in after dark, or by a click (sim test `lamps.rs` `a_lamp_bought_is_off_until_someone_comes_in`; in the browser the bought lamp is in `lampsOff`, `{E}/*/*.json`). A lot's and a house's own lamps still start on.
- **Source:** wave: lighting (lead "a newly bought lamp is already on", confirmed)

### M-outdoor-01 — Night glow draws through other parts: the grill's coals through its shut lid

> Outdoor wave: confirmed. On the outdoor objects only the grill meets the fault, and it pays for the work-round: its coals are plain red with no glow and give no light, even at night with the lid open, where the fire pit's embers and flames glow.
>
> Moved from outdoor.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** M10
- **Fix class:** code
- **Where:** `web/src/render/pc/models.ts:73` (`glows`), `tools/art/blender/families/outdoor.py:64`
- **Evidence:** A glowing material draws through the parts in front of it. `outdoor.py:64` now builds the grill's hot coals with no glow "which would show through its shut lid", so the grill may be worked round while the renderer still has the fault for other models. `extra/night/sheet-fire.png`: the grill cooking at 22:00 with the lid up shows dull red coals and no light on the cook; the fire pit beside it glows.
- **Status:** fixed (0f7cf80f); the grill's coals glow again (b4306968)
- **After:** the glow copy no longer takes a slope-scaled depth offset (`models.ts` `glowMesh`: `zOffsetUnits` -4 instead of `zOffset` -2). With the grill's coals made to glow for the check (a local manifest edit, not kept), the shut lid showed red specks of the coals from 9 m before (`fixes/lighting/webgpu-medium/grill-far-glowing-coals-before.png`) and none after (`grill-far-glowing-coals.png`, `grill-shut.png` in each folder); shades glow as before, no z-fighting. Not shot: the lid open (cooking needs the kitchen's ingredients on the stage). Giving `coal_hot` its glow back (`outdoor.py:64` and the manifest) is a model change for the Blender wave (G-35): done in b4306968 (`coal_hot` `glow` #FF7A2E in `packs/outdoor/manifest.json`; not shot cooking at night, as above).
- **Source:** `docs/objects/open.md` Still open › Rendering; `plan/authenticity-review.md` §3.1 (M10); outdoor wave (night run)

### I-systems-12 — A housemate cooking a meal nobody needs can't hear a call

- **Severity:** S3
- **Rubric:** I16, I2
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:885` (cooking, eating and washing up are never interrupted); `crates/sim-core/tests/calling_out.rs`
- **Evidence:** With every need at 1.0 at 9:00, free will sent the called housemate to cook a full meal at the fridge (`TaskKind::Cook`, recipe `meal`) once the base armchair had its sitting read (G-09); a cook can't heed a call, so `calling_out` failed ("B never heard A"). Before, the same housemate picked the bookshelf's read. Two things: a meal is cooked for a fed household at mid-morning, and someone at the hob can't so much as turn to their name. The test now gives B a little fun to fill (d0bbe6ff), which keeps its point (a call heard and answered) off the content's order.
- **Status:** fixed (342dd820, c5a96988)
- **After:** both halves: a meal nobody at home would eat (neither the cook nor a hungry housemate) tempts free will a tenth as much (`UNWANTED_MEAL`), so a fed household no longer cooks at mid-morning (`cooking::a_household_that_has_eaten_cooks_no_meal_nobody_needs`, fails without it; skipping it outright left a resident in a bare house idle for hours in the soak); and a cook on the way to the first station (nothing bought, nothing in hand) is available to be called (`world::tests::a_cook_on_the_way_to_the_first_station_can_be_called_but_not_once_started`). `calling_out` passes without d0bbe6ff's lowered fun (removed). Balance after: median weekly $443, richest $11,829, grades 1.4 / 3; soak 4 of 4 (`fixes/sweep-sim/balance-after.txt`, `soak-after.txt`).
- **Source:** fix: content

### I-systems-13 — Sitting on a thing moved off its tile's middle sits beside it

- **Severity:** S2
- **Rubric:** I3 (A1)
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs` (`slot_position`: from the footprint's middle, without the object's `offset`); `web/src/render/pc/objectMotion.ts` (`slotOf`, the same)
- **Evidence:** found while fixing G-62 (code reading, not shot): anything but stairs stands up to half a tile off its footprint's middle (`offset`, placed with Shift), and the renderer draws it there (`protocol.ts` `objectCentre`), but the sim put its sitters and sleepers at the footprint's middle, up to 0.5 m off the seat or the mattress.
- **Status:** fixed (aac64b84)
- **After:** `slot_position` starts from the middle as drawn (the offset included), for places and the spread alike; `slotOf` likewise. Not shot.
- **Source:** fix: slots

### A-systems-03 — At an object that is two stations, the recipe's own clip plays for every step there

- **Severity:** S3
- **Rubric:** A10, I13
- **Fix class:** motion
- **Where:** `crates/sim-core/src/snapshot.rs` (`doing`, Cooking: at the recipe's origin the snapshot gives its interaction for every step done at that object); `web/src/render/pc/objectMotion.ts` (`findUses`: a clip keyed by the interaction id wins over the step's tag); `tools/art/blender/families/pack_foodie.py` (`chefsIsland`: `{'uses': {'tastingMenu': 'drawer_open'}}`), `outdoor.py` (`grillStation`: `hood_up` for cook and feast)
- **Evidence:** seen in the code and content while fixing G-16 (no run): the chef's island is a `prep` and a `stove` station, so "Cook a tasting menu" chosen at it preps and cooks there, and both steps play the drawer (`drawer_open`, `hold` 2.5) and nothing on its hob (it has no hob clip, so a tasting menu is cooked on a cold hob, as A-kitchen-02 was); the grill station ([`grill`, `prep`]) lifts its hood for the prep step as for the grilling. The neighbourly oven had the same fault (its oven open while the pastry was made on its worktop) and was fixed by keying its clip by the `bake` step's tag (2dcc4ea0). Key such objects' clips by the step tags (`chop`/`mix`, `stir`, `flip`) and give the island's hob a burner clip.
- **Status:** fixed (57b3a576, db55d69b)
- **After:** the recipe's own interaction now names only its last step at the object it was started from (`snapshot.rs`): the grill station lifts its hood for the grilling, not for the prep on its counter, and the chef's island opens its drawer for a `chop` at it (the prep) and no longer while the menu cooks. Its hob still has no burner clip (a model change). Seen in the code and the snapshot test; no run. The chef's island's clips are keyed by the step tags, not the use: `chop` (the prep step) opens the drawer (`hold` 2.5), `stir` (the cooking) grows a red hot ring under the copper pan (`burner_on`, its own part: the stools went plain to keep 3 draws), so the tasting menu is cooked on a hot hob (`bench-results/review/fixes/blender-packs/clips/clips-foodie.chefsIsland-*.png`, `play/day-foodie.chefsIsland-tastingMenu-01.png`).
- **Source:** fix: motions

### I-systems-14 — A desk can't stand with its screen to a wall

- **Severity:** S3
- **Rubric:** I3
- **Fix class:** code
- **Where:** `crates/sim-core/src/home.rs:502-517` (placing: the approach tile, in front, must be free or reachable), `web/src/game/surfaces.ts` (`approachCandidates`: the front for everything not `around`)
- **Evidence:** from the content and the rule, not shot: the desks' user sits behind the desk facing its front (the computer desk's monitor "its back to the room", `office.py`), so the tile build mode keeps free is the one beyond the screen. A desk pushed against a wall with the sitter facing the wall, the usual way, can't be placed; turned the other way the chair is squeezed against the wall (`bench-results/review/fixes/positions/play/computerDesk-wall-in-200.png`). Since G-51 the desks are sat at from behind or beside (`approach: behind`), so build mode could take a free tile behind or beside the chair as well as the front for them (and the beds' sides for `approach: side`).
- **Status:** fixed (3dbfaa48)
- **After:** for a seat behind the work (`approach: behind`) `approach_tiles` gives the tiles behind, left and right that can be stood on, then the front only if it can be; `wall_edges` no longer needs the front open; build mode mirrors it (`approachCandidates`, no "Faces a wall" for them). A desk with its screen to a wall between two fridges is bought and sat at from behind; with the tiles behind taken too it is refused (`footstools::a_desk_stands_with_its_screen_to_the_wall_between_other_things`). Generated houses and worlds checks pass.
- **Source:** fix: positions

### I-systems-15 — Buying round someone in bed can shut them in where they get up

- **Severity:** S2
- **Rubric:** I3
- **Fix class:** code
- **Where:** `crates/sim-core/src/home.rs` `keeps_reach` ("No Sim at home gets shut in": each resident's own tile, which for someone lying in bed or sitting is the bed's or seat's, never reachable, so never checked)
- **Evidence:** found by the soak test while fixing G-65 (`--test soak -- --ignored`, busy player, seed 7; the games for two changed the town's run): R18 stood idle for days at (220.5, 13.5) with every need at 0, every use and visit remembered as out of reach, between the bed they had slept in (`bed` at 219,14) and a `lazy.snoozecloudBed` the player had bought at 221,13 while they slept. The check let the bed go in because R18's tile was the bed's; the tile they stood up on was closed off. `bench-results/review/fixes/partners/soak-2.txt` (34 stretches stuck), `soak-3.txt` after (4 of 4 pass).
- **Status:** fixed (15309c9d) — the check takes the tile they'll stand up on (`seated_from`) for someone in bed or on a seat.
- **After:** the soak test passes (4 of 4) with G-65; balance unchanged. Found again by fix: sweep-sim's soak (seed 7, R41 boxed in by a karaoke stage bought beside the bed while asleep) and fixed the same way (f15bbd47); G-65's version is kept at the merge, with sweep-sim's test `daily_life::a_resident_in_bed_cant_be_shut_in_where_they_get_up` (walls round the tile they lay down from, the bed still reached at its other slot: refused; fails without the fix).
- **Source:** fix: partners

### I-systems-16 — Selling a lamp at head height leaves the table under it walked through

- **Severity:** S2
- **Rubric:** I3
- **Fix class:** code
- **Where:** `crates/sim-core/src/home.rs` `remove_object` (every tile the sold thing took from walking set free, also those something else still takes)
- **Evidence:** found by fix: sweep-sim while giving the arc lamp's shade a tile (M-lighting-06): a chandelier (`headHeight`) over a sofa takes the sofa's tile from walking too; sold, the tile was free though the sofa stood there, and residents walked through it (the same for the sofa sold from under a chandelier: then walked under the lamp). `layers::a_lamp_at_head_height_hangs_over_a_table_not_over_heads` (extended) fails without the fix.
- **Status:** fixed (2cb0ca69)
- **After:** a sold thing's tiles are taken again by whatever else takes them; the test passes, and so does the arc lamp's (`layers::an_arc_lamp_reaches_over_the_sofa_in_front_of_it`).
- **Source:** fix: sweep-sim

### I-systems-17 — A desk with its back to a wall and a chair on its front tile can't be sat at

- **Severity:** S2
- **Rubric:** I3
- **Fix class:** code
- **Where:** `crates/sim-core/src/surface.rs` `approach_tiles` (I-systems-14, 3dbfaa48: for `approach: behind` the other middle tile along an even front edge was left out); `web/src/game/surfaces.ts` `approachCandidates` (the same)
- **Evidence:** CI's generated-worlds check on `review/authenticity` (e41ab47a): `"dense" 314192 ("Queen's Cross"): can't reach the deskWriting downstairs at 97,149`. The sampler's households are drawn with `Math.random`, so the furnishing differs per run; seeded, 8 of 12 draws had it. A bedroom 97–100 × 148–151 on a lot south of a street (the house turned round): a `deskSimple` at 97,148 with its `officeChair` at 98,149, and the writing desk at 97,149 turned (rot 1, 1×2) with its back to the west wall. Behind it the wall, left the `nightstandLamp` at 97,151, right the `deskSimple`, its front tile (98,149) the office chair: `use_tiles` was empty, though 98,150 beside the chair was free. The generator (`place`) kept its own front tile free, which the turn made the other middle tile; for everything else `approach_tiles` takes that one when the front is taken, but I-systems-14 excluded it for desks.
- **Status:** fixed (b8956e80)
- **After:** for a seat behind the work, after behind, left and right, the front if it can be stood on, else the other middle tile if that can; build mode's `approachCandidates` lists it too (the generator doesn't use it for desks, so the samples are unchanged). The desk's use tiles are now [98,150]; 60 seeded draws of "dense" 314192 and 72 of every kind pass, as do the 24 sample worlds and 180 houses. `footstools::a_desk_with_its_back_to_the_wall_is_sat_at_from_the_other_half_of_its_front` (a pouf on the front tile, sat at from the other half; with that taken too, refused) fails without the fix.
- **Source:** fix: CI generated worlds
