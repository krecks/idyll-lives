# Kids: authenticity ledger

What it covers: the 20 objects of [docs/objects/kids.md](../objects/kids.md), in its groups and order.

- **Evidence:** `bench-results/review/kids/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## KD1. Baby and kids' beds (4)

Family: `tools/art/blender/families/kids.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `crib` Crib | — | — | — | | |
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
