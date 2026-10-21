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
| `crib` Crib | 2x1 · floor | lie:sleep feed:tend change:clean play:play cuddle:cuddle | kids / — | Frame, Bedding, Mobile | Use | `side_down`: the front rail slides down 25 cm for feed, change and cuddle, and back up. L `mobile`: the mobile turns slowly while a grown-up plays with the baby (E1, `loops.play`). |
| `kidsBed` Kids' bed | 1x2 · floor | sleep:sleep nap:nap | kids / — | Frame, Bedding | Use | `turn_down`, as the beds. |
| `kidsBedCar` Race car bed | 1x2 · floor | sleep:sleep nap:nap | kids / — | Car body, Bedding | Use | `turn_down`; the car body is static. |
| `highChair` High chair | 1x1 · floor | — | kids / — | Frame, Bedding | Static | Static. (+use) the tray slides out. It has no use or sit yet; its `seatAt` would pull it in to the table it faces while sat on, as far as the knees fit under it. |

**In play** (signed off 2026-10-16):
- Against the four walls of the family house's living room: the crib's front rail slides down for feed, change and
  cuddle and back up when the use ends, with the baby lying in the crib half in every facing. The mobile turns while
  a grown-up plays with the baby and finishes its round after; it turns for play on purpose, since the crib's uses
  can't tell an awake baby from a sleeping one.
- The kids' bed and race car bed turn their duvet down for sleep and nap, a child lying in them under the game's
  quilt, and shut when the use ends. A child lies on the mattress, legs level with its top (`lieBack` scales with
  the body); the car bed's quilt ends at its bonnet (`BEDS` `foot`).
- The high chair is still in all four facings. Nobody can eat from it: feeding a baby in a chair needs the sim first
  (a baby is only ever placed lying).
- Known, not changed: the carer at the crib stands or kneels in different spots by facing, once a tile out in front
  of the changing table (slot positions and the shared "tend" pose).

## KD2. Toys (10)

- **Family:** `tools/art/blender/families/kids.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `toyBox` Toy box | 1x1 · floor | play:play | kids / — | Body, Lid | Use | `lid_open`: the lid swings up 100° for play; `hold` 2 s. |
| `toyChest` Toy chest | 1x1 · floor | play:play | kids / — | Body, Trim | Use | `lid_open`. |
| `dollHouse` Doll house | 1x1 · floor | dolls:tend | kids / — | Walls, Roof | Use | `front_open`: the front wall swings 120° on its side hinge for the whole of dolls. |
| `rockingHorse` Rocking horse | 1x1 · floor | ride:play | kids / — | Body, Trim | Loop (E1) | `rock`: rocks ±12° while ridden (E1), the rider riding it. |
| `playTent` Play tent | 1x1 · floor | play:play | kids / — | Canvas, Poles | Use | `flap_open`: the door flap rolls up for play. |
| `ballPit` Ball pit | 1x1 · floor | jump:play | kids / — | Pit, Rim | Loop (E1) | `bounce`: a few balls bob while played in (E1). |
| `trainTable` Train table | 2x1 · floor | trains:play | kids / — | Table, Train | Loop (E1) | `train`: the train runs round its ring of track (E1, a `spin` about the ring's middle). |
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

## KD3. Kids' study and art (6)

- **Family:** `tools/art/blender/families/kids.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kidsDesk` Kids' desk | 1x1 · floor | homework:write drawing:write | kids / — | Frame, Top, Chair | Use | `drawer_open`: the pencil drawer slides out for homework and drawing; `hold` 2 s. Its chair is part of the desk. |
| `kidsTable` Kids' table | 1x1 · floor | — | kids / — | Frame, Top | Static | Static. |
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
