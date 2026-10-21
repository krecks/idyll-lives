# Bedroom

The beds are built by `tools/art/blender/families/bedroom.py` (notes: `families/bedroom.md`): the duvet's top third
turns down (`turn_down`; the bunk's upper duvet `turn_down_upper`) and the futon's back folds flat (`unfold`). The
nightstands, wardrobes and drawers are built by `families/bedroom_storage.py` (notes: `families/bedroom_storage.md`:
`drawer_open`, `lid_open`, `doors_open`, `door_open`), and the mirrors and dressing table by
`families/bedroom_dressing.py` (notes: `families/bedroom_dressing.md`: `drawer_open`; the glass is a mirror part, E6).

Mattress tops match `BEDS` (`BUNKS` for the bunk's two), and beds that get the game's blanket are in `BLANKET_BEDS`
(`web/src/render/babylon/characters.ts`): a bed's own duvet sits under that blanket, turned down. Every bed of BR1
but the daybed is in `BLANKET_BEDS`, and all nine have their mattress tops in `BEDS`. Sleepers lie with the heels
and buttocks about 4 cm into the mattress and the legs on it; a child lies on it too (`lieBack` scales with the
body). Things with drawers and doors follow the fridge (hinge, `hold`; see `kitchen.md`).

## BR1. Beds (9)

- **Family:** `tools/art/blender/families/bedroom.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bed` Double bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`: the top third of the duvet folds back toward the foot as they get in (sleep, nap) and back up when they get up, under the game's blanket. |
| `bedSingle` Single bed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedBunk` Bunk bed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down` on the lower bunk, `turn_down_upper` on the upper, each for its own sleeper (`slot`); the ladder is static. The upper bunk is not an elder's (`upperStages`: with the lower bunk taken, an elder is refused, "too old for the top bunk"). |
| `bedFuton` Futon bed | 2x2 · floor | sleep:sleep nap:nap unfold:fetch foldAway:fetch | bedroom / beds | Frame, Headboard, Bedding | Held open | `unfold`: the back folds flat into a bed while slept in (C1: `leaves: open` on sleep, nap and Unfold, as the wall bed); Fold away closes it. |
| `bedKing` Upholstered king bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedDouble` Double bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedQueenUpholstered` Upholstered queen bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedCanopy` Four-poster bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`; the canopy drapes are static. |
| `bedDaybed` Daybed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Static | Static (a sofa by day). |

**In play** (signed off 2026-10-16):
- Each bed napped in, standing free in all four facings (`nap` plays the same clips as the 480-minute sleep). The
  duvet turns down as the sleeper gets in and back up as they get up. With two sleepers (double, king, oak double,
  upholstered queen, four-poster) both quilts cover the turned-down duvet and nothing pokes through.
- The bunk turns down the lower duvet for slot 0 and the upper for slot 1, each shutting when its sleeper gets up.
  The futon's back folds flat for a nap or a night and the last one up folds it away (as the wall bed); Unfold leaves
  it open and Fold away closes it.
- The lying pose was checked on every bed from the side at mattress height, and a child on the daybed and the bunk:
  heels and buttocks about 4 cm into the mattress, thighs and calves on it or 1–2 cm proud (a straight leg can't lie
  flatter). Getting up from the daybed was only half seen.
- The four-poster's duvet foot stands 1.5 cm past the mattress's end, so it doesn't flicker. The daybed's back
  cushions are thin and close to the back, so a sleeper's shoulder and hip stay out of them.
- Known, not changed: two residents getting up from the same bed stand on the same spot for a moment (sim). The oak
  double's "Headboard" colours its teak slats; the posts and rails go with the frame. The daybed's back runs along
  a long side, so stood head end to the wall (as built) it doesn't read as a sofa.

## BR2. Nightstands and small storage (8)

- **Family:** `tools/art/blender/families/bedroom_storage.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `nightstand` Nightstand | 1x1 · floor | — | bedroom / storage | Body, Handles | Static | Static. |
| `nightstandModern` Modern nightstand | 1x1 · floor | — | bedroom / nightstands | Body, Handles | Static | Static. |
| `nightstandRound` Drum nightstand | 1x1 · floor | — | bedroom / nightstands | Body, Handles | Static | Static. |
| `nightstandDrawers` Three-drawer nightstand | 1x1 · floor | outfit:clean | bedroom / nightstands | Body, Handles | Use | `drawer_open`: the top drawer slides out for outfit; `hold` 2.5 s. |
| `nightstandLamp` Nightstand with lamp | 1x1 · floor | — | bedroom / storage | Body, Shade | Static | Static; the shade glows at night (N, E3/E8), the light at the shade. |
| `cedarChest` Blanket chest | 1x1 · floor | takeBlanket:fetch | bedroom / storage | Body, Handles | Use | `lid_open`: the lid swings up 100° for takeBlanket; `hold` 2 s. |
| `laundryHamper` Laundry hamper | 1x1 · floor | sortWashing:clean | bedroom / storage | Hamper | Use | `lid_open`: the lid lifts for sortWashing. |
| `benchBedEnd` Bed-end bench | 2x1 · floor | sit:sit | bedroom / storage | Frame, Cushion | Static | Static. |

**In play** (signed off 2026-10-16):
- All eight in all four facings, standing free and against the wall. The three-drawer nightstand's top drawer slides
  out for "Pick an outfit", the blanket chest's lid lifts for "Take out a blanket" and the hamper's for "Sort the
  washing", each shutting after its use. The bed-end bench is sat on; the rest are still.
- The nightstand lamp lights at night: the shade glows, the top lit warm; coloured by day, the shade shows its glow's
  colour at night. Its light (content `light.at`) sits at the shade in every facing, against a wall and free.
- The blanket chest's lid underside stands 2 mm below the edge band, so it doesn't flicker with the lid up.

## BR3. Wardrobes and drawers (6)

- **Family:** `tools/art/blender/families/bedroom_storage.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `dresser` Chest of drawers | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`: the middle drawer slides out 30 cm for outfit; `hold` 2.5 s. |
| `wardrobe` Wardrobe | 2x1 · floor | outfit:clean | bedroom / storage | Body, Doors, Handles | Use | `doors_open`: both doors swing out 100° for outfit; `hold` 3 s. |
| `chestOfDrawers` Tall chest of drawers | 1x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`. |
| `dresserLow` Low dresser | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`. |
| `wardrobeSingle` Single wardrobe | 1x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `door_open`. |
| `armoire` Armoire | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `doors_open`. |

**In play** (signed off 2026-10-16):
- Each standing free in all four facings, used for "Pick an outfit": the dresser's, tall chest's and low dresser's
  drawer slides out 30 cm; the wardrobe's and armoire's doors (`doors_open`) and the single wardrobe's door
  (`door_open`) swing out onto the rail of clothes. Each shuts after its `hold`.
- The wardrobe's Body colours the white carcass and its birch doors are a part of their own, Doors.

## BR4. Dressing and mirrors (4)

- **Family:** `tools/art/blender/families/bedroom_dressing.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `mirror` Standing mirror | 1x1 · floor | speech:talk | bedroom / dressing | Frame | Static | Static; mirror part (E6). |
| `dressingTable` Dressing table | 1x1 · floor | getReady:wash | bedroom / dressing | Table, Frame | Use | `drawer_open`: the drawer slides out for getReady; mirror part (E6). |
| `mirrorWall` Wall mirror | 1x1 · wall | speech:talk | bedroom / dressing | Frame | Static | Static; mirror part (E6). |
| `mirrorFloor` Standing mirror | 1x1 · floor | primp:look | bedroom / dressing | Frame | Static | Static; mirror part (E6). |

**In play** (signed off 2026-10-16):
- The cheval mirror ("Practise a speech") and the standing mirror ("Check the mirror") standing free in all four
  facings, the wall mirror on all four walls: the resident stands in front of the glass, which reflects the room.
- The dressing table's drawer slides out for "Get ready", the resident on the pouf facing the mirror, and shuts after
  its `hold`.
- The wall mirror hides with its wall when the wall is cut down for the camera (each wall thing knows which views
  drop its wall, `BabylonRenderer.ts`).
