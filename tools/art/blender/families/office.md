# Office — notes

`office.py` builds the fourteen office models (plan O1–O3) into `web/public/assets/models/office/` and their
entries in `packs/office/manifest.json`: the desks (`model.deskSimple`, `deskCorner`, `deskStanding`, `deskExecutive`,
`computerDesk`, `deskWriting`, `deskLaptop`, `computerGaming`), the desk chairs (`model.chairExecutive`, `chairTask`,
`officeChair`) and the storage and boards (`model.filingCabinet`, `whiteboard`, `corkboard`). Built
2026-10-14; they sat in Validate › Desks, Office Chairs and Office Storage until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`). Backs against the wall stop at its face (game depth −0.43, −0.93 two tiles
deep), so the simple, standing and executive desks' tops are 3 cm shallower than their content `surface` rectangle
at the back. Desk tops are at the content's `surface` height: 0.74 (catalog2), 0.75 (the old part), 1.05 standing.

## The desks with their own chair

The computer, writing, laptop and gaming desks keep the old part's chair: residents sit in the back half of the
footprint, 17 cm behind its middle at 0.44 m (`characters.ts` SEATS), facing the desk, so the screen faces the wall
and its back the room. The chair is part of the still desk (no swivel) and its back stops short of the wall (leant 7°,
5° with the gaming chair's headrest).

- **Computer desk:** the keyboard and mouse are on a tray under the top, which slides 15 cm out to the resident
  (`tray_out`) for code, write, markets, browse and homework, for the whole use.
- **Writing desk:** a drawer of letters and pencils slides 22 cm out (`drawer_open`) for write and homework; `hold`
  2.5 s, so it shuts once they've taken their paper.
- **Laptop desk:** the laptop is shut at rest and opens 110° about the hinge on its far edge (`lid_open`) for browse,
  code, write and homework, for the whole use.
- **Gaming PC:** two case fans in the tower's front spin a turn a second (`fans`, a `G.spin` loop) while a game is
  played. The monitors, speaker and keyboard are the `case` material too, so the "PC & monitors" colour paints them all.

## The rest

- **Standing desk:** a keyboard shelf under the front slides 20 cm out to the standing resident (`tray_out`) for
  browse and code. Telescoping legs, the monitor on a stand at the back, the height buttons under the front right.
- **Simple desk:** oak on hairpin legs, a drawer box under the right. Static.
- **Corner desk:** an L of two 0.7 m wings of white laminate along the back and the left of its 2 × 2 tiles (a room's
  corner), the inner corner cut on the diagonal, on oak end panels and a modesty panel, a drawer pedestal under each
  wing, and its own swivel chair (still) in the inner corner, turned to face the corner, both wings in reach:
  content `places` `[0.11, 0.11]` facing -135, `seat` 0.44, write and homework as at the writing desk; its
  `surface` is the back wing (1.86 × 0.7 m). 2026-10-21, review M-office-01: it was a 1.86 m square slab with one
  corner cut, read as an island, a chair facing it from the front tiles at its edge or its front panel; generated
  studies now take it on its own (`housegen.json`), not as a desk set with a chair. `_turned` sets the joined part
  to Euler mode: a part whose first piece is a rod (`G.rod`) is in quaternion mode, which ignored the chair's turn.
- **Executive desk:** walnut on two pedestals of three drawers with brass handles, a green leather pad. Its middle
  drawer is a part of its own but still: left decorative in 0.3.0, because its clip would need a seated use on the
  desk.
- **Desk chairs:** a seat, back and arms on a gas lift and five castors. The seat (everything above the lift) swivels
  15° and back (`swivel`, `hold` 0.6 s) as a resident sits (and, for the executive and desk chairs, writes or does
  homework), and carries them round with it (`ride`). Seats at the content's heights: 0.49, 0.48 and 0.47 m.
- **Filing cabinet:** steel, three drawers with handles and label cards, a paper tray and a plant on top. Its top
  drawer, with a box of files behind it, slides 30 cm out (`drawer_open`) for its use, File papers; `hold` 2 s.
- **Whiteboard** (wall): an aluminium frame, a marker tray, a heading and a note. While a resident plans, the plan
  (lines, a chart) grows across the board from its left (`scribble`, `G.grow` in x and height from a point), and is
  wiped as the clip plays back at the end.
- **Corkboard** (wall): cork in a wooden frame, five pinned notes. Static.

## Detail (2026-10-22)

Rebuilt with the forms and the kit, same sizes, centres, parts, clips, paint and draws:

- **Chairs** (`_chair_base`, `_chair_seat`, shared by the three desk chairs and the desks' own chairs): five legs
  swept tapering from a turned hub, twin-wheel castors (one wide wheel, `lite`, on the desks' chairs), a tapered
  lift sleeve; the mechanism with its height lever and tension knob; a moulded shell under a piped seat; a spine bent
  from the mechanism up to a plate on the back. The back is lofted (`_contour_rings`: rounded top and foot, a lumbar
  swell, sides dished forward) on a plastic shell, a welt round its face (the office chair); the task chair's is a
  mesh in a swept frame with a lumbar bar; the executive chair's four stitched leather pillows on a shell, with loop
  arms bent from flat bar; the gaming chair's racing stripes and a headrest on two posts. T arms are a bent bar under
  a padded rest.
- **Desks:** square-tube frames each side bent in one piece (`_desk_frame`, the computer desk's sled sides) on glides;
  slab drawer fronts with reveals and bar pulls bent from one rod (`_bar_pull`); tops rounded at the edges. On them:
  monitors with a bezel, the picture, a rounded back and a curved neck to an oval foot (`_monitor`), keyboards with
  their keys (`_keyboard`, the laptop's `_laptop_keys`), mice, cables, an architect's lamp (two arms, a knuckle, its
  springs, a turned shade), a hollow mug of coffee, a ribbed cactus in a rimmed pot, notebooks with their pages and
  band, pens with clips. The writing desk's drawer: a turned brass knob, letters with stamps, sharpened pencils. The
  laptop desk: a hinge, a bezel and a logo on the lid, a charger. The gaming PC: a wrist rest, a lit mouse, speaker
  drivers in rings, headphones on a hook, the case's top vent and buttons, cards behind the tinted side, swept fan
  blades. The corner desk: an L top extruded with rounded edges, a cable grommet, a laptop, mug, paper tray, cactus
  and notebook. The standing desk: telescoping columns with collars, tapered feet on levellers, a handset with its
  display and buttons, the monitor's cable down the leg, a mug and a cactus. The simple desk: hairpin legs each bent
  from one rod, screwed plates. The executive desk: raised-panel drawer fronts (`kit.shaker`) with brass bar pulls and
  roses, a moulded plinth round each pedestal (`kit.moulding`), a panelled modesty board, a stitched border round the
  leather pad, a letter opener.
- **Filing cabinet:** pressed drawer fronts (`kit.shaker`), bar handles, chrome label holders with written cards, a
  kick plinth, a key in the lock; hanging files with coloured tabs and papers in the open drawer, its rails; a letter
  tray with sloped sides and a plant of swept leaves on top.
- **Boards:** the whiteboard's aluminium frame one mitred profile (`kit.moulding`), a J-shaped marker tray, turned
  markers with caps, magnets holding a sheet, the plan drawn in strokes (`_scrawl`: handwriting, an arrow, a ring, a
  chart). The corkboard: a moulded frame, notes curling off the cork, turned push pins, a photo and a ticket.

| Model | Size (w × h × d) | Triangles (before → now) | Draws | Colour parts |
|---|---|---|---|---|
| `model.deskSimple` | 1.50 × 0.74 × 0.69 m | 1,488 → 1,916 | 2 | Top, Frame, Drawers |
| `model.deskCorner` | 1.86 × 1.01 × 1.86 m | 4,784 → 5,520 | 2 | Top, Frame, Drawers, Chair |
| `model.deskStanding` | 0.96 × 1.54 × 0.63 m | 2,604 → 4,078 | 3 | Top, Frame, Shelf |
| `model.deskExecutive` | 1.80 × 0.75 × 0.77 m | 2,624 → 4,344 | 2 | Top, Frame, Drawers, Handles, Desk pad |
| `model.computerDesk` | 1.54 × 1.22 × 0.87 m | 6,728 → 7,396 | 3 | Top, Frame, Drawers, Keyboard tray, Chair |
| `model.deskWriting` | 1.54 × 1.11 × 0.87 m | 4,944 → 4,944 | 3 | Top, Frame, Drawers, Chair |
| `model.deskLaptop` | 0.90 × 1.01 × 0.87 m | 4,564 → 5,204 | 3 | Top, Frame, Drawer, Laptop, Chair |
| `model.computerGaming` | 1.70 × 1.30 × 0.90 m | 6,944 → 7,692 | 3 | Desk, PC & monitors, Lights, Chair |
| `model.chairExecutive` | 0.62 × 1.33 × 0.64 m | 2,864 → 6,648 | 2 | Frame, Seat |
| `model.chairTask` | 0.61 × 1.06 × 0.60 m | 2,904 → 5,124 | 3 | Frame, Seat, Back |
| `model.officeChair` | 0.61 × 1.16 × 0.62 m | 2,648 → 5,248 | 3 | Frame, Seat |
| `model.filingCabinet` | 0.46 × 1.21 × 0.64 m | 3,340 → 3,520 | 3 | Cabinet, Handles |
| `model.whiteboard` | 0.93 × 0.65 × 0.07 m | 1,608 → 3,100 | 2 | Board, Frame |
| `model.corkboard` | 0.74 × 0.52 × 0.03 m | 1,212 → 1,516 | 2 | Board, Frame |

The desks with their own chair spend about half their triangles on it: its castors and arms are lighter there
(`lite`). `check-glb`: 14 models, 0 failed. Its warnings, all expected: the two boards float (wall things). Khronos
validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art office            # the fourteen .glb, the pack, check-glb
cargo art office --preview  # previews/office/<id>.png, <id>@<clip>.png
```
