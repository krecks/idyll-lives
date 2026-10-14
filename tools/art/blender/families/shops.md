# Shops — notes

`shops.py` builds the five CM4 shop and café models into `web/public/assets/models/shops/` and their entries in
`packs/shops/manifest.json`: the shop counter (`model.shopCounter`), the bakery case (`model.bakeryCase`), the grocery
shelves (`model.groceryShelf`), the café table (`model.cafeTable`) and the café chair (`model.cafeChair`). They replace
the primitive-part placeholders of `tools/art/community.py`, which stay as the fallback. Built 2026-10-14; they sit in
Validate › Shops until checked in play (back to community / shops).

They stand on town lots, seen from far and by the dozen, so each has a far version (`far=0.25`: `<id>.far.glb`, the
model at rest, joined and decimated to a quarter). Each is built where its placeholder stands (`F`, game axes): the
counter, the case and the shelves back on their 2 × 1 tiles (`G.settle(centre=False)` and `centre=` on the `Model`),
the table centred, the chair's seat 2 cm forward. The family checks the café table's top (0.745 m) and the chair's
seat (0.47 m, 2 cm forward) against `content/community.json` before it builds.

## The shop counter

An oak counter (`wood`) with walnut rails, stiles, plinth and worktop, three raised panels in front and two cupboard
doors at the back. On the worktop a till (keypad sloping back, a display on a post), a sweet jar and a magazine rack.
One moving part:

- **The cash drawer** (`cash_drawer`): its front under the keypad and a tray of notes and coins hidden in the till's
  case at rest. `till` slides it 20 cm out to the front (towards the resident) in 0.8 s.

`motions`: `{'uses': {'treat': 'till'}, 'hold': 1.5}` (the interaction id: its `idle` tag is shared). The sweet jar is
opaque gloss with the sweets pressed into its sides, as if seen through it: real glass would have been a fourth draw
call.

## The bakery case

A cream cabinet (three panels under a band) with a walnut deck, a glass display on walnut ends (a sloping front, a
glass top, a glass shelf on brackets) and bakes on doilies and trays: buns, tarts, loaves, cakes, biscuits,
croissants and cupcakes. The back, the server's side, is two glass panes in two tracks. One moving part:

- **The back door** (`back_door`): the right-hand pane (as seen from the front), in the front track. `door` slides
  it 84 cm left, in front of the fixed pane, in 1 s.

`motions`: `{'uses': {'pastry': 'door'}, 'hold': 2.0}`. The table gives no hold; 2 s shuts the door again while the
resident is still eating. The door is plain glass with no handle, so the model stays at 3 draws.

## The rest

- **Grocery shelves**: a metal gondola (sides, back, a dark kick plate and top cap) with four shelves, dark lips and
  price tags; packets with labels, tins and jars on three shelves and sandwich packs (triangles with a filling and a
  sticker) on the bottom one. Its back is on the wall face (−0.43), its front 5 cm past its tile's middle: centred on
  game depth −0.19, within 2 cm of its placeholder.
- **Café table**: a round marble top (`gloss`) on a cast-iron column with a knop, a spider under the top and three
  curved feet. Its top is at 0.745 m, the content's `surface.height`.
- **Café chair**: a bentwood chair (`wood`) with a cane seat (`fabric`): one bent piece runs from the back feet up into
  the bow of the back, an inner loop inside it, two splayed front legs and a ring bracing all four. The seat's rim is
  at 0.47 m, the content's `seat.height`. Residents sit on it at the right height (checked in play, sitting and with a
  coffee at the table).

| Model | Size (w × h × d) | Triangles (far) | Draws | Colour parts |
|---|---|---|---|---|
| `model.shopCounter` | 1.96 × 1.28 × 0.78 m | 5,640 (1,378) | 3 | Counter, Trim |
| `model.bakeryCase` | 1.94 × 1.29 × 0.86 m | 6,604 (1,581) | 3 | Counter, Trim |
| `model.groceryShelf` | 1.92 × 1.72 × 0.48 m | 6,196 (1,368) | 1 | Shelves, Trim |
| `model.cafeTable` | 0.62 × 0.75 × 0.62 m | 1,704 (426) | 1 | Top, Frame |
| `model.cafeChair` | 0.45 × 0.95 × 0.47 m | 1,840 (460) | 2 | Frame, Seat |

Draws: the counter's still body is wood and plain parts (2) plus the cash drawer (1); the case's still body is plain
and glass (2) plus the back door's glass (1). `check_glb.py`: 5 models, 0 failed, no warnings. Khronos validator: 0
errors, 0 warnings on all ten files (models and far versions).

## Rebuild

```
python3 -B tools/art/blender/build.py shops            # the five .glb and their .far.glb, the pack, check_glb
python3 -B tools/art/blender/build.py shops --preview  # previews/<id>.png, <id>@<clip>.png, <id>.far.png
```

The Workbench previews draw glass opaque: the case's bakes and its back door only show in the game (the clips bench,
`/clips.html?keys=model.bakeryCase&alpha=-2&t=99` looks at its back with the door open).
