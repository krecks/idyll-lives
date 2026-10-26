# Kids — notes

`kids.py` builds the twenty kids' things of docs/objects/kids.md into `web/public/assets/models/kids/` and their entries
in `packs/kids/manifest.json`. Built 2026-10-14; they sat in Validate › Baby And Beds, Toys and Kids
Study until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
so the baby, the sleepers and the sitters land where the game puts them: the crib's mattress at 0.56 m
(`BEDS.crib`), the kids' bed's at 0.44 (`BEDS.kidsBed`), the car bed's at the default 0.5, the rocking horse's
saddle at 0.56 (`SEATS.rockingHorse`), the desk chair's seat at 0.365 (`SEATS.kidsDesk`), the kids' chair's at 0.31
and the bean bag's at 0.24 (content `seat`), the kids' table's top at 0.5 (`TOPS`).

Flat things on a slope (the tent's door and flap, the flags, the chalk drawing, the painting, the picture books)
are `_slab`s: an outline on a face's frame (`_plane`, `_board`), a few millimetres thick.

**Detail pass (2026-10-22)**, with the forms and the kit, every model's size, centre, parts, clips, draws and heights
kept: mattresses, pillows, duvets and cushions are soft boxes (piped, puffed, the covers rumpled), frames and rails
swept in one piece (`kit.fillet` for bent tube), shaped boards extruded with their cut-outs, turned parts revolved,
round bodies lofted. Small soft boxes take `round_segments=1` (a default one is about 300 triangles). The bean bag's
creases move its vertices up and down only (`_rumple`): `G.jitter` sideways moved its bounds by 3 mm between a
`--only` build and a family build, and the centre check allows 2 mm.

## KD1: the crib, the beds and the high chair

- **Crib** (2 × 1): an oblong cot, a 1.18 × 0.58 m mattress (a 60 × 120 cot's) running across the left of its tiles,
  its back on the wall's face, and a 0.62 m changing chest on the right (2026-10-21, review M-kids-01: the cot was
  0.80 m square). The baby lies along it (`beds.crib` `across`) at slot 0, its head to the left, the blanket folded
  at its feet; the carers reach in over the front rail, 0.25 m from the baby (`characters.ts` `CUDDLE.rail`). Two moving
  parts: the front rail (`side_down`, slides down 25 cm for feed, change and cuddle, `uses`) and the mobile over
  the baby (`mobile`, a turn every 6 s while a grown-up plays with the baby, `loops.play`: a toy for an awake
  baby, not turning all night; the crib's uses can't tell a sleeping baby from an awake one, and playing is the
  one that means awake). The mobile hangs at the back,
  clear of the arms of a grown-up lifting the baby out over the front rail.
- **Kids' bed** and **race car bed** (1 × 2): as bedroom.py's beds, the duvet's top third (`duvet_flap`) folds back
  toward the foot while a resident sleeps or naps (`turn_down`), gathered to a fifth of its length half way over so
  it rolls as cloth (2026-10-21). Both are in `BLANKET_BEDS`, so the game lays its
  quilt over the sleeper. The car's body, wheels and spoiler are
  still.
- **High chair**: its legs splay to 0.54 m each way at the floor, so it can't tip (2026-10-21, review M-kids-04:
  0.44 × 0.47 m); the seat stays at 0.61 m. Babies sit in it since G-66 (*Sit in the high chair*, `sit`, put in by
  a carer). Its tray is a part of its own: `tray_out` slides it 16 cm forward along the arms in 0.5 s as the baby is
  put in, holds it there 1 s and slides it back in front of them; it slides out again for 1.2 s as they are taken
  out (`out`) (2026-10-26, review I-kids-01). Its mat is a plain satin (`tray_mat`, the cushions' blue), not the
  cushions' fabric, so the tray is one draw: 3 in all (the body's plain and fabric, the tray).

## KD2: the toys

| Model | Clip | Kind |
|---|---|---|
| Toy box, toy chest | `lid_open`: the lid swings up 100°, toys heaped inside | `uses.play`, open while the child plays |
| Doll house | `front_open`: the front wall swings 120° on its side hinge, two furnished rooms behind | `uses.dolls` |
| Rocking horse | `rock`: ±12° about the rockers' centre, 1 m up, in 1.2 s; the rider rides it | `loops.ride`, `ride` |
| Play tent | `flap_open`: the door flap rolls up to its top edge (squashed along the canvas) | `uses.play` |
| Ball pit | `bounce`: six balls on top rise and fall (`squash`, `back`) | `loops.jump` |
| Train table | `train`: the engine and two wagons go round the ring of track in 6 s (`spin`) | `loops.trains` |
| Play mat | `blocks`: a tower of three blocks grows up from the mat | `uses.blocks` |
| Giant teddy bear | `squish`: the bear squashes 10 % and bulges 6 % | `uses.hug` |
| Kids' bean bag | `squash`: the bag flattens 15 % | `uses.flop` |

The train table is 1.24 m long, centred on its 2 × 1 tiles (2026-10-21, review M-kids-02: it filled them, 1.8 m), the
town at its left. Its track is a circle (the old part's is an oval), so the train can `spin` round its middle.

## KD3: the kids' study and art

- **Kids' desk**: the desk and its chair (one model, as the old part). A pencil drawer under the top slides out
  toward the child (`drawer_open`, `hold` 2 s) for homework and drawing: they take a pencil out and shut it (left
  open for a 45-minute use, it would read as forgotten). The toy box's and chest's lids, by contrast, stay open
  while the child plays, the toys being out.
- **Kids' table** and **kids' chair**: static, a set in blue and white.
- **Picture-book shelf** (against the wall): three sloped tiers of face-out books; the middle one tips out 20°
  (`book_out`) for read.
- **Chalkboard easel**: a sun drawn already; a house in the grass draws itself from left to right (`drawing`,
  `_draw_on`) for draw.
- **Kids' easel**: a painting (sky, sun, a flower, grass) paints itself on from the left (`painting`) for paint.
- Both pictures grow along x from their left edge (`_draw_on`); at rest they're a speck at that edge, their
  height and depth coming back in the clip's first frames. Squeezed along x only (until 2026-10-16) they
  showed as a hairline of colours down the board.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts | What it has (2026-10-22) |
|---|---|---|---|---|---|
| `model.crib` | 1.94 × 1.01 × 0.69 m | 7,344 | 3 | Frame, Bedding, Mobile | turned finials, round spindles, solid arched ends with a heart and a star cut through, a piped mattress, a folded blanket; the mobile's arm bent from one tube on a clamp, its arms curving to a star, a heart, a moon and a ball facing out; the chest's drawers shaker fronts with turned knobs, a contoured changing pad between bolsters, a guard rail in one piece |
| `model.kidsBed` | 0.97 × 0.76 × 1.88 m | 5,922 | 3 | Frame, Bedding | turned bun feet, a rounded rim round the base, two drawers under it (cut-out grips), a piped mattress, an arched headboard with a moon and two stars cut through it, bunting draped on a string, a guard rail bent from one tube, a soft pillow, a sitting teddy, a soft rumpled duvet |
| `model.kidsBedCar` | 0.99 × 0.78 × 1.92 m | 6,776 | 3 | Car body, Bedding | sides extruded from the car's profile (nose, sill, tail, arches over the wheels), bonnet and tail between them, revolved tyres and dished hubs, headlights in rims, a barred grille, bonnet stripes, side stripes and a number 7 on each roundel, an aerofoil wing on struts between end plates; mattress, pillow, duvet soft |
| `model.highChair` | 0.53 × 0.94 × 0.55 m | 2,926 | 3 | Frame, Bedding | tapered legs on rubber feet, stretchers and a footrest bar, a moulded seat, a piped cushion and back cushion, a rounded back, rolled arms bent in one piece, a harness and buckle; the tray rounded, rimmed, a mat, a bowl and a spoon |
| `model.toyBox` | 0.82 × 0.51 × 0.80 m | 3,360 | 2 | Body, Lid | walls in one piece with rounded corners and hand holds cut through the ends, a rim along their top, a star, a circle and a triangle set in framed tiles; the lid with a finger notch, a raised panel and a piano hinge; inside a banded ball, lettered blocks and a teddy; lettered blocks on the floor |
| `model.toyChest` | 0.85 × 0.53 × 0.51 m | 3,596 | 2 | Body, Trim | bun feet, plank grooves, bands and straps, iron corner plates, bail handles on plates; the lid domed, straps over the dome, a lock plate with a keyhole; toys inside |
| `model.dollHouse` | 0.84 × 0.91 × 0.45 m | 3,808 | 2 | Walls, Roof | courses of shingles on the roof, a ridge, a chimney, a round gable window; inside a sofa, a round table and lamp, a staircase, a bed with a headboard, a cupboard, a framed picture; the front with corner boards, a panelled door under a porch, framed windows with sills, glazing bars and louvred shutters, a flower box |
| `model.rockingHorse` | 0.29 × 0.83 × 0.87 m | 3,924 | 3 | Body, Trim | bow rockers curling up at their ends, a lofted barrel body, a swept neck and head with a muzzle, ears, eyes and nostrils, a mane and a tail (jittered), tapered legs on hooves, a saddle with pommel, cantle and flaps, stirrups on leathers, a bridle and reins, turned grips |
| `model.playTent` | 0.95 × 1.57 × 0.83 m | 2,212 | 3 | Canvas, Poles | the canvas lofted, sagging between the poles (taut on the door's face), stopping short of the top where the tapered poles cross, lashed with rope; a hem, a padded round mat with two cushions, ties on the rolled door, bunting on a string |
| `model.ballPit` | 0.93 × 0.47 × 0.93 m | 3,868 | 2 | Pit, Rim | a foam wall in twelve padded channels, a roll round the top puffed in the same channels, dots on the pads |
| `model.trainTable` | 1.24 × 0.68 × 0.84 m | 5,018 | 3 | Table, Train | tapered round legs, a lip round the board, a shelf with two bins of toys; a tunnel with stone portals, a station with a platform, a pitched roof and a signal, a road with its line, a house, a pond ringed with stones, two-tier trees; a wooden engine (boiler, smokebox, chimney, dome, cab, buffer, six wheels) and two wagons with wheels and loads |
| `model.playMat` | 0.92 × 0.20 × 0.92 m | 2,252 | 3 | Mat | four foam tiles interlocking with jigsaw knobs, a star, a heart, a moon and a circle pressed on them; lettered blocks, a banded ball |
| `model.teddyBear` | 0.63 × 0.91 × 0.57 m | 3,192 | 2 | Fur, Bow | a lofted pear of a body and a round head, swept arms and legs, paw and foot pads, ears with insides, a stitched mouth, a bow tied in two loops with tails |
| `model.beanBagKids` | 0.62 × 0.53 × 0.65 m | 724 | 1 | Bag | one lofted sack leaning into its back, a seat pressed into its front (at the seat's 0.24 m: the gate's 11.8 cm warning is gone), creased, a handle loop and a label |
| `model.kidsDesk` | 0.76 × 0.69 × 0.82 m | 2,292 | 3 | Frame, Top, Chair | tapered legs, stretchers and a foot bar, a rounded top with an upstand, a drawing, a pot of crayons; the chair with a rounded seat, posts with caps and a curved back; the drawer with a turned knob, crayons and an eraser |
| `model.kidsTable` | 0.64 × 0.50 × 0.64 m | 1,564 | 1 | Frame, Top | a rounded top with a band round its edge, a rounded apron, turned legs on bun feet |
| `model.kidsChair` | 0.32 × 0.57 × 0.32 m | 1,080 | 1 | Frame, Seat | a rounded seat, tapered legs, the back legs running up into posts, a back with a heart cut through it, rungs |
| `model.kidsBookshelf` | 0.77 × 0.93 × 0.40 m | 2,132 | 2 | Frame, Top | curved sides with a star cut through each, a rounded top, rounded lips; each book two boards round white pages, a spine, a picture and a title band |
| `model.kidsChalkboard` | 0.66 × 0.96 × 0.53 m | 1,748 | 3 | Frame, Board | rounded frames whose stiles run down as feet, a hinge bar with turned ends, cords between the frames, a tray with ends, chalk and a duster |
| `model.kidsEasel` | 0.66 × 1.21 × 0.63 m | 2,180 | 3 | Easel | rounded battens for legs, boards behind the paper, trays with lips, spring clips, folding stays, a bar with turned ends, open paint pots, a jar of brushes |

`check-glb` (2026-10-22): 20 models, 0 failed. Two warnings were there before the detail pass and are left: the
crib's (the gate measures the mattress at x 0.50, over the changing table: the baby lies at x -0.5) and the high
chair's (a seat at x -0.25, where the chair has none). The toy box and the chest had the lid drawn open; they're shut
now (2026-10-14), as the models at rest. The doll house stood 5 cm off the wall and the picture-book shelf 3 cm
into it; all stand on the wall face now. The crib, the two beds, the ball pit and the play mat reached 3–7 cm into
the wall; their backs are on the face now (2026-10-15). The pit
and the mat stand 3–3.5 cm forward of the middle (`_place(forward=)`; the pit's rim 1 cm smaller, so its front
stays on the tile); the crib is built in place, its back on the wall's face. The beds' mattresses stay where the game lays children: the kids'
bed's headboard is 4 cm thick (was 6), the car's tail and spoiler are shorter from the back. The play tent stands free, centred. Khronos validator: 0 errors, 0 warnings each.

The ball pit's colour parts are Pit and Rim, not "Balls": the balls are five colours, and one slot would recolour a
fifth of them.

## Rebuild

```
cargo art kids            # the twenty .glb, the pack, check-glb
cargo art kids --preview  # previews/kids/<id>.png, <id>@<clip>.png
```
