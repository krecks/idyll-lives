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
the left hand, the body against the belly. The player's own clips hold it across the body (`strum`, `strumShort` for
the ukulele, higher; G-21): the neck out to the left in the plane of the body, 30° up, the right hand strumming over
the sound hole. That part costs one draw call more (3). The piano is an upright with its
back on the wall and its pianist facing the case (content `seat.facing: "back"`); the digital piano and the sewing
table face the room, as tables do.

## H1. Music (6)

- **Family:** `tools/art/blender/families/hobbies.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `piano` Piano | 2x1 · floor | play:music | hobbies / music | Body, Trim | Use | `fallboard`: the key cover lifts and the music desk unfolds for play, 0.9 s in, once the pianist has sat down (`delay`). |
| `ukulele` Ukulele | 1x1 · floor | strum:music practise:music | hobbies / music | Body & neck, Top, Stand | Use | `off_stand`: the ukulele leaves its stand while strummed and is drawn in the player's hands (content `held`). |
| `guitarElectric` Electric guitar | 1x1 · floor | jam:music practise:music | hobbies / music | Guitar, Amp | Use | `off_stand`, as the ukulele; the amp is static. A 0.9 m solid body (a 0.648 m scale) on a 1.0 m stand since 2026-10-21 (G-40: a quarter too long). |
| `keyboardDigital` Digital piano | 2x1 · floor | play:music | hobbies / music | Body, Trim | Static | Static. A console of 88 keys at 0.745 m against the wall, the player facing it (content `seat` 0.48, `facing: back`), since 2026-10-21 (G-29: a three-octave keyboard on an X-stand, the keys at 0.845). |
| `drumKit` Drum kit | 2x2 · floor | drum:music practise:music | hobbies / music | Shells, Hardware | Use, repeating (E1) | `cymbals`: the crash and ride cymbals rock while drummed, from 0.9 s in, once the drummer has sat down (`delay`, `every` 0.9 s; it was a loop from the first frame); the hi-hat is still. Spread over 2 × 2 since 2026-10-21 (G-40), the hi-hat and snare at the drummer's left, the throne 0.4 m behind the middle. |
| `guitarAcoustic` Acoustic guitar | 1x1 · floor | strum:music practise:music | hobbies / music | Body, Top, Stand | Use | `off_stand`, as the ukulele. A 1.05 m dreadnought (a 0.645 m scale) on a 1.16 m stand since 2026-10-21 (G-40). |

**In play** (signed off 2026-10-16):
- Checked in all four facings (the piano against three walls and standing free), every part coloured, played by
  day and at night, on WebGPU and WebGL2, on Medium and Low.
- The piano's fallboard lifts and its music desk tilts for play, and shut when the use ends.
- Since 2026-10-21 (review A-hobbies-02) the seated hobbies' parts wait 0.9 s, while the resident sits down
  (`sitDown`), as the weight bench's bar does: the fallboard and the cymbals here, the pawn and the jigsaw patch, the
  presser and handwheel, the pot and wheel and the scrapbook below (`fixes/motions/g1718/play-stage.jsonl`).
- The ukulele and both guitars leave their stands for strum, jam and practise and are back on the stand when the use
  ends; the drum kit's cymbals stop with the use. The digital piano is still.
- The acoustic guitar's and ukulele's soundboard has its own colour, "Top"; "Body" (back and sides) and "Body & neck"
  say what they colour. The electric's "Guitar" colours its whole body. Rechecked from front and back, on the stand
  and played in all four facings; in the colour studio "In use" shows the empty stand.

## H2. Games (6)

- **Family:** `tools/art/blender/families/hobbies.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `chessTable` Chess table | 1x1 · floor | chess:play | hobbies / games | Table, Chair | Use | `move`: a pawn slides one square for chess, 0.9 s in, once the player has sat down (`delay`). A 0.64 × 0.48 m top, the taken pieces beside the board, since 2026-10-21 (G-40). |
| `poolTable` Pool table | 3x2 · floor | pool:play trickShots:play | hobbies / games | Frame, Cloth | Use | `rack_off`: the triangle is lifted off the balls, carried over the foot rail and hung by its apex on the table's foot end (1.4 s, `G.path`) for pool and trick shots; it hung 25 cm over the balls before 2026-10-21. The rails' top at 0.785 m, the cloth at 0.75 (G-29: 0.82). |
| `foosball` Foosball table | 2x1 · floor | foosball:play | hobbies / games | Table, Trim | Loop (E1) | `rods`: blue's defence and attack rods kick ±55° while played (E1). Eight rods since 2026-10-21 (G-41: four, every handle on the front), each team's handles on its own side, blue's to the front where the players stand. |
| `dartboard` Dartboard | 1x1 · wall | darts:play | hobbies / games | Board, Cabinet | Use | `darts`: three darts grow in the board for darts. |
| `arcadeCabinet` Arcade cabinet | 1x1 · floor | arcade:play | hobbies / games | Cabinet, Art | Loop (E1) | `joystick`: the stick waggles while played (E1). The screen and marquee glow at night (E8). |
| `puzzleTable` Jigsaw table | 1x1 · floor | jigsaw:play | hobbies / games | Table, Stool | Use | `pieces`: a patch of finished puzzle grows for jigsaw, 0.9 s in, once the puzzler has sat down (`delay`). |

**In play** (signed off 2026-10-16):
- Checked standing free, the dartboard on three inside walls and an outside face, the arcade cabinet against three
  walls and free.
- Every clip plays for its use and stops or shuts with it.
- Since 2026-10-21 (review A-hobbies-03) the pool table's rack is lifted off and hung on the foot end for the game,
  not left hanging 25 cm over the balls (`fixes/motions/g1718/views/pool-end-t99.png`).
- The chess table's and jigsaw table's second colour is "Chair" and "Stool": it colours the chess chair and the
  jigsaw stool; the chessboard's rim is the table's wood.
- The darts are thrown from the tile in front of the board, half a metre from the wall.
- Played with their own resident clips (G-26, `characters/clips/games.ts`): darts are thrown (`dartThrow`: side-on,
  aim, throw, the hand following to the board, three darts and a step back to read the score); pool is a cue stroke
  bent over the long rail, the left hand bridging on the cloth (`poolShot*`, a size per height, `STEP_IN` up to the
  rail, a lane each for two); the foosball rods are turned at their handles (`foosball*`, at a pair of rods each).
  The cue isn't in the hands yet (props in hand, G-70): it is mimed.
- With the cutaway dropping its wall, the dartboard hides with the wall and can still be used.
- Since the authenticity review (G-05): the chess table's chair is sat on at its pad (0.475).

## H3. Crafts and science (7)

- **Family:** `tools/art/blender/families/hobbies.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `easel` Easel | 1x1 · floor | paint:paint | hobbies / crafts | Wood, Metal | Use | `painting`: a painting grows on the canvas for paint. |
| `workbench` Workbench | 2x1 · floor | tinker:tinker | hobbies / crafts | Top, Frame, Tool board | Use | `vice`: the vice jaw slides shut on a plank for tinker. A 0.90 m top on an open frame with a shelf and a tool board on the wall since 2026-10-21 (G-29: a 0.78 m steel desk). |
| `telescope` Telescope | 1x1 · floor | stargaze:look | hobbies / crafts | Tube, Tripod | Use | `aim`: the tube tilts up 35° for stargaze and comes back down. In play the stargazer steps up and bends down to the eyepiece as aimed (0.765 m), the right hand on the tube (`eyepiece181`…`112` by height, G-21). |
| `sewingTable` Sewing table | 1x1 · floor | sew:tinker mend:tinker | hobbies / crafts | Table, Legs, Machine | Use, repeating (E1) | `presser`: the presser foot lowers for sew and mend, and the handwheel turns round after round (`handwheel`, `every` 0.9 s, E1), both from 0.9 s in, once the sewer has sat down (`delay`). The needle on the sewer's left, the handwheel on their right, as on a real machine (checked in play 2026-10-21: M-hobbies-08 closed, not mirrored). |
| `chemistrySet` Chemistry bench | 1x1 · floor | experiment:tinker | hobbies / crafts | Bench, Glassware | Use | `bubble`: the liquid rises in the flask for experiment. |
| `potteryWheel` Pottery wheel | 1x1 · floor | throw:tinker | hobbies / crafts | Wheel, Stool | Use, repeating (E1) | `clay`: the pot grows from a lump for throw, and the wheel head spins round after round (`wheel`, `every` 0.9 s, E1), both from 0.9 s in, once the potter has sat down (`delay`). A narrow body under a 0.48 m pan, the potter's knees either side, since 2026-10-21 (G-31: the knees went into the drum). |
| `craftTable` Craft table | 2x1 · floor | scrapbook:tinker | hobbies / crafts | Wood, Metal | Use | `book_open`: the scrapbook cover swings open for scrapbook, 0.9 s in, once the crafter has sat down (`delay`). |

**In play** (signed off 2026-10-16):
- Checked standing free (the workbench also against three walls); every clip plays for its use and shuts when the
  resident moves on.
- The sewing table's "Table" colours its top and chair; "Legs" colours its legs, apron and drawer (dark wood as
  made).
- At rest the pottery wheel's lump sits lower than a full pot.
- Since the authenticity review (G-05, G-15): the sewing table's chair at its pad (0.475); at the chemistry bench,
  pushed back on its tile, the experimenter steps in (`STAND_BACK` -0.35), the hands on the top.
