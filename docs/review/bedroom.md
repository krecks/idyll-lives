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
- **Status:** confirmed and widened (bedroom wave)
- **Source:** pilot; widened in wave: bedroom

### M-bedroom-02 — Two nightstands and the tall chest are as deep as their tile

- **Severity:** S3
- **Rubric:** M1, M11
- **Fix class:** model
- **Where:** `tools/art/blender/families/bedroom_storage.py:206-222` (`nightstandModern`: case from −0.42 to 0.16, top 0.60 deep), `:241-258` (`nightstandDrawers`: the same), `:421-437` (`chestOfDrawers`: case to 0.185, top 0.63); sizes at `:531`, `:535`, `:552`
- **Evidence:** `sizes-bedroom.md`: the modern and the three-drawer nightstand are 0.628 m deep against a nightstand's 0.30–0.50 (±10 %: 0.55), the tall chest 0.655 against 0.40–0.55 (0.605). The classic nightstand is 0.47 and the drum 0.56 (`bedroom/sheets/obj-nightstandModern.png`, `obj-nightstandDrawers.png`, `obj-chestOfDrawers.png`): beside a bed, the two are deeper than the bed's own headboard-to-pillow run and reach past the pillows. Taking them to about 0.45 (nightstands) and 0.50 (the chest) against the wall keeps their tile.
- **Status:** open
- **Source:** wave: bedroom

### M-bedroom-03 — The single wardrobe's one door is 0.88 m wide

- **Severity:** S3
- **Rubric:** M1, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/bedroom_storage.py:465-480` (`wardrobeSingle`: carcass 0.90, `door` 0.88 × 1.97 hinged at −0.44)
- **Evidence:** `sizes-bedroom.md`: W 0.90 against a single wardrobe's 0.50–0.80 (±10 %: 0.88). The one door is as wide as a room door (0.8–0.9 m, M17) and nearly as tall (`bedroom/sheets/obj-wardrobeSingle.png`); single wardrobe doors are 0.40–0.60 m (`ikeaPax`). Swung out 100° it sweeps most of the tile in front, where the resident stands (`bedroom/play/chunk2/wardrobeSingle-outfit-01.png`). A 0.60 m carcass, or two doors, reads as the thing.
- **Status:** open
- **Source:** wave: bedroom

### M-bedroom-04 — Every sleeper gets the same blue checked quilt, whatever the bed's own bedding

- **Severity:** S2
- **Rubric:** M9, M13, A12
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters/blanket.ts:197-246` (`blanketMaterial`: one texture, `#6F97A6` with a white sheet band); `web/src/render/babylon/characters.ts:1176` (one `quilt` material for every quilt), `:280` (`BLANKET_BEDS`)
- **Evidence:** `bedroom/play/chunk1/bedSingle-sleep-02.png` (a blue checked quilt over the sage single), `bedroom/play/chunk2/bedCanopy-sleep-02.png` (over the rust four-poster), `bedroom/play/chunk1/bedDouble-sleep-02.png`, `bedroom/extra/bunk-two-09.png` (teal bunks). Asleep, the bed's own duvet, the part the buyer colours (`Bedding`), is hidden under the game's quilt, which is the same blue check on every bed and every colour; it appears as the sleeper lies down and goes when they get up, so the bed changes colour twice a night. Tinting the quilt from the bed's `Bedding` colour (or its pattern by bed) would keep the bed the player chose. Other items (other ledgers): `kidsBed`, `kidsBedCar` (kids), `wallBed` (living), `lazy.snoozecloudBed` (packs): every bed in `BLANKET_BEDS`. Cause in the renderer: for `residents.md` (row "Blankets and pyjamas in bed").
- **Status:** open
- **Source:** wave: bedroom (lead from Phase 0.9 "Seen (not reviewed)")

### A-bedroom-01 — Turning down the bed, the duvet stands up as a stiff board

- **Severity:** S3
- **Rubric:** A9, A10
- **Fix class:** motion
- **Where:** `tools/art/blender/families/bedroom.py:121-127` (`_turn_down`: the flap, one rigid part, `G.swing` 180° about its fold in `SECONDS` = 1.0, `:75`, with a `G.squash`); `tools/art/blender/families/kids.py:275-285` (the kids' beds, the same)
- **Evidence:** `bedroom/clips/clips-bed-all-t0.5.png`: half way, the top third of the duvet stands upright across the head of the bed, a flat panel about 0.45 m high in front of the pillows; the same on `bedSingle`, `bedBunk` (both duvets), `bedKing`, `bedDouble`, `bedQueenUpholstered` and `bedCanopy` (`bedroom/clips/clips-*-all-t0.5.png`). A duvet is turned back by a hand and folds as cloth, never upright. It shows for about half of the 1 s clip each time someone gets into or out of bed, at the head end, before the game's quilt covers it. A clip that slides and folds the flap low (or two hinged halves) would read as cloth. Other items: `kidsBed`, `kidsBedCar` (kids ledger).
- **Status:** open
- **Source:** wave: bedroom (lead from Phase 0.9 "Seen (not reviewed)")

### A-bedroom-02 — The upper bunk's sleeper sits down in mid-air: no climb up the ladder

- **Severity:** S1
- **Rubric:** A1, A4, I2
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2526-2541` (a bunk's second sleeper gets `bedH` = the upper mattress, 1.38 m; getting in is `sitDown` with `rootY = rig.bedH − …` at `LIE_ENTRY`, `:304`, from the tile in front, as on any bed); `tools/art/blender/families/bedroom.py:225-229` (the ladder, static, at the foot end on the right); `web/src/render/babylon/characters/furniture.json:81` (`bunks.bedBunk` [0.44, 1.38])
- **Evidence:** `bedroom/extra/bunk-two-02.png`: with someone asleep below, a second resident (the elder) sent to sleep walks to the foot end and sits down 1.4 m above the floor on nothing, facing out, the legs through the upper bunk's end board; then lies on the upper duvet (`bunk-two-05.png`) and the quilt comes (`bunk-two-09.png`). Getting up, they drop the same way. Nobody uses the ladder. A climb (up the ladder, then in over the side) is new behaviour. Related: anyone may take the upper bunk, an elder or a small child too (no age rule, I-systems-04, systems ledger).
- **Status:** open
- **Source:** wave: bedroom (`bedroom/extra/scenes.mjs` phase `bunk`, `review-stage`, residents 0 and 4)

### A-bedroom-03 — Residents get into bed at its foot end, through the footboard, and slide up to the pillow

- **Severity:** S2
- **Rubric:** A1, A4, I2
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:304` (`LIE_ENTRY` 0.55: the hips sit down 0.55 m forward of the slot's middle), `:2533-2541` (getting in: `sitDown` on the edge facing the tile they came from, then the crossfade to lying); the beds' `approach` (default `front`: the tile in front, the foot end) in `web/public/content/furniture.json` and `catalog2/bedroom.json`; getting up `:2758-2807` (`fromBed`)
- **Evidence:** `bedroom/play/chunk1/bedSingle-sleep-00.png`: the resident sits down facing out, about 0.4 m in from the foot end of the single, the thighs through its 0.6 m footboard; `bedroom/play/stage-elder/bedSingle-sleep-01.png` (the elder, the same, the shoes on the mattress); `bedroom/play/chunk1/bedKing-sleep-01.png` (half way through the crossfade, half reclined in the middle of the bed); `bedroom/extra/bunk-two-02.png` (the bunk's end board). From there the body crossfades about 1.4 m up the bed to lie with the head on the pillow. People get in at a side, sit on the edge by the pillow and swing the legs in. On beds with a footboard or end arms (`bedSingle`, `bedBunk`, `bedDaybed`) the legs pass through it; on the rest (`bed`, `bedKing`, `bedDouble`, `bedQueenUpholstered`, `bedCanopy`, `bedFuton`) it is the sit at the foot and the slide. Other items: `kidsBed`, `kidsBedCar` (kids), `wallBed`, `sofaBed` (living), the packs' beds: every `lie` use on a bed. Cause in the renderer and the beds' approach: for `residents.md` (row "Lie fit").
- **Status:** open
- **Source:** wave: bedroom

### A-bedroom-04 — A stray quilt lies at the world origin whenever a bed with a blanket is on the lot

- **Severity:** S2
- **Rubric:** A12, M7
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:1489-1497` (every base body's quilt mesh `setEnabled(true)` once any object is in `BLANKET_BEDS`, with `thinInstanceCount = 0`), `:2458-2464` (`thinInstanceCount = drawn`, 0 when nobody is under that quilt); `web/src/render/babylon/characters/blanket.ts:181-188` (the mesh at the origin, `alwaysSelectAsActiveMesh`); Babylon draws a mesh with no thin instances as a plain mesh at its own transform (`Mesh.hasThinInstances` is `instancesCount > 0`, `@babylonjs/core` 9.30 `Meshes/mesh.pure.js:225`)
- **Evidence:** `bedroom/extra/quilt-origin-bed-awake.png`, `quilt-origin-bed-awake-far.png` (`review-stage`, a `bedSingle` bought, nobody in bed): a quilt lies on the lawn at the lot's corner, `(0, 0, 0)`, in view of the Live camera from the front garden. With no bed it isn't there (`quilt-origin-nobed.png`; the meshes disabled). With someone asleep it stays (`quilt-origin-asleep.png`): the sleeper's body's quilt is drawn on the bed, the other bodies' quilts (count 0) at the origin. `bedroom/extra/scenes.log` lists the seven `simBlanket:*` meshes, enabled, count 0, at the origin. The tooling agent's quilt on `poses.html` (`bedroom/sheets/age-bed.png`, right edge) is the same. Hiding a quilt mesh while its count is 0 (or `forcedInstanceCount`) fixes it. Other items: every bed in `BLANKET_BEDS` (`kidsBed`, `kidsBedCar`, `wallBed`, `lazy.snoozecloudBed` in other ledgers); where the origin falls on other lots (a street, a neighbour's lot) wasn't checked.
- **Status:** open
- **Source:** wave: bedroom (lead from the tooling round; `bedroom/extra/scenes.mjs` phase `quilt`)

### I-bedroom-01 — Pyjamas for every nap, a doze in the garden too, and the shoes stay on

- **Severity:** S2
- **Rubric:** I3, I16
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2373-2378` (`asleep`: lying in a `LYING` clip with tag `sleep` or `nap` → the night outfit); `web/src/render/babylon/characters/outfit.ts:149-154` (pyjamas recolour the top and bottom only; the shoe part stays)
- **Evidence:** `bedroom/play/chunk1/bed-nap-01.png` and every bed's nap: the resident changes into pyjamas for a 30-minute nap at noon; `bedroom/extra/lounger-doze.png`: "Doze in the sun" on a sun lounger in the garden at 10:22 is in pink pyjamas, white shoes on. `bedroom/play/stage-elder/bedSingle-sleep-01.png`: the shoes go into bed (on the mattress at the foot while getting in). People nap in their clothes, take their shoes off for bed and dress for the night only at night. Pyjamas for `sleep` only, and no shoes with them, would do. Other items: every lying `nap` use: `kidsBed`, `kidsBedCar` (kids), `sofaBed`, `wallBed` (living), `sunLounger`, `hammock` (outdoor), `lazy.snoozecloudBed`, `cheerful.rainbowHammock`, `gloomy.windowSeat`, `natureLover.gardenHammock`, `kind.hearthSettee` (packs) (`tables/uses-*.md`). Cause in the renderer: for `residents.md` (row "Blankets and pyjamas in bed").
- **Status:** open
- **Source:** wave: bedroom (lead from Phase 0.9 "Seen (not reviewed)"; `bedroom/extra/scenes.mjs` phase `lounger`)

### I-bedroom-02 — A night on the daybed with no cover

- **Severity:** S3
- **Rubric:** I3, A12
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:280` (`BLANKET_BEDS`: no `bedDaybed`); `tools/art/blender/families/bedroom.py:421` (its mattress in a fitted cover, no duvet)
- **Evidence:** `bedroom/play/chunk2/bedDaybed-sleep-02.png`: 480 minutes asleep on the daybed in pyjamas on a bare fitted cover. A daybed slept on has a blanket over the sleeper; the model has no duvet to double it (the README's reason to leave it out), so the quilt (M-bedroom-04's) would fit as it is. Napping on it with no cover is fine.
- **Status:** open
- **Source:** wave: bedroom (lead from Phase 0.9 "Seen (not reviewed)")

### I-bedroom-03 — The futon and the daybed are sofas by day, but nobody can sit on them

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:1980` (`bedFuton` interactions: sleep, nap, unfold, fold away); `web/public/content/catalog2/bedroom.json:159` (`bedDaybed`: sleep, nap)
- **Evidence:** `bedroom/sheets/obj-bedFuton.png` (folded, a sofa on a platform), `bedroom/sheets/obj-bedDaybed.png` (a back and two arms). Each is a sofa by day; folded, the futon's only use is to unfold it. The sofa bed has relax and doze (`catalog2/living.json`); a "Sit down" or "Relax" (the futon's with `when: closed`) on each would be the missing use.
- **Status:** open
- **Source:** wave: bedroom

### I-bedroom-04 — "Pick an outfit" changes nothing: the resident keeps the same clothes

- **Severity:** S3
- **Rubric:** I7, I1
- **Fix class:** code
- **Where:** `web/public/content/furniture.json:2162`, `:2191` (`dresser`, `wardrobe` `outfit`), `web/public/content/catalog2/bedroom.json:273`, `:314`, `:355`, `:384`, `:414` (`nightstandDrawers`, `chestOfDrawers`, `dresserLow`, `wardrobeSingle`, `armoire`); `web/src/render/babylon/characters/outfit.ts:120` (`outfitFor`: the day outfit is fixed by the resident's id and appearance)
- **Evidence:** `bedroom/play/chunk1/dresser-outfit-01.png`, `wardrobe-outfit-01.png`, `bedroom/play/chunk2/armoire-outfit-01.png`: the drawer or doors open, the resident works at the clothes for 10 minutes, the doors shut and they walk off dressed as before (comfort +0.05). Picking an outfit is changing into it. Changing the day outfit (from the wardrobe's clothes, or a pick from the creator's choices) is new behaviour.
- **Status:** open
- **Source:** wave: bedroom

### I-bedroom-05 — An outfit from the bedside drawer

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/catalog2/bedroom.json:273` (`nightstandDrawers` `outfit`)
- **Evidence:** `bedroom/play/chunk2/nightstandDrawers-outfit-01.png`: "Pick an outfit" from a 0.60 m nightstand's top drawer. A bedside drawer holds a book, glasses or a snack, not the day's clothes (`reference.md` `nightstand`: put a lamp, a book and a glass by the bed; a midnight snack from the drawer, as `slob.snackNightstand`). A "Read in bed" or a snack use, or no use (decor, as the other three), fits.
- **Status:** open
- **Source:** wave: bedroom

### I-bedroom-06 — The mirrors share out their two uses, and two of them have one name

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/base.json:3194-3207` (`mirror` "Standing mirror": speech only); `web/public/content/furniture.json:2239-2254` (`mirrorWall`: speech only); `web/public/content/catalog2/bedroom.json:498-512` (`mirrorFloor` "Standing mirror": primp only)
- **Evidence:** `bedroom/sheets/obj-mirrorFloor.png`, `obj-mirrorWall.png`, `play/mirror-speech-*.png` (pilot): every mirror is for checking yourself, and any can be practised at (`reference.md` `mirrorStanding`, `mirrorWall`: check your look; practise a speech). The cheval mirror and the wall mirror offer only the speech, the other standing mirror only "Check the mirror". Both standing mirrors are called "Standing mirror" in the catalog. Fix: all three get both uses (primp 5, speech 30), and the cheval mirror a name of its own. (The pilot passed `mirror` as OK, leaving the outfit check to the wardrobe; checking yourself in a mirror is a use of its own.)
- **Status:** open
- **Source:** wave: bedroom

### I-bedroom-07 — Getting ready at the dressing table is sitting with the hands in the lap

- **Severity:** S3
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:465` (`ACTIONS.wash`: `stand` clips only, no `sit`, so the seated default `sit` plays); `web/public/content/furniture.json:2219` (`dressingTable` `getReady`: `anim: wash`, `pose: sit`)
- **Evidence:** `bedroom/extra/pose-dressingTable-getReady.png` (`poses.html?seats=dressingTable:getReady`), `bedroom/play/chunk1/dressingTable-getReady-01.png`, `bedroom/sheets/age-dressingTable.png`: on the pouf facing the mirror for 15 minutes with the hands resting low, the drawer open beside the knee. Getting ready is the hands at the hair and face. A seated clip with the hands up (as `phone`'s hand to the head, or a brush stroke) on `getReady`, chosen by object as `look` and `harvest` are, is a table change; a hand prop is I-held-01's.
- **Status:** open
- **Source:** wave: bedroom
