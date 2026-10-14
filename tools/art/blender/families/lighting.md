# Lighting — notes

`lighting.py` builds the 21 lamps of plan/11-lighting.md into `web/public/assets/models/lighting/` and their
entries in `packs/lighting/manifest.json`, over the primitive-part placeholders of `catalog.py`, `catalog2.py`
and the base manifest (`model.lamp`), which stay as the fallback. Built 2026-10-14; they sit in Validate ›
Ceiling Lights, Wall Lights, Floor Lamps and Table Lamps until signed off.

- **LT1, ceiling:** `ceilingPendant`, `ceilingLight`, `chandelier`, `pendantIndustrial`, `ceilingFan`,
  `trackLights`, `paperLantern`. They hang from 2.8 m at their real height (`G.settle(floor=False)`,
  `floor=False` on the `Model`), centred.
- **LT2, wall:** `wallSconce`, `wallLampArm`, `wallLightOutdoor`. Their back is on the wall face (−0.43), at
  their real height (`G.settle(centre=False, floor=False)`, `centre=` on the `Model`).
- **LT3, floor:** `lamp`, `floorLampSimple`, `lampFloorArc`, `lampFloorTripod`, `lampTorchiere`. The arc lamp's
  base stands in the back corner, its back on the wall face (`centre=`), as its placeholder now does.
- **LT4, table:** `tableLamp` (a lamp on its own oak side table, a floor object), and the surface lamps
  `lampTableRound`, `lampTableSquare`, `lampBanker`, `lampMushroom`, `lampLantern`.

## The light

The light is the content's `light`; the model only shows where it comes from. `_bulb` (and `_near_light`)
asserts each bulb within 25 cm of the content's `light.height`, read from `base.json`, `furniture.json` and
`catalog2/lighting.json`, so a remodel can't drift away from it. Most are within 8 cm; the pendant's light
(2.1 m) sits at the top of its dome, 20 cm above the bulb, as the placeholder had it.

## Night glow

The shades, diffusers, globes, the paper and the candle's flame have `glow` (E8): they light up from dusk with
the room's lamps, nothing by day. Metal shades (the industrial pendant, the swing-arm lamp, the arc lamp, the
banker's lamp) glow only on their white `liner`, a thin shell just inside the shade.

## Clips (both ambient, `idle`)

- **Ceiling fan** (`blades`): the five blades and their irons, origin on the axis; `blades_turn` turns them
  once in 2.5 s (`G.spin`).
- **Paper lantern** (`lantern`): the cord, the paper globe and its ribs, origin on the canopy; `sway` rocks
  them 2.5° in 2 s (`G.rock`), about a pendulum's period for its length.

Not on Low (E2); at rest they look as the placeholder.

## Shapes

- `_lathe` turns a placeholder's `lathe` (profile in fractions of its size) into a `G.revolve`, closing the
  profile and optionally adding the liner. `top=True` puts the origin at the top, to tilt the swing-arm
  shade about where it hangs.
- `_drum` is an open drum or cone shade with a wall thickness (4 segments turned 45° for the square ones).
- `_trough` is the banker's half-round shade, open below, coming 20° past the axis so the tube bulb hides.

## Checks

`check_glb.py`: 21 ok, 0 failed. Its warnings, all expected:

- the ceiling and wall lights float above their origin (they hang);
- `floorLampSimple`, `lampTableSquare` and `lampLantern` are about half their placeholder's width: the gate
  measures the placeholder's 4-sided shade or roof (a cylinder of 4 segments turned 45°) by its unturned box;
  the drawn sizes match;
- `wallLampArm` is 5 cm lower than the gate's estimate of the tilted placeholder shade; the drawn shades agree
  (checked side by side in the gallery), so the placeholder is kept. Fixed 2026-10-14: the track lights'
  placeholder aims its heads as the model's (it was 3 cm shallower), and the outdoor light's lens no longer reaches
  3 cm into the wall behind its plate (its hidden back half is flattened onto the wall face), so it is 0.11 m deep,
  as its placeholder.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| ceilingPendant | 0.43 × 0.95 × 0.43 | 1,080 | 1 | Body, Shade |
| ceilingLight | 0.36 × 0.60 × 0.36 | 1,040 | 1 | Body, Shade |
| chandelier | 0.94 × 0.76 × 0.90 | 3,788 | 1 | Body, Globes |
| pendantIndustrial | 0.38 × 0.87 × 0.38 | 1,592 | 1 | Body, Shade |
| ceilingFan | 1.12 × 0.39 × 1.18 | 2,328 | 3 | Body, Blades |
| trackLights | 0.94 × 0.30 × 0.19 | 936 | 1 | Body, Shade |
| paperLantern | 0.54 × 1.00 × 0.54 | 2,204 | 3 | Paper, Ribs |
| wallSconce | 0.20 × 0.25 × 0.27 | 764 | 2 | Body, Shade |
| wallLampArm | 0.20 × 0.30 × 0.42 | 1,044 | 1 | Body, Shade |
| wallLightOutdoor | 0.22 × 0.33 × 0.11 | 1,220 | 2 | Body, Shade |
| lamp | 0.45 × 1.81 × 0.45 | 1,948 | 2 | Stand, Shade, Fittings |
| floorLampSimple | 0.28 × 1.60 × 0.28 | 396 | 2 | Stand, Shade |
| lampFloorArc | 0.85 × 1.86 × 0.81 | 3,240 | 1 | Stand, Shade, Base |
| lampFloorTripod | 0.55 × 1.61 × 0.56 | 848 | 3 | Stand, Shade |
| lampTorchiere | 0.40 × 1.85 × 0.40 | 1,112 | 2 | Stand, Shade, Base |
| tableLamp | 0.66 × 0.92 × 0.42 | 1,292 | 3 | Base, Shade, Table |
| lampTableRound | 0.34 × 0.58 × 0.34 | 1,152 | 2 | Base, Shade |
| lampTableSquare | 0.21 × 0.54 × 0.21 | 376 | 2 | Base, Shade |
| lampBanker | 0.30 × 0.39 × 0.16 | 536 | 1 | Base, Shade |
| lampMushroom | 0.30 × 0.36 × 0.30 | 916 | 1 | Base, Shade |
| lampLantern | 0.16 × 0.37 × 0.16 | 936 | 2 | Base, Glass |
