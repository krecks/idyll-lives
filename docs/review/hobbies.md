# Hobbies: authenticity ledger

What it covers: the 19 objects of [docs/objects/hobbies.md](../objects/hobbies.md), in its groups and order.

- **Evidence:** `bench-results/review/hobbies/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Wave rows** (activities wave, hobbies and fitness, 2026-10-21): Shots are paths under `bench-results/review/` in the review worktree (`hobbies/…`, `fitness/…`); the shots this wave took (front, side and top views, two residents at once, a child, night) are in `hobbies/extra/` (composites `hobbies/extra/*.png` at its top level), how in `hobbies/review-log.md`. The wave also added its findings to the pilot rows they reach (chess, piano).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## H1. Music (6)

Family: `tools/art/blender/families/hobbies.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `piano` Piano | OK | A-hobbies-02 | OK | A-hobbies-02 S3 motion (wave: the fallboard lifts while the pianist is still sitting down) | `sheets/obj-piano.png`, `play/piano-play-*.png`, `clips/clips-piano-all-t*.png`, `sheets/seat-pianoM.png`, `sheets/seat-pianoF.png`, `play-child/piano-play-*.png` |
| `ukulele` Ukulele | OK | I-hobbies-08 | I-hobbies-08 | I-hobbies-08 S2 clip | `hobbies/sheets/obj-ukulele.png`, `hobbies/extra/inst-ukulele-strum-01.png` |
| `guitarElectric` Electric guitar | M-hobbies-03 | I-hobbies-08 | I-hobbies-08 | M-hobbies-03 S3 model; I-hobbies-08 S2 clip | `hobbies/sheets/obj-guitarElectric.png`, `hobbies/extra/inst-guitarElectric-jam-01.png` |
| `keyboardDigital` Digital piano | M-hobbies-04 | A-hobbies-01 | A-hobbies-01 | M-hobbies-04 S2 model; A-hobbies-01 S2 code | `hobbies/sheets/obj-keyboardDigital.png`, `hobbies/extra/kb-side2.png`, `hobbies/play/c1/keyboardDigital-play-00.png` |
| `drumKit` Drum kit | M-hobbies-05 | A-hobbies-01, A-hobbies-02 | A-hobbies-01, I-hobbies-09, I-held-01 | M-hobbies-05 S3 model; A-hobbies-01 S2 code; A-hobbies-02 S3 motion; I-hobbies-09 S3 code (the drums pluck a tune); I-held-01 S2 code (held ledger: no sticks) | `hobbies/sheets/obj-drumKit.png`, `hobbies/extra/side2-drumKit-drum-01.png`, `hobbies/play/c1/drumKit-drum-00.png` |
| `guitarAcoustic` Acoustic guitar | M-hobbies-03 | I-hobbies-08 | I-hobbies-08 | M-hobbies-03 S3 model; I-hobbies-08 S2 clip | `hobbies/sheets/obj-guitarAcoustic.png`, `hobbies/extra/gsidetop.png`, `hobbies/extra/gdef.png` |

Wave notes on the rows:
- The guitars and the ukulele leave the stand at once (`off_stand`, 0.2 s) and are drawn in the hands with the left hand on the neck and the right over the sound hole, strumming (`writeHeld`, `activityPose` `music`, `characters.ts:1756`, `:3339`); a child plays one scaled to their size (`hobbies/extra/child/child-guitarAcoustic-strum-01.png`). The motions are keyed by the anim `music`, not the use ids: the README allows either, and `off_stand` plays for every use. The lead "the neck points up past the head" is I-hobbies-08: from the game camera behind the player it does, because the guitar sticks out forward.
- `ukulele`: 0.65 m long (a tenor), on a 0.62 m stand: in range.
- `keyboardDigital`: static (no key cover); the bench at 0.45 m is inside the 1 cm tolerance of 0.46. The hands are on the keys (`sitHands`, `ACTIONS.music`, `characters.ts:479`).
- `drumKit`: every part is there (kick with pedal, two rack toms, snare, floor tom, hi-hat, crash and ride, throne at 0.53 m against the content's 0.52); the cymbals rock and the forearms strike in turn (`characters.ts:3306`).

### Real uses against the content's (I1)

Durations are judged by §9 decision 2: none here is more than 3× from its tag's median except the telescope's (pilot note), so all are by design.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `piano` | play the piano, practise scales | play the piano (60) | OK |
| `ukulele` | strum and sing along, practise chords | strum (30), practise (60) | OK |
| `guitarElectric` | play riffs through the amp, practise | jam (45), practise (60) | OK |
| `keyboardDigital` | play, practise with headphones | play (60, seated) | OK |
| `drumKit` | play the drums, practise a beat | drum (30), practise (60) | OK (no sticks: I-held-01) |
| `guitarAcoustic` | strum and sing along, practise chords | strum (30), practise (60) | OK |

## H2. Games (6)

Family: `tools/art/blender/families/hobbies.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `chessTable` Chess table | M-hobbies-02, M-hobbies-09 | A-hobbies-01, A-hobbies-02 | I-hobbies-01, A-hobbies-01 | M-hobbies-02 S3 model; I-hobbies-01 S3 decision; wave: M-hobbies-09 S3 renderer-table (seat 1.5 cm under the pad), A-hobbies-01 S2 code (walks through the table), A-hobbies-02 S3 motion (the pawn moves itself) | `sheets/obj-chessTable.png`, `play/chessTable-chess-*.png`, `clips/clips-chessTable-all-t*.png`, `sheets/seat-chessM.png`, `sheets/seat-chessF.png`, `hobbies/extra/sit00b.png` |
| `poolTable` Pool table | M-hobbies-06 | A-hobbies-03 | I-hobbies-04, I-hobbies-05, I-hobbies-07, I-hobbies-09 | M-hobbies-06 S3 model; A-hobbies-03 S2 motion; I-hobbies-04 S2 clip; I-hobbies-05 S2 code; I-hobbies-07 S1 code; I-hobbies-09 S3 code | `hobbies/sheets/obj-poolTable.png`, `hobbies/extra/two.png`, `hobbies/clips/clips-poolTable-all-t99.png` |
| `foosball` Foosball table | M-hobbies-07 | OK | I-hobbies-01, I-hobbies-04, I-hobbies-07, I-hobbies-09 | M-hobbies-07 S3 model; I-hobbies-01 S3 decision (widened: a game for two played alone); I-hobbies-04 S2 clip; I-hobbies-07 S1 code; I-hobbies-09 S3 code | `hobbies/sheets/obj-foosball.png`, `hobbies/extra/two.png`, `hobbies/extra/two-foosball-v1.png` |
| `dartboard` Dartboard | OK | I-hobbies-04 | I-hobbies-04, I-hobbies-06, I-hobbies-09 | I-hobbies-04 S2 clip (no throw: the darts appear in the board); I-hobbies-06 S3 code; I-hobbies-09 S3 code | `hobbies/sheets/obj-dartboard.png`, `hobbies/extra/side-dartboard-darts-00.png` |
| `arcadeCabinet` Arcade cabinet | OK | OK | I-hobbies-09 | I-hobbies-09 S3 code (no game sounds) | `hobbies/sheets/obj-arcadeCabinet.png`, `hobbies/extra/night.png` |
| `puzzleTable` Jigsaw table | OK | A-hobbies-01, A-hobbies-02 | A-hobbies-01 | A-hobbies-01 S2 code; A-hobbies-02 S3 motion | `hobbies/sheets/obj-puzzleTable.png`, `hobbies/extra/sit00b.png`, `hobbies/sheets/age-puzzleTable.png` |

Wave notes on the rows:
- `poolTable`: an 8 ft table (2.5 × 1.4 m, the cloth 2.3 × 1.2) with six pockets, sights, fifteen balls racked, a cue ball, a cue and chalk; legs reach the floor.
- `foosball`: 1.42 × 0.95 × 0.94 m, the pitch at 0.71 and the rods at 0.84 m (playing height about 0.9); two of the four rods kick ±55° in a loop.
- `dartboard`: the bull at 1.725 m (the rule's 1.73), a 0.45 m board in a cabinet; it hangs on the wall's face (`hobbies/sheets/obj-dartboard.png`).
- `arcadeCabinet`: 0.67 × 1.82 × 0.87 m, the panel at 0.94 m under the player's hands, the screen and marquee glow at night and not by day (`hobbies/extra/night.png`); the joystick waggles in a loop. A careful look finds nothing wrong.
- `puzzleTable`: top 0.755 m, stool 0.48 m (content 0.48), the board half done; the patch spreads across in use. Doing a jigsaw alone is normal (no I16).

### Real uses against the content's (I1)

Durations are judged by §9 decision 2: none here is more than 3× from its tag's median except the telescope's (pilot note), so all are by design.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `chessTable` | play chess or a board game with someone, solve a chess problem alone | practise chess (60, one slot) | the game for two missing (I-hobbies-01) |
| `poolTable` | play a game of pool with someone, practise shots alone, rack and break | shoot pool (30), trick shots (60), two slots | OK as uses; the cue is never picked up (I-hobbies-05) and two players stand in one spot (I-hobbies-07) |
| `foosball` | a quick game for two or four, one team each side | play foosball (20), two slots, played alone | the game for two missing (I-hobbies-01, widened); both players on one spot (I-hobbies-07) |
| `dartboard` | throw darts from the oche, a game of 501 with someone | throw darts (15) | OK as a use; thrown from 1.4 m (I-hobbies-06) |
| `arcadeCabinet` | play a round, chase the high score | play arcade games (30) | OK |
| `puzzleTable` | work on a jigsaw, alone or with someone | do a jigsaw (40) | OK |

## H3. Crafts and science (7)

Family: `tools/art/blender/families/hobbies.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `easel` Easel | OK | OK | I-held-01 | I-held-01 S2 code (seeded; held ledger: no brush) | `sheets/obj-easel.png`, `play/easel-paint-*.png`, `clips/clips-easel-all-t*.png`, `play-child/easel-paint-*.png` |
| `workbench` Workbench | M-hobbies-01 | OK | I-bathroom-04 | M-hobbies-01 S2 model; I-bathroom-04 S3 code (systems ledger: no sound) | `sheets/obj-workbench.png`, `play/workbench-tinker-*.png`, `clips/clips-workbench-all-t*.png`, `play-child/workbench-tinker-*.png` |
| `telescope` Telescope | OK | I-hobbies-02 | I-hobbies-02, I-hobbies-03 | I-hobbies-02 S2 clip; I-hobbies-03 S2 code | `sheets/obj-telescope.png`, `play/telescope-stargaze-*.png`, `clips/clips-telescope-all-t*.png`, `play-child/telescope-stargaze-*.png` |
| `sewingTable` Sewing table | M-hobbies-08, M-hobbies-09 | A-hobbies-01, A-hobbies-02 | A-hobbies-01, I-hobbies-09 | M-hobbies-08 S3 model; M-hobbies-09 S3 renderer-table; A-hobbies-01 S2 code; A-hobbies-02 S3 motion; I-hobbies-09 S3 code (a silent machine) | `hobbies/sheets/obj-sewingTable.png`, `hobbies/extra/side2-sewingTable-sew-01.png`, `hobbies/extra/sit00.png` |
| `chemistrySet` Chemistry bench | OK | I-hobbies-11 | I-hobbies-11 | I-hobbies-11 S3 renderer-table | `hobbies/sheets/obj-chemistrySet.png`, `hobbies/extra/side1.png`, `hobbies/extra/stand3.png` |
| `potteryWheel` Pottery wheel | OK | A-hobbies-01, A-hobbies-02, I-hobbies-10 | A-hobbies-01, I-hobbies-10, I-hobbies-09 | A-hobbies-01 S2 code; A-hobbies-02 S3 motion; I-hobbies-10 S2 model; I-hobbies-09 S3 code (no motor hum) | `hobbies/sheets/obj-potteryWheel.png`, `hobbies/extra/side3.png`, `hobbies/extra/sit00.png` |
| `craftTable` Craft table | OK | A-hobbies-01, A-hobbies-02 | A-hobbies-01 | A-hobbies-01 S2 code; A-hobbies-02 S3 motion | `hobbies/sheets/obj-craftTable.png`, `hobbies/extra/sit00.png`, `hobbies/sheets/age-craftTable.png` |

Wave notes on the rows:
- `sewingTable`: top 0.7525 m (0.72–0.76), the machine with bed, pillar, arm, head, needle, spool, handwheel; cloth, reels and a basket. D 0.89 m is the table and its chair: the table is 0.46 deep. The sewer's hands are at the machine (`hobbies/extra/side2-sewingTable-sew-01.png`).
- `chemistrySet`: 0.92 × 1.40 × 0.50 m, the top at 0.90 m (0.85–0.95), a back board with test tubes, three flasks, a beaker, a microscope and notes; the middle flask fills and bubbles. Its microscope isn't used: one use, "do experiments", covers the bench (not filed).
- `potteryWheel`: the stool at 0.44 m (content 0.45), the wheel head at 0.475 m, a pedal, sponge and wire; the head spins in a loop and the pot rises from a lump. The hands are on the pot (`hobbies/extra/side3.png`).
- `craftTable`: a 1.7 m birch top at 0.7525 m on hairpin legs, the stool at 0.48 m; the scrapbook cover swings open under the hands.

### Real uses against the content's (I1)

Durations are judged by §9 decision 2: none here is more than 3× from its tag's median except the telescope's (pilot note), so all are by design.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `easel` | paint a canvas | paint (60) | OK |
| `workbench` | tinker and repair things, build something | tinker (60) | OK |
| `telescope` | look at the stars and the moon, at night, outdoors or at a window | stargaze (45, any hour, anywhere) | the night and the sky missing (I-hobbies-03); 45 min is 3.6× the `look` median of 12.5, but that tag mixes a glance at a painting with an evening's stargazing: inside the real 20–120, not filed |
| `sewingTable` | sew a project, mend and alter clothes | sew (60), mend (30) | OK |
| `chemistrySet` | run an experiment, look through the microscope | do experiments (60, standing) | OK (the microscope has no use of its own; not filed) |
| `potteryWheel` | throw a pot, trim it | throw a pot (60, seated) | OK |
| `craftTable` | scrapbook, cut and glue, draw | make a scrapbook (45, seated) | OK |

## Findings

### M-hobbies-01 — The workbench is 7 cm low and built like a steel office desk

- **Severity:** S2
- **Rubric:** M2, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:1098-1101` (`workbench`: the top at 0.762 + 0.0175 = 0.78 m, on two drawer pedestals with a kneehole)
- **Evidence:** `sheets/obj-workbench.png`, `play/workbench-tinker-01.png`: a worker stands at a 0.78 m top (a workbench is 0.85–0.95 m, `reference.json` `workbench`: you work standing, over the top). The twin steel pedestals with a kneehole and drawer pulls read as a teacher's desk with a drill and a vice on it; a workbench has an open frame or a cupboard base, a thick top and often a tool board. The sizes table can't measure the top ("no top to measure": the content has no `surface`); the height here is from the family script and the shot.
- **Status:** fixed (8b6b2364)
- **After:** a 5 cm butcher-block top at 0.90 m on an open frame of square posts with rails and a slatted shelf (a crate of offcuts, a paint can), the back posts carrying a hardboard tool board with a saw, a square, screwdrivers, a spanner, pliers, a mallet and a tape (`hobbies.py` `workbench`, `BENCH_TOP`); `cargo review sizes`: top 0.90, inside 0.85–0.95 (`crates/tools/tests/checkglb.rs` now expects it). `bench-results/review/fixes/blender-hobbies/gallery/gallery-workbench.png`, `play/adult-workbench-tinker-01.png`, `play/child-workbench-tinker-01.png`.
- **Source:** pilot

### M-hobbies-02 — The chess table's top is 46 × 40 cm, with one chair

- **Severity:** S3
- **Rubric:** M1
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:707` (`top` 0.46 × 0.40 at 0.725 m), `:719` (one `_chair`)
- **Evidence:** `sizes-hobbies.md`: W 0.46 < 0.54 (0.60–1.60 ±10 %). The board (0.37 m) nearly fills a pedestal top that leaves no room for taken pieces or a second player's elbows (`sheets/obj-chessTable.png`). Its top height (0.74 m) and seat (0.475 m pad, `SEATS.chessTable` 0.46) are fine.
- **Status:** fixed (8b6b2364)
- **After:** a 0.64 × 0.48 m top (the board 0.37 in its middle) with two taken pieces each side, a wider apron and foot; the top 0.74 and the seat 0.475 as they were (`cargo review sizes`: in range). Still one chair: a partner across the board needs a second place and a deeper footprint (G-65). 4,400 triangles (`check-glb` warns: the aim is 4,000 at 0.92 m). `fixes/blender-hobbies/gallery/gallery-chessTable.png`, `play/adult-chessTable-chess-01.png`, `poses/poses-seats-*.png`.
- **Source:** pilot

### I-hobbies-01 — The chess table seats one: chess is a game for two

> Wave (hobbies): confirmed and widened to the foosball table, a game for two or four played alone (Rubric I16 added: someone to play with).
>
> Consolidation: the park chess table's "stool either side" stands side by side (I-community-02, merged into I-outdoor-07, systems ledger).

- **Severity:** S3
- **Rubric:** I1, I2, I16
- **Fix class:** decision
- **Where:** `web/public/content/base.json:3013` (`chessTable`: no `slots`, one use "Practise chess"); compare `web/public/content/community.json` `parkChessTable` (`slots: 2`, a stool either side); `web/public/content/furniture.json:3377` (`foosball`: `slots: 2`, one use "Play foosball", no partner)
- **Evidence:** `play/chessTable-chess-01.png`: one resident at the board, moving both sides' pawns (`move`). Practising alone is a real thing (problems, openings), and the label says so; but a chess table at home is mostly for playing someone, which the park's table already does. Decide whether the home one gets a second seat and a two-player use (model and content) or stays a training object.
  Wave: the foosball table has two slots but no game between them: each resident plays alone, working both teams' rods from one side (`hobbies/play/c1/foosball-foosball-01.png`; with two, `hobbies/extra/two.png`, they also stand in one spot, I-hobbies-07, and the model has every handle on the front, M-hobbies-07). Solo foosball isn't a real pastime the way a chess problem or trick shots are; the pool table's trick shots and the jigsaw are fine alone. Content has no way to pair two users into one game (no partner field on an interaction, `crates/sim-core/src/content.rs:70`), so a real fix is `code` either way; the decision is whether to keep these as solo uses until then.
- **Status:** confirmed and widened (wave: hobbies, to `foosball`)
- **Source:** pilot; wave: hobbies

### I-hobbies-02 — The stargazer stands upright beside the telescope, half a metre above the eyepiece

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:523-526` (`ACTIONS.look`: `rail` for `telescope`); the eyepiece at 0.975 m (`tools/art/blender/families/hobbies.py:1202`)
- **Evidence:** `play/telescope-stargaze-01.png`, `sheets/obj-telescope.png`: the resident stands straight beside the tube with the hands at the chest (`rail`, a hold on a railing); the eye is about 1.55 m up, the eyecup at 0.975 m, and nobody bends to it. For a child (`play-child/telescope-stargaze-01.png`) the eyecup is about at eye height, by chance. A stoop to the eyepiece with a hand on the tube is what reads as looking through a telescope.
- **Status:** fixed (5bd8a29c, 1abb80cf)
- **After:** the stargazer steps up to the telescope and bends down to its eyepiece, the right eye at the eyecup of the tube as aimed (0.765 m, 0.3 m out: lower than the 0.975 m at rest), the right hand on the tube, the left on the knee, once a loop looking up at the sky; in five sizes for the user's height (`eyepiece181`…`eyepiece112`, `clips/uses.ts`, `STEP_IN` for `look` only): a grown-up deep in the knees and bent over, a child bent a little. `bench-results/review/fixes/clips-standing/``sheet-tele.png` (man, child, elder from the side), `play/tele-*`, `strips/sheets/eyepiece*`.
- **Source:** pilot

### I-hobbies-03 — Stargazing at noon, indoors

- **Severity:** S2
- **Rubric:** I1, I6
- **Fix class:** code
- **Where:** `web/public/content/base.json:3256` (`telescope`, `stargaze`: `tags ["training", "outdoors"]`, no hour or sky rule); `crates/sim-core/src/content.rs:70` (`Interaction` has no time-of-day field)
- **Evidence:** `play/telescope-stargaze-01.png`: "Stargaze" at 12:02, in a closed room, the tube aimed at the ceiling. Its `outdoors` tag works against it: the night schedule (`base.json:1797-1809`, 22:00–06:00) weights `outdoors` uses at 0.1, so free will is least likely to stargaze at night. Stars are a night thing seen through open sky or a window; a use that only runs after dark (and outdoors or by a window) is new behaviour, so filed as `code`.
- **Status:** fixed (69d0bc33)
- **After:** Interactions take conditions (G-64; `docs/content-packs.md`: `hours`, `sky`, `workOn`), kept by free will and the player's orders alike (`world::unmet`). *Stargaze* is `"hours": [20, 5], "sky": true`: at 10:00 the order is refused, "Ada can only stargaze from 20:00 to 5:00" (`fixes/tops-conditions/play-stars-noon.jsonl`); at 22:00 the telescope in the garden is used, the tube aimed up (`play-stars-night.jsonl`, `stars-night-telescope-10-22-0-stargaze-01.png`), and one in the room is refused, "Ada can only stargaze under open sky: the telescope has to stand outside". Its tags are `training, stargazing` (was `outdoors`), so the night schedule's `outdoors` 0.1 no longer weighs it down: free will stargazes at 22:00 and never at noon (`tests/tops_and_conditions.rs` `free_will_stargazes_only_at_night`). By a window indoors is not offered (open sky only). Paths under `bench-results/review/fixes/tops-conditions/`.
- **Source:** pilot

### M-hobbies-03 — The guitars are a quarter too long, on stands as tall as a child

- **Severity:** S3
- **Rubric:** M1, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:491-515` (`_acoustic`: body to 0.67 m, nut at 1.106, head to about 1.29 m), `:443-465` (`_electric`: nut at 1.036, head to about 1.19 m), `:521` and `:471` (the stands' yokes at 1.0 and 0.82 m)
- **Evidence:** `hobbies/sheets/obj-guitarAcoustic.png`, `hobbies/sheets/obj-guitarElectric.png` (gallery): on its stand the acoustic reaches a woman's shoulder and stands taller than the child; `sizes-hobbies.md` H 1.485 and 1.381 m against 1.32 (`guitarOnStand` 0.95–1.2 ±10 %). A dreadnought is about 1.04 m long with a 0.645 m scale, a solid-body electric about 0.99 m with 0.648; here the scale lengths (bridge to nut) are 0.97 and 0.95 m, so the necks are half as long again, and the frets (spaced for a 0.64 m scale, `_frets`) bunch up the board. The ukulele (0.65 m, a tenor) is in range. Held, the guitar is scaled to the player only below adult size (`writeHeld`, `characters.ts:1773`), so adults hold the long one.
- **Status:** fixed (8b6b2364)
- **After:** a 1.05 m dreadnought (a 0.51 m body 0.40 across the lower bout, the neck meeting it at the 14th fret, a 0.645 m scale) on a stand 1.16 m high with it, and a 0.90 m solid body (a 0.648 m scale) on one 1.00 m high, their feet in the cradle 0.12 m up (`cargo review sizes`: in range). Held, they are long guitars (`characters.ts` `GUITAR.long` 0.9, was 1.15, so both still strum, not the ukulele's short hold). `fixes/blender-hobbies/gallery/gallery-guitarAcoustic.png`, `gallery-guitarElectric.png`, `play/adult-guitar*-*.png`.
- **Source:** wave: hobbies

### M-hobbies-04 — The digital piano's keys are 10 cm higher than the piano's, and half as wide again

- **Severity:** S2
- **Rubric:** M2, M3, M11
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:536-541` (`keyboardDigital`: case top 0.83, keys 0.825–0.845 m, 21 white keys 3.9 cm apart across 0.8 m)
- **Evidence:** `hobbies/extra/kb-side2.png` (side view, seated): the hands rise to the keys at chest height; the base piano's keys are at 0.748 m (`:360`), a digital piano's at 0.72–0.76 m with its bench at 0.46–0.56. Its keys are 3.9 cm wide (a real white key is 2.35 cm, the piano's 2.3), so the 0.9 m case holds three octaves: a "digital piano" (`reference.json` `digitalPiano` 1.3–1.45 m wide, 88 keys) reads as a children's keyboard on an X-stand (`hobbies/sheets/obj-keyboardDigital.png`). `sizes-hobbies.md` W 0.90 < 1.17. A playing height 10 cm out is the size of error §4 rates S2 (a counter 10 cm low).
- **Status:** fixed (8b6b2364)
- **After:** a console digital piano on 2 × 1 (content footprint): 88 keys, 52 white at 2.35 cm (1.22 m of keys, 1.39 m overall), their tops at 0.745 m as the upright's, on two side panels with a pedal board and a music rest, a bench at 0.48 m; it stands with its back on the wall like the upright, the player facing it (content `seat` 0.48, `forward` 0.18, `facing: back`; the `seats` row dropped). The hands on the keys (`fixes/blender-hobbies/play/adult-keyboardDigital-play-01.png`, `child-keyboardDigital-play-01.png`, `poses/poses-seats-*.png`); `cargo review sizes`: in range.
- **Source:** wave: hobbies

### M-hobbies-05 — The drum kit is squeezed onto one tile

- **Severity:** S3
- **Rubric:** M1
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:606-663` (`drumKit`; footprint `[1, 1]` at `web/public/content/catalog2/hobbies.json:5`)
- **Evidence:** `sizes-hobbies.md`: 0.98 × 0.89 m against 1.2–2.0 × 1.0–1.8 (±10 %: W 0.978 < 1.08, D 0.885 < 0.90). The pieces are the right sizes (a 0.5 m kick, 0.24–0.36 m toms and snare) but packed: the toms sit on the kick, the hi-hat and snare crowd the throne and the floor tom sits under the ride (`hobbies/clips/clips-drumKit-all-t99.png`). It reads as a drum kit from the camera; a 2 × 2 footprint would let it spread.
- **Status:** fixed (8b6b2364)
- **After:** spread over 2 × 2 (content footprint, `seat.forward` -0.4): a 22" kick to the front with a 10" and a 12" tom on a mount on it, the snare and the hi-hat at the drummer's left, a 16" floor tom at the right, a 16" crash and a 20" ride on boom stands, 1.73 × 1.24 × 1.17 m (`cargo review sizes`: in range). `fixes/blender-hobbies/gallery/gallery-drumKit.png`, `play/adult-drumKit-drum-01.png`, `clips/clips-drumKit-all-t*.png`.
- **Source:** wave: hobbies

### M-hobbies-06 — The pool table's rails are 3 cm over a regulation table's

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:742` (`BED = 0.785`, the cloth), `:756` (`rail_long` 0.15 high at 0.745: top 0.82 m)
- **Evidence:** the rails' top is 0.82 m and the cloth 0.785 m; a pool table is 0.74–0.79 m to the top of the rail (`reference.json` `poolTable` top 74–79, WPA), so the rail is 3 cm over, the cloth inside. `sizes-hobbies.md` measures 0.77 ("no content `surface`"), the middle of the bed, not the rail a player leans on. Close inspection only.
- **Status:** fixed (8b6b2364)
- **After:** the whole table 3.5 cm lower: the cloth at 0.75 m, the rails' top at 0.785 (`hobbies.py` `BED`, `RAIL_TOP`); the pool clip's cloth follows (`clips/games.ts` `BED` 0.75), the rack hung 3.5 cm lower too. `fixes/blender-hobbies/clips/clips-poolTable-all-t99.png`, `play/adult-poolTable-pool-01.png`.
- **Source:** wave: hobbies

### M-hobbies-07 — The foosball table has four rods, every handle on the front

- **Severity:** S3
- **Rubric:** M6, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:811-819` (`_rod_men`: the handle at depth 0.46, the front, for every rod), `:847-854` (four rods: two still, two moving)
- **Evidence:** `hobbies/clips/clips-foosball-all-t99.png`, `hobbies/extra/two-foosball-v1.png` (top view): four rods of three men, all four handles to the front. A real table has eight rods (goalie, defence, midfield, attack each side) with each team's handles on its own long side, the players facing each other. With handles on one side a second player can't oppose the first (I-hobbies-01, I-hobbies-07).
- **Status:** fixed (8b6b2364)
- **After:** eight rods 15 cm apart, from the left goal blue's goalie, defence, red's attack, the two teams' five midfielders, blue's attack, red's defence and goalie; blue's handles to the front, red's short at the back (inside the wall's face). The two players' lanes are at blue's goalie and defence and at its midfield and attack (`clips/games.ts` `FOOS_LANES` -0.45, 0.075, a clip each with the hands 0.15 and 0.3 m apart: `foosClip`, `FOOS_HANDS`; `characters.ts` `STEP_IN` `square`: they now face the table squarely at their lane, not its middle, which put the hands 27° off the handles). `fixes/blender-hobbies/play/adult-foosball-foosball-01.png`, `clips/clips-foosball-all-t99.png`; `pnpm check:clips` 0 new.
- **Source:** wave: hobbies

### M-hobbies-08 — The sewing machine faces its sewer mirrored: needle on the right, handwheel on the left

- **Severity:** S3
- **Rubric:** M7, M8
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:1216-1217` (`NEEDLE_X = -0.12`, `WHEEL` at x 0.212), `:1233-1235` (pillar at +0.155, head at the needle)
- **Evidence:** the sewer sits on the chair at the back facing +z, so their right is game −x: the needle head is on their right and the pillar and handwheel on their left (`hobbies/extra/side2-sewingTable-sew-01.png`; `hobbies/clips/clips-sewingTable-all-t99.png` shows it from the front, the way a sewer would see it from their chair). On every domestic machine the needle is on the sewer's left and the handwheel on the right, the cloth fed towards them. Mirroring x fixes it.
- **Status:** closed (not a defect)
- **After:** checked in play from behind the sewer (`bench-results/review/fixes/blender-hobbies/behind-sewingTable-sew.png`): the needle and the cloth are on her left, the handwheel on her right, as on a real machine. A resident facing +z has their right at game +x (the corner desk's chair and its sitter, `places`, line up only so); the evidence was read from the front. The model is unchanged.
- **Source:** wave: hobbies

### M-hobbies-09 — The sewing and chess chairs' seats are 1.5 cm under their pads

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters/furniture.json:7` (`chessTable` h 0.46), `:45` (`sewingTable` h 0.46); the pad's top 0.475 m (`tools/art/blender/families/hobbies.py:261`, `_chair`)
- **Evidence:** `sizes-hobbies.md` measures both seats at 0.48 (0.475 in the family script); the renderer sits the resident at 0.46, 1.5 cm into the pad, outside the 1 cm tolerance the owner approved after the pilot (the pilot's M-hobbies-02 found the chess seat "fine" before that). One number each; neither content has a `seat` the heights check could compare.
- **Status:** fixed (a920f174)
- **After:** `seats.chessTable` and `sewingTable` 0.475, the pads' tops (`fixes/tables/play-b/b-chessTable-chess-01.png`, `b-sewingTable-sew-01.png`).
- **Source:** wave: hobbies

### A-hobbies-01 — moved to systems.md

### A-hobbies-02 — At seated hobbies the moving part starts while the resident is still sitting down

- **Severity:** S3
- **Rubric:** A10
- **Fix class:** motion
- **Where:** `tools/art/blender/families/hobbies.py:1452-1494` (`FAMILY` motions: `fallboard`, `move`, `pieces`, `presser`/`handwheel`, `clay`/`wheel`, `book_open`, `cymbals`, none with a `delay`); compare `tools/art/blender/families/fitness.py:276` (`bar_off` with `delay: BAR_DELAY`, 0.8 s, for the same reason)
- **Evidence:** `hobbies/play/c1/play.jsonl` `clips`: each clip's `first` is 0–17 ms after `using`, which for a seated use is when `sitDown` starts (about 1 s; `characters.ts:2585`). So the chess pawn slides (0.8 s) and the jigsaw patch spreads (1.2 s) before a hand is near the board, and the presser foot drops, the pot rises, the scrapbook opens and the cymbals rock while the resident is still lowering onto the seat (and, A-hobbies-01, still passing through the table). A `delay` of about 0.9 s on each, as the weight bench has, puts them at the hands. Close watching only.
- **Status:** fixed (d2c9f0de)
- **After:** each starts 0.9 s in (`SIT`, `delay`), the loops (cymbals, handwheel, wheel) now repeating uses from then (`every` 0.9 s): `fixes/motions/g1718/play-stage.jsonl`, the piano's `fallboard` first moving at +0.93 s, the cymbals at +0.93 s, the pawn at +0.93 s, the jigsaw patch at +0.92 s, and the sewing table's, the pottery wheel's and the scrapbook's the same.
- **Source:** wave: hobbies

### A-hobbies-03 — The pool rack floats 25 cm over the balls for the whole game

- **Severity:** S2
- **Rubric:** A8, M7
- **Fix class:** motion
- **Where:** `tools/art/blender/families/hobbies.py:801` (`G.slide(rack, 'rack_off', (0, 0, 0.25))`: Blender z, straight up); `:1467-1468` (`rack_off` for `pool` and `trickShots`, held until the use ends)
- **Evidence:** `hobbies/sheets/obj-poolTable.png` (play), `hobbies/extra/two.png`: the triangle hangs in mid-air above the racked balls for the 30 or 60 minutes. Racking lifts the triangle off and puts it away (on the rail, a hook or under the table). Slide it up and over onto the head rail, or squash it away (`G.squash`); the balls would want to break too (I-hobbies-05).
- **Status:** fixed (abfc32d9)
- **After:** `rack_off` (new `G.path`, 1.4 s) lifts the triangle 14 cm off the balls, carries it over the foot rail and hangs it upright by its apex on the table's foot end, out of the play, for the whole game (`fixes/motions/g1718/views/pool-end-t99.png`, `-t0.6.png`, `clips-poolTable-all-t*.png`). The balls still don't break (I-hobbies-05).
- **Source:** wave: hobbies

### I-hobbies-04 — Darts, pool and foosball are played with the same hand-weaving clip

- **Severity:** S2
- **Rubric:** I3, A10
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:477` (`ACTIONS.play`: `stand: ['spell']` for every standing game); compare `:481-495` (`ACTIONS.exercise` picks a clip by object)
- **Evidence:** `hobbies/extra/side-dartboard-darts-00.png`: "Throw darts" is the hands circling at the chest, and the three darts appear in the board 0.6 s into the use with no throw (the `darts` clip at use start); `hobbies/sheets/obj-poolTable.png` (play): "Shoot pool" stands at the long side doing the same, no stroke; `hobbies/sheets/obj-foosball.png` (play): the hands weave over the handles while two rods kick on their own. The arcade cabinet's panel is the one standing game where it reads right. A throw (the arm back and through, the darts' clip timed to it with `every`), a cue stroke bent over the rail and a two-handed rod twist are new resident clips, picked by object as `exercise` does.
- **Status:** fixed (4366ec55, 83d53185)
- **After:** each game has its own clip, picked by object (`ACTIONS.play`, `STEP_IN`; `characters/clips/games.ts`): darts side-on at the line, three throws (aim, the forearm back and through, the hand following to the board) and a step back, the first dart thrown as the board's darts grow in (`dartThrow`); pool bent over the long rail as far as the height needs, the left hand bridging on the cloth, the right forearm stroking from the elbow, two practice strokes and the shot, then up to chalk (`poolShot172/147/112`, stepped up to the rail); foosball with the hands round two handles, the wrists turning the rods and sliding them (`foosball172/147/112`, at a pair of rods). `bench-results/review/fixes/clips-games/after-dartboard-darts-0*.png`, `after-poolTable-pool-0*.png`, `after-poolTable-trickShots-0*.png`, `after-foosball-foosball-0*.png`, the child's `child-poolTable-pool-0*.png`, `child-foosball-foosball-0*.png`. The cue in the hands is G-70 (mimed), the oche distance G-55.
- **Source:** wave: hobbies

### I-hobbies-05 — Pool is played with the cue left on the cloth and the balls never broken

- **Severity:** S2
- **Rubric:** I7
- **Fix class:** code
- **Where:** `tools/art/blender/families/hobbies.py:781-783` (the cue, part of the still body), `:776-779` (the racked balls, still); the `held` grips (`web/src/render/babylon/characters.ts:371`: `bar`, `pair`, `guitar`, `pole`) have nothing for a cue
- **Evidence:** `hobbies/extra/two.png` (top view): the cue lies across the cloth and the fifteen balls stay racked under the floating triangle through "Shoot pool" (30) and "Practise trick shots" (60). The model has the cue, and the game already draws an object's own part in the hands (`held`: the bar, the dumbbells, the guitars, the mop), but a cue held along the forearms is a new grip, and the balls breaking and spreading a new clip. Related to I-held-01 (held ledger: props the models don't have).
- **Status:** open
- **Source:** wave: hobbies

### I-hobbies-06 — Darts are thrown from 1.4 m, not from the oche at 2.37

- **Severity:** S3
- **Rubric:** I2
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:357-365` (`front_tile`: a standing user stops on the tile in front); `crates/sim-core/src/content.rs:340-346` (`Approach`: `Front` or `Around`, no distance)
- **Evidence:** `hobbies/play/c1/play.jsonl` (`dartboard` at `[10, 17, 2]` on the back wall, the thrower at z 16.5) and `hobbies/extra/side-dartboard-darts-00.png`: the face is about 1.43 m from the thrower; the throw line is 2.37 m from the face (`reference.md` `darts`). `docs/objects/hobbies.md` says "thrown from the tile in front of the board, half a metre from the wall", which is neither. An approach distance (two tiles out) is new behaviour; the renderer's `STAND_BACK` (`characters.ts:160`) could step them back 0.9 m in the meantime.
- **Status:** open
- **Source:** wave: hobbies

### I-hobbies-07 — moved to systems.md

### I-hobbies-08 — The guitar is held out in front of the chest, its neck pointing forward and up

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:387` (`GUITAR`: the body `out` 0.17 m in front of the spine's base), `:392-395` (`GUITAR_ARMS`: the left upper arm raised forward), `:1756-1776` (`writeHeld`: the neck aimed from the body at the left palm)
- **Evidence:** `hobbies/extra/gsidetop.png` (from above): the guitar's body is about 0.25 m in front of the player's chest, not against the belly, and its neck runs forward and left at about 30–35° off the line of the shoulders, the guitar's head level with the face. From the front (`hobbies/extra/inst.png`) that passes for a guitar across the body; from the game's camera, behind and above the player as they face the stand, the forward neck reads as pointing straight up past the head (`hobbies/extra/gdef.png`, `hobbies/play/c1/guitarAcoustic-strum-00.png`, the child in `hobbies/extra/child/child-guitarAcoustic-strum-01.png`): the lead in `index.md`. A guitar rests on the right thigh or against the belly, the neck to the left in the plane of the body, tilted up 10–45°. The same for `guitarElectric` and `ukulele` (the only three with `grip: guitar`). The hands are where they should be on it (left on the neck, right over the sound hole).
- **Status:** fixed (5bd8a29c, 1abb80cf)
- **After:** the guitar, the electric guitar and the ukulele are held across the body by their own clips (`strum`, `strumShort` for the ukulele, held higher; `clips/uses.ts`): the body against the belly a little to the right, the neck out to the left in the plane of the body, 30° up and angled 13° forward, the left hand on the neck 0.55 m along (0.27 on the ukulele), the right strumming over the sound hole, the head bent to the strings; the procedural arm turns (`GUITAR_ARMS`) are gone, and the guitar's rest point scales with the body (a child's belly is nearer the spine). From the game camera the neck no longer points up past the head. `bench-results/review/fixes/clips-standing/``sheet-guitars.png` (front and above, man and child), `play/guitar-*`, `strips/sheets/strum*`. The full-size guitar in a child's hands is M-hobbies-03's.
- **Source:** wave: hobbies (play from the front, the side and above; `index.md` "Seen" for the lead)

### I-hobbies-09 — Games, crafts and the exercise bike are silent, and the drums play the plucked tune

> Consolidation (2026-10-21): closed as a duplicate of I-bathroom-04 (systems ledger), which takes its items and evidence.

- **Severity:** S3
- **Rubric:** I9
- **Fix class:** code
- **Where:** `web/src/audio/world.ts:34-47` (`ACTION_SOUNDS`: nothing for `play`, `tinker`, `exercise`; `music` is one sound for every instrument), `:659-678` (`musicBuffer`, a plucked melody)
- **Evidence:** `cargo review uses` Sound column (`tables/uses-hobbies.md`, `tables/uses-fitness.md`): `—` for the pool table, foosball, darts, the arcade, the sewing machine, the chemistry bench, the pottery wheel, the jigsaw and craft tables, and the exercise bike; `music` for the drum kit, so drumming sounds like the piano and the guitars (a plucked tune on a pentatonic scale) with no beat. In life: balls clacking, the rods and the ball, darts' thuds, arcade bleeps, a sewing machine's whirr, a wheel's motor, a bike's flywheel. The jigsaw and the scrapbook are near silent (fine). The same cause and fix site as I-bathroom-04 (systems ledger: the toilet, the treadmill, the workbench), which this widens; one finding at the merge.
- **Status:** closed (duplicate of I-bathroom-04)
- **Source:** wave: hobbies

### I-hobbies-10 — The potter's knees go into the wheel's body

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** model
- **Where:** `tools/art/blender/families/hobbies.py:1365` (`base`: a drum 0.33 m in radius and 0.36 m tall, centred 0.12 m ahead of the middle), `:1376` (the stool at −0.3, shifted 0.02)
- **Evidence:** `hobbies/extra/side3.png` (side view, seated): the potter sits on the stool with the thighs level and the knees and shins inside the drum; from the front the legs vanish into it (`hobbies/sheets/obj-potteryWheel.png`, play). The drum's back is at about −0.19 m, the knees about 0.45 m ahead of the hips at −0.28. A real wheel's body is low and narrow under a splash pan the knees go either side of (or a cabinet with knee room), the potter leaning over the head. The hands on the pot are right.
- **Status:** fixed (8b6b2364)
- **After:** a narrow, low motor body (0.26 m wide) set forward on four feet, a 0.48 m splash pan on a column, the pedal by the right foot; the potter sits with the knees apart either side of the pan (`characters.ts` `activityPose`, `tinker` at the pottery wheel, as the cello's), the shins either side of the body, the hands on the pot. `fixes/blender-hobbies/play/adult-potteryWheel-throw-01.png`, `child-potteryWheel-throw-01.png`, `poses/poses-seats-*.png`.
- **Source:** wave: hobbies

### I-hobbies-11 — The experimenter stands a stride off the chemistry bench

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:160` (`STAND_BACK`, where a negative value steps the user in: `garden.herbs` −0.5; nothing for `chemistrySet`); the bench pushed back on its tile, `tools/art/blender/families/hobbies.py:1488` (`centre` −0.18, the top 0.5 m deep)
- **Evidence:** `hobbies/extra/side1.png` (side): the resident stands about 0.6 m off the bench's front edge; reaching (`interact`) only the fingertips get to the edge (`hobbies/extra/stand3.png`, beside the pilot's workbench and easel, where the hands are on the work), never the flasks or the microscope at the back. A child stands as far off, the top at her head (`hobbies/extra/child2.png`). About −0.35 in `STAND_BACK` brings them to the bench.
- **Status:** fixed (0c5e889e)
- **After:** `STAND_BACK.chemistrySet` -0.35: at the bench, the hands on its top among the flasks (`fixes/tables/play-a/a-chemistrySet-experiment-01.png`).
- **Source:** wave: hobbies

### M-hobbies-10 — The piano's content seat is 2 cm under its bench

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** content
- **Where:** `web/public/content/base.json:3393` (`piano` `seat.height` 0.47); the bench's top 0.49 m (`tools/art/blender/families/hobbies.py` `piano`: `bench_top` 0.05 at 0.465)
- **Evidence:** `cargo check-glb model.piano`: "seat 0.47 m (content `seat`), the model 0.490 m there (x 0.00, depth 0.23): 2.0 cm higher", past the owner's 1 cm tolerance: the pianist sits 2 cm into the cushion. A piano bench is 0.48–0.51 m, so the content's number is the one to move (0.49). Found by the Blender wave's gate run, not in play.
- **Status:** open
- **Source:** fix: blender-hobbies
