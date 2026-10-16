# Outdoor

The garden decor (gnome, bird bath, fountain, lantern, wheelbarrow, boulders) is done (`families/garden_decor.py`); the eleven OU1 patio models and the five OU2 loungers and swings are done in `families/outdoor.py` (notes: `outdoor.md`).
Outdoor things are seen from far: watch the budget, and street-scale props wait for LOD (E7).

## OU1. Patio seating and tables (11) — **Done** (in Validate › Patio)

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

## OU2. Loungers, swings and the hot tub (5) — **Done** (in Validate › Loungers)

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

## OU4. Play and yard (9) — **Done** (in Validate › Yard)

- **Family:** `tools/art/blender/families/yard.py` (notes: `yard.md`; a family of its own rather than `outdoor.py`, so OU3–OU5 didn't all edit one script and pack)
- **Validate group:** `yard` (Validate › Yard)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sandbox` Sandpit | 2x2 · floor | sandcastle:tend | outdoor / play | Frame | Use | `castle`: a sand castle (keep, four turrets, walls, a flag) rises out of the sand (2.5 s) and stands while it's built. Far version. |
| `bikeRack` Bike rack | 2x1 · floor | tuneUp:tinker | outdoor / play | Rack, Bike | Use + Loop | `wheel_up`: the bike tips up 8.5° on its front tyre, its back axle onto the stand's cradle; `wheel_turn`: the back wheel turns while tuned up (E1 loop). Far version. |
| `raisedBed` Raised vegetable bed | 2x1 · floor | tend:tend harvest:harvest | outdoor / yard | Bed | Growth + Use | `grow` (E4): lettuces, carrots and tomato bushes ripe at rest, the bed bare when picked, seedlings when planted; `harvest`: the crop is picked away (the stage then goes to picked); `rustle`: tending sways the plants every 2.4 s, on top of the stage. Far version. |
| `shedSmall` Garden shed | 2x2 · floor | potter:tinker | outdoor / yard | Wood, Roof, Door | Use | `door_open`: the ledged door swings 100° out for the whole of potter, the dark inside with a bench and tools behind. The door is a colour part of its own (one plain draw). Far version. |
| `greenhouse` Greenhouse | 2x3 · floor | tend:tend harvest:harvest | outdoor / yard | Frame, Wall | Use | `door_open`: the glass door swings 70° and the roof vent lifts 25° for tend and harvest. 4 draws (plain, glass, door, vent), as the nature lover's glasshouse. Far version. |
| `compostBin` Compost bin | 1x1 · floor | compost:tend | outdoor / yard | Bin | Use | `lid_open`: the slatted lid lifts 75° for compost; `hold` 2 s. Far version. |
| `rainBarrel` Rain barrel | 1x1 · floor | — | outdoor / yard | Barrel | Static | Static: ribbed barrel, brass tap, the downpipe into its lid. Far version. |
| `clothesline` Washing line | 3x1 · floor | washing:clean | outdoor / yard | Posts, Washing | Use + Open | `washing`: the washing comes down from the line (1.5 s) and is the `open` clip (C1): it stays out until taken in. Far version (washing in). |
| `firewoodStack` Log store | 1x1 · floor | — | outdoor / yard | Shelter | Static | Static: the iron rack under a pent iron roof, logs ends out. Far version. |

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
