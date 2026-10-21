# Personality packs

The traits' own things: content in `web/public/content/packs/<trait>.json`. One Blender family per pack,
`tools/art/blender/families/pack_<trait>.py`, with its notes in `pack_<trait>.md` (`pack_bookworm.py` and `.md` are
the pattern). Each family writes its own override pack, `web/public/assets/packs/pack_<trait>/manifest.json`,
keyed `model.<trait>.<id>`. The example pack's two models keep the base keys `model.chefStove` and
`model.whirlpoolTub`.

Pack things are the residents' favourites and used often, so every one but the bouldering wall has a clip. Each is
built with its back on the wall face (depth −0.43) where it stands against one, and its seat at the height the
renderer puts residents (content `seat` / `stand`). Most take 3 draw calls; the jukebox,
chef's stove, whirlpool tub, pizza oven, record cabinet, lava lamp, sauna, glasshouse, veggie bed and potting bench
take 4. Back home, a pack thing has no group (`<category> / —`), except the veggie bed (garden / edibles). Seats
that face a screen or desk at the back of their own footprint carry content `seat.facing: "back"` (the writer's
bureau, the cinema wall, the game console). The writing desks with their chair built in are written at from that
chair; the chair tuck at separate tables (`seatAt`, `tableTuck.ts`) doesn't apply to them. Open questions are in
[open.md](open.md); the clip kinds and engine rules (E1–E8) are in [README.md](README.md).

## P1. Bookworm (6)

- **Family:** `tools/art/blender/families/pack_bookworm.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bookworm.readingNook` Window-seat reading nook | 2x1 · floor | curlUp:read | living / — | Wood, Fabric, Cushions | Use | `sink_in`: the window-seat cushion settles for curlUp. |
| `bookworm.libraryWall` Rare-books library wall | 2x1 · floor | research:read browse:read | office / — | Wood, Ladder | Use | `ladder`: the rolling ladder slides 40 cm along its rail for research and browse. Research and browse are read on a seat within 4 m (`seatNear`), else standing. |
| `bookworm.writingBureau` Writer's bureau | 1x1 · floor | draft:write letter:write | office / — | Wood, Fabric, Chair | Use | `fall_front`: the writing flap drops 90° towards the writer for draft and letter, and closes after. The writer sits on its chair facing the bureau and the wall (`seat.facing: "back"`). |
| `bookworm.teaTrolley` Book-club tea trolley | 1x1 · floor | brew:drink | kitchen / — | Wood, Fabric, Frame | Use | `pour`: the teapot tips 30° for brew; `hold` 1.5 s. The tea is had on a seat within 3 m (`seatNear`), else standing. |
| `bookworm.firesideLounge` Fireside reading lounge | 2x2 · floor | readByFire:read readAloud:read | living / — | Wood, Fabric, Rug | Use | `flames`: the fire grows in the grate for readByFire and readAloud. |
| `bookworm.bedsideStack` Bedside book stack | 1x1 · floor | chapter:read | bedroom / — | Wood, Fabric, Drawers | Use | `book_up`: the top book lifts and opens for chapter. The chapter is read on a seat within 3 m (`seatNear`), else standing. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the nook's cushion, the library ladder, the bureau's flap, the teapot, the fire for
  both readings, the bedside book.
- The fireside lounge's hood hangs on a flue that rises to the 2.8 m ceiling, with a collar there.
- The bureau is built turned round (`_turn_round`), back on the wall and chair in front.
- The library wall has no fabric to colour; its second colour part is the ladder.

## P2. Cheerful (6)

- **Family:** `tools/art/blender/families/pack_cheerful.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `cheerful.jukebox` Sunburst jukebox | 1x1 · floor | dance:dance listen:listen | living / — | Body, Trim | Use | `record`: a record swings up into place for dance, and `spin` turns its label (a part inside it) while the dance lasts. Pick a song and listen, from a seat within 4 m (`seatNear`); the record clip plays only for dance. |
| `cheerful.karaokeMachine` Spotlight karaoke machine | 1x1 · floor | sing:sing | hobbies / — | Body, Trim, Rug | Use | `mic_up`: the microphone rises on its stand for sing. |
| `cheerful.mocktailCart` Fizz & sunshine mocktail cart | 1x1 · floor | mix:drink | kitchen / — | Body, Trim, Frame | Use | `shake`: the shaker tips for mix. |
| `cheerful.rainbowHammock` Rainbow hammock | 2x1 · floor | swing:relax doze:nap | outdoor / — | Body, Trim, Pillow | Loop (E1) | `sway`: the sling rocks 5° each way while swung in or dozed in, and the resident rides it (`ride`), lain across it (`BEDS` `across`). |
| `cheerful.sunriseLamp` Sunrise stretch lamp | 1x1 · floor | stretch:exercise | bedroom / — | Body, Trim, Mat | Use | `sunrise`: the sun disc rises in its frame for stretch, stretched on the mat (content `stand`). Its sky glows at night (E8). |
| `cheerful.partyBooth` Confetti party booth | 2x2 · floor | party:phone | living / — | Body, Trim, Curtain | Use | `curtain`: the booth curtain is drawn aside along its rail for party (`hold` 1.5 s). |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts; nothing needed fixing.
- The hammock is lain across its 2x1 sling; the hammock's throw is left out.
- The jukebox takes 4 draws (the spinning label is a part inside the record, so it swings up with it).
- The booth's curtain is gathered along its rail, not hinged: swung, it would sweep through the backrest.
- Since the authenticity review (G-04): the karaoke singer steps up to the machine's mic stand (`STEP_IN`), the hands
  free; the party booth's video party is talked to the laptop (`sitTalk`), not a phone at the ear.
- Since the authenticity review (G-43): two dancing to the jukebox dance a pace apart in front of it, turned in a
  little (`places`), not one inside the other.

## P3. Couch potato (6)

- **Family:** `tools/art/blender/families/pack_couchPotato.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `couchPotato.cinemaWall` Grand cinema wall | 2x1 · floor | marathon:watch channelSurf:watch | living / — | Body, Upholstery, Curtains | Use | `curtains`: the side curtains slide apart for marathon and channelSurf. The screen is on the wall at the back, the twin recliners in front facing it (content `seat.facing: "back"`), two sitters at once. Provides `screen`. |
| `couchPotato.gameConsole` Pixel Arcade console | 1x1 · floor | play:play speedrun:play | hobbies / — | Body, Upholstery, Cushion | Use | `tray`: the disc tray slides out and back for play and speedrun (`hold` 1.5 s). The screen and shelf are on the wall at the back, the loveseat in front facing it (content `seat.facing: "back"`); the tray slides towards the players. |
| `couchPotato.recliner` Cloud Nine power recliner | 1x1 · floor | recline:relax doze:nap | living / — | Body, Upholstery, Throw | Use | `recline`: the back tips and the footrest rises for recline and doze; the sitter rides the back (`ride`), leant back 20° with the legs out on the footrest (content `seat`). |
| `couchPotato.snackFridge` Sofa-side snack fridge | 1x1 · floor | snack:fetch takeout:fetch | kitchen / — | Body, Upholstery | Use | `fridge_open`: the door swings for snack and takeout. |
| `couchPotato.fitnessGame` Living-room workout game | 1x1 · floor | workout:exercise | fitness / — | Body, Mat, Board | Use | `board`: the balance board tilts 10° for workout. The player steps up onto the board (content `stand`, 0.115 m). |
| `couchPotato.lavaLamp` Lava glow lamp | 1x1 · floor | zoneOut:relax | decor / — | Body, Upholstery, Glass | Use | `blobs`: the wax blobs rise slowly for zoneOut. `drift` turns two more blobs round the glass all the time, near the camera (E2). |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the curtains for both watches, the tray, the recliner with its sitter, the fridge
  door, the board, the wax over its ambient `drift`.
- The cinema wall and the console are built turned round (`_turn_round`), the screen on the wall face and the seats
  in front (content `seat` `forward` 0.05 for the cinema, 0.12 for the console, so the knees stay under the
  console's shelf). Two sitters face the picture in all four facings.
- Both cinema uses broadcast 35% of their fun to residents in the room (or within 6 m): someone at the snack fridge
  hears the film while they eat.
- The cinema's and the console's screens (the `picture`/`picture_b` materials) and the workout game's TV (`tv_screen`)
  glow and flicker while a resident uses them (`screen` in the model's manifest), day or night; the lava lamp still
  doesn't glow at night (no `glow`), and its fourth draw is the ambient wax.

## P4. Energetic (7)

- **Family:** `tools/art/blender/families/pack_energetic.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `energetic.boulderingWall` Home bouldering wall | 2x1 · floor | climb:exercise | fitness / — | Frame, Pads, Panel | Static | Static (the holds are the shape). |
| `energetic.spinBike` Interactive recumbent bike | 1x2 · floor | ride:exercise | fitness / — | Frame, Pads, Flywheel | Loop (E1) | `pedals`: the cranks and pedals turn once a second while riding (E1); the rider sits on the still seat. Renamed from "Interactive spin bike": a recumbent (kind `recumbentBike`, Q-08). |
| `energetic.trampoline` Rebound trampoline | 2x2 · floor | bounce:exercise | outdoor / — | Frame, Pads, Mat | Loop (E1) | `bounce`: the mat dips under the bouncer's feet while bounced on (E1), 1.6 times a second (content `stand.height` 0.5). |
| `energetic.streetballHoop` Streetball hoop | 1x1 · floor | shoot:exercise | outdoor / — | Frame, Pads, Base | Use | `swish`: the net sways after each shot (1.4 s into the use, then every 3.2 s: `delay`, `every`). |
| `energetic.powerBlender` Power blender station | 1x1 · floor | blend:cook | kitchen / — | Frame, Doors, Blender | Use, repeating | `lid`: the jug lid lifts and settles for blend (`hold` 1 s); `blades`: then the smoothie swirls (its fruit streaks and bits turn), once a second, from 1.5 s in till the blend ends (`every` 0.9 s; 2026-10-21, the blades themselves hidden in it). 4 draws. |
| `energetic.contrastShower` Contrast rain shower | 1x1 · floor | rinse:shower | bathroom / — | Frame, Tiles, Mat | Use | `water_on`: the rain streams grow for rinse. |
| `energetic.recoveryPod` Recovery pod | 1x2 · floor | recover:relax | fitness / — | Frame, Pads, Boots | Use | `canopy`: the canopy lowers over them for recover and lifts after. |

**In play** (signed off 2026-10-17):
- The wall is climbed on (G-26: `boulder*`, a size per height, `STEP_IN` up onto the crash mat): hands to the start
  holds, the feet onto the low ones, up the model's holds hand and foot in turn to the top, a drop onto the mat.
- Since 2026-10-21 (review A-packs-02) the blender's smoothie swirls once the lid is back on, till the blend ends
  (`fixes/motions/g1718/views/blender-t0.3.png`).
- The wall is still (looked at in all facings, climbed); the pedals turn, the mat dips with the bouncer, the net
  sways after each shot, the blender lid lifts and settles, the rain grows, the canopy lowers and lifts. Nothing
  needed fixing.
- The recovery pod stands open 40° at rest (clear of the wall behind) and closes for recover.
- The contrast shower fills its tile with its own tiled walls, as the bathroom showers do.
- Since the authenticity review (G-43, G-51): the streetball hoop's two shoot from their own spots a pace apart
  (`places`); the recumbent bike is got onto from its side (`approach: side`), not over its front.

## P5. Example pack (2)

- **Family:** `tools/art/blender/families/pack_example.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `chefStove` Chef's stove | 2x1 · floor | cook:cook practise:cook | kitchen / — | Body, Handles & knobs, Tiles | Use | `oven_open` (the door drops, shut again after 3 s) for practise and any `bake` step; `burner_on` (knob and flame ring, the whole use) for cook and any `stir` step. |
| `whirlpoolTub` Whirlpool tub | 2x2 · floor | soak:bath | wellness / — | Tub, Surround, Headrests | Use | `fill`: the water rises for soak. `bubbles` swell up through the water while soaking (a loop, E1). |

**In play** (signed off 2026-10-17):
- The stove opens its oven and lights its burner for cook and practise, each run as a recipe from the house's
  fridge (`meal`, `bakedTreats`); the tub fills and bubbles for soak, its two bathers on the bench. Nothing needed
  fixing.
- Since 2026-10-21 (G-16) practise (baking) opens only the oven and cook only lights the hob; the resident bends to
  the oven with no pan drawn (`bench-results/review/fixes/motions/recipes/practise-chefStove-*`).
- Both take 4 draws on purpose (the stove's door, knob and flame; the tub's water and bubbles).
- Wellness is the pack's own category.

## P6. Foodie (6)

- **Family:** `tools/art/blender/families/pack_foodie.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `foodie.herbGarden` Windowsill herb garden | 1x1 · floor | tendHerbs:tend | decor / — | Body, Pots | Use | `sprig`: a fresh sprig grows in a pot for tendHerbs. |
| `foodie.deliPantry` Deli pantry | 1x1 · floor | nibble:fetch cheeseBoard:cook | kitchen / — | Body, Worktop, Shelves | Use | `doors_open`: the doors swing for nibble and cheeseBoard (`hold` 2.5 s). |
| `foodie.espressoBar` Espresso bar | 1x1 · floor | espresso:drink latteArt:drink | kitchen / — | Body, Worktop, Machine | Use | Provides `coffee`; `brew`: the coffee stream grows and the steam wand swings out for espresso and latteArt (`hold` 4 s). |
| `foodie.cookbookLectern` Cookbook lectern | 1x1 · floor | studyRecipes:read foodMagazines:read | kitchen / — | Body, Stool & ledge, Cookbook | Use | `page`: a page turns over for studyRecipes and foodMagazines. |
| `foodie.pizzaOven` Wood-fired pizza oven | 2x1 · floor | bakePizza:cook warmUp:relax | outdoor / — | Body, Worktop, Brick | Use | `door_off`: the oven door lifts aside and the peel slides in for bakePizza (and any `bake` step), with the `flames`; the `flames` alone grow for warmUp. |
| `foodie.chefsIsland` Chef's island | 2x1 · floor | tastingMenu:cook barBite:eat | kitchen / — | Body, Worktop, Stools | Use | `drawer_open` for tastingMenu (`hold` 2.5 s). Sat at on its stools for barBite. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts; nothing needed fixing.
- The pizza oven bakes as a recipe (`pizza`); its fire shows through the grille door, so the door comes off a
  burning oven. It has three moving parts and takes 4 draws.
- The chef's island's top is at 0.9 m (content `surface.height`), its own height; the other kitchen worktops are
  0.95 m. Its stools' legroom runs right through under the drawer bank.
- The herb garden has no worktop; its second colour part is the pots.
- Since the authenticity review (G-14): the herb trough is tended standing at it, leaning in (`pick`, `STAND_BACK`
  -0.5), not kneeling.
- Since the authenticity review (G-43): two at the pizza oven or the chef's island stand one in front of each half,
  facing it.

## P7. Gloomy (6)

- **Family:** `tools/art/blender/families/pack_gloomy.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `gloomy.recordCabinet` Midnight record cabinet | 1x1 · floor | listen:listen newAlbum:listen | living / — | Body, Trim | Use | `lid_open`: the lid lifts for listen and newAlbum. `platter_spin` turns the platter while it plays (E1). Listened to from a seat within 4 m (`seatNear`), else standing. |
| `gloomy.cello` Nocturne cello | 1x1 · floor | practise:music lament:music | hobbies / — | Body, Trim, Cello | Use | `off_stand`: the cello comes off its stand to between the player's knees for practise and lament. |
| `gloomy.journalDesk` Candlelit writing desk | 2x1 · floor | journal:write poetry:write | office / — | Body, Trim | Use | `candle`: the candle flame grows for journal and poetry; it glows at night. Written at on its own chair with the hands on the top (`NEAR_DESKS`, `sitHandsNear`; `SEATS`), not through its front. |
| `gloomy.rainLamp` Rainfall lamp | 1x1 · floor | watchRain:look | decor / — | Body, Trim | Use | `rain`: the rain strands grow for watchRain. `ripple` swells the pool's ripples all the time, near the camera (E2). |
| `gloomy.windowSeat` Brooding window seat | 2x1 · floor | brood:look doze:nap | living / — | Body, Cushion, Trim | Use | `sink_in` for brood and doze. The dozer lies curled up on the side along the cushion, facing the room, the head on a pillow (`BEDS` 0.45, `across`; `lieCurl`, `CURLED_BEDS`: 1.3 m between the pillows is short to lie flat, since the authenticity review, G-04); the doze takes both places (`whole`): it starts only on an empty seat, and nobody sits beside the dozer. |
| `gloomy.soakingTub` Moonstone soaking tub | 2x2 · floor | soak:bath | bathroom / — | Body, Trim | Use | `fill`: the water rises for soak. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: lid and platter, the cello off its stand, the candle, the rain over its ambient
  ripple, the seat's cushion for brood and doze, the tub filling.
- The record cabinet is walnut outside, ink (Trim) only inside the bays, so it reads right from behind when it
  stands free. It takes 4 draws.
- A nap on the window seat takes the whole seat (content `whole`; sim-core `can_use_for` / `reserve_for`): it starts
  only with nobody else on it, and while it lasts the seat is full. An order for it while someone sits there is
  refused ("someone is using the brooding window seat"); an order to sit beside the dozer is dropped when it comes
  up. Two sitters still fit for brood.
- At the journal desk the writer's hands rest on the top (`sitHandsNear`), not under its apron; the apron runs only
  along the far edge and the ends.
- Since the authenticity review (G-06): the soaking tub's bather sits on its floor at the back, the knees drawn up
  inside (`Seat.room`).

## P8. Hot-headed (6)

- **Family:** `tools/art/blender/families/pack_hotHeaded.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `hotHeaded.heavyBag` Heavy punching bag | 1x1 · floor | pound:exercise | fitness / — | Body | Loop (E1) | `swing`: the bag rocks 8° back and forth while pounded (E1). |
| `hotHeaded.smashBin` Plate-smash bin | 1x1 · floor | smash:exercise | hobbies / — | Body | Use | `lid_open`: the lid flips up for smash; `hold` 1 s. |
| `hotHeaded.thunderDrums` Thunder drum kit | 2x1 · floor | drum:music | hobbies / — | Body | Loop (E1) | `cymbals`: the crash and the ride rock on their stands while drumming (E1). |
| `hotHeaded.coolDownCushion` Cool-down meditation cushion | 1x1 · floor | countToTen:meditate breathe:meditate | decor / — | Body | Use | `sink_in`: the zafu presses down to the 0.40 m seat for countToTen and breathe. |
| `hotHeaded.grudgeJournal` Grudge-to-gratitude writing desk | 1x1 · floor | writeItOut:write | office / — | Body | Use | `drawer_open` for writeItOut. Written at on its own chair with the hands on the top (`NEAR_DESKS`, `sitHandsNear`; `SEATS`), not through its front. |
| `hotHeaded.coldPlunge` Glacier cold-plunge tub | 1x2 · floor | plunge:bath | bathroom / — | Body | Use | `cover_off`: the slatted cover rolls back into its housing on the back rim for plunge. Teens and up (`stages`, G-45). |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the bag swings while pounded, the bin's lid holds 1 s, the cymbals shake, the zafu
  sinks for both meditations, the drawer opens, the cover rolls off. Nothing needed fixing.
- At the grudge desk the hands rest on the top (`sitHandsNear`).
- Smashing plates (G-26: `smashPlate*`, a size per height, `STEP_IN` up to the stool): a plate taken from the stack
  (mimed) and thrown overarm down into the crate.
- Each model has one colour part, Body: the bag, the crate, the shells, the zafu, the desk's wood, the cedar
  cladding.
- Since the authenticity review (G-05, G-06): the plunger sits on the tub's floor at the back, the knees up and the
  feet inside its front wall (`Seat.room`); the drum stool is sat on at its pad (0.475), the cool-down cushion at its
  top (0.45).

## P9. Kind (6)

- **Family:** `tools/art/blender/families/pack_kind.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `kind.neighbourlyOven` Neighbourly baking counter | 2x1 · floor | familyPie:cook treatsToShare:cook | kitchen / — | Body, Fabric, Cupboards | Use | `oven_open` for the `bake` step of familyPie and treatsToShare: the door drops as the dish goes in and shuts after 3 s; the pastry and batter are made on its worktop with the oven shut. |
| `kind.givingJar` Community giving jar | 1x1 · floor | donate:idle | decor / — | Body, Fabric | Use | `coin`: a coin slides into the slot for donate. |
| `kind.knittingRocker` Gift-knitting rocker | 1x1 · floor | knitGift:tinker rock:relax | hobbies / — | Body, Fabric | Loop (E1) | `rock`: the chair rocks ±5° on its rockers with the knitter riding it (`ride`) for knitGift and rock (E1). |
| `kind.welcomeTeaTable` Welcome tea table | 2x1 · floor | shareTea:drink | living / — | Body, Fabric, Second pouf, Rug | Use | `pour`: the teapot tips for shareTea (`hold` 1.5 s). The two on the poufs turn 45° to each other (content `places`; facing across waits for G-39). |
| `kind.songbirdFeeder` Songbird feeding station | 1x1 · floor | refill:garden watchBirds:look | outdoor / — | Body, Fabric, Roof | Use | `lid_open` for refill; `bird`: birds appear for watchBirds. |
| `kind.hearthSettee` Hearthside quilted settee | 2x1 · floor | curlUp:relax quiltNap:nap | living / — | Body, Fabric | Use | `sink_in` for curlUp and quiltNap. The napper lies along the seat between the arms, head at the lantern's end (`BEDS` 0.44, `across`); the nap takes both places (`whole`): it starts only on an empty settee, and nobody sits beside the napper. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the oven for pie and treats (recipes `pie` and `bakedTreats` from the house's fridge),
  the coin, the rocker with its sitter, the teapot, the feeder's roof and birds, the settee's cushions.
- Since 2026-10-21 (G-16, G-17) the oven door drops only for the bake, keyed by the `bake` step's tag, and shuts after
  3 s (it stood open through the 30 and more minutes); the pastry is made in a mixing bowl on its worktop
  (`bench-results/review/fixes/motions/recipes/pie-neighbourlyOven-*`).
- A nap on the hearth settee takes the whole settee, as on the gloomy window seat (content `whole`): refused with
  "someone is using the hearthside quilted settee" while someone sits there; two can still sit for curlUp.
- The coin stays in the jar while the resident is at it and comes back up into the slot when the use ends.
- The tea table's two poufs colour apart (Fabric, Second pouf); the settee's cushions take its Body colour.
- Since the authenticity review (G-05, G-15): the hearth settee's napper lies on its cushion (0.36); the feeder is
  filled stepped in to its post (`STEP_IN`, clear of the bird bath, for the refill only): the left hand steadies the
  house and the scoop (mimed) is tipped twice over its open top (`fillFeeder177`/`147`, clips/reach.ts); a child
  (`fillFeeder112`) tips it onto the seed tray, as high as they reach. A coin goes into the giving jar from beside it
  (`interact`).

## P10. Lazy (6)

- **Family:** `tools/art/blender/families/pack_lazy.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `lazy.snoozecloudBed` Snoozecloud adjustable bed | 2x2 · floor | sleep:sleep sleepIn:nap | bedroom / — | Body, Upholstery, Throw | Use | `head_up`: the head section rises 30° for sleepIn, `hold` 2 s, and settles flat for the nap. A blanket bed (`BLANKET_BEDS`), mattress at the default `BEDS` height. |
| `lazy.napPod` Daydream nap pod | 1x1 · floor | curlUp:relax powerNap:nap | living / — | Body, Upholstery, Hood | Use | `hood`: the hood swings half over for powerNap and curlUp. Both are sat in the pod (powerNap a sitting nap). |
| `lazy.quiltTable` Heated quilt table | 2x1 · floor | tuckIn:relax doze:nap | living / — | Body, Upholstery, Poufs | Use | `quilt`: the quilt edge lifts for tuckIn and doze. Both sit on a pouf at the table, legs under the quilt (doze a sitting nap). |
| `lazy.deliveryLocker` Doorstep delivery locker | 1x1 · floor | takeout:fetch noodles:fetch | kitchen / — | Body, Doors, Trim | Use | `door_open`: a locker door opens for takeout and noodles; `hold` 2 s. |
| `lazy.vibePlate` Effortless vibration plate | 1x1 · floor | shake:exercise | fitness / — | Body, Trim, Deck | Loop (E1) | `shake`: the deck rocks ±3° four times a second under the resident (E1; content `stand.height` 0.12). |
| `lazy.smartHub` Smart-home command hub | 1x1 · floor | program:type story:listen | office / — | Body, Speaker | Use | `screen_up`: the tablet tilts up for program and story. Program and story from a seat within 4 m (`seatNear`), else standing. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts.
- The quilt table's doze and the nap pod's powerNap are sitting naps (`"pose": "sit"`, anim `nap`, as the sofas' and
  recliners' dozes): at the table she sits on the left pouf, arms on the top and legs under the quilt; in the pod
  she sits in the egg chair under the lowered hood, feet on the floor. Checked in all four facings, by day, at night
  and on Low WebGL2.
- Known, accepted: at the quilt table the shoes poke out at the foot of the quilt's front, for tuckIn and doze.
- The bed's head section is held up only 2 s: up for the whole nap, it would cover the sleepers' heads.
- Since the authenticity review (G-51): the Snoozecloud bed is got into from its sides, each sleeper by their own
  half (`approach: side`), as the base beds.

## P11. Loner (6)

- **Family:** `tools/art/blender/families/pack_loner.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `loner.quietPod` Hush-shell reading pod | 1x1 · floor | retreat:relax read:read | living / — | Body, Wood, Cushions | Use | `shell`: the hood swings half closed for retreat and read. |
| `loner.shortwaveRadio` Shortwave radio desk | 1x1 · floor | airwaves:phone tune:tinker | office / — | Body, Wood, Seat | Use | `dial`: the tuning dial turns for tune; for airwaves the antenna extends, then the dial turns. |
| `loner.puzzleTable` Jigsaw puzzle table | 1x1 · floor | piece:play newPuzzle:play | hobbies / — | Body, Wood | Use | `pieces`: a patch of puzzle grows for piece and newPuzzle. |
| `loner.bonsaiBench` Bonsai potting bench | 1x1 · floor | shape:prune tend:water | hobbies / — | Body, Wood, Pot | Use | `trim`: a twig shrinks away for shape; `mist` nods the twig while tended (a loop). |
| `loner.soloSauna` Cedar sauna for one | 1x2 · floor | sweat:relax | bathroom / — | Wood, Trim | Use | `door_open`: the door opens and shuts behind them (`hold` 1.5 s) for sweat. Teens and up (`stages`, G-45). |
| `loner.deliveryHatch` No-knock delivery hatch | 1x1 · floor | order:fetch treat:fetch | kitchen / — | Body, Wood | Use | `hatch`: the hatch opens for order and treat; `hold` 2 s. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the hood for retreat and read, the dial and antenna, the puzzle for both uses, the
  twig and the mist, the sauna door behind the sitter, the hatch. Nothing needed fixing.
- The sauna's door is glass, so the resident is seen sweating inside; it takes 4 draws. It is all cedar: its colour
  parts are the cedar (Wood) and the dark trim.
- The radio's antenna is down at rest and reaches 1.77 m for airwaves.

## P12. Nature lover (7)

- **Family:** `tools/art/blender/families/pack_natureLover.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `natureLover.veggieBed` Raised veggie bed | 2x1 · floor | tendVeggies:tend pickSnack:eat | garden / edibles | Wood, Posts & rails | Use | `harvest`: a lettuce presses down into the soil for pickSnack. `grow` follows the crop's stage (E4); `rustle` sways the plants every 2.4 s while tended. |
| `natureLover.pottingBench` Potting bench | 2x1 · floor | potSeedlings:tinker tieBouquet:tinker | hobbies / — | Wood, Frame | Use | `seedling`: a seedling grows in a pot for potSeedlings; `bouquet`: a tied bouquet grows on the worktop for tieBouquet. |
| `natureLover.birdFeeder` Bird feeder post | 1x1 · floor | birdwatch:look | outdoor / — | Wood, Roof | Use | `bird`: a bird appears for birdwatch. |
| `natureLover.gardenHammock` Garden hammock | 2x1 · floor | swing:relax doze:nap | outdoor / — | Wood, Fabric, Stripes | Loop (E1) | `sway`: the sling rocks 8° each way while swung in or dozed in, and the resident rides it (`ride`), lain across it (`BEDS` `across`). |
| `natureLover.cedarShower` Cedar outdoor shower | 1x1 · floor | rinse:shower | outdoor / — | Wood, Copper | Use | `water_on`: the stream grows for rinse. |
| `natureLover.mossTerrarium` Moss terrarium | 1x1 · floor | mist:water | decor / — | Wood | Use | `lid_open`: the glass lid lifts for mist. |
| `natureLover.glasshouse` Fern glasshouse | 2x2 · floor | unwind:relax orchids:water | outdoor / — | Wood, Frame, Cushions | Use | `door_open`: the door opens to let the resident in, shuts behind them (`hold` 1.5 s) and opens again to let them out (`out` 1.5 s); `vent_open`: the roof vent lifts while they're in, for unwind and orchids (2026-10-21: the door stood open the whole use). `door_open`: the door and a roof vent open for unwind and orchids; the orchids are tended from inside (`inside`). |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the bed's plants sway while tended (over the growth pose), the seedling and the
  bouquet, the bird, the hammock with its sleeper, the stream, the terrarium lid, the glasshouse door and vent.
- Since 2026-10-21 (review A-packs-03) the glasshouse door shuts behind the resident and opens to let them out; the
  vent stays open while they're in (`fixes/motions/g1718/play-garden.jsonl`).
  Nothing needed fixing.
- pickSnack needs a ripe crop (`when: ripe`, `leaves: picked`; the bed takes two game days to ripen, content
  `grows`). Its `harvest` was seen playing and shutting when the ripe crop was picked, but not caught in a picture.
- Picked, the bed loses its fruit and keeps its plants.
- Since the authenticity review (G-68): tending brings the crop on (`tends` 0.15); "Pick something fresh" stays a
  nibble eaten at the bed (the shortcut Q-18 keeps), unlike the other beds' harvests, which go to the fridge.
- The veggie bed, the potting bench and the glasshouse take 4 draws.

## P13. Neat (6)

- **Family:** `tools/art/blender/families/pack_neat.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `neat.steamMopDock` Steam mop dock | 1x1 · floor | mop:clean | kitchen / — | Body, Mop | Use | `mop_off`: the mop leaves its dock for mop and is drawn in the resident's hands, pushed over the floor (content `held`). |
| `neat.laundryCenter` Stacked laundry centre | 2x1 · floor | laundry:clean fold:clean | bathroom / — | Body, Washer & dryer | Use | `door_open`: the washer door swings for laundry and fold and shuts after 2.5 s (2026-10-21: it stood open through the wash and the folding). Folding is 20 minutes (was 60). laundry only with dirty washing (`workOn`). |
| `neat.rainShowerTower` Rainfall steam shower | 1x1 · floor | steamShower:shower | bathroom / — | Body, Bench | Use | `water_on`: the rain grows for steamShower. |
| `neat.organizerDesk` Label-and-sort organizer desk | 2x1 · floor | sort:write ledger:write | office / — | Body, Wood, Chair | Use | `drawer_open`: a bin slides out of the hutch for sort and ledger. Written at on its own chair with the hands on the top (`NEAR_DESKS`, `sitHandsNear`; `SEATS`), not through its front. |
| `neat.robotVacuumDock` Robot vacuum and dock | 1x1 · floor | program:type | living / — | Body, Trim, Rug | Loop | `vacuum_round`: once programmed the robot drives out of its dock, once round a circle in front of it facing the way it goes, and back in, round after round (5 s a round) while program lasts (2026-10-21: it peeked 60 cm out and back). Program from a seat within 4 m (`seatNear`), else standing. |
| `neat.entryConsole` Tidy entry console | 1x1 · floor | tidy:clean | decor / — | Body, Wood | Use | `tray`: the key tray drawer slides out for tidy. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the mop into the resident's hands, the washer door, the rain, the bin, the robot's
  round trip, the key tray.
- Since 2026-10-21 (reviews A-packs-03, I-packs-15) the washer door shuts for the wash and the folding, and the robot
  drives round in front of its dock while programmed (`fixes/motions/g1718/views/robot-*.png`, `play-stage.jsonl`).
- At the organizer desk the hands rest on the top (`sitHandsNear`), no longer through the modesty panel.
- Seen, not changed: the entry console's leaning mirror and hook board stand on the wall face, so they hang in the
  air when the console stands free. The console's mirror is `chrome`, not mirror glass.

## P14. Outgoing (6)

- **Family:** `tools/art/blender/families/pack_outgoing.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `outgoing.partyBar` Home party bar | 2x1 · floor | mixDrinks:drink soda:drink barDrink:drink | kitchen / — | Body, Upholstery, Back bar | Use | `tap`: the beer tap lever pulls for mixDrinks and soda (`hold` 2 s). Have a drink at the bar, sat on its stools facing the counter (content `seat` 0.74 m, forward 0.5, `facing: back`). |
| `outgoing.conversationSectional` Conversation sectional | 2x2 · floor | hangOut:relax | living / — | Body, Upholstery | Use | `sink_in`: the seat cushions press down for hangOut. |
| `outgoing.karaokeStage` Karaoke stage | 2x1 · floor | practise:sing perform:sing | hobbies / — | Body, Speakers | Use | `mic_up`: the mic stands rise for practise and perform. Sung up on the stage, between the mics (content `stand`, 0.146 m). |
| `outgoing.videoCallNook` Video-call corner | 1x1 · floor | call:phone | office / — | Body, Upholstery, Wood | Use | `ring_light`: the ring light swings up for call. |
| `outgoing.danceFloor` Light-up dance floor | 2x2 · floor | dance:dance | hobbies / — | Body, Speakers | Loop (E1) | `tiles`: the tiles pulse up and down in two checkerboard sets, taking turns, while dancing (E1). |
| `outgoing.photoWall` Party photo wall | 1x1 · floor | reminisce:look | decor / — | Body, Board | Use | `photo`: a new photo grows on the wall for reminisce. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the tap, the cushions, the mic stands, the ring light, the tiles while dancing, the
  photo.
- Seen, not changed: the party bar's "Back bar" colours the low back cabinet, not the bottle shelves above it. Its
  flutes and tap handle are on the Body material, so they take the Body colour.
- Nothing glows at night (the dance floor's tiles and the photo wall's string lights could take a `glow`).
- Since the authenticity review (G-04): the video call is talked to the screen (`sitTalk`), the hands free; on the
  karaoke stage each singer steps up onto it to a mic stand (`STEP_IN`, the lyrics screen ahead).
- Since the authenticity review (G-43): two at the party bar, the karaoke stage or the dance floor stand one in front
  of each half (the stage's two then step up to a mic stand each), not one inside the other.

## P15. Romantic (6)

- **Family:** `tools/art/blender/families/pack_romantic.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `romantic.candlelitTable` Candlelit bistro table | 2x1 · floor | candlelitDinner:eat shareDessert:eat | kitchen / — | Body, Fabric, Tablecloth | Use | `candles`: the candle flames grow for candlelitDinner and shareDessert. Its places are its two chairs as built, side by side (G-39 rebuilds it to face across). |
| `romantic.heartTub` Heart-shaped petal tub | 2x2 · floor | petalBath:bath | bathroom / — | Body, Fabric | Use | `fill`: the water rises with petals for petalBath. |
| `romantic.vanity` Rose-gold vanity | 1x1 · floor | primp:wash (sit) | bedroom / — | Body, Fabric, Rose gold | Use | `drawer_open` for primp (`hold` 2 s), sat on the pouf facing the mirror (content `seat.facing: "back"`); the mirror is mirror glass (E6). |
| `romantic.gramophone` Moonlight gramophone | 1x1 · floor | listen:listen waltz:dance | living / — | Body, Horn | Use | `needle`: the tone arm swings onto the record for listen and waltz. `platter` spins the record while it plays (E1). Listen is from a seat within 4 m (`seatNear`), else standing; waltz stands. |
| `romantic.letterBureau` Love-letter bureau | 1x1 · floor | writeLetter:write writePoem:write | office / — | Body, Fabric | Use | `fall_front`: the writing flap drops for writeLetter and writePoem. Written at on its own chair with the hands on the top (`NEAR_DESKS`, `sitHandsNear`; `SEATS`), not through its front. |
| `romantic.roseArbourSwing` Rose-arbour swing | 2x1 · floor | swing:relax tendRoses:prune | outdoor / — | Body, Fabric | Loop (E1) | `swing`: the swing rocks with its riders (`ride`) while swung in (E1); U `cut`: a bloom shrinks away for tendRoses. |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the flames for dinner and dessert, the tub with petals, the bureau's flap, the vanity
  drawer, the arm and platter for listen and waltz, the swing with its sitter and the bloom. Nothing needed fixing.
- At the letter bureau the hands rest on the top (`sitHandsNear`).
- Known: seated bathers in the heart tub put their shins through the front of the shell where the heart narrows
  (the 0.35 m seat pose).
- Since the authenticity review (G-06): the two bathers sit on the heart tub's floor at the back, the knees drawn up,
  the feet inside its front wall (`Seat.room`).

## P16. Slob (6)

- **Family:** `tools/art/blender/families/pack_slob.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `slob.microwaveCart` Late-night microwave cart | 1x1 · floor | nuke:cook invent:cook | kitchen / — | Body | Use | `door_open`: the microwave door opens and shuts for nuke and invent (`hold` 1.5 s). |
| `slob.beanbag` Crumb-proof beanbag | 1x1 · floor | flop:relax snooze:nap | living / — | Body | Use | `squash` for flop and snooze. |
| `slob.snackNightstand` Snack-stash nightstand | 1x1 · floor | midnightSnack:eat | bedroom / — | Body, Top | Use | `drawer_open` for midnightSnack, eaten standing beside it (not sitting on its top, 2026-10-21). |
| `slob.spritzStation` Spritz & go station | 1x1 · floor | spritz:wash | bathroom / — | Body | Use | `spritz`: the spray bottle tips for spritz. |
| `slob.arcadeCabinet` Pixel-blaster arcade cabinet | 1x1 · floor | chaseHighScore:play quickRound:play | hobbies / — | Body, Sides | Loop (E1) | `joystick`: the joystick waggles side to side while playing (E1). |
| `slob.gamingRecliner` Command-couch gaming recliner | 2x1 · floor | marathon:play armrestSoda:drink | living / — | Body, Stripes | Use | `recline` for marathon, the sitter riding the back (`ride`), leant back 22° with the legs out on the footrest (content `seat`); `cup_out`: the cup holder swings out for armrestSoda. Marathon needs a working TV on the lot (`requires: screen`; a screen in front of it needs a place condition, G-64). |

**In play** (signed off 2026-10-17):
- Every clip plays and shuts: the microwave door, the beanbag for flop and snooze, the drawer, the bottle, the
  joystick, the recliner and its cup holder. Nothing needed fixing.
- Seen, not changed: the spritz station's round mirror stands on the wall face, so it hangs in the air when the
  station stands free. It is `chrome`, not mirror glass.
