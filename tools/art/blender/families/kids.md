# Kids — notes

`kids.py` builds the twenty kids' things of docs/objects/kids.md into `web/public/assets/models/kids/` and their entries
in `packs/kids/manifest.json`. Built 2026-10-14; they sit in Validate › Baby And Beds, Toys and Kids
Study until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
so the baby, the sleepers and the sitters land where the game puts them: the crib's mattress at 0.56 m
(`BEDS.crib`), the kids' bed's at 0.44 (`BEDS.kidsBed`), the car bed's at the default 0.5, the rocking horse's
saddle at 0.56 (`SEATS.rockingHorse`), the desk chair's seat at 0.365 (`SEATS.kidsDesk`), the kids' chair's at 0.31
and the bean bag's at 0.24 (content `seat`), the kids' table's top at 0.5 (`TOPS`).

Flat things on a slope (the tent's door and flap, the bunting, the chalk drawing, the painting, the picture books)
are `_slab`s: an outline on a face's frame (`_plane`, `_board`), a few millimetres thick. The rockers are a swept
`_tube`.

## KD1: the crib, the beds and the high chair

- **Crib** (2 × 1): the crib on the left half, where the baby lies, and a changing table on the right. Two moving
  parts: the front rail (`side_down`, slides down 25 cm for feed, change and cuddle, `uses`) and the mobile over
  the head end (`mobile`, a turn every 6 s while a grown-up plays with the baby, `loops.play`: a toy for an awake
  baby, not turning all night; the crib's uses can't tell a sleeping baby from an awake one, and playing is the
  one that means awake). The mobile hangs at the back,
  clear of the arms of a grown-up lifting the baby out over the front rail.
- **Kids' bed** and **race car bed** (1 × 2): as bedroom.py's beds, the duvet's top third (`duvet_flap`) folds back
  toward the foot while a resident sleeps or naps (`turn_down`). Both are in `BLANKET_BEDS`, so the game lays its
  quilt over the sleeper. The car's body, wheels and spoiler are
  still.
- **High chair**: static. Its tray is a part of its own (no clip yet), so a "tray slides out" clip can come with a
  use. Content can't give it one yet: a baby is only ever placed lying (`baby`, `care` are for a baby lying in an
  object), so feeding a baby in the chair needs the sim first.

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

The train table's track is a circle (the old part's is an oval), so the train can `spin` round its middle.

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

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.crib` | 1.86 × 1.01 × 0.93 m | 7,996 | 3 | Frame, Bedding, Mobile |
| `model.kidsBed` | 0.97 × 0.76 × 1.88 m | 2,692 | 3 | Frame, Bedding |
| `model.kidsBedCar` | 0.99 × 0.77 × 1.92 m | 3,744 | 3 | Car body, Bedding |
| `model.highChair` | 0.44 × 0.94 × 0.47 m | 1,384 | 2 | Frame, Bedding |
| `model.toyBox` | 0.82 × 0.51 × 0.80 m | 2,560 | 2 | Body, Lid |
| `model.toyChest` | 0.85 × 0.53 × 0.51 m | 3,780 | 2 | Body, Trim |
| `model.dollHouse` | 0.84 × 0.91 × 0.46 m | 3,236 | 2 | Walls, Roof |
| `model.rockingHorse` | 0.29 × 0.82 × 0.88 m | 3,528 | 3 | Body, Trim |
| `model.playTent` | 0.95 × 1.57 × 0.83 m | 386 | 3 | Canvas, Poles |
| `model.ballPit` | 0.93 × 0.47 × 0.93 m | 3,816 | 2 | Pit, Rim |
| `model.trainTable` | 1.80 × 0.68 × 0.84 m | 3,448 | 3 | Table, Train |
| `model.playMat` | 0.92 × 0.20 × 0.92 m | 1,680 | 3 | Mat |
| `model.teddyBear` | 0.64 × 0.91 × 0.57 m | 3,340 | 2 | Fur, Bow |
| `model.beanBagKids` | 0.62 × 0.53 × 0.64 m | 1,292 | 1 | Bag |
| `model.kidsDesk` | 0.76 × 0.69 × 0.82 m | 1,692 | 3 | Frame, Top, Chair |
| `model.kidsTable` | 0.64 × 0.50 × 0.64 m | 984 | 1 | Frame, Top |
| `model.kidsChair` | 0.32 × 0.57 × 0.32 m | 612 | 1 | Frame, Seat |
| `model.kidsBookshelf` | 0.77 × 0.93 × 0.40 m | 1,220 | 2 | Frame, Top |
| `model.kidsChalkboard` | 0.66 × 0.96 × 0.53 m | 916 | 3 | Frame, Board |
| `model.kidsEasel` | 0.66 × 1.21 × 0.63 m | 1,528 | 3 | Easel |

`check_glb.py`: 20 models, 0 failed, no warnings. The toy box and the chest had the lid drawn open; they're shut
now (2026-10-14), as the models at rest. The doll house stood 5 cm off the wall and the picture-book shelf 3 cm
into it; all stand on the wall face now. The crib, the two beds, the ball pit and the play mat reached 3–7 cm into
the wall; their backs are on the face now (2026-10-15). The crib, the pit
and the mat stand 3–3.5 cm forward of the middle (`_place(forward=)`; the crib's post caps and the pit's rim 1 cm
smaller, so their fronts stay on the tile). The beds' mattresses stay where the game lays children: the kids'
bed's headboard is 4 cm thick (was 6), the car's tail and spoiler are shorter from the back. The play tent stands free, centred. Khronos validator: 0 errors, 0 warnings each.

The ball pit's colour parts are Pit and Rim, not "Balls": the balls are five colours, and one slot would recolour a
fifth of them.

## Rebuild

```
python3 -B tools/art/blender/build.py kids            # the twenty .glb, the pack, check_glb
python3 -B tools/art/blender/build.py kids --preview  # previews/<id>.png, <id>@<clip>.png
```
