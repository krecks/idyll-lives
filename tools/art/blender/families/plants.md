# Plants — notes

`plants.py` builds the G1 houseplants into `web/public/assets/models/plants/` and their entries in
`packs/plants/manifest.json`: the potted plant (`model.plant`), the monstera, fiddle-leaf fig, snake plant, cactus
trio, succulent bowl, fern on a stand, parlour palm, orchid and bonsai (`model.garden.*`). Built 2026-10-14; they
sat in Validate › Houseplants until checked in play. The G2 flowers followed the same day (below, in Validate ›
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
old parts do: with the wind on, two frames 1.5 s apart differ in about 9,500 pixels of the fern and none of the
cacti (gallery, WebGPU). Heights are from the floor (the still part's origin), so the bend grows with the plant.

## The bonsai's sprig

The one moving part: a twig and a tuft of leaves sticking out of the top pad, all on `foliage` (one draw call),
its origin where it leaves the pad. `trim` squashes it to nothing in 0.8 s; `motions.uses.prune` plays it for the
whole prune (the game brings it back after). It is the model's fourth draw call (`check-glb` warns: aim for 3).

| Model | Size (w × h × d) | Triangles | Draws |
|---|---|---|---|
| `model.plant` | 0.77 × 0.83 × 0.79 m | 1,884 | 2 |
| `model.garden.monstera` | 0.78 × 0.91 × 0.79 m | 1,576 | 2 |
| `model.garden.fiddleFig` | 0.53 × 1.63 × 0.53 m | 2,080 | 3 |
| `model.garden.snakePlant` | 0.34 × 0.91 × 0.32 m | 1,452 | 2 |
| `model.garden.cacti` | 0.57 × 0.52 × 0.47 m | 1,940 | 1 |
| `model.garden.succulents` | 0.62 × 0.17 × 0.62 m | 1,980 | 1 |
| `model.garden.bostonFern` | 0.73 × 0.98 × 0.79 m | 1,968 | 3 |
| `model.garden.palm` | 0.77 × 0.89 × 0.69 m | 1,926 | 2 |
| `model.garden.orchid` | 0.43 × 1.20 × 0.38 m | 1,554 | 3 |
| `model.garden.bonsai` | 0.62 × 1.35 × 0.41 m | 3,144 | 4 |

`check-glb`: 10 models, 0 failed; warnings: the bonsai's four draw calls. Khronos validator: 0 errors, 0 warnings
each.

**By a wall, turned any way.** Houseplants turn freely (15° steps), so a plant's leaves stay within 44 cm of its
middle (the wall's face, 43 cm from a tile's middle, and the gate's 1 cm), whichever way it faces; the model and its
old part both. The monstera (was 1.33 m across, 24 cm into the wall), the fern on its stand (4 cm) and the parlour
palm (6 cm) had their leaves pulled in on 2026-10-15: the monstera's stems leave the pot closer in and its leaves are
a little smaller and hang more (its pot 36 cm across, was 42), the fern's fronds a little shorter and hanging further
over the rim, the palm's stalks more upright with shorter fronds. All ten now reach 25–43 cm from their middles.

## The flowers (G2)

`model.flowerbed` and `model.garden.tulips`, `daffodils`, `daisies`, `sunflowers`, `poppies`, `marigolds`,
`pansies`, `lilies`, `lupins`, `forgetMeNots`, `cosmos`, `meadow`, `geraniums`, `flowerBox`, over the old
primitive parts. Twelve stand on a low soil mound (`mound`, material `bed`), the part the player colours ("Bed");
where a bed's petals are one colour they are a second part ("Petals"). The geraniums colour their pot and flowers,
the flower box its box, the raised bed its boards; the meadow, all grass and mixed flowers, has none.

- **Flower faces** (`star`): a thin star of petals, cupped, leaning towards a heading; a low `dome` for the middle.
  Tulips, poppies and lilies are turned cups (`cupped`). Marigolds, forget-me-nots and lupins are leafy `lump`s and
  cones on `foliage` in their flower colours, as the old parts drew them; other flowers are `satin` (still).
- **Leafy clumps** (`clumps`, `lump`): a smooth sphere pushed about by noise, flattened.
- **The meadow's picked flower**: a big white daisy at the front, its own part (stem and face on `satin`: one draw
  call), `pick` squashes it to nothing in 0.8 s for `pickFlower`; the game brings it back after.

The old parts' mounds sank 6 cm into the ground, so their bounds read a little taller than the beds stand; the
flowers here are set so each model is within 10 % of them (the geraniums' leaves were drawn in to a rounder clump and
the old part's pot narrowed to the model's 40 cm on 2026-10-15).

| Model | Size (w × h × d) | Triangles | Draws |
|---|---|---|---|
| `model.flowerbed` | 0.94 × 0.48 × 0.94 m | 1,872 | 3 |
| `model.garden.tulips` | 0.84 × 0.62 × 0.84 m | 1,858 | 2 |
| `model.garden.daffodils` | 0.84 × 0.49 × 0.84 m | 1,938 | 2 |
| `model.garden.daisies` | 0.84 × 0.38 × 0.84 m | 1,954 | 2 |
| `model.garden.sunflowers` | 0.84 × 2.11 × 0.91 m | 2,880 | 2 |
| `model.garden.poppies` | 0.84 × 0.66 × 0.84 m | 1,840 | 2 |
| `model.garden.marigolds` | 0.88 × 0.34 × 0.84 m | 1,924 | 2 |
| `model.garden.pansies` | 0.84 × 0.26 × 0.84 m | 1,874 | 2 |
| `model.garden.lilies` | 0.84 × 0.94 × 0.87 m | 1,932 | 2 |
| `model.garden.lupins` | 0.84 × 0.99 × 0.84 m | 1,886 | 2 |
| `model.garden.forgetMeNots` | 0.87 × 0.28 × 0.84 m | 1,660 | 2 |
| `model.garden.cosmos` | 0.89 × 0.88 × 0.84 m | 1,940 | 2 |
| `model.garden.meadow` | 0.87 × 0.57 × 0.91 m | 1,986 | 3 |
| `model.garden.geraniums` | 0.42 × 0.65 × 0.47 m | 1,964 | 2 |
| `model.garden.flowerBox` | 0.95 × 0.73 × 0.50 m | 1,898 | 2 |

`check-glb`: 15 models, 0 failed, no size warnings. Khronos validator: 0 errors, 0 warnings each.

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
bush stands in the woods (`nature.ts` `natureDecor`, thin instances), so they cost what the old parts did.
The lot dressing's other two old parts, `model.bush.small` (also in the woods) and `model.bush.flowering`, and
the streets' and hedged lots' `model.hedge` are built here too (`bushSmall`, `bushFlowering`, `streetHedge`: the
shrub, the shrub with pink blossom, the garden hedge's block without its sprigs), each one draw call on `foliage`.

- **Shrubs** (`shrub`): a dark core lump and clumps round it, lighter on top; flowers (`dots`) spread over the upper
  part of an ellipsoid round the crown.
- **Clips:** a trim (`trim`, for `trim`) shrinks a cluster of shaggy sprigs (`sprigs`) on the boxwood, the topiary's
  tip, the hedge and the azalea; a prune (`cut`, for `prune`) one rose at the front; a cut (`cut`, for `cut`) the
  three lavender stems nearest the front. Each part is one draw call on its own finish. (The topiary's sprigs stood
  inside its cone, so its trim showed nothing; since 2026-10-17 they reach out of the cone's sides near the tip.)
- **Colour parts:** only the topiary has a pot ("Pot"); the hydrangea, azalea and lavender colour their flowers, the
  roses their roses; the rest none. Each one's flower heads are all one material, the colour part, so "Flowers"
  colours every head (they cycled three or four shades of one hue until 2026-10-17, and the colour reached only the
  first; the geraniums' clusters too).
- **Perches** (2026-10-22): where the game's birds sit (`birds.py`), the entry's `perches` in the model's game axes
  (`perches`, `gamekit/perch.py`, as the trees' in trees.md). Lines come in from round and above towards the shrub's
  middle, or along a hedge's top (`hedge_top`), and meet its leaves. The perch is 2 cm in from there, on leaves that
  face up a little at least, with nothing over it within 15 cm, spread at least 20 cm apart. Found before the trimmed
  sprigs and the cut rose are built. The rose bush's canes are offered too, half way up their arch, but none stands
  in the open (`G.perch.bare`), so its five are on its leafy base.
  The bush has 6, `model.bush.small` 4, `model.bush.flowering` 6, `model.hedge` 6, the boxwood 4, the topiary 4
  (on its cone), the garden hedge 6, the hydrangea 5, the rose bush 5 and the azalea 5. Lavender, ferns, grass and
  hostas have none. Perches only add data: the meshes are unchanged (same triangles as the table), and so are the
  committed files. Beside them each entry's `habitat` says what they are on: `"hedge"` for the two hedges
  (`model.hedge`, `model.garden.hedge`), `"bush"` for the other eight (the topiary and the boxwood too).

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| `model.bush` | 1.04 × 0.91 × 1.10 m | 2,580 | 700 | 1 |
| `model.garden.boxwood` | 0.70 × 0.75 × 0.72 m | 1,136 | 200 | 2 |
| `model.garden.topiaryCone` | 0.64 × 1.80 × 0.66 m | 1,864 | 660 | 3 |
| `model.garden.hedge` | 1.55 × 1.17 × 0.92 m | 1,908 | 458 | 2 |
| `model.garden.hydrangea` | 1.20 × 0.96 × 1.18 m | 3,536 | 940 | 1 |
| `model.garden.rosebush` | 0.77 × 0.90 × 0.72 m | 1,866 | 576 | 3 |
| `model.garden.azalea` | 1.16 × 1.01 × 1.15 m | 3,932 | 1,324 | 2 |
| `model.garden.lavender` | 0.61 × 0.87 × 0.63 m | 1,984 | 228 | 3 |
| `model.garden.fern` | 0.79 × 0.48 × 0.93 m | 1,976 | 616 | 1 |
| `model.garden.grass` | 1.22 × 1.21 × 1.28 m | 1,962 | 506 | 2 |
| `model.garden.hosta` | 0.62 × 0.84 × 0.68 m | 1,966 | 666 | 2 |

`check-glb`: the family's 36 models, 0 failed; its one warning is the bonsai's 4 draw calls. (The grass's and
hosta's old parts were widened to their models on 2026-10-15: they were 13 % and 11 % narrower.)
Khronos validator: 0 errors, 0 warnings on every new file.

## The edibles (G4)

`model.garden.vegPatch`, `tomatoes`, `strawberries`, `herbs`, `pumpkins`. The four crops (content `grows`) are three
parts each: the still bed, the leaves (`plants`) and the crop (`crop`), in one growth clip, `grow`
(`motions.growth`, `G.stages`): ripe at rest; picked, the crop gone; growing, the leaves 0.8 high and sunk a tenth
of their height into the soil; planted, half high and sunk a third (`SINK`), so what shows is each plant's top, small,
where the plant stands (scaled about the bed's middle, as first built, the seedlings drew together there). The
vegetable patch's plants are its crop: picked, they are under the soil and the bed is bare (`harvested`); its
lettuces, round heads 17 cm tall (13 cm until 2026-10-15, when they stayed under the soil at the planted stage), show
as small tops at every stage, as the cabbages and carrots do. There is no harvest clip: harvesting moves a crop to its
picked stage, and the game plays the clip on from ripe, so the crop goes while the resident picks (seen in play:
the tomatoes picked autonomously, stage 2 to 3, the plants left without fruit). The herb planter doesn't grow; its
basil shrinks away for `pickHerbs` (`harvest`). The bed is the colour part ("Bed"): the boards of the raised bed and
the planter (plain, to stay within three draw calls), the soil mound, straw or patch of the others.

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| `model.garden.vegPatch` | 0.92 × 0.40 × 0.92 m | 1,934 | 756 | 3 |
| `model.garden.tomatoes` | 0.80 × 1.19 × 0.80 m | 3,750 | 816 | 3 |
| `model.garden.strawberries` | 0.88 × 0.21 × 0.88 m | 1,964 | 640 | 3 |
| `model.garden.herbs` | 0.84 × 1.01 × 0.46 m | 1,960 | 680 | 3 |
| `model.garden.pumpkins` | 1.11 × 0.27 × 0.95 m | 1,822 | 642 | 3 |

`check-glb`: 5 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings. Each stage checked in
play from a state with all four crops at each (picked, planted, growing, ripe).

## The authenticity review's rebuild (G-29, G-38, 2026-10-21)

- **Bonsai** (I-garden-09): on a bonsai stand at bench height, its top at 0.76 m (was a knee-high table at 0.38) with
  a shelf below, so it is pruned standing at a working height (reference `bonsaiBench`: 75-95 cm). The sizes table
  still reads its 1.35 m against `houseplantSmall`: the tree on the stand, as before.
- **Rose bush** (M-garden-06): open, upright canes from a low leafy base, leaf clusters along them and a cupped rose
  (`ROSE`, a revolved cup curled in over its middle) on each; the one at the front is the cut one.
- **Azalea** (M-garden-06): 36 small, flat trusses all over it, down its sides (`dots` `far_every=2`: a far version
  builds every other one, drawing every random number), not the 20 big rose-like heads.
- **Meadow** (M-garden-06): long grass, 64 thin blades up to 0.5 m leaning every way and stalks gone to seed, over
  low tussocks (a far version: every third blade).
- **Strawberries** (M-garden-07): each plant a rosette of five spreading leaves, two upright in the middle and a dark
  heart; `_crop(..., sink=(0, 0.04, 0.12))`, so a seedling sits flat on the straw rather than sunk into it.

## The detail pass (2026-10-22)

Every model rebuilt with more of what a real plant has, within half its size class's triangle target (plants are
placed by the dozen: 2,000 under 1 m, 4,000 to 2.5 m), each far version (`far_of`) about a quarter to a third of it.
Same keys, sizes (each fitted to its old size, `fit_to`, since a plant grown from random numbers only shows its reach
once built), parts, clips, `motions`, `paint` materials and draw calls; perches as many (the small bush 5, was 4). The
tables above carry the new triangles.

New helpers (trees.py, street.py and garden_things.py import the old ones, unchanged):

- **`cut_leaf`**: a leaf cut from its outline (`outline`: a profile with a wavy or toothed edge; `pinnate`: leaflets
  along a midrib, swept forward and standing apart; `slits`: a monstera's splits; `lobed`: a palmate leaf), a closed
  sheet whose top and underside meet at the edge (no walls: 2 triangles a point of outline), folded and drooping as
  `leaf`. Triangulated by ear clipping: `BEAUTY` filled the notches between leaflets.
- **`rays`**: a flower's face of separate rounded petals (6 triangles a petal), cupped or curling back, with a raised
  middle: daisies, poppies (4 broad), lilies (curled back, round a throat), azaleas, pansies, a rose (two layers).
- **`leafy_shrub`**: as `trees.py` `oak`: woody stems (`twig` colour, on `foliage`, so still one draw) rising and
  forking into broken masses of clumps round a dark heart, sprays of leaves breaking the outline, flowers added on
  the masses (`on_masses`); **`florets`**: a bumpy ball of florets (mopheads, pompoms, trusses, plumes, spikes).
- **`vessel`** (a pot with a rolled lip, a foot, a saucer) and **`soil_top`** (an uneven soil mound in it); **`twig`**,
  **`stalk`**: tapering, bowed stems; `mound` is a little uneven now (the same on a far version, so it keeps its floor).

| Model | Before → after (far) | What it has now |
|---|---|---|
| `plant` | 2,588 → 1,884 | rolled-rim pot on a saucer, heart leaves on arching stalks, young rolled leaves |
| `monstera` | 2,840 → 1,576 | moss pole, leaves split from the edge, aerial roots |
| `fiddleFig` | 1,924 → 2,080 | bark trunk with a wobble, 22 wavy fiddle-shaped leaves on stalks, footed pot |
| `snakePlant` | 1,220 → 1,452 | sword blades with pale yellow edges, young blades |
| `cacti` | 1,860 → 1,940 | ribbed column (lofted) and arms (swept ribbed profile), ribbed barrel, flowers, grit |
| `succulents` | 3,300 → 1,980 | rolled-rim bowl, grit and pebbles, fat rosettes and two spiky ones |
| `bostonFern` | 2,796 → 1,968 | turned stand top, tapering splayed legs, fronds of leaflets |
| `palm` | 1,644 → 1,926 | fronds of narrow leaflets on bowed stalks, a young upright frond |
| `orchid` | 1,632 → 1,554 | turned pedestal, bark in the pot, aerial roots, stake, moth-orchid flowers and buds |
| `bonsai` | 1,368 → 3,144 | stand rails, lofted tray with a rim and feet, stones, gnarled trunk, surface roots, pads of clumps |
| `flowerbed` | 3,044 → 1,872 (784) | boards two to a side, corner posts, cap rail, leaf rosettes, petalled flowers |
| `tulips` | 1,672 → 1,858 (620) | closed three-petal cups, broad leaves |
| `daffodils` | 2,530 → 1,938 (722) | six-petal faces, ruffled trumpets, strap leaves |
| `daisies` | 3,452 → 1,954 (502) | ray petals round raised eyes, leaf rosettes |
| `sunflowers` | 1,344 → 2,880 (784) | two rows of petals, domed seed disc, green back, toothed heart leaves on petioles |
| `poppies` | 2,152 → 1,840 (666) | four-petal cups, dark middles, buds, cut leaves |
| `marigolds` | 2,660 → 1,924 (624) | ruffled pompoms, cut leaves |
| `pansies` | 2,000 → 1,874 (662) | overlapping round petals with a dark blotch, leaf rosettes |
| `lilies` | 1,444 → 1,932 (672) | petals curled back round a throat, leafy stems, buds |
| `lupins` | 1,120 → 1,886 (576) | whorled tapering spikes, hand-shaped leaves |
| `forgetMeNots` | 2,460 → 1,660 (624) | bumpy drifts of florets, single open flowers |
| `cosmos` | 2,146 → 1,940 (540) | thread-cut leaves, eight-petal flowers with eyes |
| `meadow` | 3,540 → 1,986 (802) | thin grass, seed heads, petalled flowers; the picked daisy of twelve petals |
| `geraniums` | 1,648 → 1,964 (576) | scalloped round leaves on stalks, floret trusses, rolled-rim pot |
| `flowerBox` | 2,912 → 1,898 (798) | boards, rim, petalled flowers on stems, ivy trailing over the front |
| `bush` | 700 → 2,580 (700) | stems, 12 broken masses, leaf sprays |
| `bushSmall` | 700 → 1,926 (536) | the same, smaller and darker |
| `bushFlowering` | 1,596 → 3,856 (1,072) | the same, with 26 sprays of five-petalled blossom |
| `streetHedge` | 408 → 1,656 (374) | a clipped block battered in at the foot, rounded over, lumpy, on stems, tufts grown back |
| `boxwood` | 604 → 1,136 (200) | a clipped ball of small leaves on stems |
| `topiaryCone` | 764 → 1,864 (660) | footed stone pot, lofted clipped cone, tufts grown back |
| `hedge` | 660 → 1,908 (458) | as the street hedge, its sprigs as before |
| `hydrangea` | 1,404 → 3,536 (940) | the shrub with 12 floret mopheads |
| `rosebush` | 1,920 → 1,866 (576) | three-leaflet leaves along the canes, open two-layer roses |
| `azalea` | 2,276 → 3,932 (1,324) | the shrub with 34 trusses of open flowers |
| `lavender` | 872 → 1,984 (228) | narrow leaves out of the mound, whorled spikes |
| `fern` | 1,288 → 1,976 (616) | fronds of leaflets, fiddleheads |
| `grass` | 1,120 → 1,962 (506) | 54 thin blades, feathery nodding plumes |
| `hosta` | 676 → 1,966 (666) | wavy heart leaves on stalks, nodding bells |
| `vegPatch` | 1,590 → 1,934 (756) | boards and posts, ruffled lettuces, cabbages in outer leaves, feathery carrot tops |
| `tomatoes` | 2,380 → 3,750 (816) | stems up the canes, ties, leaves of leaflets, calyxes on the fruit |
| `strawberries` | 2,212 → 1,964 (640) | uneven straw, three-lobed leaves, capped berries |
| `herbs` | 1,294 → 1,960 (680) | boarded planter with stretchers, basil and sage leaves, rosemary sprigs, tubular chives |
| `pumpkins` | 1,174 → 1,822 (642) | vines with tendrils, lobed toothed leaves, ribbed pumpkins, a flower |

`check-glb`: 44 models, 0 failed; its one warning the bonsai's four draw calls, as before. Khronos validator: 0
errors, 0 warnings on all 78 files. Seen in the game (gallery WebGPU and WebGL, clips at 0, 0.5 s and open).

## Rebuild

```
cargo art plants            # the .glb files, the pack, check-glb
cargo art plants --preview  # previews/plants/<id>.png, <id>@<clip>.png
```
