# Office

Desks keep their top height (`TOPS`): things stand on them, and those don't follow a moving top.

## O1. Desks (8)

- **Family:** `tools/art/blender/families/office.py`
- **Validate group:** `desks` (Validate › Desks)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `deskSimple` Simple desk | 2x1 · floor | — | office / desks | Top, Frame, Drawers | Static | Static. |
| `deskCorner` Corner desk | 2x2 · floor | — | office / desks | Top, Frame, Drawers | Static | Static. |
| `deskStanding` Standing desk | 1x1 · floor | browse:type code:type | office / desks | Top, Frame, Drawers | Use | `tray_out`: the keyboard shelf slides out for browse and code. (A rising top would leave the things on it behind.) |
| `deskExecutive` Executive desk | 2x1 · floor | — | office / desks | Top, Frame, Drawers | Static | Static. (+use) a drawer slides out. |
| `computerDesk` Computer desk | 2x1 · floor | code:type write:type markets:type browse:type homework:type | office / — | Top, Frame, Drawers | Use | `tray_out`: the keyboard tray slides out 15 cm for code, write, markets, browse and homework. |
| `deskWriting` Writing desk | 2x1 · floor | write:write homework:write | office / — | Top, Frame, Drawers | Use | `drawer_open` for write and homework. |
| `deskLaptop` Laptop desk | 1x1 · floor | browse:type code:type write:type homework:type | office / — | Top, Frame, Drawers | Use | `lid_open`: the laptop opens from shut to 110° for browse, code, write and homework. |
| `computerGaming` Gaming PC | 2x1 · floor | game:play code:type browse:type | office / — | Desk, Case | Loop (E1) | `fans`: the case fans spin while played (E1). Until then static. |

## O2. Desk chairs (3)

- **Family:** `tools/art/blender/families/office.py`
- **Validate group:** `office chairs` (Validate › Office Chairs)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `chairExecutive` Executive chair | 1x1 · floor | sit:sit write:write homework:write | office / chairs | Frame, Seat | Use | `swivel`: the seat turns 15° and back for sit, write and homework. |
| `chairTask` Desk chair | 1x1 · floor | sit:sit write:write homework:write | office / chairs | Frame, Seat | Use | `swivel`. |
| `officeChair` Office chair | 1x1 · floor | sit:sit | office / — | Frame, Seat | Use | `swivel`. |

## O3. Storage and boards (3)

- **Family:** `tools/art/blender/families/office.py`
- **Validate group:** `office storage` (Validate › Office Storage)
- **Placeholders to match:** `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `filingCabinet` Filing cabinet | 1x1 · floor | — | office / — | Body, Frame | Static | Static. (+use) the top drawer slides out. |
| `whiteboard` Whiteboard | 1x1 · wall | plan:paint | office / — | Body, Frame | Use | `scribble`: marker lines grow on the board for plan. |
| `corkboard` Corkboard | 1x1 · wall | — | office / — | Body, Frame | Static | Static. |
