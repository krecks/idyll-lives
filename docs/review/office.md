# Office: authenticity ledger

What it covers: the 14 objects of [docs/objects/office.md](../objects/office.md), in its groups and order.

- **Evidence:** `bench-results/review/office/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## O1. Desks (8)

Family: `tools/art/blender/families/office.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `deskSimple` Simple desk | — | — | — | | |
| `deskCorner` Corner desk | — | — | — | | |
| `deskStanding` Standing desk | — | — | — | A-office-01 S2 renderer-table (seeded) | |
| `deskExecutive` Executive desk | — | — | — | I-office-02 S3 decision (seeded) | |
| `computerDesk` Computer desk | — | — | — | | |
| `deskWriting` Writing desk | — | — | — | | |
| `deskLaptop` Laptop desk | — | — | — | I-office-01 S3 renderer-table (seeded) | |
| `computerGaming` Gaming PC | — | — | — | | |

## O2. Desk chairs (3)

Family: `tools/art/blender/families/office.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `chairExecutive` Executive chair | — | — | — | | |
| `chairTask` Desk chair | — | — | — | | |
| `officeChair` Office chair | — | — | — | A-office-01 S2 renderer-table (seeded) | |

## O3. Storage and boards (3)

Family: `tools/art/blender/families/office.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `filingCabinet` Filing cabinet | — | — | — | | |
| `whiteboard` Whiteboard | — | — | — | | |
| `corkboard` Corkboard | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-office-01 — The office chair at the standing desk: the top is 0.57 m over the seat

- **Severity:** S2
- **Rubric:** A1, I2
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/tableTuck.ts`, `web/src/render/babylon/characters.ts:268` (`TUCK`)
- **Evidence:** The tuck pulls a chair in to a top it can't reach: at the standing desk the top is 0.57 m over an office chair's seat and the hands don't reach it. Needs its own reach or clip adjustment. Deferred from 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L121)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred)

### I-office-01 — A chair facing the laptop desk is used from behind its built-in chair

- **Severity:** S3
- **Rubric:** I2, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/tableTuck.ts`
- **Evidence:** A chair that faces a desk with its own chair built in (the laptop desk, and the kids' desk: I-kids-02) is tucked in from behind and doesn't reach the top. Deferred from 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L121)
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4

### I-office-02 — The executive desk's drawer has no use

- **Severity:** S3
- **Rubric:** I1, I7
- **Fix class:** decision
- **Where:** `tools/art/blender/families/office.py:504`, `web/public/content/catalog2/office.json:111`
- **Evidence:** A modelled part with no clip, left decorative for 0.3.0: its clip would need a seated use on the desk it faces.
- **Status:** seeded (from `docs/objects/open.md` L141)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5
