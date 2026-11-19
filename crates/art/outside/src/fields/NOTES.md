# Fields (`crates/art/outside/src/fields`)

The module builds the five field-boundary models into `web/public/assets/models/fields/` and their entries in
`packs/fields/manifest.json`: the dry-stone wall (`model.fields.dryStoneWall`), the hedgerow
(`model.fields.hedgerow`), the five-bar gate (`model.fields.gate`), the stone stile (`model.fields.stile`) and the
fingerpost (`model.fields.fingerpost`). They are placeable countryside items, not fittings of a named place.

They are built in the game's axes (`F`, origin at the middle of the footprint on the floor) and centred by
`settle(&[join_all(..)])`. The sizes are the declared footprints: the wall and hedgerow 2 m long, the gate 1.9 m
wide, the stile and fingerpost within 1 m square.

## The models

- **Dry-stone wall** (`walls.rs`): six courses of stones of varying length, depth and colour, laid without mortar,
  with a coping of stones set on edge and a few lichen patches on the lane side. Every stone is a bevelled box, so
  the wall reads as stacked stone rather than brick.
- **Hedgerow** (`hedges.rs`): a clipped block of hawthorn-and-hazel foliage on bare stems, with the untrimmed tufts
  `hedge_block!` adds. It carries the `hedge` habitat and eight perch points along its top.
- **Field gate** (`gates.rs`): two oak posts with caps, five bars, two stiles, a diagonal brace and three iron
  hinges with a latch. The leaf is fixed shut; it is scenery, not an animated gate.
- **Stone stile** (`walls.rs`): a short length of the same wall with two worn steps projecting on the lane side, the
  lower one standing further out, so the climb reads from the front.
- **Fingerpost** (`gates.rs`): an iron post on a stone footing with a ball finial and three white arms at different
  heights, turned 120° apart. The arms are extruded blades with pointed ends.

## Decisions

- **No far versions.** These are placeable community decor, like the outdoor and street-furniture items, rather than
  town-overview fittings. If they are later placed by the hundred, `far_low()` should be added to the wall and
  hedgerow first; the build already draws its random numbers in a fixed order, so a reduced version can keep the
  same silhouette.
- **No lettering on the fingerpost.** `kit::letters!` supports only the letters A, E, I, M and R, and no destination
  in the game needs those alone. The three arms carry the read by silhouette and colour instead of inventing a
  parallel sign-lettering helper.
- **The gate does not open.** No interaction or clip was added; it matches the static field-boundary set and keeps
  the model out of the animation checks.
- **Content group.** The five objects are appended to `web/public/content/countryside.json` under the existing
  `community` / `countryside` group, so no new group, content file or `base.json` include is needed.
