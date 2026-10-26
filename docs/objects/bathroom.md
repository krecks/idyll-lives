# Bathroom

Sinks, toilets, showers, baths, laundry and bathroom storage. `families/bathroom.py` builds the five bathroom
sinks, the B1 toilets, the B2 showers, the B3 baths and the B5 storage and mirrors (notes: `bathroom.md`);
`families/laundry.py` builds the B4 laundry (`laundry.md`). The two kitchen sinks are on `kitchen.md`.

Water here grows the same way: a stream built full length, scaled to 0.001 so it hides at rest, and `G.grow`n for
the whole use (`water_on`, `tap_on`; the bath's water rises with `fill`). Mirror glass is in the file (E6), a flat
piece of the `mirror` material; the nearest mirror on screen reflects live (two on High). Floor things stand with
their backs on the wall face (game depth -0.43).

## Sinks (5)

- **Family:** `tools/art/blender/families/bathroom.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sink` Sink | 1x1 · floor | wash:wash brushTeeth:wash | bathroom / sinks | Cabinet, Worktop, Basin, Tap & handles, Plinth | Use | `water_on`: the stream grows from the gooseneck into the inset basin in 0.4 s, runs for the whole use and retracts as it ends. Brush your teeth (3 min, the tap running and its sound: `handwash` tag after Wash hands, so a toilet's follow-up still washes hands). |
| `sinkPedestal` Pedestal sink | 1x1 · floor | wash:wash brushTeeth:wash | bathroom / sinks | Pedestal, Basin & cup, Tap | Use | `water_on`, as the sink. Brush your teeth, the tap running. |
| `vanityDouble` Double vanity | 2x1 · floor | wash:wash brushTeeth:wash | bathroom / sinks | Cabinet, Worktop, Basins, Taps & handles, Plinth | Use | `water_on`: both taps run together (two slots). No mirrors of its own: the player hangs one above it. Brush your teeth at either basin, the taps running. The bowls stand on a 0.76 m worktop, their rims at 0.88 m (they were at 1.00 m, M-bathroom-02). |
| `sinkVessel` Vessel sink | 1x1 · floor | wash:wash brushTeeth:wash | bathroom / sinks | Vanity, Worktop, Bowl, Tap, Shelf & legs, Towels | Use | `water_on` into the raised bowl. Brush your teeth, the tap running. The bowl stands on a 0.76 m worktop, its rim at 0.88 m (it was at 1.02 m, M-bathroom-02). |
| `sinkConsole` Console sink | 1x1 · floor | wash:wash brushTeeth:wash | bathroom / sinks | Basin, Legs & tap, Towel | Use | `water_on`, as the sink. Brush your teeth, the tap running. |

**In play** (signed off 2026-10-16):
- The first group checked: bought in all four facings, every part coloured (through `buy` and in the colour studio),
  washed at by day and at night, on WebGPU and WebGL2, on Medium and Low.
- `water_on` has no `hold`: it plays for the `wash` tag and stops when the use ends.
- Each basin has painted shoulders from its floor up to the worktop, so the cabinet's colour never shows as the basin.
- Since the authenticity review (2026-10-21, G-15): the washer steps in to the basin (`STAND_BACK`), the toes at it
  and the hands over the water, not a step back.
- Since the authenticity review (G-43): two at the double vanity stand one at each basin, facing it (`stand_spot`),
  not one inside the other.

## B1. Toilets (5)

- **Family:** `tools/art/blender/families/bathroom.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `toilet` Toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat & lid, Cistern | Use | `lid_up`: the lid swings up 100° as the use starts and down as it ends. |
| `toiletCompact` Compact toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat & lid, Cistern | Use | `lid_up`. |
| `toiletSmart` Smart toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat & lid, Cistern | Use | `lid_up`: lid and seat rise on their own (slower, 0.8 s). |
| `toiletClassic` Classic toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat & lid, Cistern | Use | `lid_up`; the close-coupled cistern (the content: no high cistern and chain) and wooden seat are static. |
| `toiletWallHung` Wall-hung toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat & lid, Cistern | Use | `lid_up`. |

**In play** (signed off 2026-10-16):
- Each checked against three walls and standing free, with the bladder let down to 0.65 first so the use lasts (a use
  stops once its need is full).
- `lid_up` plays in full (0.58 s; the smart toilet's 0.79 s) as the resident sits and shuts as they get up; the
  resident sits on the seat, leaning forward (the shared `toilet` pose).
- On the wall-hung and smart toilets "Cistern" colours the boxing panel. The colour studio's "In use" lifts the lid.
- Seat tops are at the content's seat height (0.43–0.45 m); the wall-hung one follows the content, not the old
  0.6 m cistern.
- Nothing needed fixing.

## B2. Showers (4)

- **Family:** `tools/art/blender/families/bathroom.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `shower` Shower | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Frame, Head & tap, Tiles | Use | `water_on`: a rain of thin streams grows down from the head for the whole use, 2 s in. The sliding glass door (`door_open`, its own part: the 4th draw call, Q-05) slides open behind the fixed panel as the use starts, shuts after 1.5 s and opens again for 1.5 s as the use ends (`out`); the bather waits outside until it has slid open, walks in through the opening, and walks out the same way once it opens again, still in the towel (`WALK_WAIT`, `toweled`; A-bathroom-05, A-bathroom-06). The resident is already inside as it opens (A-bathroom-05). |
| `showerStall` Shower stall | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Rail, Head & tap, Curtain, Tiles | Use | `water_on`, slanted from the wall head onto the resident, 1.8 s in. The curtain's front (`curtain_draw`, its own part, plain satin: still 3 draws) is drawn across from the right corner 1 s in and back as the use ends; the side's curtain hangs closed. |
| `showerWalkIn` Walk-in rain shower | 1x1 · floor | shower:shower | bathroom / bathing | Floor, Frame, Head & tap, Wall | Use | `water_on` from a wide rain head. |
| `showerRain` Rain shower | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Frame, Head & tap, Walls | Use | `water_on` from the ceiling rose. |

**In play** (signed off 2026-10-17):
- Each showered in, in all four facings, by day and at night, on WebGPU and WebGL2, on Medium and Low, every part
  coloured. The resident steps in onto the middle of the tile (`inside`) and washes: lathers and rinses the hair,
  turns 50° to soap the arms and the chest, turns back and rinses the face (`shower`; a child turns only the chest,
  `showerChild`; G-21). The pack showers play the same.
- `water_on` (0.5 s) grows the rain as the use starts, holds it for the whole use and shuts it as it ends.
- The colour studio frames a moving part by its posed vertices, so the stall (whose rain grows from nothing and is
  turned for the slant) draws at rest and in use.
- Three draw calls each: the tray, walls and metal; the glass (the stall's curtain); the rain. Since 2026-10-21 the
  base shower's sliding door is a fourth (Q-05) and the stall's curtain front its own part, plain satin, so the
  stall keeps three (body, curtain, rain).
- Under the water the resident wears a towel, barefoot (`wearFor`, `outfit.ts` `towel`: wrapped under the arms, a
  man's round the hips), not the day clothes; they dress again as they step out. The packs' showers do the same.

## B3. Baths (3)

- **Family:** `tools/art/blender/families/bathroom.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bathtub` Bathtub | 2x1 · floor | bath:bath | bathroom / bathing | Tub, Panel, Taps, Towel | Use | `tap_on` + `fill`: the tap stream grows (0.4 s) and runs while the tub fills; the water rises from the bottom to two thirds (2 s) and stays for the whole bath; it drains as the bath ends. The bather sits along the tub, back at the end without the taps and legs out towards them (`SEATS` `along`, `legsOut`). |
| `bathtubClawfoot` Clawfoot tub | 2x1 · floor | bath:bath | bathroom / bathing | Tub, Feet, Taps, Towel | Use | `tap_on` + `fill`, as the bathtub; sat in along the tub. |
| `bathtubFreestanding` Freestanding bath | 2x1 · floor | bath:bath | bathroom / bathing | Tub, Taps, Towel | Use | `tap_on` + `fill`, as the bathtub; sat in along the tub (its back at the left end). |

**In play** (signed off 2026-10-17):
- Each used in all four facings, by day, at night on WebGPU and on Low WebGL2, every part coloured.
- `tap_on` runs 2.4 s while the tub fills; `fill` holds for the bath and drains in about 2 s as it ends.
- The bather sits along the tub (`SEATS` `along`, `legsOut`): the back at the end without the taps (the bathtub's
  and clawfoot's right end, the freestanding bath's left), the hips 0.38–0.42 m from the middle, the legs in the tub
  and under the water towards the taps, the arms on the long rims; nothing through the panel or the rim.
- The freestanding bath's seat is in `SEATS`, not its content.
- Since the authenticity review (G-06): bathers sit on the tub's floor (seats 0.15, 0.17, 0.15), the heels on it and
  the shoulders over the rim at every stage; the clawfoot bather's arm drapes over its higher rim.
- Sat in the water the bather wears a towel (the shower's); in the hot tub, the whirlpool and the cold plunge
  (outdoor and packs ledgers) swimwear: a swimsuit, or trunks (`BATH_WEAR`).

## B4. Laundry (6)

- **Family:** `tools/art/blender/families/laundry.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `laundryWasher` Washing machine | 1x1 · floor | laundry:clean unload:clean | bathroom / laundry | Body, Door, Panel | Use + running | A machine that runs on its own (content `runs`, 60 minutes, the `washer` sound). *Load the washing* (5 min, a dirty load: `workOn`, about 0.4 a resident a day; the basket of washing carried there) opens the door (`door_open` 110°, `hold` 2.5 s); once loaded it washes with nobody there, the drum turning (`running`: `drum_spin`, a turn each 1.2 s). Done, *Take out the washing* (5 min) opens the door again, leaves a washed load and carries it on in the basket to the dryer or the line. |
| `laundryDryer` Tumble dryer | 1x1 · floor | laundry:clean unload:clean | bathroom / laundry | Body, Door, Panel | Use + running | As the washer: *Load the dryer* (5 min, a washed load), it tumbles 45 minutes on its own (`drum_spin`, a turn each 1.8 s; the `dryer` sound), *Take out the dry laundry* (5 min) leaves a dry load and carries it, an armful, to the laundry basket. |
| `towelRail` Towel rail | 1x1 · wall | — | bathroom / laundry | Rail, Towels, Hand towel | Static | Static. |
| `towelLadder` Towel ladder | 1x1 · floor | — | bathroom / laundry | Ladder, Top towel, Middle towel, Bottom towel | Static | Static. |
| `laundryBasket` Laundry basket | 1x1 · floor | fold:clean | bathroom / laundry | Basket, Rim & weave | Static | A wicker basket with clean washing on top; no lid is built (`open.md`). *Fold the laundry* (15 min, a dry load), a towel being folded in the hands (`holds`: `item.washingPiece`). A generated laundry room has one when there's room after the dryer (43 of 55 in 180 sample houses). |
| `bathMat` Bath mat | 1x1 · rug | — | bathroom / laundry | Mat, Pile | Static | Static, flat. |

**In play** (signed off 2026-10-16):
- The washing machine ("Do the laundry") and tumble dryer ("Fold the laundry") against the walls in all four
  facings: the door swings open, holds 2.5 s and shuts; the drum turns (`drum_spin`) for the whole use and stops
  after it.
- Since 2026-10-21 (review A-bathroom-03) the washer's drum waits for its door to shut (it turned behind the open
  door), and the dryer's doesn't tumble while the washing is folded.
- The towel rail hangs on the walls, the towel ladder leans on them; the basket and bath mat stand free.
- The towel ladder's colours are Top towel, Middle towel and Bottom towel, listed top to bottom (manifest labels).
- Since 0.5.0 (I-bathroom-07) the washer and the dryer run on their own: loaded in 5 minutes, left to wash (60) or
  dry (45) with their drum turning and their sound, emptied in 5. Carried between them: the basket of washing
  (`item.washingBasket`) and an armful of dry washing (`item.washingArmful`), both built in `laundry.py`, shown only
  (content `brings`, `holds`). Checked on `review-stage` in WebGPU and WebGL: the basket in the arms on the way to the
  washer and from it to the dryer, the drum turning with nobody there and stopping once done, the armful to the
  basket, the towel in the hands folding (2026-10-26).

## B5. Bathroom storage and mirrors (3)

- **Family:** `tools/art/blender/families/bathroom.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `mirrorCabinet` Mirror cabinet | 1x1 · wall | brushTeeth:wash | bathroom / sinks | Cabinet, Frame | Use | `door_open`: the mirrored door swings open 100° in 0.8 s, the reflection with it, for "Brush your teeth"; `hold` 1.5 s. The mirror glass is in the file (E6); shelves, a cup and bottles behind the door. |
| `mirrorRound` Round mirror | 1x1 · wall | primp:idle speech:talk | bathroom / sinks | Frame, Strap | Static | Static; the mirror glass is in the file (E6), hung on its strap. Check the mirror and practise a speech, as the other mirrors. |
| `bathCabinetTall` Tall bathroom cabinet | 1x1 · floor | takeTowel:fetch | bathroom / sinks | Cabinet, Doors, Towels | Use | `door_open`: the upper door swings open onto its towels for "Take a fresh towel"; `hold` 1.5 s. |

**In play** (signed off 2026-10-16):
- Checked on the walls in all four facings: the mirror cabinet's door opens for "Brush your teeth" with the resident
  on the tile in front, and shuts; the tall cabinet's upper door opens for "Take a fresh towel" and shuts.
- Both uses are not autonomous, so generated houses play as before; housegen hangs a wall thing with a use only where
  its front tile is free and reachable.
- The tall cabinet's door colour is called Doors (it colours both doors); the mirror cabinet's "Frame" is its door,
  a white frame round the mirror. The colour studio's "In use" opens both doors.
