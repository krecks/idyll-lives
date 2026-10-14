# Kitchen

Appliances, counters, islands, dining. The fridge, the five K1 fridges and the two kitchen sinks are done
(`families/kitchen.py`), the six K2 cookers and the four K3 small appliances and dishwashers are done
(`families/kitchen_cooking.py`), and the eight K4 base counters, the eleven K5 tall and wall storage
things and the five K6 islands and bars are done (`families/kitchen_counters.py`). The thirteen K7 dining
tables, the twelve K8 chairs and stools and the four K9 dining storage things are done (`families/dining.py`),
so the whole kitchen is built.
Doors and drawers follow the fridge: hinged parts built closed, `G.swing` / `G.slide`, a `hold` so they shut
while the resident is still busy. Counter and table tops must keep their heights (`tools/art/catalog2.py` `TOPS`);
chairs and stools their seat height (`SEATS` in `web/src/render/babylon/characters.ts`).

## K1. Fridges (5) — **Done** (in Validate › Fridges)

- **Family:** `tools/art/blender/families/kitchen.py`
- **Validate group:** `fridges` (Validate › Fridges)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `fridgeMini` Mini fridge | 1x1 · floor | snack:fetch meal:cook | kitchen / appliances | Body, Handles, Plinth | Use | `fridge_open`: the door swings 100° on its hinge for snack and meal; `hold` 3 s, as the fridge. |
| `fridgeBuiltIn` Built-in fridge | 1x1 · floor | snack:fetch meal:cook gourmet:cook coldDrink:drink | kitchen / appliances | Panels, Handles | Use | `fridge_open`: the tall fridge door swings for snack, meal, gourmet and coldDrink; the freezer door below gets `freezer_open`, unused until a use takes frozen food. |
| `fridgeFrenchDoor` French-door fridge | 1x1 · floor | snack:fetch meal:cook gourmet:cook coldDrink:drink | kitchen / appliances | Body, Handles, Plinth | Use | `fridge_open`: both upper doors swing out (-100° / +100°) for snack, meal, gourmet; `door_right` (the right door only) for coldDrink. The freezer drawer `freezer_open` slides out 35 cm, unused for now. |
| `fridgeRetro` Retro fridge | 1x1 · floor | snack:fetch meal:cook coldDrink:drink | kitchen / appliances | Body, Handle, Trim | Use | `lever`: the chrome lever (a part inside the door) turns 30° out, then `fridge_open` (`delay` 0.3 s): the rounded door swings 100°; snack, meal, coldDrink. |
| `wineFridge` Wine fridge | 1x1 · floor | wine:drink | kitchen / appliances | Body, Door frame | Use | `door_open`: the glass door swings and the top rack slides out 15 cm for wine; `hold` 2.5 s. |

## K2. Cookers (6) — **Done** (in Validate › Cookers)

- **Family:** `tools/art/blender/families/kitchen_cooking.py`
- **Validate group:** `cookers` (Validate › Cookers)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `stoveElectric` Electric cooker | 1x1 · floor | cook:cook bake:cook | kitchen / appliances | Body, Knobs, Hob | Use | `oven_open`: the oven door drops 85° on its bottom hinge for bake; `knob_on`: the front-left knob turns 90° for cook. |
| `stoveGas` Gas range | 1x1 · floor | cook:cook gourmet:cook bake:cook | kitchen / appliances | Body, Knobs, Hob | Use | `oven_open` for bake (door drops); `burner_on` for cook and gourmet: a knob turns and a ring of blue flame (gloss, not emissive) grows on the front burner, the `water_on` way. |
| `wallOven` Oven tower | 1x1 · floor | roast:cook bake:cook | kitchen / appliances | Tower, Handles | Use | `oven_open`: the upper oven door drops and its tray slides out 10 cm for roast and bake. |
| `rangeCooker` Range cooker | 2x1 · floor | cook:cook gourmet:cook bake:cook | kitchen / appliances | Body, Knobs & rail, Hob | Use | `oven_open`: the left oven door drops for bake; `burner_on` (knob and flame ring) for cook and gourmet. |
| `cooktopCounter` Hob counter | 1x1 · floor | cook:cook | kitchen / appliances | Cabinet, Worktop, Hob | Use | `burner_on`: a knob turns and the flame ring grows for cook. |
| `microwave` Microwave counter | 1x1 · floor | reheat:cook | kitchen / appliances | Cabinet, Worktop, Microwave | Use | `door_open`: the microwave door swings open and shuts again (`hold` 1.2 s) for reheat. L `turntable`: the glass plate turns while it heats (E1). |

## K3. Small appliances and dishwashers (4) — **Done** (in Validate › Appliances)

- **Family:** `tools/art/blender/families/kitchen_cooking.py`
- **Validate group:** `appliances` (Validate › Appliances)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `coffeeMachine` Coffee machine | 1x1 · floor | espresso:drink | kitchen / appliances | Cabinet, Worktop, Machine | Use | `brew`: a cup slides under the spout and a thin coffee stream grows for espresso. |
| `breakfastCounter` Breakfast counter | 1x1 · floor | toast:cook smoothie:cook | kitchen / appliances | Counter, Toaster, Blender | Use | `toast_down`: the toaster lever slides down for toast, and the bread pops up at the end. `blend` for smoothie: the blender lid lifts; the blades spinning needs E1. |
| `dishwasher` Dishwasher | 1x1 · floor | dishes:clean | kitchen / appliances | Body, Handle, Panel | Use | `door_open`: the door drops 85° and the lower rack slides out 30 cm for dishes; `hold` 2.5 s (loading), then it closes. |
| `dishwasherCompact` Stainless dishwasher | 1x1 · floor | dishes:clean | kitchen / appliances | Body, Handle, Panel | Use | `door_open`, as the dishwasher. |

## K4. Base counters (8) — **Done** (in Validate › Counters)

- **Family:** `tools/art/blender/families/kitchen_counters.py`
- **Validate group:** `counters` (Validate › Counters)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `counter` Kitchen counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Worktop, Handles | Use | `drawer_open`: the top drawer slides out 25 cm for sandwich; `hold` 1.5 s. |
| `counterCorner` Corner counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Worktop, Handles | Use | `drawer_open`, as the counter. |
| `counterEnd` Counter end with shelves | 1x1 · floor | — | kitchen / counters | Cabinet, Worktop, Handles | Static | Static (open end shelves). |
| `counterCornerOuter` Outer corner counter | 1x1 · floor | — | kitchen / counters | Cabinet, Worktop, Handles | Static | Static. |
| `counterDrawers` Drawer counter | 1x1 · floor | — | kitchen / counters | Cabinet, Worktop, Handles | Static | Static. (+use) the three drawers slide out in turn. |
| `counterOpenShelf` Open-shelf counter | 1x1 · floor | — | kitchen / counters | Cabinet, Worktop | Static | Static. |
| `butcherBlock` Butcher block | 1x1 · floor | sandwich:cook | kitchen / counters | Base, Block top | Use | `drawer_open`: the knife drawer slides out for sandwich. |
| `counterMarble` Marble counter | 1x1 · floor | sandwich:cook | kitchen / counters | Cabinet, Marble top, Handles | Use | `drawer_open` for sandwich. |

## K5. Tall and wall storage (11) — **Done** (in Validate › Kitchen Storage)

- **Family:** `tools/art/blender/families/kitchen_counters.py`
- **Validate group:** `kitchen storage` (Validate › Kitchen Storage)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `pantry` Pantry cupboard | 1x1 · floor | snack:fetch | kitchen / counters | Cabinet, Handles | Use | `doors_open`: both doors swing out 100° for snack; `hold` 2.5 s. |
| `pantryTall` Tall larder | 1x1 · floor | snack:fetch | kitchen / counters | Cabinet, Handles | Use | `doors_open`, as the pantry. |
| `wallCabinet` Wall cabinets | 1x1 · wall | — | kitchen / counters | Cabinet, Handles | Static | Static. (+use) the doors swing up. |
| `wallCabinetGlass` Glazed wall cabinet | 1x1 · wall | — | kitchen / counters | Cabinet, Handles | Static | Static; glass doors (`glass` finish). |
| `wallCabinetCorner` Wall cabinet with end shelves | 1x1 · wall | — | kitchen / counters | Cabinet, Handles | Static | Static. |
| `rangeHood` Cooker hood | 1x1 · wall | — | kitchen / counters | Hood, Trim | Static | Static. |
| `spiceRack` Spice rack | 1x1 · wall | — | kitchen / counters | Rack | Static | Static. |
| `potRail` Pot rail | 1x1 · wall | — | kitchen / counters | Rail, Pots | Static | Static. |
| `kitchenShelfWall` Kitchen wall shelves | 1x1 · wall | — | kitchen / counters | Shelves, Brackets | Static | Static. |
| `trashCan` Pedal bin | 1x1 · floor | — | kitchen / counters | Bin, Lid | Static | Static. (+use) `lid_open`: the pedal lid lifts 70° (a "throw away" use). |
| `recyclingBins` Recycling bins | 1x1 · floor | — | kitchen / counters | Left bin, Middle bin, Right bin (each with its lid), Handles | Static | Static. (+use) the lids lift. |

## K6. Islands and bars (5) — **Done** (in Validate › Islands & Bars)

- **Family:** `tools/art/blender/families/kitchen_counters.py`
- **Validate group:** `islands` (Validate › Islands & Bars)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kitchenIsland` Kitchen island | 2x1 · floor | sandwich:cook breakfast:eat | kitchen / counters | Cabinet, Worktop | Use | `drawer_open`: a drawer slides out for sandwich (not for breakfast: residents sit at it). |
| `islandSmall` Small island | 1x1 · floor | — | kitchen / islands | Cabinet, Worktop | Static | Static. |
| `islandLong` Long island | 3x1 · floor | — | kitchen / islands | Cabinet, Worktop | Static | Static. |
| `barSegment` Bar counter | 1x1 · floor | — | kitchen / islands | Cabinet, Worktop | Static | Static. |
| `barEnd` Bar end | 1x1 · floor | — | kitchen / islands | Cabinet, Worktop | Static | Static. |

## K7. Dining tables (13) — **Done** (in Validate › Tables)

- **Family:** `tools/art/blender/families/dining.py`
- **Validate group:** `tables` (Validate › Tables)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `diningTable` Dining table | 2x1 · floor | coffee:drink | kitchen / dining | Top, Legs | Static | Static. |
| `bistroSet` Bistro table | 1x1 · floor | breakfast:eat coffee:drink | kitchen / dining | Top, Frame, Seats | Static | Static. |
| `diningGlass` Glass dining table | 2x1 · floor | breakfast:eat coffee:drink | kitchen / dining | Top, Legs | Static | Static; glass top. |
| `diningOak` Farmhouse dining table | 2x1 · floor | breakfast:eat coffee:drink dinner:eat | kitchen / dining | Top, Legs | Static | Static. |
| `tableSquare2` Square table for two | 1x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableBistroRound` Round bistro table | 1x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableDining4` Dining table for four | 2x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableDining6` Dining table for six | 3x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableBanquet8` Banquet table | 4x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableRound4` Round pedestal table | 2x2 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableSquare8` Square table for eight | 2x2 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableFarmhouse6` Farmhouse table | 3x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |
| `tableBarHigh` High café table | 1x1 · floor | — | kitchen / dining | Top, Legs | Static | Static. |

## K8. Chairs and stools (12) — **Done** (in Validate › Chairs)

- **Family:** `tools/art/blender/families/dining.py`
- **Validate group:** `chairs` (Validate › Chairs)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `barStool` Bar stool | 1x1 · floor | sit:sit drink:drink | kitchen / dining | Frame, Seat | Use | `swivel`: the seat turns 20° toward the resident and back, for sit and drink. |
| `stoolCounter` Counter stool | 1x1 · floor | sit:sit drink:drink | kitchen / dining | Frame, Seat | Use | `swivel`, as the bar stool. |
| `stoolBarHigh` High bar stool | 1x1 · floor | sit:sit drink:drink | kitchen / dining | Frame, Seat | Use | `swivel`, as the bar stool. |
| `diningChair` Dining chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static: the sitting pose is fitted to the seat, so the chair stays put. |
| `chairWood` Wooden chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. |
| `chairUpholstered` Upholstered chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Upholstery | Static | Static. |
| `chairModern` Moulded chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. |
| `chairWindsor` Windsor chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. |
| `chairBentwood` Bentwood chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. |
| `chairFolding` Folding chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. |
| `chairArm` Carver chair | 1x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. |
| `benchDining` Dining bench | 2x1 · floor | sit:sit coffee:drink | kitchen / dining | Frame, Seat | Static | Static. |

## K9. Dining storage (4) — **Done** (in Validate › Dining Storage)

- **Family:** `tools/art/blender/families/dining.py`
- **Validate group:** `dining storage` (Validate › Dining Storage)
- **Placeholders to match:** `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sideboard` Walnut sideboard | 2x1 · floor | — | kitchen / cabinets | Body, Top, Handles | Static | Static. (+use) the doors swing. |
| `buffet` Painted buffet | 2x1 · floor | — | kitchen / cabinets | Body, Top, Handles | Static | Static. (+use) the doors swing. |
| `chinaCabinet` China cabinet | 1x1 · floor | — | kitchen / cabinets | Body, Top, Handles | Static | Static; glass doors. (+use) a door swings. |
| `barCart` Bar cart | 1x1 · floor | mix:drink | kitchen / cabinets | Frame, Trays | Use | `shake`: the cocktail shaker lifts 10 cm and tips 30° for mix. |
