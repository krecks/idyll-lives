# Garden plants and trees

Plants and trees have their own pipeline today (`tools/art/nature.py`: foliage cards that face the camera and sway in
the shader, `vary` for natural variation, `*.far` keys for distance). A Blender plant keeps those: leaves as cards
with the `foliage` finish, the trunk or pot as a mesh, and a `.far.glb` (E7) before trees and hedges move over.
Wind sway stays the shader's job; clips only show a resident's use.

The ten G1 houseplants are done (`families/plants.py`, notes in `families/plants.md`): a file's parts on `foliage`
(and `bark`) are drawn with the shared finish materials the nature shader hooks, so they sway in the wind as the
placeholders do, with nothing more to build.

The fifteen G2 flowers are done in the same family: each mound bed's colour part is its soil mound ("Bed"),
and its petals where they are all one colour; the geraniums' is the pot, the flower box's the box, the raised
bed's its boards. The meadow has none.

The eleven G3 shrubs are done too. They dress every lot (`natureDecor`) and the bush the woods, so they cost what
their placeholders do; they, the flowers and the edibles each have a far version, the same build at low detail
(`far_of`). Only the topiary has a pot; the flowering shrubs colour their flowers instead, the others nothing.

The five G4 edibles are done: the four crops have a growth clip only (`grow`, G.stages: their leaves and their
crop as two parts), no harvest clip; harvesting moves them to the picked stage, so the crop goes as the clip plays
on, and grows back over the days. The herb planter, which doesn't grow, has `harvest` for `pickHerbs`.

## G1. Houseplants (10) — **Done** (in Validate › Houseplants)

- **Family:** `tools/art/blender/families/plants.py`
- **Validate group:** `houseplants` (Validate › Houseplants)
- **Placeholders to match:** base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `plant` Potted plant | 1x1 · floor | water:water | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.monstera` Monstera | 1x1 · floor | water:water talk:talk | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.fiddleFig` Fiddle-leaf fig | 1x1 · floor | water:water talk:talk | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.snakePlant` Snake plant | 1x1 · floor | water:water talk:talk | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.cacti` Cactus trio | 1x1 · floor | water:water talk:talk | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.succulents` Succulent bowl | 1x1 · floor | water:water talk:talk | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.bostonFern` Fern on a stand | 1x1 · floor | water:water talk:talk | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.palm` Parlour palm | 1x1 · floor | water:water talk:talk | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.orchid` Orchid | 1x1 · floor | water:water talk:talk admire:look | garden / houseplants | Pot | Static | Static; the foliage sways in the shader. |
| `garden.bonsai` Bonsai | 1x1 · floor | prune:tinker water:water | garden / houseplants | Pot | Use | `trim`: a stray twig shrinks away for prune. |

## G2. Flowers (15) — **Done** (in Validate › Flowers)

- **Family:** `tools/art/blender/families/plants.py`
- **Validate group:** `flowers` (Validate › Flowers)
- **Placeholders to match:** base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `flowerbed` Raised flower bed | 1x1 · floor | tend:tend smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.tulips` Tulips | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.daffodils` Daffodils | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.daisies` Daisies | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.sunflowers` Sunflowers | 1x1 · floor | water:water admire:look | garden / flowers | Pot or box | Static | Static. |
| `garden.poppies` Poppies | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.marigolds` Marigolds | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.pansies` Pansies | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.lilies` Lilies | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.lupins` Lupins | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.forgetMeNots` Forget-me-nots | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.cosmos` Cosmos | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.meadow` Wildflower meadow | 1x1 · floor | pickFlower:harvest | garden / flowers | Pot or box | Use | `pick`: one flower shrinks away for pickFlower and grows back after. |
| `garden.geraniums` Potted geraniums | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |
| `garden.flowerBox` Flower box | 1x1 · floor | water:water smell:smell | garden / flowers | Pot or box | Static | Static. |

## G3. Shrubs (11) — **Done** (in Validate › Shrubs)

- **Family:** `tools/art/blender/families/plants.py`
- **Validate group:** `shrubs` (Validate › Shrubs)
- **Placeholders to match:** base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bush` Round shrub | 1x1 · floor | trim:prune | garden / shrubs | Pot | Static | Static. |
| `garden.boxwood` Boxwood ball | 1x1 · floor | trim:prune | garden / shrubs | Pot | Use | `trim`: the shaggy top sprigs shrink away for trim. |
| `garden.topiary` Topiary cone | 1x1 · floor | trim:prune | garden / shrubs | Pot | Use | `trim`. |
| `garden.hedge` Clipped hedge | 1x1 · floor | trim:prune | garden / shrubs | Pot | Use | `trim`. |
| `garden.hydrangea` Hydrangea | 1x1 · floor | water:water admire:look | garden / shrubs | Pot | Static | Static. |
| `garden.roses` Rose bush | 1x1 · floor | smell:smell prune:prune | garden / shrubs | Pot | Use | `cut`: one bloom shrinks away for prune. |
| `garden.azalea` Azalea | 1x1 · floor | water:water trim:prune | garden / shrubs | Pot | Use | `trim`. |
| `garden.lavender` Lavender | 1x1 · floor | smell:smell cut:prune | garden / shrubs | Pot | Use | `cut`: a few stems shrink away. |
| `garden.fern` Garden fern | 1x1 · floor | water:water | garden / shrubs | Pot | Static | Static. |
| `garden.grass` Ornamental grass | 1x1 · floor | — | garden / shrubs | Pot | Static | Static. |
| `garden.hosta` Hosta | 1x1 · floor | water:water | garden / shrubs | Pot | Static | Static. |

## G4. Edibles (5) — **Done** (in Validate › Edibles)

- **Family:** `tools/art/blender/families/plants.py`
- **Validate group:** `edibles` (Validate › Edibles)
- **Placeholders to match:** base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `garden.vegPatch` Vegetable patch | 1x1 · floor | tend:tend harvest:harvest | garden / edibles | Bed | Use | `harvest`: the ripe crop shrinks away during harvest and is back after. G growth stages from seedling to ripe (E4). |
| `garden.tomatoes` Tomato plants | 1x1 · floor | tend:tend harvest:harvest | garden / edibles | Bed | Use | `harvest`: the ripe crop shrinks away during harvest and is back after. G growth stages from seedling to ripe (E4). |
| `garden.strawberries` Strawberry bed | 1x1 · floor | tend:tend harvest:harvest | garden / edibles | Bed | Use | `harvest`: the ripe crop shrinks away during harvest and is back after. G growth stages from seedling to ripe (E4). |
| `garden.herbs` Herb planter | 1x1 · floor | pickHerbs:harvest water:water | garden / edibles | Bed | Use | `harvest`: a few sprigs shrink away for pickHerbs. |
| `garden.pumpkins` Pumpkin patch | 1x1 · floor | tend:tend harvest:harvest | garden / edibles | Bed | Use | `harvest`: the ripe crop shrinks away during harvest and is back after. G growth stages from seedling to ripe (E4). |

## G5. Trees (12)

- **Family:** `tools/art/blender/families/trees.py`
- **Validate group:** `trees` (Validate › Trees)
- **Placeholders to match:** base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `tree` Oak tree | 1x1 · floor | shade:relax | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `pine` Pine tree | 1x1 · floor | — | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `garden.maple` Maple tree | 1x1 · floor | shade:relax | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `garden.birch` Silver birch | 1x1 · floor | shade:relax | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `garden.cherry` Cherry blossom tree | 1x1 · floor | shade:relax admire:admire | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `garden.apple` Apple tree | 1x1 · floor | pick:eat shade:relax | garden / trees | - | Use | `pick`: an apple shrinks away for pick. |
| `garden.japaneseMaple` Japanese maple | 1x1 · floor | shade:relax shape:prune | garden / trees | - | Use | `trim` for shape. |
| `garden.magnolia` Magnolia | 1x1 · floor | shade:relax smell:admire | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `garden.willow` Weeping willow | 1x1 · floor | shade:relax | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `garden.spruce` Blue spruce | 1x1 · floor | — | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `garden.cypress` Italian cypress | 1x1 · floor | — | garden / trees | - | Static | Static; the canopy sways in the shader. Needs `.far.glb` (E7). |
| `garden.lemonTree` Potted lemon tree | 1x1 · floor | pick:eat water:water | garden / trees | - | Use | `pick`: a lemon shrinks away. |
