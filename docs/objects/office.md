# Office

The fourteen O1–O3 models are built by `tools/art/blender/families/office.py` (notes: `families/office.md`): the
keyboard shelves slide out (`tray_out`), the writing desk's and filing cabinet's drawers open (`drawer_open`), the
laptop opens (`lid_open`), the gaming PC's fans spin (`fans`, a loop), the chairs swivel and carry the sitter
(`swivel`, `ride`) and the whiteboard fills with a plan (`scribble`). The executive desk's middle drawer is a part of
its own but still (+use).

Desks keep their top height (`TOPS`, content `surface`): things stand on them, and those don't follow a moving top.
The computer, writing, laptop and gaming desks have their chair built in: the resident sits in the back half of the
footprint (`SEATS`), facing the wall, the screen towards them. A separate chair facing a desk or table (content
`seatAt`) is drawn pulled in to it while sat on (`tableTuck.ts`, below).

## O1. Desks (8)

- **Family:** `tools/art/blender/families/office.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `deskSimple` Simple desk | 2x1 · floor | — | office / desks | Top, Frame, Drawers | Static | Static. |
| `deskCorner` Corner desk | 2x2 · floor | — | office / desks | Top, Frame, Drawers | Static | Static. |
| `deskStanding` Standing desk | 1x1 · floor | browse:type code:type | office / desks | Top, Frame, Shelf | Use | `tray_out`: the keyboard shelf slides out for browse and code. (A rising top would leave the things on it behind.) |
| `deskExecutive` Executive desk | 2x1 · floor | — | office / desks | Top, Frame, Drawers, Handles, Desk pad | Static | Static. (+use) a drawer slides out. |
| `computerDesk` Computer desk | 2x1 · floor | code:type write:type markets:type browse:type homework:type | office / — | Top, Frame, Drawers, Keyboard tray, Chair | Use | `tray_out`: the keyboard tray slides out 15 cm for code, write, markets, browse and homework. |
| `deskWriting` Writing desk | 2x1 · floor | write:write homework:write | office / — | Top, Frame, Drawers, Chair | Use | `drawer_open` for write and homework; `hold` 2.5 s. |
| `deskLaptop` Laptop desk | 1x1 · floor | browse:type code:type write:type homework:type | office / — | Top, Frame, Drawer, Laptop, Chair | Use | `lid_open`: the laptop opens from shut to 110° for browse, code, write and homework. |
| `computerGaming` Gaming PC | 2x1 · floor | game:play code:type browse:type | office / — | Desk, PC & monitors, Lights, Chair | Loop (E1) | `fans`: the case fans spin while played (E1). |

**In play** (signed off 2026-10-17):
- Each in all four facings (against the north, west and east walls and standing free), every part a distinct colour
  by day, on Low WebGL2 at night and in the colour studio, turned round to see the parts at the back.
- The standing desk's shelf slides out for Browse, the computer desk's tray for Practise coding, the writing desk's
  drawer for Write (shut after its `hold`), the laptop opens for Browse, and the gaming PC's fans go round while a
  game is played and finish their round when it ends. Each shuts when the use ends.
- The executive desk's plinths, under its pedestals, take the "Frame" colour, as the case.
- The desks with their own chair are written at with the usual desk hands; only four pack desks, their chair built
  in 0.3 m from the top, use nearer hands (`sitHandsNear`).
- A separate chair at the simple, corner or executive desk is pulled in as far as the knees fit. Where that leaves the
  top out of reach the writer leans in further (`sitHandsFar`): the executive desk's pedestals let a one-tile chair
  in only 6 cm (the writer leans rather than sliding sideways to the knee hole), and the corner desk's side panel
  stops it too.
- Known, not changed: the built-in chairs take the desk's "Frame" colour on their bases and castors ("Chair" colours
  the seat and back). An office chair at the standing desk isn't pulled in (its top is 0.57 m over the seat) and its
  hands stay short of it. A chair facing the laptop desk from behind can't come in (the desk's own chair is there)
  and reaches over it. Neither pairing is set up by anything.

## O2. Desk chairs (3)

- **Family:** `tools/art/blender/families/office.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `chairExecutive` Executive chair | 1x1 · floor | sit:sit write:write homework:write | office / chairs | Frame, Seat | Use | `swivel`: the seat turns 15° and back for sit, write and homework. Pulled in to the desk or table it faces while sat on, as far as the knees fit under it: written at with the hands on the top (reaching further where a drawer pedestal stops it). |
| `chairTask` Desk chair | 1x1 · floor | sit:sit write:write homework:write | office / chairs | Frame, Seat, Back | Use | `swivel`. Pulled in to the desk or table it faces while sat on, as far as the knees fit under it: written at with the hands on the top (reaching further where a drawer pedestal stops it). |
| `officeChair` Office chair | 1x1 · floor | sit:sit | office / — | Frame, Seat | Use | `swivel`. Pulled in to the desk or table it faces while sat on, as far as the knees fit under it. |

**In play** (signed off 2026-10-17):
- Each standing free and sat on in all four facings: the seat swivels and shuts after its `hold` (0.6 s), by day and
  at night, on WebGPU and WebGL2, Medium and Low. The swivel part carries the sitter round (`ride`).
- Pushed up to a simple desk: Write and Do homework at the desk (the executive and desk chairs, `needsTable`) and Sit
  (the office chair), the sitter facing the desk.
- Once seated the sitter shuffles in with the chair (the renderer moves its instance, contact shadow and swivel part)
  until the top's near edge is in reach, by up to 0.45 m, and pushes it back while rising; the sim's positions are
  unchanged. How far it goes is read from the table's drawn shape (`tableTuck.ts`): the knees and toes stop short of
  legs, pedestals and drawer boxes. Not done for a top more than 0.45 m over the seat (the standing desk).
- Colour parts: Frame is the base, lift and arms; Seat the seat and back; the desk chair's mesh back is its own Back.
- Known, not changed: getting up, the sim puts the resident on the tile in front at once, so they stand up while
  gliding back, the chair sliding back over the same half second.

## O3. Storage and boards (3)

- **Family:** `tools/art/blender/families/office.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `filingCabinet` Filing cabinet | 1x1 · floor | file:fetch | office / — | Cabinet, Handles | Use | `drawer_open`: the top drawer slides out 30 cm for file; `hold` 2 s. |
| `whiteboard` Whiteboard | 1x1 · wall | plan:paint | office / — | Board, Frame | Use | `scribble`: marker lines grow on the board for plan. |
| `corkboard` Corkboard | 1x1 · wall | — | office / — | Board, Frame | Static | Static. |

**In play** (signed off 2026-10-17):
- The filing cabinet against the north, west and east walls and free facing north: its top drawer slides out for
  "File papers" and shuts after its `hold` in every facing.
- The whiteboard and corkboard on the north, west and east walls and on the outside of a wall: the plan grows across
  the board while the resident plans at it, and is wiped as the clip plays back. The corkboard is still.
- Colour studio: the cabinet and its handles; each board and its frame (the whiteboard's marker tray goes with the
  frame); "In use" opens the drawer and fills the board.
