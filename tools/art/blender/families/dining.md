# Dining — notes

`dining.py` builds the kitchen's **K7 dining tables**, **K8 chairs and stools** and **K9 dining storage** into
`web/public/assets/models/dining/` and their entries in `packs/dining/manifest.json` (listed in the base
manifest's `packs`, and the only manifest that defines these keys).

K7 (2026-10-14): the dining table with its two chairs (`model.diningTable`), the bistro set
(`model.bistroSet`), the glass and farmhouse dining sets (`model.diningGlass`, `model.diningOak`) and the second
catalogue's chair-less tables: `tableSquare2`, `tableBistroRound`, `tableDining4`, `tableDining6`,
`tableBanquet8`, `tableRound4`, `tableSquare8`, `tableFarmhouse6`, `tableBarHigh`.

K8 (2026-10-14): the bar stool (`model.barStool`) and dining chair (`model.diningChair`), and the second
catalogue's `stoolCounter`, `stoolBarHigh`, `chairWood`, `chairUpholstered`, `chairModern`, `chairWindsor`,
`chairBentwood`, `chairFolding`, `chairArm` and `benchDining`.

K9 (2026-10-14): catalog2's `sideboard`, `buffet`, `chinaCabinet` and `barCart`.

## Facts kept

- **Table tops** at the content's `surface.height`: 0.75 for catalog2's tables (`TT`) and the bistro set, 1.06 for
  the high café table (`BT`), 0.76 for the three sets for two. Tabletop things bought onto `tableDining4` stand on
  its top (seen in play). The planked tops (banquet, farmhouse, the oak set, the bench) are separate planks meeting
  in a shallow rounded joint, their tops at the height; no joint lies on the middle line (the gate's probe), and the
  oak set's breadboard pegs stand 0.5 mm proud.
- **Seats** at `characters.ts` SEATS or the content's `seat`: the seat's (hard) top at that height, the pad or
  cushion a little above, as the old parts: the sets' chairs 0.46 (at the content's `places`, `fwd` 0), the bar
  stool 0.74 (sitters at 0.72), the counter stool 0.685 (0.68), the high stool 0.805 (0.8), the
  chairs 0.46–0.48, the carver chair's board 0.425 under a cushion to 0.475 (0.47, a dining chair's height: G-30),
  the bench 0.465 (0.47, two slots). catalog2's seats are centred 5 cm forward (`SZ`), as the old parts.
- **The sets for two** (`diningTable`, `diningGlass`, `diningOak`; G-39, 2026-10-21) seat their two across the top,
  facing each other: a chair at each end (`SET_X` ±0.62, `SET_Z` 0.1, the content's `places` `[±0.62, 0.1]` facing
  ±90), the top 0.84 × 0.75 (the dining table) or 0.9 × 0.8 m (glass, farmhouse) centred at depth 0.1, a place
  setting before each. A chair is built facing the front and turned a quarter (`_end_chair`). The glass set's sleds
  run along its sides, leaving the ends open for the knees. The chairs are part of the table (as the old parts drew
  them): residents sit on them.
- **Storage** stands where the game places it against the wall (`centre=` the old part's middle, then
  `G.settle()`), so the sideboard's and buffet's tops (0.8, 0.9) are where the content's `surface` is.

## Materials

Each part the player colours has a material of its own, even where two share a colour (`top_oak`, `legs_oak`).
Tops and legs are `wood` (or `gloss`/`satin` white, `satin` black, `glass`), frames `metal`/`chrome`, pads
`fabric`; what stands on the sets (plates, food, glasses, a vase of flowers, a cup) is plain `gloss`/`satin`, so
it costs no draw call. The bar cart's trays, bottles and wine glasses are `glass` (one group).

## Clips

- **Stools** (`barStool`, `stoolCounter`, `stoolBarHigh`): the seat (with the high stool's back) is a part of its
  own on a swivel plate, pivoting about its middle. `swivel` turns it 20° in 0.5 s for `sit` and `drink`; `hold`
  0.6 s, then it turns back. `ride: swivel` carries the sitter round with it, as the office chairs do. The legs,
  foot ring and hub are the still part. The wood stools' feet and swivel plate are in the frame's wood, so a
  stool draws in 3 groups (base wood; seat wood and fabric).
- **Bar cart**: the cocktail shaker is a part of its own pivoting at its foot, in the middle of the top tray's
  front edge, before the resident who mixes from the front tile. `shake` lifts it 10 cm and tips it 30° towards
  them (one clip, both channels) in 0.6 s for `mix`; `hold` 1.5 s.
- **Storage doors:** the sideboard's four fluted doors, the buffet's five panelled doors and the china cabinet's two
  glazed doors are parts of their own, hinged on their outer edges. All three are `cupboard` stations (content
  `stations`): clean things are put away in the nearest cupboard or counter (cooking `putAwayAt`), and one door
  swings open 100° in 0.9 s while they are (`door_open`, `fetch`, `hold` 1.5 s): the sideboard's second, the
  buffet's wide middle one, the china cabinet's left glazed one. One door each, as each opening door costs draws of
  its own (the sideboard's walnut and fluting two). Behind the sideboard's and buffet's is a real cupboard
  (`_cabinet(hollow=…)`) with a shelf and stacks of plates and bowls (`_crockery`); the china cabinet's opens onto
  its plates. The other doors are still (no draw call of their own). The china cabinet's base doors and the
  buffet's drawers are in the body.

## Detail pass (2026-10-22)

Every model rebuilt with the forms (`gamekit/forms.py`) for more detail, keeping its key, size, centre, parts,
clips, `motions`, `paint` and draw calls, its tops and seats at their heights. The family's own `_tube` (and the
rods joined by spheres, the tori bent into loops) is gone: bent rails and bentwood are `G.kit.tube` through
`_fillet`'s corners (`_rail`), closed loops (foot rings, sleds, hoops, galleries) a closed `F.sweep` (`_loop`).
The family's helpers:

- `_slab`, `_round_slab`: a top or seat board lofted round with a moulded edge (a rounded nose over a chamfer);
  planked tops are several, meeting in rounded joints (5 on the banquet table and the farmhouse set, 3 on the
  farmhouse table and the bench), never one on the middle line the gate probes for the surface or seat.
- `_turned`: a revolve with crisp edges (legs, finials, columns, feet, plates, cups, vases, bottles, knobs).
- `_sq_leg`: a square leg tapering, its corners rounded, a small chamfer at the floor, splayed (`at_top`).
- `_apron`: an apron rail arched between the legs (`F.extrude`). `_crest`: a back rail bowed round the sitter.
  `_stretcher`: a turned stretcher swelling in the middle.
- `_surface`: a curved panel through a grid of points, thickened (Solidify, applied): the moulded chair's shell in
  one piece, the Windsor's carved saddle seat.
- `_finish` cuts the feet flat on the floor: a leg swept up from the floor at a slant dips a few mm below it,
  and `G.settle` would otherwise lift the whole model (the shipped stools, Windsor, bentwood and folding chairs were
  2-4 mm high from their slanted rods; now every seat top is where the code puts it).

## K7 — dining tables

| Model | Size (w × h × d) | Triangles | Draws | Notes |
|---|---|---|---|---|
| `model.diningTable` | 1.69 × 1.02 × 0.76 m | 7,310 | 3 | oak top with a moulded edge to 0.76 on turned legs, arched aprons; a runner hanging over the edges; two place settings (rimmed plates with food, a knife, a fork on a folded napkin, a glass); a vase of five tulips with leaves; a ladder-back chair at each end (stuffed pads, bowed rails, bent back legs, arched aprons, turned stretchers) |
| `model.bistroSet` | 0.62 × 1.00 × 0.91 m | 3,858 | 3 | round oak top with a moulded edge on a black column with a ring and a domed cast foot; cup and saucer turned, a looped handle; three tulips; a café chair (stuffed round seat on a ring, legs bowed out with feet, ring stretcher, two hoops tied together) |
| `model.diningGlass` | 1.69 × 1.07 × 0.80 m | 6,054 | 2 | glass top with a polished edge on two chrome sleds, each bent from one tube, on glides; dark chairs bent from tube (back legs over the top in one, front legs into the seat rails), flat ladder bars |
| `model.diningOak` | 1.69 × 1.07 × 0.81 m | 7,630 | 3 | thick top of five planks between pegged breadboard ends, square tapered legs, arched aprons; runner, settings, vase, a chair at each end |
| `model.tableSquare2` | 0.82 × 0.75 × 0.82 m | 2,060 | 1 | moulded edge, turned legs with collars, arched aprons |
| `model.tableBistroRound` | 0.72 × 0.75 × 0.72 m | 1,776 | 1 | white gloss top inside a black edge band, column with a ring, domed cast foot |
| `model.tableDining4` | 1.84 × 0.75 × 0.88 m | 1,036 | 1 | moulded edge, square tapered legs, arched aprons, a drawer at each end with a turned knob |
| `model.tableDining6` | 2.84 × 0.75 × 0.88 m | 2,752 | 1 | walnut, six turned legs with collars, arched end aprons and a middle one |
| `model.tableBanquet8` | 3.84 × 0.75 × 0.88 m | 2,204 | 1 | five planks, six chamfered legs on block feet, H-stretcher with tenons and wedges through the end legs |
| `model.tableRound4` | 1.52 × 0.75 × 1.52 m | 3,088 | 1 | birch, moulded edge, turned skirt and column, four feet curving out to turned toes |
| `model.tableSquare8` | 1.86 × 0.75 × 1.86 m | 1,196 | 1 | white top, square-tube legs near the corners, a steel frame under the top and one low down, levelling feet |
| `model.tableFarmhouse6` | 2.86 × 0.75 × 0.88 m | 2,184 | 1 | three planks; trestles: chamfered posts, feet arched underneath, shaped bearers, braces up and down, a stretcher wedged through the posts |
| `model.tableBarHigh` | 0.70 × 1.06 × 0.70 m | 2,004 | 1 | moulded top, column with a ring, domed foot, foot ring on four spokes from a hub |

## K8 — chairs and stools

| Model | Size (w × h × d) | Triangles | Draws | Clips (uses) |
|---|---|---|---|---|
| `model.barStool` | 0.39 × 0.74 × 0.39 m | 3,376 | 3 | `swivel` 0.5 s (sit, drink; `hold` 0.6 s; ride): turned legs in shoes, foot ring (the `rest`), hub; seat board with a rounded edge, stuffed buttoned cushion with a piped seam |
| `model.stoolCounter` | 0.39 × 0.69 × 0.39 m | 3,376 | 3 | `swivel` (as the bar stool) |
| `model.stoolBarHigh` | 0.39 × 1.12 × 0.40 m | 3,048 | 2 | `swivel` (seat and back turn): tube legs in rubber shoes, kick plate, bent back posts, curved back board |
| `model.diningChair` | 0.44 × 1.00 × 0.47 m | 3,636 | 2 | static: turned front legs and back posts with finials, three bowed slats, piped tie-on pad (ties knotted round the posts), arched aprons, turned stretchers |
| `model.chairWood` | 0.44 × 0.90 × 0.45 m | 2,360 | 1 | static: square tapered front legs, back legs bending back into the posts, bowed rails, swelled spindles, arched aprons |
| `model.chairUpholstered` | 0.46 × 0.99 × 0.49 m | 1,968 | 2 | static: stuffed, piped seat and back, three buttons, tapered front legs, sabre back legs |
| `model.chairModern` | 0.44 × 0.79 × 0.50 m | 1,420 | 1 | static: one moulded shell (lip, seat, back wrapping round), two sleds each bent from one tube |
| `model.chairWindsor` | 0.47 × 0.97 × 0.47 m | 2,588 | 1 | static: carved saddle seat, baluster-turned splayed legs, turned H-stretcher, tapering bow, seven spindles |
| `model.chairBentwood` | 0.43 × 0.96 × 0.45 m | 2,676 | 2 | static: back legs and loop bent from one length, inner loop tied to it, seat ring, cane, leg ring, knee braces |
| `model.chairFolding` | 0.46 × 0.89 × 0.48 m | 1,948 | 1 | static: two bent tube frames crossing at rivets, seat pan and pad, curved back, rubber feet |
| `model.chairArm` | 0.57 × 0.92 × 0.47 m | 3,384 | 2 | static: arms sweeping forward and rolling down into their posts, bowed crest rail, piped cushion, padded back |
| `model.benchDining` | 1.60 × 0.47 × 0.36 m | 1,064 | 1 | static: three planks, splayed square legs through chamfered cleats, arched rail, pegged stretchers |

## K9 — dining storage

| Model | Size (w × h × d) | Triangles | Draws | Clips (uses) |
|---|---|---|---|---|
| `model.sideboard` | 1.82 × 1.10 × 0.66 m | 5,960 | 4 | `door_open` (putting clean things away, `fetch`): half-round reeds on the doors, mushroom knobs, turned legs in brass shoes, moulded top, a bead round the foot; a vase and a bowl of lemons |
| `model.buffet` | 1.88 × 0.90 × 0.67 m | 5,500 | 3 | `door_open` (putting clean things away, `fetch`): raised-field doors and drawers, brass bar pulls and knobs, moulded oak top, arched apron, turned legs |
| `model.chinaCabinet` | 0.98 × 2.10 × 0.55 m | 6,704 | 4 | `door_open` (putting clean things away, `fetch`): shaped plinth, raised panels, moulded worktop, cornice (a moulded cap over a cove swept round), glazing bars, rimmed plates leaning on the rails, cups |
| `model.barCart` | 0.77 × 1.09 × 0.45 m | 6,404 | 3 | `shake` 0.6 s (mix, `hold` 1.5 s): glass trays in bent brass rims, a gallery round the lower tier, finials, castors with forks, a handle bent in one piece, labelled bottles with corks, wine glasses, ice bucket with ice, basket with a handle |

The crockery in the sideboard's and buffet's cupboards is plain discs and cones under one rimmed plate and one
open bowl per stack: it shows only while the door is open.

`check-glb`: 29 models, 0 failed; two warnings, the sideboard's and the china cabinet's 4 draw calls (as before).
Khronos validator: 0 errors, 0 warnings each. Seen in the gallery (WebGPU and WebGL2) and the clips bench (WebGPU
and WebGL2: the stools' swivel, the doors, the shaker) on 2026-10-22.

## Rebuild

```
cargo art dining            # the .glbs, the pack, check-glb
cargo art dining --preview --lit  # previews/dining/<id>.png, <id>.lit.png, <id>@<clip>.png
```
