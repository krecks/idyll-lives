# Living room: authenticity ledger

What it covers: the 50 objects of [docs/objects/living.md](../objects/living.md), in its groups and order.

- **Evidence:** `bench-results/review/living/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Living wave rows** (2026-10-21): Shots are paths under `bench-results/review/` in the review worktree (`living/sheets/obj-<id>.png` is each object's contact sheet, `living/sheets/age-<id>.png` its seat by life stage). Extra shots taken in this wave are in `living/extra/`; the steps are in `living/review-log.md`.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## L1. Sofas and sectionals (12)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sofa` Sofa | OK | A-residents-09, A-residents-11 | I-kitchen-02 | A-residents-09 S2 clip, A-residents-11 S2 clip (residents ledger); I-kitchen-02 S3 content | `sheets/obj-sofa.png`, `play/sofa-relax-*.png`, `clips/clips-sofa-all-t*.png`, `sheets/seat-sofaM.png`, `sheets/seat-sofaF.png`, `poses/seat-sofaF-elder.png` |
| `loveseat` Loveseat | OK | A-residents-11 | OK | A-residents-11 S2 clip (residents ledger: the child kneels, the teen, the elder and here the adult man lose the shins) | `living/sheets/obj-loveseat.png`, `living/sheets/age-loveseat.png` |
| `sofaThree` Three-seat sofa | M-living-03 | A-residents-11 | I-living-09 | M-living-03 S3 decision; A-residents-11 S2 clip (residents ledger); I-living-09 S3 content | `living/sheets/obj-sofaThree.png`, `living/sheets/age-sofaThree.png` |
| `sofaDesign` Designer sofa | OK | A-residents-11 | OK | A-residents-11 S2 clip (residents ledger) | `living/sheets/obj-sofaDesign.png`, `living/sheets/age-sofaDesign.png` |
| `sofaThreeSeat` Three-seater sofa | M-living-03 | A-residents-11 | OK | M-living-03 S3 decision; A-residents-11 S2 clip (residents ledger) | `living/sheets/obj-sofaThreeSeat.png`, `living/sheets/age-sofaThreeSeat.png` |
| `sofaBed` Sofa bed | OK | A-residents-11 | I-living-04 | M-living-01 S3 renderer-table (seeded; confirmed fixed on `sofaBed`); A-residents-11 S2 clip (residents ledger); I-living-04 S2 decision | `living/sheets/obj-sofaBed.png`, `living/sheets/age-sofaBed.png`, `living/play/chunk2/sofaBed-doze-01.png` |
| `wallBed` Fold-down wall bed | M-living-07 | OK | OK | M-living-07 S3 code | `living/sheets/obj-wallBed.png`, `living/play/chunk2/wallBed-foldDown-01.png`, `living/play/chunk2/wallBed-sleep-02.png` |
| `sectionalMiddle` Sectional: middle seat | OK | A-residents-11 | OK | A-residents-11 S2 clip (residents ledger: the child) | `living/sheets/obj-sectionalMiddle.png`, `living/sheets/age-sectionalMiddle.png` |
| `sectionalArmLeft` Sectional: left arm | OK | A-residents-11 | OK | A-residents-11 S2 clip (residents ledger: the child; same seat as the middle module) | `living/sheets/obj-sectionalArmLeft.png`, `living/sheets/age-sectionalArmLeft.png` |
| `sectionalArmRight` Sectional: right arm | OK | A-residents-11 | OK | A-residents-11 S2 clip (residents ledger: the child; same seat as the middle module) | `living/sheets/obj-sectionalArmRight.png`, `living/sheets/age-sectionalArmRight.png` |
| `sectionalCorner` Sectional: corner | OK | A-residents-11 | OK | A-residents-11 S2 clip (residents ledger: the child) | `living/sheets/obj-sectionalCorner.png`, `living/sheets/age-sectionalCorner.png` |
| `chaise` Sectional: chaise | M-living-03 | OK | OK | M-living-03 S3 decision | `living/sheets/obj-chaise.png`, `living/sheets/age-chaise.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `sofa` | relax, watch TV, doze off, chat with someone beside you | relax (30, two slots); the TV's `seatNear` seat | dozing missing (I-kitchen-02) |
| `loveseat` | relax, watch TV, doze off, sit with someone | relax (30), doze (30, sitting), two slots; a TV seat | OK |
| `sofaThree` | relax, watch TV, lie down for a nap, three side by side | relax (30), doze (30, sitting), two slots | OK (dozes sitting up, as people also do); a three-seater for two (I-living-09) |
| `sofaDesign` | relax, watch TV, doze off | relax (30), doze (30), two slots | OK |
| `sofaThreeSeat` | relax, watch TV, nap, three side by side | relax (30), doze (30), three slots | OK |
| `sofaBed` | sit and relax; pull it out for a guest to sleep the night | relax (30, folded), doze (30, lying on the pulled-out bed, folded away after), pull out (2), fold away (2) | the night's sleep missing (I-living-04) |
| `wallBed` | fold it down to sleep, fold it up to free the floor | sleep (480), nap (30), fold down (2), fold up (2); folded up by the last one up | OK |
| `sectionalMiddle`, `sectionalArmLeft`, `sectionalArmRight`, `sectionalCorner` | relax, watch TV, put together into a corner sofa | relax (30), doze (30), one slot each | OK |
| `chaise` | put your feet up, lie back and read | relax (30), doze (30), legs out along it | OK (reading is on the reading chairs) |

## L2. Armchairs and small seats (11)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `armchair` Armchair | OK | A-residents-09 | I-kitchen-02 | A-residents-09 S2 clip (residents ledger); I-kitchen-02 S3 content | `sheets/obj-armchair.png`, `play/armchair-sit-*.png`, `clips/clips-armchair-all-t*.png`, `sheets/seat-armchairM.png`, `sheets/seat-armchairF.png` |
| `armchairLounge` Lounge chair | M-living-05 | A-residents-11 | OK | M-living-05 S3 content; A-residents-11 S2 clip (residents ledger) | `living/sheets/obj-armchairLounge.png`, `living/sheets/age-armchairLounge.png` |
| `armchairLeather` Leather armchair | OK | A-residents-11 | I-held-01 | A-residents-11 S2 clip (residents ledger: the child); I-held-01 S2 code (held ledger: no book) | `living/sheets/obj-armchairLeather.png`, `living/sheets/age-armchairLeather.png` |
| `armchairDesign` Designer lounge chair | M-living-06 | OK | OK | M-living-06 S3 content | `living/sheets/obj-armchairDesign.png`, `tools/art/blender/previews/living/armchairDesign.png` (review worktree) |
| `armchairWingback` Wingback armchair | OK | A-residents-11 | I-held-01 | A-residents-11 S2 clip (residents ledger: the child kneels, the shoes through the seat front); I-held-01 S2 code (held ledger) | `living/sheets/obj-armchairWingback.png`, `living/sheets/age-armchairWingback.png` |
| `recliner` Recliner | M-living-04 | A-residents-11 | OK | M-living-04 S3 model; A-residents-11 S2 clip (residents ledger: the child) | `living/sheets/obj-recliner.png`, `living/play/chunk2/recliner-doze-01.png`, `living/clips/side-small/clips-recliner-all-t99.png` |
| `rockingChair` Rocking chair | M-living-05 | A-residents-09 | I-held-01 | M-living-05 S3 content; A-residents-09 S2 clip (residents ledger: the child's feet hang); I-held-01 S2 code (held ledger) | `living/sheets/obj-rockingChair.png`, `living/sheets/age-rockingChair.png` |
| `pouf` Leather pouf | OK | OK | I-living-07 | I-living-07 S3 code | `living/sheets/obj-pouf.png`, `living/sheets/age-pouf.png` |
| `ottoman` Tufted ottoman | OK | A-residents-11 | I-living-07 | A-residents-11 S2 clip (residents ledger: the child); I-living-07 S3 code | `living/sheets/obj-ottoman.png`, `living/sheets/age-ottoman.png` |
| `floorCushion` Floor cushion | OK | I-living-06 | I-living-06 | I-living-06 S2 clip | `living/sheets/obj-floorCushion.png`, `living/sheets/age-floorCushion.png` |
| `beanBag` Bean bag | M-living-05 | OK | OK | A-living-01 S2 renderer-table (seeded; confirmed fixed on `beanBag`); M-living-05 S3 content | `living/sheets/obj-beanBag.png`, `living/sheets/age-beanBag.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `armchair` | sit and relax, read, doze off | sit down (20) | reading missing (I-kitchen-02) |
| `armchairLounge` | sit and relax, doze off | sit down (20) | OK |
| `armchairLeather` | sit, read | sit (20), read (45, sitting) | OK; no book in hand (I-held-01) |
| `armchairDesign` | lean back, doze off | relax (30), doze (30) | OK |
| `armchairWingback` | read by the fire, doze off | sit (20), read (45, sitting) | OK; no book in hand (I-held-01) |
| `recliner` | lean back with the footrest up, watch TV, doze | relax (30), doze (30), both reclined with the footrest up | OK |
| `rockingChair` | rock gently, read, knit | relax (30), read (45), both rocking | OK (knitting is a hobby object's) |
| `pouf`, `ottoman` | put your feet up, perch for a moment | sit (20) | feet up missing (I-living-07) |
| `floorCushion` | sit cross-legged, meditate | sit (20) | OK; sat on with the chair sit (I-living-06) |
| `beanBag` | flop down, play games, doze | flop (25) | OK |

## L3. Coffee and side tables (9)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `coffeeTable` Coffee table | OK | OK | I-living-03 | I-living-03 S2 content; I-held-01 S2 code (held ledger: no magazine) | `living/sheets/obj-coffeeTable.png`, `living/play/chunk1/coffeeTable-magazine-01.png` |
| `coffeeTableGlass` Glass coffee table | OK | n/a | I-living-03 | I-living-03 S2 content; I-held-01 S2 code (held ledger) | `living/sheets/obj-coffeeTableGlass.png` |
| `coffeeTableMarble` Marble side table | M-living-06 | n/a | I-living-03 | M-living-06 S3 content; I-living-03 S2 content; I-held-01 S2 code (held ledger) | `living/sheets/obj-coffeeTableMarble.png` |
| `sideTable` Side table | OK | n/a | OK | | `living/sheets/obj-sideTable.png` |
| `coffeeTableRound` Round coffee table | OK | n/a | OK | | `living/sheets/obj-coffeeTableRound.png` |
| `coffeeTableLong` Long coffee table | M-living-03 | n/a | OK | M-living-03 S3 decision | `living/sheets/obj-coffeeTableLong.png` |
| `nestingTables` Nesting tables | OK | n/a | OK | I-living-01 S3 decision (seeded; closed by owner decision 4) | `living/sheets/obj-nestingTables.png` |
| `sideTableRound` Round side table | OK | n/a | OK | | `living/sheets/obj-sideTableRound.png` |
| `consoleTable` Console table | M-living-03, M-living-06 | n/a | OK | M-living-03 S3 decision; M-living-06 S3 content | `living/sheets/obj-consoleTable.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `coffeeTable`, `coffeeTableGlass`, `coffeeTableMarble` | put drinks and snacks down, leaf through a magazine from the sofa, play a board game | flip through a magazine (15, standing at the table); a low `surface` at 0.42 | standing for the magazine (I-living-03) |
| `coffeeTableRound`, `coffeeTableLong` | put drinks down, a magazine | none; a low `surface` | OK (decor, owner decision 4) |
| `sideTable`, `sideTableRound`, `nestingTables` | a drink or a lamp beside the sofa | none; a low `surface` | OK (decor) |
| `consoleTable` | drop the keys and the post, a lamp in the hall | none; a `surface` at 0.80 | OK (decor) |

## L4. Media (8)

Family: `tools/art/blender/families/media.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `tv` Television | OK | OK | I-living-03, I-living-05 | I-living-03 S2 content; I-living-05 S2 code | `sheets/obj-tv.png`, `play/tv-watch-*.png`, `play-night/tv-watch-*.png` |
| `tvWall` Flat-screen TV | OK | n/a | I-living-03, I-living-05 | I-living-03 S2 content; I-living-05 S2 code | `living/sheets/obj-tvWall.png`, `living/extra/tv-seat/seat-tvWall-watch-01.png` |
| `tvRetro` Portable TV | M-living-06 | OK | I-living-03, I-living-05 | M-living-06 S3 content; I-living-03 S2 content; I-living-05 S2 code | `living/sheets/obj-tvRetro.png` |
| `homeCinema` Home cinema | M-living-07 | OK | I-living-03, I-living-05 | M-living-07 S3 code; I-living-03 S2 content; I-living-05 S2 code | `living/sheets/obj-homeCinema.png`, `living/extra/tv-seat/seat-homeCinema-film-01.png` |
| `stereo` Stereo system | OK | OK | I-living-03, I-living-08 | I-living-03 S2 content; I-living-08 S3 motion | `living/sheets/obj-stereo.png` |
| `boombox` Boombox | OK | OK | I-living-03 | I-living-03 S2 content | `living/sheets/obj-boombox.png` |
| `tvStandLow` Low TV bench | OK | OK | I-living-03, I-living-05 | I-living-03 S2 content; I-living-05 S2 code | `living/sheets/obj-tvStandLow.png` |
| `fireplaceElectric` Electric fireplace | OK | I-living-06 | I-living-06 | I-living-06 S2 clip | `living/sheets/obj-fireplaceElectric.png`, `living/sheets/age-fireplaceElectric.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `tv` | watch TV, play a console game | watch TV (60, from a seat within 4 m, else standing) | OK; standing at it (I-living-03), a seat facing away (I-living-05) |
| `tvWall` | watch TV, watch a film | watch TV (60, `seatNear` 4) | OK; as `tv` |
| `tvRetro` | watch TV | watch TV (60, `seatNear` 4) | OK; as `tv` |
| `homeCinema` | watch a film, binge a series | watch TV (60), watch a film (120), both `seatNear` 4 | OK; seated with a sofa in front (`living/extra/tv-seat/`), standing for the two-hour film without one (I-living-03) |
| `stereo` | put a record on, listen to music, dance | listen (30, standing), dance (20) | listening standing (I-living-03); the record never turns (I-living-08) |
| `boombox` | play a tape or the radio | listen (30, standing) | listening standing (I-living-03) |
| `tvStandLow` | watch TV | watch TV (60, `seatNear` 4) | OK; as `tv` |
| `fireplaceElectric` | switch the flames on, warm up in front of it | warm up by the fire (30, sitting on its cushion); it lights the room at night | OK; the pose (I-living-06) |

## L5. Shelves (9)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bookshelf` Bookshelf | OK | OK | I-living-03, I-kids-03, I-held-01 | I-living-03 S2 content; I-kids-03 S1 code (systems ledger: an adult does homework); I-held-01 S2 code (seeded; held ledger: no book) | `sheets/obj-bookshelf.png`, `play/bookshelf-*.png`, `clips/clips-bookshelf-all-t*.png`, `play-child/bookshelf-*.png` |
| `bookcaseLow` Low bookcase | OK | OK | I-living-03 | I-living-03 S2 content; I-held-01 S2 code (held ledger) | `living/sheets/obj-bookcaseLow.png` |
| `bookcaseWide` Library bookcase | OK | OK | I-living-03, I-kids-03 | I-living-03 S2 content; I-kids-03 S1 code (systems ledger: an adult does homework, `living/play/chunk2/bookcaseWide-homework-01.png`); I-held-01 S2 code (held ledger) | `living/sheets/obj-bookcaseWide.png`, `living/play/chunk2/bookcaseWide-homework-01.png` |
| `shelvingIndustrial` Industrial shelving | OK | n/a | OK | | `living/sheets/obj-shelvingIndustrial.png` |
| `bookcaseTall` Tall bookcase | OK | OK | I-living-03 | I-living-03 S2 content; I-held-01 S2 code (held ledger) | `living/sheets/obj-bookcaseTall.png` |
| `bookcaseCorner` Corner bookcase | OK | OK | I-living-03 | I-living-03 S2 content; I-held-01 S2 code (held ledger) | `living/sheets/obj-bookcaseCorner.png` |
| `cubeShelf` Cube shelf | OK | n/a | OK | | `living/sheets/obj-cubeShelf.png` |
| `ladderShelf` Ladder shelf | M-living-07 | n/a | OK | M-living-07 S3 code | `living/sheets/obj-ladderShelf.png`, `living/play/chunk3/ladderShelf-no-uses.png` |
| `displayCabinet` Display cabinet | OK | n/a | OK | I-living-02 S3 motion (seeded; closed by owner decision 4) | `living/sheets/obj-displayCabinet.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bookshelf` | take a book and read, study, put books away | read (45), study (60), homework (45): all standing at the shelf | sitting down to read missing (I-living-03) |
| `bookcaseLow` | take a book, put things on top | read (45, standing); a `surface` at 0.90 | standing to read (I-living-03) |
| `bookcaseWide` | take a book and read, study | read (45), study (60), homework (45), standing | standing to read (I-living-03); homework for adults (I-kids-03) |
| `bookcaseTall` | take a book and read, study | read (45), study (60), standing | standing to read (I-living-03) |
| `bookcaseCorner` | take a book and read | read (45, standing) | standing to read (I-living-03) |
| `shelvingIndustrial`, `cubeShelf` | store boxes and books | none; a `surface` on top | OK (decor) |
| `ladderShelf` | show plants and books | none | OK (decor) |
| `displayCabinet` | show off china and glass, take out the good plates | none | OK (decor, owner decision 4; I-living-02 closed) |

## L6. Stairs (1)

Family: `tools/art/blender/families/living.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `stairs` Staircase | OK | OK | OK | M-living-02 S3 code (seeded; not checked: no basement view) | `sheets/obj-stairs.png`, `scenes/stairs-*.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `stairs` | go up and down between storeys | walked on by the path finder; no use | OK |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### M-living-01 — The sofa bed's mattress against the sleeper's height (0.44 against 0.42 m)

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters/furniture.json:75` (`beds.sofaBed`), `tools/art/blender/families/living.py:249`, `web/public/content/catalog2/living.json:246`
- **Evidence:** Reported as a 0.44 m mattress against a 0.42 m sleeper height. At e725166 the content seat (0.44), `BEDS.sofaBed.h` (0.44) and `SB_MATTRESS` (0.44) agree, so it may have been fixed with the sofa-bed bug (0.3.0 §1.1). The three-way check (T2) should confirm, then close.
  Living wave: `cargo review sizes` measures the pulled-out mattress at 0.44 (`tables/sizes-living.md`), inside the sofa bed's 0.35–0.50; the content, `beds.sofaBed` and the mesh agree, and the dozer lies on it with the legs on the mattress, not in it (`living/play/chunk2/sofaBed-doze-01.png`).
- **Status:** confirmed fixed on `sofaBed`
- **Source:** `plan/release-0.3.0.md` §1.4; `plan/authenticity-review.md` §5.1 (T2); checked in wave: living

### M-living-02 — The staircase loses part of its top end in some basement views

- **Severity:** S3
- **Rubric:** M7
- **Fix class:** code
- **Where:** `web/src/render/babylon/house.ts` (the stairwell and the storey cut-away)
- **Evidence:** Seen from some basement views the stairs lose part of their top end. It is the renderer's cut-away, not the model.
- **Status:** not checked (neither the pilot's nor the living wave's evidence has a basement view; for the houses wave)
- **Source:** `docs/objects/open.md` Still open › Rendering

### A-living-01 — The bean bag's snoozer lay flat at bed height, legs in the air

- **Severity:** S2
- **Rubric:** A1, A2
- **Fix class:** renderer-table
- **Where:** `tools/art/blender/families/living.py:734`, `web/public/content/catalog2/living.json:538`
- **Evidence:** Listed for 0.3.0 polish. CHANGELOG 0.3.0 (Fixes › Poses) says "the bean bag's snoozer sits up": confirm in play and close.
  Living wave: "Flop into the bean bag" is a sit at the content's 0.30 m seat (`catalog2/living.json:549`), upright with the feet on the floor, in play and at every stage (`living/play/chunk3/beanBag-flop-01.png`, `living/sheets/age-beanBag.png`). How far the sitter sinks into the bag is M-living-05.
- **Status:** confirmed fixed on `beanBag`
- **Source:** `plan/release-0.3.0.md` §1.4; `CHANGELOG.md` 0.3.0 Fixes; checked in wave: living

### I-living-01 — The nesting tables' small table has no use

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `tools/art/blender/families/living.py:925`, `web/public/content/catalog2/living.json:735`
- **Evidence:** A modelled part with no clip, left decorative for 0.3.0 (pulling the small table out has no use to play it).
  Living wave: the small table stands under the big one and reads as a nest (`living/sheets/obj-nestingTables.png`). Side tables need no use of their own (owner decision 4, §9); the part stays ready for one.
- **Status:** closed (owner decision 4: decor stays decor)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-living-02 — The display cabinet's door has no use

- **Severity:** S3
- **Rubric:** I1, I7
- **Fix class:** motion
- **Where:** `tools/art/blender/families/living.py:1309`, `web/public/content/catalog2/living.json:971`
- **Evidence:** A modelled door with no clip, left decorative for 0.3.0.
  Living wave: a glazed cabinet with china behind the door, decor like the other shelves with no use (`living/sheets/obj-displayCabinet.png`). Taking out the good plates would need a use that the dining table's meal doesn't have.
- **Status:** closed (owner decision 4: decor stays decor)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-living-03 — Reading at the bookshelf, and TV without a seat, are an hour on foot

> Consolidation: I-packs-05 (packs ledger) merged into it: the same standing hour at `bookworm.libraryWall`, `bookworm.teaTrolley`, `bookworm.bedsideStack`, `gloomy.recordCabinet`, `romantic.gramophone`, `lazy.smartHub` and `neat.robotVacuumDock`.

- **Severity:** S2
- **Rubric:** I2, I3
- **Fix class:** content
- **Where:** `web/public/content/base.json:2649`, `:2666`, `:2683` (`bookshelf` read, study, homework: no `seatNear`); `:2622` (`tv` watch: `seatNear: 4`); `web/src/render/babylon/characters.ts:475` (`ACTIONS.read`: `lantern` standing). Living wave, the same: `web/public/content/furniture.json:1417`, `:1453`, `:1490` (the coffee tables' magazine), `:1663`, `:1706` (`stereo`, `boombox` listen), `:1735`, `:1780` (`bookcaseLow`, `bookcaseWide`); `web/public/content/catalog2/living.json` `bookcaseTall`, `bookcaseCorner` (read, study: no `seatNear`)
- **Evidence:** `play/bookshelf-homework-01.png`, `sheets/obj-bookshelf.png`: the resident stands at the shelf for 45–60 minutes holding a book that isn't there (I-held-01) at the chest. People take the book to a chair. The content already has the mechanism: the TV's watch takes a free seat within 4 m (`seatNear`, `world.rs:4298`); the three reading uses don't ask for one. The TV shows the fallback: with no seat near, the watcher stands half a metre from the screen, arms folded, for the hour (`play/tv-watch-01.png`). Giving the bookshelf uses `seatNear` fixes the common case; what a reader does with no seat near is for triage.
  Living wave: the same at every living thing used for a long time on foot: read and study at `bookcaseLow`, `bookcaseWide`, `bookcaseTall`, `bookcaseCorner` (45–60 min, `living/play/chunk3/bookcaseTall-read-01.png`); "Flip through a magazine" standing over `coffeeTable`, `coffeeTableGlass` and `coffeeTableMarble` (15 min, `living/play/chunk1/coffeeTable-magazine-01.png`); listening to music standing at `stereo` and `boombox` (30 min). The TVs (`tvWall`, `tvRetro`, `tvStandLow`) and the home cinema's watch (60) and film (120 min) were watched standing 0.5 m from the screen in Phase 0 play only because play buys each thing alone on the stage (`living/play/chunk2/homeCinema-film-01.png`). With a sofa 3 m in front, the home cinema's watch and film are watched sitting on it (`living/extra/tv-seat/seat-homeCinema-film-01.png`): the standing hour and the standing two-hour film are the no-seat fallback above. Which seat is taken is I-living-05.
- **Status:** fixed (9a5cc144)
- **After:** `seatNear` (4 m) on the bookshelf, the four bookcases, the stereo, the boombox and the packs' library wall, record cabinet, gramophone, smart hub and robot vacuum dock; 3 m on the tea trolley and the bedside stack; 2 m on the three coffee tables' magazine. Each sat with a seat placed near (`bench-results/review/fixes/content/g03/`, records `pose: sit`); with none near they stand, as the TV's watcher (Q-14).
- **Source:** pilot; widened in wave: living

### M-living-03 — The three-seat sofas, the chaise, the long coffee table and the console table are over the real size

- **Severity:** S3
- **Rubric:** M1
- **Fix class:** decision
- **Where:** `tools/art/blender/families/living.py:449`, `:1376` (`sofaThree`, 2.82 m wide), `:500`, `:1383` (`sofaThreeSeat`, 2.86), `:509`, `:1396` (`chaise`, 1.90 deep), `:914`, `:1438` (`coffeeTableLong`, 1.66 wide), `:952`, `:1444` (`consoleTable`, 1.80 × 0.56)
- **Evidence:** `cargo review sizes` (`tables/sizes-living.md`) against `reference.json` with the ±10 % of owner decision 1: the three-seat sofas are 2.82 and 2.86 m against 1.90–2.40 (out above 2.64), the chaise 1.90 m deep against 1.40–1.70 (above 1.87), the long coffee table 1.66 m against 0.60–1.40 (above 1.54), the console table 1.80 × 0.56 m against 0.80–1.60 × 0.25–0.45 (0.56 deep is a sideboard's depth, not a console's; its description says "slim"). The sofas fill their 3-tile footprint and read as large sofas, not wrong (`living/sheets/obj-sofaThree.png`); the console table reads as a long open table (`living/sheets/obj-consoleTable.png`). Whether to build them inside the range (a 2.4 m sofa centred on 3 tiles) or accept the large ones is the owner's. The other flags in the table are the measuring, not the model, and are closed here: `coffeeTableGlass` and `coffeeTableMarble` H is the plant bowl and the vase on top (tops 0.42, inside); `boombox` H is the crate it stands on (the kind's range is the boombox's); `shelvingIndustrial` 0.53 m is boltless utility shelving's usual depth; `wallBed` W includes its two side bookshelves (its description); `fireplaceElectric` D includes its floor cushion; `tvRetro` W is a console set's (M-living-06); `recliner` is M-living-04.
- **Status:** won't fix (accepted, Q-08) for the sofas, chaise and long coffee table; the console table's text fixed (b3f6b82d)
- **After:** They fill their tiles and read as large, not wrong (owner, Q-08); the console table no longer says slim.
- **Source:** wave: living (`cargo review sizes`)

### M-living-04 — The recliner's back stops below the sitter's shoulders, so reclined it doesn't read

- **Severity:** S3
- **Rubric:** M1, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/living.py:634` (`recliner`), `:1414` (size 0.92 × 0.843 × 0.70)
- **Evidence:** The lead "the recliner doesn't look reclined in play though its clip plays": the clip does play in play, from 1 s into relax and doze to the end (`play.jsonl`: `recline` and `footrest_up` open at t = 1), and at full size the footrest is up under the sitter's feet and she leans back a little (`living/play/chunk2/recliner-doze-01.png`; the clip alone, `living/clips/side-small/clips-recliner-all-t99.png`). It reads weakly because the chair is 0.84 m high and 0.70 m deep against a real recliner's 0.95–1.10 × 0.85–1.05 (out on both, `tables/sizes-living.md`): the back ends at the sitter's shoulder blades with no headrest, so tipping it 25° moves little that shows, and the dark leather hides the footrest from the game camera. A taller back with a head cushion, and a deeper seat, would make the recline its silhouette.
- **Status:** fixed (773afc7d)
- **After:** 0.92 × 0.99 × 0.80 m: the back rises to 1.0 m with a head cushion, the base to the tile's front; reclined, the seat and back tip 25° and glide 8 cm forward (a wall-hugger's, so the back's top stays off the wall), the sitter's head on the cushion and the legs out on the footrest (`fixes/blender-rooms/gallery/gallery-recliner.png`, `clips/clips-recliner-all-t99.png`, `play/rooms-recliner-relax-02.png`, `poses/poses-recliner-relax-teen.png`). `cargo review sizes` living: 13 outside, not the recliner. The child's shins still go into the seat front (A-residents-11, `poses/poses-recliner-sit-child.png`, as before).
- **Source:** wave: living (Phase 0 "Seen" lead)

### M-living-05 — Three seats disagree with their models: the lounge chair, the rocking chair and the bean bag

- **Severity:** S3
- **Rubric:** M2, A1
- **Fix class:** content
- **Where:** `web/src/render/babylon/characters/furniture.json:26` (`seats.armchairLounge` 0.42), `web/public/content/catalog2/living.json:497` (`rockingChair` seat 0.48), `:549` (`beanBag` seat 0.30)
- **Evidence:** The heights check (`heights.txt`) and `cargo review sizes`: the lounge chair's seat is 0.42 in `seats` against 0.45 in the mesh (3.0 cm in; its cushion sinks 2 cm, so the elder's thighs show a centimetre in, `living/sheets/age-armchairLounge.png`); the rocking chair's is 0.48 in the content against 0.458 in the mesh (the rider 2.2 cm above the cushion, rocking with it); the bean bag's is 0.30 against 0.458 (squashed 15 % for flop, about 0.39, so the sitter is about 9 cm into the bag: it reads as sinking in, which a bean bag does, but a child sinks to the chest with only the head and arms out, `living/sheets/age-beanBag.png`). The first two are outside the 1 cm tolerance; the bean bag's is a choice (A-living-01's fix) to keep, with a seat per life stage or a shallower one for children.
- **Status:** fixed (a920f174)
- **After:** `seats.armchairLounge` 0.44 (its cushion 0.45, sat a centimetre into), `rockingChair` content seat 0.46; the bean bag's 0.30 kept (sinking in, A-living-01's choice). `cargo check-glb`: the lounge chair's and the rocker's warnings gone; `fixes/tables/play-b/b-armchairLounge-sit-01.png`, `b-rockingChair-relax-01.png`.
- **Source:** wave: living (heights check, `cargo review sizes`)

### M-living-06 — Names and descriptions that don't match their models

- **Severity:** S3
- **Rubric:** M5
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:1343`, `:1353` (`armchairDesign`), `:1491`, `:1501` (`coffeeTableMarble`), `:1583` (`tvRetro`); `web/public/content/catalog2/living.json:808` (`consoleTable`); `tools/art/blender/families/living.py:595` (`armchair_design`)
- **Evidence:**
  - `armchairDesign` "Designer lounge chair", "a mid-century swivel chair in leather and bent wood": the model is a high-backed executive chair, leather pads on a chrome frame and a five-star base on castors, no bent wood (`tools/art/blender/previews/living/armchairDesign.png`, `living/sheets/obj-armchairDesign.png`). In a living room it reads as an office chair; either the description and name follow the model, or the model becomes the low bent-plywood lounge chair the text describes (a decision).
  - `coffeeTableMarble` "Marble side table", "a round marble top on bent steel legs": its top is at coffee-table height (0.42, the reference's choice of kind) and it stands on a brass pedestal (`living/sheets/obj-coffeeTableMarble.png`).
  - `tvRetro` "Portable TV": a 0.94 m wooden console set on legs with rabbit ears (`living/sheets/obj-tvRetro.png`); right for a console television, not for a portable one.
  - `consoleTable` "A slim oak table": 0.56 m deep (M-living-03) on black steel legs (`living/sheets/obj-consoleTable.png`).
- **Status:** fixed (b3f6b82d)
- **After:** "Executive lounge chair" (chrome five-star base, padded leather), "Round marble coffee table" (brass pedestal), "Console TV", the console table not slim.
- **Source:** wave: living

### M-living-07 — Things made for a wall can stand in mid-room with nothing holding them

> Consolidation (2026-10-21): closed as a duplicate of M-bathroom-03 (systems ledger), which takes its items and evidence.

- **Severity:** S3
- **Rubric:** M4, M7
- **Fix class:** code
- **Where:** `web/public/content/catalog2/living.json:317` (`wallBed`), `:956` (`ladderShelf`); `web/public/content/furniture.json:1613` (`homeCinema`); `tools/art/blender/families/media.py:274` (the screen's case "on the wall above"); no content field says a floor thing must have a wall behind it (`fillsWall` only keeps the wall over it clear, `crates/sim-core/src/world.rs:1584`)
- **Evidence:** The lead "the wall bed placed freestanding in mid-room": Phase 0 play put the wall bed at the middle of the stage floor (`[9, 11, 0]`) and the game bought it there; folded down it is an open cabinet standing in the room with the bed on the floor in front (`living/play/chunk2/wallBed-foldDown-01.png`). A real one is bolted to the wall or it tips over. The same for the ladder shelf, which leans on nothing (`living/play/chunk3/ladderShelf-no-uses.png`; `docs/objects/living.md` L5 notes it), and the home cinema, whose screen case hangs in the air over the media unit with no wall to hold it (`living/extra/tv-seat/seat-homeCinema-film-01.png`). Players will mostly put them on a wall; the game could ask for one (a "needs a wall behind" flag the build tool checks) or the models could carry their own support. Related, content: the wall bed has no `fillsWall` (the tall bookcases and the display cabinet have it), so a picture or a shelf can be hung on the wall where its 2.22 m cabinet stands, and play's free-spot rule (`tools/bench/check.mjs:561`, by `fillsWall`) put it mid-room although `tools/bench/README.md` lists wall beds among the things it puts on the back wall.
- **Status:** closed (duplicate of M-bathroom-03)
- **Source:** wave: living (Phase 0 "Seen" lead)

### I-living-04 — The sofa bed can't be slept in for a night

- **Severity:** S2
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `web/public/content/catalog2/living.json:246` (`sofaBed`: relax, doze, pull out, fold away); `web/public/content/base.json:3679` (`roomKinds`: a `sleep` tag makes the room a bedroom)
- **Evidence:** A sofa bed is bought to sleep a guest or a resident for the night (`reference.json`: "pull it out for a guest to sleep", sleep 420–540 min). Its only lying use is "Doze off", 30 minutes with the `nap` tag; you can pull the bed out (`living/sheets/obj-sofaBed.png`) but nobody can sleep the night in it, so a household with a sofa bed and no bed has nowhere to sleep. The content can't just add a `sleep` use: the tag turns the living room into a bedroom (`tools/art/blender/README.md`, "Room kind"), which is why only `nap` is there. Either a sleep that doesn't type the room (a tag or rule for guest beds), or accept the sofa bed as a napping sofa and say so in its description.
- **Status:** fixed (aba5f915): a sleep that doesn't type the room (Q-15)
- **After:** The sofa bed offers "Sleep the night" (480 min, energy 0.85, comfort 0.25, the mattress pulled out for it and folded away by the last one up), with `spare`: its tags don't count towards the room's kind, so a living room with a sofa bed stays a living room, and residents sleep in it only with no other bed free for them; it counts as a bed place for a household moving in. On `review-stage-night` (no bed) Ada sleeps the night on it in pyjamas (`fixes/babies-crops/sofa-bed/sofa-bed-sofaBed-sleep-01.png`, against `living/sheets/obj-sofaBed.png`). Sim tests `tests/spare_beds.rs`: one in the bed and the other on the sofa bed for the night, the room still `living`; with two beds, nobody on the sofa bed. The futon and the wall bed keep their own `sleep` (bedroom furniture: they do make the room a bedroom).
- **Source:** wave: living (I1 against `reference.json` uses)

### I-living-05 — moved to systems.md

### I-living-06 — moved to residents.md

### I-living-07 — The pouf and the ottoman are never footstools

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** code
- **Where:** `web/public/content/furniture.json:1389` (`pouf`), `web/public/content/catalog2/living.json:572` (`ottoman`): one use each, "Sit down"
- **Evidence:** A pouf or an ottoman is first a place to put your feet up from the armchair or sofa beside it (`reference.json` pouf: "put your feet up; perch for a moment"); in the game both are only sat on, as a stool (`living/sheets/obj-pouf.png`, `living/sheets/obj-ottoman.png`). Feet up needs a pairing: a seat's use that finds a footstool in front of it and a legs-up pose, like the recliner's footrest. New behaviour, filed only (owner decision 3).
- **Status:** fixed (5f4411b9)
- **After:** a resident sitting with a pouf or an ottoman in front of them (within reach, nobody on it) puts their feet up on it, the heels on its top, slid forward and slouched in the seat as reaching it takes (characters.ts `footstoolFor`, `feetUp`; `footstools` in characters/furniture.json): the armchair with the pouf, the sofa with the ottoman, for the teen, the adult and the elder; a child's legs don't reach a stool a tile off (`fixes/pairings/seats/`: `after-armchair-sit-02.png`, `sofa-sofa-relax-02.png`, `teen-armchair-sit-01.png`, `elder-sofa-relax-01.png`, `child-armchair-sit-01.png`). The recliner doesn't pair: its own footrest comes up (`after-recliner-relax-02.png`). An armchair with a footstool in front, refused before ("would block the way"), is sat in from beside it (sim-core `approach_tiles`, `tests/footstools.rs`). A lone sitter on a sofa takes its first free seat, not the one before the footstool (I-living-10). Before: `fixes/pairings/before/obj-pouf.png`, `obj-ottoman.png`.
- **Source:** wave: living (I1)

### I-living-08 — The stereo's record player never turns; a CD tray opens instead

- **Severity:** S3
- **Rubric:** I7
- **Fix class:** motion
- **Where:** `tools/art/blender/families/media.py:300` (`stereo`), `:344`–`:357` (`tray`, the only use clip); `web/public/content/furniture.json:1663` ("put a record on, or dance")
- **Evidence:** The stereo is a hi-fi cabinet with a turntable and a record on top, the amplifier and a CD player below, and records in the shelf (`living/sheets/obj-stereo.png`). "Listen to music" and "Dance" slide out the CD tray under the amplifier for 1.2 s and pulse the woofers; the record on top, the thing the description puts on, never turns and its lid or arm never moves. The platter spinning while music plays (a loop, as the woofers) would match the description; the tray could stay.
- **Status:** fixed (abfc32d9)
- **After:** the record (platter, vinyl and label, a stripe on the label) turns at 33 rpm while the music plays (`record`, a loop) and the woofers pulse (`every` 0.45 s); the CD tray is still now, as its part's draw went to the record (4 draws: `fixes/motions/g1718/views/stereo-t0.5.png`, `play-stage.jsonl`).
- **Source:** wave: living

### I-living-09 — The three-seat sofa seats two

- **Severity:** S3
- **Rubric:** I2
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:1185` (`sofaThree` `slots: 2`)
- **Evidence:** "Three-seat sofa", 2.82 m with three seat cushions (`living/sheets/obj-sofaThree.png`), takes two residents; the other three-seater, `sofaThreeSeat`, takes three (`catalog2/living.json:396`). A third member of the family can't join the two on it to watch TV.
- **Status:** fixed (eaca9ed8)
- **After:** `slots: 3`: three on the three-seat sofa.
- **Source:** wave: living (`cargo review uses`)

### I-living-10 — A lone sitter takes the sofa's first seat, not the one before the footstool

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:272` (`pick_slot`: without `places`, the first free slot)
- **Evidence:** `fixes/pairings/seats/after2-sofa-relax-02.png` (`play-after2.json`): a two-seat sofa with the ottoman in front of its second seat; Ada, alone, sits in the first and keeps her feet down. Someone on their own would take the seat with the footstool. The sim doesn't know footstools (the pairing is the renderer's, G-69): a content `footstool` flag and a `pick_slot` that prefers a slot with one in front would do.
- **Status:** open
- **Source:** fix: pairings

## Seen here for findings in other ledgers

For the coordinator; nothing new is filed for these here. Carried into those findings at the consolidation (2026-10-21).

- **A-residents-11** (residents ledger), widened: the child "kneels" on most living seats (the lead "children kneel on armchairs"): thighs on the seat, the shins down inside the cushion, the shoes showing through or under the seat front, on `loveseat`, `sofaThree`, `sofaDesign`, `sofaThreeSeat`, `sofaBed`, the four sectional modules, `armchairLounge`, `armchairLeather`, `armchairWingback`, `recliner` and `ottoman` (`living/sheets/age-armchairWingback.png`, `age-sectionalMiddle.png`, `age-ottoman.png`); not on `pouf` (0.50 deep) or `rockingChair`. Teens and elders lose the shins on `sofaThree`, `sofaDesign`, `loveseat` and `armchairLounge`, and so do the adult man on the loveseat and the adult woman on `sofaThree` (`living/sheets/age-loveseat.png`, `age-sofaThree.png`), where the pilot saw adults fine on the base sofa.
- **A-residents-09** (residents ledger): the child's feet hang on `rockingChair` and `pouf`.
- **I-held-01** (held ledger): no book on the reading chairs (`armchairLeather`, `armchairWingback`, `rockingChair`) or at the four bookcases, no magazine at the three coffee tables (the coffee table's drawer opens with one and it stays there).
- **I-kids-03** (systems ledger): an adult does homework at `bookcaseWide` (`living/play/chunk2/bookcaseWide-homework-01.png`).
