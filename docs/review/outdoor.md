# Outdoor: authenticity ledger

What it covers: the 43 objects of [docs/objects/outdoor.md](../objects/outdoor.md), in its groups and order.

- **Evidence:** `bench-results/review/outdoor/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## OU1. Patio seating and tables (11)

Family: `tools/art/blender/families/outdoor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bench` Park bench | — | — | — | | |
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
