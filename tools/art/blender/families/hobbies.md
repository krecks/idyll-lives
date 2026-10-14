# Hobbies — notes

`hobbies.py` builds the nineteen hobby models (plan/08-hobbies.md, H1–H3) into `web/public/assets/models/hobbies/`
and their entries in `packs/hobbies/manifest.json`. They replace the primitive-part placeholders of
`furniture.py` (the base manifest), `catalog.py` and `catalog2.py`, which stay as the fallback. Built 2026-10-14;
they sit in Validate › Music, Games and Crafts until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
`Model`), so seats and stools stay where the game seats residents. The dartboard hangs at its real height
(`floor=False`).

## Music (H1)

- **Piano**: an upright with its back on the wall's face and its bench in front; the pianist faces the case
  (content `"seat": {"height": 0.47, "forward": 0.227, "facing": "back"}`). It is built as the old placeholder
  stood (keys to the back) and turned round (`_turn_round`), so its clips open the other way; the placeholder
  (`furniture.py`, `turned`) is turned the same. Two
  parts share one clip, `fallboard`, played by `play` until the use ends: the fallboard lifts 90° off the keys
  to stand against the case, and the music desk, hinged at its top, swings its foot out 18° so the sheet leans
  back.
- **Ukulele, electric and acoustic guitars**: each instrument leans on a black A-frame stand (the electric
  with a practice amp behind). The instrument is a part of its own, but has no clip: the music animation
  doesn't draw an instrument in the hands, so hiding it from the stand would only make it vanish
  (08-hobbies.md's condition). When the hands hold one, add `G.squash(part, 'off_stand', 0.001)`.
- **Digital piano**: still. A slim keyboard on an X-stand, an X-legged bench behind.
- **Drum kit**: the crash and ride cymbals are parts of their own, origins on their stands, and rock in one
  loop, `cymbals` (`drum`, `practise`), the crash about X and the ride about Y. The hi-hat is still. The
  throne is satin (a fabric seat would cost a fourth draw call).

## Games (H2)

- **Chess table**: a black pawn is its own part and slides one square (`move`) for `chess`.
- **Pool table**: the triangle lifts 25 cm off the fifteen racked balls (`rack_off`) for `pool` and
  `trickShots`. The cue lies on the cloth, so the model is 1.40 m deep where the placeholder's cue stuck out to
  1.69 m: `check_glb.py` warns of it.
- **Foosball**: four rods; the two outer ones (one each side, by where the players stand) kick ±55° in a loop,
  `rods`. Four turning rods would be six draw calls.
- **Dartboard**: twenty segments and the rings are one flat mesh (`_dart_face`). Three darts, one part, grow
  out of the board (`darts`). Hidden at rest, they leave the model 7 cm deep and centred on the board, where
  the placeholder (darts in) is 19 cm: `check_glb.py` warns of the depth and the centre.
  Their origin is 2 cm inside the board, so at rest they flatten out of sight (on the face, the flights showed
  as crosses).
- **Arcade cabinet**: the sides are prisms cut to the classic profile (`_prism`). The screen and the marquee
  glow at night. The joystick waggles in a loop, `joystick`.
- **Jigsaw table**: half the picture is done; a patch beside it spreads across (`pieces`, `G.grow` along X).

## Crafts and science (H3)

- **Easel**: the canvas is blank at rest; the painting (sky, grass, trees, a stream, the sun) grows across it
  from its left edge (`painting`).
- **Workbench**: its back on the wall face (the placeholder's went 2.5 cm into the wall). A bench vice holds a
  plank on edge; its front jaw, screw and handle slide 4 cm shut (`vice`).
- **Telescope**: the tube, with dew shield, finder and eyepiece, tilts up 35° more about the mount's axle
  (`aim`), and back down when the use ends.
- **Sewing table**: the presser foot and needle lower 1.6 cm (`presser`) and the handwheel turns in a loop
  (`handwheel`) for `sew` and `mend`.
- **Chemistry bench**: glass flasks and test tubes (one see-through draw call). The middle flask's liquid
  rises from its floor (`bubble`). Its origin is 1 cm inside the bench top, so at rest it flattens out of sight.
- **Pottery wheel**: the pot rises from a squat lump, 30 % of its height (`clay`), and the wheel head turns in
  a loop (`wheel`) for `throw`. At rest it is lower than the placeholder's full pot: `check_glb.py` warns of it.
- **Craft table**: the scrapbook's cover swings 176° open over its spine (`book_open`), onto the photos of its
  first page.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.piano` | 1.54 × 1.48 × 0.83 m | 5,072 | 3 | Body, Trim |
| `model.ukulele` | 0.35 × 0.84 × 0.26 m | 2,124 | 2 | Instrument, Stand |
| `model.guitarElectric` | 0.46 × 1.37 × 0.58 m | 3,256 | 2 | Guitar, Amp |
| `model.keyboardDigital` | 0.90 × 1.09 × 0.75 m | 2,420 | 2 | Body, Trim |
| `model.drumKit` | 0.98 × 1.09 × 0.89 m | 5,876 | 3 | Shells, Hardware |
| `model.guitarAcoustic` | 0.36 × 1.48 × 0.40 m | 2,856 | 2 | Guitar, Stand |
| `model.chessTable` | 0.46 × 0.92 × 0.85 m | 3,760 | 3 | Table, Trim |
| `model.poolTable` | 2.50 × 0.84 × 1.40 m | 5,268 | 3 | Frame, Cloth |
| `model.foosball` | 1.42 × 0.95 × 0.99 m | 7,928 | 3 | Table, Trim |
| `model.dartboard` | 0.62 × 0.62 × 0.07 m | 1,844 | 2 | Board, Cabinet |
| `model.arcadeCabinet` | 0.67 × 1.82 × 0.87 m | 1,532 | 2 | Cabinet, Art |
| `model.puzzleTable` | 0.87 × 0.79 × 0.89 m | 2,256 | 3 | Table, Trim |
| `model.easel` | 0.70 × 1.75 × 0.76 m | 2,004 | 3 | Wood, Metal |
| `model.workbench` | 1.62 × 1.00 × 0.67 m | 5,064 | 3 | Wood, Metal |
| `model.telescope` | 0.66 × 1.52 × 0.91 m | 1,724 | 2 | Tube, Tripod |
| `model.sewingTable` | 0.90 × 1.08 × 0.89 m | 3,420 | 3 | Table, Machine |
| `model.chemistrySet` | 0.92 × 1.40 × 0.50 m | 3,540 | 3 | Bench, Glassware |
| `model.potteryWheel` | 0.70 × 0.54 × 0.95 m | 2,836 | 3 | Wheel, Stool |
| `model.craftTable` | 1.70 × 0.90 × 0.89 m | 2,460 | 3 | Wood, Metal |

`check_glb.py`: 19 models, 0 failed; the warnings are the five above (the dartboard's three, the pool table's
depth, the pottery wheel's height). Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
python3 -B tools/art/blender/build.py hobbies            # the nineteen .glb, the pack, check_glb
python3 -B tools/art/blender/build.py hobbies --preview  # previews/<id>.png, <id>@<clip>.png
```
