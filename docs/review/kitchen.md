# Kitchen: authenticity ledger

What it covers: the 72 objects of [docs/objects/kitchen.md](../objects/kitchen.md), in its groups and order.

- **Evidence:** `bench-results/review/kitchen/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## Kitchen sinks (2)

Family: `tools/art/blender/families/kitchen.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `farmhouseSink` Farmhouse sink | — | — | — | | |
| `counterSinkDouble` Double sink | — | — | — | | |

## K1. Fridges (7)

Family: `tools/art/blender/families/kitchen.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `fridge` Fridge | OK | OK | I-kitchen-02 | I-kitchen-01 S3 content (seeded; confirmed fixed on `fridge`); I-kitchen-02 S3 content | `sheets/obj-fridge.png`, `play/fridge-*.png`, `clips/clips-fridge-all-t*.png`, `play-child/fridge-*.png` |
| `fridgeMini` Mini fridge | — | — | — | | |
| `fridgeBuiltIn` Built-in fridge | — | — | — | I-kitchen-01 S3 content (seeded) | |
| `fridgeFrenchDoor` French-door fridge | — | — | — | I-kitchen-01 S3 content (seeded) | |
| `fridgeRetro` Retro fridge | — | — | — | | |
| `fridgeMiniRetro` Retro mini fridge | — | — | — | | |
| `wineFridge` Wine fridge | — | — | — | | |

## K2. Cookers (6)

Family: `tools/art/blender/families/kitchen_cooking.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `stoveElectric` Electric cooker | — | — | — | | |
| `stoveGas` Gas range | — | — | — | | |
| `wallOven` Oven tower | — | — | — | | |
| `rangeCooker` Range cooker | — | — | — | | |
| `cooktopCounter` Hob counter | — | — | — | | |
| `microwave` Microwave counter | — | — | — | | |

## K3. Small appliances and dishwashers (4)

Family: `tools/art/blender/families/kitchen_cooking.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `coffeeMachine` Coffee machine | — | — | — | | |
| `breakfastCounter` Breakfast counter | — | — | — | | |
| `dishwasher` Dishwasher | — | — | — | | |
| `dishwasherCompact` Stainless dishwasher | — | — | — | | |

## K4. Base counters (8)

Family: `tools/art/blender/families/kitchen_counters.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `counter` Kitchen counter | — | — | — | | |
| `counterCorner` Corner counter | — | — | — | | |
| `counterEnd` Counter end with shelves | — | — | — | | |
| `counterCornerOuter` Outer corner counter | — | — | — | | |
| `counterDrawers` Drawer counter | — | — | — | | |
| `counterOpenShelf` Open-shelf counter | — | — | — | | |
| `butcherBlock` Butcher block | — | — | — | | |
| `counterMarble` Marble counter | — | — | — | | |

## K5. Tall and wall storage (11)

Family: `tools/art/blender/families/kitchen_counters.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `pantry` Pantry cupboard | — | — | — | | |
| `pantryTall` Tall larder | — | — | — | | |
| `wallCabinet` Wall cabinets | — | — | — | | |
| `wallCabinetGlass` Glazed wall cabinet | — | — | — | | |
| `wallCabinetCorner` Wall cabinet with end shelves | — | — | — | | |
| `rangeHood` Cooker hood | — | — | — | | |
| `spiceRack` Spice rack | — | — | — | | |
| `potRail` Pot rail | — | — | — | | |
| `kitchenShelfWall` Kitchen wall shelves | — | — | — | | |
| `trashCan` Pedal bin | — | — | — | | |
| `recyclingBins` Recycling bins | — | — | — | | |

## K6. Islands and bars (5)

Family: `tools/art/blender/families/kitchen_counters.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `kitchenIsland` Kitchen island | — | — | — | | |
| `islandSmall` Small island | — | — | — | | |
| `islandLong` Long island | — | — | — | | |
| `barSegment` Bar counter | — | — | — | | |
| `barEnd` Bar end | — | — | — | | |

## K7. Dining tables (13)

Family: `tools/art/blender/families/dining.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `diningTable` Dining table | M-kitchen-01 | OK | I-held-01 | M-kitchen-01 S2 model; I-held-01 S2 code (seeded; held ledger: no cup) | `sheets/obj-diningTable.png`, `gallery-all/gallery-diningTable.png`, `scenes/dining-coffee-*.png`, `sheets/seat-diningM.png`, `sheets/seat-diningF.png` |
| `bistroSet` Bistro table | — | — | — | | |
| `diningGlass` Glass dining table | — | — | — | | |
| `diningOak` Farmhouse dining table | — | — | — | | |
| `tableSquare2` Square table for two | — | — | — | | |
| `tableBistroRound` Round bistro table | — | — | — | | |
| `tableDining4` Dining table for four | — | — | — | | |
| `tableDining6` Dining table for six | — | — | — | | |
| `tableBanquet8` Banquet table | — | — | — | | |
| `tableRound4` Round pedestal table | — | — | — | | |
| `tableSquare8` Square table for eight | — | — | — | | |
| `tableFarmhouse6` Farmhouse table | — | — | — | | |
| `tableBarHigh` High café table | — | — | — | | |

## K8. Chairs and stools (12)

Family: `tools/art/blender/families/dining.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `barStool` Bar stool | — | — | — | | |
| `stoolCounter` Counter stool | — | — | — | | |
| `stoolBarHigh` High bar stool | — | — | — | | |
| `diningChair` Dining chair | — | — | — | | |
| `chairWood` Wooden chair | — | — | — | | |
| `chairUpholstered` Upholstered chair | — | — | — | | |
| `chairModern` Moulded chair | — | — | — | | |
| `chairWindsor` Windsor chair | — | — | — | | |
| `chairBentwood` Bentwood chair | — | — | — | | |
| `chairFolding` Folding chair | — | — | — | | |
| `chairArm` Carver chair | — | — | — | | |
| `benchDining` Dining bench | — | — | — | | |

## K9. Dining storage (4)

Family: `tools/art/blender/families/dining.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sideboard` Walnut sideboard | — | — | — | | |
| `buffet` Painted buffet | — | — | — | | |
| `chinaCabinet` China cabinet | — | — | — | | |
| `barCart` Bar cart | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows; durations are judged by §9 decision 2 (short by design, outliers over 3× the tag median only).

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `fridge` | get a snack or a cold drink, take out ingredients, put the shopping away | snack (2), ice cream (2), make a meal (cook), gourmet meal (cook, needs Cooking) | cold drink missing (I-kitchen-02); the shopping is the sim's |
| `diningTable` | eat a meal, have a coffee, homework or a puzzle | have a coffee (20, needs a coffee machine on the lot); meals are carried to it and eaten there | OK |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### I-kitchen-01 — The freezer doors had a clip but no use

- **Severity:** S3
- **Rubric:** I1, I7
- **Fix class:** content
- **Where:** `web/public/content/base.json:2363`, `web/public/content/furniture.json:198`, `web/public/content/catalog2/kitchen.json:568`
- **Evidence:** Fixed in 0.3.0 by "Get some ice cream" (`frozenTreat`, CHANGELOG 0.3.0 Objects). Confirm that `freezer_open` plays for `frozenTreat` only, on all three fridges, and close.
  Pilot, `fridge`: `frozenTreat` plays `freezer_open` alone and `fridge_open` never moves (play.jsonl: `fridge_open` events `[]`); `snack` and `meal` play `fridge_open` alone (`play/fridge-frozenTreat-01.png`, `play/fridge-snack-01.png`). The doors swing 118° (freezer) and 102° (fridge) about the left edge, inside the reference 90–125° (`kitchen.py:71-72`). The motions' `drink` key names no use of `fridge` (T4 flag): harmless, it is the shared motions table that `fridgeBuiltIn`'s `coldDrink` uses (`kitchen.py:720`).
- **Status:** confirmed fixed on `fridge` (pilot); `fridgeBuiltIn` and `catalog2/kitchen.json:568` left for the kitchen pass
- **Source:** `plan/release-0.3.0.md` §1.4; `docs/objects/open.md` Still open › Content and sim; `CHANGELOG.md` 0.3.0 Objects; pilot (`check.mjs play --state review-stage`)

### M-kitchen-01 — The base dining table is a 38 cm deep plank with both diners side by side

- **Severity:** S2
- **Rubric:** M1, M3, M6, I2
- **Fix class:** model
- **Where:** `tools/art/blender/families/dining.py:259` (`diningTable`: the top 1.62 × 0.38 at `y = 0.765`, both chairs behind it at `z = -0.2`); `web/public/content/base.json:3327` (`surface.size [1.62, 0.38]`, `slots 2`)
- **Evidence:** `gallery-all/gallery-diningTable.png`, `sheets/seat-diningM.png`, `scenes/dining-coffee-02.png`. The top is 0.38 m deep (a dining table is 0.75–1.0 m, `reference.json` `diningTable4`), with two chairs side by side along one long edge and nothing across: it reads as a console or a breakfast bar, not a table people eat at facing each other. Its top is 0.765 m, 5 mm over the 0.71–0.76 range (inside measuring noise, but the strict rule says out). The sizes table's "H 1.03 > 0.86" is the chair backs, not the table (the kind's height range is for a table alone).
- **Status:** open
- **Source:** pilot

### I-kitchen-02 — The base set lacks the second uses its siblings have

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/base.json:2363` (`fridge`: no `coldDrink`), `:2580` (`sofa`: no `doze`), `:2704` (`armchair`: no `read`), `:2449` (`sink`: no `brushTeeth`)
- **Evidence:** Every other full fridge has "Get a cold drink" (`fridgeBuiltIn`, `fridgeFrenchDoor`, `fridgeRetro`); `loveseat`, `sofaThree`, `sofaDesign`, `sofaThreeSeat` have `doze`; `armchairLeather` and `armchairWingback` have `read`; `mirrorCabinet` has "Brush your teeth". The starter-house versions are the plainest in use, though they are what every new household has. The real uses (`reference.json`: fridge "get a snack or a drink", sofa "doze off", armchair "read", basin "brush teeth") are listed in the I1 table of each ledger.
- **Status:** open
- **Source:** pilot (`cargo review uses` tables against `reference.json` `uses`)
