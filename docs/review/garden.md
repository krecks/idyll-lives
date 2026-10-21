# Garden plants and trees: authenticity ledger

What it covers: the 53 objects of [docs/objects/garden.md](../objects/garden.md), in its groups and order.

- **Evidence:** `bench-results/review/garden/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## G1. Houseplants (10)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `plant` Potted plant | — | — | — | | |
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
| `flowerbed` Raised flower bed | — | — | — | | |
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
| `bush` Round shrub | — | — | — | | |
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
| `tree` Oak tree | — | — | — | | |
| `pine` Pine tree | — | — | — | | |
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
