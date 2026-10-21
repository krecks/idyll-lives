# Real-world reference

The sizes, uses, durations and mechanisms of the real things the game's models stand for. Every model, animation
and interaction in the authenticity review ([plan/authenticity-review.md](../../plan/authenticity-review.md) §3.4)
is checked against this page. Its machine-readable twin is [reference.json](reference.json); this page is written
from it, so the two say the same.

## How to use this

- `cargo review sizes` reads `reference.json`, finds each object's kind and compares the built W × H × D (from the
  `cargo check-glb` summary) and the functional heights with the ranges here. It prints the outliers as a table,
  each value to the millimetre: the first pass of M1, M2 and M11. What it measures:
  - **Seat, top, mattress:** the model's face looking up nearest the height the game uses (content `seat` and
    `surface`, furniture.json `seats` and `beds`). A bed's mattress is measured at slot 0 only (a crib's slot 1 is
    its changing table).
  - **A top the content hasn't:** for a kind with a `surface` range, the model's top all the same: on nine vertical
    lines (the middle of the model seen from above, and a quarter of its width and depth round it), leaving out
    those within 0.3 m of a seat's hips, the face looking up nearest the range on each; of those, the highest
    that doesn't stand on what another line found (a vase or a tool on the top stands on it; a basin's floor is
    lower than the worktop round it). It counts as the top within 20 % of the range's ends; past that the table
    says "no top to measure" and names the nearest face.
  - **H (table):** for a kind with a `surface` and no `seat` range (a table or a desk alone) whose object is sat at
    and whose model carries the seats (the base `diningTable`'s two chairs), H leaves out what rises beside the top
    higher than it: the faces no more than 2 cm over the top or over the top's own extent (what stands on it, a vase
    or a screen, stays). A set's kind (with a `seat` range: "with its chairs") keeps the whole model's H.
  - **H (top step):** for a stair (content `stairs`), the highest tread along its middle, which is the storey it
    climbs (`h` here), not the handrail over it; the table also gives the number of steps and the rise of each.
- A kind is one real-world type, shared by every object of that type, so all the dining chairs are checked against
  one range and against each other.
- **Overall sizes** (`w`, `h`, `d`) are real-world ranges. The tool adds the 10 % `tolerance` itself (owner
  decision 1: overall size may stray about ±10 % for readability), so the ranges here are not widened for the style.
- **Functional heights** (`seat`, `surface`, `mattress`) get no tolerance. They must be real.
- Every measure is optional. A kind gives only what means something for it.
- An object finds its kind through `ids` (its id, or a model key for things that aren't buyable) or else through
  `groups` (its content `group`). Only three groups are uniform enough to map whole (`nightstands`, `chairs`,
  `toilets`); every other object is mapped by id. No object is in both, so the order doesn't matter today; if it
  ever does, `ids` wins.
- `decor` lists the kinds where no use is expected. Use it for I1: an object of a decor kind with no use is fine;
  any other object with no use is a candidate "missing use".
- `minutes` are real durations. The game's durations are short on purpose (owner decision 2), so the ledger
  compares only outliers against similar uses.

### Conventions

- Metres and degrees. `w` is across the front, `d` front to back, `h` overall height, as the resident faces the
  front (+Z in game).
- `seat` is the top of the seat cushion, `surface` the top of a work or table top (for a cooker, the hob; for a
  basin, the rim; for a pool table, the bed), `mattress` the top of the mattress.
- Mechanism keys are hinge angles in degrees, except keys ending in `Travel` (metres) and `drum` and `blades`
  (rpm). An angle range is the real working range: a fridge door opens 90° to get the shelves out, and up to
  about 125° at full swing.
- A composite thing (a desk with its chair built in, a TV wall with seats in front, a fireplace with two chairs)
  takes the kind of the main thing, with the ranges of the whole set where that is clear. The tables below say so
  in the label.

## Residents

Heights are the 5th to 95th percentile over the life stage, girls' or women's low end to boys' or men's high end.
Children and adults come from the US NHANES 2015–2018 tables (`cdcAnthro`), babies from the WHO length-for-age
standard, 3rd to 97th percentile (`whoLength`). The game's stage ages are `base.json` `life.stages`.

| Stage | Ages | Real height (m) | Median, girl / boy (m) | Rig height, f / m (m) | Real head-to-height | Rig head-to-height (`rig.rs`) |
|---|---|---|---|---|---|---|
| `resident.baby` | 0–2 | 0.45–0.94 | birth 0.49 / 0.50; 1 y 0.74 / 0.76; 2 y 0.86 / 0.88 | 0.675 / 0.663 (about 6–9 months) | about 1:4 | 1:3.4 |
| `resident.child` | 2–12 | 0.83–1.70 | 6 y 1.19 / 1.18; 12 y 1.54 / 1.54 | 1.094 / 1.128 (about 5 years) | 1:5 at 2–5, 1:6 at 5–7 | 1:4.6 |
| `resident.teen` | 13–17 | 1.45–1.88 | 13 y 1.58 / 1.64; 17 y 1.62 / 1.75 | 1.442 / 1.512 | about 1:7 from 14–15 | 1:6.0 |
| `resident.youngAdult` | 18–29 | 1.50–1.88 | 1.63 / 1.76 | 1.67 / 1.77 | 1:7–7.5 | 1:6.5 |
| `resident.adult` | 30–59 | 1.50–1.89 | 1.63 / 1.77 | 1.672 / 1.755 | 1:7–7.5 | 1:6.5 |
| `resident.elder` | 60+ | 1.45–1.85 | 60–69: 1.61 / 1.74; 80+: 1.56 / – | 1.661 / 1.742 | 1:7–7.5, a little shorter, often a stoop | 1:6.4 |

- Head-to-height ratios are the classic figures for art students (`dunlop`: about 4 heads at birth, 6 at 5 years,
  7½ for adults) and match the rubric's M14 (baby about 1:4, child 1:5–6, teen 1:6.5–7, adult 1:7–7.5). Art
  canons often use 8 heads for an idealised adult.
- The rig's heads are bigger than real at every stage (the toy look). That is a style choice; changes to `rig.rs`
  are filed only and decided at triage (owner decision 5).
- The girl teen at 1.442 m is just under the real 5th percentile at 13 (1.450 m). Every other stage sits inside
  its range.
- Sleep needs per stage are in the `resident.*` kinds' `minutes` (`aasm`, `nsf`): babies 11–16 h a day with naps,
  children 9–12 h, teens 8–10 h, adults 7–9 h, older adults 7–8 h.

## House standards

The code-built house (`web/src/render/babylon/house.ts`, `walls.ts`) against building practice.

| Part | Real | Game today | Sources |
|---|---|---|---|
| Door leaf | 0.76–0.93 m wide (UK 762 or 838 mm, DIN 18101 nominal 885 mm), 1.98–2.11 m high (UK 1981 mm, DIN 2005 mm), 35–55 mm thick; swings 90–110° | `LEAF_W` 0.8, `DOOR_HEIGHT` 2.1, `LEAF_T` 0.045 | `din18101`, `neufert` |
| Window sill | 0.8–1.1 m above the floor for most rooms; guarding at least 800 mm (Approved Document K), 1100 mm where the window is opened to cool the room (Approved Document O); below 600 mm counts as climbable for children; floor-length glazing needs a guard or safety glass | `SILL` 0.9, `HEAD` 2.1; styles from 0.25 (floor-length) to 1.55 (high light) | `adKWindows` |
| Window head | 2.0–2.3 m, level with or just above the door heads | 1.95–2.3 by style | `neufert` |
| Stairs | Private stair: rise 150–220 mm, going 220–300 mm, 2R + G 550–700 mm, pitch at most 42°; about 13–14 risers for one storey; width 0.8–1.2 m | `stairs` object 1 × 4 tiles over one 2.8 m storey | `adK`, `neufert` |
| Ceiling height | 2.4–2.7 m in most new homes (Germany asks at least 2.4 m in living rooms, the US IRC 7 ft / 2.13 m); older houses 2.7–3.0 m | `WALL_HEIGHT` 2.8 | `neufert` |
| Storey height | Ceiling plus a 0.2–0.35 m floor: 2.6–3.1 m floor to floor | 2.8 (wall height) | `neufert` |
| Light switch | 0.9–1.2 m above the floor, about 0.15 m from the door frame | – | `neufert` |
| Socket | 0.3–0.45 m above the floor; 0.15–0.2 m above a worktop | – | `neufert` |
| Wall cabinet | Underside 0.45–0.6 m above the worktop (about 1.40–1.50 m above the floor); cooker hood 0.65–0.75 m above the hob | – | `en1116`, `neufert` |
| Pendant lamp | Bottom 0.75–0.9 m above a dining table; at least 2.1 m clear where people walk under it | – | `pendant` |

## Notes on the ranges

- **Worktops.** EN 1116 gives three worktop heights, 850, 900 and 950 mm, each with 0 to +50 mm, so a real
  kitchen worktop is 0.85–1.00 m. Every kitchen worktop in the game is 0.95 m, inside the range. Bar counters are
  1.00–1.10 m (dimensions.com: 102–107 cm for bar stools).
- **Tables and desks.** Dining tables are 0.71–0.76 m (28–30 in). Desks are 0.72–0.76 m (EN 527 uses 74 cm
  fixed-height desks; IKEA MICKE is 75 cm). The game's dining tables at 0.765 m are just over.
- **Seats.** Dining chairs 0.42–0.48 m, sofas 0.40–0.48, office chairs 0.40–0.54 (EN 1335 type A 400–520),
  counter stools 0.58–0.71 (23–28 in, for 0.90–0.95 m worktops), bar stools 0.74–0.81 (29–32 in), toilets
  0.40–0.485 (ADA 430–485 mm; dwellings from 380; most European pans sit about 400–420).
- **Beds.** UK single 90 × 190, EU single 90 × 200, UK double 135 × 190, EU double 140 × 200, UK king 150 × 200,
  super king 180 × 200 (mattress sizes; the frame adds 5–20 cm). Mattress tops on common frames are 0.45–0.60 m
  (dimensions.com: 53–64 cm with a 25 cm mattress); a futon or low platform is 0.15–0.40.
- **Garden plants.** The ranges are for a garden specimen some years after planting, not the ultimate size
  (a silver birch reaches 15–25 m, a Japanese maple 6–10 m). Garden trees on a 1 × 1 footprint are a
  deliberate shorthand; the crown is what reads.
- **Held items.** A dinner plate is 25–29 cm across; a 24–28 cm frying pan is 40–52 cm with its handle.
- **Cars.** Lengths are `d` (front to back), widths without mirrors.

## Objects whose real identity was a choice

The kind each of these got, and why. Reviewers can move an object to another kind if the model shows otherwise.

| Object | Kind | Why |
|---|---|---|
| `stoveGas` Gas range (six flames, 1 tile) | `cooker` | A six-burner range is usually 90 cm (`rangeCooker`); on one tile it is read as a 76 cm (30 in) range. |
| `coffeeTableMarble` "Round marble coffee table" (was "Marble side table") | `coffeeTable` | Its top is at 0.42 m, coffee-table height, not side-table height; renamed to match (G-11). |
| `bed` "King-size bed" (was "Double bed"), `bedFuton` "King futon" | `bedKing`, `bedLow` | 1.90–1.96 m wide, a king's width: renamed rather than narrowed (Q-08); `bedLow` takes king futons (to 1.95 m). |
| `fridgeBuiltIn` "Counter-depth fridge" (was "Built-in fridge") | `fridgeAmerican` | 0.84 m wide, free-standing, with a dispenser: a counter-depth bottom-freezer fridge, not an integrated one (Q-08). |
| `ceilingLight` "Lantern pendant" (was "Ceiling light") | `pendantLamp` | A 0.36 m frosted lantern on a stem, 0.6 m deep: a pendant, not a flush fitting (Q-08). |
| `energetic.spinBike` "Interactive recumbent bike" | `recumbentBike` | A low seat with a back and the pedals in front (Q-08). |
| `sink` (base) | `kitchenSink` | The same vanity unit stands in kitchens and bathrooms; its 0.92 m rim is a kitchen sink's, and reads right in both (Q-08). |
| `tableDining4` "for six", `tableDining6` and `tableFarmhouse6` "for eight", `tableBanquet8` (twelve), `tableRound4` (six) | `diningTable6`, `diningTable8`, `diningTable12`, `roundTable6` | Built to fill their tiles: named by the seats they take (Q-08). |
| `sideboard`, `buffet`, `wallOven`, `rangeCooker`, `barSegment`, `barEnd`, `outdoorBar` | `sideboard`, `ovenTower`, `rangeCooker`, `barCounter` | Whole-tile pieces are the style: the ranges are widened to take them (Q-08). |
| `wineFridge`, `dishwasher`, `dishwasherCompact` | `wineCooler`, `dishwasher` | Built in under a worktop, so the kind includes the worktop (h and top 0.85–1.00). |
| `microwave`, `coffeeMachine`, `breakfastCounter`, `energetic.powerBlender`, `slob.microwaveCart`, `foodie.espressoBar` | `counterAppliance` | A base unit or trolley with the appliance on top: the worktop range plus the appliance's height. |
| `barStool` "A tall stool for the counter or the island" | `counterStool` | The game's counters and islands are 0.95 m, so this is a counter stool, not a bar stool. |
| `diningTable` (base) | `diningTable4` | A 2 × 1 table with no chairs of its own. |
| `diningGlass`, `diningOak`, `romantic.candlelitTable` | `diningSet` | Described as a table with its chairs. |
| `couchPotato.cinemaWall`, `couchPotato.gameConsole` | `mediaNook` | A screen on the wall with seats in front of it inside the footprint. |
| `bookworm.firesideLounge` | `fireplace` | A hearth flanked by two wing chairs: the fireplace is the thing you recognise. |
| `slob.gamingRecliner` (2 × 1) | `recliner` | A recliner with a side table; the side table takes the second tile. |
| `bookworm.bedsideStack`, `slob.snackNightstand` | `nightstand` | Nightstands with books or snacks on and in them. |
| `bookworm.readingNook`, `gloomy.windowSeat` | `windowSeat` | Built-in window seats; the nook's bookcases are part of the set. |
| `lazy.napPod`, `loner.quietPod`, `energetic.recoveryPod` | `recoveryLounger` | Padded cocoon or lounger seats; no common real size, so the ranges are wide. |
| `lazy.quiltTable`, `kind.welcomeTeaTable` | `lowTable` | A kotatsu and a floor-seated tea table: 30–45 cm tops, floor cushions. |
| `outgoing.videoCallNook`, `loner.shortwaveRadio`, `hotHeaded.grudgeJournal` | `deskSmall` | Small desks, with their chair built in. |
| `bookworm.writingBureau`, `romantic.letterBureau` | `bureau` | Writing bureaus with their chair (fall-front at 72–80 cm). |
| `gloomy.journalDesk`, `neat.organizerDesk` | `desk` | Two-tile writing desks. |
| `kind.neighbourlyOven` | `kitchenRun` | "A roomy counter and oven": a run of base units with a built-in oven. |
| `neat.laundryCenter` | `laundryRun` | Washer, dryer and a folding counter in one set. |
| `couchPotato.fitnessGame` | `tvOnStand` | A screen with a balance board; the screen sets the size. |
| `lazy.smartHub` | `smartHub` | A tablet on a stand; no common real size. |
| `cheerful.sunriseLamp`, `couchPotato.lavaLamp`, `gloomy.rainLamp` | `noveltyLamp` | Mood lamps; the lava lamp stands on a little side table. |
| `tableLamp` "A lamp on its own little table" | `lampOnTable` | Not a table lamp on its own: the set is about 0.8–1.25 m tall. |
| `garden.wheelbarrow`, `garden.flowerBox`, `garden.geraniums`, `garden.herbs`, `flowerbed` | `planter` | Containers of flowers or herbs. |
| `model.marker` | `selectionMarker` | The selection star: a UI marker with no real counterpart. Only a rough height is given. |

## Kinds by category

Sizes are W × H × D in metres; functional heights in centimetres. *(decor)* marks a kind where no use is expected.
Kitchen gadgets on the worktop (kettle, toaster, blender, mixer, espresso machine), the laptop and the monitor are
not decor: a real one is used, so an object of that kind with no use is a possible I1 finding.

### Kitchen

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `fridge` | Fridge-freezer, freestanding | 0.54–0.7 × 1.45–2.01 × 0.55–0.75 |  | get a snack or a drink; take out ingredients to cook; put the shopping away | snack 1–3, coldDrink 1–3 | fridgeDoor 90–125°, freezerDoor 90–120° | dimFridges, lgFridges, samsungFit, electroluxFit | `fridge` `fridgeRetro` |
| `fridgeIntegrated` | Integrated fridge-freezer in a tall housing | 0.58–0.62 × 1.77–2.3 × 0.55–0.65 |  | get a snack or a drink; take out ingredients to cook | snack 1–3 | fridgeDoor 90–115°, freezerDoor 90–115° | en1116, lgFridges, blum |  |
| `fridgeAmerican` | American-style / French-door fridge | 0.83–0.92 × 1.7–1.85 × 0.65–0.91 |  | get a snack or a drink; take out ingredients to cook; get ice from the dispenser | snack 1–3 | fridgeDoor 90–125°, freezerDrawerTravel 0.4–0.6 m | dimFridges, samsungFit | `fridgeBuiltIn` `fridgeFrenchDoor` |
| `fridgeMini` | Mini / under-counter fridge | 0.4–0.6 × 0.5–0.88 × 0.45–0.62 |  | get a snack or a cold drink | snack 1–3 | fridgeDoor 90–120° | lgFridges, en1116 | `fridgeMini` `fridgeMiniRetro` `couchPotato.snackFridge` |
| `wineCooler` | Under-counter wine cooler under a worktop | 0.3–0.6 × 0.82–1 × 0.55–0.65 | top 85–100 | take out a bottle; pour a glass | wine 2–10 | door 90–110° | en1116, lgFridges | `wineFridge` |
| `dishwasher` | Built-in dishwasher under a worktop (45 or 60 cm) | 0.45–0.6 × 0.82–1 × 0.55–0.65 | top 85–100 | load dirty dishes; unload clean dishes | load 5–15 | doorDrop 80–95°, rackTravel 0.35–0.55 m | dimDishwasher, en1116 | `dishwasher` `dishwasherCompact` |
| `baseCabinet` | Kitchen base unit with worktop | 0.3–1.2 × 0.85–1 × 0.55–0.65 | top 85–100 | prepare and chop food; make a sandwich; store pans and food | sandwich 3–10, prep 5–20 | door 95–110°, drawerTravel 0.4–0.55 m | en1116, dimKitchen, blum | `counter` `counterCorner` `counterEnd` `counterCornerOuter` `counterDrawers` `counterOpenShelf` `butcherBlock` `counterMarble` `cooktopCounter` |
| `counterAppliance` | Worktop or trolley with a countertop appliance on it (microwave, coffee machine, toaster, blender) | 0.45–1 × 1.05–1.45 × 0.4–0.65 | top 85–100 | reheat food; make a coffee; make toast; blend a smoothie | reheat 2–5, espresso 1–5, toast 3–5, smoothie 2–5 | microwaveDoor 90–130° | en1116, microwave, neufert | `microwave` `coffeeMachine` `breakfastCounter` `energetic.powerBlender` `slob.microwaveCart` `foodie.espressoBar` |
| `kitchenSink` | Kitchen sink unit (single bowl or apron front) | 0.6–1.2 × 0.85–1.3 × 0.55–0.65 | top 85–100 | wash hands; wash up by hand; fill a pan or kettle | wash 1–2, washUp 10–30 | tapLever 30–90° | en1116, neufert | `farmhouseSink` `sink` |
| `kitchenSinkDouble` | Double-bowl sink run | 0.8–2 × 0.85–1.3 × 0.55–0.65 | top 85–100 | wash hands; wash up by hand; rinse in one bowl, wash in the other | wash 1–2, washUp 10–30 |  | en1116, neufert | `counterSinkDouble` |
| `kitchenRun` | Run of base units with a built-in oven or hob | 1.2–2.4 × 0.85–1 × 0.55–0.65 | top 85–100 | prepare food; bake in the oven; cook on the hob | bake 20–60 | ovenDoorDrop 85–95°, door 95–110° | en1116, aoCookers | `kind.neighbourlyOven` |
| `larder` | Tall larder / pantry cupboard | 0.4–0.9 × 1.8–2.3 × 0.55–0.65 |  | take out a snack or ingredients; put the shopping away | snack 1–3 | door 95–110°, pullOutTravel 0.4–0.55 m | en1116, neufert | `pantry` `pantryTall` `foodie.deliPantry` |
| `wallCabinet` | Kitchen wall cabinet | 0.3–1 × 0.35–0.95 × 0.3–0.37 |  | take out plates, glasses or food; put clean dishes away | putAway 1–5 | door 95–110°, liftUp 90–107° | en1116, blum | `wallCabinet` `wallCabinetGlass` `wallCabinetCorner` |
| `cookerHood` | Cooker hood (chimney) | 0.6–1 × 0.5–1.1 × 0.45–0.55 |  | extract steam while cooking; light the hob |  |  | en1116, neufert | `rangeHood` |
| `kitchenShelf` | Wall shelves for a kitchen | 0.5–1.2 × 0.2–0.8 × 0.15–0.35 |  | take a jar or a mug; show things off *(decor)* |  |  | neufert | `kitchenShelfWall` |
| `spiceRack` | Wall spice rack | 0.3–0.8 × 0.15–0.6 × 0.06–0.15 |  | take a spice while cooking *(decor)* |  |  | neufert | `spiceRack` |
| `potRail` | Wall rail with hanging pans and utensils | 0.6–1.2 × 0.3–0.6 × 0.05–0.25 |  | take a pan or utensil while cooking *(decor)* |  |  | neufert | `potRail` |
| `kitchenIsland` | Kitchen island | 0.9–3 × 0.85–1 × 0.6–1.2 | top 85–100 | prepare food; eat a quick breakfast on stools; chat while cooking | breakfast 10–20, prep 5–20 | door 95–110°, drawerTravel 0.4–0.55 m | en1116, neufert | `kitchenIsland` `islandSmall` `islandLong` `foodie.chefsIsland` |
| `barCounter` | Bar-height counter | 0.6–3 × 1–1.15 × 0.4–0.9 | top 100–110 | have a drink on a bar stool; mix drinks; eat a quick bite | drink 10–30 |  | dimStools, neufert | `barSegment` `barEnd` `outdoorBar` `outgoing.partyBar` |
| `cooker` | Freestanding cooker (hob and oven, 50-76 cm) | 0.5–0.77 × 0.85–1 × 0.55–0.7 | top 85–95 | cook on the hob; bake or roast in the oven | cook 10–60, bake 20–60 | ovenDoorDrop 85–95°, hobLid 80–95° | aoCookers, en1116 | `stoveElectric` `stoveGas` |
| `rangeCooker` | Range cooker (90-150 cm) | 0.9–1.6 × 0.9–1.15 × 0.6–0.76 | top 90–95 | cook on many burners at once; bake or roast in the ovens | cook 10–60, bake 20–60 | ovenDoorDrop 85–95° | aoCookers | `rangeCooker` `chefStove` |
| `ovenTower` | Tall housing with built-in oven(s) | 0.58–0.72 × 1.95–2.3 × 0.55–0.65 |  | bake or roast | bake 20–60, roast 45–120 | ovenDoorDrop 85–95° | en1116, aoCookers | `wallOven` |
| `pizzaOven` | Wood-fired pizza oven on a stand | 0.8–1.6 × 1.2–2.2 × 0.8–1.4 | top 90–110 | light the fire; bake pizza on the hearth; warm up by the fire | bakePizza 2–5, fireUp 20–45 | door 0–90° | neufert | `foodie.pizzaOven` |
| `pedalBin` | Kitchen pedal bin | 0.25–0.45 × 0.4–0.8 × 0.25–0.45 |  | throw rubbish away; take the bag out | empty 1–3 | lid 70–100° | neufert | `trashCan` |
| `recyclingBins` | Recycling bins (sorted) | 0.4–0.9 × 0.4–0.9 × 0.3–0.6 |  | sort the recycling | sortRecycling 1–5 | lid 70–100° | neufert | `recyclingBins` |

### Dining

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `diningTable2` | Small table for two (square, round, bistro, café) | 0.6–0.9 × 0.71–0.78 × 0.6–0.9 | top 71–76 | eat a meal; have a coffee; chat across the table | meal 15–40, coffee 10–30 |  | dimDiningTables, dimCafeTables, neufert | `tableSquare2` `tableBistroRound` `cafeTable` `patioTableRound` |
| `diningTable4` | Dining table for four | 1.2–1.6 × 0.71–0.78 × 0.75–1 | top 71–76 | eat a meal; have a coffee; do homework or a puzzle | meal 15–40, dinner 30–60 |  | dimDiningTables, neufert, atus | `diningTable` `patioTable4` |
| `diningTable6` | Dining table for six | 1.6–2.2 × 0.71–0.78 × 0.85–1.1 | top 71–76 | eat a family meal; host a dinner | dinner 30–90 |  | dimDiningTables, neufert | `tableDining4` |
| `diningTable8` | Long dining table for eight to ten | 2.2–3 × 0.71–0.78 × 0.9–1.2 | top 71–76 | host a dinner party | dinner 45–120 |  | dimDiningTables, neufert | `tableDining6` `tableFarmhouse6` |
| `diningTable12` | Banquet table for ten to twelve | 3–4 × 0.71–0.78 × 0.9–1.2 | top 71–76 | host a big family dinner | dinner 45–150 |  | dimDiningTables, neufert | `tableBanquet8` |
| `roundTable4` | Round dining table for four | 0.9–1.25 × 0.71–0.78 × 0.9–1.25 | top 71–76 | eat a meal; play cards | meal 15–40 |  | dimDiningTables, neufert |  |
| `roundTable6` | Round dining table for six | 1.3–1.6 × 0.71–0.78 × 1.3–1.6 | top 71–76 | eat a family meal; play cards | meal 15–40 |  | dimDiningTables, neufert | `tableRound4` |
| `squareTable8` | Large square dining table for eight | 1.4–1.8 × 0.71–0.78 × 1.4–1.8 | top 71–76 | host a dinner | dinner 45–120 |  | dimDiningTables, neufert | `tableSquare8` |
| `diningSetSmall` | Bistro or folding set: small table with one or two chairs | 0.6–1.3 × 0.78–1 × 0.6–1.3 | seat 42–48, top 71–76 | eat breakfast; have a coffee | breakfast 10–25, coffee 10–30 | foldingChair 0–90° | dimCafeTables, dimSideChairs | `bistroSet` `patioSet` |
| `diningSet` | Dining table with its chairs (two to four places) | 1.2–2 × 0.78–1 × 0.75–1.8 | seat 42–48, top 71–76 | eat a meal; have a coffee; a dinner for two | breakfast 10–25, dinner 30–60, candlelitDinner 45–120 |  | dimDiningTables, dimSideChairs, neufert | `diningGlass` `diningOak` `romantic.candlelitTable` |
| `picnicTable` | Picnic table with fixed benches | 1.5–2.4 × 0.7–0.8 × 1.4–1.8 | seat 42–48, top 70–78 | eat outdoors; have a picnic; play cards | picnic 20–60 |  | dimPicnic | `picnicTable` |
| `barTable` | High café / bar table | 0.6–0.8 × 1–1.12 × 0.6–0.8 | top 100–110 | have a drink standing or on a bar stool | drink 10–30 |  | dimStools, neufert | `tableBarHigh` |
| `lowTable` | Low floor-seated table (tea table, kotatsu) | 0.6–1.3 × 0.3–0.45 × 0.6–0.9 | seat 5–15, top 30–45 | share tea on floor cushions; snack and chat; warm legs under the quilt | tea 15–45, doze 20–60 |  | neufert | `kind.welcomeTeaTable` `lazy.quiltTable` |
| `diningChair` | Dining / side chair | 0.4–0.56 × 0.78–1 × 0.45–0.6 | seat 42–48 | sit at a table to eat; have a coffee | sit 5–60, meal 15–40 | foldingSeat 80–95° | dimSideChairs, neufert | `diningChair` `chairWood` `chairUpholstered` `chairModern` `chairWindsor` `chairBentwood` `chairFolding` `cafeChair` `patioChair` |
| `carverChair` | Dining armchair (carver) | 0.5–0.65 × 0.78–1.05 × 0.5–0.62 | seat 42–48 | sit at the head of the table | meal 15–40 |  | dimSideChairs, dimChairs | `chairArm` |
| `diningBench` | Backless dining bench | 1–2 × 0.42–0.5 × 0.3–0.45 | seat 42–48 | sit at a table, two or three abreast | meal 15–40 |  | dimBench, dimSideChairs | `benchDining` |
| `counterStool` | Counter-height stool (for 90-95 cm worktops) | 0.35–0.5 × 0.6–1.05 × 0.35–0.5 | seat 58–71 | sit at a kitchen counter or island; have a drink | sit 5–30, drink 10–30 | swivel 0–360° | dimStools | `barStool` `stoolCounter` |
| `barStoolHigh` | Bar-height stool (for 102-107 cm bars) | 0.35–0.5 × 0.74–1.15 × 0.35–0.5 | seat 74–81 | sit at a bar; have a drink | drink 10–30 |  | dimStools | `stoolBarHigh` |
| `highChair` | Baby high chair | 0.5–0.65 × 0.85–1.05 × 0.55–0.8 | seat 50–60 | feed a baby or toddler at the table | feed 10–30 | trayTravel 0.05–0.2 m | ikeaAntilop | `highChair` |
| `sideboard` | Sideboard / buffet (to a base unit's depth: whole-tile style) | 1.2–2 × 0.7–0.95 × 0.4–0.68 | top 70–95 | store plates and table linen; serve from the top | putAway 1–5 | door 95–110°, drawerTravel 0.3–0.45 m | dimDressers, neufert | `sideboard` `buffet` |
| `displayCabinet` | Glazed display / china cabinet | 0.6–1.2 × 1.6–2.1 × 0.35–0.5 |  | show off china and glass; take out the good plates | putAway 1–5 | door 95–110° | neufert, dimArmoires | `chinaCabinet` `displayCabinet` |
| `trolley` | Bar cart / tea trolley | 0.6–0.95 × 0.75–1 × 0.4–0.55 | top 75–95 | mix a drink; serve tea; wheel it to the guests | mix 3–10, brew 5–15 |  | neufert | `barCart` `bookworm.teaTrolley` `cheerful.mocktailCart` |

### Living

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `sofa2` | Two-seat sofa / loveseat | 1.3–1.9 × 0.7–1 × 0.8–1 | seat 40–48 | relax; watch TV; doze off; chat with someone beside you | relax 15–120, doze 10–30, tv 30–180 |  | dimSofa, ikeaEktorp | `sofa` `loveseat` `sofaDesign` `kind.hearthSettee` |
| `sofaBed` | Sofa bed (pull-out) | 1.4–2.1 × 0.75–1 × 0.85–1.05 | seat 40–48, mattress 35–50 | sit and relax; pull it out for a guest to sleep | relax 15–120, pullOut 1–3, sleep 420–540 | bedTravel 1–1.6 m | dimSofa, ikeaEktorp, bedSizes | `sofaBed` |
| `sofa3` | Three-seat sofa | 1.9–2.4 × 0.7–1 × 0.8–1.05 | seat 40–48 | relax; watch TV; lie down for a nap | relax 15–120, doze 10–30, tv 30–180 |  | dimSofa, ikeaEktorp | `sofaThree` `sofaThreeSeat` |
| `sectionalModule` | Modular sofa section (seat, arm or corner) | 0.7–1.05 × 0.7–0.95 × 0.85–1.05 | seat 40–48 | relax; watch TV | relax 15–120 |  | ikeaEktorp, dimSofa | `sectionalMiddle` `sectionalArmLeft` `sectionalArmRight` `sectionalCorner` |
| `chaise` | Chaise section of a sofa | 0.7–1 × 0.7–0.95 × 1.4–1.7 | seat 40–48 | put your feet up; lie back and read | relax 15–120 |  | ikeaEktorp | `chaise` |
| `cornerSofa` | L-shaped corner sofa | 2–3 × 0.7–1 × 1.5–2.6 | seat 40–48 | hang out with friends; watch TV together | hangOut 30–180 |  | ikeaEktorp | `outgoing.conversationSectional` |
| `windowSeat` | Built-in window seat / nook | 1–2 × 0.42–1.3 × 0.4–0.7 | seat 42–50 | curl up and read; look out of the window; doze | read 20–90, doze 10–30 |  | neufert, dimChairs | `gloomy.windowSeat` `bookworm.readingNook` |
| `armchair` | Upholstered armchair | 0.65–0.95 × 0.75–1.15 × 0.7–0.95 | seat 38–48 | sit and relax; read; doze off | sit 10–60, read 20–90, doze 10–30 |  | ikeaPoang, dimChairs | `armchair` `armchairLounge` `armchairLeather` `armchairWingback` `armchairDesign` |
| `recliner` | Recliner chair | 0.75–1 × 0.95–1.1 × 0.85–1.05 | seat 43–50 | lean back with the footrest up; watch TV; doze | recline 15–120, doze 10–30 | backrest 100–160°, footrest 60–90° | dimChairs, neufert | `recliner` `couchPotato.recliner` `slob.gamingRecliner` |
| `rockingChair` | Rocking chair | 0.55–0.75 × 0.9–1.15 × 0.75–1 | seat 40–48 | rock gently; read; knit | relax 10–60, knit 30–120 | rock 5–15° | dimChairs, neufert | `rockingChair` `kind.knittingRocker` |
| `beanBag` | Bean bag (adult) | 0.7–1.2 × 0.5–0.9 × 0.7–1.2 | seat 25–45 | flop down; play games; doze | flop 10–60 |  | neufert | `beanBag` `slob.beanbag` |
| `beanBagKids` | Bean bag (child size) | 0.5–0.8 × 0.4–0.7 × 0.5–0.8 | seat 20–35 | flop down; read a picture book | flop 10–30 |  | neufert, ikeaLatt | `beanBagKids` |
| `pouf` | Pouf / ottoman / footstool | 0.4–0.8 × 0.35–0.48 × 0.4–0.8 | seat 35–48 | put your feet up; perch for a moment | sit 5–30 | lid 80–100° | dimChairs, neufert | `pouf` `ottoman` |
| `floorCushion` | Floor or meditation cushion | 0.35–0.8 × 0.08–0.22 × 0.35–0.8 | seat 8–22 | sit cross-legged; meditate | sit 5–30, breathe 10–30 |  | neufert | `floorCushion` `hotHeaded.coolDownCushion` |
| `coffeeTable` | Coffee table | 0.6–1.4 × 0.3–0.5 × 0.45–0.85 | top 30–50 | put drinks and snacks down; leaf through a magazine; play a board game | magazine 5–20 |  | dimCoffeeTables, ikeaLack | `coffeeTable` `coffeeTableGlass` `coffeeTableMarble` `coffeeTableRound` `coffeeTableLong` |
| `sideTable` | Side / end table, nesting tables | 0.35–0.65 × 0.45–0.65 × 0.35–0.6 | top 45–65 | put a drink or a lamp beside the sofa |  |  | dimCoffeeTables, dimNightstands | `sideTable` `sideTableRound` `nestingTables` |
| `consoleTable` | Console / hall table | 0.8–1.6 × 0.7–0.9 × 0.25–0.45 | top 70–90 | drop keys and post; tidy the hall | tidy 2–10 | drawerTravel 0.2–0.35 m | neufert | `consoleTable` `neat.entryConsole` |
| `tvWall` | Flat-screen TV on the wall (55-75 in) | 1.2–1.7 × 0.7–1 × 0.03–0.12 |  | watch TV; watch a film; play a console game | tv 30–180, film 90–150 |  | neufert, atus | `tvWall` |
| `tvOnStand` | TV on a stand or low bench (incl. console or workout game) | 0.8–1.8 × 0.85–1.4 × 0.3–0.55 |  | watch TV; play a console game; follow a workout | tv 30–180, game 20–120, workout 20–45 | trayTravel 0.05–0.15 m | neufert | `tv` `tvStandLow` `couchPotato.fitnessGame` |
| `tvCrt` | Small CRT television on a cabinet | 0.4–0.75 × 0.7–1.1 × 0.35–0.55 |  | watch TV | tv 30–180 |  | neufert | `tvRetro` |
| `homeCinema` | Home cinema: big screen with speakers (or seats in front) | 1.6–2.4 × 1.2–2.2 × 0.35–1 |  | watch a film; binge a series | film 90–150, marathon 120–360 | curtainTravel 0.3–0.8 m | neufert | `homeCinema` |
| `mediaNook` | Screen on the wall with its own seat in front (viewing or gaming nook) | 0.9–2.4 × 1–2.2 × 0.8–1.2 | seat 40–48 | watch a film; play a console game | film 90–150, game 20–120 | curtainTravel 0.3–0.8 m | neufert, dimSofa | `couchPotato.cinemaWall` `couchPotato.gameConsole` |
| `hifiCabinet` | Hi-fi or record player on a low cabinet | 0.6–1.2 × 0.6–1.1 × 0.3–0.5 |  | put a record on; listen to music; dance | listen 20–90, dance 5–30 | dustLid 60–90° | neufert | `stereo` `gloomy.recordCabinet` |
| `gramophone` | Horn gramophone on a stand | 0.35–0.65 × 0.4–1.4 × 0.35–0.65 |  | wind it and play a record; dance | listen 3–60 | lid 60–90° | neufert | `romantic.gramophone` |
| `boombox` | Boombox (on a crate) | 0.4–0.7 × 0.2–0.75 × 0.15–0.4 |  | play a tape or the radio | listen 20–90 | cassetteDoor 20–45° | neufert | `boombox` |
| `jukebox` | Jukebox | 0.75–0.95 × 1.3–1.65 × 0.6–0.75 |  | pick a song; dance | dance 3–30 |  | jukebox | `cheerful.jukebox` |
| `bookcase` | Bookcase (full height) | 0.4–1 × 1.6–2.2 × 0.25–0.4 |  | take a book and read; study; put books away | read 20–90, study 30–120 |  | ikeaBilly, neufert | `bookshelf` `bookcaseTall` `bookcaseCorner` |
| `bookcaseLow` | Low bookcase / cube shelf | 0.6–1.2 × 0.7–1.1 × 0.25–0.42 | top 70–110 | take a book; put things on top | read 20–90 |  | ikeaBilly, neufert | `bookcaseLow` `cubeShelf` |
| `bookcaseWide` | Wide bookcase or shelving unit | 1.2–2 × 1.8–2.4 × 0.28–0.45 |  | take a book and read; study | read 20–90, study 30–120 |  | ikeaBilly, neufert | `bookcaseWide` `shelvingIndustrial` |
| `libraryWall` | Floor-to-ceiling library shelves with a ladder | 1.2–2.4 × 2.3–2.9 × 0.3–0.45 |  | climb the ladder for a book; research | research 30–120 | ladderTravel 0.3–2 m | ikeaBilly, neufert | `bookworm.libraryWall` |
| `ladderShelf` | Leaning ladder shelf | 0.4–0.8 × 1.5–2 × 0.3–0.5 |  | show plants and books *(decor)* |  |  | neufert | `ladderShelf` |
| `fireplace` | Fireplace with mantel and hearth (seat: a raised hearth or a stool by it) | 1–1.8 × 1–1.4 × 0.25–0.65 | seat 38–48 | light a fire; warm up by it; read by the fire | relax 15–90, lightFire 5–15 |  | neufert | `fireplace` `bookworm.firesideLounge` |
| `electricFire` | Electric fire / media fireplace | 0.8–1.8 × 0.5–1.2 × 0.15–0.45 |  | switch on the flame effect; warm up | relax 15–90 |  | neufert | `fireplaceElectric` |

### Bedroom

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `bedSingle` | Single bed (90 x 190-200) | 0.9–1.05 × 0.4–1.1 × 1.9–2.15 | mattress 45–60 | sleep; nap; sit on the edge | sleep 420–540, nap 10–30 |  | bedSizes, dimBedFrames, ikeaHemnesDaybed | `bedSingle` `bedDaybed` |
| `bedDouble` | Double bed (135-140 x 190-200) | 1.35–1.6 × 0.4–1.2 × 1.9–2.2 | mattress 45–60 | sleep; nap; read in bed | sleep 420–540, nap 10–30 |  | bedSizes, dimBedFrames | `bedDouble` |
| `bedKing` | King or queen bed (150-180 x 200) | 1.5–1.95 × 0.4–1.4 × 2–2.3 | mattress 45–65 | sleep; nap; lie in | sleep 420–540, nap 10–30, lieIn 30–90 | headRaise 0–60° | bedSizes, dimBedFrames | `bed` `bedKing` `bedQueenUpholstered` `lazy.snoozecloudBed` |
| `bedCanopy` | Four-poster double or king bed | 1.5–2 × 1.8–2.4 × 2–2.3 | mattress 50–70 | sleep; draw the curtains | sleep 420–540, nap 10–30 |  | bedSizes, dimBedFrames, neufert | `bedCanopy` |
| `bedLow` | Low platform / futon bed, double to king | 1.35–1.95 × 0.15–1 × 1.95–2.15 | mattress 15–40 | sleep; fold the futon away by day | sleep 420–540, nap 10–30 |  | bedSizes, neufert | `bedFuton` |
| `bunkBed` | Bunk bed (two 90 x 200 berths) | 0.95–1.05 × 1.5–1.8 × 1.95–2.15 | mattress 30–45 | sleep (two people, one above the other); climb the ladder | sleep 420–540, nap 10–30 |  | ikeaSvarta | `bedBunk` |
| `wallBed` | Fold-down wall bed (double) | 1.5–1.75 × 2–2.3 × 0.4–2.2 | mattress 45–60 | fold it down to sleep; fold it up to free the floor | sleep 420–540, nap 10–30, foldDown 1–2 | fold 85–95° | bedSizes, neufert | `wallBed` |
| `kidsBed` | Junior bed with guard rail (70 x 140-160) | 0.75–0.95 × 0.4–1 × 1.45–2.1 | mattress 25–50 | a child sleeps; bedtime story | sleep 540–720, nap 30–90 |  | ikeaKritter, aasm | `kidsBed` `kidsBedCar` |
| `crib` | Cot with changing table | 1.2–2 × 0.85–1.05 × 0.6–0.8 | top 85–95, mattress 20–60 | a baby sleeps; change a nappy; feed and cuddle beside it | sleep 660–960, change 3–10, feed 10–30 | sideRail 0–90° | ikeaSundvik, aasm, neufert | `crib` |
| `nightstand` | Nightstand / bedside table | 0.35–0.66 × 0.45–0.7 × 0.3–0.5 | top 45–70 | put a lamp, a book and a glass beside the bed; grab a midnight snack from the drawer | midnightSnack 2–10 | drawerTravel 0.2–0.35 m | dimNightstands | `nightstand` `slob.snackNightstand` `bookworm.bedsideStack` `group nightstands` |
| `nightstandLamp` | Nightstand with a reading lamp | 0.35–0.66 × 0.8–1.1 × 0.3–0.5 |  | switch the reading light on and off; read in bed |  | drawerTravel 0.2–0.35 m | dimNightstands, neufert | `nightstandLamp` |
| `dresser` | Chest of drawers / dresser (wide, low) | 0.76–1.88 × 0.7–1 × 0.4–0.61 | top 70–100 | pick out clothes; put laundry away | outfit 3–15 | drawerTravel 0.3–0.45 m | dimDressers, ikeaMalm6 | `dresser` `dresserLow` |
| `chestTall` | Tall chest of drawers | 0.6–1 × 1–1.4 × 0.4–0.55 | top 100–140 | pick out clothes | outfit 3–15 | drawerTravel 0.3–0.45 m | dimDressers | `chestOfDrawers` |
| `wardrobe` | Wardrobe / armoire (double) | 0.9–2 × 1.8–2.4 × 0.5–0.65 |  | pick out clothes; hang clothes up | outfit 3–15 | door 90–110°, slideTravel 0.5–1 m | dimArmoires, ikeaPax | `wardrobe` `armoire` |
| `wardrobeSingle` | Single wardrobe | 0.5–0.8 × 1.8–2.4 × 0.5–0.65 |  | pick out clothes | outfit 3–15 | door 90–110° | ikeaPax, dimArmoires | `wardrobeSingle` |
| `blanketChest` | Blanket chest / trunk | 0.7–1.2 × 0.4–0.6 × 0.4–0.6 | top 40–60 | take out a blanket; sit on the lid | takeBlanket 1–3 | lid 80–110° | neufert | `cedarChest` |
| `bedEndBench` | Bed-end bench | 1–1.6 × 0.42–0.5 × 0.35–0.5 | seat 42–48 | sit to put on shoes; lay out clothes | sit 2–10 |  | dimBench | `benchBedEnd` |
| `dressingTable` | Dressing table with mirror and stool | 0.8–1.2 × 1.2–1.7 × 0.4–0.55 | seat 42–48, top 72–78 | do your hair and make-up; get ready to go out | getReady 10–30, primp 5–20 | drawerTravel 0.25–0.4 m | neufert, dimSideChairs | `dressingTable` `romantic.vanity` |
| `mirrorStanding` | Full-length standing mirror | 0.4–0.8 × 1.5–1.9 × 0.3–0.6 |  | check your outfit; practise a speech | primp 1–5, speech 10–30 | tilt 0–20° | neufert | `mirror` `mirrorFloor` |
| `mirrorWall` | Wall mirror (head and shoulders or longer) | 0.4–1 × 0.5–1.2 × 0.02–0.06 |  | check your look; practise a speech | primp 1–5, speech 10–30 |  | neufert | `mirrorWall` `mirrorRound` |
| `laundryHamper` | Laundry hamper / basket | 0.35–0.6 × 0.4–0.7 × 0.3–0.5 |  | drop dirty clothes; sort the washing | sortWashing 2–10 | lid 80–110° | neufert | `laundryHamper` `laundryBasket` |

### Bathroom

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `toilet` | WC (close-coupled or wall-hung) | 0.35–0.52 × 0.4–0.85 × 0.48–0.76 | seat 40–48.5 | use the toilet; flush; wash hands after | use 2–10 | lid 95–110°, seatRing 95–110° | ada604, dimToilets | `group toilets` |
| `basinPedestal` | Washbasin (pedestal, vessel, console) | 0.45–1 × 0.76–1.15 × 0.35–0.6 | top 76–91 | wash hands and face; brush teeth; shave | wash 1–3, brushTeeth 2–3 | tapLever 30–90° | dimBasin, dimVanities | `sinkPedestal` `sinkVessel` `sinkConsole` |
| `vanityDouble` | Double vanity unit | 1.2–1.8 × 0.76–1.15 × 0.45–0.6 | top 76–91 | two people wash and brush at once | wash 1–3, brushTeeth 2–3 | drawerTravel 0.3–0.45 m | dimVanities, dimBasin | `vanityDouble` |
| `mirrorCabinet` | Mirrored bathroom wall cabinet | 0.4–1 × 0.55–0.8 × 0.1–0.2 |  | brush teeth in front of it; take out toiletries | brushTeeth 2–3 | door 95–110° | neufert, dimVanities | `mirrorCabinet` |
| `bathtub` | Bathtub (built-in or freestanding) | 1.5–1.8 × 0.5–0.65 × 0.7–0.85 |  | take a bath; bath a child; shower over the bath | bath 15–45 |  | bathSizes, dimBathClear | `bathtub` `bathtubFreestanding` |
| `bathtubClawfoot` | Clawfoot / roll-top tub | 1.5–1.8 × 0.6–0.8 × 0.7–0.85 |  | take a long soak | bath 15–45 |  | bathSizes, dimBathClear | `bathtubClawfoot` |
| `soakingTub` | Large soaking or whirlpool tub (one or two people) | 1.5–2 × 0.5–0.8 × 0.8–1.8 |  | soak, alone or for two; turn on the jets | soak 15–60 |  | bathSizes, hotTub | `whirlpoolTub` `gloomy.soakingTub` `romantic.heartTub` |
| `coldPlunge` | Cold-plunge tub | 0.7–1 × 0.6–0.9 × 1.2–1.9 |  | take a short ice bath | plunge 2–10 | lid 80–110° | bathSizes | `hotHeaded.coldPlunge` |
| `shower` | Shower enclosure / walk-in shower | 0.7–1.2 × 1.85–2.3 × 0.7–1.2 |  | take a shower; rinse off after sport | shower 5–15 | door 90–180° | showerEnclosures, estShower | `shower` `showerStall` `showerWalkIn` `showerRain` `neat.rainShowerTower` `energetic.contrastShower` |
| `outdoorShower` | Outdoor shower | 0.8–1.3 × 2–2.5 × 0.8–1.3 |  | rinse off garden soil or sea water | rinse 3–10 | door 90–110° | showerEnclosures, estShower | `natureLover.cedarShower` |
| `sauna` | Sauna cabin for one or two | 0.9–1.5 × 1.9–2.2 × 1–1.8 | seat 45–100 | sit and sweat; cool off after | sweat 8–20 | door 90–110° | neufert | `loner.soloSauna` |
| `bathCabinetTall` | Tall bathroom cabinet | 0.3–0.6 × 1.4–2 × 0.25–0.4 |  | take a fresh towel; store toiletries | takeTowel 1–2 | door 95–110° | neufert | `bathCabinetTall` |
| `towelRail` | Towel rail or towel ladder | 0.4–0.8 × 0.5–1.8 × 0.05–0.45 |  | hang a towel to dry; take a towel |  |  | neufert | `towelRail` `towelLadder` |
| `bathMat` | Bath mat | 0.5–0.9 × 0.005–0.03 × 0.4–0.6 |  | stand on it after a bath *(decor)* |  |  | neufert | `bathMat` |
| `washer` | Washing machine (front-loading) | 0.595–0.69 × 0.82–1 × 0.53–0.84 |  | load the washing; unload and hang it out | load 2–5 | door 90–180°, drum 400–1600 rpm | washerUk, dimLaundry, ww8500 | `laundryWasher` |
| `dryer` | Tumble dryer | 0.595–0.69 × 0.82–1 × 0.53–0.84 |  | load wet washing; take out warm laundry and fold it | fold 5–20 | door 90–180° | washerUk, dimLaundry | `laundryDryer` |
| `laundryRun` | Washer and dryer with a folding counter | 1.2–2 × 0.85–2 × 0.55–0.84 |  | wash and dry clothes; fold and iron | load 2–5, fold 10–40 | door 90–180° | dimLaundry, washerUk | `neat.laundryCenter` |
| `toiletriesStation` | Wall shelf or stand with toiletries and a mirror | 0.4–0.9 × 0.8–1.8 × 0.15–0.45 |  | spray deodorant or dry shampoo; check yourself in the mirror | spritz 1–5 |  | neufert | `slob.spritzStation` |

### Kids

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `toyBox` | Toy box / toy chest | 0.5–0.9 × 0.35–0.65 × 0.35–0.55 |  | take out toys and play; tidy toys away | play 15–60 | lid 80–110° | neufert, ikeaLatt | `toyBox` `toyChest` |
| `playMat` | Play mat (with blocks) | 0.8–1.5 × 0.01–0.3 × 0.8–1.5 |  | build with blocks; tummy time for a baby | blocks 10–40 |  | neufert | `playMat` |
| `kidsEasel` | Children's easel (chalk or paint) | 0.45–0.7 × 0.9–1.3 × 0.45–0.7 |  | draw in chalk; paint | draw 10–40 |  | neufert, ikeaLatt | `kidsChalkboard` `kidsEasel` |
| `dollHouse` | Doll house | 0.5–1 × 0.5–1.1 × 0.25–0.45 |  | play with dolls | dolls 15–60 |  | neufert | `dollHouse` |
| `rockingHorse` | Rocking horse | 0.3–0.5 × 0.55–0.95 × 0.75–1.2 | seat 40–65 | rock back and forth | ride 5–20 | rock 10–25° | neufert | `rockingHorse` |
| `kidsDesk` | Child's desk (with chair) | 0.6–1 × 0.5–0.75 × 0.45–0.9 | seat 28–38, top 50–64 | do homework; draw | homework 20–60, draw 10–40 |  | ikeaLatt, neufert | `kidsDesk` |
| `kidsTable` | Children's table | 0.5–0.9 × 0.4–0.55 × 0.45–0.7 | top 40–55 | draw; have a snack; play a game | draw 10–40 |  | ikeaLatt | `kidsTable` |
| `kidsChair` | Children's chair | 0.28–0.4 × 0.5–0.65 × 0.28–0.4 | seat 26–35 | sit at the kids table | sit 5–40 |  | ikeaLatt | `kidsChair` |
| `kidsBookshelf` | Low picture-book shelf | 0.5–1 × 0.5–1 × 0.2–0.4 |  | pick a picture book | read 10–30 |  | ikeaBilly, ikeaLatt | `kidsBookshelf` |
| `playTent` | Play tent / teepee | 0.9–1.3 × 1.2–1.7 × 0.9–1.3 |  | hide and play inside | play 15–60 |  | neufert | `playTent` |
| `trainTable` | Train table | 0.9–1.3 × 0.35–0.5 × 0.6–0.9 | top 35–50 | play with the train set | trains 15–60 |  | ikeaLatt | `trainTable` |
| `ballPit` | Ball pit | 0.9–1.5 × 0.25–0.5 × 0.9–1.5 |  | jump in; throw the balls about | jump 10–30 |  | neufert | `ballPit` |
| `teddyGiant` | Giant teddy bear | 0.6–1 × 0.8–1.5 × 0.5–0.9 |  | hug it; sit against it | hug 1–5 |  | neufert | `teddyBear` |

### Office

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `desk` | Desk (writing, computer, executive) | 1–1.8 × 0.72–1.1 × 0.5–0.9 | top 72–76 | work at a computer; write; do homework | work 30–240, homework 20–60 | drawerTravel 0.3–0.45 m | en527, ikeaMicke | `computerDesk` `deskWriting` `computerGaming` `deskSimple` `deskExecutive` `gloomy.journalDesk` `neat.organizerDesk` |
| `deskSmall` | Small desk / laptop desk | 0.7–1.1 × 0.72–1 × 0.45–0.7 | top 72–76 | browse on a laptop; write; do homework | browse 15–90, homework 20–60 | laptopLid 100–135° | en527, ikeaMicke, dellLid | `deskLaptop` `outgoing.videoCallNook` `loner.shortwaveRadio` `hotHeaded.grudgeJournal` |
| `deskCorner` | Corner desk (L-shaped) | 1.2–1.8 × 0.72–0.8 × 1.2–1.8 | top 72–76 | work with two screens; spread out papers | work 30–240 |  | en527 | `deskCorner` |
| `deskStanding` | Standing desk (sit-stand, raised) | 1–1.8 × 0.95–1.3 × 0.6–0.8 | top 95–125 | work standing up | work 20–120 | liftTravel 0.45–0.65 m | en527 | `deskStanding` |
| `bureau` | Writing bureau (fall-front or roll-top) with its chair | 0.75–1.2 × 1–1.3 × 0.45–1 | seat 42–50, top 72–80 | write letters; draft on a typewriter | letter 10–45, draft 30–120 | fallFront 85–95° | neufert, en527 | `bookworm.writingBureau` `romantic.letterBureau` |
| `officeChair` | Office swivel chair | 0.55–0.75 × 0.85–1.35 × 0.55–0.75 | seat 40–54 | sit at a desk; swivel | sit 10–240 | swivel 0–360°, recline 0–25° | en1335 | `officeChair` `group chairs` |
| `filingCabinet` | Filing cabinet | 0.4–0.5 × 0.7–1.35 × 0.45–0.7 |  | file papers; find a document | file 2–15 | drawerTravel 0.4–0.65 m | neufert | `filingCabinet` |
| `whiteboard` | Wall whiteboard | 0.6–1.8 × 0.45–1.2 × 0.02–0.08 |  | plan and sketch ideas | plan 10–45 |  | neufert | `whiteboard` |
| `pinboard` | Corkboard / pinboard / photo board | 0.4–1.2 × 0.3–1 × 0.01–0.06 |  | pin up notes and photos; look back at photos | reminisce 2–15 |  | neufert | `corkboard` `outgoing.photoWall` |
| `smartHub` | Smart-home hub (tablet on a stand) | 0.2–0.6 × 0.2–1.3 × 0.15–0.5 |  | set up routines; ask it for a story | program 10–60 | screenTilt 0–60° | neufert | `lazy.smartHub` |

### Hobbies

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `pianoUpright` | Upright piano with bench | 1.4–1.6 × 1–1.35 × 0.54–1.1 | seat 46–56 | play the piano; practise scales | play 15–60 | fallboard 90–110°, topLid 0–90° | yamahaB1, pianoBench | `piano` |
| `digitalPiano` | Digital piano on a stand, with bench | 1.3–1.45 × 0.75–1 × 0.3–1 | seat 46–56 | play; practise with headphones | play 15–60 | keyCover 0–100° | yamahaB1, pianoBench | `keyboardDigital` |
| `guitarOnStand` | Guitar on a floor stand (acoustic or electric, with amp) | 0.35–0.7 × 0.95–1.2 × 0.3–0.6 |  | strum; practise chords; jam | strum 10–45, practise 20–60 |  | neufert | `guitarElectric` `guitarAcoustic` |
| `ukulele` | Ukulele on a stand | 0.2–0.35 × 0.5–0.8 × 0.2–0.35 |  | strum a tune; practise | strum 10–45 |  | neufert | `ukulele` |
| `cello` | Cello with a stool | 0.4–0.7 × 1.2–1.3 × 0.4–1 | seat 45–55 | practise; play a slow piece | practise 20–60 |  | neufert | `gloomy.cello` |
| `drumKit` | Drum kit with throne | 1.2–2 × 0.9–1.7 × 1–1.8 | seat 46–60 | play the drums; practise a beat | drum 15–60 |  | neufert | `drumKit` `hotHeaded.thunderDrums` |
| `karaoke` | Karaoke machine or stage with mics and a screen | 0.4–2 × 1–1.8 × 0.4–1.5 |  | sing along; perform for friends | sing 3–60 |  | neufert | `cheerful.karaokeMachine` `outgoing.karaokeStage` |
| `poolTable` | Pool table (7-9 ft) | 1.9–2.8 × 0.74–0.82 × 1.1–1.55 | top 74–79 | play a game of pool; practise trick shots | pool 10–30 |  | wpa | `poolTable` |
| `foosball` | Foosball table | 1.2–1.45 × 0.86–0.95 × 0.7–1.2 |  | play a quick game for two or four | foosball 5–20 | rodSpin 0–360° | foosball | `foosball` |
| `dartboard` | Dartboard on the wall | 0.45–0.7 × 0.45–0.7 × 0.04–0.12 |  | throw darts from the oche | darts 5–30 | cabinetDoor 90–180° | darts | `dartboard` |
| `chessTable` | Chess / games table with seats | 0.6–1.6 × 0.7–1 × 0.6–1.6 | seat 42–48, top 70–76 | play chess or a board game | chess 10–60 |  | dimCafeTables, dimSideChairs | `chessTable` `parkChessTable` |
| `jigsawTable` | Jigsaw / puzzle table | 0.7–1 × 0.7–1 × 0.6–1 | seat 42–48, top 70–78 | work on a jigsaw | jigsaw 20–120 |  | dimDiningTables, dimSideChairs | `puzzleTable` `loner.puzzleTable` |
| `arcadeCabinet` | Upright arcade cabinet | 0.6–0.85 × 1.55–1.85 × 0.7–0.9 |  | play a quick round; chase the high score | arcade 5–30 |  | arcade | `arcadeCabinet` `slob.arcadeCabinet` |
| `easel` | Studio easel | 0.5–0.8 × 1.5–2.1 × 0.5–0.9 |  | paint a canvas | paint 30–180 |  | neufert | `easel` |
| `workbench` | Workbench | 1.2–2 × 0.85–1.6 × 0.55–0.8 | top 85–95 | tinker and repair things; pot seedlings | tinker 30–120 | viceTravel 0–0.2 m | neufert, en1116 | `workbench` `natureLover.pottingBench` |
| `telescope` | Telescope on a tripod | 0.6–1 × 1.2–1.8 × 0.6–1.2 |  | look at the stars and the moon | stargaze 20–120 | tubeTilt 0–90° | neufert | `telescope` |
| `sewingTable` | Sewing table with machine | 0.6–1.2 × 0.72–1 × 0.45–0.7 | top 72–76 | sew; mend clothes | sew 30–120, mend 10–40 |  | en527, neufert | `sewingTable` |
| `chemistryBench` | Hobby chemistry bench | 0.8–1.4 × 0.85–1.8 × 0.5–0.75 | top 85–95 | run an experiment; look through the microscope | experiment 30–120 |  | neufert | `chemistrySet` |
| `potteryWheel` | Pottery wheel with stool | 0.55–0.9 × 0.45–0.7 × 0.6–1.2 | seat 40–55 | throw a pot | throw 20–90 | wheel 0–300° | neufert | `potteryWheel` |
| `craftTable` | Craft table with seat | 1–1.8 × 0.72–1 × 0.55–0.9 | seat 42–48, top 72–76 | scrapbook; cut and glue | scrapbook 20–90 |  | en527, dimSideChairs | `craftTable` |
| `cookbookLectern` | Book stand / lectern | 0.4–0.7 × 1–1.3 × 0.35–0.6 |  | read standing up; study recipes | read 10–60 |  | neufert | `foodie.cookbookLectern` |
| `bonsaiBench` | Bonsai or potting bench (small) | 0.6–1 × 0.75–1 × 0.45–0.7 | top 75–95 | prune and wire a bonsai; repot | shape 20–90 |  | neufert | `loner.bonsaiBench` |
| `smashBin` | Rage-room crate for smashing plates | 0.5–1 × 0.5–1.1 × 0.5–1 |  | throw old plates in | smash 5–20 | lid 80–110° | neufert | `hotHeaded.smashBin` |

### Fitness

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `treadmill` | Treadmill | 0.6–0.95 × 1.2–1.55 × 1.5–2.15 |  | walk or run; warm up | run 20–60 | incline 0–15° | treadmill | `treadmill` |
| `weightBench` | Weight bench with bar | 0.6–1.8 × 0.4–1.4 × 1.2–2 | seat 40–48 | bench press; lift weights | lift 20–60 | backrest 0–85° | neufert | `weightBench` |
| `exerciseBike` | Exercise / spin bike | 0.45–0.65 × 1–1.45 × 0.85–1.4 | seat 75–105 | cycle; follow a ride on the screen | cycle 20–60 |  | dimExBike | `exerciseBike` |
| `recumbentBike` | Recumbent exercise bike (seat with a back, pedals in front) | 0.55–0.75 × 1–1.4 × 1.3–1.75 | seat 40–60 | cycle sitting back; follow a ride on the screen | cycle 20–60 |  | dimExBike | `energetic.spinBike` |
| `yogaMat` | Yoga mat | 0.6–0.7 × 0.003–0.15 × 1.7–1.9 | seat 0–3 | do yoga; stretch | yoga 15–60 |  | neufert | `yogaMat` |
| `dumbbellRack` | Dumbbell rack | 0.5–1.2 × 0.6–1 × 0.3–0.6 |  | lift dumbbells | lift 15–45 |  | neufert | `dumbbells` |
| `heavyBag` | Free-standing punching bag | 0.4–0.7 × 1.5–1.9 × 0.4–0.7 |  | punch and kick | pound 10–45 | sway 0–25° | neufert | `hotHeaded.heavyBag` |
| `boulderingWall` | Home bouldering wall with crash mat | 1.8–3 × 2.2–2.8 × 0.8–2 |  | climb a problem; drop onto the mat | climb 20–90 |  | neufert | `energetic.boulderingWall` |
| `trampoline` | Garden trampoline | 1.8–3.7 × 0.6–2.7 × 1.8–3.7 |  | bounce | bounce 10–40 |  | neufert | `energetic.trampoline` |
| `vibrationPlate` | Vibration plate | 0.45–0.85 × 0.15–1.3 × 0.45–0.85 |  | stand on it while it shakes | shake 10–15 |  | neufert | `lazy.vibePlate` |
| `recoveryLounger` | Padded lounger / pod (recovery or nap) | 0.7–1 × 0.8–1.6 × 0.9–2 | seat 35–50 | lie back and recover; take a power nap | recover 15–45, powerNap 10–30 | canopy 0–90° | neufert, nsf | `energetic.recoveryPod` `lazy.napPod` `loner.quietPod` |

### Decor

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `wallArt` | Picture, print or canvas on the wall | 0.3–1.5 × 0.3–1.2 × 0.02–0.08 |  | look at it *(decor)* | admire 1–5 |  | neufert | `printFramed` `photoPrint` `paintingClassic` `canvasAbstract` |
| `wallClock` | Wall clock | 0.25–0.6 × 0.25–0.6 × 0.03–0.08 |  | check the time *(decor)* |  |  | neufert | `wallClock` |
| `wallShelf` | Floating wall shelves with ornaments | 0.6–1.2 × 0.3–0.8 × 0.15–0.3 |  | show books and ornaments *(decor)* |  |  | neufert | `wallShelf` |
| `curtains` | Curtains on a rod (window height) | 1–2.4 × 1.2–2.2 × 0.05–0.2 |  | draw them at night; open them in the morning |  | curtainTravel 0.5–1.2 m | neufert | `curtains` |
| `curtainsLong` | Floor-length curtains | 1.5–3 × 2.2–2.8 × 0.05–0.2 |  | draw them at night; open them in the morning |  | curtainTravel 0.5–1.5 m | neufert | `curtainsLong` |
| `rug` | Area rug or runner | 0.6–3 × 0.005–0.03 × 0.6–4 |  | soften the floor; sit or play on it *(decor)* |  |  | neufert | `rugRound` `rugLarge` `rugRunner` `rugPattern` |
| `grandfatherClock` | Longcase (grandfather) clock | 0.4–0.6 × 1.8–2.3 × 0.25–0.4 |  | wind it once a week; check the time | wind 1–5 | hoodDoor 90–110° | neufert | `grandfatherClock` |
| `floorVase` | Tall floor vase or urn | 0.2–0.5 × 0.4–1.2 × 0.2–0.5 |  | decorate a corner *(decor)* |  |  | neufert | `vaseTall` `urnPorcelain` |
| `pedestalOrnament` | Ornament on a plinth (vase, bust, statue) | 0.3–0.7 × 0.8–1.7 × 0.3–0.7 |  | look at it *(decor)* | admire 1–5 |  | neufert | `vasePedestal` `bustMarble` `statueHorse` |
| `statueSmall` | Small statue / garden ornament | 0.15–0.6 × 0.2–0.8 × 0.15–0.6 |  | look at it; pat it for luck *(decor)* | pat 0–2 |  | neufert | `statueCat` `garden.gnome` |
| `coatStand` | Coat stand | 0.4–0.65 × 1.65–1.95 × 0.4–0.65 |  | hang up a coat and hat *(decor)* | hangCoat 0–1 |  | neufert | `coatRack` |
| `hangingPlant` | Hanging plant from the ceiling | 0.25–0.6 × 0.4–1.2 × 0.25–0.6 |  | water it | water 1–5 |  | rhsPlants | `hangingPlant` |
| `givingJar` | Charity jar on a stand | 0.25–0.5 × 0.8–1.2 × 0.25–0.5 |  | drop in some change | donate 0–2 |  | neufert | `kind.givingJar` |
| `terrarium` | Terrarium on a stand | 0.3–0.6 × 0.8–1.3 × 0.3–0.6 |  | mist the plants; look at the little forest | mist 2–10 |  | neufert, rhsPlants | `natureLover.mossTerrarium` |
| `herbPlanter` | Indoor herb planter on a stand or sill | 0.4–1 × 0.2–1.1 × 0.15–0.5 |  | pick herbs for cooking; water them | pickHerbs 1–5, water 1–5 |  | rhsPlants | `foodie.herbGarden` |
| `partyBooth` | Horseshoe booth seat with a screen | 1.8–2.6 × 1–2 × 1.5–2.4 | seat 42–50 | host a video party; sit and chat | party 30–180 |  | dimSofa, neufert | `cheerful.partyBooth` |
| `danceFloor` | Light-up dance floor | 1.5–3 × 0.05–2.5 × 1.5–3 |  | dance | dance 5–60 |  | neufert | `outgoing.danceFloor` |
| `steamMop` | Steam mop in a wall dock | 0.25–0.5 × 1–1.3 × 0.15–0.4 |  | mop the floor | mop 10–30 |  | neufert | `neat.steamMopDock` |
| `robotVacuum` | Robot vacuum with self-emptying dock | 0.3–0.45 × 0.08–0.55 × 0.3–0.5 |  | set its route; empty the dust | program 5–30 |  | neufert | `neat.robotVacuumDock` |
| `deliveryHatch` | Parcel locker or wall delivery hatch | 0.4–1 × 0.4–1.8 × 0.2–0.6 |  | collect a delivery; order food | collect 1–3 | door 90–110° | neufert | `lazy.deliveryLocker` `loner.deliveryHatch` |

### Lighting

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `floorLamp` | Floor lamp (standard, tripod, uplighter) | 0.25–0.65 × 1.3–1.9 × 0.25–0.65 |  | switch it on and off; read by it |  | shadeTilt 0–45° | neufert | `lamp` `floorLampSimple` `lampFloorTripod` `lampTorchiere` |
| `arcLamp` | Arc floor lamp | 0.35–1.8 × 1.7–2.2 × 0.35–1.8 |  | light a sofa or a table from above |  |  | neufert | `lampFloorArc` |
| `tableLamp` | Table lamp | 0.12–0.45 × 0.15–0.7 × 0.12–0.45 |  | switch it on and off; read by it |  |  | neufert | `lampTableRound` `lampTableSquare` `lampBanker` `lampMushroom` `lampLantern` |
| `lampOnTable` | Lamp standing on its own side table | 0.35–0.65 × 0.8–1.25 × 0.35–0.6 |  | switch it on and off |  |  | neufert, dimNightstands | `tableLamp` |
| `noveltyLamp` | Mood lamp (lava, rain, sunrise) | 0.15–0.6 × 0.3–1.6 × 0.15–0.6 |  | watch it glow; wake up to it | zoneOut 5–30 |  | neufert | `couchPotato.lavaLamp` `gloomy.rainLamp` `cheerful.sunriseLamp` |
| `pendantLamp` | Pendant lamp on a cord or chain | 0.2–0.7 × 0.3–1.2 × 0.2–0.7 |  | switch it on and off |  |  | pendant, neufert | `ceilingPendant` `pendantIndustrial` `paperLantern` `ceilingLight` |
| `ceilingLight` | Flush ceiling light or track | 0.2–1.2 × 0.05–0.3 × 0.2–0.6 |  | switch it on and off |  | spotTilt 0–90° | neufert | `trackLights` |
| `chandelier` | Chandelier | 0.5–1 × 0.4–1 × 0.5–1 |  | switch it on and off |  |  | pendant, neufert | `chandelier` |
| `ceilingFan` | Ceiling fan with light | 0.9–1.4 × 0.25–0.6 × 0.9–1.4 |  | switch the fan or light on |  | blades 60–250 rpm | neufert | `ceilingFan` |
| `wallLight` | Wall light (sconce, swing arm, bulkhead) | 0.1–0.4 × 0.15–0.5 × 0.08–0.8 |  | switch it on and off; swing it over to read |  | armSwing 0–180° | neufert | `wallSconce` `wallLampArm` `wallLightOutdoor` |
| `gardenLampPost` | Garden lamp post or lantern on a post | 0.2–0.5 × 1.8–3 × 0.2–0.5 |  | light the garden path at night |  |  | streetlight, neufert | `lampPost` |
| `stringLights` | Festoon lights between posts | 1.5–4 × 2–2.8 × 0.1–0.4 |  | light a patio at night |  |  | neufert | `stringLights` |
| `stoneLantern` | Japanese stone lantern | 0.3–0.8 × 0.6–1.5 × 0.3–0.8 |  | light the garden at night |  |  | neufert | `garden.lantern` |

### Outdoor

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `gardenBench` | Garden or park bench (two or three seats) | 1.2–1.9 × 0.75–0.95 × 0.5–0.75 | seat 42–48 | sit and enjoy the garden; feed the birds | sit 10–60 |  | dimBench, dimChairs | `bench` `benchGarden3` `model.street.bench` |
| `parkBench` | Long park bench | 1.8–2.5 × 0.75–0.95 × 0.55–0.75 | seat 42–48 | sit; feed the birds; wait for someone | sit 10–60, feedBirds 5–20 |  | dimBench | `parkBench` |
| `gardenChair` | Garden armchair (slatted or Adirondack) | 0.6–0.85 × 0.8–1.05 × 0.65–0.95 | seat 30–48 | sit outside; read in the sun | sit 10–60 |  | dimChairs, neufert | `gardenChair` `adirondackChair` |
| `porchSwing` | Porch or arbour swing seat for two | 1.2–2.4 × 1.8–2.4 × 0.7–1.5 | seat 42–55 | swing gently; sit together | swing 10–60 | swing 5–20° | dimBench, neufert | `porchSwing` `romantic.roseArbourSwing` |
| `sunLounger` | Sun lounger | 0.55–0.8 × 0.3–1 × 1.8–2.1 | seat 25–45 | sunbathe; doze | sunbathe 15–60, doze 15–45 | backrest 0–75° | neufert | `sunLounger` |
| `hammock` | Hammock on a stand | 1–1.6 × 1–1.5 × 2.7–4 |  | lie and swing; doze | swing 10–60, doze 15–45 | swing 5–20° | neufert | `hammock` `cheerful.rainbowHammock` `natureLover.gardenHammock` |
| `hotTub` | Hot tub (two to six people) | 1.5–2.4 × 0.8–1 × 1.5–2.4 |  | soak in warm water; chat with friends; put the cover on | soak 15–30 | cover 80–110° | hotTub | `hotTubCedar` |
| `parasol` | Garden parasol on a base | 2–3.5 × 2.2–2.7 × 2–3.5 |  | open it for shade |  | canopy 0–90° | neufert | `parasol` |
| `kettleGrill` | Kettle barbecue | 0.55–0.75 × 0.9–1.1 × 0.55–0.75 | top 70–80 | light the charcoal; grill food | cook 15–45, fireUp 15–30 | lid 0–180° | neufert | `grill` |
| `grillStation` | Gas grill station / outdoor kitchen | 1.2–2.2 × 1.1–1.3 × 0.55–0.7 | top 85–95 | grill food for a crowd | cook 15–45 | hood 70–90° | neufert, en1116 | `grillStation` |
| `firePit` | Fire pit (seat: the stools or benches round it) | 0.6–1.2 × 0.25–0.6 × 0.6–1.2 | seat 35–48 | light a fire; warm up; toast marshmallows | relax 15–90 |  | neufert | `firePit` |
| `swingSet` | Garden swing set (two swings) | 2–3.5 × 1.9–2.5 × 1.2–2.2 | seat 35–60 | swing; push a child | swing 5–30 | swing 30–70° | en1176, neufert | `swingSet` |
| `sandpit` | Sandpit | 1–2 × 0.15–0.4 × 1–2 |  | build sandcastles; dig | sandcastle 15–60 | cover 0–110° | neufert | `sandbox` |
| `bikeRack` | Bike rack with a bicycle | 1.6–2 × 0.9–1.2 × 0.4–0.7 |  | park the bike; tune it up | tuneUp 15–60 |  | neufert | `bikeRack` |
| `planter` | Planter or raised bed (flowers, herbs) | 0.4–1.2 × 0.3–0.9 × 0.3–1 |  | water and tend the plants | water 1–10, tend 10–30 |  | rhsPlants | `planterBox` `flowerbed` `garden.flowerBox` `garden.wheelbarrow` `model.street.flowerTub` `model.street.pot` `garden.geraniums` `garden.herbs` |
| `raisedVegBed` | Raised vegetable bed | 1–2.4 × 0.3–0.9 × 0.6–1.2 |  | sow, weed and water; harvest vegetables | tend 15–45, harvest 10–30 |  | rhsPlants | `raisedBed` `natureLover.veggieBed` |
| `shed` | Garden shed | 1.5–3 × 2–2.5 × 1.2–2.5 |  | store tools; potter about | potter 15–90 | door 90–110° | neufert | `shedSmall` |
| `greenhouse` | Greenhouse / glasshouse | 1.8–3 × 2–2.8 × 1.8–3.8 |  | tend seedlings and tomatoes; sit among the plants | tend 15–60 | door 90–110°, ventTilt 0–45° | neufert, rhsPlants | `greenhouse` `natureLover.glasshouse` |
| `compostBin` | Compost bin | 0.6–1 × 0.75–1.1 × 0.6–1 |  | tip in kitchen scraps; turn the compost | compost 2–15 | lid 80–110° | neufert, rhsPlants | `compostBin` |
| `waterButt` | Rain barrel / water butt | 0.5–0.8 × 0.8–1.3 × 0.5–0.8 |  | fill the watering can | fill 1–3 | lid 80–110° | neufert | `rainBarrel` |
| `washingLine` | Washing line between posts | 2–4 × 1.7–2.2 × 0.1–0.4 |  | hang out the washing; take it in | washing 10–20, takeIn 5–10 |  | neufert | `clothesline` |
| `logStore` | Log store | 0.6–1.5 × 1–1.8 × 0.4–0.8 |  | fetch logs for the fire | fetch 1–5 |  | neufert | `firewoodStack` |
| `birdhouse` | Bird box on a pole | 0.15–0.4 × 1.5–2.5 × 0.15–0.4 |  | watch birds nest *(decor)* |  |  | neufert | `birdhouse` |
| `birdFeeder` | Bird feeder post or station | 0.3–1 × 1.5–2.4 × 0.3–1 |  | fill it with seed; watch the birds | birdwatch 10–40, refill 2–10 |  | neufert | `birdFeederPole` `natureLover.birdFeeder` `kind.songbirdFeeder` |
| `birdbath` | Bird bath | 0.4–0.7 × 0.6–1 × 0.4–0.7 |  | watch birds bathe; top up the water | birdwatch 10–40 |  | neufert | `garden.birdbath` |
| `dogHouse` | Dog kennel | 0.6–1.2 × 0.6–1.1 × 0.7–1.3 |  | a dog sleeps in it *(decor)* |  |  | neufert | `dogHouse` |
| `sundial` | Sundial on a pedestal | 0.3–0.6 × 0.7–1.2 × 0.3–0.6 |  | tell the time by the sun *(decor)* |  |  | neufert | `sundial` |
| `steppingStones` | Stepping stones | 0.3–0.7 × 0.02–0.1 × 1–3 |  | walk across the lawn *(decor)* |  |  | neufert | `steppingStones` |
| `gardenFountain` | Two-tier garden fountain | 0.8–1.5 × 1–1.8 × 0.8–1.5 |  | listen to the water; toss in a coin | listen 5–20 |  | neufert | `garden.fountain` |
| `pond` | Garden pond | 1.5–3 × 0–0.6 × 1.5–3 |  | watch and feed the fish; fish (in a big pond) | feedFish 1–5, fish 15–60 |  | rhsPlants, neufert | `garden.pond` |
| `boulders` | Garden boulders or rocks | 0.4–1.5 × 0.2–0.8 × 0.4–1.5 |  | sit on one; admire them *(decor)* |  |  | neufert | `garden.boulder` `model.rock` |

### Garden plants

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `shadeTree` | Broad deciduous garden tree (oak, maple, birch, willow, magnolia), garden age | 2–10 × 4–20 × 2–10 |  | sit in the shade; rake the leaves | shade 10–60 |  | birch, rhsPlants | `tree` `garden.maple` `garden.birch` `garden.willow` `garden.magnolia` |
| `smallTree` | Small ornamental tree (cherry, Japanese maple) | 1.5–8 × 2–8 × 1.5–8 |  | admire the blossom or the colour; shape it | admire 1–5 |  | acerPalmatum, rhsPlants | `garden.cherry` `garden.japaneseMaple` |
| `fruitTree` | Garden apple tree (on M9-MM106 rootstock) | 1.5–5 × 1.5–5.5 × 1.5–5 |  | pick fruit; prune it in winter | pick 5–30 |  | rhsRootstocks | `garden.apple` |
| `conifer` | Conifer (pine, spruce), garden age | 1.5–8 × 3–20 × 1.5–8 |  | give year-round green and shelter *(decor)* |  |  | rhsPlants | `pine` `garden.spruce` |
| `columnarConifer` | Italian cypress (slim column) | 0.5–1.5 × 3–15 × 0.5–1.5 |  | line a path *(decor)* |  |  | rhsPlants | `garden.cypress` |
| `pottedTree` | Potted citrus or patio tree | 0.5–1.5 × 0.8–2 × 0.5–1.5 |  | water it; pick a lemon | water 1–5, pick 1–10 |  | rhsPlants | `garden.lemonTree` |
| `clippedShrub` | Clipped shrub, topiary or hedge section | 0.3–1.6 × 0.3–1.8 × 0.3–1.2 |  | clip it into shape | trim 10–40 |  | rhsPlants | `bush` `garden.boxwood` `garden.topiary` `garden.hedge` `model.hedge` `model.street.hedge` `model.bush.small` |
| `floweringShrub` | Flowering shrub (hydrangea, rose, azalea) | 0.5–2 × 0.5–2 × 0.5–2 |  | smell the flowers; prune and deadhead; water it | smell 0–2, prune 10–40 |  | rhsPlants | `garden.hydrangea` `garden.roses` `garden.azalea` `model.bush.flowering` |
| `perennial` | Perennial clump (lavender, fern, grass, hosta) | 0.3–1.2 × 0.2–1.5 × 0.3–1.2 |  | water it; cut it back; smell the lavender | water 1–5, cut 5–20 |  | rhsPlants | `garden.lavender` `garden.fern` `garden.grass` `garden.hosta` `model.street.flowers` |
| `flowerBed` | Bed of bulbs or bedding flowers (one square metre) | 0.5–1.2 × 0.1–1.5 × 0.5–1.2 |  | water the flowers; smell them; pick a few | water 1–10, smell 0–2 |  | rhsPlants | `garden.tulips` `garden.daffodils` `garden.daisies` `garden.poppies` `garden.marigolds` `garden.pansies` `garden.lilies` `garden.lupins` `garden.forgetMeNots` `garden.cosmos` `garden.meadow` |
| `sunflowers` | Sunflowers | 0.5–1.2 × 1.5–3 × 0.5–1.2 |  | water them; admire them | water 1–10 |  | sunflower | `garden.sunflowers` |
| `vegPlot` | Vegetable patch (one to two square metres) | 0.8–2 × 0.1–1.8 × 0.8–2 |  | sow, weed and water; harvest | tend 15–45, harvest 10–30 |  | rhsPlants | `garden.vegPatch` `garden.tomatoes` `garden.strawberries` `garden.pumpkins` |
| `houseplantFloor` | Large floor houseplant (monstera, fig, palm, fern on stand) | 0.4–1.2 × 0.5–2 × 0.4–1.2 |  | water it; talk to it; wipe the leaves | water 1–5, talk 1–10 |  | rhsPlants | `plant` `garden.monstera` `garden.fiddleFig` `garden.snakePlant` `garden.palm` `garden.bostonFern` `garden.orchid` |
| `houseplantSmall` | Small potted plant (cacti, succulents, bonsai, tabletop pot) | 0.1–0.6 × 0.1–0.6 × 0.1–0.6 |  | water it now and then; prune the bonsai | water 1–3, prune 10–45 |  | rhsPlants | `garden.cacti` `garden.succulents` `garden.bonsai` `potSucculent` `potCactus` `potSmall` |
| `wildflower` | Single wildflower clump in a lawn | 0.05–0.3 × 0.1–0.45 × 0.05–0.3 |  | pick a flower *(decor)* |  |  | rhsPlants | `model.flowers` |
| `treePlanter` | Street planter with a small tree | 0.6–1.3 × 1.5–3.5 × 0.6–1.3 |  | green the street *(decor)* |  |  | rhsPlants, neufert | `model.street.planter` |
| `grassTuft` | Tuft of lawn grass | 0.05–0.3 × 0.05–0.3 × 0.05–0.3 |  | walk on the lawn *(decor)* |  |  | rhsPlants | `model.grassTuft` |

### Community and town

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `streetLamp` | Street lamp (heritage lantern or column) | 0.25–1.5 × 3–8 × 0.25–1.5 |  | light the pavement at night |  |  | streetlight, neufert | `streetLamp` `street.streetlight` |
| `drinkingFountain` | Drinking fountain | 0.3–0.7 × 0.75–1.1 × 0.3–0.7 |  | have a drink of water | drink 0–2 |  | ada604, neufert | `drinkingFountain` |
| `litterBin` | Street litter bin | 0.4–0.7 × 0.8–1.2 × 0.4–0.7 |  | throw rubbish away |  |  | neufert | `litterBin` |
| `busShelter` | Bus shelter with bench | 2.5–4 × 2.3–2.8 × 1.2–1.8 | seat 42–50 | wait for the bus; read the timetable | wait 2–20 |  | neufert, dimBench | `busStop` |
| `postBox` | Pillar post box | 0.45–0.6 × 1.5–1.7 × 0.45–0.6 |  | post a letter | post 0–2 |  | pillarBox | `postBox` |
| `noticeBoard` | Notice board on posts | 0.8–1.5 × 1.6–2.2 × 0.1–0.4 |  | read the notices; pin one up | read 1–10 |  | neufert | `noticeBoard` |
| `phoneBox` | Phone box (K6 kiosk) | 0.85–1 × 2.4–2.8 × 0.85–1 |  | make a call | call 2–20 | door 80–110° | k6 | `phoneBooth` |
| `townFountain` | Town fountain with a sitting edge | 2.5–6 × 1–3.5 × 2.5–6 | seat 42–55 | sit on the edge; toss in a coin | sitBy 5–30, wish 0–2 |  | neufert, dimBench | `fountainTown` |
| `townStatue` | Statue on a plinth | 1–2.5 × 2–4.5 × 1–2.5 |  | admire it; meet beside it *(decor)* | admire 1–10 |  | neufert | `townStatue` |
| `slide` | Playground slide | 0.5–1 × 1.5–2.6 × 2–4 |  | climb up and slide down | slide 5–20 |  | en1176, neufert | `slide` |
| `climbingFrame` | Climbing frame | 1.5–3 × 1.5–2.6 × 1.5–3 |  | climb; hang from the bars | climb 10–30 |  | en1176, neufert | `climbingFrame` |
| `seesaw` | Seesaw | 0.3–0.6 × 0.5–1 × 2.4–4 | seat 40–75 | seesaw with a friend | seesaw 5–15 | tilt 10–25° | en1176, neufert | `seesaw` |
| `springRider` | Spring rider | 0.25–0.5 × 0.7–1.1 × 0.7–1.1 | seat 40–70 | bounce and rock | ride 3–10 | rock 10–25° | en1176, neufert | `springRider` |
| `roundabout` | Playground roundabout (merry-go-round) | 1.5–2.5 × 0.3–1 × 1.5–2.5 | seat 25–45 | spin round; push it | spin 3–15 | spin 0–360° | en1176, neufert | `merryGoRound` |
| `basketballHoop` | Basketball hoop on a pole | 0.9–1.8 × 3.2–4 × 0.8–2 |  | shoot hoops; play one-on-one | shoot 10–60 |  | fiba | `basketballHoop` `energetic.streetballHoop` |
| `pingPongTable` | Table tennis table | 1.5–1.6 × 0.76–0.95 × 2.7–2.8 | top 76–76 | play table tennis | pingPong 10–40 |  | ittf | `pingPongTable` |
| `foodCart` | Street food cart (ice cream, hot dogs) | 1.2–2.2 × 1.8–2.6 × 0.7–1.2 | top 90–105 | buy an ice cream or a hot dog | iceCream 3–15, hotDog 5–15 | lid 80–110° | neufert | `iceCreamCart` `hotDogStand` |
| `vendingMachine` | Vending machine | 0.7–1 × 1.75–1.95 × 0.7–0.95 |  | buy a snack or a drink | snack 1–3 |  | neufert | `vendingSnack` `vendingDrink` |
| `marketStall` | Market stall with awning | 1.5–3 × 2–2.6 × 0.8–1.5 | top 80–100 | browse the produce; buy fruit | browse 5–20 |  | neufert | `marketStall` |
| `shopCounter` | Shop counter or bakery case | 1.2–2.5 × 0.95–1.4 × 0.6–1 | top 90–110 | buy a treat; pay | treat 2–10, pastry 2–10 |  | neufert | `shopCounter` `bakeryCase` |
| `groceryShelf` | Supermarket shelving (gondola) | 1–2.5 × 1.4–2.2 × 0.4–1.2 |  | browse and pick groceries | browse 5–20 |  | neufert | `groceryShelf` |
| `mailbox` | Kerbside mailbox on a post | 0.15–0.3 × 1.04–1.4 × 0.4–0.6 |  | post or collect letters | post 0–2 | door 90–180° | usps | `street.mailbox` |
| `hydrant` | Fire hydrant (pillar) | 0.25–0.5 × 0.6–1 × 0.25–0.5 |  | (firefighters connect hoses) |  |  | hydrant, hydrantNj | `street.hydrant` |
| `wheelieBin` | Wheelie bin (120-240 L) | 0.48–0.6 × 0.93–1.1 × 0.55–0.75 |  | put the rubbish out; wheel it to the kerb |  | lid 180–270° | bin240 | `street.bin` |
| `streetSign` | Street-name sign on posts | 0.6–1.2 × 2–3 × 0.05–0.9 |  | read the street name *(decor)* |  |  | neufert | `model.street.sign` |
| `busStopSign` | Bus stop flag on a pole | 0.3–0.6 × 2.3–3 × 0.05–0.2 |  | check the bus stop *(decor)* |  |  | neufert | `model.street.busSign` |
| `pathLight` | Garden path light (bollard) | 0.1–0.25 × 0.3–1 × 0.1–0.25 |  | light the path at night |  |  | neufert | `model.street.pathLight` |

### Farm

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `barn` | Barn | 6–15 × 5–10 × 6–20 |  | store hay and machinery; house animals *(decor)* |  | door 90–180° | neufert | `farm.barn` |
| `silo` | Grain silo (small farm) | 2.5–6 × 6–20 × 2.5–6 |  | store grain *(decor)* |  |  | neufert | `farm.silo` |
| `tractor` | Compact farm tractor | 1–1.6 × 1.7–2.4 × 2–3.5 |  | drive it round the farm *(decor)* |  |  | tractor | `farm.tractor` |
| `hayBaleRound` | Round hay bale | 1.2–1.8 × 1.2–1.8 × 1.2–1.8 |  | feed livestock; sit on it *(decor)* |  |  | neufert | `farm.hayBale` |
| `hayStack` | Stack of small square hay bales | 1–2.5 × 0.7–2 × 0.45–1.2 |  | sit on it; feed animals *(decor)* |  |  | neufert | `farm.hayStack` |
| `scarecrow` | Scarecrow | 0.8–1.6 × 1.5–2.2 × 0.2–0.6 |  | keep birds off the crops *(decor)* |  |  | neufert | `farm.scarecrow` |
| `trough` | Water trough | 1.2–2.5 × 0.4–0.7 × 0.5–0.9 |  | water the animals *(decor)* |  |  | neufert | `farm.trough` |
| `chickenCoop` | Chicken coop with run | 1.2–2.5 × 1–2 × 1–2.5 |  | collect eggs; feed the hens *(decor)* | collectEggs 2–10 | lid 80–110° | neufert | `farm.coop` |

### Tabletop

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `kettle` | Electric kettle | 0.15–0.25 × 0.19–0.3 × 0.13–0.25 |  | boil water for tea | boil 2–4 | lid 60–90° | neufert | `kettle` |
| `toaster` | Toaster (two or four slots) | 0.25–0.45 × 0.17–0.25 × 0.15–0.25 |  | make toast | toast 2–5 | leverTravel 0.08–0.12 m | neufert | `toaster` |
| `blender` | Jug blender | 0.15–0.25 × 0.35–0.5 × 0.15–0.25 |  | blend a smoothie | smoothie 1–3 |  | neufert | `blender` |
| `standMixer` | Stand mixer | 0.2–0.3 × 0.33–0.42 × 0.3–0.4 |  | mix dough or batter | mix 3–15 | headTilt 55–75° | neufert | `standMixer` |
| `espressoMachine` | Espresso machine | 0.2–0.4 × 0.3–0.45 × 0.25–0.45 |  | pull a shot; steam milk | espresso 1–5 |  | neufert | `espressoMachineSmall` |
| `knifeBlock` | Knife block | 0.1–0.2 × 0.2–0.35 × 0.1–0.25 |  | take a knife *(decor)* |  |  | neufert | `knifeBlock` |
| `utensilPot` | Utensil pot | 0.1–0.18 × 0.25–0.4 × 0.1–0.18 |  | take a spoon or spatula *(decor)* |  |  | neufert | `utensilCrock` |
| `choppingBoard` | Chopping board | 0.3–0.5 × 0.015–0.05 × 0.2–0.35 |  | chop vegetables | chop 2–10 |  | neufert | `cuttingBoard` `tool.board` |
| `breadBin` | Bread bin | 0.3–0.45 × 0.15–0.3 × 0.2–0.3 |  | take out bread |  | lid 80–110° | neufert | `breadBin` |
| `spiceJars` | Spice jars | 0.1–0.4 × 0.08–0.15 × 0.05–0.15 |  | season food *(decor)* |  |  | neufert | `spiceJars` |
| `cookieJar` | Cookie jar | 0.12–0.22 × 0.18–0.3 × 0.12–0.22 |  | sneak a biscuit |  | lid 0–90° | neufert | `cookieJar` |
| `fruitBowl` | Fruit bowl | 0.2–0.35 × 0.08–0.2 × 0.2–0.35 |  | take a piece of fruit |  |  | neufert | `fruitBowl` |
| `placeSetting` | Place setting (plate, cutlery, glass) | 0.4–0.65 × 0.02–0.25 × 0.3–0.45 |  | lay the table *(decor)* |  |  | neufert, dimDrinkware | `plateSet` |
| `cakeStand` | Cake on a stand | 0.2–0.35 × 0.15–0.35 × 0.2–0.35 |  | cut a slice *(decor)* |  |  | neufert | `cakeStand` |
| `teaSet` | Teapot with cups | 0.25–0.5 × 0.12–0.25 × 0.15–0.35 |  | pour tea |  |  | neufert, dimDrinkware | `teapotSet` |
| `mugs` | Mugs | 0.08–0.3 × 0.07–0.12 × 0.08–0.2 |  | have a hot drink |  |  | dimDrinkware | `mugs` |
| `wineAndGlasses` | Bottle of wine with glasses | 0.15–0.35 × 0.28–0.35 × 0.08–0.2 |  | pour a glass |  |  | dimDrinkware | `wineGlasses` |
| `candlesticks` | Candlesticks | 0.08–0.3 × 0.15–0.45 × 0.08–0.2 |  | light the candles *(decor)* |  |  | neufert | `candlesticks` |
| `books` | Books (stack or with bookends) | 0.15–0.4 × 0.08–0.3 × 0.13–0.25 |  | pick one up to read |  |  | neufert | `bookStack` `booksUpright` |
| `vase` | Vase of flowers / bud vase | 0.05–0.3 × 0.12–0.5 × 0.05–0.3 |  | change the water *(decor)* |  |  | neufert | `vaseFlowers` `vaseSmall` |
| `photoFrame` | Photo frame | 0.1–0.3 × 0.13–0.3 × 0.02–0.15 |  | look at a photo *(decor)* |  |  | neufert | `photoFrame` |
| `alarmClock` | Alarm clock | 0.08–0.18 × 0.08–0.15 × 0.04–0.1 |  | set the alarm; switch it off in the morning |  |  | neufert | `tableClock` |
| `globe` | Desk globe | 0.2–0.4 × 0.3–0.55 × 0.2–0.4 |  | spin it and find a country |  | spin 0–360° | neufert | `globe` |
| `snowGlobe` | Snow globe | 0.06–0.15 × 0.08–0.2 × 0.06–0.15 |  | shake it *(decor)* |  |  | neufert | `snowGlobe` |
| `trophy` | Trophy cup | 0.08–0.25 × 0.15–0.45 × 0.08–0.2 |  | show off a win *(decor)* |  |  | neufert | `trophy` |
| `sculptureSmall` | Small sculpture | 0.08–0.3 × 0.15–0.45 × 0.08–0.3 |  | look at it *(decor)* |  |  | neufert | `sculptureSmall` |
| `soapSet` | Soap dispenser and dish | 0.1–0.25 × 0.08–0.22 × 0.06–0.12 |  | wash hands with it *(decor)* |  |  | neufert | `soapSet` |
| `towelStack` | Folded towels | 0.25–0.4 × 0.08–0.3 × 0.2–0.35 |  | take a towel *(decor)* |  |  | neufert | `towelStack` |
| `perfumeTray` | Tray of perfume bottles | 0.2–0.4 × 0.08–0.2 × 0.1–0.25 |  | dab on some scent *(decor)* |  |  | neufert | `perfumeTray` |
| `deskOrganiser` | Desk organiser | 0.15–0.35 × 0.08–0.2 × 0.08–0.2 |  | take a pen *(decor)* |  |  | neufert | `deskOrganiser` |
| `laptop` | Laptop (13-16 in) | 0.3–0.37 × 0.013–0.025 × 0.21–0.26 |  | open it and work | browse 15–90 | lid 100–135° | dellLid, neufert | `laptopClosed` |
| `monitorKeyboard` | Monitor with keyboard (22-27 in) | 0.48–0.65 × 0.35–0.55 × 0.15–0.3 |  | work at the screen |  | tilt -5–20° | en527, neufert | `monitor` |
| `softToy` | Teddy / soft toy | 0.15–0.4 × 0.2–0.45 × 0.12–0.3 |  | cuddle it |  |  | neufert | `stuffedToy` |
| `toyBlocks` | Toy blocks | 0.1–0.35 × 0.04–0.2 × 0.1–0.3 |  | stack them |  |  | neufert | `toyBlocks` |

### Held items and tools

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `heldPlate` | Dinner plate with food (or empty) | 0.25–0.29 × 0.015–0.1 × 0.25–0.29 |  | carry to the table; eat from it | eat 10–30 |  | neufert | `item.mealPlate` `item.pizzaSlice` `item.treatPlate` `item.pieSlice` `item.sandwich` `item.plateDirty` |
| `heldBowl` | Mixing or serving bowl | 0.2–0.32 × 0.08–0.15 × 0.2–0.32 |  | carry prepped food; mix |  |  | neufert | `item.prepBowl` |
| `heldGroceries` | Armful of ingredients (bag or crate) | 0.2–0.45 × 0.15–0.4 × 0.12–0.35 |  | carry ingredients to the worktop |  |  | neufert | `item.ingredients` |
| `heldPot` | Saucepan or casserole (20-28 cm, with handles) | 0.28–0.42 × 0.1–0.2 × 0.2–0.3 |  | carry the meal to the table; take it to the sink |  |  | neufert | `item.mealPot` `item.potDirty` |
| `heldPlatter` | Serving platter or board | 0.3–0.45 × 0.02–0.1 × 0.2–0.32 |  | serve food for several people |  |  | neufert | `item.tastingBoard` `item.grillPlatter` `item.platterDirty` |
| `heldPizza` | Whole pizza on a board or peel | 0.28–0.42 × 0.02–0.06 × 0.28–0.42 |  | carry the pizza to the table |  |  | neufert | `item.pizza` |
| `heldTray` | Baking tray (oven tray) | 0.3–0.45 × 0.015–0.06 × 0.25–0.38 |  | take baking out of the oven |  |  | en1116, neufert | `item.bakingTray` `item.trayDirty` |
| `heldPieDish` | Pie dish | 0.22–0.34 × 0.04–0.09 × 0.22–0.34 |  | carry the pie to the table |  |  | neufert | `item.pieDish` |
| `fryingPan` | Frying pan (24-28 cm, with handle) | 0.24–0.3 × 0.04–0.08 × 0.4–0.52 |  | fry on the hob | fry 5–20 |  | neufert | `tool.pan` |
| `grillFood` | Food on the grill (sausages, skewers) | 0.15–0.35 × 0.02–0.06 × 0.08–0.2 |  | turn it on the grill | cook 8–20 |  | neufert | `tool.grillFood` |

### Houses

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `house.door` | Door leaf in its frame (interior or front door) | 0.76–0.93 × 1.98–2.11 × 0.035–0.055 |  | open and walk through; knock (visitors); shut for privacy |  | door 90–110° | din18101, neufert | `house.door` |
| `house.window` | Window | 0.5–2.4 × 0.6–2.2 × 0.05–0.2 |  | look out; open for air |  | casement 30–90°, tiltTurn 10–15° | adKWindows, neufert | `house.window` |
| `house.stairs` | Straight stair, one storey (2.6-3.0 m) | 0.8–1.2 × 2.4–3.1 × 2.8–4.2 |  | go up and down between storeys |  |  | adK, neufert | `stairs` `house.stairs` |
| `house.wall` | Wall, one storey | 0.1–30 × 2.4–3.1 × 0.1–0.4 |  | divide rooms; carry pictures, cabinets and lights |  |  | neufert, adKWindows | `house.wall` |

### Cars

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `car.saloon` | Saloon car (D segment) | 1.75–1.87 × 1.4–1.5 × 4.4–4.95 |  | drive to work; park on the drive |  | door 60–75°, bootLid 70–90° | bmw3 | `car.saloon` |
| `car.hatchback` | Hatchback (B-C segment) | 1.7–1.82 × 1.4–1.55 × 3.9–4.35 |  | drive to the shops |  | door 60–75°, tailgate 80–100° | golf | `car.hatchback` |
| `car.suv` | Compact SUV | 1.8–1.95 × 1.6–1.75 × 4.3–4.8 |  | drive the family |  | door 60–75°, tailgate 80–100° | rav4 | `car.suv` |
| `car.estate` | Estate car | 1.78–1.87 × 1.45–1.55 × 4.55–4.9 |  | carry luggage or a dog |  | door 60–75°, tailgate 80–100° | octavia, golf | `car.estate` |

### Residents

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `resident.baby` | Baby (0-2 years), lying or sitting | h 0.45–0.94 |  | be fed, changed, cuddled; sleep 11-16 h a day | sleep 660–960, feed 10–30, change 3–10 |  | whoLength, aasm, dunlop | `resident.baby` |
| `resident.child` | Child (2-12 years) | h 0.83–1.7 |  | play; go to school; sleep 9-12 h | sleep 540–780 |  | cdcAnthro, aasm, dunlop | `resident.child` |
| `resident.teen` | Teen (13-17 years) | h 1.45–1.88 |  | go to school; sleep 8-10 h | sleep 480–600 |  | cdcAnthro, aasm | `resident.teen` |
| `resident.youngAdult` | Young adult (18-29 years) | h 1.5–1.88 |  | work or study; sleep 7-9 h | sleep 420–540 |  | cdcAnthro, nsf | `resident.youngAdult` |
| `resident.adult` | Adult (30-59 years) | h 1.5–1.89 |  | work; sleep 7-9 h | sleep 420–540 |  | cdcAnthro, nsf | `resident.adult` |
| `resident.elder` | Elder (60+ years) | h 1.45–1.85 |  | retire; sleep 7-8 h | sleep 420–480 |  | cdcAnthro, nsf | `resident.elder` |

### Other non-content props

| Kind | Real thing | W × H × D (m) | Functional (cm) | Uses | Minutes | Mechanisms | Sources | Objects |
|---|---|---|---|---|---|---|---|---|
| `selectionMarker` | Selection star over a resident (UI marker, no real-world size) | h 0.2–0.4 |  | show who is selected *(decor)* |  |  | neufert | `model.marker` |

## Sources

Every kind cites at least one of these keys. Several are retailer or guide pages that quote a standard or a catalogue; where sources disagreed at the edges, the range covers both. Neufert is the general reference for things no catalogue or standard covers (ornaments, toys, garden structures, street props); those ranges are the weakest and the first to revisit if a review finding depends on them.

| Key | Source |
|---|---|
| `neufert` | [Neufert, Architects' Data, 5th edition (Wiley-Blackwell 2019, ISBN 9781119284352)](https://bookshop.riai.ie/fr/products/architects-data) |
| `cdcAnthro` | [Fryar et al., Anthropometric Reference Data for Children and Adults: United States, 2015-2018 (NCHS Vital Health Stat 3(46), 2021), tables 7, 9, 11](https://stacks.cdc.gov/view/cdc/100478) |
| `whoLength` | [WHO Child Growth Standards, length-for-age, birth to 2 years (percentile field tables)](https://www.who.int/tools/child-growth-standards/standards/length-height-for-age) |
| `dunlop` | [Dunlop, Anatomical Diagrams for the Use of Art Students (1912): relation of head to height by age (USF ClipArt ETC)](https://etc.usf.edu/clipart/52100/52139/52139_head.htm) |
| `en1116` | [EN 1116:2018 Kitchen furniture, coordinating sizes (CATAS summary: worktop 850/900/950 mm, 0/+50)](https://catas.com/uploads/media/unien1116-eng.pdf) |
| `en527` | [BS EN 527-1:2011 Office furniture, work tables and desks, dimensions](https://knowledge.bsigroup.com/products/office-furniture-work-tables-and-desks-dimensions) |
| `en1335` | [BS EN 1335-1:2020 Office work chair dimensions (seat height types A 400-520, B 420-510 mm)](https://knowledge.bsigroup.com/products/office-furniture-office-work-chair-dimensions-determination-of-dimensions) |
| `ada604` | [2010 ADA Standards 604.4: water closet seat 17-19 in (430-485 mm); dwellings 15-19 in](https://www.corada.com/documents/2010ADAStandards/604-4) |
| `adK` | [Approved Document K, private stairs (BWF summary: rise 150-220 mm, going 220-300 mm, 2R+G 550-700, pitch max 42 deg)](https://www.bwf.org.uk/question-of-the-week/what-are-the-current-recommendations-for-the-rise-and-going-of-stairs/) |
| `adKWindows` | [LABC, Childproofing windows: Approved Document K guarding 800 mm, Approved Document O 1100 mm](https://labc.co.uk/news/how-get-it-right-childproofing-windows) |
| `din18101` | [DIN 18101 door sizes (Baunetz Wissen: e.g. 875 x 2000 mm opening, nominal 885 x 2005)](https://www.baunetzwissen.de/fenster-und-tueren/fachwissen/konstruktion-funktion/wandoeffnungen-und-tuergroessen-nach-din-18101-155227) |
| `dimSideChairs` | [Dimensions.com, Side chairs: seat 42-48 cm, width 42-56 cm](https://www.dimensions.com/collection/side-chairs-dining-chairs) |
| `dimChairs` | [Dimensions.com, Chairs: seats 40.6-50.8 cm high](https://www.dimensions.com/collection/chairs-stools) |
| `dimStools` | [Dimensions.com, Stools: counter 23-28 in, bar 29-32 in, tall bar 33-36 in seat height](https://www.dimensions.com/collection/stools) |
| `dimDiningTables` | [Dimensions.com, Dining tables: 28-30 in (71-76 cm)](https://www.dimensions.com/collection/dining-tables) |
| `dimCafeTables` | [Dimensions.com, Cafe tables: 28-30 in high](https://www.dimensions.com/element/cafe-tables) |
| `dimCoffeeTables` | [Dimensions.com, coffee tables 10-18 in (25-46 cm), level with or 2 in below the sofa seat](https://www.dimensions.com/element/cage-coffee-table) |
| `dimNightstands` | [Dimensions.com, nightstands (e.g. IKEA Tarva): 18-26 in high, 13.75-18 in deep](https://www.dimensions.com/element/ikea-tarva-nightstand) |
| `dimDressers` | [Dimensions.com, Dressers & chests: d 46-61, h 76-127, w 76-188 cm](https://www.dimensions.com/collection/dressers-chests) |
| `dimArmoires` | [Dimensions.com, Armoires & wardrobes: depth 50-61 cm](https://www.dimensions.com/collection/armoires-wardrobes) |
| `dimBedFrames` | [Dimensions.com, Bed frames: mattress tops 53-64 cm on common frames](https://www.dimensions.com/collection/bed-frames) |
| `dimSofa` | [Dimensions.com, Floyd 3-seater sofa (seat 45 cm) and sofa listings 39-48 cm](https://www.dimensions.com/element/floyd-3-seater-sofa) |
| `dimToilets` | [Dimensions.com, Toilets: depth 69-76 cm, width ~51 cm, back 53-79 cm high](https://www.dimensions.com/collection/toilets) |
| `dimVanities` | [Dimensions.com, Bathroom vanities: 30-32 in, comfort height to 36 in](https://www.dimensions.com/collection/bathroom-vanities) |
| `dimBasin` | [Dimensions.com, Bathroom sink heights: 76-91 cm](https://www.dimensions.com/element/bathroom-sink-washbasin-heights) |
| `dimBathClear` | [Dimensions.com, Bathtub clearances: tub 51-60 cm high](https://www.dimensions.com/element/bathtubs-baths-side-clearances) |
| `dimFridges` | [Dimensions.com, Refrigerators (top freezer 66 x 30 x 32 in; French door 69.9 x 35.6 x 35.6 in)](https://www.dimensions.com/collection/refrigerators-fridges) |
| `dimLaundry` | [Dimensions.com, Whirlpool front-load washer and dryer 99 x 69 x 79-84 cm; stacked 198 cm](https://www.dimensions.com/element/laundry-machines-whirlpool-front-load-washer-dryer) |
| `dimDishwasher` | [Dimensions.com, Bosch 24 in dishwasher (60 x 86 x 60 cm); 18 in models 45 cm wide](https://www.dimensions.com/element/bosch-500-series-pocket-handle-dishwasher-24) |
| `dimExBike` | [Dimensions.com, Exercise bicycles](https://www.dimensions.com/element/exercise-bicycles) |
| `dimPicnic` | [Dimensions.com, Rectangular picnic table: 183 x 149 x 77 cm, bench 43 cm](https://www.dimensions.com/element/rectangular-picnic-table) |
| `dimBench` | [Dimensions.com, Plank bench: benches 46-51 cm high; 127 cm seats two](https://www.dimensions.com/element/plank-bench) |
| `dimDrinkware` | [Dimensions.com, Drinkware (wine glasses 15.5-26.4 cm, IKEA 365+ mug 7 x 10.5 cm)](https://www.dimensions.com/collection/drinkware) |
| `dimKitchen` | [Dimensions.com, Kitchen cabinetry clearances: base cabinet 61 cm deep, worktop 91 cm](https://www.dimensions.com/element/kitchen-cabinetry-clearances) |
| `lgFridges` | [LG UK, Standard fridge sizes](https://lg.com/uk/lg-experience/helpful-hints/standard-fridge-sizes) |
| `samsungFit` | [Samsung "Will it fit" guide RF27CG5400: doors open 90 deg with clearance, 125 deg full](https://image-us.samsung.com/SamsungUS/home/pdf/08062024/WillitFit_RF27CG5400_V3.pdf) |
| `electroluxFit` | [Electrolux bottom-freezer installation guide: 120 deg door opening](https://www.electroluxarabia.com/globalassets/appliances/fridge/ebe4302c-b/260312adyi.pdf) |
| `aoCookers` | [AO.com cooker and oven measuring guide: freestanding 50/55/60 cm, range 90/100/110 cm, h 90, d 60-65](https://ao.com/help-and-advice/guides-and-advice/measurement-guides/cooker-and-oven) |
| `washerUk` | [Marks Electrical washing machine size guide: H 85 x W 60 cm, D 53-67 cm](https://markselectrical.co.uk/laundry/guides/washing-machine-size-guide) |
| `ww8500` | [iF Design Award, Samsung WW8500 washer: 180 deg door opening](https://ifdesign.com/en/winner-ranking/project/ww8500/176358) |
| `microwave` | [KitchenAid microwave dimensions: countertop 12-15 in high, 21-25 in wide, 16-20 in deep](https://www.kitchenaid.com/pinch-of-help/major-appliances/microwave-dimensions.html) |
| `blum` | [Blum CLIP top hinges (95/100/110/120/155/170 deg), retailer listing](https://www.cabinetparts.com/p/blum-hinges-european-cabinet-hinges-BH71B7550BLK-p141757) |
| `dellLid` | [Dell, maximum lid opening angle per laptop (135 or 180 deg stops)](https://dell.com/support/kbdoc/en-in/000203122/identify-the-maximum-possible-lid-opening-angle-on-a-dell-laptop) |
| `ittf` | [ITTF Laws of Table Tennis 2026-27, law 2.1.1: 2.74 x 1.525 m at 76 cm](https://www.tabletennisengland.co.uk/content/uploads/2026/07/Laws-of-Table-Tennis-2026-27.pdf) |
| `wpa` | [WPA pool table specification: bed 29.25-31 in (74.3-78.7 cm), 9 ft play area 50 x 100 in](https://www.cuesportsindia.com/equipments/3/pool-table) |
| `darts` | [DartCounter, dartboard height and distance: bull 173 cm, oche 237 cm](https://dartcounter.net/darts-manual/dartboard-height-distance) |
| `fiba` | [Basket (basketball), Wikipedia: FIBA rim 3,050 mm, ring 460 mm](https://en.wikipedia.org/wiki/Basket_(basketball)) |
| `foosball` | [Pool Table Portfolio, foosball table sizes: 137-142 x 74-76 x 86-91 cm](https://pooltableportfolio.com/blogs/magazine/how-big-is-a-foosball-table-a-comprehensive-guide-to-foosball-dimensions-and-table-options) |
| `yamahaB1` | [Thomann, Yamaha b1 upright piano: 109 x 148 x 54 cm](https://m.thomann.de/ro/yamaha_b1_pec.htm) |
| `pianoBench` | [Piano bench height: 48-56 cm for adult benches](https://www.piano-composer-teacher-london.co.uk/how-tall-is-a-piano-bench/) |
| `arcade` | [Dreamcade, arcade cabinet dimensions (uprights 62-71 in tall)](https://support.dreamarcades.com/knowledgebase.php?article=12) |
| `jukebox` | [Liberty Games, jukebox dimensions (Wurlitzer 1015: about 155 x 85 x 68 cm)](https://www.libertygames.co.uk/faq/answer/jukebox-dimensions/) |
| `treadmill` | [Garage Gym Reviews, treadmill dimensions: 60-84 x 24-36 in, 48-60 in tall](https://www.garagegymreviews.com/treadmill-dimensions) |
| `hotTub` | [Sundance Spas, hot tub dimensions: 33-38.5 in tall, medium 6 ft 6 in-7 ft 9 in](https://www.sundancespas.com/en-us/A-Comprehensive-Guide-to-Size-and-Space-Considerations.html) |
| `en1176` | [BS EN 1176-2 Playground equipment, swings](https://knowledge.bsigroup.com/products/playground-equipment-and-surfacing-additional-specific-safety-requirements-and-test-methods-for-swings) |
| `pendant` | [Juno/Loloi, pendant height over a table: 30-36 in above the top](https://support.joonloloi.com/knowledge-base/what-is-the-correct-height-to-hang-a-pendant-over-a-dining-table-or-kitchen-island) |
| `bin240` | [Test Valley Borough Council 240 L wheeled bin specification (EN 840): w 580, d 740, h max 1100 mm](https://testvalley.gov.uk/wasteandrecycling/bin-specifications) |
| `usps` | [USPS Postal Bulletin 22649 (2024): curbside mailbox 41-45 in from road to box bottom](https://about.usps.com/postal-bulletin/2024/pb22649/html/cover.htm) |
| `streetlight` | [Larimer County street lighting standard ch. 15 (residential mounting height caps)](https://www.larimer.gov/sites/default/files/uploads/2021/ch15_-_street_lighting_2.pdf) |
| `hydrant` | [McWane dry-barrel fire hydrant submittal sheet](https://pe.mcwane.com/upl/downloads/catalog/products/submittal-sheet-929-75c04d89.pdf) |
| `hydrantNj` | [N.J.A.C. 5:21-5.4 Fire hydrants (AWWA C502; outlets at least 12 in above grade)](https://www.law.cornell.edu/regulations/new-jersey/N-J-A-C-5-21-5-4) |
| `golf` | [Volkswagen Golf Mk8, Wikipedia: 4,284 x 1,789 x 1,456 mm; estate 4,633 mm long](https://en.wikipedia.org/wiki/Volkswagen_Golf_Mk8) |
| `bmw3` | [BMW 3 Series (G20), Wikipedia: 4,709 x 1,827 x 1,442 mm](https://en.wikipedia.org/wiki/BMW_3_Series_(G20)) |
| `rav4` | [CarExpert, Toyota RAV4 dimensions: 4,600 x 1,855 x 1,685 mm](https://www.carexpert.com.au/toyota/rav4/2023/dimensions) |
| `octavia` | [Parkers, Skoda Octavia Estate (2020 on) specs: about 4,689-4,698 mm long](https://parkers.co.uk/skoda/octavia/estate-2020/specs/) |
| `k6` | [Science Museum Group, Post Office telephone kiosk No. 6 (K6): 2540 x 910 x 910 mm](https://collection.sciencemuseum.org.uk/objects/co33578/post-office-telephone-kiosk-no-6-1936-1968-telephone-box) |
| `pillarBox` | [Letter Box Study Group, Type K pillar box: 63 in high, 19.25 in wide](https://lbsg.org/?p=48) |
| `tractor` | [TractorData, Hinomoto E2004 compact tractor: 248 x 121 x 185 cm](https://www.tractordata.com/farm-tractors/004/7/5/4757-hinomoto-e2004-dimensions.html) |
| `rhsRootstocks` | [RHS, Rootstocks for fruit: M27 1.5-1.8 m, M9 2.4-3.6 m, M106 3.6-5.4 m](https://www.rhs.org.uk/fruit/fruit-trees/rootstocks) |
| `rhsPlants` | [RHS Find a plant (ultimate height and spread, time to reach)](https://www.rhs.org.uk/plants/search-form) |
| `birch` | [Betula pendula, Wikipedia: typically 15-25 m](https://en.wikipedia.org/wiki/Betula_pendula) |
| `acerPalmatum` | [Acer palmatum, Wikipedia: 6-10 m](https://en.wikipedia.org/wiki/Acer_palmatum) |
| `sunflower` | [Common sunflower, Wikipedia: typical height 3 m](https://en.wikipedia.org/wiki/Common_sunflower) |
| `bedSizes` | [NaturalMat, UK bed sizes: single 90 x 190, double 135 x 190, king 150 x 200, super king 180 x 200](https://naturalmat.co.uk/blogs/faq/what-are-the-standard-uk-bed-sizes) |
| `ikeaBilly` | [IKEA BILLY bookcase 80 x 28 x 202 cm](https://www.ikea.com/gb/en/p/billy-bookcase-white-00263850/) |
| `ikeaSvarta` | [IKEA SVÄRTA bunk bed frame 90 x 200: 208 x 97 x 159 cm](https://www.ikea.com/lv/en/p/svaerta-bunk-bed-frame-silver-colour-10247973/) |
| `ikeaKritter` | [IKEA KRITTER junior bed 70 x 160: 165 x 75 x 68 cm](https://www.ikea.com/gb/en/p/-80125124) |
| `ikeaSundvik` | [IKEA SUNDVIK cot 60 x 120: 125 x 67 x 85 cm](https://www.ikea.cn/cn/en/p/sundvik-cot-grey-30494076) |
| `ikeaAntilop` | [IKEA ANTILOP highchair: seat 54 cm, 56 x 59-62 x 90 cm](https://www.ikea.cn/cn/en/p/antilop-highchair-with-tray-white-white-59597516/) |
| `ikeaLatt` | [IKEA LÄTT children's table and chairs: table 63 x 48 x 45, seat 28 cm](https://www.ikea.com/us/en/catalog/products/50178411/) |
| `ikeaPoang` | [IKEA POÄNG armchair: 68 x 82 x 100 cm, seat 42 cm](https://mgmt.ikea.gr/en/products/poang-armchair/49625020/) |
| `ikeaEktorp` | [IKEA EKTORP 3-seat sofa: 218 x 88 x 88 cm, seat 45 cm; with chaise 252 wide, chaise 163 deep](https://mgmt.ikea.gr/en/products/ektorp-3-seat-sofa/39508998) |
| `ikeaMalm6` | [IKEA MALM chest of 6 drawers: 160 x 48 x 78 cm](https://ikea.com/nl/en/p/malm-chest-of-6-drawers-white-60403584) |
| `ikeaPax` | [IKEA PAX wardrobe frames: 50/75/100 wide, 58 deep, 201/236 high](https://itemfits.com/dimensions/ikea/ikea-pax) |
| `ikeaMicke` | [IKEA MICKE desk: 105 x 50 x 75 cm (73 and 142 wide variants)](https://itemfits.com/dimensions/ikea/ikea-micke) |
| `ikeaHemnesDaybed` | [Dimensions.com, IKEA HEMNES daybed: 83 high, 199 x 108, seat about 53 cm](https://www.dimensions.com/element/ikea-hemnes-daybed-3-drawers) |
| `ikeaLack` | [Dimensions.com, IKEA LACK coffee table: 90 x 55 x 45 cm](https://www.dimensions.com/element/ikea-lack-coffee-table) |
| `bathSizes` | [Bathroom City, standard bath size guide: 1700 x 700 mm](https://www.bathroomcity.co.uk/blog/standard-bath-size-guide) |
| `showerEnclosures` | [Homebuilding & Renovating, shower enclosures guide: 1.7-2.0 m high, 800-1200 mm trays](https://www.homebuilding.co.uk/buying-guides/shower-enclosures-guide) |
| `estShower` | [Energy Saving Trust At Home with Water (UKERC case study): mean shower 7.5 min, 87 % under 10 min](https://ukerc-observatory.ac.uk/case_study/energy-saving-trust-research-into-home-water-energy-practices-and-trial-interventions/) |
| `atus` | [US BLS American Time Use Survey, latest numbers (2024: eating 1.24 h/day, sleep 9.04 h)](https://www.bls.gov/tus/latest-numbers.htm) |
| `aasm` | [AASM paediatric sleep duration consensus (teens 8-10 h, 6-12 y 9-12 h, 1-2 y 11-14 h)](https://aasm.org/resources/pdf/pediatricsleepdurationmethods.pdf) |
| `nsf` | [National Sleep Foundation updated sleep duration recommendations (2015): adults 7-9 h](https://profiles.wustl.edu/en/publications/national-sleep-foundations-updated-sleep-duration-recommendations/) |
