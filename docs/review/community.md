# Community and farm: authenticity ledger

What it covers: the 36 objects of [docs/objects/community.md](../objects/community.md), in its groups and order.

- **Evidence:** `bench-results/review/community/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## CM1. Playground (8)

Family: `tools/art/blender/families/playground.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `slide` Playground slide | — | — | — | | |
| `climbingFrame` Climbing frame | — | — | — | A-community-01 S3 clip (seeded) | |
| `seesaw` Seesaw | — | — | — | | |
| `springRider` Spring rider | — | — | — | | |
| `merryGoRound` Merry-go-round | — | — | — | | |
| `basketballHoop` Basketball hoop | — | — | — | | |
| `pingPongTable` Ping-pong table | — | — | — | | |
| `parkChessTable` Park chess table | — | — | — | | |

## CM2. Street furniture (10)

Family: `tools/art/blender/families/street_furniture.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `streetLamp` Street lamp | — | — | — | | |
| `parkBench` Long park bench | — | — | — | | |
| `drinkingFountain` Drinking fountain | — | — | — | | |
| `litterBin` Litter bin | — | — | — | | |
| `busStop` Bus shelter | — | — | — | | |
| `postBox` Post box | — | — | — | | |
| `noticeBoard` Notice board | — | — | — | | |
| `phoneBooth` Phone box | — | — | — | | |
| `fountainTown` Town fountain | — | — | — | | |
| `townStatue` Founder's statue | — | — | — | | |

## CM3. Food stalls and vending (5)

Family: `tools/art/blender/families/stalls.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `iceCreamCart` Ice cream cart | — | — | — | | |
| `hotDogStand` Hot dog stand | — | — | — | | |
| `vendingSnack` Snack machine | — | — | — | | |
| `vendingDrink` Drinks machine | — | — | — | | |
| `marketStall` Market stall | — | — | — | | |

## CM4. Shops and café (5)

Family: `tools/art/blender/families/shops.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `shopCounter` Shop counter | — | — | — | | |
| `bakeryCase` Bakery case | — | — | — | | |
| `groceryShelf` Grocery shelves | — | — | — | | |
| `cafeTable` Café table | — | — | — | | |
| `cafeChair` Café chair | — | — | — | | |

## CM5. Farm (8)

Family: `tools/art/blender/families/farm.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `farm.barn` Red barn | — | — | — | | |
| `farm.silo` Grain silo | — | — | — | | |
| `farm.tractor` Little tractor | — | — | — | | |
| `farm.hayBale` Round hay bale | — | — | — | | |
| `farm.hayStack` Stack of hay bales | — | — | — | | |
| `farm.scarecrow` Scarecrow | — | — | — | | |
| `farm.trough` Water trough | — | — | — | | |
| `farm.coop` Chicken coop | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-community-01 — Climbers who start at other offsets can still meet at the pole

- **Severity:** S3
- **Rubric:** A7
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/street.ts:127` (`CLIMB`, lanes), `web/src/render/babylon/characters.ts`
- **Evidence:** Two climbers on the frame have a lane each, but if they start at other offsets they can still meet at the fireman's pole.
- **Status:** seeded (from `docs/objects/open.md` L126)
- **Source:** `docs/objects/open.md` Still open › Residents
