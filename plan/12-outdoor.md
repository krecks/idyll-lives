# Outdoor

The garden decor (gnome, bird bath, fountain, lantern, wheelbarrow, boulders) is done (`families/garden_decor.py`).
Outdoor things are seen from far: watch the budget, and street-scale props wait for LOD (E7).

## OU1. Patio seating and tables (11)

- **Family:** `tools/art/blender/families/outdoor.py`
- **Validate group:** `patio` (Validate › Patio)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bench` Park bench | 2x1 · floor | sit:sit | outdoor / seating | Frame, Slats | Static | Static. |
| `patioSet` Folding bistro set | 1x2 · floor | coffee:drink sit:sit | outdoor / seating | Frame, Slats | Static | Static. |
| `picnicTable` Picnic table | 3x2 · floor | picnic:eat coffee:drink | outdoor / seating | Frame, Slats | Static | Static. |
| `gardenChair` Garden chair | 1x1 · floor | sit:sit | outdoor / seating | Frame, Slats | Static | Static. |
| `patioTableRound` Round patio table | 1x1 · floor | — | outdoor / seating | Frame, Slats | Static | Static. |
| `patioTable4` Teak patio table | 2x1 · floor | — | outdoor / seating | Frame, Slats | Static | Static. |
| `patioChair` Patio chair | 1x1 · floor | sit:sit coffee:drink | outdoor / seating | Frame, Slats | Static | Static. |
| `adirondackChair` Adirondack chair | 1x1 · floor | relax:relax | outdoor / seating | Frame, Slats | Static | Static. |
| `benchGarden3` Garden bench | 3x1 · floor | sit:sit | outdoor / seating | Frame, Slats | Static | Static. |
| `outdoorBar` Outdoor bar | 2x1 · floor | — | outdoor / seating | Frame, Slats | Static | Static. |
| `parasol` Parasol | 1x1 · floor | — | outdoor / seating | Canopy, Pole | Static | Static, open. A it stirs in the wind (E2); (+use) it folds. |

## OU2. Loungers, swings and the hot tub (5)

- **Family:** `tools/art/blender/families/outdoor.py`
- **Validate group:** `loungers` (Validate › Loungers)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sunLounger` Sun lounger | 1x2 · floor | sunbathe:relax doze:nap | outdoor / seating | Frame, Fabric | Use | `recline`: the back lowers 30° for sunbathe and doze. |
| `hammock` Hammock | 1x2 · floor | swing:relax doze:nap | outdoor / seating | Frame, Fabric | Loop (E1) | `sway`: swings ±8° while lain in (E1). Until then static. |
| `porchSwing` Porch swing | 2x1 · floor | swing:relax | outdoor / seating | Seat, Frame, Chains | Loop (E1) | `swing`: the seat swings ±10° while sat in (E1). Until then static. |
| `swingSet` Swing set | 2x1 · floor | swing:play | outdoor / play | Frame, Fabric | Loop (E1) | `swing`: the seat swings while played (E1). Until then U: the seat tips back 15° and holds. |
| `hotTubCedar` Cedar hot tub | 2x2 · floor | soak:bath | outdoor / seating | Cedar, Cover | Use | `cover_off`: the cover slides off and folds back for soak. L the water bubbles (E1). |

## OU3. Grills and fire (3)

- **Family:** `tools/art/blender/families/outdoor.py`
- **Validate group:** `grills` (Validate › Grills)
- **Placeholders to match:** `tools/art/catalog.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `grill` Charcoal grill | 1x1 · floor | cook:cook | outdoor / cooking | Body, Lid | Use | `lid_open`: the lid swings up for the whole of cook. |
| `grillStation` Gas grill station | 2x1 · floor | cook:cook feast:cook | outdoor / cooking | Cart, Hood | Use | `hood_up`: the hood swings up for cook and feast. |
| `firePit` Fire pit | 1x1 · floor | relax:relax | outdoor / cooking | Bowl, Stones | Use | `flames`: the flames grow on the logs for relax (the light is the content's). |

## OU4. Play and yard (9)

- **Family:** `tools/art/blender/families/outdoor.py`
- **Validate group:** `yard` (Validate › Yard)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sandbox` Sandpit | 2x2 · floor | sandcastle:tend | outdoor / play | Frame | Use | `castle`: a sand castle grows for sandcastle. |
| `bikeRack` Bike rack | 2x1 · floor | tuneUp:tinker | outdoor / play | Rack, Bike | Use | `wheel_up`: the bike's back wheel lifts on its stand for tune up. L the wheel turns (E1). |
| `raisedBed` Raised vegetable bed | 2x1 · floor | tend:tend harvest:harvest | outdoor / yard | Bed | Use | `harvest`: the ripe vegetables shrink away during harvest and return after. G growth stages (E4). |
| `shedSmall` Garden shed | 2x2 · floor | potter:tinker | outdoor / yard | Wood, Roof | Use | `door_open`: the door swings open for the whole of potter. |
| `greenhouse` Greenhouse | 2x3 · floor | tend:tend harvest:harvest | outdoor / yard | Frame | Use | `door_open`: the door swings and a roof vent tilts open for tend and harvest. |
| `compostBin` Compost bin | 1x1 · floor | compost:tend | outdoor / yard | Bin | Use | `lid_open`: the lid lifts for compost; `hold` 2 s. |
| `rainBarrel` Rain barrel | 1x1 · floor | — | outdoor / yard | Barrel | Static | Static. |
| `clothesline` Washing line | 3x1 · floor | washing:clean | outdoor / yard | Posts, Washing | Use | `washing`: the washing grows on the line for washing. Keeping it out after needs O (C1). |
| `firewoodStack` Log store | 1x1 · floor | — | outdoor / yard | Shelter | Static | Static. |

## OU5. Pond, small garden things and lights (9)

- **Family:** `tools/art/blender/families/outdoor.py`
- **Validate group:** `garden things` (Validate › Garden Things)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `garden.pond` Garden pond | 2x2 · floor | fish:look feedFish:garden | outdoor / decor | Stones | Use | `fish`: a koi rises to the surface for feedFish and fish, and sinks. A ripples (E2). |
| `planterBox` Planter box | 1x1 · floor | water:water | outdoor / decor | Box | Static | Static. |
| `birdhouse` Birdhouse | 1x1 · floor | — | outdoor / decor | House, Roof | Static | Static. |
| `birdFeederPole` Bird feeders | 1x1 · floor | birdwatch:look | outdoor / decor | Pole, Feeders | Use | `bird`: a bird appears on the perch for birdwatch. |
| `dogHouse` Dog house | 1x1 · floor | — | outdoor / decor | House, Roof | Static | Static. |
| `sundial` Sundial | 1x1 · floor | — | outdoor / decor | Plinth, Dial | Static | Static. |
| `steppingStones` Stepping stones | 1x3 · rug | — | outdoor / decor | Stones | Static | Static, flat. |
| `lampPost` Garden lamp post | 1x1 · floor | — | outdoor / lights | Post, Lantern | Static | Static. N glow (E8). |
| `stringLights` String lights | 2x1 · floor | — | outdoor / lights | Posts, Bulbs | Static | Static. A a gentle sway (E2). |
