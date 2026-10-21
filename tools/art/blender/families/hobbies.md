# Hobbies — notes

`hobbies.py` builds the nineteen hobby models (docs/objects/hobbies.md, H1–H3) into `web/public/assets/models/hobbies/`
and their entries in `packs/hobbies/manifest.json`. Built 2026-10-14;
they sat in Validate › Music, Games and Crafts until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
`Model`), so seats and stools stay where the game seats residents. The dartboard hangs at its real height
(`floor=False`).

## Music (H1)

- **Piano**: an upright with its back on the wall's face and its bench in front; the pianist faces the case
  (content `"seat": {"height": 0.47, "forward": 0.227, "facing": "back"}`). It is built as the old part
  stood (keys to the back) and turned round (`_turn_round`), so its clips open the other way; the old part
  was turned the same. Two
  parts share one clip, `fallboard`, played by `play` until the use ends: the fallboard lifts 90° off the keys
  to stand against the case, and the music desk, hinged at its top, swings its foot out 18° so the sheet leans
  back.
- **Ukulele, electric and acoustic guitars**: each instrument leans on a black A-frame stand (the electric
  with a practice amp behind). The instrument is a part of its own (`ukulele`, `guitar`), built upright with
  its foot at its origin; its lean on the stand is the node's turn (`_lean`), not baked into the mesh. While
  it's played, `off_stand` hides it (squashed to 0.001 in 0.2 s; `uses: {music: off_stand}`) and the game
  draws the same part in the player's hands (content `"held": {"part": "guitar", "grip": "guitar"}`,
  `characters.ts` `writeHeld`): the neck in the left hand, the body against the belly, strummed. Each costs
  one draw call more for it (3). The guitars are their real lengths (2026-10-21, review M-hobbies-03: a quarter too
  long, on stands as tall as a child): a 1.05 m dreadnought (a 0.51 m body, a 0.645 m scale, the neck meeting the
  body at the 14th fret) and a 0.9 m solid body (a 0.648 m scale), their feet in the cradle 0.12 m up and the necks
  in the yoke at 0.71–0.79 m; the stands 1.16 and 1.0 m high with the guitar. `characters.ts` `GUITAR.long` is 0.9 m,
  so both are held as guitars.
- **Digital piano** (2 × 1): still. A console of 88 full-size keys (52 white at 2.35 cm, A0 at the player's left, game -x as built),
  their tops at 0.745 m as the upright's, on two side panels with a pedal board, a control strip and a music rest, an
  X-legged bench at 0.48 m; built facing the front and turned round (`_turn_round`) to stand with its back on the
  wall like the upright, the player facing it (content `seat` 0.48 m, `forward` 0.18, `facing: back`). 2026-10-21,
  review M-hobbies-04: a three-octave keyboard on an X-stand on one tile, the keys at 0.845 m.
- **Drum kit** (2 × 2): a real five-piece spread out (2026-10-21, review M-hobbies-05: packed onto one tile): a 22"
  kick to the front with the toms on a mount on it, the snare and hi-hat to the drummer's left (game -x: facing
  the front, +z, their left is -x), a 16" floor tom to the right, the crash and ride on boom stands, the throne 0.4 m behind the middle
  (content `seat` 0.52 m, `forward` -0.4). The crash and ride cymbals are parts of their own, origins on their stands, and rock in one
  loop, `cymbals` (`drum`, `practise`), the crash about X and the ride about Y. The hi-hat is still. The
  throne is satin (a fabric seat would cost a fourth draw call).

## Games (H2)

- **Chess table**: a 0.64 × 0.48 m top, room beside the board for the taken pieces (2026-10-21, review M-hobbies-02:
  0.46 × 0.40, the board filling it); still one chair (a partner across the board needs a second place: G-65). A black
  pawn is its own part and slides one square (`move`) for `chess`. Its chair is coloured on
  its own ("Chair"); the board's rim is the table's wood.
- **Pool table**: the cloth at 0.75 m, the rails' top at 0.785 (a table is 0.74–0.79 to the rail; 2026-10-21, review
  M-hobbies-06: the rails were at 0.82; `clips/games.ts` `BED` follows). The triangle is lifted off the fifteen racked balls, carried over the foot rail and hung up by its
  apex on the foot end (`rack_off`, `G.path`, 1.4 s; 2026-10-21: it hung 25 cm over the balls) for `pool` and
  `trickShots`. The cue lies on the cloth, as it now does (it stuck out to 1.69 m).
- **Foosball**: the eight rods of a real table, 15 cm apart, from the left goal blue's goalie and defence, red's attack,
  the two teams' five midfielders, blue's attack, red's defence and goalie; blue's handles to the front, where the
  players stand (one at the goalie and defence, one at the midfield and attack: `clips/games.ts` `FOOS_LANES`, a
  clip for each, `foosClip`; squared to the table at their lane, `characters.ts` `STEP_IN` `square`), red's short at the back so the table stands against a wall (2026-10-21, review
  M-hobbies-07: four rods, every handle on the front). Blue's defence and attack rods kick ±55° in a loop, `rods`.
  Four turning rods would be six draw calls.
- **Dartboard**: twenty segments and the rings are one flat mesh (`_dart_face`). Three darts, one part, grow
  out of the board (`darts`). Hidden at rest, they leave the model 7 cm deep and centred on the board, where
  the old part (darts in) was 19 cm.
  Their origin is 2 cm inside the board, so at rest they flatten out of sight (on the face, the flights showed
  as crosses).
- **Arcade cabinet**: the sides are prisms cut to the classic profile (`_prism`). The screen and the marquee
  glow at night. The joystick waggles in a loop, `joystick`.
- **Jigsaw table**: half the picture is done; a patch beside it spreads across (`pieces`, `G.grow` along X).

## Crafts and science (H3)

- **Easel**: the canvas is blank at rest; the painting (sky, grass, trees, a stream, the sun) grows across it
  from its left edge (`painting`).
- **Workbench**: a 5 cm butcher-block top at 0.90 m, a standing height, on an open frame of square posts with rails
  and a slatted shelf (a crate of offcuts, a paint can), the back posts carrying a hardboard tool board on the wall
  with a saw, a square, screwdrivers, a spanner, pliers, a mallet and a tape, a shelf of jars along its top
  (2026-10-21, review M-hobbies-01: a 0.78 m top on two steel drawer pedestals, a teacher's desk). Its back on the
  wall face (the old part's went 2.5 cm into the wall). A bench vice holds a plank on edge; its front jaw, screw and handle slide 4 cm shut (`vice`).
- **Telescope**: the tube, with dew shield, finder and eyepiece, tilts up 35° more about the mount's axle
  (`aim`), and back down when the use ends.
- **Sewing table**: the machine's needle and head on the sewer's left (game -x: facing the front, +z, their left is
  -x) and the pillar and handwheel on their right, as on every domestic machine (checked in play 2026-10-21: review
  M-hobbies-08 read it as mirrored). The presser foot and needle lower 1.6 cm (`presser`) and the handwheel turns in a loop
  (`handwheel`) for `sew` and `mend`.
- **Chemistry bench**: glass flasks and test tubes (one see-through draw call). The middle flask's liquid
  rises from its floor (`bubble`). Its origin is 1 cm inside the bench top, so at rest it flattens out of sight.
- **Pottery wheel**: a narrow, low motor body on four feet under a 0.48 m splash pan, the pedal by the right foot (game +x);
  the potter sits with the knees either side of the pan (`characters.ts` `activityPose`, `tinker`), the shins either
  side of the body (2026-10-21, review I-hobbies-10: the knees went into a 0.66 m drum). The pot rises from a squat lump, 30 % of its height (`clay`), and the wheel head turns in
  a loop (`wheel`) for `throw`. At rest it is lower than the old part's full pot.
- **Craft table**: the scrapbook's cover swings 176° open over its spine (`book_open`), onto the photos of its
  first page.

**Seated uses wait for the hands** (2026-10-21, review G-17): the piano's fallboard, the cymbals, the pawn, the jigsaw
patch, the presser and handwheel, the pot and wheel and the scrapbook start 0.9 s into the use (`SIT`, `delay`),
once the resident has sat down (`sitDown`, about 1 s), as the weight bench's bar does. What went round while used
(the cymbals, the handwheel, the wheel) is now a use's clip played `every` 0.9 s from then (a hair under its 1 s
round), since a loop can't wait.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.piano` | 1.54 × 1.48 × 0.83 m | 5,072 | 3 | Body, Trim |
| `model.ukulele` | 0.35 × 0.84 × 0.26 m | 2,124 | 3 | Body & neck, Top, Stand |
| `model.guitarElectric` | 0.46 × 1.00 × 0.58 m | 3,184 | 3 | Guitar, Amp |
| `model.keyboardDigital` | 1.39 × 1.05 × 0.77 m | 3,584 | 2 | Body, Trim |
| `model.drumKit` | 1.73 × 1.24 × 1.17 m | 5,940 | 3 | Shells, Hardware |
| `model.guitarAcoustic` | 0.40 × 1.16 × 0.40 m | 2,832 | 3 | Body, Top, Stand |
| `model.chessTable` | 0.64 × 0.92 × 0.89 m | 4,400 | 3 | Table, Chair |
| `model.poolTable` | 2.50 × 0.81 × 1.40 m | 5,268 | 3 | Frame, Cloth |
| `model.foosball` | 1.42 × 0.95 × 0.93 m | 7,224 | 3 | Table, Trim |
| `model.dartboard` | 0.62 × 0.62 × 0.07 m | 1,844 | 2 | Board, Cabinet |
| `model.arcadeCabinet` | 0.67 × 1.82 × 0.87 m | 1,532 | 2 | Cabinet, Art |
| `model.puzzleTable` | 0.87 × 0.79 × 0.89 m | 2,256 | 3 | Table, Stool |
| `model.easel` | 0.70 × 1.75 × 0.76 m | 2,004 | 3 | Wood, Metal |
| `model.workbench` | 1.62 × 1.69 × 0.67 m | 5,884 | 3 | Top, Frame, Tool board |
| `model.telescope` | 0.66 × 1.52 × 0.91 m | 1,724 | 2 | Tube, Tripod |
| `model.sewingTable` | 0.90 × 1.08 × 0.89 m | 3,420 | 3 | Table, Machine |
| `model.chemistrySet` | 0.92 × 1.40 × 0.50 m | 3,540 | 3 | Bench, Glassware |
| `model.potteryWheel` | 0.60 × 0.54 × 0.88 m | 2,888 | 3 | Wheel, Stool |
| `model.craftTable` | 1.70 × 0.90 × 0.89 m | 2,460 | 3 | Wood, Metal |

`check-glb`: 19 models, 0 failed; warnings: the dartboard floats (a wall thing), the chess table's 4,400 triangles
(over the 4,000 aimed for at 0.92 m, under the 8,000 most), and the piano's seat (content 0.47 m, the bench 0.49:
M-hobbies-10). Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art hobbies            # the nineteen .glb, the pack, check-glb
cargo art hobbies --preview  # previews/hobbies/<id>.png, <id>@<clip>.png
```
