# Decor — notes

`decor.py` builds the 22 decor models into `web/public/assets/models/decor/` and their entries in
`packs/decor/manifest.json`: wall art and clocks (D1), ornaments (D2) and textiles (D3). They replace the
primitive-part placeholders of `catalog.py`, which stay as the fallback. Built 2026-10-14; they sit in Validate ›
Wall Decor, Ornaments and Textiles until checked in play.

Each is built where its placeholder stands (`F`, game axes):
- **Wall things** (the pictures, the clocks, the shelf, the curtains) hang at their real height with their back on the
  wall face: `G.settle(centre=False, floor=False)`, `floor=False` and `centre=` on the `Model`. `check_glb.py` warns
  that they float (right on a wall).
- **The hanging plant** hangs from the ceiling (2.8 m), centred, `floor=False`.
- **The fireplace** has a raised hearth bench (0.42 m, `HEARTH`) with the firebox above it: residents warming up
  sit on the bench, the fire behind them (content `seat`: height 0.42, forward 0.02; the fire's `light.height` 0.7).
- **Against the wall**, the grandfather clock and the fireplace have their backs on the wall face. Their placeholders
  reach 2–5 cm into the wall, so `check_glb.py` warns that they're off its centre (and the curtains, whose rail
  brackets now reach the wall). Those warnings are expected.
- **The rest** stand centred on their tile, or lie flat (the rugs).

The pictures are flat colour shapes (matte, one draw call with the frame), not textures.

## What moves

| Model | Clip | Kind | What |
|---|---|---|---|
| `wallClock`, `grandfatherClock` | `hour_hand`, `minute_hand` | `clock` (`at` 610) | the hands tell the game's time: each clip turns its hand once round clockwise (`G.spin`, 12 s), and the game holds it where the time has it (ten past ten at rest, the catalogue picture). No second hand: at normal speed a game minute is a real second |

The grandfather clock's trunk door and pendulum are still (they were the `door_open` use clip for `wind` and an
ambient `pendulum`): with its walnut body at 2 draws, the two hands take the last two of the 4. A satin body (1 draw)
would give one back for the pendulum, at the cost of the wood grain.
| `hangingPlant` | `sway` | use `water`, `hold` 1.6 s | the plant, pot and hanger swing 5° about the ceiling hook and back (`G.rock`); played back after the hold, so it sways twice and settles |
| `fireplace` | `flames` | use `relax` | an ember bed and tongues of flame (gloss) grow from nothing on the logs (`G.grow`) for the whole use; the light is the content's |
| `curtains`, `curtainsLong` | `close` | night | the two pleated panels, gathered open at the sides, widen from their outer ends until they meet (`G.squash` in x, about the rail end) |

The grandfather clock takes **4 draw calls** (the most allowed): its walnut body with its plain parts (2), the door
(wood only, 1) and the brass pendulum (1). Its door has no glass so that it doesn't need a fifth. The curtains' tabs
are their own fabric, so the moving panels cost one draw call each.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.printFramed` | 0.50 × 0.71 × 0.03 m | 1,440 | 1 | Frame |
| `model.photoPrint` | 0.75 × 0.50 × 0.03 m | 616 | 1 | Frame |
| `model.paintingClassic` | 0.84 × 0.65 × 0.05 m | 1,224 | 1 | Frame |
| `model.canvasAbstract` | 1.50 × 0.90 × 0.05 m | 716 | 2 | Frame |
| `model.wallClock` | 0.36 × 0.36 × 0.07 m | 940 | 2 | Case, Face |
| `model.wallShelf` | 0.90 × 0.66 × 0.24 m | 2,220 | 3 | Shelves |
| `model.grandfatherClock` | 0.60 × 2.12 × 0.40 m | 2,752 | 4 | Case, Face |
| `model.vaseTall` | 0.33 × 1.27 × 0.23 m | 1,040 | 2 | Body |
| `model.urnPorcelain` | 0.36 × 0.62 × 0.36 m | 3,392 | 1 | Body |
| `model.vasePedestal` | 0.40 × 1.60 × 0.40 m | 1,612 | 2 | Vase, Pedestal |
| `model.bustMarble` | 0.44 × 1.54 × 0.44 m | 1,932 | 1 | Bust, Plinth |
| `model.statueCat` | 0.31 × 0.60 × 0.41 m | 2,144 | 1 | Body |
| `model.statueHorse` | 0.46 × 1.41 × 0.44 m | 1,232 | 2 | Body |
| `model.coatRack` | 0.42 × 1.79 × 0.34 m | 2,040 | 3 | Body |
| `model.hangingPlant` | 0.46 × 1.83 × 0.46 m | 2,032 | 3 | Pot, Cord |
| `model.fireplace` | 1.96 × 2.80 × 0.57 m | 2,648 | 3 | Surround, Mantel |
| `model.curtains` | 0.99 × 1.52 × 0.10 m | 1,716 | 3 | Fabric, Rail |
| `model.curtainsLong` | 1.93 × 2.58 × 0.10 m | 2,100 | 3 | Fabric, Rail |
| `model.rugRound` | 1.90 × 0.02 × 1.90 m | 2,296 | 1 | Rug |
| `model.rugLarge` | 2.85 × 0.02 × 1.85 m | 728 | 1 | Rug |
| `model.rugRunner` | 0.80 × 0.02 × 2.80 m | 944 | 1 | Rug |
| `model.rugPattern` | 2.84 × 0.02 × 1.86 m | 1,916 | 1 | Rug |

`check_glb.py`: 22 models, 0 failed. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
python3 -B tools/art/blender/build.py decor            # the 22 .glb, the pack, check_glb
python3 -B tools/art/blender/build.py decor --preview  # previews/<id>.png, <id>@<clip>.png
```
