# Tabletop — notes

`tabletop.py` builds the 39 tabletop things (docs/objects/tabletop.md, TT1–TT5) into `web/public/assets/models/tabletop/`
and their entries in `packs/tabletop/manifest.json`. Built 2026-10-14; they sat in Validate (Kitchenware, Tableware, Books And Pots, Small Ornaments, Desk And
Bath Bits) until checked in play, then went back to `tabletop` and their old group (`kitchenware`, `tableware`, `books`,
`plants`, `ornaments`, `bath`, `desk`, `toys`).

## What they keep

- **Origin at the base, centred** (the `surface` layer: the renderer lifts them to their host's top). `G.settle` puts
  them there.
- **Size** no wider than the content's `radius` (the disc a thing takes on a top; the gate warns of one reaching
  over 10 % past it). The four widest (`plateSet`, `teapotSet`, `espressoMachineSmall`, `monitor`, radius 0.23–0.3 m) don't fit
  on a counter's top with its edge margin, as before: they go on tables and desks.
- **One draw call** where it can: every finish is a plain one. Wood is `satin` brown, towels and the teddy `matte`,
  leaves `satin` green, so none needs a textured finish. Glass that is seen through is `glass` (a second draw call):
  the cookie jar, the wine glasses and the snow globe. Elsewhere glass is opaque `gloss` (the blender's jug shows the
  smoothie as its lower half).
- **Triangles** under the 1,500 target (226–1,496). Spheres are 6–14 segments; small parts fewer.
- **Player colour**: the main part's material, labelled `Body` (the cookie jar: its `Lid`); sets of like things (mugs,
  books, towels, blocks) list each one.

## Parts left decorative for 0.3.0

These are built with that part on its own node, its origin where it would turn, so a clip can be added without
remodelling (two draw calls until then). Content rules forbid uses on things on a top, so 0.3.0 leaves them
decorative:

- `kettle`: the `lid`, hinged at its back edge (a lid lifts about Blender X).
- `breadBin`: the `roll_top`, a half-tube shell round the bin's axis (it rolls back about Blender X).
- `laptopClosed`: the `lid`, hinged at its back edge.

## The alarm clock

`tableClock` tells the game's time: its hour and minute hands are parts of their own, each turning about the face's
middle once round, `G.spin(hand, '<name>_hand', 1, seconds=12, axis='Y')`, and the game holds each where the time has
it (`motions.clock`, `at` 610: ten past ten at rest, the catalogue picture), on every preset. A positive turn about
Blender +Y is clockwise seen from the front. The red second hand is gone (a game minute is a real second at normal
speed); its red hub stays on the face. Three draws: the body and the two hands.

## The candles (2026-10-21)

`candlesticks` was drawn with its flames lit all day and dark at night (review M-tabletop-01). Its two flames are now
one part of their own (`flames`, one more draw: two), built lit and sunk 3.4 cm into the candles' tops at rest, hidden
in the wax (1.1 cm across to their 1.5). `lit` slides them up over the wicks (`G.slide`, 0.6 s) and is the model's
`night` clip, held open dusk to dawn: by day the wicks are dark, at night the flames stand on them. The flame's
material glows (`#FFC46A`, as the lantern's): bright while the room's lights are on, out with them (a dark room's
candles show their flames unlit). Its height is the candles' at rest: 0.33 m.

## The alarm clock's size (2026-10-21)

`tableClock` was 0.15 × 0.23 × 0.06 m, half again a real twin-bell clock's height (review M-tabletop-02). It is built
at 0.72 (`S`): the case 10.7 cm across, 0.11 × 0.16 × 0.05 m in all, its ticks and hands as thick as
before.
