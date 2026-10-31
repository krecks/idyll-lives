# Outdoor (`families/outdoor.py`)

The outdoor things. One pack, `packs/outdoor/manifest.json`. Each model is built in place where the
old part stood (`F`, game axes), so seat heights (content `seat`) and table tops (content `surface`)
stay where the game has them; models that stand off the middle of their footprint say so with `centre=`.

2026-10-22 (detail pass, with gamekit's forms and kit; the same sizes, heights, parts, clips, draws and paint): cast
ends and bent tubes in one piece, shaped and rounded boards, soft piped cushions, lofted cups, jugs, bottles and a
basket, a parasol canopy sagging between its ribs, a mosaic of proud tiles, extruded stones and wedges. The common
pieces (chairs, benches, tables, the parasol) keep to about half their size class's target: the town overview draws
every lot's outdoor objects whole, and these have no far version. Helpers: `_bent` (kit.fillet + kit.tube), `_bar` (a
rounded-rect section swept), `_plank` (an outline placed like a board), `_side_slab`, `_slab`, `_cup`, `_jug`,
`_bottle`, `_mosaic`, `_cast_end`, `_flame`, `_log`, and `_jitter`: G.jitter's `noise_vector` gives other numbers in
each Blender process, so a jittered model came out a little different on each run (the fire pit's centre moved 2 mm
and failed the size check); `_jitter` uses `noise()`, which doesn't.

2026-10-21 (authenticity review, Phase 4): the park bench's and the picnic table's heights, the bistro set's
chairs, the garden bench, the hammock, the porch swing's seat, the swing set's frame, the hot tub's cover and bench
and the grill's height and glow were rebuilt to real sizes (`docs/review/outdoor.md`, `systems.md` M-outdoor-01).

What each has since the detail pass (2026-10-22):

| Model | Now |
|---|---|
| bench | each cast end three one-piece bars (the back leg curling back over the top, the front leg running up into the arm, the seat rail), a scroll, an arched stretcher, an armrest, pad feet; carriage bolts |
| patioSet | each chair's frame two bent tubes (over the top; under the seat's front), a tube round the seat, bowed back slats, seat slats with rounded ends, pivots; the table's framed top on two crossing U legs; a turned cup with a handle, a jug with a spout |
| picnicTable | legs, bearers and cleats cut level and chamfered (side outlines), carriage bolts, the gingham laid across and hanging over both edges, a lofted wicker basket with a rim, lid flaps, a bent handle and a strap |
| gardenChair, adirondackChair | arms tapering to round fronts, curved corbels, stringers rounded at the front, slats with rounded tops (11 cm seat slats), the garden chair's arched top rail, the Adirondack's bowed upper rail; a turned cup |
| patioTableRound | a mosaic of proud tiles in rings (`_mosaic`: the grout a baked V between them), an iron band, a turned column with collars, a moulded foot |
| patioTable4 | breadboard ends with pegs, arched aprons, tapered legs with eased corners, an H stretcher |
| patioChair | a bent-tube back over the top, front legs running into the arms, a piped soft cushion, a seat frame, bowed slats, glides |
| benchGarden3 | the park bench's cast ends on three frames (the middle one armless), bolts |
| outdoorBar | a moulded counter, slatted ends, eighteen front slats with gaps, open shelves at the back with bottles and glasses (bottle_green, bottle_amber, bottle_clear), a one-piece bent foot rail with flanges |
| parasol | the canopy lofted in one (eight panels sagging between the ribs, top and underside), a scalloped valance, ribs, struts and tips, a turned finial; a moulded base with its screw, the crank's housing and handle |
| sunLounger | rails rounded at the ends, tapered legs with rails, piped soft cushions (the long one now meeting the seat's end, so the bed probe finds it), a towel laid over the foot and one rolled; a piped back pad and a plump pillow |
| hammock | the sling's sides curling up where it sags (its middle line keeps its heights), turned spreader bars, five cords fanning to a ring and a rope to an eye hook, whipped hems, a soft pillow; the stand's base and posts cut level, rounded feet, post caps |
| porchSwing | legs cut level meeting under a beam with cut-back ends, gussets; shaped arms on turned posts, an arched top rail, a front rail; two chains from the corners to a ring, one up to a hook; a piped cushion |
| swingSet | round timbers tapering, the beam's end caps; belt seats sagging between their chains, clips, hanger rings round the beam |
| hotTubCedar | one wall with a V between each of its 32 staves, a moulded rim, flat bands with lugs, a step with a nosing and a riser, rippled water and the jets' foam; a cover with a rounded edge, straps, clips and handles |
| grill | a rolled lip, dampers, bent side handles, a grate ring and bars, patties and bent sausages, leg sockets, a wire rack, an ash pan, wheels with hub caps; a lid with a vent tab, a thermometer and a shielded handle |
| grillStation | a moulded worktop, a tilted control panel with turned knobs and an igniter, framed stainless doors with bar pulls and hinges, a towel bar; the hood's dark end caps, bar handle and tilted gauge |
| firePit | two courses of rough stones round a dark wall, ten rough wedge capstones (three pushed out a little so the bounds stay the old random stones'), logs with rough bark and pale sawn ends (`log_end`), flames as twisted lofted tongues |

## OU1. Patio seating and tables (Validate › Patio)

Built 2026-10-14; they sat in Validate › Patio until signed off (back to Outdoor › Seating).

- **Park bench** (`bench`): cast-iron ends (legs, arm with a rounded end and a scroll, a stretcher, feet), four
  seat slats (their top at 0.45 m, `seats.bench`) and three back slats leaning 8°.
- **Folding bistro set** (`patioSet`): a slatted table on iron legs crossed at its sides (top at 0.72 m, where the
  content's `surface` is), between two folding chairs facing each other across it, their seats' fronts 4 cm under
  its edges (content `places` at ±0.43 m; the legs clear of the sitters' knees); a cup and a jug on the table.
- **Picnic table** (`picnicTable`): five planks, their top at 0.73 m (content `surface`), a two-plank bench each
  side at 0.44 m (`seats.picnicTable`; 0.29 m between bench and top), A-frames with bearers,
  a centre cleat and braces; a red gingham cloth, a basket and two cups.
- **Garden chair** (`gardenChair`) and **Adirondack chair** (`adirondackChair`): one slatted frame (seat sloping
  back, wide arms, stringers, braces); the garden chair is white with five parallel back slats and a top rail
  and a cup on its arm, the Adirondack green with a fanned, arched back.
- **Round patio table** (`patioTableRound`): a teal top with a ring of yellow and cream tiles, an iron rim, a
  pedestal and a domed foot. It is coloured as Frame, Top and Tiles (not "Slats": it has none).
- **Teak patio table** (`patioTable4`): five planks at 0.74 m on aprons, square legs and a low stretcher,
  centred on the content's top (−0.43 to 0.35 deep).
- **Patio chair** (`patioChair`): a bent green-iron frame, three back slats and a seat cushion (Frame, Cushion).
- **Garden bench** (`benchGarden3`): three seat and three back slats, 1.9 m long, on three iron frames, arms at the
  ends; on two tiles, its three sitters 0.67 m apart (it was 2.7 m on three).
- **Outdoor bar** (`outdoorBar`): a teak body with twenty front slats, a lighter counter at 1.06 m (the content's
  bar top), a back shelf and a steel foot rail (Body, Counter, Foot rail).
- **Parasol** (`parasol`): a weighted base and pole, an octagonal canopy on eight ribs and struts, with a valance.
  The canopy is its own part, hinged on the pole at 1.3 m: `stir` rocks it 3° in 2.4 s, an ambient clip
  (`idle`), so it moves in the wind near the camera (not on Low). At first 1.2°: from the game camera the rim moved
  a pixel or two between the stir's ends, too little to notice; at 3° it moves 3–4 px and the shading shifts.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| bench | 1.74 × 0.89 × 0.53 | 3132 | 2 | Frame, Slats |
| patioSet | 0.60 × 0.94 × 1.42 | 3328 | 2 | Frame, Slats |
| picnicTable | 2.50 × 1.05 × 1.62 | 3488 | 3 | Frame, Planks, Cloth |
| gardenChair | 0.84 × 1.04 × 0.75 | 2316 | 1 | Chair |
| patioTableRound | 0.82 × 0.73 × 0.82 | 2412 | 1 | Frame, Top, Tiles |
| patioTable4 | 1.60 × 0.74 × 0.79 | 2032 | 1 | Frame, Top |
| patioChair | 0.46 × 0.93 × 0.45 | 1488 | 2 | Frame, Cushion |
| adirondackChair | 0.88 × 1.10 × 0.79 | 2180 | 1 | Chair |
| benchGarden3 | 1.90 × 0.90 × 0.60 | 3616 | 2 | Frame, Slats |
| outdoorBar | 2.00 × 1.06 × 0.89 | 3288 | 2 | Body, Counter, Foot rail |
| parasol | 2.23 × 2.43 × 2.23 | 3272 | 3 | Canopy, Pole, Edge |

`check-glb`: 0 failed, no warnings (the Adirondack's seat probe fell between two slats: the seat slats are now
11 cm deep). Khronos validator: 0 errors, 0 warnings each.

**In the game** (2026-10-14, headless, WebGPU and WebGL2): the gallery next to the old parts; the clips bench
(`stir`); in play, all eleven bought in a garden (one chair at each of the four facings, three recoloured), a
resident relaxing in the Adirondack chair, sitting on both benches, the bistro set and the patio chair (at the
round table), and having a picnic at the picnic table, by day and on into the night.

## OU2. Loungers, swings and the hot tub (Validate › Loungers)

Built 2026-10-14; they sat in Validate › Loungers until signed off (back to Outdoor › Seating; the swing set to Outdoor › Play).

- **Sun lounger** (`sunLounger`): a slatted frame, a seat cushion and a beach towel; the back is its own part, an
  upholstered pad and pillow hinged at the seat's end, raised 35°. `recline` lowers it 30° (nearly flat) for
  sunbathe and doze, so the lying resident (`BEDS` 0.35 m) lies along it; it rises again after.
- **Hammock** (`hammock`): a 2.8 m wooden stand on its 1 × 3 tiles (Q-07; it was 1.86 m on 1 × 2, lain in
  folded); the sling (a sagging striped mesh 0.74 m wide, spreader bars, ropes, a pillow) is hinged on the line
  through the hooks (1.1 m). `sway` rocks it ±8° in 2 s, a loop for swing and doze, and the resident rides it
  (`ride`). They lie in the sling's dip (`furniture.json` `beds.hammock.sling`, its heights from `_sling` and
  `SLING_*`: 0.52 m in the middle, 0.84 at the spreader bars ±0.98, the pillow 0.78 towards the head), the head
  on the pillow and the feet inside the spreader bar. A changed sling needs those numbers changed too; the two pack
  hammocks have theirs.
- **Porch swing** (`porchSwing`): an A-frame; the slatted seat with arms, cushion (its top 0.52 m, the content's
  `seat`; 5 cm lower than it was) and chains swings from the beam,
  `swing` ±10° in 2 s while sat in, ridden. Chains are plain satin, so the seat stays one draw call.
- **Swing set** (`swingSet`): a round-timber A-frame, its legs splayed ±0.72 m at the foot (2 × 2 tiles; it was
  ±0.42 on 2 × 1, too upright for seats swinging 20°); both seats and their chains are one part, `swing` ±20° in
  1.6 s while played on, ridden.
- **Cedar hot tub** (`hotTubCedar`): banded staves with grooves, a rim and a step; the water with bubbles is its own
  part (`bubbles`: a whole turn in 2 s, a loop while soaking); a bench runs round the inside wall at 0.35 m under
  the water (`seats.hotTubCedar`). The cover is two halves, a hair apart at the seam across its middle,
  the front one inside the back one (`cover_front` attached to `cover`). `cover_off` (1.5 s) folds the front half
  back over the back half on the seam (half a second), then lifts the two together, turns them down behind the tub
  and lowers them to hang there on their folded edge, as on a cover lifter, the round edge just off the ground
  (before, it stood on end in the air behind the tub; then, 2026-10-21, one part squashed to half its depth about its
  back edge). The second half is a fourth draw, as the shower's door was (Q-05; 2026-10-26, review A-outdoor-01). It is the model's `open` clip too: soaking
  leaves the tub open (content `leaves`), so the cover stays off until a resident puts it back on.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| sunLounger | 0.62 × 0.76 × 1.86 | 2376 | 3 | Frame, Cushions, Towel |
| hammock | 1.00 × 1.20 × 2.80 | 2164 | 3 | Frame, Fabric, Stripes |
| porchSwing | 1.92 × 2.05 × 0.92 | 2580 | 3 | Seat, Frame, Chains, Cushion |
| swingSet | 1.92 × 1.99 × 1.51 | 1628 | 2 | Frame, Seats |
| hotTubCedar | 1.88 × 0.93 × 1.90 | 4368 | 4 | Cedar, Cover |

`check-glb`: 0 failed; one warning, the hot tub's 4 draws (aim for 3; at most 4, as the shower's, Q-05). Khronos
validator: 0 errors, 0 warnings each.

**In the game** (2026-10-14, headless WebGPU): all five bought; a resident sunbathing on the reclined lounger, lying
in the hammock, riding the porch swing and the swing set (frames in a row show seat and resident moving together),
and soaking in the hot tub (cover up, water bubbling; the cover stayed up after the soak).

## OU3. Grills and fire (Validate › Grills)

Built 2026-10-16; they sat in Validate › Grills until signed off (back to Outdoor › Cooking).

- **Charcoal grill** (`grill`): an enamelled kettle on three legs, wheels on the back two, a wire rack and an ash
  catcher; inside, a bed of red coals and a grate with burgers and sausages. The rim is at 0.79 m and the grate at
  0.77 (a 57 cm kettle's; they were 0.99 and 0.97, at the cook's elbow), 1.04 m with the lid. The lid (with a vent
  and a front handle) is hinged at the back of the rim: `lid_open` swings it up 95° for the whole of cook. The coals
  glow at night (`coal_hot`): the glow's copy no longer shows through the shut lid (`models.ts` `glowMesh`).
- **Gas grill station** (`grillStation`): a stainless cart on a plinth, two doors with bar handles and a control
  panel's chrome knobs, under a dark counter at 0.9 m (the content's `surface`); a chopping board with a tray of
  patties on the left, a bottle and a pepper mill at the back right, off the surface. The grill in the middle (a dark
  firebox, a grate with burgers, sausages and corn) has a stainless barrel hood with a handle and a thermometer,
  hinged at the back: `hood_up` swings it up 75° for cook and feast. The game puts its cooking props (the chopping
  board, food on the grill) on the station's middle, which is the grate.
- **Fire pit** (`firePit`): a stone bowl with a ring of ten capstones, an ash bed with embers that glow at night
  (`glow`, with the content's light, which is always on after dark) and a teepee of logs. `flames` grows seven
  tongues and a bright core on the logs (gloss, aglow at night) for the whole of relax, from a point under the ash
  in all three axes (flattened, their glow showed a disc through the ash). Residents warming up sit on the front
  capstones, facing out (content `seat`: 0.38 m high, 0.38 forward): before, they sat in the middle of the fire.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| grill | 0.64 × 1.04 × 0.64 | 4434 | 2 | Body, Lid |
| grillStation | 1.66 × 1.20 × 0.66 | 4016 | 3 | Cart, Hood |
| firePit | 0.92 × 0.62 × 0.92 | 3006 | 3 | Bowl, Stones |

`check-glb`: 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.

**In the game** (2026-10-16, headless WebGPU and WebGL2): all three bought in a garden (the grill at all four facings, the station at three, the fire pit at three), by day
and at night; a resident grilling dinner (lid up), grilling a feast at the station (hood up) and warming up by the
fire pit (flames up, sitting on its rim), by day and at night; the colour studio.

## Rebuild

```
cargo art outdoor            # every .glb, the pack, check-glb
cargo art outdoor --preview  # previews/outdoor/bench.png, parasol@stir.png, …
```
