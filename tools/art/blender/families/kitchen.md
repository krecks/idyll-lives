# Kitchen — notes

`kitchen.py` builds the kitchen models into `web/public/assets/models/kitchen/` and their entries in
`packs/kitchen/manifest.json`: the **fridge** (`model.fridge`, `fridge.glb`) and the two **sinks**
(`model.farmhouseSink`, `farmhouseSink.glb`; `model.counterSinkDouble`, `counterSinkDouble.glb`). They
replace the primitive-part placeholders, which stay as the fallback. The fridge was first made as an
evaluation on 2026-10-13; on 2026-10-14 the flat build and the doors build (`kitchen_doors.py`,
`fridge.doors.glb`) became this one build, and the two sinks were added.

## The fridge

A top-freezer fridge: the **top compartment is the freezer** (frozen food), the **bottom compartment is
the fridge** (normal food), separated by a real divider.

- 0.740 × 1.800 × 0.722 m (w × h × d): exactly the placeholder's bounds, built in its numbers (`F`, a
  `GameFrame` shifted 9.4 cm back to centre the placeholder's back-aligned box on its tile)
- 4,286 triangles, 3 nodes (`cabinet`, `door_freezer`, `door_fridge`), 3 draw calls (one per node), 14
  materials, no UVs
- `check_glb.py`: ok, with one warning (below). Khronos glTF validator: 0 errors, 0 warnings.

The cabinet is five panels (sides, top, bottom, back), so the doors open onto an interior, not a solid
block. Inside are a matte liner on all five faces, a gloss **divider** at game height 1.234 (the freezer's
floor), three fridge shelves and one freezer shelf. The food colours are baked into the materials:
- **Freezer** (1.244–1.767 m): a frozen pizza on the floor; an ice cream tub and a bag of peas on the shelf.
- **Fridge** (0.094–1.224 m): milk and juice on the top shelf, apples and an orange on the middle one, eggs
  and a tub on the bottom one.

| Material | Finish | Used for |
|---|---|---|
| `body` | gloss | cabinet, doors; takes the player's colour (`paint`) |
| `trim` | satin | plinth, badge |
| `seal` | satin | door gap |
| `chrome` | chrome | bar handles |
| `liner` | matte | interior walls |
| `shelf` | gloss | shelves, divider, tub |
| `carton` | satin | milk / egg cartons |
| `bottle` | gloss | juice bottle |
| `produce`, `produce2` | satin | apples, orange |
| `egg` | matte | eggs |
| `frozen`, `icecream`, `peas` | satin | the freezer's food |

## Doors

Each door's origin sits on its hinge, at game `x = +0.364 m` (the handles are on the left). Its handle,
and the freezer's badge, are joined into it. There is one clip per door. Both take the same 1.75 s, so the
doors move at one speed, but each has its own angle:

| Clip | Node | Duration | Swing |
|---|---|---|---|
| `freezer_open` | `door_freezer` | 1.75 s | 0° → −118° (the short door swings wide) |
| `fridge_open` | `door_fridge` | 1.75 s | 0° → −102° (the tall door stops a little shorter) |

The pack's `motions` plays `fridge_open` for the `fetch` and `drink` animation tags (snacks, a recipe's
ingredients, a cold drink, putting dishes away) and holds it for 3.5 s. `freezer_open` plays for no use
yet, because no interaction takes frozen food: that is `check_glb.py`'s one warning.

## The sinks

The two kitchen sinks share the fridge's `GameFrame` style and the family's palette (extended with
`porcelain`, `stone`, `wood`, `metal`, `dark`, `plant`, `water`). Each has a `body` material (paint) and a
`water` moving part: the stream is built full length from the spout into the basin, hidden at rest (scaled
to 0.001 along its axis), and a `water_on` clip grows it to 1 over 0.4 s. `motions` plays `water_on` for the
`wash` animation tag, with no `hold`, so the water runs for the whole use and retracts afterwards (the `wash`
tag covers both the `wash` and `washUp` interactions).

| Model | Size (w × h × d) | Triangles | Draws | Note |
|---|---|---|---|---|
| `model.farmhouseSink` | 1.000 × 1.263 × 0.670 m | 3,084 | 2 | apron-front basin, wooden worktop, plant and bottle |
| `model.counterSinkDouble` | 2.000 × 1.265 × 0.790 m | 5,496 | 3 | two inset basins, a bridge tap each |

Both match their placeholder bounds (`check_glb.py`: ok; Khronos validator: 0 errors, 0 warnings).

## Rebuild

```
python3 -B tools/art/blender/build.py kitchen            # fridge.glb, the pack, check_glb
python3 -B tools/art/blender/build.py kitchen --preview  # previews/fridge.png, fridge@<clip>.png
```
