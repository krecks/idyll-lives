# Garden decor: notes

The garden decor of `garden.json`: the gnome, the bird bath, the two-tier fountain, the stone lantern, the
wheelbarrow planter and the boulders. Family `garden`: the models go to `web/public/assets/models/garden/`, the
entries to `packs/gardenDecor/manifest.json` (`cargo art garden_decor`). Built in the model's build axes (z up) at the old parts' sizes (within
10 %), each stood on the floor and centred by `settle`. The fountain's water moves all the time near the camera (`flow`, ambient);
the rest are still.

- **Gnome**: boots with turned-up toes, a bell tunic with a hem, a belt with a brass buckle and buttons, sleeves swept
  round to his hands resting on his belly; a round face with ears, rosy cheeks, eyes under bushy brows and a big nose,
  a long lofted beard and a moustache; a hat with a rolled brim flopping back to its tip. Paint: Hat, Tunic, Belt.
- **Bird bath**: a moulded plinth, a fluted column on a foot under a turned capital, a bowl lofted out to a rim
  waving in twelve scallops, water with a ring on it, a robin on the far rim dipping its beak (`songbird`, head
  tipped down). Paint: Bowl & column, Plinth.
- **Fountain**: an octagonal step, a round basin with a moulded rim, a turned stem with four spouts each pouring an
  arc of water into it, an upper bowl with a scalloped lip spilling over in four streams, a finial in tiers and a
  knob; the jet a lofted bubble (`flow` swells it), rings on both waters (`flow` spreads them). Paint: Fountain,
  Plinth.
- **Stone lantern** (tōrō): a hexagonal footing under a ring of carved lotus petals, a round post with a band, a
  platform on petals, the fire box (the warm light inside, its glow the content's `light`) between six posts with
  moon windows pierced through four side panels, open to the front and back, a roof whose corners turn up and curl,
  a lotus cup and the jewel. Paint: Lantern, Base.
- **Wheelbarrow planter**: a pressed tray lofted from a rounded bottom out to a rolled rim, full of earth; wooden
  handles in one bent piece each from the axle back under the tray, rubber grips; bent steel legs with stays and feet,
  axle brackets; a wheel of a treaded tyre, rim, six spokes and hub; leafy clumps on `foliage`, flowers on stems with
  eyes, leaves trailing over the rim. Paint: Tray, Handles, Legs & tyre.
- **Boulders**: three weathered stones in three greys, faces split off flat, feet cut into the lawn (the woods' rock's
  cuts), moss on their shaded tops and in the crack, pale lichen, tufts of long grass (blades fanning out of a low
  clump) and a few loose pebbles. Paint: each boulder.

Detail pass (2026-10-22): rebuilt with the forms and the kit, same sizes, materials' finishes, paint and draws.
Garden decor is placed on many lots, so each aims at half its size-class target (objects.md, Budgets): the gnome and
the wheelbarrow come down by half while showing more.

| Model | Size (w × h × d) | Triangles (before → after) | Draws |
|---|---|---|---|
| garden.gnome | 0.24 × 0.59 × 0.19 | 5840 → 3296 | 1 |
| garden.birdbath | 0.71 × 0.90 × 0.71 | 2458 → 2756 | 1 |
| garden.fountain | 0.94 × 1.14 × 0.94 | 3828 → 4496 | 3 |
| garden.lantern | 0.57 × 1.03 × 0.49 | 932 → 2076 | 1 |
| garden.wheelbarrow | 0.64 × 0.80 × 1.00 | 7204 → 3958 | 3 |
| garden.boulder | 1.03 × 0.47 × 0.89 | 1902 → 2492 | 2 |

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings on each file. In the game
(2026-10-22, headless, WebGPU and WebGL2): the gallery and the fountain's `flow` on the clips bench, no errors.

Built in Rust since 2026-11-18, proved against the last Blender build (`cargo check-glb --against ref/blender`). The boulders'
cuts are the ground's (`common::cut` (the nature group's), on artkit's `edit::bisect_plane`, `holes_fill` and `triangulate`): a
stone's fractures are cut where it was made, at the origin.

## The gate's edges and texels (2026-11-25)

The stone lantern's hexagons have their corners eased 5 mm (`common::eased`: each 60° turn in two of 30°), the
platform's rims and the roof's eave rounded over (rows added, no turn of 60°), its lotus petals' rim rolled, the
posts bevelled boxes (were square cones) and the panels bevelled 4 mm: 52 edges to none, 3,992 triangles (was
2,076; the aim for its 1.03 m is 8,000). The fountain's stem has its collar and top rounded and both basins' water,
like the birdbath's, a 4 mm meniscus. The gnome's beard is 14 round with its top rounded in. The boulder's moss,
lichen and grass tufts have their scale in their meshes (a `boulder!` is scaled so), so the foliage is laid on them as
they are drawn (squashed after, it stretched over 2×).
