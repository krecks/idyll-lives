# Tabletop

Small things that stand on tops (`surface` layer, 1×1). No interactions, so no use clips: keep them to one draw
call and the smallest triangle budget (objects.md: 0.35 m, 1.5k/3k). If uses come later (a kettle lid, the
laptop opening), add the clip then.

All five tasks are done (2026-10-14): `families/tabletop.py` (notes: `families/tabletop.md`), in Validate. The
kettle's lid, the bread bin's roll-top and the laptop's lid are parts of their own, hinged for the clip to come.

## TT1. Kitchenware (11) — **Done**, signed off 2026-10-16 (back in tabletop / kitchenware)

- **Family:** `tools/art/blender/families/tabletop.py`
- **Validate group:** `kitchenware` (Validate › Kitchenware)
- **Placeholders to match:** `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kettle` Electric kettle | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. (+use) the lid lifts. |
| `toaster` Toaster | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `blender` Blender | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `standMixer` Stand mixer | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `espressoMachineSmall` Espresso machine | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `knifeBlock` Knife block | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `utensilCrock` Utensil pot | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `cuttingBoard` Chopping board | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `breadBin` Bread bin | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. (+use) the roll-top opens. |
| `spiceJars` Spice jars | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |
| `cookieJar` Cookie jar | 1x1 · surface | — | tabletop / kitchenware | Body | Static | Static. |

## TT2. Tableware (7) — **Done**, signed off 2026-10-16 (back in tabletop / tableware)

- **Family:** `tools/art/blender/families/tabletop.py`
- **Validate group:** `tableware` (Validate › Tableware)
- **Placeholders to match:** `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `fruitBowl` Fruit bowl | 1x1 · surface | — | tabletop / tableware | Body | Static | Static. |
| `plateSet` Place setting | 1x1 · surface | — | tabletop / tableware | Body | Static | Static. |
| `cakeStand` Cake on a stand | 1x1 · surface | — | tabletop / tableware | Body | Static | Static. |
| `teapotSet` Tea set | 1x1 · surface | — | tabletop / tableware | Body | Static | Static. |
| `mugs` Mugs | 1x1 · surface | — | tabletop / tableware | Body | Static | Static. |
| `wineGlasses` Wine and glasses | 1x1 · surface | — | tabletop / tableware | Body | Static | Static. |
| `candlesticks` Candlesticks | 1x1 · surface | — | tabletop / tableware | Body | Static | Static. |

## TT3. Books and small plants (7) — **Done** (in Validate › Books And Pots)

- **Family:** `tools/art/blender/families/tabletop.py`
- **Validate group:** `books and pots` (Validate › Books And Pots)
- **Placeholders to match:** `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bookStack` Stack of books | 1x1 · surface | — | tabletop / books | Body | Static | Static. |
| `booksUpright` Books and bookends | 1x1 · surface | — | tabletop / books | Body | Static | Static. |
| `vaseFlowers` Vase of flowers | 1x1 · surface | — | tabletop / plants | Body | Static | Static. |
| `vaseSmall` Bud vase | 1x1 · surface | — | tabletop / plants | Body | Static | Static. |
| `potSucculent` Succulent | 1x1 · surface | — | tabletop / plants | Body | Static | Static. |
| `potCactus` Cactus | 1x1 · surface | — | tabletop / plants | Body | Static | Static. |
| `potSmall` Small potted plant | 1x1 · surface | — | tabletop / plants | Body | Static | Static. |

## TT4. Small ornaments (6) — **Done**, signed off 2026-10-16 (back in tabletop / ornaments)

- **Family:** `tools/art/blender/families/tabletop.py`
- **Validate group:** `small ornaments` (Validate › Small Ornaments)
- **Placeholders to match:** `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `photoFrame` Photo frame | 1x1 · surface | — | tabletop / ornaments | Body | Static | Static. |
| `tableClock` Alarm clock | 1x1 · surface | — | tabletop / ornaments | Body | Ambient (E2) | `hour_hand`, `minute_hand`: the hands tell the game's time (`motions.clock`); no second hand. |
| `globe` Globe | 1x1 · surface | — | tabletop / ornaments | Body | Static | Static. |
| `snowGlobe` Snow globe | 1x1 · surface | — | tabletop / ornaments | Body | Static | Static. |
| `trophy` Trophy | 1x1 · surface | — | tabletop / ornaments | Body | Static | Static. |
| `sculptureSmall` Small sculpture | 1x1 · surface | — | tabletop / ornaments | Body | Static | Static. |

## TT5. Bath, desk and toys (8) — **Done**, signed off 2026-10-16 (back in tabletop / bath, desk and toys)

- **Family:** `tools/art/blender/families/tabletop.py`
- **Validate group:** `desk and bath bits` (Validate › Desk And Bath Bits)
- **Placeholders to match:** `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `soapSet` Soap set | 1x1 · surface | — | tabletop / bath | Body | Static | Static. |
| `towelStack` Folded towels | 1x1 · surface | — | tabletop / bath | Body | Static | Static. |
| `perfumeTray` Perfume tray | 1x1 · surface | — | tabletop / bath | Body | Static | Static. |
| `deskOrganiser` Desk organiser | 1x1 · surface | — | tabletop / desk | Body | Static | Static. |
| `laptopClosed` Laptop | 1x1 · surface | — | tabletop / desk | Body | Static | Static. (+use) the lid opens. |
| `monitor` Monitor and keyboard | 1x1 · surface | — | tabletop / desk | Body | Static | Static. |
| `stuffedToy` Teddy | 1x1 · surface | — | tabletop / toys | Body | Static | Static. |
| `toyBlocks` Toy blocks | 1x1 · surface | — | tabletop / toys | Body | Static | Static. |
