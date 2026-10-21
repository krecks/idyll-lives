# Hobbies: authenticity ledger

What it covers: the 19 objects of [docs/objects/hobbies.md](../objects/hobbies.md), in its groups and order.

- **Evidence:** `bench-results/review/hobbies/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## H1. Music (6)

Family: `tools/art/blender/families/hobbies.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `piano` Piano | OK | OK | OK | | `sheets/obj-piano.png`, `play/piano-play-*.png`, `clips/clips-piano-all-t*.png`, `sheets/seat-pianoM.png`, `sheets/seat-pianoF.png`, `play-child/piano-play-*.png` |
| `ukulele` Ukulele | — | — | — | | |
| `guitarElectric` Electric guitar | — | — | — | | |
| `keyboardDigital` Digital piano | — | — | — | | |
| `drumKit` Drum kit | — | — | — | | |
| `guitarAcoustic` Acoustic guitar | — | — | — | | |

## H2. Games (6)

Family: `tools/art/blender/families/hobbies.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `chessTable` Chess table | M-hobbies-02 | OK | I-hobbies-01 | M-hobbies-02 S3 model; I-hobbies-01 S3 decision | `sheets/obj-chessTable.png`, `play/chessTable-chess-*.png`, `clips/clips-chessTable-all-t*.png`, `sheets/seat-chessM.png`, `sheets/seat-chessF.png` |
| `poolTable` Pool table | — | — | — | | |
| `foosball` Foosball table | — | — | — | | |
| `dartboard` Dartboard | — | — | — | | |
| `arcadeCabinet` Arcade cabinet | — | — | — | | |
| `puzzleTable` Jigsaw table | — | — | — | | |

## H3. Crafts and science (7)

Family: `tools/art/blender/families/hobbies.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `easel` Easel | OK | OK | I-held-01 | I-held-01 S2 code (seeded; held ledger: no brush) | `sheets/obj-easel.png`, `play/easel-paint-*.png`, `clips/clips-easel-all-t*.png`, `play-child/easel-paint-*.png` |
| `workbench` Workbench | M-hobbies-01 | OK | I-bathroom-04 | M-hobbies-01 S2 model; I-bathroom-04 S3 code (bathroom ledger: no sound) | `sheets/obj-workbench.png`, `play/workbench-tinker-*.png`, `clips/clips-workbench-all-t*.png`, `play-child/workbench-tinker-*.png` |
| `telescope` Telescope | OK | I-hobbies-02 | I-hobbies-02, I-hobbies-03 | I-hobbies-02 S2 clip; I-hobbies-03 S2 code | `sheets/obj-telescope.png`, `play/telescope-stargaze-*.png`, `clips/clips-telescope-all-t*.png`, `play-child/telescope-stargaze-*.png` |
| `sewingTable` Sewing table | — | — | — | | |
| `chemistrySet` Chemistry bench | — | — | — | | |
| `potteryWheel` Pottery wheel | — | — | — | | |
| `craftTable` Craft table | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows; durations are judged by §9 decision 2.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `piano` | play the piano, practise scales | play the piano (60) | OK |
| `chessTable` | play chess or a board game with someone, solve a chess problem alone | practise chess (60, one slot) | the game for two missing (I-hobbies-01) |
| `easel` | paint a canvas | paint (60) | OK |
| `workbench` | tinker and repair things, build something | tinker (60) | OK |
| `telescope` | look at the stars and the moon, at night, outdoors or at a window | stargaze (45, any hour, anywhere) | the night and the sky missing (I-hobbies-03); 45 min is 3.6× the `look` median of 12.5, but that tag mixes a glance at a painting with an evening's stargazing: inside the real 20–120, not filed |

## Findings

### M-hobbies-01 — The workbench is 7 cm low and built like a steel office desk

- **Severity:** S2
- **Rubric:** M2, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:1098-1101` (`workbench`: the top at 0.762 + 0.0175 = 0.78 m, on two drawer pedestals with a kneehole)
- **Evidence:** `sheets/obj-workbench.png`, `play/workbench-tinker-01.png`: a worker stands at a 0.78 m top (a workbench is 0.85–0.95 m, `reference.json` `workbench`: you work standing, over the top). The twin steel pedestals with a kneehole and drawer pulls read as a teacher's desk with a drill and a vice on it; a workbench has an open frame or a cupboard base, a thick top and often a tool board. The sizes table can't measure the top ("no top to measure": the content has no `surface`); the height here is from the family script and the shot.
- **Status:** open
- **Source:** pilot

### M-hobbies-02 — The chess table's top is 46 × 40 cm, with one chair

- **Severity:** S3
- **Rubric:** M1
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:707` (`top` 0.46 × 0.40 at 0.725 m), `:719` (one `_chair`)
- **Evidence:** `sizes-hobbies.md`: W 0.46 < 0.54 (0.60–1.60 ±10 %). The board (0.37 m) nearly fills a pedestal top that leaves no room for taken pieces or a second player's elbows (`sheets/obj-chessTable.png`). Its top height (0.74 m) and seat (0.475 m pad, `SEATS.chessTable` 0.46) are fine.
- **Status:** open
- **Source:** pilot

### I-hobbies-01 — The chess table seats one: chess is a game for two

- **Severity:** S3
- **Rubric:** I1, I2
- **Fix class:** decision
- **Where:** `web/public/content/base.json:3013` (`chessTable`: no `slots`, one use "Practise chess"); compare `web/public/content/community.json` `parkChessTable` (`slots: 2`, a stool either side)
- **Evidence:** `play/chessTable-chess-01.png`: one resident at the board, moving both sides' pawns (`move`). Practising alone is a real thing (problems, openings), and the label says so; but a chess table at home is mostly for playing someone, which the park's table already does. Decide whether the home one gets a second seat and a two-player use (model and content) or stays a training object.
- **Status:** open
- **Source:** pilot

### I-hobbies-02 — The stargazer stands upright beside the telescope, half a metre above the eyepiece

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:523-526` (`ACTIONS.look`: `rail` for `telescope`); the eyepiece at 0.975 m (`tools/art/blender/families/hobbies.py:1202`)
- **Evidence:** `play/telescope-stargaze-01.png`, `sheets/obj-telescope.png`: the resident stands straight beside the tube with the hands at the chest (`rail`, a hold on a railing); the eye is about 1.55 m up, the eyecup at 0.975 m, and nobody bends to it. For a child (`play-child/telescope-stargaze-01.png`) the eyecup is about at eye height, by chance. A stoop to the eyepiece with a hand on the tube is what reads as looking through a telescope.
- **Status:** open
- **Source:** pilot

### I-hobbies-03 — Stargazing at noon, indoors

- **Severity:** S2
- **Rubric:** I1, I6
- **Fix class:** code
- **Where:** `web/public/content/base.json:3256` (`telescope`, `stargaze`: `tags ["training", "outdoors"]`, no hour or sky rule); `crates/sim-core/src/content.rs:70` (`Interaction` has no time-of-day field)
- **Evidence:** `play/telescope-stargaze-01.png`: "Stargaze" at 12:02, in a closed room, the tube aimed at the ceiling. Its `outdoors` tag works against it: the night schedule (`base.json:1797-1809`, 22:00–06:00) weights `outdoors` uses at 0.1, so free will is least likely to stargaze at night. Stars are a night thing seen through open sky or a window; a use that only runs after dark (and outdoors or by a window) is new behaviour, so filed as `code`.
- **Status:** open
- **Source:** pilot
