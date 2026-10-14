# Lighting

The light itself is the content's `light` (position, colour, power): the model must put its bulb where
`tools/art/catalog2.py` `LIGHTS` says. Lamps have no on/off state; a shade that glows at night needs E3/E8.

## LT1. Ceiling lights (7)

- **Family:** `tools/art/blender/families/lighting.py`
- **Validate group:** `ceiling lights` (Validate › Ceiling Lights)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `ceilingPendant` Pendant lamp | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. N shade glow (E8). |
| `ceilingLight` Ceiling light | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. |
| `chandelier` Ring chandelier | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. |
| `pendantIndustrial` Industrial pendant | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. |
| `ceilingFan` Ceiling fan | 1x1 · ceiling | — | lighting / ceiling | Body, Blades | Ambient (E2) | `spin`: the blades turn slowly (E2). Until then static. |
| `trackLights` Track lights | 1x1 · ceiling | — | lighting / ceiling | Body, Shade | Static | Static. |
| `paperLantern` Paper lantern | 1x1 · ceiling | — | lighting / ceiling | Paper | Ambient (E2) | `sway`: a slow sway (E2). Until then static. |

## LT2. Wall lights (3)

- **Family:** `tools/art/blender/families/lighting.py`
- **Validate group:** `wall lights` (Validate › Wall Lights)
- **Placeholders to match:** `tools/art/catalog.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `wallSconce` Wall sconce | 1x1 · wall | — | lighting / wall | Body, Shade | Static | Static. |
| `wallLampArm` Swing-arm wall lamp | 1x1 · wall | — | lighting / wall | Body, Shade | Static | Static. |
| `wallLightOutdoor` Bulkhead light | 1x1 · wall | — | lighting / wall | Body, Shade | Static | Static. |

## LT3. Floor lamps (5)

- **Family:** `tools/art/blender/families/lighting.py`
- **Validate group:** `floor lamps` (Validate › Floor Lamps)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `lamp` Floor lamp | 1x1 · floor | — | lighting / floor | Stand, Shade | Static | Static. |
| `floorLampSimple` Simple floor lamp | 1x1 · floor | — | lighting / floor | Stand, Shade | Static | Static. |
| `lampFloorArc` Arc floor lamp | 1x1 · floor | — | lighting / floor | Stand, Shade | Static | Static. |
| `lampFloorTripod` Tripod floor lamp | 1x1 · floor | — | lighting / floor | Stand, Shade | Static | Static. |
| `lampTorchiere` Uplighter | 1x1 · floor | — | lighting / floor | Stand, Shade | Static | Static. |

## LT4. Table lamps (6)

- **Family:** `tools/art/blender/families/lighting.py`
- **Validate group:** `table lamps` (Validate › Table Lamps)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `tableLamp` Table lamp | 1x1 · floor | — | lighting / table | Base, Shade | Static | Static. |
| `lampTableRound` Ginger-jar lamp | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. |
| `lampTableSquare` Square table lamp | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. |
| `lampBanker` Banker's lamp | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. |
| `lampMushroom` Mushroom lamp | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. |
| `lampLantern` Candle lantern | 1x1 · surface | — | lighting / table | Base, Shade | Static | Static. |
