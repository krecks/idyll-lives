# Fridge — Blender model for `model.fridge`

Built as an evaluation outside the repo on 2026-10-13 and brought onto branch `v0.3/fridge` as it was.
Its eyeballing scripts (`compare.py`, `render_views.py`) and its renders, the door frames and swing GIFs
included, are kept untracked in `tools/art/blender/previews/` (`eval/` for the ones from outside `tools/`).

## What it is

The kitchen fridge (`model.fridge`, "Idyll Lives") built as a real Blender model and
exported to glTF, following `docs/design/objects.md` ("How to build a new object",
the 0.3.0 standard). It replaces the primitive-part placeholder
(`furniture_prims.fridge_classic`) while keeping that placeholder as the fallback.

A top-freezer fridge: the **top compartment is the freezer** (frozen food), the **bottom
compartment is the fridge** (normal food), separated by a real divider.

- 0.740 × 1.800 × 0.722 m (w × h × d) — exactly the placeholder's bounds
- origin at the footprint centre, bottom on the floor, front −Y in Blender
- 4,394 triangles, 1 draw call after the finish mapping, 14 materials, no UVs
- `check_glb.py`: ok; Khronos glTF validator 2.0.0-dev.3.10: 0 errors, 0 warnings

## Files

| File | Change |
|---|---|
| `tools/art/blender/kitchen.py` | new family script (built on `gamekit.py`) |
| `web/public/assets/models/kitchen/fridge.glb` | new model (211.8 KB) |
| `web/public/assets/packs/kitchen/manifest.json` | new override pack |
| `web/public/assets/manifest.json` | lists `packs/kitchen/manifest.json` |
| `web/public/assets/CREDITS.md` | credit line |

`fridge.patch` holds the text diff; the `.glb` files are the binaries.

### Doors (the model half of open/close, done ahead of the engine)

| File | Change |
|---|---|
| `tools/art/blender/kitchen_doors.py` | new: the same fridge with hinged door nodes and one clip per door |
| `web/public/assets/models/kitchen/fridge.doors.glb` | new model (212.2 KB) |

`fridge.doors.glb` is the file the pack registers (branch `v0.3/object-motion`): the game keeps
each door as a moving part and plays `fridge_open` while a resident takes something out
(the entry's `motions`: the `fetch` and `drink` animation tags, `hold` 3.5 s). `freezer_open`
plays for no use yet: no interaction takes frozen food. `fridge.glb` stays as the flat build.

On lots every fridge is styled (`model.fridge@modern` by default), and the style variants are
placeholders of other shapes (docs/design/objects.md, styles). For now the pack also draws
`@modern` with this model (`STYLED` in `kitchen.py`), keeping the `@modern` placeholder as its fallback.
Other fridges (mini, built-in, French door, retro) are objects of their own and get models of their own.

## Design

A top-freezer fridge: a white gloss cabinet built as five panels (so the doors open onto
an interior, not a solid block) with a matte liner, a divider and shelves, a rounded freezer
door over a taller fridge door, a dark gap between them, vertical chrome bar handles on the left
(hinges right) and a satin badge on the freezer door. Materials are named for what they are and
map to the game's finishes so a model costs one draw call:

| Material | Finish | Used for |
|---|---|---|
| `body` | gloss | cabinet, doors |
| `trim` | satin | plinth, badge |
| `seal` | satin | door gap |
| `chrome` | chrome | bar handles |
| `liner` | matte | interior walls |
| `shelf` | gloss | shelves, divider, tub |
| `carton` | satin | milk / egg cartons |
| `bottle` | gloss | juice bottle |
| `produce` | satin | apples |
| `produce2` | satin | orange |
| `egg` | matte | eggs |
| `frozen` | satin | frozen pizza |
| `icecream` | satin | ice cream tub |
| `peas` | satin | frozen peas |

## Interior and compartments

`_shell()` builds the cabinet as five panels (sides, top, bottom, back) so the front is a real
opening. `_interior()` adds a matte liner on all five inside faces, a gloss **divider** at game
height 1.234 (the freezer's floor and the fridge's ceiling, so the two compartments are separate
boxes), three fridge shelves and one freezer shelf. Food colours are baked into the materials, so they
survive the loader's finish mapping (one draw call).

- **Freezer (top, height 1.244–1.767 m)** — `_freezer_food()`: a frozen pizza lying flat on the
  freezer floor, an ice cream tub and a bag of frozen peas on the freezer shelf.
- **Fridge (bottom, height 0.094–1.224 m)** — `_fridge_food()`: milk carton + gable and a juice
  bottle on the top shelf, three apples and an orange on the middle shelf, an egg carton + eggs and a
  tub on the bottom shelf.

Every part is placed with `gbox` / `gcyl` / `gsphere` in game coordinates and verified to sit inside
its own compartment and inside the cavity (game `x −0.34..0.34`, depth `−0.405..0.145`). `_interior()` is
shared by both builds, so the flat file and the doors file carry the same food.

## Doors

Three mesh nodes instead of one: `cabinet`, `door_freezer`, `door_fridge`. Each door's Blender origin
sits on its hinge (game `x = +0.364 m`, the handles are on the left), so a Z rotation swings it; the
handle and badge are joined into the door they belong to. There are **two clips, one per door**, both over the
same 1.75 s so the doors open and close at one speed, but each with its own angle so they still read
differently:

| Clip | Node | Duration | Swing |
|---|---|---|---|
| `freezer_open` | `door_freezer` | 1.75 s (42 frames) | 0° → −118° (short door: swings wide) |
| `fridge_open` | `door_fridge` | 1.75 s (42 frames) | 0° → −102° (tall door: stops a little shorter) |

The engine can open either without the other. Each door's first key is closed, so the file's rest pose is
closed; playing a clip forward opens that door, and reversing it closes it. No skin, no bones — a node's own
rotation, which the game's thin instances can drive without new machinery.

The doors swing forward: the free edge reaches about game `+Z 1.0 m` open, from `0.36 m` closed.

## Verification

- `check_glb.py model.fridge` → ok, 4,394 tris, 1 draw, 14 materials, 0.74 × 1.80 × 0.72 m
- `check_glb.py` over all models → 7 models, 0 failed
- Khronos glTF validator 2.0.0-dev.3.10 → 0 errors, 0 warnings, 0 infos, 0 hints
- `previews/fridge_compare.png` — placeholder vs. model, front and 3/4 (flat Workbench)
- `previews/s_closed.png`, `s_freezer.png`, `s_fridge.png`, `s_both.png` — rendered from the
  model's front (Blender −Y) with each door driven on its own

Note: `gamekit.preview` puts its camera at Blender **+Y**, which is the model's *back*, so the
stock `previews/fridge.png` / `doors_*.png` do not show the front or the interior. The `s_*.png`
renders are a throwaway script (`/tmp/state_render.py`), not a repo change.

## Doors verification

- `check_glb.py model.fridge` with the doors file in place → ok, 0 failed, one warning:
  `has animations; the game drops them` (expected until the engine adapts)
- Khronos glTF validator 2.0.0-dev.3.10 on `fridge.doors.glb` → 0 errors, 0 warnings
- 2 animations, each targeting one node, both over the same 1.75 s (42 keys) so the doors share one speed
  but keep their own angle: `freezer_open` → `door_freezer` (0° → −118°) and `fridge_open` → `door_fridge`
  (0° → −102°); every node's rest rotation is identity (closed)
- food in its own compartment, measured per part on the un-joined interior: fridge 0.094–1.224 m
  (milk, juice, apples, orange, eggs, tub), freezer 1.244–1.767 m (pizza, ice cream, peas)
- front renders (`s_*.png`): the freezer-only frame changes rows 124–284 and shows only the frozen
  colours (893 icy, 127 green); the fridge-only frame changes rows 251–540 and shows only the normal
  colours (331 red, 117 orange); each has none of the other's food. `s_closed.png` shows no food colours.

## Rebuild

```
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --python tools/art/blender/kitchen.py            # writes the .glb and the pack
python3 -I -B tools/art/blender/check_glb.py model.fridge

# the door build (the registered file; also rewrites the pack)
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --python tools/art/blender/kitchen_doors.py      # writes fridge.doors.glb
```
