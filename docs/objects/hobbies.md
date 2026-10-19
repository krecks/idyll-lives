# Hobbies

The nineteen hobby models are built by `tools/art/blender/families/hobbies.py` (notes: `families/hobbies.md`), each
at the height the game seats residents, so seats and stools stay where it puts them. The dartboard hangs at its
real height. Most hobbies move a part while a resident uses them: the piano's fallboard, the chess pawn, the pool
rack, the vice, the telescope's tube, the presser foot and the scrapbook (use clips); the drums' cymbals, the
foosball rods, the arcade joystick, the sewing machine's handwheel and the pottery wheel turn in a loop while in use
(E1); the darts, the painting, the jigsaw patch, the pot and the flask's liquid grow from nothing.

Instruments a resident holds (the guitars and the ukulele) are a part of their own: while played, the object hides
it on its stand (T0 `squash` to 0, `off_stand`) and the game draws the same part in the player's hands, in the
object's colours (content `"held": {"part": "guitar", "grip": "guitar"}`, `characters.ts` `writeHeld`): the neck in
the left hand, the body against the belly. That part costs one draw call more (3). The piano is an upright with its
back on the wall and its pianist facing the case (content `seat.facing: "back"`); the digital piano and the sewing
table face the room, as tables do.

## H1. Music (6)

- **Family:** `tools/art/blender/families/hobbies.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `piano` Piano | 2x1 · floor | play:music | hobbies / music | Body, Trim | Use | `fallboard`: the key cover lifts and the music desk unfolds for play. |
| `ukulele` Ukulele | 1x1 · floor | strum:music practise:music | hobbies / music | Body & neck, Top, Stand | Use | `off_stand`: the ukulele leaves its stand while strummed and is drawn in the player's hands (content `held`). |
| `guitarElectric` Electric guitar | 1x1 · floor | jam:music practise:music | hobbies / music | Guitar, Amp | Use | `off_stand`, as the ukulele; the amp is static. |
| `keyboardDigital` Digital piano | 1x1 · floor | play:music | hobbies / music | Body, Trim | Static | Static. |
| `drumKit` Drum kit | 1x1 · floor | drum:music practise:music | hobbies / music | Shells, Hardware | Loop (E1) | `cymbals`: the crash and ride cymbals rock while drummed (E1); the hi-hat is still. |
| `guitarAcoustic` Acoustic guitar | 1x1 · floor | strum:music practise:music | hobbies / music | Body, Top, Stand | Use | `off_stand`, as the ukulele. |

**In play** (signed off 2026-10-16):
- Checked in all four facings (the piano against three walls and standing free), every part coloured, played by
  day and at night, on WebGPU and WebGL2, on Medium and Low.
- The piano's fallboard lifts and its music desk tilts for play, and shut when the use ends.
- The ukulele and both guitars leave their stands for strum, jam and practise and are back on the stand when the use
  ends; the drum kit's cymbals stop with the use. The digital piano is still.
- The acoustic guitar's and ukulele's soundboard has its own colour, "Top"; "Body" (back and sides) and "Body & neck"
  say what they colour. The electric's "Guitar" colours its whole body. Rechecked from front and back, on the stand
  and played in all four facings; in the colour studio "In use" shows the empty stand.

## H2. Games (6)

- **Family:** `tools/art/blender/families/hobbies.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `chessTable` Chess table | 1x1 · floor | chess:play | hobbies / games | Table, Chair | Use | `move`: a pawn slides one square for chess. |
| `poolTable` Pool table | 3x2 · floor | pool:play trickShots:play | hobbies / games | Frame, Cloth | Use | `rack_off`: the triangle lifts off the balls for pool and trick shots. |
| `foosball` Foosball table | 2x1 · floor | foosball:play | hobbies / games | Table, Trim | Loop (E1) | `rods`: the two outer rods kick ±55° while played (E1); the inner two are still. |
| `dartboard` Dartboard | 1x1 · wall | darts:play | hobbies / games | Board, Cabinet | Use | `darts`: three darts grow in the board for darts. |
| `arcadeCabinet` Arcade cabinet | 1x1 · floor | arcade:play | hobbies / games | Cabinet, Art | Loop (E1) | `joystick`: the stick waggles while played (E1). The screen and marquee glow at night (E8). |
| `puzzleTable` Jigsaw table | 1x1 · floor | jigsaw:play | hobbies / games | Table, Stool | Use | `pieces`: a patch of finished puzzle grows for jigsaw. |

**In play** (signed off 2026-10-16):
- Checked standing free, the dartboard on three inside walls and an outside face, the arcade cabinet against three
  walls and free.
- Every clip plays for its use and stops or shuts with it.
- The chess table's and jigsaw table's second colour is "Chair" and "Stool": it colours the chess chair and the
  jigsaw stool; the chessboard's rim is the table's wood.
- The darts are thrown from the tile in front of the board, half a metre from the wall.
- With the cutaway dropping its wall, the dartboard hides with the wall and can still be used.

## H3. Crafts and science (7)

- **Family:** `tools/art/blender/families/hobbies.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `easel` Easel | 1x1 · floor | paint:paint | hobbies / crafts | Wood, Metal | Use | `painting`: a painting grows on the canvas for paint. |
| `workbench` Workbench | 2x1 · floor | tinker:tinker | hobbies / crafts | Wood, Metal | Use | `vice`: the vice jaw slides shut on a plank for tinker. |
| `telescope` Telescope | 1x1 · floor | stargaze:look | hobbies / crafts | Tube, Tripod | Use | `aim`: the tube tilts up 35° for stargaze and comes back down. |
| `sewingTable` Sewing table | 1x1 · floor | sew:tinker mend:tinker | hobbies / crafts | Table, Legs, Machine | Use + Loop (E1) | `presser`: the presser foot lowers for sew and mend, and the handwheel turns in a loop (`handwheel`, E1). |
| `chemistrySet` Chemistry bench | 1x1 · floor | experiment:tinker | hobbies / crafts | Bench, Glassware | Use | `bubble`: the liquid rises in the flask for experiment. |
| `potteryWheel` Pottery wheel | 1x1 · floor | throw:tinker | hobbies / crafts | Wheel, Stool | Use + Loop (E1) | `clay`: the pot grows from a lump for throw, and the wheel head spins in a loop (`wheel`, E1). |
| `craftTable` Craft table | 2x1 · floor | scrapbook:tinker | hobbies / crafts | Wood, Metal | Use | `book_open`: the scrapbook cover swings open for scrapbook. |

**In play** (signed off 2026-10-16):
- Checked standing free (the workbench also against three walls); every clip plays for its use and shuts when the
  resident moves on.
- The sewing table's "Table" colours its top and chair; "Legs" colours its legs, apron and drawer (dark wood as
  made).
- At rest the pottery wheel's lump sits lower than a full pot.
