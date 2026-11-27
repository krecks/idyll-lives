# Fields (`crates/art/outside/src/fields`)

The module builds the five field-boundary models into `web/public/assets/models/fields/` and their entries in
`packs/fields/manifest.json`: the dry-stone wall (`model.fields.dryStoneWall`), the hedgerow
(`model.fields.hedgerow`), the five-bar gate (`model.fields.gate`), the stone stile (`model.fields.stile`) and the
fingerpost (`model.fields.fingerpost`). Their content is `web/public/content/countryside.json`
(`countryside.dryStoneWall`, `.hedgerow`, `.gate`, `.stile`, `.fingerpost`): bought in Build mode's Community,
Countryside group, free, outdoors, no uses. Nothing places them in the world yet: no world-generation layout, spot
or decor list names them.

They are built in the game's axes (`F`, origin at the middle of the footprint on the floor). The sizes are real ones
(`docs/review/reference.json`, with their sources): the wall 2 m long and 1.14 m high, the gate 1.9 m wide, the stile
and fingerpost on one 1 m tile.

## The models

- **Dry-stone wall** (`walls.rs`, 2,944 triangles, one draw): six courses of field stones narrowing from 0.6 m at the
  foot to 0.36 m under the coping (the batter: the West of Scotland DSWA's training wall is 1.4 m high, 70 cm at the
  base, 35 cm at the top), each stone its own length, a little lower than its course or set back, in the boulders'
  weathered greys (`garden_decor`'s rock colours); a coping of stones set on edge; lichen patches on the front face.
  The stones are bevelled boxes with a one-step bevel (two steps doubled the triangles and showed nothing at the play
  camera).
- **Hedgerow** (`hedges.rs`, 1,656 triangles, one draw on `foliage`): the garden and street hedge's clipped block
  (`hedge_block!`) in hawthorn-and-hazel greens, with its untrimmed tufts. It takes the botany palette as its own
  (`.palette(hedges::colours())`), so its stems and twigs are on `foliage` as the street hedge's are, not a second
  draw on `bark`. It carries the `hedge` habitat and eight perches along its top.
- **Five-bar gate** (`gates.rs`): two oak posts with caps, five bars, two stiles, a diagonal brace and three iron
  hinges with a latch. The leaf is fixed shut; it is scenery, not an animated gate. 1.9 m: a small field gate (they
  are made 5 to 14 ft wide).
- **Stone stile** (`walls.rs`): a length of the wall a course lower (0.96 m), with two through-stones standing out of
  each face as steps, 0.3 m apart (BS 5709's step) and staggered, so the climb goes up one side and down the other.
- **Fingerpost** (`gates.rs`): an iron post on a stone footing with a ball finial and three white arms with pointed
  ends, 2.1 to 2.5 m up, at a T junction: two along the lane (0° and 180°) and one down the side lane (90°, to the
  front). The arms reach 0.5 m from the post, so the two along the lane span its tile (a real one's are 0.6 m and
  more). The side arm makes the box reach forward: it is settled with `centre = false` and declares
  `.centre(0, 0.17)`, so the post stays on the middle of its tile.

## Decisions

- **No far versions.** These are placeable decor, like the outdoor and street-furniture items, not drawn in the town
  view by the hundred. If the world generator comes to place them along fields, give the wall and hedgerow
  `far_low()` first (the hedgerow's block already has a low build, the street hedge's far version).
- **The hedge block is the kernel's, unchanged.** The session that made this family had clad `botany::hedge_block`
  in leaf cards for the hedgerow, which rebuilt every hedge in the game: the street hedge (`model.hedge`, the most
  placed of them) went from 1,656 triangles to 7,794 (far 374 to 3,722), 83 KB to 461 KB, one draw call to two. It
  was taken back out (review of 2026-11-19): a card-clad hedge is the owner's call, with a far version and a cost to
  match.
- **No lettering on the fingerpost.** `kit::letters!` supports only the letters A, E, I, M and R, and no destination
  in the game needs those alone. The arms carry the read by silhouette and colour.
- **The gate does not open.** No interaction or clip; it matches the static field-boundary set.
- **Content group.** The five objects are in `web/public/content/countryside.json` under the existing `community` /
  `countryside` group, so no new group, content file or `base.json` include is needed.

## Hedgerow (2026-11-27)

The hedgerow's block is `hedge_block` with `rough = 1`: an uneven top of swollen clumps, five bare saplings with leaf
sprays standing out of it to 1.45 m, and nine haws (red, on `foliage`). 3,006 triangles, one draw.
