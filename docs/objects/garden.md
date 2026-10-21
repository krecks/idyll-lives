# Garden plants and trees

Plants and trees keep the shape of the old pipeline (foliage cards that sway in the shader, `vary` for natural
variation, `far` files for distance). A plant has its leaves on the `foliage` finish (and trunks on `bark`), drawn
with the shared finish materials the nature shader hooks, so they sway and tint in the wind; the trunk or pot is a
mesh. Wind sway is the shader's job; clips only show a resident's use.

`families/plants.py` builds the houseplants, flowers, shrubs and edibles (notes: `plants.md`); `families/trees.py` the
trees (notes: `trees.md`), with the shapes and helpers of `plants.py`. The flowers, shrubs, edibles and trees each have
a far version, the same build at low detail (`far_of`). The shrubs dress every lot (`natureDecor`) and the bush the
woods, so they stay cheap. The far woods draw a file's own far version where it has one
(`nature.ts` `forests`), so the woods and the town match.

Colour parts: each houseplant's pot ("Pot"); each mound bed's soil mound ("Bed"), and its petals where they are all
one colour; the geraniums' pot and flowers, the flower box's and raised bed's boxes ("Box"). The meadow has none. Of
the shrubs only the topiary has a pot; the flowering shrubs colour their flowers instead, the others nothing. The
trees have none, but the lemon tree's pot.

The four crops grow (`grow`, `G.stages`, E4: their leaves and their crop as two parts) and have no harvest clip:
harvesting moves them to the picked stage, so the crop goes as the clip plays on, and grows back over the days. The
herb planter, which doesn't grow, has `harvest` for `pickHerbs`.

## G1. Houseplants (10)

- **Family:** `tools/art/blender/families/plants.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
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

**In play** (signed off 2026-10-17):
- Checked in the living room: watered in all four facings (the resident stands in front and pours), talked to
  (monstera), admired (orchid).
- The bonsai pruned in all four facings: its stray twig shrinks away (`trim`, 0.79 s) and is back after the use.
- "Pot" colours the pot of each: the bonsai's tray, the fern's pot on its stand, the orchid's pot on its pedestal.

## G2. Flowers (15)

- **Family:** `tools/art/blender/families/plants.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `flowerbed` Raised flower bed | 1x1 · floor | tend:tend smell:smell | garden / flowers | Box | Static | Static. |
| `garden.tulips` Tulips | 1x1 · floor | water:water smell:smell | garden / flowers | Bed | Static | Static. |
| `garden.daffodils` Daffodils | 1x1 · floor | water:water smell:smell | garden / flowers | Bed, Petals | Static | Static. |
| `garden.daisies` Daisies | 1x1 · floor | water:water smell:smell | garden / flowers | Bed, Petals | Static | Static. |
| `garden.sunflowers` Sunflowers | 1x1 · floor | water:water admire:admire | garden / flowers | Bed, Petals | Static | Static. Admired looking up at the heads (`anim: admire`), not down at the soil. |
| `garden.poppies` Poppies | 1x1 · floor | water:water admire:look | garden / flowers | Bed, Petals | Static | Static. Admired (`look`), not smelled: no scent, no Sweet scent feeling. |
| `garden.marigolds` Marigolds | 1x1 · floor | water:water admire:look | garden / flowers | Bed | Static | Static. Admired (`look`), not smelled: no Sweet scent feeling. |
| `garden.pansies` Pansies | 1x1 · floor | water:water smell:smell | garden / flowers | Bed | Static | Static; each face on a thin stem over the leaves. |
| `garden.lilies` Lilies | 1x1 · floor | water:water smell:smell | garden / flowers | Bed | Static | Static. |
| `garden.lupins` Lupins | 1x1 · floor | water:water smell:smell | garden / flowers | Bed | Static | Static. |
| `garden.forgetMeNots` Forget-me-nots | 1x1 · floor | water:water admire:look | garden / flowers | Bed | Static | Static. Admired (`look`), not smelled: no Sweet scent feeling. |
| `garden.cosmos` Cosmos | 1x1 · floor | water:water smell:smell | garden / flowers | Bed | Static | Static. |
| `garden.meadow` Wildflower meadow | 1x1 · floor | pickFlower:harvest | garden / flowers | — | Use | `pick`: one flower (a white daisy) shrinks away for pickFlower and grows back after. |
| `garden.geraniums` Potted geraniums | 1x1 · floor | water:water smell:smell | garden / flowers | Pot, Flowers | Static | Static. |
| `garden.flowerBox` Flower box | 1x1 · floor | water:water smell:smell | garden / flowers | Box | Static | Static. |

**In play** (signed off 2026-10-17):
- Checked outdoors: watered (the raised bed tended, kneeling at its front), smelled or admired; the meadow's white
  daisy shrinks away for "Pick a wildflower" (`pick`) and is back after, in all four facings.
- "Bed" colours each mound, "Petals" the daffodils', daisies', sunflowers' and poppies' petals, "Box" the raised
  bed's boards and the flower box.
- The geraniums' "Flowers" colours all six clusters (all on the painted material).
- Each pansy face stands on a thin stem over its leaves, at the same size and draw calls.
- Known, not changed: the marigolds' back poms sit a little above the leaves, but read as one mass of flowers.

## G3. Shrubs (11)

- **Family:** `tools/art/blender/families/plants.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bush` Round shrub | 1x1 · floor | trim:prune | garden / shrubs | — | Static | Static. |
| `garden.boxwood` Boxwood ball | 1x1 · floor | trim:prune | garden / shrubs | — | Use | `trim`: the shaggy top sprigs shrink away for trim. |
| `garden.topiary` Topiary cone | 1x1 · floor | trim:prune | garden / shrubs | Pot | Use | `trim`: sprigs reaching out of the cone's sides near the tip shrink away for trim. |
| `garden.hedge` Clipped hedge | 1x1 · floor | trim:prune | garden / shrubs | — | Use | `trim`. |
| `garden.hydrangea` Hydrangea | 1x1 · floor | water:water admire:look | garden / shrubs | Flowers | Static | Static. |
| `garden.roses` Rose bush | 1x1 · floor | smell:smell prune:prune | garden / shrubs | Roses | Use | `cut`: one bloom shrinks away for prune. |
| `garden.azalea` Azalea | 1x1 · floor | water:water trim:prune | garden / shrubs | Flowers | Use | `trim`. |
| `garden.lavender` Lavender | 1x1 · floor | smell:smell cut:prune | garden / shrubs | Flowers | Use | `cut`: a few stems shrink away. |
| `garden.fern` Garden fern | 1x1 · floor | water:water | garden / shrubs | — | Static | Static. |
| `garden.grass` Ornamental grass | 1x1 · floor | — | garden / shrubs | — | Static | Static. |
| `garden.hosta` Hosta | 1x1 · floor | water:water | garden / shrubs | — | Static | Static. |

**In play** (signed off 2026-10-17):
- Checked outdoors in all four facings: trimmed (bush, boxwood, topiary, hedge, azalea), pruned (roses), cut
  (lavender), each clip (`trim`, `cut`) playing its 0.79 s and shutting after the use; watered or admired
  (hydrangea, azalea, fern, hosta), smelled (roses, lavender); the ornamental grass looked at.
- "Flowers" colours every head: all the hydrangea's mopheads, the azalea's flowers and the lavender spikes, far
  versions too. The rose bush's "Roses" colours every rose.
- The topiary's trimmed sprigs reach out of the cone's sides near the tip, so they show going during "Clip the
  topiary".
- Known: from the west (facing 3) the house hides most of the grass at the check spot (camera, not model).

## G4. Edibles (5)

- **Family:** `tools/art/blender/families/plants.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `garden.vegPatch` Vegetable patch | 1x1 · floor | tend:tend harvest:harvest | garden / edibles | Bed | Growth | `grow` (E4): growth stages from seedling to ripe (lettuces and cabbages full); harvest moves it to picked, the crop shrinking away while the resident picks, and it grows back over the days. |
| `garden.tomatoes` Tomato plants | 1x1 · floor | tend:tend harvest:harvest | garden / edibles | Bed | Growth | `grow` (E4): growth stages from seedling to ripe (tomatoes red); harvest moves it to picked, the crop shrinking away while the resident picks, and it grows back over the days. |
| `garden.strawberries` Strawberry bed | 1x1 · floor | tend:tend harvest:harvest | garden / edibles | Bed | Growth | `grow` (E4): growth stages from seedling to ripe (strawberries red); harvest moves it to picked, the crop shrinking away while the resident picks, and it grows back over the days. |
| `garden.herbs` Herb planter | 1x1 · floor | pickHerbs:harvest water:water | garden / edibles | Bed | Use | `harvest`: a few sprigs (the basil) shrink away for pickHerbs and are back after. The resident steps in to it (`STAND_BACK` -0.5, as to a counter: it is 0.46 m deep) and picks standing, hands at the plants (the `pick` clip); watering steps in too. |
| `garden.pumpkins` Pumpkin patch | 1x1 · floor | tend:tend harvest:harvest | garden / edibles | Bed | Growth | `grow` (E4): growth stages from seedling to ripe (pumpkins orange); harvest moves it to picked, the crop shrinking away while the resident picks, and it grows back over the days. |

**In play** (signed off 2026-10-17):
- The four crops are bought new at the planted stage and were tended (kneeling at the bed) in all four facings.
- Grown at speed through their stages: `grow` is held at planted (1.5 s), growing (1.0) and ripe (0). Harvested, the
  clip plays on to picked (0.5), the crop shrinking away while the resident picks and the plants staying.
- "Pick fresh herbs" is a standing reach into the planter (`pick`), the basil shrinking under the hands; other crops
  still kneel. The resident steps in to the waist-high planter (`STAND_BACK` -0.5 in `characters.ts`) for every use,
  watering too, rather than standing a tile off.
- "Bed" colours each bed (the raised bed's and the planter's boards).
- Not watched: growing back from picked over the days.

## G5. Trees (12)

- **Family:** `tools/art/blender/families/trees.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `tree` Oak tree | 1x1 · floor | shade:relax | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). |
| `pine` Pine tree | 1x1 · floor | — | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). |
| `garden.maple` Maple tree | 1x1 · floor | shade:relax | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). |
| `garden.birch` Silver birch | 1x1 · floor | shade:relax | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). |
| `garden.cherry` Cherry blossom tree | 1x1 · floor | shade:relax admire:admire | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). |
| `garden.apple` Apple tree | 1x1 · floor | pick:eat shade:relax | garden / trees | — | Use | `pick`: an apple at the front shrinks away for pick. |
| `garden.japaneseMaple` Japanese maple | 1x1 · floor | shade:relax shape:prune | garden / trees | — | Use | `trim`: the top sprigs shrink away for shape. |
| `garden.magnolia` Magnolia | 1x1 · floor | shade:relax smell:admire | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). Its blossom use reads "Admire the magnolia" (looking up). |
| `garden.willow` Weeping willow | 1x1 · floor | shade:relax | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). |
| `garden.spruce` Blue spruce | 1x1 · floor | — | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). |
| `garden.cypress` Italian cypress | 1x1 · floor | — | garden / trees | — | Static | Static; the canopy sways in the shader. Far version (E7). |
| `garden.lemonTree` Potted lemon tree | 1x1 · floor | pick:eat water:water | garden / trees | Pot | Use | `pick`: a lemon at the front shrinks away. |

**In play** (signed off 2026-10-17):
- Checked outdoors (the lemon tree also close up): relaxed under in the shade (standing by the trunk), admired
  (cherry), smelled (magnolia), watered (lemon tree).
- The apple and the lemon tree each lose their front fruit for "Pick" (`pick`), the Japanese maple its top sprigs
  for "Shape the maple" (`trim`), in all four facings, shutting after the use.
- Known, not changed: two tiles from the house the pine's and spruce's crowns (6 m and 5.6 m across on a one-tile
  footprint) reach through the house's wall. The game lets a tree stand that close, and no model can be
  narrower without losing its shape.
