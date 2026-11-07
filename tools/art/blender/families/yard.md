# Yard (`families/yard.py`)

The OU4 play and yard things (docs/objects/outdoor.md), and the rides (a city bike, a kids' bike, an e-scooter,
below). One pack, `packs/yard/manifest.json`,
models in `models/yard/`. Built 2026-10-16; they sat in Validate › Yard until signed off (the sandpit and the bike
rack back to Outdoor › Play, the rest to Outdoor › Yard).

The plan names `outdoor.py` as the family; these nine are a family of their own so that three tasks (OU3, OU4, OU5)
building at once don't all edit one script, palette and pack. Moving them into `outdoor.py` later changes only the
files' folder (`models/yard/` to `models/outdoor/`): the keys stay `model.<id>`.

Each is built in place where the game places it (`F`, game axes), with `G.settle(centre=False)` and its
`Model`'s `centre` where it stands off the footprint's middle. Outdoor things are drawn in the town overview, so
each has a far version (`far=`): the model's own build at low detail (`G.far_of`), at rest, all one part, without
the castle and the washing; the log store's too since 2026-10-22 (it was decimated, `far=0.25`, which
turned its logs to spikes).

What each has since the detail pass (2026-10-22, with gamekit's forms and kit; the same sizes, heights, parts, clips,
draws, paint and far versions, each far version at most half its model):

| Model | Now |
|---|---|
| sandbox | two boards high a side, a mitred capping frame (`kit.moulding`), triangular corner seats, lumpy sand (`_ground`), a lofted bucket with its sand and wire handle, a spade laid in one line (blade, shaft, T grip), star moulds; the castle with the bucket's ridges and a door |
| bikeRack | hoops each one bent tube on feet; the bike's tapered tubes, head tube, raked curved fork, curved seat stays, laced spokes from flanged hubs, C-section mudguards, a lofted saddle, swept-back bars, a cog; the stand's feet |
| raisedBed | lettuces lofted (ruffled outer leaves round a cupped heart), carrot fronds (leaflets, tilted), tomato bushes up canes (the canes in the still part), calyxes on the tomatoes, lumpy soil, post caps |
| shedSmall | weatherboards: each wall swept from a lapped saw-tooth section (`_lap_wall`), cut every 0.3 m so its shading isn't only its buried ends'; a moulded window frame, a sill, a lofted window box on brackets, corner boards, a drip cap, felt laps and drip edges, finials; T-hinges with bolts, a ring latch |
| greenhouse | mortar courses on the dwarf wall (`mortar`), the path's joints, ridge finials, a glazing bar across each side bay, a kick panel on the door |
| compostBin | post caps, galvanised nails, more on the heap (eggshells, `shell`), a shaped grip |
| rainBarrel | one loft: a foot ring, two hoop bands, sixteen ribs between them, a rolled top; a domed lid with grips and a mesh inlet; a flanged tap with a bent spout and a lever |
| clothesline | swept posts with caps, arms with end caps, bent braces, turned feet, tensioners; a lofted basket with a rim and handles, soft folded washing; the washing cut to shape (T-shirts, trousers with their legs, socks with their feet, a sheet with a soft hem, towels with woven bands) and pegs astride the line, all within the line's 16 mm when flattened |
| firewoodStack | round, half and quarter logs (swept rough sections), pale sawn ends with darker hearts, the rack's ends bent tube on feet, a corrugated roof |

Yard keeps G.jitter out (it gives other numbers in each Blender process: outdoor.md); its noise is `noise()` only.

| Model | Moving part and clip | Played by |
|---|---|---|
| `sandbox` | the sand castle (a keep, four turrets, walls, a flag), origin 2 cm down in the sand: `castle` raises it (2.5 s) and holds it while the castle is built | `sandcastle` |
| `bikeRack` | the bike, hinged on its front tyre's foot: `wheel_up` tips it up 8.5° (0.8 s), so the back axle rests in the stand's cradle; the back wheel, inside the bike (`G.attach`): `wheel_turn` turns it once in 1.2 s, a loop | `tuneUp` (both) |
| `raisedBed` | the plants (foliage: lettuces, carrot tops, tomato bushes) and the crop (plain: carrots, tomatoes), origins on the soil: `grow` holds each at its stage (ripe at rest; picked, the bed bare; growing, lower and no crop; planted, seedlings); `harvest` picks the crop away (1 s); `rustle` sways both 3° about the soil line (0.8 s, every 2.4 s), on top of the stage | `growth`; `harvest`; `tend` |
| `shedSmall` | the ledged and braced door with black T-hinges and a latch, hinged on the left: `door_open` swings it 100° out (1 s) for the whole of pottering | `potter` |
| `greenhouse` | the glass door, hinged on its left post, and the roof vent on the left slope, hinged at the ridge: `door_open` swings the door 70° out and lifts the vent 25° (0.9 s), one clip on both | `tend`, `harvest` |
| `compostBin` | the slatted lid, hinged along the back: `lid_open` lifts it 75° (0.8 s), back down after `hold` 2 s | `compost` |
| `clothesline` | the washing (towels, a sheet, shirts, jeans, socks, pegs), a part a line (`washing_a` front, `washing_b`, `washing_c` back), each with its origin on its line: `washing` lets them down one after another (1.5 s, half a second each, `G.grow_in_turn`), so played along with the hang-out they come a line at a time. It is the `open` clip too: hanging the washing out leaves the line open (content `leaves`), so the washing stays out until it is taken in | `washing`; `open` |

The rain barrel and the log store are still. The rides have no clips: the renderer turns their parts by code (below).

- **The sandpit**: boards with capping rails all round, two corner seats at the back, sand with two heaps, a red
  bucket full of sand, a blue spade, a star mould turned out and its mould.
- **The bike rack**: four galvanised hoops on a rail (each one bent tube, `_bent`, so no seams show where rods met),
  a blue bike in front of them (frame, mudguards, spoked wheels, chain ring, cranks, chain, saddle, bars and bell),
  and a repair stand either side of the back wheel. The bars are 0.44 m wide and the bike stands on the tile's
  middle line, so the model keeps within 10 % of the old part's depth.
- **The raised bed**: two boards high on each side, corner posts, soil 2 cm below the rim, a white plant label;
  six lettuces at the back, six carrots in the middle, three tomato bushes at the front. It is plain satin cedar so
  the leaves take the one textured draw (as the nature lover's veggie bed). The plants are one part, so they don't
  shrink towards the bed's middle: they flatten in height and sink (shares of their height: 0.32 picked, under the
  soil; 0.03 growing; 0.1 planted, so each shows a small top where it stands).
- **The garden shed**: sage weatherboards (textured wood) with lap lines, white corner boards, barge boards and
  frames, a felt roof with a ridge cap, a four-pane window (dark gloss, opaque) and a window box in flower. Inside,
  in shadow, a lining, a bench and a shelf of pots, a fork and a rake, seen when the door opens. The door is its
  own painted colour (Door) so the door part is one plain draw; the plan's colour parts were Wood and Roof.
- **The greenhouse**: a dwarf wall (open at the door), a gravel floor and a paved path, the white frame, glass sides,
  ends, gables and roof; two slatted staging benches with pots of seedlings, three tomato plants on canes in grow
  bags at the back. Colour parts: Frame, Wall.
- **The compost bin**: corner posts, slats with gaps on the sides and back, three front boards (the top one lifted
  out) in guide battens, the heap with peel and greens on top.
- **The rain barrel**: a ribbed green barrel with two hoops and a lid with its inlet, a brass tap and lever, in the
  middle of its tile. No downpipe (Q-06, 2026-10-21): a free-standing barrel's pipe ended in mid-air at 2.1 m, and
  0.7 m under the eaves against a wall.
- **The washing line**: two grey T-posts with braces and the three lines; a wicker basket of folded washing.
- **The log store**: the black iron rack (the old part's) under a pent roof of iron sheet (the "Shelter"),
  stacked with logs in brick bond, their pale ends to the front.

**Draw calls: 3 each but the greenhouse (4), the log store (2) and the rain barrel (1).**
- The greenhouse: the still body takes plain and glass; the door and the vent are glass, one draw each. Their edges
  are a white material in the glass finish (`glass_bar`), so each stays one draw yet shows an edge; the door has
  no handle (a metal one would be a second draw on it), as the nature lover's glasshouse. Its plants are plain
  satin (a foliage draw would make 5). `check-glb` warns of the 4.
- The washing is all fabric, its pegs too (a plain peg would be a second draw on a part). Since 0.5.0
  (A-outdoor-02) each line's washing is a part of its own, a draw each, so the still body is one finish to stay
  within four: the posts (were metal), the lines and the basket by the post (was fabric) are satin. `check-glb`
  warns of the 4.
- The compost bin's lid has a wooden grip, not metal: not furniture hardware, and metal would be a fourth draw.

**Grown from nothing**: the washing is 6 mm thick and its pegs 12 mm, inside the 16 mm line: flattened onto the
line at rest, it is inside the line and doesn't show (`check-glb` warned when it was 12 mm against a 1 cm line).
The castle is flattened 2 cm down in the sand.

**Left out from the old parts**: the watering can by the barrel and the log store's chopping block (both first
built, then dropped: the gate warned the models stood 7 cm off the old parts' centres and 20–140 % deeper).

| Model | Size (w × h × d) | Triangles | Far | Draws | Colour parts |
|---|---|---|---|---|---|
| sandbox | 1.98 × 0.35 × 1.98 | 3260 | 602 | 3 | Frame |
| bikeRack | 1.80 × 0.83 × 0.58 | 5248 | 1784 | 3 | Rack, Bike |
| raisedBed | 1.86 × 0.54 × 0.88 | 4888 | 1504 | 3 | Bed |
| shedSmall | 1.98 × 2.37 × 1.91 | 5436 | 1180 | 3 | Wood, Roof, Door |
| greenhouse | 1.81 × 2.43 × 2.84 | 6980 | 2380 | 4 | Frame, Wall |
| compostBin | 0.84 × 0.78 × 0.79 | 2148 | 484 | 3 | Bin |
| rainBarrel | 0.60 × 0.97 × 0.69 | 1744 | 536 | 1 | Barrel |
| clothesline | 2.76 × 1.82 × 0.76 | 2812 | 940 | 4 | Posts, Washing |
| firewoodStack | 1.00 × 1.10 × 0.38 | 3320 | 1316 | 2 | Shelter |
| bicycle | 1.84 × 1.01 × 0.65 | 6312 | 1892 | 1 | Frame |
| bicycleChild | 0.95 × 0.68 × 0.57 | 3920 | 1548 | 1 | Frame |
| eScooter | 0.95 × 1.07 × 0.60 | 4200 | 1172 | 1 | Body |

`check-glb`: 0 failed; one warning, the greenhouse's 4 draws. Khronos validator: 0 errors, 0 warnings on every
file and far file.

**In the game** (2026-10-16, headless, WebGPU and WebGL2, by day and at 22:00): the gallery next to the old parts;
the clips bench; all nine bought in the garden of `bathroom-check` (the compost bin, the barrel and the log store at
rot 1, 2 and 3), and a resident tending in the greenhouse, pottering in the shed, building a sand castle, hanging out
the washing (still out after), turning the compost, tuning up the bike, and harvesting a ripe raised bed (a state
with the bed ripe: the crop went with the harvest, the bed bare after); the colour studio for each.

## The rides: the city bike, the kids' bike, the e-scooter (2026-11-03)

What residents ride to work or school (content `vehicle`; `bicycle` 2 × 1, `bicycleChild` and `eScooter` 1 × 1,
outdoors). All three original, CC0. Each is built along game x, its **front wheel to +x**, its middle line at depth 0,
standing upright on the floor (`G.settle(centre=False)`; the `centre` is the box's middle, a few mm behind the
footprint's because the back mudguard reaches farther than the front). Plain finishes only, so each is **one draw**.

The two bikes are the bike rack's bike made with other numbers: `Bike` holds a bike's numbers (the rack's are its
defaults, `RACK_BIKE`, so the rack is built as before: 5248 triangles, far 1784), `_wheel` and `_bike` build from it
(tapered tubes, a raked curved fork, laced spokes from flanged hubs, C-section mudguards, a lofted saddle, bars bent
in one piece, the chain ring, cranks, pedals and chain). The rack's chain is on +z, to the front; the rides' is on the
**rider's right, game −z** (`drive=-1`: facing +x, +z is the rider's left).

No clips: the renderer moves their parts by code (as the birds', `birds.md`). The parts are the scene's top-level
nodes, unrotated, at scale 1, each with its origin on its pivot:

| Node | What | Origin (game x, height, depth) |
|---|---|---|
| `frame` | everything else, still: frame, fork, mudguards, saddle, bars, grips, bell, chain and cog, chain guard, basket, carrier, lamps, stabilisers, streamers; the scooter's deck, stem, bars, display | (0, 0, 0) |
| `wheel_front`, `wheel_rear` | a whole wheel: tyre, rim, hub, spokes (the scooter's: the hub motor in front, the brake disc behind) | its axle; turned about game z |
| `cranks` (the bikes) | the chain ring, the spindle, both crank arms and both pedals, the arms level at rest | the bottom bracket; turned about game z |
| `stand` (the city bike, the scooter) | the stand, down, holding it upright; hidden while it's ridden, so nothing else rests on it | its pivot |

The geometry the renderer rides them by (game metres, the model's own frame, as exported):

| | City bike | Kids' bike | E-scooter |
|---|---|---|---|
| Back axle (x, h), tyre radius | (−0.55, 0.335), 0.335 | (−0.28, 0.18), 0.18 | (−0.36, 0.105), 0.105 |
| Front axle (x, h), tyre radius | (0.55, 0.335), 0.335 | (0.28, 0.18), 0.18 | (0.37, 0.105), 0.105 |
| Bottom bracket (x, h), crank | (−0.07, 0.29), 0.17 | (−0.05, 0.19), 0.105 | — |
| Pedals at rest (x, depth) | (0.10, +0.13), (−0.24, −0.13) | (0.055, +0.098), (−0.155, −0.098) | — |
| Saddle's top, where the rider sits (x, h) | (−0.30, 0.85) | (−0.17, 0.52) | — |
| Grips' middles (x, h, depth) | (0.20, 0.975, ±0.272) | (0.16, 0.66, ±0.20) | (0.284, 1.03, ±0.23) |
| Deck's top, its x | — | — | 0.125, −0.25 to 0.25 (grip tape −0.23 to 0.23) |
| Stand's pivot (x, h, depth) | (−0.13, 0.27, 0) | — (the stabilisers hold it) | (0.10, 0.072, +0.07) |

- **The city bike** (dark green, `city`): a step-through, its top tube curving down from the head tube into the seat
  tube low down over the curved down tube (the content's "step-through city bike"); 28-inch wheels of 24 spokes;
  long mudguards; a comfort saddle; swept-back bars with grips 0.50 m ahead of the saddle; a bell on the left; a
  chain guard over the chain ring and the top run, between the ring and the crank; a wicker basket on the bars
  (woven bands and ribs, open, its floor inside) on two stays down to the fork ends, a lamp on its front; a carrier
  on stays over the back wheel, a red light at its back; a double-leg centre stand (a yoke on its pivot under the
  chainstays, two legs splayed to rubber feet ±0.16 either side). Without `stand` it is a bike being ridden (the
  stand's plate stays on the frame).
- **The kids' bike** (red, `kid`; white mudguards and chain case): 14-inch wheels with fat tyres, a sloping top
  tube, high-rise bars with a brace and a bell, grips with pink caps and three streamers each (pink, yellow, blue),
  a full chain case, stabilisers: a bracket from each side of the back axle out and down to a little wheel on the
  floor 0.175 m either side (in `frame`; they don't turn). It has no stand: the stabilisers hold it up.
- **The e-scooter** (charcoal, `scooter`; red trim): a deck 0.50 × 0.165 m, its top 0.125 m with black grip tape,
  red strips along its sides; solid 8.5-inch tyres, a fender over each, a red light on the back one's end; the hub
  motor's drum filling the front wheel, five spokes and a brake disc (on the left) in the back one; a neck from the
  deck up to the head tube; the stem raked at 85° from the front axle to the T-bars, a red folding latch on it; grips
  with red ends, a brake lever on the left, a thumb throttle on the right, a bell; a display on the bars' clamp and a
  headlight on the stem; a short side stand on the left (+z) under the deck's front, its foot 0.18 m out, the
  scooter upright on it. It is 0.95 m long to keep within its tile (a real one is 1.05–1.15: the review's 10 %).

Triangles: the city bike 6312 (frame 3384, each wheel 1056, cranks 476, stand 340), the kids' bike 3920 (frame 2228,
each wheel 672, cranks 348), the scooter 4200 (frame 2216, wheels 972 and 848, stand 164). Far versions
(`G.far_of`: no spokes or rims, fewer points round, all one still part, parked): 1892, 1548, 1172. `check-glb`: no
warnings; Khronos validator: 0 errors, 0 warnings on all six files. `cargo review sizes` (kinds `cityBike`,
`kidsBike`, `eScooter`): within range.

## Rebuild

```
cargo art yard            # every .glb, the pack, check-glb
cargo art yard --preview  # previews/yard/sandbox.png, sandbox@castle.png, …
```
