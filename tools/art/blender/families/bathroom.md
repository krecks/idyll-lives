# Bathroom — notes

`bathroom.py` builds the five bathroom sinks, the five toilets, the four showers, the three baths and the three storage and mirror models (below) into `web/public/assets/models/bathroom/` and their entries
in `packs/bathroom/manifest.json`: `model.sink` (`sink.glb`), `model.sinkPedestal` (`sinkPedestal.glb`),
`model.vanityDouble` (`vanityDouble.glb`), `model.sinkVessel` (`sinkVessel.glb`) and `model.sinkConsole`
(`sinkConsole.glb`). First made 2026-10-14. The pack is listed in the base manifest's `packs` and is the
only manifest that defines these keys.

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
- **Against the wall.** The sinks stand with their backs on the wall face (game depth -0.43): `centre=` on each
  `Model`, where `G.settle()` centres it (they stood centred on the tile, 15–22 cm off the wall, until
  2026-10-14). Their old parts (the base sink and the rest) reached 1–4 cm into
  the wall and were moved forward onto it.
- **Basins.** Each basin is one shell lofted through rings (`_lathe`, `_bowl_shell`): up the outside, over a
  rounded rim and down the inside to its floor, so the inside faces up into the bowl. The base sink's is an
  undermount steel basin under a worktop slab cut with a rounded opening. Its cabinet's hidden top stays 3 cm under
  the basin's floor, with painted "shoulders" round the basin up to the worktop: a face within 1 cm of the basin's
  floor made `cargo review` take the worktop for something standing on it (ll-tools `checkglb` test).
- **The detail pass (2026-10-22).** Every model was rebuilt with the forms (`G.sweep`, `G.loft`, `F.extrude`,
  `F.soft`, `G.kit.tube`): same keys, sizes, centres, heights, parts, pivots, clips, `motions`, paint and draws.
  The family's helpers: `_lathe` (a loft through game rings: squareness, an egg's narrowing, a deeper back, a ring
  moved forward), `_gooseneck` (a tap in one sweep: riser, half-circle arc, aerator, domed rose, side lever),
  `_pull` (a one-piece bar pull), `_shaker` + `_panel` (a frame with a hole round a panel set 6 mm back; the
  panel is a band and its middle, so the shading baked per vertex darkens only the band in the crease, not the
  whole face), `_bends` (a path's corners turned in round bends as points, swept as they are: Catmull-Rom through
  long runs beside short corner spans overshot by 12 cm), `_frame`, `_tray`, `_grate`, `_head`, `_hand_shower`,
  `_bar_mixer`, `_caddy`, `_curtain` (one wavy sheet), `_bottle`, `_cup`, `_drain`, `_rose`/`_wall_rose`.

## The models

| Model | Size (w × h × d) | Triangles | Draws | Note |
|---|---|---|---|---|
| `model.sink` | 0.86 × 1.26 × 0.62 m | 3,480 | 2 | shaker doors and a false drawer front with bar pulls, sides in one piece, a worktop slab with a rounded opening over an undermount steel basin and its plug, a gooseneck with a lever, a soap pump |
| `model.sinkPedestal` | 0.56 × 1.06 × 0.46 m | 2,724 | 2 | a waisted oval column flaring into its foot; one basin shell with a rolled rim, a flat tap deck at its back and the bowl set forward, an overflow slot, a plug; a mono tap, a soap on its dish, a tooth mug with a brush |
| `model.vanityDouble` | 1.90 × 1.00 × 0.57 m | 5,864 | 3 | two drawers over four shaker doors with bar pulls, a recessed plinth, a soft-edged worktop with an upstand, two vessel bowls (shells, rolled rims, plugs), two goosenecks (no mirrors); worktop 0.76, rims 0.88 |
| `model.sinkVessel` | 0.84 × 1.06 × 0.54 m | 4,264 | 3 | a drawer front with a black bar pull, a soft-edged top, a thin-walled vessel bowl, a tall black tap; the vanity on four tapered wooden legs with side rails and a slatted shelf, two towels folded in two layers each; worktop 0.76, rim 0.88 |
| `model.sinkConsole` | 0.60 × 1.06 × 0.49 m | 2,420 | 3 | one basin shell (a rounded slab, a flat deck, the bowl set forward), a brass frame each side bent from the floor up and back under it to the wall (one tube, a foot, a wall plate), a towel bar with collars and a towel hanging over it (one swept sheet), a brass gooseneck and plug |

`check-glb`: 0 failed; Khronos validator: 0 errors, 0 warnings. Each
lists the parts the player may colour in `paint`, the `body` first.

## The vessel bowls' rims at a basin's height

The double vanity's and the vessel sink's bowls stand on the worktop, so the worktop is lower than a counter:
0.76 m, the bowls 12 cm tall, their rims at 0.88 m as the pedestal and console basins' (a basin's rim is
0.76–0.91 m). Until 2026-10-21 the worktops were at 0.88 and 0.86 m and the rims at 1.00 and 1.02 m
(M-bathroom-02); the taps came down with them.

## The double vanity has no mirrors

It had two (frames in the file, the glass as primitive `parts`). They were taken off on 2026-10-14: the
player chooses the mirror they hang above it (the wall mirror, the mirror cabinet), as for any sink. Its
old part (`catalog.double_vanity`) has none either.

## B1 — the toilets

`model.toilet` (`toilet.glb`), `model.toiletCompact`, `model.toiletSmart`, `model.toiletClassic` and
`model.toiletWallHung`, built 2026-10-14. They sat in Validate › Toilets until checked in play.

- **Against the wall, not centred.** The game sits a resident at the footprint's middle plus the seat's
  `fwd` (characters.ts `SEATS`, else the content's `seat`), so each toilet is built where the game places it:
   the cistern (or the boxed-in cistern) on the wall face at game depth -0.43,
  `G.settle(centre=False)`, and `centre=` on its `Model`. The bowl's middle is 8–16 cm in front of the hips.
- **Seat heights.** The seat's top is the seat height: 0.43 m (`toilet`, `toiletCompact`), 0.44 m
  (`toiletSmart`), 0.445 m (`toiletWallHung`) and 0.45 m (`toiletClassic`). The wall-hung old part had its
  seat at 0.6 m; the model follows the content (0.45 m).
- **Shapes.** Egg-shaped from above (`EGG`, narrower to the front), as an elongated bowl is. The bowl is one
  shell (`_bowl`): up the outside to a rounded rim and down the inside to a water disc. A waisted foot rising into
  the bowl's underside (`_foot`) with a bolt cap each side, or a wall-hung bowl on a rounded porcelain block
  (`_column`) into the panel. The seat is a rounded section swept round the egg (`_seat`) on two buffers, the lid
  a domed loft, the hinges a knuckle on a post each. The cisterns are rounded lofts (`_cistern`) under a soft lid a
  little proud of them (`_cistern_lid`; the classic's stepped, with a band round it), the flush levers a boss and
  a swept arm (`_lever`). The roll (`_roll`) hangs on a one-piece holder with its loose end down the front.
- **The lid** is the one moving part (`lid`), hinged across its back edge, on its underside; `lid_up`
  swings it 100° about X in 0.6 s, to lean on the cistern (the hinge is 8.6–9.5 cm in front of it so the
  lid just meets it). The smart toilet's lid is level with its seat housing (the bidet unit) and hinged on
  its top edge, so it lifts over the housing; it rises slower, 0.8 s. Only the lid lifts: the resident sits
  on the seat. `motions` plays `lid_up` for the `toilet` animation tag until the use ends.
- **The classic toilet** is close-coupled with a wooden seat and lid, as its content says
  (not a high cistern and chain).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.toilet` | 0.44 × 0.81 × 0.68 m | 3,524 | 2 | Bowl, Seat & lid, Cistern |
| `model.toiletCompact` | 0.59 × 0.81 × 0.63 m | 3,828 | 2 | Bowl, Seat & lid, Cistern (a slim cistern, a dual flush button in a ring, a roll on the wall) |
| `model.toiletSmart` | 0.90 × 1.20 × 0.71 m | 3,980 | 2 | Bowl, Seat & lid, Cistern (the panel, with a shelf, a display with keys and a ready light, a remote in its holder and a roll; a seam on the housing) |
| `model.toiletClassic` | 0.47 × 0.88 × 0.71 m | 3,724 | 3 | Bowl, Seat & lid (wood), Cistern (a stepped lid, a band, a lever with a ball end) |
| `model.toiletWallHung` | 0.60 × 1.10 × 0.67 m | 3,224 | 2 | Bowl, Seat & lid, Cistern (the boxed-in panel with a stone top, a soft flush plate and buttons) |

The close-coupled ones (under 1 m) keep under the 4,000-triangle target: the seat's section has 12 points round 32
steps.

`check-glb`: 0 failed; its warnings are the two wall mirrors floating above their origins (wall things). The
compact and wall-hung toilets stood 2.5 and 5 cm into the wall; they stand on the wall face now (2026-10-14), the
classic one's moved back 2 cm onto it, and the wall-hung one's seat is at 0.45 m, not 0.6. Khronos validator: 0
errors, 0 warnings each.

## B2 — the showers

`model.shower`, `model.showerStall`, `model.showerWalkIn` and `model.showerRain`, built 2026-10-14. They sat in
Validate › Showers until checked in play.

- **They fill their tile**, their backs on the wall's face (game depth -0.43; `centre=` 2–3.5 cm forward of
  the middle, as the old parts), so they reached 2–4 cm into the wall until 2026-10-15. The rain shower is
  0.93 m deep (its tray and back wall 1 cm shallower), so its front is on the tile's edge. The resident steps in
  onto the middle (`inside`). Walls at the back and on the left (game -x), the open sides glass or a curtain.
- **The rain** is the one moving part (`water`): thin streams (`_rain`: a disc of rings, or a grid under the
  square head) built full length from the head's underside to the tray, joined with the origin on the head
  and scaled to 0.001 along their length, so they're hidden in the head at rest. `water_on` grows them down
  in 0.5 s and `motions` holds it for the whole use (the `shower` tag). The stall's head is on the wall, so
  its rain is turned about its origin (a fixed node rotation; the clip keys only the scale) to fall from the
  head onto the resident in the middle.
- **Draw calls: 3**, the base shower 4. The tray, walls and metal (1), the glass (1, see-through `glass`), the
  rain (1). The base shower's sliding door is its own part, a fourth (Q-05, owner 2026-10-21): all glass, its
  grip a bar of smoked glass (`grip`, a `glass` material) on the inside by its leading edge, since a chrome
  handle on it would be a fifth. The stall's curtain is plain satin (a polyester curtain), so its moving front
  costs no extra draw: body, curtain, rain.
- **The base shower's door** (`door_open`) slides 41 cm left behind the fixed panel in 0.7 s, opening the right
  half of the front (A-bathroom-01): open as the use starts, shut after 1.5 s (`hold`), open again for 1.5 s as
  the use ends (`out`); the water comes on 2 s in, once it's shut. The resident is moved in by the sim at once
  (`inside`), so they are already in when it opens and step out while it opens again (A-bathroom-05).
- **The stall's curtain** hangs along the whole open side (still) and across the front as its own part
  (`curtain`): built drawn across, one wavy sheet in 7 folds (`_curtain`) on 14 rings from the right corner, and
  gathered into that corner at rest
  (scaled to 0.32 along the rail about its origin there); `curtain_draw` grows it across in 0.9 s, 1 s into the
  use, held till the use ends (A-bathroom-02). The water comes on at 1.8 s.
- **Tiles** (`_tiled_walls`) are thin satin squares a little proud of a grout backing: the base shower 30 cm,
  the stall 25 cm. The walk-in shower's stone wall has grooved joints; the rain shower's walls are plain stone.
  The rain shower's front glass covers only the left half: the right half is the way in.
- **Details (2026-10-22).** Trays are one loft with a raised lip round a floor falling to the drain (`_tray`), the
  drains slotted grates (`_grate`). Frames are square-section sweeps bent round their corners (`_frame`). The base
  shower has a bar mixer, a hand shower on a slide rail with its hose looping down, a corner shelf with bottles and
  a domed head on a bent arm; the stall a bent L rail with wall roses, the side curtain a wavy sheet on 12 rings, a
  riser bent up into its head, a lever mixer on a round plate and a wire caddy; the walk-in a linear grate, a floor
  channel, a stay bar and a riser bent over into the arm in one tube; the rain shower a niche with two sills and
  bottles, a bent arm, a nozzle-lined head and a lever mixer.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.shower` | 0.90 × 2.15 × 0.90 m | 4,484 | 4 | Tray, Frame, Head & tap, Tiles |
| `model.showerStall` | 0.90 × 2.08 × 0.90 m | 4,892 | 3 | Tray, Rail, Head & tap, Curtain, Tiles |
| `model.showerWalkIn` | 0.92 × 2.26 × 0.92 m | 3,392 | 3 | Floor, Frame, Head & tap, Wall |
| `model.showerRain` | 0.94 × 2.18 × 0.93 m | 2,532 | 3 | Tray, Frame, Head & tap, Walls |

`check-glb`: 0 failed; the base shower warns of its 4 draws (allowed, Q-05). Khronos validator: 0 errors, 0 warnings each.
The clips bench (`/clips.html?keys=model.showerStall`) frames the stall from far off, before the detail pass too.

## B3 — the baths

`model.bathtub`, `model.bathtubClawfoot` and `model.bathtubFreestanding`, built 2026-10-14. They sat in
Validate › Baths until checked in play.

- **2 × 1 tiles, centred.** The resident sits along the tub, back at the end without the taps and legs out
  towards them (`SEATS` `along`, `legsOut`: 0.25, 0.28 and 0.27 m high, the hips 0.38–0.42 m from the middle);
  the tub floors are 0.15–0.17 m, so they sit in the water.
- **The tub** is one closed shell (`_loft`): superellipse rings up the outside, over the rim and down the inside
  to the floor, so the inside faces up into the tub; flat-shaded above 40°. The built-in bath is squarer
  (higher exponent) behind a gloss front and end panels over a dark plinth set back under them.
- **Two moving parts.** The tap's stream (`stream`, the sinks' `_water` way) grows in 0.4 s (`tap_on`, held
  2.4 s, so it runs while the tub fills). The water (`bathwater`) is the tub's inside from the floor up to two
  thirds of the depth, its origin just under the floor, scaled to nothing upright and in to the floor's size
  across: at rest it's hidden under the floor; `fill` waits for the tap, then rises (and widens with the walls)
  in 2 s. `motions` plays both for the `bath` tag; the water stays for the whole bath and drains as it ends.
- **Draw calls: 3.** The still body (plain finishes), the stream, the water.
- **Sizes against the old parts.** The freestanding bath's filler stands beyond the tub's end (1.89 m
  wide, the old part's tub 1.80 m); the clawfoot's rolled rim makes it 0.83 m deep (0.78 m). Both fit
  their 2 × 1 tiles.
- **Details (2026-10-22).** Each has a plug and an overflow plate (`_waste`). The built-in bath's front panel
  carries a moulded frame; its deck mixer a pillar with a collar, a spout bent down over the water and two lever
  handles on roses; a towel folded in two layers and a duck with a tail. The clawfoot tub has a rolled bead swept
  round its rim, ball-and-claw feet (a curved leg, a ball, three swept claws), a filler bent in one tube with
  cross handles on hubs and a telephone hand shower on a cradle, and a towel hanging over its back rim (one swept
  sheet). The freestanding bath's filler is bent in one tube from a floor rose, with a mixer collar and lever, a
  handset and its hose; its towel too hangs over the rim.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.bathtub` | 1.82 × 0.69 × 0.87 m | 3,660 | 3 | Tub, Panel, Taps, Towel (and a duck) |
| `model.bathtubClawfoot` | 1.73 × 1.02 × 0.83 m | 5,632 | 3 | Tub, Feet, Taps, Towel |
| `model.bathtubFreestanding` | 1.89 × 0.92 × 0.83 m | 2,552 | 3 | Tub, Taps, Towel |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.

## B5 — bathroom storage and mirrors

`model.mirrorCabinet`, `model.mirrorRound` and `model.bathCabinetTall`, built 2026-10-14. They sat in
Validate › Bathroom Storage until checked in play. The mirror cabinet's door opens while a resident brushes
their teeth there, and the tall cabinet's upper door while one takes a fresh towel; the round mirror is still.

- **Wall things.** The mirror cabinet and the round mirror hang at their real height, their backs on the wall
  face: `G.settle(centre=False, floor=False)` and `floor=False` on their `Model`. `check-glb` warns that they
  float, which is right for a wall thing.
- **Mirror glass from the file** (E6): a flat piece of the `mirror` material (`gloss`, `'front'`). Every preset
  reflects live: the nearest mirror on screen on Low and Medium, two on High (`mirrors: 2`). It was see-through
  `glass` at first, which showed the round mirror's brass backing and the cabinet's door.
- **Doors.** The mirror cabinet's mirrored door and the tall cabinet's upper door are parts of their own (`door`),
  hinged on their left edge. The mirror cabinet's swings open 100° (`door_open`, 0.8 s, the reflection with it)
  for the content's "Brush your teeth" (`brushTeeth`, `wash`, 4 minutes, hygiene; not autonomous, so generated
  houses play as before), held 1.5 s. The tall cabinet's swings open the same way onto its towels for "Take a
  fresh towel" (`takeTowel`, `fetch`, 2 minutes, comfort; not autonomous), held 1.5 s: a second draw call.
- **Against the wall.** The tall cabinet stood 2 cm into the wall, as its old part did; both stand on the wall
  face now.
- The mirror cabinet is an open box with two lipped shelves, hinges, a cup of toothbrushes, bottles (one a pump), a
  jar with its lid and a tube of paste behind its soft-edged door (a bar pull, hinge plates, the flat mirror), and a
  soft light bar on a bent arm from the wall. The round mirror's frame is a moulded section (flat behind, rounded to
  the front) swept round the circle, its leather strap one swept band riveted to it and over a bent brass hook on a
  wall rose. The tall cabinet has shaker doors with bar pulls and hinges, an open niche between them with two rolled
  towels, and behind its upper door a stack of three soft folded towels, a shelf with two more and two bottles.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.mirrorCabinet` | 0.56 × 0.77 × 0.20 m | 2,932 | 2 | Cabinet, Frame |
| `model.mirrorRound` | 0.64 × 0.78 × 0.05 m | 1,740 | 1 | Frame, Strap |
| `model.bathCabinetTall` | 0.42 × 1.82 × 0.38 m | 4,352 | 2 | Cabinet, Frame, Towels |

`check-glb`: 0 failed. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art bathroom            # every .glb, the pack, check-glb
cargo art bathroom --preview --lit  # previews/bathroom/sink.png, sink.lit.png, sink@water_on.png, …
```
