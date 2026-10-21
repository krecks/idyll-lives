# Residents: authenticity ledger

What it covers: the residents: 2 bodies × 6 life stages, the `build` slider, hair, painted clothes and shells, the resident clips and the procedural layers. Built by `cargo characters` (`crates/tools/src/characters/`); contract in [docs/characters-format.md](../characters-format.md), look in [docs/design/look.md](../design/look.md). Hug, kiss and conversation pairs are in [social.md](social.md).

- **Evidence:** `bench-results/review/residents/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## Bodies and life stages (12)

Proportions and heights: `crates/tools/src/characters/rig.rs` (`STAGES`, `proportions`, `target_height`). Rubric M14.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `male` baby | — | — | — | | |
| `male` child | — | — | — | | |
| `male` teen | — | — | — | | |
| `male` youngAdult | — | — | — | | |
| `male` adult | — | — | — | | |
| `male` elder | — | — | — | | |
| `female` baby | — | — | — | | |
| `female` child | — | — | — | | |
| `female` teen | — | — | — | | |
| `female` youngAdult | — | — | — | | |
| `female` adult | — | — | — | | |
| `female` elder | — | — | — | | |

## Build (2)

`appearance.build`, M16.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `build` 0.85 (slim) | — | — | — | | |
| `build` 1.2 (broad) | — | — | — | | |

## Hair (13)

`crates/tools/src/characters/hair.rs`. M15.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `hair.none` (bald) | — | — | — | | |
| `hair.short` | — | — | — | | |
| `hair.crop` | — | — | — | | |
| `hair.layered` | — | — | — | | |
| `hair.tousled` | — | — | — | | |
| `hair.sideSwept` | — | — | — | | |
| `hair.afro` | — | — | — | | |
| `hair.bob` | — | — | — | | |
| `hair.long` | — | — | — | | |
| `hair.ponytail` | — | — | — | | |
| `hair.braid` | — | — | — | | |
| `hair.bun` | — | — | — | | |
| `hair.beard` (men; hidden for baby, child, teen) | — | — | — | | |

## Clothes (24)

Painted garments and their shells: `crates/tools/src/characters/garments.rs`, `clothes.rs`; ids from `outfit.ts`. M15.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `top.tee` (male) | — | — | — | | |
| `top.vneck` (male) | — | — | — | | |
| `top.polo` (male) | — | — | — | | |
| `top.long` (male) | — | — | — | | |
| `top.shirt` (male) | — | — | — | | |
| `top.jacket` (male) | — | — | — | | |
| `top.suit` (male) | — | — | — | | |
| `bottom.trousers` (male) | — | — | — | | |
| `bottom.shorts` (male) | — | — | — | | |
| `bottom.suit` (male) | — | — | — | | |
| `top.tee` (female) | — | — | — | | |
| `top.tank` (female) | — | — | — | | |
| `top.blouse` (female) | — | — | — | | |
| `top.long` (female) | — | — | — | | |
| `top.shirt` (female) | — | — | — | | |
| `top.jacket` (female) | — | — | — | | |
| `bottom.trousers` (female) | — | — | — | | |
| `bottom.shorts` (female) | — | — | — | | |
| `bottom.capri` (female) | — | — | — | | |
| `bottom.skirt` (female) | — | — | — | | |
| `shoes.sneakers` (both) | — | — | — | | |
| `shoes.trainers` (both) | — | — | — | | |
| `shoes.boots` (both) | — | — | — | | |
| `shoes.dress` (both) | — | — | — | | |

## Clips: idle.ts (8)

`web/src/render/babylon/characters/clips/idle.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `idle` | — | — | — | | |
| Clip `talk` | — | — | — | | |
| Clip `foldArms` | — | — | — | | |
| Clip `yes` | — | — | — | | |
| Clip `no` | — | — | — | | |
| Clip `phone` | — | — | — | | |
| Clip `lantern` | — | — | — | | |
| Clip `wave` | — | — | — | | |

## Clips: locomotion.ts (3)

`web/src/render/babylon/characters/clips/locomotion.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `walk` | — | — | — | A-residents-03 S3 clip (seeded); A-residents-04 S3 code (seeded) | |
| Clip `jog` | — | — | — | | |
| Clip `jogAlong` | — | — | — | A-residents-02 S3 clip (seeded); A-residents-03 S3 clip (seeded); A-residents-06 S3 code (seeded) | |

## Clips: seated.ts (9)

`web/src/render/babylon/characters/clips/seated.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `sit` | — | — | — | | |
| Clip `sitTalk` | — | — | — | | |
| Clip `sitDown` | — | — | — | A-residents-05 S3 code (seeded) | |
| Clip `standUp` | — | — | — | A-residents-01 S2 code (seeded) | |
| Clip `sitHands` | — | — | — | | |
| Clip `sitHandsNear` | — | — | — | | |
| Clip `sitHandsFar` | — | — | — | | |
| Clip `eat` | — | — | — | | |
| Clip `drink` | — | — | — | | |

## Clips: work.ts (10)

`web/src/render/babylon/characters/clips/work.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `interact` | — | — | — | | |
| Clip `pickUp` | — | — | — | | |
| Clip `spell` | — | — | — | | |
| Clip `chop` | — | — | — | | |
| Clip `stir` | — | — | — | | |
| Clip `flip` | — | — | — | | |
| Clip `carry` | — | — | — | | |
| Clip `carryOne` | — | — | — | | |
| Clip `rail` | — | — | — | | |
| Clip `push` | — | — | — | | |

## Clips: garden.ts (6)

`web/src/render/babylon/characters/clips/garden.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `water` | — | — | — | | |
| Clip `harvest` | — | — | — | | |
| Clip `plant` | — | — | — | | |
| Clip `kneel` | — | — | — | | |
| Clip `crouch` | — | — | — | | |
| Clip `pick` | — | — | — | | |

## Clips: action.ts (4)

`web/src/render/babylon/characters/clips/action.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `dance` | — | — | — | | |
| Clip `punchJab` | — | — | — | | |
| Clip `punchCross` | — | — | — | | |
| Clip `hit` | — | — | — | | |

## Clips: lie.ts (1)

`web/src/render/babylon/characters/clips/lie.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `lie` | — | — | — | | |

## Clips: baby.ts (2)

`web/src/render/babylon/characters/clips/baby.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `babyLieAsleep` | — | — | — | | |
| Clip `babyLieAwake` | — | — | — | | |

## Clips: street.ts (11)

`web/src/render/babylon/characters/clips/street.ts`; lengths and uses in [docs/characters-format.md](../characters-format.md#clips).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Clip `fountainDrink181` | — | — | — | | |
| Clip `fountainDrink163` | — | — | — | | |
| Clip `fountainDrink147` | — | — | — | | |
| Clip `fountainDrink129` | — | — | — | | |
| Clip `fountainDrink112` | — | — | — | | |
| Clip `climb172a` | — | — | — | | |
| Clip `climb147a` | — | — | — | | |
| Clip `climb112a` | — | — | — | | |
| Clip `climb172b` | — | — | — | | |
| Clip `climb147b` | — | — | — | | |
| Clip `climb112b` | — | — | — | | |

## Actions (45)

Content `anim` → clip, hands and layers: `ACTIONS` in `web/src/render/babylon/characters.ts`. Rubric I3; the object ledgers judge each use, this row the entry itself.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `ACTIONS.eat` | — | — | — | | |
| `ACTIONS.drink` | — | — | — | | |
| `ACTIONS.cook` | — | — | — | | |
| `ACTIONS.wash` | — | — | — | | |
| `ACTIONS.shower` | — | — | — | | |
| `ACTIONS.bath` | — | — | — | | |
| `ACTIONS.toilet` | — | — | — | | |
| `ACTIONS.sleep` | — | — | — | | |
| `ACTIONS.nap` | — | — | — | | |
| `ACTIONS.sit` | — | — | — | | |
| `ACTIONS.relax` | — | — | — | | |
| `ACTIONS.type` | — | — | — | | |
| `ACTIONS.write` | — | — | — | | |
| `ACTIONS.read` | — | — | — | | |
| `ACTIONS.watch` | — | — | — | | |
| `ACTIONS.play` | — | — | — | | |
| `ACTIONS.music` | — | — | — | | |
| `ACTIONS.paint` | — | — | — | | |
| `ACTIONS.exercise` | — | — | — | | |
| `ACTIONS.run` | — | — | — | | |
| `ACTIONS.lift` | — | — | — | | |
| `ACTIONS.dance` | — | — | — | | |
| `ACTIONS.talk` | — | — | — | | |
| `ACTIONS.sing` | — | — | — | | |
| `ACTIONS.clean` | — | — | — | | |
| `ACTIONS.water` | — | — | — | | |
| `ACTIONS.tend` | — | — | — | | |
| `ACTIONS.cuddle` | — | — | — | | |
| `ACTIONS.harvest` | — | — | — | | |
| `ACTIONS.prune` | — | — | — | | |
| `ACTIONS.garden` | — | — | — | | |
| `ACTIONS.tinker` | — | — | — | | |
| `ACTIONS.look` | — | — | — | | |
| `ACTIONS.smell` | — | — | — | | |
| `ACTIONS.admire` | — | — | — | | |
| `ACTIONS.phone` | — | — | — | | |
| `ACTIONS.listen` | — | — | — | | |
| `ACTIONS.meditate` | — | — | — | | |
| `ACTIONS.idle` | — | — | — | | |
| `ACTIONS.knock` | — | — | — | | |
| `ACTIONS.fetch` | — | — | — | | |
| `ACTIONS.chop` | — | — | — | | |
| `ACTIONS.stir` | — | — | — | | |
| `ACTIONS.flip` | — | — | — | | |
| `ACTIONS.serve` | — | — | — | | |

## Procedural layers (11)

Code in `web/src/render/babylon/characters.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Face: expressions (moods) and blinks | — | — | — | A-residents-07 S2 code (seeded) | |
| Face: lip sync (visemes) | — | — | — | | |
| Eyes: looking at people and things | — | — | — | | |
| Elder's stoop | — | — | — | | |
| Arms a little out over a skirt | — | — | — | | |
| Hands: open or gripping per activity | — | — | — | | |
| Seat fit: pelvis `seatDrop` over the seat (`SEATS`, content `seat`) | — | — | — | | |
| Lie fit: `lie` turned flat on the mattress (`BEDS`, `LIE_*`) | — | — | — | | |
| Blankets and pyjamas in bed | — | — | — | | |
| Turning: towards travel, and on the spot | — | — | — | A-residents-08 S3 code (seeded) | |
| Approach: stepping to a use and back | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-residents-01 — Getting up from a chair glides back to the tile

- **Severity:** S2
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2839` (`standUp`), `crates/sim-core/src/world.rs`
- **Evidence:** The sim puts the resident on the tile in front at once, so `standUp` plays while they glide back to it. Deferred from 0.3.0 unless the stand-up is reworked.
- **Status:** seeded (from `docs/objects/open.md` L124)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred)

### A-residents-02 — `check_clips.ts` fails `jogAlong`: a loop seam and sliding feet

- **Severity:** S3
- **Rubric:** A1, A3
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/locomotion.ts:208`, `tools/characters/check_clips.ts`
- **Evidence:** Fails since before fbf8883. The plan takes it as the baseline failure for T5.
- **Status:** seeded (from `docs/objects/open.md` L127)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/authenticity-review.md` §5.1 (T5)

### A-residents-03 — Feet still slide at game speed 3 and above

- **Severity:** S3
- **Rubric:** A1, A3
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:136` (`GAIT_MAX_RATE`), `web/src/render/babylon/characters/clips/locomotion.ts:208`
- **Evidence:** After R1-05's jog switch (76ac264), foot slide at height 1.0 with no traits: adults at speed 3 50–53 %, speeds 4–5 83–95 %. Children walk at the adults' 1.5 m/s on short legs.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §3.1 R1-05)
- **Source:** `docs/release-0.2.0-review.md` §2.1 and §3.1 (R1-05)

### A-residents-04 — The start and end of a walk jump at speeds 4–5

- **Severity:** S3
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (walker interpolation)
- **Evidence:** R1-04's rest: a walker standing in one snapshot wasn't interpolated (jumps up to 4.6 m at speed 5). CHANGELOG 0.3.0 (Fixes) says "Walks no longer jump at the start and the end": confirm and close.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 R1-04 (rest))
- **Source:** `docs/release-0.2.0-review.md` §4 (R1-04 rest); `plan/release-0.3.0.md` §3.2; `CHANGELOG.md` 0.3.0 Fixes

### A-residents-05 — After a hidden tab, someone who sat down meanwhile replays the sit-down

- **Severity:** S3
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (a rig drawn again)
- **Evidence:** R1-12's fix covers residents coming back into view, not a hidden tab: they count as on screen while it is hidden. It ends on its own.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 R1-12 (hidden tab))
- **Source:** `docs/release-0.2.0-review.md` §4 (R1-12 hidden tab, new), target 0.2.1

### A-residents-06 — Walkers jog on the spot for about 0.25 s after a hidden tab

- **Severity:** S3
- **Rubric:** A3, A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (walking speed estimate)
- **Evidence:** Speed is estimated from the distance covered over the capped 0.1 s frame (8.7–22 m → 13.2 m/s), past the jog switch, then fades back over about 0.3 s. Seen in all four hidden-tab runs.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 "Jogging after a hidden tab")
- **Source:** `docs/release-0.2.0-review.md` §4 (new, R1), target 0.2.1

### A-residents-07 — Body language doesn't follow feelings; only the face does

- **Severity:** S2
- **Rubric:** A6
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:684` (face moods)
- **Evidence:** A known gap named in the rubric. Emotional body language is new behaviour: filed as `code`, planned separately (§9, decision 3).
- **Status:** seeded (from `plan/authenticity-review.md` §3.2 A6)
- **Source:** `plan/authenticity-review.md` §3.2 (A6) and §9 (decision 3)

### A-residents-08 — No turn-on-the-spot clip

- **Severity:** S3
- **Rubric:** A4
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters/clips/locomotion.ts`, `web/src/render/babylon/characters.ts` (turning)
- **Evidence:** Residents turn in place without stepping. A turn-on-the-spot clip is new behaviour: filed only, planned separately (§9, decision 3).
- **Status:** seeded (from `plan/authenticity-review.md` §9)
- **Source:** `plan/authenticity-review.md` §3.2 (A4) and §9 (decision 3)
