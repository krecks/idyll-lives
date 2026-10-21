# Lighting

All 21 lamps are built by `tools/art/blender/families/lighting.py` (notes: `families/lighting.md`): the ceiling lights
hang from 2.8 m at their real height, centred; the wall lights have their back on the wall face at their real
height; the floor and table lamps stand centred (the arc lamp's base in the back corner, its back on the wall face,
so its arc reaches out over the room).

The light itself is the content's `light` (height, range, intensity, tiles); the model only shows where it comes
from, and must put its bulb at its `light.height`. `_bulb` asserts each bulb within 25 cm of
`light.height` (most are within 8 cm); it doesn't check `at`. Across, the light is at the footprint's middle unless
the content's `light.at` [x, z] (object frame) moves it: the arc lamp's (0.245, 0.2) and the table lamp's (−0.12, 0)
put it at their shade in every facing. The wall lights keep theirs at the middle, 13–38 cm out from the bulb.

Where the light goes is the content's `light.aim` (the renderer's room lights are spot lights): `down` under a shade
that sends it down (the pendant's dome, the industrial pendant, the track heads, the swing-arm lamp, the arc lamp,
the banker's lamp: full below, about half 30–45° off, next to none side-on, so no disc up the wall behind), `up`
for the sconce's open cup (a wash up the wall above it), `out` for the outdoor bulkhead (away from its wall and
down: nothing behind the wall, so not the floor inside) and `ceiling` for the uplighter (its light comes off the
ceiling, 30 cm under it, over the lamp, reaching 1.5 m further: the room lit from above, not a pool round its
foot). The rest light all round. A lamp's light still passes through walls into the next room (lights kept to their
room are not done).

Lamps switch on and off (`crates/sim-core/src/lights.rs`): the player per lamp (click: Turn on / Turn off), residents
per room, on when someone awake comes into a dark room, off when the last one leaves or sleeps. A lamp bought comes
off (Q-11): it goes on with the rest as someone comes in, or by a click. The shades,
diffusers, globes, the paper and the candle's flame glow (E8) from dusk while their lamp is on, never by day; metal
shades (the industrial pendant, the swing-arm lamp, the arc lamp, the banker's lamp) glow only on a white liner just
inside. The ceiling fan's blades turn (`blades_turn`, once in 2.5 s) while its light is switched on, and the paper
lantern sways (`sway`, 2.5° in 2 s) while it's on (to drop in the Blender wave: a paper globe indoors hangs still),
both ambient (E2): near the camera, not on Low.

The ring chandelier and the industrial pendant hang at head height (content `headHeight`, Q-12): nobody walks under
them, though furniture (a dining table) goes under. The pendant lamp and the paper lantern still hang 3–16 cm over a
man's head; their shorter drop is the Blender wave's (M-lighting-05). A wall can't come down where it would leave a
ceiling lamp outdoors (M-lighting-01).

## LT1. Ceiling lights (7)

- **Family:** `tools/art/blender/families/lighting.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `ceilingPendant` Pendant lamp | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. The shade glows at night (E8). |
| `ceilingLight` Ceiling light | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. The diffuser glows at night. |
| `chandelier` Ring chandelier | 1x1 · ceiling | — | lighting / ceiling | Body, Globes | Static | Static. The globes glow at night. |
| `pendantIndustrial` Industrial pendant | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. Only the white liner inside the dome glows at night. |
| `ceilingFan` Ceiling fan | 1x1 · ceiling | — | lighting / ceiling | Body, Blades | Ambient (E2) | `blades_turn`: the blades turn slowly (E2). |
| `trackLights` Track lights | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. The heads' liners and lenses glow at night. |
| `paperLantern` Paper lantern | 1x1 · ceiling | — | lighting / ceiling | Paper, Ribs | Ambient (E2) | `sway`: a slow sway (E2). The paper glows at night. |

**In play** (signed off 2026-10-16):
- Hung in the living room (camera at 2.1 m) in all four facings, nothing floating or cut.
- At night each lights a pool on the floor under it and its shade glows; by day none glows.
- The fan's `blades_turn` and the lantern's `sway` run all the time near the camera on WebGPU, day and night, and
  are off on Low.
- The industrial pendant's green dome looks black from above at night (only its liner glows, inside), as intended.
- Nothing needed fixing.

## LT2. Wall lights (3)

- **Family:** `tools/art/blender/families/lighting.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `wallSconce` Wall sconce | 1x1 · wall | — | lighting / wall | Body, Shade | Static | Static. The shade glows at night. |
| `wallLampArm` Swing-arm wall lamp | 1x1 · wall | — | lighting / wall | Body, Shade | Static | Static. The shade's liner glows at night. |
| `wallLightOutdoor` Bulkhead light | 1x1 · wall | — | lighting / wall | Body, Shade | Static | Static. The lens glows at night. |

**In play** (signed off 2026-10-16):
- Checked on four walls (camera at 1.6 m), day and night: each sits with its back on the wall face at its height,
  throws a warm pool on the wall round it at night, and its shade or lens glows.
- With the cutaway dropping its wall (an inner wall, or an outer wall seen from outside), a wall light hides with
  the wall and still lights.
- The bulkhead's "Shade" is its glass lens: coloured, it tints faintly, as see-through glass does.
- The light stays at the footprint's middle, out from the bulb: at the bulb the sconce and bulkhead lit only a small
  hot ring round themselves.
- Nothing needed fixing.

## LT3. Floor lamps (5)

- **Family:** `tools/art/blender/families/lighting.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `lamp` Floor lamp | 1x1 · floor | — | lighting / floor | Stand, Shade, Fittings | Static | Static. The drum shade glows at night. |
| `floorLampSimple` Simple floor lamp | 1x1 · floor | — | lighting / floor | Stand, Shade | Static | Static. The square shade glows at night. |
| `lampFloorArc` Arc floor lamp | 1x1 · floor | — | lighting / floor | Stand, Shade, Base | Static | Static. The metal dome glows inside at night; the light is at the shade (`light.at`). |
| `lampFloorTripod` Tripod floor lamp | 1x1 · floor | — | lighting / floor | Stand, Shade | Static | Static. The linen shade glows at night. |
| `lampTorchiere` Uplighter | 1x1 · floor | — | lighting / floor | Stand, Shade, Base | Static | Static. The bowl glows at night. |

**In play** (signed off 2026-10-16):
- Checked standing free in all four facings (camera at 1.1 m), the arc lamp also with its base against two walls;
  at night each lights the floor and wall round it.
- The floor lamp's "Fittings" colours its brass collars and harp; the arc lamp's and uplighter's "Base" their foot.
- The arc lamp's light sits at its shade in every facing, against a wall and free (content `light.at`, `house.ts`).
- The uplighter's coloured glass bowl tints faintly.

## LT4. Table lamps (6)

- **Family:** `tools/art/blender/families/lighting.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `tableLamp` Table lamp | 1x1 · floor | — | lighting / table | Base, Shade, Table | Static | Static. A lamp on its own oak side table; the shade glows at night and the light is at it (`light.at`). |
| `lampTableRound` Ginger-jar lamp | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. The shade glows at night. |
| `lampTableSquare` Square table lamp | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. The shade glows at night. |
| `lampBanker` Banker's lamp | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. The shade's liner glows at night. |
| `lampMushroom` Mushroom lamp | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. The cap glows at night. |
| `lampLantern` Candle lantern | 1x1 · surface | — | lighting / table | Base, Glass | Static | Static. The flame glows at night. |

**In play** (signed off 2026-10-16):
- The table lamp (a floor object; "Table" colours its table) and the five surface lamps, each on a side table, in
  all four facings (camera at 0.7–0.8 m). The surface lamps stand on the top, carried at its height (lift 0.55 m),
  their light at the top plus `light.height`; their content `radius` (0.12–0.17 m) follows the drawn model, so they
  fit a side table's or nightstand's top.
- At night the shades, the mushroom's cap, the banker's liner and the lantern's flame glow, and the table and floor
  round them are lit.
- The table lamp's light sits at its shade in every facing (content `light.at`; the lamp stands off-centre on its
  table).
- A top under a lit lamp is washed almost white at night: the light is 30–45 cm above it and the shade casts no
  shadow (renderer; seen, not changed).
- The lantern's "Glass" tints faintly, as glass does.
