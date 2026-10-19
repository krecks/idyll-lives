# Yard (`families/yard.py`)

The OU4 play and yard things (docs/objects/outdoor.md). One pack, `packs/yard/manifest.json`,
models in `models/yard/`. Built 2026-10-16; they sit in Validate › Yard until signed off (the sandpit and the bike
rack back to Outdoor › Play, the rest to Outdoor › Yard).

The plan names `outdoor.py` as the family; these nine are a family of their own so that three tasks (OU3, OU4, OU5)
building at once don't all edit one script, palette and pack. Moving them into `outdoor.py` later changes only the
files' folder (`models/yard/` to `models/outdoor/`): the keys stay `model.<id>`.

Each is built in place where the game places it (`F`, game axes), with `G.settle(centre=False)` and its
`Model`'s `centre` where it stands off the footprint's middle. Outdoor things are drawn in the town overview, so
each has a far version (`far=`): the model's own build at low detail (`G.far_of`), at rest, all one part, without
the castle and the washing; the log store's is decimated (`far=0.25`).

| Model | Moving part and clip | Played by |
|---|---|---|
| `sandbox` | the sand castle (a keep, four turrets, walls, a flag), origin 2 cm down in the sand: `castle` raises it (2.5 s) and holds it while the castle is built | `sandcastle` |
| `bikeRack` | the bike, hinged on its front tyre's foot: `wheel_up` tips it up 8.5° (0.8 s), so the back axle rests in the stand's cradle; the back wheel, inside the bike (`G.attach`): `wheel_turn` turns it once in 1.2 s, a loop | `tuneUp` (both) |
| `raisedBed` | the plants (foliage: lettuces, carrot tops, tomato bushes) and the crop (plain: carrots, tomatoes), origins on the soil: `grow` holds each at its stage (ripe at rest; picked, the bed bare; growing, lower and no crop; planted, seedlings); `harvest` picks the crop away (1 s); `rustle` sways both 3° about the soil line (0.8 s, every 2.4 s), on top of the stage | `growth`; `harvest`; `tend` |
| `shedSmall` | the ledged and braced door with black T-hinges and a latch, hinged on the left: `door_open` swings it 100° out (1 s) for the whole of pottering | `potter` |
| `greenhouse` | the glass door, hinged on its left post, and the roof vent on the left slope, hinged at the ridge: `door_open` swings the door 70° out and lifts the vent 25° (0.9 s), one clip on both | `tend`, `harvest` |
| `compostBin` | the slatted lid, hinged along the back: `lid_open` lifts it 75° (0.8 s), back down after `hold` 2 s | `compost` |
| `clothesline` | the washing (towels, a sheet, shirts, jeans, socks, pegs), origin on the line: `washing` lets it down (1.5 s). It is the `open` clip too: hanging the washing out leaves the line open (content `leaves`), so the washing stays out until it is taken in | `washing`; `open` |

The rain barrel and the log store are still.

- **The sandpit**: boards with capping rails all round, two corner seats at the back, sand with two heaps, a red
  bucket full of sand, a blue spade, a star mould turned out and its mould.
- **The bike rack**: four galvanised hoops on a rail (each one bent tube, `_tube`, so no seams show where rods met),
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
- **The rain barrel**: a ribbed green barrel with two hoops and a lid, a brass tap and lever, the downpipe behind
  with clips, its shoe turned into the lid.
- **The washing line**: two galvanised T-posts with braces and the three lines; a wicker basket of folded washing.
- **The log store**: the black iron rack (the old part's) under a pent roof of iron sheet (the "Shelter"),
  stacked with logs in brick bond, their pale ends to the front.

**Draw calls: 3 each but the greenhouse (4), the log store (2) and the rain barrel (1).**
- The greenhouse: the still body takes plain and glass; the door and the vent are glass, one draw each. Their edges
  are a white material in the glass finish (`glass_bar`), so each stays one draw yet shows an edge; the door has
  no handle (a metal one would be a second draw on it), as the nature lover's glasshouse. Its plants are plain
  satin (a foliage draw would make 5). `check_glb.py` warns of the 4.
- The washing is all fabric, its pegs too (a plain peg would be a second draw on the part).
- The compost bin's lid has a wooden grip, not metal: not furniture hardware, and metal would be a fourth draw.

**Grown from nothing**: the washing is 6 mm thick and its pegs 12 mm, inside the 16 mm line: flattened onto the
line at rest, it is inside the line and doesn't show (`check_glb.py` warned when it was 12 mm against a 1 cm line).
The castle is flattened 2 cm down in the sand.

**Left out from the old parts**: the watering can by the barrel and the log store's chopping block (both first
built, then dropped: the gate warned the models stood 7 cm off the old parts' centres and 20–140 % deeper).

| Model | Size (w × h × d) | Triangles | Far | Draws | Colour parts |
|---|---|---|---|---|---|
| sandbox | 1.98 × 0.36 × 1.98 | 2980 | 680 | 3 | Frame |
| bikeRack | 1.80 × 0.83 × 0.58 | 3968 | 1716 | 3 | Rack, Bike |
| raisedBed | 1.86 × 0.54 × 0.88 | 4764 | 1344 | 3 | Bed |
| shedSmall | 1.98 × 2.37 × 1.91 | 4552 | 1548 | 3 | Wood, Roof, Door |
| greenhouse | 1.81 × 2.40 × 2.84 | 6444 | 2304 | 4 | Frame, Wall |
| compostBin | 0.84 × 0.78 × 0.79 | 3040 | 436 | 3 | Bin |
| rainBarrel | 0.61 × 2.12 × 0.70 | 2044 | 748 | 1 | Barrel |
| clothesline | 2.76 × 1.80 × 0.76 | 4972 | 684 | 3 | Posts, Washing |
| firewoodStack | 1.00 × 1.10 × 0.38 | 2220 | 548 | 2 | Shelter |

`check_glb.py`: 0 failed; one warning, the greenhouse's 4 draws. Khronos validator: 0 errors, 0 warnings on every
file and far file.

**In the game** (2026-10-16, headless, WebGPU and WebGL2, by day and at 22:00): the gallery next to the old parts;
the clips bench; all nine bought in the garden of `bathroom-check` (the compost bin, the barrel and the log store at
rot 1, 2 and 3), and a resident tending in the greenhouse, pottering in the shed, building a sand castle, hanging out
the washing (still out after), turning the compost, tuning up the bike, and harvesting a ripe raised bed (a state
with the bed ripe: the crop went with the harvest, the bed bare after); the colour studio for each.

## Rebuild

```
python3 -B tools/art/blender/build.py yard            # every .glb, the pack, check_glb
python3 -B tools/art/blender/build.py yard --preview  # previews/sandbox.png, sandbox@castle.png, …
```
