# Ground (`families/ground.py`)

The last things on the ground and over the residents' heads, built 2026-10-16.
One pack, `packs/ground/manifest.json`. None of them is an object the player buys: no content entry, no Validate
group. The keys are unchanged.

- **Grass tuft** (`model.grassTuft`): drawn by the ten thousand as thin instances on every unpaved tile and round
  the town (`render/babylon/nature.ts` `lawns`, in squares the camera leaves out, half of them on Low). Four bent
  blades leaning out from the middle in the old part's three greens and the fourth, one draw call
  (`foliage`: the nature shader's wind and per-instance tint). Each blade is a flat ribbon with one joint drawn
  from both sides, two faces back to back with their own vertices (6 triangles), so the camera draws only the 12 of
  the tuft's 24 triangles turned to it. A first build of closed three-sided blades had the old part's 36
  triangles but drew all of them, where the old part's thin cones are mostly degenerate tips and hidden caps:
  it measured 0.9 ms more GPU time on WebGL (0.4 on WebGPU) over the village from afar (`render.mjs`, `wide`,
  Medium); the ribbons measure the same as the old part. The blades are flat-shaded with their normals bent
  towards the sky (`_mesh(sky=…)`, custom normals): near upright, they drew dark against the lawn from above,
  where the old part's cones read light.
- **Wildflowers** (`model.flowers`): on about one tile in seventeen and a tenth of the scatter round the town, one mesh
  over the whole landscape. Five low leaves (ribbons as above), three kinked stems and three five-petalled heads (a
  white, a yellow and a violet, each with a raised eye) where the old part's three stand: 156 triangles where the
  old part took 192, and one draw call where it took two. Every part is on `foliage`, so the heads sway with
  their stems (the old part's satin heads stayed still while their stems swayed) and take the tint; the petals'
  colours are a little brighter than the old part's, as the foliage finish's leaf texture darkens what it is on.
- **Rock** (`model.rock`): the woods' rocks (`nature.ts` `forests`, scaled 0.6–1.3, always at the low level of
  detail and without a shadow), so what the game draws is its far version. Two lumpy boulders where the
  old part's blobs are (the lighter one leaning on the bigger), the big one with three faces split off flat, both
  cut flat 11.5 cm under the ground as the old part's reach, plain matte (one draw call): 492 triangles. Its far
  version is the same two built on icosahedra without the split faces, a little wider to fill the same box:
  56 triangles (the old part at that level: 40), few enough that the woods still draw all of them as one mesh
  (`SPLIT_TRIANGLES`: 36,000 over every rock of the village's woods). Centred where the old part is (game x 8.5 cm,
  depth 1 cm); its `sink` (0.115 m) is how far `check_glb.py` lets it reach under the floor.
- **Marker** (`model.marker`): the sparkle star over the selected resident (`BabylonRenderer` `setupMarker`), which
  the game draws in its own unlit material, shaded by its normals and tinted by mood, with an outline of the same
  mesh pushed out along its normals (an inverted hull), turned to the camera, bobbing and breathing. One closed
  shell: the old part's four concave rays and round core as a smooth union of their depths, the front and back
  meeting at the outline (96 points spaced evenly along it, so the tips are sharp), in rings closer together towards
  the edge. Its normals are worked out from that shape, not from the triangles (thin ones along the rays bent the
  outline through the star's face as dark lines). Its origin is its middle, as the old part's: 1,344 triangles
  where the old part took 1,936 (and its outline as many again).

| Model | Size (w × h × d) | Triangles | Far | Draws |
|---|---|---|---|---|
| grassTuft | 0.12 × 0.29 × 0.14 | 24 | 8 | 1 |
| flowers | 0.14 × 0.27 × 0.17 | 156 | 58 | 1 |
| rock | 1.14 × 0.55 × 0.83 | 492 | 56 | 1 |
| marker | 0.23 × 0.34 × 0.06 | 1,344 | — | 1 |

The lawn always draws the full tuft and flowers (`lawns` builds them at level 0); their far versions are there for
anything that draws them far off. `check_glb.py`: 4 models, 0 failed, no warnings. Khronos validator: 0 errors,
0 warnings on each file and far version.

The spare `model.stove`, `model.plantSmall` and `model.ceilingLamp` went at the same time: nothing in
the game, its content, the crates or the tools drew them.
