# Bathroom

The seven sinks are done (`families/bathroom.py`, `water_on`), the five B1 toilets (`lid_up`), the four B2 showers (`water_on`) and the three B3 baths (`tap_on`, `fill`); the six B4 laundry models are done in `families/laundry.py`. Water here grows the same way: a stream built at
0.001 and `G.grow`n for the whole use. Mirror glass stays a primitive `part` until the engine reads mirrors from
files (E6), as the double vanity did.

## B1. Toilets (5) — **Done** (in Validate › Toilets)

- **Family:** `tools/art/blender/families/bathroom.py`
- **Validate group:** `toilets` (Validate › Toilets)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `toilet` Toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat and lid, Cistern | Use | `lid_up`: the lid swings up 100° as the use starts and down as it ends. |
| `toiletCompact` Compact toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat and lid, Cistern | Use | `lid_up`. |
| `toiletSmart` Smart toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat and lid, Cistern | Use | `lid_up`: lid and seat rise on their own (slower, 0.8 s). |
| `toiletClassic` Classic toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat and lid, Cistern | Use | `lid_up`; the close-coupled cistern (the content and placeholder: no high cistern and chain) and wooden seat are static. |
| `toiletWallHung` Wall-hung toilet | 1x1 · floor | use:toilet | bathroom / toilets | Bowl, Seat and lid, Cistern | Use | `lid_up`. |

## B2. Showers (4) — **Done** (in Validate › Showers)

- **Family:** `tools/art/blender/families/bathroom.py`
- **Validate group:** `showers` (Validate › Showers)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `shower` Shower | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Frame, Head and tap | Use | `water_on`: a rain of thin streams grows down from the head for the whole use. The sliding door stays shut: a moving glass door would be a 4th draw call. |
| `showerStall` Shower stall | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Frame, Head and tap | Use | `water_on`, slanted from the wall head onto the resident. |
| `showerWalkIn` Walk-in rain shower | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Frame, Head and tap | Use | `water_on` from a wide rain head. |
| `showerRain` Rain shower | 1x1 · floor | shower:shower | bathroom / bathing | Tray, Frame, Head and tap | Use | `water_on` from the ceiling rose. |

## B3. Baths (3) — **Done** (in Validate › Baths)

- **Family:** `tools/art/blender/families/bathroom.py`
- **Validate group:** `baths` (Validate › Baths)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bathtub` Bathtub | 2x1 · floor | bath:bath | bathroom / bathing | Tub, Feet or panel, Taps | Use | `tap_on` + `fill`: the tap stream grows (0.4 s) and runs while the tub fills; the water rises from the bottom to two thirds (2 s) and stays for the whole bath; it drains as the bath ends. |
| `bathtubClawfoot` Clawfoot tub | 2x1 · floor | bath:bath | bathroom / bathing | Tub, Feet or panel, Taps | Use | `fill`. |
| `bathtubFreestanding` Freestanding bath | 2x1 · floor | bath:bath | bathroom / bathing | Tub, Feet or panel, Taps | Use | `fill`. |

## B4. Laundry (6) — **Done** (in Validate › Laundry)

- **Family:** `tools/art/blender/families/laundry.py`
- **Validate group:** `laundry` (Validate › Laundry)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `laundryWasher` Washing machine | 1x1 · floor | laundry:clean | bathroom / laundry | Body, Door, Panel | Use | `door_open`: the round door swings 110° for laundry, `hold` 2.5 s (loading), then shuts. L `drum_spin` (E1). |
| `laundryDryer` Tumble dryer | 1x1 · floor | laundry:clean | bathroom / laundry | Body, Door, Panel | Use | `door_open`, as the washer. L `drum_spin` (E1). |
| `towelRail` Towel rail | 1x1 · wall | — | bathroom / laundry | Rail, Towels | Static | Static. |
| `towelLadder` Towel ladder | 1x1 · floor | — | bathroom / laundry | Ladder, Towels | Static | Static. |
| `laundryBasket` Laundry basket | 1x1 · floor | — | bathroom / laundry | Basket | Static | Static. (+use) the lid lifts. |
| `bathMat` Bath mat | 1x1 · rug | — | bathroom / laundry | Mat | Static | Static, flat. |

## B5. Bathroom storage and mirrors (3)

- **Family:** `tools/art/blender/families/bathroom.py`
- **Validate group:** `bathroom storage` (Validate › Bathroom Storage)
- **Placeholders to match:** `tools/art/catalog.py`, `tools/art/catalog2.py`

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `mirrorCabinet` Mirror cabinet | 1x1 · wall | — | bathroom / sinks | Cabinet, Frame | Static | Static; mirror glass as a primitive part (E6). (+use) the mirror door swings. |
| `mirrorRound` Round mirror | 1x1 · wall | — | bathroom / sinks | Frame | Static | Static; mirror part (E6). |
| `bathCabinetTall` Tall bathroom cabinet | 1x1 · floor | — | bathroom / sinks | Cabinet, Frame | Static | Static. (+use) the door swings. |
