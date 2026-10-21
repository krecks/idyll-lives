# Bedroom: authenticity ledger

What it covers: the 27 objects of [docs/objects/bedroom.md](../objects/bedroom.md), in its groups and order.

- **Evidence:** `bench-results/review/bedroom/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## BR1. Beds (9)

Family: `tools/art/blender/families/bedroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bed` Double bed | M-bedroom-01 | OK | OK | M-bedroom-01 S3 decision | `sheets/obj-bed.png`, `play/bed-*.png`, `clips/clips-bed-all-t*.png`, `sheets/bedpose-0.png`, `play-child/bed-*.png`, `play-elder/bed-*.png` |
| `bedSingle` Single bed | — | — | — | | |
| `bedBunk` Bunk bed | — | — | — | | |
| `bedFuton` Futon bed | — | — | — | | |
| `bedKing` Upholstered king bed | — | — | — | | |
| `bedDouble` Double bed | — | — | — | | |
| `bedQueenUpholstered` Upholstered queen bed | — | — | — | | |
| `bedCanopy` Four-poster bed | — | — | — | | |
| `bedDaybed` Daybed | — | — | — | | |

## BR2. Nightstands and small storage (8)

Family: `tools/art/blender/families/bedroom_storage.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `nightstand` Nightstand | — | — | — | | |
| `nightstandModern` Modern nightstand | — | — | — | | |
| `nightstandRound` Drum nightstand | — | — | — | | |
| `nightstandDrawers` Three-drawer nightstand | — | — | — | | |
| `nightstandLamp` Nightstand with lamp | — | — | — | | |
| `cedarChest` Blanket chest | — | — | — | | |
| `laundryHamper` Laundry hamper | — | — | — | | |
| `benchBedEnd` Bed-end bench | — | — | — | | |

## BR3. Wardrobes and drawers (6)

Family: `tools/art/blender/families/bedroom_storage.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `dresser` Chest of drawers | — | — | — | | |
| `wardrobe` Wardrobe | — | — | — | | |
| `chestOfDrawers` Tall chest of drawers | — | — | — | | |
| `dresserLow` Low dresser | — | — | — | | |
| `wardrobeSingle` Single wardrobe | — | — | — | | |
| `armoire` Armoire | — | — | — | | |

## BR4. Dressing and mirrors (4)

Family: `tools/art/blender/families/bedroom_dressing.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `mirror` Standing mirror | OK | OK | OK | | `sheets/obj-mirror.png`, `play/mirror-speech-*.png`, `gallery-all/gallery-mirror.png` |
| `dressingTable` Dressing table | — | — | — | | |
| `mirrorWall` Wall mirror | — | — | — | | |
| `mirrorFloor` Standing mirror | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows; durations are judged by §9 decision 2.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bed` | sleep, nap, read in bed, sit on the edge | sleep (480), take a nap (30); two slots | OK (reading in bed would be new; not a clear gap) |
| `mirror` | check your outfit, practise a speech | practise a speech (30) | OK (the outfit change is the wardrobe's) |

## Findings

### M-bedroom-01 — The "Double bed" is king-size

- **Severity:** S3
- **Rubric:** M1, M11
- **Fix class:** decision
- **Where:** `tools/art/blender/families/bedroom.py:153-161` (`bed`: headboard 1.90 m, base 1.82, mattress 1.72 × 1.78); `web/public/content/base.json:2479` (`name: "Double bed"`)
- **Evidence:** `sizes-bedroom.md`: 1.90 × 1.13 × 1.88 against a double's 1.35–1.60 m (±10 %: 1.76). Its 1.72 m mattress is a king's (`bedKing` is 1.98 wide, the other `bedDouble` 1.62 with a 1.56 mattress). It fills its 2 × 2 footprint, which may be why. Either call it a king (content name and `reference.json` kind) or narrow it to a double; the mattress height 0.58 m is right (`BEDS.bed`, `furniture.json:53`) and sleepers lie well on it (`play/bed-sleep-02.png`). Its 1.78 m long mattress is short for 1.90–2.00 m beds, as for every bed in the family (`bedroom.py:161`, `:190`, `:322`).
- **Status:** open
- **Source:** pilot
