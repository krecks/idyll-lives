# Office — notes

`office.py` builds the fourteen office models (plan O1–O3) into `web/public/assets/models/office/` and their
entries in `packs/office/manifest.json`: the desks (`model.deskSimple`, `deskCorner`, `deskStanding`, `deskExecutive`,
`computerDesk`, `deskWriting`, `deskLaptop`, `computerGaming`), the desk chairs (`model.chairExecutive`, `chairTask`,
`officeChair`) and the storage and boards (`model.filingCabinet`, `whiteboard`, `corkboard`). They replace the
primitive-part placeholders of `catalog.py`, `catalog2.py` and `furniture.py`, which stay the fallback. Built
2026-10-14; they sit in Validate › Desks, Office Chairs and Office Storage until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
except the corner desk, which is centred. Backs against the wall stop at its face (game depth −0.43, −0.93 two tiles
deep), so the simple, standing and executive desks' tops are 3 cm shallower than their content `surface` rectangle
at the back. Desk tops are at the content's `surface` height: 0.74 (catalog2), 0.75 (furniture.py), 1.05 standing.

## The desks with their own chair

The computer, writing, laptop and gaming desks keep their placeholder's chair: residents sit in the back half of the
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
- **Corner desk:** a white top filling its 2 × 2 tiles but for the front-right corner, cut on the diagonal (the
  placeholder's shape), on oak panels with a drawer pedestal. Static.
- **Executive desk:** walnut on two pedestals of three drawers with brass handles, a green leather pad. Its middle
  drawer is a part of its own but still: no use opens it yet (+use).
- **Desk chairs:** a seat, back and arms on a gas lift and five castors. The seat (everything above the lift) swivels
  15° and back (`swivel`, `hold` 0.6 s) as a resident sits (and, for the executive and desk chairs, writes or does
  homework), and carries them round with it (`ride`). Seats at the content's heights: 0.49, 0.48 and 0.47 m.
- **Filing cabinet:** steel, three drawers with handles and label cards, a paper tray and a plant on top. Its top
  drawer, with a box of files behind it, slides 30 cm out (`drawer_open`) for its use, File papers; `hold` 2 s.
- **Whiteboard** (wall): an aluminium frame, a marker tray, a heading and a note. While a resident plans, the plan
  (lines, a chart) grows across the board from its left (`scribble`, `G.grow` in x and height from a point), and is
  wiped as the clip plays back at the end.
- **Corkboard** (wall): cork in a wooden frame, five pinned notes. Static.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.deskSimple` | 1.50 × 0.74 × 0.69 m | 1,488 | 2 | Top, Frame, Drawers |
| `model.deskCorner` | 1.86 × 0.74 × 1.86 m | 1,324 | 2 | Top, Frame, Drawers |
| `model.deskStanding` | 0.96 × 1.54 × 0.63 m | 2,604 | 3 | Top, Frame, Shelf |
| `model.deskExecutive` | 1.80 × 0.75 × 0.77 m | 2,624 | 2 | Top, Frame, Drawers, Handles, Desk pad |
| `model.computerDesk` | 1.54 × 1.22 × 0.87 m | 6,728 | 3 | Top, Frame, Drawers, Keyboard tray, Chair |
| `model.deskWriting` | 1.54 × 1.11 × 0.87 m | 4,944 | 3 | Top, Frame, Drawers, Chair |
| `model.deskLaptop` | 0.90 × 1.01 × 0.87 m | 4,564 | 3 | Top, Frame, Drawer, Laptop, Chair |
| `model.computerGaming` | 1.70 × 1.30 × 0.90 m | 6,944 | 3 | Desk, PC & monitors, Lights, Chair |
| `model.chairExecutive` | 0.62 × 1.34 × 0.64 m | 2,864 | 2 | Frame, Seat |
| `model.chairTask` | 0.62 × 1.05 × 0.60 m | 2,904 | 3 | Frame, Seat, Back |
| `model.officeChair` | 0.62 × 1.16 × 0.62 m | 2,648 | 3 | Frame, Seat |
| `model.filingCabinet` | 0.46 × 1.21 × 0.64 m | 3,340 | 3 | Cabinet, Handles |
| `model.whiteboard` | 0.93 × 0.65 × 0.07 m | 1,608 | 2 | Board, Frame |
| `model.corkboard` | 0.74 × 0.52 × 0.03 m | 1,212 | 2 | Board, Frame |

`check_glb.py`: 14 models, 0 failed. Its warnings, all expected: the two boards float (wall things), the filing
cabinet is 11 cm off centre (against the wall, as its placeholder) and 17 cm lower than its placeholder's tall plant,
and the boards' depths differ from their placeholders' by a centimetre or two. Khronos validator: 0 errors, 0
warnings each.

## Rebuild

```
python3 -B tools/art/blender/build.py office            # the fourteen .glb, the pack, check_glb
python3 -B tools/art/blender/build.py office --preview  # previews/<id>.png, <id>@<clip>.png
```
