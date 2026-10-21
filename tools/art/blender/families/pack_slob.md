# Slob pack — notes

`pack_slob.py` builds the six P16 slob models into `web/public/assets/models/pack_slob/` and their entries in
`packs/pack_slob/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sat in
Validate › Slob until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
backs on the wall's face where they stand against one. Seats keep the old part's: the beanbag's dimple and the
nightstand's padded top at about 0.45, the recliner's seat top at 0.45 in the middle.

| Model | Moving part and clip | Played by |
|---|---|---|
| `microwaveCart` | the microwave's door, window and handle, hinged on its outer edge: `door_open` swings it 100° open (0.6 s), `hold` 1.5 s (open and shut) | `nuke`, `invent` |
| `beanbag` | the whole bag (body, back, dimple, seam), origin on the floor: `squash` settles it to 86 % high, 4 % wider (0.5 s) | `flop`, `snooze` |
| `snackNightstand` | the top side drawer full of chip bags and a cookie: `drawer_open` slides it 16 cm out to the right (0.6 s) | `midnightSnack` |
| `spritzStation` | the trigger spray bottle, hinged on its foot: `spritz` tips it 35° to the front (0.4 s) | `spritz` |
| `arcadeCabinet` | the joystick, its origin in the deck: `joystick` waggles it 18° each way, side to side, in 0.6 s (a loop) | `chaseHighScore`, `quickRound` (`loops`) |
| `gamingRecliner` | the tall back with its stripe and headrest, hinged at the seat's back: `recline` lays it back 20° (1 s); the cup holder with a soda on the right armrest: `cup_out` swings it 110° out (0.6 s) | `marathon`; `armrestSoda` |

The arcade's joystick is the loop the table asks for (E1). The recliner's back reclines and the sitter rides
it (`ride: recline`), tipping back with it.

**Draw calls: 2–3 each.** Nothing here needs more:
- the cart, the beanbag and the arcade cabinet are all plain (2: the body and the moving part); the beanbag's
  vinyl is satin ("wipe-clean");
- the nightstand's padded top and the spritz station's towels and laundry are `fabric` (3);
- the recliner is leatherette satin, its two moving parts the 2nd and 3rd draw; the mini-fridge's door is plain
  gloss, no glass;
- the spritz station's round mirror is chrome, as the old part's (no mirror glass: it is not the point here).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.slob.microwaveCart` | 0.72 × 1.10 × 0.78 m | 2,944 | 2 | Body |
| `model.slob.beanbag` | 0.89 × 0.66 × 0.87 m | 2,116 | 2 | Body |
| `model.slob.snackNightstand` | 0.90 × 0.45 × 0.73 m | 1,652 | 3 | Body, Top |
| `model.slob.spritzStation` | 0.77 × 1.58 × 0.80 m | 1,948 | 3 | Body |
| `model.slob.arcadeCabinet` | 0.77 × 1.88 × 0.78 m | 1,676 | 2 | Body, Sides |
| `model.slob.gamingRecliner` | 1.77 × 1.10 × 0.87 m | 2,764 | 3 | Body, Stripes |

Body is the cart's red frame, the bag, the nightstand, the spritz station's purple shelves, the arcade cabinet and
the recliner's leatherette. The nightstand's top, the cabinet's side panels and the recliner's stripes are extra
colour parts where they fit.

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Clips bench: every
clip at rest and open on WebGPU and WebGL (the joystick mid-waggle). In play (`wants-asking`, WebGPU, in the yard):
the beanbag (flop, seated), the recliner (marathon, seated), the nightstand (midnightSnack, seated on it, the
drawer out), the cart (nuke) and the arcade (chaseHighScore) used by a resident; the arcade cabinet at all four
facings, the cart, nightstand and spritz station turned; the recliner, beanbag, cart and nightstand bought
coloured; the same at night (`office-night`, WebGL).

## Rebuild

```
cargo art pack_slob            # the six .glb, the pack, check-glb
cargo art pack_slob --preview  # previews/<id>.png, <id>@<clip>.png
```
