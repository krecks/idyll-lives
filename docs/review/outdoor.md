# Outdoor: authenticity ledger

What it covers: the 43 objects of [docs/objects/outdoor.md](../objects/outdoor.md), in its groups and order.

- **Evidence:** `bench-results/review/outdoor/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## OU1. Patio seating and tables (11)

Family: `tools/art/blender/families/outdoor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bench` Park bench | M-outdoor-02 | OK | OK | M-outdoor-02 S3 model | `sheets/obj-bench.png`, `play/bench-sit-*.png`, `sheets/seat-benchM.png`, `sheets/seat-benchF.png`, `play-elder/bench-sit-*.png` |
| `patioSet` Folding bistro set | — | — | — | | |
| `picnicTable` Picnic table | — | — | — | | |
| `gardenChair` Garden chair | — | — | — | | |
| `patioTableRound` Round patio table | — | — | — | | |
| `patioTable4` Teak patio table | — | — | — | | |
| `patioChair` Patio chair | — | — | — | | |
| `adirondackChair` Adirondack chair | — | — | — | | |
| `benchGarden3` Garden bench | — | — | — | | |
| `outdoorBar` Outdoor bar | — | — | — | | |
| `parasol` Parasol | — | — | — | | |

## OU2. Loungers, swings and the hot tub (5)

Family: `tools/art/blender/families/outdoor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sunLounger` Sun lounger | — | — | — | | |
| `hammock` Hammock | — | — | — | | |
| `porchSwing` Porch swing | — | — | — | | |
| `swingSet` Swing set | — | — | — | | |
| `hotTubCedar` Cedar hot tub | — | — | — | | |

## OU3. Grills and fire (3)

Family: `tools/art/blender/families/outdoor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `grill` Charcoal grill | — | — | — | M-outdoor-01 S2 code (seeded) | |
| `grillStation` Gas grill station | — | — | — | | |
| `firePit` Fire pit | — | — | — | | |

## OU4. Play and yard (9)

Family: `tools/art/blender/families/yard.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sandbox` Sandpit | — | — | — | | |
| `bikeRack` Bike rack | — | — | — | | |
| `raisedBed` Raised vegetable bed | — | — | — | | |
| `shedSmall` Garden shed | — | — | — | | |
| `greenhouse` Greenhouse | — | — | — | | |
| `compostBin` Compost bin | — | — | — | | |
| `rainBarrel` Rain barrel | — | — | — | | |
| `clothesline` Washing line | — | — | — | | |
| `firewoodStack` Log store | — | — | — | | |

## OU5. Pond, small garden things and lights (9)

Family: `tools/art/blender/families/garden_things.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `garden.pond` Garden pond | — | — | — | | |
| `planterBox` Planter box | — | — | — | | |
| `birdhouse` Birdhouse | — | — | — | | |
| `birdFeederPole` Bird feeders | — | — | — | | |
| `dogHouse` Dog house | — | — | — | | |
| `sundial` Sundial | — | — | — | | |
| `steppingStones` Stepping stones | — | — | — | | |
| `lampPost` Garden lamp post | — | — | — | | |
| `stringLights` String lights | — | — | — | | |

## Garden decor (6)

Family: `tools/art/blender/families/garden_decor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `garden.gnome` Garden gnome | — | — | — | | |
| `garden.birdbath` Bird bath | — | — | — | | |
| `garden.fountain` Stone fountain | — | — | — | | |
| `garden.lantern` Stone lantern | — | — | — | | |
| `garden.wheelbarrow` Wheelbarrow planter | — | — | — | | |
| `garden.boulder` Garden boulders | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows; durations are judged by §9 decision 2.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bench` | sit and enjoy the garden, chat, feed the birds | sit on the bench (30, two slots) | OK |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### M-outdoor-01 — Night glow draws through other parts: the grill's coals through its shut lid

- **Severity:** S2
- **Rubric:** M10
- **Fix class:** code
- **Where:** `web/src/render/babylon/models.ts:73` (`glows`), `tools/art/blender/families/outdoor.py:64`
- **Evidence:** A glowing material draws through the parts in front of it. `outdoor.py:64` now builds the grill's hot coals with no glow "which would show through its shut lid", so the grill may be worked round while the renderer still has the fault for other models.
- **Status:** seeded (from `docs/objects/open.md` L131)
- **Source:** `docs/objects/open.md` Still open › Rendering; `plan/authenticity-review.md` §3.1 (M10)

### M-outdoor-02 — The park bench's slats are 4 cm above where the sitter is put

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/outdoor.py:153` (seat slats at 0.47 m, 3.5 cm thick: top 0.49); `web/src/render/babylon/characters/furniture.json:5` (`SEATS.bench` 0.45)
- **Evidence:** `cargo check-glb` heights warning (`model.bench seat 0.45 m … the model 0.491 m there: 4.1 cm higher`) and `sizes-outdoor.md` (seat 0.491 > 0.48, the top of the 0.42–0.48 range). In play (`play/bench-sit-01.png`, `sheets/seat-benchF.png`) the thighs hide the 4 cm the pelvis sinks into the slats, so it reads as seated; it fails the strict range by 1 cm and the three-way check by 4 cm. Lowering the slats to 0.45 fixes both; raising `SEATS.bench` alone would leave the seat out of range.
- **Status:** open
- **Source:** pilot (T1 warning, sizes table)
