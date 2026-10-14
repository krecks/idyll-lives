# Kitchen counters — notes

`kitchen_counters.py` builds the kitchen's **K4 base counters**, **K5 tall and wall storage** and **K6 islands and bars** into
`web/public/assets/models/kitchen_counters/` and their entries in `packs/kitchen_counters/manifest.json`.

K4 (2026-10-14): the plain counter (`model.counter`), the corner counter (`model.counterCorner`), the
counter end with shelves (`model.counterEnd`), the outer corner counter (`model.counterCornerOuter`), the
drawer counter (`model.counterDrawers`), the open-shelf counter (`model.counterOpenShelf`), the butcher
block (`model.butcherBlock`) and the marble counter (`model.counterMarble`).

K5 (2026-10-14): the pantry cupboard (`model.pantry`), the tall larder (`model.pantryTall`), the wall
cabinets (`model.wallCabinet`, `model.wallCabinetGlass`, `model.wallCabinetCorner`), the cooker hood
(`model.rangeHood`), the spice rack (`model.spiceRack`), the pot rail (`model.potRail`), the kitchen wall
shelves (`model.kitchenShelfWall`), the pedal bin (`model.trashCan`) and the recycling bins
(`model.recyclingBins`).

K6 (2026-10-14): the kitchen island (`model.kitchenIsland`), the small island (`model.islandSmall`), the long
island (`model.islandLong`), the bar counter (`model.barSegment`) and the bar end (`model.barEnd`).

They replace the primitive-part placeholders of `catalog.py` / `catalog2.py`, which stay as the fallback.

## Materials

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
| `brass` | metal | the marble counter's handles, the larder's |
| `block`, `block2` | wood | the butcher block's top and its top layer |
| `leg` | wood | the block's legs and shelf |
| `wood2` | satin | the block's knife-drawer front (plain, so it draws with the body) |
| `basket` | satin | the block's basket, a larder basket |
| `metal` | metal | the block's dark drawer handle, the pot rail |
| `porcelain` | gloss | plates, a bowl, a jar, a box |
| `steel` | metal | the cooker hood, the pedal bin |
| `trim` | satin | the hood's front trim |
| `bulb` | gloss | a strip light (under a wall cabinet, in the hood) |
| `copper` | metal | the pot rail's pans and pot |
| `glass` | glass | a glazed door, a jar; the wall cabinet's glazed doors |
| `back` | satin | the glazed cabinet's pale back panel |
| `leaf` | satin | a pot plant's foliage |
| `oakd` | wood | the brackets under the kitchen wall shelves |
| `bin1`, `bin2`, `bin3` | satin | the recycling bins (blue, green, yellow) |
| `lid` | satin | a bin's dark lid and handle |
| `soil` | matte | a plant pot's soil |

A transparent finish (`glass`) and a textured finish (`wood`, `foliage`) are each their own draw group.
The glazed cabinet keeps its carcass and doors `cab` and only the panes `glass`, so it draws in two
groups; the still bodies of the textured things (oak shelves, a basket) are kept to one group each.

## Clips

Every moving part is its own object with its origin on the hinge (or the front face for a slide); each
clip is one NLA track named for the pack's `motions`. Frame 1 is the rest pose. A door that a resident
opens swings out 100° (`DOOR_SECONDS` 1.6 s, `DOOR_HOLD` 2.5 s); a drawer slides out 25 cm
(`SECONDS` 1.4 s, `HOLD` 1.5 s). `G.settle()` runs before `G.swing()`/`G.slide()`, since a clip keys the
rest pose where it is.

## K4 — base counters

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

## K5 — tall and wall storage

A wall thing sits on the wall face (`WALL = -0.43`) at its real height: it is built at that height and
`settle(floor=False)` / `check_size(floor=False)` leave it alone, so `check_glb` reports it as floating,
which is fine on a wall. A floor thing stands and is centred as usual.

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.pantry` | 0.940 × 2.270 × 0.630 m | 2,592 | 3 | two doors | `doors_open` 1.6 s (snack, `hold` 2.5 s) |
| `model.pantryTall` | 0.960 × 2.390 × 0.700 m | 3,024 | 3 | two doors | `doors_open` (snack, `hold` 2.5 s) |
| `model.wallCabinet` | 0.980 × 0.740 × 0.379 m | 1,080 | 1 | — | static (future: the doors swing up) |
| `model.wallCabinetGlass` | 0.960 × 0.720 × 0.406 m | 2,460 | 2 | — | static; glass doors |
| `model.wallCabinetCorner` | 0.960 × 0.723 × 0.406 m | 1,292 | 2 | — | static |
| `model.rangeHood` | 0.900 × 0.712 × 0.515 m | 648 | 1 | — | static |
| `model.spiceRack` | 0.520 × 0.440 × 0.106 m | 2,268 | 3 | — | static |
| `model.potRail` | 0.910 × 0.547 × 0.230 m | 1,236 | 1 | — | static |
| `model.kitchenShelfWall` | 0.900 × 0.757 × 0.240 m | 1,476 | 3 | — | static |
| `model.trashCan` | 0.310 × 0.685 × 0.365 m | 540 | 1 | — | static (future: `lid_open`) |
| `model.recyclingBins` | 0.920 × 0.660 × 0.455 m | 972 | 1 | — | static (future: the lids lift) |

## K6 — islands and bars

The freestanding islands put their cupboards on the back and overhang the worktop at the front for stools;
the bars have a slatted guest side, a top at 1.06 and a brass foot rail. The kitchen island's drawer
is on the stools' side, between them, since residents stand at the front (+Z) to use it: it slides out
towards them for a sandwich (a fixed drawer front on the cupboard side matches it); everything else is
static this pass.

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.kitchenIsland` | 1.940 × 1.035 × 0.929 m | 2,972 | 3 | a drawer | `drawer_open` 1.4 s (sandwich, `hold` 1.5 s) |
| `model.islandSmall` | 0.960 × 0.970 × 0.951 m | 1,032 | 1 | — | static |
| `model.islandLong` | 2.960 × 0.980 × 0.951 m | 1,944 | 2 | — | static |
| `model.barSegment` | 1.000 × 1.060 × 0.890 m | 1,656 | 2 | — | static |
| `model.barEnd` | 1.000 × 1.060 × 0.890 m | 1,656 | 2 | — | static |

All match their placeholder bounds (`check_glb.py`: 0 failed; Khronos glTF validator: 0 errors,
0 warnings each). The eight K4 counters sit in Validate › Counters, the eleven K5 things in
Validate › Kitchen Storage and the five K6 islands and bars in Validate › Islands & Bars, until
checked in play.

## Rebuild

```
python3 -B tools/art/blender/build.py kitchen_counters            # the .glbs, the pack, check_glb
python3 -B tools/art/blender/build.py kitchen_counters --preview  # previews/<id>.png, <id>@<clip>.png
```
