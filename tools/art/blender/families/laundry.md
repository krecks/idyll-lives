# Laundry — notes

`laundry.py` builds the six B4 laundry models into `web/public/assets/models/laundry/` and their entries in
`packs/laundry/manifest.json`: the washing machine (`model.laundryWasher`), the tumble dryer
(`model.laundryDryer`), the towel rail (`model.towelRail`), the towel ladder (`model.towelLadder`), the laundry
basket (`model.laundryBasket`) and the bath mat (`model.bathMat`); and since 0.5.0 three things carried doing the
laundry (content `items`, shown in the hands by `brings` and `holds`): the basket of washing
(`item.washingBasket`), an armful of dry washing (`item.washingArmful`) and a towel being folded
(`item.washingPiece`). Built 2026-10-14; they sat in Validate › Laundry
until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
`Model`): the machines, the ladder and the basket against the wall, the mat flat and centred. The washer, the
dryer and the towel ladder were built 3 cm into the wall, as the old parts; they and the old parts stand
on the wall face now (`G.GameFrame(shift=0.03)`, 2026-10-14). The machines keep 4 draw calls (cabinet, door ring,
door glass, drum): an opaque window would save one, but then the drum can't be seen turning.

## The washer and the dryer

A gloss cabinet with a control strip (a soap drawer or a filter, a display and light, a knob) and a dark seal
round the porthole. Two moving parts:

- **The door** (`door`): a ring with a window (`glass`) and a handle on the right, hinged on its left edge.
  `door_open` swings it 110° in 1 s; `hold` 2.5 s, so it shuts while the resident is still loading.
- **The drum's face** (`drum`): a dark disc with three paddles and the washing, just behind the window, its
  origin on the axis. `drum_spin` turns it once in 1.2 s (`G.spin`); on the washer only.

`motions` (2026-10-21, review A-bathroom-03): the washer `{'uses': {'laundry': ['door_open', {'clip': 'drum_spin',
'delay': 3.5, 'every': 1.1}]}, 'hold': 2.5}`: the drum turns once the door has shut (3.5 s in), round after round
(`every` a hair under the turn, so there's no pause) till the wash ends; it turned behind the open door before. The
dryer `{'uses': {'laundry': 'door_open'}, 'hold': 2.5}`: "Fold the laundry" opens it to take the washing out, and
the drum stays still (it tumbled the whole fold). By the interaction id, as the `clean` tag is shared with other
chores. The washing on the top (folded towels, the
dryer's basket of dry washing) is plain satin, so it costs no draw call.

**Draw calls: 4** each (the most allowed, as the microwave): the cabinet, the door ring, the door's glass and
the drum. `check-glb` warns of it. An opaque window would save one, but then the drum couldn't be seen
turning.

## Running on their own (0.5.0, I-bathroom-07)

The machines run on their own once loaded (content `runs`). Their motions now: `{'uses': {'laundry': 'door_open',
'unload': 'door_open'}, 'hold': 2.5, 'running': 'drum_spin'}`: loading and taking the washing out open the door,
which shuts again after 2.5 s, and the drum goes round while the machine runs with nobody there, from rest to rest
(`running`), a turn each 1.2 s on the washer (unchanged) and each 1.8 s on the dryer (new: its drum didn't turn
before). The washer's file is as it was.

The carried things, each one draw, base where it rests on the palms (`hold: both`):

| Model | Size (w × h × d) | Triangles | What it is |
|---|---|---|---|
| `item.washingBasket` | 0.42 × 0.28 × 0.30 m | 1,688 | an oval wicker tub with bands, a rim and hand holes, washing heaped above it (all satin) |
| `item.washingArmful` | 0.34 × 0.19 × 0.24 m | 1,100 | three towels folded loosely, a crumpled shirt and a sock on top (satin) |
| `item.washingPiece` | 0.40 × 0.17 × 0.25 m | 176 | a towel half folded, its far half lifted over towards the hands, a woven band at its end (fabric) |

`check-glb`: 0 failed. Khronos validator: 0 errors, 0 warnings on the dryer and the three.

## The rest

- **Towel rail** (wall): two chrome bars on posts and wall plates, a bath towel over the top bar and a hand
  towel over the lower one. It hangs at its real height (bars at 0.95 and 1.2 m), its back on the wall face:
  `G.settle(centre=False, floor=False)` and `floor=False` on its `Model`. `check-glb` warns that it floats
  (right for a wall thing) and that it's 2 cm shallower than its old part.
- **Towel ladder**: two birch rails leaning 10.66° back to the wall, five rungs, a towel over three of them.
- **Laundry basket**: a wicker tub (`fabric`) with a rim, woven bands and side handles, clean washing on top.
- **Bath mat** (rug): a soft mat with a raised pile and two stripes, 2.5 cm high.

## Detail (2026-10-22)

Rebuilt with the forms, same sizes, centres, parts, clips (the washer's drum loop: `delay` 3.5, `every` 1.1 for its
1.2 s turn, unchanged), paint and draws:

- **Washer and dryer:** a rounded cabinet under a top panel, a kick plinth; the control strip's soap drawer (or water
  tank) with its marks, the time on the display, three buttons, the program knob turned with a pointer in a ring of
  settings; rubber bellows turned round the opening. The door ring turned in one profile, its window domed, a grip bent
  from flat bar. The drum's face: a ring of holes, three shaped lifters, lumpy washing (jittered). On top: folded
  towels (washer), a basket bound with bands and a heap of washing with a sock (dryer).
- **Towel rail:** each bar bent from one tube into the wall (`kit.fillet`), roses at the wall, rounded wall plates;
  the towels draped over the bars in one piece, each with a woven hem.
- **Towel ladder:** squared rails rounded at the top, round rungs, the towels draped over in one piece, two hemmed.
- **Laundry basket:** a lofted wicker tub woven in checked rows, a rolled rim, a plaited foot band, two loop handles;
  folded towels and a crumpled shirt in it.
- **Bath mat:** a soft mat and a puffed pile, its stripes, a stitched seam round the pile.

| Model | Size (w × h × d) | Triangles (before → now) | Draws | Colour parts |
|---|---|---|---|---|
| `model.laundryWasher` | 0.60 × 0.98 × 0.65 m | 3,652 → 3,644 | 4 | Body, Door, Panel |
| `model.laundryDryer` | 0.60 × 1.09 × 0.65 m | 4,284 → 4,744 | 4 | Body, Door, Panel |
| `model.towelRail` | 0.73 × 0.59 × 0.10 m | 1,312 → 1,728 | 2 | Rail, Towels, Hand towel |
| `model.towelLadder` | 0.52 × 1.70 × 0.36 m | 1,240 → 1,272 | 2 | Ladder, Towels, Second towel, Third towel |
| `model.laundryBasket` | 0.54 × 0.40 × 0.42 m | 3,872 → 3,640 | 2 | Basket, Rim & weave |
| `model.bathMat` | 0.76 × 0.03 × 0.50 m | 592 → 840 | 1 | Mat, Pile |

The new hem colours (`towel_hem`, `towel_hem2`) are `fabric` like the towels they're on, in the same draw.

`check-glb`: 6 models, 0 failed. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art laundry            # the six .glb, the pack, check-glb
cargo art laundry --preview  # previews/laundry/<id>.png, <id>@<clip>.png
```
