# Community and farm

Town things, built in Blender. Five
families: `families/playground.py` (notes: `playground.md`), `families/street_furniture.py` (`street_furniture.md`),
`families/stalls.py` (`stalls.md`), `families/shops.py` (`shops.md`) and `families/farm.py` (`farm.md`). The street's
dressing props (`model.street.*`, `families/street.py`) aren't buyable and aren't listed here.

They stand on town lots (the park lots, `houses.json` `parks`; the farm things on farm lots, placed by id in
`web/src/game/town.ts` `farmstead`). They are seen from far and by the hundred, so
every model has a far version (E7): the model at rest, joined and decimated (`far=0.25`), or built by hand where that
looked bad (the chess table, the barn, the silo, the tractor). Each is built to the content's seat and top
heights. Check them on a town lot (the `town-things` lot) as well as in a house.

Moving parts: the seesaw, spring rider and merry-go-round loop while ridden (E1); the town fountain's water runs and
the scarecrow sways (ambient, E2, off on Low); the coop's hatch shuts at night (E3); the rest open, drop or slide for
their uses. The climbing frame has no clip of its own: the resident's clips climb round it.

**Who uses them** (content `stages`, 2026-10-21, G-45): the spring rider, the slide and the climbing frame are a
child's; the seesaw and the merry-go-round a child's or a teen's. Grown-ups and elders are refused ("Ada is too big
for the spring rider"), never pick them, and see them greyed out. The climbing frame is tagged `toys` like the other
rides.

## CM1. Playground (8)

- **Family:** `tools/art/blender/families/playground.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `slide` Playground slide | 1x2 · floor | slide:play | community / play | Frame, Slide, Roof | Static | Static. |
| `climbingFrame` Climbing frame | 2x2 · floor | climb:exercise | community / play | Frame, Roof, Net | Static | Static. Climbed round: up its rope net, across the deck (ducking under the roof) and down the fireman's pole, then back round to the net (`climb172a`/`147a`/`112a` by height, `b` for the second lane: two side by side, the second waiting on the deck for the first to slide; one who steps in while someone goes round waits at their lane until that round is back at its start, so the two stay in step, G-25). |
| `seesaw` Seesaw | 3x1 · floor | seesaw:play | community / play | Frame, Seats, Plank | Loop (E1) | `seesaw`: the plank tips ±12° while two play (E1). The two ride at the two ends, facing each other over the handles (content `places`). |
| `springRider` Spring rider | 1x1 · floor | ride:play | community / play | Horse, Saddle, Spring | Loop (E1) | `rock` on its spring while ridden (E1). |
| `merryGoRound` Merry-go-round | 2x2 · floor | spin:play | community / play | Platform, Rails, Stripes | Loop (E1) | `spin` while ridden (E1). Four sit round the platform between the rails, facing the column (content `places`). |
| `basketballHoop` Basketball hoop | 1x1 · floor | shoot:exercise | community / play | Frame, Backboard, Hoop | Use | `swish`: the net sways after each shot (1.4 s into the use, then every 3.2 s: `delay`, `every`). |
| `pingPongTable` Ping-pong table | 2x1 · floor | pingPong:play | community / play | Table, Legs | Static | Static. The two players stand at the two ends, facing each other across the net (content `places`, off its tiles; a blocked end is played from the front). |
| `parkChessTable` Park chess table | 1x1 · floor | chess:play | community / play | Table, Stools | Use | `move`: a pawn slides one square for chess. A stool either side, the players facing each other across the board (content `places`; rebuilt 2026-10-21, the stools were both behind it). |

**In play** (signed off 2026-10-16):
- Checked in the yard of the check house in all four facings: the slide gone down, the ping-pong table played at;
  table tennis is a rally of forehands and backhands (`pingPong`, G-26), the bat mimed in a closed hand (props in hand:
  G-70; where the players stand: G-62);
  the seesaw, spring rider and merry-go-round loops go round with the rider on them and finish their round when the
  use ends; the hoop's net swishes after each shot; a pawn slides for chess. Every part coloured in buy and in the
  colour studio.
- The climbing frame is climbed round in 13 s: up the net three ropes (hands and feet in turn), over the edge onto
  the deck, a look round, across the deck under the roof, both hands on the pole, round it out over the edge, hung
  from the hands with the legs round it, down it, a landing with bent knees, and back round, clear of the net, to
  step on again. Every round includes the pole.
- Under the roof the climber bends and crouches as far as the body's height needs (solved while baking, so the top
  of the head stays 1 m over the deck): grown-ups cross in a deep crouch (the roof leaves about 1.1 m), children walk
  nearly upright.
- One clip per lane, since the walk back differs (`STEP_IN` `perLane` in `characters.ts`; clips in
  `characters/clips/street.ts`). The second lane's climber waits on the deck for the first to slide, so two who
  start together don't meet at the pole. Checked for a man, a woman and a child, two children at once, by day and
  on Low WebGL2 at night.
- Since the authenticity review (G-04): "Shoot some hoops" picks the ball up and reaches with it (the streetball
  hoop's `pickUp`, `interact`), not jogging on the spot; a shot from out on the court is a clip still to make.

## CM2. Street furniture (10)

- **Family:** `tools/art/blender/families/street_furniture.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `streetLamp` Street lamp | 1x1 · floor | — | community / street | Lamp post | Static | Static. The lantern lights at night (E8); far version (E7). |
| `parkBench` Long park bench | 3x1 · floor | sit:sit feedBirds:sit | community / street | Frame | Static | Static. |
| `drinkingFountain` Drinking fountain | 1x1 · floor | drink:drink | community / street | Pillar | Use | `water_on`: the water arc grows for drink; the drinker steps up and bends over the spout (`fountainDrink<cm>` by height). |
| `litterBin` Litter bin | 1x1 · floor | — | community / street | Bin | Static | Static. |
| `busStop` Bus shelter | 3x1 · floor | wait:sit timetable:read | community / street | Frame | Static | Static. |
| `postBox` Post box | 1x1 · floor | post:write | community / street | Box | Use | `flap`: the slot flap swings in for post (`hold` 1 s). |
| `noticeBoard` Notice board | 1x1 · floor | read:read | community / street | Frame | Static | Static. |
| `phoneBooth` Phone box | 1x1 · floor | call:phone | community / street | Kiosk | Use | `door_open`: the door opens out (`delay` 0.3 s), the caller steps inside and it shuts behind them (`hold` 2.4 s); it opens again to let them out (`out` 1.6 s). |
| `fountainTown` Town fountain | 3x3 · floor | wish:look sitBy:relax | community / street | Stone | Ambient (E2) | `water_run`: the water runs all the time, ambient (not on Low). Its text says square, as built. |
| `townStatue` Founder's statue | 2x2 · floor | admire:admire | community / street | Statue | Static | Static. |

**In play** (signed off 2026-10-16):
- Checked outdoors (the notice board indoors) and on the `town-things` lot: sat on (bench, shelter), pigeons fed,
  the timetable read, the post box's flap, the statue admired, a wish made at the town fountain and its rim sat on.
  The town fountain's water runs on WebGPU Medium and is off on Low, as designed. The street lamp lights at night
  (WebGPU and Low WebGL).
- Drinking fountain: the drinker steps up and bends over the bowl, hands on the rim, mouth at the arc
  (`fountainDrink<cm>` in `characters/clips/street.ts`, one per height).
- Phone box: the door opens outwards, the caller steps inside onto the plinth, it shuts behind them, and it opens
  again to let them out (motions `out`: a use's clip opens again for N s at its end).
- Since the authenticity review (G-15): the notice board is read looking at it (`idle`), not at a book in the hands;
  the timetable reader steps into the shelter to its end (`STEP_IN`); the statue is admired from a step further back,
  looking up at it.

## CM3. Food stalls and vending (5)

- **Family:** `tools/art/blender/families/stalls.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `iceCreamCart` Ice cream cart | 2x1 · floor | iceCream:fetch | community / food | Body, Parasol, Bands | Use | `lid_open`: the freezer lid slides open for iceCream (*Take an ice cream from the freezer*); `hold` 2 s. Nobody sells: it is self-serve (G-67), fetched and paid for here, then eaten sitting on a bench or at a café table nearby (else standing). |
| `hotDogStand` Hot dog stand | 2x1 · floor | hotDog:fetch | community / food | Body, Awning | Use | `lid_open`: the hot-plate lid lifts for hotDog (*Help yourself to a hot dog*). Nobody sells: it is self-serve (G-67), fetched and paid for here, then eaten sitting on a bench or at a café table nearby (else standing). |
| `vendingSnack` Snack machine | 1x1 · floor | snack:fetch | community / food | Body | Use | `drop`: a snack slides down to the tray for snack, then eaten sitting on a bench or at a table nearby (G-67). |
| `vendingDrink` Drinks machine | 1x1 · floor | drink:drink | community / food | Body | Use | `drop`: a can drops to the tray for drink. |
| `marketStall` Market stall | 2x1 · floor | browse:look fruit:fetch | community / food | Body, Awning | Static | Static. *Pick a piece of fruit* (was "a bag of fruit"), paid into an honesty box. Nobody sells: it is self-serve (G-67), fetched and paid for here, then eaten sitting on a bench or at a café table nearby (else standing). |

**In play** (signed off 2026-10-16):
- Checked with the carts and market stall outdoors, the machines indoors: the ice cream cart's lid for an ice cream
  (held 2 s), the hot dog stand's lid, a snack and a can dropping in the machines, the market stall browsed and
  fruit bought.
- Each clip plays its full length and shuts in every run. Nothing needed fixing.

## CM4. Shops and café (5)

- **Family:** `tools/art/blender/families/shops.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `shopCounter` Shop counter | 2x1 · floor | treat:idle | community / shops | Counter, Trim | Use | `till`: the cash drawer slides out for treat (`hold` 1.5 s): *Pick a little something and pay at the till*, a self-service till (G-67: nobody serves). |
| `bakeryCase` Bakery case | 2x1 · floor | pastry:fetch takeCake:fetch | community / shops | Counter, Trim | Use | `door`: the back glass door slides aside for pastry (`hold` 2 s; the door is still the baker's side's). Provides `cake`: the café chair's slice of cake is taken from it first (`takeCake`, not chosen on its own). Nobody sells: it is self-serve (G-67), fetched and paid for here, then eaten sitting on a bench or at a café table nearby (else standing). |
| `groceryShelf` Grocery shelves | 2x1 · floor | browse:look sandwich:fetch | community / shops | Shelves, Trim | Static | Static. Nobody sells: it is self-serve (G-67), fetched and paid for here, then eaten sitting on a bench or at a café table nearby (else standing). |
| `cafeTable` Café table | 1x1 · floor | — | community / shops | Top, Frame | Static | Static. |
| `cafeChair` Café chair | 1x1 · floor | sit:sit coffee:drink cake:eat | community / shops | Frame, Seat | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. The coffee is poured at the lot's coffee machine first and the cake taken from its bakery case (content `fetch`, G-67); a cake needs a bakery case on the lot. |

**In play** (signed off 2026-10-16):
- Checked indoors: the till drawer (held 1.5 s) and the bakery case's back door (held 2 s) for their uses, the
  grocery shelves browsed and a sandwich taken.
- The café chair is a seat at a table (`seatAt: ["table"]`): coffee and cake at the café table in two placements,
  refused at a chair standing alone (`needsTable`). Its coffee also needs a working coffee machine on the lot
  (`requires` / `provides`): the Corner Café has one, but a café chair bought for a home does not serve coffee
  without one. Once seated, the sitter shuffles in with the chair until the
  top's near edge is a fixed reach ahead of the hips (up to 0.45 m, read from the table's triangles in
  `tableTuck.ts`), and pushes it back while rising; the sim's positions are unchanged.
- The shop counter's "Counter" colours its front panels, "Trim" the walnut top and frame.

## CM5. Farm (8)

- **Family:** `tools/art/blender/families/farm.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `farm.barn` Red barn | 6x6 · floor | — | community / farm | Paint, Roof, Trim | Static | Static; its far version built by hand (E7). |
| `farm.silo` Grain silo | 3x3 · floor | — | community / farm | Paint, Roof | Static | Static. |
| `farm.tractor` Little tractor | 2x4 · floor | — | community / farm | Paint, Wheels | Static | Static. |
| `farm.hayBale` Round hay bale | 1x1 · floor | — | community / farm | Twine | Static | Static. |
| `farm.hayStack` Stack of hay bales | 2x1 · floor | sit:sit | community / farm | Twine | Static | Static. Sit on the hay bales, two at once on the lower bales' front edges (content `seat` 0.45 m, forward 0.25; 2026-10-21, Q-19). |
| `farm.scarecrow` Scarecrow | 1x1 · floor | — | community / farm | Shirt, Trousers, Hat | Ambient (E2) | `sway`: a slow sway, ambient (not on Low). |
| `farm.trough` Water trough | 2x1 · floor | — | community / farm | Paint | Static | Static. |
| `farm.coop` Chicken coop | 2x2 · floor | — | community / farm | Paint, Roof, Trim | Night (E3) | `close`: the hatch slides shut at night (held from dusk) and is open by day. Decor by decision (Q-19: kind `chickenCoop` marked decor); hens and eggs would be new behaviour. |

**In play** (signed off 2026-10-16):
- All looked at in four facings by day, at night and on Low WebGL, and together on the `town-things` lot, near and
  far.
- The scarecrow sways (off on Low, as designed); the coop's hatch is shut at night and open by day.
- Nothing needed fixing.
