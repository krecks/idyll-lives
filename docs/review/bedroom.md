# Bedroom: authenticity ledger

What it covers: the 27 objects of [docs/objects/bedroom.md](../objects/bedroom.md), in its groups and order.

- **Evidence:** `bench-results/review/bedroom/` (gallery, clip, pose and play shots; shots stay out of git). Shots in this ledger are relative to `bench-results/review/` in the review worktree; how the bedroom wave took the extra ones is in `bedroom/review-log.md`.
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): their Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md). The bedroom wave added its new findings to the pilot's two rows.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Durations** (§9 decision 2): every bedroom use is inside the reference's range or the game's usual shortening (sleep 480, nap 30, outfit 10, getReady 15, speech 30, primp 5, takeBlanket 3, sortWashing 5, sit 20, futon unfold and fold away 2). `cargo review uses` flags getReady (15 minutes against the median 2 of its anim `wash`, the hand wash); it is the real 10–30 of `reference.md`, so not an outlier.

## BR1. Beds (9)

Family: `tools/art/blender/families/bedroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bed` Double bed | M-bedroom-01, M-bedroom-04 | A-bedroom-01, A-bedroom-03, A-bedroom-04 | I-bedroom-01 | M-bedroom-01 S3 decision; M-bedroom-04 S2 code; A-bedroom-01 S3 motion; A-bedroom-03 S2 code; A-bedroom-04 S2 code; I-bedroom-01 S2 code | `sheets/obj-bed.png`, `play/bed-*.png`, `clips/clips-bed-all-t*.png`, `sheets/bedpose-0.png`, `play-child/bed-*.png`, `play-elder/bed-*.png` (pilot); `bedroom/clips/clips-bed-all-t0.5.png` (bedroom wave) |
| `bedSingle` Single bed | M-bedroom-04 | A-bedroom-01, A-bedroom-03, A-bedroom-04 | I-bedroom-01 | M-bedroom-04 S2 code; A-bedroom-01 S3 motion; A-bedroom-03 S2 code; A-bedroom-04 S2 code; I-bedroom-01 S2 code | `bedroom/sheets/obj-bedSingle.png`, `bedroom/play/chunk1/bedSingle-sleep-02.png`, `bedroom/play/chunk1/bedSingle-sleep-00.png` |
| `bedBunk` Bunk bed | M-bedroom-04 | A-bedroom-02, A-bedroom-01, A-bedroom-03, A-bedroom-04 | I-bedroom-01 | A-bedroom-02 S1 code; M-bedroom-04 S2 code; A-bedroom-03 S2 code; A-bedroom-04 S2 code; I-bedroom-01 S2 code; A-bedroom-01 S3 motion | `bedroom/sheets/obj-bedBunk.png`, `bedroom/extra/bunk-two-02.png`, `bedroom/extra/bunk-two-09.png` |
| `bedFuton` Futon bed | M-bedroom-01, M-bedroom-04 | A-bedroom-03, A-bedroom-04 | I-bedroom-03, I-bedroom-01 | M-bedroom-01 S3 decision (widened); M-bedroom-04 S2 code; A-bedroom-03 S2 code; A-bedroom-04 S2 code; I-bedroom-01 S2 code; I-bedroom-03 S3 content | `bedroom/sheets/obj-bedFuton.png`, `bedroom/play/chunk1/bedFuton-sleep-01.png`, `bedroom/play/chunk1/bedFuton-foldAway-01.png` |
| `bedKing` Upholstered king bed | M-bedroom-04 | A-bedroom-01, A-bedroom-03, A-bedroom-04 | I-bedroom-01 | M-bedroom-04 S2 code; A-bedroom-01 S3 motion; A-bedroom-03 S2 code; A-bedroom-04 S2 code; I-bedroom-01 S2 code | `bedroom/sheets/obj-bedKing.png`, `bedroom/play/chunk1/bedKing-sleep-01.png` |
| `bedDouble` Double bed | M-bedroom-04 | A-bedroom-01, A-bedroom-03, A-bedroom-04 | I-bedroom-01 | M-bedroom-04 S2 code; A-bedroom-01 S3 motion; A-bedroom-03 S2 code; A-bedroom-04 S2 code; I-bedroom-01 S2 code | `bedroom/sheets/obj-bedDouble.png`, `bedroom/play/chunk1/bedDouble-sleep-02.png` |
| `bedQueenUpholstered` Upholstered queen bed | M-bedroom-04 | A-bedroom-01, A-bedroom-03, A-bedroom-04 | I-bedroom-01 | M-bedroom-04 S2 code; A-bedroom-01 S3 motion; A-bedroom-03 S2 code; A-bedroom-04 S2 code; I-bedroom-01 S2 code | `bedroom/sheets/obj-bedQueenUpholstered.png`, `bedroom/play/chunk1/bedQueenUpholstered-sleep-02.png` |
| `bedCanopy` Four-poster bed | M-bedroom-04 | A-bedroom-01, A-bedroom-03, A-bedroom-04 | I-bedroom-01 | M-bedroom-04 S2 code; A-bedroom-01 S3 motion; A-bedroom-03 S2 code; A-bedroom-04 S2 code; I-bedroom-01 S2 code | `bedroom/sheets/obj-bedCanopy.png`, `bedroom/play/chunk2/bedCanopy-sleep-02.png` |
| `bedDaybed` Daybed | OK | A-bedroom-03 | I-bedroom-02, I-bedroom-03, I-bedroom-01 | A-bedroom-03 S2 code; I-bedroom-01 S2 code; I-bedroom-02 S3 renderer-table; I-bedroom-03 S3 content | `bedroom/sheets/obj-bedDaybed.png`, `bedroom/play/chunk2/bedDaybed-sleep-02.png` |

Measured (`sizes-bedroom.md`, reference `bedSingle`, `bedDouble`, `bedKing`, `bedCanopy`, `bedLow`, `bunkBed`): every mattress top is in its range and agrees with `furniture.json` `beds` and the model (no `heights.txt` warning): single 0.50, bunk 0.44 and 1.38 (`bunks`), futon 0.40, king 0.59, oak double, queen and four-poster 0.52, daybed 0.51, `bed` 0.58. Overall sizes in range but `bed` and `bedFuton` (M-bedroom-01). The futon's H 0.995 against `bedLow`'s 0.80 is its back at rest, leaning up as a sofa's (a futon sofa bed, not the platform futon the kind describes): a reference artefact, not filed. Every bed has a frame, mattress, pillows and bedding; the bunk its guard rail and ladder (static); the four-poster its posts, valance and tied curtains; the daybed its back and arms. A sleeper lies on the mattress on every bed, a child, teen and elder too (`bedroom/play/stage-*/`).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bed` | sleep, nap, read in bed, sit on the edge | sleep (480), take a nap (30); two slots | OK (pilot) |
| `bedSingle` | sleep, nap, read in bed | sleep (480), nap (30) | OK |
| `bedBunk` | two sleep one above the other; climb the ladder | sleep (480), nap (30); slot 0 the lower bunk, slot 1 the upper | the right uses; the climb isn't there (A-bedroom-02) |
| `bedFuton` | sit on it as a sofa; unfold to sleep; fold it away by day | sleep (480), nap (30), unfold (2), fold away (2) | sitting on it folded is missing (I-bedroom-03) |
| `bedKing` | sleep, nap, lie in | sleep (480), nap (30) | OK |
| `bedDouble` | sleep, nap, read in bed | sleep (480), nap (30) | OK |
| `bedQueenUpholstered` | sleep, nap, lie in | sleep (480), nap (30) | OK |
| `bedCanopy` | sleep, nap, draw the curtains | sleep (480), nap (30); curtains static | OK (drawing the curtains would be new; the drapes are decor) |
| `bedDaybed` | sit on it as a sofa by day; nap; a guest sleeps on it | sleep (480), nap (30) | sitting on it is missing (I-bedroom-03); the sleeper has no cover (I-bedroom-02) |

## BR2. Nightstands and small storage (8)

Family: `tools/art/blender/families/bedroom_storage.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `nightstand` Nightstand | OK | n/a | OK | | `bedroom/sheets/obj-nightstand.png` |
| `nightstandModern` Modern nightstand | M-bedroom-02 | n/a | OK | M-bedroom-02 S3 model | `bedroom/sheets/obj-nightstandModern.png` |
| `nightstandRound` Drum nightstand | OK | n/a | OK | | `bedroom/sheets/obj-nightstandRound.png` |
| `nightstandDrawers` Three-drawer nightstand | M-bedroom-02 | OK | I-bedroom-05, I-bedroom-04 | M-bedroom-02 S3 model; I-bedroom-04 S3 code; I-bedroom-05 S3 content | `bedroom/sheets/obj-nightstandDrawers.png`, `bedroom/play/chunk2/nightstandDrawers-outfit-01.png` |
| `nightstandLamp` Nightstand with lamp | OK | n/a | OK | | `bedroom/sheets/obj-nightstandLamp.png` |
| `cedarChest` Blanket chest | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no blanket in hand) | `bedroom/sheets/obj-cedarChest.png`, `bedroom/play/chunk2/cedarChest-takeBlanket-01.png` |
| `laundryHamper` Laundry hamper | OK | OK | OK | | `bedroom/sheets/obj-laundryHamper.png`, `bedroom/play/chunk2/laundryHamper-sortWashing-01.png` |
| `benchBedEnd` Bed-end bench | OK | OK | OK | | `bedroom/sheets/obj-benchBedEnd.png`, `bedroom/sheets/age-benchBedEnd.png` |

Measured: nightstand tops 0.56–0.60 (45–70); the drum's 0.56 m depth is its 0.54 m drum and foot, 1 cm over the ±10 % bound, taken as in. The blanket chest's top 0.50 (40–60), the bench's seat 0.47 (42–48), the hamper 0.48 × 0.63. The drawer, lid and lamp: the three-drawer nightstand's top drawer slides 0.30 m, the chest's lid swings up 95° and the hamper's 70° on hinges at their backs (`lid_open`), each from the resident's arrival to the use's end (`play.jsonl` clip events; `bedroom/extra/clipt.mjs`). The nightstand lamp's use is its switch in the pie menu (`web/src/game/session.ts:1163`), as the floor lamp's (lighting ledger); its night glow is for the lighting wave.

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `nightstand` | a lamp, a book and a glass beside the bed | none | OK (decor) |
| `nightstandModern` | the same | none | OK (decor) |
| `nightstandRound` | the same | none | OK (decor) |
| `nightstandDrawers` | the same; a book or a midnight snack from the drawer | pick an outfit (10) | an outfit from a bedside drawer (I-bedroom-05) |
| `nightstandLamp` | switch the reading light on and off; read in bed | its switch | OK |
| `cedarChest` | take out a blanket; sit on the lid | take out a blanket (3) | OK; the blanket isn't in hand and goes nowhere (I-held-01) |
| `laundryHamper` | drop dirty clothes in; take the washing to the machine | sort the washing (5) | OK |
| `benchBedEnd` | sit to put on shoes; lay out clothes | sit down (20, two slots) | OK |

## BR3. Wardrobes and drawers (6)

Family: `tools/art/blender/families/bedroom_storage.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `dresser` Chest of drawers | OK | OK | I-bedroom-04 | I-bedroom-04 S3 code | `bedroom/sheets/obj-dresser.png`, `bedroom/play/chunk1/dresser-outfit-01.png` |
| `wardrobe` Wardrobe | OK | OK | I-bedroom-04 | I-bedroom-04 S3 code | `bedroom/sheets/obj-wardrobe.png`, `bedroom/play/chunk1/wardrobe-outfit-01.png` |
| `chestOfDrawers` Tall chest of drawers | M-bedroom-02 | OK | I-bedroom-04 | M-bedroom-02 S3 model; I-bedroom-04 S3 code | `bedroom/sheets/obj-chestOfDrawers.png`, `bedroom/play/chunk2/chestOfDrawers-outfit-01.png` |
| `dresserLow` Low dresser | OK | OK | I-bedroom-04 | I-bedroom-04 S3 code | `bedroom/sheets/obj-dresserLow.png`, `bedroom/play/chunk2/dresserLow-outfit-01.png` |
| `wardrobeSingle` Single wardrobe | M-bedroom-03 | OK | I-bedroom-04 | M-bedroom-03 S3 model; I-bedroom-04 S3 code | `bedroom/sheets/obj-wardrobeSingle.png`, `bedroom/play/chunk2/wardrobeSingle-outfit-01.png` |
| `armoire` Armoire | OK | OK | I-bedroom-04 | I-bedroom-04 S3 code | `bedroom/sheets/obj-armoire.png`, `bedroom/play/chunk2/armoire-outfit-01.png` |

Measured: the dresser's H 1.35 is the picture frame standing on it; its top is 0.89 (70–100). Tops 0.80 (low dresser) and 1.16 (tall chest, 100–140). Wardrobes 2.11–2.41 tall (1.8–2.4; the wardrobe's 2.41 with the boxes on top), 0.62–0.63 deep. The doors swing out 100° (90–110) on hinges at their outer edges, the drawers slide 0.30 m (0.30–0.45), opening as the resident arrives and closing at the end of the use. Inside: folded clothes in the drawers, shelves and a rail of shirts (flat slabs, in the style).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `dresser` | pick out clothes; put the laundry away | pick an outfit (10) | the right use; the clothes don't change (I-bedroom-04) |
| `wardrobe` | pick out clothes; hang clothes up | pick an outfit (10) | as the dresser |
| `chestOfDrawers` | pick out clothes | pick an outfit (10) | as the dresser |
| `dresserLow` | pick out clothes; put the laundry away | pick an outfit (10) | as the dresser |
| `wardrobeSingle` | pick out clothes | pick an outfit (10) | as the dresser |
| `armoire` | pick out clothes; hang clothes up | pick an outfit (10) | as the dresser |

## BR4. Dressing and mirrors (4)

Family: `tools/art/blender/families/bedroom_dressing.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `mirror` Standing mirror | OK | OK | I-bedroom-06 | I-bedroom-06 S3 content (bedroom wave) | `sheets/obj-mirror.png`, `play/mirror-speech-*.png`, `gallery-all/gallery-mirror.png` (pilot) |
| `dressingTable` Dressing table | OK | I-bedroom-07 | I-bedroom-07 | I-bedroom-07 S3 renderer-table | `bedroom/sheets/obj-dressingTable.png`, `bedroom/sheets/age-dressingTable.png`, `bedroom/extra/pose-dressingTable-getReady.png` |
| `mirrorWall` Wall mirror | OK | OK | I-bedroom-06 | I-bedroom-06 S3 content | `bedroom/sheets/obj-mirrorWall.png`, `bedroom/play/chunk1/mirrorWall-speech-01.png` |
| `mirrorFloor` Standing mirror | OK | OK | I-bedroom-06 | I-bedroom-06 S3 content | `bedroom/sheets/obj-mirrorFloor.png`, `bedroom/play/chunk2/mirrorFloor-primp-01.png` |

Measured: the dressing table's top 0.76 (72–78) and its pouf's seat 0.45 (42–48, `furniture.json` `seats.dressingTable` 0.45); its D 0.85 is the table and the pouf in front of it, a tool artefact. The wall mirror's D 0.09 is its small ledge. The standing mirrors are 1.78–1.86 m tall (1.5–1.9). The dressing table's drawer slides out for getReady, from arrival to the end.

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `mirror` | check your outfit; practise a speech | practise a speech (30) | checking yourself is missing (I-bedroom-06) |
| `dressingTable` | do your hair and make-up; get ready to go out | get ready (15, seated) | the right use; the hands stay in the lap (I-bedroom-07) |
| `mirrorWall` | check your look; practise a speech | practise a speech (30) | checking yourself is missing (I-bedroom-06) |
| `mirrorFloor` | check your outfit; practise a speech | check the mirror (5) | practising a speech is missing (I-bedroom-06) |

## Findings

### M-bedroom-01 — The "Double bed" is king-size, and so is the futon

> Bedroom wave: confirmed and widened to the futon.

- **Severity:** S3
- **Rubric:** M1, M11
- **Fix class:** decision
- **Where:** `tools/art/blender/families/bedroom.py:153-161` (`bed`: headboard 1.90 m, base 1.82, mattress 1.72 × 1.78); `web/public/content/base.json:2479` (`name: "Double bed"`); `tools/art/blender/families/bedroom.py:250-268` (`bedFuton`: seat and back 1.86 wide, headboard 1.96); `web/public/content/furniture.json:1968` (`bedFuton`, kind `bedLow` in `docs/review/reference.json`)
- **Evidence:** `sizes-bedroom.md`: 1.90 × 1.13 × 1.88 against a double's 1.35–1.60 m (±10 %: 1.76). Its 1.72 m mattress is a king's (`bedKing` is 1.98 wide, the other `bedDouble` 1.62 with a 1.56 mattress). It fills its 2 × 2 footprint, which may be why. Either call it a king (content name and `reference.json` kind) or narrow it to a double; the mattress height 0.58 m is right (`BEDS.bed`, `furniture.json:53`) and sleepers lie well on it (`play/bed-sleep-02.png`). Its 1.78 m long mattress is short for 1.90–2.00 m beds, as for every bed in the family (`bedroom.py:161`, `:190`, `:322`). Widened (bedroom wave): the futon is 1.96 wide with a 1.86 m mattress (`bedroom/sheets/obj-bedFuton.png`), 19 % over the `bedLow` double's 1.65; it fills its 2 × 2 the same way. Futons come king-size, so the decision is the same: name or kind, or narrower.
- **Status:** fixed (b3f6b82d)
- **After:** "King-size bed" (kind `bedKing`) and "King futon" (`bedLow`, now to 1.95 m wide): `cargo review sizes` bedroom 10 → 8 outside.
- **Source:** pilot; widened in wave: bedroom

### M-bedroom-02 — Two nightstands and the tall chest are as deep as their tile

- **Severity:** S3
- **Rubric:** M1, M11
- **Fix class:** model
- **Where:** `tools/art/blender/families/bedroom_storage.py:206-222` (`nightstandModern`: case from −0.42 to 0.16, top 0.60 deep), `:241-258` (`nightstandDrawers`: the same), `:421-437` (`chestOfDrawers`: case to 0.185, top 0.63); sizes at `:531`, `:535`, `:552`
- **Evidence:** `sizes-bedroom.md`: the modern and the three-drawer nightstand are 0.628 m deep against a nightstand's 0.30–0.50 (±10 %: 0.55), the tall chest 0.655 against 0.40–0.55 (0.605). The classic nightstand is 0.47 and the drum 0.56 (`bedroom/sheets/obj-nightstandModern.png`, `obj-nightstandDrawers.png`, `obj-chestOfDrawers.png`): beside a bed, the two are deeper than the bed's own headboard-to-pillow run and reach past the pillows. Taking them to about 0.45 (nightstands) and 0.50 (the chest) against the wall keeps their tile.
- **Status:** fixed (dca59e66)
- **After:** The modern and the three-drawer nightstands are 0.47 m deep (the classic one's), the tall chest 0.51, their backs on the wall; the content surfaces follow (0.58 × 0.46, 0.88 × 0.48). `cargo review sizes` bedroom: 4 outside, none of the three (`fixes/blender-rooms/gallery/gallery-nightstandModern.png`, `gallery-nightstandDrawers.png`, `gallery-chestOfDrawers.png`; `play/rooms-chestOfDrawers-outfit-*.png`).
- **Source:** wave: bedroom

### M-bedroom-03 — The single wardrobe's one door is 0.88 m wide

- **Severity:** S3
- **Rubric:** M1, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/bedroom_storage.py:465-480` (`wardrobeSingle`: carcass 0.90, `door` 0.88 × 1.97 hinged at −0.44)
- **Evidence:** `sizes-bedroom.md`: W 0.90 against a single wardrobe's 0.50–0.80 (±10 %: 0.88). The one door is as wide as a room door (0.8–0.9 m, M17) and nearly as tall (`bedroom/sheets/obj-wardrobeSingle.png`); single wardrobe doors are 0.40–0.60 m (`ikeaPax`). Swung out 100° it sweeps most of the tile in front, where the resident stands (`bedroom/play/chunk2/wardrobeSingle-outfit-01.png`). A 0.60 m carcass, or two doors, reads as the thing.
- **Status:** fixed (dca59e66)
- **After:** A 0.60 m carcass with a 0.58 m door (the description's "narrow white wardrobe"); its box on the hat shelf no longer comes up through the top and the side (nor the double wardrobe's through its top): `fixes/blender-rooms/gallery/gallery-wardrobeSingle.png`, `clips/clips-wardrobeSingle-all-t99.png`, `play/rooms-wardrobeSingle-outfit-01.png`.
- **Source:** wave: bedroom

### M-bedroom-04 — moved to residents.md

### A-bedroom-01 — Turning down the bed, the duvet stands up as a stiff board

- **Severity:** S3
- **Rubric:** A9, A10
- **Fix class:** motion
- **Where:** `tools/art/blender/families/bedroom.py:121-127` (`_turn_down`: the flap, one rigid part, `G.swing` 180° about its fold in `SECONDS` = 1.0, `:75`, with a `G.squash`); `tools/art/blender/families/kids.py:275-285` (the kids' beds, the same)
- **Evidence:** `bedroom/clips/clips-bed-all-t0.5.png`: half way, the top third of the duvet stands upright across the head of the bed, a flat panel about 0.45 m high in front of the pillows; the same on `bedSingle`, `bedBunk` (both duvets), `bedKing`, `bedDouble`, `bedQueenUpholstered` and `bedCanopy` (`bedroom/clips/clips-*-all-t0.5.png`). A duvet is turned back by a hand and folds as cloth, never upright. It shows for about half of the 1 s clip each time someone gets into or out of bed, at the head end, before the game's quilt covers it. A clip that slides and folds the flap low (or two hinged halves) would read as cloth. Other items: `kidsBed`, `kidsBedCar` (kids ledger).
- **Status:** fixed (abfc32d9)
- **After:** as the flap turns over it is gathered along the bed to a fifth of its length half way and springs out again as it lies down (a scale there and back on the same clip, `ROLL`), so half way it is a low roll at the fold, not a 0.45 m board: `fixes/motions/g1718/clips-bed-all-t0.5.png`, `clips-bedBunk-all-t0.5.png`, `views/bunk-t0.5.png` and the other beds' and the kids' beds' (`clips-kidsBed-all-t0.5.png`, `clips-kidsBedCar-all-t0.5.png`). Folded, it is where it was (under the game's quilt).
- **Source:** wave: bedroom (lead from Phase 0.9 "Seen (not reviewed)")

### A-bedroom-02 — The upper bunk's sleeper sits down in mid-air: no climb up the ladder

- **Severity:** S1
- **Rubric:** A1, A4, I2
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2526-2541` (a bunk's second sleeper gets `bedH` = the upper mattress, 1.38 m; getting in is `sitDown` with `rootY = rig.bedH − …` at `LIE_ENTRY`, `:304`, from the tile in front, as on any bed); `tools/art/blender/families/bedroom.py:225-229` (the ladder, static, at the foot end on the right); `web/src/render/babylon/characters/furniture.json:81` (`bunks.bedBunk` [0.44, 1.38])
- **Evidence:** `bedroom/extra/bunk-two-02.png`: with someone asleep below, a second resident (the elder) sent to sleep walks to the foot end and sits down 1.4 m above the floor on nothing, facing out, the legs through the upper bunk's end board; then lies on the upper duvet (`bunk-two-05.png`) and the quilt comes (`bunk-two-09.png`). Getting up, they drop the same way. Nobody uses the ladder. A climb (up the ladder, then in over the side) is new behaviour. Related: anyone may take the upper bunk, an elder or a small child too (no age rule, I-systems-04, systems ledger).
- **Status:** fixed (6a1438d5, 1768f8b0)
- **After:** the upper bunk's sleeper walks from the tile round the foot end to the ladder, climbs its four rungs, crawls in through the gap in the guard rail, sits with the legs swung round and lies back into `lie` (`bunkUp172`/`147`/`112`); woken, the same way down and a walk round to the tile (`bunkDown…`), sim-core keeping them at the bed meanwhile (`Sim::ladder_ticks`, sim test `up_from_the_upper_bunk_they_climb_down_its_ladder_before_anything_else`). Shots, the lower bunk's sleeper in already: `fixes/rides/bunk-up-child.png`, `bunk-up-child-close.png`, `bunk-down-child.png`, `bunk-up-adult-down-teen.png` (the adult up, the teen down). The lower bunk as before. The duvet's turn-down comes early: A-bedroom-06.
- **Source:** wave: bedroom (`bedroom/extra/scenes.mjs` phase `bunk`, `review-stage`, residents 0 and 4)

### A-bedroom-03 — moved to residents.md

### A-bedroom-04 — moved to residents.md

### I-bedroom-01 — moved to residents.md

### I-bedroom-02 — A night on the daybed with no cover

- **Severity:** S3
- **Rubric:** I3, A12
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:280` (`BLANKET_BEDS`: no `bedDaybed`); `tools/art/blender/families/bedroom.py:421` (its mattress in a fitted cover, no duvet)
- **Evidence:** `bedroom/play/chunk2/bedDaybed-sleep-02.png`: 480 minutes asleep on the daybed in pyjamas on a bare fitted cover. A daybed slept on has a blanket over the sleeper; the model has no duvet to double it (the README's reason to leave it out), so the quilt (M-bedroom-04's) would fit as it is. Napping on it with no cover is fine.
- **Status:** fixed (3cd0bf87)
- **After:** `bedDaybed` in `BLANKET_BEDS`: the sleeper is under the quilt (`fixes/tables/play-a/a-bedDaybed-sleep-01.png`); it also covers a nap, as on the other beds.
- **Source:** wave: bedroom (lead from Phase 0.9 "Seen (not reviewed)")

### I-bedroom-03 — The futon and the daybed are sofas by day, but nobody can sit on them

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:1980` (`bedFuton` interactions: sleep, nap, unfold, fold away); `web/public/content/catalog2/bedroom.json:159` (`bedDaybed`: sleep, nap)
- **Evidence:** `bedroom/sheets/obj-bedFuton.png` (folded, a sofa on a platform), `bedroom/sheets/obj-bedDaybed.png` (a back and two arms). Each is a sofa by day; folded, the futon's only use is to unfold it. The sofa bed has relax and doze (`catalog2/living.json`); a "Sit down" or "Relax" (the futon's with `when: closed`) on each would be the missing use.
- **Status:** fixed in part (eaca9ed8): the futon; the daybed needs a renderer seat row
- **After:** Folded, the futon offers "Sit and relax" on its front edge (content `seat` 0.40 m, forward 0.68: `bench-results/review/fixes/content/g09/seat68-bedFuton-relax-00.png`). The daybed is sat on lengthways, which content `seat` can't say (a `seats` row with `along` in `characters/furniture.json`).
- **Source:** wave: bedroom

### I-bedroom-04 — "Pick an outfit" changes nothing: the resident keeps the same clothes

- **Severity:** S3
- **Rubric:** I7, I1
- **Fix class:** code
- **Where:** `web/public/content/furniture.json:2162`, `:2191` (`dresser`, `wardrobe` `outfit`), `web/public/content/catalog2/bedroom.json:273`, `:314`, `:355`, `:384`, `:414` (`nightstandDrawers`, `chestOfDrawers`, `dresserLow`, `wardrobeSingle`, `armoire`); `web/src/render/babylon/characters/outfit.ts:120` (`outfitFor`: the day outfit is fixed by the resident's id and appearance)
- **Evidence:** `bedroom/play/chunk1/dresser-outfit-01.png`, `wardrobe-outfit-01.png`, `bedroom/play/chunk2/armoire-outfit-01.png`: the drawer or doors open, the resident works at the clothes for 10 minutes, the doors shut and they walk off dressed as before (comfort +0.05). Picking an outfit is changing into it. Changing the day outfit (from the wardrobe's clothes, or a pick from the creator's choices) is new behaviour.
- **Status:** fixed (5f4411b9)
- **After:** "Pick an outfit" (content `changesOutfit`, on all six) puts the resident in their next outfit halfway through, the doors or the drawer open for the whole use now (no `hold`, `bedroom_storage.py`), and they walk off in it: the creator's, then two made from the id (`outfit.ts` `spareLook`: other garments, another top colour), in turn, kept and saved until they change again (`Sim::outfit`, save v31; `tests/outfits.rs`). `fixes/pairings/outfit/`: at the wardrobe in the creator's green top and navy capris (`after-wardrobe-outfit-01.png`), the white top and brown trousers from halfway (`-05`), leaving in them (`-07`); then at the dresser from those (`after-dresser-outfit-02.png`, the drawer out) into the yellow top and black shorts (`-06`). The use lasts its 10 minutes with comfort full. Not done: choosing which outfit. Before: `fixes/pairings/before/wardrobe-outfit-01.png`.
- **Source:** wave: bedroom

### I-bedroom-05 — An outfit from the bedside drawer

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/catalog2/bedroom.json:273` (`nightstandDrawers` `outfit`)
- **Evidence:** `bedroom/play/chunk2/nightstandDrawers-outfit-01.png`: "Pick an outfit" from a 0.60 m nightstand's top drawer. A bedside drawer holds a book, glasses or a snack, not the day's clothes (`reference.md` `nightstand`: put a lamp, a book and a glass by the bed; a midnight snack from the drawer, as `slob.snackNightstand`). A "Read in bed" or a snack use, or no use (decor, as the other three), fits.
- **Status:** fixed (eaca9ed8)
- **After:** The outfit is gone; "Grab a snack from the drawer" (fetched, eaten sitting). The drawer clip is keyed to the old `outfit` use in `bedroom_storage.py` (`DRAWER_USE`): it opens again once the key is `snack` and the family is rebuilt (`cargo review uses` flags the key meanwhile). Re-keyed (dca59e66): the drawer slides out for the snack with a biscuit tin, chocolate and a paperback in it (`fixes/blender-rooms/play/rooms-nightstandDrawers-snack-01.png`, `play-rooms.jsonl`: `drawer_open` 4151–7102 ms).
- **Source:** wave: bedroom

### I-bedroom-06 — The mirrors share out their two uses, and two of them have one name

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/base.json:3194-3207` (`mirror` "Standing mirror": speech only); `web/public/content/furniture.json:2239-2254` (`mirrorWall`: speech only); `web/public/content/catalog2/bedroom.json:498-512` (`mirrorFloor` "Standing mirror": primp only)
- **Evidence:** `bedroom/sheets/obj-mirrorFloor.png`, `obj-mirrorWall.png`, `play/mirror-speech-*.png` (pilot): every mirror is for checking yourself, and any can be practised at (`reference.md` `mirrorStanding`, `mirrorWall`: check your look; practise a speech). The cheval mirror and the wall mirror offer only the speech, the other standing mirror only "Check the mirror". Both standing mirrors are called "Standing mirror" in the catalog. Fix: all three get both uses (primp 5, speech 30), and the cheval mirror a name of its own. (The pilot passed `mirror` as OK, leaving the outfit check to the wardrobe; checking yourself in a mirror is a use of its own.)
- **Status:** fixed (eaca9ed8, 1222c510)
- **After:** All three mirrors check the look (5 min, standing straight: `idle`) and practise a speech (30); the base one is the "Cheval mirror".
- **Source:** wave: bedroom

### I-bedroom-07 — Getting ready at the dressing table is sitting with the hands in the lap

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:465` (`ACTIONS.wash`: `stand` clips only, no `sit`, so the seated default `sit` plays); `web/public/content/furniture.json:2219` (`dressingTable` `getReady`: `anim: wash`, `pose: sit`)
- **Evidence:** `bedroom/extra/pose-dressingTable-getReady.png` (`poses.html?seats=dressingTable:getReady`), `bedroom/play/chunk1/dressingTable-getReady-01.png`, `bedroom/sheets/age-dressingTable.png`: on the pouf facing the mirror for 15 minutes with the hands resting low, the drawer open beside the knee. Getting ready is the hands at the hair and face. A seated clip with the hands up (as `phone`'s hand to the head, or a brush stroke) on `getReady`, chosen by object as `look` and `harvest` are, is a table change; a hand prop is I-held-01's.
- **Status:** fixed (3cd0bf87, 0f613d59)
- **After:** `ACTIONS.wash.upper` by def: seated at the dressing table the `phone` clip's upper body brings a hand up to the hair (`fixes/tables/play-a/a-dressingTable-getReady-01.png`). A brush stroke clip and a hand prop stay with the clip and props waves. Then (fix: clips-reach) the brush stroke: `brushHair` (clips/self.ts) as the seated upper body, the hand closed (`HANDS.grip`): four strokes from the crown down the right side, the head tilting into each, then the left hand smoothing the other side (`fixes/clips-reach/poses/poses-dressingTable-getReady-strokes.png`, `play-a/a-dressingTable-getReady-01.png`, `play-teen/`, `play-child/`, `play-elder/`; `compare/I-bedroom-07.png`). The brush itself is I-held-01's.
- **Source:** wave: bedroom

### A-bedroom-05 — The daybed's quilt hangs down through its front rail

- **Severity:** S3
- **Rubric:** A1, M2
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts` (`writeBlanket`, `BLANKET_BEDS`: the daybed in it since I-bedroom-02); `tools/art/blender/families/bedroom.py:421` (`bedDaybed`: a back, two arms and a front rail of spindles round the mattress)
- **Evidence:** `fixes/tables/play-a/a-bedDaybed-sleep-01.png`: the quilt's sides drop past the mattress's edges as on an open bed, so on the front side it hangs down between the rail's spindles to below the mattress, and at the back it runs into the back. A bed with sides wants the quilt tucked in at the mattress's edges (no side drop), as a cot's would be.
- **Status:** open
- **Source:** fix: tables (G-04 re-check, 2026-10-21)

### A-bedroom-06 — The upper bunk's duvet turns down while its sleeper is still at the foot of the ladder

- **Severity:** S3
- **Rubric:** A2, A4
- **Fix class:** motion
- **Where:** `web/public/assets/packs/bedroom/manifest.json` (`model.bedBunk` `motions.uses.sleep`: `turn_down_upper` for slot 1, played as the use starts); `tools/art/blender/families/bedroom.py:219` (`bedBunk`: `_turn_down(upper, 'turn_down_upper', …)`)
- **Evidence:** `fixes/rides/bunk-up-child.png` (t 0.1 to 0.86): the upper duvet's flap folds back the moment the use starts, about 6 s before its sleeper, now climbing the ladder (G-44, `bunkUp`), gets onto the mattress, and stands folded up beside them while they crawl in and sit. Someone turns their own duvet back once in bed. The upper slot's turn-down wants a delay of the climb's length (`bunkUp`, 7 s at speed 1), as the moving parts in time with the use (G-17) do.
- **Status:** fixed (ac62888a)
- **After:** `turn_down_upper` waits 5 s (`delay`): on the ladder the duvet lies flat (`fixes/blender-rooms/bunk/upper-v-03.png`, `-04.png`, the child 2.6–4.3 s into `bunkUp112`); it is folded back as they sit up on the mattress (`upper-v-07.png`, 6.2 s) and the quilt comes as they lie (`upper-v-09.png`).
- **Source:** fix: rides (G-44)

### I-bedroom-08 — At a mirror the resident looks down, not at the glass

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:3378` (`talk`, `look`, `listen` tip the head 0.2 rad down at any object but the telescope)
- **Evidence:** "Practise a speech" (`anim: talk`) at every mirror tips the head down to the object, as at a plant or a bird bath; a mirror is looked into at eye height. "Check the mirror" was `look` (the same tip) and is `idle` now (1222c510); the speech wants the head level at a mirror (by def, or a `mirror` anim).
- **Status:** open
- **Source:** fix: content
