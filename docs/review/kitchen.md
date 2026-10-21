# Kitchen: authenticity ledger

What it covers: the 72 objects of [docs/objects/kitchen.md](../objects/kitchen.md), in its groups and order.

- **Evidence:** `bench-results/review/kitchen/` (gallery, clip, pose and play shots; shots stay out of git). Shots in the rows are relative to `bench-results/review/` in the review worktree: `kitchen/sheets/obj-<id>.png` is the object's contact sheet, `kitchen/play/c<n>/…` the evidence agent's play shots, `kitchen/extra/…` the kitchen wave's own scenes (scripted recipes and clear-aways, a child at the wine fridge and the bar cart, chairs at plain tables; how they were taken is in `kitchen/review-log.md`).
- **Pilot rows** (`fridge`, `diningTable`, 2026-10-21): their Shots are relative to `bench-results/review/pilot/`; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Durations** are judged by §9 decision 2 (short by design; only outliers over 3× the tag median). The `uses` table flags `coldDrink`, `reheat`, `toast`, `smoothie`, `sandwich` and `washUp` against their tag's median, but each is inside its real range (`reference.json`: cold drink 1–3, reheat 2–5, toast 3–5, smoothie 2–5, sandwich 3–10, washing up 10–30 minutes): the tags mix a toast with a full meal, so the flags are not outliers.
- **Kitchen wave** (2026-10-21): rows filled for the other 70 objects; the pilot's rows kept, with pointers added.

## Kitchen sinks (2)

Family: `tools/art/blender/families/kitchen.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `farmhouseSink` Farmhouse sink | OK | OK | I-kitchen-03, I-kitchen-11 | I-kitchen-03 S2 renderer-table; I-kitchen-11 S3 code | `kitchen/sheets/obj-farmhouseSink.png`, `kitchen/play/c1/farmhouseSink-washUp-01.png`, `kitchen/extra/putaway-wallCabinet-04.png` |
| `counterSinkDouble` Double sink | OK | A-kitchen-03 | I-kitchen-03, I-kitchen-11 | A-kitchen-03 S3 motion; I-kitchen-03 S2 renderer-table; I-kitchen-11 S3 code | `kitchen/sheets/obj-counterSinkDouble.png`, `kitchen/play/c2/counterSinkDouble-wash-01.png` |

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `farmhouseSink` | wash hands; wash up by hand; fill a pan or kettle | wash hands (2), wash the dishes (15, not autonomous); a `dishes` station when a meal is cleared | OK; washing up needs nothing to wash (I-kitchen-11) |
| `counterSinkDouble` | as the sink, two at once; rinse in one bowl, wash in the other | the same, two slots | OK; one washer runs both taps (A-kitchen-03) |

## K1. Fridges (7)

Family: `tools/art/blender/families/kitchen.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `fridge` Fridge | OK | OK | I-kitchen-02 | I-kitchen-01 S3 content (seeded; closed); I-kitchen-02 S3 content | `sheets/obj-fridge.png`, `play/fridge-*.png`, `clips/clips-fridge-all-t*.png`, `play-child/fridge-*.png` |
| `fridgeMini` Mini fridge | M-kitchen-03 | OK | I-kitchen-08 | M-kitchen-03 S3 decision; I-kitchen-08 S3 content | `kitchen/sheets/obj-fridgeMini.png`, `kitchen/play/c1/fridgeMini-meal-01.png` |
| `fridgeBuiltIn` Built-in fridge | M-kitchen-04 | OK | OK | M-kitchen-04 S3 decision; I-kitchen-01 S3 content (seeded; closed) | `kitchen/sheets/obj-fridgeBuiltIn.png`, `kitchen/play/c6/fridgeBuiltIn-frozenTreat-01.png` |
| `fridgeFrenchDoor` French-door fridge | M-kitchen-06 | OK | OK | M-kitchen-06 S3 model; I-kitchen-01 S3 content (seeded; closed) | `kitchen/sheets/obj-fridgeFrenchDoor.png`, `kitchen/play/c6/fridgeFrenchDoor-coldDrink-01.png` |
| `fridgeRetro` Retro fridge | OK | OK | OK | | `kitchen/sheets/obj-fridgeRetro.png`, `kitchen/clips/clips-fridgeRetro-all-t99.png` |
| `fridgeMiniRetro` Retro mini fridge | OK | OK | OK | | `kitchen/sheets/obj-fridgeMiniRetro.png` |
| `wineFridge` Wine fridge | M-kitchen-06 | OK | I-kitchen-06, I-held-01 | M-kitchen-06 S3 model; I-kitchen-06 S1 code; I-held-01 S2 code (held ledger: no glass) | `kitchen/sheets/obj-wineFridge.png`, `kitchen/extra/play-child/child-wineFridge-wine-01.png` |

Notes: every fridge door swings 100–118° on the handle's far side (reference 90–125°) and shuts after its `hold`, as the use ends; the French-door fridge's freezer drawer slides 35 cm (reference 0.4–0.6 m: a little short, not filed). The retro fridge's lever turns 30° and is back after 0.4 s, before the door is far open: the "detached lever" seen in `clips-fridgeRetro-all-t99.png` is both clips held open at once, which play never shows. Snacks are eaten standing at the fridge on the bare stage, and the plate goes on the nearest top (I-systems-03 when there is none).

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `fridge` | get a snack or a cold drink, take out ingredients, put the shopping away | snack (2), ice cream (2), make a meal (cook), gourmet meal (cook, needs Cooking) | cold drink missing (I-kitchen-02); the shopping is the sim's |
| `fridgeMini` | get a snack or a cold drink | snack (2), simple meal (cook) | cold drink missing, though it is a mini fridge's main use and the retro mini fridge has it (I-kitchen-08) |
| `fridgeBuiltIn` | snack, drink, ingredients; water or ice from the dispenser | snack, ice cream, meal, gourmet, cold drink (5) | OK; the drink opens the door, not the dispenser on it (fine) |
| `fridgeFrenchDoor` | snack, drink, ingredients, the freezer drawer | snack, ice cream (freezer drawer), meal, gourmet, cold drink (right door only) | OK |
| `fridgeRetro` | snack, drink, ingredients; the ice box inside | snack, meal, cold drink | OK (no ice cream: its ice box isn't built; fine) |
| `fridgeMiniRetro` | snack or a cold drink | snack, simple meal, cold drink | OK |
| `wineFridge` | take out a bottle, pour a glass | pour a glass of wine (10) | OK for adults; a child pours one too (I-kitchen-06) |

## K2. Cookers (6)

Family: `tools/art/blender/families/kitchen_cooking.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `stoveElectric` Electric cooker | OK | A-kitchen-02 | I-kitchen-04, I-kitchen-05, I-systems-01, I-systems-04 | A-kitchen-02 S2 motion; I-kitchen-04 S2 code; I-kitchen-05 S2 renderer-table; I-systems-01 S2 renderer-table, I-systems-04 S2 code (systems ledger) | `kitchen/sheets/obj-stoveElectric.png`, `kitchen/play/c5/stoveElectric-bake-01.png` |
| `stoveGas` Gas range | OK | A-kitchen-02 | I-kitchen-04, I-kitchen-05, I-systems-01, I-systems-04 | A-kitchen-02 S2 motion; I-kitchen-04 S2 code; I-kitchen-05 S2 renderer-table; I-systems-01, I-systems-04 (systems ledger) | `kitchen/sheets/obj-stoveGas.png`, `kitchen/extra/meal-fridge-stoveGas-06.png`, `kitchen/play/c5/stoveGas-cook-01.png` |
| `wallOven` Oven tower | M-kitchen-03 | A-kitchen-02 | I-kitchen-05, I-systems-04 | M-kitchen-03 S3 decision; A-kitchen-02 S2 motion; I-kitchen-05 S2 renderer-table; I-systems-04 (systems ledger) | `kitchen/sheets/obj-wallOven.png`, `kitchen/extra/meal-fridge-wallOven-06.png`, `kitchen/clips/clips-wallOven-all-t0.5.png` |
| `rangeCooker` Range cooker | M-kitchen-03 | A-kitchen-02 | I-kitchen-04, I-kitchen-05, I-systems-01, I-systems-04 | M-kitchen-03 S3 decision; A-kitchen-02 S2 motion; I-kitchen-04 S2 code; I-kitchen-05 S2 renderer-table; I-systems-01, I-systems-04 (systems ledger) | `kitchen/sheets/obj-rangeCooker.png`, `kitchen/play/c5/rangeCooker-bake-01.png` |
| `cooktopCounter` Hob counter | OK | A-kitchen-02 | I-kitchen-04, I-systems-01, I-systems-04 | A-kitchen-02 S2 motion; I-kitchen-04 S2 code; I-systems-01, I-systems-04 (systems ledger) | `kitchen/sheets/obj-cooktopCounter.png`, `kitchen/play/c5/cooktopCounter-cook-01.png` |
| `microwave` Microwave counter | OK | A-kitchen-02 | I-kitchen-03, I-kitchen-05, I-kitchen-12 | A-kitchen-02 S2 motion; I-kitchen-03 S2 renderer-table; I-kitchen-05 S2 renderer-table; I-kitchen-12 S3 code | `kitchen/sheets/obj-microwave.png`, `kitchen/play/c1/microwave-reheat-01.png`, `kitchen/extra/meal-fridge-microwave-06.png` |

Notes: the hobs are 0.89–0.95 m (reference 0.85–0.95) and every oven door drops 85° on its bottom hinge (reference 85–95°). The wall oven's "door swinging up into a slab at chest height" in the evidence notes is the upper oven's door dropping on its bottom hinge, as it should (`clips-wallOven-all-t0.5.png` shows it tilting towards the camera from its bottom edge); an eye-level oven is real. The gas range is 0.92 m wide on one tile: as a 90 cm range (`rangeCooker` kind) its 1.11 m height is the backguard, 1 % over, not filed.

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `stoveElectric` | cook on the hob; bake or roast in the oven | cook dinner (recipe `meal`), bake treats (`bakedTreats`) | OK; baking is stirred on the hob (I-kitchen-05) |
| `stoveGas` | the same | cook, gourmet (Cooking 3 for its feeling), bake | OK; as above |
| `wallOven` | bake or roast | roast dinner (`meal`), bake treats | OK; roasting is stirring a pan in front of it (I-kitchen-05) |
| `rangeCooker` | cook on many burners; bake or roast | cook, gourmet, bake | OK |
| `cooktopCounter` | cook on the hob; prepare food on the rest of the top | cook | OK |
| `microwave` | reheat food; cook a ready meal | heat up leftovers (4, then eat 12) | OK; as a meal's Cook station it shows a frying pan (I-kitchen-05) |

## K3. Small appliances and dishwashers (4)

Family: `tools/art/blender/families/kitchen_cooking.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `coffeeMachine` Coffee machine | OK | OK | I-held-01, I-kitchen-12 | I-held-01 S2 code (held ledger: no cup in hand); I-kitchen-12 S3 code | `kitchen/sheets/obj-coffeeMachine.png`, `kitchen/play/c1/coffeeMachine-espresso-01.png` |
| `breakfastCounter` Breakfast counter | OK | OK | I-kitchen-03, I-kitchen-12 | I-kitchen-03 S2 renderer-table; I-kitchen-12 S3 code | `kitchen/sheets/obj-breakfastCounter.png`, `kitchen/play/c1/breakfastCounter-toast-01.png` |
| `dishwasher` Dishwasher | M-kitchen-02 | A-kitchen-01 | I-kitchen-11 | M-kitchen-02 S2 model; A-kitchen-01 S2 motion; I-kitchen-11 S3 code | `kitchen/sheets/obj-dishwasher.png`, `kitchen/extra/meal-fridge-stoveGas-15.png`, `kitchen/clips/clips-dishwasher-all-t99.png` |
| `dishwasherCompact` Stainless dishwasher | M-kitchen-02 | A-kitchen-01 | I-kitchen-11 | M-kitchen-02 S2 model; A-kitchen-01 S2 motion; I-kitchen-11 S3 code | `kitchen/sheets/obj-dishwasherCompact.png` |

Notes: the blender lid that "floats above the jar" at t 0.5 is the `blend` clip lifting it off for the fruit before the blades turn; it is back before the swirl's second round (OK). The toaster's carriage goes down and pops up after 2 s. The coffee machine's cup slides under the spout and fills; the resident then sips at nothing for 10 minutes beside it (I-held-01). The dishwasher's door lies just off the floor when open (a real one drops to about 10 cm), and the loader stands 15 cm back from it (`STAND_BACK`).

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `coffeeMachine` | make a coffee | make a coffee (10, standing); provides `coffee` for the tables and chairs | OK; a child has one too (I-kitchen-06, lesser part) |
| `breakfastCounter` | make toast; blend a smoothie; prepare food on the top | make toast (4, eat 8), blend a smoothie (3, eat 5); a `prep` station | OK |
| `dishwasher` | load dirty dishes; unload clean ones | load the dishwasher (10, not autonomous); a `dishes` station when a meal is cleared | OK; needs nothing to load (I-kitchen-11); stays shut while loaded (A-kitchen-01) |
| `dishwasherCompact` | the same | run the dishwasher (10) | as above |

## K4. Base counters (8)

Family: `tools/art/blender/families/kitchen_counters.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `counter` Kitchen counter | OK | OK | I-kitchen-04 | I-kitchen-04 S2 code | `kitchen/sheets/obj-counter.png`, `kitchen/extra/meal-fridge-stoveGas-01.png` |
| `counterCorner` Corner counter | OK | OK | I-kitchen-04 | I-kitchen-04 S2 code | `kitchen/sheets/obj-counterCorner.png`, `kitchen/play/c5/counterCorner-sandwich-01.png` |
| `counterEnd` Counter end with shelves | M-kitchen-06 | n/a | I-kitchen-08 | M-kitchen-06 S3 model; I-kitchen-08 S3 content | `kitchen/sheets/obj-counterEnd.png`, `kitchen/play/c2/counterEnd-no-uses.png` |
| `counterCornerOuter` Outer corner counter | OK | n/a | I-kitchen-08 | I-kitchen-08 S3 content | `kitchen/sheets/obj-counterCornerOuter.png` |
| `counterDrawers` Drawer counter | OK | OK | I-kitchen-08 | I-kitchen-08 S3 content | `kitchen/sheets/obj-counterDrawers.png`, `kitchen/clips/clips-counterDrawers-all-t99.png` |
| `counterOpenShelf` Open-shelf counter | OK | n/a | I-kitchen-08 | I-kitchen-08 S3 content | `kitchen/sheets/obj-counterOpenShelf.png` |
| `butcherBlock` Butcher block | OK | OK | I-kitchen-04 | I-kitchen-04 S2 code | `kitchen/sheets/obj-butcherBlock.png`, `kitchen/play/c5/butcherBlock-sandwich-01.png` |
| `counterMarble` Marble counter | OK | OK | I-kitchen-04 | I-kitchen-04 S2 code | `kitchen/sheets/obj-counterMarble.png` |

Notes: every worktop is 0.95 m (reference 0.85–1.00). The corner counter's 1.26 m height is its utensil pot and its 0.93 m depth the square piece where two runs meet; the butcher block's 0.82 m depth is a free-standing block's, not a base unit's (neither filed). The drawers slide 25 cm (a real one opens 40–55 cm fully; a drawer pulled out for a knife is fine).

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `counter` | prepare and chop food; make a sandwich; store pans | make a sandwich (recipe, 9); a `prep` station | OK |
| `counterCorner` | the same | make a sandwich; `prep` | OK |
| `counterEnd` | prepare food; keep cookbooks on the end shelves | `prep` only | sandwich missing (I-kitchen-08) |
| `counterCornerOuter` | prepare food | `prep` only | sandwich missing (I-kitchen-08) |
| `counterDrawers` | prepare food; keep cutlery | `prep`; clean things put away in its top drawer | sandwich missing (I-kitchen-08) |
| `counterOpenShelf` | prepare food; keep plates on show | `prep` only | sandwich missing (I-kitchen-08) |
| `butcherBlock` | chop on the block | make a sandwich; `prep` | OK |
| `counterMarble` | prepare food; roll pastry | make a sandwich; `prep` | OK |

## K5. Tall and wall storage (11)

Family: `tools/art/blender/families/kitchen_counters.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `pantry` Pantry cupboard | OK | OK | OK | | `kitchen/sheets/obj-pantry.png`, `kitchen/play/c1/pantry-snack-01.png` |
| `pantryTall` Tall larder | OK | OK | OK | | `kitchen/sheets/obj-pantryTall.png` |
| `wallCabinet` Wall cabinets | OK | OK | I-kitchen-03 | I-kitchen-03 S2 renderer-table | `kitchen/sheets/obj-wallCabinet.png`, `kitchen/extra/putaway-wallCabinet-05.png` |
| `wallCabinetGlass` Glazed wall cabinet | OK | OK | I-kitchen-03 | I-kitchen-03 S2 renderer-table | `kitchen/sheets/obj-wallCabinetGlass.png` |
| `wallCabinetCorner` Wall cabinet with end shelves | OK | OK | I-kitchen-03 | I-kitchen-03 S2 renderer-table | `kitchen/sheets/obj-wallCabinetCorner.png` |
| `rangeHood` Cooker hood | OK | n/a | I-kitchen-09 | I-kitchen-09 S3 decision | `kitchen/sheets/obj-rangeHood.png`, `kitchen/play/c1/rangeHood-no-uses.png` |
| `spiceRack` Spice rack | OK | n/a | I-kitchen-09 | I-kitchen-09 S3 decision | `kitchen/sheets/obj-spiceRack.png` |
| `potRail` Pot rail | OK | n/a | I-kitchen-09 | I-kitchen-09 S3 decision | `kitchen/sheets/obj-potRail.png` |
| `kitchenShelfWall` Kitchen wall shelves | OK | n/a | I-kitchen-09 | I-kitchen-09 S3 decision | `kitchen/sheets/obj-kitchenShelfWall.png` |
| `trashCan` Pedal bin | OK | OK | OK | | `kitchen/sheets/obj-trashCan.png`, `kitchen/clips/clips-trashCan-all-t99.png` |
| `recyclingBins` Recycling bins | OK | OK | I-kitchen-11 | I-kitchen-11 S3 code | `kitchen/sheets/obj-recyclingBins.png`, `kitchen/play/c3/recyclingBins-sortRecycling-01.png` |

Notes: the wall cabinets' undersides are at 1.44 m (`kitchen_counters.py:394`; reference 1.40–1.50) and the hood's filter at 1.55 m, 0.60–0.66 m over a hob (reference 0.65–0.75: the low end, not filed). The gallery draws the hood on the floor; in play it hangs on the wall (a gallery problem, in the log). The bin's lid lifts 75° when a pot is scraped (`kitchen/extra/meal-fridge-stoveGas.json`, 111.4 s); the recycling bins' green lid seen "sideways" is it mid-swing.

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `pantry` | take a snack or ingredients; put the shopping away | grab a snack (2); an `ingredients` station | OK |
| `pantryTall` | the same | the same | OK |
| `wallCabinet` | take out plates and glasses; put clean dishes away | a `cupboard` station: clean dishes put away | OK; the reach is wrong (I-kitchen-03) |
| `wallCabinetGlass` | the same; show the glasses | the same | OK; as above |
| `wallCabinetCorner` | the same | the same | OK; as above |
| `rangeHood` | extract steam and light the hob while cooking | none (not decor in `reference.json`) | nothing ties it to the hob under it (I-kitchen-09) |
| `spiceRack` | take a spice while cooking | none | never reached for (I-kitchen-09) |
| `potRail` | take a pan or a utensil while cooking | none | never reached for (I-kitchen-09) |
| `kitchenShelfWall` | take a jar or a mug; show things | none | never reached for (I-kitchen-09) |
| `trashCan` | throw rubbish away; take the bag out | a `bin` station: plates scraped | OK |
| `recyclingBins` | sort the recycling | sort the recycling (4, not autonomous) | OK; needs nothing to sort (I-kitchen-11) |

## K6. Islands and bars (5)

Family: `tools/art/blender/families/kitchen_counters.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `kitchenIsland` Kitchen island | OK | OK | I-kitchen-04, I-kitchen-07, I-kitchen-10 | I-kitchen-04 S2 code; I-kitchen-07 S2 content; I-kitchen-10 S3 decision | `kitchen/sheets/obj-kitchenIsland.png`, `kitchen/play/c5/kitchenIsland-breakfast-01.png`, `kitchen/extra/seats/seats-barStool-drink-01.png` |
| `islandSmall` Small island | OK | n/a | I-kitchen-08 | I-kitchen-08 S3 content | `kitchen/sheets/obj-islandSmall.png` |
| `islandLong` Long island | OK | n/a | I-kitchen-08 | I-kitchen-08 S3 content | `kitchen/sheets/obj-islandLong.png` |
| `barSegment` Bar counter | M-kitchen-03 | n/a | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-barSegment.png` |
| `barEnd` Bar end | M-kitchen-03 | n/a | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-barEnd.png` |

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `kitchenIsland` | prepare food; eat a quick breakfast on stools; chat while cooking | make a sandwich, have breakfast at the island (20, standing at the cupboard side); `prep`; stools sit at its back | breakfast in the wrong place and pose (I-kitchen-07), with no food (I-kitchen-10) |
| `islandSmall` | prepare food; a quick breakfast | `prep` only | sandwich and breakfast missing (I-kitchen-08) |
| `islandLong` | the same | `prep` only | as above |
| `barSegment` | have a drink on a bar stool; mix drinks behind it | none of its own; a stool's drink needs it (`needsTable`, kind `bar`) | OK (the drink is the stool's) |
| `barEnd` | the same | the same | OK |

## K7. Dining tables (13)

Family: `tools/art/blender/families/dining.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `diningTable` Dining table | M-kitchen-01 | OK | I-held-01, I-office-01 | M-kitchen-01 S2 model (confirmed and widened); I-held-01 S2 code (seeded; held ledger: no cup); I-office-01 S3 renderer-table (office ledger: a chair facing it sits behind its own chairs) | `sheets/obj-diningTable.png`, `gallery-all/gallery-diningTable.png`, `scenes/dining-coffee-*.png`, `sheets/seat-diningM.png`, `sheets/seat-diningF.png` |
| `bistroSet` Bistro table | OK | A-residents-09 | I-kitchen-10, I-held-01, I-office-01 | A-residents-09 S2 clip (residents ledger); I-kitchen-10 S3 decision; I-held-01 S2 code (held ledger); I-office-01 S3 renderer-table (office ledger) | `kitchen/sheets/obj-bistroSet.png`, `kitchen/sheets/age-bistroSet.png` |
| `diningGlass` Glass dining table | M-kitchen-01 | A-residents-09 | I-kitchen-10, I-held-01, I-office-01 | M-kitchen-01 S2 model; A-residents-09 S2 clip (residents ledger); I-kitchen-10 S3 decision; I-held-01 S2 code, I-office-01 S3 renderer-table (other ledgers) | `kitchen/sheets/obj-diningGlass.png`, `kitchen/sheets/age-diningGlass.png` |
| `diningOak` Farmhouse dining table | M-kitchen-01 | A-residents-09 | I-kitchen-10, I-held-01, I-office-01 | M-kitchen-01 S2 model; A-residents-09 S2 clip (residents ledger); I-kitchen-10 S3 decision; I-held-01 S2 code, I-office-01 S3 renderer-table (other ledgers) | `kitchen/sheets/obj-diningOak.png`, `kitchen/sheets/age-diningOak.png`, `kitchen/play/c5/diningOak-dinner-01.png` |
| `tableSquare2` Square table for two | OK | n/a | OK | | `kitchen/sheets/obj-tableSquare2.png`, `kitchen/extra/seats/seats-chairArm-coffee-01.png` |
| `tableBistroRound` Round bistro table | OK | n/a | OK | | `kitchen/sheets/obj-tableBistroRound.png` |
| `tableDining4` Dining table for four | M-kitchen-03 | n/a | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-tableDining4.png`, `kitchen/extra/seats/seats-benchDining-coffee-01.png` |
| `tableDining6` Dining table for six | M-kitchen-03 | n/a | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-tableDining6.png` |
| `tableBanquet8` Banquet table | M-kitchen-03 | n/a | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-tableBanquet8.png` |
| `tableRound4` Round pedestal table | M-kitchen-03 | n/a | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-tableRound4.png`, `kitchen/extra/seats/seats-diningChair-coffee-01.png` |
| `tableSquare8` Square table for eight | OK | n/a | OK | | `kitchen/sheets/obj-tableSquare8.png` |
| `tableFarmhouse6` Farmhouse table | M-kitchen-03 | n/a | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-tableFarmhouse6.png` |
| `tableBarHigh` High café table | OK | n/a | OK | | `kitchen/sheets/obj-tableBarHigh.png`, `kitchen/play/c7/stoolBarHigh-drink-01.png` |

Notes: the plain tables' tops are 0.75 m (the high café table's 1.06 m), inside 0.71–0.76 (1.00–1.10). The glass set's 0.761 and the farmhouse set's 0.766 m are inside the 5 mm table-top tolerance (content 0.76 and 0.765), and their H of 1.10–1.11 m is the chair backs, 1 % over the set's range (not filed). A chair, bench or stool facing a plain table or the island sits at it, knees under the top (`kitchen/extra/seats/`). The chairs in the evidence agent's `play/c5/*-coffee-01.png` stood behind the base dining table's own chairs, a metre from its top: that is I-office-01's cause on a set (below, in "Evidence for findings in other ledgers").

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `diningTable` | eat a meal, have a coffee, homework or a puzzle | have a coffee (20, needs a coffee machine on the lot); meals are carried to it and eaten there | OK |
| `bistroSet` | breakfast; a coffee | have breakfast (20), have a coffee (20) | OK; breakfast with no food (I-kitchen-10) |
| `diningGlass` | a meal for two; a coffee | breakfast, coffee; cooked meals eaten at it | OK; as above |
| `diningOak` | family meals; a dinner for two; a coffee | breakfast, coffee, linger over dinner (40) | OK; as above |
| `tableSquare2`, `tableBistroRound` | a meal or a coffee for two | none of their own: eaten at from a chair facing them (`needsTable`) | OK |
| `tableDining4`, `tableDining6`, `tableBanquet8`, `tableRound4`, `tableSquare8`, `tableFarmhouse6` | family meals; a dinner party; homework or a puzzle | the same, from the chairs | OK (homework and games are the desks' and the game tables') |
| `tableBarHigh` | a drink standing or on a bar stool | from a stool facing it | OK |

## K8. Chairs and stools (12)

Family: `tools/art/blender/families/dining.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `barStool` Bar stool | OK | OK | I-kitchen-14, I-held-01 | I-kitchen-14 S3 content; I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-barStool.png`, `kitchen/sheets/age-barStool.png`, `kitchen/extra/seats/seats-barStool-drink-01.png` |
| `stoolCounter` Counter stool | OK | OK | I-kitchen-14, I-held-01 | I-kitchen-14 S3 content; I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-stoolCounter.png`, `kitchen/sheets/age-stoolCounter.png` |
| `stoolBarHigh` High bar stool | OK | OK | I-kitchen-14, I-held-01 | I-kitchen-14 S3 content; I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-stoolBarHigh.png`, `kitchen/play/c7/stoolBarHigh-drink-01.png` |
| `diningChair` Dining chair | OK | A-residents-09, A-residents-01 | I-held-01 | A-residents-09 S2 clip, A-residents-01 S2 code (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-diningChair.png`, `kitchen/sheets/age-diningChair.png`, `kitchen/extra/seats/seats-diningChair-coffee-01.png` |
| `chairWood` Wooden chair | OK | A-residents-09 | I-held-01 | A-residents-09 S2 clip (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-chairWood.png`, `kitchen/sheets/age-chairWood.png` |
| `chairUpholstered` Upholstered chair | OK | A-residents-09 | I-held-01 | A-residents-09 S2 clip (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-chairUpholstered.png`, `kitchen/sheets/age-chairUpholstered.png` |
| `chairModern` Moulded chair | OK | A-residents-09 | I-held-01 | A-residents-09 S2 clip (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-chairModern.png`, `kitchen/sheets/age-chairModern.png` |
| `chairWindsor` Windsor chair | OK | A-residents-09 | I-held-01 | A-residents-09 S2 clip (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-chairWindsor.png`, `kitchen/sheets/age-chairWindsor.png` |
| `chairBentwood` Bentwood chair | OK | A-residents-09 | I-held-01 | A-residents-09 S2 clip (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-chairBentwood.png`, `kitchen/sheets/age-chairBentwood.png` |
| `chairFolding` Folding chair | OK | A-residents-09 | I-held-01 | A-residents-09 S2 clip (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-chairFolding.png`, `kitchen/sheets/age-chairFolding.png` |
| `chairArm` Carver chair | M-kitchen-05 | A-residents-09 | I-held-01 | M-kitchen-05 S3 model; A-residents-09 S2 clip (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-chairArm.png`, `kitchen/extra/seats/seats-chairArm-coffee-01.png` |
| `benchDining` Dining bench | OK | A-residents-09 | I-held-01 | A-residents-09 S2 clip (residents ledger); I-held-01 S2 code (held ledger) | `kitchen/sheets/obj-benchDining.png`, `kitchen/extra/seats/seats-benchDining-coffee-01.png` |

Notes: seats against the reference and the renderer table (M2): the dining chairs 0.46–0.48 m (content and `furniture.json` agree within 1 cm; the upholstered chair's 0.483 is inside the tolerance); the counter stools 0.69–0.70 (counter-stool range 0.58–0.71; the bar stool's cushion is 0.74 with its sitter at 0.72, a 2 cm sink into the cushion), the high stool 0.81 (0.74–0.81). The seat-to-top drop is 27–28 cm at the tables and 25 cm at the counter and the bar (ergonomic 25–30). The stools swivel 20° and carry the sitter (`ride`). `pnpm check:clips` passes the kitchen's clips; its known faults are the seated feet (A-residents-09). The poses page shows the bench's man hovering and the bistro sitter in the table: that page builds `catalog2/dining.json` objects on a 1 × 1 footprint (a tool problem, in the log); in play both sit right.

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `barStool`, `stoolCounter`, `stoolBarHigh` | sit at a counter, island or bar; have a drink | sit (20), have a drink (15, needs a counter, bar or high table it faces) | OK; each takes both heights (I-kitchen-14) |
| the eight dining chairs and `benchDining` | sit at a table to eat; have a coffee | sit (20), have a coffee (20, needs a table it faces and a coffee machine); cooked meals eaten from them | OK |

## K9. Dining storage (4)

Family: `tools/art/blender/families/dining.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sideboard` Walnut sideboard | M-kitchen-03 | OK | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-sideboard.png`, `kitchen/clips/clips-sideboard-all-t99.png` |
| `buffet` Painted buffet | M-kitchen-03 | OK | OK | M-kitchen-03 S3 decision | `kitchen/sheets/obj-buffet.png` |
| `chinaCabinet` China cabinet | OK | OK | I-kitchen-03 | I-kitchen-03 S2 renderer-table | `kitchen/sheets/obj-chinaCabinet.png`, `kitchen/clips/clips-chinaCabinet-all-t99.png` |
| `barCart` Bar cart | OK | I-kitchen-13 | I-kitchen-13, I-kitchen-06 | I-kitchen-13 S3 code; I-kitchen-06 S1 code | `kitchen/sheets/obj-barCart.png`, `kitchen/play/c4/barCart-mix-01.png`, `kitchen/extra/play-child/child-barCart-mix-01.png` |

Real uses against the content's (I1):

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `sideboard`, `buffet` | store plates and table linen; serve from the top | `cupboard` stations: clean dishes put away behind a door | OK |
| `chinaCabinet` | show the china; take out the good plates | a `cupboard` station | OK; the reach is low (I-kitchen-03) |
| `barCart` | mix a drink; wheel it to the guests | mix a drink (10) | OK for adults; the shaker moves by itself (I-kitchen-13); a child mixes one (I-kitchen-06) |

## Cooking: the station flow at the kitchen objects (I13)

Scripted on `review-stage` (`kitchen/extra/meal-fridge-*.json` hold the resident's and every object's state every 200 ms): a meal started at the fridge, with a gas range, a bin, a dishwasher and the base dining table bought, then two clear-aways; the same meal with only a microwave, then only an oven tower, as the cooker; a snack cleared with a sink and a wall cabinet. The order is a believable kitchen: fridge (door opens, ingredients taken) → counter (chop, 10 min) → cooker (stir, 20 min) → the pot set on the counter, a plate served from it → the plate carried to the table and eaten seated (25 min) → the plate carried to the dishes station, the pot scraped into the bin (its lid opens) and washed → put away in the cupboard or the counter. What goes wrong at the kitchen objects is filed: the carried crate and bowl stay in both hands while chopping and stirring (I-kitchen-04); the hob, the oven and the microwave don't move when the recipe was started elsewhere (A-kitchen-02); the Cook step is a frying pan stirred whatever the station, so a microwave shows a pan on its worktop, an oven tower a pan through its shut door, and baking a pan on the hob (I-kitchen-05); the pan is not on the burner that is lit (I-systems-01); the dishwasher stays shut while it is loaded (A-kitchen-01); dishes go into a wall cabinet with the hands at the waist (I-kitchen-03).

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### I-kitchen-01 — The freezer doors had a clip but no use

- **Severity:** S3
- **Rubric:** I1, I7
- **Fix class:** content
- **Where:** `web/public/content/base.json:2363`, `web/public/content/furniture.json:198`, `web/public/content/catalog2/kitchen.json:568`
- **Evidence:** Fixed in 0.3.0 by "Get some ice cream" (`frozenTreat`, CHANGELOG 0.3.0 Objects). Confirm that `freezer_open` plays for `frozenTreat` only, on all three fridges, and close.
  Pilot, `fridge`: `frozenTreat` plays `freezer_open` alone and `fridge_open` never moves (play.jsonl: `fridge_open` events `[]`); `snack` and `meal` play `fridge_open` alone (`play/fridge-frozenTreat-01.png`, `play/fridge-snack-01.png`). The doors swing 118° (freezer) and 102° (fridge) about the left edge, inside the reference 90–125° (`kitchen.py:71-72`). The motions' `drink` key names no use of `fridge` (T4 flag): harmless, it is the shared motions table that `fridgeBuiltIn`'s `coldDrink` uses (`kitchen.py:720`).
  Kitchen wave: `fridgeBuiltIn` (`furniture.json:198`) and `fridgeFrenchDoor` (`catalog2/kitchen.json:568`) the same: `frozenTreat` moves `freezer_open` alone (the bottom door; the French-door fridge's drawer, 35 cm), `fridge_open` has no events; snack, meal, gourmet and cold drink move the fridge door (`door_right` alone for the French-door fridge's drink). `kitchen/play/c6/play.jsonl`, `kitchen/play/c6/fridgeBuiltIn-frozenTreat-01.png`, `kitchen/play/c6/fridgeFrenchDoor-frozenTreat-01.png`.
- **Status:** closed (fixed in 0.3.0: confirmed on `fridge` by the pilot, on `fridgeBuiltIn` and `fridgeFrenchDoor` by the kitchen wave)
- **Source:** `plan/release-0.3.0.md` §1.4; `docs/objects/open.md` Still open › Content and sim; `CHANGELOG.md` 0.3.0 Objects; pilot (`check.mjs play --state review-stage`); wave: kitchen

### M-kitchen-01 — The base dining table is a 38 cm deep plank with both diners side by side

> Kitchen wave: confirmed and widened to the glass and farmhouse dining sets.

- **Severity:** S2
- **Rubric:** M1, M3, M6, I2
- **Fix class:** model
- **Where:** `tools/art/blender/families/dining.py:259` (`diningTable`: the top 1.62 × 0.38 at `y = 0.765`, both chairs behind it at `z = -0.2`); `web/public/content/base.json:3327` (`surface.size [1.62, 0.38]`, `slots 2`); widened: `web/public/content/furniture.json:945` (`diningGlass`, `surface.size [1.7, 0.62]`), `:1005` (`diningOak`, `[1.76, 0.66]`), both built in `dining.py` with their two chairs on one long side
- **Evidence:** `gallery-all/gallery-diningTable.png`, `sheets/seat-diningM.png`, `scenes/dining-coffee-02.png`. The top is 0.38 m deep (a dining table is 0.75–1.0 m, `reference.json` `diningTable4`), with two chairs side by side along one long edge and nothing across: it reads as a console or a breakfast bar, not a table people eat at facing each other. Its top is 0.765 m, 5 mm over the 0.71–0.76 range (inside measuring noise, but the strict rule says out). The sizes table's "H 1.03 > 0.86" is the chair backs, not the table (the kind's height range is for a table alone).
  Kitchen wave: the glass set (top 0.62 m deep) and the farmhouse set (0.66 m) have the same layout, both chairs along the back edge, side by side, nothing across (`kitchen/sheets/obj-diningGlass.png`, `kitchen/sheets/obj-diningOak.png`); a couple at breakfast or dinner sits shoulder to shoulder facing the room. Their tops are deeper than the base table's but still under the 0.75 m a table for two people facing each other needs (the reference's `diningSet` 0.75–1.8 m overall depth is met only with the chairs). With the 5 mm tolerance the 0.765 m tops are now inside.
- **Status:** confirmed and widened (kitchen wave: `diningGlass`, `diningOak`)
- **Source:** pilot; wave: kitchen

### I-kitchen-02 — The base set lacks the second uses its siblings have

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/base.json:2363` (`fridge`: no `coldDrink`), `:2580` (`sofa`: no `doze`), `:2704` (`armchair`: no `read`), `:2449` (`sink`: no `brushTeeth`)
- **Evidence:** Every other full fridge has "Get a cold drink" (`fridgeBuiltIn`, `fridgeFrenchDoor`, `fridgeRetro`); `loveseat`, `sofaThree`, `sofaDesign`, `sofaThreeSeat` have `doze`; `armchairLeather` and `armchairWingback` have `read`; `mirrorCabinet` has "Brush your teeth". The starter-house versions are the plainest in use, though they are what every new household has. The real uses (`reference.json`: fridge "get a snack or a drink", sofa "doze off", armchair "read", basin "brush teeth") are listed in the I1 table of each ledger.
- **Status:** fixed (eaca9ed8)
- **After:** The base fridge's cold drink, the sofa's doze, the armchair's sitting read and the sink's "Brush your teeth" (3 min): each played (`bench-results/review/fixes/content/g09/`). Balance and soak: see I-bathroom-06.
- **Source:** pilot (`cargo review uses` tables against `reference.json` `uses`)

### M-kitchen-02 — Both dishwashers are a metre wide

- **Severity:** S2
- **Rubric:** M1, M3, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/kitchen_cooking.py:423` (`dishwasher`: `w=1.0`, a 0.90 m door), `:429` (`dishwasherCompact`: `w=1.0`), sizes at `:472`, `:476`
- **Evidence:** `kitchen/sheets/obj-dishwasher.png`, `kitchen/clips/clips-dishwasher-all-t99.png`, `kitchen/sheets/obj-dishwasherCompact.png`. The whole 1 m front is the dishwasher's door (0.90 × 0.72 m), dropping open across the full tile; real dishwashers are 45 or 60 cm wide (`reference.json` `dishwasher` 0.45–0.60, sizes table "W 1.000 > 0.660"). The wine fridge in the same 1 m unit is built right: a 45 cm cooler beside a drawer stack (`kitchen.py:674`). Open, the door reads as an oven's or a chest freezer's lid. A 60 cm dishwasher with a 40 cm cabinet or drawer beside it would keep the tile.
- **Status:** open
- **Source:** wave: kitchen (`cargo review sizes`, sheets)

### M-kitchen-03 — Tables, storage and cookers built to fill whole tiles run past the real range

- **Severity:** S3
- **Rubric:** M1, M11
- **Fix class:** decision
- **Where:** `tools/art/blender/families/dining.py:965` (`tableDining4` 1.84 m), `:967` (`tableDining6` 2.84), `:969` (`tableBanquet8` 3.84), `:971` (`tableRound4` 1.52 across), `:975` (`tableFarmhouse6` 2.86), `:1004` (`sideboard` 0.668 deep, 1.10 m high with its vase), `:1006` (`buffet` 0.668 deep); `kitchen_cooking.py:447` (`wallOven` 0.70 wide), `:451` (`rangeCooker` 1.58 × 1.13); `kitchen_counters.py:740`, `:742` (`barSegment`, `barEnd` 0.89 deep); `kitchen.py:724` (`fridgeMini` 1.00 high)
- **Evidence:** `bench-results/review/tables/sizes-kitchen.md` (each over the reference range plus 10 %): the table "for four" is 1.84 m long (a four-seat table is 1.20–1.60; 1.84 seats six), the tables "for six" 2.84–2.86 (1.60–2.20: they seat eight to ten), the banquet table 3.84 (2.20–3.00), the round table 1.52 m across (0.90–1.25: it seats six); the sideboard and buffet are as deep as a kitchen base unit, 0.67 m (0.40–0.55); the oven tower is 0.70 m wide (0.58–0.62, 3 % past the tolerance), the range cooker 1.58 m (0.90–1.10; real ones reach 1.5 m but are rare), the bars 0.89 m deep (0.40–0.75), the mini fridge 1.00 m high with its board on top (0.50–0.88). Each takes its footprint minus a margin; side by side with the chairs and residents (`kitchen/sheets/obj-tableBanquet8.png`, `obj-tableDining4.png`) they read as large, not wrong. Decide whether whole-tile sizes are the style here (then widen the reference ranges for these kinds) or whether the names change ("table for six" for the 1.84 m one) or the models shrink inside their tiles.
- **Status:** fixed (b3f6b82d)
- **After:** Tables named by their seats ("Dining table for six" 1.84 m, "for eight" 2.84, the farmhouse table for eight, the banquet table for twelve, the round table for six; kinds `diningTable6`, `diningTable8`, new `diningTable12` and `roundTable6`); whole-tile sideboards, oven tower, range cooker and bars widened in `reference.json`: kitchen 24 → 13 outside. The sideboard's H 1.10 is its vase (tooling) and the mini fridge's 1.00 m is not in Q-08's list.
- **Source:** wave: kitchen (`cargo review sizes`)

### M-kitchen-04 — The built-in fridge is a free-standing fridge with a dispenser

- **Severity:** S3
- **Rubric:** M1, M6
- **Fix class:** decision
- **Where:** `tools/art/blender/families/kitchen.py:334` (`_fridgeBuiltIn`), `:727` (size 0.84 × 2.025 × 0.656); `web/public/content/furniture.json:198` (name "Built-in fridge")
- **Evidence:** `kitchen/sheets/obj-fridgeBuiltIn.png`: a 0.84 m wide, 2.03 m tall white cabinet with a fridge door over a freezer door, long bar handles and a water dispenser panel on the door; it stands free like the other fridges. An integrated fridge is 0.58–0.62 m wide behind a kitchen-unit door with no dispenser (`reference.json` `fridgeIntegrated`; sizes table "W 0.840 > 0.682"). As a counter-depth bottom-freezer fridge (`fridgeAmerican` 0.83–0.92 wide) it is inside every range. Decide: rename it (and move it to that kind), or rebuild it as an integrated 60 cm unit with a cabinet front and no dispenser.
- **Status:** fixed (b3f6b82d)
- **After:** "Counter-depth fridge", kind `fridgeAmerican`: inside every range.
- **Source:** wave: kitchen

### M-kitchen-05 — The carver chair's seat is 4 cm higher than a dining chair's

- **Severity:** S3
- **Rubric:** M2, M11, A1
- **Fix class:** model
- **Where:** `tools/art/blender/families/dining.py:676` (`chairArm`: cushion centred at `y 0.49`, 6 cm thick: top 0.52); `web/public/content/catalog2/dining.json:530` (`seat.height 0.5`)
- **Evidence:** Sizes table "seat 0.521 > 0.480 (0.42–0.48)"; heights check "seat 0.50 m (content), the model 0.521 m there: 2.1 cm higher" (`agent-rust/heights.txt`). A carver sits at the head of the same table as the side chairs, whose seats are 0.46–0.48 m (`reference.json` `carverChair` 42–48 cm): at 0.52 m the knees come within 20 cm of a 0.75 m top. The sitter is placed 2 cm into the cushion, which reads as a soft seat (`kitchen/extra/seats/seats-chairArm-coffee-01.png`); the height itself is the problem. Lower the seat and cushion by 4 cm (cushion top 0.48, content 0.47).
- **Status:** open
- **Source:** wave: kitchen (`cargo review sizes`, heights check)

### M-kitchen-06 — Small faults on the wine fridge, the French-door fridge and the counter end

- **Severity:** S3
- **Rubric:** M9, M7, A8
- **Fix class:** model
- **Where:** `tools/art/blender/families/kitchen.py:674` (`_wineFridge`: the door's glass is `M.smoked`, the moving "top rack" a drawer front at `(0.30, 0.74)` beside the cooler); `kitchen.py:379` (`_fridgeFrenchDoor`); `kitchen_counters.py:151` (`counterEnd`)
- **Evidence:**
  - Wine fridge: the "smoked-glass" door draws as an opaque black panel, closed, so the bottles behind it never show (`kitchen/sheets/obj-wineFridge.png`, gallery); a wine cooler's selling point is its glass door. The part that slides out with the door is the top of the drawer stack beside the cooler, which reads as the counter's drawer, not a wine rack (`kitchen/extra/play-child/child-wineFridge-wine-01.png`, `kitchen/clips/clips-wineFridge-all-t99.png`).
  - French-door fridge: with one door open for a cold drink, the closed door looks set back behind the side panel's edge, inside the body, rather than flush with the front (`kitchen/play/c6/fridgeFrenchDoor-coldDrink-01.png`).
  - Counter end: a seam line crosses the worktop about a quarter in, where the end shelves meet the cabinet (`kitchen/play/c2/counterEnd-no-uses.png`).
- **Status:** open
- **Source:** wave: kitchen (evidence agent's "Seen" notes, checked against the sheets)

### A-kitchen-01 — The dishwasher stays shut while it is loaded

- **Severity:** S2
- **Rubric:** A10, I7, I13, A13
- **Fix class:** motion
- **Where:** `web/public/assets/packs/kitchen_cooking/manifest.json:398`, `:442` (`dishwasher`, `dishwasherCompact` motions: `uses: {dishes: door_open}`, `hold 2.5`); `web/src/render/babylon/objectMotion.ts:413` (clips keyed by interaction id or animation tag); `crates/sim-core/src/snapshot.rs:156` (clearing away is `Doing::plain`: no interaction, tag `wash`); `web/src/audio/world.ts:189` (`wash` with no object def: the washing-up water)
- **Evidence:** `kitchen/extra/meal-fridge-stoveGas.json` and `-15.png`, `-17.png`: with a dishwasher as the lot's only `dishes` station, the dirty plate (98.4–103.1 s) and the scraped pot (113.1–118.1 s) are each "washed" at it for 5 s with the `wash` clip (hands working at the shut door, holding the plate) and no clip on the dishwasher at all; the sound is a running tap. On its own use "Load the dishwasher" the door drops and the rack slides out, then shuts 2.5 s in (its `hold`; `kitchen/play/c1/play.jsonl`: closing from +2.5 s), while the load lasts 10 game minutes (10 s at speed 1, longer at the slower paces): for most of it the loader works at a shut door. Add `wash` to both dishwashers' `uses` (or key the dishes station), drop the `hold` or make it the use's length, and give clearing away at a dishwasher its own sound (or none).
- **Status:** open
- **Source:** wave: kitchen (`scratch/scene.mjs` recipe scene, `check.mjs play`)

### A-kitchen-02 — moved to systems.md

### A-kitchen-03 — One person washing at the double sink runs both taps

- **Severity:** S3
- **Rubric:** A10, I7
- **Fix class:** motion
- **Where:** `web/public/assets/packs/kitchen/manifest.json:140` (`counterSinkDouble` motions: `wash → water_on`, one clip that grows both streams)
- **Evidence:** `kitchen/play/c2/counterSinkDouble-wash-01.png`, `kitchen/clips/clips-counterSinkDouble-all-t99.png`: one resident at the left bowl, water running from both bridge taps. The sink has two slots; the motions support a clip per slot (`slot`, as the bunk bed's duvets), so each tap could run for the person at its bowl.
- **Status:** open
- **Source:** wave: kitchen

### I-kitchen-03 — moved to residents.md

### I-kitchen-04 — moved to systems.md

### I-kitchen-05 — moved to systems.md

### I-kitchen-06 — moved to systems.md

### I-kitchen-07 — Breakfast at the kitchen island is eaten standing at its cupboards

- **Severity:** S2
- **Rubric:** I2, I3
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:784` (`kitchenIsland` "Have breakfast at the island": no `pose`, `approach` front, `slots 2`)
- **Evidence:** `kitchen/play/c5/kitchenIsland-breakfast-01.png`: the resident stands at the island's cupboard side for 20 minutes, eating from empty hands with the eat clip. The island's worktop overhangs its back for stools (`docs/objects/kitchen.md` K6; a stool at the back is pulled in to it, `kitchen/extra/seats/seats-barStool-drink-01.png`), which is where a quick breakfast at an island is eaten (`reference.json` `kitchenIsland`: "eat a quick breakfast on stools"). Make it a stool's use when one stands at the island's back (`seatNear`, as the TV's watch), or sit at the back.
- **Status:** fixed (9a5cc144)
- **After:** Breakfast `seatNear` 3: eaten on the stool at the island's back (`bench-results/review/fixes/content/g03/after-kitchenIsland-breakfast-00.png`); 2.5 m missed it (the stool's approach tile is 3 m from the island's front tile).
- **Source:** wave: kitchen

### I-kitchen-08 — Kitchen pieces without the uses their siblings have

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:147` (`fridgeMini`: snack and meal, no `coldDrink`); `web/public/content/catalog2/kitchen.json:5` (`counterEnd`), `:34` (`counterCornerOuter`), `:63` (`counterDrawers`), `:92` (`counterOpenShelf`), `:295` (`islandSmall`), `:324` (`islandLong`): `prep` stations with no use
- **Evidence:** `bench-results/review/tables/uses-kitchen.md`. The mini fridge is the one fridge for cold drinks above all, and the retro mini fridge beside it has "Pour a cold drink"; the four counters and two islands are prep stations for a meal but don't offer "Make a sandwich", which the counter, the corner counter, the butcher block, the marble counter and the kitchen island do, and the islands lack the kitchen island's breakfast. A player who buys a run of the plainer counters gets no sandwich from it. The same pattern as I-kitchen-02 for the base set.
- **Status:** fixed (eaca9ed8)
- **After:** The mini fridge's cold drink; a sandwich on the counter end, the outer corner, the drawer and open-shelf counters and both islands, and the islands' breakfast (`seatNear` 3). All played (`bench-results/review/fixes/content/g09/`). `cargo review uses` flags the copies' 9 and 5 minutes as their siblings' are (short by design).
- **Source:** wave: kitchen (`cargo review uses`)

### I-kitchen-09 — The cooking wall takes no part in cooking

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `web/public/content/furniture.json:768` (`rangeHood`), `web/public/content/catalog2/kitchen.json:967` (`spiceRack`), `:983` (`potRail`), `:999` (`kitchenShelfWall`): no use, no station; not `decor` kinds in `docs/review/reference.json`
- **Evidence:** `kitchen/sheets/obj-rangeHood.png`, `obj-spiceRack.png`, `obj-potRail.png`, `obj-kitchenShelfWall.png`. The hood has a light strip and a filter but stays dark and silent while someone cooks on the hob under it; the spice rack, the pot rail's pans and the shelves' jars are never reached for, though the recipes take a pan and the cook stirs one. Owner default: decor stays decor. Decide whether the hood comes on (its light and a fan's hum) while the hob below is in use, and whether the rack, rail and shelves become optional recipe stations (a spice taken during the Cook step), or stay decor (then mark their kinds `decor` in `reference.json`).
- **Status:** fixed (198e90d9) for the spice rack, pot rail and shelves (decor, Q-19); the hood's light and hum later
- **After:** Their kinds are `decor` in `reference.json`: `cargo review uses` no longer flags them. The range hood keeps its flag until its light and hum come with the hob (polish, code).
- **Source:** wave: kitchen (I1)

### I-kitchen-10 — Breakfast and dinner at a dining set come from nowhere

- **Severity:** S3
- **Rubric:** I1, I5, I7
- **Fix class:** decision
- **Where:** `web/public/content/furniture.json:887` (`bistroSet` breakfast), `:945` (`diningGlass` breakfast), `:1005` (`diningOak` breakfast and "Linger over dinner"), `:784` (`kitchenIsland` breakfast): `anim: eat`, hunger 0.3 (0.45 for dinner), no `eat` item, no recipe
- **Evidence:** `kitchen/play/c5/bistroSet-breakfast-01.png`, `diningOak-dinner-01.png`, `kitchenIsland-breakfast-01.png`: 20 or 40 minutes of the eat clip with nothing cooked, fetched or put on the table (the plates there are the model's own place settings), against the fridge's snack, the toaster's toast and every recipe, which make an item that is carried and eaten. Fine as a game shortcut if it's meant; else make these uses eat a served item (the meal on the lot) or fetch something first. Decide.
- **Status:** open
- **Source:** wave: kitchen (I1)

### I-kitchen-11 — Washing up, loading the dishwasher and sorting the recycling need nothing to wash or sort

- **Severity:** S3
- **Rubric:** I16
- **Fix class:** code
- **Where:** `web/public/content/furniture.json:669` (`farmhouseSink` `washUp`), `web/public/content/catalog2/kitchen.json:197` (`counterSinkDouble` `washUp`), `web/public/content/furniture.json:558` (`dishwasher` `dishes`), `web/public/content/catalog2/kitchen.json:840` (`dishwasherCompact`), `:901` (`recyclingBins` `sortRecycling`)
- **Evidence:** `kitchen/play/c1/farmhouseSink-washUp-01.png`, `dishwasher-dishes-01.png`, `kitchen/play/c3/recyclingBins-sortRecycling-01.png`: each runs on the bare stage with nothing to wash or sort, the water running into an empty bowl and the dishwasher loaded from empty hands. They aren't autonomous, so only the player starts them, but a player asked to wash up with nothing to wash sees a mime. Clearing away after a meal already washes the real plates at these stations. A condition (a dirty item on the lot, or recycling) is new behaviour: `code`.
- **Status:** open
- **Source:** wave: kitchen (I16)

### I-kitchen-12 — The kitchen's machines sound like a frying pan or not at all

> Consolidation (2026-10-21): closed as a duplicate of I-bathroom-04 (systems ledger), which takes its items and evidence.

- **Severity:** S3
- **Rubric:** I9, A13
- **Fix class:** code
- **Where:** `web/src/audio/world.ts:34` (`ACTION_SOUNDS`: `cook`, `stir`, `flip` → `sizzle`, by tag only), `:185` (`soundOf`: only `wash` and `clean` look at the object)
- **Evidence:** `bench-results/review/tables/uses-kitchen.md` (Sound column): "Make toast", "Blend a smoothie" and "Heat up leftovers" (`anim: cook`) play the frying sizzle; the Cook step at a microwave or an oven tower sizzles too (I-kitchen-05). Silent: the dishwasher running, the coffee machine grinding and hissing, the blender's motor, the microwave's hum and ping, the toaster's pop, the fridge door and the bin lid. A sound per object (or per object kind) for the `cook` tag, and the missing ones, are new sounds: `code`.
- **Status:** closed (duplicate of I-bathroom-04)
- **Source:** wave: kitchen (I9)

### I-kitchen-13 — The bar cart's shaker lifts and tips by itself

- **Severity:** S3
- **Rubric:** I7, A10, A1
- **Fix class:** code
- **Where:** `web/public/assets/packs/dining/manifest.json:853` (`barCart` motions: `mix → shake`, the shaker part rising 10 cm and tipping 30°); `web/public/content/catalog2/dining.json:845` (`mix`: `anim: drink`)
- **Evidence:** `kitchen/play/c4/barCart-mix-01.png`, `kitchen/clips/clips-barCart-all-t99.png`: the resident stands half a metre off, sipping at nothing with the drink clip, while the shaker rises off the cart's top and tips towards them on its own. Shaking a cocktail is both hands on the shaker. The game already draws an object's part in the hands (`held`: the barbell); a shaking clip and the shaker held would fix it. Props in hand are new behaviour (§9 decision 3): `code`. Related: I-held-01 (held ledger).
- **Status:** open
- **Source:** wave: kitchen

### I-kitchen-14 — Counter and bar stools take both heights

- **Severity:** S3
- **Rubric:** I2, M11
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:838` (`barStool` `seatAt: [bar, counter]`, seat 0.72), `web/public/content/catalog2/dining.json:638` (`stoolCounter` `[counter, bar]`, 0.68), `:691` (`stoolBarHigh` `[bar, counter]`, 0.80)
- **Evidence:** Counters and islands are 0.95 m, bars and the high café table 1.06 m (content `surface`). A seat 25–30 cm under the top fits (`reference.json`: counter stools for 0.90–0.95 m tops, bar stools for 1.02–1.07). The 0.68–0.72 m stools at a 1.06 m bar sit 34–38 cm under it (the top at the chest), and the 0.80 m high stool at a 0.95 m counter sits 15 cm under it, where the thighs don't fit under an overhang (the tuck stops the knees short instead). Housegen and players can pair either. Let `barStool` and `stoolCounter` seat at `counter`, and `stoolBarHigh` at `bar`.
- **Status:** fixed (543c6121)
- **After:** `barStool` and `stoolCounter` seat at `counter` only, `stoolBarHigh` at `bar` only; the breakfast counter's 0.95 m top is a `counter`; generated islands get counter stools only. Play: the counter stool's drink at a counter and the high stool's at a bar run; the bar stool at a bar and the high stool at a counter say "needs a table" (`bench-results/review/fixes/content/g07/`).
- **Source:** wave: kitchen (content against the reference)

## Evidence for findings in other ledgers

Kitchen-wave evidence for causes filed elsewhere; not filed again here. Carried into those findings at the consolidation (2026-10-21).

- **I-systems-01** (systems ledger; seeded): confirmed and widened at the hobs. The pan is drawn at the station's middle, a little towards the cook, not on the burner the clip lights: on the gas range the flame ring is at the front right and the pan at the front left (`kitchen/play/c5/stoveGas-cook-01.png`, `kitchen/extra/meal-fridge-stoveGas-06.png`); on the hob counter the glowing ring is at the back and the pan at the front (`kitchen/play/c5/cooktopCounter-cook-01.png`); on the range cooker the ring is in the middle and the pan to its left (`kitchen/play/c5/rangeCooker-cook-01.png`). Items: `stoveElectric`, `stoveGas`, `rangeCooker`, `cooktopCounter`. A per-model hob spot (the burner the clip lights) would fix both.
- **I-systems-04** (systems ledger; seeded): applies to every kitchen cooker (`stoveElectric`, `stoveGas`, `wallOven`, `rangeCooker`, `cooktopCounter`); the alcohol uses are filed here as I-kitchen-06.
- **I-systems-03** (systems ledger; seeded): seen on the bare stage: a snack's plate left on the floor with no top, sink or bin about (`kitchen/play/c1/rangeHood-no-uses.png`, bottom middle). With a counter, the plate goes on the counter (`kitchen/extra/putaway-wallCabinet.json`).
- **I-office-01** (office ledger; seeded): widened to the dining sets. A chair bought facing the base dining table, the glass or the farmhouse set (each with its own chairs built in at the back) passes `needsTable` and sits behind or beside the set's own chair, about a metre from the top, for its coffee (`kitchen/play/c5/chairModern-coffee-01.png`, `benchDining-coffee-01.png`, `chairWindsor-coffee-01.png`); the same chairs at plain tables sit right (`kitchen/extra/seats/`). Items: `diningTable`, `diningGlass`, `diningOak`, `bistroSet`.
- **I-held-01** (held ledger): no cup, glass or plate for the coffee (tables, chairs, coffee machine), the drinks (stools, fridges' cold drink, wine), breakfast and dinner (`kitchen/play/c5/diningGlass-coffee-01.png`, `kitchen/play/c7/stoolBarHigh-drink-01.png`). Cooked meals do show a plate (`kitchen/extra/meal-fridge-stoveGas-11.png`).
- **A-residents-09** (residents ledger): the chairs' and sets' rows; `pnpm check:clips` still lists the teen and elder women's feet off a 0.45 m seat.
- **I-kids-03** (systems ledger): the child's wine and cocktail above were direct uses; free will has no rule either way (I-kitchen-06).
