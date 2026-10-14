# Plants — notes

`plants.py` builds the G1 houseplants into `web/public/assets/models/plants/` and their entries in
`packs/plants/manifest.json`: the potted plant (`model.plant`), the monstera, fiddle-leaf fig, snake plant, cactus
trio, succulent bowl, fern on a stand, parlour palm, orchid and bonsai (`model.garden.*`). They replace the
primitive-part placeholders of `furniture.py` and `nature.py`, which stay as the fallback. Built 2026-10-14; they
sit in Validate › Houseplants until checked in play. The G2 flowers followed the same day (below, in Validate ›
Flowers); the shrubs and edibles (G3, G4) come into this family next.

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

## Rebuild

```
python3 -B tools/art/blender/build.py plants            # the .glb files, the pack, check_glb
python3 -B tools/art/blender/build.py plants --preview  # previews/<id>.png, <id>@<clip>.png
```
