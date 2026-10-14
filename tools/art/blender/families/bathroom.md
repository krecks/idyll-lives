# Bathroom — notes

`bathroom.py` builds the five bathroom sinks, the five toilets, the four showers, the three baths and the three storage and mirror models (below) into `web/public/assets/models/bathroom/` and their entries
in `packs/bathroom/manifest.json`: `model.sink` (`sink.glb`), `model.sinkPedestal` (`sinkPedestal.glb`),
`model.vanityDouble` (`vanityDouble.glb`), `model.sinkVessel` (`sinkVessel.glb`) and `model.sinkConsole`
(`sinkConsole.glb`). They replace the primitive-part placeholders in `catalog.py` / `catalog2.py`, which
stay as the fallback. First made 2026-10-14. The pack is listed last in the base manifest's `packs`, so it
overrides the base (`model.sink`), `catalog` and `catalog2` entries by key.

## Shared

- **Palette.** One `G.Palette` for the family (`M`): `body` (the painted cabinet / apron), `porcelain`,
  `stone`, `wood`, `trim`, `metal`, `chrome`, `brass`, `blackmetal`, `fabric`, `dark`, `teal`, `water`, and
  `towel` (a plain satin, so a towel does not cost an extra draw call). `PAINT = ['body']`.
- **Water on use.** Each sink has one `water` moving part. The stream is built full length from the spout
  tip into the basin, its origin on the spout tip; before `G.settle()` it is scaled to 0.001 along its
  length, so it is hidden at rest and does not affect the bounds. `G.grow(water, 'water_on', seconds=0.4)`
  keys it from there to scale 1. `motions` plays `water_on` for the `wash` animation tag, with no `hold`,
  so the water runs for the whole use and retracts afterwards (the `wash` tag covers both the `wash` and
  `washUp` interactions).
- **Basins.** `_basin` is a `G.revolve` with thickness (an oval bowl); `_rect_basin` is a rectangular shell
  (floor and four thin walls) for the base cabinet sink, which has a rectangular worktop cut-out. The solid
  cabinet top must not sit above the basin floor, so each sink has painted "shoulders" filling from the
  basin floor up to the worktop — otherwise the painted cabinet shows through as the basin interior.

## The models

| Model | Size (w × h × d) | Triangles | Draws | Note |
|---|---|---|---|---|
| `model.sink` | 0.86 × 1.26 × 0.62 m | 3,320 | 2 | base cabinet, shaker doors, rectangular inset basin, gooseneck |
| `model.sinkPedestal` | 0.56 × 1.06 × 0.47 m | 1,540 | 2 | column, oval bowl, tap deck with soap and a tooth mug |
| `model.vanityDouble` | 1.90 × 1.12 × 0.56 m | 4,396 | 3 | two-basin cabinet, two taps (no mirrors) |
| `model.sinkVessel` | 0.84 × 1.16 × 0.54 m | 1,920 | 3 | counter with a raised vessel bowl and a folded towel |
| `model.sinkConsole` | 0.60 × 1.06 × 0.48 m | 1,108 | 3 | open console on two legs, oval bowl |

All match their placeholder bounds (`check_glb.py`: ok; Khronos validator: 0 errors, 0 warnings). Each
lists the parts the player may colour in `paint`, the `body` first.

## The double vanity has no mirrors

It had two (frames in the file, the glass as primitive `parts`). They were taken off on 2026-10-14: the
player chooses the mirror they hang above it (the wall mirror, the mirror cabinet), as for any sink. Its
placeholder (`catalog.double_vanity`) has none either.

## B1 — the toilets

`model.toilet` (`toilet.glb`), `model.toiletCompact`, `model.toiletSmart`, `model.toiletClassic` and
`model.toiletWallHung`, built 2026-10-14. They sit in Validate › Toilets until checked in play.

- **Against the wall, not centred.** The game sits a resident at the footprint's middle plus the seat's
  `fwd` (characters.ts `SEATS`, else the content's `seat`), so each toilet is built where its placeholder
  stands: the cistern (or the boxed-in cistern) on the wall face at game depth -0.43,
  `G.settle(centre=False)`, and `centre=` on its `Model`. The bowl's middle is 8–16 cm in front of the hips.
- **Seat heights.** The seat's top is the seat height: 0.43 m (`toilet`, `toiletCompact`), 0.44 m
  (`toiletSmart`), 0.445 m (`toiletWallHung`) and 0.45 m (`toiletClassic`). The wall-hung placeholder has its
  seat at 0.6 m; the model follows the content (0.45 m).
- **Shapes.** The bowl is an oval revolve with thickness (`_bowl`), its inside down to a water disc; a foot
  on the floor (`_foot`), or a wall-hung bowl on an oval porcelain block (`_column`) into the panel. The
  seat is a flattened torus, the lid a bevelled oval disc.
- **The lid** is the one moving part (`lid`), hinged across its back edge, on its underside; `lid_up`
  swings it 100° about X in 0.6 s, to lean on the cistern (the hinge is 8.6–9.5 cm in front of it so the
  lid just meets it). The smart toilet's lid is level with its seat housing (the bidet unit) and hinged on
  its top edge, so it lifts over the housing; it rises slower, 0.8 s. Only the lid lifts: the resident sits
  on the seat. `motions` plays `lid_up` for the `toilet` animation tag until the use ends.
- **The classic toilet** is close-coupled with a wooden seat and lid, as its content and placeholder say
  (not a high cistern and chain).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.toilet` | 0.44 × 0.82 × 0.68 m | 3,364 | 2 | Bowl, Seat & lid, Cistern |
| `model.toiletCompact` | 0.59 × 0.81 × 0.63 m | 3,532 | 2 | Bowl, Seat & lid, Cistern (a slim cistern, a push button, a roll on the wall) |
| `model.toiletSmart` | 0.90 × 1.20 × 0.71 m | 3,232 | 2 | Bowl, Seat & lid, Cistern (the panel, with a display, a remote and a roll) |
| `model.toiletClassic` | 0.47 × 0.88 × 0.71 m | 3,600 | 3 | Bowl, Seat & lid (wood), Cistern |
| `model.toiletWallHung` | 0.60 × 1.10 × 0.67 m | 3,088 | 2 | Bowl, Seat & lid, Cistern (the boxed-in panel and flush plate) |

`check_glb.py`: 0 failed. It warns that the compact and wall-hung toilets are off centre from their
placeholders (whose backs are 2.5 and 5 cm inside the wall), and that the wall-hung one is deeper than its
placeholder. Khronos validator: 0 errors, 0 warnings each.

## B2 — the showers

`model.shower`, `model.showerStall`, `model.showerWalkIn` and `model.showerRain`, built 2026-10-14. They sit in
Validate › Showers until checked in play.

- **They fill their tile**, centred, as their placeholders do: the resident steps in onto the middle
  (`inside`). Walls at the back and on the left (game -x), the open sides glass or a curtain.
- **The rain** is the one moving part (`water`): thin streams (`_rain`: a disc of rings, or a grid under the
  square head) built full length from the head's underside to the tray, joined with the origin on the head
  and scaled to 0.001 along their length, so they're hidden in the head at rest. `water_on` grows them down
  in 0.5 s and `motions` holds it for the whole use (the `shower` tag). The stall's head is on the wall, so
  its rain is turned about its origin (a fixed node rotation; the clip keys only the scale) to fall from the
  head onto the resident in the middle.
- **Draw calls: 3** each. The tray, walls and metal (1), the glass (1, see-through `glass`), the rain (1);
  the stall has a fabric curtain instead of glass. The base shower's sliding door stays shut: a moving glass
  door would be a 4th.
- **Tiles** (`_tiled_walls`) are thin satin squares a little proud of a grout backing: the base shower 30 cm,
  the stall 25 cm. The walk-in shower's stone wall has grooved joints; the rain shower's walls are plain stone.
  The rain shower's front glass covers only the left half: the right half is the way in.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.shower` | 0.90 × 2.15 × 0.90 m | 2,968 | 3 | Tray, Frame, Head & tap, Tiles |
| `model.showerStall` | 0.90 × 2.08 × 0.90 m | 2,056 | 3 | Tray, Rail, Head & tap, Curtain, Tiles |
| `model.showerWalkIn` | 0.92 × 2.26 × 0.92 m | 2,348 | 3 | Floor, Frame, Head & tap, Wall |
| `model.showerRain` | 0.94 × 2.19 × 0.94 m | 1,684 | 3 | Tray, Frame, Head & tap, Walls |

`check_glb.py`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.

## B3 — the baths

`model.bathtub`, `model.bathtubClawfoot` and `model.bathtubFreestanding`, built 2026-10-14. They sit in
Validate › Baths until checked in play.

- **2 × 1 tiles, centred.** The resident sits at the middle (`SEATS` 0.25 m and 0.28 m, the content's `seat`
  0.28 m); the tub floors are 0.15–0.17 m, so they sit in the water.
- **The tub** is one closed shell (`_loft`): superellipse rings up the outside, over the rim and down the inside
  to the floor, so the inside faces up into the tub; flat-shaded above 40°. The built-in bath is squarer
  (higher exponent) behind a gloss front and end panels over a dark plinth set back under them.
- **Two moving parts.** The tap's stream (`stream`, the sinks' `_water` way) grows in 0.4 s (`tap_on`, held
  2.4 s, so it runs while the tub fills). The water (`bathwater`) is the tub's inside from the floor up to two
  thirds of the depth, its origin just under the floor, scaled to nothing upright and in to the floor's size
  across: at rest it's hidden under the floor; `fill` waits for the tap, then rises (and widens with the walls)
  in 2 s. `motions` plays both for the `bath` tag; the water stays for the whole bath and drains as it ends.
- **Draw calls: 3.** The still body (plain finishes), the stream, the water.
- **Sizes against the placeholders.** The freestanding bath's filler stands beyond the tub's end (1.89 m
  wide, its placeholder's tub 1.80 m); the clawfoot's rolled rim makes it 0.83 m deep (0.78 m). Both fit
  their 2 × 1 tiles.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.bathtub` | 1.82 × 0.69 × 0.86 m | 2,144 | 3 | Tub, Panel, Taps, Towel (and a duck) |
| `model.bathtubClawfoot` | 1.73 × 1.02 × 0.83 m | 4,428 | 3 | Tub, Feet, Taps, Towel |
| `model.bathtubFreestanding` | 1.89 × 0.92 × 0.84 m | 2,248 | 3 | Tub, Taps, Towel |

`check_glb.py`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.

## B5 — bathroom storage and mirrors

`model.mirrorCabinet`, `model.mirrorRound` and `model.bathCabinetTall`, built 2026-10-14. They sit in
Validate › Bathroom Storage until checked in play. All three are still (no use opens them yet).

- **Wall things.** The mirror cabinet and the round mirror hang at their real height, their backs on the wall
  face: `G.settle(centre=False, floor=False)` and `floor=False` on their `Model`. `check_glb.py` warns that they
  float, which is right for a wall thing.
- **Mirror glass from the file** (E6): a flat piece of the `mirror` material (`gloss`, `'front'`). Only the High
  preset reflects live (`mirrors: 2`); on Medium and Low it's plain pale gloss, as the placeholders' mirrors. It
  was see-through `glass` at first, which showed the round mirror's brass backing and the cabinet's door.
- **Doors for later.** The mirror cabinet's mirrored door and the tall cabinet's upper door are parts of their own
  (`door`), hinged on their left edge, with no clip yet: a swing can be added once content gives them a use (the
  plan's "(+use)"). A still part costs no draw call of its own.
- The mirror cabinet is an open box with two shelves, a cup and bottles behind its door, and a light strip on top.
  The tall cabinet has an open niche between its doors with two folded towels.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.mirrorCabinet` | 0.56 × 0.77 × 0.20 m | 1,308 | 1 | Cabinet, Frame |
| `model.mirrorRound` | 0.64 × 0.77 × 0.05 m | 1,528 | 1 | Frame, Strap |
| `model.bathCabinetTall` | 0.42 × 1.82 × 0.38 m | 1,524 | 1 | Cabinet, Frame, Towels |

`check_glb.py`: 0 failed. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
python3 -B tools/art/blender/build.py bathroom            # every .glb, the pack, check_glb
python3 -B tools/art/blender/build.py bathroom --preview  # previews/sink.png, sink@water_on.png, …
```
