# Bathroom: authenticity ledger

What it covers: the 26 objects of [docs/objects/bathroom.md](../objects/bathroom.md), in its groups and order.

- **Evidence:** `bench-results/review/bathroom/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## Sinks (5)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sink` Sink | — | — | — | | |
| `sinkPedestal` Pedestal sink | — | — | — | | |
| `vanityDouble` Double vanity | — | — | — | | |
| `sinkVessel` Vessel sink | — | — | — | | |
| `sinkConsole` Console sink | — | — | — | | |

## B1. Toilets (5)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `toilet` Toilet | — | — | — | | |
| `toiletCompact` Compact toilet | — | — | — | | |
| `toiletSmart` Smart toilet | — | — | — | | |
| `toiletClassic` Classic toilet | — | — | — | | |
| `toiletWallHung` Wall-hung toilet | — | — | — | | |

## B2. Showers (4)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `shower` Shower | — | — | — | | |
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
