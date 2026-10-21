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
| `male` child | M-residents-01 | A-residents-09, A-residents-11 | I-kids-03 | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip; I-kids-03 S1 code (kids ledger) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `sheets/seat-sofaM.png`, `play-child/*.png` |
| `male` teen | M-residents-01 | A-residents-09, A-residents-11 | I-kids-03 | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip; I-kids-03 S1 code (kids ledger) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play-teen/*.png` |
| `male` youngAdult | M-residents-01 | OK | OK | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png` |
| `male` adult | M-residents-01 | OK | OK | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play/*.png` |
| `male` elder | M-residents-01 | OK | OK | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `residents/chars-elder-side.png`, `poses/seat-sofaF-elder.png`, `play-elder/*.png` |
| `female` baby | M-residents-01 | OK | n/a | M-residents-01 S3 decision | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play/crib-lie-*.png` |
| `female` child | M-residents-01 | A-residents-09, A-residents-11 | I-kids-03 | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip; I-kids-03 S1 code (kids ledger) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `sheets/seat-sofaM.png`, `play-child/*.png` |
| `female` teen | M-residents-01 | A-residents-09, A-residents-11 | I-kids-03 | M-residents-01 S3 decision; A-residents-09 S2 clip; A-residents-11 S2 clip; I-kids-03 S1 code (kids ledger) | `residents/chars-stages.png`, `residents/chars-stages-side.png`, `play-teen/*.png` |
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
| Clip `spell` | n/a | OK | I-bathroom-02 | I-bathroom-02 S3 clip (bathroom ledger: the shower) | `residents/strip-spell.png`, `play/shower-shower-01.png` |
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
| `ACTIONS.bath` | n/a | OK | A-bathroom-04 | A-bathroom-04 S3 renderer-table (bathroom ledger: bathers 10–12 cm above the tub floor); I-bathroom-03 S2 code (bathroom ledger: in clothes, also in the hot tubs). The hot tub, whirlpool, cold plunge and heart tub sit low in the water, hands on the rim | `bathroom/play/chunk1/bathtub-bath-01.png`, `systems/sheets/act-write-garden-bath.png` |
| `ACTIONS.toilet` | n/a | OK | OK |  | `play/toilet-use-01.png` |
| `ACTIONS.sleep` | n/a | OK | OK |  | `play/bed-sleep-02.png` |
| `ACTIONS.nap` | n/a | OK | OK |  | `play/bed-nap-01.png` |
| `ACTIONS.sit` | n/a | A-residents-09 | OK | A-residents-09 S2 clip | `play/armchair-sit-01.png`, `play/bench-sit-01.png` |
| `ACTIONS.relax` | n/a | OK | I-garden-02 | I-garden-02 S2 clip (garden ledger: standing in the shade) | `play/sofa-relax-01.png`, `play/tree-shade-01.png` |
| `ACTIONS.type` | n/a | OK | OK |  | `play/computerDesk-code-01.png` |
| `ACTIONS.write` | n/a | OK | I-kids-03 | Seated at the desk, the hands on the paper (`sitHands`, `sitHandsNear` at the built-in-chair desks); posting a letter standing at the post box. An adult at the kids' desk: I-kids-03 S1 code (kids ledger) | `systems/sheets/act-write-garden-bath.png` |
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
| `ACTIONS.harvest` | n/a | OK | I-garden-03 | I-garden-03 S2 code and I-garden-04 S2 renderer-table (garden ledger: the crop goes at the start; the hand half a metre over low crops); the herb planter's `pick` fits. The greenhouse is harvested from outside its door (outdoor ledger's object) | `garden/extra/harvest/s-garden.strawberries-harvest.png`, `systems/sheets/act-write-garden-bath.png` |
| `ACTIONS.prune` | n/a | OK | I-held-01 | I-held-01 S2 code (no shears) | `play/bush-trim-01.png` |
| `ACTIONS.garden` | n/a | OK | I-residents-03 | Feeding the fish fits (`water` at the pond's rim); I-residents-03 S3 renderer-table (the songbird feeder refilled at knee height, 1 m under it) | `systems/sheets/act-write-garden-bath.png` |
| `ACTIONS.tinker` | n/a | OK | OK |  | `play/workbench-tinker-01.png` |
| `ACTIONS.look` | n/a | OK | I-hobbies-02 | I-hobbies-02 S2 clip (hobbies ledger: the telescope) | `play/telescope-stargaze-01.png` |
| `ACTIONS.smell` | n/a | OK | OK |  | `play/flowerbed-smell-01.png` |
| `ACTIONS.admire` | n/a | OK | I-residents-03 | I-residents-03 S3 renderer-table (the statue admired with the nose to its plinth); I-garden-02 S2 clip (garden ledger: the trees) | `systems/sheets/act-phone-meditate-idle.png`, `garden/play/c1/garden.cherry-admire-01.png` |
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
| Seat fit: pelvis `seatDrop` over the seat (`SEATS`, content `seat`) | n/a | A-residents-09, A-residents-11 | I-kids-03 | A-residents-09 S2 clip; A-residents-11 S2 clip (widened: the child in the bean bag); M-residents-02 S3 model (widened: every seated woman's skirt a hoop round the seat); M-outdoor-02 S3 model (outdoor ledger: the bench 4 cm into its slats); I-kids-03 S1 code (kids ledger: teens, adults and elders sit on the 0.31 m kids' chair as on any). Cross-cutting wave, over the other waves' age sheets (`<cat>/sheets/age-*.png`: 253, of which the dining chairs, bar stools, sofas, armchairs, bean bags, office chairs, kids' chair, fountain rim read): the pelvis sits on the seat at every stage and every height of seat; a child's feet dangle on adult chairs and bar stools (as a child's do), a teen's and an elder woman's hang a few cm off a 0.45 m seat (A-residents-09); deep low seats swallow short shins | `kitchen/sheets/age-diningChair.png`, `kitchen/sheets/age-barStool.png`, `living/sheets/age-beanBag.png`, `kids/sheets/age-kidsChair.png` |
| Lie fit: `lie` turned flat on the mattress (`BEDS`, `LIE_*`) | n/a | A-fitness-01, A-bedroom-03 | OK | A-fitness-01 S2 clip (fitness ledger); I-kids-03 S1 code (kids ledger: an adult in the cot); A-bedroom-03 S2 code (bedroom ledger: into bed at the foot end, at every stage, the bunk's elder too); A-bedroom-02 S2 code (bedroom ledger: the upper bunk). Cross-cutting wave, over the bedroom, kids, living and packs age sheets: the sleeper lies on the mattress at every stage on every bed; `poses.html` puts only a child in the kids' bed (sim-core's stage rules for free will; a direct use skips them, I-kids-03) | `bedroom/sheets/age-bedBunk.png`, `kids/sheets/age-kidsBed.png`, `play/bed-sleep-02.png` |
| Blankets and pyjamas in bed | OK | OK | n/a |  | `play/bed-sleep-02.png`, `sheets/bedpose-0.png` |
| Turning: towards travel, and on the spot | n/a | A-residents-08 | n/a | A-residents-08 S3 code (seeded; seen: residents turn in place at each use) | `play/*-00.png` |
| Approach: stepping to a use and back | n/a | OK | OK | A-bathroom-01 S2 model (bathroom ledger: through the shower door) | `play/*-00.png`, `play/shower-shower-00.png` |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-residents-01 — Getting up from a chair glides back to the tile

- **Severity:** S2
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2839` (`standUp`), `crates/sim-core/src/world.rs`
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

- **Severity:** S2
- **Rubric:** A1, A5
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/` (sit), `web/src/render/babylon/characters.ts` (seat fit per stage)
- **Evidence:** `pnpm check:clips` (`tools/characters/clip-baseline.json`): a child on the 0.31 m kids' chair has the feet 7 cm (male) and 8 cm (female) off the floor; a teen on a 0.45 m seat 7 cm and 9 cm; an elder woman on a 0.45 m seat 5 cm. Adults pass.
- **Status:** confirmed (pilot: `check:clips` as above; in shots, a child on the sofa, armchair and bench has the feet off the floor, `sheets/seat-sofaM.png`, `sheets/seat-benchM.png`; the teen and the elder woman less so)
- **Source:** `tools/characters/check_clips.ts --body all --stage all`

### A-residents-10 — A child's feet slide in `jog` and on the climbing frame

- **Severity:** S3
- **Rubric:** A1, A5
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/` (jog, climb)
- **Evidence:** `pnpm check:clips`: a child's right foot slides 1.5 cm in `jog` (both bodies); the left foot slides 1.3–1.9 cm in `climb112a`/`climb112b`, and an elder woman's 1.1 cm in `climb172a`.
- **Status:** confirmed (pilot: `check:clips` prints all five as known)
- **Source:** `tools/characters/check_clips.ts --body all --stage all`

### A-residents-11 — An elder woman, a teen or a child on the sofa loses the shins

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
- **Status:** open
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

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:481-495` (`ACTIONS.exercise`: `boulder` and `sunrise` get `idle` with `activityPose` on top; `fitnessGame` `dance`, `punchJab`, `punchCross` from the tile in front), `:198` (`STEP_IN.climbingFrame`, the climbing clips the frame already has)
- **Evidence:** `systems/sheets/act-exercise.png` (the packs wave's `packs/play/chunk2/energetic.boulderingWall-climb-01.png`): for the 60 minutes of "Climb" the resident stands on the crash mat at the wall's foot with both arms raised, the feet on the floor, never on a hold; the climbing frame beside it climbs its net (`community/play/climbingFrame-climb-01.png`, `climb*` clips at five heights). The fitness game (`packs/play/chunk1/couchPotato.fitnessGame-workout-01.png`) is danced and punched standing beside its balance board, not on it. The exercise bike, spin bike, hoops, trampoline, heavy bag, vibe plate and the sunrise lamp's stretch fit their objects.
- **Status:** open
- **Source:** wave: cross-cutting (ACTIONS entries no base object uses, judged at the packs wave's objects)

### I-residents-02 — Singing mimes a hand microphone beside the karaoke machine's own microphone stand

- **Severity:** S3
- **Rubric:** I3
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:501` (`ACTIONS.sing`: `talk` with the `phone` clip's hand at the mouth as a microphone)
- **Evidence:** `systems/sheets/act-dance-sing-listen.png` (`packs/play/chunk3/cheerful.karaokeMachine-sing-01.png`, `packs/play/chunk4/outgoing.karaokeStage-perform-01.png`): at the karaoke machine the singer stands to one side of its mic stand, an empty hand held to the mouth; on the karaoke stage they stand at the front edge, the two mic stands behind them. A singer at a stand sings into it, hands on the stand or free; a karaoke singer holds the machine's microphone (a prop: I-held-01's family).
- **Status:** open
- **Source:** wave: cross-cutting

### I-residents-03 — Standing uses with no reach to the part used: the bird feeder, the giving jar, the statue

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:514-517` (`ACTIONS.garden`: `water` for the feeder), `:529` (`admire`: `idle`, `foldArms`), `:533` (`idle`)
- **Evidence:** `systems/sheets/act-write-garden-bath.png`, `systems/sheets/act-phone-meditate-idle.png` (the packs and community waves' play shots): "Refill the feeder" is the watering clip at knee height beside the bird bath, a metre under the 1.5 m feeder box; "Donate" at the giving jar stands by it with the hands at the sides; "Admire the statue" stands with the face to its plinth, the statue 2 m over the head, never looked up at. The pond's "Feed the fish" and the shop counter's treat (served across it) fit. A reach to the part's height (`STEP_IN`-style sizes, or the `look` entry's head tilt) would fix each.
- **Status:** open
- **Source:** wave: cross-cutting
