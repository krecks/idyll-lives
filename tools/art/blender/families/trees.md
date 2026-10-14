# Trees — notes

`trees.py` builds the G5 trees into `web/public/assets/models/trees/` and their entries in
`packs/trees/manifest.json`: the oak (`model.tree`), maple, silver birch, cherry blossom, apple, Japanese maple,
magnolia and weeping willow (`model.tree.*`), the pine, blue spruce and Italian cypress (`model.pine`, `model.spruce`,
`model.cypress`) and the potted lemon tree (`model.garden.lemonTree`). They replace the primitive-part placeholders of
`nature.py`, which stay as the fallback. Built 2026-10-14; they sit in Validate › Trees until checked in play. The
shapes and helpers (`lump`, `tube`, `dots`, `sprigs`, `far_of`) come from `plants.py`.

## How a tree is made

- **Crowns** (`crown`): leafy lumps over an ellipsoid, lighter towards the top, round a dark core, on bark limbs
  (`limbs`) from a flared trunk (`trunk`); conifers are stacked cones (`conifer`), the cypress a column of lumps.
- **Finishes:** leaves on `foliage`, trunks and limbs on `bark` (the nature shader sways and tints both), the
  birch's white bark and fruit plain.
- **Trunk on the origin:** crowns are built round the trunk, so `settle` moves a lopsided crown's bounds only by a
  few centimetres.
- **Clips:** `pick` shrinks one apple (or lemon) at the front for `pick`; `trim` the Japanese maple's stray sprigs
  for `shape`. Each is one more draw call (the apple and lemon trees come to four; `check_glb` warns).

## Far versions and the woods

Each tree has a far version (`far_of`): every third clump of each crown, round a smooth core spanning all the full
crown's clumps (their random numbers drawn as in the full build, so the core is measured, not guessed); conifers
pointed cones of eight sides. It keeps the tree's size within 5 % at 18–40 % of its triangles. The far woods (`nature.ts` `forests`) used to draw the `*.far` placeholder keys whatever the model; they
now draw a file's own far version, so the woods match the town, and those keys are gone from `nature.py` (a tree
whose file fails to load falls back to its placeholder at the low level of detail). The town overview draws them too.

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| `model.tree` | 5.49 × 7.47 × 5.23 m | 1,488 | 334 | 2 |
| `model.tree.maple` | 4.85 × 7.62 × 4.97 m | 1,136 | 208 | 2 |
| `model.tree.birch` | 3.45 × 8.18 × 3.29 m | 1,196 | 322 | 3 |
| `model.tree.blossom` | 5.25 × 5.45 × 5.14 m | 1,360 | 286 | 2 |
| `model.tree.fruit` | 4.47 × 5.57 × 4.38 m | 1,760 | 412 | 4 |
| `model.tree.japaneseMaple` | 3.89 × 3.72 × 4.02 m | 1,752 | 424 | 3 |
| `model.tree.magnolia` | 4.10 × 4.95 × 3.81 m | 1,864 | 436 | 3 |
| `model.tree.willow` | 4.48 × 6.10 × 4.66 m | 1,544 | 340 | 2 |
| `model.pine` | 6.03 × 8.85 × 6.03 m | 328 | 122 | 2 |
| `model.spruce` | 5.60 × 10.20 × 5.57 m | 416 | 148 | 2 |
| `model.cypress` | 2.12 × 7.95 × 2.05 m | 1,496 | 370 | 2 |
| `model.garden.lemonTree` | 1.29 × 2.37 × 1.34 m | 1,736 | 698 | 4 |

`check_glb.py`: 12 models, 0 failed. Warnings: four draw calls (apple, lemon tree); sizes under the placeholders'
bounds for the oak, maple, pine, spruce and cypress (those bounds count each turned blob's or cone's whole box: the
spruce's read 7.8 m for cones 5.4 m across). Khronos validator: 0 errors, 0 warnings on all 24 files.

## Rebuild

```
python3 -B tools/art/blender/build.py trees            # the .glb files, the pack, check_glb
python3 -B tools/art/blender/build.py trees --preview  # previews/<id>.png, <id>.far.png, <id>@<clip>.png
```
