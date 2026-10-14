# Kitchen counters — notes

`kitchen_counters.py` builds the kitchen's **K4 base counters** into
`web/public/assets/models/kitchen_counters/` and their entries in `packs/kitchen_counters/manifest.json`:
the plain counter (`model.counter`), the corner counter (`model.counterCorner`), the counter end with
shelves (`model.counterEnd`), the outer corner counter (`model.counterCornerOuter`), the drawer counter
(`model.counterDrawers`), the open-shelf counter (`model.counterOpenShelf`), the butcher block
(`model.butcherBlock`) and the marble counter (`model.counterMarble`). They replace the primitive-part
placeholders of `catalog.py` / `catalog2.py`, which stay as the fallback. Built 2026-10-14.

A base counter is a plinth, a carcass and a worktop; the front carries a drawer over two doors (the plain,
corner and marble counters), one door, four drawers, or nothing (the open shelves). Two worktop heights are
in play: **0.96 m** for the catalogue counters (`_unit66`) and **0.95 m** for the catalog2 ones
(`_unit79`), matching each content entry's `surface`. The parts are placed in game axes (`F`, a
`GameFrame` shifted back by the placeholder's depth centre, so the model is centred on its tile).

## Materials

Each model maps its materials onto the game's finishes, so it costs one draw call per finish group. A
textured finish (`wood`, `fabric`) is a group of its own, so the butcher block keeps its wood on the still
body only — the knife drawer's front is a plain `wood2` satin, and the basket a satin tan — to stay within
the draw budget.

| Material | Finish | Used for |
|---|---|---|
| `cab` | satin | a cabinet carcass and its fronts; the player's colour (`paint`) |
| `worktop` | gloss | a laminate worktop |
| `dark` | satin | plinths |
| `chrome` | chrome | bar handles and pulls |
| `oak` | wood | a wood end unit, open shelves, a utensil handle |
| `navy` | satin | the marble counter's painted carcass |
| `marble` | gloss | the marble worktop |
| `vein` | gloss | a vein through the marble |
| `brass` | metal | the marble counter's handles |
| `block`, `block2` | wood | the butcher block's top and its top layer |
| `leg` | wood | the block's legs and shelf |
| `wood2` | satin | the block's knife-drawer front (plain, so it draws with the body) |
| `basket` | satin | the block's basket |
| `metal` | metal | the block's dark drawer handle |
| `porcelain` | gloss | plates, a bowl, a jar, a box |
| `cream` | gloss | a cream jar |
| `rust` | gloss | a terracotta tin |
| `teal`, `sage` | gloss | pots |
| `yellow` | satin | a lemon |
| `soil` | matte | the dark back of an open shelf |

## Clips

A drawer is built as its own object with its origin on its front face; the model is settled and the clip
keyed after (`settle()` must come before `slide()`). Four counters slide their top drawer out 25 cm for a
sandwich (`drawer_open`); the rest are static. `counterDrawers` is static in this pass — its four drawers
sliding out in turn would be a second clip (E1). `counterOpenShelf` has no handles, so its `paint` is
Cabinet/Worktop only.

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.counter` | 1.000 × 0.960 × 0.660 m | 1,404 | 2 | top drawer | `drawer_open` 1.4 s (sandwich, `hold` 1.5 s) |
| `model.counterCorner` | 1.000 × 1.270 × 1.000 m | 1,520 | 3 | top drawer | `drawer_open` (sandwich, `hold` 1.5 s) |
| `model.counterEnd` | 1.000 × 0.950 × 0.790 m | 1,428 | 2 | — | static |
| `model.counterCornerOuter` | 0.810 × 1.035 × 0.790 m | 972 | 1 | — | static |
| `model.counterDrawers` | 1.000 × 0.950 × 0.790 m | 1,188 | 1 | — | static |
| `model.counterOpenShelf` | 1.000 × 0.950 × 0.790 m | 1,452 | 2 | — | static |
| `model.butcherBlock` | 0.900 × 0.945 × 0.813 m | 1,512 | 3 | knife drawer | `drawer_open` (sandwich, `hold` 1.5 s) |
| `model.counterMarble` | 1.000 × 0.953 × 0.790 m | 1,620 | 2 | top drawer | `drawer_open` (sandwich, `hold` 1.5 s) |

All match their placeholder bounds (`check_glb.py`: 0 failed; Khronos glTF validator: 0 errors,
0 warnings each). The eight sit in Validate › Counters until checked in play.

## Rebuild

```
python3 -B tools/art/blender/build.py kitchen_counters            # the .glbs, the pack, check_glb
python3 -B tools/art/blender/build.py kitchen_counters --preview  # previews/<id>.png, <id>@<clip>.png
```
