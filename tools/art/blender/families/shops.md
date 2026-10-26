# Shops — notes

`shops.py` builds the five CM4 shop and café models into `web/public/assets/models/shops/` and their entries in
`packs/shops/manifest.json`: the shop counter (`model.shopCounter`), the bakery case (`model.bakeryCase`), the grocery
shelves (`model.groceryShelf`), the café table (`model.cafeTable`) and the café chair (`model.cafeChair`). Built 2026-10-14; they sat in
Validate › Shops until checked in play (back to community / shops).

They stand on town lots, seen from far and by the dozen, so each has a far version (`<id>.far.glb`): `_far(build)`,
the model at rest, joined, the vertices that sit on one another merged, decimated to a quarter with its feet and
outermost faces kept; the café table's by hand (`G.far_of`: coarser, no veins or curls), as decimating its thin iron
took its column and legs away. Each is built where the game places it (`F`, game axes): the
counter, the case and the shelves back on their 2 × 1 tiles (`G.settle(centre=False)` and `centre=` on the `Model`),
the counter's worktop and the case's deck at the back on the wall face (−0.43: their parts are placed 4 cm forward,
`GameFrame(shift=0.04)`, as are the old parts, which reached 4 cm into the wall before),
the table centred, the chair's seat 2 cm forward. The family checks the café table's top (0.745 m) and the chair's
seat (0.47 m, 2 cm forward) against `content/community.json` before it builds.

## The shop counter

An oak counter (`wood`) on a walnut plinth: a walnut worktop with a rounded nose and a cove under it, three
frame-and-panel bays in front (`kit.shaker`, walnut frames round raised oak panels), two shaker cupboard doors with
turned knobs at the back. On the worktop a till (the keypad sloping to the shopkeeper, the customer's display on a post), a receipt printer with its
paper curling out, a chrome service bell, a sweet jar with a shoulder and a domed lid, and a magazine rack with shaped
ends, a lip and four magazines. The till is the shopkeeper's (0.5.0, I-community-09): its keypad slopes up to the
front, to whoever stands behind the counter, and the customer's display stands on its post at the till's front.
One moving part:

- **The cash drawer** (`cash_drawer`): its front under the keypad, at the back, and a tray of notes and coins hidden
  in the till's case at rest. `till` slides it 10 cm out to the back (to the shopkeeper; at home against a wall it
  stops at the wall's face) in 0.8 s.

`motions`: `{'uses': {'treat': {'clip': 'till', 'delay': 3.9, 'hold': 1.1}}}`: as the game's shopkeeper takes the
money (`characters/clips/serve.ts`, paid at 4.2 s). The sweet jar is
opaque gloss with the sweets pressed into its sides, as if seen through it: real glass would have been a fourth draw
call.

## The bakery case

A cream cabinet (three raised panels, `kit.shaker`, under a band) with a walnut deck and sill, a glass display on walnut
ends (the front glass curving up and back into the glass top, a glass shelf on brackets) and bakes on doilies and
trays with price cards: glazed buns, tarts in fluted cases, scored loaves, banded cakes with cherries, biscuits,
crescent croissants (one swept piece each, plump in the middle) and cupcakes in fluted cups under piped frosting. The back, the server's side, is two glass panes in two tracks. One moving part:

- **The back door** (`back_door`): the right-hand pane (as seen from the front), in the front track. `door` slides
  it 84 cm left, in front of the fixed pane, in 1 s.

`motions`: `pastry` and `takeCake` each `{'clip': 'door', 'delay': 1.0, 'hold': 1.4}`: the baker reaches in through
it a second into the sale (`characters/clips/serve.ts`), and it shuts as they hand the pastry over. The door is plain glass with no handle, so the model stays at 3 draws.

## The rest

- **Grocery shelves**: a metal gondola (shaped sides rounded at the top front, a slotted back, a dark kick plate, a
  top cap with a coloured header strip) with four shelves, each with a price channel (a dark lip, a white strip and
  coloured tags); packets with a label and a band, tins with a label round the front one, bottles with a shoulder,
  neck and cap on three shelves, and sandwich packs (triangles with a filling and a sticker) on the bottom one. Its back is on the wall face (−0.43), its front 5 cm past its tile's middle: centred on
  game depth −0.19, within 2 cm of its old part.
- **Café table**: a round marble top (`gloss`) with a moulded edge and grey veins, on a fluted cast-iron column with a
  turned knop, a collar and three scrolled spider arms under the top, a turned hub and three curved legs, tapering, on
  round feet. Its top is at 0.745 m, the content's `surface.height`.
- **Café chair**: a bentwood chair (`wood`) with a cane seat (`fabric`, dished a little in its ring): one bent piece,
  tapering to the feet, runs from the back feet up into the bow of the back, an inner loop inside it, two tapering
  splayed front legs with little bent brackets up to the seat, and a ring bracing all four. All swept (`F.sweep`; they
  were Blender curve objects converted to meshes). The seat's rim is
  at 0.47 m, the content's `seat.height`. Residents sit on it at the right height (checked in play, sitting and with a
  coffee at the table).

| Model | Size (w × h × d) | Triangles (far) | Draws | Colour parts |
|---|---|---|---|---|
| `model.shopCounter` | 1.96 × 1.28 × 0.79 m | 6,588 (1,533) | 3 | Counter, Trim |
| `model.bakeryCase` | 1.94 × 1.29 × 0.86 m | 7,600 (1,742) | 3 | Counter, Trim |
| `model.groceryShelf` | 1.92 × 1.72 × 0.48 m | 7,372 (1,666) | 1 | Shelves, Trim |
| `model.cafeTable` | 0.62 × 0.75 × 0.62 m | 1,864 (644) | 1 | Top, Frame |
| `model.cafeChair` | 0.44 × 0.95 × 0.47 m | 1,932 (482) | 2 | Frame, Seat |

Detail pass 2026-10-22 (the forms and the kit). Before it: the counter 5,640, the case 6,604, the shelves 6,196, the
table 1,704, the chair 1,840. The café table and chair, placed by the dozen (18 chairs, 9 tables in the town), stay
under half their size class's target (2,000); the counter, the case and the shelves (3 to 7 placed) under the full one
(8,000).

Draws: the counter's still body is wood and plain parts (2) plus the cash drawer (1); the case's still body is plain
and glass (2) plus the back door's glass (1). `check-glb`: 5 models, 0 failed, no warnings. Khronos validator: 0
errors, 0 warnings on all ten files (models and far versions).

## Rebuild

```
cargo art shops            # the five .glb and their .far.glb, the pack, check-glb
cargo art shops --preview  # previews/shops/<id>.png, <id>@<clip>.png, <id>.far.png
```

The Workbench previews draw glass opaque: the case's bakes and its back door only show in the game (the clips bench,
`/clips.html?keys=model.bakeryCase&alpha=-2&t=99` looks at its back with the door open).
