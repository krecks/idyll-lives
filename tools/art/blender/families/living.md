# Living — notes

`living.py` builds the living room's models into `web/public/assets/models/living/` and their entries in
`packs/living/manifest.json`: the fold-down wall bed (`model.wallBed`), the sofa bed (`model.sofaBed`), the ten
L1 sofas and sectionals and the eleven L2 armchairs and small seats. These replace the primitive-part placeholders
of `furniture_prims.py` (the classic sofa, the armchair), `catalog.py` and `catalog2.py`, which stay as the
fallback. Built 2026-10-14; they sit in Validate › Sofas and Validate › Armchairs until checked in play.

## The sofas and sectionals (L1)

Each copies its placeholder where it stands (`F`, game axes), its back on the wall's face, and is settled in place
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
| `model.sectionalCorner` | 1.00 × 0.82 × 0.96 m | 1,412 | 3 | Upholstery, Cushions, Legs, Pillow |
| `model.chaise` (1 × 2) | 0.96 × 0.82 × 1.90 m | 1,144 | 3 | Upholstery, Cushions, Legs, Throw |

`check_glb.py`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Seen in play (WebGPU and
WebGL, day and night, all four facings across the set): residents sit on each, and `sink_in` plays and holds.

- The sectional modules are 1.00 m wide, so they butt up in a row; the arms are where the placeholders have them
  (`sectionalArmLeft`'s on +x, the sitter's left).
- Where a placeholder had no frame (the `sofa_run` sofas), there is no Frame colour part, as on the sofa bed.
- The pillows lean on the back cushions; the seat sinking leaves them where they are (1 cm into the cushion at
  rest, so the gap is small).

## The armchairs and small seats (L2)

Built as the sofas are, from their placeholders (`furniture_prims.py` `armchair_bergere`, `catalog.py`,
`catalog2.py`). Most have a still part and a `seat` that sinks 2 cm (`sink_in`) for `sit` (and `read`).

- **Recliner** (`recline`, `footrest_up`, for relax and doze): a skirted base and arms (still); the seat cushion and
  the back as one part (`seat`) turning back 25° about the sitter's hips (content seat 0.44 m, forward 0), with
  `ride` on it, so the resident reclines with it; the padded footrest, folded flat against the front of the base at
  rest, swings up 80°. It stands in the back 0.70 m of its tile (`check_glb.py` warns it's 7 cm off the placeholder's
  middle and shallower than the placeholder, which was drawn with its footrest up).
- **Rocking chair** (`rock`, a loop for relax and read, `ride`): the whole chair rocks ±6° in 1.4 s about the
  rockers' centre of curvature (1 m above the floor), so the rockers roll on the floor; the rockers are flat bent
  strips.
- **Bean bag** (`squash` for flop): the bag flattens to 85 % and spreads 6 %.
- **Floor cushion**: all of it is the seat that sinks; **pouf** and **ottoman**: the buttoned top sinks.
- **Designer lounge chair**: still, as in the plan (moulded shells on a five-star base).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.armchair` (bergère) | 0.74 × 0.98 × 0.80 m | 4,496 | 3 | Upholstery, Cushion, Frame, Legs, Pillow |
| `model.armchairLounge` | 0.82 × 0.83 × 0.84 m | 1,224 | 3 | Upholstery, Cushions, Legs |
| `model.armchairLeather` | 0.73 × 0.95 × 0.85 m | 1,348 | 3 | Frame, Upholstery |
| `model.armchairDesign` | 0.76 × 0.96 × 0.85 m | 1,756 | 2 | Frame, Upholstery, Base |
| `model.armchairWingback` | 0.84 × 1.10 × 0.80 m | 1,788 | 3 | Upholstery, Cushion, Legs |
| `model.recliner` | 0.92 × 0.84 × 0.70 m | 1,520 | 3 | Upholstery, Cushions & footrest |
| `model.rockingChair` | 0.53 × 1.03 × 0.90 m | 1,148 | 2 | Wood, Cushion |
| `model.pouf` | 0.72 × 0.47 × 0.50 m | 1,032 | 3 | Leather, Buttons, Legs |
| `model.ottoman` | 0.62 × 0.45 × 0.62 m | 1,672 | 3 | Upholstery, Buttons, Feet |
| `model.floorCushion` | 0.62 × 0.14 × 0.62 m | 1,120 | 1 | Cushion, Piping, Button |
| `model.beanBag` | 0.84 × 0.70 × 0.84 m | 1,824 | 1 | Bag, Seat |

`check_glb.py`: 0 failed (warnings: the bergère's 4,496 triangles against a target of 4,000; the recliner's as
above). Khronos validator: 0 errors, 0 warnings each. Seen in play on WebGPU (all four facings across the set) and
WebGL at night: residents sit, recline, rock and flop on each, and each clip plays.

## Rebuild

```
python3 -B tools/art/blender/build.py living            # the .glb files, the pack, check_glb
python3 -B tools/art/blender/build.py living --preview  # previews/<id>.png, <id>@<clip>.png
```
