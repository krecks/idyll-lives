# Personality packs

The traits' own things (`web/public/content/packs/<trait>.json`, placeholders from `tools/art/pack_<trait>.py`). One
task per pack, one Blender family per pack (`families/pack_<trait>.py`), one Validate group per pack. Pack
things are the residents' favourites, used often: give each its clip.

Each family writes its own override pack, `packs/pack_<trait>/manifest.json`, keyed `model.<trait>.<id>`, so the
trait's pack keeps its placeholders as the fallback (`families/pack_bookworm.py` and `.md` are the pattern). P1 is
done.

## P1. Bookworm (6) — **Done** (in Validate › Bookworm)

- **Family:** `tools/art/blender/families/pack_bookworm.py`
- **Validate group:** `bookworm` (Validate › Bookworm)
- **Placeholders to match:** `tools/art/pack_bookworm.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bookworm.readingNook` Window-seat reading nook | 2x1 · floor | curlUp:read | living / — | Wood, Fabric | Use | `sink_in`: the window-seat cushion settles for curlUp. |
| `bookworm.libraryWall` Rare-books library wall | 2x1 · floor | research:read browse:read | office / — | Wood, Fabric | Use | `ladder`: the rolling ladder slides 40 cm along its rail for research and browse. |
| `bookworm.writingBureau` Writer's bureau | 1x1 · floor | draft:write letter:write | office / — | Wood, Fabric | Use | `fall_front`: the writing flap drops 90° for draft and letter, and closes after. |
| `bookworm.teaTrolley` Book-club tea trolley | 1x1 · floor | brew:drink | kitchen / — | Wood, Fabric | Use | `pour`: the teapot tips 30° for brew; `hold` 1.5 s. |
| `bookworm.firesideLounge` Fireside reading lounge | 2x2 · floor | readByFire:read readAloud:read | living / — | Wood, Fabric | Use | `flames`: the fire grows in the grate for readByFire and readAloud. |
| `bookworm.bedsideStack` Bedside book stack | 1x1 · floor | chapter:read | bedroom / — | Wood, Fabric | Use | `book_up`: the top book lifts and opens for chapter. |

## P2. Cheerful (6)

- **Family:** `tools/art/blender/families/pack_cheerful.py`
- **Validate group:** `cheerful` (Validate › Cheerful)
- **Placeholders to match:** `tools/art/pack_cheerful.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `cheerful.jukebox` Sunburst jukebox | 1x1 · floor | dance:dance | living / — | Body, Trim | Use | `record`: a record swings up into place for dance. L it spins (E1). |
| `cheerful.karaokeMachine` Spotlight karaoke machine | 1x1 · floor | sing:sing | hobbies / — | Body, Trim | Use | `mic_up`: the microphone rises on its stand for sing. |
| `cheerful.mocktailCart` Fizz & sunshine mocktail cart | 1x1 · floor | mix:drink | kitchen / — | Body, Trim | Use | `shake`: the shaker tips for mix. |
| `cheerful.rainbowHammock` Rainbow hammock | 2x1 · floor | swing:relax doze:nap | outdoor / — | Body, Trim | Loop (E1) | `sway` (E1). Until then static. |
| `cheerful.sunriseLamp` Sunrise stretch lamp | 1x1 · floor | stretch:exercise | bedroom / — | Body, Trim | Use | `sunrise`: the sun disc rises in its frame for stretch. N glow (E8). |
| `cheerful.partyBooth` Confetti party booth | 2x2 · floor | party:phone | living / — | Body, Trim | Use | `curtain`: the booth curtain swings aside for party (`hold` 1.5 s). |

## P3. Couch potato (6)

- **Family:** `tools/art/blender/families/pack_couchPotato.py`
- **Validate group:** `couch potato` (Validate › Couch Potato)
- **Placeholders to match:** `tools/art/pack_couchPotato.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `couchPotato.cinemaWall` Grand cinema wall | 2x1 · floor | marathon:watch channelSurf:watch | living / — | Body, Upholstery | Use | `curtains`: the side curtains slide apart for marathon and channelSurf. |
| `couchPotato.gameConsole` Pixel Arcade console | 1x1 · floor | play:play speedrun:play | hobbies / — | Body, Upholstery | Use | `tray`: the disc tray slides out and back for play and speedrun. |
| `couchPotato.recliner` Cloud Nine power recliner | 1x1 · floor | recline:relax doze:nap | living / — | Body, Upholstery | Use | `recline`: the back tips and the footrest rises for recline and doze. |
| `couchPotato.snackFridge` Sofa-side snack fridge | 1x1 · floor | snack:fetch takeout:fetch | kitchen / — | Body, Upholstery | Use | `fridge_open`: the door swings for snack and takeout. |
| `couchPotato.fitnessGame` Living-room workout game | 1x1 · floor | workout:exercise | fitness / — | Body, Upholstery | Use | `board`: the balance board tilts 10° for workout. |
| `couchPotato.lavaLamp` Lava glow lamp | 1x1 · floor | zoneOut:relax | decor / — | Body, Upholstery | Use | `blobs`: the wax blobs rise slowly for zoneOut. A drifting blobs (E2). |

## P4. Energetic (7)

- **Family:** `tools/art/blender/families/pack_energetic.py`
- **Validate group:** `energetic` (Validate › Energetic)
- **Placeholders to match:** `tools/art/pack_energetic.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `energetic.boulderingWall` Home bouldering wall | 2x1 · floor | climb:exercise | fitness / — | Frame, Pads | Static | Static (the holds are the shape). |
| `energetic.spinBike` Interactive spin bike | 1x2 · floor | ride:exercise | fitness / — | Frame, Pads | Loop (E1) | `pedals` (E1). Until then static. |
| `energetic.trampoline` Rebound trampoline | 2x2 · floor | bounce:exercise | outdoor / — | Frame, Pads | Loop (E1) | `bounce`: the mat dips while bounced on (E1). Until then U: dips once. |
| `energetic.streetballHoop` Streetball hoop | 1x1 · floor | shoot:exercise | outdoor / — | Frame, Pads | Use | `swish`: the net sways after a shot. |
| `energetic.powerBlender` Power blender station | 1x1 · floor | blend:cook | kitchen / — | Frame, Pads | Use | `lid`: the jug lid lifts and settles for blend. L blades (E1). |
| `energetic.contrastShower` Contrast rain shower | 1x1 · floor | rinse:shower | bathroom / — | Frame, Pads | Use | `water_on`: the rain streams grow for rinse. |
| `energetic.recoveryPod` Recovery pod | 1x2 · floor | recover:relax | fitness / — | Frame, Pads | Use | `canopy`: the canopy lowers over them for recover and lifts after. |

## P5. Example pack (2)

- **Family:** `tools/art/blender/families/pack_example.py`
- **Validate group:** `example` (Validate › Example)
- **Placeholders to match:** base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `chefStove` Chef's stove | 2x1 · floor | cook:cook practise:cook | kitchen / — | Body | Use | `oven_open` (door drops) and `burner_on` (knob and flame ring) for cook and practise. |
| `whirlpoolTub` Whirlpool tub | 2x2 · floor | soak:bath | wellness / — | Tub, Surround | Use | `fill`: the water rises for soak. L bubbles (E1). |

## P6. Foodie (6)

- **Family:** `tools/art/blender/families/pack_foodie.py`
- **Validate group:** `foodie` (Validate › Foodie)
- **Placeholders to match:** `tools/art/pack_foodie.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `foodie.herbGarden` Windowsill herb garden | 1x1 · floor | tendHerbs:tend | decor / — | Body, Worktop | Use | `sprig`: a fresh sprig grows in a pot for tendHerbs. |
| `foodie.deliPantry` Deli pantry | 1x1 · floor | nibble:fetch cheeseBoard:cook | kitchen / — | Body, Worktop | Use | `doors_open`: the doors swing for nibble and cheeseBoard. |
| `foodie.espressoBar` Espresso bar | 1x1 · floor | espresso:drink latteArt:drink | kitchen / — | Body, Worktop | Use | `brew`: the coffee stream grows and the steam wand swings out for espresso and latteArt. |
| `foodie.cookbookLectern` Cookbook lectern | 1x1 · floor | studyRecipes:read foodMagazines:read | kitchen / — | Body, Worktop | Use | `page`: a page turns over for studyRecipes and foodMagazines. |
| `foodie.pizzaOven` Wood-fired pizza oven | 2x1 · floor | bakePizza:cook warmUp:relax | outdoor / — | Body, Worktop | Use | `door_off`: the oven door lifts aside and the peel slides in for bakePizza; `flames` grow for warmUp. |
| `foodie.chefsIsland` Chef's island | 2x1 · floor | tastingMenu:cook barBite:eat | kitchen / — | Body, Worktop | Use | `drawer_open` for tastingMenu. |

## P7. Gloomy (6)

- **Family:** `tools/art/blender/families/pack_gloomy.py`
- **Validate group:** `gloomy` (Validate › Gloomy)
- **Placeholders to match:** `tools/art/pack_gloomy.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `gloomy.recordCabinet` Midnight record cabinet | 1x1 · floor | listen:listen newAlbum:listen | living / — | Body, Trim | Use | `lid_open`: the lid lifts for listen and newAlbum. L the platter turns (E1). |
| `gloomy.cello` Nocturne cello | 1x1 · floor | practise:music lament:music | hobbies / — | Body, Trim | Use | `off_stand` if the music animation holds it. |
| `gloomy.journalDesk` Candlelit writing desk | 2x1 · floor | journal:write poetry:write | office / — | Body, Trim | Use | `candle`: the candle flame grows for journal and poetry. |
| `gloomy.rainLamp` Rainfall lamp | 1x1 · floor | watchRain:look | decor / — | Body, Trim | Use | `rain`: the rain strands grow for watchRain. A rain (E2). |
| `gloomy.windowSeat` Brooding window seat | 2x1 · floor | brood:look doze:nap | living / — | Body, Trim | Use | `sink_in`. |
| `gloomy.soakingTub` Moonstone soaking tub | 2x2 · floor | soak:bath | bathroom / — | Body, Trim | Use | `fill`: the water rises for soak. |

## P8. Hot-headed (6) — **Done** (in Validate › Hot Headed)

- **Family:** `tools/art/blender/families/pack_hotHeaded.py`
- **Validate group:** `hot headed` (Validate › Hot Headed)
- **Placeholders to match:** `tools/art/pack_hotHeaded.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `hotHeaded.heavyBag` Heavy punching bag | 1x1 · floor | pound:exercise | fitness / — | Body | Loop (E1) | `swing` while punched (E1). Until then U: swings 15° and settles. |
| `hotHeaded.smashBin` Plate-smash bin | 1x1 · floor | smash:exercise | hobbies / — | Body | Use | `lid_open`: the lid flips up for smash; `hold` 1 s. |
| `hotHeaded.thunderDrums` Thunder drum kit | 2x1 · floor | drum:music | hobbies / — | Body | Loop (E1) | `cymbals` (E1). Until then static. |
| `hotHeaded.coolDownCushion` Cool-down meditation cushion | 1x1 · floor | countToTen:meditate breathe:meditate | decor / — | Body | Use | `sink_in`. |
| `hotHeaded.grudgeJournal` Grudge-to-gratitude writing desk | 1x1 · floor | writeItOut:write | office / — | Body | Use | `drawer_open` for writeItOut. |
| `hotHeaded.coldPlunge` Glacier cold-plunge tub | 1x2 · floor | plunge:bath | bathroom / — | Body | Use | `cover_off`: the cover slides off for plunge. |

## P9. Kind (6) — **Done** (in Validate › Kind)

- **Family:** `tools/art/blender/families/pack_kind.py`
- **Validate group:** `kind` (Validate › Kind)
- **Placeholders to match:** `tools/art/pack_kind.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kind.neighbourlyOven` Neighbourly baking counter | 2x1 · floor | familyPie:cook treatsToShare:cook | kitchen / — | Body, Fabric | Use | `oven_open` for familyPie and treatsToShare. |
| `kind.givingJar` Community giving jar | 1x1 · floor | donate:idle | decor / — | Body, Fabric | Use | `coin`: a coin slides into the slot for donate. |
| `kind.knittingRocker` Gift-knitting rocker | 1x1 · floor | knitGift:tinker rock:relax | hobbies / — | Body, Fabric | Loop (E1) | `rock` (E1). Until then static. |
| `kind.welcomeTeaTable` Welcome tea table | 2x1 · floor | shareTea:drink | living / — | Body, Fabric | Use | `pour`: the teapot tips for shareTea. |
| `kind.songbirdFeeder` Songbird feeding station | 1x1 · floor | refill:garden watchBirds:look | outdoor / — | Body, Fabric | Use | `lid_open` for refill; `bird`: birds appear for watchBirds. |
| `kind.hearthSettee` Hearthside quilted settee | 2x1 · floor | curlUp:relax quiltNap:nap | living / — | Body, Fabric | Use | `sink_in`. |

## P10. Lazy (6) — **Done** (in Validate › Lazy)

- **Family:** `tools/art/blender/families/pack_lazy.py`
- **Validate group:** `lazy` (Validate › Lazy)
- **Placeholders to match:** `tools/art/pack_lazy.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `lazy.snoozecloudBed` Snoozecloud adjustable bed | 2x2 · floor | sleep:sleep sleepIn:nap | bedroom / — | Body, Upholstery | Use | `head_up`: the head section rises 30° for sleepIn (check `BEDS`). |
| `lazy.napPod` Daydream nap pod | 1x1 · floor | curlUp:relax powerNap:nap | living / — | Body, Upholstery | Use | `hood`: the hood swings half over for powerNap and curlUp. |
| `lazy.quiltTable` Heated quilt table | 2x1 · floor | tuckIn:relax doze:nap | living / — | Body, Upholstery | Use | `quilt`: the quilt edge lifts for tuckIn and doze. |
| `lazy.deliveryLocker` Doorstep delivery locker | 1x1 · floor | takeout:fetch noodles:fetch | kitchen / — | Body, Upholstery | Use | `door_open`: a locker door opens for takeout and noodles; `hold` 2 s. |
| `lazy.vibePlate` Effortless vibration plate | 1x1 · floor | shake:exercise | fitness / — | Body, Upholstery | Loop (E1) | `shake` (E1). Until then static. |
| `lazy.smartHub` Smart-home command hub | 1x1 · floor | program:type story:listen | office / — | Body, Upholstery | Use | `screen_up`: the tablet tilts up for program and story. |

## P11. Loner (6)

- **Family:** `tools/art/blender/families/pack_loner.py`
- **Validate group:** `loner` (Validate › Loner)
- **Placeholders to match:** `tools/art/pack_loner.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `loner.quietPod` Hush-shell reading pod | 1x1 · floor | retreat:relax read:read | living / — | Body, Wood | Use | `shell`: the hood swings half closed for retreat and read. |
| `loner.shortwaveRadio` Shortwave radio desk | 1x1 · floor | airwaves:phone tune:tinker | office / — | Body, Wood | Use | `dial`: the tuning dial turns for tune; the antenna extends for airwaves. |
| `loner.puzzleTable` Jigsaw puzzle table | 1x1 · floor | piece:play newPuzzle:play | hobbies / — | Body, Wood | Use | `pieces`: a patch of puzzle grows for piece and newPuzzle. |
| `loner.bonsaiBench` Bonsai potting bench | 1x1 · floor | shape:prune tend:water | hobbies / — | Body, Wood | Use | `trim`: a twig shrinks away for shape. |
| `loner.soloSauna` Cedar sauna for one | 1x2 · floor | sweat:relax | bathroom / — | Body, Wood | Use | `door_open`: the door opens and shuts behind them (`hold` 1.5 s) for sweat. |
| `loner.deliveryHatch` No-knock delivery hatch | 1x1 · floor | order:fetch treat:fetch | kitchen / — | Body, Wood | Use | `hatch`: the hatch opens for order and treat; `hold` 2 s. |

## P12. Nature lover (7)

- **Family:** `tools/art/blender/families/pack_natureLover.py`
- **Validate group:** `nature lover` (Validate › Nature Lover)
- **Placeholders to match:** `tools/art/pack_natureLover.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `natureLover.veggieBed` Raised veggie bed | 2x1 · floor | tendVeggies:tend pickSnack:eat | garden / edibles | Wood | Use | `harvest`: a vegetable shrinks away for pickSnack. G growth (E4). |
| `natureLover.pottingBench` Potting bench | 2x1 · floor | potSeedlings:tinker tieBouquet:tinker | hobbies / — | Wood | Use | `seedling`: a seedling grows in a pot for potSeedlings. |
| `natureLover.birdFeeder` Bird feeder post | 1x1 · floor | birdwatch:look | outdoor / — | Wood | Use | `bird`: a bird appears for birdwatch. |
| `natureLover.gardenHammock` Garden hammock | 2x1 · floor | swing:relax doze:nap | outdoor / — | Wood | Loop (E1) | `sway` (E1). Until then static. |
| `natureLover.cedarShower` Cedar outdoor shower | 1x1 · floor | rinse:shower | outdoor / — | Wood | Use | `water_on`: the stream grows for rinse. |
| `natureLover.mossTerrarium` Moss terrarium | 1x1 · floor | mist:water | decor / — | Wood | Use | `lid_open`: the glass lid lifts for mist. |
| `natureLover.glasshouse` Fern glasshouse | 2x2 · floor | unwind:relax orchids:water | outdoor / — | Wood | Use | `door_open`: the door and a roof vent open for unwind and orchids. |

## P13. Neat (6)

- **Family:** `tools/art/blender/families/pack_neat.py`
- **Validate group:** `neat` (Validate › Neat)
- **Placeholders to match:** `tools/art/pack_neat.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `neat.steamMopDock` Steam mop dock | 1x1 · floor | mop:clean | kitchen / — | Body | Use | `mop_out`: the mop leaves its dock for mop (hidden while in the hands, if held). |
| `neat.laundryCenter` Stacked laundry centre | 2x1 · floor | laundry:clean fold:clean | bathroom / — | Body | Use | `door_open`: the washer door swings for laundry and fold. |
| `neat.rainShowerTower` Rainfall steam shower | 1x1 · floor | steamShower:shower | bathroom / — | Body | Use | `water_on`: the rain grows for steamShower. |
| `neat.organizerDesk` Label-and-sort organizer desk | 2x1 · floor | sort:write ledger:write | office / — | Body | Use | `drawer_open` for sort and ledger. |
| `neat.robotVacuumDock` Robot vacuum and dock | 1x1 · floor | program:type | living / — | Body | Use | `vacuum_out`: the robot rolls 60 cm out of its dock, turns and rolls back for program. |
| `neat.entryConsole` Tidy entry console | 1x1 · floor | tidy:clean | decor / — | Body | Use | `tray`: the key tray drawer slides out for tidy. |

## P14. Outgoing (6)

- **Family:** `tools/art/blender/families/pack_outgoing.py`
- **Validate group:** `outgoing` (Validate › Outgoing)
- **Placeholders to match:** `tools/art/pack_outgoing.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `outgoing.partyBar` Home party bar | 2x1 · floor | mixDrinks:drink soda:drink | kitchen / — | Body, Upholstery | Use | `tap`: the beer tap lever pulls for mixDrinks and soda. |
| `outgoing.conversationSectional` Conversation sectional | 2x2 · floor | hangOut:relax | living / — | Body, Upholstery | Use | `sink_in`. |
| `outgoing.karaokeStage` Karaoke stage | 2x1 · floor | practise:sing perform:sing | hobbies / — | Body, Upholstery | Use | `mic_up`: the mic stand rises for practise and perform. |
| `outgoing.videoCallNook` Video-call corner | 1x1 · floor | call:phone | office / — | Body, Upholstery | Use | `ring_light`: the ring light swings up for call. |
| `outgoing.danceFloor` Light-up dance floor | 2x2 · floor | dance:dance | hobbies / — | Body, Upholstery | Loop (E1) | `tiles`: the tiles pulse up and down (E1). Until then U: they lift a few millimetres in a pattern. |
| `outgoing.photoWall` Party photo wall | 1x1 · floor | reminisce:look | decor / — | Body, Upholstery | Use | `photo`: a new photo grows on the wall for reminisce. |

## P15. Romantic (6)

- **Family:** `tools/art/blender/families/pack_romantic.py`
- **Validate group:** `romantic` (Validate › Romantic)
- **Placeholders to match:** `tools/art/pack_romantic.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `romantic.candlelitTable` Candlelit bistro table | 2x1 · floor | candlelitDinner:eat shareDessert:eat | kitchen / — | Body, Fabric | Use | `candles`: the candle flames grow for candlelitDinner and shareDessert. |
| `romantic.heartTub` Heart-shaped petal tub | 2x2 · floor | petalBath:bath | bathroom / — | Body, Fabric | Use | `fill`: the water rises with petals for petalBath. |
| `romantic.vanity` Rose-gold vanity | 1x1 · floor | primp:wash | bedroom / — | Body, Fabric | Use | `drawer_open` for primp; mirror part (E6). |
| `romantic.gramophone` Moonlight gramophone | 1x1 · floor | listen:listen waltz:dance | living / — | Body, Fabric | Use | `needle`: the tone arm swings onto the record for listen and waltz. L the platter spins (E1). |
| `romantic.letterBureau` Love-letter bureau | 1x1 · floor | writeLetter:write writePoem:write | office / — | Body, Fabric | Use | `fall_front`: the writing flap drops for writeLetter and writePoem. |
| `romantic.roseArbourSwing` Rose-arbour swing | 2x1 · floor | swing:relax tendRoses:prune | outdoor / — | Body, Fabric | Loop (E1) | `swing` (E1); U `cut`: a bloom shrinks away for tendRoses. |

## P16. Slob (6)

- **Family:** `tools/art/blender/families/pack_slob.py`
- **Validate group:** `slob` (Validate › Slob)
- **Placeholders to match:** `tools/art/pack_slob.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `slob.microwaveCart` Late-night microwave cart | 1x1 · floor | nuke:cook invent:cook | kitchen / — | Body | Use | `door_open`: the microwave door opens and shuts for nuke and invent. |
| `slob.beanbag` Crumb-proof beanbag | 1x1 · floor | flop:relax snooze:nap | living / — | Body | Use | `squash` for flop and snooze. |
| `slob.snackNightstand` Snack-stash nightstand | 1x1 · floor | midnightSnack:eat | bedroom / — | Body | Use | `drawer_open` for midnightSnack. |
| `slob.spritzStation` Spritz & go station | 1x1 · floor | spritz:wash | bathroom / — | Body | Use | `spritz`: the spray bottle tips for spritz. |
| `slob.arcadeCabinet` Pixel-blaster arcade cabinet | 1x1 · floor | chaseHighScore:play quickRound:play | hobbies / — | Body | Loop (E1) | `joystick` (E1). Until then static. |
| `slob.gamingRecliner` Command-couch gaming recliner | 2x1 · floor | marathon:play armrestSoda:drink | living / — | Body | Use | `recline` for marathon; `cup_out`: the cup holder swings out for armrestSoda. |
