# Example pack — notes

`pack_example.py` builds the two P5 example-pack models (`content/packs/example.json`) into
`web/public/assets/models/pack_example/` and their entries in `packs/pack_example/manifest.json`, an override pack
listed after the base manifest's own. Their keys are the existing `model.chefStove` and `model.whirlpoolTub`.
Built 2026-10-14; they sat in Validate › Example until checked in play (back to Kitchen and Wellness).

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
The stove's back is on the wall's face (depth −0.43; the old part's sat 2.5 cm into the wall). The tub keeps the
old part's 1.86 m square and its bench under the two bathers (seat 0.45, the default; they sit at x ±0.5 in the
middle, facing the front), with the water up to 0.65.

| Model | Moving part and clip | Played by |
|---|---|---|
| `chefStove` | the left oven door, on its bottom hinge: `oven_open` drops it 85° (1.2 s), shut again after 3 s | `practise`, the `bake` tag |
| `chefStove` | a knob and the front middle burner's flame ring (built full size, scaled to nothing): `burner_on` turns the knob 90° and grows the flame (0.5 s), for the whole use | `cook`, the `stir` tag |
| `whirlpoolTub` | the water, origin under the footwell floor, flat at rest: `fill` raises it (0.8 s), for the whole soak | `soak` |
| `whirlpoolTub` | the bubbles over the jets, origin under the floor, flat at rest: `bubbles` (a loop, `G.squash` there and back, 1.6 s) swells them up through the water to the surface and back | `soak` (loop) |

`motions`: the stove `{'uses': {'cook': 'burner_on', 'stir': 'burner_on', 'practise': {'clip': 'oven_open', 'hold': 3},
'bake': …}}` (by the cooking step's tag too, for a meal started elsewhere; 2026-10-21: a bake no longer lights the
hob, nor a meal on the hob opens the oven); the tub
`{'uses': {'soak': 'fill'}, 'loops': {'soak': 'bubbles'}}`. The bubbles' rise takes as long as `fill` from the same
origin, so on the first round they ride up on the rising water instead of floating over an empty tub.

**Draw calls: 4 each** (`check-glb` warns "aim for 3"), on purpose:
- the stove: its body (all plain finishes) plus the door, the knob and the flame, the plan's "knob and flame ring" —
  as the gas range and the range cooker (`kitchen_cooking.py`) do;
- the tub: its body (plain, plus the surround's `wood`) plus the water and the bubbles — the plan's extra loop (E1).
  Making the cedar surround plain satin would bring it to 3.

The old part's glass water surface, fabric towels and foliage plant are gloss, satin and satin here (no glass).
The old part's step was buried in the tub's surround; there is none.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.chefStove` | 1.62 × 2.55 × 0.66 m | 6,828 | 4 | Body, Handles & knobs, Tiles |
| `model.whirlpoolTub` | 1.86 × 0.93 × 1.86 m | 7,104 | 4 | Tub, Surround, Headrests |

`check-glb`: 2 models, 0 failed; the two 4-draw warnings above. Khronos validator: 0 errors, 0 warnings each.
Clips bench at rest, open and mid-loop on WebGPU and WebGL, coloured. In play (`wants-asking`, WebGPU): the stove in
the living room cooked on (door down, then shut, flame on throughout); the tub in the yard soaked in (water up,
bubbles, the resident sitting in the water). At night (`office-night`, WebGL): the stove in all four facings and
the tub in the yard.

## Rebuild

```
cargo art pack_example            # the two .glb, the pack, check-glb
cargo art pack_example --preview  # previews/pack_example/<id>.png, <id>@<clip>.png
```
