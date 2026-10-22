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
  lemons are all picked at once, so its front lemon is one with the rest (no `pick` clip): still four draws.
  Still to do (I-garden-12, a model change, 2026-10-22): "Pick the apples" (`harvest`, `pickHigh`: the hand up to
  about 1.8 m, twice a loop for its 20 minutes) reaches under a crown whose fruit hangs from about 2.5 m (the
  `dots` band from 3.6 m less 0.65 of the crown's 1.65 m depth): only the one front apple is within reach. A garden
  apple (dwarf or semi-dwarf) carries fruit from about 1 m: bring the crown's middle down to about 2.6 m (the lowest
  leaves at about 1.3 m, the tree about 4.3 m), or hang a ring of six to eight apples on low twigs round its front
  half between 1.4 and 1.9 m, so `pickHigh` closes on fruit; the clip and the content can stay. Far
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
| `model.garden.lemonTree` | 1.29 × 1.99 × 1.34 m | 1,736 | 698 | 4 |

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
- **Potted lemon tree** (M-garden-04): trunk and crown 0.38 m lower (crown middle 1.47 m), two metres tall in all;
  the picked lemon stays at 1.62 m, on the crown's front, as `FRUIT.lemon` in `clips/uses.ts` has it.
- The trunks the shade sit leans on are as they were (`furniture.json` `seats` rows unchanged).

## Perches (2026-10-22)

Every tree's entry has `perches`: the places the game's birds (`birds.py`) sit on it. Each is `[x, y, z]` in the
model's game axes (x, height, depth: Blender (−x, z, −y)), in metres, at rest, before the entry's `scale` and the
instance's turn. They are found on the tree as built (`gamekit/perch.py`, `perch_on`, `clump_aims`, `tier_aims`):

- A broadleaf's crown: lines come in from outside towards each clump `crown` returns (out from the trunk ±50°,
  10–55° up, steeper over the top). Where a line meets the leaves, the perch is 3 cm further in, on the outer twigs,
  so a bird's feet are among the leaves and its body shows. A perch is kept only where the leaves face up a little at
  least and nothing hangs over it within 25 cm. The rest are spread round the crown: the highest first, then each the
  farthest from those kept, 35–45 cm apart at least.
- The willow's crown gets lines from above only (lower ones would meet the curtains). Then a few perches go on bare
  limbs: the birch has one, and the willow four under its crown inside the curtain. These are points on a limb's
  top with nothing over it and not inside the leaves (`limb_tops`, `G.perch.bare`).
- The conifers: the lines aim just over each tier's foot from a little above, so they meet the tier near its rim
  where its boughs end. The top (`TOP`) is a song post. The cypress's upright column takes upright leaves too.
- The fruit trees, the Japanese maple and the lemon tree: found on the still part before the fruit and the sprigs are
  built, so no bird sits on an apple that is picked away.

Finding them casts rays at the meshes and draws its own random numbers (each tree its own seed, 111–122), so the
meshes and the far versions are what they were. The rebuild gave the same triangles and sizes as the table above,
and the committed .glb files are unchanged: perches only add data. `cargo check-glb` fails a perch that isn't three
numbers or lies over 1 cm outside the model's box. `--preview` renders `<id>@perches.png` and `<id>@perches.top.png`
with a red ball where each bird would sit.

| Model | Perches |
|---|---|
| `model.tree` | 19 |
| `model.tree.maple` | 24 |
| `model.tree.birch` | 16 (1 on a bare limb) |
| `model.tree.blossom` | 20 |
| `model.tree.fruit` | 16 |
| `model.tree.japaneseMaple` | 18 |
| `model.tree.magnolia` | 18 |
| `model.tree.willow` | 17 (4 on bare limbs) |
| `model.pine` | 17 (its top among them) |
| `model.spruce` | 21 (its top among them) |
| `model.cypress` | 15 (its top among them) |
| `model.garden.lemonTree` | 12 |

The shrubs and hedges in `plants.py` have 4–6 each, found the same way (`perches` there).

## Rebuild

```
cargo art trees            # the .glb files, the pack, check-glb
cargo art trees --preview  # previews/trees/<id>.png, <id>.far.png, <id>@<clip>.png, <id>@perches.png
```
