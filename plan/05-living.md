# Living room

Seats keep their height and depth (`SEATS`); a seat that moves while sat on must keep the sitting pose believable.
The wall bed is done (`families/living.py`, the model for anything held open). L1 is done (notes:
`families/living.md`): every sofa's seat cushions are a part of their own that sinks while sat on (`sink_in`).

## L1. Sofas and sectionals (11) — **Done** (in Validate › Sofas)

- **Family:** `tools/art/blender/families/living.py`
- **Validate group:** `sofas` (Validate › Sofas)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sofa` Sofa | 2x1 · floor | relax:relax | living / seating | Frame, Upholstery, Cushions, Legs | Use | `sink_in`: the seat cushions settle 2 cm while sat on (relax, doze; T0 `squash`). |
| `loveseat` Loveseat | 2x1 · floor | relax:relax doze:nap | living / seating | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sofaThree` Three-seat sofa | 3x1 · floor | relax:relax doze:nap | living / seating | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sofaDesign` Designer sofa | 2x1 · floor | relax:relax doze:nap | living / seating | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sofaThreeSeat` Three-seater sofa | 3x1 · floor | relax:relax doze:nap | living / seating | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sofaBed` Sofa bed | 2x1 · floor | relax:relax doze:nap | living / seating | Frame, Upholstery, Cushions, Legs | Held open | **Done** (in Validate › Sofas): `bed_out`, the mattress pulls out forward from under the seat while dozed on or pulled out; `BEDS` height 0.44 and `fwd` 0.6 (head on the seat). |
| `sectionalMiddle` Sectional: middle seat | 1x1 · floor | relax:relax doze:nap | living / sectionals | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sectionalArmLeft` Sectional: left arm | 1x1 · floor | relax:relax doze:nap | living / sectionals | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sectionalArmRight` Sectional: right arm | 1x1 · floor | relax:relax doze:nap | living / sectionals | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |
| `sectionalCorner` Sectional: corner | 1x1 · floor | relax:relax doze:nap | living / sectionals | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |
| `chaise` Sectional: chaise | 1x2 · floor | relax:relax doze:nap | living / sectionals | Frame, Upholstery, Cushions, Legs | Use | `sink_in`. |

## L2. Armchairs and small seats (11)

- **Family:** `tools/art/blender/families/living.py`
- **Validate group:** `armchairs` (Validate › Armchairs)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `armchair` Armchair | 1x1 · floor | sit:sit | living / seating | Frame, Upholstery | Use | `sink_in`. |
| `armchairLounge` Lounge chair | 1x1 · floor | sit:sit | living / seating | Frame, Upholstery | Use | `sink_in`. |
| `armchairLeather` Leather armchair | 1x1 · floor | sit:sit read:read | living / seating | Frame, Upholstery | Use | `sink_in`. |
| `armchairDesign` Designer lounge chair | 1x1 · floor | relax:relax doze:nap | living / seating | Frame, Upholstery | Static | Static (moulded shell). |
| `armchairWingback` Wingback armchair | 1x1 · floor | sit:sit read:read | living / seating | Frame, Upholstery | Use | `sink_in`. |
| `recliner` Recliner | 1x1 · floor | relax:relax doze:nap | living / seating | Frame, Upholstery | Use | `recline`: the back tips 25° and the footrest swings up 80° for relax and doze. Check the sitting pose against it. |
| `rockingChair` Rocking chair | 1x1 · floor | relax:relax read:read | living / seating | Wood, Cushion | Loop (E1) | `rock`: rocks ±6° while sat in (E1). Until then static. |
| `pouf` Leather pouf | 1x1 · floor | sit:sit | living / seating | Leather | Use | `sink_in`. |
| `ottoman` Tufted ottoman | 1x1 · floor | sit:sit | living / seating | Upholstery, Feet | Use | `sink_in`. |
| `floorCushion` Floor cushion | 1x1 · floor | sit:sit | living / seating | Cushion | Use | `sink_in`. |
| `beanBag` Bean bag | 1x1 · floor | flop:relax | living / seating | Bag | Use | `squash`: the bag flattens 15% for flop. |

## L3. Coffee and side tables (9)

- **Family:** `tools/art/blender/families/living.py`
- **Validate group:** `coffee tables` (Validate › Coffee Tables)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `coffeeTable` Coffee table | 2x1 · floor | magazine:read | living / tables | Top, Base | Use | `drawer_open`: the drawer slides out with a magazine for magazine. |
| `coffeeTableGlass` Glass coffee table | 2x1 · floor | magazine:read | living / tables | Top, Base | Static | Static; glass top. |
| `coffeeTableMarble` Marble side table | 1x1 · floor | magazine:read | living / tables | Top, Base | Static | Static. |
| `sideTable` Side table | 1x1 · floor | — | living / tables | Top, Base | Static | Static. |
| `coffeeTableRound` Round coffee table | 1x1 · floor | — | living / tables | Top, Base | Static | Static. |
| `coffeeTableLong` Long coffee table | 2x1 · floor | — | living / tables | Top, Base | Static | Static. |
| `nestingTables` Nesting tables | 1x1 · floor | — | living / tables | Top, Base | Static | Static. (+use) the small table slides out. |
| `sideTableRound` Round side table | 1x1 · floor | — | living / tables | Top, Base | Static | Static. |
| `consoleTable` Console table | 2x1 · floor | — | living / tables | Top, Base | Static | Static. |

## L4. Media (8)

- **Family:** `tools/art/blender/families/media.py`
- **Validate group:** `media` (Validate › Media)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `tv` Television | 1x1 · floor | watch:watch | living / media | Body, Trim | Static | Static. A lit screen is a material question (no emissive; E8 night glow). |
| `tvWall` Flat-screen TV | 2x1 · wall | watch:watch | living / media | Body, Trim | Static | Static. |
| `tvRetro` Portable TV | 1x1 · floor | watch:watch | living / media | Body, Trim | Use | `antenna`: the rabbit-ear antennae extend and turn 20° for watch. |
| `homeCinema` Home cinema | 2x1 · floor | watch:watch film:watch | living / media | Body, Trim | Use | `screen_down`: the projector screen unrolls from its case for watch and film, and rolls up at the end. |
| `stereo` Stereo system | 1x1 · floor | listen:listen dance:dance | living / media | Body, Trim | Use | `tray`: the CD tray slides out and back (`hold` 1.2 s) for listen and dance. L speaker cones pulse (E1). |
| `boombox` Boombox | 1x1 · floor | listen:listen | living / media | Body, Trim | Use | `cassette`: the cassette door swings open and shuts for listen. |
| `tvStandLow` Low TV bench | 2x1 · floor | watch:watch | living / media | Bench, TV | Use | `tv_up`: the TV rises 60 cm out of the bench for watch, and sinks back at the end. |
| `fireplaceElectric` Electric fireplace | 2x1 · floor | relax:relax | living / media | Surround, Firebox | Use | `flames`: the flame panel grows for relax (gloss, not emissive; the light is the content's). |

## L5. Shelves (9)

- **Family:** `tools/art/blender/families/living.py`
- **Validate group:** `shelves` (Validate › Shelves)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bookshelf` Bookshelf | 1x1 · floor | read:read study:read homework:read | living / shelves | Frame, Back | Use | `book_out`: one book tips out 25° from its shelf for read, study and homework, and back at the end. |
| `bookcaseLow` Low bookcase | 1x1 · floor | read:read | living / shelves | Frame, Back | Use | `book_out`. |
| `bookcaseWide` Library bookcase | 2x1 · floor | read:read study:read homework:read | living / shelves | Frame, Back | Use | `book_out`. |
| `shelvingIndustrial` Industrial shelving | 2x1 · floor | — | living / shelves | Frame, Shelves | Static | Static. |
| `bookcaseTall` Tall bookcase | 1x1 · floor | read:read study:read | living / shelves | Frame, Back | Use | `book_out`. |
| `bookcaseCorner` Corner bookcase | 1x1 · floor | read:read | living / shelves | Frame, Back | Use | `book_out`. |
| `cubeShelf` Cube shelf | 1x1 · floor | — | living / shelves | Frame, Back | Static | Static. |
| `ladderShelf` Ladder shelf | 1x1 · floor | — | living / shelves | Frame, Back | Static | Static. |
| `displayCabinet` Display cabinet | 1x1 · floor | — | living / shelves | Frame, Back | Static | Static; glass door. (+use) the door swings. |

## L6. Stairs (1)

- **Family:** `tools/art/blender/families/living.py`
- **Validate group:** `stairs` (Validate › Stairs)
- **Placeholders to match:** base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `stairs` Staircase | 1x4 · floor | — | living / stairs | Treads, Railing | Static | Static. Must match the storey height and the walkable run exactly; check with every storey and the basement. |
