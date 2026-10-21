# Systems: authenticity ledger

What it covers: the behaviour that spans objects: cooking and its 9 recipes over stations (`cooking.rs`, `base.json` `recipes`, `cooking`), baby care at the crib, lamps (`lights.rs`), house doors, objects that open and stay open (`when`, `leaves`), crops (`grows`) and visitors knocking (`base.json` `visits`). Each object's own uses are judged in its own ledger; these rows judge the system. Rubric I13–I15 in particular.

- **Evidence:** `bench-results/review/systems/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Never looked at before** (`docs/objects/open.md` "Not looked at"): a meal from the fridge eaten at a pulled-in chair (eating was checked with the café's cake).

## Recipes (9)

Steps from `base.json` `recipes` (station, clip, minutes).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `meal` Make a meal: ingredients (fetch, 3) → prep (chop, 10, optional) → stove/oven/microwave (stir, 20, optional) (minutes) | — | — | — | I-systems-04 S2 code (seeded) | |
| `gourmet` Cook a gourmet meal: ingredients (fetch, 3) → prep (chop, 20) → stove (stir, 45) (minutes) | — | — | — | | |
| `tastingMenu` Cook a tasting menu: ingredients (fetch, 3) → prep (chop, 30) → stove (stir, 45) (minutes) | — | — | — | | |
| `bbq` Grill dinner: ingredients (fetch, 3) → prep (chop, 10, optional) → grill (flip, 25) (minutes) | — | — | — | | |
| `bbqFeast` Grill a barbecue feast: ingredients (fetch, 3) → prep (chop, 20) → grill (flip, 50) (minutes) | — | — | — | | |
| `pizza` Bake a wood-fired pizza: ingredients (fetch, 3) → prep (chop, 15, optional) → oven (stir, 25) (minutes) | — | — | — | | |
| `bakedTreats` Bake treats: ingredients (fetch, 3) → prep (chop, 15, optional) → oven (stir, 35) (minutes) | — | — | — | | |
| `pie` Bake a family pie: ingredients (fetch, 3) → prep (chop, 15, optional) → oven (stir, 30) (minutes) | — | — | — | | |
| `sandwich` Make a sandwich: ingredients (fetch, 2) → prep (chop, 7) (minutes) | — | — | — | | |

## Cooking flow (7)

`crates/sim-core/src/cooking.rs`; props `STATION_PROPS` in `characters.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Props on the station (board, pan, grill food) | — | — | — | I-systems-01 S2 renderer-table (seeded) | |
| Serving out (`serving`) | — | — | — | | |
| Eating at a table or seat | — | — | — | | |
| Scraping a plate | — | — | — | | |
| Washing up (sink, dishwasher, bin) | — | — | — | I-systems-03 S3 decision (seeded) | |
| Putting clean things away (`putAwayAt`) | — | — | — | | |
| Leftovers and spoiling | — | — | — | | |

## Baby care (7)

The crib (`base.json`), `babyWakes` and the cuddle lift (`characters.ts`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `lie` Lie in the crib (sleep, 720 min) | — | — | — | | |
| `feed` Feed the baby (tend, 15 min) | — | — | — | | |
| `change` Change the baby (clean, 10 min) | — | — | — | | |
| `play` Play with the baby (play, 20 min) | — | — | — | | |
| `cuddle` Cuddle the baby (cuddle, 15 min) | — | — | — | | |
| A crying baby wakes a parent | — | — | — | | |
| The cuddle: the baby lifted into the arms | — | — | — | | |

## Lamps (4)

`crates/sim-core/src/lights.rs`; fades in the renderer.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clicking a lamp on or off | — | — | — | | |
| Residents switch a room's lamps on after dark and off as they leave | — | — | — | | |
| Lamp fades | — | — | — | | |
| Light coming from the shade | — | — | — | | |

## House doors (3)

`BabylonRenderer.ts` `swingDoors` (the door models are in [houses.md](houses.md)).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Front door: who opens it and when | — | — | — | | |
| Bathroom door: shut while in use | — | — | — | I-systems-05 S3 code (seeded) | |
| Garden door to the patio | — | — | — | | |

## Objects that open (6)

Content `when` / `leaves` (crops below); the objects themselves are in their own ledgers.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Open state: Futon bed (`bedFuton`): sleep, nap, unfold, foldAway | — | — | — | | |
| Open state: Cedar hot tub (`hotTubCedar`): soak, coverOn | — | — | — | | |
| Open state: Sofa bed (`sofaBed`): relax, doze, pullOut, foldAway | — | — | — | | |
| Open state: Fold-down wall bed (`wallBed`): sleep, nap, foldDown, foldUp | — | — | — | | |
| Open state: Washing line (`clothesline`): washing, takeIn | — | — | — | | |
| Resizing furniture with moving parts | — | — | — | A-systems-01 S3 code (seeded) | |

## Crops (4)

Content `grows`; `harvest` from ripe to picked.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Planting | — | — | — | | |
| Watering | — | — | — | | |
| Growing from planted to ripe | — | — | — | | |
| Harvesting and picking | — | — | — | | |

## Visitors (5)

`base.json` `visits` (`door`: knock, wait 10 min, greet with `chat`; farewell).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Knocking (`knock`) | — | — | — | | |
| Answering the door, or turning away | — | — | — | | |
| Greeting at the door | — | — | — | | |
| What visitors use | — | — | — | I-systems-05 S3 code (seeded) | |
| Farewell (`goodbyeHug`, `goodbye`) | — | — | — | | |

## Use length (1)

Sim rules over every use.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| A use ends when every need it raises is full | — | — | — | I-systems-02 S3 decision (seeded) | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-systems-01 — Moving parts don't follow a resize, so furniture can't be resized

- **Severity:** S3
- **Rubric:** A10, M4
- **Fix class:** code
- **Where:** `web/src/game/buildmode.ts` (resize handles)
- **Evidence:** Furniture resizing is off in 0.3.0. Something resized in an older save keeps its size, and its moving parts don't follow it; "Reset size" takes it back. Doors and windows still resize.
- **Status:** seeded (from `CHANGELOG.md` 0.3.0 Changes)
- **Source:** `CHANGELOG.md` 0.3.0 Changes ("Furniture can't be resized for now")

### I-systems-01 — Cooking props are drawn at a station's middle: on the gas grill station, on the grate

- **Severity:** S2
- **Rubric:** I13
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:630` (`STATION_PROPS`) and `:2182` (`writeTool`)
- **Evidence:** The chopping board and the food are drawn at the station's middle, a little towards the cook. On `grillStation` that is the grate, so the chopping happens on the grill.
- **Status:** seeded (from `docs/objects/open.md` L133)
- **Source:** `docs/objects/open.md` Still open › Rendering; `plan/authenticity-review.md` §3.3 (I13)

### I-systems-02 — Uses end once every need they raise is full

- **Severity:** S3
- **Rubric:** I4
- **Fix class:** decision
- **Where:** `crates/sim-core/src/world.rs`
- **Evidence:** By design. It makes uses (and their clips) short and easy to miss in checks: a reviewer should lower the needs first.
- **Status:** seeded (from `docs/objects/open.md` L146)
- **Source:** `docs/objects/open.md` Still open › Content and sim

### I-systems-03 — With neither a sink nor a bin, dirty plates pile up on the floor

- **Severity:** S3
- **Rubric:** I13
- **Fix class:** decision
- **Where:** `crates/sim-core/src/cooking.rs`
- **Evidence:** R2-12's rest: with a bin, clearing away ends at the bin; with neither, plates and spoiled food pile up and are scanned on every decision to eat.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 R2-12 (rest))
- **Source:** `docs/release-0.2.0-review.md` §4 (R2-12 rest), target v0.3

### I-systems-04 — Nothing stops a child using a stove

- **Severity:** S2
- **Rubric:** I6
- **Fix class:** code
- **Where:** `crates/sim-core/src/content.rs` (interactions have no age rule)
- **Evidence:** A known gap named in the rubric. Hard age rules are new behaviour: filed as `code`, planned separately (§9, decision 3).
- **Status:** seeded (from `plan/authenticity-review.md` §3.3 I6)
- **Source:** `plan/authenticity-review.md` §3.3 (I6) and §9 (decision 3)

### I-systems-05 — No privacy: the bathroom door is renderer-only

- **Severity:** S3
- **Rubric:** I6
- **Fix class:** code
- **Where:** `web/src/render/babylon/BabylonRenderer.ts:3778` (`swingDoors`), `crates/sim-core/src/content.rs`
- **Evidence:** No private flag exists, so nothing keeps a visitor from using a private thing, and the bathroom door shuts only in the drawing. A privacy flag is new behaviour: filed as `code`, planned separately (§9, decision 3).
- **Status:** seeded (from `plan/authenticity-review.md` §3.3 I6)
- **Source:** `plan/authenticity-review.md` §3.3 (I6) and §9 (decision 3)
