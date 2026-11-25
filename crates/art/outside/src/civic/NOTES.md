# Civic — notes

This module builds the fittings of the town's named places (`content/civic.json`) into `web/public/assets/models/civic/`
and their entries in `packs/civic/manifest.json`. Built 2026-11-17 (in Blender) in place of the
stand-ins the content pointed at until then; ported to Rust 2026-11-18. None of them has a moving part: no use of
theirs opens anything.

Each is built where the game places it (`F`, game axes). The desks and counters with `staff` face the customer at the
front (+z); whoever serves stands 0.85 m behind their middle (content `staff.at`), so the monitors, drawers, tills and
the bar's shelves face the back. The lockers, the cash machine, the wall bars and the vault door stand with their backs
on the wall face (-0.43). A test (`content_facts`) holds the pew's and the waiting bench's seat (0.45 m, forward 0,
three slots) to `content/civic.json`.

## The desks and counters (2 × 1)

- **Reception desk**: a white carcass whose ends step down from the customer's oak ledge (1.08 m) to the worktop
  behind (0.75 m), oak fins under an oak band along the front, a drawer pedestal on the receptionist's side. On the
  ledge a bell, a stepped stand of leaflets and a plant; on the worktop a monitor, a keyboard and mouse, a phone and a
  pot of pens.
- **Hotel reception**: the same carcass in walnut with three raised panels and a brass kick strip, a marble ledge with
  a rounded nose on a walnut bead. On the ledge the open guest book and its pen, a bell and a banker's lamp (its green
  shade glows at night); at the back a walnut board of brass hooks with the room keys on red fobs, a few rooms out.
- **Duty desk**: the carcass in blue-grey with a chequered band along the front, a glazed screen in an aluminium frame
  over the ledge with a hatch under its middle pane and a grille to speak through, the log book open in front of the
  hatch with a pen on a chain; behind, a monitor, a keyboard and a radio on its charger.
- **Teller counter**: oak, three raised panels, the glass nearly 2 m up between oak rails and posts, a grille and a
  deal tray sunk through the ledge at each window, a pen on a chain; behind, cash trays of notes and coins, a monitor
  and a keyboard.
- **Bar**: panelled mahogany with a thick top rounded to the drinkers, a cove under it and a brass foot rail on three
  brackets. Three pumps with ceramic handles and coloured clips over a drip tray, a runner with two pints and beer
  mats; on the barman's side, shelves of glasses and bottles under the top.
- **Teacher's desk**: oak, a pedestal of three drawers to the teacher, a raised modesty panel to the class; the
  register, a pile of exercise books, an apple, a pot of pencils and a globe.

## The seats

- **Pew** (3 × 1): an oak seat at 0.45 m with a reclined back, a rounded rail, shaped ends with a raised field rising to a
  scroll at the back, and the ledge of hymn books behind for the row at the back.
- **Waiting bench** (3 × 1): three moulded seats (one swept outline, rounded) at the slots (-1, 0, 1) on a steel beam
  and two T legs, a laminate table between each pair with a pile of magazines.
- **Pupil's desk** (1 × 1): the old kind, a lidded pine desk with a flat strip for the inkwell and the pen's groove,
  joined to its seat and back slats by cast-iron sides on floor runners. The seat is at the default seat (0.45 m at
  the tile's middle), so the desk's near edge is 0.2 m ahead of the hips and its lid only 0.23 m deep.
- **Couch** (1 × 2): a wipe-clean pad at 0.45 m (the default seat) on a white tubular frame with a shelf, the head rest
  raised 30 degrees at the back, the paper roll at its head and the sheet down it and along the pad.

## The rest

- **Clothes rail** (2 × 1): one chrome tube on two castored feet, fifteen garments edge on (shirts, dresses, coats,
  trousers over the bar), each a rounded outline on its wire hook.
- **Plant table** (2 × 1): slatted staging on a galvanised frame, ten terracotta pots of plants (mounds, some in flower,
  upright ones up a stem) with price tags, and two seed trays of seedlings on the shelf below. A far version (decimated
  to a quarter), as it may stand in a garden centre's yard.
- **Cash machine** (1 × 1): a dark cabinet with a sloping fascia round the screen (it glows), a keypad on a shelf with
  coloured keys, card, cash and receipt slots, privacy wings and a hood with a lit green band.
- **Wall bars** (1 × 1, on the wall): sixteen beech rungs between two uprights on steel brackets, the top ones
  standing out over the climber.
- **Altar** (2 × 1): stone on a step, the fair linen over its top and down its ends, a purple frontal with a gold cross
  and fringe, two brass candlesticks with lit candles (the flames glow), a brass cross and an open missal.
- **Lockers** (2 × 1): five columns of two steel doors with vents, a lock, a number and a name card each, on a dark plinth.
- **Mannequin** (1 × 1): a gloss dummy with an egg head on a chrome disc and rod, in a red jacket over a white shirt,
  navy trousers and shoes.
- **Luggage trolley** (1 × 1): a carpeted deck on four castors with a brass rim and bumpers, brass uprights under an
  arch and a crown, a hanging rail with a garment bag, a suitcase standing and one lying.
- **Vault door** (2 × 1, on the wall): a round steel door 0.18 m proud of its frame, a ring of bolts, a six-spoked wheel,
  the dial over it and the hinge on its right.

| Model | Size (w × h × d) | Triangles (far) | Draws | Colour parts |
|---|---|---|---|---|
| `model.receptionDesk` | 1.92 × 1.37 × 0.85 m | 4,916 | 2 | Desk, Ledge |
| `model.hotelReception` | 1.94 × 1.48 × 0.86 m | 4,440 | 2 | Desk, Top |
| `model.dutyDesk` | 1.92 × 1.93 × 0.85 m | 3,660 | 2 | Counter, Top |
| `model.tellerCounter` | 1.92 × 2.00 × 0.85 m | 4,420 | 3 | Counter, Worktop |
| `model.teacherDesk` | 1.50 × 1.12 × 0.71 m | 4,064 | 2 | Desk |
| `model.lectern` | 0.66 × 1.23 × 0.56 m | 1,668 | 2 | Wood |
| `model.barCounter` | 1.96 × 1.46 × 0.83 m | 6,640 | 2 | Bar, Rail |
| `model.schoolDesk` | 0.62 × 0.83 × 0.78 m | 2,136 | 2 | Wood, Frame |
| `model.pew` | 2.98 × 1.04 × 0.67 m | 3,020 | 2 | Wood |
| `model.waitingBench` | 2.70 × 0.85 × 0.53 m | 2,760 | 1 | Seats, Frame |
| `model.examCouch` | 0.62 × 0.81 × 1.87 m | 1,500 | 1 | Pad, Frame |
| `model.clothesRack` | 1.76 × 1.51 × 0.52 m | 6,284 | 2 | Rail |
| `model.plantTable` | 1.88 × 1.21 × 0.80 m | 7,980 (1,891) | 3 | Slats, Pots |
| `model.atm` | 0.70 × 1.68 × 0.64 m | 2,712 | 1 | Cabinet, Fascia |
| `model.wallBars` | 0.95 × 2.50 × 0.25 m | 1,760 | 2 | Wood |
| `model.altar` | 1.92 × 1.69 × 0.85 m | 2,048 | 2 | Stone, Frontal |
| `model.lockerRow` | 1.92 × 1.85 × 0.45 m | 2,988 | 1 | Doors, Frame |
| `model.mannequin` | 0.55 × 1.75 × 0.40 m | 2,912 | 2 | Jacket, Trousers, Dummy |
| `model.luggageTrolley` | 0.90 × 1.87 × 0.54 m | 3,196 | 2 | Frame, Carpet |
| `model.vaultDoor` | 1.92 × 1.92 × 0.32 m | 4,056 | 1 | Door, Frame |

Draws: one for the plain finishes, one for wood (the pew, the desks, the bar, the wall bars), fabric (the clothes, the
altar's cloths, the trolley's carpet, the mannequin's outfit), foliage (the plant table) and glass (the teller's
screen). The duty desk's screen is glass too, its frame and everything else plain. `check-glb`: 20 models, 0 failed,
no warnings.

## Rebuild

```
cargo art civic            # the .glb (and the plant table's .far.glb), the pack, check-glb
cargo art civic --preview  # previews/civic/<id>.png
cargo art civic --check    # the committed bytes, nothing written
```

`bun tools/bench/check.mjs gallery --keys model.<id> --each --scale adult` shows the glass as the game does.

## The port (2026-11-18)

Built in Rust since 2026-11-18, proved against the last Blender build (`cargo check-glb --against ref/blender-civic
--family civic`). The plant table's far version is the kernel's decimation.

## Edges at the play camera (2026-11-25)

The gate's `edges` found 75 (lectern 14, plant table 10, hotel reception 10, bar counter 8, duty desk 6, altar 6,
vault door 4, mannequin 4, teller counter 2, exam couch 2, three desks 1); none now, none accepted. The family's
revolves, lofts, sweeps and extrusions that named no bevel take the crate's `eased` (a 4 mm chamfer on turns over 50°,
none on a part under 12 mm one way or 10 cm every way): the open books' page blocks, the pots, the brass, the
candles, the vault's rims. Not the bar's tumblers, the pen pot or the hotel lamp's shade: thin-walled, their chamfer
made edges of its own. The plant table's pots have their rim's top edge cut across in the profile instead, ten
round (a chamfer all round took it past 8,000 triangles), its legs a one-step chamfer; the teller's speaking
grille's back rim is cut across the same way; a book's boards take a 2 mm chamfer (3 mm met in a 6 mm board's
middle, a ridge). The teacher's globe's turned base lays box UVs on its chamfered mesh (`boxed`): round about its
axis, the walnut's grain stretched over its slopes.
