# Office: authenticity ledger

What it covers: the 14 objects of [docs/objects/office.md](../objects/office.md), in its groups and order.

- **Evidence:** `bench-results/review/office/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Wave rows** (office/kids wave, 2026-10-21): Shots are relative to `bench-results/review/` in the review worktree; the shots taken for this wave are in `office/extra/` (how: `office/review-log.md`).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## O1. Desks (8)

Family: `tools/art/blender/families/office.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `deskSimple` Simple desk | OK | n/a | OK | — (used through a desk chair; I-office-03 for the office chair at it) | `office/sheets/obj-deskSimple.png`, `office/play/pair-chairTask/chairTask-write-01.png`, `office/extra/gaps-officeChair-9-13-2-sit-01.png` |
| `deskCorner` Corner desk | M-office-01 | n/a | OK | M-office-01 S2 model | `office/sheets/obj-deskCorner.png`, `office/extra/gaps-chairTask-9-13-2-write-01.png` |
| `deskStanding` Standing desk | OK | A-office-01 | A-office-01, I-kids-03 | A-office-01 S2 content (seeded; confirmed and widened); I-kids-03 S1 code (systems ledger: a child at the standing desk) | `office/sheets/obj-deskStanding.png`, `office/extra/standing-chairTask-write-01.png`, `office/extra/child-deskStanding-browse-01.png` |
| `deskExecutive` Executive desk | OK | n/a | I-office-02 | I-office-02 S3 decision (seeded; confirmed) | `office/sheets/obj-deskExecutive.png`, `office/play/pair-chairExecutive/chairExecutive-write-01.png` |
| `computerDesk` Computer desk | OK | OK | I-kids-03 | I-kids-03 S1 code (systems ledger: an adult does homework) | `sheets/obj-computerDesk.png`, `play/computerDesk-*.png`, `clips/clips-computerDesk-all-t*.png`, `sheets/seat-deskM.png`, `sheets/seat-deskF.png`, `play-child/computerDesk-*.png`, `play-teen/computerDesk-*.png` |
| `deskWriting` Writing desk | OK | OK | I-kids-03 | I-kids-03 S1 code (systems ledger: an adult does homework) | `office/sheets/obj-deskWriting.png`, `office/sheets/age-deskWriting.png`, `office/play/c1/deskWriting-homework-01.png` |
| `deskLaptop` Laptop desk | OK | OK | I-kids-03 | I-kids-03 S1 code (systems ledger: an adult does homework); I-office-01 S3 renderer-table (seeded; closed) | `office/sheets/obj-deskLaptop.png`, `office/extra/laptop-chairTask-write-01.png` |
| `computerGaming` Gaming PC | OK | OK | I-office-05 | I-office-05 S3 content | `office/sheets/obj-computerGaming.png`, `office/sheets/age-computerGaming.png` |

### Real uses against the content's (I1)

Durations are judged by §9 decision 2 (short by design; only outliers against similar uses are filed).

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `deskSimple` | work, write, do homework at it on a chair; put things on it | none of its own; a desk chair facing it: write (60), homework (45); `surface` 0.74 | OK; the office chair can't work at it (I-office-03) |
| `deskCorner` | work with two screens, spread out papers, two people at once | none of its own; a desk chair per face; `surface` 1.86 × 1.86 | OK as uses; the model isn't an L (M-office-01) |
| `deskStanding` | work standing up (and, on a sit-stand desk, sitting) | browse (30), practise coding (60), standing; `surface` 1.05 | OK standing; a desk chair works at its 1.05 top too (A-office-01) |
| `deskExecutive` | work and write at it, keep papers in its drawers | none of its own; a desk chair facing it: write (60), homework (45) | OK; the drawer has no use (I-office-02) |
| `computerDesk` | work at a computer, write, browse, game, do homework | practise coding (60), write (60), study the markets (60), browse the web (45), do homework (45) | OK; homework offered to adults too (I-kids-03) |
| `deskWriting` | write, do homework, read, pay bills | write (60), homework (45) | OK; homework offered to adults too (I-kids-03) |
| `deskLaptop` | browse, write, code and do homework on a laptop | browse (45), code (60), write (60), homework (45) | OK; homework offered to adults too (I-kids-03) |
| `computerGaming` | play PC games, browse, code, stream | play games (60), code (60), browse (45) | OK; nothing can be put on its top (I-office-05) |

## O2. Desk chairs (3)

Family: `tools/art/blender/families/office.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `chairExecutive` Executive chair | OK | A-residents-09 | I-kids-03 | A-residents-09 S2 clip (residents ledger: a child's feet off the floor); I-kids-03 S1 code (systems ledger: an adult does homework) | `office/sheets/obj-chairExecutive.png`, `office/sheets/age-chairExecutive.png`, `office/play/pair-chairExecutive/chairExecutive-homework-01.png` |
| `chairTask` Desk chair | OK | A-residents-09 | I-kids-03 | A-residents-09 S2 clip (residents ledger); I-kids-03 S1 code (systems ledger: an adult does homework; a desk chair works at the kids' desk) | `office/sheets/obj-chairTask.png`, `office/sheets/age-chairTask.png`, `office/play/pair-chairTask/chairTask-homework-01.png` |
| `officeChair` Office chair | OK | A-residents-09 | I-office-03 | I-office-03 S2 content; A-residents-09 S2 clip (residents ledger); A-office-01 S2 content (seeded; it sits beside the standing desk) | `office/sheets/obj-officeChair.png`, `office/extra/gaps-officeChair-9-13-2-sit-01.png`, `office/extra/side-officeChair-sit-01.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `chairExecutive` | sit; work, write, do homework at a desk; swivel, lean back | sit (20); at a desk: write (60), homework (45) | OK; homework offered to adults too (I-kids-03) |
| `chairTask` | sit; work, write, do homework at a desk; swivel | sit (20); at a desk: write (60), homework (45) | OK; homework offered to adults too (I-kids-03) |
| `officeChair` | sit; work at a desk; swivel | sit (20) | work at a desk missing (I-office-03) |

## O3. Storage and boards (3)

Family: `tools/art/blender/families/office.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `filingCabinet` Filing cabinet | OK | OK | OK | | `office/sheets/obj-filingCabinet.png`, `office/play/c1/filingCabinet-file-01.png` |
| `whiteboard` Whiteboard | OK | I-office-04 | I-office-04 | I-office-04 S2 renderer-table | `office/sheets/obj-whiteboard.png`, `office/play/c1/whiteboard-plan-01.png`, `office/extra/child-whiteboard-plan-01.png` |
| `corkboard` Corkboard | OK | n/a | OK | — (decor 0.3: decor stays decor, §9.4) | `office/sheets/obj-corkboard.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `filingCabinet` | file papers, find a document | file papers (10) | OK (10 is real, 2–15; the T4 flag against the `fetch` median is not an outlier against filing) |
| `whiteboard` | plan, sketch ideas, do sums, wipe it clean | plan (30) | OK as a use; it is done from 1.4 m away (I-office-04) |
| `corkboard` | pin up notes and photos, read them | none (decor 0.3) | decor, no use needed (§9.4) |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-office-01 — A desk chair works at the standing desk: the top is 0.57 m over the seat

> Office/kids wave: confirmed and widened. A chair can no longer be bought on the standing desk's front tile ("that would block the way"), but one beside it, facing it, is pulled up and its `write` and `homework` run. Fix class changed from `renderer-table` to `content`: no tuck makes a 1.05 m top workable from a 0.48 m seat.

- **Severity:** S2
- **Rubric:** A1, I2, M2
- **Fix class:** content
- **Where:** `web/public/content/catalog2/office.json:65` (`deskStanding` `surface.kind: "desk"`), against `office.json:176` and `:252` (`chairExecutive`, `chairTask` `seatAt: ["desk", "table"]`, whose `needsTable` uses accept any `desk` top, `crates/sim-core/src/surface.rs:402` `faced_table`); `web/src/render/babylon/tableTuck.ts`
- **Evidence:** `office/extra/standing-chairTask-write-01.png`: the adult on the desk chair beside the standing desk "writes at the desk" with her hands under the 1.05 m top at chest height (seat 0.48; the reference `deskStanding` top 0.95–1.25 is for standing). `office/extra/side-officeChair-sit-01.png`: the office chair beside it, the top at chest height (sit only, I-office-03). A real standing desk is used standing, or with a drafting stool at about 0.7 m; a sit-stand desk would come down to 0.72–0.76. Fix: give the standing desk a kind of its own (or `counter`) that desk chairs don't `seatAt`, or make it sit-stand.
- **Status:** fixed (543c6121)
- **After:** The standing desk's top is a `shelf` (things still stand on it; no seat tucks under it): `chairTask@11,12,3 --with deskStanding@10,12,0` write and homework say "needs a table".
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4 and §8 (deferred); wave: office/kids (`check.mjs play --objects chairTask@11,12,3 --with deskStanding@10,12,0 --use write`)

### I-office-01 — A chair facing the laptop desk is used from behind its built-in chair

> Office/kids wave: closed. The case can't be set up today.
>
> Consolidation: reopened. The kitchen wave set the case up on the dining sets, which have their own chairs built in: a chair bought facing the base dining table, the glass or the farmhouse set passes `needsTable` and sits behind or beside the set's own chair, about a metre from the top, for its coffee (`kitchen/play/c5/chairModern-coffee-01.png`, `benchDining-coffee-01.png`, `chairWindsor-coffee-01.png`). Items: `diningTable`, `diningGlass`, `diningOak`, `bistroSet`. Still closed for the desks.

- **Severity:** S3
- **Rubric:** I2, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/tableTuck.ts`
- **Evidence:** A chair that faces a desk with its own chair built in (the laptop desk, and the kids' desk: I-kids-02) is tucked in from behind and doesn't reach the top. Deferred from 0.3.0. Wave check: buying a desk chair on the laptop desk's (and the kids' desk's) front tile, facing it, is refused by the game, "that would block the way", on both tries (`office/extra/play-gaps.jsonl`). Beside the desk, facing its end, the chair isn't pulled in and the resident writes at the end of the top with a hand on its edge (`office/extra/laptop-chairTask-write-01.png`): believable enough, not filed.
- **Status:** fixed (COMMIT)
- **After:** a top sat at by a use of its own (a dining set with its chairs, a desk with its chair built in) is no table for another seat: sim-core `surface::faced_table` skips it (`seats_itself`), and so do the game's `facedTable` (build mode, the menu) and the renderer's tuck (`tableTuck.ts`). A chair bought facing the base dining table or the farmhouse set is no seat at it: "Have a coffee needs a table: pull the moulded chair up to one" (`fixes/sweep-render/sets/play-sets.jsonl`: `chairModern` and `chairWindsor` behind `diningTable` and `diningOak`, refused; the same chair at `tableSquare2` drinks there, `sets-chairModern-9-11-0-coffee-01.png`); the set's own coffee is from its own chair with the loose chair there (`own-diningTable-coffee-01.png`, was `kitchen/play/c5/chairModern-coffee-01.png`). Test: `dining.rs` `a_chair_is_no_seat_at_a_dining_set_with_its_own_chairs`.
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4; wave: office/kids (`check.mjs play --objects-file gaps.txt`)

### I-office-02 — The executive desk's drawer has no use

> Office/kids wave: confirmed. The desk has no use of its own; a desk chair's `write` at it doesn't open the drawer.

- **Severity:** S3
- **Rubric:** I1, I7
- **Fix class:** decision
- **Where:** `tools/art/blender/families/office.py:524-529` (the middle drawer, a still part of its own), `web/public/content/catalog2/office.json:111` (`deskExecutive`, `interactions: []`)
- **Evidence:** A modelled part with no clip, left decorative for 0.3.0: its clip would need a seated use on the desk it faces. Wave: `office/play/pair-chairExecutive/chairExecutive-write-01.png`, the drawer stays shut while the executive chair's `write` runs at the desk. The writing desk's drawer, by contrast, slides out to the sitter (`drawer_open`).
- **Status:** won't fix (Q-19: the drawer stays a still part)
- **After:** Decor by decision.
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5; wave: office/kids

### M-office-01 — The corner desk is a square slab with one corner cut, not an L

- **Severity:** S2
- **Rubric:** M6, M3, M5
- **Fix class:** model
- **Where:** `tools/art/blender/families/office.py:448-466` (`deskCorner`: `_slab('top', [(-e, -e), (e, -e), (e, 0.33), (0.33, e), (-e, e)], …)` at `:452`, `e` 0.93)
- **Evidence:** `office/sheets/obj-deskCorner.png`: the top covers the whole 1.86 × 1.86 m footprint with only the front-right corner chamfered (0.6 m), so it reads as a white island or a conference table from the game camera and in play; the build's own comment calls it "an L". A corner desk is two wings 0.6–0.8 m deep meeting in a corner, the sitter in the inner corner with both wings in reach (reference `deskCorner`: overall 1.2–1.8 m each way; the depth of each wing isn't in the range, but its use, "work with two screens, spread out papers", needs the reach). Here the middle of the top is 0.9 m and more from any edge. `office/extra/gaps-chairTask-9-13-2-write-01.png`: a desk chair facing the left half meets the oak front panel (`panel_front`, `:457`) and writes with the hands on the edge, the knees at the panel. Top height 0.74 m is right.
- **Status:** fixed (e8fa632e)
- **After:** an L of two 0.7 m wings along the back and the left of its 2 × 2 tiles, the inner corner cut on the diagonal, a drawer pedestal under each wing, and its own swivel chair in the inner corner facing the corner (content `places` `[0.11, 0.11]` facing -135, `seat` 0.44, uses write and homework as at the writing desk; `surface` the back wing, 1.86 × 0.7). The writer sits in the corner with a hand on each wing (`bench-results/review/fixes/blender-hobbies/play/top-deskCorner-write.png` from above, `adult-deskCorner-write-01.png`, `child-deskCorner-homework-01.png`). A chair outside its tiles can't reach an L's inner corner, so generated studies take it on its own, not as a desk set with a chair (`housegen.json`). `cargo review sizes` counts the chair in its height (1.01, the built-in chairs' known overcount, §7).
- **Source:** wave: office/kids

### I-office-03 — The office chair can only "Sit down": it can't be worked at a desk

- **Severity:** S2
- **Rubric:** I1, M11
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:3144-3174` (`officeChair`: `seatAt: ["desk", "table"]` at `:3155`, but only the `sit` interaction), against `web/public/content/catalog2/office.json:240-315` (`chairTask`'s `write` and `homework`, `needsTable`); `web/public/content/housegen.json:76` and `:195` (generated homes put the office chair at a `deskSimple` or `deskCorner` as its chair)
- **Evidence:** `office/extra/gaps-officeChair-9-13-2-sit-01.png`: the office chair pulled up to the simple desk offers only "Sit down"; the resident sits with her hands in her lap. The other two desk chairs (`chairTask`, `chairExecutive`) have "Write at the desk" and "Do homework at the desk" there. The desks of the second catalogue (`deskSimple`, `deskCorner`, `deskExecutive`) have no uses of their own, so a generated home that gave its desk an office chair (half the time, `housegen.json:76`, `:195`) has a desk nobody can work at. An office chair's one real use is sitting at a desk.
- **Status:** fixed (eaca9ed8)
- **After:** Write and homework at a desk it faces (`needsTable`), as the desk chairs: played at the simple desk (`bench-results/review/fixes/content/g09/after-officeChair-write-00.png`).
- **Source:** wave: office/kids

### I-office-04 — Planning on the whiteboard is done from 1.4 m away, the hands in the air

- **Severity:** S2
- **Rubric:** I2, I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:193` (`STEP_IN`: no entry for the whiteboard, so the user stays on the tile in front); `characters.ts:480` (`ACTIONS.paint`: `spell`, `interact`, hands at chest height); `web/public/content/catalog2/office.json:317-346` (`whiteboard`, wall layer, `plan`); `tools/art/blender/families/office.py:575-579` (the board 1.11–1.69 m up on the wall's face)
- **Evidence:** `office/play/c1/whiteboard-plan-01.png`: the sim stands the resident at (10.5, 16.5) and the board's face is at z 17.92 (`office/play/c1/play.jsonl`), 1.4 m away; she makes the `paint` strokes at chest height in the air while the plan draws itself on the board. The child does the same, lower (`office/extra/child-whiteboard-plan-01.png`). Writing on a wall board is done at arm's length (about 0.4–0.5 m), the hand on the board at shoulder to head height. A `STEP_IN` entry (step up to the board, `to` about 0.45 m from the wall) with a raised-arm clip would place it; the clip is the `clip` half of the fix.
- **Status:** fixed (0c5e889e, df11b893)
- **After:** `STEP_IN.whiteboard`: stepped up to 0.5 m from the board's face, the paint strokes at its foot (`fixes/tables/play-a/a-whiteboard-plan-01.png`). The raised-arm stroke at shoulder to head height is the clip half (resident-clip wave). Then (fix: clips-reach) `writeBoard177`/`147`/`112` (clips/reach.ts, `STEP_IN` 0.39–0.27 m from the face by size): three lines written on the board at shoulder to head height (1.52–1.34 m for a grown-up, 1.3–1.16 m for a child), the marker off it between words, the weight and the head going with the hand (`fixes/clips-reach/play-a/a-whiteboard-plan-01.png`, `play-teen/`, `play-child/`, `play-elder/`; `compare/I-office-04.png`).
- **Source:** wave: office/kids

### I-office-05 — The gaming PC's desk has no surface: nothing can be put on it

- **Severity:** S3
- **Rubric:** I1, M11
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:3074` (`computerGaming`: no `surface`), against `furniture.json:2926` and `:2989` (the writing and laptop desks' `surface`, `kind: "desk"`)
- **Evidence:** `cargo review sizes` measures its top at 0.75 m ("no content `surface`", `tables/sizes-office.md`); its siblings, the computer, writing and laptop desks, declare one, so a lamp, a mug or a plant can stand on them but not on the gaming desk (1.7 × 0.44 m of top, the monitors at the back).
- **Status:** fixed (eaca9ed8)
- **After:** A `desk` surface, 1.70 × 0.44 m at 0.75 m (at [0, 0.24]), as the other computer desks'.
- **Source:** wave: office/kids
