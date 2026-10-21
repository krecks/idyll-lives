# Kids: authenticity ledger

What it covers: the 20 objects of [docs/objects/kids.md](../objects/kids.md), in its groups and order.

- **Evidence:** `bench-results/review/kids/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## KD1. Baby and kids' beds (4)

Family: `tools/art/blender/families/kids.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `crib` Crib | M-kids-01 | I-kids-04 | I-kids-03, I-kids-05 | I-kids-03 S1 code (confirmed, widened); I-kids-04 S2 renderer-table; I-kids-05 S2 code; M-kids-01 S3 model | `sheets/obj-crib.png`, `sheets/crib-care.png`, `play/crib-*.png`, `scenes/crib-*.png`, `scenes/care-*.png`, `clips/clips-crib-all-t*.png`, `sheets/bedpose-6.png`, `sheets/bedpose-9.png` |
| `kidsBed` Kids' bed | — | — | — | | |
| `kidsBedCar` Race car bed | — | — | — | | |
| `highChair` High chair | — | — | — | I-kids-01 S3 code (seeded) | |

## KD2. Toys (10)

Family: `tools/art/blender/families/kids.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `toyBox` Toy box | — | — | — | | |
| `toyChest` Toy chest | — | — | — | | |
| `dollHouse` Doll house | — | — | — | | |
| `rockingHorse` Rocking horse | — | — | — | | |
| `playTent` Play tent | — | — | — | | |
| `ballPit` Ball pit | — | — | — | | |
| `trainTable` Train table | — | — | — | | |
| `playMat` Play mat | — | — | — | | |
| `teddyBear` Giant teddy bear | — | — | — | | |
| `beanBagKids` Kids' bean bag | — | — | — | | |

## KD3. Kids' study and art (6)

Family: `tools/art/blender/families/kids.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `kidsDesk` Kids' desk | — | — | — | I-kids-02 S3 renderer-table (seeded) | |
| `kidsTable` Kids' table | — | — | — | | |
| `kidsChair` Kids' chair | — | — | — | A-kids-01 S3 renderer-table (seeded) | |
| `kidsBookshelf` Picture-book shelf | — | — | — | | |
| `kidsChalkboard` Chalkboard easel | — | — | — | | |
| `kidsEasel` Kids' easel | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows; durations are judged by §9 decision 2.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `crib` | a baby sleeps; feed (bottle, in arms), change a nappy on the changing table, play, cuddle, settle them | lie in the crib (720, the baby), feed (15), change (10), play (20), cuddle (15) | the right set; feeding and changing don't look like it (I-kids-04, I-kids-05), and anyone may take any of them (I-kids-03) |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-kids-01 — A kids' chair at a coffee table: the hands hover

- **Severity:** S3
- **Rubric:** A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/tableTuck.ts`, `web/src/render/babylon/characters.ts:268` (`TUCK`)
- **Evidence:** A kids' chair pulled up to a coffee table (living.md L3) leaves the hands hovering over the top. Deferred from 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L121)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4

### I-kids-01 — The high chair's tray has no use: the sim never seats a baby in it

- **Severity:** S3
- **Rubric:** I1, I14
- **Fix class:** code
- **Where:** `tools/art/blender/families/kids.py:443`, `web/public/content/catalog2/dining.json:744`
- **Evidence:** A modelled part with no clip, left decorative for 0.3.0. A baby is never seated in the high chair, so nothing could use the tray.
- **Status:** seeded (from `docs/objects/open.md` L141)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-kids-02 — A chair facing the kids' desk is used from behind its built-in chair

- **Severity:** S3
- **Rubric:** I2, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/tableTuck.ts`
- **Evidence:** As I-office-01: a chair facing a desk with its own chair built in is tucked in from behind. Deferred from 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L121)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4

### I-kids-03 — A direct use skips the rules free will keeps: an adult lies in the crib, a child cares for the baby, an adult does homework

> Pilot: confirmed and widened to its cause. Severity raised from S2 to S1 (an adult in pyjamas lying in a cot, legs through the rails, from the pie menu); fix class from `content` to `code`.

- **Severity:** S1
- **Rubric:** I1, I6
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:2972` (`Command::Use`: checks broken, `requires`, `when`/open, `whole`, `needsTable`; nothing about the user); against `world.rs:3829-3856` (free will: a `baby` use only for a baby, care only by an adult with `Access::Full` and a baby in the crib, homework only at a school stage); `web/src/game/session.ts:1160` (the pie menu offers every interaction to whoever is selected)
- **Evidence:** `play/crib-lie-01.png`: the adult resident 0 lies in the crib in pyjamas, the legs out over the end rail (`sheets/crib-care.png` shows the cot at its real size beside the carers); the elder and the teen take it too (`play-elder/crib-lie-*.png`, `play-teen/crib-lie-*.png`). A newly bought, empty crib's feed, change, play and cuddle all run (`play/crib-feed-*.png` …), as Phase 0 saw. On `review-stage`'s own crib, with its baby, the child (8) and the teen (15) each feed, cuddle, change and play with the baby on a direct use (`scenes/care-child-*.png`, `scenes/care-teen-*.png`, `scenes/scenes.log`), which free will never lets them; an adult (`play/bookshelf-homework-*.png`, `play/computerDesk-homework-*.png`) does homework, which free will offers only to pupils. One cause: the command doesn't apply the user tests free will applies. Related: I-systems-04 (no age rule exists at all for a stove); this one is rules that exist but only for free will.
- **Status:** confirmed (pilot)
- **Source:** `check.mjs play --state review-stage --objects crib --use all` (Phase 0 and pilot, adult, teen, elder); pilot `scripts/scenes.mjs` childcare

### I-kids-04 — Feeding the baby is the gardening kneel at the cot rail

- **Severity:** S2
- **Rubric:** I3, A2, I14
- **Fix class:** renderer-table
- **Where:** `web/public/content/base.json:3397` (`feed`: `anim: tend`); `web/src/render/babylon/characters.ts:508` (`ACTIONS.tend`: `kneel`, the flower bed's clip)
- **Evidence:** `sheets/crib-care.png` (top left, `scenes/crib-adult-feed-02.png`): the carer kneels on the floor at the cot's rail for 15 minutes while the baby lies on the mattress inside, and a child sent to do it stands at the rail (`sheets/crib-care.png` bottom right, `scenes/care-child-feed-01.png`). A baby is fed held in the arms (or sitting up), with a bottle. `cuddle` already lifts the baby out and holds them (`CUDDLE`), so a feed that plays the cuddle hold (the bottle is I-held-01) is a table change: `feed` on its own `ACTIONS` entry, or `tend` chosen by object as `harvest` and `look` are.
- **Status:** open
- **Source:** pilot

### I-kids-05 — Changing the baby happens over the cot rail; the changing table is never used

- **Severity:** S2
- **Rubric:** I2, I14
- **Fix class:** code
- **Where:** `crates/sim-core/src/content.rs:273-276` (slot 0 the baby's, the carers' slots by the changing table); `crates/sim-core/src/world.rs:322` (`slot_position`); `web/src/render/babylon/characters.ts:502-505` (`ACTIONS.clean`: `interact`, `spell`)
- **Evidence:** `sheets/crib-care.png` (top right, `scenes/crib-elder-change-02.png`): the carer stands at the cot and works the hands over the rail at the lying baby for 10 minutes; the pad on the changing chest beside it (0.865 m, inside the 0.85–0.95 range) stays empty. A nappy is changed with the baby lying on the changing pad. Moving the baby to the pad and back (as `cuddle` lifts them) is new behaviour.
- **Status:** open
- **Source:** pilot

### M-kids-01 — The cot's mattress is square

- **Severity:** S3
- **Rubric:** M1, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/kids.py:345-373` (`crib`: posts ±0.42 m both ways, `mattress` 0.80 × 0.80 at `CRIB_TOP` 0.56, `:291`)
- **Evidence:** `sheets/obj-crib.png`, `clips/clips-crib-all-t99.png`: the cot is 0.88 m square in plan on the 1 × 1 half of its footprint; a cot is oblong (mattress 60 × 120 or 70 × 140 cm, `reference.json` `crib`). Its depth 0.93 m is over the 0.60–0.80 (±10 %) range for the same reason. The mattress height (0.56 m) and the rail top (0.92 m) are right. The check-glb warning "mattress 0.56 m, the model 0.780 m there (x 0.50)" is the changing pad under the second slot, not the cot (`BEDS.crib` applies to slot 0): a tool artefact.
- **Status:** open
- **Source:** pilot
