# Street (`families/street.py`)

The street's dressing props (`model.street.*`): the kerbside bench, the planter with its little tree, the flower tub,
the street-name sign, the bus-stop sign, the door pot, the garden path light and the flower clump.
One pack, `packs/street/manifest.json`, models in `models/street/`. Built 2026-10-16. They aren't buyable: no content entry, no Validate group.

`render/babylon/street.ts` draws them as thin instances (`PROP`, `FOLIAGE`): benches, planters and flower tubs along
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
bevels, no lettering, timetable lines or bus picture, half the flowers of the clump.

## The models

| Model | Size (w × h × d) | Triangles | Far | Draws | What |
|---|---|---|---|---|---|
| `bench` | 1.70 × 0.91 × 0.51 m | 820 | 308 | 2 | four wooden slats and three back slats on two cast-iron ends (legs, back upright, seat rail, armrest), its back to the road |
| `planter` | 0.80 × 2.42 × 0.73 m | 1,092 | 336 | 2 | a concrete tub with a lip and plinth, soil, a young tree: a slim trunk forking into a crown of six leafy lumps |
| `flowerTub` | 0.70 × 0.57 × 0.70 m | 1,408 | 522 | 2 | the same tub, a leafy mound and corner leaves, five big blooms (red, yellow, pink, blue, white) and nine smaller ones |
| `sign` | 0.90 × 2.66 × 0.90 m | 944 | 164 | 1 | a grey pole with a finial, two green blades crossing (white panels, dark green lettering on both faces), clamps |
| `busSign` | 0.46 × 2.63 × 0.11 m | 700 | 208 | 1 | a blue flag with a white face and a little blue bus on both sides, on top of the pole (the pole no longer runs across its faces), and the timetable in a case |
| `pot` | 0.43 × 0.87 × 0.42 m | 704 | 296 | 2 | a terracotta pot with a rim, soil, a clipped leafy ball with a smaller one on top |
| `pathLight` | 0.18 × 0.56 × 0.18 m | 308 | 140 | 1 (+1 at night) | a dark bollard on a flared foot, a frosted lens between four thin bars, a flat cap |
| `flowers` | 0.59 × 0.34 × 0.53 m | 860 | 288 | 1 | three leafy clumps and fourteen little flowers in seven colours |

The street's hedges are `model.hedge` (plants.py's `streetHedge`, street.ts's `FOLIAGE.hedge`): the pack's own hedge,
which nothing drew, was dropped (2026-10-21, authenticity review M-town-02).

Against the old parts on the street beside the viewed lot (the village bench scene): 11 draws for the props instead
of 13 (the path light's glass and the planter's bark are gone), and a third of the triangles per copy or less (the
bench 820 against 8,820, the sign 944 against 2,784, the planter 1,092 against 3,144); the path light is the one with
more (308 against 120).

`check-glb`: 9 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings on all eighteen files.

## Rebuild

```
cargo art street            # the nine .glb and .far.glb, the pack, check-glb
cargo art street --preview  # previews/street/<id>.png, <id>.far.png
```
