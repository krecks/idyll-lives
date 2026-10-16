# Kids

Things for babies and children. Kids' seats and beds keep their heights (`SEATS`, `BEDS`). Toys a child holds in
the hand are drawn by the character, not the object.

All three tasks are done (2026-10-14): `families/kids.py` (notes: `families/kids.md`), in Validate › Baby And
Beds, Toys and Kids Study. The crib's mobile and the high chair's tray are parts of their own; the train
table's track is a ring, so the train can go round it.

## KD1. Baby and kids' beds (4) — **Done**, signed off 2026-10-16 (back in kids / —)

- **Family:** `tools/art/blender/families/kids.py`
- **Validate group:** `baby and beds` (Validate › Baby And Beds)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `crib` Crib | 2x1 · floor | lie:sleep feed:tend change:clean play:play cuddle:cuddle | kids / — | Frame, Bedding, Mobile | Use | `side_down`: the front rail slides down 25 cm for feed, change and cuddle, and back up. L `mobile`: the mobile turns slowly while the baby lies in it (E1). |
| `kidsBed` Kids' bed | 1x2 · floor | sleep:sleep nap:nap | kids / — | Frame, Bedding | Use | `turn_down`, as the beds. |
| `kidsBedCar` Race car bed | 1x2 · floor | sleep:sleep nap:nap | kids / — | Car body, Bedding | Use | `turn_down`; the car body is static. |
| `highChair` High chair | 1x1 · floor | — | kids / — | Frame, Bedding | Static | Static. (+use) the tray slides out. |

## KD2. Toys (10) — **Done**, signed off 2026-10-16 (back in kids / —)

- **Family:** `tools/art/blender/families/kids.py`
- **Validate group:** `toys` (Validate › Toys)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `toyBox` Toy box | 1x1 · floor | play:play | kids / — | Body, Lid | Use | `lid_open`: the lid swings up 100° for play; `hold` 2 s. |
| `toyChest` Toy chest | 1x1 · floor | play:play | kids / — | Body, Trim | Use | `lid_open`. |
| `dollHouse` Doll house | 1x1 · floor | dolls:tend | kids / — | Walls, Roof | Use | `front_open`: the front wall swings 120° on its side hinge for the whole of dolls. |
| `rockingHorse` Rocking horse | 1x1 · floor | ride:play | kids / — | Body, Trim | Loop (E1) | `rock`: rocks ±12° while ridden (E1). Until then U: tips forward 8° and holds. |
| `playTent` Play tent | 1x1 · floor | play:play | kids / — | Canvas, Poles | Use | `flap_open`: the door flap rolls up for play. |
| `ballPit` Ball pit | 1x1 · floor | jump:play | kids / — | Pit, Balls | Loop (E1) | `bounce`: a few balls bob while played in (E1). Until then static. |
| `trainTable` Train table | 2x1 · floor | trains:play | kids / — | Table, Train | Loop (E1) | `train`: the train runs round its ring of track (E1, a `spin` about the ring's middle). Until then static. |
| `playMat` Play mat | 1x1 · floor | blocks:tend | kids / — | Mat | Use | `blocks`: a tower of three blocks grows on the mat for blocks. |
| `teddyBear` Giant teddy bear | 1x1 · floor | hug:relax | kids / — | Fur, Bow | Use | `squish`: the bear squashes a little for hug (T0 `squash`). |
| `beanBagKids` Kids' bean bag | 1x1 · floor | flop:relax | kids / — | Bag | Use | `squash`: the bag flattens 15% while sat on (T0 `squash`). |

## KD3. Kids' study and art (6) — **Done**, signed off 2026-10-16 (back in kids / —)

- **Family:** `tools/art/blender/families/kids.py`
- **Validate group:** `kids study` (Validate › Kids Study)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kidsDesk` Kids' desk | 1x1 · floor | homework:write drawing:write | kids / — | Frame, Top | Use | `drawer_open` for homework and drawing. |
| `kidsTable` Kids' table | 1x1 · floor | — | kids / — | Frame, Top | Static | Static. |
| `kidsChair` Kids' chair | 1x1 · floor | sit:sit draw:write homework:write | kids / — | Frame, Seat | Static | Static. |
| `kidsBookshelf` Picture-book shelf | 1x1 · floor | read:read | kids / — | Frame, Top | Use | `book_out`: a picture book tips out 20° for read. |
| `kidsChalkboard` Chalkboard easel | 1x1 · floor | draw:paint | kids / — | Frame, Board | Use | `drawing`: a chalk drawing grows on the board for draw. |
| `kidsEasel` Kids' easel | 1x1 · floor | paint:paint | kids / — | Easel | Use | `painting`: a painting grows on the paper for paint. |
