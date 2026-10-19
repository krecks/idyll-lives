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
| `sink` Sink | 1x1 · floor | wash:wash | bathroom / sinks | Cabinet, Worktop, Basin, Tap & handles, Plinth | Use | `water_on`: the stream grows from the gooseneck into the inset basin in 0.4 s, runs for the whole use and retracts as it ends. |
| `sinkPedestal` Pedestal sink | 1x1 · floor | wash:wash | bathroom / sinks | Pedestal, Basin & cup, Tap | Use | `water_on`, as the sink. |
| `vanityDouble` Double vanity | 2x1 · floor | wash:wash | bathroom / sinks | Cabinet, Worktop, Basins, Taps & handles, Plinth | Use | `water_on`: both taps run together (two slots). No mirrors of its own: the player hangs one above it. |
| `sinkVessel` Vessel sink | 1x1 · floor | wash:wash | bathroom / sinks | Vanity, Worktop, Bowl, Tap, Shelf & legs, Towels | Use | `water_on` into the raised bowl. |
| `sinkConsole` Console sink | 1x1 · floor | wash:wash | bathroom / sinks | Basin, Legs & tap, Towel | Use | `water_on`, as the sink. |

**In play** (signed off 2026-10-16):
- The first group checked: bought in all four facings, every part coloured (through `buy` and in the colour studio),
  washed at by day and at night, on WebGPU and WebGL2, on Medium and Low.
- `water_on` has no `hold`: it plays for the `wash` tag and stops when the use ends.
- Each basin has painted shoulders from its floor up to the worktop, so the cabinet's colour never shows as the basin.

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
| `shower` Shower | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Frame, Head & tap, Tiles | Use | `water_on`: a rain of thin streams grows down from the head for the whole use. The sliding door stays shut: a moving glass door would be a 4th draw call. |
| `showerStall` Shower stall | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Rail, Head & tap, Curtain, Tiles | Use | `water_on`, slanted from the wall head onto the resident. |
| `showerWalkIn` Walk-in rain shower | 1x1 · floor | shower:shower | bathroom / bathing | Floor, Frame, Head & tap, Wall | Use | `water_on` from a wide rain head. |
| `showerRain` Rain shower | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Frame, Head & tap, Walls | Use | `water_on` from the ceiling rose. |

**In play** (signed off 2026-10-17):
- Each showered in, in all four facings, by day and at night, on WebGPU and WebGL2, on Medium and Low, every part
  coloured. The resident steps in onto the middle of the tile (`inside`).
- `water_on` (0.5 s) grows the rain as the use starts, holds it for the whole use and shuts it as it ends.
- The colour studio frames a moving part by its posed vertices, so the stall (whose rain grows from nothing and is
  turned for the slant) draws at rest and in use.
- Three draw calls each: the tray, walls and metal; the glass (the stall's curtain); the rain.

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

## B4. Laundry (6)

- **Family:** `tools/art/blender/families/laundry.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `laundryWasher` Washing machine | 1x1 · floor | laundry:clean | bathroom / laundry | Body, Door, Panel | Use | `door_open`: the round door swings 110° for laundry, `hold` 2.5 s (loading), then shuts. L `drum_spin` (E1). |
| `laundryDryer` Tumble dryer | 1x1 · floor | laundry:clean | bathroom / laundry | Body, Door, Panel | Use | `door_open`, as the washer. L `drum_spin` (E1). |
| `towelRail` Towel rail | 1x1 · wall | — | bathroom / laundry | Rail, Towels, Hand towel | Static | Static. |
| `towelLadder` Towel ladder | 1x1 · floor | — | bathroom / laundry | Ladder, Top towel, Middle towel, Bottom towel | Static | Static. |
| `laundryBasket` Laundry basket | 1x1 · floor | — | bathroom / laundry | Basket, Rim & weave | Static | Static: a wicker basket with clean washing on top; no lid is built (`open.md`). |
| `bathMat` Bath mat | 1x1 · rug | — | bathroom / laundry | Mat, Pile | Static | Static, flat. |

**In play** (signed off 2026-10-16):
- The washing machine ("Do the laundry") and tumble dryer ("Fold the laundry") against the walls in all four
  facings: the door swings open, holds 2.5 s and shuts; the drum turns (`drum_spin`) for the whole use and stops
  after it.
- The towel rail hangs on the walls, the towel ladder leans on them; the basket and bath mat stand free.
- The towel ladder's colours are Top towel, Middle towel and Bottom towel, listed top to bottom (manifest labels).

## B5. Bathroom storage and mirrors (3)

- **Family:** `tools/art/blender/families/bathroom.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `mirrorCabinet` Mirror cabinet | 1x1 · wall | brushTeeth:wash | bathroom / sinks | Cabinet, Frame | Use | `door_open`: the mirrored door swings open 100° in 0.8 s, the reflection with it, for "Brush your teeth"; `hold` 1.5 s. The mirror glass is in the file (E6); shelves, a cup and bottles behind the door. |
| `mirrorRound` Round mirror | 1x1 · wall | — | bathroom / sinks | Frame, Strap | Static | Static; the mirror glass is in the file (E6), hung on its strap. |
| `bathCabinetTall` Tall bathroom cabinet | 1x1 · floor | takeTowel:fetch | bathroom / sinks | Cabinet, Doors, Towels | Use | `door_open`: the upper door swings open onto its towels for "Take a fresh towel"; `hold` 1.5 s. |

**In play** (signed off 2026-10-16):
- Checked on the walls in all four facings: the mirror cabinet's door opens for "Brush your teeth" with the resident
  on the tile in front, and shuts; the tall cabinet's upper door opens for "Take a fresh towel" and shuts.
- Both uses are not autonomous, so generated houses play as before; housegen hangs a wall thing with a use only where
  its front tile is free and reachable.
- The tall cabinet's door colour is called Doors (it colours both doors); the mirror cabinet's "Frame" is its door,
  a white frame round the mirror. The colour studio's "In use" opens both doors.
