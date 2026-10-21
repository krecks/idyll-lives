# Living room

The sofas, armchairs, coffee tables, shelves, staircase, sofa bed and wall bed are built by
`tools/art/blender/families/living.py` (notes: `families/living.md`), the media by `families/media.py` (notes:
`families/media.md`). Every sofa's seat cushions are a part of their own that sinks while sat on (`sink_in`); the
recliner's seat and back tip back with the resident riding them (`recline`, `ride`) and the rocking chair rocks with
its rider (`rock`, E1); the coffee table's drawer opens for a magazine. The wall bed is the model for anything held
open (C1: content `leaves: open`, the clip held while the object is open), as the sofa bed and the futon
(`bedroom.md`) are.

Seats keep their height and depth (`seats` in `web/src/render/babylon/characters/furniture.json`, content `seat`); a seat that
moves while sat on keeps the sitting pose believable. Shelf and table tops are the content `surface` heights.

## L1. Sofas and sectionals (12)

- **Family:** `tools/art/blender/families/living.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sofa` Sofa | 2x1 · floor | relax:relax doze:nap | living / seating | Upholstery, Cushions, Frame, Legs, Pillow, Second pillow | Use | `sink_in`: the seat cushions settle 2 cm while sat on (relax, doze; T0 `squash`). Doze off, as the other sofas. |
| `loveseat` Loveseat | 2x1 · floor | relax:relax doze:nap | living / seating | Upholstery, Cushions, Legs, Pillow | Use | `sink_in`. |
| `sofaThree` Three-seat sofa | 3x1 · floor | relax:relax doze:nap | living / seating | Upholstery, Cushions, Legs, Pillows, Throw | Use | `sink_in`. Three residents at once (`slots: 3`, was 2). |
| `sofaDesign` Designer sofa | 2x1 · floor | relax:relax doze:nap | living / seating | Upholstery, Cushions, Frame, Legs | Use | `sink_in`. |
| `sofaThreeSeat` Three-seater sofa | 3x1 · floor | relax:relax doze:nap | living / seating | Upholstery, Cushions, Legs, Pillow, Second pillow | Use | `sink_in`. |
| `sofaBed` Sofa bed | 2x1 · floor | relax:relax doze:nap pullOut:fetch foldAway:fetch | living / seating | Upholstery, Cushions, Legs, Bedding, Pillow, Second pillow | Held open | `bed_out`: the mattress pulls out forward from under the seat for doze and "Pull out the bed" and stays out until "Fold the bed away" (relax only while folded); `beds` height 0.44 and `fwd` 0.6 (head on the seat; `cargo check-glb` measures the mattress pulled out at 0.44); pulling it out and folding it away the resident stands 1 m further back, clear of the mattress (`STAND_BACK`). |
| `wallBed` Fold-down wall bed | 2x3 · floor | sleep:sleep nap:nap foldDown:fetch foldUp:fetch | living / seating | Cabinet, Bed frame, Mattress, Pillows, Throw, Inside & shelves, Handle | Held open | `bed_down`: the bed turns down 90° on a hinge at the cabinet's foot to fill the footprint in front, for sleep, nap and "Fold down", and stays down while open; "Fold up" closes it. In `BLANKET_BEDS`; `BEDS` height 0.46. |
| `sectionalMiddle` Sectional: middle seat | 1x1 · floor | relax:relax doze:nap | living / sectionals | Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sectionalArmLeft` Sectional: left arm | 1x1 · floor | relax:relax doze:nap | living / sectionals | Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sectionalArmRight` Sectional: right arm | 1x1 · floor | relax:relax doze:nap | living / sectionals | Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sectionalCorner` Sectional: corner | 1x1 · floor | relax:relax doze:nap | living / sectionals | Upholstery, Cushions, Legs, Pillow | Use | `sink_in`. |
| `chaise` Sectional: chaise | 1x2 · floor | relax:relax doze:nap | living / sectionals | Upholstery, Cushions, Legs, Throw | Use | `sink_in`. |

**In play** (signed off 2026-10-17):
- Each sofa and sectional sat on in all four facings, by day and at night, on WebGPU and WebGL2: `sink_in` plays and
  holds while sat on. The pillows lean on the back cushions and stay put as the seat sinks.
- The sofa bed's mattress pulls out and folds away; for both the resident stands half a metre past the mattress's
  foot and reaches to it, clear of it throughout (checked in all four facings, from the side and above). Doze lies
  the resident on the pulled-out bed, legs on the mattress.
- The wall bed folds down and up, and is slept in under the game's blanket; the last one up folds it away. Its
  lying pose was checked with the other beds (heels and buttocks about 4 cm into the mattress, legs on it).
- Where the old parts had no frame (the `sofa_run` sofas, the sofa bed), there is no Frame colour part.
- Seated at every life stage (G-20, 2026-10-21): the feet are on the floor (the runtime's leg fit), and on these deep
  seats the hips sit forward until the knees clear the cushion's front (`furniture.json` `seats` `front`, measured by
  `cargo check-glb`): a child at the front edge with the shins and feet showing, a grown-up a few centimetres
  forward on the loveseat and the three-seat sofa; no one's shins end inside the seat.

## L2. Armchairs and small seats (11)

- **Family:** `tools/art/blender/families/living.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `armchair` Armchair | 1x1 · floor | sit:sit read:read | living / seating | Upholstery, Cushion, Frame, Legs, Pillow | Use | `sink_in`. Read a book sitting, as the leather and wingback chairs. |
| `armchairLounge` Lounge chair | 1x1 · floor | sit:sit | living / seating | Upholstery, Cushions, Legs | Use | `sink_in`. |
| `armchairLeather` Leather armchair | 1x1 · floor | sit:sit read:read | living / seating | Frame, Upholstery | Use | `sink_in`. |
| `armchairDesign` Executive lounge chair | 1x1 · floor | relax:relax doze:nap | living / seating | Frame, Upholstery, Base | Static | Static (moulded shell). Renamed from "Designer lounge chair", its text now the model's (chrome five-star base, no bent wood). |
| `armchairWingback` Wingback armchair | 1x1 · floor | sit:sit read:read | living / seating | Upholstery, Cushion, Legs | Use | `sink_in`. |
| `recliner` Recliner | 1x1 · floor | relax:relax doze:nap | living / seating | Upholstery, Cushions & footrest | Use | `recline`: the seat and back tip 25° and glide 8 cm forward (a wall-hugger's) and the footrest swings up 80° for relax and doze; the resident reclines with it (`ride`), the head on the head cushion, the legs out on the footrest (content `seat.legs: "out"`). 0.92 × 0.99 × 0.80 m, a high back with a head cushion (it was 0.84 × 0.70, the back below the shoulders, M-living-04). |
| `rockingChair` Rocking chair | 1x1 · floor | relax:relax read:read | living / seating | Wood, Cushion | Loop (E1) | `rock`: rocks ±6° while sat in (E1), the rider with it. |
| `pouf` Leather pouf | 1x1 · floor | sit:sit | living / seating | Leather, Buttons, Legs | Use | `sink_in`. A footstool: the feet go up on it from a seat in front (`footstools`). |
| `ottoman` Tufted ottoman | 1x1 · floor | sit:sit | living / seating | Upholstery, Buttons, Feet | Use | `sink_in`. A footstool: the feet go up on it from a seat in front (`footstools`). |
| `floorCushion` Floor cushion | 1x1 · floor | sit:sit | living / seating | Cushion, Piping, Button | Use | `sink_in`. |
| `beanBag` Bean bag | 1x1 · floor | flop:relax | living / seating | Bag, Seat | Use | `squash`: the bag flattens 15% for flop. |

**In play** (signed off 2026-10-16):
- Each standing free, sat on in all four facings, by day and at night, on WebGPU and WebGL2, Medium and Low, every
  colour part its own colour. `sink_in` plays and holds while sat on (sit, and read on the leather and wingback
  chairs) and shuts when the resident gets up.
- Feet up (G-69, 2026-10-21): a resident sitting in an armchair or on a sofa (or any seat but those below) with a
  pouf or an ottoman in front of them, within reach and nobody on it, puts their feet up on it: the legs out,
  the heels on its top, turned to it if it's a little to one side (characters.ts `footstoolFor`, `feetUp`;
  `docs/characters-format.md`). An armchair with one in front of it is sat in from beside it. The recliner
  doesn't pair: its own footrest comes up (`legs: out`), nor do the chaise, the tubs and the bikes.
- The recliner's back and seat tip back with the sitter riding them and the footrest swings up for relax and doze,
  the sitter's legs out along it (2026-10-21);
  the rocking chair rocks with its rider for relax and read and stops after; the bean bag flattens for flop (a
  sitting flop at seat 0.3 m, not a lie); the designer lounge chair is still.
- The leather armchair's "Read a book" is a sitting read (`"pose": "sit"`, as the wingback's), so the reader sits in
  the chair while its cushion sinks.
- G-20 and G-22 (2026-10-21): on the deep armchairs, the ottoman and the bean bag (`front` in
  `furniture.json` `seats`) a child sits forward, not kneeling, the feet down; the floor cushion is sat on
  cross-legged (`sitFloor`, any seat under 0.2 m: characters.ts `floorSit`), the knees out over it, not through the
  floor.
- Since the authenticity review (G-05): the lounge chair's seat is 0.44 (its cushion 0.45, sat a centimetre into), the
  rocking chair's 0.46 (its cushion); the bean bag's 0.30 stays, sunk into on purpose.

## L3. Coffee and side tables (9)

- **Family:** `tools/art/blender/families/living.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `coffeeTable` Coffee table | 2x1 · floor | magazine:read | living / tables | Wood, Knob | Use | `drawer_open`: the drawer slides out 30 cm with a magazine for magazine; `hold` 2.5 s. The magazine is read on a seat within 2 m (`seatNear`), else standing. |
| `coffeeTableGlass` Glass coffee table | 2x1 · floor | magazine:read | living / tables | Frame, Glass | Static | Static; glass top. The magazine is read on a seat within 2 m (`seatNear`), else standing. |
| `coffeeTableMarble` Round marble coffee table | 1x1 · floor | magazine:read | living / tables | Top, Base | Static | Static. The magazine is read on a seat within 2 m (`seatNear`), else standing. Renamed from "Marble side table" (a coffee-table top on a brass pedestal). |
| `sideTable` Side table | 1x1 · floor | — | living / tables | Wood, Knob | Static | Static. |
| `coffeeTableRound` Round coffee table | 1x1 · floor | — | living / tables | Top, Base | Static | Static. |
| `coffeeTableLong` Long coffee table | 2x1 · floor | — | living / tables | Top, Legs | Static | Static. |
| `nestingTables` Nesting tables | 1x1 · floor | — | living / tables | Top, Small table, Legs | Static | Static. (+use) the small table slides out: it is a part of its own (`small_table`). |
| `sideTableRound` Round side table | 1x1 · floor | — | living / tables | Top, Base | Static | Static. |
| `consoleTable` Console table | 2x1 · floor | — | living / tables | Top, Legs, Baskets | Static | Static. Its text no longer says slim (0.56 m deep). |

**In play** (signed off 2026-10-16):
- Each standing free in all four facings (the console table against a wall, and free at 180°). The coffee table's
  drawer slides out with its magazine for "Flip through a magazine" and shuts after its `hold`; the glass and marble
  tables' magazine use runs without a clip.
- The six plain tables looked at by day and night, WebGPU and Low WebGL, coloured; the nesting tables' small table
  takes its own colour.
- Known, not changed: a kids' chair facing a coffee table (top 0.11 m over its seat) gets the desk hands, hovering
  over the top; nothing sets that pairing up.

## L4. Media (8)

- **Family:** `tools/art/blender/families/media.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `tv` Television | 1x1 · floor | watch:watch | living / media | Body, Trim | Static | Static: a slim set on its stand on an oak unit, a soundbar; the screen flickers while watched (E8). 2 draws. Provides `screen` (the gaming recliner's marathon needs one). |
| `tvWall` Flat-screen TV | 2x1 · wall | watch:watch | living / media | Body, Trim | Static | Static: a thin panel on a bracket, a soundbar under it, on the wall face; the screen flickers while watched. 1 draw. Provides `screen`. |
| `tvRetro` Console TV | 1x1 · floor | watch:watch | living / media | Body, Trim | Use | `antenna`: the rabbit-ear antennae turn 20° and stretch 1.6× up for watch; the tube flickers while watched. Provides `screen`. Renamed from "Portable TV": a wooden console set on legs. |
| `homeCinema` Home cinema | 2x1 · floor | watch:watch film:watch | living / media | Body, Trim | Use | `screen_down`: the projector screen unrolls from its case on the wall for watch and film, and rolls up at the end; a short-throw projector on the unit; the screen flickers while down and watched. Provides `screen`. |
| `stereo` Stereo system | 1x1 · floor | listen:listen dance:dance | living / media | Body, Trim | Loop + repeating | `record`: the record turns on the turntable, 33 rpm, for listen and dance (a loop; the label's stripe shows the turn); the woofers `pulse` (`every` 0.45 s). The CD tray no longer moves (2026-10-21: it slid out for the music the description says is a record). 4 draws. Listened to from a seat within 4 m (`seatNear`), else standing; dance stands. |
| `boombox` Boombox | 1x1 · floor | listen:listen | living / media | Body, Trim | Use | `cassette`: the cassette door tips 32° open from its foot for listen and shuts; `hold` 1.2 s. Listened to from a seat within 4 m (`seatNear`), else standing. |
| `tvStandLow` Low TV bench | 2x1 · floor | watch:watch | living / media | Bench, TV | Use | `tv_up`: the TV rises 60 cm out of a lift hood at the bench's back for watch (0.8 m high at rest, its lid closing the hood), and sinks back at the end; it flickers while up and watched. Provides `screen`. |
| `fireplaceElectric` Electric fireplace | 2x1 · floor | relax:relax | living / media | Surround, Firebox | Use | `flames`: five tongues grow from the log bed for relax (gloss, not emissive; the light is the content's); the embers glow at night, the flames while lit. A floor cushion in front, and `characters/furniture.json` `seats` sits the resident on it cross-legged facing the fire. |

**In play** (signed off 2026-10-16):
- Against the north, west and east walls and free at 180° (the wall TV on window-free walls): watched, listened to,
  danced to and warmed up at. Every clip plays for its use and shuts after.
- Warm up by the fire (G-22, 2026-10-21): cross-legged on the hearth cushion (`sitFloor`), the hips at its back
  (`seats.fireplaceElectric` `fwd` 0.36), the knees short of the surround.
- Since 2026-10-21 (review I-living-08) the stereo's record turns while the music plays, where the CD tray slid
  out (`fixes/motions/g1718/views/stereo-t0.5.png`).
- Screens glow while on, brighter at night (the TV and wall TV while watched, the home cinema's screen while down,
  the bench TV while up), and the fire lights the room.
- *Watch TV* sits the resident on a free sofa or armchair within 4 m when there is one (`seatNear`), facing the way
  the seat does; the seat is reserved while they watch. Only a seat that looks at the screen from in front of it
  counts (`seatFacing: front`, 2026-10-21): not a sofa with its back to the TV, nor one behind it. With no seat
  like that free they stand in front of it as before.
- Residents in the same room (or within 6 m, `broadcast`) hear the TV too: they get 35% of its fun while they eat,
  drink coffee or do something else. The screen flickers whenever someone is watching, day or night (`prefers-reduced-motion`
  holds it steady); the TV's own sound loop plays as before.
- Known, not changed: the low bench's "TV" colour lands on the set's thin bezel and the remote (the lift hood is the
  bench's), so it shows little. The TV's and home cinema's "Body" is the unit's carcase and "Trim" its top and doors.

## L5. Shelves (9)

- **Family:** `tools/art/blender/families/living.py` (notes: `families/living.md`)

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bookshelf` Bookshelf | 1x1 · floor | read:read study:read homework:read | living / shelves | Frame, Back | Use | `book_out`: a red book on the third shelf tips out 25° for read, study and homework, and back at the end. Its back is on the wall face. Read, study and homework are done on a seat within 4 m (`seatNear`), else standing (Q-14). |
| `bookcaseLow` Low bookcase | 1x1 · floor | read:read | living / shelves | Frame, Back | Use | `book_out`. Top 0.9 (content `surface`); a back panel. Read on a seat within 4 m (`seatNear`), else standing. |
| `bookcaseWide` Library bookcase | 2x1 · floor | read:read study:read homework:read | living / shelves | Frame, Back | Use | `book_out`. Brass knobs on the cupboard doors. Read, study and homework on a seat within 4 m (`seatNear`), else standing. |
| `shelvingIndustrial` Industrial shelving | 2x1 · floor | — | living / shelves | Frame, Shelves | Static | Static. Top 1.83 (content `surface`); boards 53 cm deep, back on the wall face. |
| `bookcaseTall` Tall bookcase | 1x1 · floor | read:read study:read | living / shelves | Frame, Back | Use | `book_out`. A dark walnut back. Read and study on a seat within 4 m (`seatNear`), else standing. |
| `bookcaseCorner` Corner bookcase | 1x1 · floor | read:read | living / shelves | Frame, Back | Use | `book_out`. Quarter-round open end shelves to the wall. Read on a seat within 4 m (`seatNear`), else standing. |
| `cubeShelf` Cube shelf | 1x1 · floor | — | living / shelves | Frame, Back | Static | Static. Top 0.8 (content `surface`); 36 cm deep, back on the wall face. |
| `ladderShelf` Ladder shelf | 1x1 · floor | — | living / shelves | Frame, Shelves | Static | Static; no back, its top on the wall face. |
| `displayCabinet` Display cabinet | 1x1 · floor | — | living / shelves | Frame, Back | Static | Static; the brass-framed glass door is a part of its own (`door`, origin on its left hinge), still. (+use) the door swings. |

**In play** (signed off 2026-10-16):
- Against the north, west and east walls and free at 180° (the corner bookcase in a north-west corner at facings 0
  and 1, its quarter-round end into the room): the five bookcases' red book tips out for read (and study) and back at
  the end.
- The industrial shelving, cube shelf, ladder shelf and display cabinet looked at and coloured, by day and night,
  WebGPU and Low WebGL.
- Known, not changed: standing free, the ladder shelf leans on nothing (it is made to lean on a wall).

## L6. Stairs (1)

- **Family:** `tools/art/blender/families/living.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `stairs` Staircase | 1x4 · floor | — | living / — | Treads, Railing | Static | Static: 14 steps of 0.2 × 4/14 m, oak treads on white risers between closed strings, an iron railing each side. Placed by build mode and housegen, not the buy catalog. |

**In play** (signed off 2026-10-16):
- Bought in all four facings and coloured (treads, railing; by day and night, WebGPU and Low WebGL).
- Climbed in generated houses (the check house has no floor above or below): residents walk up flights at facings 0
  and 2 to the floor above and back down, and down basement flights at facings 2 and 3 and back up; at facing 1 one
  started down and stopped on the flight, where her goal tile was. The stairwells, with their railings, seen from the
  floor above.
