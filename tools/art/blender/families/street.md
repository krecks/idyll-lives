# Street (`families/street.py`)

The street's dressing props (`model.street.*`): the kerbside bench, the planter with its little tree, the flower tub,
the street-name sign, the bus-stop sign, the door pot, the garden path light and the flower clump.
One pack, `packs/street/manifest.json`, models in `models/street/`. Built 2026-10-16. They aren't buyable: no content entry, no Validate group.

`render/pc/street.ts` draws them as thin instances (`PROP`, `FOLIAGE`): benches, planters and flower tubs along
the kerb in front of the public places, the bus-stop and street-name signs at the ends of the streets, pots by the
neighbours' front doors and path lights along their garden paths, flower clumps in the beds along their house fronts.
The town overview draws the same street again from afar, from the far versions (`BuildOptions.far`). Each model is
built where the game places it (game axes, `F`), at its size within 2 %; the bench and the bus-stop sign stand
off their middle as the old parts do (`G.settle(centre=False)` and `centre=`).

Nothing moves: no clips. The leaves and flowers are on the `foliage` finish, as the old parts' were, so the nature
shader sways and tints them (the planter's crown, the tub's and the pot's plants, the flower clump). The
planter's trunk is plain `satin` (`trunk`), not `bark`, so the planter stays two draws.

## Night

The path light's lens is `lens` (satin) with `glow` `#FFD9A0`, where the old part had its glass bulb. street.ts
now draws a thin-instanced template's `glows` (it didn't before): one mesh per glowing model, lit with the
streetlights' level (`setNight`), off by day. That also lights the glows of the public places' things the street draws
from another lot (the street lamp's lantern, the vending machines' fronts).

## Far versions

Each has a far version built by hand from its own build at low detail (`G.far_of`): fewer segments (`G.low`), no
bevels, no lettering, timetable lines or bus picture, no bolts, scrolls or stretchers on the bench ends, the tub
without its groove, the flowers as lumps (the tub's big five only, half the clump's), no louvres on the path light.

## The models

Rebuilt in detail on 2026-10-22 with the forms and the kit (`kit.bench_end`, `kit.scroll`, sweeps, lofts, extrusions),
each at most half its size class's triangle target (street props stand by the dozen and the town overview draws them all).

| Model | Size (w × h × d) | Triangles | Far | Draws | What |
|---|---|---|---|---|---|
| `bench` | 1.70 × 0.91 × 0.51 m | 3,076 | 316 | 2 | four rounded slats and three back slats with domed bolts on two cast-iron ends in one piece each (`kit.bench_end`: the back leg curving up into the upright, the front leg bending back into the armrest that curls down in a scroll, the seat rail, a sagging stretcher, a curl under the seat, pad feet), its back to the road |
| `planter` | 0.80 × 2.42 × 0.73 m | 2,532 | 718 | 2 | a concrete tub in one lofted piece (chamfered foot, a groove under a proud rim with chamfered edges, hollow to its jittered soil), a young tree: a tapering trunk with a root flare, four swept branches, two stakes with a rubber tie, a crown of nine leafy lumps |
| `flowerTub` | 0.70 × 0.56 × 0.70 m | 1,952 | 466 | 2 | the same tub (no groove), a leafy mound and corner leaves, five big open blooms (five or six rounded petals round a raised eye, each on a cushion of leaves) and twelve smaller ones, tilted |
| `sign` | 0.90 × 2.66 × 0.90 m | 1,684 | 272 | 1 | a post on a flanged foot with four bolts and a collar, a turned finial, two green blades with rounded corners crossing, white panels set in, a clamp band above and below each, dark green lettering (a capital to each word) on both faces |
| `busSign` | 0.46 × 2.63 × 0.11 m | 1,944 | 356 | 1 | a slim post on a flanged foot, a blue flag with a rolled rim and a white face, on each face a bus with its windows and door cut out and two wheels; the timetable (lines and times) in a framed case under a hood, clamped by two bands with lugs |
| `pot` | 0.43 × 0.87 × 0.42 m | 1,264 | 282 | 2 | a terracotta pot with a foot ring and a rolled rim on a saucer, jittered soil, a clipped leafy ball with a smaller one on top |
| `pathLight` | 0.18 × 0.56 × 0.18 m | 992 | 172 | 1 (+1 at night) | a dark bollard on a stepped, flared foot, a collar, the frosted lens behind three louvre rings on four thin bars, a domed cap, the access hatch's screw |
| `flowers` | 0.60 × 0.34 × 0.53 m | 1,868 | 288 | 1 | three leafy clumps and three tufts, fourteen open flowers in seven colours, each on its stem from where the clump is under it (found by a ray cast), turned to face out |

The street's hedges are `model.hedge` (plants.py's `streetHedge`, street.ts's `FOLIAGE.hedge`): the pack's own hedge,
which nothing drew, was dropped (2026-10-21, authenticity review M-town-02).

Draws are unchanged by the detail pass. Before it (2026-10-16 to 10-22) the bench was 820 triangles, the planter
1,092, the tub 1,408, the sign 944, the bus sign 700, the pot 704, the path light 308 and the clump 860.

`check-glb`: 8 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings on all sixteen files.

## Rebuild

```
cargo art street            # the eight .glb and .far.glb, the pack, check-glb
cargo art street --preview  # previews/street/<id>.png, <id>.far.png
```
