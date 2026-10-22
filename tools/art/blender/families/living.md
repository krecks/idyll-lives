# Living — notes

`living.py` builds the living room's models into `web/public/assets/models/living/` and their entries in
`packs/living/manifest.json`: the fold-down wall bed (`model.wallBed`), the sofa bed (`model.sofaBed`), the ten
L1 sofas and sectionals, the eleven L2 armchairs and small seats, the nine L3 coffee and side tables and the L6
staircase (`model.stairs`). Built 2026-10-14; they sat in Validate › Sofas, Armchairs and Coffee Tables until checked in play.

## The sofas and sectionals (L1)

Each is built where the old part stood (`F`, game axes), its back on the wall's face, and is settled in place
(`G.settle(centre=False)`, `centre=` on its `Model`). Two nodes each:

- **`sofa`**: the upholstery (base, arms, back), any show-wood or metal frame, the back cushions, the pillows and
  the legs. Fabric plus wood (or metal): 2 draws.
- **`seat`**: the seat cushions (and a throw lying on them), their origin on their underside. `sink_in` squashes
  them 2 cm in 0.6 s (`G.squash`), held while the use lasts: `motions: {'uses': {'relax': 'sink_in', 'doze':
  'sink_in'}}` (the classic sofa has no `doze`). 1 draw.

The seat cushions' tops are the seat heights (`characters.ts` `SEATS`, content `seat`): 0.44–0.45 m, the designer
sofa's 0.425 m.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.sofa` (camel back, cabriole legs) | 1.88 × 0.94 × 0.82 m | 6,084 | 3 | Upholstery, Cushions, Frame, Legs, Pillow, Second pillow |
| `model.loveseat` | 1.84 × 0.83 × 0.84 m | 5,644 | 3 | Upholstery, Cushions, Legs, Pillow |
| `model.sofaThree` | 2.82 × 0.87 × 0.86 m | 7,456 | 3 | Upholstery, Cushions, Legs, Pillows, Throw |
| `model.sofaDesign` (brass rail and legs) | 1.87 × 0.72 × 0.82 m | 5,936 | 3 | Upholstery, Cushions, Frame, Legs |
| `model.sofaThreeSeat` | 2.86 × 0.82 × 0.88 m | 6,556 | 3 | Upholstery, Cushions, Legs, Pillow, Second pillow |
| `model.sectionalMiddle` | 1.00 × 0.82 × 0.88 m | 2,656 | 3 | Upholstery, Cushions, Legs |
| `model.sectionalArmLeft` (arm at +x) | 1.00 × 0.82 × 0.88 m | 3,196 | 3 | Upholstery, Cushions, Legs |
| `model.sectionalArmRight` (arm at −x) | 1.00 × 0.82 × 0.88 m | 3,196 | 3 | Upholstery, Cushions, Legs |
| `model.sectionalCorner` | 1.00 × 0.82 × 0.89 m | 3,816 | 3 | Upholstery, Cushions, Legs, Pillow |
| `model.chaise` (1 × 2) | 0.96 × 0.82 × 1.90 m | 3,384 | 3 | Upholstery, Cushions, Legs, Throw |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Seen in play (WebGPU and
WebGL, day and night, all four facings across the set): residents sit on each, and `sink_in` plays and holds.

- The sectional modules are 1.00 m wide, so they butt up in a row; the arms are where the old parts had them
  (`sectionalArmLeft`'s on +x, the sitter's left).
- Where an old part had no frame (the `sofa_run` sofas), there is no Frame colour part, as on the sofa bed.
- The pillows lean on the back cushions; the seat sinking leaves them where they are (1 cm into the cushion at
  rest, so the gap is small).

## The armchairs and small seats (L2)

Built as the sofas are, from the old primitive parts (`armchair_bergere` and the rest). Most have a still part and a `seat` that sinks 2 cm (`sink_in`) for `sit` (and `read`).

- **Recliner** (`recline`, `footrest_up`, for relax and doze): a skirted base and arms (still); the seat cushion,
  the back and its head cushion as one part (`seat`) turning back 25° about the sitter's hips (content seat 0.44 m,
  forward 0.12) and gliding 8 cm forward as it does (`G.path`, a wall-hugger's), with `ride` on it, so the resident
  reclines with it and rests the head on the head cushion; the padded footrest, folded flat against the front of the
  base at rest, swings up 80°. Since 2026-10-21 (M-living-04) the back stands 1.0 m high with a head cushion, over a
  0.80 m deep base (it was 0.84 × 0.70, the back stopping at the sitter's shoulder blades). It stands 11 cm off the
  wall's face; with the glide the tipped back's top stops 2 cm short of the wall.
- **Rocking chair** (`rock`, a loop for relax and read, `ride`): the whole chair rocks ±6° in 1.4 s about the
  rockers' centre of curvature (1 m above the floor), so the rockers roll on the floor; the rockers are flat bent
  strips.
- **Bean bag** (`squash` for flop): the bag flattens to 85 % and spreads 6 %.
- **Floor cushion**: all of it is the seat that sinks; **pouf** and **ottoman**: the buttoned top sinks.
- **Designer lounge chair**: still, as in the plan (moulded shells on a five-star base).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.armchair` (bergère) | 0.74 × 0.98 × 0.80 m | 3,964 | 3 | Upholstery, Cushion, Frame, Legs, Pillow |
| `model.armchairLounge` | 0.82 × 0.83 × 0.84 m | 3,736 | 3 | Upholstery, Cushions, Legs |
| `model.armchairLeather` | 0.73 × 0.95 × 0.85 m | 3,964 | 3 | Frame, Upholstery |
| `model.armchairDesign` | 0.76 × 0.96 × 0.85 m | 3,588 | 2 | Frame, Upholstery, Base |
| `model.armchairWingback` | 0.84 × 1.10 × 0.80 m | 3,840 | 3 | Upholstery, Cushion, Legs |
| `model.recliner` | 0.92 × 0.99 × 0.80 m | 3,716 | 3 | Upholstery, Cushions & footrest |
| `model.rockingChair` | 0.53 × 1.03 × 0.90 m | 3,732 | 2 | Wood, Cushion |
| `model.pouf` | 0.72 × 0.47 × 0.50 m | 2,892 | 3 | Leather, Buttons, Legs |
| `model.ottoman` | 0.62 × 0.45 × 0.62 m | 3,688 | 3 | Upholstery, Buttons, Feet |
| `model.floorCushion` | 0.62 × 0.14 × 0.62 m | 2,176 | 1 | Cushion, Piping, Button |
| `model.beanBag` | 0.84 × 0.70 × 0.84 m | 2,500 | 1 | Bag, Seat |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Seen in play on WebGPU (all four facings across the set) and
WebGL at night: residents sit, recline, rock and flop on each, and each clip plays.

## The coffee and side tables (L3)

Copied from the old parts, tops at the content `surface` heights (0.42 m for the coffee tables, 0.51–0.80 m
for the side, nesting and console tables). Still, but for two:

- **Coffee table** (`drawer_open` for magazine, `hold` 2.5 s): a drawer in the front apron, a magazine in it,
  slides out 30 cm. It is 6 cm lower than the old part (that one had magazines on the top; they're in the
  drawer now).
- **Nesting tables**: the small oak table is a part of its own (`small_table`), left decorative in 0.3.0: a clip
  can pull it out once it has a use.
- **Side table**: the model copies the catalog pack's `side_table` (oak with a drawer and a brass knob), which stood
  in front of the base manifest's version; the two differed before the model, as `check-glb` warned.
- What stands on the tops (a plant, vases, magazines, a box) is plain finish, so it costs no draw call.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.coffeeTable` | 1.30 × 0.42 × 0.74 m | 2,124 | 3 | Wood, Knob |
| `model.coffeeTableGlass` | 1.30 × 0.60 × 0.74 m | 2,428 | 2 | Frame, Glass |
| `model.coffeeTableMarble` | 0.88 × 0.58 × 0.88 m | 2,436 | 1 | Top, Base |
| `model.sideTable` | 0.55 × 0.55 × 0.45 m | 720 | 2 | Wood, Knob |
| `model.coffeeTableRound` | 0.84 × 0.42 × 0.84 m | 2,036 | 2 | Top, Base |
| `model.coffeeTableLong` | 1.66 × 0.42 × 0.72 m | 1,056 | 2 | Top, Legs |
| `model.nestingTables` | 0.57 × 0.51 × 0.55 m | 1,128 | 2 | Top, Small table, Legs |
| `model.sideTableRound` | 0.52 × 0.56 × 0.52 m | 1,396 | 1 | Top, Base |
| `model.consoleTable` | 1.80 × 0.80 × 0.56 m | 2,360 | 3 | Top, Legs, Baskets |

`check-glb`: 0 failed. Khronos validator: 0 errors, 0 warnings each. Seen in play on WebGPU (each bought, all four
facings across the set; the drawer opening while a resident flips through a magazine) and on the WebGL clips bench.

## The shelves (L5)

Built where the game places them stood (`bookshelf` and the rest), each with its
back on the wall's face (depth -0.43); the old parts moved 3–6 cm forward with them (they reached into the wall),
the corner bookcase's and the ladder shelf's redrawn as the models are. Tops at the content `surface` heights: the
low bookcase 0.90 m, the cube shelf 0.80 m, the industrial shelving 1.83 m (the content rectangles are unchanged; the
models' tops now start at the wall's face, 1–4 cm in front of the rectangles' backs).

- **Books** are rows of plain satin boxes (`_book_row`, seeded, so they rebuild the same): a cover, the page block
  showing 2 mm above it, a gilt band on a third of the spines, the odd gap and a few lying flat at the end of some
  rows. Joined into the still body, they cost no draw call of their own.
- **`book_out`** (the five bookcases): one red book at reading height is a part of its own, hinged on the foot of
  its spine, and tips out 25° in 0.6 s for read, study and homework (as each offers them), back at the end of the
  use. 1 draw.
- **Corner bookcase**: a white bookcase with a teal back and quarter-round open shelves round its right end, to the
  wall (the old part had square ones); the old part draws them as boxes.
- **Ladder shelf**: two rails leaning on the wall (on its face at the top, 32 cm out at the floor), four boards
  through them, 6 cm proud of their fronts and back to the wall, narrowing upwards. Colour parts Frame and Shelves
  (it has no back).
- **Display cabinet**: walnut, solid sides, a pale back, glass-look shelves (plain `gloss`, so the door is its only
  glass), ornaments; a brass-framed glass door with a brass knob, a node of its own (`door`, its origin on the hinge
  at its left, `DOOR_HINGE`) with no clip yet: the (+use) swing needs only `G.swing(door, 'door_open', …)` and a use.
  Still, it counts 3 draws; with a clip the door's brass and glass draw on their own (4).
- Plants on the shelves are plain satin spheres, not `foliage`, so they cost no draw; the cube shelf's bins are
  `fabric` (its second draw), the ladder shelf's basket too.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts | Clip |
|---|---|---|---|---|---|
| `model.bookshelf` | 0.90 × 1.92 × 0.38 m | 4,136 | 3 | Frame, Back | `book_out` (read, study, homework) |
| `model.bookcaseLow` | 0.92 × 1.17 × 0.38 m | 2,820 | 3 | Frame, Back | `book_out` (read) |
| `model.bookcaseWide` (2 × 1) | 1.86 × 2.00 × 0.47 m | 7,244 | 3 | Frame, Back | `book_out` (read, study, homework) |
| `model.shelvingIndustrial` (2 × 1) | 1.86 × 1.83 × 0.53 m | 6,776 | 2 | Frame, Shelves | — |
| `model.bookcaseTall` | 0.90 × 2.00 × 0.34 m | 4,584 | 3 | Frame, Back | `book_out` (read, study) |
| `model.bookcaseCorner` | 0.90 × 1.90 × 0.34 m | 4,196 | 2 | Frame, Back | `book_out` (read) |
| `model.cubeShelf` | 0.80 × 0.80 × 0.36 m | 2,676 | 2 | Frame, Back | — |
| `model.ladderShelf` | 0.64 × 1.80 × 0.36 m | 3,416 | 3 | Frame, Shelves | — |
| `model.displayCabinet` | 0.86 × 1.91 × 0.45 m | 3,536 | 3 | Frame, Back | — (door a part of its own, left decorative in 0.3.0) |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Seen in the gallery beside the
old parts, on the clips bench (WebGPU), and in play: the bookshelf read at (`book_out` plays, held while reading),
all nine bought in the house across the four facings, coloured, on WebGPU by day and WebGL with the night light.

## The staircase (L6)

Built 2026-10-16; they sat in Validate › Stairs until signed off (back to Living › Stairs). Build mode's Stairs tool and the
house generator place it (`cfg.stairs`), not the buy catalog, which leaves stairs out: the Validate group showed empty.

- **Steps where the old ones are**: 14 steps of 0.2 m up and 4/14 m deep over the 1 × 4 footprint, the first
  starting at the front edge, the top of the last (2.8 m, house.ts `WALL_HEIGHT`) level with the floor above at the
  back edge. Residents are drawn rising evenly from the foot of the first step to the top of the last (sim-core
  `World::sim_height`), as over the old part, so their feet meet the treads as before. The same flight serves
  stairs up and stairs down to the basement, at every facing.
- **Oak treads** (0.03 m, their tops the steps' tops; each but the first reaches 2 cm past the riser below: the first
  stays inside the footprint) on **white risers**, between **closed strings** cut level on the floor in front and
  with the floor above at the back; a **black iron railing** each side on the strings (x ±0.40, inside the tile's
  wall face): a newel at the foot and the head, two balusters a step and a sloping handrail where the old part's
  rails are (0.7 m above the nosings). The flight is solid underneath, as the old part was.
- `check-glb` no longer takes the stairs' back for a wall (content `stairs`): it is the top of the flight, where
  the floor above goes on.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.stairs` | 0.86 × 3.68 × 4.00 m | 5,468 | 2 | Treads, Railing |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings. Seen in play (WebGPU, and WebGL2 at
night): generated houses with a flight up (facing 2) and flights down to the basement (facings 0, 1 and 3), a
resident walking up and down each; the stairwell from the floor above and from the ground over stairs down; the
colour studio.

## The detail pass (2026-10-22)

Every model but the two worked examples (`model.armchair`, `model.sofa`, rebuilt the day before) rebuilt with the
forms (gamekit/forms.py), same keys, sizes, centres, nodes, clips, motions, paint and draws; the triangles in the
tables above. The helpers in living.py:

- **`_cushion`** is a `F.soft` box: rounded, puffed on its two largest faces, piped round both seams (`piping`
  None: none). `keep_top=True` for a seat cushion: its puffed middle where the old box's top was, its edges lower,
  so the seat stays at the content height (the gate checks within 2 cm; a puff on top of the old top was 2.5 cm
  high). **`_panel`**: firm upholstery (a base, an arm, a back), rounded, welted. Never puff a part across its
  width or depth at the model's edge: `soft_box` puffs both faces, and the model grows (the sofas did, 1–2 cm).
- **`_leg`** (a turned leg: collar, taper, splay), **`_throw`** (a throw swept over a seat's front edge,
  wobbled), **`_rolled_arm`** (an English arm: its section swept, a welt round its scroll), **`_press`** (tufts
  pressed into a top), **`_round_rings`**/**`_disc`** (turned things lofted from (height, radius) rows: table
  tops with moulded edges, feet, vases, finials), **`_shell`** (a dished plywood shell), **`_top`** (a table top
  with rounded corners and eased edges), **`_turned_leg`**, **`_knob`** (a turned knob), **`_fillet`** (a polyline
  with its corners rounded, for bent tube: a Catmull-Rom sweep through close corner points bulges past them, which
  stood the console table 1.3 cm taller), **`_basket`**, **`_plant_bowl`**.

What each has now:

- **Sofas and sectionals**: welted bases, track arms boxed to the base's foot and welted (the loveseat, the lounge
  chair, the three-seater, the sectionals, the chaise, the sofa bed), piped and puffed seat and back cushions,
  plump pillows, splayed turned legs; the three-seat sofa English rolled arms with welted scrolls and a throw draped
  over its seat's front; the designer sofa a buttoned, piped back in two rows of tufts, a bent brass front rail
  turning into the arms, brass arm caps rolling over the front, tapering splayed brass legs in cups; the chaise a
  throw draped over its foot.
- **Armchairs**: the leather armchair's back legs bend up into the raked posts in one piece, teak paddle arms, a back
  cushion in three stitched channels; the designer chair dished plywood shells, piped cushions, cast arms in one bent
  piece, a waisted column and five flat cast spokes; the wingback an arched back, shaped wings, rolled arms with
  welts, a deck raised on cabriole front legs and raked back legs; the recliner pillow-top arms and a bustle back
  (lumbar roll, back cushion, head cushion); the rocking chair one swept strip per rocker, turned legs and posts,
  paddle arms, bowed spindles, a curved crest rail, stretchers, a saddled seat and the cushion's ties; the pouf
  welted, its top tufted; the ottoman a bellied drum, welts, bun feet, a domed top tufted to seven buttons; the floor
  cushion boxed and welted, tufted to its button; the bean bag a slumped, wrinkled loft with the seat panel (`bag_seat`)
  as the faces of its dent, and a carry strap.
- **Tables**: tops with rounded corners and eased edges, or turned with moulded edges; the coffee table turned legs
  under square blocks, side aprons, stretchers under the slats, a fielded drawer front and a turned knob, a second
  magazine; the glass table bent-tube frames in closed loops, feet, a leafy plant, a turned vase, books on the
  shelf; the marble table a moulded edge, a tulip foot with a waist, swept veins; the round table a reeded drum; the
  long table sled frames of square tube, a lidded box and a book; the nesting tables bent-rod leg frames; the
  console table bent square-tube ends, rails, woven baskets with rolled rims and rope handles.
- **Wall bed**: a cove cornice and bead, edged shelves, books with pages and gilt bands; the bed a piped, puffed
  mattress (its middle at 0.46), puffed pillows, a soft throw, fielded door panels, a bent bar handle on roses and a
  folding foot with bracket ends. **Sofa bed**: as the sofas, one long seat cushion with a welt down its middle (two
  cushions showed the tucked mattress in the gap between them), the mattress rounded, a thin top sheet, a bent-tube
  leg.
- **Staircase**: bullnose treads over a cove, a moulded capping on the strings, newels with a collar and a ball
  finial, balusters with a knuckle, a handrail in one moulded piece.
- **Shelves** (lighter): a bead under each top, lipped shelf fronts, turned vases, pots with rolled rims and lumpy
  foliage with a few long leaves, frames with a moulding round the photo, knobs turned; the display cabinet a cove
  cornice, beads and bun feet; the cube shelf's bins soft with rolled rims.

`cargo art living`: 0 failed, no warnings for these models. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art living            # the .glb files, the pack, check-glb
cargo art living --preview  # previews/living/<id>.png, <id>@<clip>.png
```
