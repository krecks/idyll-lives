# Water's edge

The set (`model.waterEdge.*`): a jetty, a rowing boat, a clump of reeds, a beach hut and a bench that faces the
water. Built by `cargo art water_edge`; the pack is `packs/water_edge/manifest.json`.

- **Jetty** (2 × 4): twelve planks across two bearers on eight posts, with a mooring bollard at each front corner.
  The deck is at 0.585 m; the posts reach the floor so it stands in the water rather than floating.
- **Rowing boat** (2 × 3): a lofted hull with a dark interior, two thwarts and two oars. The keel touches the floor
  amidships; the ends lift.
- **Reeds** (1 × 1): eighteen strap blades, mostly upright, and six bulrushes with brown seed heads. Blades and
  stems are on `foliage`, so they sway in the wind; the heads are bark.
- **Beach hut** (2 × 2): a striped cabin on two timber skids, a door and a window to the front, under a shallow
  gable with a ridge cap. 2.18 m to the ridge.
- **Bench** (2 × 1): the `kit::bench_end` ends the street bench uses, four slats and a brass plaque on the top back
  slat. Its back is to the land, so a sitter looks out over the water (+z).

The family's palette is shared; the bench and boat keep their own wood and paint colours. No new helpers were
needed: `bench_end`, `leaf`, `loft`/`ring` and `fsweep` cover everything.
