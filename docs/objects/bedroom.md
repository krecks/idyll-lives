# Bedroom

The beds are built by `tools/art/blender/families/bedroom.py` (notes: `families/bedroom.md`): the duvet's top third
turns down (`turn_down`; the bunk's upper duvet `turn_down_upper`) and the futon's back folds flat (`unfold`). The
nightstands, wardrobes and drawers are built by `families/bedroom_storage.py` (notes: `families/bedroom_storage.md`:
`drawer_open`, `lid_open`, `doors_open`, `door_open`), and the mirrors and dressing table by
`families/bedroom_dressing.py` (notes: `families/bedroom_dressing.md`: `drawer_open`; the glass is a mirror part, E6).

Mattress tops match `BEDS` (`BUNKS` for the bunk's two), and beds that get the game's blanket are in `BLANKET_BEDS`
(`web/src/render/shared/characters/seats.ts`): a bed's own duvet sits under that blanket, turned down, and the blanket
takes the duvet's colour (the "Bedding" part, as the player painted it or as made; `QUILT_PART` names the futon's own
quilt and the wall bed's throw), so a bed keeps its colours with someone in it. Residents sleep the night in pyjamas
(a long top and trousers, barefoot) and nap in their day clothes with the shoes off; on a lounger or a sofa they doze
as they are (`wearFor`, `outfit.ts`). Every bed of BR1
but the daybed is in `BLANKET_BEDS`, and all nine have their mattress tops in `BEDS`. Sleepers lie with the heels
and buttocks about 4 cm into the mattress and the legs on it; a child lies on it too (`lieBack` scales with the
body). Things with drawers and doors follow the fridge (hinge, `hold`; see `kitchen.md`).

## BR1. Beds (9)

- **Family:** `tools/art/blender/families/bedroom.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bed` King-size bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`: the top third of the duvet folds back toward the foot as they get in (sleep, nap) and back up when they get up, under the game's blanket; half way it is gathered low, as cloth rolls, not a board standing up (2026-10-21). Renamed from "Double bed": 1.90 m wide with a 1.72 m mattress, a king (reference kind `bedKing`, Q-08). |
| `bedSingle` Single bed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedBunk` Bunk bed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down` on the lower bunk, `turn_down_upper` on the upper, each for its own sleeper (`slot`), the upper one 5 s in, once its sleeper has climbed in (`delay`, A-bedroom-06); the ladder is static. The upper bunk is not an elder's (`upperStages`: with the lower bunk taken, an elder is refused, "too old for the top bunk"). Its sleeper climbs the ladder from the tile in front and in through the gap in the guard rail (`bunkUp172`/`147`/`112`), and back down it getting up (`bunkDown…`, sim-core waiting for them); the lower bunk is got into from its foot as any bed. |
| `bedFuton` King futon | 2x2 · floor | sleep:sleep nap:nap unfold:fetch foldAway:fetch relax:relax | bedroom / beds | Frame, Headboard, Bedding | Held open | `unfold`: the back folds flat into a bed while slept in (C1: `leaves: open` on sleep, nap and Unfold, as the wall bed); Fold away closes it. Folded, a sofa: Sit and relax on its front edge (`when: closed`, content `seat` 0.40 m, forward 0.68). Renamed from "Futon bed": a king-size futon, 1.96 m wide (kind `bedLow`, now to 1.95 m, Q-08). |
| `bedKing` Upholstered king bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedDouble` Double bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedQueenUpholstered` Upholstered queen bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`. |
| `bedCanopy` Four-poster bed | 2x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Use | `turn_down`; the canopy drapes are static. |
| `bedDaybed` Daybed | 1x2 · floor | sleep:sleep nap:nap | bedroom / beds | Frame, Headboard, Bedding | Static | Static (a sofa by day). The sleeper's quilt is tucked in at the mattress's edges, inside its back, arms and rail (`BEDS` `tuck`, A-bedroom-05). |

**In play** (signed off 2026-10-16):
- Each bed napped in, standing free in all four facings (`nap` plays the same clips as the 480-minute sleep). The
  duvet turns down as the sleeper gets in and back up as they get up (since 2026-10-21, review A-bedroom-01, rolling
  over low half way, `fixes/motions/g1718/clips-bed*-all-t0.5.png`, not standing up as a board). With two sleepers (double, king, oak double,
  upholstered queen, four-poster) both quilts cover the turned-down duvet and nothing pokes through.
- The bunk turns down the lower duvet for slot 0 and the upper for slot 1, each shutting when its sleeper gets up.
  The futon's back folds flat for a nap or a night and the last one up folds it away (as the wall bed); Unfold leaves
  it open and Fold away closes it.
- The lying pose was checked on every bed from the side at mattress height, and a child on the daybed and the bunk:
  heels and buttocks about 4 cm into the mattress, thighs and calves on it or 1–2 cm proud (a straight leg can't lie
  flatter). Getting up from the daybed was only half seen.
- The four-poster's duvet foot stands 1.5 cm past the mattress's end, so it doesn't flicker. The daybed's back
  cushions are thin and close to the back, so a sleeper's shoulder and hip stay out of them.
- Known, not changed (until G-51, below): two residents getting up from the same bed stand on the same spot for a moment (sim). The oak
  double's "Headboard" colours its teak slats; the posts and rails go with the frame. The daybed's back runs along
  a long side, so stood head end to the wall (as built) it doesn't read as a sofa.
- Since the authenticity review (G-04): the daybed's sleeper gets the quilt (`BLANKET_BEDS`), as the other beds do.
- Since the authenticity review (G-51): every bed here but the futon (a sofa by day, sat on from the front) is got
  into from its side (`approach: side`): the sleeper
  comes to the tile beside their own half, by the foot row of the middle (a double's two on its two sides, a single's
  from the side nearer them, the bunk's lower one from its ladder side), sits on the edge facing out 0.2 m in from it
  (`SIDE_IN`), and turns to lie along it as they lie back; getting up, sim-core turns them out to that side and they
  sit up there and step off (the two of a double on their own sides, not on one spot). With both sides blocked they
  get in over the foot (`LIE_ENTRY`), as before. The bunk's upper sleeper comes from the front, round its foot and up the ladder (G-44).

## BR2. Nightstands and small storage (8)

- **Family:** `tools/art/blender/families/bedroom_storage.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `nightstand` Nightstand | 1x1 · floor | — | bedroom / storage | Body, Handles | Static | Static. |
| `nightstandModern` Modern nightstand | 1x1 · floor | — | bedroom / nightstands | Body, Handles | Static | Static. 0.47 m deep (it was 0.63, M-bedroom-02). |
| `nightstandRound` Drum nightstand | 1x1 · floor | — | bedroom / nightstands | Body, Handles | Static | Static. |
| `nightstandDrawers` Three-drawer nightstand | 1x1 · floor | snack:fetch | bedroom / nightstands | Body, Handles | Use | `drawer_open`: the top drawer, a biscuit tin, chocolate and a paperback in it, slides out for the snack (`SNACK_DRAWER`); `hold` 2.5 s. A snack from the drawer (fetched, then eaten sitting), not an outfit. 0.47 m deep (it was 0.63, M-bedroom-02). |
| `nightstandLamp` Nightstand with lamp | 1x1 · floor | — | bedroom / storage | Body, Shade | Static | Static; the shade glows at night (N, E3/E8), the light at the shade. |
| `cedarChest` Blanket chest | 1x1 · floor | takeBlanket:fetch | bedroom / storage | Body, Handles | Use | `lid_open`: the lid swings up 100° for takeBlanket; `hold` 2 s. |
| `laundryHamper` Laundry hamper | 1x1 · floor | sortWashing:clean | bedroom / storage | Hamper | Use | `lid_open`: the lid lifts for sortWashing. |
| `benchBedEnd` Bed-end bench | 2x1 · floor | sit:sit | bedroom / storage | Frame, Cushion | Static | Static. |

**In play** (signed off 2026-10-16):
- All eight in all four facings, standing free and against the wall. The three-drawer nightstand's top drawer slides
  out for "Grab a snack from the drawer" (2026-10-21; "Pick an outfit" before), the blanket chest's lid lifts for "Take out a blanket" and the hamper's for "Sort the
  washing", each shutting after its use. The bed-end bench is sat on; the rest are still.
- The nightstand lamp lights at night: the shade glows, the top lit warm; coloured by day, the shade shows its glow's
  colour at night. Its light (content `light.at`) sits at the shade in every facing, against a wall and free.
- The blanket chest's lid underside stands 2 mm below the edge band, so it doesn't flicker with the lid up.

## BR3. Wardrobes and drawers (6)

- **Family:** `tools/art/blender/families/bedroom_storage.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `dresser` Chest of drawers | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`: the middle drawer slides out 30 cm for outfit; `hold` 2.5 s. `drawer_open`: the middle drawer slides out 30 cm for outfit, open while they change. |
| `wardrobe` Wardrobe | 2x1 · floor | outfit:clean | bedroom / storage | Body, Doors, Handles | Use | `doors_open`: both doors swing out 100° for outfit; `hold` 3 s. `doors_open`: both doors swing out 100° for outfit, open while they change. |
| `chestOfDrawers` Tall chest of drawers | 1x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`. 0.51 m deep (it was 0.66, M-bedroom-02). |
| `dresserLow` Low dresser | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `drawer_open`. |
| `wardrobeSingle` Single wardrobe | 1x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `door_open`. 0.60 m wide, its one door 0.58 (it was 0.90 and 0.88, M-bedroom-03). |
| `armoire` Armoire | 2x1 · floor | outfit:clean | bedroom / storage | Body, Handles | Use | `doors_open`. |

**In play** (signed off 2026-10-16):
- Each standing free in all four facings, used for "Pick an outfit": the dresser's, tall chest's and low dresser's
  drawer slides out 30 cm; the wardrobe's and armoire's doors (`doors_open`) and the single wardrobe's door
  (`door_open`) swing out onto the rail of clothes. Each shuts after its `hold`.
- 2026-10-21 (G-69): "Pick an outfit" changes the clothes (content `changesOutfit`): halfway through, with the
  drawer or the doors open (held for the whole use now, no `hold`), the resident is in their next outfit (three
  in turn: the creator's, then two made from the id, `docs/characters-format.md`), walks off in it and keeps it,
  saved, until they change again. The use lasts its 10 minutes even with comfort full.
- The wardrobe's Body colours the white carcass and its birch doors are a part of their own, Doors.

## BR4. Dressing and mirrors (4)

- **Family:** `tools/art/blender/families/bedroom_dressing.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `mirror` Cheval mirror | 1x1 · floor | speech:talk primp:idle | bedroom / dressing | Frame | Static | Static; mirror part (E6). Renamed from "Standing mirror" (the floor mirror's name too); checks the look as well as the speech. |
| `dressingTable` Dressing table | 1x1 · floor | getReady:wash | bedroom / dressing | Table, Frame | Use | `drawer_open`: the drawer slides out for getReady; mirror part (E6). |
| `mirrorWall` Wall mirror | 1x1 · wall | speech:talk primp:idle | bedroom / dressing | Frame | Static | Static; mirror part (E6). Checks the look as well as the speech. |
| `mirrorFloor` Standing mirror | 1x1 · floor | primp:idle speech:talk | bedroom / dressing | Frame | Static | Static; mirror part (E6). Practise a speech as well. |

**In play** (signed off 2026-10-16):
- The cheval mirror ("Practise a speech") and the standing mirror ("Check the mirror") standing free in all four
  facings, the wall mirror on all four walls: the resident stands in front of the glass, which reflects the room.
  Practising a speech, they look into the glass with the head level (`LEVEL_GAZE`), not down at the thing as at a
  plant (I-bedroom-08).
- The dressing table's drawer slides out for "Get ready", the resident on the pouf facing the mirror, and shuts after
  its `hold`.
- The wall mirror hides with its wall when the wall is cut down for the camera (each wall thing knows which views
  drop its wall, `BabylonRenderer.ts`).
- Since the authenticity review (G-04): getting ready at the dressing table is brushing the hair in the mirror
  (`brushHair`, clips/self.ts, the seated upper body): four strokes from the crown down the right side, the head
  tilting into each, then the left hand smoothing the other side; the brush is mimed in the closed hand (a hand prop is
  I-held-01's).
