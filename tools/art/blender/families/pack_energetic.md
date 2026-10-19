# Energetic pack — notes

`pack_energetic.py` builds the seven P4 energetic models into `web/public/assets/models/pack_energetic/` and their
entries in `packs/pack_energetic/manifest.json`, an override pack listed in the base manifest. Built
2026-10-14; they sit in Validate › Energetic until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
The contrast shower stood 2 cm into the wall; it is centred on its `Model` `centre` now (`G.settle()`), its back on
the wall face at −0.43 and its front 3 cm short of the tile's, as the wall-standing ones' old parts stood, so the
model and the wall agree within a centimetre.
The contrast shower fills its tile with its own tiled walls, as the bathroom family's showers do.

| Model | Moving part and clip | Played by |
|---|---|---|
| `boulderingWall` | none: still, as the plan says (the holds are the shape) | — |
| `spinBike` | the cranks, pedals and axle in the flywheel's hub: `pedals` turns them once a second, a loop; `ride: pedals` is not set (the rider sits on the still seat) | `ride` (loop) |
| `trampoline` | the mat, a shallow dish from the springs in with its origin at its rim: `bounce` deepens the dip five times (the middle from 0.58 to 0.50 m, where the bouncing feet land, content `stand.height`) and back, 0.625 s, a loop at the residents' 1.6 bounces a second | `bounce` (loop) |
| `streetballHoop` | the net (crossing cords and two rings), pivoting on the rim: `swish` sways it 9° either way and home (`G.rock`, 1 s), after each shot: 1.4 s into the use, then every 3.2 s (`delay`, `every`) | `shoot` |
| `powerBlender` | the jug's lid: `lid` lifts it 9 cm (0.5 s), `hold` 1 s, then it settles back | `blend` |
| `contrastShower` | the rain: a 5 × 5 grid of streams under the square head, scaled to nothing: `water_on` grows them down (0.5 s) | `rinse` |
| `recoveryPod` | the tinted canopy and its rim, hinged at the head end, standing open 40° at rest: `canopy` lowers it over the resident (1.2 s) and lifts it after | `recover` |

**Draw calls: 3 each (the hoop 2).**
- The wall is one still node: its plywood and timber `wood`, the holds and frame plain, the crash mat `fabric`.
- The bike's seat and backrest are the body's `fabric`, the cranks plain.
- The trampoline's pad is the body's `fabric`, the mat plain matte.
- The hoop is all plain, the net plain.
- The blender's jug and its two glasses are `glass` (the smoothies show through); the fruit bowl and board plain
  satin; the lid plain.
- The shower's glass panels are `glass`, the towel plain satin; the rain plain gloss.
- The pod is all plain on its still body; the canopy part takes its `glass` and its plain rim (2 draws).

Not done, as the plan allows:
- the blender's blades loop "L blades (E1)": they would turn inside the smoothie, which hides them, and a second
  moving part is one more draw;
- the trampoline's "Until then U: dips once" interim: the E1 loop is built instead;
- the hoop's net sways when the shooting starts (and again as the clip plays back at the end), not after each
  shot: the game has no per-shot event.

Colour parts: the plan's "Frame, Pads" where they fit; the blender (a kitchen unit) and the shower take names
that say what they colour: the blender `blend_cab` "Frame", `blend_door` "Doors", `motor` "Blender"; the shower
`shower_frame` "Frame", `tile` "Tiles", `shower_mat` "Mat".

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.energetic.boulderingWall` | 1.87 × 2.28 × 0.88 m | 3,772 | 3 | Frame, Pads, Panel |
| `model.energetic.spinBike` | 0.62 × 1.20 × 1.60 m | 2,536 | 3 | Frame, Pads, Flywheel |
| `model.energetic.trampoline` | 1.90 × 0.63 × 1.90 m | 3,420 | 3 | Frame, Pads, Mat |
| `model.energetic.streetballHoop` | 0.86 × 2.83 × 0.89 m | 3,668 | 2 | Frame, Pads, Base |
| `model.energetic.powerBlender` | 0.82 × 1.32 × 0.63 m | 3,972 | 3 | Frame, Doors, Blender |
| `model.energetic.contrastShower` | 0.90 × 2.15 × 0.90 m | 2,944 | 3 | Frame, Tiles, Mat |
| `model.energetic.recoveryPod` | 0.86 × 1.18 × 1.86 m | 2,900 | 3 | Frame, Pads, Boots |

`check-glb`: 7 models, 0 failed, 2 warnings, both by design:
- the trampoline's height (0.63 m, old part 0.83 m): the old part's ball sat on the mat; here it is on the
  grass by a leg (a ball on the mat would float while it dips);
- the recovery pod's height (1.18 m, old part 0.82 m): the old part showed the canopy closed; here it stands
  open at rest (40°, so it stays clear of the wall behind) and closes for `recover`.

Khronos validator: 0 errors, 0 warnings each. Checked: the gallery next to the old parts; the clips bench at rest
and open on WebGPU (the wall from the side for its overhang, the bike's cranks half way round, the trampoline's dip
at its deepest), colours on WebGPU and WebGL; in play (`wants-asking`, WebGPU by day), each used by the household's
resident: the bike ridden (seated, the cranks turning), the blender used (lid up and back), the pod (lying, the
canopy closed over the head and lifted after), the wall climbed, the shower (rain on), the trampoline bounced on
(the feet on the dipping mat) and the hoop shot at; the pod turned to four facings and a turned hoop in the yard;
`office-night` on WebGL (the shower used at night).

## Rebuild

```
cargo art pack_energetic            # the seven .glb, the pack, check-glb
cargo art pack_energetic --preview  # previews/<id>.png, <id>@<clip>.png
```
