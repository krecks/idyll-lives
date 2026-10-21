# Living room: authenticity ledger

What it covers: the 50 objects of [docs/objects/living.md](../objects/living.md), in its groups and order.

- **Evidence:** `bench-results/review/living/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## L1. Sofas and sectionals (12)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sofa` Sofa | — | — | — | | |
| `loveseat` Loveseat | — | — | — | | |
| `sofaThree` Three-seat sofa | — | — | — | | |
| `sofaDesign` Designer sofa | — | — | — | | |
| `sofaThreeSeat` Three-seater sofa | — | — | — | | |
| `sofaBed` Sofa bed | — | — | — | M-living-01 S3 renderer-table (seeded) | |
| `wallBed` Fold-down wall bed | — | — | — | | |
| `sectionalMiddle` Sectional: middle seat | — | — | — | | |
| `sectionalArmLeft` Sectional: left arm | — | — | — | | |
| `sectionalArmRight` Sectional: right arm | — | — | — | | |
| `sectionalCorner` Sectional: corner | — | — | — | | |
| `chaise` Sectional: chaise | — | — | — | | |

## L2. Armchairs and small seats (11)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `armchair` Armchair | — | — | — | | |
| `armchairLounge` Lounge chair | — | — | — | | |
| `armchairLeather` Leather armchair | — | — | — | | |
| `armchairDesign` Designer lounge chair | — | — | — | | |
| `armchairWingback` Wingback armchair | — | — | — | | |
| `recliner` Recliner | — | — | — | | |
| `rockingChair` Rocking chair | — | — | — | | |
| `pouf` Leather pouf | — | — | — | | |
| `ottoman` Tufted ottoman | — | — | — | | |
| `floorCushion` Floor cushion | — | — | — | | |
| `beanBag` Bean bag | — | — | — | A-living-01 S2 renderer-table (seeded) | |

## L3. Coffee and side tables (9)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `coffeeTable` Coffee table | — | — | — | | |
| `coffeeTableGlass` Glass coffee table | — | — | — | | |
| `coffeeTableMarble` Marble side table | — | — | — | | |
| `sideTable` Side table | — | — | — | | |
| `coffeeTableRound` Round coffee table | — | — | — | | |
| `coffeeTableLong` Long coffee table | — | — | — | | |
| `nestingTables` Nesting tables | — | — | — | I-living-01 S3 decision (seeded) | |
| `sideTableRound` Round side table | — | — | — | | |
| `consoleTable` Console table | — | — | — | | |

## L4. Media (8)

Family: `tools/art/blender/families/media.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `tv` Television | — | — | — | | |
| `tvWall` Flat-screen TV | — | — | — | | |
| `tvRetro` Portable TV | — | — | — | | |
| `homeCinema` Home cinema | — | — | — | | |
| `stereo` Stereo system | — | — | — | | |
| `boombox` Boombox | — | — | — | | |
| `tvStandLow` Low TV bench | — | — | — | | |
| `fireplaceElectric` Electric fireplace | — | — | — | | |

## L5. Shelves (9)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bookshelf` Bookshelf | — | — | — | | |
| `bookcaseLow` Low bookcase | — | — | — | | |
| `bookcaseWide` Library bookcase | — | — | — | | |
| `shelvingIndustrial` Industrial shelving | — | — | — | | |
| `bookcaseTall` Tall bookcase | — | — | — | | |
| `bookcaseCorner` Corner bookcase | — | — | — | | |
| `cubeShelf` Cube shelf | — | — | — | | |
| `ladderShelf` Ladder shelf | — | — | — | | |
| `displayCabinet` Display cabinet | — | — | — | I-living-02 S3 motion (seeded) | |

## L6. Stairs (1)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `stairs` Staircase | — | — | — | M-living-02 S3 code (seeded) | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### M-living-01 — The sofa bed's mattress against the sleeper's height (0.44 against 0.42 m)

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:339` (`BEDS.sofaBed`), `tools/art/blender/families/living.py:244`, `web/public/content/catalog2/living.json:246`
- **Evidence:** Reported as a 0.44 m mattress against a 0.42 m sleeper height. At e725166 the content seat (0.44), `BEDS.sofaBed.h` (0.44) and `SB_MATTRESS` (0.44) agree, so it may have been fixed with the sofa-bed bug (0.3.0 §1.1). The three-way check (T2) should confirm, then close.
- **Status:** seeded (from `plan/release-0.3.0.md` §1.4)
- **Source:** `plan/release-0.3.0.md` §1.4; `plan/authenticity-review.md` §5.1 (T2)

### M-living-02 — The staircase loses part of its top end in some basement views

- **Severity:** S3
- **Rubric:** M7
- **Fix class:** code
- **Where:** `web/src/render/babylon/house.ts` (the stairwell and the storey cut-away)
- **Evidence:** Seen from some basement views the stairs lose part of their top end. It is the renderer's cut-away, not the model.
- **Status:** seeded (from `docs/objects/open.md` L132)
- **Source:** `docs/objects/open.md` Still open › Rendering

### A-living-01 — The bean bag's snoozer lay flat at bed height, legs in the air

- **Severity:** S2
- **Rubric:** A1, A2
- **Fix class:** renderer-table
- **Where:** `tools/art/blender/families/living.py:734`, `web/public/content/catalog2/living.json:538`
- **Evidence:** Listed for 0.3.0 polish. CHANGELOG 0.3.0 (Fixes › Poses) says "the bean bag's snoozer sits up": confirm in play and close.
- **Status:** seeded (from `plan/release-0.3.0.md` §1.4)
- **Source:** `plan/release-0.3.0.md` §1.4; `CHANGELOG.md` 0.3.0 Fixes

### I-living-01 — The nesting tables' small table has no use

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `tools/art/blender/families/living.py:925`, `web/public/content/catalog2/living.json:735`
- **Evidence:** A modelled part with no clip, left decorative for 0.3.0 (pulling the small table out has no use to play it).
- **Status:** seeded (from `docs/objects/open.md` L141)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-living-02 — The display cabinet's door has no use

- **Severity:** S3
- **Rubric:** I1, I7
- **Fix class:** motion
- **Where:** `tools/art/blender/families/living.py:1309`, `web/public/content/catalog2/living.json:971`
- **Evidence:** A modelled door with no clip, left decorative for 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L141)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5
