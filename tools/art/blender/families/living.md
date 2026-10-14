# Living — notes

`living.py` builds the living room's models into `web/public/assets/models/living/` and their entries in
`packs/living/manifest.json`: the fold-down wall bed (`model.wallBed`), the sofa bed (`model.sofaBed`) and the ten
L1 sofas and sectionals. The sofas replace the primitive-part placeholders of `furniture_prims.py` (the classic
sofa), `catalog.py` and `catalog2.py`, which stay as the fallback. Built 2026-10-14; they sit in Validate › Sofas
until checked in play.

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

## Rebuild

```
python3 -B tools/art/blender/build.py living            # the .glb files, the pack, check_glb
python3 -B tools/art/blender/build.py living --preview  # previews/<id>.png, <id>@<clip>.png
```
