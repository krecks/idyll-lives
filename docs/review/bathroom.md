# Bathroom: authenticity ledger

What it covers: the 26 objects of [docs/objects/bathroom.md](../objects/bathroom.md), in its groups and order.

- **Evidence:** `bench-results/review/bathroom/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Bathroom wave rows** (2026-10-21): Shots are relative to `bench-results/review/` in the review worktree; the evidence's own index is `bathroom/index.md`, the review's log `bathroom/review-log.md`. One extra run: `bathroom/extra/mirror-over-sink/` (the mirror cabinet hung over a pedestal sink).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Shared, filed elsewhere:** a use ends once its needs are full (I-systems-02, systems ledger: why the toilet sits in the evidence last 0.3–3.9 s, the bladder at 0.34–0.96 when each run began); no privacy for toilets, showers and baths (I-systems-05, systems ledger); no prop in hand (I-held-01, held ledger: the toothbrush, the towel, the washing).

## Sinks (5)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sink` Sink | M-bathroom-01 | OK | I-kitchen-02 | M-bathroom-01 S3 decision; I-kitchen-02 S3 content | `sheets/obj-sink.png`, `play/sink-wash-*.png`, `clips/clips-sink-all-t*.png`, `play-child/sink-wash-*.png`, `sheets/kitchen-sinks.png` |
| `sinkPedestal` Pedestal sink | OK | I-bathroom-05 | I-bathroom-05, I-bathroom-06 | I-bathroom-05 S2 renderer-table; I-bathroom-06 S2 content | `bathroom/sheets/obj-sinkPedestal.png`, `bathroom/play/chunk1/sinkPedestal-wash-01.png`, `bathroom/clips/clips-sinkPedestal-all-t99.png` |
| `vanityDouble` Double vanity | M-bathroom-02 | I-bathroom-05 | I-bathroom-05, I-bathroom-06 | M-bathroom-02 S2 model; I-bathroom-05 S2 renderer-table; I-bathroom-06 S2 content | `bathroom/sheets/obj-vanityDouble.png`, `bathroom/play/chunk1/vanityDouble-wash-01.png`, `bathroom/clips/clips-vanityDouble-all-t99.png` |
| `sinkVessel` Vessel sink | M-bathroom-02 | I-bathroom-05 | I-bathroom-05, I-bathroom-06 | M-bathroom-02 S2 model; I-bathroom-05 S2 renderer-table; I-bathroom-06 S2 content | `bathroom/sheets/obj-sinkVessel.png`, `bathroom/play/chunk3-short/sinkVessel-wash-01.png`, `bathroom/clips/clips-sinkVessel-all-t99.png` |
| `sinkConsole` Console sink | OK | I-bathroom-05 | I-bathroom-05, I-bathroom-06 | I-bathroom-05 S2 renderer-table; I-bathroom-06 S2 content | `bathroom/sheets/obj-sinkConsole.png`, `bathroom/play/chunk3-short/sinkConsole-wash-01.png`, `bathroom/clips/clips-sinkConsole-all-t99.png` |

Measured (M1, M2): the basins' rims are 0.88 m (pedestal, console), 1.00 m (vanity) and 1.02 m (vessel) against 0.76–0.91. `cargo review sizes` reads the pedestal's "top 0.751" from its drain and the vanity's and vessel's 0.84 / 0.82 from the underside of the worktop slab, not the rims (`bathroom.py:206`, `:238`, `:260`); the rims are the built numbers. Widths, heights and depths are inside ±10 % (the vanity 1.90 m against 1.2–1.8). `water_on` grows from each spout into its basin for the whole use (A9, A10: OK). The vanity's second slot was not tried with two residents.

### Real uses against the content's (I1)

Durations are judged by §9 decision 2 (short by design, outliers over 3× the tag median only).

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `sink` | wash hands and face, brush teeth, shave; in a kitchen also wash up and fill a pan | wash hands (2); a `dishes` station for washing up after meals | brushing teeth missing (I-kitchen-02, I-bathroom-06); it also stands in 3 of 4 generated kitchens (M-bathroom-01) |
| `sinkPedestal` | wash hands and face, brush teeth | wash hands (2) | brushing teeth missing, though a cup and toothbrush are modelled on its deck (I-bathroom-06) |
| `vanityDouble` | two people wash and brush their teeth at once | wash hands (2, two slots) | brushing teeth missing, though its description promises it (I-bathroom-06) |
| `sinkVessel` | wash hands and face, brush teeth | wash hands (2) | brushing teeth missing (I-bathroom-06) |
| `sinkConsole` | wash hands and face, brush teeth; dry the hands on its towel bar | wash hands (2) | brushing teeth missing (I-bathroom-06) |

## B1. Toilets (5)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `toilet` Toilet | OK | OK | I-bathroom-04 | I-bathroom-04 S3 code | `sheets/obj-toilet.png`, `play/toilet-use-*.png`, `clips/clips-toilet-all-t*.png`, `sheets/seat-toiletM.png`, `sheets/seat-toiletF.png` |
| `toiletCompact` Compact toilet | OK | OK | I-bathroom-04 | I-bathroom-04 S3 code | `bathroom/sheets/obj-toiletCompact.png`, `bathroom/sheets/age-toiletCompact-2.png`, `bathroom/play/chunk3-short/toiletCompact-use-01.png` |
| `toiletSmart` Smart toilet | M-bathroom-03 | OK | I-bathroom-04 | M-bathroom-03 S3 code; I-bathroom-04 S3 code | `bathroom/sheets/obj-toiletSmart.png`, `bathroom/sheets/age-toiletSmart.png`, `bathroom/play/chunk1/toiletSmart-use-01.png` |
| `toiletClassic` Classic toilet | OK | OK | I-bathroom-04 | I-bathroom-04 S3 code | `bathroom/sheets/obj-toiletClassic.png`, `bathroom/sheets/age-toiletClassic.png`, `bathroom/play/stage-teen-redo/toiletClassic-use-01.png` |
| `toiletWallHung` Wall-hung toilet | M-bathroom-03 | OK | I-bathroom-04 | M-bathroom-03 S3 code; I-bathroom-04 S3 code | `bathroom/sheets/obj-toiletWallHung.png`, `bathroom/sheets/age-toiletWallHung-2.png`, `bathroom/play/chunk4-redo/toiletWallHung-use-00.png` |

Measured (M1, M2, M11): seat tops 0.429 (toilet, compact), 0.44 (smart), 0.449 (classic) and 0.444 m (wall-hung), all inside 0.40–0.485 and within 1 cm of the game's seat (furniture.json 0.43 / 0.43 / 0.44; content 0.45 / 0.45). The "0.47", "0.46" and heights.txt's "smart 2.5 cm higher" are the closed lid's top (seat + 2–2.5 cm), which `lid_up` lifts before anyone sits. The sizes table's W and H "out" on the compact (0.59: its roll holder), smart (0.90 × 1.20) and wall-hung (0.60 × 1.10) are the roll arm and the boxed-in cistern panel, which the kind's WC range leaves out: OK. `lid_up` swings 100° (95–110°) as they sit and shuts as they rise (A8–A10: OK). Children, teens and elders sat on all five in play (`play/stage-*`); feet are A-residents-09 (residents ledger).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `toilet` | use it, flush, wash hands after | use toilet (5, then `handwash`) | OK; no flush sound (I-bathroom-04) |
| `toiletCompact` | as the toilet | use toilet (5, then `handwash`; wears faster) | OK; no flush sound (I-bathroom-04) |
| `toiletSmart` | as the toilet; the bidet wash, the warm seat | use toilet (5, then `handwash`; +hygiene, +comfort for the bidet and seat) | OK; no flush sound (I-bathroom-04) |
| `toiletClassic` | as the toilet | use toilet (5, then `handwash`) | OK; no flush sound (I-bathroom-04) |
| `toiletWallHung` | as the toilet | use toilet (5, then `handwash`) | OK; no flush sound (I-bathroom-04) |

## B2. Showers (4)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `shower` Shower | OK | A-bathroom-01, A-bathroom-05 | I-bathroom-02, I-bathroom-03 | A-bathroom-01 S2 model; A-bathroom-05 S3 renderer-table; I-bathroom-02 S3 clip; I-bathroom-03 S3 decision | `sheets/obj-shower.png`, `play/shower-shower-*.png`, `clips/clips-shower-all-t*.png`, `play-child/shower-shower-*.png` |
| `showerStall` Shower stall | OK | A-bathroom-02 | I-bathroom-02, I-bathroom-03 | A-bathroom-02 S3 model; I-bathroom-02 S3 clip; I-bathroom-03 S3 decision | `bathroom/sheets/obj-showerStall.png`, `bathroom/play/chunk1/showerStall-shower-01.png`, `bathroom/clips/redo-tall/clips-showerStall-all-t99.png` |
| `showerWalkIn` Walk-in rain shower | OK | OK | I-bathroom-02, I-bathroom-03 | I-bathroom-02 S3 clip; I-bathroom-03 S3 decision | `bathroom/sheets/obj-showerWalkIn.png`, `bathroom/play/chunk1/showerWalkIn-shower-01.png`, `bathroom/clips/redo-tall/clips-showerWalkIn-all-t99.png` |
| `showerRain` Rain shower | OK | OK | I-bathroom-02, I-bathroom-03 | I-bathroom-02 S3 clip; I-bathroom-03 S3 decision | `bathroom/sheets/obj-showerRain.png`, `bathroom/play/chunk2/showerRain-shower-01.png`, `bathroom/clips/redo-tall/clips-showerRain-all-t99.png` |

Measured (M1): 0.90–0.94 × 2.08–2.26 × 0.90–0.93 m, inside 0.7–1.2 × 1.85–2.3. The walk-in shower has an open side and the rain shower an open half front, so their residents step in through a gap (A-bathroom-01 is the base shower's alone). `water_on` grows from the head as the use starts and shuts as it ends (A9, A10: OK).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `shower` | take a shower, rinse off after sport | take a shower (20) | OK; 20 min is over the 5–15 real range but not an outlier |
| `showerStall` | take a shower | take a shower (25, "a bit slow" by its description) | OK; by design |
| `showerWalkIn` | take a shower | take a shower (15, refreshed) | OK |
| `showerRain` | take a shower | take a shower (15, refreshed) | OK |

## B3. Baths (3)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bathtub` Bathtub | A-bathroom-04 | A-bathroom-04 | I-bathroom-03 | A-bathroom-04 S3 renderer-table; I-bathroom-03 S3 decision | `bathroom/sheets/obj-bathtub.png`, `bathroom/sheets/age-bathtub.png`, `bathroom/play/chunk1/bathtub-bath-01.png` |
| `bathtubClawfoot` Clawfoot tub | A-bathroom-04 | A-bathroom-04 | I-bathroom-03 | A-bathroom-04 S3 renderer-table; I-bathroom-03 S3 decision | `bathroom/sheets/obj-bathtubClawfoot.png`, `bathroom/play/chunk1/bathtubClawfoot-bath-01.png`, `bathroom/clips/clips-bathtubClawfoot-all-t99.png` |
| `bathtubFreestanding` Freestanding bath | A-bathroom-04 | A-bathroom-04 | I-bathroom-03 | A-bathroom-04 S3 renderer-table; I-bathroom-03 S3 decision | `bathroom/sheets/obj-bathtubFreestanding.png`, `bathroom/play/chunk2/bathtubFreestanding-bath-01.png`, `bathroom/clips/clips-bathtubFreestanding-all-t99.png` |

Measured (M1): 1.82 / 1.73 / 1.89 m long (1.5–1.8 ±10 %), rims 0.58, 0.70 and 0.58 m. The sizes table's H "out" on the clawfoot (1.02) and freestanding bath (0.92) is the tall filler over the rim; the tubs are in range. `ACTIONS.bath` (left to this wave by the residents ledger): OK at all three, the bather lying back along the tub, legs out to the taps, arms on the rims, every life stage (`play/stage-*`). The tap runs and the water rises round a bather already sitting (`tap_on` + `fill`, 2.4 s): a game shortening, not filed.

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bathtub` | take a bath; bath a small child; shower over the bath | take a bath (45: hygiene, comfort, fun) | OK; bathing a baby or child is not in the game (baby care is the cross-cutting wave's) |
| `bathtubClawfoot` | take a long soak | take a bath (50, pampered) | OK; by design |
| `bathtubFreestanding` | take a long soak | take a bath (50, pampered) | OK; by design |

## B4. Laundry (6)

Family: `tools/art/blender/families/laundry.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `laundryWasher` Washing machine | OK | A-bathroom-03 | I-bathroom-07 | A-bathroom-03 S3 motion; I-bathroom-07 S3 code | `bathroom/sheets/obj-laundryWasher.png`, `bathroom/play/chunk4-redo/laundryWasher-laundry-01.png`, `bathroom/clips/clips-laundryWasher-all-t99.png` |
| `laundryDryer` Tumble dryer | OK | A-bathroom-03 | I-bathroom-07, I-bathroom-04 | A-bathroom-03 S3 motion; I-bathroom-07 S3 code; I-bathroom-04 S3 code | `bathroom/sheets/obj-laundryDryer.png`, `bathroom/play/chunk2/laundryDryer-laundry-01.png`, `bathroom/clips/clips-laundryDryer-all-t99.png` |
| `towelRail` Towel rail | OK | n/a | OK | | `bathroom/sheets/obj-towelRail.png`, `bathroom/play/chunk1/towelRail-no-uses.png` |
| `towelLadder` Towel ladder | M-bathroom-03 | n/a | OK | M-bathroom-03 S3 code | `bathroom/sheets/obj-towelLadder.png`, `bathroom/play/chunk2/towelLadder-no-uses.png`, `bathroom/play/chunk4-redo/towelLadder-no-uses.png` |
| `laundryBasket` Laundry basket | OK | n/a | OK | I-bathroom-01 S3 model (seeded; closed: an open basket of clean washing has no lid) | `bathroom/sheets/obj-laundryBasket.png`, `bathroom/play/chunk2/laundryBasket-no-uses.png` |
| `bathMat` Bath mat | OK | n/a | OK | | `bathroom/sheets/obj-bathMat.png`, `bathroom/play/chunk2/bathMat-no-uses.png` |

Measured (M1): washer and dryer 0.60 × 0.98 / 1.09 × 0.65 m with the towels and the basket on top (the cabinets 0.86 m: 0.82–1 ±10 %); rail bars at 0.95 and 1.2 m; ladder 1.71 m leaning 10.7°; basket 0.39 m high (0.4–0.7 −10 %); mat 0.76 × 0.5 × 0.025. The round doors swing 110° about the side opposite the handle (90–180°: OK), and the drum turns once in 1.2 s (50 rpm, a wash tumble).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `laundryWasher` | load the washing and start it; unload it and hang it out or move it to the dryer | do the laundry (30, standing at it) | the load is the whole wash, stood beside (I-bathroom-07) |
| `laundryDryer` | load the wet washing; take out the warm laundry and fold it | fold the laundry (20, at the tumbling drum) | folding while it tumbles (A-bathroom-03, I-bathroom-07) |
| `towelRail` | hang a towel to dry; take one | none | decor (§9 decision 4); the fresh towel is the tall cabinet's |
| `towelLadder` | hang towels on it | none | decor |
| `laundryBasket` | carry the washing; fold it | none | decor; folding is the dryer's (I-bathroom-07) |
| `bathMat` | stand on it out of the bath | none | decor |

## B5. Bathroom storage and mirrors (3)

Family: `tools/art/blender/families/bathroom.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `mirrorCabinet` Mirror cabinet | OK | I-bathroom-06 | I-bathroom-06 | I-bathroom-06 S2 content | `bathroom/sheets/obj-mirrorCabinet.png`, `bathroom/extra/mirror-over-sink/mirrorCabinet-brushTeeth-01.png`, `bathroom/clips/clips-mirrorCabinet-all-t99.png` |
| `mirrorRound` Round mirror | OK | n/a | I-bathroom-08 | I-bathroom-08 S3 content | `bathroom/sheets/obj-mirrorRound.png`, `bathroom/play/chunk2/mirrorRound-no-uses.png` |
| `bathCabinetTall` Tall bathroom cabinet | OK | OK | OK | I-held-01 S2 code (held ledger: no towel in hand) | `bathroom/sheets/obj-bathCabinetTall.png`, `bathroom/play/chunk3-short/bathCabinetTall-takeTowel-01.png`, `bathroom/clips/redo-tall/clips-bathCabinetTall-all-t99.png` |

Measured (M1): mirror cabinet 0.56 × 0.77 × 0.20 m hung from 1.2 to 1.92 m, its door 100° (95–110°) with the pull opposite the hinge; round mirror 0.6 m across at 1.55 m; tall cabinet 0.42 × 1.82 × 0.38 m, its upper door 100°, the open shelf's towels at 0.6–0.8 m. At the tall cabinet the resident stands at its front and reaches up as its door opens (`fetch`: OK).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `mirrorCabinet` | brush teeth in front of it (at the basin under it); take out toiletries | brush your teeth (4, by order only) | the right use on the wrong object, with no basin needed (I-bathroom-06) |
| `mirrorRound` | check your look; practise a speech | none | missing: the other mirrors have "Practise a speech" (I-bathroom-08) |
| `bathCabinetTall` | take a fresh towel; store toiletries | take a fresh towel (2, by order only) | OK |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### I-bathroom-01 — The laundry basket's lid isn't built, so it can't open

- **Severity:** S3
- **Rubric:** I1, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/laundry.py:193`, `web/public/content/catalog2/bath.json:270`
- **Evidence:** Left decorative for 0.3.0: the lid is not a part of its own, so no use can open it.
  Bathroom wave: the model is an open wicker basket with handles and clean washing heaped in it, as its description says ("a wicker basket of clean washing waiting to be folded"; `bathroom/sheets/obj-laundryBasket.png`). Such a basket has no lid; a lidded hamper for dirty clothes would be another object. Its real use, folding the washing, is the dryer's today (I-bathroom-07).
- **Status:** closed (bathroom wave: an open basket of clean washing has no lid to build)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5; wave: bathroom

### M-bathroom-01 — The base sink is both the bathroom basin and the kitchen sink

- **Severity:** S3
- **Rubric:** M2, M11
- **Fix class:** decision
- **Where:** `tools/art/blender/families/bathroom.py:156` (`h = 0.92`, the worktop and basin rim); `web/public/content/base.json:2449` (`category: bathroom`, `stations: ["dishes"]`); `web/public/content/housegen.json:96` (`sink` in 3 of 4 kitchen runs)
- **Evidence:** `sheets/obj-sink.png`, `sheets/kitchen-sinks.png` (generated-49, -82, -84: the same unit in a counter run beside the cooker). Its rim is 0.92 m: 1 cm over a washbasin's 0.76–0.91 (`reference.json` `basinPedestal`), inside a kitchen sink's 0.85–0.95. As a vanity unit with doors it reads right in both rooms; the strict rule only fails it as a basin. Decide which kind it is (and so which range applies), or split it into a kitchen sink and a bathroom basin.
  Bathroom wave: `cargo review sizes` now reads this top as 0.88 m: the tops of the painted shoulders under the worktop (`bathroom.py:169-173`), not the 0.92 m worktop. A tool artefact; the 0.92 m stands.
- **Status:** fixed (b3f6b82d)
- **After:** One kind for the base sink, `kitchenSink` (0.85–1.00 m): its 0.92 m rim is inside, and it reads right in both rooms.
- **Source:** pilot

### A-bathroom-01 — The shower's glass door never opens: the resident walks through it

- **Severity:** S2
- **Rubric:** A8, A10, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/bathroom.py:554` (`glass_door`, part of the still body) and `:999` (`SHOWER = {'uses': {'shower': 'water_on'}}`); `web/public/content/base.json:2564` (`inside: true`)
- **Evidence:** `play/shower-shower-00.png` to `-02.png`, `sheets/obj-shower.png`: the front has a sliding glass door with a handle, but it is built into the still body and no clip moves it; the resident steps in through the shut glass and showers behind it. The README's own example (`{'uses': {'shower': [{'clip': 'door_open', 'hold': 1}, 'water_on']}}`) is the fix: the door its own part, slid open as they step in and shut behind them, and again as they leave (`out`).
  Bathroom wave: the base shower's alone. `docs/objects/bathroom.md` (B2) says the door stays shut because a moving glass part would be a 4th draw call: a budget choice for triage. The walk-in and rain showers have an open way in (`bathroom/sheets/obj-showerWalkIn.png`, `obj-showerRain.png`); the stall's curtain is A-bathroom-02.
- **Status:** fixed (6190c955)
- **After:** The door is its own part (Q-05: the shower's 4th draw; its grip smoked glass, so the door is one): `door_open` slides it 41 cm left behind the fixed panel in 0.7 s as the use starts, shuts it after 1.5 s and opens it again for 1.5 s as the use ends (`out`); the water comes on 2 s in, behind the shut door (`fixes/blender-rooms/clips/clips-shower-all-t99.png`, `play/showers-shower-shower-00.png` to `-07.png`, `play-showers.jsonl`: `door_open` 3322–5738 ms and again 9204–11410 ms as the resident leaves, `water_on` from 5539). The resident is moved in by the sim at the use's start, so they are inside as the door begins to open and step out as it opens again: A-bathroom-05.
- **After (the pack showers, fix: blender-packs):** the packs ledger lists `neat.rainShowerTower` and `energetic.contrastShower` under this finding. Each now has a glass door of its own across the middle of its front (all glass, no pull, so one draw: the fourth, Q-05), narrow fixed panes either side, hinged on the right: `door_open` swings it 95° out for the bather to step in, shuts it after 1.5 s and opens it again for 1.5 s as the use ends (`out`) (0ff6cf97; `bench-results/review/fixes/blender-packs/clips/clips-neat.rainShowerTower-*.png`, `clips-energetic.contrastShower-*.png`, `play/day-neat.rainShowerTower-steamShower-00.png`, `day-energetic.contrastShower-rinse-00.png`).
- **Source:** pilot; wave: bathroom

### I-bathroom-02 — moved to residents.md

### I-bathroom-03 — moved to residents.md

### I-bathroom-04 — moved to systems.md

### M-bathroom-02 — The vessel basins' rims stand at 1.00–1.02 m, 9–11 cm over a basin's height

- **Severity:** S2
- **Rubric:** M2, M11
- **Fix class:** model
- **Where:** `tools/art/blender/families/bathroom.py:231` and `:238` (`vanityDouble`: worktop at 0.88, the bowls from 0.88 up to `rim_y = 1.0`); `:255` and `:260` (`sinkVessel`: worktop at 0.86, the bowl up to 1.02)
- **Evidence:** `bathroom/sheets/obj-vanityDouble.png`, `bathroom/sheets/obj-sinkVessel.png` (the gallery: the bowls' rims at a standing woman's hip bone). A washbasin's rim is 0.76–0.91 m (`reference.json` `basinPedestal`, `vanityDouble`); a vessel bowl sits on a lower worktop (about 0.75–0.80 m) so that its rim lands there. Here the worktops are at counter height and the bowls stand on top: 1.00 m and 1.02 m, against 0.88 m for the pedestal and console basins beside them in the catalogue. `cargo review sizes` missed it: its "top 0.84 / 0.82" is the underside of the slab (tooling, in the review log).
- **Status:** fixed (6190c955)
- **After:** The worktops come down to 0.76 m and the bowls (12 cm) stand on them, their rims at 0.88 m as the pedestal's and the console's; the taps come down with them (`fixes/blender-rooms/gallery/gallery-vanityDouble.png`, `gallery-sinkVessel.png`, `play/rooms-vanityDouble-wash-*.png`, `vessel2-sinkVessel-wash-*.png`). `cargo review sizes`: neither outside (it reads their worktops, 0.76).
- **Source:** wave: bathroom

### M-bathroom-03 — moved to systems.md

### A-bathroom-02 — The shower stall's curtain never closes

- **Severity:** S3
- **Rubric:** A10, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/bathroom.py:609-610` (`_pleats`: the curtain drawn back along the side and a third of the front, in the still body); `:999` (`SHOWER`: only `water_on`)
- **Evidence:** `bathroom/sheets/obj-showerStall.png`, `bathroom/play/chunk1/showerStall-shower-01.png`: the resident steps in through the open two thirds of the front and showers there with the curtain bunched at the side, the rain slanted onto them in plain view; nothing stops the water. A stall's curtain is drawn across once you're in and back as you leave. The curtain as its own part with a `door_open`-like clip (`hold` while in, `out` on leaving) fixes it.
- **Status:** fixed (6190c955)
- **After:** The curtain's front is its own part, gathered in the right corner at rest and drawn across the front 1 s into the use (`curtain_draw`, 0.9 s), back as the use ends; the side's curtain hangs closed along the whole side; the water comes on at 1.8 s. Plain satin now (a polyester curtain), so the stall keeps 3 draws (`fixes/blender-rooms/clips/clips-showerStall-all-t0.png`, `-t99.png`; `play/showers-showerStall-shower-05.png`: drawn, the bather behind it).
- **Source:** wave: bathroom

### A-bathroom-03 — The washer's drum turns with its door open; the dryer tumbles while the laundry is folded

- **Severity:** S3
- **Rubric:** A9, A10
- **Fix class:** motion
- **Where:** `tools/art/blender/families/laundry.py:226` (`LAUNDRY = {'uses': {'laundry': 'door_open'}, 'hold': 2.5, 'loops': {'laundry': 'drum_spin'}}`, the same for both machines)
- **Evidence:** `bathroom/play/chunk2/play.jsonl` and `chunk4-redo/play.jsonl` (clip events): on the washer `door_open` and `drum_spin` start in the same frame (4162 ms) and the drum goes round behind the open door for its 2.5 s hold; a washer won't turn until it is shut. On the dryer ("Fold the laundry", 20 min) the door opens and the drum tumbles the washing for the whole use (`bathroom/play/chunk2/laundryDryer-laundry-01.png`: the drum's washing turning while the resident stands at it to fold), where folding means the drum has stopped and the washing is out. Fix: the washer's spin after its door shuts (a delay, or a loop that waits for the door); the dryer's use opens the door with no spin.
- **Status:** fixed (d2c9f0de)
- **After:** the washer's drum turns from the moment its door has shut (`delay` 3.5 s, `every` 1.1 s, a turn each 1.2 s, till the wash ends): `fixes/motions/g1718/play-stage.jsonl`, `door_open` back at rest at +4.96 s, `drum_spin` from +4.97 s; the dryer opens its door to take the washing out and its drum stays still (no `drum_spin` clip).
- **Source:** wave: bathroom

### A-bathroom-04 — Bathers sit 10–12 cm above the tub's floor

- **Severity:** S3
- **Rubric:** M2, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/shared/characters/furniture.json:34-36` (`seats`: `bathtub` 0.25, `bathtubClawfoot` 0.28, `bathtubFreestanding` 0.27); `tools/art/blender/families/bathroom.py:796`, `:837`, `:868` (the tubs' floors at 0.15, 0.17 and 0.15 m)
- **Evidence:** heights warnings (`cargo check-glb`): the model is 10.0, 11.0 and 12.0 cm lower than the seat at the hips. `h` is the seat surface (`characters.ts:146`) and `legsOut` rests the heels at that height (`:3473`), so the bather's seat and heels ride on nothing over the floor. Once the water is in (two thirds of the way up the tub, `fill`, 2.4 s after they sit) it hides the gap, but it shows in those first seconds, and the shins and knees break the surface (`bathroom/play/chunk1/bathtub-bath-01.png`, `bathroom/play/chunk1/bathtubClawfoot-bath-01.png`). Seats at the floors (0.15–0.17 m), or the floors raised to the seats, line them up; then check that the shoulders still clear the rim.
- **Status:** fixed (96d54386)
- **After:** The three baths' seats at their floors (0.15, 0.17, 0.15): the bather sits and rests the heels on the tub floor, the shoulders over the rim at every stage (`fixes/tables/after/baths-sheet.png`, `play-b/b-bathtub-bath-02.png`); the clawfoot bather's arm drapes over its higher rim. Heights warnings gone.
- **Source:** wave: bathroom (T1 heights warnings)

### I-bathroom-05 — At the basins the washer stands a step back, the hands short of the tap

- **Severity:** S2
- **Rubric:** I2, I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:160` (`STAND_BACK`: no entry for the basins; a negative one steps in to a thing smaller than its tile, as `garden.herbs: -0.5`); `:465` (`ACTIONS.wash`: `spell`, `interact` at the chest)
- **Evidence:** `bathroom/play/chunk1/sinkPedestal-wash-01.png` (a clear air gap between the hands and the basin), `bathroom/play/chunk3-short/sinkConsole-wash-01.png`, `bathroom/play/chunk3-short/sinkVessel-wash-01.png`, `bathroom/play/chunk1/vanityDouble-wash-01.png`. The resident stands at the middle of the tile in front (1.0 m from the object's middle, `play.jsonl` x/z); the basins' fronts are 0.22–0.28 m from it and the streams about at it, so the feet are 0.7–0.8 m from the basin and the working hands at the chest stop about half a metre short of the water. People stand with their toes at the basin and lean over it. The pilot passed the base sink, where the hands at least reach its worktop's front (0.30 m out); the small basins show the gap plainly. Entries of about −0.4 to −0.5 m for `sinkPedestal`, `sinkConsole`, `sinkVessel` and `vanityDouble` (and the base `sink`, −0.3) bring the hands over the basin.
- **Status:** fixed (0c5e889e)
- **After:** `STAND_BACK`: `sink` -0.35, `sinkPedestal` -0.5, `sinkConsole`, `sinkVessel`, `vanityDouble` -0.45: the washer stands at the basin, the hands over it (`fixes/tables/play-a/a-sink*-wash-*.png`, `play-b/b-sinkPedestal-wash-*.png`).
- **Source:** wave: bathroom

### I-bathroom-06 — Teeth are brushed at the wall cabinet only, with no basin, no water and nothing in hand

- **Severity:** S2
- **Rubric:** I1, I2, I3, I16, A10
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:2530-2540` (`mirrorCabinet` `brushTeeth`: `anim: wash`, no `handwash` tag, `autonomous: false`); no basin has it: `furniture.json:2457` (`sinkPedestal`), `:2485` (`vanityDouble`, whose description at `:2497` promises "no more queueing to brush your teeth"), `catalog2/bath.json:137` (`sinkVessel`), `:166` (`sinkConsole`), `base.json:2449` (`sink`, I-kitchen-02); `tools/art/blender/families/bathroom.py:1052` (`door_open`, `hold: 1.5`)
- **Evidence:** `bathroom/play/chunk1/mirrorCabinet-brushTeeth-01.png`: on a bare wall the resident stands a tile out (1.27 m from the cabinet's front) for 4 minutes, hands working at the chest, empty; the door swings open by itself as the use starts, out of reach. Hung over a pedestal sink (`bathroom/extra/mirror-over-sink/mirrorCabinet-brushTeeth-01.png`) they stand at the basin, but the tap stays off (no `handwash` tag: no `water_on`, no tap sound, `world.ts:191`) and the hands still hold no brush (I-held-01, held ledger). The real use is the basin's, the mirror above it a help: today the one object with "Brush your teeth" is the cabinet, needing no basin anywhere (I16), and residents never choose it (`autonomous: false`), so nobody brushes their teeth unless told to. The pedestal sink even has a cup and toothbrush modelled on its deck (`bathroom.py:213-214`). Fix: `brushTeeth` on the basins (`wash`, the tap running), at least the vanity's two slots; the cabinet's door then opens for the basin's use under it, or the cabinet keeps a "take out toiletries" use.
- **Status:** fixed (eaca9ed8)
- **After:** "Brush your teeth" on the five basins (3 min, hygiene 0.03, tags `hygiene` and `handwash` after Wash hands, so a toilet's follow-up still washes hands): the tap runs (`water_on`) and sounds; residents choose it. With hygiene 0.05 the busy-player soak had no birth in exactly its 120-hour window (bisected to this use); at 0.03 all four soak tests pass. Balance (180 days): median weekly savings 369 → 395, richest 12 742 → 14 520, grades 1.2/2 → 1.3/3, all inside the targets. The cabinet keeps its own use; nothing is held in hand (G-70).
- **Source:** wave: bathroom (from the evidence's "Seen (not reviewed)"; extra run `bathroom/extra/mirror-over-sink/`)

### I-bathroom-07 — Doing the laundry is a stand at the running machine

- **Severity:** S3
- **Rubric:** I1, I3, I7
- **Fix class:** code
- **Where:** `web/public/content/furniture.json:2574` (`laundryWasher` "Do the laundry", 30 min), `:2603` (`laundryDryer` "Fold the laundry", 20 min); `web/src/render/babylon/characters.ts:503` (`ACTIONS.clean` for `laundry`: `spell`, `interact`, `pickUp`)
- **Evidence:** `bathroom/play/chunk4-redo/laundryWasher-laundry-01.png`, `bathroom/play/chunk2/laundryDryer-laundry-01.png`. A wash is loaded in 2–5 minutes (`reference.json` `washer`) and then runs by itself; here the resident stands upright at the washer for the whole 30 minutes, hands at the chest while the drum turns, and the machine stops when they leave. At the dryer they fold for 20 minutes standing in front of its tumbling drum, with no washing in the hands (I-held-01) and nowhere to fold it; the laundry basket of clean washing (`laundryBasket`) plays no part, and nothing passes from washer to dryer. A machine that runs on after a short load (a timed state, as cooking's steps), and folding at the basket or a surface, is new behaviour (§9 decision 3). Durations themselves are by design.
- **Status:** fixed in part (69d0bc33): something to work on and the washer-to-dryer step; the machine running on by itself and folding at the basket are left (new behaviour)
- **After:** Washing piles up (G-64, content `laundry`: 0.4 loads a resident in town a day, at most 4 waiting; saved, `households[].laundry`, a new household and an older save start with one dirty and one washed load). *Do the laundry* (and the neat laundry centre's wash) is `"workOn": "laundry"`: only with a dirty load, which it washes; *Fold the laundry* at the dryer (and *Hang out the washing* on the line) is `"workOn": "washed"`: only with a washed load, which it takes. On `review-stage` the dryer folds the washed load, the washer washes the dirty one, the dryer folds it, and the next wash is refused, "Do the laundry: there's no washing to do" (`fixes/tops-conditions/play-laundry.jsonl`; the play runner's retry reopens the state, so its second try washes again); free will does no laundry with none waiting (`tests/tops_and_conditions.rs` `free_will_does_no_chore_with_nothing_to_work_on`, `the_laundry_is_washed_then_dried_and_piles_up_again`). Still as before: the resident stands at the washer for its 30 minutes and folds at the dryer with empty hands; the laundry basket plays no part (a timed machine and a fold at the basket are new behaviour, not in G-64). Paths under `bench-results/review/fixes/tops-conditions/`.
- **Source:** wave: bathroom

### I-bathroom-08 — The round mirror has none of the mirrors' uses

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/catalog2/bath.json:254` and `:267` (`mirrorRound`: `interactions: []`)
- **Evidence:** `bathroom/sheets/obj-mirrorRound.png`. The wall mirror (`furniture.json:2239`, "check your look, or practise a speech") and the standing mirror (`base.json:3194`) both have "Practise a speech"; the round mirror, hung at the same height for the same look in the mirror, has nothing (`tables/uses-bathroom.md` flags "no use"). Not decor in the §9 sense: its siblings have the use.
- **Status:** fixed (eaca9ed8, 1222c510)
- **After:** The round mirror checks the look and practises a speech (`bench-results/review/fixes/content/g12/after-mirrorRound-*.png`); `cargo review uses` no longer flags it.
- **Source:** wave: bathroom (`cargo review uses` against the mirrors' siblings)

### A-bathroom-05 — The bather is inside the shower before its door opens, and steps out as it opens

- **Severity:** S3
- **Rubric:** A2, A4
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:297` (`STEP_IN`: no entry for the showers; `inside` uses are moved in by sim-core at the use's start, `crates/sim-core/src/world.rs:5218`, and drawn there after one snapshot's glide); `tools/art/blender/families/bathroom.py` (`SHOWER_DOOR`: `door_open` from the use's start, `out` 1.5 s at its end)
- **Evidence:** `fixes/blender-rooms/play/showers-shower-shower-00.png` and `play-showers.jsonl`: the resident is using the shower and standing in its middle at 3304 ms; the door starts to slide at 3322 ms, so it opens round someone already through the glass (in one snapshot's glide). At the end the use stops at 9189 ms and they are back on the tile while the door is only starting to open again (9204–9905 ms). The phone box times its caller's steps to its door with `STEP_IN` (`wait`, `back`, `out`); the base shower wants the same (wait about 0.7 s while the door slides, step in through the right half; wait `out` before stepping out). The stall's curtain is drawn once they're in (`delay` 1 s), so it doesn't need it.
- **Status:** fixed (a5968771)
- **After:** the renderer walks the step sim-core takes in one tick (`characters.ts` `WALK_IN`, shared with A-residents-13), and for the base shower waits for its door (`WALK_WAIT.shower`: 0.7 s standing on the tile while `door_open` slides, then in through the opening, 0.21 m to the shower's right, turned round inside; at the end 0.7 s for the door to open again, out the same way). Done in the walk-in rather than a `STEP_IN` entry: sim-core already stands the bather inside (content `inside`), and `STEP_IN` steps in from the tile sim-core stands them on. `fixes/sweep-render/shower/strip-shower.png` (`shower-in-*`, `shower-out-*` with `shower.json`: on the tile 0–0.7 s with the door opening, walking in 0.7–2.0 s, the door shutting behind from 1.5 s; out: waiting while it opens, out by 2.0 s), `shower-in-3400.png` (in the towel under the water). Against `fixes/blender-rooms/play/showers-shower-shower-00.png` (in the middle before the door moved).
- **Source:** fix: blender-rooms (A-bathroom-01's re-check)

### A-bathroom-06 — The bather is dressed again the moment the shower ends, still inside it

- **Severity:** S3
- **Rubric:** A4, I3
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts` (`wearFor`: the towel only while the use's tag is `shower`; the walk out, `WALK_WAIT`)
- **Evidence:** `fixes/sweep-render/shower/shower-out-0.png` … `-900.png`: as the use ends the towel gives way to the day clothes at once, while the bather still stands under the head waiting 0.7 s for the door, then walks out dressed. Someone steps out in the towel and dresses outside (or the shower's walk out keeps the towel until they are clear of it).
- **Status:** fixed (da7e41ef)
- **After:** the towel stays on as they step out (`characters.ts` `toweled`, set while under the shower): through the 0.7 s wait for the door and the walk out, and 0.8 s once on the tile (`TOWEL_OUT`), then the day clothes. The step out now also finds its shower from the one they were under (`Characters.shower`): the use's end and the sim's step back to the tile can come a tick apart, and with the use gone the walk-out had lost its door wait. `fixes/sweep-render/shower2/strip-out.png` (`shower-out-0` … `-3200`: in the towel waiting while the door opens, 0–700 ms, walking out in it 1100–2000, on the tile at 2600; was `fixes/sweep-render/shower/shower-out-*`'s day clothes inside), `strip-later.png` (dressed by 3600 ms).
- **Source:** fix: sweep-render (A-bathroom-05's re-check)

