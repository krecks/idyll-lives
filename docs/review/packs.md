# Personality packs: authenticity ledger

What it covers: the 94 objects of [docs/objects/packs.md](../objects/packs.md), in its groups and order.

- **Evidence:** `bench-results/review/packs/` in the review worktree (gallery, game-camera, clip, pose and play shots, contact sheets; shots stay out of git). Shots cells are paths under `bench-results/review/`. How the evidence was taken: `packs/index.md` and `packs/log.md`; how it was read: `packs/review-log.md`. Extra shots taken in the review: `packs/extra/night/` (12 uses on `review-stage-night`, 22:18–00:16 game time).
- **Reviewed** 2026-10-21 (wave: packs), headless, one core, against the rubric and [reference.md](reference.md); functional heights strict (1 cm, 5 mm on table tops), overall ±10 %. Pack objects carry a personality theme: the real thing each depicts still has to work and look right; whimsical extras pass when they read.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Durations** are judged by §9 decision 2: short by design; only outliers over 3× the tag median against similar uses are filed.

## Sizes table: the outliers judged

`tables/sizes-packs.md` flags 59 of the 94. Most are bundles whose kind fits only their main part; there the reference kind is the wrong yardstick, not the model. Judged one by one:

- **Bundle or extra part, the main part in range (no finding):** `whirlpoolTub` H (it is a hot tub, 0.93 m, not a bath), `chefStove` W, H (range with hood and splashback; 150 cm ranges exist), `foodie.herbGarden` H, D (chives, basket), `foodie.cookbookLectern` W (cookbook stool, spoon jar), `foodie.pizzaOven` W (prep counter), `foodie.chefsIsland` H (pot on the top), `bookworm.readingNook` H (bookcases), `bookworm.libraryWall` D (leaning ladder), `bookworm.firesideLounge` H, D (flue and two wing chairs), `couchPotato.fitnessGame` D (balance board), `couchPotato.lavaLamp` W, D (the chair), `neat.steamMopDock` W, D (bucket, pad), `neat.organizerDesk` H (hutch), `neat.robotVacuumDock` W, D (rug, plant), `neat.entryConsole` H (mirror, hook board), `slob.snackNightstand` W, D (snacks on the floor), `slob.spritzStation` D (bottles, stool), `slob.gamingRecliner` W (side tables), `lazy.deliveryLocker`, `loner.deliveryHatch` D (parcels, mat), `lazy.smartHub` D (robot disc), `cheerful.mocktailCart` H (canopy), `cheerful.sunriseLamp`, `gloomy.rainLamp` W, D (mat, pebbles), `gloomy.recordCabinet` D (vase), `gloomy.windowSeat` H (window frame), `gloomy.cello` W (stool, bow, music; 1 % over), `natureLover.veggieBed` H (canes), `natureLover.pottingBench` H (backboard), `natureLover.mossTerrarium` D (bottle), `outgoing.partyBar` H, D (back bar), `outgoing.conversationSectional` H (lamp behind), `outgoing.videoCallNook`, `loner.shortwaveRadio` H (ring light, aerial), `outgoing.photoWall` H, D (a free-standing board, not a wall pinboard), `loner.bonsaiBench` H (back board, tree), `romantic.candlelitTable` H (chair backs), `romantic.vanity` D (pouf), `romantic.gramophone` W, H (horn; 3–4 %), `hotHeaded.heavyBag` W, D (base plate), `kind.givingJar` W, D (plant), `kind.neighbourlyOven` H (wall shelf), `kind.welcomeTeaTable` W, H, seat (pouf-seated tea table, not floor seating: `lowTable` is the wrong kind), `lazy.quiltTable` W, H, seat (a chair-height kotatsu with a bench: `lowTable` is the wrong kind; its own problems are I-packs-01).
- **In tolerance or rounding:** `couchPotato.recliner` D 0.760 < 0.765, `loner.soloSauna` seat 0.447 < 0.45, `neat.organizerDesk` top 0.762, `energetic.streetballHoop` H 2.83 (a portable hoop adjusts 2.3–3.05 m; the 3.2 m floor is for a fixed post), `slob.microwaveCart` top 0.815 (a cart, not a worktop), `cheerful.jukebox` D 0.495 (reads; slim wall jukeboxes exist).
- **The kind is wrong:** `energetic.recoveryPod`, `loner.quietPod` (no seat to measure / D under range: a lounger and a shell chair share one wide kind), `energetic.spinBike` seat (a recumbent: M-packs-09).
- **Filed:** desk tops M-packs-08; hammocks M-packs-06; `bookworm.bedsideStack`, `hotHeaded.coolDownCushion`, `gloomy.soakingTub` M-packs-12; `foodie.chefsIsland`'s stools (not flagged: the kind has no stool line) M-packs-04.

## P1. Bookworm (6)

Family: `tools/art/blender/families/pack_bookworm.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bookworm.readingNook` Window-seat reading nook | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no book) | `packs/sheets/obj-bookworm.readingNook.png`, `packs/sheets/age-bookworm.readingNook.png` |
| `bookworm.libraryWall` Rare-books library wall | OK | OK | I-packs-05, I-held-01 | I-packs-05 S2 content; I-held-01 S2 code (held ledger) | `packs/sheets/obj-bookworm.libraryWall.png`, `packs/play/chunk1/bookworm.libraryWall-research-01.png` |
| `bookworm.writingBureau` Writer's bureau | OK | OK | OK | | `packs/sheets/obj-bookworm.writingBureau.png`, `packs/sheets/age-bookworm.writingBureau.png` |
| `bookworm.teaTrolley` Book-club tea trolley | OK | OK | I-packs-05, I-held-01 | I-packs-05 S2 content; I-held-01 S2 code (held ledger: no cup) | `packs/sheets/obj-bookworm.teaTrolley.png` |
| `bookworm.firesideLounge` Fireside reading lounge | M-packs-10 | OK | I-held-01 | M-packs-10 S3 model; I-held-01 S2 code (held ledger) | `packs/sheets/obj-bookworm.firesideLounge.png`, `packs/extra/night/night-bookworm.firesideLounge-readByFire-01.png` |
| `bookworm.bedsideStack` Bedside book stack | M-packs-12 | OK | I-packs-05 | M-packs-12 S3 model; I-packs-05 S2 content | `packs/sheets/obj-bookworm.bedsideStack.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bookworm.readingNook` | curl up and read by the window, look out, doze | curl up with a book (60, sit) | OK; read sitting square, legs down; dozing is the window seat's (`gloomy.windowSeat`) |
| `bookworm.libraryWall` | find a book, read it in a chair, climb the ladder to the top shelves | research (60), browse (45): both standing at the shelf | browse OK; an hour's research on foot (I-packs-05) |
| `bookworm.writingBureau` | write, type a letter, keep papers | draft (60), write a letter (30), seated | OK |
| `bookworm.teaTrolley` | brew and pour tea, serve it to guests, drink it sitting | brew a pot (20, standing) | the 20 minutes of tea are drunk on foot (I-packs-05) |
| `bookworm.firesideLounge` | read by the fire, read aloud to someone, warm up | read by the fire (60), read aloud (45, 2 slots) | OK; reading aloud needs no listener (I16, as I-hobbies-01's question: not filed again) |
| `bookworm.bedsideStack` | read a chapter in bed before sleep, keep a lamp and a glass by the bed | one more chapter (20, standing at it) | read standing beside a nightstand, not in bed (I-packs-05) |

## P2. Cheerful (6)

Family: `tools/art/blender/families/pack_cheerful.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `cheerful.jukebox` Sunburst jukebox | M-packs-07 | OK | I-packs-16 | M-packs-07 S2 model; I-packs-16 S3 content | `packs/sheets/obj-cheerful.jukebox.png`, `packs/extra/night/night-cheerful.jukebox-dance-01.png` |
| `cheerful.karaokeMachine` Spotlight karaoke machine | M-packs-07 | OK | I-bathroom-04 | M-packs-07 S2 model (lyrics screen, from its material; not shot at night); I-bathroom-04 S3 code (systems ledger: singing has no sound) | `packs/sheets/obj-cheerful.karaokeMachine.png` |
| `cheerful.mocktailCart` Fizz & sunshine mocktail cart | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no glass or shaker) | `packs/sheets/obj-cheerful.mocktailCart.png` |
| `cheerful.rainbowHammock` Rainbow hammock | M-packs-06 | M-packs-06 | OK | M-packs-06 S2 decision | `packs/sheets/obj-cheerful.rainbowHammock.png`, `packs/sheets/age-cheerful.rainbowHammock.png` |
| `cheerful.sunriseLamp` Sunrise stretch lamp | M-packs-07 | OK | I-packs-02 | M-packs-07 S2 model (no `light`); I-packs-02 S2 content | `packs/sheets/obj-cheerful.sunriseLamp.png`, `packs/extra/night/night-cheerful.sunriseLamp-stretch-01.png` |
| `cheerful.partyBooth` Confetti party booth | OK | OK | I-packs-10 | I-packs-10 S3 renderer-table | `packs/sheets/obj-cheerful.partyBooth.png`, `packs/play/chunk3/cheerful.partyBooth-party-01.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `cheerful.jukebox` | pick a song, listen, dance to it | dance (45, 2 slots) | listening missing (I-packs-16) |
| `cheerful.karaokeMachine` | sing along to the lyrics, duet | sing (60) | OK; no sound (I-bathroom-04) |
| `cheerful.mocktailCart` | mix and serve a drink, drink it | mix a mocktail (15) | OK |
| `cheerful.rainbowHammock` | lie and swing, doze | swing (30), doze (45), lying | OK; the frame is short (M-packs-06) |
| `cheerful.sunriseLamp` | wake to it, stretch on the mat, read by it | morning stretch (20) | OK; stretched beside the mat (I-packs-02) |
| `cheerful.partyBooth` | sit and chat, a party or a video party with friends | video party (90, 2 slots, seated) | OK; the call is a phone at the ear (I-packs-10) |

## P3. Couch potato (6)

Family: `tools/art/blender/families/pack_couchPotato.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `couchPotato.cinemaWall` Grand cinema wall | OK | OK | OK | | `packs/sheets/obj-couchPotato.cinemaWall.png`, `packs/sheets/age-couchPotato.cinemaWall.png`, `packs/extra/night/night-couchPotato.cinemaWall-marathon-01.png` |
| `couchPotato.gameConsole` Pixel Arcade console | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no controller) | `packs/sheets/obj-couchPotato.gameConsole.png` |
| `couchPotato.recliner` Cloud Nine power recliner | OK | A-packs-06 | A-packs-06 | A-packs-06 S2 content | `packs/sheets/obj-couchPotato.recliner.png`, `packs/play/chunk1/couchPotato.recliner-recline-02.png` |
| `couchPotato.snackFridge` Sofa-side snack fridge | OK | OK | OK | | `packs/sheets/obj-couchPotato.snackFridge.png` |
| `couchPotato.fitnessGame` Living-room workout game | OK | OK | I-packs-02 | I-packs-02 S2 content | `packs/sheets/obj-couchPotato.fitnessGame.png` |
| `couchPotato.lavaLamp` Lava glow lamp | M-packs-07 | OK | I-packs-11 | M-packs-07 S2 model; I-packs-11 S3 model | `packs/sheets/obj-couchPotato.lavaLamp.png`, `packs/extra/night/night-couchPotato.lavaLamp-zoneOut-00.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `couchPotato.cinemaWall` | watch a film or a series from the recliners, with company | movie marathon (90), channel surf (60), 2 slots, seated facing the screen | OK |
| `couchPotato.gameConsole` | play a game, play with a friend | play (60), speedrun (60), 2 slots | OK |
| `couchPotato.recliner` | lean back with the feet up, watch TV, doze | recline (40), doze (30) | right uses, upright pose (A-packs-06) |
| `couchPotato.snackFridge` | a snack or a cold drink from the sofa | snack (2), takeout (5) | OK; takeout from a mini fridge is the pack's joke |
| `couchPotato.fitnessGame` | an exercise game on the balance board in front of the screen | workout (45) | OK; done beside the board (I-packs-02) |
| `couchPotato.lavaLamp` | a mood light to watch, sitting | zone out (20, seated) | OK; the chair faces away from it (I-packs-11) |

## P4. Energetic (7)

Family: `tools/art/blender/families/pack_energetic.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `energetic.boulderingWall` Home bouldering wall | OK | I-packs-03 | I-packs-03 | I-packs-03 S2 clip | `packs/sheets/obj-energetic.boulderingWall.png`, `packs/play/chunk2/energetic.boulderingWall-climb-01.png` |
| `energetic.spinBike` Interactive spin bike | M-packs-09 | OK | OK | M-packs-09 S3 decision | `packs/sheets/obj-energetic.spinBike.png`, `packs/sheets/age-energetic.spinBike.png` |
| `energetic.trampoline` Rebound trampoline | OK | OK | OK | | `packs/sheets/obj-energetic.trampoline.png` |
| `energetic.streetballHoop` Streetball hoop | OK | OK | I-packs-12 | I-packs-12 S3 content | `packs/sheets/obj-energetic.streetballHoop.png` |
| `energetic.powerBlender` Power blender station | OK | A-packs-02 | I-packs-19 | A-packs-02 S3 motion (seeded; confirmed); I-packs-19 S3 code | `packs/sheets/obj-energetic.powerBlender.png` |
| `energetic.contrastShower` Contrast rain shower | OK | A-bathroom-01 | I-bathroom-02 | A-bathroom-01 S2 model (bathroom ledger: the glass door never opens); I-bathroom-02 S3 clip (residents ledger) | `packs/sheets/obj-energetic.contrastShower.png` |
| `energetic.recoveryPod` Recovery pod | OK | OK | OK | | `packs/sheets/obj-energetic.recoveryPod.png`, `packs/play/chunk2/energetic.recoveryPod-recover-01.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `energetic.boulderingWall` | climb problems on the wall, rest on the mat | climb (60) | climbed on the floor beside the mat (I-packs-03) |
| `energetic.spinBike` | a cycling class or ride | ride (45, seated, pedalling) | OK as a recumbent (M-packs-09) |
| `energetic.trampoline` | bounce, play | bounce (30, on the mat) | OK |
| `energetic.streetballHoop` | shoot hoops, play one on one | shoot hoops (40, 2 slots) | OK; shot from under the board (I-packs-12) |
| `energetic.powerBlender` | blend a smoothie and drink it | blend (4) | OK; the `cook` median (33) is for cooked meals, a smoothie is 2–5 |
| `energetic.contrastShower` | shower, hot and cold | rinse (15) | OK |
| `energetic.recoveryPod` | lie back under the canopy to recover | recover (45, lying) | OK |

## P5. Example pack (2)

Family: `tools/art/blender/families/pack_example.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `chefStove` Chef's stove | OK | OK | OK | | `packs/sheets/obj-chefStove.png`, `packs/play/rerun1/chefStove-cook-01.png` |
| `whirlpoolTub` Whirlpool tub | OK | OK | OK | | `packs/sheets/obj-whirlpoolTub.png`, `packs/sheets/age-whirlpoolTub.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `chefStove` | cook on the hob, bake in the oven, practise recipes | cook (33), practise (53), needs ingredients | OK; the oven door opens 3 s and shuts (`hold: 3.0`) |
| `whirlpoolTub` | soak with the jets on, with company | soak (45, 2 slots) | OK; a hot tub, not a bath: the `soakingTub` height range doesn't apply |

## P6. Foodie (6)

Family: `tools/art/blender/families/pack_foodie.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `foodie.herbGarden` Windowsill herb garden | OK | I-packs-06 | I-packs-06 | I-packs-06 S2 renderer-table | `packs/sheets/obj-foodie.herbGarden.png`, `packs/play/chunk1/foodie.herbGarden-tendHerbs-01.png` |
| `foodie.deliPantry` Deli pantry | OK | OK | I-packs-19 | I-packs-19 S3 code | `packs/sheets/obj-foodie.deliPantry.png` |
| `foodie.espressoBar` Espresso bar | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no cup) | `packs/sheets/obj-foodie.espressoBar.png` |
| `foodie.cookbookLectern` Cookbook lectern | OK | OK | I-packs-12 | I-packs-12 S3 content | `packs/sheets/obj-foodie.cookbookLectern.png` |
| `foodie.pizzaOven` Wood-fired pizza oven | OK | OK | OK | | `packs/sheets/obj-foodie.pizzaOven.png`, `packs/extra/night/night-foodie.pizzaOven-bakePizza-01.png` |
| `foodie.chefsIsland` Chef's island | M-packs-04 | OK | M-packs-04 | M-packs-04 S2 model | `packs/sheets/obj-foodie.chefsIsland.png`, `packs/sheets/age-foodie.chefsIsland.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `foodie.herbGarden` | water and trim herbs, pick some for cooking | tend herbs (20) | OK; the pose is the flower bed's kneel (I-packs-06) |
| `foodie.deliPantry` | take out a snack or ingredients, make a board | nibble (2), cheese board (5) | OK; 5 min for a board is real (the `cook` median is for cooked meals) |
| `foodie.espressoBar` | pull an espresso, steam milk, latte art | espresso (10), latte art (30) | OK |
| `foodie.cookbookLectern` | follow a recipe while cooking, browse cookbooks | study recipes (60), food magazines (45), standing | OK; the open book isn't the one read (I-packs-12) |
| `foodie.pizzaOven` | fire it up, bake pizza, warm up by it | bake pizza (43), warm up (30) | OK |
| `foodie.chefsIsland` | prepare and cook, eat at the counter on the stools | tasting menu (78), bar bite (20, on a stool) | OK; the stools are chair height (M-packs-04) |

## P7. Gloomy (6)

Family: `tools/art/blender/families/pack_gloomy.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `gloomy.recordCabinet` Midnight record cabinet | OK | OK | I-packs-05 | I-packs-05 S2 content | `packs/sheets/obj-gloomy.recordCabinet.png` |
| `gloomy.cello` Nocturne cello | OK | OK | OK | | `packs/sheets/obj-gloomy.cello.png`, `packs/sheets/age-gloomy.cello.png` |
| `gloomy.journalDesk` Candlelit writing desk | M-packs-08 | OK | OK | M-packs-08 S3 model | `packs/sheets/obj-gloomy.journalDesk.png`, `packs/sheets/age-gloomy.journalDesk.png` |
| `gloomy.rainLamp` Rainfall lamp | M-packs-07 | OK | OK | M-packs-07 S2 content (no `light`) | `packs/sheets/obj-gloomy.rainLamp.png`, `packs/extra/night/night-gloomy.rainLamp-watchRain-01.png` |
| `gloomy.windowSeat` Brooding window seat | OK | A-packs-05 | A-packs-05 | A-packs-05 S2 renderer-table | `packs/sheets/obj-gloomy.windowSeat.png`, `packs/play/chunk3/gloomy.windowSeat-doze-01.png`, `packs/sheets/age-gloomy.windowSeat.png` |
| `gloomy.soakingTub` Moonstone soaking tub | M-packs-12 | A-packs-01 | OK | M-packs-12 S3 model; A-packs-01 S2 renderer-table (widened) | `packs/sheets/obj-gloomy.soakingTub.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `gloomy.recordCabinet` | put a record on, listen from a chair | listen (60), new album (45), standing | an hour on foot beside the cabinet (I-packs-05) |
| `gloomy.cello` | practise, play a piece | practise (60), lament (45), seated, the cello between the knees | OK |
| `gloomy.journalDesk` | write a journal or poetry by candlelight | journal (40), poetry (60) | OK |
| `gloomy.rainLamp` | a mood light to watch | watch the rain (30) | OK |
| `gloomy.windowSeat` | sit and look out, read, doze | brood (40), doze (30, lying) | OK; brooding 40 is 3.2× the `look` median (12.5), but that tag mixes glances with sitting at a window: not filed; the doze pose is A-packs-05 |
| `gloomy.soakingTub` | a long hot soak | soak (50) | OK |

## P8. Hot-headed (6)

Family: `tools/art/blender/families/pack_hotHeaded.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `hotHeaded.heavyBag` Heavy punching bag | OK | OK | I-bathroom-04 | I-bathroom-04 S3 code (systems ledger: no thud) | `packs/sheets/obj-hotHeaded.heavyBag.png` |
| `hotHeaded.smashBin` Plate-smash bin | OK | I-packs-13 | I-packs-13 | I-packs-13 S3 clip | `packs/sheets/obj-hotHeaded.smashBin.png` |
| `hotHeaded.thunderDrums` Thunder drum kit | M-packs-13 | OK | OK | M-packs-13 S3 renderer-table | `packs/sheets/obj-hotHeaded.thunderDrums.png`, `packs/sheets/age-hotHeaded.thunderDrums.png` |
| `hotHeaded.coolDownCushion` Cool-down meditation cushion | M-packs-12, M-packs-13 | OK | OK | M-packs-12 S3 model; M-packs-13 S3 renderer-table | `packs/sheets/obj-hotHeaded.coolDownCushion.png`, `packs/sheets/age-hotHeaded.coolDownCushion.png` |
| `hotHeaded.grudgeJournal` Grudge-to-gratitude writing desk | M-packs-08 | OK | OK | M-packs-08 S3 model | `packs/sheets/obj-hotHeaded.grudgeJournal.png` |
| `hotHeaded.coldPlunge` Glacier cold-plunge tub | OK | A-packs-01 | I-systems-04 | A-packs-01 S2 renderer-table (widened); I-systems-04 S2 code (systems ledger: no age rule; a child takes the plunge) | `packs/sheets/obj-hotHeaded.coldPlunge.png`, `packs/sheets/age-hotHeaded.coldPlunge.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `hotHeaded.heavyBag` | punch and kick the bag | pound the bag (45) | OK |
| `hotHeaded.smashBin` | (a rage room) throw old plates into the bin | smash plates (20) | the pack's idea reads; the throw is a punch (I-packs-13) |
| `hotHeaded.thunderDrums` | play the drums | drum (60, seated) | OK |
| `hotHeaded.coolDownCushion` | sit and meditate, breathe | count to ten (20), breathe (60) | OK |
| `hotHeaded.grudgeJournal` | write it out | write it out (45) | OK |
| `hotHeaded.coldPlunge` | a cold plunge of a few minutes | plunge (30) | 30 minutes in ice water is long (real 2–10), but `bath` uses are 30–50 and it's the pack's whimsy: by design, not filed |

## P9. Kind (6)

Family: `tools/art/blender/families/pack_kind.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `kind.neighbourlyOven` Neighbourly baking counter | M-packs-01 | A-packs-03 | I-packs-19 | M-packs-01 S2 model (widened); A-packs-03 S2 motion; I-packs-19 S3 code | `packs/sheets/obj-kind.neighbourlyOven.png`, `packs/play/rerun1/kind.neighbourlyOven-familyPie-01.png` |
| `kind.givingJar` Community giving jar | OK | OK | OK | | `packs/sheets/obj-kind.givingJar.png` |
| `kind.knittingRocker` Gift-knitting rocker | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no needles or yarn) | `packs/sheets/obj-kind.knittingRocker.png` |
| `kind.welcomeTeaTable` Welcome tea table | OK | OK | I-packs-11 | I-packs-11 S3 model | `packs/sheets/obj-kind.welcomeTeaTable.png`, `packs/sheets/age-kind.welcomeTeaTable.png` |
| `kind.songbirdFeeder` Songbird feeding station | OK | OK | OK | | `packs/sheets/obj-kind.songbirdFeeder.png` |
| `kind.hearthSettee` Hearthside quilted settee | M-packs-13 | OK | OK | M-packs-13 S3 renderer-table | `packs/sheets/obj-kind.hearthSettee.png`, `packs/play/chunk5/kind.hearthSettee-quiltNap-01.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `kind.neighbourlyOven` | bake, prepare food on the counter | family pie (48), treats to share (53), need ingredients | OK; the oven stays open (A-packs-03) |
| `kind.givingJar` | drop in a coin | donate (8) | OK |
| `kind.knittingRocker` | rock, knit, doze | knit a gift (60), rock (20) | OK |
| `kind.welcomeTeaTable` | have tea with a guest | share tea (30, 2 slots) | OK; the two face the room, not each other (I-packs-11) |
| `kind.songbirdFeeder` | fill the feeder, watch the birds | refill (10), watch birds (30) | OK |
| `kind.hearthSettee` | sit with company, nap under the quilt | curl up (45, 2 slots), quilt nap (40, lying) | OK; the quilt stays folded on the seat under the napper (the code blanket is for beds; not filed: decor) |

## P10. Lazy (6)

Family: `tools/art/blender/families/pack_lazy.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `lazy.snoozecloudBed` Snoozecloud adjustable bed | OK | OK | OK | | `packs/sheets/obj-lazy.snoozecloudBed.png`, `packs/play/chunk2/lazy.snoozecloudBed-sleepIn-02.png` |
| `lazy.napPod` Daydream nap pod | A-packs-04 | A-packs-04 | OK | A-packs-04 S2 model | `packs/sheets/obj-lazy.napPod.png`, `packs/play/chunk3/lazy.napPod-powerNap-01.png` |
| `lazy.quiltTable` Heated quilt table | OK | I-packs-01 | I-packs-01 | I-packs-01 S2 model | `packs/sheets/obj-lazy.quiltTable.png`, `packs/play/chunk3/lazy.quiltTable-tuckIn-01.png`, `packs/sheets/age-lazy.quiltTable.png` |
| `lazy.deliveryLocker` Doorstep delivery locker | OK | OK | OK | | `packs/sheets/obj-lazy.deliveryLocker.png` |
| `lazy.vibePlate` Effortless vibration plate | I-packs-04 | I-packs-04 | I-packs-04 | I-packs-04 S2 model | `packs/sheets/obj-lazy.vibePlate.png`, `packs/play/chunk3/lazy.vibePlate-shake-02.png` |
| `lazy.smartHub` Smart-home command hub | OK | OK | I-packs-05, I-packs-19 | I-packs-05 S2 content; I-packs-19 S3 code | `packs/sheets/obj-lazy.smartHub.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `lazy.snoozecloudBed` | sleep, lie in with the head raised, read or watch in bed | sleep (480), sleep in (45, the head section up) | OK; the loose quilt on the floor in `packs/poses/all/poses-lazy.snoozecloudBed-adult.png` is the poses page's (a sleeper's blanket drawn between its two stations while it seats people on the beds); play draws it on the sleeper (`packs/play/chunk2/lazy.snoozecloudBed-sleep-01.png`): tooling, not filed |
| `lazy.napPod` | curl up, nap | curl up (30), power nap (25), sitting | OK; the hood meets the head (A-packs-04) |
| `lazy.quiltTable` | (a kotatsu) sit with the legs under the heated quilt, eat, doze | tuck in (40), doze (30), 2 slots | right uses; the legs come out of the front (I-packs-01) |
| `lazy.deliveryLocker` | collect a parcel or a delivered meal | takeout (5), noodles (4) | OK |
| `lazy.vibePlate` | stand on the plate holding the bar | shake (30, on the plate) | right use, back to the bar (I-packs-04) |
| `lazy.smartHub` | ask it things, set routines, hear a story | program routines (60), story (30), standing | an hour tapping a tablet on foot (I-packs-05) |

## P11. Loner (6)

Family: `tools/art/blender/families/pack_loner.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `loner.quietPod` Hush-shell reading pod | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no book) | `packs/sheets/obj-loner.quietPod.png`, `packs/sheets/age-loner.quietPod.png` |
| `loner.shortwaveRadio` Shortwave radio desk | OK | OK | OK | | `packs/sheets/obj-loner.shortwaveRadio.png`, `packs/sheets/age-loner.shortwaveRadio.png` |
| `loner.puzzleTable` Jigsaw puzzle table | OK | OK | OK | | `packs/sheets/obj-loner.puzzleTable.png`, `packs/sheets/age-loner.puzzleTable.png` |
| `loner.bonsaiBench` Bonsai potting bench | OK | OK | I-packs-12 | I-packs-12 S3 content | `packs/sheets/obj-loner.bonsaiBench.png` |
| `loner.soloSauna` Cedar sauna for one | OK | I-packs-17 | I-packs-17, I-systems-04 | I-packs-17 S3 model; I-systems-04 S2 code (systems ledger: a child takes the sauna) | `packs/sheets/obj-loner.soloSauna.png`, `packs/play/chunk4/loner.soloSauna-sweat-02.png`, `packs/sheets/age-loner.soloSauna.png` |
| `loner.deliveryHatch` No-knock delivery hatch | OK | OK | OK | | `packs/sheets/obj-loner.deliveryHatch.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `loner.quietPod` | sit shut away, read | retreat (40), read (45) | OK |
| `loner.shortwaveRadio` | tune the bands, talk to other operators | airwaves (45), tune (60) | OK |
| `loner.puzzleTable` | work on a jigsaw | a piece (45), new puzzle (90) | OK |
| `loner.bonsaiBench` | prune and wire the tree, water and mist it | shape (60), tend (15) | OK; worked from the bench's corner (I-packs-12) |
| `loner.soloSauna` | sit and sweat, door shut | sweat (40) | OK; the door opens 1.5 s and shuts; the feet (I-packs-17) |
| `loner.deliveryHatch` | take in a delivery without opening the door | order (5), treat (2) | OK |

## P12. Nature lover (7)

Family: `tools/art/blender/families/pack_natureLover.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `natureLover.veggieBed` Raised veggie bed | OK | OK | OK | | `packs/sheets/obj-natureLover.veggieBed.png` |
| `natureLover.pottingBench` Potting bench | OK | OK | OK | | `packs/sheets/obj-natureLover.pottingBench.png` |
| `natureLover.birdFeeder` Bird feeder post | OK | OK | OK | | `packs/sheets/obj-natureLover.birdFeeder.png` |
| `natureLover.gardenHammock` Garden hammock | M-packs-06 | M-packs-06 | OK | M-packs-06 S2 decision | `packs/sheets/obj-natureLover.gardenHammock.png`, `packs/sheets/age-natureLover.gardenHammock.png` |
| `natureLover.cedarShower` Cedar outdoor shower | OK | OK | I-bathroom-02 | I-bathroom-02 S3 clip (residents ledger) | `packs/sheets/obj-natureLover.cedarShower.png` |
| `natureLover.mossTerrarium` Moss terrarium | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: the mister stays on the floor) | `packs/sheets/obj-natureLover.mossTerrarium.png` |
| `natureLover.glasshouse` Fern glasshouse | OK | A-packs-03 | I-packs-09 | A-packs-03 S2 motion; I-packs-09 S2 content | `packs/sheets/obj-natureLover.glasshouse.png`, `packs/play/chunk4/natureLover.glasshouse-orchids-01.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `natureLover.veggieBed` | weed, water and tend, pick what's ripe | tend (30), pick a snack (15, when ripe) | OK |
| `natureLover.pottingBench` | pot seedlings, make up bouquets | pot seedlings (60), tie a bouquet (30) | OK |
| `natureLover.birdFeeder` | fill it, watch the birds | birdwatch (40) | OK; 3.2× the `look` median, inside the real 20–60: not filed; filling it missing, as `kind.songbirdFeeder` has (S3, not filed: decor-side) |
| `natureLover.gardenHammock` | lie and swing, doze | swing (45), doze (45) | OK; the frame is short (M-packs-06) |
| `natureLover.cedarShower` | rinse off after the garden or a swim | rinse (25) | OK |
| `natureLover.mossTerrarium` | mist it, look at it | mist (15) | OK |
| `natureLover.glasshouse` | tend the plants inside, sit among them | unwind (45, seated inside), orchids (60) | OK; orchids tended from outside the door (I-packs-09); 60 is 6× the `water` median, a session among the plants (real 20–60): not filed |

## P13. Neat (6)

Family: `tools/art/blender/families/pack_neat.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `neat.steamMopDock` Steam mop dock | OK | OK | OK | | `packs/sheets/obj-neat.steamMopDock.png` |
| `neat.laundryCenter` Stacked laundry centre | OK | A-packs-03 | I-packs-18 | A-packs-03 S2 motion; I-packs-18 S3 content | `packs/sheets/obj-neat.laundryCenter.png`, `packs/play/chunk2/neat.laundryCenter-laundry-01.png` |
| `neat.rainShowerTower` Rainfall steam shower | OK | A-bathroom-01 | I-bathroom-02 | A-bathroom-01 S2 model (bathroom ledger: the glass door never opens); I-bathroom-02 S3 clip (residents ledger) | `packs/sheets/obj-neat.rainShowerTower.png` |
| `neat.organizerDesk` Label-and-sort organizer desk | M-packs-05 | OK | OK | M-packs-05 S2 model | `packs/sheets/obj-neat.organizerDesk.png`, `packs/sheets/age-neat.organizerDesk.png` |
| `neat.robotVacuumDock` Robot vacuum and dock | OK | I-packs-15 | I-packs-15, I-packs-05 | I-packs-15 S3 motion; I-packs-05 S2 content | `packs/sheets/obj-neat.robotVacuumDock.png` |
| `neat.entryConsole` Tidy entry console | M-packs-01 | OK | OK | M-packs-01 S2 model (seeded; confirmed and widened) | `packs/sheets/obj-neat.entryConsole.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `neat.steamMopDock` | take the mop and mop the floor | mop (30, the mop in hand) | OK |
| `neat.laundryCenter` | load the washer, move it to the dryer, fold on the counter | laundry (45), fold (60) | OK; folding an hour (I-packs-18); the porthole stays open (A-packs-03) |
| `neat.rainShowerTower` | shower, steam | steam shower (25) | OK |
| `neat.organizerDesk` | sort and label, do the accounts | sort (40), ledger (60) | OK |
| `neat.robotVacuumDock` | set a schedule and let it clean | program a routine (45) | the robot never cleans (I-packs-15); setting it is 45 on foot (I-packs-05) |
| `neat.entryConsole` | drop the keys, check the mirror | tidy (10) | OK |

## P14. Outgoing (6)

Family: `tools/art/blender/families/pack_outgoing.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `outgoing.partyBar` Home party bar | M-packs-03 | OK | I-packs-16, I-held-01 | M-packs-03 S3 model (seeded; confirmed); I-packs-16 S3 content; I-held-01 S2 code (held ledger: no glass) | `packs/sheets/obj-outgoing.partyBar.png` |
| `outgoing.conversationSectional` Conversation sectional | OK | OK | OK | | `packs/sheets/obj-outgoing.conversationSectional.png`, `packs/sheets/age-outgoing.conversationSectional.png` |
| `outgoing.karaokeStage` Karaoke stage | M-packs-07 | OK | I-packs-02, I-bathroom-04 | M-packs-07 S2 model; I-packs-02 S2 content; I-bathroom-04 S3 code (systems ledger: no sound) | `packs/sheets/obj-outgoing.karaokeStage.png`, `packs/extra/night/night-outgoing.karaokeStage-practise-01.png` |
| `outgoing.videoCallNook` Video-call corner | M-packs-07 | OK | I-packs-10 | M-packs-07 S2 model; I-packs-10 S3 renderer-table | `packs/sheets/obj-outgoing.videoCallNook.png`, `packs/extra/night/night-outgoing.videoCallNook-call-01.png` |
| `outgoing.danceFloor` Light-up dance floor | M-packs-07 | OK | OK | M-packs-07 S2 model | `packs/sheets/obj-outgoing.danceFloor.png`, `packs/extra/night/night-outgoing.danceFloor-dance-01.png` |
| `outgoing.photoWall` Party photo wall | OK | OK | OK | | `packs/sheets/obj-outgoing.photoWall.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `outgoing.partyBar` | mix drinks behind the bar, sit at it on the stools for a drink | mix drinks (30), soda (10), both standing at the guest side | sitting at the bar missing; the stools are never used (I-packs-16) |
| `outgoing.conversationSectional` | sit with friends, chat, lounge | hang out (30, 2 slots) | OK |
| `outgoing.karaokeStage` | sing on the stage at the mic, duet | practise (60), perform (30), 2 slots | right uses, sung in front of the stage (I-packs-02) |
| `outgoing.videoCallNook` | a video call at the laptop | call (30, seated) | OK; played as a phone call (I-packs-10) |
| `outgoing.danceFloor` | dance, with others | dance (45, 2 slots) | OK |
| `outgoing.photoWall` | look at the photos, pin new ones | reminisce (15) | OK |

## P15. Romantic (6)

Family: `tools/art/blender/families/pack_romantic.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `romantic.candlelitTable` Candlelit bistro table | M-packs-11, M-packs-07 | I-packs-01 | I-packs-01, I-systems-03 | M-packs-11 S2 model; M-packs-07 S2 model; I-packs-01 S2 model; I-systems-03 S3 (systems ledger: a dirty plate left on the floor) | `packs/sheets/obj-romantic.candlelitTable.png`, `packs/play/chunk4/romantic.candlelitTable-candlelitDinner-01.png`, `packs/sheets/age-romantic.candlelitTable.png` |
| `romantic.heartTub` Heart-shaped petal tub | OK | A-packs-01 | I-systems-03 | A-packs-01 S2 renderer-table (seeded; confirmed and widened); I-systems-03 S3 (systems ledger: the plate on the floor) | `packs/sheets/obj-romantic.heartTub.png`, `packs/sheets/age-romantic.heartTub.png` |
| `romantic.vanity` Rose-gold vanity | OK | OK | I-packs-08 | I-packs-08 S2 content | `packs/sheets/obj-romantic.vanity.png` |
| `romantic.gramophone` Moonlight gramophone | OK | OK | I-packs-05 | I-packs-05 S2 content | `packs/sheets/obj-romantic.gramophone.png` |
| `romantic.letterBureau` Love-letter bureau | M-packs-05 | OK | OK | M-packs-05 S2 model | `packs/sheets/obj-romantic.letterBureau.png`, `packs/sheets/age-romantic.letterBureau.png` |
| `romantic.roseArbourSwing` Rose-arbour swing | OK | OK | I-packs-12 | I-packs-12 S3 content | `packs/sheets/obj-romantic.roseArbourSwing.png`, `packs/sheets/age-romantic.roseArbourSwing.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `romantic.candlelitTable` | a dinner for two facing each other, dessert, a drink | candlelit dinner (50), share dessert (20), 2 slots | right uses; the two sit side by side (M-packs-11); 50 is 3.3× the `eat` median (15), a real dinner out is 45–90: not filed |
| `romantic.heartTub` | a bath for two | petal bath (45, 2 slots) | OK |
| `romantic.vanity` | sit at the mirror, make up, do the hair | primp (20, standing) | the stool is never sat on (I-packs-08); 20 vs the `wash` median 2 is getting ready, not a hand wash: not filed |
| `romantic.gramophone` | put a record on, listen, dance | listen (45), waltz (30) | listening is 45 on foot (I-packs-05) |
| `romantic.letterBureau` | write a letter or a poem | write a letter (60), a poem (45) | OK |
| `romantic.roseArbourSwing` | swing for two, tend the roses | swing (40, 2 slots), tend roses (30) | OK; the roses tended at the seat (I-packs-12) |

## P16. Slob (6)

Family: `tools/art/blender/families/pack_slob.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `slob.microwaveCart` Late-night microwave cart | OK | OK | I-packs-19 | I-packs-19 S3 code | `packs/sheets/obj-slob.microwaveCart.png` |
| `slob.beanbag` Crumb-proof beanbag | OK | OK | OK | | `packs/sheets/obj-slob.beanbag.png`, `packs/sheets/age-slob.beanbag.png` |
| `slob.snackNightstand` Snack-stash nightstand | OK | I-packs-07 | I-packs-07 | I-packs-07 S2 content | `packs/sheets/obj-slob.snackNightstand.png`, `packs/play/chunk2/slob.snackNightstand-midnightSnack-01.png` |
| `slob.spritzStation` Spritz & go station | M-packs-02 | OK | OK | M-packs-02 S2 model (seeded; confirmed) | `packs/sheets/obj-slob.spritzStation.png` |
| `slob.arcadeCabinet` Pixel-blaster arcade cabinet | M-packs-07 | OK | I-bathroom-04 | M-packs-07 S2 model; I-bathroom-04 S3 code (systems ledger: no sound) | `packs/sheets/obj-slob.arcadeCabinet.png`, `packs/extra/night/night-slob.arcadeCabinet-chaseHighScore-01.png` |
| `slob.gamingRecliner` Command-couch gaming recliner | OK | A-packs-06 | A-packs-06, I-packs-14 | A-packs-06 S2 content; I-packs-14 S3 content | `packs/sheets/obj-slob.gamingRecliner.png`, `packs/sheets/age-slob.gamingRecliner.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `slob.microwaveCart` | heat something up, improvise a snack | nuke (4), invent (45) | OK; the door opens and shuts (`hold: 1.5`) |
| `slob.beanbag` | flop down, lounge, doze | flop (30), snooze (40) | OK; sunk 11 cm into the bag, as a bean bag gives |
| `slob.snackNightstand` | keep snacks by the bed, reach for one from the bed | midnight snack (8, sitting) | eaten sitting on the nightstand (I-packs-07) |
| `slob.spritzStation` | a quick freshen-up at the mirror | spritz (6) | OK |
| `slob.arcadeCabinet` | play standing at the stick | chase the high score (60), quick round (30) | OK |
| `slob.gamingRecliner` | game for hours in front of a screen, a drink from the armrest | marathon (90), armrest soda (5) | gaming with no screen (I-packs-14); upright (A-packs-06) |

## Findings

Seeded from earlier docs first; then the review's, in the same form.

### M-packs-01 — The entry console's mirror and hook board hang in the air when it stands free

> Review: confirmed and widened to the baking counter's wall shelf (same cause: a wall-hung part built into a floor object).

- **Severity:** S2
- **Rubric:** M7, M4
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_neat.py:384` (`entryConsole`), `web/public/content/packs/neat.json:153`; `tools/art/blender/families/pack_kind.py:184-191` (`neighbourlyOven`: `shelf` and three jars at 1.5 m, `WALL + 0.1`), `web/public/content/packs/kind.json:81`
- **Evidence:** `packs/sheets/obj-neat.entryConsole.png`: the hook board stands beside the mirror over nothing; against a wall it reads as hung. `packs/sheets/obj-kind.neighbourlyOven.png` (gallery, clips): the jar shelf floats 0.6 m over the worktop with nothing behind it on free floor. `slob.spritzStation` is the same cause (M-packs-02). Deferred from 0.3.0 unless the model is touched for another reason; both could get a back panel or a post, or be wall-only.
- **Status:** confirmed and widened
- **Source:** `docs/objects/open.md` Still open › Models; `plan/release-0.3.0.md` §1.4; wave: packs

### M-packs-02 — The spritz station's round mirror hangs in the air when it stands free

- **Severity:** S2
- **Rubric:** M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_slob.py:232`, `web/public/content/packs/slob.json:98`
- **Evidence:** As M-packs-01. Deferred from 0.3.0 unless the model is touched for another reason. Review: `packs/sheets/obj-slob.spritzStation.png`: the round mirror floats over the bottle shelf and the towel panel at its side hangs on nothing.
- **Status:** confirmed
- **Source:** `docs/objects/open.md` Still open › Models; `plan/release-0.3.0.md` §1.4; wave: packs

### M-packs-03 — The party bar's "Back bar" colours the back cabinet, not the bottle shelves

- **Severity:** S3
- **Rubric:** M9
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_outgoing.py:137`, `web/public/content/packs/outgoing.json:5`
- **Evidence:** The colour part named "Back bar" paints the cabinet behind, not the shelves a buyer would expect. Deferred from 0.3.0. Review: `back_cabinet` is `M.wood` (`pack_outgoing.py:141`), the shelves `M.oak` (`:148`), and the paint list names `wood` "Back bar" (`:473`); `packs/sheets/obj-outgoing.partyBar.png`.
- **Status:** confirmed
- **Source:** `docs/objects/open.md` Still open › Models; `plan/release-0.3.0.md` §1.4; wave: packs

### A-packs-01 — Bathers' shins go through the front of the heart tub

> Review: confirmed and widened to the cold plunge (feet through and below its front wall) and the soaking tub (the pelvis 15 cm over the tub floor, hidden by the water).

- **Severity:** S2
- **Rubric:** A1, M2
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters/furniture.json:10-12` (`seats` `gloomy.soakingTub` 0.25, `romantic.heartTub` 0.35, `hotHeaded.coldPlunge` 0.35) and `:56-58` (`beds`, the same); `tools/art/blender/families/pack_romantic.py:205` (`heartTub`), `pack_hotHeaded.py:384` (`coldPlunge`), `pack_gloomy.py:417` (`soakingTub`)
- **Evidence:** `packs/sheets/obj-romantic.heartTub.png` (play: the knees come up over and through the near wall), `packs/sheets/age-romantic.heartTub.png`; the heights check (`heights.txt`): the heart tub's floor is at 0.20 under a 0.35 seat (15 cm), the soaking tub's at 0.095 under 0.25 (15.5 cm). `packs/play/chunk5/hotHeaded.coldPlunge-plunge-01.png`, `packs/sheets/age-hotHeaded.coldPlunge.png`: the plunger's shins and feet pass through the tub's front and show on the floor outside it at every stage but the child (the plunge is 0.86 × 1.39 with the seat in the middle; the heights check measures 0.596 there, the water). Seated bathers in a short tub need the legs drawn up inside or the seat moved back (`fwd`), each checked at every stage.
- **Status:** confirmed and widened
- **Source:** `docs/objects/open.md` Still open › Models; `plan/release-0.3.0.md` §1.4; wave: packs

### A-packs-02 — The power blender's blades loop was never built

- **Severity:** S3
- **Rubric:** A11, I7
- **Fix class:** motion
- **Where:** `tools/art/blender/families/pack_energetic.py:343`, `web/public/content/packs/energetic.json:105`
- **Evidence:** The blender is used but its blades never turn. Review: its motions are `{"uses": {"blend": "lid"}, "hold": 1.0}` only (manifest); `packs/sheets/obj-energetic.powerBlender.png`: the lid lifts, nothing spins.
- **Status:** confirmed
- **Source:** `docs/objects/open.md` Still open › Models; wave: packs

### M-packs-04 — The chef's island's stools are chair height under a counter

- **Severity:** S2
- **Rubric:** M2, M11
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_foodie.py:577-579` (`_stool`: seat at 0.45), `:638-639`; `web/public/content/packs/foodie.json:174` (`surface.height` 0.9, `slots` 2)
- **Evidence:** `packs/sheets/age-foodie.chefsIsland.png`, `packs/play/rerun1/foodie.chefsIsland-barBite-01.png`: a 0.45 m seat under a 0.90 m worktop puts the top at the sitter's chest, 45 cm over the seat (a desk is 28–30 cm over its chair). Counter stools for a 0.90–0.95 top are 0.58–0.71 (`reference.md`, Seats). The sizes table can't see it: the kind has no stool line.
- **Status:** open
- **Source:** wave: packs

### M-packs-05 — The organizer desk and the love-letter bureau turn their backs to the room

- **Severity:** S2
- **Rubric:** M5, M8, M11
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_neat.py:291-326` (`organizerDesk`: the top at depth 0.28, the modesty panel at 0.44 on the front, the chair at −0.12 behind); `tools/art/blender/families/pack_romantic.py:353` (`letterBureau`); `web/public/content/packs/romantic.json:64` (no `seat.facing`); against `web/public/content/packs/bookworm.json:64` (`writingBureau`: `"facing": "back"`)
- **Evidence:** `packs/sheets/obj-neat.organizerDesk.png` (gallery, game camera and clips are all a plain 1.6 × 1.25 m birch board: the hutch's cubbies, the bins, the ledger and the label maker face the wall), `packs/sheets/age-neat.organizerDesk.png` (only the sitters' heads over the board). From the game camera it can't be named (M5). `packs/sheets/obj-romantic.letterBureau.png`: the same with the bureau's pink-framed back. A bureau stands with its back to the wall and the writer facing it, as the pack's own `bookworm.writingBureau` does (`facing: back`); the desk the same, or with its hutch facing the sitter in the room.
- **Status:** open
- **Source:** wave: packs

### M-packs-06 — The hammocks' 1.88 m frames: grown-ups lie with the knees up

- **Severity:** S2
- **Rubric:** M1, A1, A5
- **Fix class:** decision
- **Where:** `tools/art/blender/families/pack_cheerful.py:424-434` (`rainbowHammock`: `SLING_END, SLING_LOW`), `pack_natureLover.py:354-359` (`gardenHammock`: hooks at x 0.88); `web/src/render/babylon/characters/furniture.json:68-69` (`beds`, `across`, `sling`); `web/public/content/packs/cheerful.json:116`, `natureLover.json:144` (2 × 1 footprints)
- **Evidence:** `packs/sheets/age-cheerful.rainbowHammock.png`, `packs/sheets/age-natureLover.gardenHammock.png`: the adult, the teen and the elder lie with the knees bent up and the feet at the far post; only the child fits. A hammock on a stand is 2.7–4 m long (`reference.md` `hammock`; space-saving stands 2.4–2.8). A 2 × 1 footprint can't hold one: a 3 × 1 footprint (model and content), a sitting-swinging pose, or accepting the knees is the owner's call.
- **Status:** open
- **Source:** wave: packs

### M-packs-07 — Things that glow in life stay dark at night; the mood lamps light nothing

- **Severity:** S2
- **Rubric:** M10, I1
- **Fix class:** model (glow materials); content (`light`)
- **Where:** `tools/art/blender/families/pack_couchPotato.py:90` (`wax`, no `glow`), `pack_cheerful.py:26-27` (`amber`, `sunny`: the jukebox's arch and bubble tubes) and `:44` (`lyrics`), `pack_slob.py:37` (`screen_lit`), `pack_outgoing.py:34` (`screen`), `:51` (`ring`), `:58-59` (`tile_a`, `tile_b`), `pack_romantic.py:40-41` (`flame`, `flame_core`); `web/public/content/packs/couchPotato.json:207`, `gloomy.json:150`, `cheerful.json:149` (the three lamps: no `light`)
- **Evidence:** `packs/extra/night/` (`review-stage-night`, 22:18–00:16): the lava lamp's wax is dark red (`night-couchPotato.lavaLamp-zoneOut-00.png`), the "glowing retro jukebox" (its description) is a dark red box (`night-cheerful.jukebox-dance-01.png`), the arcade screen is unlit (`night-slob.arcadeCabinet-chaseHighScore-01.png`), the karaoke screen and the video-call ring light and laptop are dark (`night-outgoing.karaokeStage-practise-01.png`, `night-outgoing.videoCallNook-call-01.png`), the dance floor's tiles are flat colour (`night-outgoing.danceFloor-dance-01.png`), and the candlelit table's candles show no flame (`night-romantic.candlelitTable-candlelitDinner-01.png`). The cinema screen, the sunrise disc, the fire and the journal desk's candle do glow (their materials have `glow`). None of the three mood lamps (lava, rain, sunrise) has a content `light`, so none lights the room or can be switched (a lamp you can switch counts as a use).
- **Status:** open
- **Source:** wave: packs (`check.mjs play --state review-stage-night`)

### M-packs-08 — Two writing desks' tops are 1–2 cm over desk height

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_gloomy.py:245` (`journalDesk`, top 0.78), `pack_hotHeaded.py:323` (`grudgeJournal`, top 0.77)
- **Evidence:** `tables/sizes-packs.md`: top 0.780 and 0.770 against a desk's 0.72–0.76 (5 mm tolerance on tops). `neat.organizerDesk` (0.762) is inside. With seats at 0.46–0.47 the gap to the top (31–32 cm) is still within a desk's; only side by side would anyone see it. `packs/sheets/obj-gloomy.journalDesk.png`, `packs/sheets/obj-hotHeaded.grudgeJournal.png`.
- **Status:** open
- **Source:** wave: packs

### M-packs-09 — The "spin bike" is a recumbent bike

- **Severity:** S3
- **Rubric:** M6, M1
- **Fix class:** decision
- **Where:** `tools/art/blender/families/pack_energetic.py:192` (`spinBike`: a bucket seat with a back at 0.45, the pedals forward), `web/public/content/packs/energetic.json:29`
- **Evidence:** `packs/sheets/obj-energetic.spinBike.png`, `packs/sheets/age-energetic.spinBike.png`: a low seat with a backrest and pedals out in front. A spin bike is upright (saddle 0.75–1.05, `reference.md` `exerciseBike`, which is why the sizes table flags the seat). The rider pedals right on it at every stage; rename it (recumbent bike) or rebuild it upright.
- **Status:** open
- **Source:** wave: packs

### M-packs-10 — The fireside lounge's flue and hood stand behind the chairs, away from the fire

- **Severity:** S3
- **Rubric:** M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_bookworm.py:388` (`firesideLounge`)
- **Evidence:** `packs/extra/night/night-bookworm.firesideLounge-readByFire-01.png`, `packs/sheets/obj-bookworm.firesideLounge.png`: a black suspended hood and flue rise between and behind the two wing chairs while the fire burns in a bowl on a pedestal in front of them; a hood draws smoke only right over the fire.
- **Status:** open
- **Source:** wave: packs

### M-packs-11 — The candlelit table for two seats both diners side by side

- **Severity:** S2
- **Rubric:** M3, M6, I2
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_romantic.py:120` (`candlelitTable`: a narrow top, both `_velvet_chair`s on its back edge), `web/public/content/packs/romantic.json:5` (`slots` 2)
- **Evidence:** `packs/sheets/obj-romantic.candlelitTable.png`, `packs/sheets/age-romantic.candlelitTable.png`, `packs/extra/night/night-romantic.candlelitTable-candlelitDinner-01.png`: a long, about 0.4 m deep top with both chairs behind it facing the room, the place settings side by side. A dinner for two faces across the table. The same pattern as the base dining table (M-kitchen-01, kitchen ledger).
- **Status:** open
- **Source:** wave: packs

### M-packs-12 — Three things are bigger than their kind: the floor cushion, the soaking tub's rim, the bedside stack

- **Severity:** S3
- **Rubric:** M1, M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_hotHeaded.py:281` (`coolDownCushion`), `pack_gloomy.py:417` (`soakingTub`), `pack_bookworm.py:443` (`bedsideStack`)
- **Evidence:** `tables/sizes-packs.md`: the meditation "cushion" is a 0.45 m pouf (a zafu is 0.08–0.22; `packs/sheets/obj-hotHeaded.coolDownCushion.png`), which reads as a pouf to sit cross-legged on; the soaking tub's rim is 0.92 m (0.5–0.8; an ofuro is deep, but this has no step to climb over; `packs/sheets/obj-gloomy.soakingTub.png`); the bedside stack is a 0.85 m wide chest (a nightstand is 0.35–0.66; `packs/sheets/obj-bookworm.bedsideStack.png`).
- **Status:** open
- **Source:** wave: packs

### M-packs-13 — The drum stool, the meditation pouf and the quilted settee don't match their renderer heights

- **Severity:** S3
- **Rubric:** M2, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters/furniture.json:51` (`defaultSeat` 0.45, used by `hotHeaded.thunderDrums`), `:13` (`seats` `hotHeaded.coolDownCushion` 0.40), `:59` (`beds` `hotHeaded.coolDownCushion` 0.2, for a use that never lies), `:77` (`beds` `kind.hearthSettee` 0.44)
- **Evidence:** The heights check (`heights.txt`): the drum stool's mesh is 0.475 under a 0.45 seat (2.5 cm into it), the pouf 0.45 under 0.40 (5 cm into it), the settee's cushion 0.36 under a 0.44 lie (8 cm over it: `packs/play/chunk5/kind.hearthSettee-quiltNap-01.png`). Each is small in the shots; together they are the three-way M2 check failing.
- **Status:** open
- **Source:** wave: packs (`cargo check-glb` heights)

### A-packs-03 — The washer, the baking oven and the glasshouse door stay open for the whole use

- **Severity:** S2
- **Rubric:** A10, I8
- **Fix class:** motion
- **Where:** `tools/art/blender/families/pack_neat.py:447` (`laundryCenter`: `door_open`, no `hold`), `pack_kind.py:512` (`neighbourlyOven`: `oven_open`, no `hold`), `pack_natureLover.py:648` (`glasshouse`: `door_open`, no `hold`)
- **Evidence:** `packs/play/chunk2/neat.laundryCenter-laundry-01.png` and `-fold-01.png`: the washer's porthole is open through the 45-minute wash and the hour's folding (play.jsonl: `door_open` at its end, 0.79/0.79, in every shot); `packs/play/rerun1/kind.neighbourlyOven-familyPie-01.png`: the oven door down through the 48- and 53-minute bakes; `packs/play/chunk4/natureLover.glasshouse-orchids-01.png`: the door open while the plants are tended and while the resident sits inside. The same family's other doors shut after a moment (`hold`: the microwave cart's 1.5, the chef's stove's 3.0, the sauna's 1.5). A washer and an oven are shut while they run.
- **Status:** open
- **Source:** wave: packs (play.jsonl clip times)

### A-packs-04 — The nap pod's hood closes onto the sitter's head

- **Severity:** S2
- **Rubric:** A1, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_lazy.py:170-204` (`HOOD` centre at 0.95, radius 0.48; `hood` swings 60° forward)
- **Evidence:** `packs/play/chunk3/lazy.napPod-powerNap-01.png`, `packs/sheets/obj-lazy.napPod.png`: with the hood over, an adult's head goes up into the hood shell; the seat (0.45) puts a seated adult's crown at about 1.3 m, inside the 0.95 + 0.48 m sphere the hood turns on. A child clears it (`packs/sheets/age-lazy.napPod.png`). `loner.quietPod`'s shell clears the head.
- **Status:** open
- **Source:** wave: packs

### A-packs-05 — Dozing on the window seat: face down, across the cushion, the head at the glass

- **Severity:** S2
- **Rubric:** A1, I3
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters/furniture.json:76` (`beds` `gloomy.windowSeat`: `h` 0.45, `across`), `web/public/content/packs/gloomy.json:173` (`doze`: `pose: lie`)
- **Evidence:** `packs/play/chunk3/gloomy.windowSeat-doze-01.png`: the dozer lies prone, diagonally over the back of the cushion, the face on the window glass, the feet at the left pillow. `across` should lay them along the seat on the back, head to its left, as `kind.hearthSettee` does (`packs/play/chunk5/kind.hearthSettee-quiltNap-01.png`). The cushion between the two pillows is about 1.3 m clear, short for an adult lying flat (a curled-up side lie fits a window seat).
- **Status:** open
- **Source:** wave: packs

### A-packs-06 — The recliners tip their backs and raise the footrests; the sitter stays upright, feet on the floor

- **Severity:** S2
- **Rubric:** A10, A1, I3
- **Fix class:** content
- **Where:** `web/public/content/packs/couchPotato.json:116` (`couchPotato.recliner`: no `seat`), `web/public/content/packs/slob.json:159` (`slob.gamingRecliner`: no `seat`); `web/src/render/babylon/characters.ts:228` (`seatOf`: `recline` from content `seat.recline`, as `slob.beanbag` has at `slob.json:50`)
- **Evidence:** `packs/play/chunk1/couchPotato.recliner-recline-02.png`, `packs/sheets/obj-slob.gamingRecliner.png` (play): the `recline` ride clip leans the chair back and lifts the footrest ("Feet up. Way up."), while the resident sits straight up with the feet on the floor in front of the footrest: the ride carries nothing (A10). A content `seat` with `recline` (and the legs out on the footrest) is the existing mechanism. Also likely on the living room's `recliner` (`catalog2/living.json:436`, no `seat`; living ledger, not reviewed here).
- **Status:** open
- **Source:** wave: packs

### I-packs-01 — At the quilt table and the candlelit table the legs come out through the front

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_lazy.py:213` (`quiltTable`: a solid quilt skirt to the floor in front of the bench), `pack_romantic.py:120` (`candlelitTable`: the deep front apron); `web/public/content/packs/lazy.json:102`, `romantic.json:5`
- **Evidence:** `packs/play/chunk3/lazy.quiltTable-tuckIn-01.png`, `packs/sheets/age-lazy.quiltTable.png`: the sitter's shins and feet stick out of the front of the red quilt at every stage but the child (the bench is at the back, the table about 0.9 m deep). `packs/play/chunk4/romantic.candlelitTable-candlelitDinner-01.png`, `packs/sheets/age-romantic.candlelitTable.png`: the diner's legs pass through the table to its near side. A kotatsu is sat at with the legs under the quilt, so the knees need room under the top and the feet must stay inside.
- **Status:** open
- **Source:** wave: packs (leads from the evidence index's "Seen")

### I-packs-02 — Exercise and singing next to the board, the mat and the stage, not on them

- **Severity:** S2
- **Rubric:** I2
- **Fix class:** content
- **Where:** `web/public/content/packs/couchPotato.json:184` (`fitnessGame`), `cheerful.json:149` (`sunriseLamp`), `outgoing.json:63` (`karaokeStage`): no `stand` surface; the mechanism as `energetic.json:58` (`trampoline`: `"stand": {"height": 0.5, ...}`) and `lazy.json:169` (`vibePlate`)
- **Evidence:** `packs/sheets/obj-couchPotato.fitnessGame.png` (play): the workout is danced on the floor to the left of the balance board; `packs/sheets/obj-cheerful.sunriseLamp.png`, `packs/extra/night/night-cheerful.sunriseLamp-stretch-01.png`: stretches beside the mat; `packs/sheets/obj-outgoing.karaokeStage.png`: the singer stands on the floor in front of the stage's edge, the two mics 0.4 m behind her. Each thing's point is the surface it has.
- **Status:** open
- **Source:** wave: packs

### I-packs-03 — Bouldering on the floor beside the crash mat

> Consolidation: I-residents-01 (residents ledger) merged into it for the bouldering wall; its fitness-game half (danced beside the balance board) is I-packs-02's.

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:490` (`ACTIONS.exercise`: `idle` for `boulder`) and `:3384` (`activityPose`: climbing in place); `web/public/content/packs/energetic.json:5`
- **Evidence:** `packs/play/chunk2/energetic.boulderingWall-climb-01.png`, `packs/sheets/obj-energetic.boulderingWall.png`: for an hour the climber stands on the floor off the end of the mat, reaching for holds overhead in turn; the feet never leave the floor and the hands don't touch the wall. Climbing needs the body on the wall (a rise up the panel and back, the hands and feet on holds).
- **Status:** confirmed and widened (consolidation: I-residents-01 merged)
- **Source:** wave: packs

### I-packs-04 — On the vibration plate the resident turns their back to the handlebar

- **Severity:** S2
- **Rubric:** I2, A1, M8
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_lazy.py:315-316` (`column` and `handlebar` at depth −0.22 to −0.3, the plate's back)
- **Evidence:** `packs/play/chunk3/lazy.vibePlate-shake-02.png`, `packs/sheets/obj-lazy.vibePlate.png`: the stander faces the object's front, so the column and console are behind them and the bar sticks out at the hip; the hands hold nothing. On a real plate you face the console and hold the bar. The column belongs at the front, or the stander faces the back.
- **Status:** open
- **Source:** wave: packs

### I-packs-05 — Long sit-down uses done standing at the object (the packs' share of I-living-03)

> Consolidation (2026-10-21): closed as a duplicate of I-living-03 (living ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I2, I3
- **Fix class:** content
- **Where:** `web/public/content/packs/bookworm.json:27` (`libraryWall`: research 60, browse 45), `:98` (`teaTrolley`: brew 20), `:154` (`bedsideStack`: chapter 20); `gloomy.json:39` (`recordCabinet`: listen 60, newAlbum 45); `romantic.json:126` (`gramophone`: listen 45); `lazy.json:188` (`smartHub`: program 60); `neat.json:127` (`robotVacuumDock`: program 45); `crates/sim-core/src/content.rs:79` (`seatNear`)
- **Evidence:** Same cause as I-living-03 (living ledger: the bookshelf's uses and the TV lack `seatNear`). `packs/play/chunk1/bookworm.libraryWall-research-01.png`, `packs/sheets/obj-gloomy.recordCabinet.png`, `packs/sheets/obj-romantic.gramophone.png`, `packs/sheets/obj-lazy.smartHub.png`: the resident stands beside the thing, arms folded or the hands at the chest, for 20–60 minutes: a record is listened to and a library book read in a chair, the bedside chapter in bed, the tea drunk sitting. Giving these `seatNear` (a seat within a few metres) fixes the common case; what happens with no seat near is I-living-03's triage. The coordinator may merge this into I-living-03.
- **Status:** closed (duplicate of I-living-03)
- **Source:** wave: packs

### I-packs-06 — Tending the waist-high herb garden is the flower bed's kneel

> Consolidation (2026-10-21): closed as a duplicate of I-garden-04 (residents ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:508` (`ACTIONS.tend`: `kneel`); compare `:512` (`ACTIONS.harvest`: `pick` for `garden.herbs`, the standing waist-high case); `web/public/content/packs/foodie.json:5` (`tendHerbs`: `anim: tend`)
- **Evidence:** `packs/play/chunk1/foodie.herbGarden-tendHerbs-01.png`: the resident kneels on the floor beside the stand's legs, the herbs 0.9 m up out of reach. A planter at worktop height is tended standing. (The cot's `feed` also plays this kneel: I-kids-04, kids ledger.)
- **Status:** closed (duplicate of I-garden-04)
- **Source:** wave: packs

### I-packs-07 — The midnight snack is eaten sitting on the nightstand

- **Severity:** S2
- **Rubric:** I3, I2
- **Fix class:** content
- **Where:** `web/public/content/packs/slob.json:76` (`snackNightstand`: `midnightSnack` `pose: sit`, no `seat`, so `defaultSeat` 0.45 on its own top, `furniture.json:51`)
- **Evidence:** `packs/play/chunk2/slob.snackNightstand-midnightSnack-01.png`: the resident sits on top of the 0.45 m nightstand for 8 minutes. A snack from the nightstand is reached for from the bed, or taken standing; a `stand` pose (or `seatNear` for the bed) fits.
- **Status:** open
- **Source:** wave: packs

### I-packs-08 — Primping at the vanity on foot; its stool is never sat on

- **Severity:** S2
- **Rubric:** I3, I2
- **Fix class:** content
- **Where:** `web/public/content/packs/romantic.json:101` (`vanity`: `primp` `pose: stand`, no `seat`); compare `web/src/render/babylon/characters/furniture.json:31` (`dressingTable`: seated facing the mirror, `back: true`)
- **Evidence:** `packs/sheets/obj-romantic.vanity.png` (play): 20 minutes standing beside the pink pouf, half a metre from the mirror. A vanity is sat at, facing the mirror; the base `dressingTable` already does it.
- **Status:** open
- **Source:** wave: packs

### I-packs-09 — The glasshouse's orchids are tended from outside the door

- **Severity:** S2
- **Rubric:** I2
- **Fix class:** content
- **Where:** `web/public/content/packs/natureLover.json:229` (`glasshouse`: `orchids` stands at the front, no `inside`; `unwind` sits inside)
- **Evidence:** `packs/play/chunk4/natureLover.glasshouse-orchids-01.png`: for an hour the resident stands outside the open door, the plants on the shelves a metre in. `unwind` already takes them inside to the bench.
- **Status:** open
- **Source:** wave: packs

### I-packs-10 — Video calls are played as a phone at the ear

- **Severity:** S3
- **Rubric:** I3
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:530` (`ACTIONS.phone`: `phone`, the hand at the ear); `web/public/content/packs/outgoing.json:100` (`videoCallNook` `call`), `cheerful.json:173` (`partyBooth` `party`)
- **Evidence:** `packs/extra/night/night-outgoing.videoCallNook-call-01.png`, `packs/play/chunk3/cheerful.partyBooth-party-01.png`: the caller sits at the laptop (and the ring light) holding a hand to the ear; in the booth the laptop faces the other bench. A video call is talked to the screen, the hands free or on the keys (`sitHands`). The phone prop itself is I-social-01 (social ledger).
- **Status:** open
- **Source:** wave: packs

### I-packs-11 — The tea guests and the lava-lamp watcher face away from what they're there for

- **Severity:** S3
- **Rubric:** I2
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_kind.py:341` (`welcomeTeaTable`: a pouf either side of the table, both sitters facing the front); `tools/art/blender/families/pack_couchPotato.py:425` (`lavaLamp`: the lamp behind the chair's shoulder)
- **Evidence:** `packs/sheets/age-kind.welcomeTeaTable.png`, `packs/play/chunk5/kind.welcomeTeaTable-shareTea-01.png`: host and guest sit side-on to the table between them, both facing the room, not each other. `packs/extra/night/night-couchPotato.lavaLamp-zoneOut-00.png`: "Ooh, the blobs" with the lamp behind the sitter's head.
- **Status:** open
- **Source:** wave: packs

### I-packs-12 — Worked from the object's edge, not at the part: the bonsai, the roses, the lectern, the hoop

- **Severity:** S3
- **Rubric:** I2, I3
- **Fix class:** content
- **Where:** `web/public/content/packs/loner.json:109` (`bonsaiBench`), `romantic.json:160` (`roseArbourSwing` `tendRoses`), `foodie.json:102` (`cookbookLectern`, and `web/src/render/babylon/characters.ts:475`: `ACTIONS.read` holds a book that isn't there), `energetic.json:80` (`streetballHoop`)
- **Evidence:** `packs/sheets/obj-loner.bonsaiBench.png`: the pruner stands at the bench's left corner by the soil bag, not at the tree in the middle; `packs/sheets/obj-romantic.roseArbourSwing.png` (tendRoses): the roses are on the posts and the top, the hands at the seat cushion; `packs/sheets/obj-foodie.cookbookLectern.png`: a step off the lectern, reading a mimed book at the chest beside the open one; `packs/sheets/obj-energetic.streetballHoop.png`: shooting from under the board, at the pole. Slot offsets (`slot_position`, `approach`) or a lectern-reading hand pose.
- **Status:** open
- **Source:** wave: packs

### I-packs-13 — Smashing plates is a boxing combination in the air

- **Severity:** S3
- **Rubric:** I3
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:483` (`ACTIONS.exercise`: `punchJab`, `punchCross` for `smash`)
- **Evidence:** `packs/sheets/obj-hotHeaded.smashBin.png` (play): the resident jabs at nothing beside the plate stack; no plate is picked up or thrown. A pick-up then an overarm throw into the bin reads (the plate is I-held-01, held ledger).
- **Status:** open
- **Source:** wave: packs

### I-packs-14 — A gaming marathon in the recliner with no screen to play on

- **Severity:** S3
- **Rubric:** I16
- **Fix class:** content
- **Where:** `web/public/content/packs/slob.json:159` (`gamingRecliner` `marathon`: `anim: play`, no `seatNear` or `requires` for a screen)
- **Evidence:** `packs/sheets/obj-slob.gamingRecliner.png` (play): 90 minutes of `sitHands` gaming into the air; the set has side tables and a headset stand but no screen, and the use doesn't ask for a TV in front of it.
- **Status:** open
- **Source:** wave: packs

### I-packs-15 — The robot vacuum only peeks out of its dock

- **Severity:** S3
- **Rubric:** I7, A9
- **Fix class:** motion
- **Where:** `tools/art/blender/families/pack_neat.py:343` (`robotVacuumDock`: `vacuum_out` slides it out and back), `:457`
- **Evidence:** `packs/sheets/obj-neat.robotVacuumDock.png`: the robot slides a little way out and back; it never goes round the room or its rug, and the use is 45 minutes of the owner programming it on foot (I-packs-05). A loop that drives the robot round the rug while the use lasts would read as cleaning.
- **Status:** open
- **Source:** wave: packs

### I-packs-16 — Missing obvious uses: the jukebox can't be listened to; the party bar's stools are never sat on

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/packs/cheerful.json:40` (`jukebox`: `dance` only), `outgoing.json:5` (`partyBar`: `mixDrinks`, `soda`, both standing)
- **Evidence:** `packs/sheets/obj-cheerful.jukebox.png`: picking a song and listening, the jukebox's first use, isn't offered (the record cabinet and the gramophone have `listen`). `packs/sheets/obj-outgoing.partyBar.png`: the bar is built with two stools for its guests but both uses stand at the guest side; a seated drink at the bar is missing.
- **Status:** open
- **Source:** wave: packs

### I-packs-17 — The sauna-goer's feet poke through the shut glass door

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** model
- **Where:** `tools/art/blender/families/pack_loner.py:344` (`soloSauna`: the bench and the door at `FRONT`), `web/public/content/packs/loner.json:140`
- **Evidence:** `packs/play/chunk4/loner.soloSauna-sweat-02.png`: with the door shut again (`hold: 1.5`, play.jsonl `door_open` 0/0.79), the seated adult's shoes stand at and past the glass. The bench needs to sit further back, or the cabin to be deeper in front of it.
- **Status:** open
- **Source:** wave: packs

### I-packs-18 — Folding one load takes an hour

- **Severity:** S3
- **Rubric:** I4
- **Fix class:** content
- **Where:** `web/public/content/packs/neat.json:30` (`laundryCenter` `fold`: 60 minutes)
- **Evidence:** `tables/uses-packs.md`: 6× the `clean` median (10). The wash (45) is a machine's cycle and fine; folding a load is 10–20 minutes. The only `clean` outlier of the packs that isn't a different kind of job.
- **Status:** open
- **Source:** wave: packs (`cargo review uses`)

### I-packs-19 — A blender, a cheese board, a microwave and a pie sizzle; a story plays music

> Consolidation (2026-10-21): closed as a duplicate of I-bathroom-04 (systems ledger), which takes its items and evidence.

- **Severity:** S3
- **Rubric:** I9
- **Fix class:** code
- **Where:** `web/src/audio/world.ts:35` (`ACTION_SOUNDS.cook`: `sizzle`) and `:45` (`listen`: `music`); `web/public/content/packs/energetic.json:105` (`blend`), `foodie.json:53` (`deliPantry` `cheeseBoard`), `slob.json:6` (`microwaveCart`), `kind.json:81` (`neighbourlyOven`), `lazy.json:188` (`smartHub` `story`)
- **Evidence:** `tables/uses-packs.md` Sound column: every `cook`-tagged use sounds like frying, so the smoothie, the cold cheese board, the microwave and the oven bake all sizzle; the smart speaker's story plays the music loop. The sound follows the clip's tag, not the thing. Missing sounds (singing, the punch bag, the arcade) are I-bathroom-04's (systems ledger). Shared beyond this ledger: any `cook` use away from a hob (31 in the other content files).
- **Status:** closed (duplicate of I-bathroom-04)
- **Source:** wave: packs
