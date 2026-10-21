# Held items and tools: authenticity ledger

What it covers: the 20 held models (`item.*`, `tool.*`; family `tools/art/blender/families/held.py`) and the 25 items in `base.json` `items` that use them. Things residents hold that are parts of objects (the barbell, dumbbells, guitar) are in their object's ledger. Cooking steps are in [systems.md](systems.md).

- **Evidence:** `bench-results/review/held/` (gallery and close-ups, `inhand/` from eight recipe runs in `family-check` with what Ada held at each picture in `inhand/timeline.jsonl`, `inhand/poses/`; `extra/`: contact sheets of the close-ups, the in-hand shots and the poses, and crops of the cook at the stove and the grill; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Never looked at before** (`docs/objects/open.md` "Not looked at"): the held things that no recipe in the test house makes: the tasting board, pizza, pie, slices, sandwich and grill platter. Wave held: all six made in play now (`inhand/tastingMenu-*`, `pizza-*`, `pie-*`, `sandwich-*`, `bbq-*`); the whole dishes are never in a hand (I-held-04).
- **Wave: held** (2026-10-21). Every model's size is inside its kind's range (`tables/sizes-models.md`). A is the carry pose (both palms under a dish, one hand under a plate: `inhand/poses/poses-carry-hold*.png`), `n/a` where the thing is never carried. I covers carried, held at a station, put down and eaten (I7, I13).

## Held models (20)

`web/public/assets/packs/held/manifest.json`; drawn by `web/src/render/babylon/items.ts` and `characters.ts` (`STATION_PROPS` for the tools).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `item.ingredients` | OK | OK | I-held-03 | 0.20 × 0.34 × 0.14 paper bag with a tomato, a cabbage and a carrot; carried in both arms; I-held-03 S2 code: still held while chopping | `held/sheets/obj-item.ingredients.png`, `held/inhand/meal-01.png`, `held/inhand/poses/poses-carry-hold-focus0.png` |
| `item.prepBowl` | OK | OK | I-held-03 | 0.32 m white mixing bowl of diced vegetables; I-held-03: held over the hob and the grill for the whole cook step | `held/sheets/obj-item.prepBowl.png`, `held/extra/inhand-crops.png` |
| `item.mealPot` | OK | n/a | I-held-04 | 0.39 m black stew pot with two handles; I-held-04 S3 code: never carried, left on the stove | `held/sheets/obj-item.mealPot.png`, `held/inhand/poses/poses-carry-hold.png` |
| `item.potDirty` | OK | OK | OK | the pot with a brown film inside; carried to the bin and the sink in both hands | `held/sheets/obj-item.potDirty.png`, `held/inhand/gourmet-08.png` |
| `item.tastingBoard` | OK | n/a | I-held-04 | 0.40 × 0.24 wooden board with cheese, tomato and grapes; I-held-04 | `held/sheets/obj-item.tastingBoard.png`, `held/inhand/tastingMenu-06.png` |
| `item.grillPlatter` | OK | n/a | I-held-04 | 0.40 × 0.28 white oval platter with sausages, a burger and corn; I-held-04: stays on the counter | `held/sheets/obj-item.grillPlatter.png`, `held/inhand/bbq-06.png` |
| `item.platterDirty` | M-held-02 | OK | OK | the grill platter with sauce and a bone; M-held-02 S3 content: also what an emptied pizza board, tasting board or cheese board becomes | `held/sheets/obj-item.platterDirty.png`, `held/inhand/pizza-08.png` |
| `item.pizza` | OK | n/a | I-held-02, I-held-04 | 0.40 m pizza on a round wooden board; I-held-02 S2 content: "baked" by stirring a pan; I-held-04 | `held/sheets/obj-item.pizza.png`, `held/inhand/pizza-04.png`, `held/inhand/pizza-05.png` |
| `item.bakingTray` | OK | n/a | I-held-02, I-held-04 | 0.38 × 0.28 metal tray of six buns; I-held-02, I-held-04 | `held/sheets/obj-item.bakingTray.png`, `held/inhand/bakedTreats-05.png` |
| `item.trayDirty` | M-held-02 | OK | OK | the tray with a brown smear; M-held-02: also what an emptied pie dish becomes | `held/sheets/obj-item.trayDirty.png`, `held/inhand/pie-08.png` |
| `item.pieDish` | OK | n/a | I-held-02, I-held-04 | 0.34 m lattice pie in its dish; I-held-02, I-held-04 | `held/sheets/obj-item.pieDish.png`, `held/inhand/pie-07.png` |
| `item.mealPlate` | OK | OK | I-held-01 | 0.29 m plate with meat, potato and greens; carried on one hand, put down in front of the diner, eaten with no knife or fork (I-held-01) | `held/sheets/obj-item.mealPlate.png`, `held/inhand/bbq-06.png`, `held/inhand/poses/poses-carry-hold.png` |
| `item.pizzaSlice` | OK | OK | OK | a slice on a plate; finger food, so the bare-hand eat clip fits | `held/sheets/obj-item.pizzaSlice.png`, `held/inhand/pizza-05.png` |
| `item.treatPlate` | OK | OK | OK | cookies and a cupcake on a plate; finger food | `held/sheets/obj-item.treatPlate.png`, `held/inhand/bakedTreats-07.png` |
| `item.pieSlice` | OK | OK | I-held-01 | a slice with cream on a plate; eaten with no fork (I-held-01) | `held/sheets/obj-item.pieSlice.png`, `held/inhand/pie-05.png` |
| `item.sandwich` | OK | OK | OK | a sandwich on a plate; finger food (the in-play shot has it behind her: `sandwich-03`) | `held/sheets/obj-item.sandwich.png`, `held/inhand/sandwich-03.png` |
| `item.plateDirty` | OK | OK | OK | the plate with a smear and cutlery (the only cutlery in the game); carried to the bin and the sink | `held/sheets/obj-item.plateDirty.png`, `held/inhand/meal-08.png` |
| `tool.board` | OK | n/a | I-held-02, I-systems-01 | 0.34 × 0.24 board with a knife and chopped carrot, on the counter while chopping: right; I-held-02: also for mixing batter and rolling pastry; I-systems-01 S2 renderer-table (systems ledger): on the grill station's grate | `held/sheets/obj-tool.board.png`, `held/inhand/poses/poses-tools-focus0.png` |
| `tool.pan` | OK | n/a | I-held-02 | 0.28 × 0.39 frying pan with food (in the 24–28 cm range with its handle); I-held-02: the pan comes out for baking at the oven and the pizza oven, and on the microwave counter | `held/sheets/obj-tool.pan.png`, `held/inhand/pie-04.png`, `held/inhand/pizza-04.png` |
| `tool.grillFood` | OK | n/a | I-systems-01 | a burger and two sausages on the grill's bars while flipping; right on the kettle grill; I-systems-01 (systems ledger) | `held/sheets/obj-tool.grillFood.png`, `held/inhand/bbq-05.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| `item.ingredients` | take from the fridge, carry to the worktop, unpack | taken at the fridge, carried to the prep station, held through the chopping | I-held-03 |
| `item.prepBowl` | carry prepped food; tip it into the pan or the dish | carried to the stove or grill, held through the whole cook step, never tipped in | I-held-03 |
| `item.mealPot` | carry the meal to the table, serve from it; take it to the sink | left on the stove, servings taken from there; carried only when dirty | I-held-04 |
| `item.tastingBoard`, `item.grillPlatter`, `item.pizza`, `item.bakingTray`, `item.pieDish` | carry the dish to the table, serve | left at the station or on the counter beside it | I-held-04 |
| `item.potDirty`, `item.platterDirty`, `item.trayDirty`, `item.plateDirty` | scrape, carry to the sink, wash | scraped into the bin, carried to the sink or dishwasher, washed | OK |
| `item.mealPlate`, `item.pieSlice` | carry to the table, eat with a knife and fork | carried, put down, eaten with bare hands | I-held-01 |
| `item.pizzaSlice`, `item.treatPlate`, `item.sandwich` | carry, eat with the fingers | carried, put down, eaten by hand | OK |
| `tool.board` | chop on it | chopping; also mixing and rolling pastry | I-held-02 |
| `tool.pan` | fry and stir on the hob | stirring on the hob; also "baking" at an oven, a pizza oven, a microwave counter | I-held-02 |
| `tool.grillFood` | turn on the grill | flipped on the grill | OK |

## Items (25)

`base.json` `items`: what each is, its model and its grip (`hold`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `ingredients` Ingredients (model `item.ingredients`, both hands) | OK | OK | I-held-03 | from the fridge or pantry to the prep station (every recipe's first step) | `held/sheets/obj-item.ingredients.png`, `held/inhand/meal-01.png` |
| `prepBowl` Prepped bowl (model `item.prepBowl`, both hands) | OK | OK | I-held-03 | from prep to the hob, oven or grill | `held/extra/inhand-crops.png`, `held/inhand/bbq-05.png` |
| `mealPot` Home-cooked meal (model `item.mealPot`, both hands) | OK | n/a | I-held-04 | servings `mealPlate`, empty `potDirty`: right | `held/sheets/obj-item.mealPot.png`, `held/inhand/gourmet-05.png` |
| `tastingBoard` Tasting menu (model `item.tastingBoard`, both hands) | OK | n/a | I-held-04, M-held-02 | servings `mealPlate`; M-held-02: empties to the white oval `platterDirty` | `held/sheets/obj-item.tastingBoard.png`, `held/inhand/tastingMenu-11.png` |
| `grillPlatter` Barbecue (model `item.grillPlatter`, both hands) | OK | n/a | I-held-04 | servings `mealPlate` (barbecue on plates: right), empty `platterDirty` | `held/sheets/obj-item.grillPlatter.png`, `held/inhand/bbq-06.png` |
| `pizza` Wood-fired pizza (model `item.pizza`, both hands) | OK | n/a | I-held-02, I-held-04, M-held-02 | servings `pizzaSlice`; M-held-02: the round board empties to the oval `platterDirty` | `held/sheets/obj-item.pizza.png`, `held/inhand/pizza-07.png` |
| `bakingTray` Baked treats (model `item.bakingTray`, both hands) | OK | n/a | I-held-02, I-held-04 | servings `treatPlate`, empty `trayDirty`: right | `held/sheets/obj-item.bakingTray.png`, `held/inhand/bakedTreats-05.png` |
| `pieDish` Family pie (model `item.pieDish`, both hands) | OK | n/a | I-held-02, I-held-04, M-held-02 | servings `pieSlice`; M-held-02: the round dish empties to the rectangular `trayDirty` | `held/sheets/obj-item.pieDish.png`, `held/inhand/pie-08.png` |
| `mealPlate` Plate of food (model `item.mealPlate`, one hand) | OK | OK | I-held-01 | a serving of the meal, the barbecue and the tasting menu; at the island the white plate is hard to see on the pale top (`inhand/meal-06.png`, only the food shows): not checked closer | `held/inhand/bbq-06.png`, `held/inhand/meal-06.png` |
| `pizzaSlice` Slice of pizza (model `item.pizzaSlice`, one hand) | OK | OK | OK | finger food | `held/inhand/pizza-05.png` |
| `treatPlate` Plate of treats (model `item.treatPlate`, one hand) | OK | OK | OK | finger food | `held/inhand/bakedTreats-07.png` |
| `pieSlice` Slice of pie (model `item.pieSlice`, one hand) | OK | OK | I-held-01 | wants a fork | `held/inhand/pie-05.png` |
| `sandwich` Sandwich (model `item.sandwich`, one hand) | OK | OK | OK | made and eaten at once (`eatNow`) | `held/inhand/sandwich-03.png` |
| `snack` Snack (model `item.treatPlate`, one hand) | OK | OK | OK | grabbed from a fridge or pantry (10 objects): a plate of cookies stands in for a snack well enough | `held/sheets/obj-item.treatPlate.png` |
| `iceCream` Ice cream (model `item.treatPlate`, one hand) | M-held-01 | OK | M-held-01, I-held-01 | M-held-01 S2 model: a plate of cookies and a cupcake; wants a bowl or cone, and a spoon (I-held-01) | `held/sheets/obj-item.treatPlate.png` |
| `toast` Toast (model `item.sandwich`, one hand) | M-held-01 | OK | OK | M-held-01: a filled sandwich | `held/sheets/obj-item.sandwich.png` |
| `leftovers` Reheated meal (model `item.mealPlate`, one hand) | OK | OK | I-held-01 | from the microwave (`microwave.reheat`, `slob.microwaveCart.nuke`): a plate of food, right | `held/sheets/obj-item.mealPlate.png` |
| `cheeseBoard` Cheese board (model `item.tastingBoard`, both hands) | OK | OK | M-held-02 | the tasting board is a cheese board (`foodie.deliPantry`); finger food; M-held-02: empties to the oval platter | `held/sheets/obj-item.tastingBoard.png` |
| `smoothie` Smoothie (model `item.prepBowl`, one hand) | M-held-01 | M-held-01 | M-held-01, I-held-01 | M-held-01: a 32 cm mixing bowl of diced vegetables held on one palm; wants a glass | `held/sheets/obj-item.prepBowl.png` |
| `noodles` Noodles (model `item.prepBowl`, one hand) | M-held-01 | M-held-01 | M-held-01, I-held-01 | M-held-01: the same 32 cm bowl of diced vegetables; wants a noodle bowl or box and chopsticks or a fork | `held/sheets/obj-item.prepBowl.png` |
| `takeout` Takeout (model `item.mealPlate`, one hand) | M-held-01 | OK | M-held-01, I-held-01 | M-held-01: a china plate of meat and potatoes from a delivery locker; wants a box or a bag | `held/sheets/obj-item.mealPlate.png` |
| `plateDirty` Dirty plate (model `item.plateDirty`, one hand) | OK | OK | OK | scraped, washed | `held/inhand/meal-08.png` |
| `potDirty` Dirty pot (model `item.potDirty`, both hands) | OK | OK | OK | scraped, washed | `held/inhand/gourmet-08.png` |
| `platterDirty` Dirty platter (model `item.platterDirty`, both hands) | M-held-02 | OK | OK | M-held-02 | `held/inhand/bbq-09.png`, `held/inhand/pizza-08.png` |
| `trayDirty` Dirty baking tray (model `item.trayDirty`, both hands) | M-held-02 | OK | OK | M-held-02 (for the pie) | `held/inhand/pie-08.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| `ingredients`, `prepBowl` | carry to the next station, put down, use | carried, held through the next station's work | I-held-03 |
| `mealPot`, `tastingBoard`, `grillPlatter`, `pizza`, `bakingTray`, `pieDish` | cook, carry to the table, serve, put the rest away | cooked, left at the station, served from there, the rest kept there or in the fridge | I-held-04 (pizza, tray and pie also I-held-02) |
| `mealPlate`, `pieSlice`, `leftovers`, `takeout`, `iceCream`, `noodles`, `smoothie` | eat with a knife and fork, a spoon, chopsticks; drink from a glass | eaten with bare hands | I-held-01 |
| `pizzaSlice`, `treatPlate`, `sandwich`, `snack`, `toast`, `cheeseBoard` | eat with the fingers | eaten by hand | OK |
| `plateDirty`, `potDirty`, `platterDirty`, `trayDirty` | scrape, wash, put away | scraped into the bin, washed at the sink or dishwasher | OK |
| `smoothie`, `noodles`, `takeout`, `iceCream`, `toast` | look like themselves | look like a mixing bowl, a roast dinner, cookies, a sandwich | M-held-01 |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

Not tied to one row: I-held-01.

### I-held-01 — No book, cup or cutlery prop: residents mime them

> Consolidation: widened by the object waves. Kitchen: no cup, glass or plate for the coffee at tables and chairs, the coffee machine, the stools' and fridges' drinks, the wine, breakfast and dinner (`kitchen/play/c5/diningGlass-coffee-01.png`, `kitchen/play/c7/stoolBarHigh-drink-01.png`; cooked meals do show a plate). Living: no book on the reading chairs or at the four bookcases, no magazine at the three coffee tables. Community: the card at the post box, the cone at the ice cream cart, the snack and the can at the machines, the café's cup, cake and plate, the coin at the fountain, the basketball.

- **Severity:** S2
- **Rubric:** I7
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (`ACTIONS`, `writeProp`)
- **Evidence:** A known gap named in the rubric: reading (`lantern`), sips (`drink`) and eating (`eat`) show no prop. The phone is I-social-01. Props in hand are new behaviour: filed as `code`, planned separately (§9, decision 3).
  Pilot, the base objects: the same gap for tools. Watering the plant (`water`) with no can, trimming the shrub (`prune`) with no shears, painting at the easel with no brush (the brush jar stays full), feeding the baby with no bottle, the book at the bookshelf, the coffee at the dining table (`play/plant-water-01.png`, `play/bush-trim-01.png`, `play/easel-paint-01.png`, `scenes/crib-adult-feed-02.png`, `play/bookshelf-read-01.png`, `scenes/dining-coffee-02.png`). The only props drawn today are the cooking tools on a station (`STATION_PROPS`, `characters.ts:546`) and the parts a resident lifts off an object (`held`: the barbell).
  Wave held, the items: the plate, the slice or the bowl is put down in front of the diner and eaten with bare hands. Fine for finger food (pizza slice, treats, snack, sandwich, toast, cheese board); a meal, a slice of pie, reheated leftovers, takeout, ice cream and noodles want cutlery or chopsticks, and the smoothie a glass (`held/inhand/bbq-06.png`, `held/inhand/meal-06.png`). The only cutlery in the game is drawn on the dirty plate, after the meal (`held/sheets/obj-item.plateDirty.png`).
- **Status:** confirmed (pilot, for the base objects; wave held, for the eaten items)
- **Source:** `plan/authenticity-review.md` §3.3 (I7) and §9 (decision 3); pilot; wave: held

### M-held-01 — Five items borrow a model that shows something else

- **Severity:** S2
- **Rubric:** M6, I7, M1
- **Fix class:** model
- **Where:** `web/public/content/base.json:4050` (`iceCream`: `item.treatPlate`), `:4058` (`toast`: `item.sandwich`), `:4082` (`smoothie`: `item.prepBowl`), `:4089` (`noodles`: `item.prepBowl`), `:4096` (`takeout`: `item.mealPlate`); new models in `tools/art/blender/families/held.py`
- **Evidence:** Ice cream from the fridge is a plate of cookies and a cupcake (`held/sheets/obj-item.treatPlate.png`). Toast from the breakfast counter is a filled sandwich (`obj-item.sandwich.png`). A smoothie from the breakfast counter or the power blender, and noodles from the delivery locker, are the cook's 32 cm mixing bowl of diced vegetables (`obj-item.prepBowl.png`) held on one palm (`hold: hand`): a smoothie is a 0.3 L glass about 8 cm across, a noodle bowl 15–18 cm. Takeout from a delivery locker or hatch is a china plate of meat and potatoes (`obj-item.mealPlate.png`) rather than a box or a bag. Each is fetched, carried to a seat and eaten in play (`base.json` fridge `frozenTreat`; `furniture.json` `breakfastCounter`; packs `energetic`, `lazy`, `loner`, `couchPotato`). Small models (a cone or a bowl of ice cream, a slice of toast, a glass, a noodle box, a takeaway carton) fix it; `snack` on the treat plate and `leftovers` on the meal plate are fine.
- **Status:** open
- **Source:** wave: held

### M-held-02 — An emptied dish turns into a different dish

- **Severity:** S3
- **Rubric:** M13, M6
- **Fix class:** content
- **Where:** `web/public/content/base.json:3957` (`tastingBoard`), `:3975` (`pizza`), `:3993` (`pieDish`), `:4074` (`cheeseBoard`): their `emptyAs`
- **Evidence:** The pizza on its round wooden board, the tasting board and the cheese board all become `platterDirty`, the grill platter's white oval with sauce and a bone (`held/sheets/obj-item.platterDirty.png`; `held/inhand/pizza-07.png` … `pizza-10.png`, `tastingMenu-11.png` … `tastingMenu-14.png`); the round pie dish becomes the rectangular metal `trayDirty` (`held/inhand/pie-07.png` … `pie-10.png`). The dish that went in should come out: a dirty board for the boards and the pizza, a dirty pie dish for the pie (two small models, or the boards' and dish's own models with a smear part).
- **Status:** open
- **Source:** wave: held

### I-held-02 — Baking is played as frying: a pan comes out at the oven

> Consolidation (2026-10-21): closed as a duplicate of I-kitchen-05 (systems ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I13, I7, I3
- **Fix class:** content
- **Where:** `web/public/content/base.json:4391`, `:4435`, `:4478` (the `pizza`, `bakedTreats` and `pie` "Bake" steps: `anim: stir`), `:4425` and `:4468` ("Mix" and "Roll the pastry": `anim: chop`), `:4169-4176` (`meal`'s "Cook" also at the `microwave` station, `stir`); `web/src/render/babylon/characters.ts:546` (`STATION_PROPS`, by clip tag, not by station), `:2102` (a station without a `surface` gets the tool at 0.9 m)
- **Evidence:** Baking a pie at the stove, a pizza at the pizza oven, treats at the oven: the cook stands at the oven stirring, a frying pan drawn on its top (`held/inhand/pie-04.png`, `held/inhand/pizza-04.png`; `held/extra/inhand-b.png`). Mixing the batter and rolling the pastry are chopping with a knife on a board. A meal "cooked" at the microwave counter puts the frying pan on it (code: `meal` lists `microwave`; the microwave counter has no `surface`, so the pan goes at 0.9 m over its middle). Real baking is: put the tray or dish in the oven, wait, take it out; mixing is stirring in a bowl. A `bake` anim (the oven's door motion, a pick-up and put-in pose, the dish as its prop: `item.bakingTray`, `item.pieDish`, `item.pizza` exist) and a `mix` one (the prep bowl on the counter) fix it, each with an `ACTIONS` and `STATION_PROPS` entry.
- **Status:** closed (duplicate of I-kitchen-05)
- **Source:** wave: held (shared with the systems ledger's `pizza`, `bakedTreats`, `pie` and `meal` recipe rows)

### I-held-03 — The cook keeps the last step's things in hand through the next

> Consolidation (2026-10-21): closed as a duplicate of I-kitchen-04 (systems ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I13, I7, A1
- **Fix class:** code
- **Where:** `crates/sim-core/src/cooking.rs:1501` (`held` is set to the step's `holds` when it ends and kept through the whole next step), `web/src/render/babylon/characters.ts:2425` (`writeCarried` draws it at the palms whatever the clip; only the carry arms are dropped at a station, `:2766`)
- **Evidence:** In every recipe run Ada chops at the counter with the bag of ingredients still in her hands (`inhand/timeline.jsonl`: `held: ingredients, action: chop`; `held/inhand/meal-01.png`), and stands over the hob and the grill holding the prepped bowl for the whole cook step (`held: prepBowl, action: stir` and `flip`; `held/extra/inhand-crops.png`, `held/inhand/bbq-05.png`), the pan or the grill food beside it. In a kitchen the bag is unpacked onto the worktop and the bowl tipped into the pan as the step starts. Putting the held thing down on the station (or dropping `held` once the step has begun, with the tool showing) fixes it.
- **Status:** closed (duplicate of I-kitchen-04)
- **Source:** wave: held (shared with the systems ledger's cooking flow rows "carried items appear at each step")

### I-held-04 — moved to systems.md

