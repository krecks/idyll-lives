# Lighting — notes

`lighting.py` builds the 21 lamps of docs/objects/lighting.md into `web/public/assets/models/lighting/` and their
entries in `packs/lighting/manifest.json`. Built 2026-10-14; they sat in Validate ›
Ceiling Lights, Wall Lights, Floor Lamps and Table Lamps until signed off.

- **LT1, ceiling:** `ceilingPendant`, `ceilingLight`, `chandelier`, `pendantIndustrial`, `ceilingFan`,
  `trackLights`, `paperLantern`. They hang from 2.8 m at their real height (`G.settle(floor=False)`,
  `floor=False` on the `Model`), centred. The pendant lamp and the paper lantern, which residents walk under, hang
  clear of heads: their lowest point 2.11 and 2.12 m up (a 0.69 m drop; 2026-10-21, review M-lighting-05 and Q-12,
  was 1.85 and 1.80); the chandelier and the industrial pendant hang lower, over furniture only (content
  `headHeight`).
- **LT2, wall:** `wallSconce`, `wallLampArm`, `wallLightOutdoor`. Their back is on the wall face (−0.43), at
  their real height (`G.settle(centre=False, floor=False)`, `centre=` on the `Model`).
- **LT3, floor:** `lamp`, `floorLampSimple`, `lampFloorArc`, `lampFloorTripod`, `lampTorchiere`. The arc lamp
  reaches over the next tile (2026-10-26, review M-lighting-06): its base in the back left corner of its tile, its
  back on the wall face, the arc rising to 2.05 m and coming down over the middle of the tile in front, 1.33 m from
  the base across the floor (`ARC_BASE`, `ARC_SHADE_AT`), the shade from 1.6 to 1.8 m, over a sofa's seat. It is
  built where it stands (`G.settle(centre=False)`, `centre` the box's middle, -0.1245, 0.3755). Content: `overhang`
  [[0, 1]] (furniture under the shade, nobody walking there, nothing tall or hanging low), `light.at` (0, 1) and
  `light.height` 1.65 at the bulb, and `freeRotation` false (the sim refuses an `overhang` that turns freely: the
  shade's tile turns in quarter turns). Build mode checks the shade's tile as the sim does; the house generator hangs
  it over a seat beside the lamp when there is one, else over free floor, which it takes from walking (2 of 24 over a
  sofa in the 180 sample houses). The gate counts the tiles it reaches over as its own.
- **LT4, table:** `tableLamp` (a lamp on its own oak side table, a floor object), and the surface lamps
  `lampTableRound`, `lampTableSquare`, `lampBanker`, `lampMushroom`, `lampLantern`. Their content `radius` (the room
  they take on a top) follows the drawn model: the round, square, mushroom and lantern lamps' were taken from their
  old parts' boxes (0.18–0.29 m), too wide for a side table's or nightstand's top, which refused them; they are
  0.12–0.17 m now (2026-10-14), and all five stand on a side table in play.

## The light

The light is the content's `light`; the model only shows where it comes from. `_bulb` (and `_near_light`)
asserts each bulb within 25 cm of the content's `light.height`, read from `base.json`, `furniture.json` and
`catalog2/lighting.json`, so a remodel can't drift away from it. Most are within 8 cm. The pendant's light was
at the top of its dome (2.1 m, as the old part had it), 20 cm above the bulb; it is at the bulb now (1.9 m,
2026-10-14), a little brighter on the floor and a little darker on the ceiling. Across, the light is at the
footprint's middle unless the content's `light.at` moves it (in the game's frame, the old part's: the GLB's x
mirrored, as the glTF loader turns it): the arc lamp's (0, 1), over the tile in front, and the table lamp's (−0.12, 0)
put it at their bulbs, the nightstand lamp's (0, −0.26, bedroom) at its bulb near the wall (2026-10-17). Since
2026-10-26 the build checks `at` too: once a lamp of this family with a `light.at` is built and settled, `_check_at`
finds its bulb (the middle of the faces in the bulb's material, which `_bulb` notes) and asserts it within 5 cm across
of `at`. The wall lights keep theirs at the middle, 13–38 cm out from the bulb: tried at the bulb, the sconce and
bulkhead lit only a small hot ring round themselves and lost the warm pool on the wall and floor.

## Night glow

The shades, diffusers, globes, the paper and the candle's flame have `glow` (E8): they light up from dusk with
the room's lamps, nothing by day. Metal shades (the industrial pendant, the swing-arm lamp, the arc lamp) glow only
on their white `liner`, a thin shell just inside the shade. Coloured glass glows in its own colour, not the cream
`GLOW` (2026-10-21, review M-lighting-03): the banker's green shade `#4FB27A` over its white liner, the mushroom
lamp's coral cap `#FFAA78`.

## Clips (ambient, `idle`)

- **Ceiling fan** (`blades`): the five blades and their irons, origin on the axis; `blades_turn` turns them
  once in 2.5 s (`G.spin`). The axis is on the tile's middle, so the blades sweep a circle 0.974 m across
  inside the tile (the fan's box, reaching farther one way, is centred 3.7 cm off: its `centre`, -0.0374 since the
  blades' tips were rounded, 2026-10-22).
- **Paper lantern**: still since 2026-10-21 (review A-lighting-01: a paper globe indoors hangs still without a
  draught; it swayed 2.5° in 2 s, `G.rock`). One node, joined.

Not on Low (E2); at rest they look as the old part.

## Shapes

- `_lathe` turns an old part's `lathe` (profile in fractions of its size) into a `G.revolve`, closing the
  profile and optionally adding the liner. `top=True` puts the origin at the top, to tilt the swing-arm
  shade about where it hangs.
- `_drum` is an open drum or cone shade with a wall thickness (4 segments turned 45° for the square ones).
- `_trough` is the banker's half-round shade, open below, coming 20° past the axis so the tube bulb hides.

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit: same keys, sizes (within 2 %), centres (the fan's re-measured), bulb heights,
paint and draws; every lamp under its size class's target. Shared turned parts: `_turned` (a profile from a foot),
`_canopy` (a ceiling cup with a cord grip), `_socket` (ribbed), `_pear` (a pear bulb), `_ring` (rolled rims, wires),
`_loft` (rounded-square bodies, `G.loft`/`G.ring`); `_drum` takes `rims` (rolled bindings in the shade's own fabric)
and `spider` (the wire fitter seen from above); cords, stems, arms and harps are `G.kit.tube` sweeps. Shades keep
their materials and glows.
- **Ceiling**: the pendant's and the industrial dome's rolled lips, collars and fitters, pear bulbs; the lantern's
  glazing bars, head rails and hipped lid; the chandelier's turned column, S-scrolled arms and curved spokes (one
  sweep each, no rod-and-sphere chains), bobeches and a finial; the fan's banded motor, ball cover and coupler,
  extruded blades with rounded tips on curved irons (axis still on the tile's middle, `settle(centre=False)`); the
  track's slotted channel, live end and finned spot cans aimed as before; the paper lantern's skin pinched in at each
  rib, open top and bottom on wire rims, a lining and bulb seen through them.
- **Wall**: turned back plates, the sconce's gooseneck arm and candle cup, the swing arm's bent arms, thumb screw and
  rolled shade rim, the bulkhead's drip hood, screwed plate and cage bars arching over the lens.
- **Floor**: stepped turned feet and bases, turned collars, the classic lamp's harp, knob and pull; the square lamp's
  boss, coupling, switch, black-bound shade and fitter; the arc lamp's stem and arc one sweep with a sleeve and a
  dimmer; the tripod's tapered legs on brass ferrules; the torchiere's petalled glass bowl (lofted), dimmer and cup.
- **Table**: the side table's rounded top, aprons and tapered legs; brass foot rings, caps, harps and finials; the
  banker's stepped base, upright curving into its cradle, brass band and bound ends, beaded chain; the mushroom's
  flared stem and rolled cap lip; the lantern's rails, curved roof with knob, latch, and a candle with a drip and wick.

## Checks

`check-glb`: 21 ok, 0 failed. Its warnings, all expected:

- the ceiling and wall lights float above their origin (they hang);
- `floorLampSimple`, `lampTableSquare` and `lampLantern` are about half the old part's width: the gate
  measures the old part's 4-sided shade or roof (a cylinder of 4 segments turned 45°) by its unturned box;
  the drawn sizes match;
- `wallLampArm` is 5 cm lower than the gate's estimate of the tilted old shade; the drawn shades agree
  (checked side by side in the gallery), so the shade is kept. Fixed 2026-10-14: the track lights'
  old part aims its heads as the model's (it was 3 cm shallower), and the outdoor light's lens no longer reaches
  3 cm into the wall behind its plate (its hidden back half is flattened onto the wall face), so it is 0.11 m deep,
  as the model does.

As of 2026-10-26 the gate shows only the floats warnings: `lampFloorArc`'s light (1.65 m at (0, 1)) is inside its
shade, and the tiles it reaches over count as its own. Khronos validator: 0 errors, 0 warnings each.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| ceilingPendant | 0.43 × 0.68 × 0.43 | 2,388 | 1 | Body, Shade |
| ceilingLight | 0.36 × 0.60 × 0.36 | 1,248 | 1 | Body, Shade |
| chandelier | 0.94 × 0.76 × 0.90 | 3,964 | 1 | Body, Globes |
| pendantIndustrial | 0.38 × 0.87 × 0.38 | 3,108 | 1 | Body, Shade |
| ceilingFan | 0.90 × 0.39 × 0.94 | 3,344 | 3 | Body, Blades |
| trackLights | 0.94 × 0.30 × 0.19 | 2,692 | 1 | Body, Shade |
| paperLantern | 0.54 × 0.68 × 0.54 | 3,820 | 2 | Paper, Ribs |
| wallSconce | 0.20 × 0.25 × 0.27 | 1,372 | 2 | Body, Shade |
| wallLampArm | 0.20 × 0.30 × 0.42 | 1,608 | 1 | Body, Shade |
| wallLightOutdoor | 0.22 × 0.33 × 0.11 | 1,228 | 2 | Body, Shade |
| lamp | 0.45 × 1.81 × 0.45 | 3,324 | 2 | Stand, Shade, Fittings |
| floorLampSimple | 0.28 × 1.60 × 0.28 | 1,016 | 2 | Stand, Shade |
| lampFloorArc | 0.61 × 2.06 × 1.61 | 3,632 | 1 | Stand, Shade, Base |
| lampFloorTripod | 0.54 × 1.62 × 0.55 | 2,584 | 3 | Stand, Shade |
| lampTorchiere | 0.40 × 1.85 × 0.40 | 2,284 | 2 | Stand, Shade, Base |
| tableLamp | 0.66 × 0.92 × 0.42 | 3,220 | 3 | Base, Shade, Table |
| lampTableRound | 0.34 × 0.58 × 0.34 | 3,380 | 2 | Base, Shade |
| lampTableSquare | 0.22 × 0.54 × 0.22 | 976 | 2 | Base, Shade |
| lampBanker | 0.30 × 0.39 × 0.16 | 2,536 | 1 | Base, Shade |
| lampMushroom | 0.30 × 0.36 × 0.30 | 1,668 | 1 | Base, Shade |
| lampLantern | 0.16 × 0.37 × 0.16 | 1,070 | 2 | Base, Glass |
