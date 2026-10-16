# Decor

Mostly still things. Clocks and swaying things wait for ambient clips (E2); curtains that close at night for E3.

**Status (2026-10-14): D1–D3 done**, all 22 built by `tools/art/blender/families/decor.py` (notes: `decor.md`) and in
Validate › Wall Decor, Ornaments and Textiles. The ambient clips (E2: clock hands, pendulum) and the curtains' night
clip (E3) are built, not left static.

## D1. Wall art and clocks (6) — **Done**, signed off 2026-10-16 (back in decor / wall)

- **Family:** `tools/art/blender/families/decor.py`
- **Validate group:** `wall decor` (Validate › Wall Decor)
- **Placeholders to match:** `tools/art/catalog.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `printFramed` Framed print | 1x1 · wall | — | decor / wall | Frame | Static | Static. |
| `photoPrint` Photo print | 1x1 · wall | — | decor / wall | Frame | Static | Static. |
| `paintingClassic` Oil painting | 1x1 · wall | admire:look | decor / wall | Frame | Static | Static. |
| `canvasAbstract` Abstract canvas | 2x1 · wall | admire:look | decor / wall | Frame | Static | Static. |
| `wallClock` Wall clock | 1x1 · wall | — | decor / wall | Case, Face | Ambient (E2) | `hour_hand`, `minute_hand`: the hands tell the game's time (`motions.clock`); no second hand (a game minute is a real second). |
| `wallShelf` Floating shelves | 1x1 · wall | — | decor / wall | Shelves | Static | Static. |

## D2. Ornaments (10) — **Done**, signed off 2026-10-16 (back in decor / ornaments)

- **Family:** `tools/art/blender/families/decor.py`
- **Validate group:** `ornaments` (Validate › Ornaments)
- **Placeholders to match:** `tools/art/catalog.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `grandfatherClock` Grandfather clock | 1x1 · floor | wind:tinker | decor / ornaments | Case, Face | Use | `hour_hand`, `minute_hand`: the hands tell the game's time (`motions.clock`). The case door and the pendulum are still, to stay within 4 draws (decor.md). |
| `vaseTall` Tall vase | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `urnPorcelain` Porcelain urn | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `vasePedestal` Vase on a pedestal | 1x1 · floor | — | decor / ornaments | Vase, Pedestal | Static | Static. |
| `bustMarble` Marble bust | 1x1 · floor | admire:look | decor / ornaments | Bust, Plinth | Static | Static. |
| `statueCat` Cat statue | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `statueHorse` Bronze horse | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `coatRack` Coat stand | 1x1 · floor | — | decor / ornaments | Body | Static | Static. |
| `hangingPlant` Hanging plant | 1x1 · ceiling | water:water | decor / ornaments | Pot, Cord | Use | `sway`: the pot sways 5° and settles for water. |
| `fireplace` Fireplace | 2x1 · floor | relax:relax | decor / ornaments | Surround, Mantel | Use | `flames`: the flames grow on the logs for relax (gloss; the light is the content's). |

## D3. Textiles (6) — **Done**, signed off 2026-10-16 (back in decor / textiles)

- **Family:** `tools/art/blender/families/decor.py`
- **Validate group:** `textiles` (Validate › Textiles)
- **Placeholders to match:** `tools/art/catalog.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `curtains` Curtains | 1x1 · wall | — | decor / textiles | Fabric, Rail | Night (E3) | `close`: drawn closed at night (E3). Until then static, open. |
| `curtainsLong` Floor-length curtains | 2x1 · wall | — | decor / textiles | Fabric, Rail | Night (E3) | `close` (E3). |
| `rugRound` Round rug | 2x2 · rug | — | decor / textiles | Rug | Static | Static, flat (rug layer). |
| `rugLarge` Large area rug | 3x2 · rug | — | decor / textiles | Rug | Static | Static, flat. |
| `rugRunner` Hall runner | 1x3 · rug | — | decor / textiles | Rug | Static | Static, flat. |
| `rugPattern` Patterned rug | 3x2 · rug | — | decor / textiles | Rug | Static | Static, flat. |
