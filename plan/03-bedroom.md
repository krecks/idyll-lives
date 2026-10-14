# Bedroom

Mattress heights must match `BEDS`, and beds that get the game's blanket are in `BLANKET_BEDS`
(`web/src/render/babylon/characters.ts`): a bed's own duvet sits under that blanket. Things with drawers and doors
follow the fridge (hinge, `hold`).

All four tasks are built: the beds in `families/bedroom.py` (`turn_down`, the futon's `unfold`), the nightstands,
wardrobes and drawers in `families/bedroom_storage.py` (`drawer_open`, `doors_open`, `door_open`), and the mirrors and
dressing table in `families/bedroom_dressing.py` (`drawer_open`). The three new beds that turn down (`bedDouble`,
`bedQueenUpholstered`, `bedCanopy`) are in `BLANKET_BEDS`, and those four not in `BEDS` before have their mattress tops there.

## BR1. Beds (9) — **Done** (in Validate › Beds)

- **Family:** `tools/art/blender/families/bedroom.py`
- **Validate group:** `beds` (Validate › Beds)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bed` Double bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`: the top third of the duvet folds back toward the foot as they get in (sleep, nap) and back up when they get up. Check it against the game's blanket. |
| `bedSingle` Single bed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedBunk` Bunk bed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down` on the lower bunk; the ladder is static. |
| `bedFuton` Futon bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Held open | `unfold`: the back folds flat into a bed while slept in (needs C1: `leaves: open` on sleep and nap, as the wall bed). |
| `bedKing` Upholstered king bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedDouble` Double bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedQueenUpholstered` Upholstered queen bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedCanopy` Four-poster bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`; the canopy drapes are static. |
| `bedDaybed` Daybed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Static | Static (a sofa by day). |

## BR2. Nightstands and small storage (8) — **Done** (in Validate › Nightstands)

- **Family:** `tools/art/blender/families/bedroom_storage.py`
- **Validate group:** `nightstands` (Validate › Nightstands)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `nightstand` Nightstand | 1x1 · floor | — | bedroom / storage | Body, Handles | Static | Static. |
| `nightstandModern` Modern nightstand | 1x1 · floor | — | bedroom / nightstands | Body, Handles | Static | Static. |
| `nightstandRound` Drum nightstand | 1x1 · floor | — | bedroom / nightstands | Body, Handles | Static | Static. |
| `nightstandDrawers` Three-drawer nightstand | 1x1 · floor | — | bedroom / nightstands | Body, Handles | Static | Static. (+use) a drawer slides out. |
| `nightstandLamp` Nightstand with lamp | 1x1 · floor | — | bedroom / storage | Body, Shade | Static | Static; the shade glows at night (N, E3/E8). |
| `cedarChest` Blanket chest | 1x1 · floor | — | bedroom / storage | Body, Handles | Static | Static. (+use) `lid_open`: the lid swings up 100°. |
| `laundryHamper` Laundry hamper | 1x1 · floor | — | bedroom / storage | Hamper | Static | Static. (+use) the lid lifts. |
| `benchBedEnd` Bed-end bench | 2x1 · floor | sit:sit | bedroom / storage | Frame, Cushion | Static | Static. |

## BR3. Wardrobes and drawers (6) — **Done** (in Validate › Wardrobes)

- **Family:** `tools/art/blender/families/bedroom_storage.py`
- **Validate group:** `wardrobes` (Validate › Wardrobes)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `dresser` Chest of drawers | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`: the middle drawer slides out 30 cm for outfit; `hold` 2.5 s. |
| `wardrobe` Wardrobe | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `doors_open`: both doors swing out 100° for outfit; `hold` 3 s. |
| `chestOfDrawers` Tall chest of drawers | 1x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`. |
| `dresserLow` Low dresser | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`. |
| `wardrobeSingle` Single wardrobe | 1x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `door_open`. |
| `armoire` Armoire | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `doors_open`. |

## BR4. Dressing and mirrors (4) — **Done** (in Validate › Dressing)

- **Family:** `tools/art/blender/families/bedroom_dressing.py`
- **Validate group:** `dressing` (Validate › Dressing)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `mirror` Standing mirror | 1x1 · floor | speech:talk | bedroom / dressing | Frame | Static | Static; mirror part (E6). |
| `dressingTable` Dressing table | 1x1 · floor | getReady:wash | bedroom / dressing | Table, Frame | Use | `drawer_open`: the drawer slides out for getReady; mirror part (E6). |
| `mirrorWall` Wall mirror | 1x1 · wall | speech:talk | bedroom / dressing | Frame | Static | Static; mirror part (E6). |
| `mirrorFloor` Standing mirror | 1x1 · floor | primp:look | bedroom / dressing | Frame | Static | Static; mirror part (E6). |
