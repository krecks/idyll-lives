# Residents: authenticity ledger

What it covers: the residents: 2 bodies × 6 life stages, the `build` slider, hair, painted clothes and shells, the resident clips and the procedural layers. Built by `cargo characters` (`crates/tools/src/characters/`); contract in [docs/characters-format.md](../characters-format.md), look in [docs/design/look.md](../design/look.md). Hug, kiss and conversation pairs are in [social.md](social.md).

- **Evidence:** `bench-results/review/residents/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## Bodies and life stages (12)

Proportions and heights: `crates/tools/src/characters/rig.rs` (`STAGES`, `proportions`, `target_height`). Rubric M14.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `male` baby | M-residents-01 | OK | n/a | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play/crib-lie-*.png` |
| `male` child | M-residents-01 | A-residents-09, A-residents-11 | I-kids-03 | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip; I-kids-03 S1 code (systems ledger) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `sheets/seat-sofaM.png`, `play-child/*.png` |
| `male` teen | M-residents-01 | A-residents-09, A-residents-11 | I-kids-03 | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip; I-kids-03 S1 code (systems ledger) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play-teen/*.png` |
| `male` youngAdult | M-residents-01 | OK | OK | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png` |
| `male` adult | M-residents-01 | OK | OK | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play/*.png` |
| `male` elder | M-residents-01 | OK | OK | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `residents/chars-elder-side.png`, `poses/seat-sofaF-elder.png`, `play-elder/*.png` |
| `female` baby | M-residents-01 | OK | n/a | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play/crib-lie-*.png` |
| `female` child | M-residents-01 | A-residents-09, A-residents-11 | I-kids-03 | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip; I-kids-03 S1 code (systems ledger) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `sheets/seat-sofaM.png`, `play-child/*.png` |
| `female` teen | M-residents-01 | A-residents-09, A-residents-11 | I-kids-03 | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip; I-kids-03 S1 code (systems ledger) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play-teen/*.png` |
| `female` youngAdult | M-residents-01 | OK | OK | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png` |
| `female` adult | M-residents-01 | OK | OK | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play/*.png` |
| `female` elder | M-residents-01 | A-residents-09, A-residents-11 | OK | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip (confirmed) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `residents/chars-elder-side.png`, `poses/seat-sofaF-elder.png`, `play-elder/*.png` |

## Build (2)

`appearance.build`, M16.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `build` 0.85 (slim) | OK | OK | n/a |  | `residents/builds-side.png`, `residents/chars-builds-walk.png` |
| `build` 1.2 (broad) | OK | OK | n/a |  | `residents/builds-side.png`, `residents/chars-builds-walk.png` |

## Hair (13)

`crates/tools/src/characters/hair.rs`. M15.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `hair.none` (bald) | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.short` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.crop` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.layered` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.tousled` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.sideSwept` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.afro` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.bob` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.long` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.ponytail` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.braid` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.bun` | OK | n/a | n/a |  | `residents/hair-female-front.png`, `residents/hair-male-front.png`, `residents/hair-female-side-talk.png`, `residents/hair-male-side-talk.png`, `residents/hair-female-back.png`, `residents/chars-hair-sit.png` |
| `hair.beard` (men; hidden for baby, child, teen) | OK | n/a | n/a |  | `residents/hair-male-front.png`, `residents/chars-hair-sit.png` |

## Clothes (24)

Painted garments and their shells: `crates/tools/src/characters/garments.rs`, `clothes.rs`; ids from `outfit.ts`. M15.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `top.tee` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `top.vneck` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `top.polo` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `top.long` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `top.shirt` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `top.jacket` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `top.suit` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `bottom.trousers` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `bottom.shorts` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `bottom.suit` (male) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png` |
| `top.tee` (female) | OK | n/a | n/a |  | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `top.tank` (female) | OK | n/a | n/a |  | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `top.blouse` (female) | OK | n/a | n/a |  | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `top.long` (female) | OK | n/a | n/a |  | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `top.shirt` (female) | OK | n/a | n/a |  | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `top.jacket` (female) | M-residents-02 | n/a | n/a | M-residents-02 S3 model | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `bottom.trousers` (female) | OK | n/a | n/a |  | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `bottom.shorts` (female) | OK | n/a | n/a |  | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `bottom.capri` (female) | OK | n/a | n/a |  | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `bottom.skirt` (female) | M-residents-02 | n/a | n/a | M-residents-02 S3 model | `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `shoes.sneakers` (both) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png`, `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `shoes.trainers` (both) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png`, `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `shoes.boots` (both) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png`, `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |
| `shoes.dress` (both) | OK | n/a | n/a |  | `residents/tops-male-front.png`, `residents/tops-male-sit-side.png`, `residents/tops-female-front.png`, `residents/tops-female-sit-side.png`, `residents/tops-female-walk-side.png` |

## Clips: idle.ts (8)

`web/src/render/babylon/characters/clips/idle.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `idle` | n/a | OK | OK |  | `residents/strip-idle.png` |
| Clip `talk` | n/a | OK | OK |  | `residents/strip-talk.png`, `play/mirror-speech-01.png` |
| Clip `foldArms` | n/a | OK | OK |  | `residents/strip-foldArms.png`, `play/tv-watch-01.png` |
| Clip `yes` | n/a | OK | n/a |  | `residents/strip-yes.png` |
| Clip `no` | n/a | OK | n/a |  | `residents/strip-no.png` |
| Clip `phone` | n/a | OK | n/a |  | `residents/strip-phone.png` |
| Clip `lantern` | n/a | OK | I-living-03 | I-living-03 S2 content (living ledger: read standing) | `residents/strip-lantern.png`, `play/bookshelf-read-01.png` |
| Clip `wave` | n/a | OK | n/a |  | `residents/strip-wave.png` |

## Clips: locomotion.ts (3)

`web/src/render/babylon/characters/clips/locomotion.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `walk` | n/a | A-residents-03, A-residents-04 | OK | A-residents-03 S3 clip (seeded; speed 3+ not run in the pilot); A-residents-04 S3 code (seeded; not run) | `residents/strip-walk.png`, `scenes/stairs-*.png` |
| Clip `jog` | n/a | A-residents-10 | OK | A-residents-10 S3 clip (confirmed by check:clips) | `residents/strip-jog.png`, `play/treadmill-run-01.png` |
| Clip `jogAlong` | n/a | A-residents-02, A-residents-03, A-residents-06 | n/a | A-residents-02 S3 clip (seeded; confirmed: seam known); A-residents-03 S3 clip (seeded); A-residents-06 S3 code (seeded) | `residents/strip-jogAlong.png` |

## Clips: seated.ts (9)

`web/src/render/babylon/characters/clips/seated.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `sit` | n/a | A-residents-09 | OK | A-residents-09 S2 clip (confirmed) | `residents/strip-sit.png`, `sheets/seat-*.png` |
| Clip `sitTalk` | n/a | OK | n/a |  | `residents/strip-sitTalk.png` |
| Clip `sitDown` | n/a | A-residents-05 | OK | A-residents-05 S3 code (seeded; hidden tab not run) | `residents/strip-sitDown.png` |
| Clip `standUp` | n/a | A-residents-01 | OK | A-residents-01 S2 code (seeded; the glide not timed in the pilot) | `residents/strip-standUp.png` |
| Clip `sitHands` | n/a | OK | OK |  | `residents/strip-sitHands.png`, `play/computerDesk-code-01.png`, `play/piano-play-01.png` |
| Clip `sitHandsNear` | n/a | OK | n/a |  | `residents/strip-sitHandsNear.png` |
| Clip `sitHandsFar` | n/a | OK | n/a |  | `residents/strip-sitHandsFar.png` |
| Clip `eat` | n/a | OK | I-held-01 | I-held-01 S2 code (held ledger: no cutlery) | `residents/strip-eat.png`, `play/fridge-snack-02.png` |
| Clip `drink` | n/a | OK | I-held-01 | I-held-01 S2 code (held ledger: no cup) | `residents/strip-drink.png`, `scenes/dining-coffee-02.png` |

## Clips: work.ts (10)

`web/src/render/babylon/characters/clips/work.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `interact` | n/a | OK | OK |  | `residents/strip-interact.png`, `play/fridge-snack-01.png` |
| Clip `pickUp` | n/a | OK | OK |  | `residents/strip-pickUp.png` |
| Clip `spell` | n/a | OK | I-bathroom-02 | I-bathroom-02 S3 clip (residents ledger: the shower) | `residents/strip-spell.png`, `play/shower-shower-01.png` |
| Clip `chop` | n/a | OK | n/a |  | `residents/strip-chop.png` |
| Clip `stir` | n/a | OK | n/a |  | `residents/strip-stir.png` |
| Clip `flip` | n/a | OK | n/a |  | `residents/strip-flip.png` |
| Clip `carry` | n/a | OK | n/a |  | `residents/strip-carry.png` |
| Clip `carryOne` | n/a | OK | n/a |  | `residents/strip-carryOne.png` |
| Clip `rail` | n/a | OK | I-hobbies-02 | I-hobbies-02 S2 clip (hobbies ledger: the telescope) | `residents/strip-rail.png`, `play/telescope-stargaze-01.png` |
| Clip `push` | n/a | OK | n/a |  | `residents/strip-push.png` |

## Clips: garden.ts (6)

`web/src/render/babylon/characters/clips/garden.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `water` | n/a | OK | OK |  | `residents/strip-water.png`, `play/plant-water-01.png` |
| Clip `harvest` | n/a | OK | n/a |  | `residents/strip-harvest.png` |
| Clip `plant` | n/a | OK | n/a |  | `residents/strip-plant.png` |
| Clip `kneel` | n/a | OK | I-kids-04 | I-kids-04 S2 renderer-table (kids ledger: feeding the baby) | `residents/strip-kneel.png`, `play/flowerbed-tend-01.png`, `scenes/crib-adult-feed-02.png` |
| Clip `crouch` | n/a | OK | OK |  | `residents/strip-crouch.png`, `play/flowerbed-smell-01.png` |
| Clip `pick` | n/a | OK | n/a |  | `residents/strip-pick.png` |

## Clips: action.ts (4)

`web/src/render/babylon/characters/clips/action.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `dance` | n/a | OK | n/a |  | `residents/strip-dance.png` |
| Clip `punchJab` | n/a | OK | n/a |  | `residents/strip-punchJab.png` |
| Clip `punchCross` | n/a | OK | n/a |  | `residents/strip-punchCross.png` |
| Clip `hit` | n/a | OK | n/a |  | `residents/strip-hit.png` |

## Clips: lie.ts (1)

`web/src/render/babylon/characters/clips/lie.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `lie` | n/a | OK | OK | A-fitness-01 S2 clip (fitness ledger: the bench press) | `residents/strip-lie.png`, `play/bed-sleep-02.png`, `play/weightBench-lift-02.png` |

## Clips: baby.ts (2)

`web/src/render/babylon/characters/clips/baby.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `babyLieAsleep` | n/a | OK | OK |  | `residents/strip-babyLieAsleep.png`, `sheets/bedpose-6.png` |
| Clip `babyLieAwake` | n/a | OK | OK |  | `residents/strip-babyLieAwake.png`, `sheets/crib-care.png` |

## Clips: street.ts (11)

`web/src/render/babylon/characters/clips/street.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `fountainDrink181` | n/a | OK | OK | One of five heights for the drinking fountain (`STEP_IN.drinkingFountain`); `check:clips` clean at every stage; not shot (an adult plays `fountainDrink163`) | |
| Clip `fountainDrink163` | n/a | OK | OK | The adult bends to the spout, hands on the bowl | `residents/strip-fountainDrink163.png`, `community/play/drinkingFountain-drink-01.png` |
| Clip `fountainDrink147` | n/a | OK | OK | As `fountainDrink181` | |
| Clip `fountainDrink129` | n/a | OK | OK | As `fountainDrink181` | |
| Clip `fountainDrink112` | n/a | OK | OK | As `fountainDrink181` | |
| Clip `climb172a` | n/a | A-residents-10 | OK | A-residents-10 S3 clip (confirmed by check:clips, elder woman) | `community/play/climbingFrame-climb-01.png` |
| Clip `climb147a` | n/a | A-residents-10 | OK | A-residents-10 S3 clip (confirmed by check:clips for climb112a/b, climb172a); up the frame's net, hands and feet on it | `residents/strip-climb147a.png`, `community/play/climbingFrame-climb-01.png` |
| Clip `climb112a` | n/a | A-residents-10 | OK | A-residents-10 S3 clip (confirmed by check:clips) | `community/sheets/age-climbingFrame.png` |
| Clip `climb172b` | n/a | OK | OK | The second lane of `climb172a`; `check:clips` clean | `community/sheets/age-climbingFrame.png` |
| Clip `climb147b` | n/a | OK | OK | The second lane of `climb147a`; `check:clips` clean | `community/sheets/age-climbingFrame.png` |
| Clip `climb112b` | n/a | A-residents-10 | OK | A-residents-10 S3 clip (confirmed by check:clips) | `community/sheets/age-climbingFrame.png` |

## Actions (45)

Content `anim` → clip, hands and layers: `ACTIONS` in `web/src/render/babylon/characters.ts`. Rubric I3; the object ledgers judge each use, this row the entry itself.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `ACTIONS.eat` | n/a | OK | I-held-01 | I-held-01 S2 code (no cutlery) | `play/fridge-snack-02.png` |
| `ACTIONS.drink` | n/a | OK | I-held-01 | I-held-01 S2 code (no cup) | `scenes/dining-coffee-02.png` |
| `ACTIONS.cook` | n/a | OK | OK |  | `play/fridge-meal-01.png` |
| `ACTIONS.wash` | n/a | OK | OK |  | `play/sink-wash-01.png` |
| `ACTIONS.shower` | n/a | OK | I-bathroom-02 | I-bathroom-02 S3 clip | `play/shower-shower-01.png` |
| `ACTIONS.bath` | n/a | OK | A-bathroom-04 | A-bathroom-04 S3 renderer-table (bathroom ledger: bathers 10–12 cm above the tub floor); I-bathroom-03 S2 code (residents ledger: in clothes, also in the hot tubs). The hot tub, whirlpool, cold plunge and heart tub sit low in the water, hands on the rim | `bathroom/play/chunk1/bathtub-bath-01.png`, `systems/sheets/act-write-garden-bath.png` |
| `ACTIONS.toilet` | n/a | OK | OK |  | `play/toilet-use-01.png` |
| `ACTIONS.sleep` | n/a | OK | OK |  | `play/bed-sleep-02.png` |
| `ACTIONS.nap` | n/a | OK | OK |  | `play/bed-nap-01.png` |
| `ACTIONS.sit` | n/a | A-residents-09 | OK | A-residents-09 S2 clip | `play/armchair-sit-01.png`, `play/bench-sit-01.png` |
| `ACTIONS.relax` | n/a | OK | I-garden-02 | I-garden-02 S2 clip (residents ledger: standing in the shade) | `play/sofa-relax-01.png`, `play/tree-shade-01.png` |
| `ACTIONS.type` | n/a | OK | OK |  | `play/computerDesk-code-01.png` |
| `ACTIONS.write` | n/a | OK | I-kids-03 | Seated at the desk, the hands on the paper (`sitHands`, `sitHandsNear` at the built-in-chair desks); posting a letter standing at the post box. An adult at the kids' desk: I-kids-03 S1 code (systems ledger) | `systems/sheets/act-write-garden-bath.png` |
| `ACTIONS.read` | n/a | OK | I-living-03 | I-living-03 S2 content; I-held-01 S2 code (no book) | `play/bookshelf-read-01.png` |
| `ACTIONS.watch` | n/a | OK | I-living-03 | I-living-03 S2 content (standing when no seat is near) | `play/tv-watch-01.png` |
| `ACTIONS.play` | n/a | OK | OK |  | `play/chessTable-chess-01.png`, `play/crib-play-01.png` |
| `ACTIONS.music` | n/a | OK | OK |  | `play/piano-play-01.png` |
| `ACTIONS.paint` | n/a | OK | I-held-01 | I-held-01 S2 code (no brush) | `play/easel-paint-01.png` |
| `ACTIONS.exercise` | n/a | I-residents-01 | I-residents-01 | I-residents-01 S2 clip (the bouldering wall is "climbed" standing on its mat; the fitness game played off its board). The exercise bike, spin bike, climbing frame (A-residents-10), hoops, trampoline, heavy bag, vibe plate and stretch fit | `systems/sheets/act-exercise.png` |
| `ACTIONS.run` | n/a | OK | OK | A-fitness-02 S3 motion (fitness ledger: the belt) | `play/treadmill-run-01.png` |
| `ACTIONS.lift` | n/a | A-fitness-01 | OK | A-fitness-01 S2 clip (fitness ledger) | `play/weightBench-lift-02.png` |
| `ACTIONS.dance` | n/a | OK | OK | The `dance` clip at the stereo, jukebox, dance floor and gramophone (the gramophone's "waltz" is danced alone: the packs ledger's use) | `systems/sheets/act-dance-sing-listen.png` |
| `ACTIONS.talk` | n/a | OK | OK |  | `play/mirror-speech-01.png` |
| `ACTIONS.sing` | n/a | OK | I-residents-02 | I-residents-02 S3 renderer-table (a mimed hand microphone beside the karaoke's own mic stand) | `systems/sheets/act-dance-sing-listen.png` |
| `ACTIONS.clean` | n/a | OK | I-kids-05 | I-kids-05 S2 code (kids ledger: changing over the cot rail) | `scenes/crib-elder-change-02.png` |
| `ACTIONS.water` | n/a | OK | I-held-01 | I-held-01 S2 code (no can) | `play/plant-water-01.png` |
| `ACTIONS.tend` | n/a | OK | I-kids-04 | I-kids-04 S2 renderer-table (kids ledger: the feed kneels) | `play/flowerbed-tend-01.png`, `scenes/crib-adult-feed-02.png` |
| `ACTIONS.cuddle` | n/a | OK | OK |  | `scenes/crib-adult-cuddle-03.png` |
| `ACTIONS.harvest` | n/a | OK | I-garden-03 | I-garden-03 S2 code and I-garden-04 S2 renderer-table (residents ledger: the crop goes at the start; the hand half a metre over low crops); the herb planter's `pick` fits. The greenhouse is harvested from outside its door (outdoor ledger's object) | `garden/extra/harvest/s-garden.strawberries-harvest.png`, `systems/sheets/act-write-garden-bath.png` |
| `ACTIONS.prune` | n/a | OK | I-held-01 | I-held-01 S2 code (no shears) | `play/bush-trim-01.png` |
| `ACTIONS.garden` | n/a | OK | I-residents-03 | Feeding the fish fits (`water` at the pond's rim); I-residents-03 S3 renderer-table (the songbird feeder refilled at knee height, 1 m under it) | `systems/sheets/act-write-garden-bath.png` |
| `ACTIONS.tinker` | n/a | OK | OK |  | `play/workbench-tinker-01.png` |
| `ACTIONS.look` | n/a | OK | I-hobbies-02 | I-hobbies-02 S2 clip (hobbies ledger: the telescope) | `play/telescope-stargaze-01.png` |
| `ACTIONS.smell` | n/a | OK | OK |  | `play/flowerbed-smell-01.png` |
| `ACTIONS.admire` | n/a | OK | I-residents-03 | I-residents-03 S3 renderer-table (the statue admired with the nose to its plinth); I-garden-02 S2 clip (residents ledger: the trees) | `systems/sheets/act-phone-meditate-idle.png`, `garden/play/c1/garden.cherry-admire-01.png` |
| `ACTIONS.phone` | n/a | OK | I-social-04 | I-social-04 S2 clip (social ledger: the hand at the ear for every phone use, a video call at a laptop too); I-social-01 S2 code (social ledger: no phone). The phone booth fits (inside, at its receiver) | `systems/sheets/act-phone-meditate-idle.png`, `social/sheets/phone.png` |
| `ACTIONS.listen` | n/a | OK | I-living-03 | Standing with the arms folded for 30–60 minutes at the stereo, the boombox, the record cabinet and the gramophone: I-living-03 S2 content (living ledger: no `seatNear`, widen there) | `systems/sheets/act-dance-sing-listen.png` |
| `ACTIONS.meditate` | n/a | OK | OK | Cross-legged on the yoga mat and the cool-down cushion | `systems/sheets/act-phone-meditate-idle.png` |
| `ACTIONS.idle` | n/a | OK | I-residents-03 | Fits at the shop counter (served across it); I-residents-03 S3 renderer-table (the giving jar's donation made with the hands at the sides) | `systems/sheets/act-phone-meditate-idle.png` |
| `ACTIONS.knock` | n/a | OK | I-systems-09 | Visitors at the front door (`visits.door.knockAnim`): I-systems-09 S3 code (systems ledger) | `systems/sheets/knock.png` |
| `ACTIONS.fetch` | n/a | OK | OK |  | `play/fridge-snack-01.png` |
| `ACTIONS.chop` | n/a | OK | I-systems-01 | The board drawn on the counter and chopped on (`systems/sheets/cook-meal-1.png`); I-systems-01 S2 renderer-table (systems ledger: on the gas grill station, the grate) | `systems/sheets/cook-meal-1.png`, `outdoor/play/chunk5-kitchen/grillStation-cook-01.png` |
| `ACTIONS.stir` | n/a | OK | I-systems-06 | Fits on a hob; I-systems-06 S2 renderer-table (systems ledger: every oven step is stirring a pan); I-systems-07 S3 code (the prep bowl in the other hand) | `systems/sheets/cook-meal-1.png`, `systems/sheets/cook-ovens.png` |
| `ACTIONS.flip` | n/a | OK | I-systems-07 | Turning the food on the grill fits; I-systems-07 S3 code (systems ledger: the prep bowl in the other hand) | `systems/sheets/cook-bbq-1.png` |
| `ACTIONS.serve` | n/a | n/a | n/a | Nothing plays it: no interaction, recipe step or cooking rule names `serve` (a dish is put down at the end of the last step, under its clip) | |

## Procedural layers (11)

Code in `web/src/render/babylon/characters.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Face: expressions (moods) and blinks | OK | A-residents-07 | n/a | A-residents-07 S2 code (seeded; confirmed: the face carries every mood, the body none) | `residents/face-female.png`, `residents/chars-face.png` |
| Face: lip sync (visemes) | OK | OK | n/a |  | `residents/chars-visemes.png` |
| Eyes: looking at people and things | OK | OK | n/a |  | `play/tv-watch-01.png`, `play/easel-paint-01.png` |
| Elder's stoop | OK | OK | n/a |  | `residents/chars-elder-side.png`, `residents/chars-stages-side.png` |
| Arms a little out over a skirt | OK | OK | n/a |  | `residents/tops-female-front.png` |
| Hands: open or gripping per activity | n/a | OK | OK |  | `play/computerDesk-code-01.png`, `play/weightBench-lift-02.png` |
| Seat fit: pelvis `seatDrop` over the seat (`SEATS`, content `seat`) | n/a | A-residents-09, A-residents-11 | I-kids-03 | A-residents-09 S2 clip; A-residents-11 S2 clip (widened: the child in the bean bag); M-residents-02 S3 model (widened: every seated woman's skirt a hoop round the seat); M-outdoor-02 S3 model (outdoor ledger: the bench 4 cm into its slats); I-kids-03 S1 code (systems ledger: teens, adults and elders sit on the 0.31 m kids' chair as on any). Cross-cutting wave, over the other waves' age sheets (`<cat>/sheets/age-*.png`: 253, of which the dining chairs, bar stools, sofas, armchairs, bean bags, office chairs, kids' chair, fountain rim read): the pelvis sits on the seat at every stage and every height of seat; a child's feet dangle on adult chairs and bar stools (as a child's do), a teen's and an elder woman's hang a few cm off a 0.45 m seat (A-residents-09); deep low seats swallow short shins | `kitchen/sheets/age-diningChair.png`, `kitchen/sheets/age-barStool.png`, `living/sheets/age-beanBag.png`, `kids/sheets/age-kidsChair.png` |
| Lie fit: `lie` turned flat on the mattress (`BEDS`, `LIE_*`) | n/a | A-fitness-01, A-bedroom-03 | OK | A-fitness-01 S2 clip (fitness ledger); I-kids-03 S1 code (systems ledger: an adult in the cot); A-bedroom-03 S2 code (residents ledger: into bed at the foot end, at every stage, the bunk's elder too); A-bedroom-02 S1 code (bedroom ledger: the upper bunk). Cross-cutting wave, over the bedroom, kids, living and packs age sheets: the sleeper lies on the mattress at every stage on every bed; `poses.html` puts only a child in the kids' bed (sim-core's stage rules for free will; a direct use skips them, I-kids-03) | `bedroom/sheets/age-bedBunk.png`, `kids/sheets/age-kidsBed.png`, `play/bed-sleep-02.png` |
| Blankets and pyjamas in bed | M-bedroom-04 | A-bedroom-04 | I-bedroom-01 | M-bedroom-04 S2 code (one blue checked quilt on every bed); A-bedroom-04 S2 code (a stray quilt at the world origin); I-bedroom-01 S2 code (pyjamas for every nap, the shoes on); all three filed by the bedroom wave, moved here at the consolidation | `play/bed-sleep-02.png`, `sheets/bedpose-0.png` |
| Turning: towards travel, and on the spot | n/a | A-residents-08 | n/a | A-residents-08 S3 code (seeded; seen: residents turn in place at each use) | `play/*-00.png` |
| Approach: stepping to a use and back | n/a | OK | OK | A-bathroom-01 S2 model (bathroom ledger: through the shower door) | `play/*-00.png`, `play/shower-shower-00.png` |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-residents-01 — Getting up from a chair glides back to the tile

- **Severity:** S2
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2755` (`standUp`), `crates/sim-core/src/world.rs`
- **Evidence:** The sim puts the resident on the tile in front at once, so `standUp` plays while they glide back to it. Deferred from 0.3.0 unless the stand-up is reworked.
- **Status:** seeded (from `docs/objects/open.md` L124); not checked in the pilot (the shots are not timed to the stand-up)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred)

### A-residents-02 — `check_clips.ts` fails `jogAlong`: a loop seam and sliding feet

> Phase 0: the sliding feet were the checker testing a moving clip as if standing; with the ground-speed check they pass. The loop seam (calf_r 29.1°, foot_r 21.7°) is real.

- **Severity:** S3
- **Rubric:** A1, A3
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/locomotion.ts:208`, `tools/characters/check_clips.ts`
- **Evidence:** Fails since before fbf8883. The plan takes it as the baseline failure for T5.
- **Status:** confirmed (pilot: `pnpm check:clips` prints the seam on `calf_r` 29.1° and `foot_r` 21.7° as known, every body and stage)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/authenticity-review.md` §5.1 (T5)

### A-residents-03 — Feet still slide at game speed 3 and above

- **Severity:** S3
- **Rubric:** A1, A3
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:136` (`GAIT_MAX_RATE`), `web/src/render/babylon/characters/clips/locomotion.ts:208`
- **Evidence:** After R1-05's jog switch (76ac264), foot slide at height 1.0 with no traits: adults at speed 3 50–53 %, speeds 4–5 83–95 %. Children walk at the adults' 1.5 m/s on short legs.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §3.1 R1-05); not checked in the pilot (its play runs at speed 1 near the object)
- **Source:** `docs/release-0.2.0-review.md` §2.1 and §3.1 (R1-05)

### A-residents-04 — The start and end of a walk jump at speeds 4–5

- **Severity:** S3
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (walker interpolation)
- **Evidence:** R1-04's rest: a walker standing in one snapshot wasn't interpolated (jumps up to 4.6 m at speed 5). CHANGELOG 0.3.0 (Fixes) says "Walks no longer jump at the start and the end": confirm and close.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 R1-04 (rest)); not checked in the pilot
- **Source:** `docs/release-0.2.0-review.md` §4 (R1-04 rest); `plan/release-0.3.0.md` §3.2; `CHANGELOG.md` 0.3.0 Fixes

### A-residents-05 — After a hidden tab, someone who sat down meanwhile replays the sit-down

- **Severity:** S3
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (a rig drawn again)
- **Evidence:** R1-12's fix covers residents coming back into view, not a hidden tab: they count as on screen while it is hidden. It ends on its own.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 R1-12 (hidden tab)); not checked in the pilot (headless pages are never hidden)
- **Source:** `docs/release-0.2.0-review.md` §4 (R1-12 hidden tab, new), target 0.2.1

### A-residents-06 — Walkers jog on the spot for about 0.25 s after a hidden tab

- **Severity:** S3
- **Rubric:** A3, A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (walking speed estimate)
- **Evidence:** Speed is estimated from the distance covered over the capped 0.1 s frame (8.7–22 m → 13.2 m/s), past the jog switch, then fades back over about 0.3 s. Seen in all four hidden-tab runs.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 "Jogging after a hidden tab"); not checked in the pilot
- **Source:** `docs/release-0.2.0-review.md` §4 (new, R1), target 0.2.1

### A-residents-07 — Body language doesn't follow feelings; only the face does

- **Severity:** S2
- **Rubric:** A6
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:684` (face moods)
- **Evidence:** A known gap named in the rubric. Emotional body language is new behaviour: filed as `code`, planned separately (§9, decision 3).
- **Status:** confirmed (pilot: `residents/face-female.png` has 18 distinct moods; no clip or layer changes with them)
- **Source:** `plan/authenticity-review.md` §3.2 (A6) and §9 (decision 3)

### A-residents-08 — No turn-on-the-spot clip

- **Severity:** S3
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters/clips/locomotion.ts`, `web/src/render/babylon/characters.ts` (turning)
- **Evidence:** Residents turn in place without stepping. A turn-on-the-spot clip is new behaviour: filed only, planned separately (§9, decision 3).
- **Status:** confirmed (pilot: residents turn in place to face each object, no steps, in every `play/*-00.png`)
- **Source:** `plan/authenticity-review.md` §3.2 (A4) and §9 (decision 3)

### A-residents-09 — Seated feet don't reach the floor for children, teens and elder women

> Consolidation: widened by the object waves: `pnpm check:clips` still lists the teen and elder women's feet off a 0.45 m seat (kitchen chairs and sets); the child's feet hang on `rockingChair` and `pouf` (living) and on the park bench, the bus shelter and the café chair (community).

- **Severity:** S2
- **Rubric:** A1, A5
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/` (sit), `web/src/render/babylon/characters.ts` (seat fit per stage)
- **Evidence:** `pnpm check:clips` (`tools/characters/clip-baseline.json`): a child on the 0.31 m kids' chair has the feet 7 cm (male) and 8 cm (female) off the floor; a teen on a 0.45 m seat 7 cm and 9 cm; an elder woman on a 0.45 m seat 5 cm. Adults pass.
- **Status:** confirmed (pilot: `check:clips` as above; in shots, a child on the sofa, armchair and bench has the feet off the floor, `sheets/seat-sofaM.png`, `sheets/seat-benchM.png`; the teen and the elder woman less so)
- **Source:** `tools/characters/check_clips.ts --body all --stage all`

### A-residents-10 — A child's feet slide in `jog` and on the climbing frame

> Consolidation: the community wave's `check:clips` run still prints the child's 1.3–1.9 cm and the elder woman's 1.1 cm foot slide in the climbing frame's rounds (`climb112a`, `climb112b`, `climb172a`).

- **Severity:** S3
- **Rubric:** A1, A5
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/` (jog, climb)
- **Evidence:** `pnpm check:clips`: a child's right foot slides 1.5 cm in `jog` (both bodies); the left foot slides 1.3–1.9 cm in `climb112a`/`climb112b`, and an elder woman's 1.1 cm in `climb172a`.
- **Status:** confirmed (pilot: `check:clips` prints all five as known)
- **Source:** `tools/characters/check_clips.ts --body all --stage all`

### A-residents-11 — An elder woman, a teen or a child on the sofa loses the shins

> Consolidation: widened by the living wave: the child "kneels" on most living seats, the shins down inside the cushion and the shoes through or under the seat front (`loveseat`, `sofaThree`, `sofaDesign`, `sofaThreeSeat`, `sofaBed`, the four sectional modules, `armchairLounge`, `armchairLeather`, `armchairWingback`, `recliner`, `ottoman`; `living/sheets/age-armchairWingback.png`, `age-sectionalMiddle.png`, `age-ottoman.png`), not on `pouf` or `rockingChair`. Teens and elders lose the shins on `sofaThree`, `sofaDesign`, `loveseat` and `armchairLounge`, and so do the adult man on the loveseat and the adult woman on `sofaThree` (`living/sheets/age-loveseat.png`, `age-sofaThree.png`).

- **Severity:** S2
- **Rubric:** A1, A5
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts` (seat fit per stage), sofa seat in `web/src/render/babylon/characters/furniture.json`
- **Evidence:** `poses.html?stage=elder`: the woman on the TV sofa has no shins and her boots stand apart on the floor; the adult version is fine. Shots: `poses-tvsofa-elder.png` against `poses-tvsofa-default.png` (Phase 0 gallery agent's scratch folder).
- **Status:** confirmed and widened (pilot: `poses/seat-sofaF-elder.png`: the shins end in the cushion and the boots stand on the floor in front of the sofa, apart from the legs. The same for the child and the teen on the sofa, `sheets/seat-sofaM.png` top right and bottom left: their knees stop short of the 0.8 m deep seat's front, so the shins go down inside it. The adults are fine. A shorter seat depth per stage (sit further forward) or a legs-out pose for short thighs)
- **Source:** Phase 0.7b check of `poses.html?stage=`; wave: cross-cutting (widened: `living/sheets/age-beanBag.png`, the child sits in the bean bag with only the shoes showing past its front, the shins inside it; teen, adult and elder sit upright on it, the shins in it to mid-calf)

### M-residents-01 — Heads are about half a head bigger than the rubric's ratios at every stage

- **Severity:** S3
- **Rubric:** M14
- **Fix class:** decision
- **Where:** `crates/tools/src/characters/rig.rs:129` (`proportions`: `hh` 1/3.4 baby, 1/4.6 child, 1/6.0 teen, 1/6.5 young adult and adult, 1/6.4 elder); `web/public/content/base.json:3657` (`life.stages`: `scale`, `head`)
- **Evidence:** `residents/chars-stages.png`, `residents/chars-stages-side.png`. The rubric asks baby about 1:4, child 1:5–6, teen 1:6.5–7, adult 1:7–7.5; the rig has 1:3.4, 1:4.6, 1:6.0, 1:6.5. The order and the steps between stages are right, and heights are real (0.67, 1.13/1.09, 1.51/1.44, 1.77/1.67, 1.76/1.66, 1.74/1.64 m against `target_height`, `rig.rs:80`); the bigger heads, like the 1.15× hands and feet (`rig.rs:174`), are the stylised look (`docs/design/look.md`). Every stage reads as its age at the game camera. Filed only (§9 decision 5): either the owner accepts the style and M14's ranges move half a head (proposed in `pilot-notes.md`), or `rig.rs` changes, which moves every seat and bed fit.
- **Status:** closed (the owner approved M14's ratios based on the style, half a head bigger, after the pilot; any `rig.rs` change stays filed only, §9 decision 5)
- **Source:** pilot

### M-residents-02 — Seated, the skirt and the women's jacket keep their standing flare

- **Severity:** S3
- **Rubric:** M15
- **Fix class:** model
- **Where:** `crates/tools/src/characters/clothes.rs:10` (`bottom.skirt`: an A-line loft from the waistband to above the knees, swinging with the thighs); `crates/tools/src/characters/garments.rs` (the jacket's hem)
- **Evidence:** `residents/tops-female-sit-side.png`: seated, the skirt stands off the thighs as a stiff bell, its hem in the air past the knees, and the jacket's hem flares out over the lap the same way; nothing clips. A skirt seated lies on the thighs and drapes off the knees. Visible at the close camera only.
- **Status:** confirmed and widened (cross-cutting wave: every seated woman at teen, adult and elder in a skirt, on every seat of the age sheets, shows the skirt as a dark hoop standing out round the seat at the game camera's height, e.g. `kitchen/sheets/age-diningChair.png`, `kitchen/sheets/age-barStool.png`; the hem doesn't clip)
- **Source:** pilot; wave: cross-cutting

### I-residents-01 — The bouldering wall is "climbed" standing on its mat, and the fitness game is played off its board

> Consolidation (2026-10-21): closed as a duplicate of I-packs-03 (packs ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:481-495` (`ACTIONS.exercise`: `boulder` and `sunrise` get `idle` with `activityPose` on top; `fitnessGame` `dance`, `punchJab`, `punchCross` from the tile in front), `:198` (`STEP_IN.climbingFrame`, the climbing clips the frame already has)
- **Evidence:** `systems/sheets/act-exercise.png` (the packs wave's `packs/play/chunk2/energetic.boulderingWall-climb-01.png`): for the 60 minutes of "Climb" the resident stands on the crash mat at the wall's foot with both arms raised, the feet on the floor, never on a hold; the climbing frame beside it climbs its net (`community/play/climbingFrame-climb-01.png`, `climb*` clips at five heights). The fitness game (`packs/play/chunk1/couchPotato.fitnessGame-workout-01.png`) is danced and punched standing beside its balance board, not on it. The exercise bike, spin bike, hoops, trampoline, heavy bag, vibe plate and the sunrise lamp's stretch fit their objects.
- **Status:** closed (duplicate of I-packs-03)
- **Source:** wave: cross-cutting (ACTIONS entries no base object uses, judged at the packs wave's objects)

### I-residents-02 — Singing mimes a hand microphone beside the karaoke machine's own microphone stand

- **Severity:** S3
- **Rubric:** I3
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:501` (`ACTIONS.sing`: `talk` with the `phone` clip's hand at the mouth as a microphone)
- **Evidence:** `systems/sheets/act-dance-sing-listen.png` (`packs/play/chunk3/cheerful.karaokeMachine-sing-01.png`, `packs/play/chunk4/outgoing.karaokeStage-perform-01.png`): at the karaoke machine the singer stands to one side of its mic stand, an empty hand held to the mouth; on the karaoke stage they stand at the front edge, the two mic stands behind them. A singer at a stand sings into it, hands on the stand or free; a karaoke singer holds the machine's microphone (a prop: I-held-01's family).
- **Status:** fixed (0c5e889e)
- **After:** `STEP_IN` steps the singer up to the karaoke machine's mic stand (lane 0.18) and onto the stage to one of its two stands (`up` 0.146), the hands free (`ACTIONS.sing` without the dead seated `phone` layer): `fixes/tables/play-a/a-cheerful.karaokeMachine-sing-01.png`, `a-outgoing.karaokeStage-perform-01.png`. The machine's microphone in the hand is the props wave's.
- **Source:** wave: cross-cutting

### I-residents-03 — Standing uses with no reach to the part used: the bird feeder, the giving jar, the statue

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:514-517` (`ACTIONS.garden`: `water` for the feeder), `:529` (`admire`: `idle`, `foldArms`), `:533` (`idle`)
- **Evidence:** `systems/sheets/act-write-garden-bath.png`, `systems/sheets/act-phone-meditate-idle.png` (the packs and community waves' play shots): "Refill the feeder" is the watering clip at knee height beside the bird bath, a metre under the 1.5 m feeder box; "Donate" at the giving jar stands by it with the hands at the sides; "Admire the statue" stands with the face to its plinth, the statue 2 m over the head, never looked up at. The pond's "Feed the fish" and the shop counter's treat (served across it) fit. A reach to the part's height (`STEP_IN`-style sizes, or the `look` entry's head tilt) would fix each.
- **Status:** fixed (0c5e889e)
- **After:** The feeder: `STEP_IN` to its post (lane -0.2, clear of the bird bath) and `ACTIONS.garden` by def (`interact`, `spell`), the hands at the post at chest height, no longer at knee height by the bird bath (`fixes/tables/play-b/b-kind.songbirdFeeder-*.png`); the box at 1.5 m wants a reach-up clip. The giving jar: stepped in (`STAND_BACK` -0.4), a coin in (`interact`) (`play-a/a-kind.givingJar-donate-*.png`). The statue: admired from 0.8 m further back, looking up at it (`STAND_BACK` 0.8).
- **Source:** wave: cross-cutting

Moved here at the consolidation (2026-10-21) from the ledgers they were first seen in, ids unchanged: the blankets and pyjamas, getting into bed, the bath outfit, and the `ACTIONS` entries and clips used by objects in several ledgers. Pilot evidence in them stays relative to `bench-results/review/pilot/`, the waves' to `bench-results/review/`, as in the ledgers they came from.

### I-bedroom-01 — Pyjamas for every nap, a doze in the garden too, and the shoes stay on

> Consolidation: I-outdoor-02 (outdoor ledger) merged into it: the sun lounger's and the hammock's doze in pink pyjamas, the elder's too (`outdoor/sheets/obj-sunLounger.png`, `outdoor/sheets/age-sunLounger.png`, `outdoor/sheets/obj-hammock.png`).
>
> Moved from bedroom.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I3, I16
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2373-2378` (`asleep`: lying in a `LYING` clip with tag `sleep` or `nap` → the night outfit); `web/src/render/babylon/characters/outfit.ts:149-154` (pyjamas recolour the top and bottom only; the shoe part stays)
- **Evidence:** `bedroom/play/chunk1/bed-nap-01.png` and every bed's nap: the resident changes into pyjamas for a 30-minute nap at noon; `bedroom/extra/lounger-doze.png`: "Doze in the sun" on a sun lounger in the garden at 10:22 is in pink pyjamas, white shoes on. `bedroom/play/stage-elder/bedSingle-sleep-01.png`: the shoes go into bed (on the mattress at the foot while getting in). People nap in their clothes, take their shoes off for bed and dress for the night only at night. Pyjamas for `sleep` only, and no shoes with them, would do. Other items: every lying `nap` use: `kidsBed`, `kidsBedCar` (kids), `sofaBed`, `wallBed` (living), `sunLounger`, `hammock` (outdoor), `lazy.snoozecloudBed`, `cheerful.rainbowHammock`, `gloomy.windowSeat`, `natureLover.gardenHammock`, `kind.hearthSettee` (packs) (`tables/uses-*.md`). Cause in the renderer: for `residents.md` (row "Blankets and pyjamas in bed").
- **Status:** confirmed and widened (consolidation: I-outdoor-02 merged)
- **Source:** wave: bedroom (lead from Phase 0.9 "Seen (not reviewed)"; `bedroom/extra/scenes.mjs` phase `lounger`)

### M-bedroom-04 — Every sleeper gets the same blue checked quilt, whatever the bed's own bedding

> Moved from bedroom.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** M9, M13, A12
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters/blanket.ts:197-246` (`blanketMaterial`: one texture, `#6F97A6` with a white sheet band); `web/src/render/babylon/characters.ts:1176` (one `quilt` material for every quilt), `:280` (`BLANKET_BEDS`)
- **Evidence:** `bedroom/play/chunk1/bedSingle-sleep-02.png` (a blue checked quilt over the sage single), `bedroom/play/chunk2/bedCanopy-sleep-02.png` (over the rust four-poster), `bedroom/play/chunk1/bedDouble-sleep-02.png`, `bedroom/extra/bunk-two-09.png` (teal bunks). Asleep, the bed's own duvet, the part the buyer colours (`Bedding`), is hidden under the game's quilt, which is the same blue check on every bed and every colour; it appears as the sleeper lies down and goes when they get up, so the bed changes colour twice a night. Tinting the quilt from the bed's `Bedding` colour (or its pattern by bed) would keep the bed the player chose. Other items (other ledgers): `kidsBed`, `kidsBedCar` (kids), `wallBed` (living), `lazy.snoozecloudBed` (packs): every bed in `BLANKET_BEDS`. Cause in the renderer: for `residents.md` (row "Blankets and pyjamas in bed").
- **Status:** open
- **Source:** wave: bedroom (lead from Phase 0.9 "Seen (not reviewed)")

### A-bedroom-04 — A stray quilt lies at the world origin whenever a bed with a blanket is on the lot

> Moved from bedroom.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** A12, M7
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:1489-1497` (every base body's quilt mesh `setEnabled(true)` once any object is in `BLANKET_BEDS`, with `thinInstanceCount = 0`), `:2458-2464` (`thinInstanceCount = drawn`, 0 when nobody is under that quilt); `web/src/render/babylon/characters/blanket.ts:181-188` (the mesh at the origin, `alwaysSelectAsActiveMesh`); Babylon draws a mesh with no thin instances as a plain mesh at its own transform (`Mesh.hasThinInstances` is `instancesCount > 0`, `@babylonjs/core` 9.30 `Meshes/mesh.pure.js:225`)
- **Evidence:** `bedroom/extra/quilt-origin-bed-awake.png`, `quilt-origin-bed-awake-far.png` (`review-stage`, a `bedSingle` bought, nobody in bed): a quilt lies on the lawn at the lot's corner, `(0, 0, 0)`, in view of the Live camera from the front garden. With no bed it isn't there (`quilt-origin-nobed.png`; the meshes disabled). With someone asleep it stays (`quilt-origin-asleep.png`): the sleeper's body's quilt is drawn on the bed, the other bodies' quilts (count 0) at the origin. `bedroom/extra/scenes.log` lists the seven `simBlanket:*` meshes, enabled, count 0, at the origin. The tooling agent's quilt on `poses.html` (`bedroom/sheets/age-bed.png`, right edge) is the same. Hiding a quilt mesh while its count is 0 (or `forcedInstanceCount`) fixes it. Other items: every bed in `BLANKET_BEDS` (`kidsBed`, `kidsBedCar`, `wallBed`, `lazy.snoozecloudBed` in other ledgers); where the origin falls on other lots (a street, a neighbour's lot) wasn't checked.
- **Status:** open
- **Source:** wave: bedroom (lead from the tooling round; `bedroom/extra/scenes.mjs` phase `quilt`)

### A-bedroom-03 — Residents get into bed at its foot end, through the footboard, and slide up to the pillow

> Moved from bedroom.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** A1, A4, I2
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:304` (`LIE_ENTRY` 0.55: the hips sit down 0.55 m forward of the slot's middle), `:2533-2541` (getting in: `sitDown` on the edge facing the tile they came from, then the crossfade to lying); the beds' `approach` (default `front`: the tile in front, the foot end) in `web/public/content/furniture.json` and `catalog2/bedroom.json`; getting up `web/src/render/babylon/characters.ts:2755-2807` (`standUp`, `fromBed`)
- **Evidence:** `bedroom/play/chunk1/bedSingle-sleep-00.png`: the resident sits down facing out, about 0.4 m in from the foot end of the single, the thighs through its 0.6 m footboard; `bedroom/play/stage-elder/bedSingle-sleep-01.png` (the elder, the same, the shoes on the mattress); `bedroom/play/chunk1/bedKing-sleep-01.png` (half way through the crossfade, half reclined in the middle of the bed); `bedroom/extra/bunk-two-02.png` (the bunk's end board). From there the body crossfades about 1.4 m up the bed to lie with the head on the pillow. People get in at a side, sit on the edge by the pillow and swing the legs in. On beds with a footboard or end arms (`bedSingle`, `bedBunk`, `bedDaybed`) the legs pass through it; on the rest (`bed`, `bedKing`, `bedDouble`, `bedQueenUpholstered`, `bedCanopy`, `bedFuton`) it is the sit at the foot and the slide. Other items: `kidsBed`, `kidsBedCar` (kids), `wallBed`, `sofaBed` (living), the packs' beds: every `lie` use on a bed. Cause in the renderer and the beds' approach: for `residents.md` (row "Lie fit").
- **Status:** open
- **Source:** wave: bedroom

### I-bathroom-03 — Residents shower in their clothes, behind clear glass

> Moved from bathroom.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** I3
- **Fix class:** decision
- **Where:** `web/src/render/babylon/characters/outfit.ts:120` (day outfit or pyjamas only); `tools/art/blender/families/bathroom.py:552-554` (clear `glass` panels)
- **Evidence:** `play/shower-shower-01.png`: the day outfit stays on under the water and shows through the enclosure. Bed has its pyjamas; the bath and shower have nothing. A towel or swimwear-like outfit, frosted glass, or leaving it as the style's convention is the owner's call.
  Bathroom wave: every shower and every bath. Dressed under the rain in the stall, the walk-in and the rain shower (glass at `bathroom.py:637`, `:668-669`), and lying dressed in the bath water in all three tubs at every life stage (`bathroom/play/chunk1/showerStall-shower-01.png`, `bathroom/play/chunk1/bathtub-bath-01.png`, `bathroom/play/stage-elder/bathtub-bath-01.png`). The same outfit rule covers the tubs and showers of the other ledgers that use `bath` or `shower` (`gloomy.soakingTub`, `romantic.heartTub`, `hotHeaded.coldPlunge`, `hotTubCedar`, `whirlpoolTub` and the three pack showers), where a swimsuit is the real thing for the hot tubs.
- **Status:** confirmed and widened (bathroom wave: all four showers and three baths; the other tubs named)
- **Source:** pilot; wave: bathroom

### I-bathroom-02 — The shower clip has no washing in it

> Moved from bathroom.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:466` (`ACTIONS.shower`: `spell`, `interact`, `idle`)
- **Evidence:** `play/shower-shower-01.png`, `play-child/shower-shower-01.png`: under the water the resident works the hands at chest height (`spell`) and stands; no hands to the hair, no turning under the head. Reads as waiting in the shower.
  Bathroom wave: the same at the stall, the walk-in and the rain shower (`bathroom/play/chunk1/showerStall-shower-01.png`, `showerWalkIn-shower-01.png`, `bathroom/play/chunk2/showerRain-shower-01.png`). The entry is shared with the packs' `energetic.contrastShower`, `natureLover.cedarShower` and `neat.rainShowerTower` (packs ledger).
- **Status:** confirmed and widened (bathroom wave: all four showers; the three pack showers named)
- **Source:** pilot; wave: bathroom

### I-kitchen-03 — Hands at the cabinet doors, not on the tap, the appliance or the shelf

> Moved from kitchen.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:460` (`ACTIONS.cook`: `spell, interact, spell, pickUp`), `:465` (`ACTIONS.wash`: `spell, interact`), `:538` (`ACTIONS.fetch`: `interact, pickUp, idle`); no per-object variant for these (as `look` has for the telescope)
- **Evidence:**
  - Sinks: washing hands or washing up at the farmhouse and double sinks, the hands work at hip height in front of the cabinet doors, 15 cm under the 0.95 m rim and 30 cm short of the basin, the water running into an empty bowl (`kitchen/play/c1/farmhouseSink-washUp-01.png`, `kitchen/play/c2/counterSinkDouble-wash-01.png`, `kitchen/extra/putaway-wallCabinet-04.png`). The base sink is the same (pilot `play/sink-wash-01.png`); the pilot passed `ACTIONS.wash` there, this wave doesn't.
  - Microwave, toaster, blender: "Heat up leftovers", "Make toast" and "Blend a smoothie" (`anim: cook`) are worked at the cabinet below while the microwave door (1.0–1.25 m) or the toaster's lever opens by itself on the top (`kitchen/play/c1/microwave-reheat-01.png`, `breakfastCounter-toast-01.png`); `docs/objects/kitchen.md` K2 already notes it for the microwave.
  - Putting clean dishes away (`fetch`) in a wall cabinet: its doors open at 1.44–2.2 m while the hands stay at the waist over the sink below (`kitchen/extra/putaway-wallCabinet-05.png`); the same for the three wall cabinets and the china cabinet's glazed top (K9 "Not right yet"); `pickUp` bends to the floor in it.
  Items: `farmhouseSink`, `counterSinkDouble`, `sink` (bathroom ledger), `microwave`, `breakfastCounter`, `wallCabinet`, `wallCabinetGlass`, `wallCabinetCorner`, `chinaCabinet`. A reach to the worktop (the `chop` clip's arms, without the board) for the sinks and appliances, and a reach up for the wall cupboards, by object def as `look` does, would fix all of them.
- **Status:** fixed (0c5e889e)
- **After:** Sinks: `STAND_BACK` -0.45 (farmhouse, double), -0.35 (base sink): the hands in the basin (`fixes/tables/play-a/a-farmhouseSink-washUp-01.png`). Microwave: `STEP_IN` to it on the counter's right, the hands at its door; breakfast counter stepped in; neither squats to the floor (`ACTIONS.cook` by def). Wall cupboards and the china cabinet: `fetch` without the squat (`WALL_CUPBOARDS`). A reach up to a wall cupboard's doors needs a clip (resident-clip wave).
- **Source:** wave: kitchen (play shots, `scratch/scene.mjs` put-away scene)

### I-garden-04 — Harvesting and tending don't follow the plant's height: the hand stops half a metre above low crops, and the tomatoes are tied up kneeling at the soil

> Consolidation: I-outdoor-04 (outdoor ledger: the greenhouse's staging and the compost bin, 0.7–0.8 m, tended kneeling) and I-packs-06 (packs ledger: the waist-high herb garden tended kneeling) merged into it. One cause: `ACTIONS.tend` kneels and `ACTIONS.harvest` picks at one height for every object; the cot's feed (I-kids-04) also plays the kneel.
>
> Moved from garden.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:508` (`ACTIONS.tend`: `kneel` for every crop) and `:512` (`ACTIONS.harvest`: `harvest`, `plant` for every def but the herb planter); `web/src/render/babylon/characters/clips/garden.ts:37-44` (`harvestPose`: standing, bent 22–32°, the picking hand at 0.68–0.80 m)
- **Evidence:**
  - Harvest: `extra/harvest/s-garden.strawberries-harvest.png` and `s-garden.pumpkins-harvest.png`: the resident stands bent a little with the hand at the knees, 0.4–0.5 m above berries at 0.07 m and pumpkins at 0.2 m; nothing is touched. At the veg patch (0.4 m, `s-garden.vegPatch-harvest.png`) the hand is at the box's rim; at the tomatoes (1.2 m) it is in the plant, which is right. The meadow's "Pick a wildflower" plays the same clip over a 0.5 m patch (`play/c3/garden.meadow-pickFlower-01.png`). Low crops want the crouch the `plant` clip already has (`garden.ts:110-135`), or a deeper harvest pose.
  - Tend: "Tie up and water" at the tomatoes is the weeding kneel (`play/c3/garden.tomatoes-tend-01.png`): right while planted, wrong at a ripe, staked 1.2 m plant, which is tied up standing.
  - The same entries play at the outdoor ledger's `raisedBed` (hand at its rim, fine) and the packs ledger's `natureLover.veggieBed` (`tendVeggies`).
- **Status:** fixed (425486ee)
- **After:** `ACTIONS.tend` and `harvest` by def: strawberries and pumpkins picked crouched at the soil (`plant`, `kneel`), the meadow's flower too; the staked tomatoes tied up and watered standing (`harvest`, `water`); the greenhouse staging, the compost bin and the herb trough tended standing, leaning in (`pick`); the raised beds bent over (`harvest`). `fixes/tables/play-a/a-compostBin-compost-01.png`, `a-foodie.herbGarden-tendHerbs-01.png`, `play-b/b-foodie.herbGarden-*.png`, `play-garden/`.
- **Source:** wave: garden

### I-garden-02 — "Relax in the shade" is half an hour standing by the trunk

> Moved from garden.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I3, A2, I2, I16
- **Fix class:** clip
- **Where:** `web/public/content/base.json:2861` (`shade`: `anim: relax`, no `pose`); `web/public/content/garden.json:122, 150, 178, 243, 271, 322, 365` (the same use on the maple, birch, cherry, apple, Japanese maple, magnolia and willow); `web/src/render/babylon/characters.ts:472` (`ACTIONS.relax.stand`: `idle`, `foldArms`)
- **Evidence:**
  - Pilot: `play/tree-shade-01.png`, `sheets/obj-tree.png`: the resident stands at the trunk, arms at the sides or folded, for 30 minutes. Relaxing in a tree's shade is sitting against the trunk or lying in the grass. No clip sits on the ground today (`sit` needs a seat height); a sit-on-the-ground pose, or `pose: sit` at a seat height of 0, is the fix.
  - Garden wave: the same on all seven garden trees that have the use (`play/trees-wide/garden.<tree>-shade-01.png`, `extra/close/close-garden.apple-shade-01.png`, `close-garden.willow-shade-01.png`). Two more things wrong with the place and the time:
    - The slot is the front tile wherever the shadow falls: at the maple, birch and cherry the resident stands on the lit side of the trunk with the crown's shadow beside them (`sheets/obj-garden.maple.png`, `sheets/obj-garden.birch.png`, play shots).
    - The use has no time rule: it is offered at night, when there is no shade to relax in (I16; the same gap as stargazing at noon, I-hobbies-03).
- **Status:** confirmed and widened (garden wave: maple, birch, cherry, apple, Japanese maple, magnolia, willow; the slot and the night)
- **Source:** pilot; wave: garden

### I-living-06 — Sitting on the floor is the chair sit: shins under the floor on the floor cushion, legs into the fire at the electric fireplace

> Moved from living.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/seated.ts` (the `sit` clip, made for a chair seat); `web/public/content/catalog2/living.json:617` (`floorCushion` seat 0.14), `web/src/render/babylon/characters/furniture.json:30` (`seats.fireplaceElectric`: 0.14, `fwd` 0.3, facing the fire)
- **Evidence:** The lead "legs below the floor on the floor cushion": on the 0.14 m cushion the resident sits with the knees bent as on a chair, so the shins go down through the floor, worst for the teen and the elder (`living/sheets/age-floorCushion.png`; the adult in play has the feet at the floor's edge, `living/play/chunk3/floorCushion-sit-01.png`). At the electric fireplace the same sit on its built-in cushion, 0.3 m in front, facing the fire, puts the knees and shins into the surround and the firebox among the flames (`living/play/chunk3/fireplaceElectric-relax-01.png`, `living/sheets/age-fireplaceElectric.png`). Real people sit on a floor cushion cross-legged or kneeling. A floor-sitting clip (cross-legged, for seats under about 0.25 m), or at least legs out (`legsOut`, as the chaise), with the fireplace's cushion moved back so the feet stop short of the surround. Other low seats may share it, not checked here: `yogaMat` (seat 0.03), `beanBagKids` (0.24, kids ledger).
- **Status:** open
- **Source:** wave: living (Phase 0 "Seen" lead)

### I-outdoor-03 — On the swing set the hands reach forward as at a desk, not up on the chains

> Moved from outdoor.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S2
- **Rubric:** I3 (A1)
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:477` (`ACTIONS.play`: `sit: 'sitHands'`, `HANDS.keys`)
- **Evidence:** `extra/swing-hands.png` (from `play/chunk2/swingSet-swing-01.png` and `play/chunk8-child/swingSet-swing-01.png`): the adult and the child sit on the swing with their forearms out over their knees and typing fingers, the chains passing beside their shoulders. On a swing both hands hold the chains at about shoulder height. `play` sitting is the desk-hands clip, right for the chess table, puzzles and consoles but not for rides; the same entry drives `rockingHorse` (kids: hands on the handles), `seesaw`, `springRider` and `merryGoRound` (community). A rides entry (or `sit` chosen by def, as `music` does), holding up in a grip.
- **Status:** open
- **Source:** outdoor wave (`play` adult and child)

### I-community-13 — Rides are sat like chairs: no astride pose on the spring horse or the seesaw

> Moved from community.md at the consolidation (2026-10-21): the cause is shared, not one object's.

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:471` (`ACTIONS.sit`), the seated clip (`characters/clips/seated.ts`); `tools/art/blender/families/playground.py:319` (the horse's barrel, 0.26 m across at the saddle), `:285-290` (the seesaw's seats and handles)
- **Evidence:** `extra/poses/rides-child.png`, `sheets/age-springRider.png`: on the spring horse the knees come forward together over the neck as on a chair, the thighs through the barrel's sides, the hands loose rather than on the handles; on the seesaw both riders sit sideways off the plank (also I-community-02, the facing). A rider sits astride, the legs either side, the feet on the pegs, the hands on the grips. Needs an astride seated pose (a `legs: astride` seat, as `legs: out` exists).
- **Status:** open
- **Source:** community wave (poses)
