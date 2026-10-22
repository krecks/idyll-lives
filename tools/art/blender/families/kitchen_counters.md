# Kitchen counters — notes

`kitchen_counters.py` builds the kitchen's **K4 base counters**, **K5 tall and wall storage** and **K6 islands and bars** into
`web/public/assets/models/kitchen_counters/` and their entries in `packs/kitchen_counters/manifest.json`.

K4 (2026-10-14): the plain counter (`model.counter`), the corner counter (`model.counterCorner`), the
counter end with shelves (`model.counterEnd`), the outer corner counter (`model.counterCornerOuter`), the
drawer counter (`model.counterDrawers`), the open-shelf counter (`model.counterOpenShelf`), the butcher
block (`model.butcherBlock`) and the marble counter (`model.counterMarble`).

K5 (2026-10-14): the pantry cupboard (`model.pantry`), the tall larder (`model.pantryTall`), the wall
cabinets (`model.wallCabinet`, `model.wallCabinetGlass`, `model.wallCabinetCorner`), the cooker hood
(`model.rangeHood`), the spice rack (`model.spiceRack`), the pot rail (`model.potRail`), the kitchen wall
shelves (`model.kitchenShelfWall`), the pedal bin (`model.trashCan`) and the recycling bins
(`model.recyclingBins`).

K6 (2026-10-14): the kitchen island (`model.kitchenIsland`), the small island (`model.islandSmall`), the long
island (`model.islandLong`), the bar counter (`model.barSegment`) and the bar end (`model.barEnd`).

## Materials

| Material | Finish | Used for |
|---|---|---|
| `cab` | satin | a cabinet carcass and its fronts; the player's colour (`paint`) |
| `worktop` | gloss | a laminate worktop |
| `dark` | satin | plinths |
| `chrome` | chrome | bar handles and pulls |
| `oak` | wood | a wood end unit, open shelves, a utensil handle |
| `navy` | satin | the marble counter's painted carcass |
| `marble` | gloss | the marble worktop |
| `vein` | gloss | a vein through the marble |
| `brass` | metal | the marble counter's handles, the larder's |
| `block`, `block2` | wood | the butcher block's top and its top layer |
| `leg` | wood | the block's legs and shelf |
| `wood2` | satin | the block's knife-drawer front (plain, so it draws with the body) |
| `basket` | satin | the block's basket, a larder basket |
| `metal` | metal | the block's dark drawer handle, the pot rail |
| `porcelain` | gloss | plates, a bowl, a jar, a box |
| `steel` | metal | the cooker hood, the pedal bin, cutlery, knife blades, the ladles |
| `trim` | satin | the hood's front trim |
| `bulb` | gloss | a strip light (under a wall cabinet, in the hood) |
| `copper` | metal | the pot rail's pans and pot |
| `glass` | glass | a glazed door, a jar; the wall cabinet's glazed doors |
| `back` | satin | the glazed cabinet's pale back panel |
| `leaf` | satin | a pot plant's foliage |
| `oakd` | wood | the brackets under the kitchen wall shelves |
| `spoon` | satin | pale wood that draws with the plain group: a drawer's cutlery tray, the crock's spoon and spatula, lids |
| `towel` | satin | the tea towel on the outer corner counter's rail |
| `bin1`, `bin2`, `bin3` | satin | the recycling bins (blue, green, yellow) |
| `lid` | satin | a bin's dark lid and handle |
| `soil` | matte | a plant pot's soil |

A transparent finish (`glass`) and a textured finish (`wood`, `foliage`) are each their own draw group.
The glazed cabinet keeps its carcass and doors `cab` and only the panes `glass`, so it draws in two
groups; the still bodies of the textured things (oak shelves, a basket) are kept to one group each.

## Clips

Every moving part is its own object with its origin on the hinge (or the front face for a slide); each
clip is one NLA track named for the pack's `motions`. Frame 1 is the rest pose. A door that a resident
opens swings out 100° (`DOOR_SECONDS` 1.6 s, `DOOR_HOLD` 2.5 s); a drawer slides out 25 cm
(`SECONDS` 1.4 s, `HOLD` 1.5 s). `G.settle()` runs before `G.swing()`/`G.slide()`, since a clip keys the
rest pose where it is.

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit (`G.sweep`, `G.loft`/`G.ring`, `F.extrude` with holes, `F.soft`, `G.kit.tube`),
keeping every key, size, centre, footprint, worktop height, node, hinge, clip, `motions`, `paint` and draw count. The
shared parts, in the script:

- `_worktop`: a worktop as a side-plane extrude, its front edge rolled over (postformed) and its underside eased, its
  ends cut square so the tops of a run meet without a seam; drawn through `COUNTER66` like the old box, the same
  depth (-0.43..0.23 as drawn) and height (0.90 or 0.91 to 0.95).
- `_shaker`: a shaker door or drawer front: a frame (an outline with a hole, its edges eased) round a flat panel, a
  middle rail on the larders' tall doors. The panel is a margin ring under the frame and a field whose corners are
  clear of it: the baked shading is per vertex, and a panel whose four corners hide under the frame went dark all over.
- `_pull`: a bar pull swept in one piece (legs bending into the bar); its `reach` is set so it never stands past the
  old front-most point (the worktop's nose on a counter), so no bounding box, centre or wall gap moved.
- `_drawer`: the shaker drawer front, its box, and a cutlery tray (spoons, forks, knives, a serving spoon as flat
  outlines) in the front of the box, which shows when it slides out; the butcher block's drawer holds a knife insert.
- `_lathe` and what is turned with it: stacked plates, bowls, mugs with handles, jugs, canisters with lids, cans, a
  crock, pans, a saucepan, ladles, jars (glass with what is in them); `_basket`, a ribbed, hollow woven basket.

Doors on the 1 m counters are now 4 mm apart and line up with the drawer above (they were 3 cm apart). A counter's
fronts (2 cm behind the worktop's nose) and plinth line up with its neighbours' in a run: checked by importing the
exports side by side (counter, marble, drawer counter; open shelf and counter end; three bars).

What each model has now:

- **counter, counterCorner, counterMarble**: the postformed worktop, two shaker doors and a shaker drawer with swept
  pulls (brass on the marble), the drawer's cutlery tray; the corner counter's stoneware crock holds a wooden spoon, a
  slotted spatula and a wire whisk; the marble top has three veins swept as thin ribbons.
- **counterEnd**: a shaker door, the worktop, the oak end unit's shelves lipped, a jug, a lidded canister and two cans.
- **counterCornerOuter**: a shaker door, a towel rail on the end panel with a red tea towel over it, a plate, two
  lemons and a leaf.
- **counterDrawers**: four shaker drawer fronts with long pulls, the top one with its tray.
- **counterOpenShelf**: lipped oak shelves, stacked plates, nested bowls, a sage jug, a teal casserole with its lid
  and handles, two serving dishes.
- **butcherBlock**: twelve staves across the top, rounded legs, aprons round the drawer, stretchers under the slatted
  shelf, a woven basket and a crock on it, the knife drawer's insert of four knives.
- **pantry, pantryTall**: shaker doors with a middle rail and long pulls, a moulded cornice, lipped shelves stocked
  with cans, cereal boxes, packets, canisters, bowls, a casserole and a platter; woven baskets (on top, on the
  floor); the tall larder's two shaker drawers.
- **wallCabinet**: shaker doors, a stack of plates, nested bowls, four mugs with handles, a strip light in a housing.
- **wallCabinetGlass**: glazed doors in one frame each with a glazing bar, tumblers, wine glasses, a glass carafe,
  plates and bowls.
- **wallCabinetCorner**: a shaker door, open oak shelves rounded at their outer corner, mugs, plates, a potted plant.
- **rangeHood**: a canopy sloping up to a two-piece chimney, keys and a display on its trim, a slotted grease
  filter and two lamps under it.
- **spiceRack**: an arched board with a hanging hole, brass rails on posts in one piece, lofted jars with lids.
- **potRail**: a rail with ball ends on arms from wall plates, S-hooks, two frying pans, a lidded saucepan on a bail
  handle, two ladles.
- **kitchenShelfWall**: curved corbels under both shelves, glass jars, plates, a canister, a mug, a pilea in a pot
  with stems and two trailing vines.
- **trashCan**: a tapering drum with a rolled bead, a plastic foot, a rounded pedal on its hinge, a domed lid with a
  lip, the inner bucket seen when it lifts.
- **recyclingBins**: tapering, hollow tubs with a rolled rim, a dark foot, the recycling triangle on the front, a
  hand grip; lids with a front lip and hinge knuckles.
- **kitchenIsland, islandSmall, islandLong**: a worktop rounded in plan with eased edges, shaker doors (and the
  kitchen island's fixed drawer fronts either side of the one that opens), framed end panels, applied panels on the
  stools' side, corbels under the overhang.
- **barSegment, barEnd**: a top with a rolled nose and square ends, rounded staves between a top rail and a kick
  board, a brass foot rail on curved brackets; the end's waterfall side with an eased edge.

## K4 — base counters

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.counter` | 1.000 × 0.950 × 0.660 m | 2,428 (was 1,404) | 2 | top drawer | `drawer_open` 1.4 s (sandwich, `hold` 1.5 s) |
| `model.counterCorner` | 1.000 × 1.261 × 0.930 m | 3,540 (was 1,520) | 2 | top drawer | `drawer_open` (sandwich, `hold` 1.5 s) |
| `model.counterEnd` | 1.000 × 0.950 × 0.660 m | 2,608 (was 1,320) | 2 | — | static (one worktop over the unit and the open end, no seam: G-41) |
| `model.counterCornerOuter` | 0.810 × 1.035 × 0.660 m | 1,784 (was 972) | 1 | — | static |
| `model.counterDrawers` | 1.000 × 0.950 × 0.660 m | 2,900 (was 1,620) | 2 | top drawer | `drawer_open` (putting clean dishes away: `fetch` at its prep station, `hold` 1.5 s) |
| `model.counterOpenShelf` | 1.000 × 0.950 × 0.660 m | 3,888 (was 1,560) | 2 | — | static |
| `model.butcherBlock` | 0.900 × 0.950 × 0.820 m | 3,520 (was 1,944) | 3 | knife drawer | `drawer_open` (sandwich, `hold` 1.5 s) |
| `model.counterMarble` | 1.000 × 0.951 × 0.660 m | 3,040 (was 1,620) | 2 | top drawer | `drawer_open` (sandwich, `hold` 1.5 s) |

## K5 — tall and wall storage

A wall thing sits on the wall face (`WALL = -0.43`) at its real height: it is built at that height and
`settle(floor=False)` / `check_size(floor=False)` leave it alone, so `check-glb` reports it as floating,
which is fine on a wall. A floor thing stands where the game places it, its back on the wall (below).

The wall cabinets are `cupboard` stations (content `stations`): clean things are put away in the nearest cupboard or
counter, a wall cabinet over a counter before the counter (cooking `putAwayAt`), and its doors swing open 100° in
0.8 s while they are (`fetch`, `hold` 1.5 s), onto plates, bowls and mugs. The glazed cabinet opens only its right
door, as each glazed door is two draw calls (frame and glass). The recycling bins' three lids lift one after another
(0.6 s apart, each 80° in 0.5 s, held 0.9 s) for the content's "Sort the recycling" (`sortRecycling`, `clean`, 4
minutes, comfort, chores; not autonomous, so generated houses play as before).

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.pantry` | 0.940 × 2.268 × 0.630 m | 6,860 (was 2,592) | 3 | two doors | `doors_open` 1.6 s (snack, `hold` 2.5 s) |
| `model.pantryTall` | 0.960 × 2.388 × 0.700 m | 7,328 (was 3,024) | 3 | two doors | `doors_open` (snack, `hold` 2.5 s) |
| `model.wallCabinet` | 0.980 × 0.732 × 0.380 m | 3,752 (was 1,880) | 3 | two doors | `doors_open` 0.8 s (putting clean things away: `fetch` at its cupboard station, `hold` 1.5 s) |
| `model.wallCabinetGlass` | 0.960 × 0.720 × 0.408 m | 3,640 (was 2,460) | 4 | the right glass door | `door_open` 0.8 s (as the wall cabinet) |
| `model.wallCabinetCorner` | 0.960 × 0.723 × 0.408 m | 3,776 (was 1,832) | 3 | the door | `door_open` 0.8 s (as the wall cabinet) |
| `model.rangeHood` | 0.900 × 0.712 × 0.515 m | 1,184 (was 648) | 1 | — | static |
| `model.spiceRack` | 0.520 × 0.440 × 0.105 m | 3,744 (was 2,268) | 3 | — | static |
| `model.potRail` | 0.908 × 0.546 × 0.220 m | 3,372 (was 1,236) | 1 | — | static |
| `model.kitchenShelfWall` | 0.900 × 0.757 × 0.240 m | 3,456 (was 1,476) | 3 | — | static |
| `model.trashCan` | 0.310 × 0.685 × 0.365 m | 1,692 (was 540) | 2 | lid | `lid_open` (scraping a plate in: `clean` at its bin station, `hold` 1 s) |
| `model.recyclingBins` | 0.920 × 0.660 × 0.455 m | 2,910 (was 972) | 4 | three lids | `lid0_open`, `lid1_open`, `lid2_open` 0.5 s, 0.6 s apart (sortRecycling, `hold` 0.9 s) |

## K6 — islands and bars

The kitchen island has its cupboards and drawer at the front, where residents stand to use it, and its
worktop overhangs the back, where housegen sets a used table's stools; it has no stools of its own (stools
are things of their own, which residents sit on at the island) and, as the other islands, a bare top
(housegen sets a fruit bowl, a cake stand or mugs on it). The small and long islands are turned the same way,
cupboards at the front and the worktop overhanging the back 30 cm, its edge on the wall face;
the bars have a slatted guest side, a top at 1.06 and a brass foot rail. The kitchen island's drawer
is on the stools' side, between them, since residents stand at the front (+Z) to use it: it slides out
towards them for a sandwich (a fixed drawer front on the cupboard side matches it); everything else is
static this pass.

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.kitchenIsland` | 1.940 × 0.950 × 0.840 m | 4,612 (was 1,404) | 2 | a drawer | `drawer_open` 1.4 s (sandwich, `hold` 1.5 s) |
| `model.islandSmall` | 0.960 × 0.950 × 0.912 m | 2,480 (was 972) | 1 | — | static |
| `model.islandLong` | 2.960 × 0.950 × 0.912 m | 5,180 (was 1,836) | 1 | — | static |
| `model.barSegment` | 1.000 × 1.060 × 0.890 m | 2,316 (was 1,656) | 2 | — | static |
| `model.barEnd` | 1.000 × 1.060 × 0.890 m | 2,180 (was 1,656) | 2 | — | static |

Also from the review: the butcher block's knife drawer has its box; the wall cabinet's pulls sit on its
doors; the pot rail's brackets start at the wall face (0.22 m deep); the recycling bins' colour parts are
the left, middle and right bin (each with its lid) and the handles.

`check-glb`: 0 failed; its warnings are the wall cabinets, the range hood, the spice rack, the pot rail and the
wall shelf floating above their origins (wall things), and 4 draw calls on the glass wall cabinet and the recycling
bins. Khronos glTF validator: 0 errors, 0 warnings each. The eight K4 counters sat in Validate › Counters, the
eleven K5 things in
Validate › Kitchen Storage and the five K6 islands and bars in Validate › Islands & Bars, until
checked in play.

## Where it stands (review, 2026-10-14)

Each floor model is centred where the game places it (its `Model`'s `centre`, which `G.settle()` uses),
not on its tile: its back is on the wall face (-0.43), as the old part's, a run of counters and
appliances lines up, and its top is under its content `surface`. Since the
2026-10-14 fix-ups the old parts no longer reach into the wall: both moved forward onto the face.
There is one counter depth, the catalogue's 0.66 m (worktop -0.43..0.23): the catalog2 counters keep
the old 0.79-deep numbers and are drawn through `COUNTER66` (gamekit `GameFrame(depths=)`,
the old `shallow`). Islands stand free, centred.

## Rebuild

```
cargo art kitchen_counters            # the .glbs, the pack, check-glb
cargo art kitchen_counters --preview  # previews/kitchen_counters/<id>.png, <id>@<clip>.png
```
