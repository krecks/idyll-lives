# Laundry — notes

`laundry.py` builds the six B4 laundry models into `web/public/assets/models/laundry/` and their entries in
`packs/laundry/manifest.json`: the washing machine (`model.laundryWasher`), the tumble dryer
(`model.laundryDryer`), the towel rail (`model.towelRail`), the towel ladder (`model.towelLadder`), the laundry
basket (`model.laundryBasket`) and the bath mat (`model.bathMat`). They replace the primitive-part placeholders
of `catalog.py` / `catalog2.py`, which stay as the fallback. Built 2026-10-14; they sit in Validate › Laundry
until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
`Model`): the machines, the ladder and the basket against the wall, the mat flat and centred. The washer, the
dryer and the towel ladder were built 3 cm into the wall, as their placeholders; they and the placeholders stand
on the wall face now (`G.GameFrame(shift=0.03)`, 2026-10-14). The machines keep 4 draw calls (cabinet, door ring,
door glass, drum): an opaque window would save one, but then the drum can't be seen turning.

## The washer and the dryer

A gloss cabinet with a control strip (a soap drawer or a filter, a display and light, a knob) and a dark seal
round the porthole. Two moving parts:

- **The door** (`door`): a ring with a window (`glass`) and a handle on the right, hinged on its left edge.
  `door_open` swings it 110° in 1 s; `hold` 2.5 s, so it shuts while the resident is still loading.
- **The drum's face** (`drum`): a dark disc with three paddles and the washing, just behind the window, its
  origin on the axis. `drum_spin` turns it once in 1.2 s, a loop (`G.spin`) for the whole use.

`motions`: `{'uses': {'laundry': 'door_open'}, 'hold': 2.5, 'loops': {'laundry': 'drum_spin'}}` (the
interaction id, as the `clean` tag is shared with other chores). The washing on the top (folded towels, the
dryer's basket of dry washing) is plain satin, so it costs no draw call.

**Draw calls: 4** each (the most allowed, as the microwave): the cabinet, the door ring, the door's glass and
the drum. `check_glb.py` warns of it. An opaque window would save one, but then the drum couldn't be seen
turning.

## The rest

- **Towel rail** (wall): two chrome bars on posts and wall plates, a bath towel over the top bar and a hand
  towel over the lower one. It hangs at its real height (bars at 0.95 and 1.2 m), its back on the wall face:
  `G.settle(centre=False, floor=False)` and `floor=False` on its `Model`. `check_glb.py` warns that it floats
  (right for a wall thing) and that it's 2 cm shallower than its placeholder.
- **Towel ladder**: two birch rails leaning 10.66° back to the wall, five rungs, a towel over three of them.
- **Laundry basket**: a wicker tub (`fabric`) with a rim, woven bands and side handles, clean washing on top.
- **Bath mat** (rug): a soft mat with a raised pile and two stripes, 2.5 cm high.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.laundryWasher` | 0.60 × 0.98 × 0.65 m | 3,652 | 4 | Body, Door, Panel |
| `model.laundryDryer` | 0.60 × 1.09 × 0.65 m | 4,284 | 4 | Body, Door, Panel |
| `model.towelRail` | 0.73 × 0.60 × 0.10 m | 1,312 | 2 | Rail, Towels, Hand towel |
| `model.towelLadder` | 0.52 × 1.71 × 0.36 m | 1,240 | 2 | Ladder, Towels, Second towel, Third towel |
| `model.laundryBasket` | 0.54 × 0.39 × 0.42 m | 3,872 | 2 | Basket, Rim & weave |
| `model.bathMat` | 0.76 × 0.03 × 0.50 m | 592 | 1 | Mat, Pile |

`check_glb.py`: 6 models, 0 failed. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
python3 -B tools/art/blender/build.py laundry            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py laundry --preview  # previews/<id>.png, <id>@<clip>.png
```
