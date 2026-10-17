# Hobbies

The nineteen hobby models are done in `families/hobbies.py` (notes: `hobbies.md`), in Validate › Music, Games and Crafts. The guitars and the ukulele are a part of their own with no clip yet: the music animation doesn't draw an instrument in the hands.

Instruments a resident holds (guitars, the ukulele) hide from their stand while played if the music animation draws
them in the hands (T0 `squash` to 0).

## H1. Music (6) — **Done**, signed off 2026-10-16 (back in hobbies / music)

- **Family:** `tools/art/blender/families/hobbies.py`
- **Validate group:** `music` (Validate › Music)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `piano` Piano | 2x1 · floor | play:music | hobbies / music | Body, Trim | Use | `fallboard`: the key cover lifts and the music desk unfolds for play. |
| `ukulele` Ukulele | 1x1 · floor | strum:music practise:music | hobbies / music | Body & neck, Top, Stand | Use | `off_stand`: the ukulele leaves its stand while strummed (if held in the hands). |
| `guitarElectric` Electric guitar | 1x1 · floor | jam:music practise:music | hobbies / music | Guitar, Amp | Use | `off_stand`, as the ukulele; the amp is static. |
| `keyboardDigital` Digital piano | 1x1 · floor | play:music | hobbies / music | Body, Trim | Static | Static. |
| `drumKit` Drum kit | 1x1 · floor | drum:music practise:music | hobbies / music | Shells, Hardware | Loop (E1) | `cymbals`: the cymbals wobble while drummed (E1). Until then U: the hi-hat closes. |
| `guitarAcoustic` Acoustic guitar | 1x1 · floor | strum:music practise:music | hobbies / music | Body, Top, Stand | Use | `off_stand`. |

## H2. Games (6) — **Done**, signed off 2026-10-16 (back in hobbies / games)

- **Family:** `tools/art/blender/families/hobbies.py`
- **Validate group:** `games` (Validate › Games)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `chessTable` Chess table | 1x1 · floor | chess:play | hobbies / games | Table, Chair | Use | `move`: a pawn slides one square for chess. |
| `poolTable` Pool table | 3x2 · floor | pool:play trickShots:play | hobbies / games | Frame, Cloth | Use | `rack_off`: the triangle lifts off the balls for pool and trick shots. |
| `foosball` Foosball table | 2x1 · floor | foosball:play | hobbies / games | Table, Trim | Loop (E1) | `rods`: the rods turn while played (E1). Until then U: the rods slide in 5 cm. |
| `dartboard` Dartboard | 1x1 · wall | darts:play | hobbies / games | Board, Cabinet | Use | `darts`: three darts grow in the board for darts. |
| `arcadeCabinet` Arcade cabinet | 1x1 · floor | arcade:play | hobbies / games | Cabinet, Art | Loop (E1) | `joystick`: the stick waggles while played (E1). Until then static. |
| `puzzleTable` Jigsaw table | 1x1 · floor | jigsaw:play | hobbies / games | Table, Stool | Use | `pieces`: a patch of finished puzzle grows for jigsaw. |

## H3. Crafts and science (7) — **Done**, signed off 2026-10-16 (back in hobbies / crafts)

- **Family:** `tools/art/blender/families/hobbies.py`
- **Validate group:** `crafts` (Validate › Crafts)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `easel` Easel | 1x1 · floor | paint:paint | hobbies / crafts | Wood, Metal | Use | `painting`: a painting grows on the canvas for paint. |
| `workbench` Workbench | 2x1 · floor | tinker:tinker | hobbies / crafts | Wood, Metal | Use | `vice`: the vice jaw slides shut on a plank for tinker. |
| `telescope` Telescope | 1x1 · floor | stargaze:look | hobbies / crafts | Tube, Tripod | Use | `aim`: the tube tilts up 35° for stargaze and comes back down. |
| `sewingTable` Sewing table | 1x1 · floor | sew:tinker mend:tinker | hobbies / crafts | Table, Legs, Machine | Use | `presser`: the presser foot lowers for sew and mend. L the handwheel turns (E1). |
| `chemistrySet` Chemistry bench | 1x1 · floor | experiment:tinker | hobbies / crafts | Bench, Glassware | Use | `bubble`: the liquid rises in the flask for experiment. |
| `potteryWheel` Pottery wheel | 1x1 · floor | throw:tinker | hobbies / crafts | Wheel, Stool | Use | `clay`: the pot grows from a lump for throw. L the wheel spins (E1). |
| `craftTable` Craft table | 2x1 · floor | scrapbook:tinker | hobbies / crafts | Wood, Metal | Use | `book_open`: the scrapbook cover swings open for scrapbook. |
