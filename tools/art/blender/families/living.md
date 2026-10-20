# Living — notes

`living.py` builds the living room's models into `web/public/assets/models/living/` and their entries in
`packs/living/manifest.json`: the fold-down wall bed (`model.wallBed`), the sofa bed (`model.sofaBed`), the ten
L1 sofas and sectionals, the eleven L2 armchairs and small seats, the nine L3 coffee and side tables and the L6
staircase (`model.stairs`). Built 2026-10-14; they sit in Validate › Sofas, Armchairs and Coffee Tables until checked in play.

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
| `model.sofa` (camel back, cabriole legs) | 1.88 × 0.94 × 0.82 m | 7,028 | 3 | Upholstery, Cushions, Frame, Legs, Pillow, Second pillow |
| `model.loveseat` | 1.84 × 0.83 × 0.84 m | 1,788 | 3 | Upholstery, Cushions, Legs, Pillow |
| `model.sofaThree` | 2.82 × 0.87 × 0.86 m | 2,460 | 3 | Upholstery, Cushions, Legs, Pillows, Throw |
| `model.sofaDesign` (brass rail and legs) | 1.87 × 0.72 × 0.82 m | 1,996 | 3 | Upholstery, Cushions, Frame, Legs |
| `model.sofaThreeSeat` | 2.86 × 0.82 × 0.88 m | 2,352 | 3 | Upholstery, Cushions, Legs, Pillow, Second pillow |
| `model.sectionalMiddle` | 1.00 × 0.82 × 0.88 m | 848 | 3 | Upholstery, Cushions, Legs |
| `model.sectionalArmLeft` (arm at +x) | 1.00 × 0.82 × 0.88 m | 1,036 | 3 | Upholstery, Cushions, Legs |
| `model.sectionalArmRight` (arm at −x) | 1.00 × 0.82 × 0.88 m | 1,036 | 3 | Upholstery, Cushions, Legs |
| `model.sectionalCorner` | 1.00 × 0.82 × 0.89 m | 1,412 | 3 | Upholstery, Cushions, Legs, Pillow |
| `model.chaise` (1 × 2) | 0.96 × 0.82 × 1.90 m | 1,144 | 3 | Upholstery, Cushions, Legs, Throw |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Seen in play (WebGPU and
WebGL, day and night, all four facings across the set): residents sit on each, and `sink_in` plays and holds.

- The sectional modules are 1.00 m wide, so they butt up in a row; the arms are where the old parts had them
  (`sectionalArmLeft`'s on +x, the sitter's left).
- Where an old part had no frame (the `sofa_run` sofas), there is no Frame colour part, as on the sofa bed.
- The pillows lean on the back cushions; the seat sinking leaves them where they are (1 cm into the cushion at
  rest, so the gap is small).

## The armchairs and small seats (L2)

Built as the sofas are, from the old primitive parts (`armchair_bergere` and the rest). Most have a still part and a `seat` that sinks 2 cm (`sink_in`) for `sit` (and `read`).

- **Recliner** (`recline`, `footrest_up`, for relax and doze): a skirted base and arms (still); the seat cushion and
  the back as one part (`seat`) turning back 25° about the sitter's hips (content seat 0.44 m, forward 0), with
  `ride` on it, so the resident reclines with it; the padded footrest, folded flat against the front of the base at
  rest, swings up 80°. It stands 13 cm off the wall's face, so the tipped back stops 4 cm short of the wall; the
  hips (content `seat.forward` 0.12) and the old part (drawn at rest) moved forward with it.
- **Rocking chair** (`rock`, a loop for relax and read, `ride`): the whole chair rocks ±6° in 1.4 s about the
  rockers' centre of curvature (1 m above the floor), so the rockers roll on the floor; the rockers are flat bent
  strips.
- **Bean bag** (`squash` for flop): the bag flattens to 85 % and spreads 6 %.
- **Floor cushion**: all of it is the seat that sinks; **pouf** and **ottoman**: the buttoned top sinks.
- **Designer lounge chair**: still, as in the plan (moulded shells on a five-star base).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.armchair` (bergère) | 0.74 × 0.98 × 0.80 m | 3,880 | 3 | Upholstery, Cushion, Frame, Legs, Pillow |
| `model.armchairLounge` | 0.82 × 0.83 × 0.84 m | 1,224 | 3 | Upholstery, Cushions, Legs |
| `model.armchairLeather` | 0.73 × 0.95 × 0.85 m | 1,348 | 3 | Frame, Upholstery |
| `model.armchairDesign` | 0.76 × 0.96 × 0.85 m | 1,756 | 2 | Frame, Upholstery, Base |
| `model.armchairWingback` | 0.84 × 1.10 × 0.80 m | 1,788 | 3 | Upholstery, Cushion, Legs |
| `model.recliner` | 0.92 × 0.84 × 0.70 m | 1,248 | 3 | Upholstery, Cushions & footrest |
| `model.rockingChair` | 0.53 × 1.03 × 0.90 m | 1,148 | 2 | Wood, Cushion |
| `model.pouf` | 0.72 × 0.47 × 0.50 m | 1,032 | 3 | Leather, Buttons, Legs |
| `model.ottoman` | 0.62 × 0.45 × 0.62 m | 1,672 | 3 | Upholstery, Buttons, Feet |
| `model.floorCushion` | 0.62 × 0.14 × 0.62 m | 1,120 | 1 | Cushion, Piping, Button |
| `model.beanBag` | 0.84 × 0.70 × 0.84 m | 1,824 | 1 | Bag, Seat |

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
| `model.coffeeTable` | 1.30 × 0.42 × 0.74 m | 1,428 | 3 | Wood, Knob |
| `model.coffeeTableGlass` | 1.30 × 0.60 × 0.74 m | 1,764 | 2 | Frame, Glass |
| `model.coffeeTableMarble` | 0.88 × 0.58 × 0.88 m | 1,164 | 1 | Top, Base |
| `model.sideTable` | 0.55 × 0.55 × 0.45 m | 708 | 2 | Wood, Knob |
| `model.coffeeTableRound` | 0.84 × 0.42 × 0.84 m | 884 | 2 | Top, Base |
| `model.coffeeTableLong` | 1.66 × 0.42 × 0.72 m | 864 | 2 | Top, Legs |
| `model.nestingTables` | 0.57 × 0.51 × 0.55 m | 1,080 | 2 | Top, Small table, Legs |
| `model.sideTableRound` | 0.52 × 0.56 × 0.52 m | 964 | 1 | Top, Base |
| `model.consoleTable` | 1.80 × 0.80 × 0.56 m | 1,024 | 3 | Top, Legs, Baskets |

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
| `model.bookshelf` | 0.90 × 1.92 × 0.38 m | 3,512 | 3 | Frame, Back | `book_out` (read, study, homework) |
| `model.bookcaseLow` | 0.92 × 1.17 × 0.38 m | 2,172 | 3 | Frame, Back | `book_out` (read) |
| `model.bookcaseWide` (2 × 1) | 1.86 × 2.00 × 0.47 m | 6,056 | 3 | Frame, Back | `book_out` (read, study, homework) |
| `model.shelvingIndustrial` (2 × 1) | 1.86 × 1.83 × 0.53 m | 6,196 | 2 | Frame, Shelves | — |
| `model.bookcaseTall` | 0.90 × 2.00 × 0.34 m | 3,860 | 3 | Frame, Back | `book_out` (read, study) |
| `model.bookcaseCorner` | 0.90 × 1.90 × 0.34 m | 3,416 | 2 | Frame, Back | `book_out` (read) |
| `model.cubeShelf` | 0.80 × 0.80 × 0.36 m | 1,476 | 2 | Frame, Back | — |
| `model.ladderShelf` | 0.64 × 1.80 × 0.36 m | 2,340 | 3 | Frame, Shelves | — |
| `model.displayCabinet` | 0.86 × 1.91 × 0.45 m | 2,580 | 3 | Frame, Back | — (door a part of its own, left decorative in 0.3.0) |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Seen in the gallery beside the
old parts, on the clips bench (WebGPU), and in play: the bookshelf read at (`book_out` plays, held while reading),
all nine bought in the house across the four facings, coloured, on WebGPU by day and WebGL with the night light.

## The staircase (L6)

Built 2026-10-16; in Validate › Stairs until signed off (back to Living › Stairs). Build mode's Stairs tool and the
house generator place it (`cfg.stairs`), not the buy catalog, which leaves stairs out: the Validate group shows empty.

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
| `model.stairs` | 0.86 × 3.68 × 4.00 m | 3,400 | 2 | Treads, Railing |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings. Seen in play (WebGPU, and WebGL2 at
night): generated houses with a flight up (facing 2) and flights down to the basement (facings 0, 1 and 3), a
resident walking up and down each; the stairwell from the floor above and from the ground over stairs down; the
colour studio.

## Rebuild

```
cargo art living            # the .glb files, the pack, check-glb
cargo art living --preview  # previews/<id>.png, <id>@<clip>.png
```
