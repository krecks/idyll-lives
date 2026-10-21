# Bathroom: authenticity ledger

What it covers: the 26 objects of [docs/objects/bathroom.md](../objects/bathroom.md), in its groups and order.

- **Evidence:** `bench-results/review/bathroom/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## Sinks (5)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sink` Sink | M-bathroom-01 | OK | I-kitchen-02 | M-bathroom-01 S3 decision; I-kitchen-02 S3 content | `sheets/obj-sink.png`, `play/sink-wash-*.png`, `clips/clips-sink-all-t*.png`, `play-child/sink-wash-*.png`, `sheets/kitchen-sinks.png` |
| `sinkPedestal` Pedestal sink | — | — | — | | |
| `vanityDouble` Double vanity | — | — | — | | |
| `sinkVessel` Vessel sink | — | — | — | | |
| `sinkConsole` Console sink | — | — | — | | |

## B1. Toilets (5)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `toilet` Toilet | OK | OK | I-bathroom-04 | I-bathroom-04 S3 code | `sheets/obj-toilet.png`, `play/toilet-use-*.png`, `clips/clips-toilet-all-t*.png`, `sheets/seat-toiletM.png`, `sheets/seat-toiletF.png` |
| `toiletCompact` Compact toilet | — | — | — | | |
| `toiletSmart` Smart toilet | — | — | — | | |
| `toiletClassic` Classic toilet | — | — | — | | |
| `toiletWallHung` Wall-hung toilet | — | — | — | | |

## B2. Showers (4)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `shower` Shower | OK | A-bathroom-01 | I-bathroom-02, I-bathroom-03 | A-bathroom-01 S2 model; I-bathroom-02 S3 clip; I-bathroom-03 S3 decision | `sheets/obj-shower.png`, `play/shower-shower-*.png`, `clips/clips-shower-all-t*.png`, `play-child/shower-shower-*.png` |
| `showerStall` Shower stall | — | — | — | | |
| `showerWalkIn` Walk-in rain shower | — | — | — | | |
| `showerRain` Rain shower | — | — | — | | |

## B3. Baths (3)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bathtub` Bathtub | — | — | — | | |
| `bathtubClawfoot` Clawfoot tub | — | — | — | | |
| `bathtubFreestanding` Freestanding bath | — | — | — | | |

## B4. Laundry (6)

Family: `tools/art/blender/families/laundry.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `laundryWasher` Washing machine | — | — | — | | |
| `laundryDryer` Tumble dryer | — | — | — | | |
| `towelRail` Towel rail | — | — | — | | |
| `towelLadder` Towel ladder | — | — | — | | |
| `laundryBasket` Laundry basket | — | — | — | I-bathroom-01 S3 model (seeded) | |
| `bathMat` Bath mat | — | — | — | | |

## B5. Bathroom storage and mirrors (3)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `mirrorCabinet` Mirror cabinet | — | — | — | | |
| `mirrorRound` Round mirror | — | — | — | | |
| `bathCabinetTall` Tall bathroom cabinet | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows; durations are judged by §9 decision 2.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `sink` | wash hands and face, brush teeth, shave; in a kitchen also wash up and fill a pan | wash hands (2); a `dishes` station for washing up after meals | brushing teeth missing (I-kitchen-02); it also stands in 3 of 4 generated kitchens (M-bathroom-01) |
| `toilet` | use it, flush, wash hands after | use toilet (5, then `handwash`) | OK; no flush sound (I-bathroom-04) |
| `shower` | take a shower, rinse off after sport | take a shower (20) | OK; 20 min is over the 5–15 real range but not an outlier |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### I-bathroom-01 — The laundry basket's lid isn't built, so it can't open

- **Severity:** S3
- **Rubric:** I1, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/laundry.py:193`, `web/public/content/catalog2/bath.json:270`
- **Evidence:** Left decorative for 0.3.0: the lid is not a part of its own, so no use can open it.
- **Status:** seeded (from `docs/objects/open.md` L141)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### M-bathroom-01 — The base sink is both the bathroom basin and the kitchen sink

- **Severity:** S3
- **Rubric:** M2, M11
- **Fix class:** decision
- **Where:** `tools/art/blender/families/bathroom.py:156` (`h = 0.92`, the worktop and basin rim); `web/public/content/base.json:2449` (`category: bathroom`, `stations: ["dishes"]`); `web/public/content/housegen.json:96` (`sink` in 3 of 4 kitchen runs)
- **Evidence:** `sheets/obj-sink.png`, `sheets/kitchen-sinks.png` (generated-49, -82, -84: the same unit in a counter run beside the cooker). Its rim is 0.92 m: 1 cm over a washbasin's 0.76–0.91 (`reference.json` `basinPedestal`), inside a kitchen sink's 0.85–0.95. As a vanity unit with doors it reads right in both rooms; the strict rule only fails it as a basin. Decide which kind it is (and so which range applies), or split it into a kitchen sink and a bathroom basin.
- **Status:** open
- **Source:** pilot

### A-bathroom-01 — The shower's glass door never opens: the resident walks through it

- **Severity:** S2
- **Rubric:** A8, A10, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/bathroom.py:554` (`glass_door`, part of the still body) and `:999` (`SHOWER = {'uses': {'shower': 'water_on'}}`); `web/public/content/base.json:2552` (`inside: true`)
- **Evidence:** `play/shower-shower-00.png` to `-02.png`, `sheets/obj-shower.png`: the front has a sliding glass door with a handle, but it is built into the still body and no clip moves it; the resident steps in through the shut glass and showers behind it. The README's own example (`{'uses': {'shower': [{'clip': 'door_open', 'hold': 1}, 'water_on']}}`) is the fix: the door its own part, slid open as they step in and shut behind them, and again as they leave (`out`).
- **Status:** open
- **Source:** pilot

### I-bathroom-02 — The shower clip has no washing in it

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:466` (`ACTIONS.shower`: `spell`, `interact`, `idle`)
- **Evidence:** `play/shower-shower-01.png`, `play-child/shower-shower-01.png`: under the water the resident works the hands at chest height (`spell`) and stands; no hands to the hair, no turning under the head. Reads as waiting in the shower.
- **Status:** open
- **Source:** pilot

### I-bathroom-03 — Residents shower in their clothes, behind clear glass

- **Severity:** S3
- **Rubric:** I3
- **Fix class:** decision
- **Where:** `web/src/render/babylon/characters/outfit.ts:120` (day outfit or pyjamas only); `tools/art/blender/families/bathroom.py:552-554` (clear `glass` panels)
- **Evidence:** `play/shower-shower-01.png`: the day outfit stays on under the water and shows through the enclosure. Bed has its pyjamas; the bath and shower have nothing. A towel or swimwear-like outfit, frosted glass, or leaving it as the style's convention is the owner's call.
- **Status:** open
- **Source:** pilot

### I-bathroom-04 — Noisy uses with no sound: the toilet, the treadmill, the workbench

- **Severity:** S3
- **Rubric:** I9
- **Fix class:** code
- **Where:** `web/src/audio/world.ts:34` (`ACTION_SOUNDS`: no `toilet`, `run` or `tinker`)
- **Evidence:** `cargo review uses` Sound column (`—` for `toilet`, `treadmill`, `workbench`) and `world.ts`: no flush after the toilet, no belt hum or footfalls on the treadmill, no tools at the workbench. The shower, tap, TV and piano have theirs. Listed as missing, as I9 asks; the sounds are synthesised in code (no sound files).
- **Status:** open
- **Source:** pilot
