# Community and farm

Town things (`tools/art/community.py`). They are placed on town lots and seen from far and by the hundred: LOD (E7)
before the street furniture. Check them on a town lot (`?state=` from a test dump) as well as in Validate.

## CM1. Playground (8) — **Done** (in Validate › Playground)

- **Family:** `tools/art/blender/families/playground.py`
- **Validate group:** `playground` (Validate › Playground)
- **Placeholders to match:** `tools/art/community.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `slide` Playground slide | 1x2 · floor | slide:play | community / play | Frame, Seats | Static | Static. |
| `climbingFrame` Climbing frame | 2x2 · floor | climb:exercise | community / play | Frame, Seats | Static | Static. |
| `seesaw` Seesaw | 3x1 · floor | seesaw:play | community / play | Frame, Seats | Loop (E1) | `seesaw`: the plank tips ±12° while two play (E1). Until then U: tips to one side and holds. |
| `springRider` Spring rider | 1x1 · floor | ride:play | community / play | Frame, Seats | Loop (E1) | `rock` on its spring (E1). Until then U: leans forward 10°. |
| `merryGoRound` Merry-go-round | 2x2 · floor | spin:play | community / play | Frame, Seats | Loop (E1) | `spin` while ridden (E1). Until then static. |
| `basketballHoop` Basketball hoop | 1x1 · floor | shoot:exercise | community / play | Frame, Seats | Use | `swish`: the net sways after a shot (`hold` 0.6 s). |
| `pingPongTable` Ping-pong table | 2x1 · floor | pingPong:play | community / play | Frame, Seats | Static | Static. |
| `parkChessTable` Park chess table | 1x1 · floor | chess:play | community / play | Frame, Seats | Use | `move`: a pawn slides one square for chess. |

## CM2. Street furniture (10)

- **Family:** `tools/art/blender/families/community.py`
- **Validate group:** `street` (Validate › Street)
- **Placeholders to match:** `tools/art/community.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `streetLamp` Street lamp | 1x1 · floor | — | community / street | Body | Static | Static. N glow (E8). LOD (E7). |
| `parkBench` Long park bench | 3x1 · floor | sit:sit feedBirds:sit | community / street | Body | Static | Static. |
| `drinkingFountain` Drinking fountain | 1x1 · floor | drink:drink | community / street | Body | Use | `water_on`: the water arc grows for drink. |
| `litterBin` Litter bin | 1x1 · floor | — | community / street | Body | Static | Static. |
| `busStop` Bus shelter | 3x1 · floor | wait:sit timetable:read | community / street | Body | Static | Static. |
| `postBox` Post box | 1x1 · floor | post:write | community / street | Body | Use | `flap`: the slot flap swings in for post (`hold` 1 s). |
| `noticeBoard` Notice board | 1x1 · floor | read:read | community / street | Body | Static | Static. |
| `phoneBooth` Phone box | 1x1 · floor | call:phone | community / street | Body | Use | `door_open`: the folding door opens and shuts behind them (`hold` 1.5 s) for call. |
| `fountainTown` Town fountain | 3x3 · floor | wish:look sitBy:relax | community / street | Body | Ambient (E2) | the water runs (E2). Until then static jets, as the garden fountain. |
| `townStatue` Founder's statue | 2x2 · floor | admire:admire | community / street | Body | Static | Static. |

## CM3. Food stalls and vending (5)

- **Family:** `tools/art/blender/families/community.py`
- **Validate group:** `stalls` (Validate › Stalls)
- **Placeholders to match:** `tools/art/community.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `iceCreamCart` Ice cream cart | 2x1 · floor | iceCream:eat | community / food | Body, Awning | Use | `lid_open`: the freezer lid slides open for iceCream; `hold` 2 s. |
| `hotDogStand` Hot dog stand | 2x1 · floor | hotDog:eat | community / food | Body, Awning | Use | `lid_open`: the hot-plate lid lifts for hotDog. |
| `vendingSnack` Snack machine | 1x1 · floor | snack:eat | community / food | Body, Awning | Use | `drop`: a snack slides down to the tray for snack. |
| `vendingDrink` Drinks machine | 1x1 · floor | drink:drink | community / food | Body, Awning | Use | `drop`: a can drops to the tray for drink. |
| `marketStall` Market stall | 2x1 · floor | browse:look fruit:eat | community / food | Body, Awning | Static | Static. |

## CM4. Shops and café (5)

- **Family:** `tools/art/blender/families/community.py`
- **Validate group:** `shops` (Validate › Shops)
- **Placeholders to match:** `tools/art/community.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `shopCounter` Shop counter | 2x1 · floor | treat:idle | community / shops | Counter, Trim | Use | `till`: the cash drawer slides out for treat (`hold` 1.5 s). |
| `bakeryCase` Bakery case | 2x1 · floor | pastry:eat | community / shops | Counter, Trim | Use | `door`: the back glass door slides aside for pastry. |
| `groceryShelf` Grocery shelves | 2x1 · floor | browse:look sandwich:eat | community / shops | Counter, Trim | Static | Static. |
| `cafeTable` Café table | 1x1 · floor | — | community / shops | Counter, Trim | Static | Static. |
| `cafeChair` Café chair | 1x1 · floor | sit:sit coffee:drink cake:eat | community / shops | Counter, Trim | Static | Static. |

## CM5. Farm (8) — **Done** (in Validate › Farm)

- **Family:** `tools/art/blender/families/farm.py`
- **Validate group:** `farm` (Validate › Farm)
- **Placeholders to match:** `tools/art/community.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `farm.barn` Red barn | 6x6 · floor | — | community / farm | Paint, Roof | Static | Static (6×6: LOD first, E7). |
| `farm.silo` Grain silo | 3x3 · floor | — | community / farm | Paint, Roof | Static | Static. |
| `farm.tractor` Little tractor | 2x4 · floor | — | community / farm | Paint, Roof | Static | Static. |
| `farm.hayBale` Round hay bale | 1x1 · floor | — | community / farm | Paint, Roof | Static | Static. |
| `farm.hayStack` Stack of hay bales | 2x1 · floor | — | community / farm | Paint, Roof | Static | Static. |
| `farm.scarecrow` Scarecrow | 1x1 · floor | — | community / farm | Paint, Roof | Ambient (E2) | a slow sway (E2). Until then static. |
| `farm.trough` Water trough | 2x1 · floor | — | community / farm | Paint, Roof | Static | Static. |
| `farm.coop` Chicken coop | 2x2 · floor | — | community / farm | Paint, Roof | Static | Static. (+use) the hatch opens. |
