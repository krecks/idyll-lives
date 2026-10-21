# Systems: authenticity ledger

What it covers: the behaviour that spans objects: cooking and its 9 recipes over stations (`cooking.rs`, `base.json` `recipes`, `cooking`), baby care at the crib, lamps (`lights.rs`), house doors, objects that open and stay open (`when`, `leaves`), crops (`grows`) and visitors knocking (`base.json` `visits`). Each object's own uses are judged in its own ledger; these rows judge the system. Rubric I13–I15 in particular.

- **Evidence:** `bench-results/review/systems/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Never looked at before** (`docs/objects/open.md` "Not looked at"): a meal from the fridge eaten at a pulled-in chair (eating was checked with the café's cake). Cross-cutting wave: seen, the sandwich eaten at the dining table on a pulled-in chair with its plate on the table (`held/inhand/sandwich-04.png`).
- **Cross-cutting wave (2026-10-21):** the recipes from the held wave's runs on `family-check` (`held/inhand/<recipe>-NN.png` and `timeline.jsonl`; contact sheets `systems/sheets/cook-<recipe>-N.png`), the kitchen, outdoor and packs waves' station shots, and scenes of its own (`bench-results/review/scripts/scenes.mjs`, log in `systems/scenes/scenes.log`): baby care on `review-baby` (a copy of `review-stage` with the baby hungry and wet and free will on), lamps on `lamps-night` with free will on, the toilet and the front door on `bathroom-check` (also from above, `systems/doors-top/`), a visitor invited over on `calling-out` (a fixed camera on the lot's entry). Systems have no model of their own: M is `n/a`, the models are in their ledgers.

## Recipes (9)

Steps from `base.json` `recipes` (station, clip, minutes).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `meal` Make a meal: ingredients (fetch, 3) → prep (chop, 10, optional) → stove/oven/microwave (stir, 20, optional) (minutes) | n/a | I-systems-07 | I-systems-04, I-systems-06 | Fridge (door open, the hands in it) → counter (board, chopping) → hob (pan on a lit burner) → the pot left on the hob → eaten at the island → dishes: a real kitchen flow. I-systems-07 S3 code (the prep bowl in the other hand while stirring); I-systems-06 S2 renderer-table (at an oven or microwave the step is the same stirring of a pan; not shot at a microwave); I-systems-04 S2 code (seeded: a child may cook) | `systems/sheets/cook-meal-1.png`, `systems/sheets/cook-eat.png` |
| `gourmet` Cook a gourmet meal: ingredients (fetch, 3) → prep (chop, 20) → stove (stir, 45) (minutes) | n/a | I-systems-07 | OK | As `meal` at the stove; the dirty pot scraped and washed after. I-systems-07 S3 code | `systems/sheets/cook-gourmet-1.png`, `systems/sheets/cook-gourmet-2.png` |
| `tastingMenu` Cook a tasting menu: ingredients (fetch, 3) → prep (chop, 30) → stove (stir, 45) (minutes) | n/a | I-systems-07 | OK | Prep and cook at the one chef's island, then the tasting board served there. I-systems-07 S3 code | `systems/sheets/cook-tastingMenu-1.png` |
| `bbq` Grill dinner: ingredients (fetch, 3) → prep (chop, 10, optional) → grill (flip, 25) (minutes) | n/a | I-systems-07 | I-systems-01 | Out to the kettle grill with the bowl, food turned on the grate; the platter served on the kitchen counter. I-systems-07 S3 code (flipping with the bowl in hand); I-systems-01 S2 renderer-table (confirmed: the grill station's chopping board drawn on its grate) | `systems/sheets/cook-bbq-1.png`, `outdoor/play/chunk5-kitchen/grillStation-cook-01.png` |
| `bbqFeast` Grill a barbecue feast: ingredients (fetch, 3) → prep (chop, 20) → grill (flip, 50) (minutes) | n/a | I-systems-07 | I-systems-01 | The `bbq` steps, longer (not run: the same stations and clips); at the grill station I-systems-01 (`outdoor/play/chunk5-kitchen/grillStation-feast-01.png`) | `outdoor/sheets/obj-grillStation.png` |
| `pizza` Bake a wood-fired pizza: ingredients (fetch, 3) → prep (chop, 15, optional) → oven (stir, 25) (minutes) | n/a | I-systems-06 | I-systems-06 | I-systems-06 S2 renderer-table (the pizza is "baked" stirring a pan at the oven's side); the pizza then served on the kitchen counter, 6 m off, fine | `systems/sheets/cook-pizza-1.png` |
| `bakedTreats` Bake treats: ingredients (fetch, 3) → prep (chop, 15, optional) → oven (stir, 35) (minutes) | n/a | I-systems-06 | I-systems-06 | I-systems-06 S2 renderer-table (at the stove the oven door drops open and the cook stirs a pan on the hob for 35 minutes, the bowl of batter in the other hand); the tray served on the counter | `systems/sheets/cook-bakedTreats-1.png`, `systems/sheets/cook-ovens.png` |
| `pie` Bake a family pie: ingredients (fetch, 3) → prep (chop, 15, optional) → oven (stir, 30) (minutes) | n/a | I-systems-06 | I-systems-06 | I-systems-06 S2 renderer-table (the neighbourly oven: the pastry rolled on its worktop, then a pan stirred on its top) | `systems/sheets/cook-pie-1.png` |
| `sandwich` Make a sandwich: ingredients (fetch, 2) → prep (chop, 7) (minutes) | n/a | OK | OK | Fridge → counter → eaten at the dining table with the plate drawn | `systems/sheets/cook-sandwich-1.png`, `held/inhand/sandwich-04.png` |

Durations are by design (§9.2); none is an outlier against the others (the feasts and gourmet dishes 2–3× a plain meal).

## Cooking flow (7)

`crates/sim-core/src/cooking.rs`; props `STATION_PROPS` in `characters.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Props on the station (board, pan, grill food) | n/a | OK | I-systems-01 | I-systems-01 S2 renderer-table (confirmed); the board on a counter and the pan on a hob are right (`cook-meal-1`), the pan on an oven's or a pizza oven's top is I-systems-06 | `systems/sheets/cook-meal-1.png`, `outdoor/play/chunk5-kitchen/grillStation-cook-01.png` |
| Serving out (`serving`) | n/a | OK | OK | The dish put down on a counter by the last station (the pot on the hob, the tray, platter and pizza on the counter, the pie dish on the oven) | `systems/sheets/cook-pizza-1.png`, `systems/sheets/cook-pie-1.png` |
| Eating at a table or seat | n/a | OK | OK | At the island on a bar stool and at the dining table on a pulled-in chair, `eat` with the hands at the mouth (no cutlery: I-held-01, held ledger); the white plate on the white island barely shows, only the food on it (`cook-eat.png`) | `systems/sheets/cook-eat.png` |
| Scraping a plate | n/a | OK | OK | A minute at the bin (`clean`), then to the sink with the stack | `held/inhand/gourmet-08.png`, `held/inhand/bbq-09.png` |
| Washing up (sink, dishwasher, bin) | n/a | OK | I-systems-03 | At the sink (`wash`); I-systems-03 S3 decision (seeded; not checked: no home without a sink and a bin was set up) | `held/inhand/meal-08.png` |
| Putting clean things away (`putAwayAt`) | n/a | OK | OK | `fetch` at a cupboard or counter, two minutes | `held/inhand/gourmet-11.png` |
| Leftovers and spoiling | n/a | n/a | OK | 8 hours on a counter, 72 in the fridge (`cooking.spoilHours`, `storedSpoilHours`), spoilt food greyed (`items.ts:151`); waiting cooks: I-systems-10 S2 code (two cooks stand inside each other at the fridge) | `systems/sheets/baby.png` |

## Baby care (7)

The crib (`base.json`), `babyWakes` and the cuddle lift (`characters.ts`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `lie` Lie in the crib (sleep, 720 min) | n/a | OK | I-kids-03 | The baby lies on the mattress, asleep or awake (pilot); I-kids-03 S1 code (kids ledger: an adult told to lies in it) | `kids/sheets/obj-crib.png`, `systems/sheets/baby.png` |
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
| Lamp fades | n/a | OK | n/a | 1.2 s (`BabylonRenderer.ts:191` `LIGHTS_OUT_FADE`), the shade's glow with it | `systems/sheets/lamps.png` |
| Light coming from the shade | n/a | n/a | OK | The shade glows and the pool of light falls round the base; each lamp is judged in the lighting ledger | `lighting/night/extra/lamp-switched-on.png` |

## House doors (3)

`BabylonRenderer.ts` `swingDoors` (the door models are in [houses.md](houses.md)).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Front door: who opens it and when | n/a | A-houses-01 | OK | Shut; swings in as she walks up to it, open while she passes, shut behind her (`bathroom-check`); a visitor knocking stands still, so it stays shut until the host walks out. A-houses-01 S3 code (houses ledger, confirmed: by nearness, no hand) | `systems/sheets/doors.png` |
| Bathroom door: shut while in use | n/a | A-houses-01 | I-systems-05 | Opens as she walks in, shuts behind her and stays shut on the toilet and at the basin (from above, `systems/sheets/doors-top.png`). I-systems-05 S3 code (confirmed and widened: the drawing only; every other inside door stands open always) | `systems/sheets/doors-top.png` |
| Garden door to the patio | n/a | A-houses-01 | OK | A door with outdoors on one side is a "front" door (`house.ts:293`, `:760`): shut, opening as someone passes, like the front door (code read; not shot apart) | `systems/sheets/doors.png` |

## Objects that open (6)

Content `when` / `leaves` (crops below); the objects themselves are in their own ledgers.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Open state: Futon bed (`bedFuton`): sleep, nap, unfold, foldAway | n/a | A-bedroom-03 | I-bedroom-03 | Going to bed unfolds it, the last one up folds it away (tests `open_objects.rs`); nobody folds it over a sleeper. A-bedroom-03 S2 code, I-bedroom-03 S3 content (bedroom ledger: in at the foot; no sitting on it folded) | `bedroom/sheets/obj-bedFuton.png` |
| Open state: Cedar hot tub (`hotTubCedar`): soak, coverOn | n/a | OK | I-bathroom-03 | The cover off as they get in (stood up behind the tub), the last one out puts it back (`closeAfter`); I-bathroom-03 S2 code (bathroom ledger, widened there: soaking in clothes) | `outdoor/sheets/obj-hotTubCedar.png` |
| Open state: Sofa bed (`sofaBed`): relax, doze, pullOut, foldAway | n/a | OK | OK | Relax only folded; pulled out for a doze and put away after; nobody opens it under someone sitting (tests). The sofa bed's own shots: living ledger | `living/play/chunk2/sofaBed-pullOut-01.png`, `living/play/chunk2/sofaBed-doze-01.png` |
| Open state: Fold-down wall bed (`wallBed`): sleep, nap, foldDown, foldUp | n/a | OK | OK | Folded down from its foot, up by the last one up (tests); nobody folds it up over a sleeper | `living/sheets/obj-wallBed.png` |
| Open state: Washing line (`clothesline`): washing, takeIn | n/a | A-systems-02 | OK | Hung out, it stays out until someone takes it in (autonomous); A-systems-02 S3 motion (all the washing on the line in 1.5 s at the start of a 20-minute use) | `outdoor/sheets/obj-clothesline.png` |
| Resizing furniture with moving parts | n/a | A-systems-01 | n/a | A-systems-01 S3 code (seeded; not checked: resizing is off) | |

## Crops (4)

Content `grows`; `harvest` from ripe to picked.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Planting | n/a | n/a | I-systems-08 | I-systems-08 S3 decision (nobody plants: a picked bed plants itself half a day later) | `garden/sheets/obj-garden.vegPatch.png` |
| Watering | n/a | OK | I-systems-08 | "Weed and water" / "Tie up and water" (`tend`) changes nothing in the crop: I-systems-08; the clip and pose at each crop: I-garden-04 S2 renderer-table (garden ledger); no can: I-held-01 | `garden/play/c3/garden.tomatoes-tend-01.png` |
| Growing from planted to ripe | n/a | OK | OK | Planted, growing and ripe stages over 2–4 days (`grows.days`), shown by the model's growth clip; the strawberries' planted stage is M-garden-07 (garden ledger) | `garden/sheets/obj-garden.strawberries.png` |
| Harvesting and picking | n/a | I-garden-03 | I-garden-10 | I-garden-03 S2 code (garden ledger: the crop is gone as the harvest starts); I-garden-04 S2 renderer-table; I-garden-10 S3 decision (eaten on the spot) | `garden/extra/harvest/s-garden.vegPatch-harvest.png` |

## Visitors (5)

`base.json` `visits` (`door`: knock, wait 10 min, greet with `chat`; farewell).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Knocking (`knock`) | n/a | I-systems-09 | I-systems-09 | I-systems-09 S3 code (the visitor stands 1.5 m before the door, turned to the middle of the lot, and knocks on the wall beside it) | `systems/sheets/knock.png` |
| Answering the door, or turning away | n/a | OK | OK | The nearest free grown-up drops what they're doing (R2 leaves a meal in the kitchen), walks through the house and out of the door to the visitor (`life.rs:669` `door_host`); turned away at night or by someone who can't stand them (`answer`, code read; not shot) | `systems/sheets/knock.png` |
| Greeting at the door | n/a | OK | OK | A chat on the doorstep, 1 m apart, facing each other (`visits.door.greet`) | `systems/sheets/knock.png`, `social/sheets/visit.png` |
| What visitors use | n/a | n/a | I-systems-05 | I-systems-05 S3 code (seeded: no private flag) | |
| Farewell (`goodbyeHug`, `goodbye`) | n/a | OK | OK | A hug for a friend, a wave for others, then home (`goodbye-hug`, `goodbye-wave`; social ledger rows) | `social/sheets/goodbye.png` |

## Use length (1)

Sim rules over every use.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| A use ends when every need it raises is full | n/a | n/a | I-systems-02 | I-systems-02 S3 decision (confirmed: every review state sets needs to 0.5 for this; with them full in `review-baby` care ended once the baby's needs were up) | `systems/sheets/baby.png` |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-systems-01 — Moving parts don't follow a resize, so furniture can't be resized

- **Severity:** S3
- **Rubric:** A10, M4
- **Fix class:** code
- **Where:** `web/src/game/buildmode.ts` (resize handles)
- **Evidence:** Furniture resizing is off in 0.3.0. Something resized in an older save keeps its size, and its moving parts don't follow it; "Reset size" takes it back. Doors and windows still resize.
- **Status:** not checked (resizing furniture is off in 0.3.0, and the review states have nothing resized)
- **Source:** `CHANGELOG.md` 0.3.0 Changes ("Furniture can't be resized for now")

### I-systems-01 — Cooking props are drawn at a station's middle: on the gas grill station, on the grate

- **Severity:** S2
- **Rubric:** I13
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:546` (`STATION_PROPS`) and `:2097` (`writeTool`)
- **Evidence:** The chopping board and the food are drawn at the station's middle, a little towards the cook. On `grillStation` that is the grate, so the chopping happens on the grill. Cross-cutting wave: `outdoor/play/chunk5-kitchen/grillStation-cook-01.png` and `grillStation-feast-01.png` (the outdoor wave's runs): the orange board lies over the burning grate while the station's own wooden prep board, at its right end, stays bare. The kettle grill (`bbq`) and kitchen counters are right (`systems/sheets/cook-bbq-1.png`, `cook-meal-1.png`).
- **Status:** confirmed
- **Source:** `docs/objects/open.md` Still open › Rendering; `plan/authenticity-review.md` §3.3 (I13); wave: cross-cutting

### I-systems-02 — Uses end once every need they raise is full

- **Severity:** S3
- **Rubric:** I4
- **Fix class:** decision
- **Where:** `crates/sim-core/src/world.rs`
- **Evidence:** By design. It makes uses (and their clips) short and easy to miss in checks: a reviewer should lower the needs first. Cross-cutting wave: every `review-*` state sets the needs to 0.5 for this reason; on `review-baby` (the grown-ups' needs full) the feed and change ended as soon as the baby's needs were up.
- **Status:** confirmed
- **Source:** `docs/objects/open.md` Still open › Content and sim

### I-systems-03 — With neither a sink nor a bin, dirty plates pile up on the floor

- **Severity:** S3
- **Rubric:** I13
- **Fix class:** decision
- **Where:** `crates/sim-core/src/cooking.rs`
- **Evidence:** R2-12's rest: with a bin, clearing away ends at the bin; with neither, plates and spoiled food pile up and are scanned on every decision to eat.
- **Status:** not checked (every review state has a sink and a bin)
- **Source:** `docs/release-0.2.0-review.md` §4 (R2-12 rest), target v0.3

### I-systems-04 — Nothing stops a child using a stove

> Cross-cutting wave: confirmed and widened to the socials: any resident may start a fist fight with a child.

- **Severity:** S2
- **Rubric:** I6
- **Fix class:** code
- **Where:** `crates/sim-core/src/content.rs` (interactions have no age rule); `crates/sim-core/src/conversation.rs:63` (`social_allowed`: the relationship, and for romance both grown up; nothing for a fight)
- **Evidence:** A known gap named in the rubric. Hard age rules are new behaviour: filed as `code`, planned separately (§9, decision 3). Cross-cutting wave: the one age rule that exists outside free will is the phone's (`phone.fromStage`: the child is refused, "too young for a phone", `social/scenes/scenes.log`). A teen starts a fight with a child (`social/sheets/fights.png`, A-social-03 for the punches), and an adult could the same (only the relationship is checked). The kids' chair takes any stage (`kids/sheets/age-kidsChair.png`), the residents ledger's seat fit row.
- **Status:** confirmed and widened
- **Source:** `plan/authenticity-review.md` §3.3 (I6) and §9 (decision 3); wave: cross-cutting

### I-systems-05 — No privacy: the bathroom door is renderer-only

> Cross-cutting wave: confirmed and widened. Every inside door but a bathroom's stands open all the time.

- **Severity:** S3
- **Rubric:** I6
- **Fix class:** code
- **Where:** `web/src/render/babylon/BabylonRenderer.ts:3778` (`swingDoors`), `web/src/render/babylon/house.ts:757-760` (only front and bathroom doors move: "the others stand open"), `crates/sim-core/src/content.rs`
- **Evidence:** No private flag exists, so nothing keeps a visitor from using a private thing, and the bathroom door shuts only in the drawing. A privacy flag is new behaviour: filed as `code`, planned separately (§9, decision 3). Cross-cutting wave: `systems/sheets/doors-top.png` (`bathroom-check`, from above): the bathroom door swings open as she walks in, shuts behind her and stays shut through the toilet and the basin; nothing in the sim knows. A bedroom's door, or any other inside one, stands open day and night, a sleeper's too.
- **Status:** confirmed and widened
- **Source:** `plan/authenticity-review.md` §3.3 (I6) and §9 (decision 3); wave: cross-cutting

### I-systems-06 — Every oven step is stirring a pan: pizzas, pies and cakes are "baked" on the hob

- **Severity:** S2
- **Rubric:** I13, I3
- **Fix class:** renderer-table
- **Where:** `web/public/content/base.json:4396`, `:4440`, `:4483` (`pizza`, `bakedTreats`, `pie`: the oven step's `anim: stir`) and `:4176` (`meal`'s cook step at `stove`, `oven` or `microwave`, `stir`); `web/src/render/babylon/characters.ts:540` (`ACTIONS.stir`: `stir`, `interact`), `:546` (`STATION_PROPS.stir`: the pan)
- **Evidence:** `systems/sheets/cook-ovens.png` (the kitchen wave's `kitchen/play/c5/stoveGas-bake-01.png`, `stoveElectric-bake-01.png`, `wallOven-bake-01.png`): baking at a stove the oven door drops open and stays open while the cook stirs a pan on the lit hob for 35 minutes, the prep bowl of batter in the other hand; at the oven tower, with no hob, she stirs at the open oven door. `systems/sheets/cook-pizza-1.png` (`held/inhand/pizza-04.png`): the pizza "baked" stirring a pan at the wood-fired oven's side; `cook-pie-1.png` the same at the neighbourly oven. Baking is putting the tray in, shutting the door and waiting (a check through the glass), then taking it out: an oven step of its own (`anim: bake` with a tray prop, the door open at the start and end).
- **Status:** open
- **Source:** wave: cross-cutting (held, kitchen and packs waves' runs)

### I-systems-07 — The prep bowl stays in the hand through the cooking step

- **Severity:** S3
- **Rubric:** I13, I7
- **Fix class:** code
- **Where:** `crates/sim-core/src/cooking.rs:1501` (`step_done`: `held` is set to what the step `holds` and kept until the next step ends), the renderer drawing `held` in the hand whatever the step (`web/src/render/babylon/characters.ts`, `writeHeld`)
- **Evidence:** `systems/sheets/cook-meal-1.png`, `cook-bbq-1.png`, `cook-bakedTreats-1.png` (`held/inhand/meal-03.png`, `meal-04.png`, `bbq-05.png`, `bakedTreats-04.png`, `pie-04.png`, `tastingMenu-04.png`; the held wave's "Seen"): the cook stirs the pan or turns the food on the grill one-handed for 20–50 minutes with the bowl of chopped food held in the other. The bowl is tipped into the pan and put down as the step starts.
- **Status:** open
- **Source:** wave: cross-cutting (held wave's runs)

### I-systems-08 — Nobody sows: a picked bed plants itself, and tending changes nothing

- **Severity:** S3
- **Rubric:** I15, I8
- **Fix class:** decision
- **Where:** `crates/sim-core/src/world.rs:2765-2786` (`grow`: growth by time alone, the fallow ground "planted afresh" at 0), `crates/sim-core/src/content.rs:2064-2074` (`grows.fallow`, half a day by default); `web/public/content/garden.json` (the crops' `tend`: fun, gardening skill, no growth)
- **Evidence:** A harvested crop lies fallow for half a game day and then is planted again with nobody there; it ripens in its 2–4 days whether anyone weeds and waters it or not ("Weed and water", "Tie up and water" give fun and the gardening skill only). In life a bed is sown or planted out by hand, and a crop not watered does worse. For the owner: keep it (a game shortcut), or a "Sow" use on fallow ground and `tend` speeding growth, which is new behaviour (`code`).
- **Status:** open
- **Source:** wave: cross-cutting (code read, with the garden wave's crop shots)

### I-systems-09 — Visitors knock facing the middle of the lot, 1.5 m short of the door

- **Severity:** S3
- **Rubric:** I15, A1
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:4917-4922` (`TaskKind::Visit` arriving: the yaw towards the plot's centre, at the plot's `entry`); `web/src/render/babylon/characters.ts:535` (`ACTIONS.knock`: `interact`, `idle`)
- **Evidence:** `systems/sheets/knock.png` (`calling-out`, R10 invited over; `systems/scenes/scenes.log`): the visitor stops at the lot's entry (25.5, 27.5), 1.5 m in front of the front door (25.5, 26), turned 0.49 rad off it towards the middle of the lot, and knocks with the reach clip on the wall beside the doorway. A knock is at the door, an arm's length from it, facing it. The host coming out and the greeting are right.
- **Status:** open
- **Source:** wave: cross-cutting

### I-systems-10 — A cook waiting for a busy station stands where they are, and the next one stands inside them

- **Severity:** S2
- **Rubric:** I13, A1
- **Fix class:** code
- **Where:** `crates/sim-core/src/cooking.rs:760-763` (`Found::Busy`: `Phase::Queued` where the cook stands, the tile of the station they just used)
- **Evidence:** `systems/sheets/baby.png` (`review-baby`, `systems/scenes/baby-09.png` … `baby-13.png`, positions in `systems/scenes/scenes.log`): Cal got his ingredients and, the counter busy (Ben chopping), waits at the fridge's spot (14.5, 8.5); Gus comes to the free fridge and stands on the same spot: the two bodies one inside the other for a minute and more, until the counter frees. A waiting cook steps aside (a free tile next to the next station).
- **Status:** open
- **Source:** wave: cross-cutting

### A-systems-02 — The washing appears on the line in 1.5 s at the start of a 20-minute hang-out

- **Severity:** S3
- **Rubric:** A10, A9
- **Fix class:** motion
- **Where:** `web/public/assets/packs/yard/manifest.json:361` (`clothesline` `motions.washing`, 1.5 s, played as the use starts; `takeIn` plays it back)
- **Evidence:** `outdoor/play/chunk3/play.jsonl` (the outdoor wave's run): the `washing` clip runs from 0.18 to its full 1.5 s by the first shot, so every piece is pegged up before the resident has lifted one, and the basket stays full beside them (`outdoor/sheets/obj-clothesline.png`); taking it in, the line is bare in the first second. Hanging washing out goes piece by piece over the use (a `hold` spread over it, or the pieces in turn).
- **Status:** open
- **Source:** wave: cross-cutting (the outdoor wave's run)
