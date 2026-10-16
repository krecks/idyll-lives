# Kitchen — notes

`kitchen.py` builds the kitchen models into `web/public/assets/models/kitchen/` and their entries in
`packs/kitchen/manifest.json`: the **fridge** (`model.fridge`, `fridge.glb`), the six **K1 fridges**
(`fridgeMini`, `fridgeBuiltIn`, `fridgeFrenchDoor`, `fridgeRetro`, `fridgeMiniRetro`, `wineFridge`) and the two **sinks**
(`model.farmhouseSink`, `farmhouseSink.glb`; `model.counterSinkDouble`, `counterSinkDouble.glb`). They
replace the primitive-part placeholders, which stay as the fallback. The fridge was first made as an
evaluation on 2026-10-13; on 2026-10-14 the flat build and the doors build (`kitchen_doors.py`,
`fridge.doors.glb`) became this one build, and the two sinks and the five K1 fridges were added.

## The fridge

A top-freezer fridge: the **top compartment is the freezer** (frozen food), the **bottom compartment is
the fridge** (normal food), separated by a real divider.

- 0.740 × 1.800 × 0.722 m (w × h × d): exactly the placeholder's bounds, built in its numbers (`F`, a
  `GameFrame`) and standing where the placeholder stands (`centre=(0, -0.094)`), its back on the wall
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

The pack's `motions` plays `fridge_open` for the `fetch`, `cook` and `drink` animation tags (snacks, a
meal's ingredients, a cold drink, putting dishes away), as the K1 fridges do, and holds it for 3.5 s. `freezer_open` plays for no use
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
| `model.farmhouseSink` | 1.000 × 1.253 × 0.670 m | 3,084 | 2 | apron-front basin, wooden worktop, plant and bottle |
| `model.counterSinkDouble` | 2.000 × 1.265 × 0.660 m | 5,496 | 3 | two inset basins, a bridge tap each |

Both match their placeholder bounds (`check_glb.py`: ok; Khronos validator: 0 errors, 0 warnings).

## The K1 fridges

Six more fridges match their placeholders' bounds. Each hinges a door built closed and swings it open
with a `hold`, so it shuts while the resident is busy; the French-door freezer and the wine fridge's top
rack slide instead. The clip each use plays is in the pack's `motions`:

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.fridgeMini` | 0.600 × 1.000 × 0.600 m | 2,194 | 2 | door | `fridge_open` 1.6 s (snack `fetch`, meal `cook`); `hold` 3 s |
| `model.fridgeBuiltIn` | 0.840 × 2.025 × 0.656 m | 3,404 | 3 | fridge door, freezer door | `fridge_open` 1.75 s (fetch, cook, cold drink); `freezer_open` 1.75 s, unused |
| `model.fridgeFrenchDoor` | 0.900 × 1.820 × 0.760 m | 2,756 | 4 | left door, right door, drawer | `fridge_open` 1.75 s (fetch, cook); `door_right` 1.75 s (cold drink); `freezer_open` 1.75 s, unused |
| `model.fridgeRetro` | 0.720 × 1.640 × 0.725 m | 1,920 | 3 | door; the chrome lever inside it (`G.attach`) | `lever` turns 30° out (0.3 s, `hold` 0.4), then `fridge_open` 1.75 s (`delay` 0.3; fetch, cook, cold drink) |
| `model.fridgeMiniRetro` | 0.574 × 0.860 × 0.580 m | 3,975 | 2 | door (its handle, "MARIE" letters and unicorn sticker on it) | `fridge_open` 1.6 s (fetch, cook, cold drink); `hold` 3 s |
| `model.wineFridge` | 1.000 × 0.950 × 0.660 m | 2,316 | 4 | glass door, top rack | `door_open` 1.5 s (wine `drink`); `hold` 2.5 s |

The built-in's freezer door and the French-door freezer drawer carry clips no use plays yet, because no
interaction takes frozen food: `check_glb.py` warns of both (as it does for `model.fridge`'s freezer).
`fridgeFrenchDoor` and `wineFridge` take 4 draw calls (the most allowed, one per moving part plus the
still body); the rest take 2 or 3. The five were built and exported on 2026-10-14 and sit in Validate ›
Fridges until checked in play.

## Rebuild

```
python3 -B tools/art/blender/build.py kitchen            # fridge.glb, the pack, check_glb
python3 -B tools/art/blender/build.py kitchen --preview  # previews/fridge.png, fridge@<clip>.png
```

## Where it stands (review, 2026-10-14)

Each floor model is centred where its placeholder stands (its `Model`'s `centre`, which `G.settle()` uses),
not on its tile: its back is on the wall face (-0.43), as the placeholder's, a run of counters and
appliances lines up, and its top is under its content `surface` (`catalog2.py` TOPS). Since the
2026-10-14 fix-ups the placeholders no longer reach into the wall: both moved forward onto the face.
There is one counter depth, the catalogue's 0.66 m (worktop -0.43..0.23): the catalog2 counters keep
their placeholder's old 0.79-deep numbers and are drawn through `COUNTER66` (gamekit `GameFrame(depths=)`,
catalog2.py `shallow`). Islands stand free, centred.
