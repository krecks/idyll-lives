# Kids

Things for babies and children, built by `tools/art/blender/families/kids.py` (notes: `families/kids.md`). The
crib's front rail slides down (`side_down`) and its mobile turns (`mobile`, a loop); the beds turn their duvet down
(`turn_down`, as the bedroom's); the toys open, rock, bob, squish and grow (`lid_open`, `front_open`, `rock`,
`bounce`, `train`, `blocks`, `squish`, `squash`); the study things open a drawer, tip out a book and draw or paint
themselves (`drawer_open`, `book_out`, `drawing`, `painting`). The high chair's tray is a part of its own; the train
table's track is a ring, so the train can go round it.

Kids' seats and beds keep their heights (`seats`, `beds` in `web/src/render/babylon/characters/furniture.json`; tops in
`TOPS`). Toys a child holds in the hand are drawn by the character, not the object.

**Who uses them** (content `stages`, 2026-10-21, G-45): the race car bed, rocking horse, kids' chair, kids' desk,
kids' bean bag, ball pit, play tent, toy box and toy chest are a child's (`"stages": ["child"]`): a grown-up, a teen
or an elder is refused ("Ada is too big for the rocking horse"), never picks them on their own, and the pie menu greys
them out; a kids' chair is no seat for them. The crib's *Lie in the crib* is a baby's; its feed, change, play and
cuddle are a grown-up's, with the baby in it ("Dee is too young to look after a baby", "there's no baby in the
crib"). Homework anywhere is a pupil's. The kids' bed (a single bed's mattress), the doll house, train table, play
mat, teddy, easels, chalkboard and picture books stay anyone's: grown-ups use them too. A child eats at a kids' chair
pulled up to the kids' table before the dining table (a seat made for their age first, I-kids-08).

## KD1. Baby and kids' beds (4)

- **Family:** `tools/art/blender/families/kids.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `crib` Crib | 2x1 · floor | lie:sleep feed:cuddle change:change play:play cuddle:cuddle | kids / — | Frame, Bedding, Mobile | Use | `side_down`: the front rail slides down 25 cm for feed, change and cuddle, and back up. L `mobile`: the mobile turns slowly while a grown-up plays with the baby (E1, `loops.play`). An oblong cot (a 1.18 × 0.58 m mattress), the baby lying along it (`beds.crib` `across`), since 2026-10-21 (G-40: square). |
| `kidsBed` Kids' bed | 1x2 · floor | sleep:sleep nap:nap | kids / — | Frame, Bedding | Use | `turn_down`, as the beds (rolling over low half way, 2026-10-21). |
| `kidsBedCar` Race car bed | 1x2 · floor | sleep:sleep nap:nap | kids / — | Car body, Bedding | Use | `turn_down`, as the beds; the car body is static. |
| `highChair` High chair | 1x1 · floor | sit:sit feed:feed | kids / — | Frame, Bedding | Static | Static. (+use) the tray slides out. *Sit in the high chair* is a baby's (`baby`, put in by a carer, `carried`); *Feed the baby* is done from beside it. Its legs splay to 0.54 m each way since 2026-10-21 (G-40). |

**In play** (signed off 2026-10-16):
- Against the four walls of the family house's living room: the crib's front rail slides down for feed, change and
  cuddle and back up when the use ends, with the baby lying in the crib half in every facing. The mobile turns while
  a grown-up plays with the baby and finishes its round after; it turns for play on purpose, since the crib's uses
  can't tell an awake baby from a sleeping one.
- The kids' bed and race car bed turn their duvet down for sleep and nap, a child lying in them under the game's
  quilt, and shut when the use ends. A child lies on the mattress, legs level with its top (`lieBack` scales with
  the body); the car bed's quilt ends at its bonnet (`BEDS` `foot`).
- The high chair is still in all four facings.
- Since the authenticity review (G-66, 2026-10-21): by day "Feed the baby" at the crib, with a high chair on the lot,
  is done at the high chair: the grown-up lifts the baby out of the cot (the cuddle's lift), carries them in the arms
  (`CARRY`), lays them into the high chair, feeds them standing in beside it (`STAND_BACK` -0.5, `interact`/`spell`)
  and carries them back to the cot; at night the feed stays at the crib, in the arms. "Change the baby" is done at
  the changing table (`atSlot`): the carer stands in front of it, lifts the baby out of the cot onto the pad (laid
  across it, `CHANGE`), works at them there and lays them back in the cot near the end (render only, as the cuddle).
  `play --state review-baby` (fixes/babies-crops).
- Known, not changed (until G-43 and G-66, below): the carer at the crib stands or kneels in different spots by facing, once a tile out in front
  of the changing table (slot positions and the shared "tend" pose).
- Since the authenticity review (G-14, G-05): "Feed the baby" lifts the baby out and holds them, as the cuddle does
  (`anim: cuddle`), not the gardening kneel at the rail (the bottle is the props wave's); the high chair's content
  seat 0.60, its mesh's.
- Since the authenticity review (G-43, G-51): the carer at the crib stands in front of the baby (the crib half, slot
  0) in every facing, facing it, not by facing in front of the changing table or the crib (the change at the
  changing table, G-66's `atSlot`, in front of it); the kids' bed and the race
  car bed are got into from their side (`approach: side`), sitting on the edge first.

## KD2. Toys (10)

- **Family:** `tools/art/blender/families/kids.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `toyBox` Toy box | 1x1 · floor | play:play | kids / — | Body, Lid | Use | `lid_open`: the lid swings up 100° for play; `hold` 2 s. |
| `toyChest` Toy chest | 1x1 · floor | play:play | kids / — | Body, Trim | Use | `lid_open`. Its text no longer says the lid is open (it opens only for play). |
| `dollHouse` Doll house | 1x1 · floor | dolls:tend | kids / — | Walls, Roof | Use | `front_open`: the front wall swings 120° on its side hinge for the whole of dolls. |
| `rockingHorse` Rocking horse | 1x1 · floor | ride:play | kids / — | Body, Trim | Loop (E1) | `rock`: rocks ±12° while ridden (E1), the rider riding it. Ridden astride, the hands on the handle (`rideHorse112`). |
| `playTent` Play tent | 1x1 · floor | play:play | kids / — | Canvas, Poles | Use | `flap_open`: the door flap rolls up for play. `flap_open`: the door flap rolls up for play; the child crawls in and out through it (`STEP_IN`, `tentPlay112`). |
| `ballPit` Ball pit | 1x1 · floor | jump:play | kids / — | Pit, Rim | Loop (E1) | `bounce`: a few balls bob while played in (E1). `bounce`: a few balls bob while played in (E1); the child steps in over the rim (`STEP_IN` `over`, `ballPit112`). |
| `trainTable` Train table | 2x1 · floor | trains:play | kids / — | Table, Train | Loop (E1) | `train`: the train runs round its ring of track (E1, a `spin` about the ring's middle). 1.24 m long since 2026-10-21 (G-40: 1.8). |
| `playMat` Play mat | 1x1 · floor | blocks:tend | kids / — | Mat | Use | `blocks`: a tower of three blocks grows on the mat for blocks. |
| `teddyBear` Giant teddy bear | 1x1 · floor | hug:relax | kids / — | Fur, Bow | Use | `squish`: the bear squashes a little for hug (T0 `squash`). The hugger crouches to it, arms round its neck, cheek on its head (`characters.ts` `HUGGED`). |
| `beanBagKids` Kids' bean bag | 1x1 · floor | flop:relax | kids / — | Bag | Use | `squash`: the bag flattens 15% while sat on (T0 `squash`). |

**In play** (signed off 2026-10-16):
- A child (9) played with each. Free-standing toys in all four facings in the middle of the room; the doll house,
  ball pit and play mat against all four walls. Every clip plays for its use and shuts or finishes its round after;
  the train was also seen at quarter turns in the clips bench.
- The toy box's lid is its own colour part, Lid (the box has no trim). The ball pit's parts are Pit and Rim: the
  balls are five colours, and one slot would recolour a fifth of them.
- The teddy's hugger crouches in, arms round its neck and cheek on its head, while it squishes (`hugArms`, shared
  with the social hug, with the teddy keeping the default `wrap`).
- Known, not changed: the toy box's player stands at its front corner.
- Played in from inside (G-22, 2026-10-21; characters.ts `STEP_IN`, clips/play.ts): "Jump in the ball pit" steps in
  over the rim to the pit's middle, wades and jumps in the balls, sits down in them tossing balls up and gets up
  again, the legs hidden in the balls (the child faces the pit's back, the way they stepped in); "Play in the tent"
  kneels at the open flap, crawls in, sits back on the heels to play on the mat, and crawls back out each round
  (24 s). A beanbag or the kids' chair sits the child with the feet on the floor (G-20).
- Since the authenticity review (G-14): the toy box and the toy chest are played at kneeling on the floor (`kneel`),
  not standing with the hands out over them; at the train table the players kneel up at its edge, a lane each (the
  ring of track, the little town: `STEP_IN`, `kneelTable177`/`147`/`112`, clips/low.ts), the right hand pushing the
  engine along the track and the left on the top, now and then moving a tree.
- Since the authenticity review (G-04): a child on the kids' chair at the coffee table bends over it with both hands on
  its top, drawing (`sitHandsLow177`/`147`/`112`, clips/low.ts: a top less than `LOW_TOP`, 0.2 m for a body of the
  reference height, over the seat).
- Since the authenticity review (G-43): two at the train table come to the tile in front of their own half
  (`stand_spot`) and kneel at the lane on their side, not both from one spot.

## KD3. Kids' study and art (6)

- **Family:** `tools/art/blender/families/kids.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kidsDesk` Kids' desk | 1x1 · floor | homework:write drawing:write | kids / — | Frame, Top, Chair | Use | `drawer_open`: the pencil drawer slides out for homework and drawing; `hold` 2 s. Its chair is part of the desk. |
| `kidsTable` Kids' table | 1x1 · floor | — | kids / — | Frame, Top | Static | Static. Its text says blue legs, as built. |
| `kidsChair` Kids' chair | 1x1 · floor | sit:sit draw:write homework:write | kids / — | Frame, Seat | Static | Static. Pulled in to the low table it faces while sat on (`seatAt` `low`), as far as the knees fit under it: written at with the hands on the top. |
| `kidsBookshelf` Picture-book shelf | 1x1 · floor | read:read | kids / — | Frame, Top | Use | `book_out`: a picture book tips out 20° for read. |
| `kidsChalkboard` Chalkboard easel | 1x1 · floor | draw:paint | kids / — | Frame, Board | Use | `drawing`: a chalk drawing grows on the board for draw. |
| `kidsEasel` Kids' easel | 1x1 · floor | paint:paint | kids / — | Easel | Use | `painting`: a painting grows on the paper for paint. |

**In play** (signed off 2026-10-16):
- The desk's pencil drawer slides out for homework and drawing, the child sitting at it, against all four walls; the
  picture-book shelf's middle book tips out for read, against all four walls; the chalkboard and easel stand free in
  all four facings.
- The kids' chair sat on in all four facings, and pushed up to the kids' table for "Draw at the little table" and "Do
  homework at the little table" (`needsTable`). Once seated, the child shuffles in with the chair until the top is in
  reach (`tableTuck.ts`, read from the table's drawn shape), and pushes it back while rising.
- The chalk drawing and the painting rest as a speck at their left edge and get their height and depth in the clip's
  first frames, so nothing shows on the board at rest.
- The kids' chair's seat is its own colour part, Seat.
- Known, not changed: the drawing grows from the board's right as you face it (the family's "left" is the object's
  own). The kids' chair at a coffee table (top 0.11 m over the seat) gets the desk hands, hovering over it, and a
  desk chair facing the kids' desk from behind can't come in (the desk's own chair is there) and reaches over it;
  nothing sets either up.
- Since the authenticity review (G-51): the kids' desk is sat at from behind its chair (`approach: behind`), else
  beside it, not through the desk.
