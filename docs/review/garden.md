# Garden plants and trees: authenticity ledger

What it covers: the 53 objects of [docs/objects/garden.md](../objects/garden.md), in its groups and order.

- **Evidence:** `bench-results/review/garden/` (gallery, clip, pose and play shots; shots stay out of git). Shots in the garden wave's rows are relative to that folder: `sheets/obj-<id>.png` (contact sheets), `play/…`, `clips-stages/…`, and `extra/` for the shots the garden wave took (`extra/harvest/`: every crop's ripe harvest on `review-garden`; `extra/close/`: the apple, lemon tree, Japanese maple and willow close up; `s-*.png` are side-by-side strips of a use's three shots).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Garden wave** (2026-10-21): the other 48 rows, one reviewer; steps, commands and timings in `bench-results/review/garden/review-log.md`.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **A column for plants:** the foliage sways by the nature shader (`render/babylon/nature.ts`), the flowers, pots and stems stay still (`plants.py:5-8`). Sway can't be judged from stills; it is `OK` unless a clip or a pose at the plant is wrong. A resident's pose at a plant is filed under I3 with A1 (README), and the A column names the same id.
- **Durations** are by design (§9 decision 2). `cargo review uses` flags `pickHerbs` and `pickFlower` (5 min against a `harvest` median of 20): picking a sprig or one flower is not a crop harvest, so they are not outliers against similar uses.
- **No tools in hand** anywhere in the garden (watering, trimming, cutting, harvesting into a basket): I-held-01 (held ledger), named in each row it touches as "I-held-01 (held ledger)".

## G1. Houseplants (10)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `plant` Potted plant | OK | OK | I-held-01 | I-held-01 S2 code (seeded; held ledger: no watering can) | `sheets/obj-plant.png`, `play/plant-water-*.png` |
| `garden.monstera` Monstera | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.monstera.png` |
| `garden.fiddleFig` Fiddle-leaf fig | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.fiddleFig.png` |
| `garden.snakePlant` Snake plant | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.snakePlant.png` |
| `garden.cacti` Cactus trio | M-garden-08 | OK | I-held-01 | M-garden-08 S3 decision; I-held-01 S2 code (held ledger) | `sheets/obj-garden.cacti.png` |
| `garden.succulents` Succulent bowl | M-garden-08 | OK | I-held-01 | M-garden-08 S3 decision; I-held-01 S2 code (held ledger) | `sheets/obj-garden.succulents.png` |
| `garden.bostonFern` Fern on a stand | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.bostonFern.png` |
| `garden.palm` Parlour palm | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.palm.png`, `play/short-redo/garden.palm-water-01.png` |
| `garden.orchid` Orchid | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.orchid.png`, `extra/s-orchid.png` |
| `garden.bonsai` Bonsai | OK | I-garden-09 | I-garden-09 | I-garden-09 S3 model; I-held-01 S2 code (held ledger: no scissors) | `sheets/obj-garden.bonsai.png` |

Notes, M then A then I:
- **Sizes** (`cargo review sizes`): all inside `houseplantFloor` / `houseplantSmall` but three. The snake plant's 0.34 m width is under the floor-plant range (0.36), which is set for broad plants; a sansevieria in a 25–30 cm pot is that narrow: OK. The bonsai's 0.97 m height and the orchid's 1.20 m include their table and plinth: OK.
- **Species:** each reads as itself at the game camera and close up: split monstera leaves, the fig's single stem, striped snake-plant straps, three potted cacti with one in bloom, the rosette bowl, the fern spilling over a wooden stand, palm fronds, orchid sprays on a plinth, a cloud-pruned bonsai in a glazed tray.
- **Pose:** watering is the `water` clip (`clips/garden.ts:26-35`): the can hand held out at 1.0 m and tipped, which reads right for a can over pots on the floor once a can is drawn (I-held-01). Talking to the plant is `talk` with the head lowered (`characters.ts:3377-3381`); admiring the orchid faces it with the arms folded.

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `plant` | water it, wipe the leaves, talk to it | water the plant (5, not on free will) | OK |
| `garden.monstera` | water, wipe the big leaves, repot, talk to it | water (5), talk (10) | OK |
| `garden.fiddleFig` | water, wipe the leaves, turn it to the light | water (5), talk (10) | OK |
| `garden.snakePlant` | water now and then | water (5), talk (10) | OK |
| `garden.cacti` | water rarely, admire the bloom | water (5), talk (10) | OK |
| `garden.succulents` | water rarely | water (5), talk (10) | OK |
| `garden.bostonFern` | water and mist it | water (5), talk (10) | OK |
| `garden.palm` | water, mist | water (5), talk (10) | OK |
| `garden.orchid` | water (soak), admire the flowers | water (5), talk (10), admire (5) | OK |
| `garden.bonsai` | prune and wire it (seated at a bench), water | prune (45, standing; I-garden-09), water (5) | pose wrong, use right |

## G2. Flowers (15)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `flowerbed` Raised flower bed | OK | OK | OK | | `sheets/obj-flowerbed.png`, `play/flowerbed-*.png`, `play-elder/flowerbed-*.png` |
| `garden.tulips` Tulips | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.tulips.png` |
| `garden.daffodils` Daffodils | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.daffodils.png` |
| `garden.daisies` Daisies | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.daisies.png` |
| `garden.sunflowers` Sunflowers | OK | I-garden-07 | I-garden-07 | I-garden-07 S3 content; I-held-01 S2 code (held ledger) | `sheets/obj-garden.sunflowers.png` |
| `garden.poppies` Poppies | OK | OK | I-garden-08 | I-garden-08 S3 content; I-held-01 S2 code (held ledger) | `sheets/obj-garden.poppies.png` |
| `garden.marigolds` Marigolds | OK | OK | I-garden-08 | I-garden-08 S3 content; I-held-01 S2 code (held ledger) | `sheets/obj-garden.marigolds.png` |
| `garden.pansies` Pansies | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.pansies.png` |
| `garden.lilies` Lilies | OK | I-garden-06 | I-garden-06 | I-garden-06 S3 renderer-table; I-held-01 S2 code (held ledger) | `sheets/obj-garden.lilies.png` |
| `garden.lupins` Lupins | OK | I-garden-06 | I-garden-06 | I-garden-06 S3 renderer-table; I-held-01 S2 code (held ledger) | `sheets/obj-garden.lupins.png` |
| `garden.forgetMeNots` Forget-me-nots | OK | OK | I-garden-08 | I-garden-08 S3 content; I-held-01 S2 code (held ledger) | `sheets/obj-garden.forgetMeNots.png` |
| `garden.cosmos` Cosmos | OK | I-garden-06 | I-garden-06 | I-garden-06 S3 renderer-table; I-held-01 S2 code (held ledger) | `sheets/obj-garden.cosmos.png` |
| `garden.meadow` Wildflower meadow | M-garden-06 | I-garden-04 | I-garden-04 | M-garden-06 S3 model; I-garden-04 S2 renderer-table | `sheets/obj-garden.meadow.png` |
| `garden.geraniums` Potted geraniums | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.geraniums.png` |
| `garden.flowerBox` Flower box | OK | I-garden-06 | I-garden-06 | I-garden-06 S3 renderer-table; I-held-01 S2 code (held ledger) | `sheets/obj-garden.flowerBox.png` |

Notes:
- **Sizes:** every bed is a 0.84–0.89 m round of soil on one tile, inside `flowerBed`, with the flowers at their species' heights (pansies 0.26, forget-me-nots 0.28, marigolds 0.34, daisies 0.38, daffodils 0.49, tulips 0.62, poppies 0.66, cosmos 0.88, lilies 0.94, lupins 0.99, sunflowers 2.11 m). The geranium pot (0.65) and the petunia trough on legs (0.73) are inside `planter`.
- **Species:** each reads as itself: tulip cups on stems, yellow trumpets, white daisies with yellow eyes, sunflower heads on 2 m stalks, red poppies on thin stems over leaf mounds, orange marigold pompoms, pansy faces, orange and pink lilies, lupin spikes (solid cones, dark navy and maroon: stylised but readable), blue forget-me-not clouds, cosmos on feathery stems, red geraniums, petunias spilling over a trough. The meadow is the exception (M-garden-06).
- **Smelling** is the `crouch` clip with the nose lowered (`clips/garden.ts:104-108`, `characters.ts:3366-3370`): right for the low beds, too low for the waist-high ones (I-garden-06).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `flowerbed` | water, weed and tend, smell the flowers, cut some | tend the flowers (20), smell the flowers (5) | OK |
| `garden.tulips` | water, cut a bunch for a vase, deadhead | water (10), smell (5) | cutting missing (I-garden-08) |
| `garden.daffodils` | water, cut a bunch, smell (light scent) | water (10), smell (5) | cutting missing (I-garden-08) |
| `garden.daisies` | water, pick a few, make a daisy chain | water (10), smell (5) | picking missing (I-garden-08) |
| `garden.sunflowers` | water, admire (look up), cut a head, save seeds | water (10), admire (5, looks down: I-garden-07) | pose wrong |
| `garden.poppies` | water, admire (no scent to speak of) | water (10), smell (5, "Sweet scent") | smell on a scentless flower (I-garden-08) |
| `garden.marigolds` | water, deadhead (pungent leaves) | water (10), smell (5, "Sweet scent") | smell is sharp, not sweet (I-garden-08) |
| `garden.pansies` | water, deadhead | water (10), smell (5) | OK |
| `garden.lilies` | water, smell (heady scent), cut for a vase | water (10), smell (5) | cutting missing (I-garden-08) |
| `garden.lupins` | water, admire, deadhead | water (10), smell (5) | OK |
| `garden.forgetMeNots` | water, admire (scentless) | water (10), smell (5, "Sweet scent") | smell on a scentless flower (I-garden-08) |
| `garden.cosmos` | water, cut for a vase | water (10), smell (5) | cutting missing (I-garden-08) |
| `garden.meadow` | walk or lie in it, pick wildflowers, watch the bees | pick a wildflower (5) | OK |
| `garden.geraniums` | water, deadhead, smell the leaves | water (5), smell (5) | OK |
| `garden.flowerBox` | water, deadhead, smell | water (5), smell (5) | OK |

## G3. Shrubs (11)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bush` Round shrub | OK | OK | I-held-01 | I-held-01 S2 code (seeded; held ledger: no shears) | `sheets/obj-bush.png`, `play/bush-trim-*.png` |
| `garden.boxwood` Boxwood ball | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no shears) | `sheets/obj-garden.boxwood.png` |
| `garden.topiary` Topiary cone | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no shears) | `sheets/obj-garden.topiary.png` |
| `garden.hedge` Clipped hedge | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no shears) | `sheets/obj-garden.hedge.png` |
| `garden.hydrangea` Hydrangea | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.hydrangea.png` |
| `garden.roses` Rose bush | M-garden-06 | OK | I-held-01 | M-garden-06 S3 model; I-held-01 S2 code (held ledger: no secateurs) | `sheets/obj-garden.roses.png` |
| `garden.azalea` Azalea | M-garden-06 | OK | I-held-01 | M-garden-06 S3 model; I-held-01 S2 code (held ledger) | `sheets/obj-garden.azalea.png` |
| `garden.lavender` Lavender | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no secateurs) | `sheets/obj-garden.lavender.png` |
| `garden.fern` Garden fern | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.fern.png` |
| `garden.grass` Ornamental grass | OK | n/a | n/a | | `sheets/obj-garden.grass.png` |
| `garden.hosta` Hosta | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.hosta.png` |

Notes:
- **Sizes:** all inside `clippedShrub`, `floweringShrub` and `perennial`. The hedge is 1.55 m long on a one-tile footprint by design: rows overlap into one wall (`plants.py:889-893`, the street hedge's note), so M4 is OK.
- **Clips:** the boxwood's, azalea's, Japanese maple's and topiary's stray sprigs shrink away while trimmed, the rose and lavender lose a stem (`clips/clips-garden.*-all-t99.png`): the right part goes for the right use, at use start (a 25-minute use, so the timing doesn't show).
- **Trimming** is `prune` (`characters.ts:513`: `interact`, `spell`, `interact`, a gripping hand): hands at the shrub's face, standing, which fits shrubs of 0.75–1.8 m; the shears are missing (I-held-01).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bush` | clip it into shape | trim the shrub (25) | OK |
| `garden.boxwood` | clip it into a ball | trim (25) | OK |
| `garden.topiary` | clip the cone | trim (25) | OK |
| `garden.hedge` | trim the sides and top | trim (25) | OK |
| `garden.hydrangea` | water (thirsty), admire, cut a flower head | water (10), admire (5) | OK |
| `garden.roses` | smell, deadhead and prune, cut a rose | smell (5), prune (25) | OK |
| `garden.azalea` | water, prune after flowering | water (10), prune (25) | OK |
| `garden.lavender` | smell, cut and dry the spikes | smell (5), cut (25) | OK |
| `garden.fern` | water in a dry spell | water (10) | OK |
| `garden.grass` | cut back in spring | none | decor, no use needed |
| `garden.hosta` | water, watch for slugs | water (10) | OK |

## G4. Edibles (5)

Family: `tools/art/blender/families/plants.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `garden.vegPatch` Vegetable patch | OK | I-garden-03 | I-garden-03, I-garden-10 | I-garden-03 S2 code; I-garden-10 S3 decision | `sheets/obj-garden.vegPatch.png`, `extra/harvest/s-garden.vegPatch-harvest.png` |
| `garden.tomatoes` Tomato plants | OK | I-garden-03 | I-garden-03, I-garden-04 | I-garden-03 S2 code; I-garden-04 S2 renderer-table (tend kneels at a 1.2 m plant) | `sheets/obj-garden.tomatoes.png`, `extra/harvest/s-garden.tomatoes-harvest.png` |
| `garden.strawberries` Strawberry bed | M-garden-07 | I-garden-03, I-garden-04 | I-garden-03, I-garden-04, I-garden-10 | M-garden-07 S3 model; I-garden-03 S2 code; I-garden-04 S2 renderer-table; I-garden-10 S3 decision | `sheets/obj-garden.strawberries.png`, `extra/harvest/s-garden.strawberries-harvest.png`, `extra/stages-strawberries.png` |
| `garden.herbs` Herb planter | OK | OK | I-held-01 | I-held-01 S2 code (held ledger) | `sheets/obj-garden.herbs.png` |
| `garden.pumpkins` Pumpkin patch | OK | I-garden-03, I-garden-04 | I-garden-03, I-garden-04, I-garden-10 | I-garden-03 S2 code; I-garden-04 S2 renderer-table; I-garden-10 S3 decision | `sheets/obj-garden.pumpkins.png`, `extra/harvest/s-garden.pumpkins-harvest.png` |

Notes:
- **Ripe harvests** were skipped in the first pass (a crop bought is just planted); this wave ran them on `review-garden` with `--use-existing` (`extra/harvest/`, adult, 0 errors), with the outdoor ledger's `raisedBed` beside them for the shared causes. `natureLover.veggieBed` has no `harvest` (its ripe use is `pickSnack`, `anim: eat`): for the packs wave.
- **Sizes:** inside `vegPlot` and `planter`. The herb planter's 1.01 m is the chives standing over a 0.73 m trough on legs (the `pick` clip is fitted to that trough, `clips/garden.ts:46-61`): OK.
- **Stages** (`clips-stages/`): the veg patch's lettuces and cabbages, staked tomatoes, the strawberries on straw and the pumpkins on their vines read right ripe; planted, the strawberries don't (M-garden-07). Picked, the veg patch is bare soil and the pumpkins only leaves, which reads as cleared.
- **Tending** is the `kneel` clip (both knees, pulling weeds, `clips/garden.ts:63-101`): right for the low beds; for "Tie up and water" at ripe 1.2 m tomatoes it stays at the soil (I-garden-04).
- **The herb planter** picks with the `pick` clip at the trough, standing: OK.

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `garden.vegPatch` | sow, weed and water, harvest to cook | weed and water (30), harvest (20, ripe; fills hunger on the spot: I-garden-10) | OK, effect odd |
| `garden.tomatoes` | tie up, water, pinch out, pick | tie up and water (30, kneeling), pick (20, ripe) | OK, pose (I-garden-04) |
| `garden.strawberries` | weed, net, pick and eat a few | weed and water (30), pick (20, ripe) | OK |
| `garden.herbs` | pick a sprig for cooking, water | pick herbs (5, cooking boost), water (5) | OK |
| `garden.pumpkins` | weed, water, cut a pumpkin to cook or carve | weed and water (30), harvest (20, ripe; eaten raw on the spot: I-garden-10) | OK, effect odd |

## G5. Trees (12)

Family: `tools/art/blender/families/trees.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `tree` Oak tree | OK | I-garden-02 | I-garden-02 | I-garden-02 S2 clip | `sheets/obj-tree.png`, `play/tree-shade-01.png`, `sheets/tree-walk.png` |
| `pine` Pine tree | M-garden-01 | n/a | OK | M-garden-01 S2 model | `sheets/obj-pine.png`, `sheets/tree-walk.png`, `scenes/tree-walk-pine*.png` |
| `garden.maple` Maple tree | M-garden-05 | I-garden-02 | I-garden-02 | M-garden-05 S3 model; I-garden-02 S2 clip | `sheets/obj-garden.maple.png` |
| `garden.birch` Silver birch | OK | I-garden-02 | I-garden-02 | I-garden-02 S2 clip | `sheets/obj-garden.birch.png` |
| `garden.cherry` Cherry blossom tree | OK | I-garden-02 | I-garden-02 | I-garden-02 S2 clip | `sheets/obj-garden.cherry.png` |
| `garden.apple` Apple tree | OK | I-garden-02, I-garden-05 | I-garden-02, I-garden-05, I-garden-10 | I-garden-05 S2 clip; I-garden-02 S2 clip; I-garden-10 S3 decision | `sheets/obj-garden.apple.png`, `extra/close/s-garden.apple-pick.png`, `extra/s-garden.apple-pick.png` |
| `garden.japaneseMaple` Japanese maple | M-garden-01 | I-garden-02 | M-garden-01, I-garden-02 | M-garden-01 S2 model (widened); I-garden-02 S2 clip; I-held-01 S2 code (held ledger) | `sheets/obj-garden.japaneseMaple.png`, `extra/close/s-jm-willow.png` |
| `garden.magnolia` Magnolia | M-garden-05 | I-garden-02 | I-garden-02, I-garden-08 | M-garden-05 S3 model; I-garden-02 S2 clip; I-garden-08 S3 content | `sheets/obj-garden.magnolia.png` |
| `garden.willow` Weeping willow | M-garden-02 | I-garden-02 | I-garden-02 | M-garden-02 S2 model; I-garden-02 S2 clip | `sheets/obj-garden.willow.png`, `extra/close/s-jm-willow.png` |
| `garden.spruce` Blue spruce | M-garden-01 | n/a | OK | M-garden-01 S2 model (widened) | `sheets/obj-garden.spruce.png` |
| `garden.cypress` Italian cypress | M-garden-03 | n/a | OK | M-garden-03 S3 model | `sheets/obj-garden.cypress.png` |
| `garden.lemonTree` Potted lemon tree | M-garden-04 | I-garden-05 | I-garden-05, I-garden-10 | M-garden-04 S3 model; I-garden-05 S2 clip; I-garden-10 S3 decision; I-held-01 S2 code (held ledger) | `sheets/obj-garden.lemonTree.png`, `extra/close/s-garden.lemonTree-pick.png` |

Notes:
- **Sizes:** the broad trees are 4.5–8.2 m tall and 3.5–5.5 m across, the size of a garden tree a few decades old; the small ones (Japanese maple 3.7, cherry 5.5 m) and the apple on dwarfing stock (5.6 m) too. Out of range: the cypress's width (M-garden-03) and the lemon tree's height (M-garden-04).
- **Species:** the birch's white bark with black marks, the cherry's pink crown, the apple's fruit, the Japanese maple's layered red crowns, the conifers' tiers and the potted lemon read as themselves at the game camera. The maple is the oak's crown again and the magnolia's blooms are balls (M-garden-05); the willow's curtains are spikes (M-garden-02).
- **The camera** at the default play framing shows only the trunk of a tree; `play/trees-wide/` (radius 16, y 3) and `extra/close/` (radius 7, y 1.4) are the shots to read.
- **Clips:** the apple and the lemon shrink away from the crown as they are picked; the Japanese maple's sprigs go when it is shaped. The picked apple hangs 1.2 m over the picker's head (I-garden-05).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `tree` | sit or lie in its shade, climb it (children), rake the leaves | relax in the shade (30, standing) | sitting missing from the use (I-garden-02) |
| `pine` | year-round green, shelter | none | decor, no use needed |
| `garden.maple` | sit in its shade, rake the leaves | relax in the shade (30, standing) | I-garden-02 |
| `garden.birch` | light shade, admire the bark | relax in the shade (30, standing) | I-garden-02 |
| `garden.cherry` | admire the blossom, sit under it (hanami) | relax in the shade (30), admire (5) | I-garden-02 |
| `garden.apple` | pick apples in season, prune in winter, sit under it | pick an apple (15, eats it standing; any time: I-garden-05, I-garden-10), relax in the shade (30) | pose and conditions wrong |
| `garden.japaneseMaple` | admire the leaves, shape it lightly | relax in the shade (30, inside the crown: M-garden-01), shape (25) | place wrong |
| `garden.magnolia` | admire the flowers, sit under it | relax in the shade (30), smell (5, looking up at 3 m: I-garden-08) | OK, label odd |
| `garden.willow` | sit inside the curtain, shade by water | relax in the shade (30, standing) | I-garden-02 |
| `garden.spruce` | year-round green, shelter, a Christmas tree | none | decor, no use needed |
| `garden.cypress` | line a path, a windbreak | none | decor, no use needed |
| `garden.lemonTree` | water, pick a lemon to cook with or for a drink, bring it in for winter | pick a lemon (15, eaten like an apple: I-garden-10), water (5) | effect wrong |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

Not tied to one row: I-garden-01.

Causes shared beyond this ledger, moved at the consolidation (2026-10-21): I-garden-03 and I-garden-10 to [systems.md](systems.md) (the sim picks a crop at use start; harvests feed on the spot), I-garden-02 and I-garden-04 to [residents.md](residents.md) (`ACTIONS.relax.stand` at trees; `ACTIONS.harvest` and `tend`). I-held-01 is the held ledger's.

### I-garden-01 — Residents once didn't reach things in a crowded garden

- **Severity:** S3
- **Rubric:** I2
- **Fix class:** code
- **Where:** `crates/sim-core/tests/crowded_garden.rs`
- **Evidence:** Seen once in play; a sim test of eleven in a row could not reproduce it. The garden wave's play runs placed one thing at a time on an empty garden and could not show it either.
- **Status:** not checked (the evidence places one object at a time; no crowded garden was played)
- **Source:** `docs/objects/open.md` Still open › Content and sim

### M-garden-01 — Trees whose foliage comes down to head height over a one-tile footprint: residents walk into them and use them from inside

- **Severity:** S2
- **Rubric:** M4, M7
- **Fix class:** model
- **Where:**
  - `tools/art/blender/families/trees.py:270-277` (`pine`: `conifer(rng, 6, 6.0, 1.4, …)`: six tiers, the first 6.0 m across with its foot at 1.4 m, on a 1 × 1 footprint);
  - `trees.py:280-287` (`spruce`: nine tiers, the first 5.4 m across from 0.35 m);
  - `trees.py:199-212` (`japanese_maple`: the first crown at 2.0 m, 3.9 m across, its lumps down to about 1.3 m).
- **Evidence:**
  - Pine (pilot): `sheets/tree-walk.png` (top row): a resident standing 1 m from the pine's trunk is hidden inside the foliage, seen from the game's camera and from low down; the oak beside it (bottom row) leaves them in clear view under a crown that starts at 3 m. `play/pine-no-uses.png` is taken from inside the cone (the camera radius comes from the 1 × 1 footprint). A garden pine's lowest boughs do sweep low, but nothing stops residents walking under them (the sim sees one tile).
  - Japanese maple (garden wave): both its uses are taken from the front tile, inside the crown: "Relax in the shade" and "Shape the maple" show the resident from the knees down, the rest inside the red foliage (`extra/close/s-jm-willow.png`, first two frames; `play/trees-wide/garden.japaneseMaple-shape-01.png`). A real Japanese maple is often that low, but you stand outside it to prune it.
  - Blue spruce (garden wave): its skirt is 5.6 m across from 0.35 m up on one tile (`sheets/obj-garden.spruce.png`), so anyone walking within 2.7 m of the trunk walks into it, as at the pine.
  - Fix: lift the first tier or crown (to about 2.2 m), or narrow it, or give each a footprint as wide as its skirt (and, for the Japanese maple, a slot outside the crown).
- **Status:** confirmed and widened (garden wave: the Japanese maple, the blue spruce)
- **Source:** pilot (`scripts/scenes.mjs` pine); wave: garden

### I-garden-02 — moved to residents.md

### M-garden-02 — The weeping willow's curtains are thick solid spikes that stop 1–2.6 m above the lawn

- **Severity:** S2
- **Rubric:** M5, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/trees.py:240-252` (`willow`: 26 strands, each a cone 0.5–0.7 m across at the crown narrowing to 6 cm, 2.4–3.5 m long from 4.6–5.0 m up, so their tips end 1.1–2.6 m off the ground; 8 inner cones 0.8 m across)
- **Evidence:** `sheets/obj-garden.willow.png` (gallery and game camera), `extra/close/s-jm-willow.png` (third frame): the curtains read as green stalactites or icicles, not as the long thin trailing strands that make a weeping willow; the content says they "sweep down to the lawn" (`garden.json:362`). From the game's camera a resident relaxing under it is hidden behind the solid wedges. Many thin strips (or a few curtain sheets with a strand texture) reaching to about 0.5 m off the ground would read right.
- **Status:** open
- **Source:** wave: garden

### M-garden-03 — The Italian cypress is twice as broad as a real one

- **Severity:** S3
- **Rubric:** M1, M5
- **Fix class:** model
- **Where:** `tools/art/blender/families/trees.py:290-306` (`cypress`: column lumps 2.0, 1.9, 1.7, 1.4 and 1.05 m across); `trees.py:344` (`size=(2.12, 7.95, 2.05)`)
- **Evidence:** `cargo review sizes`: W 2.12 and D 2.05 m against the `columnarConifer` range 0.5–1.5 m (out past +10 %). At 7.95 m tall it is 3.7 times as tall as it is wide; an Italian cypress is a pencil, 6 to 10 times (`sheets/obj-garden.cypress.png`: it reads as a fat column next to the spruce and the topiary). Narrow the column to about 1–1.2 m.
- **Status:** open
- **Source:** wave: garden

### M-garden-04 — The potted lemon tree is 2.37 m tall

- **Severity:** S3
- **Rubric:** M1
- **Fix class:** model
- **Where:** `tools/art/blender/families/trees.py:311-325` (`lemon_tree`: the crown at 1.85 m); `trees.py:345` (`size=(1.288, 2.374, 1.344)`)
- **Evidence:** `cargo review sizes`: H 2.374 > 2.2 (`pottedTree` 0.8–2.0 m, +10 %). A potted citrus kept indoors or on a terrace is about 1–2 m with its pot; at 2.37 m its crown is above an adult's head (`sheets/obj-garden.lemonTree.png`, `extra/close/s-garden.lemonTree-pick.png`), which is also why the lemon picked is out of reach (I-garden-05). Lower the crown by 0.3–0.4 m.
- **Status:** open
- **Source:** wave: garden

### M-garden-05 — The maple is the oak's crown, and the magnolia's blooms are balls, some floating off it

- **Severity:** S3
- **Rubric:** M5, M6, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/trees.py:139-146` (`maple`: the same `crown` of round lumps as `oak`, `trees.py:127-136`, only greener); `trees.py:226-227` (`magnolia`: 16 blooms as `dots` of radius 0.24 set out to 0.98 of the crown's radius)
- **Evidence:** `sheets/obj-garden.maple.png` next to `sheets/obj-tree.png`: the two can't be told apart without the label (no lobed-leaf texture, no broader, denser dome). `sheets/obj-garden.magnolia.png` (gallery): the magnolia's flowers are pink and white spheres on the crown's edge, several standing clear of it in the air; a magnolia's are big upright goblets on the branch tips, often before the leaves. Both trees are recognisable as "a tree" and "a pink tree", which is why S3.
- **Status:** open
- **Source:** wave: garden

### M-garden-06 — The rose bush reads as a berry bush (the azalea looks more like roses), and the meadow has no long grass

- **Severity:** S3
- **Rubric:** M5, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/plants.py:944-953` (`rosebush`: a dark round shrub with 15 red `dots` of radius 0.1, round lumps); `plants.py:956-964` (`azalea`: 20 larger, flatter crimson-pink `dots`); `plants.py:720-746` (`meadow`)
- **Evidence:** `sheets/obj-garden.roses.png`: a dark ball dotted with small red spheres, like a holly or a currant bush, where a rose bush is open, upright canes with cupped blooms. `sheets/obj-garden.azalea.png`: the azalea's blooms are the rose-like ones (big, layered, crimson). `sheets/obj-garden.meadow.png`: "a patch of long grass and wildflowers" is three round leafy mounds with flowers on stalks; no grass shows.
- **Status:** open
- **Source:** wave: garden

### M-garden-07 — Planted, the strawberry plants are rings of three leaves with a hole in the middle

- **Severity:** S3
- **Rubric:** M5, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/plants.py:1167-1171` (`strawberries`: each crown is three leaves tilted 45° out from one point, 120° apart, nothing in the middle)
- **Evidence:** `extra/stages-strawberries.png` (planted, growing, ripe, picked), `play/c3/garden.strawberries-tend-01.png`: at the planted stage the three leaves shrink to a triangle round a straw-coloured hole, seven of them on the bed; even ripe the crowns are open in the middle. A strawberry plant is a dense rosette of trifoliate leaves on stalks round a crown. A few upright leaves in the middle would close it.
- **Status:** open
- **Source:** wave: garden

### M-garden-08 — The cactus trio and the succulent bowl only stand on the floor

- **Severity:** S3
- **Rubric:** M4
- **Fix class:** decision
- **Where:** `web/public/content/garden.json:1870` (`garden.cacti`) and `:1912` (`garden.succulents`): no `layer`, so floor; `tools/art/blender/families/plants.py:288-334`
- **Evidence:** `sheets/obj-garden.cacti.png`, `sheets/obj-garden.succulents.png`: three pots 0.3–0.5 m high and a 0.62 m bowl 0.17 m high stand on the floor tile, ankle high, where real ones sit on a sill, a shelf or a table. The tabletop catalog has small pots that do (`potSucculent`, `potCactus`, `catalog2/tabletop.json:379`, `layer: surface`). Either make these two surface things (the bowl then needs to shrink to about 0.3 m) or keep them as floor pots by design.
- **Status:** open
- **Source:** wave: garden

### I-garden-03 — moved to systems.md

### I-garden-04 — moved to residents.md

### I-garden-05 — Picking an apple or a lemon is eating at the trunk; the fruit shrinks away 1.2 m over the picker's head

- **Severity:** S2
- **Rubric:** I3, A1, I7
- **Fix class:** clip
- **Where:** `web/public/content/garden.json:221-223` (apple `pick`: `anim: eat`) and `:416-418` (lemon `pick`: `anim: eat`); `web/src/render/babylon/characters.ts:457` (`ACTIONS.eat.stand`: `eat`, `idle`); `tools/art/blender/families/trees.py:191` (the picked apple at 2.95 m, 1.95 m in front of the trunk) and `:311-325` (the lemon in a crown at 1.85 m)
- **Evidence:** `extra/close/s-garden.apple-pick.png`: the resident stands at the trunk with the eating clip, the hand at the mouth and empty, for 15 minutes; the apple the `pick` clip shrinks hangs on its own in the air about 1.2 m above her head, never reached. `extra/close/s-garden.lemonTree-pick.png`: the same at the lemon tree, the lemon in the crown above her. No clip reaches up into a tree (the `pick` clip is fitted to the waist-high herb trough). The fix is a reach-up-and-twist pick, then the eat clip with the fruit in hand (the fruit prop is I-held-01, held ledger).
- **Status:** open
- **Source:** wave: garden

### I-garden-06 — Smelling is a crouch for every flower, also for waist-high ones

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:527-528` (`ACTIONS.smell.stand`: `crouch` for every def); `clips/garden.ts:103-108` (`crouchPose`: the pelvis 0.42 m down, the head about 0.8 m up)
- **Evidence:** `sheets/obj-garden.flowerBox.png` (smell): the resident crouches under the petunia trough, whose flowers are at 0.73 m, her head level with its side; `sheets/obj-garden.lilies.png`, `obj-garden.lupins.png`, `obj-garden.cosmos.png`: she crouches to the stems of flowers 0.9–1.0 m tall, below their heads. Fine for the low beds (tulips to pansies) and the pots. For things whose flowers are above about 0.7 m, a standing lean-in (the `pick` clip's lean without the hands, or `idle` with the head bent) fits.
- **Status:** open
- **Source:** wave: garden

### I-garden-07 — "Admire the sunflowers" looks down at the soil under 2 m flowers

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** content
- **Where:** `web/public/content/garden.json:1007-1009` (sunflowers `admire`: `anim: look`); `web/src/render/babylon/characters.ts:3377-3381` (`look` lowers the head 0.2 rad at any object but the telescope)
- **Evidence:** `sheets/obj-garden.sunflowers.png` (admire): the resident stands at the bed with the head tipped down while the flower heads are at 1.6–2.1 m. The cherry's and the magnolia's uses take `anim: admire`, which looks up into the crown (`characters.ts:3371-3376`); the sunflowers want the same.
- **Status:** open
- **Source:** wave: garden

### I-garden-08 — Every flower bed is "water" and "smell": scentless flowers are smelled, and none can be cut for a vase

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/garden.json:1052` (poppies `smell`, `feeling: garden.sweetScent`), `:1098` (marigolds), `:1282` (forget-me-nots), `:337-339` (magnolia "Smell the magnolia": `anim: admire`); the cut-flower uses would go beside the tulips', daffodils', lilies', cosmos' and daisies' (`:869`, `:915`, `:1190`, `:1328`, `:961`)
- **Evidence:** `cargo review uses` (`tables/uses-garden.md`): the 11 beds have the same two uses. Poppies and forget-me-nots have no scent to speak of and marigolds' is sharp, yet each gives "Sweet scent". The magnolia's smell is looking up at flowers about 3 m above the nose (`sheets/obj-garden.magnolia.png`). The reference's real uses for a bed (`reference.json` `flowerBed`: water, smell, pick a few) include cutting, which the game has nowhere: the meadow's "Pick a wildflower" is the only picking. Make the scentless beds' second use "Admire" (anim `look`), relabel the magnolia's "Admire the magnolia", and decide at triage whether cut flowers (for a vase) are wanted.
- **Status:** open
- **Source:** wave: garden

### I-garden-09 — The bonsai is pruned for 45 minutes standing, stooped over a knee-high table

- **Severity:** S3
- **Rubric:** I3, A1, M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/plants.py:410-418` (`bonsai`: the table top at 0.36–0.38 m on 0.34 m legs); `web/public/content/garden.json:2103-2105` (`prune`: `anim: tinker`, 45 min, no `pose`)
- **Evidence:** `sheets/obj-garden.bonsai.png` (prune, water): the resident bends over a tray at 0.4 m with the `tinker` clip (`interact`, `spell`, `pickUp`, `characters.ts:518-522`), hands reaching down. Bonsai are pruned at a bench or table at 0.75–0.95 m, often seated (`reference.json` `bonsaiBench`: top 75–95 cm, the packs' `loner.bonsaiBench`). Raise the table to about 0.75 m (the tree then stands at 1.3 m), or keep it low and give the use a seat nearby.
- **Status:** open
- **Source:** wave: garden

### I-garden-10 — moved to systems.md

