# Outdoor: authenticity ledger

What it covers: the 43 objects of [docs/objects/outdoor.md](../objects/outdoor.md), in its groups and order.

- **Evidence:** `bench-results/review/outdoor/` in the review worktree (gallery, game camera, clip, pose and play shots, contact sheets in `sheets/`, its `index.md`); shots stay out of git. Shots below are relative to that folder. The outdoor wave's extra shots are in `extra/` (a top-down picnic, a ripe raised bed's harvest, night lights and fires, the rain barrel against a wall, a grill station close-up); how they were taken is in `review-log.md` there.
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Durations** are judged by §9 decision 2: today's short ones are by design; only outliers against similar uses are flagged. None of the outdoor uses is one (the gnome's 2-minute pat against the `tend` median of 30 is a gesture, not a garden job).

## OU1. Patio seating and tables (11)

Family: `tools/art/blender/families/outdoor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bench` Park bench | M-outdoor-02 | OK | OK | M-outdoor-02 S3 model | `sheets/obj-bench.png`, `play/bench-sit-*.png`, `sheets/seat-benchM.png`, `sheets/seat-benchF.png`, `play-elder/bench-sit-*.png` |
| `patioSet` Folding bistro set | M-outdoor-08 | OK | I-outdoor-07, I-held-01 | M-outdoor-08 S3 model; I-outdoor-07 S3 code; I-held-01 S2 code (held ledger: no cup in hand) | `sheets/obj-patioSet.png`, `sheets/age-patioSet.png` |
| `picnicTable` Picnic table | M-outdoor-03 | OK | I-outdoor-07, I-held-01 | M-outdoor-03 S2 model; I-outdoor-07 S3 code; I-held-01 S2 code (held ledger) | `sheets/obj-picnicTable.png`, `extra/picnic-top/top-picnicTable-picnic-01.png`, `sheets/age-picnicTable.png` |
| `gardenChair` Garden chair | M-outdoor-09 | OK | OK | M-outdoor-09 S3 renderer-table | `sheets/obj-gardenChair.png`, `sheets/age-gardenChair.png` |
| `patioTableRound` Round patio table | OK | n/a | OK | | `sheets/obj-patioTableRound.png`, `sheets/obj-patioChair.png` |
| `patioTable4` Teak patio table | OK | n/a | OK | | `sheets/obj-patioTable4.png` |
| `patioChair` Patio chair | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: the coffee has no cup at all) | `sheets/obj-patioChair.png`, `sheets/age-patioChair.png` |
| `adirondackChair` Adirondack chair | OK | OK | OK | | `sheets/obj-adirondackChair.png`, `sheets/age-adirondackChair.png` |
| `benchGarden3` Garden bench | M-outdoor-08 | OK | OK | M-outdoor-08 S3 model | `sheets/obj-benchGarden3.png`, `sheets/age-benchGarden3.png` |
| `outdoorBar` Outdoor bar | M-outdoor-08 | n/a | I-outdoor-08 | M-outdoor-08 S3 model; I-outdoor-08 S3 content | `sheets/obj-outdoorBar.png` |
| `parasol` Parasol | OK | OK | OK | | `sheets/obj-parasol.png` |

Notes: the bistro set's and the picnic table's seats and tops are where the content says (0.44 / 0.72 and 0.40 / 0.63): the heights check's "no model under it" falls in the gaps between slats (below the Findings). The Adirondack chair shares the garden chair's frame (seat 0.42, sloped 8.5°) with a fan back: a real one sits lower and more raked, but it is inside the garden-chair range and reads as an Adirondack. The picnic sitter sits on the back bench facing across the table (`extra/picnic-top/`); the "short end" seen in the play and pose shots was the low camera.

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bench` | sit and enjoy the garden, chat, feed the birds | sit on the bench (30, two slots) | OK |
| `patioSet` | breakfast or a coffee for two, sit in the sun | have a coffee (20, needs a coffee machine), sit in the sun (30); one slot | OK; only one of its two chairs is used (I-outdoor-07), no cup (I-held-01) |
| `picnicTable` | eat outdoors with family, a picnic, play cards | have a picnic (25), have a coffee (20); two slots, both on one bench | OK; the two sit side by side, not across (I-outdoor-07) |
| `gardenChair` | sit outside, read in the sun, a drink on the arm | sit in the garden (30) | OK |
| `patioTableRound` | coffee or a meal for two | none of its own: a seat at it (`patioChair` `seatAt`), meals served on it | OK |
| `patioTable4` | an outdoor meal for four | none of its own, as above | OK |
| `patioChair` | sit at a table, coffee | sit down (20), have a coffee (20, at a table) | OK |
| `adirondackChair` | lounge in the sun with a drink | relax in the sun (30) | OK |
| `benchGarden3` | sit, three together | sit on the bench (30, three slots) | OK |
| `outdoorBar` | have a drink at it on a stool, serve drinks | none; bar stools sit at it, meals can be put on it | a bar where no one has a drink (I-outdoor-08) |
| `parasol` | shade, opened and closed | none | decor, no use needed |

## OU2. Loungers, swings and the hot tub (5)

Family: `tools/art/blender/families/outdoor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sunLounger` Sun lounger | OK | OK | I-outdoor-02 | I-outdoor-02 S2 code | `sheets/obj-sunLounger.png`, `sheets/age-sunLounger.png` |
| `hammock` Hammock | M-outdoor-05 | M-outdoor-05 | I-outdoor-02 | M-outdoor-05 S2 model; I-outdoor-02 S2 code | `sheets/obj-hammock.png`, `sheets/age-hammock.png` |
| `porchSwing` Porch swing | M-outdoor-08 | OK | OK | M-outdoor-08 S3 model | `sheets/obj-porchSwing.png`, `sheets/age-porchSwing.png` |
| `swingSet` Swing set | M-outdoor-08 | I-outdoor-03 | I-outdoor-03 | I-outdoor-03 S2 renderer-table; M-outdoor-08 S3 model | `sheets/obj-swingSet.png`, `sheets/age-swingSet.png`, `extra/swing-hands.png` |
| `hotTubCedar` Cedar hot tub | A-outdoor-01 | A-outdoor-01 | I-bathroom-03 | A-outdoor-01 S2 model; I-bathroom-03 S3 decision (residents ledger: soaking in day clothes) | `sheets/obj-hotTubCedar.png`, `clips/clips-hotTubCedar-all-t99.png` |

Notes: the lounger's cushion top is 0.35 m, as `BEDS.sunLounger` (the check's 0.28 is the gap between the seat cushion and the leg cushion); its back folds nearly flat for both uses, which is how a lounger is lain on. Children's feet dangle on the garden seats, as they do.

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `sunLounger` | sunbathe, doze, read lying in the sun | sunbathe (40), doze in the sun (45) | OK; dozing puts on pyjamas (I-outdoor-02) |
| `hammock` | lie and swing, doze, read | swing in the hammock (30), doze (45) | OK; lain in folded, as in a sling chair (M-outdoor-05) |
| `porchSwing` | swing gently, sit together | swing on the porch swing (30, two slots) | OK |
| `swingSet` | children swing; push a child; grown-ups have a go | swing (20, two slots) | OK; pushing a child missing (minor, not filed); hands not on the chains (I-outdoor-03) |
| `hotTubCedar` | soak in warm water, with friends; put the cover on and off | soak (45, two slots, opens it and closes it after), put the cover on (2, when left open) | OK |

## OU3. Grills and fire (3)

Family: `tools/art/blender/families/outdoor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `grill` Charcoal grill | M-outdoor-04, M-outdoor-01 | OK | I-systems-04 | M-outdoor-04 S2 model; M-outdoor-01 S2 code (seeded, confirmed); I-systems-04 S2 code (systems ledger: no age rule) | `sheets/obj-grill.png`, `extra/night/sheet-fire.png` |
| `grillStation` Gas grill station | OK | OK | I-systems-01, I-systems-04 | I-systems-01 S2 renderer-table (systems ledger, seen here: the board drawn on the grill); I-systems-04 S2 code (systems ledger) | `sheets/obj-grillStation.png`, `extra/grillstation-crop.png` |
| `firePit` Fire pit | OK | I-outdoor-01 | I-outdoor-01 | I-outdoor-01 S1 content | `sheets/obj-firePit.png`, `sheets/age-firePit.png`, `extra/night/sheet-fire.png` |

Notes: the grills' lid and hood are hinged at the back and swing up about 95° and 75°, the cook flow is fridge (in the house) → prep → grill → served; the grill station's counter is 0.90 m.

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `grill` | light the charcoal, grill food | grill dinner (`bbq`: ingredients, prep, grill) | OK |
| `grillStation` | grill food for a crowd | grill dinner (`bbq`), a barbecue feast (`bbqFeast`) | OK |
| `firePit` | light a fire, sit round it and warm up, toast marshmallows | warm up by the fire (30, sitting on its rim) | the right use, in the wrong place (I-outdoor-01); toasting marshmallows missing (minor, not filed) |

## OU4. Play and yard (9)

Family: `tools/art/blender/families/yard.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `sandbox` Sandpit | OK | OK | OK | | `sheets/obj-sandbox.png`, `sheets/age-sandbox.png` |
| `bikeRack` Bike rack | OK | OK | OK | | `sheets/obj-bikeRack.png` |
| `raisedBed` Raised vegetable bed | OK | A-outdoor-02 | A-outdoor-02, I-held-01 | A-outdoor-02 S3 code; I-held-01 S2 code (held ledger: no can, no basket) | `sheets/obj-raisedBed.png`, `extra/harvest/harvest-sheet.png` |
| `shedSmall` Garden shed | OK | OK | OK | | `sheets/obj-shedSmall.png` |
| `greenhouse` Greenhouse | OK | OK | I-outdoor-04, I-outdoor-05 | I-outdoor-04 S2 renderer-table; I-outdoor-05 S2 content | `sheets/obj-greenhouse.png` |
| `compostBin` Compost bin | OK | OK | I-outdoor-04 | I-outdoor-04 S2 renderer-table | `sheets/obj-compostBin.png` |
| `rainBarrel` Rain barrel | M-outdoor-06 | n/a | OK | M-outdoor-06 S2 decision | `sheets/obj-rainBarrel.png`, `extra/wall/wall-rainBarrel-no-uses.png` |
| `clothesline` Washing line | OK | A-outdoor-02 | A-outdoor-02 | A-outdoor-02 S3 code | `sheets/obj-clothesline.png` |
| `firewoodStack` Log store | OK | n/a | OK | | `sheets/obj-firewoodStack.png` |

Notes: the washing line's 0.76 m depth is outside the reference range, which is for a single line; three lines on 0.7 m T-arms are that deep (not filed). The tuner stands by the bike's front wheel while the lifted back wheel turns (polish, not filed). The shed is pottered in from its open door, which reads as rummaging in it.

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `sandbox` | build sandcastles, dig (children mostly) | build a sandcastle (30) | OK |
| `bikeRack` | park the bike, tune it up, ride off | tune up the bike (30) | OK (there's no riding in the game) |
| `raisedBed` | sow, weed and water, harvest | weed and water (30), harvest vegetables (20, when ripe) | OK; the crop is gone in two seconds (A-outdoor-02) |
| `shedSmall` | store tools, potter about | potter in the shed (45) | OK |
| `greenhouse` | tend seedlings and tomatoes, pick them, sit among the plants | tend the seedlings (45), pick tomatoes (20) | OK; done from outside the door, kneeling (I-outdoor-04, I-outdoor-05) |
| `compostBin` | tip in scraps, turn the compost | turn the compost (15) | OK; kneeling at a waist-high bin (I-outdoor-04) |
| `rainBarrel` | fill a watering can | none | decor, no use needed (the watering uses don't need water) |
| `clothesline` | hang out the washing, take it in | hang out the washing (20), take it in (10) | OK |
| `firewoodStack` | fetch logs for the fire | none | decor, no use needed |

## OU5. Pond, small garden things and lights (9)

Family: `tools/art/blender/families/garden_things.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `garden.pond` Garden pond | OK | OK | OK | | `sheets/obj-garden.pond.png` |
| `planterBox` Planter box | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no can) | `sheets/obj-planterBox.png` |
| `birdhouse` Birdhouse | OK | n/a | OK | | `sheets/obj-birdhouse.png` |
| `birdFeederPole` Bird feeders | OK | OK | I-outdoor-06 | I-outdoor-06 S3 content | `sheets/obj-birdFeederPole.png` |
| `dogHouse` Dog house | OK | n/a | OK | | `sheets/obj-dogHouse.png` |
| `sundial` Sundial | OK | n/a | OK | | `sheets/obj-sundial.png` |
| `steppingStones` Stepping stones | OK | n/a | OK | | `sheets/obj-steppingStones.png` |
| `lampPost` Garden lamp post | OK | n/a | OK | | `sheets/obj-lampPost.png`, `extra/night/sheet-lights.png` |
| `stringLights` String lights | OK | OK | OK | | `sheets/obj-stringLights.png`, `extra/night/sheet-lights.png` |

Notes: the pond's 0.80 m height is its reeds (the reference range is the rim's). All three lights glow at night (`extra/night/sheet-lights.png`).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `garden.pond` | watch and feed the fish | watch the fish (20), feed the fish (5) | OK |
| `planterBox` | water and tend the flowers | water the flowers (5) | OK |
| `birdhouse` | watch birds nest | none | decor, no use needed |
| `birdFeederPole` | fill it with seed, watch the birds from a distance | watch the birds (20, standing at it) | OK; watched from arm's length, refilling missing (I-outdoor-06) |
| `dogHouse` | a dog sleeps in it | none | decor (no pets) |
| `sundial` | tell the time by the sun | none | decor, no use needed |
| `steppingStones` | walk across the lawn | none | decor, no use needed |
| `lampPost` | light the path at night | its light (a lamp that switches) | OK |
| `stringLights` | light the patio at night | its light | OK |

## Garden decor (6)

Family: `tools/art/blender/families/garden_decor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `garden.gnome` Garden gnome | OK | OK | OK | | `sheets/obj-garden.gnome.png` |
| `garden.birdbath` Bird bath | OK | n/a | I-outdoor-06 | I-outdoor-06 S3 content | `sheets/obj-garden.birdbath.png` |
| `garden.fountain` Stone fountain | OK | A-outdoor-03 | A-outdoor-03 | A-outdoor-03 S3 motion; I-bathroom-04 S3 code (systems ledger: no sound) | `sheets/obj-garden.fountain.png` |
| `garden.lantern` Stone lantern | OK | n/a | OK | | `sheets/obj-garden.lantern.png`, `extra/night/sheet-lights.png` |
| `garden.wheelbarrow` Wheelbarrow planter | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no can) | `sheets/obj-garden.wheelbarrow.png` |
| `garden.boulder` Garden boulders | M-outdoor-07 | n/a | OK | M-outdoor-07 S2 model | `sheets/obj-garden.boulder.png` |

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `garden.gnome` | look at it, pat it for luck | pat the gnome (2, not on free will) | OK |
| `garden.birdbath` | watch birds bathe, top up the water | watch the birds (20, standing at it) | OK; watched from arm's length, topping up missing (I-outdoor-06) |
| `garden.fountain` | listen to the water, toss a coin | toss a coin and wish (5), listen to the water (20) | OK; the water doesn't move or sound (A-outdoor-03) |
| `garden.lantern` | light the garden at night | its light | OK |
| `garden.wheelbarrow` | water and smell the flowers | water the flowers (10), smell them (5) | OK |
| `garden.boulder` | sit on one, admire them | none | decor, no use needed |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

**Heights warnings closed as artefacts** (outdoor wave): `cargo check-glb`'s "no model under it" for `patioSet` (seat, depth −0.62), `picnicTable` (seat, depth −0.68) and `adirondackChair` (seat), and its lower readings for the `patioSet` top (0.363 at depth 0.30), `sunLounger` (0.28 at depth 0) and `firePit` (0.30 at depth 0.38), all probe one point that falls in a gap between slats, planks, cushions or capstones (`outdoor.py:179-191`, `:200-203`, `:421-425`, `:750-753`). The parts there are at the content's heights. `cargo review sizes`'s grill "top 0.71" is not the grate (0.965, M-outdoor-04). Reported as a tool problem in the wave's log.

### M-outdoor-01 — moved to systems.md

### M-outdoor-02 — The park bench's slats are 4 cm above where the sitter is put

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/outdoor.py:153` (seat slats at 0.47 m, 3.5 cm thick: top 0.49); `web/src/render/babylon/characters/furniture.json:5` (`SEATS.bench` 0.45)
- **Evidence:** `cargo check-glb` heights warning (`model.bench seat 0.45 m … the model 0.491 m there: 4.1 cm higher`) and `sizes-outdoor.md` (seat 0.491 > 0.48, the top of the 0.42–0.48 range). In play (`play/bench-sit-01.png`, `sheets/seat-benchF.png`) the thighs hide the 4 cm the pelvis sinks into the slats, so it reads as seated; it fails the strict range by 1 cm and the three-way check by 4 cm. Lowering the slats to 0.45 fixes both; raising `SEATS.bench` alone would leave the seat out of range.
- **Status:** open
- **Source:** pilot (T1 warning, sizes table)

### M-outdoor-03 — The picnic table is 7 cm low at the top and 2 cm low on the benches

- **Severity:** S2
- **Rubric:** M2, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/outdoor.py:200` (top planks at 0.61, 4 cm: top 0.63) and `:203` (bench planks at 0.38: top 0.40); `web/public/content/furniture.json:4577` (`surface.height` 0.63); `web/src/render/babylon/characters/furniture.json:47` (`SEATS.picnicTable` 0.40)
- **Evidence:** `tables/sizes-outdoor.md` (top 0.643 < 0.70, seat 0.400 < 0.42; reference `picnicTable` top 70–78, seat 42–48). The three ways agree with each other, so the check is quiet, but both heights are out of range, the top by 7 cm, and the gap between bench and top (0.23 m) is about 7 cm short of a real one's 0.28–0.32: the picnicker's knees and the table top nearly meet (`sheets/obj-picnicTable.png`, `extra/picnic-top/top-picnicTable-picnic-01.png`). Raise the top to about 0.73 and the benches to 0.44, with the content `surface` and `SEATS` row.
- **Status:** open
- **Source:** outdoor wave (sizes table)

### M-outdoor-04 — The charcoal grill's bowl stands 20 cm high: the grate at 0.97 m

- **Severity:** S2
- **Rubric:** M1, M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/outdoor.py:627` (`KETTLE_R, KETTLE_RIM = 0.3, 0.99`, "old part"), `:635-636` (bowl from 0.64), `:646` (grate at 0.965), `:659` (legs from 0.68)
- **Evidence:** The rim is at 0.99 m and the grate at 0.965, the lid's top at 1.24 (`sizes-outdoor.md`: H 1.242 > 1.21). A 57 cm kettle grill's rim is at about 0.75–0.8 m and it stands about 1.0–1.07 m with its lid (reference `kettleGrill` 0.9–1.1 m, top 70–80). In play the rim is at the cook's elbow (`sheets/obj-grill.png`, `play/chunk5-kitchen/grill-cook-01.png`), the height of the gas station's counter. Shorten the legs by about 0.2 m. (The sizes table's "top 0.71" is a wrong point, not the grate.)
- **Status:** open
- **Source:** outdoor wave (sizes table, `outdoor.py`)

### M-outdoor-05 — The hammock is 1.86 m long: residents lie in it folded, as in a sling chair

- **Severity:** S2
- **Rubric:** M1, M3, I3 (A1)
- **Fix class:** model
- **Where:** `tools/art/blender/families/outdoor.py:481-499` (stand base 1.86 m, posts at ±0.89; `_sling('sling', 0.62, 0.66, 0.6, 0.86, …)`: a bed 1.32 m long and 0.62 wide); `web/public/content/furniture.json:4703` (footprint 1 × 2); `web/src/render/babylon/characters/furniture.json:67` (`BEDS.hammock` sling)
- **Evidence:** `sizes-outdoor.md` (0.70 × 1.86 against a stand hammock's 1–1.6 × 2.7–4 m). An adult (1.75 m) on a 1.32 m sling lies in a V, knees up and head high (`sheets/obj-hammock.png`, `play/chunk2/hammock-doze-01.png`); the elder sits up in it (`sheets/age-hammock.png`, `play/chunk9-lie-elder/hammock-swing-01.png`). The packs' hammocks, lain across a longer bed, read right in the same pose shots (`poses/poses-only-hammock-default.png`). A longer stand and bed (footprint 1 × 3, about 2.8 m) is the fix.
- **Status:** open
- **Source:** outdoor wave (sizes table, Phase 0 "Seen")

### M-outdoor-06 — The rain barrel's downpipe ends in mid-air, wherever the barrel stands

- **Severity:** S2
- **Rubric:** M4, M7
- **Fix class:** decision
- **Where:** `tools/art/blender/families/yard.py:776-782` (`downpipe` up to 2.10 m behind the barrel, a collar on top); `web/public/content/catalog2/outdoor.json:495-509` (a floor object, any tile)
- **Evidence:** On the lawn the pipe stands free and stops at 2.1 m (`play/chunk3/rainBarrel-no-uses.png`). Against the house's back wall (`extra/wall/wall-rainBarrel-no-uses.png`, `rainBarrel@8,18,0`) it stops 0.7 m under the eaves of a 2.8 m storey, with no gutter, and in the cut-away view stands over the lowered wall. A downpipe comes down from a gutter. Either drop the pipe (a barrel with a lid inlet, as under a shed's gutter) or run it to the roof and require a wall behind: the owner's call. `sizes-outdoor.md`'s H 2.115 is the pipe; the barrel is 0.96.
- **Status:** open
- **Source:** outdoor wave (lead; Phase 0 "Seen")

### M-outdoor-07 — The garden boulders read as eggs, not rock

- **Severity:** S2
- **Rubric:** M5, M9
- **Fix class:** model
- **Where:** `tools/art/blender/families/garden_decor.py:147-158` (`boulder`: three `G.boulder`s with the default low noise, scaled to smooth spheroids, pale `stone`, flat moss pads on top)
- **Evidence:** `sheets/obj-garden.boulder.png`: smooth, pale, rounded, egg-shaped stones with moss lying on them as flat green discs; from the game camera they read as eggs or a toy turtle, not weathered stones. Real garden boulders are faceted and flat-bottomed, wider than tall, darker and mottled, with moss in their cracks. More noise and flattening, a darker mottled finish, and the stones sunk a little into the grass.
- **Status:** fixed (d81aded7)
- **After:** three stones broader than tall (1.01 × 0.47 × 0.92 m, inside `boulders`), roughened and split into flat faces, their feet cut flat where they go into the lawn (the woods' rock's cuts, `ground.py`), in darker greys (their own `rock` materials; the other decor keeps its dressed `stone`), moss in patches on the tops and in the crack between the two big ones, pale lichen on the faces, the two small stones on one side so the group reads as a heap, not a shell with a head and a tail (`bench-results/review/fixes/blender-trees/gallery/gallery-garden.boulder.png`, `gallery-game/gallery-game-garden.boulder.png`). Its colour parts are now `rock`, `rock_dark`, `rock_warm` (labels unchanged).
- **Source:** outdoor wave (lead; Phase 0 "Seen")

### M-outdoor-08 — Overall sizes outside the range: a long bench, a narrow swing frame, a deep bar, a high swing seat, a bistro set pulled apart

- **Severity:** S3
- **Rubric:** M1, M2, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/outdoor.py:341` (`benchGarden3` slats 2.7 m), `:510` (`_a_frame` legs at ±0.42 m, shared by the porch swing and the swing set), `:367` (`outdoorBar` counter 0.88 deep), `:538` (`porchSwing` cushion top 0.565), `:191-192` (`patioSet` chairs centred at −0.62 and +0.72 round a table from 0.03 to 0.57)
- **Evidence:** `sizes-outdoor.md`: the garden bench is 2.70 m for three (range 1.2–1.9; 0.9 m a person); the swing set is 0.91 deep (1.2–2.2), its A-frames too upright for seats swinging 20°; the outdoor bar is 0.89 deep (0.40–0.75); the porch swing's seat is 0.571 (42–55; content 0.55). The bistro set is 1.86 m deep (0.6–1.3) because its chairs stand 46 cm back from the table's edges: the sitter's knees are well short of it (`sheets/obj-patioSet.png`). Each reads as its thing; side by side with real ones they're off.
- **Status:** open
- **Source:** outdoor wave (sizes table)

### M-outdoor-09 — The garden chair's sitter is put 3.4 cm into the slats

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters/furniture.json:48` (`SEATS.gardenChair` 0.40); `tools/art/blender/families/outdoor.py:244-245` (`_slatted_chair` seat slats, 0.40–0.42 sloping, their tops about 0.43)
- **Evidence:** `cargo check-glb` heights warning (`model.gardenChair seat 0.40 m … the model 0.434 m there: 3.4 cm higher`). The mesh is inside the range (30–48), so the table row is what is off; in play the thighs hide it (`sheets/obj-gardenChair.png`). Set `SEATS.gardenChair.h` to 0.43.
- **Status:** fixed (a920f174)
- **After:** `seats.gardenChair` 0.43; its heights warning gone (`fixes/tables/play-b/b-gardenChair-sit-01.png`).
- **Source:** outdoor wave (T1 warning)

### A-outdoor-01 — The hot tub's open cover stands upright, floating above and behind the tub

- **Severity:** S2
- **Rubric:** A8, M7, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/outdoor.py:590-597` (the cover one rigid disc, `G.hinged(… F.at(0, 0.84, -TUB_R - 0.02))`, `G.swing(cover, 'cover_off', -105, …)`); `:568-569` (the staves' inside: a floor at 0.12, no bench); `web/src/render/babylon/characters/furniture.json:37` (`SEATS.hotTubCedar` 0.35)
- **Evidence:** `clips/clips-hotTubCedar-all-t99.png`, `play/chunk2/hotTubCedar-soak-01.png`: opened, the 1.9 m cover stands on end behind the tub, its lower edge clear of the rim, a grey disc hanging in the air above the soaker. A hot tub's cover folds in half on its seam (the model has the seam, `cover_seam`) and lies back over a lifter or on the ground beside the tub. While rebuilding it: the sitter is placed at 0.35 over a floor at 0.12 with no bench under them, hidden only by the opaque water; a bench ring at 0.35 would make the three heights agree.
- **Status:** open
- **Source:** outdoor wave (lead; Phase 0 "Seen")

### A-outdoor-02 — moved to systems.md

### A-outdoor-03 — The stone fountain's water stands still and makes no sound

- **Severity:** S3
- **Rubric:** A11, I9
- **Fix class:** motion
- **Where:** `tools/art/blender/families/garden_decor.py:77-92` (`fountain`: a still jet and drop, no clip; `no `motions` in its `G.Model`, `:163-175`); `web/src/audio/world.ts:34` (`ACTION_SOUNDS`: nothing for `listen` or the fountain)
- **Evidence:** `sheets/obj-garden.fountain.png`: the jet is a static blue rod with a drop on it, the basins flat; "Listen to the water" (20 minutes) plays to silence. The pond has its ripple (`idle`) and the parasol its stir; a fountain's moving water is what makes it one. An ambient clip (water rings in the basins, a pulsing jet) and a water sound. The sound part widens I-bathroom-04 (systems ledger).
- **Status:** fixed (abfc32d9), the water; its sound is fixed with I-bathroom-04 (54139115): the fountain runs by itself, `ambient: fountain`
- **After:** `flow` (ambient, `idle`): the jet bubbles up and falls back and a ring swells out on each basin's water and settles, 1.2 s a round, all the time near the camera (`fixes/motions/g1718/views/fountain-t0.6.png`, `clips-garden.fountain-all-t*.png`). It still plays to silence (G-53).
- **Source:** outdoor wave

### I-outdoor-01 — Warming up by the fire pit is sitting on its rim, back to the flames

- **Severity:** S1
- **Rubric:** I2, I3 (A1)
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:4478-4481` (`firePit` `seat`: height 0.38, forward 0.38: on the capstones, facing out), `:4485-4489` (`relax`, `pose: sit`)
- **Evidence:** `play/chunk1/firePit-relax-01.png`, `sheets/age-firePit.png` (child, teen, adult, elder), `extra/night/sheet-fire.png`: the resident sits on the capstone rim of a burning pit, legs out over the grass, the flames a hand's breadth behind their back, for half an hour. Nobody sits on a lit fire pit's wall; people sit on chairs or log benches round it, facing the fire, about a metre off. Drop the `seat` and use the fire from a seat nearby (`seatNear`, as the TV does) or standing in front of it facing in; or give the model a log bench (`model`).
- **Status:** fixed (aac64b84, 278de962)
- **After:** the pit's `seat` is gone; *Warm up by the fire* is used from a seat within 3 m that faces the pit (`seatNear: 3`, `seatFacing: around`: a garden chair round it), else standing at one of four places round it 0.85 m from its middle, facing the fire (`slots: 4`, `places`), four at once. At night with a garden chair facing it and four residents: one in the chair, three standing round it facing in (`bench-results/review/fixes/slots/play/firePit-0*.png`; `bench-results/review/fixes/slots/poses/poses-default-firePit_relax.png`), against `play/chunk1/firePit-relax-01.png`. Sim test `tests/slots.rs` (`round_the_fire_pit_from_a_chair_facing_it_or_standing_facing_in`). Standing, it is the `relax` stand clip, arms folded (I-outdoor-09). Not re-run at every stage (`--sim`): on foot each stage plays its own stand clip, with no seat height left to fit.
- **Source:** outdoor wave (lead; Phase 0 "Seen")

### I-outdoor-02 — Dozing outdoors, and on a sofa bed or window seat, puts the resident in pyjamas

> Consolidation (2026-10-21): closed as a duplicate of I-bedroom-01 (residents ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I3
- **Fix class:** code
- **Where:** `web/src/render/babylon/characters.ts:2373-2377` (pyjamas whenever the pose is `lie` and the tag is `sleep` or `nap`)
- **Evidence:** `sheets/obj-sunLounger.png` (`play/chunk1/sunLounger-doze-01.png`: pink pyjamas on the lounger; sunbathing beside it keeps the day clothes), `sheets/age-sunLounger.png` (the elder too), `sheets/obj-hammock.png` (`play/chunk2/hammock-doze-01.png`). A doze outside or on a sofa is taken in day clothes. Lying naps are: `sunLounger`, `hammock` (this ledger), `sofaBed` (living), `cheerful.rainbowHammock`, `natureLover.gardenHammock`, `gloomy.windowSeat`, `kind.hearthSettee` (packs), and the beds' daytime naps. Pyjamas for `sleep` only, or for `nap` only in a bed.
- **Status:** closed (duplicate of I-bedroom-01)
- **Source:** outdoor wave (lead; Phase 0 "Seen")

### I-outdoor-03 — moved to residents.md

### I-outdoor-04 — Waist-high garden jobs are done kneeling: the greenhouse benches, the compost bin

> Consolidation (2026-10-21): closed as a duplicate of I-garden-04 (residents ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I3 (A1)
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:508` (`ACTIONS.tend`: `kneel` for every object); `web/public/content/catalog2/outdoor.json:418-420` (`greenhouse` `tend`), `:475-477` (`compostBin` `compost`, `anim: tend`)
- **Evidence:** `play/chunk7-greenhouse-wide/greenhouse-tend-01.png`: the seedlings are on staging about 0.8 m high inside, and the resident kneels on the grass for 45 minutes. `play/chunk3/compostBin-compost-01.png`: kneeling at a bin whose rim is 0.72 m, the hands at its slats, below the heap they're turning. The kneel is right at the raised bed, the sandpit and low flowers. `tend` chosen by object (standing, `harvest` or `plant`, at the greenhouse and the compost bin), as `harvest` and `look` already are.
- **Status:** closed (duplicate of I-garden-04)
- **Source:** outdoor wave

### I-outdoor-05 — The greenhouse is tended and picked from outside its door

- **Severity:** S2
- **Rubric:** I2
- **Fix class:** content
- **Where:** `web/public/content/catalog2/outdoor.json:416-455` (`greenhouse` `tend`, `harvest`: no `inside`); `crates/sim-core/src/content.rs:86-88` (`inside`: used standing on the object, as the shower)
- **Evidence:** `sheets/obj-greenhouse.png` (`play/chunk7-greenhouse-wide/greenhouse-tend-01.png`, `greenhouse-harvest-01.png`): the door swings open and the resident works on the grass in front of it, a metre from the nearest plant. A 2 × 3 greenhouse has a path down its middle (`clips/clips-greenhouse-all-t99.png`) that a gardener stands in. `inside: true` puts them on the centre, the path; check the door clip still reads with someone in it.
- **Status:** fixed (0cc15781)
- **After:** `inside: true` on tend and harvest: the gardener steps in onto the path between the benches; the door clip still plays (`bench-results/review/fixes/content/g02/after-greenhouse-tend-01.png`).
- **Source:** outdoor wave (Phase 0 "Seen")

### I-outdoor-06 — Birds are watched from arm's length, and the feeder and bath are never refilled

- **Severity:** S3
- **Rubric:** I2, I1
- **Fix class:** content
- **Where:** `web/public/content/garden.json:2155-2158` (`garden.birdbath` `birdwatch`, `look`, standing at the front tile); `web/public/content/catalog2/outdoor.json:613-616` (`birdFeederPole` `birdwatch`)
- **Evidence:** `sheets/obj-garden.birdbath.png`, `sheets/obj-birdFeederPole.png`: the watcher stands half a metre from the bath and a metre from the feeders for 20 minutes, close enough to put any bird off. Birds are watched from a seat or window a few metres away (`seatNear`, as I-living-03 proposes for reading). Filling the feeder with seed and topping up the bath (reference `birdFeeder` refill 2–10) are the real upkeep uses and are missing; at triage, a short refill use or none.
- **Status:** fixed (9a5cc144)
- **After:** Birdwatch at the bird bath and the feeders takes a seat within 6 m (`seatNear`): watched from the bench (record `pose: sit`). Refilling the feeder and the bath: not added (a mime without props, G-70).
- **Source:** outdoor wave

### I-outdoor-07 — moved to systems.md

### I-outdoor-08 — The outdoor bar has no drink to have at it

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** content
- **Where:** `web/public/content/catalog2/outdoor.json:240-265` (`outdoorBar`, no uses: "pull up bar stools"); `web/public/content/furniture.json:838-860` (`barStool`: `seatAt` bar, only `sit`)
- **Evidence:** `sheets/obj-outdoorBar.png`. A bar's use is a drink on a stool at it. A stool pulled up offers only "sit", while the patio chair at a table has "Have a coffee" (`needsTable`, `catalog2/outdoor.json:93-110`). A drink use on the bar stool with `needsTable` (kitchen ledger: `barStool`) gives the indoor bar counters (`barSegment`, `barEnd`) the same.
- **Status:** fixed (543c6121, eaca9ed8)
- **After:** A high bar stool pulled up to the outdoor bar has "Have a drink at the bar" (`stoolBarHigh`, `seatAt: bar`, played); the bars' texts say high bar stools.
- **Source:** outdoor wave

### I-outdoor-09 — Warming up by the fire on foot is standing with the arms folded

- **Severity:** S3
- **Rubric:** I3 (A1)
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts` (`ACTIONS.relax`: `stand: ['idle', 'foldArms']`), `web/public/content/furniture.json` (`firePit` `relax`, `anim: relax`)
- **Evidence:** `bench-results/review/fixes/slots/play/firePit-0*.png`: since G-01 those without a chair stand round the pit facing it, idling or with the arms folded for 30 minutes. People at a fire hold their hands out to it now and then, or rub them. A standing warm-up clip (the hands out to the fire, palms in, in turns with rubbing them), picked by object as G-26 picks the game clips.
- **Status:** open
- **Source:** fix: slots
