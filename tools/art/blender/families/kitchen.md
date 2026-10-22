# Kitchen — notes

`kitchen.py` builds the kitchen models into `web/public/assets/models/kitchen/` and their entries in
`packs/kitchen/manifest.json`: the **fridge** (`model.fridge`, `fridge.glb`), the six **K1 fridges**
(`fridgeMini`, `fridgeBuiltIn`, `fridgeFrenchDoor`, `fridgeRetro`, `fridgeMiniRetro`, `wineFridge`) and the two **sinks**
(`model.farmhouseSink`, `farmhouseSink.glb`; `model.counterSinkDouble`, `counterSinkDouble.glb`). The fridge was
first made as an evaluation on 2026-10-13; on 2026-10-14 the flat build and the doors build (`kitchen_doors.py`,
`fridge.doors.glb`) became this one build, and the two sinks and the five K1 fridges were added.

## The fridge

A top-freezer fridge: the **top compartment is the freezer** (frozen food), the **bottom compartment is
the fridge** (normal food), separated by a real divider.

- 0.740 × 1.800 × 0.722 m (w × h × d): exactly the old part's bounds, built in its numbers (`F`, a
  `GameFrame`) and standing where the game places it (`centre=(0, -0.094)`), its back on the wall
- 4,286 triangles, 3 nodes (`cabinet`, `door_freezer`, `door_fridge`), 3 draw calls (one per node), 14
  materials, no UVs
- `check-glb`: ok, with one warning (below). Khronos glTF validator: 0 errors, 0 warnings.

The cabinet is five panels (sides, top, bottom, back), so the doors open onto an interior, not a solid
block. Inside are a matte liner on all five faces, a gloss **divider** at game height 1.234 (the freezer's
floor), three fridge shelves and one freezer shelf. The food colours are baked into the materials:
- **Freezer** (1.244–1.767 m): a frozen pizza on the floor; an ice cream tub and a bag of peas on the shelf.
- **Fridge** (0.094–1.224 m): milk and juice on the top shelf, apples and an orange on the middle one, eggs
  and a tub on the bottom one.

| Material | Finish | Used for |
|---|---|---|
| `body` | gloss | cabinet, doors; takes the player's colour (`paint`) |
| `trim` | satin | plinth, badge |
| `seal` | satin | door gap |
| `chrome` | chrome | bar handles |
| `liner` | matte | interior walls |
| `shelf` | gloss | shelves, divider, tub |
| `carton` | satin | milk / egg cartons |
| `bottle` | gloss | juice bottle |
| `produce`, `produce2` | satin | apples, orange |
| `egg` | matte | eggs |
| `frozen`, `icecream`, `peas` | satin | the freezer's food |

## Doors

Each door's origin sits on its hinge, at game `x = +0.364 m` (the handles are on the left). Its handle,
and the freezer's badge, are joined into it. There is one clip per door. Both take the same 1.75 s, so the
doors move at one speed, but each has its own angle:

| Clip | Node | Duration | Swing |
|---|---|---|---|
| `freezer_open` | `door_freezer` | 1.75 s | 0° → −118° (the short door swings wide) |
| `fridge_open` | `door_fridge` | 1.75 s | 0° → −102° (the tall door stops a little shorter) |

The pack's `motions` plays `fridge_open` for the `fetch`, `cook` and `drink` animation tags (snacks, a
meal's ingredients, a cold drink), as the K1 fridges do, and holds it for 3.5 s.
`freezer_open` plays for the `frozenTreat` use ("Get some ice cream").

## The sinks

The two kitchen sinks share the fridge's `GameFrame` style and the family's palette (extended with
`porcelain`, `stone`, `wood`, `metal`, `dark`, `plant`, `water`, `pot`). Each has a `body` material (paint) and a
`water` moving part: the stream is built full length from the spout into the basin, hidden at rest (scaled
to 0.001 along its axis), and a `water_on` clip grows it to 1 over 0.4 s. `motions` plays `water_on` for the
`wash` animation tag, with no `hold`, so the water runs for the whole use and retracts afterwards (the `wash`
tag covers both the `wash` and `washUp` interactions). The double sink has a stream for each bowl, each its own clip
(`water_on_0`, `water_on_1`) played for the resident in that slot (`slot`; 2026-10-21, review A-kitchen-03: one
washer ran both taps).

Detail pass (2026-10-22): each bowl is one lofted piece (`_bowl`: a flat rim round a rounded opening, the walls running
down to a floor that dips to the drain; the farmhouse one square outside, an apron proud of the cabinet), with a
chrome-ringed drain and strainer. The tap is one bent tube (`G.kit.tube`) on a turned escutcheon, with a spray head and
a lever on a valve beside it. The cabinets have shaker doors (`_shaker`: a frame with a hole round a recessed panel)
and bar pulls in one piece (`_pull`); the farmhouse worktop is one U-shaped slab round the basin, the double sink's
stone worktop one slab with the two openings cut in it. A lofted soap dish with a bar of soap and a terracotta pot
(`pot`) with a mound of herb leaves stand on the worktop.

| Model | Size (w × h × d) | Triangles | Draws | Note |
|---|---|---|---|---|
| `model.farmhouseSink` | 1.000 × 1.253 × 0.667 m | 4,410 | 3 | fireclay apron basin, U-shaped wood worktop, face frame and shaker doors, gooseneck tap and lever, soap dish, potted herb |
| `model.counterSinkDouble` | 2.000 × 1.263 × 0.660 m | 6,836 | 3 | two deep steel bowls set into a stone slab, four shaker doors, a gooseneck tap and lever each, soap dish, potted herb |

The double sink is built in the counters' drawn depth (the worktop -0.43..0.23, as COUNTER66 drew it), its bowls 17 cm
deep, the carcass stopping under them behind rails. `check-glb`: 0 failed; its warnings are 4 draw calls on the
French-door and wine fridges. Khronos validator: 0 errors, 0 warnings.

## The K1 fridges

Six more fridges match the old parts' bounds. Each hinges a door built closed and swings it open
with a `hold`, so it shuts while the resident is busy; the French-door freezer and the wine fridge's top
rack slide instead. The clip each use plays is in the pack's `motions`:

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.fridgeMini` | 0.600 × 1.000 × 0.605 m | 3,868 | 2 | door | `fridge_open` 1.6 s (snack `fetch`, meal `cook`); `hold` 3 s |
| `model.fridgeBuiltIn` | 0.840 × 2.025 × 0.664 m | 5,960 | 3 | fridge door, freezer door | `fridge_open` 1.75 s (fetch, cook, cold drink); `freezer_open` 1.75 s (frozen treat) |
| `model.fridgeFrenchDoor` | 0.900 × 1.820 × 0.756 m | 6,884 | 4 | left door, right door, drawer | `fridge_open` 1.75 s (fetch, cook); `door_right` 1.75 s (cold drink); `freezer_open` 1.75 s (frozen treat) |
| `model.fridgeRetro` | 0.720 × 1.640 × 0.722 m | 4,662 | 3 | door; the chrome lever inside it (`G.attach`) | `lever` turns 30° out (0.3 s, `hold` 0.4), then `fridge_open` 1.75 s (`delay` 0.3; fetch, cook, cold drink) |
| `model.fridgeMiniRetro` | 0.574 × 0.860 × 0.580 m | 3,729 | 2 | door (its handle, "MARIE" letters and unicorn sticker on it) | `fridge_open` 1.6 s (fetch, cook, cold drink); `hold` 3 s |
| `model.wineFridge` | 1.000 × 0.950 × 0.660 m | 3,720 | 4 | glass door, top shelf of bottles | `door_open` 1.5 s (wine `drink`), `hold` 2.5 s; `rack_out` 0.8 s, the shelf out 15 cm (`delay` 0.9, `hold` 0.9: out once the door is open, in before it shuts) |

What each has since the detail pass (2026-10-22), on the fridge's pattern (doors with seals, liners, bins and stock):
- **fridgeMini**: sides rounded on their edges, a vented kick plate, an ice box with a flap and grip, a trimmed shelf, a
  crisper with a grip cut in its front; a gable-top carton, a bottle and apples (with stalks) inside; a chopping board
  with a hanging hole, a jar and an apple on top; the door rounded (`F.soft`), a badge, a one-piece bar handle, its
  seal, liner and two bins with bottles, a can and a jar.
- **fridgeBuiltIn**: shaker cabinet fronts (frame round a recessed, lightly puffed panel), a cornice with a cove
  moulding swept round it, a vented kick plate; a water dispenser set into the fridge door (trim, recess, paddle,
  spout, display, drip tray); round bar handles in one piece; inside, trimmed glass shelves, two crispers, two freezer
  baskets with grips, food (carton, bottle, jam, apples, an orange, eggs, a tub, pizza, peas, ice cream); seals,
  liners, bins with bottles, a carton, jars and butter on the doors.
- **fridgeFrenchDoor**: steel sides rounded on their edges, a vented kick plate, rounded doors with slim handles, an
  ice and water dispenser with its display on the door the viewer sees on the left, the flip mullion; inside, trimmed
  shelves, two crispers and food (a cake, eggs, apples, jam, a tub); the doors' seals, liners, bins, bottles, jars and
  butter; the freezer drawer a bin with pizzas, ice cream and peas in it.
- **fridgeRetro**: the body one loft (rounded-square rings, the top rolling back) with the cavity cut out (a boolean,
  its faces lined); a chrome kick grille, chrome hinges; a door with big rounded corners, a trim line, a chrome
  nameplate, an escutcheon and a swept lever; inside an ice box with a flap and latch, a drip tray, shelves with
  chrome rails, a crisper, glass milk bottles, a pie, apples and butter; railed door shelves with bottles and eggs.
- **fridgeMiniRetro**: the brass D handle one bent tube (was rods joined by spheres); a door seal, a real door bin
  with a can and a jar, a gable-top carton, apples with stalks, a bottle, a trimmed shelf.
- **wineFridge**: rebuilt in drawn depths (worktop -0.43..0.23): racks with wooden lips and bottles lofted lying necks
  out with foil capsules, a pale liner, a ceiling over the door, the door a framed pane (one slab with a hole) with a
  display and a round bar handle, its seal; three drawers flush with the door with long pulls; a vented kick plate.

The built-in's freezer door and the French-door freezer drawer play `freezer_open` for "Get some ice
cream"; `model.fridge`'s freezer door does the same (`FRIDGE_USES`, by the interaction `frozenTreat`; the G-17
rebuild of 2026-10-21 had lost it, as the family script never named it: back the same day).

Fixed 2026-10-21 (G-41, M-kitchen-06): the wine fridge's door is a dark frame round a pane of smoked `glass` (it was
a solid slab with the glass in front of it, so it drew black), and what slides out is its top shelf of bottles
behind the door, not the drawer stack's top front beside it; the French-door fridge's sides, top and bottom come
forward 5 cm to the doors' backs, so a shut door sits flush on the cabinet instead of standing off it.
`fridgeFrenchDoor` and `wineFridge` take 4 draw calls (the most allowed, one per moving part plus the
still body); the rest take 2 or 3. The five were built and exported on 2026-10-14 and sat in Validate ›
Fridges until checked in play.

## Rebuild

```
cargo art kitchen            # fridge.glb, the pack, check-glb
cargo art kitchen --preview  # previews/kitchen/fridge.png, fridge@<clip>.png
```

## Where it stands (review, 2026-10-14)

Each floor model is centred where the game places it (its `Model`'s `centre`, which `G.settle()` uses),
not on its tile: its back is on the wall face (-0.43), as the old part's, a run of counters and
appliances lines up, and its top is under its content `surface`. Since the
2026-10-14 fix-ups the old parts no longer reach into the wall: both moved forward onto the face.
There is one counter depth, the catalogue's 0.66 m (worktop -0.43..0.23): the catalog2 counters keep
the old 0.79-deep numbers and are drawn through `COUNTER66` (gamekit `GameFrame(depths=)`,
the old `shallow`). Islands stand free, centred.
