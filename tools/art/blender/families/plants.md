# Plants — notes

`plants.py` builds the G1 houseplants into `web/public/assets/models/plants/` and their entries in
`packs/plants/manifest.json`: the potted plant (`model.plant`), the monstera, fiddle-leaf fig, snake plant, cactus
trio, succulent bowl, fern on a stand, parlour palm, orchid and bonsai (`model.garden.*`). They replace the
primitive-part placeholders of `furniture.py` and `nature.py`, which stay as the fallback. Built 2026-10-14; they
sit in Validate › Houseplants until checked in play. The G2 flowers followed the same day (below, in Validate ›
Flowers), then the G3 shrubs (Validate › Shrubs) and the G4 edibles (Validate › Edibles).

## How a plant is made

- **Leaves** (`leaf`): a closed, thin blade, its cross-section a flattened ring lifted into a V (`fold`), along a
  midrib that bends down evenly by its tip (`droop`, keeping its length), with a width profile (`PROFILES`: oval,
  lance, heart, strap, petal). `notch` narrows every nth row: the monstera's cuts, the fern's and palm's leaflets.
  Closed because materials are one-sided.
- **Stems and trunks** (`tube`): a tube swept through points, tapering; `arc` bows a stem up.
- **Pots** (`pot`): a surface of revolution with an optional rim, a hollow top and a soil disc. The pot is `body`,
  the part the player colours ("Pot"); each model has its own colours (`palette`), the finishes stay the family's.
- **Finishes:** leaves and stems on `foliage`, trunks on `bark` (the fig), stands on `wood`, cacti, succulents and
  flowers on `satin` (still: no sway), pots `satin`, soil and sand `matte`.

## Wind

The nature shader (`render/babylon/nature.ts`, `installNature`) hooks the shared `material.finish.foliage` and
`bark` materials, which a file's parts mapped to those finishes are drawn with, so the files sway as the
placeholders do: with the wind on, two frames 1.5 s apart differ in about 9,500 pixels of the fern and none of the
cacti (gallery, WebGPU). Heights are from the floor (the still part's origin), so the bend grows with the plant.

## The bonsai's sprig

The one moving part: a twig and a tuft of leaves sticking out of the top pad, all on `foliage` (one draw call),
its origin where it leaves the pad. `trim` squashes it to nothing in 0.8 s; `motions.uses.prune` plays it for the
whole prune (the game brings it back after). It is the model's fourth draw call (`check_glb` warns: aim for 3).

| Model | Size (w × h × d) | Triangles | Draws |
|---|---|---|---|
| `model.plant` | 0.77 × 0.83 × 0.79 m | 2,588 | 2 |
| `model.garden.monstera` | 1.33 × 0.92 × 1.34 m | 2,840 | 2 |
| `model.garden.fiddleFig` | 0.53 × 1.63 × 0.53 m | 1,924 | 3 |
| `model.garden.snakePlant` | 0.34 × 0.91 × 0.32 m | 1,220 | 2 |
| `model.garden.cacti` | 0.57 × 0.52 × 0.47 m | 1,860 | 1 |
| `model.garden.succulents` | 0.62 × 0.17 × 0.62 m | 3,300 | 1 |
| `model.garden.bostonFern` | 0.88 × 0.98 × 0.94 m | 2,796 | 3 |
| `model.garden.palm` | 1.07 × 0.87 × 0.98 m | 1,644 | 2 |
| `model.garden.orchid` | 0.43 × 1.20 × 0.38 m | 1,632 | 3 |
| `model.garden.bonsai` | 0.62 × 0.97 × 0.41 m | 1,260 | 4 |

`check_glb.py`: 10 models, 0 failed; warnings: the bonsai's four draw calls, the palm 11 % shallower and the
succulents 15 % lower than their placeholders' bounds (which count each tilted part's whole box). Khronos
validator: 0 errors, 0 warnings each.

## The flowers (G2)

`model.flowerbed` and `model.garden.tulips`, `daffodils`, `daisies`, `sunflowers`, `poppies`, `marigolds`,
`pansies`, `lilies`, `lupins`, `forgetMeNots`, `cosmos`, `meadow`, `geraniums`, `flowerBox`, over the placeholders
of `nature.py`. Twelve stand on a low soil mound (`mound`, material `bed`), the part the player colours ("Bed");
where a bed's petals are one colour they are a second part ("Petals"). The geraniums colour their pot and flowers,
the flower box its box, the raised bed its boards; the meadow, all grass and mixed flowers, has none.

- **Flower faces** (`star`): a thin star of petals, cupped, leaning towards a heading; a low `dome` for the middle.
  Tulips, poppies and lilies are turned cups (`cupped`). Marigolds, forget-me-nots and lupins are leafy `lump`s and
  cones on `foliage` in their flower colours, as the placeholders draw them; other flowers are `satin` (still).
- **Leafy clumps** (`clumps`, `lump`): a smooth sphere pushed about by noise, flattened.
- **The meadow's picked flower**: a big white daisy at the front, its own part (stem and face on `satin`: one draw
  call), `pick` squashes it to nothing in 0.8 s for `pickFlower`; the game brings it back after.

The placeholders' mounds sink 6 cm into the ground, so their bounds read a little taller than the beds stand; the
flowers here are set so each model is within 10 % of them, but for the geraniums' width (11 % under).

| Model | Size (w × h × d) | Triangles | Draws |
|---|---|---|---|
| `model.flowerbed` | 0.94 × 0.48 × 0.94 m | 3,044 | 3 |
| `model.garden.tulips` | 0.84 × 0.62 × 0.84 m | 1,672 | 2 |
| `model.garden.daffodils` | 0.84 × 0.49 × 0.84 m | 2,530 | 2 |
| `model.garden.daisies` | 0.84 × 0.38 × 0.84 m | 3,452 | 2 |
| `model.garden.sunflowers` | 0.84 × 2.11 × 0.91 m | 1,344 | 2 |
| `model.garden.poppies` | 0.84 × 0.66 × 0.84 m | 2,152 | 2 |
| `model.garden.marigolds` | 0.88 × 0.34 × 0.84 m | 2,660 | 2 |
| `model.garden.pansies` | 0.84 × 0.26 × 0.84 m | 1,820 | 2 |
| `model.garden.lilies` | 0.84 × 0.94 × 0.87 m | 1,444 | 2 |
| `model.garden.lupins` | 0.84 × 0.99 × 0.84 m | 1,120 | 2 |
| `model.garden.forgetMeNots` | 0.87 × 0.28 × 0.84 m | 2,460 | 2 |
| `model.garden.cosmos` | 0.89 × 0.88 × 0.84 m | 2,146 | 2 |
| `model.garden.meadow` | 0.81 × 0.52 × 0.92 m | 2,548 | 3 |
| `model.garden.geraniums` | 0.42 × 0.65 × 0.50 m | 1,648 | 2 |
| `model.garden.flowerBox` | 0.95 × 0.73 × 0.50 m | 2,912 | 2 |

`check_glb.py`: 15 models, 0 failed (the geraniums' width warning). Khronos validator: 0 errors, 0 warnings each.

## Far versions

Every outdoor model (G2, G3) has a far version, `<id>.far.glb`, drawn in the town overview and the far woods: the
same build function run with `LOW` set (`far_of`, now gamekit's `G.far_of`, `gamekit/far.py`), so every shape is the same but coarser (leaves with half the
rows, three-sided stems open at their ends, lumps and cups with fewer segments, smaller stars) and its moving part
joined in, still; boxes lose their bevels, sprigs keep one leaf, flower faces three petals and no middle, cups
three sides, mounds twelve. Where it leaves things out it still draws every random number the full build draws, so
what's left stands where it did. It keeps the model's size (within 5 %); a decimated copy (`far=0.25`) dipped below
the floor and lost thin parts. Every far version comes to 30–49 % of its model's triangles.

## The shrubs (G3)

`model.bush` and `model.garden.boxwood`, `topiaryCone`, `hedge`, `hydrangea`, `rosebush`, `azalea`, `lavender`,
`fern`, `grass`, `hosta`. The bush, hydrangea, roses, lavender and boxwood also dress every lot's border, and the
bush stands in the woods (`nature.ts` `natureDecor`, thin instances), so they cost what their placeholders do.
The lot dressing's other two placeholders, `model.bush.small` (also in the woods) and `model.bush.flowering`, and
the streets' and hedged lots' `model.hedge` are built here too (`bushSmall`, `bushFlowering`, `streetHedge`: the
shrub, the shrub with pink blossom, the garden hedge's block without its sprigs), each one draw call on `foliage`.

- **Shrubs** (`shrub`): a dark core lump and clumps round it, lighter on top; flowers (`dots`) spread over the upper
  part of an ellipsoid round the crown.
- **Clips:** a trim (`trim`, for `trim`) shrinks a cluster of shaggy sprigs (`sprigs`) on the boxwood, the topiary's
  tip, the hedge and the azalea; a prune (`cut`, for `prune`) one rose at the front; a cut (`cut`, for `cut`) the
  three lavender stems nearest the front. Each part is one draw call on its own finish.
- **Colour parts:** only the topiary has a pot ("Pot"); the hydrangea, azalea and lavender colour their flowers, the
  roses their roses; the rest none.

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| `model.bush` | 1.04 × 0.91 × 1.10 m | 700 | 252 | 1 |
| `model.garden.boxwood` | 0.70 × 0.75 × 0.72 m | 604 | 196 | 2 |
| `model.garden.topiaryCone` | 0.64 × 1.80 × 0.66 m | 728 | 338 | 3 |
| `model.garden.hedge` | 1.55 × 1.17 × 0.92 m | 660 | 204 | 2 |
| `model.garden.hydrangea` | 1.20 × 0.96 × 1.18 m | 1,404 | 516 | 1 |
| `model.garden.rosebush` | 0.93 × 0.86 × 0.95 m | 1,394 | 636 | 3 |
| `model.garden.azalea` | 1.24 × 1.01 × 1.24 m | 2,260 | 828 | 2 |
| `model.garden.lavender` | 0.61 × 0.87 × 0.63 m | 872 | 264 | 3 |
| `model.garden.fern` | 0.79 × 0.48 × 0.93 m | 1,288 | 576 | 1 |
| `model.garden.grass` | 1.22 × 1.21 × 1.28 m | 1,120 | 480 | 2 |
| `model.garden.hosta` | 0.62 × 0.84 × 0.68 m | 676 | 234 | 2 |

`check_glb.py`: the family's 36 models, 0 failed. Warnings: the grass 13 % wider than its placeholder's bounds.
Khronos validator: 0 errors, 0 warnings on every new file.

## The edibles (G4)

`model.garden.vegPatch`, `tomatoes`, `strawberries`, `herbs`, `pumpkins`. The four crops (content `grows`) are three
parts each: the still bed, the leaves (`plants`) and the crop (`crop`), in one growth clip, `grow`
(`motions.growth`, `G.stages`): ripe at rest; picked, the crop gone; growing, the leaves 0.8 high and sunk a tenth
of their height into the soil; planted, half high and sunk a third (`SINK`), so what shows is each plant's top, small,
where the plant stands (scaled about the bed's middle, as first built, the seedlings drew together there). The
vegetable patch's plants are its crop: picked, they are under the soil and the bed is bare (`harvested`); its
lettuces, the lowest, show from growing on. There is no harvest clip: harvesting moves a crop to its
picked stage, and the game plays the clip on from ripe, so the crop goes while the resident picks (seen in play:
the tomatoes picked autonomously, stage 2 to 3, the plants left without fruit). The herb planter doesn't grow; its
basil shrinks away for `pickHerbs` (`harvest`). The bed is the colour part ("Bed"): the boards of the raised bed and
the planter (plain, to stay within three draw calls), the soil mound, straw or patch of the others.

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| `model.garden.vegPatch` | 0.92 × 0.40 × 0.92 m | 1,590 | 540 | 3 |
| `model.garden.tomatoes` | 0.80 × 1.19 × 0.80 m | 2,380 | 924 | 3 |
| `model.garden.strawberries` | 0.88 × 0.21 × 0.88 m | 1,456 | 692 | 3 |
| `model.garden.herbs` | 0.84 × 1.01 × 0.46 m | 1,294 | 444 | 3 |
| `model.garden.pumpkins` | 1.11 × 0.27 × 0.95 m | 1,174 | 542 | 3 |

`check_glb.py`: 5 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings. Each stage checked in
play from a state with all four crops at each (picked, planted, growing, ripe).

## Rebuild

```
python3 -B tools/art/blender/build.py plants            # the .glb files, the pack, check_glb
python3 -B tools/art/blender/build.py plants --preview  # previews/<id>.png, <id>@<clip>.png
```
