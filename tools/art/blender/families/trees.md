# Trees — notes

`trees.py` builds the G5 trees into `web/public/assets/models/trees/` and their entries in
`packs/trees/manifest.json`: the oak (`model.tree`), maple, silver birch, cherry blossom, apple, Japanese maple,
magnolia and weeping willow (`model.tree.*`), the pine, blue spruce and Italian cypress (`model.pine`, `model.spruce`,
`model.cypress`) and the potted lemon tree (`model.garden.lemonTree`). Built 2026-10-14; they sat in Validate › Trees until checked in play.
Rebuilt with real skeletons 2026-10-22 (the oak first, by hand, then the rest in its way). The leafy lumps and small
helpers (`lump`, `tube`, `pot`, `cupped`, `sprigs`, `far_of`) come from `plants.py`; the tree builders are in
`trees.py` (plants.py is shared, so they stay out of it).

## How a tree is made

- **Trunks** (`_trunk`): one swept, tapering piece through a few points (a lean, a wave), its bark wobbled by noise
  (`G.jitter`, not on a far version), flared on root buttresses (`_roots`) that run out to just above the ground.
- **Broadleaves** (`_branches`): bough tips spread evenly over the crown's ellipsoid (a golden-angle spiral), grouped
  by bearing onto a few limbs; each limb runs from the trunk to part way to its tips' middle, bowed up and wandering
  (`_bend`), and forks into a curving bough to each tip; tips over the middle go on a leader. At each tip a leaf
  mass (`_mass`): a clump on the tip and two round it, a little below and out, lighter towards the top, with gaps
  between the masses where the limbs show; a few dark lumps close the crown's heart (`_hearts`). The oak is built
  the same way by hand. Its own shape per tree: the birch's slender white limbs up the stem and hanging masses,
  the cherry's wide low limbs under flat blossom clouds, the apple's crooked trunk and gnarled limbs, the Japanese
  maple's six S-curved stems under flat layered clouds, the magnolia's upswept stems, the willow's five limbs
  rising and arching over with masses along their crests and curtains (`streamer`) hanging from them.
- **Conifers** (`conifer`): whorls of boughs round the trunk, a tier at a time, each bough a spray of needles (a
  gull-wing profile, `_pad_profile`, swept out level from high in the tier and drooping to its foot at the rim, wide
  in the middle and narrowing to its tip, the spruce's tips turned up), a dark cone inside closing the whorl; the
  lowest tiers' bare boughs show from below. The cypress is a lofted flame of foliage covered in upright sprays.
- **Fruit and blooms** sit on the leaves as built: apples and lemons where lines in from round the crown meet the
  leaves (`_hang`, a ray cast), the magnolia's goblets where lines out of each mass's tip clump leave it.
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

Each tree has a far version (`far_of`): the same build at low detail, every random number drawn in the same order:
no boughs, one lump a mass spanning its clumps (so the crown keeps its size), the trunk and limbs on fewer sides, the
conifers' sprays on three-point paths with a wedge profile, the willow's outer curtain only, half the magnolia's
blooms. It keeps the tree's size within 5 % at 16–30 % of its triangles, and its foot where the model's is. The far woods (`nature.ts` `forests`) used to draw the `*.far` old part keys whatever the model; they
now draw a file's own far version, so the woods match the town, and those keys are gone (2026-10-18, with the
primitive parts). The town overview draws them too.

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| `model.tree` | 5.27 × 7.35 × 5.04 m | 4,410 | 740 | 2 |
| `model.tree.maple` | 5.09 × 7.15 × 5.38 m | 4,376 | 832 | 2 |
| `model.tree.birch` | 3.35 × 8.27 × 3.52 m | 4,316 | 692 | 3 |
| `model.tree.blossom` | 5.09 × 5.37 × 5.25 m | 3,880 | 734 | 2 |
| `model.tree.fruit` | 4.33 × 5.39 × 4.41 m | 4,412 | 914 | 4 |
| `model.tree.japaneseMaple` | 3.75 × 4.10 × 3.90 m | 3,786 | 692 | 3 |
| `model.tree.magnolia` | 4.20 × 5.25 × 3.83 m | 4,346 | 936 | 3 |
| `model.tree.willow` | 5.06 × 5.80 × 4.96 m | 4,196 | 1,214 | 2 |
| `model.pine` | 5.94 × 9.21 × 6.17 m | 3,828 | 892 | 2 |
| `model.spruce` | 5.23 × 10.50 × 5.40 m | 4,174 | 1,048 | 2 |
| `model.cypress` | 1.03 × 7.95 × 1.05 m | 2,464 | 468 | 2 |
| `model.garden.lemonTree` | 1.25 × 2.00 × 1.27 m | 3,896 | 1,168 | 4 |

Each at most 4,500 triangles (trees stand by the thousand in the woods). `check-glb`: 12 models, 0 failed; its
warnings are the apple's and the lemon tree's 4 draw calls (as before). The sizes were held to the
old parts' bounds as they were built (those bounds counted each turned blob's or cone's whole box: the
spruce's read 7.8 m for cones 5.4 m across); the oak's and maple's crowns were drawn in a little to match (`spread`
0.75 to 0.71 and 0.78 to 0.74, 2026-10-15). Khronos validator: 0 errors, 0 warnings on all 24 files.

## Walking past them (G-38, 2026-10-21)

A one-tile tree must let residents walk past and work at it from the next tile: what reaches past 0.75 m from the
trunk stands above head height (the tallest resident is 1.77 m). Measured on the exported files, the lowest point
farther than 0.85 m from the trunk (the rebuild, 2026-10-22): pine 2.51 m, spruce 2.22, Japanese maple 2.35 (its narrow first layer stays
inside 0.85 m at about 1.4-1.6 m, inside the reach of someone shaping it from the next tile), magnolia 2.41, apple 2.47
(but its picked apple at 1.84, 0.7 m out), cherry 2.87, oak 3.31, maple 3.42, birch 4.01. The root buttresses reach
0.4-0.75 m out, a few centimetres high. The willow's curtains come down to
0.45 m by design (sitting inside the curtain is the willow), thin streamers with gaps, not solid wedges.

- **Pine:** its lowest tier's foot at 2.35 m (was 1.4); its sprays' tips droop to about 2.5 m.
- **Blue spruce:** limbed up, its skirt from 2.2 m (was 0.35), 5.0 m across (was 5.4), on a 3 m trunk.
- **Japanese maple:** a short trunk and six slender limbs, a narrow first layer at 1.55 m inside its tile, the broad
  tiers at 2.7, 3.25 and 3.65 m (were 2.0, 2.7, 3.2).
- **Weeping willow:** the 26 solid cones and 8 inner ones are `streamer`s, thin leafy ribbons out of the crown's edge
  and straight down to 0.45-0.8 m (the outer row) and 0.9-1.5 m (the inner): 68 lenses of four points a row (4,944
  triangles), since 2026-10-22 60 wedges of three points a row, five rows (4,196 in all, under the trees' 4,500).
- **Italian cypress:** a pencil about a metre across (was 2.1), the same 7.95 m.
- **Maple:** not the oak's crown again: a Norway maple's dense, even dome (since 2026-10-22 eighteen leaf masses on
  six steep limbs round a dark heart), in a cooler, deeper green, on a straight trunk with a leader.
- **Magnolia:** its blooms are upright goblets (`GOBLET`, revolved), two over each mass of the upper crown and one on
  the side of each lower one, each set where a line out from the mass's tip clump leaves the crown (a ray cast), so
  none stands off it. `_mass` lists the clumps for this.
- **Potted lemon tree** (M-garden-04): trunk and crown 0.38 m lower (crown middle 1.47 m), two metres tall in all;
  the picked lemon stays at 1.62 m, on the crown's front, as `FRUIT.lemon` in `clips/uses.ts` has it.
- The trunks the shade sit leans on are as thick as they were at a sitter's back (`furniture.json` `seats` rows
  unchanged): the rebuilt trunks' radius at 0.3-0.6 m is 0.1-0.33 m, each inside its `fwd`, with root buttresses
  at the ground.

## Perches (2026-10-22)

Every tree's entry has `perches`: the places the game's birds (`birds.py`) sit on it. Each is `[x, y, z]` in the
model's game axes (x, height, depth: Blender (−x, z, −y)), in metres, at rest, before the entry's `scale` and the
instance's turn. They are found on the tree as built (`gamekit/perch.py` `find`, `clump_aims`, `tier_aims`):

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

Beside them each entry says what they are on, `habitat` (the `Model`'s `habitat=`), so the game's birds needn't guess
it from the key: `"conifer"` for the pine, spruce and cypress, `"tree"` for the rest (the fruit trees and the potted
lemon tree too). The shrubs' and hedges' are `"bush"` and `"hedge"` (plants.md). `cargo check-glb` fails any other
value, and a `habitat` without `perches`. The manifest writes each perch on one line.

| Model | Perches |
|---|---|
| `model.tree` | 24 |
| `model.tree.maple` | 24 |
| `model.tree.birch` | 20 (2 on bare limbs) |
| `model.tree.blossom` | 22 |
| `model.tree.fruit` | 20 |
| `model.tree.japaneseMaple` | 18 |
| `model.tree.magnolia` | 20 |
| `model.tree.willow` | 20 (4 on bare limbs) |
| `model.pine` | 18 (its top among them) |
| `model.spruce` | 21 (its top among them) |
| `model.cypress` | 15 (its top among them) |
| `model.garden.lemonTree` | 12 |

(After the rebuild, 2026-10-22: the clumps are the masses' clumps (`_mass`), the willow's limb perches on its arching
limbs (`limb_points`), the conifers' aims at each whorl's foot.)

The shrubs and hedges in `plants.py` have 4–6 each, found the same way (`perches` there).

## Rebuild

```
cargo art trees            # the .glb files, the pack, check-glb
cargo art trees --preview  # previews/trees/<id>.png, <id>.far.png, <id>@<clip>.png, <id>@perches.png
```
