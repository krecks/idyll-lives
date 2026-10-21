# Kids: authenticity ledger

What it covers: the 20 objects of [docs/objects/kids.md](../objects/kids.md), in its groups and order.

- **Evidence:** `bench-results/review/kids/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Wave rows** (office/kids wave, 2026-10-21): Shots are relative to `bench-results/review/` in the review worktree; the shots taken for this wave are in `kids/extra/` and `office/extra/` (how: `office/review-log.md`). Kids' furniture is judged against children's sizes (reference `resident.child`, the kinds' child ranges).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## KD1. Baby and kids' beds (4)

Family: `tools/art/blender/families/kids.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `crib` Crib | M-kids-01 | I-kids-04 | I-kids-03, I-kids-05 | I-kids-03 S1 code (confirmed, widened); I-kids-04 S2 renderer-table; I-kids-05 S2 code; M-kids-01 S3 model | `sheets/obj-crib.png`, `sheets/crib-care.png`, `play/crib-*.png`, `scenes/crib-*.png`, `scenes/care-*.png`, `clips/clips-crib-all-t*.png`, `sheets/bedpose-6.png`, `sheets/bedpose-9.png` |
| `kidsBed` Kids' bed | OK | OK | OK | — (its 0.88 × 1.82 m mattress is a single bed's, so an adult in it isn't the I-kids-03 case) | `kids/sheets/obj-kidsBed.png`, `kids/sheets/age-kidsBed.png`, `kids/play/child-c1/kidsBed-sleep-02.png` |
| `kidsBedCar` Race car bed | OK | OK | I-kids-03 | I-kids-03 S1 code (an adult's feet on the bonnet) | `kids/sheets/obj-kidsBedCar.png`, `kids/sheets/age-kidsBedCar.png`, `kids/poses/poses-only-elder-0-car_bed.png` |
| `highChair` High chair | M-kids-04 | n/a | I-kids-01 | I-kids-01 S2 code (seeded; confirmed, raised); M-kids-04 S3 content | `kids/sheets/obj-highChair.png`, `kids/play/c1/highChair-no-uses.png` |

### Real uses against the content's (I1)

Durations are judged by §9 decision 2 (short by design; only outliers against similar uses are filed).

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `crib` | a baby sleeps; feed (bottle, in arms), change a nappy on the changing table, play, cuddle, settle them | lie in the crib (720, the baby), feed (15), change (10), play (20), cuddle (15) | the right set; feeding and changing don't look like it (I-kids-04, I-kids-05), and anyone may take any of them, the 8-year-old and adults lying in it too (I-kids-03) |
| `kidsBed` | a child sleeps, naps, hears a bedtime story | sleep (480), nap (30) | OK (480 for a child, who needs 540–720, is the game's short night, by design); no bedtime story (a two-person use, not filed) |
| `kidsBedCar` | a child sleeps, naps, plays at driving it | sleep (480), nap (30) | OK; an adult lies in it with the feet on the bonnet (I-kids-03) |
| `highChair` | a baby or toddler sits in it to be fed at the table | none | the one use missing (I-kids-01) |

## KD2. Toys (10)

Family: `tools/art/blender/families/kids.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `toyBox` Toy box | OK | I-kids-06 | I-kids-06 | I-kids-06 S2 renderer-table | `kids/sheets/obj-toyBox.png`, `kids/sheets/age-toyBox.png` |
| `toyChest` Toy chest | M-kids-03 | I-kids-06 | I-kids-06 | I-kids-06 S2 renderer-table; M-kids-03 S3 content | `kids/sheets/obj-toyChest.png`, `kids/play/c1/toyChest-play-01.png` |
| `dollHouse` Doll house | OK | OK | OK | | `kids/sheets/obj-dollHouse.png`, `kids/sheets/age-dollHouse.png` |
| `rockingHorse` Rocking horse | OK | OK | I-kids-03 | I-kids-03 S1 code (adults, teens and elders ride it) | `kids/sheets/obj-rockingHorse.png`, `kids/sheets/age-rockingHorse.png` |
| `playTent` Play tent | OK | I-kids-07 | I-kids-07, I-kids-03 | I-kids-07 S2 renderer-table; I-kids-03 S1 code | `kids/sheets/obj-playTent.png`, `kids/play/child-c1/playTent-play-01.png` |
| `ballPit` Ball pit | OK | I-kids-07 | I-kids-07, I-kids-03 | I-kids-07 S2 renderer-table; I-kids-03 S1 code | `kids/sheets/obj-ballPit.png`, `kids/play/child-c1/ballPit-jump-01.png` |
| `trainTable` Train table | M-kids-02 | I-kids-06 | I-kids-06 | I-kids-06 S2 renderer-table; M-kids-02 S3 model | `kids/sheets/obj-trainTable.png`, `kids/sheets/age-trainTable.png` |
| `playMat` Play mat | OK | OK | OK | | `kids/sheets/obj-playMat.png`, `kids/sheets/age-playMat.png` |
| `teddyBear` Giant teddy bear | OK | OK | OK | | `kids/sheets/obj-teddyBear.png`, `kids/sheets/age-teddyBear.png` |
| `beanBagKids` Kids' bean bag | OK | OK | I-kids-03 | I-kids-03 S1 code (adults flop into it) | `kids/sheets/obj-beanBagKids.png`, `kids/sheets/age-beanBagKids.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `toyBox` | take toys out and play on the floor; tidy them away | play with toys (30) | the use is right; it is played standing over the box (I-kids-06); tidying away missing (minor, not filed) |
| `toyChest` | as the toy box | play with toys (30) | as the toy box (I-kids-06) |
| `dollHouse` | play with dolls, kneeling or sitting at its open front | play with the doll house (30), kneeling | OK |
| `rockingHorse` | a small child rocks on it | ride (15) | OK for a child; adults ride it too (I-kids-03) |
| `playTent` | crawl in, hide, play inside | play in the tent (30) | played outside the flap (I-kids-07) |
| `ballPit` | climb in, jump, throw the balls about | jump in the ball pit (20) | played beside the pit (I-kids-07) |
| `trainTable` | push the trains round, one or two children kneeling or standing at the low table | play with the trains (30, two slots) | the use is right; hands far over the track (I-kids-06) |
| `playMat` | build with blocks; a baby's tummy time | build with blocks (30), kneeling | OK; no baby use (a baby only lies in the crib today; not filed) |
| `teddyBear` | hug it, sit against it | hug (5) | OK (5 minutes is real, 1–5; the T4 flag against the `relax` median isn't an outlier against hugs) |
| `beanBagKids` | flop down, read a picture book in it | flop (20) | OK; adults flop into it too (I-kids-03) |

## KD3. Kids' study and art (6)

Family: `tools/art/blender/families/kids.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `kidsDesk` Kids' desk | OK | OK | I-kids-03 | I-kids-03 S1 code (an adult does homework at it; an adult desk chair works at it); I-kids-02 S3 renderer-table (seeded; closed) | `kids/sheets/obj-kidsDesk.png`, `kids/sheets/age-kidsDesk.png`, `kids/extra/side-chairTask-11-12-3-write-01.png` |
| `kidsTable` Kids' table | M-kids-03 | n/a | I-kids-08 | M-kids-03 S3 content; I-kids-08 S3 decision | `kids/sheets/obj-kidsTable.png`, `kids/play/child-pair-kidsChair/child-kidsChair-homework-01.png` |
| `kidsChair` Kids' chair | OK | A-kids-01, A-residents-09 | I-kids-03 | A-kids-01 S2 renderer-table (seeded; confirmed, raised); A-residents-09 S2 clip (residents ledger); I-kids-03 S1 code (adults' feet through the floor) | `kids/sheets/obj-kidsChair.png`, `kids/sheets/age-kidsChair-2.png`, `kids/extra/coffee-kidsChair-draw-01.png` |
| `kidsBookshelf` Picture-book shelf | OK | OK | I-living-03, I-held-01 | I-living-03 S2 content (living ledger: read standing at the shelf); I-held-01 S2 code (held ledger: no book) | `kids/sheets/obj-kidsBookshelf.png`, `kids/play/c1/kidsBookshelf-read-01.png` |
| `kidsChalkboard` Chalkboard easel | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no chalk) | `kids/sheets/obj-kidsChalkboard.png`, `kids/sheets/age-kidsChalkboard.png` |
| `kidsEasel` Kids' easel | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no brush) | `kids/sheets/obj-kidsEasel.png`, `kids/sheets/age-kidsEasel.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `kidsDesk` | a child does homework, draws | homework (45), draw (30) | OK for a child; adults do both (I-kids-03) |
| `kidsTable` | children draw, have a snack, play a game, on little chairs | none of its own; a kids' chair at it: draw (30), homework (45) | OK for drawing; never a place to eat (I-kids-08) |
| `kidsChair` | sit at the kids' table | sit (20); at a low table: draw (30), homework (45) | OK; at a coffee table the hands fall short (A-kids-01) |
| `kidsBookshelf` | pick a picture book and read it sitting down | read a picture book (30), standing | read standing at the shelf, no book (I-living-03, I-held-01) |
| `kidsChalkboard` | draw in chalk, wipe it | draw (30) | OK (no chalk in hand: I-held-01) |
| `kidsEasel` | paint (one child each side of a double easel) | paint (30), one slot | OK (no brush: I-held-01) |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-kids-01 — A kids' chair at a coffee table: the hands hover

> Office/kids wave: confirmed with the child it is for; raised from S3 to S2 (hands hovering is the README's S2 example, and a child drawing at the coffee table is a normal sight).

- **Severity:** S2
- **Rubric:** A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/tableTuck.ts`, `web/src/render/babylon/characters.ts:216` (`TUCK`)
- **Evidence:** A kids' chair pulled up to a coffee table (living.md L3) leaves the hands hovering over the top. Deferred from 0.3.0. Wave: `kids/extra/coffee-kidsChair-draw-01.png` (the 8-year-old, `draw`, kids' chair at the coffee table's long side): the chair isn't pulled in, the child sits a hand's breadth back from the 0.42 m top with the hands at the knees, short of it. At the kids' table the same chair is pulled in and the hands are on the top (`kids/play/child-pair-kidsChair/child-kidsChair-homework-01.png`).
- **Status:** open: needs a clip (re-checked, fix: tables, 2026-10-21)
- **After:** Played again (`fixes/tables/play-child/child-kidsChair-draw-02.png`): the chair is pulled in (`TUCK`) until the child's knees meet the 0.42 m top's edge, as far as it goes (a child's thighs on the 0.31 m seat are higher than the 0.38 m underside); the top is then about 0.4 m ahead of the hips and only 0.11 m over the seat, while the desk-hands clips (`sitHands`, `sitHandsFar`) hold the hands about 0.3 m over the seat (scaled) and at most 0.43 m out: they hover just over and short of its edge. No table row reaches it: a seated clip for a low top (leaning in, the hands about 0.12 m over the seat and 0.45 m out) is for the resident-clip wave.
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4; wave: office/kids (`check.mjs play --objects kidsChair@9,13,2 --with coffeeTable@9,12,0 --use draw --sim 3`)

### I-kids-01 — The high chair's tray has no use: the sim never seats a baby in it

> Office/kids wave: confirmed; raised from S3 to S2: the high chair has no use at all, and its one real use is obvious (the README's S2 example "a missing obvious use").

- **Severity:** S2
- **Rubric:** I1, I14
- **Fix class:** code
- **Where:** `tools/art/blender/families/kids.py:443-464`, `web/public/content/catalog2/dining.json:744` (`highChair`, no `interactions`)
- **Evidence:** A modelled part with no clip, left decorative for 0.3.0. A baby is never seated in the high chair, so nothing could use the tray. Wave: `kids/play/c1/highChair-no-uses.png` and `kids/play/child-c1/highChair-no-uses.png`: no uses for anyone. A baby is fed in the crib instead (I-kids-04). Seating a baby in it at a table, fed by an adult, is new behaviour.
- **Status:** confirmed
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5; wave: office/kids

### I-kids-02 — A chair facing the kids' desk is used from behind its built-in chair

> Office/kids wave: closed, as I-office-01.

- **Severity:** S3
- **Rubric:** I2, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/tableTuck.ts`
- **Evidence:** As I-office-01: a chair facing a desk with its own chair built in is tucked in from behind. Deferred from 0.3.0. Wave check: a desk chair can't be bought on the kids' desk's front tile ("that would block the way", both tries, `office/extra/play-gaps.jsonl`), and the kids' chair can't face it at all (`seatAt: ["low"]`, the kids' desk is a `desk`). Beside it, an adult in a desk chair writes at the end of the 0.57 m top, 9 cm over her seat (`kids/extra/side-chairTask-11-12-3-write-01.png`): that is an adult at a child's desk, filed under I-kids-03.
- **Status:** closed (a chair can no longer stand on the desk's front tile: "that would block the way")
- **Source:** `docs/objects/open.md` Still open › Residents; `plan/release-0.3.0.md` §1.4; wave: office/kids (`check.mjs play --objects-file gaps2.txt`)

### I-kids-03 — moved to systems.md

### I-kids-04 — Feeding the baby is the gardening kneel at the cot rail

- **Severity:** S2
- **Rubric:** I3, A2, I14
- **Fix class:** renderer-table
- **Where:** `web/public/content/base.json:3397` (`feed`: `anim: tend`); `web/src/render/babylon/characters.ts:508` (`ACTIONS.tend`: `kneel`, the flower bed's clip)
- **Evidence:** `sheets/crib-care.png` (top left, `scenes/crib-adult-feed-02.png`): the carer kneels on the floor at the cot's rail for 15 minutes while the baby lies on the mattress inside, and a child sent to do it stands at the rail (`sheets/crib-care.png` bottom right, `scenes/care-child-feed-01.png`). A baby is fed held in the arms (or sitting up), with a bottle. `cuddle` already lifts the baby out and holds them (`CUDDLE`), so a feed that plays the cuddle hold (the bottle is I-held-01) is a table change: `feed` on its own `ACTIONS` entry, or `tend` chosen by object as `harvest` and `look` are.
- **Status:** fixed (425486ee)
- **After:** "Feed the baby" is `anim: cuddle` (`base.json`): the carer lifts the baby out and holds them in the arms (`CUDDLE`) for the feed, not the gardening kneel (`fixes/tables/play-a/a-crib-feed-01.png`). The bottle is the props wave's (I-held-01); who may feed is the age rules' (G-45).
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

### I-kids-06 — Toys are played with standing over them, the hands out in the air

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:477` (`ACTIONS.play`: standing `spell`, `HANDS.keys`, for every object whose use is `anim: play`); content `anim: play` at `web/public/content/furniture.json:2663` (`toyBox`), `web/public/content/catalog2/kids.json:135` (`toyChest`), `kids.json:232` (`trainTable`)
- **Evidence:** `kids/play/c1/toyChest-play-01.png`, `kids/play/c1/toyBox-play-01.png`, `kids/play/child-c1/toyBox-play-01.png`: adult and child stand at the open box for 30 minutes with the hands held out over it; no toy comes out. `kids/play/c1/trainTable-trains-01.png`, `kids/play/child-c1/trainTable-trains-01.png`: at the 0.44 m train table they stand upright, the hands at the waist 0.3 m (child) to 0.5 m (adult) above the track. A child plays with toys kneeling or sitting on the floor, and at a train table kneels or bends over it. The play mat and the doll house already play the kneel (`anim: tend`, `ACTIONS.tend` `kneel`, `kids/sheets/obj-playMat.png`); choosing the `play` clip by object (as `look` and `harvest` do, `characters.ts:512` and `:523`) would give these the kneel, with toys out of the box as a `motion` or a prop.
- **Status:** fixed (425486ee)
- **After:** `ACTIONS.play` by def: the toy box and chest played at kneeling on the floor (`kneel`), the train table crouched at its edge (`crouch`) (`fixes/tables/play-a/a-toyBox-play-01.png`, `a-toyChest-play-01.png`, `a-trainTable-trains-01.png`, `play-child/`). Toys out of the box are a motion or a prop; a kneel at a low table with the hands on its top is a clip for the resident-clip wave.
- **Source:** wave: office/kids

### I-kids-07 — The ball pit and the play tent are used from outside

- **Severity:** S2
- **Rubric:** I2, I3
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:193` (`STEP_IN`: the drinking fountain, the phone box and the climbing frame step in; no entry for `ballPit` or `playTent`); `web/public/content/catalog2/kids.json:264` (`ballPit` "Jump in the ball pit"), `kids.json:166` (`playTent` "Play in the tent"); `tools/art/blender/families/kids.py:1118-1121` (`flap_open`, `bounce`)
- **Evidence:** `kids/play/c1/ballPit-jump-01.png`, `kids/play/child-c1/ballPit-jump-01.png`: "Jump in the ball pit" is 20 minutes standing on the floor beside the pit, reaching in, while its balls bounce on their own. `kids/play/c1/playTent-play-01.png`, `kids/play/child-c1/playTent-play-01.png`: "Play in the tent" is standing outside the open flap. In life a child climbs into the pit and crawls into the tent. A `STEP_IN` entry each (into the pit, on its floor 0.03 m up, with a bounce or sit clip; through the flap, crouched or sitting) would place them; the clips are the `clip` half of the fix.
- **Status:** open
- **Source:** wave: office/kids

### M-kids-02 — The train table is 1.8 m long

- **Severity:** S3
- **Rubric:** M1
- **Fix class:** model
- **Where:** `tools/art/blender/families/kids.py:759-764` (`trainTable`: `frame` 1.8 × 0.84), `:1122` (`size=(1.8, 0.68, 0.84)`)
- **Evidence:** `kids/sheets/obj-trainTable.png`: the table fills its 2 × 1 footprint, 1.80 m long against 0.90–1.30 m for a train table (1.43 with the 10 %; `tables/sizes-kids.md`). The top (0.44 m) is right; the H 0.68 is the trees on it. From the game camera it reads as a coffee table with a track on it.
- **Status:** open
- **Source:** wave: office/kids

### M-kids-03 — Two descriptions say what the models don't show

- **Severity:** S3
- **Rubric:** M6
- **Fix class:** content
- **Where:** `web/public/content/catalog2/kids.json:23` (`kidsTable`: "legs in four colours", against `tools/art/blender/families/kids.py:130` and `:919`, one `ktable` blue for all four legs); `kids.json:145` (`toyChest`: "with its lid open", against `kids.py:511-534`, the lid shut at rest and opened only while played with)
- **Evidence:** `kids/sheets/obj-kidsTable.png` (four blue legs), `kids/sheets/obj-toyChest.png` (the gallery and game shots show the lid shut). Either the text or the model changes.
- **Status:** open
- **Source:** wave: office/kids

### M-kids-04 — The high chair's seat is 2 cm over the range, and its base is narrow

- **Severity:** S3
- **Rubric:** M1, M2
- **Fix class:** content
- **Where:** `web/public/content/catalog2/dining.json:759` (`highChair` `seat.height` 0.62); `tools/art/blender/families/kids.py:443-450` (legs splayed to ±0.205 × −0.16…0.28)
- **Evidence:** `tables/sizes-kids.md`: the model's seat is 0.609 m (inside the 0.50–0.60 range with the 1 cm tolerance), but the content's `seat` is 0.62, 2 cm over; W 0.444 and D 0.474 are a little under the 0.45 and 0.495 low ends (0.50–0.65 × 0.55–0.80 less 10 %): real high chairs splay their legs wide so they can't tip. `kids/sheets/obj-highChair.png`. Nothing sits in it today (I-kids-01), so only the content number matters once something does: 0.60 or under.
- **Status:** fixed (a920f174)
- **After:** `highChair` content seat 0.60 (the mesh's 0.609, inside the 1 cm tolerance). Nothing sits in it yet (I-kids-01). The narrow base is a model change, left to the Blender band.
- **Source:** wave: office/kids

### I-kids-08 — The kids' table is never a place to eat

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `web/public/content/catalog2/kids.json:17` (`kidsTable` `surface.kind: "low"`), `kids.json:38` (`kidsChair` `seatAt: ["low"]`); meals are eaten at a `table` kind (`crates/sim-core/src/surface.rs`)
- **Evidence:** The kids' table has no uses of its own and the kids' chair at it only draws and does homework (`kids/play/child-pair-kidsChair/child-kidsChair-homework-01.png`, which looks right). Children have a snack or a meal at a little table; here a child carrying a plate goes to the dining table. Making the kids' table a `table` would let adults dine at it too (I-kids-03), so it needs a decision.
- **Status:** open
- **Source:** wave: office/kids
