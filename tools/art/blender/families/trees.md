# Trees — notes

`trees.py` builds the G5 trees into `web/public/assets/models/trees/` and their entries in
`packs/trees/manifest.json`: the oak (`model.tree`), maple, silver birch, cherry blossom, apple, Japanese maple,
magnolia and weeping willow (`model.tree.*`), the pine, blue spruce and Italian cypress (`model.pine`, `model.spruce`,
`model.cypress`) and the potted lemon tree (`model.garden.lemonTree`). Built 2026-10-14; they sat in Validate › Trees until checked in play. The
shapes and helpers (`lump`, `tube`, `dots`, `sprigs`, `far_of`) come from `plants.py`.

## How a tree is made

- **Crowns** (`crown`): leafy lumps over an ellipsoid, lighter towards the top, round a dark core, on bark limbs
  (`limbs`) from a flared trunk (`trunk`); conifers are stacked cones (`conifer`), the cypress a column of lumps.
- **Finishes:** leaves on `foliage`, trunks and limbs on `bark` (the nature shader sways and tints both), the
  birch's white bark and fruit plain.
- **Trunk on the origin:** crowns are built round the trunk, so `settle` moves a lopsided crown's bounds only by a
  few centimetres.
- **Clips:** `pick` shrinks one apple at the front for `pick` and `harvest`, 1.6 s into it (`PICK_DELAY`: the picker's
  hand closes on it then, `pickApple*`/`pickLemon*` in `characters/clips/uses.ts`, fitted to where it hangs; move
  the fruit and those clips' `FRUIT` together). The apple hangs within reach, 1.84 m up and 0.7 m out under the
  crown on a stalk into it (2026-10-21; it was 2.95 m up, 1.95 m out). `trim` the Japanese maple's stray sprigs
  for `shape`. Each is one more draw call (the apple and lemon trees come to four; `check-glb` warns).
- **Ripeness (G-68, 2026-10-21):** the fruit trees `grows` in content: their fruit over the crown is a moving part
  (`_fruiting`) with the growth clip `ripen` (`motions.growth`, `G.stages`): in fruit at rest, gone (shrunk into the
  crown's middle) when picked and while it fruits again; the apple's front apple follows it too. The lemon tree's
  lemons are all picked at once, so its front lemon is one with the rest (no `pick` clip): still four draws. Far
  versions: all one still part, in fruit.

## Far versions and the woods

Each tree has a far version (`far_of`): every third clump of each crown, round a smooth core spanning all the full
crown's clumps (their random numbers drawn as in the full build, so the core is measured, not guessed); conifers
pointed cones of eight sides. It keeps the tree's size within 5 % at 18–40 % of its triangles. The far woods (`nature.ts` `forests`) used to draw the `*.far` old part keys whatever the model; they
now draw a file's own far version, so the woods match the town, and those keys are gone (2026-10-18, with the
primitive parts). The town overview draws them too.

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| `model.tree` | 5.49 × 7.47 × 5.23 m | 1,488 | 334 | 2 |
| `model.tree.maple` | 4.98 × 7.31 × 5.34 m | 1,952 | 316 | 2 |
| `model.tree.birch` | 3.45 × 8.18 × 3.29 m | 1,196 | 322 | 3 |
| `model.tree.blossom` | 5.25 × 5.45 × 5.14 m | 1,360 | 286 | 2 |
| `model.tree.fruit` | 4.47 × 5.57 × 4.38 m | 1,760 | 412 | 4 |
| `model.tree.japaneseMaple` | 3.72 × 4.15 × 3.83 m | 2,248 | 538 | 3 |
| `model.tree.magnolia` | 4.10 × 5.08 × 3.76 m | 3,136 | 592 | 3 |
| `model.tree.willow` | 4.94 × 6.10 × 5.04 m | 4,944 | 732 | 2 |
| `model.pine` | 6.03 × 9.20 × 6.05 m | 328 | 122 | 2 |
| `model.spruce` | 5.19 × 10.50 × 5.15 m | 440 | 164 | 2 |
| `model.cypress` | 1.01 × 7.95 × 1.02 m | 1,636 | 450 | 2 |
| `model.garden.lemonTree` | 1.29 × 2.37 × 1.34 m | 1,736 | 698 | 4 |

`check-glb`: 12 models, 0 failed; its one warning is the lemon tree's 4 draw calls. The sizes were held to the
old parts' bounds as they were built (those bounds counted each turned blob's or cone's whole box: the
spruce's read 7.8 m for cones 5.4 m across); the oak's and maple's crowns were drawn in a little to match (`spread`
0.75 to 0.71 and 0.78 to 0.74, 2026-10-15). Khronos validator: 0 errors, 0 warnings on all 24 files.

## Walking past them (G-38, 2026-10-21)

A one-tile tree must let residents walk past and work at it from the next tile: what reaches past 0.75 m from the
trunk stands above head height (the tallest resident is 1.77 m). Measured on the exported files, the lowest point
farther than 0.85 m from the trunk: pine 2.19 m, spruce 2.15, Japanese maple 2.24 (its narrow first layer reaches
0.8 m at 1.4-1.5 m, inside the reach of someone shaping it from the next tile), magnolia 2.48, apple 2.45 (but its
picked apple at 1.84, 0.7 m out), cherry 2.77, maple 3.19, oak 3.90, birch 4.66. The willow's curtains come down to
0.45 m by design (sitting inside the curtain is the willow), thin streamers with gaps, not solid wedges.

- **Pine:** its lowest tier's foot at 2.35 m (was 1.4), tilted 3 degrees at most so its rim stays over 2.1 m.
- **Blue spruce:** limbed up, its skirt from 2.2 m (was 0.35), 5.0 m across (was 5.4), on a 3 m trunk.
- **Japanese maple:** a short trunk and six slender limbs, a narrow first layer at 1.55 m inside its tile, the broad
  tiers at 2.7, 3.25 and 3.65 m (were 2.0, 2.7, 3.2).
- **Weeping willow:** the 26 solid cones and 8 inner ones are 68 `streamer`s, thin leafy ribbons (a lens across, four
  points a row, smooth) out of the crown's edge and straight down to 0.45-0.8 m (the outer row) and 0.9-1.5 m (the
  inner). 4,944 triangles: inside the 6,000 for a big thing placed often, and it is only bought (the woods and the
  lots' dressing have none).
- **Italian cypress:** a pencil about a metre across (was 2.1), the same 7.95 m.
- **Maple:** not the oak's crown again: a Norway maple's dense, even dome of 26 small clumps round a large core, in a
  cooler, deeper green, on a straight trunk with a leader.
- **Magnolia:** its blooms are upright goblets (`GOBLET`, revolved), three over each clump of the upper crown, each
  set where a line out from its clump's middle leaves the crown (a ray cast), so none stands off it. `crown` returns
  its clumps for this.
- The trunks the shade sit leans on are as they were (`furniture.json` `seats` rows unchanged).

## Rebuild

```
cargo art trees            # the .glb files, the pack, check-glb
cargo art trees --preview  # previews/trees/<id>.png, <id>.far.png, <id>@<clip>.png
```
