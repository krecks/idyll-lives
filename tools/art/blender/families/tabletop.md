# Tabletop — notes

`tabletop.py` builds the 39 tabletop things (plan/10-tabletop.md, TT1–TT5) into `web/public/assets/models/tabletop/`
and their entries in `packs/tabletop/manifest.json`, over the primitive-part placeholders of `catalog2.py`, which stay
the fallback. Built 2026-10-14; they sit in Validate (Kitchenware, Tableware, Books And Pots, Small Ornaments, Desk And
Bath Bits) until checked in play, then go back to `tabletop` and their old group (`kitchenware`, `tableware`, `books`,
`plants`, `ornaments`, `bath`, `desk`, `toys`).

## What they keep

- **Origin at the base, centred** (the `surface` layer: the renderer lifts them to their host's top). `G.settle` puts
  them there.
- **Size** within 10 % of the placeholder on every axis, so the content's `radius` (the disc a thing takes on a top,
  from the placeholder) still holds and `check_glb.py` has nothing to warn of. The four widest (`plateSet`,
  `teapotSet`, `espressoMachineSmall`, `monitor`, radius 0.23–0.3 m) don't fit on a counter's top with its edge margin,
  as before: they go on tables and desks.
- **One draw call** where it can: every finish is a plain one. Wood is `satin` brown, towels and the teddy `matte`,
  leaves `satin` green, so none needs a textured finish. Glass that is seen through is `glass` (a second draw call):
  the cookie jar, the wine glasses and the snow globe. Elsewhere glass is opaque `gloss` (the blender's jug shows the
  smoothie as its lower half).
- **Triangles** under the 1,500 target (226–1,496). Spheres are 6–14 segments; small parts fewer.
- **Player colour**: the main part's material, labelled `Body` (the cookie jar: its `Lid`); sets of like things (mugs,
  books, towels, blocks) list each one.

## Parts kept for a use to come

The plan's "(+use)" items are built with that part on its own node, its origin where it would turn, so a clip can be
added without remodelling (two draw calls until then):

- `kettle`: the `lid`, hinged at its back edge (a lid lifts about Blender X).
- `breadBin`: the `roll_top`, a half-tube shell round the bin's axis (it rolls back about Blender X).
- `laptopClosed`: the `lid`, hinged at its back edge.

## The alarm clock

`tableClock` stands at ten past ten; its red second hand is its own part, turning about the face's middle once a
minute: `hands`, `G.spin(hand, 'hands', 1, seconds=60, axis='Y')`, played as an ambient loop (`motions.idle`), so it
goes round near the camera and not on Low. A positive turn about Blender +Y is clockwise seen from the front (checked on
the clips bench: at `&t=15` it points to three).
