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

All are played for the `outfit` use (`motions.uses.outfit`), and held open for the whole of it: the resident
changes into another outfit there (content `changesOutfit`, halfway through) and the drawer or the doors shut as
they leave (until 2026-10-21 they shut after 2.5 or 3 s, before the change):

| Model | Clip | Moves | Hold |
|---|---|---|---|
| `dresser` | `drawer_open` | the middle (wide) drawer slides out 30 cm in 0.9 s | 2.5 s |
| `chestOfDrawers` | `drawer_open` | the middle (third) drawer, 30 cm | 2.5 s |
| `dresserLow` | `drawer_open` | the middle drawer of the middle column, 30 cm | 2.5 s |
| `wardrobe` | `doors_open` | both doors swing out 100° in 1.2 s | 3 s |
| `armoire` | `doors_open` | both doors, 100° | 3 s |
| `wardrobeSingle` | `door_open` | its 0.58 m door, hinged on the left (the old part's handle is on the right), 100° | 3 s |

The chest, the low dresser and the wardrobes' doors stay open for the whole "Pick an outfit" use (G-69: no `hold` there), and shut as the resident leaves.

The dresser's old part has two columns of three drawers; the model has two small drawers over two wide ones,
so "the middle drawer" is one drawer.

## Lids and drawers for a use (BR2, +use)

The blanket chest's lid (`lid`, hinged along its back edge at 0.455 m; the chest is hollow with a folded blanket in
it) lifts 95° (`lid_open`, 0.9 s, held 2 s) for the content's "Take out a blanket" (`takeBlanket`, `fetch`, 3
minutes, comfort; not autonomous). It costs the chest a third draw call.

The three-drawer nightstand's top drawer (`drawer`, origin on the carcass face, box behind it; a biscuit tin, two
chocolate bars and a paperback in it) slides out 30 cm (`drawer_open`, as the dressers') for the content's "Grab a
snack from the drawer" (`snack`, `SNACK_DRAWER`), held 2.5 s: a second draw call. It was keyed to "Pick an outfit"
with clothes in it until 2026-10-21, when the content gave the drawer the snack (I-bedroom-05). The hamper's lid
(`lid`, hinged at its back; the basket is hollow with washing in it) lifts 70° (`lid_open`, 0.7 s; any further and
it goes into the wall the hamper stands at) for the whole of the content's "Sort the washing" (`sortWashing`,
`clean`, 5 minutes, comfort, chores; not autonomous): a third draw call. (Washing that goes from the hamper to the
washer would need a laundry loop in the sim.)

## Sizes

The modern and the three-drawer nightstands are 0.47 m deep, as the classic one (their case's front at game depth
0, `NS_FRONT`; the top 0.46 deep), and the tall chest 0.51 (`CHEST_FRONT` 0.04): until 2026-10-21 they were
0.63–0.66 m, as deep as their tile, reaching past a bed's pillows (M-bedroom-02). The content `surface`s follow
(0.58 × 0.46 at -0.2; 0.88 × 0.48 at -0.19). The single wardrobe is 0.60 m wide with a 0.58 m door (`WS_W`; it was
0.90 with a door as wide as a room's, M-bedroom-03). The wardrobes' box on the hat shelf is 0.13 m high and inside
the side: it came up through the carcass's top (and the single one's side).

## The models

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.nightstand` | 0.54 × 0.66 × 0.46 m | 1,884 | 2 | Body, Handles (a shaped apron, a turned knob; a book with pages and a mug of coffee on top, books in the niche) |
| `model.nightstandModern` | 0.58 × 0.56 × 0.47 m | 1,416 | 2 | Body, Handles (rounded oak top, splayed oak legs, a black bar pull, a lidded box in the niche) |
| `model.nightstandRound` | 0.54 × 0.57 × 0.56 m | 2,468 | 2 | Body, Handles (a swelling drum, the drawer front between two seams, a turned knob, a moulded top) |
| `model.nightstandDrawers` | 0.58 × 0.60 × 0.47 m | 1,968 | 2 | Body, Handles (rounded case and top, bar pulls; a lidded biscuit tin and a paperback in the drawer) |
| `model.nightstandLamp` | 0.54 × 1.09 × 0.46 m | 3,284 | 3 | Body, Shade (glows `#FFD9A0` at night; brass foot and collar, trimmed shade on a harp, a finial) |
| `model.cedarChest` | 0.96 × 0.505 × 0.52 m | 3,212 | 3 | Body, Handles (bracket-cut plinth, two raised panels in frames, iron corner brackets and drop handles, a brass lock; a framed raised field on the lid) |
| `model.laundryHamper` | 0.47 × 0.63 × 0.48 m | 3,620 | 3 | Hamper (woven staves, braided bands and rim, a domed lid with a loop; rumpled washing, a sleeve over the rim) |
| `model.benchBedEnd` | 1.60 × 0.47 × 0.42 m | 2,320 | 2 | Frame, Cushion (each end one bent brass tube, glides; a piped, buttoned, puffed cushion) |
| `model.dresser` | 1.44 × 1.35 × 0.54 m | 4,688 | 4 | Body, Handles (bracket feet, a shaped apron, bar pulls, a moulded top; a framed picture on an easel, a turned vase, a lidded box with a clasp) |
| `model.wardrobe` | 1.84 × 2.41 × 0.63 m | 5,036 | 3 | Body (the doors), Handles (frame-and-panel doors, hinge knuckles; clothes on wire hangers, lidded boxes with labels) |
| `model.chestOfDrawers` | 0.88 × 1.16 × 0.51 m | 3,012 | 2 | Body, Handles (frame-and-panel drawer fronts, bar pulls, a rounded top) |
| `model.dresserLow` | 1.82 × 0.80 × 0.64 m | 3,796 | 4 | Body, Handles (rounded case, splayed tapered legs, a shadow line under the top, black bar pulls) |
| `model.wardrobeSingle` | 0.60 × 2.28 × 0.62 m | 2,012 | 2 | Body, Handles (a frame-and-panel door, hinge knuckles, clothes on hangers, a lidded box on top) |
| `model.armoire` | 1.86 × 2.11 × 0.63 m | 6,296 | 3 | Body, Handles (a swept crown moulding and base moulding, turned bun feet, beaded stiles, doors with two raised panels each, turned knobs, an escutcheon) |

### Detail (2026-10-22)

Rebuilt with the forms and the kit (`G.kit.pull`, `knob`, `shaker`, `fillet`), same sizes, centres, tops, parts, clips
and draws. Fronts are `_front` ('plain': rounded; 'shaker': a frame round a recessed panel), hardware is turned or
swept in one piece, tops rounded (`_top`), clothes on wire hangers with sloping shoulders (`_hanger`). A panel set in a
frame stops a hair inside the opening, and no bead or cove wraps a carcass's edge: a vertex buried in another part
bakes black (`shade.py`) and a face with only corner vertices goes dark all over (the shaker panels and the dresser's
side did, in the lit preview).

`check-glb`: 0 failed; warnings: 4 draw calls on the dresser and the low dresser (their drawers carry metal pulls,
as before). Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art bedroom_storage            # every .glb, the pack, check-glb
cargo art bedroom_storage --preview  # previews/bedroom_storage/dresser.png, dresser@drawer_open.png, …
```
