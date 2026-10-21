# Held items and tools: authenticity ledger

What it covers: the 20 held models (`item.*`, `tool.*`; family `tools/art/blender/families/held.py`) and the 25 items in `base.json` `items` that use them. Things residents hold that are parts of objects (the barbell, dumbbells, guitar) are in their object's ledger. Cooking steps are in [systems.md](systems.md).

- **Evidence:** `bench-results/review/held/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Never looked at before** (`docs/objects/open.md` "Not looked at"): the held things that no recipe in the test house makes: the tasting board, pizza, pie, slices, sandwich and grill platter.

## Held models (20)

`web/public/assets/packs/held/manifest.json`; drawn by `web/src/render/babylon/items.ts` and `characters.ts` (`STATION_PROPS` for the tools).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `item.ingredients` | — | — | — | | |
| `item.prepBowl` | — | — | — | | |
| `item.mealPot` | — | — | — | | |
| `item.potDirty` | — | — | — | | |
| `item.tastingBoard` | — | — | — | | |
| `item.grillPlatter` | — | — | — | | |
| `item.platterDirty` | — | — | — | | |
| `item.pizza` | — | — | — | | |
| `item.bakingTray` | — | — | — | | |
| `item.trayDirty` | — | — | — | | |
| `item.pieDish` | — | — | — | | |
| `item.mealPlate` | — | — | — | | |
| `item.pizzaSlice` | — | — | — | | |
| `item.treatPlate` | — | — | — | | |
| `item.pieSlice` | — | — | — | | |
| `item.sandwich` | — | — | — | | |
| `item.plateDirty` | — | — | — | | |
| `tool.board` | — | — | — | | |
| `tool.pan` | — | — | — | | |
| `tool.grillFood` | — | — | — | | |

## Items (25)

`base.json` `items`: what each is, its model and its grip (`hold`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `ingredients` Ingredients (model `item.ingredients`, both hands) | — | — | — | | |
| `prepBowl` Prepped bowl (model `item.prepBowl`, both hands) | — | — | — | | |
| `mealPot` Home-cooked meal (model `item.mealPot`, both hands) | — | — | — | | |
| `tastingBoard` Tasting menu (model `item.tastingBoard`, both hands) | — | — | — | | |
| `grillPlatter` Barbecue (model `item.grillPlatter`, both hands) | — | — | — | | |
| `pizza` Wood-fired pizza (model `item.pizza`, both hands) | — | — | — | | |
| `bakingTray` Baked treats (model `item.bakingTray`, both hands) | — | — | — | | |
| `pieDish` Family pie (model `item.pieDish`, both hands) | — | — | — | | |
| `mealPlate` Plate of food (model `item.mealPlate`, one hand) | — | — | — | | |
| `pizzaSlice` Slice of pizza (model `item.pizzaSlice`, one hand) | — | — | — | | |
| `treatPlate` Plate of treats (model `item.treatPlate`, one hand) | — | — | — | | |
| `pieSlice` Slice of pie (model `item.pieSlice`, one hand) | — | — | — | | |
| `sandwich` Sandwich (model `item.sandwich`, one hand) | — | — | — | | |
| `snack` Snack (model `item.treatPlate`, one hand) | — | — | — | | |
| `iceCream` Ice cream (model `item.treatPlate`, one hand) | — | — | — | | |
| `toast` Toast (model `item.sandwich`, one hand) | — | — | — | | |
| `leftovers` Reheated meal (model `item.mealPlate`, one hand) | — | — | — | | |
| `cheeseBoard` Cheese board (model `item.tastingBoard`, both hands) | — | — | — | | |
| `smoothie` Smoothie (model `item.prepBowl`, one hand) | — | — | — | | |
| `noodles` Noodles (model `item.prepBowl`, one hand) | — | — | — | | |
| `takeout` Takeout (model `item.mealPlate`, one hand) | — | — | — | | |
| `plateDirty` Dirty plate (model `item.plateDirty`, one hand) | — | — | — | | |
| `potDirty` Dirty pot (model `item.potDirty`, both hands) | — | — | — | | |
| `platterDirty` Dirty platter (model `item.platterDirty`, both hands) | — | — | — | | |
| `trayDirty` Dirty baking tray (model `item.trayDirty`, both hands) | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

Not tied to one row: I-held-01.

### I-held-01 — No book, cup or cutlery prop: residents mime them

- **Severity:** S2
- **Rubric:** I7
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (`ACTIONS`, `writeProp`)
- **Evidence:** A known gap named in the rubric: reading (`lantern`), sips (`drink`) and eating (`eat`) show no prop. The phone is I-social-01. Props in hand are new behaviour: filed as `code`, planned separately (§9, decision 3).
  Pilot, the base objects: the same gap for tools. Watering the plant (`water`) with no can, trimming the shrub (`prune`) with no shears, painting at the easel with no brush (the brush jar stays full), feeding the baby with no bottle, the book at the bookshelf, the coffee at the dining table (`play/plant-water-01.png`, `play/bush-trim-01.png`, `play/easel-paint-01.png`, `scenes/crib-adult-feed-02.png`, `play/bookshelf-read-01.png`, `scenes/dining-coffee-02.png`). The only props drawn today are the cooking tools on a station (`STATION_PROPS`, `characters.ts:546`) and the parts a resident lifts off an object (`held`: the barbell).
- **Status:** confirmed (pilot, for the base objects)
- **Source:** `plan/authenticity-review.md` §3.3 (I7) and §9 (decision 3); pilot
