# Tabletop

Small things that stand on tops (`surface` layer, 1×1). All 39 are built by `tools/art/blender/families/tabletop.py`
(notes: `families/tabletop.md`). The origin is at the base,
centred, and the renderer lifts each to its host's top. Each is no wider than the `radius` the content gives it
(the disc it takes on a top; the gate warns of one reaching over 10 % past it), so a thing still fits where the game puts it;
the four widest (`plateSet`, `teapotSet`, `espressoMachineSmall`, `monitor`, radius 0.23–0.3 m) don't fit on a
counter's top with its edge margin and go on tables and desks.

Twenty of them have uses (G-63): the appliances and screens, food and drink, books, small plants and toys. A thing
on a top is used from the floor beside its top (`surface::rider_tiles`: in front of the counter, desk or table it
stands on, else at the top's side), standing and facing it; a use with `"pose": "sit"` (the laptop, monitor, books,
tea set, mugs, wine, blocks) takes a chair pulled up to that top if one is free, else a seat within `seatNear`, else
it is done standing; at a top with chairs all round, any use is done from one of its chairs. The other nineteen are decor (the chopping board, the alarm clock, the globe and the
ornaments, towels, soap and the like). Food, drink, books, plants and toys don't wear (`wearPerUse: 0`); the
appliances and screens do. No use has a clip yet: one draw call where it can (wood `satin`, towels and the teddy
`matte`), a second only for glass that is seen through (the cookie jar, the wine glasses, the snow globe), and the
smallest triangle budget (`docs/design/objects.md`: 0.35 m, 1.5k/3k; built at 226–1,496). The player colours the
main part, labelled `Body` (the cookie jar: its `Lid`); sets of like things (mugs, books, towels, blocks) list each
one. The kettle's lid, the bread bin's roll-top and the laptop's lid are parts of their own, hinged where they would
turn, for a clip later (the laptop is used closed until then). The alarm clock is the one thing that moves: its
hands tell the game's time.

Each was checked in play bought on a square table (`tableSquare2`, top at 0.75 m) standing free in the living room,
looked at close up from its front three-quarter and level from its side: in all four facings by day, every paint
slot a distinct colour by day and at night (lamps lit), at night on Low with WebGL2, and in the colour studio. Each
sits on the top, turns with its facing and lights right at night.

## TT1. Kitchenware (11)

- **Family:** `tools/art/blender/families/tabletop.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kettle` Electric kettle | 1x1 · surface | tea:drink | tabletop / kitchenware | Body, "Base, handle & lid" | Static | Static. (+use) the lid lifts. |
| `toaster` Toaster | 1x1 · surface | toast:cook | tabletop / kitchenware | Body | Static | Static. |
| `blender` Blender | 1x1 · surface | smoothie:cook | tabletop / kitchenware | Body, Smoothie | Static | Static. |
| `standMixer` Stand mixer | 1x1 · surface | bake:cook | tabletop / kitchenware | Body | Static | Static. |
| `espressoMachineSmall` Espresso machine | 1x1 · surface | espresso:drink | tabletop / kitchenware | Body | Static | Static. |
| `knifeBlock` Knife block | 1x1 · surface | — | tabletop / kitchenware | Body, Handles | Static | Static. |
| `utensilCrock` Utensil pot | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `cuttingBoard` Chopping board | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `breadBin` Bread bin | 1x1 · surface | sandwich:chop | tabletop / kitchenware | Body, Ends & handle | Static | Static. (+use) the roll-top opens. |
| `spiceJars` Spice jars | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `cookieJar` Cookie jar | 1x1 · surface | biscuit:fetch | tabletop / kitchenware | Lid | Static | Static. |

**In play** (signed off 2026-10-16):
- The stand mixer and cookie jar, red by default, were also coloured purple to see the colour land.
- The bread bin's wooden ends are flat D shapes (the round end flat-shaded, the foot 2 mm thinner than the end so
  they don't share a plane and flicker).
- The kettle's second colour, "Base, handle & lid", colours all three.
- The knife block's tilted block rests on one edge of its foot, with a small gap at the other (stylised, left).

## TT2. Tableware (7)

- **Family:** `tools/art/blender/families/tabletop.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `fruitBowl` Fruit bowl | 1x1 · surface | fruit:fetch | tabletop / tableware | Body | Static | Static. |
| `plateSet` Place setting | 1x1 · surface | — | tabletop / tableware | Body, Rim, Napkin | Static | Static. |
| `cakeStand` Cake on a stand | 1x1 · surface | — | tabletop / tableware | Body, Icing | Static | Static. |
| `teapotSet` Tea set | 1x1 · surface | tea:drink | tabletop / tableware | Body, Tray | Static | Static. |
| `mugs` Mugs | 1x1 · surface | drink:drink | tabletop / tableware | Mug 1, Mug 2, Mug 3 | Static | Static. |
| `wineGlasses` Wine and glasses | 1x1 · surface | wine:drink | tabletop / tableware | Body | Static | Static. |
| `candlesticks` Candlesticks | 1x1 · surface | — | tabletop / tableware | Body, Candles | Night (E3) | `lit`: by day dark wicks; from dusk the two flames rise out of the candles' tops (held open dusk to dawn) and glow while the room's lights are on (E8; M-tabletop-01, 2026-10-21: they were drawn lit all day and dark at night). |

**In play** (signed off 2026-10-16):
- The place setting's Body, Rim and Napkin and the cake's Body and Icing land where named.
- The tea set's Body colours the cups as well as the pot (a set).
- Nothing needed fixing.

## TT3. Books and small plants (7)

- **Family:** `tools/art/blender/families/tabletop.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bookStack` Stack of books | 1x1 · surface | read:read | tabletop / books | Book 1–4 | Static | Static. |
| `booksUpright` Books and bookends | 1x1 · surface | read:read | tabletop / books | Bookends, Book 1–5 (each cover colour) | Static | Static. |
| `vaseFlowers` Vase of flowers | 1x1 · surface | — | tabletop / plants | Body | Static | Static. |
| `vaseSmall` Bud vase | 1x1 · surface | — | tabletop / plants | Body | Static | Static. |
| `potSucculent` Succulent | 1x1 · surface | water:water | tabletop / plants | Body | Static | Static. |
| `potCactus` Cactus | 1x1 · surface | water:water | tabletop / plants | Body | Static | Static. |
| `potSmall` Small potted plant | 1x1 · surface | water:water | tabletop / plants | Body | Static | Static. |

**In play** (signed off 2026-10-17):
- The stack of books and the five plants needed nothing.
- The books and bookends have eight books in five cover colours, listed as the stack of books lists its own:
  Bookends, then Book 1 to 5, left to right in the order each colour first shows; Book 1 to 3 colour two books each
  (manifest only, the GLB is the same). Rechecked in all four facings, every slot its own colour, at night on WebGPU
  and Low WebGL2, and in the colour studio, each label landing on its covers.

## TT4. Small ornaments (6)

- **Family:** `tools/art/blender/families/tabletop.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `photoFrame` Photo frame | 1x1 · surface | — | tabletop / ornaments | Body | Static | Static. |
| `tableClock` Alarm clock | 1x1 · surface | — | tabletop / ornaments | Body, Bells & legs | Clock (`motions.clock`) | `hour_hand`, `minute_hand`: the hands tell the game's time, on every preset; no second hand (its red hub stays). 0.11 × 0.16 × 0.05 m, a real twin-bell clock's size (M-tabletop-02, 2026-10-21: it was 0.15 × 0.23). |
| `globe` Globe | 1x1 · surface | — | tabletop / ornaments | Body, Stand | Static | Static. |
| `snowGlobe` Snow globe | 1x1 · surface | — | tabletop / ornaments | Body | Static | Static. |
| `trophy` Trophy | 1x1 · surface | — | tabletop / ornaments | Body | Static | Static. |
| `sculptureSmall` Small sculpture | 1x1 · surface | — | tabletop / ornaments | Body, Ball | Static | Static. |

**In play** (signed off 2026-10-16):
- The alarm clock's hands tell the game's time (ten o'clock at 10:01, 9:35 at night) and follow it when time runs,
  at top speed and paused. Three draws: the body and the two hands.
- Its second colour, "Bells & legs", colours the bells, legs and feet.

## TT5. Bath, desk and toys (8)

- **Family:** `tools/art/blender/families/tabletop.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `soapSet` Soap set | 1x1 · surface | — | tabletop / bath | Body, Dispenser | Static | Static. |
| `towelStack` Folded towels | 1x1 · surface | — | tabletop / bath | Towel 1, Towel 2, Towel 3 | Static | Static. |
| `perfumeTray` Perfume tray | 1x1 · surface | — | tabletop / bath | Body | Static | Static. |
| `deskOrganiser` Desk organiser | 1x1 · surface | — | tabletop / desk | Body | Static | Static. |
| `laptopClosed` Laptop | 1x1 · surface | browse:type code:type write:type homework:type | tabletop / desk | Body | Static | Static. (+use) the lid opens. |
| `monitor` Monitor and keyboard | 1x1 · surface | browse:type code:type homework:type | tabletop / desk | Body | Static | Static. Takes a 0.23 m disc (half its 0.46 m depth with the keyboard): it fits on counters, islands and the simple, executive, corner and standing desks, not on the 0.38–0.44 m deep computer, writing, laptop and kids' desks. |
| `stuffedToy` Teddy | 1x1 · surface | play:play | tabletop / toys | Body | Static | Static. |
| `toyBlocks` Toy blocks | 1x1 · surface | blocks:tend | tabletop / toys | Block 1, Block 2, Block 3 | Static | Static. |

**In play** (signed off 2026-10-16):
- The toy blocks' Block 1–3 are the red, blue and yellow blocks; the green blocks, pink block and roof keep their
  colours.
- The monitor's Body colours its bezel, keyboard and mouse, not the stand.
- Nothing needed fixing.
