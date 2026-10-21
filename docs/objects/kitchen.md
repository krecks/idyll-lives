# Kitchen

Appliances, sinks, counters, islands and dining furniture. `families/kitchen.py` builds the fridges and the two
kitchen sinks (notes: `kitchen.md`), `families/kitchen_cooking.py` the K2 cookers and the K3 small appliances and
dishwashers (`kitchen_cooking.md`), `families/kitchen_counters.py` the K4 base counters, the K5 tall and wall storage
and the K6 islands and bars (`kitchen_counters.md`), and `families/dining.py` the K7 dining tables, the K8 chairs and
stools and the K9 dining storage (`dining.md`).

Doors and drawers follow the fridge: hinged parts built closed, `G.swing` / `G.slide`, a `hold` so they shut while
the resident is still busy. Water grows from the tap (`water_on`) for the whole use. Counter and table tops keep
their heights (the content's `surface`; every kitchen worktop is 0.95 m); chairs and
stools keep their seat height (`seats` in `web/src/render/babylon/characters/furniture.json`). Clean things are put away in the
nearest cupboard or counter (cooking `putAwayAt: ["cupboard", "prep"]`): the wall cabinets and the dining storage are
`cupboard` stations and open a door while it happens, the drawer counter its top drawer.

**Who uses them** (content `stages`, 2026-10-21, G-45): the wine fridge's *Pour a glass of wine*, the bar cart's *Mix a
drink* and the party bar's *Mix drinks for everyone* are for young adults and up (a child or a teen is refused, "Dee
is too young to pour a glass of wine", never picks them, and sees them greyed out; the party bar's soda is anyone's).
Every recipe with a hot step (meal, gourmet, tasting menu, both barbecues, pizza, baked treats, pie; recipe `stages`)
is cooked by teens and up, wherever it is started (a fridge, a stove, an oven, a grill); a child may make a sandwich
and heat up leftovers. Coffee is left to anyone (not an age rule yet).

## Kitchen sinks (2)

- **Family:** `tools/art/blender/families/kitchen.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `farmhouseSink` Farmhouse sink | 1x1 · floor | wash:wash washUp:wash | kitchen / counters | Cabinet, Worktop, Basin, Tap & handles, Plinth | Use | `water_on`: the stream grows from the gooseneck tap into the apron-front basin in 0.4 s, runs for the whole use and retracts as it ends. |
| `counterSinkDouble` Double sink | 2x1 · floor | wash:wash washUp:wash | kitchen / counters | Cabinets, Worktop, Basins, Tap & handles, Plinth | Use | `water_on_0`, `water_on_1`: each bridge tap runs for the one washing at its bowl (`slot`; 2026-10-21: one washer ran both). |

**In play** (signed off 2026-10-16):
- Checked with the bathroom sinks, the first group: bought in all four facings, every part coloured (through `buy` and
  in the colour studio), washed at and washed up at, by day and at night, on WebGPU and WebGL2, on Medium and Low.
- `water_on` has no `hold`: it plays for the `wash` tag (both `wash` and `washUp`) and stops when the use ends.
- Since 2026-10-21 (review A-kitchen-03) the double sink runs only the tap of the bowl in use (`slot` 0 at -x, 1 at
  +x: `fixes/motions/g1718/play-stage.jsonl`).
- Both are `dishes` stations: a cleared plate is rinsed here between the bin and the cupboard. Housegen keeps small
  things off sinks.
- Since the authenticity review (G-15): at the farmhouse and double sinks the washer steps in (`STAND_BACK` -0.45),
  the hands in the basin under the tap.

## K1. Fridges (7)

- **Family:** `tools/art/blender/families/kitchen.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `fridge` Fridge | 1x1 · floor | snack:fetch frozenTreat:fetch meal:cook gourmet:cook coldDrink:drink | kitchen / appliances | Body, Handles, Plinth & badge, Inside | Use | `fridge_open`: the tall fridge door below swings 102° for snack, meal and gourmet; `hold` 3.5 s. The freezer door above swings 118° (`freezer_open`) for frozenTreat. Has the other fridges' cold drink (2026-10-21). |
| `fridgeMini` Mini fridge | 1x1 · floor | snack:fetch meal:cook coldDrink:drink | kitchen / appliances | Body, Handles, Plinth | Use | `fridge_open`: the door swings 100° on its hinge for snack and meal; `hold` 3 s. Has a cold drink, as the retro mini fridge. |
| `fridgeBuiltIn` Counter-depth fridge | 1x1 · floor | snack:fetch frozenTreat:fetch meal:cook gourmet:cook coldDrink:drink | kitchen / appliances | Panels, Handles | Use | `fridge_open`: the tall fridge door swings for snack, meal, gourmet and coldDrink; the freezer door below opens (`freezer_open`) for frozenTreat. Renamed from "Built-in fridge": a free-standing counter-depth fridge with a dispenser (kind `fridgeAmerican`, Q-08). |
| `fridgeFrenchDoor` French-door fridge | 1x1 · floor | snack:fetch frozenTreat:fetch meal:cook gourmet:cook coldDrink:drink | kitchen / appliances | Body, Handles, Plinth | Use | `fridge_open`: both upper doors swing out (-100° / +100°) for snack, meal, gourmet; `door_right` (the right door only) for coldDrink. The freezer drawer slides out 35 cm (`freezer_open`) for frozenTreat. |
| `fridgeRetro` Retro fridge | 1x1 · floor | snack:fetch meal:cook coldDrink:drink | kitchen / appliances | Body, Handle, Trim | Use | `lever`: the chrome lever (a part inside the door) turns 30° out, then `fridge_open` (`delay` 0.3 s): the rounded door swings 100°; snack, meal, coldDrink. |
| `fridgeMiniRetro` Retro mini fridge | 1x1 · floor | snack:fetch meal:cook coldDrink:drink | kitchen / appliances | Body, Handle & feet, Trim | Use | `fridge_open`: the pastel door with its unicorn sticker swings 100° on its hinge; snack, meal, coldDrink. `hold` 3 s. |
| `wineFridge` Wine fridge | 1x1 · floor | wine:drink | kitchen / appliances | Body, Door frame | Use | `door_open`: the framed smoked-glass door (the bottles show through it) swings for wine, `hold` 2.5 s; once it is open its top shelf of bottles slides out 15 cm (`rack_out`, `delay` 0.9, `hold` 0.9). |

**In play** (signed off 2026-10-16):
- Every use (snack, meal, gourmet, cold drink, wine) opens its door clip and shuts it after its `hold`; the retro
  fridge pulls its lever first, the French-door fridge opens only its right door for a cold drink.
- Gourmet also needs a stove and a counter (its prep and cook steps); checked with both bought.
- The three freezers (`model.fridge`, the built-in, the French-door) open for "Get some ice cream" (`frozenTreat`,
  0.3.0); not part of the 2026-10-16 sign-off.
- The colour studio's "In use" opens the door, so the fridge's "Inside" colour can be seen.

## K2. Cookers (6)

- **Family:** `tools/art/blender/families/kitchen_cooking.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `stoveElectric` Electric cooker | 1x1 · floor | cook:cook bake:cook | kitchen / appliances | Body, Knobs, Hob | Use | `oven_open`: the oven door drops 85° on its bottom hinge for bake (and any `bake` step), shut after 3 s; `burner_on`: the front-left knob turns 90° and the front ring on its side glows red for cook (and any `stir` step: a meal started at the fridge). 4 draws. |
| `stoveGas` Gas range | 1x1 · floor | cook:cook gourmet:cook bake:cook | kitchen / appliances | Body, Knobs, Hob | Use | `oven_open` for bake and any `bake` step (door drops); `burner_on` for cook, gourmet and any `stir` step: a knob turns and a ring of blue flame (gloss, not emissive) grows on the front burner, the `water_on` way. |
| `wallOven` Oven tower | 1x1 · floor | roast:cook bake:cook | kitchen / appliances | Tower, Handles | Use | `oven_open`: the upper oven door drops and its tray slides out 10 cm for roast, bake and any `bake` step (a meal cooked in it is put in, `animAt`), shut after 3 s. |
| `rangeCooker` Range cooker | 2x1 · floor | cook:cook gourmet:cook bake:cook | kitchen / appliances | Body, Knobs & rail, Hob | Use | `oven_open`: the left oven door drops for bake and any `bake` step; `burner_on` (knob and flame ring) for cook, gourmet and any `stir` step. |
| `cooktopCounter` Hob counter | 1x1 · floor | cook:cook | kitchen / appliances | Cabinet, Worktop, Hob | Use | `burner_on`: a knob turns and the front-right ceramic ring glows red (grown from nothing, as a flame) for cook and any `stir` step. |
| `microwave` Microwave counter | 1x1 · floor | reheat:cook | kitchen / appliances | Cabinet, Worktop, Microwave | Use | `door_open`: the microwave door swings open and shuts again (`hold` 1.2 s) for reheat and a meal's `bake` step (a meal cooked in it, `animAt`). `turntable`: once the door has shut (`delay` 2.2 s) the glass plate turns, every 2 s while it heats. |

**In play** (signed off 2026-10-16):
- Each use run as a recipe (fridge, then a counter, then the cooker): the flame rings and the electric cooker's knob
  for cook and gourmet, the oven doors for bake and roast, the hob counter's hot ring, the microwave's door and
  turntable for reheat.
- The hob clip (`burner_on`) lasts the whole use; `oven_open` shuts after its `hold` (3 s). Each shuts
  when the resident moves on.
- Since 2026-10-21 (G-16) the clips play by the cooking step's tag too, so a meal started at the fridge lights the hob
  (`stir`), and a bake or a meal put in an oven or a microwave (`bake`, the meal's `animAt`) opens its door; the
  resident bends to a cooker's oven or stands at the tower's and the microwave, with no pan drawn. The Mix step and the
  pie's pastry (`mix`) are stirred in a mixing bowl (`tool.mixingBowl`) on the counter. Recipe runs:
  `bench-results/review/fixes/motions/recipes/`.
- Each oven has a dark cavity inside its door and the microwave is hollow, so an open door shows an oven, not a block.
- Since the authenticity review (G-08, G-15): the pan stands on the burner the cooker's clip lights (`STATION_SPOTS`:
  the gas range's flame, the range cooker's middle ring, the hob counter's hot ring; the electric cooker's front ring
  on the turning knob's side); the microwave is used stepped up to it on the counter's right (`STEP_IN`), the hands at
  its door, not at the cabinet below.

## K3. Small appliances and dishwashers (4)

- **Family:** `tools/art/blender/families/kitchen_cooking.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `coffeeMachine` Coffee machine | 1x1 · floor | espresso:drink | kitchen / appliances | Cabinet, Worktop, Machine | Use | Provides `coffee`; `brew`: a cup slides under the spout and a thin coffee stream grows for espresso. |
| `breakfastCounter` Breakfast counter | 1x1 · floor | toast:cook smoothie:cook | kitchen / appliances | Counter, Toaster, Blender | Use | `toast_down`: the toaster's carriage (lever and toast, one part) slides 5 cm down for toast and pops up at the end. `blend` for smoothie: the blender lid lifts. L `blades`: the smoothie under the lid swirls while it blends (E1). Its 0.95 m top is a `counter` (counter stools tuck under it). |
| `dishwasher` Dishwasher | 1x1 · floor | dishes:clean | kitchen / appliances | Body, Handle, Panel | Use | A 60 cm dishwasher beside a 40 cm cupboard under one worktop (G-32). `door_open`: its door drops 85° and the lower rack slides out 30 cm for dishes and for clearing away at it (the `wash` tag), open for the whole loading. |
| `dishwasherCompact` Stainless dishwasher | 1x1 · floor | dishes:clean | kitchen / appliances | Body, Handle, Panel | Use | A slimline 45 cm dishwasher beside a 55 cm stack of three drawers (G-32). `door_open`, as the dishwasher. |

**In play** (signed off 2026-10-16):
- The dishwashers' doors drop 0.44 m into the tile in front: the loader stands 15 cm further back while using either
  one, at the edge of the open door (`characters.ts` `STAND_BACK`).
- The breakfast counter keeps 4 draw calls: the toaster's lever and toast are one part, and the smoothie swirls
  (`blades`, a loop) under the lifted lid.
- The coffee machine's cup and stream, the toaster and both dishwashers' doors and racks play and shut as they should.
- Since 2026-10-21 (G-16) a plate or a pot cleared away at a dishwasher opens its door (the `wash` tag) and makes no
  sound (it was the washing-up water); the door stays open for the whole load.
- Coffee at the dining tables and chairs needs a working coffee machine on the same lot (`requires` / `provides`).
- Since the authenticity review (G-15): toast and smoothies at the breakfast counter are made stepped in (`STAND_BACK`
  -0.3), the hands on the top, with no squat to the floor.

## K4. Base counters (8)

- **Family:** `tools/art/blender/families/kitchen_counters.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `counter` Kitchen counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Worktop, Handles | Use | `drawer_open`: the top drawer slides out 25 cm for sandwich; `hold` 1.5 s. |
| `counterCorner` Corner counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Worktop, Handles | Use | `drawer_open`, as the counter. |
| `counterEnd` Counter end with shelves | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Worktop, Handles | Static | Static (open end shelves under one worktop with the unit). A sandwich, as the other counters. |
| `counterCornerOuter` Outer corner counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Worktop, Handles | Static | Static. A sandwich, as the other counters. |
| `counterDrawers` Drawer counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Worktop, Handles | Use | `drawer_open`: the top drawer slides out 25 cm while clean things are put away in it (`fetch` at its `prep` station, `hold` 1.5 s). A sandwich, as the other counters. |
| `counterOpenShelf` Open-shelf counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Worktop | Static | Static. A sandwich, as the other counters. |
| `butcherBlock` Butcher block | 1x1 · floor | sandwich:cook | kitchen / counters | Base, Block, Top | Use | `drawer_open`: the knife drawer slides out for sandwich. |
| `counterMarble` Marble counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Marble top, Handles | Use | `drawer_open` for sandwich. |

**In play** (signed off 2026-10-16):
- The counter, corner counter, butcher block and marble counter open their drawer for a sandwich (fridge, then the
  counter); the drawer counter opens its top drawer when a snack's plate is cleared (bin, sink, then `putAwayAt`).
- The end, outer corner and open-shelf counters serve as a meal's prep station (`prep`).
- The open-shelf counter's back is the cabinet's colour outside, so it reads when the counter stands free.
- The butcher block's top surface has its own colour (`Top`); "Block" is the block's rim and body.

## K5. Tall and wall storage (11)

- **Family:** `tools/art/blender/families/kitchen_counters.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `pantry` Pantry cupboard | 1x1 · floor | snack:fetch | kitchen / counters | Cabinet, Handles | Use | `doors_open`: both doors swing out 100° for snack; `hold` 2.5 s. |
| `pantryTall` Tall larder | 1x1 · floor | snack:fetch | kitchen / counters | Cabinet, Handles | Use | `doors_open`, as the pantry. |
| `wallCabinet` Wall cabinets | 1x1 · wall | — | kitchen / counters | Cabinet, Handles | Use | `doors_open`: both doors swing open 100° in 0.8 s onto plates, bowls and mugs while clean things are put away (a `cupboard` station, `fetch`, `hold` 1.5 s). |
| `wallCabinetGlass` Glazed wall cabinet | 1x1 · wall | — | kitchen / counters | Cabinet, Handles | Use | Glass doors (`glass` finish). `door_open`: the right door opens as the wall cabinet's (each glazed door costs two draws). |
| `wallCabinetCorner` Wall cabinet with end shelves | 1x1 · wall | — | kitchen / counters | Cabinet, Handles | Use | `door_open`: the door opens as the wall cabinet's. |
| `rangeHood` Cooker hood | 1x1 · wall | — | kitchen / counters | Hood, Trim | Static | Static. |
| `spiceRack` Spice rack | 1x1 · wall | — | kitchen / counters | Rack | Static | Static. Decor by decision (Q-19: kind marked decor). |
| `potRail` Pot rail | 1x1 · wall | — | kitchen / counters | Rail, Pots | Static | Static. Decor by decision (Q-19: kind marked decor). |
| `kitchenShelfWall` Kitchen wall shelves | 1x1 · wall | — | kitchen / counters | Shelves, Brackets | Static | Static. Decor by decision (Q-19: kind marked decor). |
| `trashCan` Pedal bin | 1x1 · floor | — | kitchen / counters | Bin, Lid | Use | `lid_open`: the pedal lid lifts 75° while a plate is scraped in (`clean` at its `bin` station, `hold` 1 s). |
| `recyclingBins` Recycling bins | 1x1 · floor | sortRecycling:clean | kitchen / counters | Left bin, Middle bin, Right bin (each with its lid), Handles | Use | `lid0_open`, `lid1_open`, `lid2_open`: the three lids lift one after another (0.6 s apart, each 80° in 0.5 s, `hold` 0.9 s) for "Sort the recycling". |

**In play** (signed off 2026-10-16):
- The pantry's and tall larder's doors open for a snack; the three wall cabinets' doors when clean dishes are put
  away in them, the cupboard over the counter before the counter.
- The pedal bin's lid lifts when a plate is scraped; the recycling bins' lids lift in turn for "Sort the recycling"
  (not autonomous, so generated houses play as before).
- "Left bin" and "Right bin" are left and right as seen from the front.
- The hood, spice rack, pot rail and wall shelves are still; looked at on all four walls.
- Since the authenticity review (G-15): putting dishes away in a wall cabinet is a reach at the counter (`interact`),
  with no squat to the floor; a reach up to its doors is a clip still to make.

## K6. Islands and bars (5)

- **Family:** `tools/art/blender/families/kitchen_counters.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kitchenIsland` Kitchen island | 2x1 · floor | sandwich:cook breakfast:eat | kitchen / counters | Cabinet, Worktop | Use | `drawer_open`: a drawer slides out for sandwich (not for breakfast). Breakfast is eaten on a stool at its back, or any seat within 3 m (`seatNear`), else standing. |
| `islandSmall` Small island | 1x1 · floor | sandwich:cook breakfast:eat | kitchen / islands | Cabinet, Worktop | Static | Static. A sandwich, and breakfast on a stool or seat within 3 m (`seatNear`), as the kitchen island. |
| `islandLong` Long island | 3x1 · floor | sandwich:cook breakfast:eat | kitchen / islands | Cabinet, Worktop | Static | Static. A sandwich, and breakfast on a stool or seat within 3 m (`seatNear`), as the kitchen island. |
| `barSegment` Bar counter | 1x1 · floor | — | kitchen / islands | Cabinet, Worktop | Static | Static. High bar stools seat at it (the counter stools at counters, G-07). |
| `barEnd` Bar end | 1x1 · floor | — | kitchen / islands | Cabinet, Worktop | Static | Static. |

**In play** (signed off 2026-10-16):
- Each checked standing free: the kitchen island's drawer for a sandwich and none for breakfast, eaten at it with and
  without a stool behind; the small and long islands as a meal's prep station; the bars from all four sides.
- The islands have their cupboards at the front, where residents stand, and the worktop overhanging the back, where
  housegen sets the stools (they have none of their own). Their tops are bare; housegen sets things on them.
- A stool at an island's back or a bar is drawn pulled in to it while sat on (see K8).
- Nothing needed fixing.

## K7. Dining tables (13)

- **Family:** `tools/art/blender/families/dining.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `diningTable` Dining table | 2x1 · floor | coffee:drink | kitchen / dining | Top, Legs | Static | Static. A chair at each end of a 0.84 × 0.75 top: its two (content `places` `[±0.62, 0.1]`, facing ±90) face each other across it (G-39). |
| `bistroSet` Bistro table | 1x1 · floor | breakfast:eat coffee:drink | kitchen / dining | Top, Frame, Seats | Static | Static. |
| `diningGlass` Glass dining table | 2x1 · floor | breakfast:eat coffee:drink | kitchen / dining | Top, Legs | Static | Static; a 0.9 × 0.8 glass top on two sleds along its sides, a chair at each end: its two face each other across it (G-39). |
| `diningOak` Farmhouse dining table | 2x1 · floor | breakfast:eat coffee:drink dinner:eat | kitchen / dining | Top, Legs | Static | Static; a 0.9 × 0.8 oak top with breadboard ends, a chair at each end: its two face each other across it (G-39). |
| `tableSquare2` Square table for two | 1x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableBistroRound` Round bistro table | 1x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableDining4` Dining table for six | 2x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. Renamed from "Dining table for four": 1.84 m seats six (kind `diningTable6`, Q-08). |
| `tableDining6` Dining table for eight | 3x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. Renamed from "Dining table for six": 2.84 m seats eight (kind `diningTable8`). |
| `tableBanquet8` Banquet table | 4x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. Seats twelve (kind `diningTable12`). |
| `tableRound4` Round pedestal table | 2x2 · floor | — | kitchen / dining | Top, Legs | Static | Static. 1.52 m across: seats six (kind `roundTable6`). |
| `tableSquare8` Square table for eight | 2x2 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableFarmhouse6` Farmhouse table | 3x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. 2.86 m: seats eight (kind `diningTable8`). |
| `tableBarHigh` High café table | 1x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |

**In play** (signed off 2026-10-16):
- Each checked standing free in all four facings. The four with their own seats are used as they are: coffee at the
  dining table (with a coffee machine on the lot); breakfast and coffee at the bistro set and the glass table;
  breakfast, coffee and dinner at the farmhouse dining table.
- The dining, glass and farmhouse sets' chairs are part of the model (residents sit on them) and take the table's
  "Legs" colour.
- At the nine plain tables a snack is eaten from a dining chair (a bar stool at the high café table) set facing it:
  the plate on the top, the chair drawn pulled in to the table while sat on, as far as the knees clear its legs,
  column or trestle feet (K8).
- Tops are at the content's `surface.height`: 0.75 m (1.06 m for the high café table, 0.76 m for the glass set,
  0.765 m for the dining table and the farmhouse set); tabletop things stand on them.
- Nothing needed fixing.

## K8. Chairs and stools (12)

- **Family:** `tools/art/blender/families/dining.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `barStool` Bar stool | 1x1 · floor | sit:sit drink:drink | kitchen / dining | Frame, Seat | Use | `swivel`: the seat turns 20° toward the resident and back, for sit and drink. Pulled in to the table it faces while sat on, as far as the knees fit under it. Seats at counters and islands only (`seatAt: counter`, seat 0.72 m under a 0.95 m top), not at 1.06 m bars. |
| `stoolCounter` Counter stool | 1x1 · floor | sit:sit drink:drink | kitchen / dining | Frame, Seat | Use | `swivel`, as the bar stool. Pulled in to the table it faces while sat on, as far as the knees fit under it. Seats at counters and islands only (`seatAt: counter`, seat 0.68 m). |
| `stoolBarHigh` High bar stool | 1x1 · floor | sit:sit drink:drink | kitchen / dining | Frame, Seat | Use | `swivel`, as the bar stool. Pulled in to the table it faces while sat on, as far as the knees fit under it. Seats at bars and the high café table only (`seatAt: bar`, seat 0.80 m under a 1.06 m top); generated islands no longer get it. |
| `diningChair` Dining chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static: the sitting pose is fitted to the seat, so the chair stays put. Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `chairWood` Wooden chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `chairUpholstered` Upholstered chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Upholstery | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `chairModern` Moulded chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `chairWindsor` Windsor chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `chairBentwood` Bentwood chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `chairFolding` Folding chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `chairArm` Carver chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static; the cushion at 0.475, sat on at 0.47 as a dining chair (G-30). Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `benchDining` Dining bench | 2x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. |

**In play** (signed off 2026-10-16):
- Each sat on in all four facings, standing free and at a table, counter or high café table it faces; a snack eaten
  from each.
- The three stools swivel for sit and drink with the sitter riding them (`ride: swivel`) and turn back after their
  `hold` (0.6 s).
- Coffee and a drink on the stools need a table (`needsTable`): refused for a seat standing alone. Coffee also
  needs a working coffee machine on the lot (`requires` / `provides`), as at any table.
- Pulled in: once seated at a table it faces (`seatAt`, sim-core `faced_table`), the sitter shuffles in with the seat
  until the top's near edge is a fixed reach ahead of the hips (scaled to the body's height), and pushes it back
  while rising. The renderer moves the seat's instance, its contact shadow and a stool's swivel part, up to 0.45 m,
  so the seat overlaps into the table's tile; the sim's positions are unchanged. Plain "sit" pulls it in too.
- How far it comes in is read from the table as drawn (`tableTuck.ts`): the knees and toes stop short of legs,
  pedestals, a counter's cabinet or a trestle's foot, and go no further than the top's middle.
- Seen, not changed: on getting up the sim puts the resident on the tile in front at once, so `standUp` plays while
  they glide back, the seat sliding back over the same half second.

## K9. Dining storage (4)

- **Family:** `tools/art/blender/families/dining.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sideboard` Walnut sideboard | 2x1 · floor | — | kitchen / cabinets | Body, Top, Handles | Use | `door_open`: the second fluted door swings open 100° in 0.9 s onto shelved plates and bowls while clean things are put away (a `cupboard` station, `fetch`, `hold` 1.5 s). |
| `buffet` Painted buffet | 2x1 · floor | — | kitchen / cabinets | Body, Top, Handles | Use | `door_open`: the wide middle door swings open, as the sideboard's. |
| `chinaCabinet` China cabinet | 1x1 · floor | — | kitchen / cabinets | Body, Top, Handles | Use | Glass doors. `door_open`: the left glazed door swings open onto its plates, as the sideboard's. |
| `barCart` Bar cart | 1x1 · floor | mix:drink | kitchen / cabinets | Frame, Trays | Use | `shake`: the cocktail shaker lifts 10 cm and tips 30° for mix. |

**In play** (signed off 2026-10-16):
- The sideboard's, buffet's and china cabinet's door opens when clean dishes are put away in them (a snack's plate
  cleared: bin, sink, then the cupboard), in all four facings, three against a wall and one standing free. One door
  each, since each opening door costs draws of its own.
- The bar cart's shaker lifts and tips towards the resident for "Mix a drink" and shuts after its `hold` (1.5 s).
- The china cabinet's hutch back is the body's colour outside (blue only inside), so it reads standing free or with
  the wall cut away.
- Not right yet: at the china cabinet the resident reaches at the waist while the glazed door above opens (a reach-up
  clip still to make; since the authenticity review, G-15, at least no squat to the floor).
