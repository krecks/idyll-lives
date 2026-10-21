# Bedroom storage — notes

`bedroom_storage.py` builds the bedroom's nightstands and small storage (BR2) and its wardrobes and drawers (BR3)
into `web/public/assets/models/bedroom_storage/` and their entries in `packs/bedroom_storage/manifest.json`, keyed
by the existing `model.<id>` keys. Built
2026-10-14. They sat in Validate › Nightstands and Validate › Wardrobes until checked in play.

## Shared

- **Built in place.** The parts are placed in game axes with no shift (`F = G.GameFrame()`), where the game places it,
  and settled with `G.settle(centre=False)` and `centre=` on each `Model`. Most stand at the
  back of their tile: their backs are on the wall face (game depth -0.43, `WALL`). Several old parts reached
  3.5–7 cm into the wall (the nightstand and lamp nightstand to -0.465, the tall chest and low dresser to -0.48,
  the armoire to -0.50); they stand on the wall face now too (2026-10-14; the armoire's and the chests' a little
  shallower, as the models), and the content `surface`s of the nightstand, tall chest and low dresser start at
  the wall face. The drum nightstand, hamper, chest and bench stand where the game places them.
- **Tops are the content's `surface` heights**: nightstand, modern and lamp nightstand 0.56, drum 0.57, three-drawer
  0.60, blanket chest 0.505, dresser 0.89, low dresser 0.80, tall chest 1.16 (catalog2 `TOPS`). The bench's
  cushion top is its seat, 0.47. The lamp's bulb is at 0.935–0.97, around the content's `light.height` 0.95.
- **Palette.** One for the family. Each model's main colour is its own material (`oak`, `white`, `clay`,
  `walnut`, `cedar`, `wicker`, `birch`, `linen`, `mahogany`), listed first in `paint`, then the handles. Clothes
  (`cloth1`–`cloth4`) are named, not held as materials at import (a `reset` removes materials).
- **Draw calls (at most 3).** A textured finish costs a call per part it's on, so the moving parts are kept to one
  textured finish: the oak dresser's pulls, picture frame, print and vase are `wood` (with the body), and the low
  dresser's black handles are `wood` too (`ebony`). The three-drawer nightstand and the armoire are `satin`
  (walnut, mahogany: a little lighter than the old parts' `wood` colours, which the wood texture brightens), so
  their drawer and doors carry brass and the armoire's clothes without more calls.
- **Drawers** (`_drawer`) are a front, its pulls and a box behind it (inside the solid carcass at rest), with folded
  clothes in it, joined with the origin on the carcass face; `_gap` puts a dark panel on the carcass behind the
  front, so the opening reads dark when it slides out. **Wardrobes** are open shells (`_shell`) with a rail of
  shirts hung side-on, a hat shelf and, in the double wardrobe, shelves of folded clothes; their doors are hinged
  on their outer edges.

## The clips (BR3)

All are played for the `outfit` use (`motions.uses.outfit`):

| Model | Clip | Moves | Hold |
|---|---|---|---|
| `dresser` | `drawer_open` | the middle (wide) drawer slides out 30 cm in 0.9 s | 2.5 s |
| `chestOfDrawers` | `drawer_open` | the middle (third) drawer, 30 cm | 2.5 s |
| `dresserLow` | `drawer_open` | the middle drawer of the middle column, 30 cm | 2.5 s |
| `wardrobe` | `doors_open` | both doors swing out 100° in 1.2 s | 3 s |
| `armoire` | `doors_open` | both doors, 100° | 3 s |
| `wardrobeSingle` | `door_open` | its door, hinged on the left (the old part's handle is on the right), 100° | 3 s |

The dresser's old part has two columns of three drawers; the model has two small drawers over two wide ones,
so "the middle drawer" is one drawer.

## Lids and drawers for a use (BR2, +use)

The blanket chest's lid (`lid`, hinged along its back edge at 0.455 m; the chest is hollow with a folded blanket in
it) lifts 95° (`lid_open`, 0.9 s, held 2 s) for the content's "Take out a blanket" (`takeBlanket`, `fetch`, 3
minutes, comfort; not autonomous). It costs the chest a third draw call.

The three-drawer nightstand's top drawer (`drawer`, origin on the carcass face, box behind it, folded clothes in it)
slides out 30 cm (`drawer_open`, as the dressers') for the content's "Pick an outfit" (`outfit`, the dressers' use,
but not autonomous there, so generated houses play as before), held 2.5 s: a second draw call. The hamper's lid
(`lid`, hinged at its back; the basket is hollow with washing in it) lifts 70° (`lid_open`, 0.7 s; any further and
it goes into the wall the hamper stands at) for the whole of the content's "Sort the washing" (`sortWashing`,
`clean`, 5 minutes, comfort, chores; not autonomous): a third draw call. (Washing that goes from the hamper to the
washer would need a laundry loop in the sim.)

## The models

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.nightstand` | 0.54 × 0.66 × 0.47 m | 1,936 | 2 | Body, Handles (a book and a mug on top, books in the niche) |
| `model.nightstandModern` | 0.58 × 0.56 × 0.63 m | 1,616 | 2 | Body, Handles (oak top and legs) |
| `model.nightstandRound` | 0.54 × 0.57 × 0.56 m | 2,396 | 2 | Body, Handles |
| `model.nightstandDrawers` | 0.58 × 0.60 × 0.63 m | 1,572 | 2 | Body, Handles |
| `model.nightstandLamp` | 0.54 × 1.08 × 0.47 m | 2,472 | 3 | Body, Shade (glows `#FFD9A0` at night) |
| `model.cedarChest` | 0.97 × 0.505 × 0.52 m | 1,572 | 3 | Body, Handles |
| `model.laundryHamper` | 0.48 × 0.63 × 0.48 m | 2,544 | 3 | Hamper |
| `model.benchBedEnd` | 1.60 × 0.47 × 0.42 m | 1,680 | 2 | Frame, Cushion |
| `model.dresser` | 1.44 × 1.35 × 0.54 m | 3,420 | 3 | Body, Handles (a picture, a vase and a box on top) |
| `model.wardrobe` | 1.84 × 2.41 × 0.63 m | 4,428 | 3 | Body (the doors), Handles (boxes on top) |
| `model.chestOfDrawers` | 0.88 × 1.16 × 0.66 m | 2,004 | 2 | Body, Handles |
| `model.dresserLow` | 1.82 × 0.80 × 0.635 m | 2,936 | 3 | Body, Handles |
| `model.wardrobeSingle` | 0.90 × 2.28 × 0.62 m | 2,220 | 2 | Body, Handles (a box on top) |
| `model.armoire` | 1.86 × 2.11 × 0.63 m | 5,676 | 3 | Body, Handles |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art bedroom_storage            # every .glb, the pack, check-glb
cargo art bedroom_storage --preview  # previews/dresser.png, dresser@drawer_open.png, …
```
