# Water's edge

The set (`model.waterEdge.*`, content in `catalog2/outdoor.json`, Buy mode's outdoor group `waterside`): a jetty, a
rowing boat, a clump of reeds, a beach hut and a bench that faces the water. Built by `cargo art water_edge`; the pack
is `packs/water_edge/manifest.json`. Sizes are held to `docs/review/reference.json`'s kinds `jetty`, `rowingBoat`,
`reeds` and `beachHut` (each with its sources) and `parkBench` for the bench; the first four are decor there (no use).

| Model | Footprint | Size (w × h × d) | Triangles | Draws | What |
|---|---|---|---|---|---|
| jetty | 2 × 4 | 1.92 × 0.98 × 3.82 | 2832 | 1 | twelve planks across two bearers on eight posts, the deck at 0.585 m, a mooring bollard at each corner of the far (+z) end |
| rowingBoat | 2 × 3 | 1.28 × 0.70 × 2.97 | 4636 | 2 | a lofted hull through nine stations, two more on a Catmull-Rom curve between each pair (`forms::catmull`) so the sheer runs fair; a cream liner, a wood transom and stem, cap rails, floorboards, two thwarts and two oars stowed along them |
| reeds | 1 × 1 | 0.80 × 1.54 × 1.09 | 1542 | 2 | 24 strap blades (`leaf!`), mostly upright, and seven bulrushes with brown seed heads; blades and stems on `foliage`, heads on `bark`, so they sway in the wind |
| beachHut | 2 × 2 | 1.94 × 2.51 × 1.95 | 4768 | 2 | a cabin on two skids under a 27° gable to the front, the door (0.76 × 1.72 m) in the middle between two small windows, white bargeboards; stripes down the sides and the back. Eaves at 2 m and the ridge at 2.5 m, as Hove's huts |
| bench | 2 × 1 | 1.75 × 0.93 × 0.52 | 3184 | 2 | the kit's `bench_end` ends, four seat slats (seat 0.475 m, the content's `seat.height`), three back slats and a brass plaque on the top one; two sitters, as the garden bench |

None has moving parts or a far version: none is placed in the world (they are bought), and every one is under half
its size class's target. The windows are `pane` (opaque dark gloss, as the garden shed's), not see-through `glass`
over a solid wall.

Faults found in review (2026-11-19) and fixed:
- The beach hut was a shed: eaves to the front, the door off to one side, a 1.45 m door and 1.62 m walls (a
  playhouse's), a see-through window showing the wall behind it, a bare back. Now gabled to the front at a real
  hut's height.
- The boat's sheer and bilge kinked at each of the nine stations. The fairer hull can't take the kernel's bevel
  (`bevel_edges` panics on its edges), so its hard ends are covered instead: the transom stands a centimetre proud of
  the planking all round, and a stem board closes the bow.
- The bench had three slots on two tiles: the outer sitters, 0.67 m out, sat in the armrests (at 0.85 m).
- The reeds had a "Water the reeds" use; reeds stand in the water, and the ornamental grass has none.
