# Garden plants and trees: authenticity ledger

What it covers: the 53 objects of [docs/objects/garden.md](../objects/garden.md), in its groups and order.

- **Evidence:** `bench-results/review/garden/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## G1. Houseplants (10)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `plant` Potted plant | OK | OK | I-held-01 | I-held-01 S2 code (seeded; held ledger: no watering can) | `sheets/obj-plant.png`, `play/plant-water-*.png` |
| `garden.monstera` Monstera | — | — | — | | |
| `garden.fiddleFig` Fiddle-leaf fig | — | — | — | | |
| `garden.snakePlant` Snake plant | — | — | — | | |
| `garden.cacti` Cactus trio | — | — | — | | |
| `garden.succulents` Succulent bowl | — | — | — | | |
| `garden.bostonFern` Fern on a stand | — | — | — | | |
| `garden.palm` Parlour palm | — | — | — | | |
| `garden.orchid` Orchid | — | — | — | | |
| `garden.bonsai` Bonsai | — | — | — | | |

## G2. Flowers (15)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `flowerbed` Raised flower bed | OK | OK | OK | | `sheets/obj-flowerbed.png`, `play/flowerbed-*.png`, `play-elder/flowerbed-*.png` |
| `garden.tulips` Tulips | — | — | — | | |
| `garden.daffodils` Daffodils | — | — | — | | |
| `garden.daisies` Daisies | — | — | — | | |
| `garden.sunflowers` Sunflowers | — | — | — | | |
| `garden.poppies` Poppies | — | — | — | | |
| `garden.marigolds` Marigolds | — | — | — | | |
| `garden.pansies` Pansies | — | — | — | | |
| `garden.lilies` Lilies | — | — | — | | |
| `garden.lupins` Lupins | — | — | — | | |
| `garden.forgetMeNots` Forget-me-nots | — | — | — | | |
| `garden.cosmos` Cosmos | — | — | — | | |
| `garden.meadow` Wildflower meadow | — | — | — | | |
| `garden.geraniums` Potted geraniums | — | — | — | | |
| `garden.flowerBox` Flower box | — | — | — | | |

## G3. Shrubs (11)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bush` Round shrub | OK | OK | I-held-01 | I-held-01 S2 code (seeded; held ledger: no shears) | `sheets/obj-bush.png`, `play/bush-trim-*.png` |
| `garden.boxwood` Boxwood ball | — | — | — | | |
| `garden.topiary` Topiary cone | — | — | — | | |
| `garden.hedge` Clipped hedge | — | — | — | | |
| `garden.hydrangea` Hydrangea | — | — | — | | |
| `garden.roses` Rose bush | — | — | — | | |
| `garden.azalea` Azalea | — | — | — | | |
| `garden.lavender` Lavender | — | — | — | | |
| `garden.fern` Garden fern | — | — | — | | |
| `garden.grass` Ornamental grass | — | — | — | | |
| `garden.hosta` Hosta | — | — | — | | |

## G4. Edibles (5)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `garden.vegPatch` Vegetable patch | — | — | — | | |
| `garden.tomatoes` Tomato plants | — | — | — | | |
| `garden.strawberries` Strawberry bed | — | — | — | | |
| `garden.herbs` Herb planter | — | — | — | | |
| `garden.pumpkins` Pumpkin patch | — | — | — | | |

## G5. Trees (12)

Family: `tools/art/blender/families/trees.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `tree` Oak tree | OK | I-garden-02 | I-garden-02 | I-garden-02 S2 clip | `sheets/obj-tree.png`, `play/tree-shade-*.png`, `sheets/tree-walk.png` |
| `pine` Pine tree | M-garden-01 | n/a | OK | M-garden-01 S2 model | `sheets/obj-pine.png`, `sheets/tree-walk.png`, `scenes/tree-walk-pine*.png` |
| `garden.maple` Maple tree | — | — | — | | |
| `garden.birch` Silver birch | — | — | — | | |
| `garden.cherry` Cherry blossom tree | — | — | — | | |
| `garden.apple` Apple tree | — | — | — | | |
| `garden.japaneseMaple` Japanese maple | — | — | — | | |
| `garden.magnolia` Magnolia | — | — | — | | |
| `garden.willow` Weeping willow | — | — | — | | |
| `garden.spruce` Blue spruce | — | — | — | | |
| `garden.cypress` Italian cypress | — | — | — | | |
| `garden.lemonTree` Potted lemon tree | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows; durations are judged by §9 decision 2.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `plant` | water it, wipe the leaves, talk to it | water the plant (5, not on free will) | OK |
| `flowerbed` | water, weed and tend, smell the flowers, cut some | tend the flowers (20), smell the flowers (5) | OK |
| `bush` | clip it into shape | trim the shrub (25) | OK |
| `tree` | sit or lie in its shade, climb it (children), rake the leaves | relax in the shade (30, standing) | sitting missing from the use (I-garden-02) |
| `pine` | year-round green, shelter | none | decor, no use needed |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

Not tied to one row: I-garden-01.

### I-garden-01 — Residents once didn't reach things in a crowded garden

- **Severity:** S3
- **Rubric:** I2
- **Fix class:** code
- **Where:** `crates/sim-core/tests/crowded_garden.rs`
- **Evidence:** Seen once in play; a sim test of eleven in a row could not reproduce it.
- **Status:** seeded (from `docs/objects/open.md` L148)
- **Source:** `docs/objects/open.md` Still open › Content and sim

### M-garden-01 — The pine's lowest tier is 6 m across from 1.4 m: residents walk into it

- **Severity:** S2
- **Rubric:** M4, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/trees.py:270-277` (`pine`: `conifer(rng, 6, 6.0, 1.4, …)`: six tiers, the first 6.0 m across with its foot at 1.4 m, on a 1 × 1 footprint)
- **Evidence:** `sheets/tree-walk.png` (top row): a resident standing 1 m from the pine's trunk is hidden inside the foliage, seen from the game's camera and from low down; the oak beside it (bottom row) leaves them in clear view under a crown that starts at 3 m. `play/pine-no-uses.png` is taken from inside the cone (the camera radius comes from the 1 × 1 footprint). A garden pine's lowest boughs do sweep low, but nothing stops residents walking under them (the sim sees one tile). Lift the first tier (to about 2.2 m) or narrow it, or give the pine a footprint as wide as its skirt.
- **Status:** open
- **Source:** pilot (`scripts/scenes.mjs` pine)

### I-garden-02 — "Relax in the shade" is half an hour standing by the trunk

- **Severity:** S2
- **Rubric:** I3, A2
- **Fix class:** clip
- **Where:** `web/public/content/base.json:2861` (`shade`: `anim: relax`, no `pose`); `web/src/render/babylon/characters.ts:472` (`ACTIONS.relax.stand`: `idle`, `foldArms`)
- **Evidence:** `play/tree-shade-01.png`, `sheets/obj-tree.png`: the resident stands at the trunk, arms at the sides or folded, for 30 minutes. Relaxing in a tree's shade is sitting against the trunk or lying in the grass. No clip sits on the ground today (`sit` needs a seat height); a sit-on-the-ground pose, or `pose: sit` at a seat height of 0, is the fix.
- **Status:** open
- **Source:** pilot
