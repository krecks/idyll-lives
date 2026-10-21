# Community and farm: authenticity ledger

What it covers: the 36 objects of [docs/objects/community.md](../objects/community.md), in its groups and order.

- **Evidence:** `bench-results/review/community/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Community wave** (2026-10-21): all 36 rows. Shots are paths under `bench-results/review/` in the review worktree: the Phase 0 evidence in `community/` (its `index.md` says how each was taken) and this wave's extra shots in `community/extra/` (café chair with a table, the night stage, `poses.html` with the real footprints; how they were taken is in `community/review-log.md`). Durations are judged by §9 decision 2: the `cargo review uses` flags (the drinking fountain, the timetable, the notice board, the post box against the `drink`, `read` and `write` medians) are shorter than a book or a meal because the real things are, and are recorded as by design.

## CM1. Playground (8)

Family: `tools/art/blender/families/playground.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `slide` Playground slide | OK | I-community-01 | I-community-01, I-community-04 | I-community-01 S1 clip; I-community-04 S1 code | `community/sheets/obj-slide.png`, `community/sheets/age-slide.png` |
| `climbingFrame` Climbing frame | OK | A-community-01, A-residents-10 | I-community-04 | A-community-01 S3 clip (seeded; confirmed by the code); A-residents-10 S3 clip (residents ledger); I-community-04 S1 code | `community/sheets/obj-climbingFrame.png`, `community/sheets/age-climbingFrame.png` |
| `seesaw` Seesaw | OK | I-community-13 | I-community-02, I-community-03, I-community-04 | I-community-02 S2 code; I-community-03 S2 code; I-community-04 S1 code; I-community-13 S3 clip | `community/sheets/obj-seesaw.png`, `community/sheets/age-seesaw.png`, `community/extra/poses/rides-child.png` |
| `springRider` Spring rider | M-community-03 | I-community-13 | I-community-04 | M-community-03 S3 model; I-community-04 S1 code; I-community-13 S3 clip | `community/sheets/obj-springRider.png`, `community/sheets/age-springRider.png`, `community/extra/poses/rides-child.png` |
| `merryGoRound` Merry-go-round | M-community-03 | OK | I-community-02, I-community-04 | I-community-02 S2 code; I-community-04 S1 code; M-community-03 S3 model | `community/sheets/obj-merryGoRound.png`, `community/sheets/age-merryGoRound.png`, `community/extra/poses/rides-child.png` |
| `basketballHoop` Basketball hoop | M-community-02 | I-community-05 | I-community-05, I-bathroom-04 | I-community-05 S2 renderer-table; M-community-02 S3 model; I-bathroom-04 S3 code (systems ledger: no sound) | `community/sheets/obj-basketballHoop.png`, `community/sheets/age-basketballHoop.png` |
| `pingPongTable` Ping-pong table | M-community-01 | I-community-06 | I-community-02, I-community-03, I-community-06, I-community-08 | M-community-01 S2 model; I-community-02 S2 code; I-community-03 S2 code; I-community-06 S2 clip; I-community-08 S3 code | `community/sheets/obj-pingPongTable.png`, `community/sheets/age-pingPongTable.png` |
| `parkChessTable` Park chess table | I-community-02 | OK | I-community-02, I-community-03 | I-community-02 S2 code; I-community-03 S2 code | `community/sheets/obj-parkChessTable.png`, `community/sheets/age-parkChessTable.png`, `community/extra/poses/chess-adult.png` |

Notes:
- `slide`: deck 1.40 m, chute run-out 0.28 m, 0.75 × 2.36 × 1.93 m (D 3 % under 2.0, inside ±10 %).
- `climbingFrame`: the sizes table's H 3.45 m is the flag on its pole (`playground.py:238`); the roof's ridge is 2.74 m and the deck 1.1 m, inside the range. A tool artefact, no finding.
- `seesaw`: the sizes table's W 2.50 > 0.66 and D 0.42 < 2.16 are the plank built along the object's width, the reference along its depth: an artefact. The seats are at 0.56 m (40–75 cm), the handles inboard of them (`playground.py:285-290`), the plank rocks ±12° and carries both riders (`ride`).
- `parkChessTable`: top 0.76 m (70–76 cm), stools 0.47 m (42–48 cm), both inside; the board and pieces read from Live. The two stools stand side by side behind the table (I-community-02), against the content's own "a stool either side" (`community.json:629`).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `slide` | climb the ladder, slide down, again; mostly children | go down the slide (15, `play`) | the right use, never acted: standing at the chute's foot (I-community-01); adults take it (I-community-04) |
| `climbingFrame` | climb the net, cross the deck, down the pole, play fort; children | climb the frame (25, `exercise`, a fitted round) | OK; adults take it at full weight (I-community-04) |
| `seesaw` | ride with a partner, facing each other | ride the seesaw (15, sit, two slots) | needs a partner (I-community-03); riders side-on (I-community-02, I-community-13) |
| `springRider` | rock and bounce; toddlers to about 6 | bounce (10, sit) | OK; adults take it (I-community-04) |
| `merryGoRound` | push it round, jump on, ride; a few children round the rim | spin round (15, sit, four slots) | the riders sit in a row, not round it (I-community-02); no pushing |
| `basketballHoop` | shoot hoops, play one-on-one, dribble | shoot some hoops (40, `exercise`, two slots) | the use is right; jogging on the spot under the rim (I-community-05) |
| `pingPongTable` | a rally between two players at the ends, bats in hand | play ping-pong (30, `play`, two slots) | needs a partner (I-community-03); played from the side by the net with the bats left on the table (I-community-02, I-community-06) |
| `parkChessTable` | a game for two across the board; watching one | play chess in the park (45, sit, two slots) | the use is right; the players sit side by side (I-community-02) and one plays alone (I-community-03) |

## CM2. Street furniture (10)

Family: `tools/art/blender/families/street_furniture.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `streetLamp` Street lamp | M-community-06 | n/a | OK | M-community-06 S3 code | `community/sheets/obj-streetLamp.png`, `community/extra/night/night-streetLamp-no-uses.png` |
| `parkBench` Long park bench | M-community-04 | OK | I-community-11 | M-community-04 S3 model; I-community-11 S3 code | `community/sheets/obj-parkBench.png`, `community/sheets/age-parkBench.png` |
| `drinkingFountain` Drinking fountain | OK | OK | I-bathroom-04 | I-bathroom-04 S3 code (systems ledger: no sound) | `community/sheets/obj-drinkingFountain.png` |
| `litterBin` Litter bin | OK | n/a | OK | | `community/sheets/obj-litterBin.png` |
| `busStop` Bus shelter | M-community-04 | I-community-07 | I-community-07 | I-community-07 S3 renderer-table; M-community-04 S3 model | `community/sheets/obj-busStop.png`, `community/sheets/age-busStop.png` |
| `postBox` Post box | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no card) | `community/sheets/obj-postBox.png` |
| `noticeBoard` Notice board | OK | I-community-07 | I-community-07 | I-community-07 S3 renderer-table | `community/sheets/obj-noticeBoard.png` |
| `phoneBooth` Phone box | OK | OK | I-social-01 | I-social-01 S2 code (social ledger: the receiver mimed) | `community/sheets/obj-phoneBooth.png`, `community/play/phoneBooth-call-02.png` |
| `fountainTown` Town fountain | M-community-05 | I-community-11 | I-community-11, I-bathroom-04 | I-community-11 S3 code; M-community-05 S3 content; I-bathroom-04 S3 code (systems ledger: no water sound) | `community/sheets/obj-fountainTown.png`, `community/sheets/age-fountainTown.png` |
| `townStatue` Founder's statue | OK | OK | I-community-08 | I-community-08 S3 code | `community/sheets/obj-townStatue.png` |

Notes:
- `streetLamp`: 4.24 m, the lantern's panes 3.46–3.88 m and the content's `light.height` 3.5 m agree (`street_furniture.py:82`); the lantern glows at night.
- `parkBench`: the seat (0.45 m) is in range and agrees with the content; the heights check's "no model under it (x -1.00)" is a tool artefact: the slats run 2.86 m across at depth 0 (`street_furniture.py:141`) and the play shot has the sitter on them. A child's feet hang (A-residents-09, residents ledger).
- `drinkingFountain`: the bowl about 0.95 m up, the bubbler's arc grows while they drink, and the fitted clip bends them to the spout (`play/drinkingFountain-drink-01.png`). 3 minutes for a drink of water is over the real 0–2 but by design.
- `phoneBooth`: 0.95 × 2.70 m (a K6 is 0.91 × 2.5); in play the caller waits back while the door opens, steps in, the door shuts and the receiver goes to the ear (`play/phoneBooth-call-00…02.png`). The first clip shot from inside its walls is a framing artefact (the camera's radius from the 1 × 1 footprint), redone in `clips/wide/`.
- `fountainTown`: rim 0.50 m (42–55 cm); four sitters spread along the front rim, legs over its outer face (`sheets/age-fountainTown.png`), which reads right; the index's "legs into the rim" isn't seen in the pose shots.
- `townStatue`: the admirer looks up (`admire`, `characters.ts:3371`).

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `streetLamp` | light the pavement at night | none (a `light`, lit every night) | OK: a light, no use needed |
| `parkBench` | sit, rest, watch, feed the birds, chat | sit (30, three slots), feed the pigeons (20, $1) | OK; the pigeons aren't there (I-community-11) |
| `drinkingFountain` | a drink of water | have a drink (3) | OK |
| `litterBin` | throw rubbish away | none | decor, no use needed (nothing in the game makes litter) |
| `busStop` | wait for the bus, sit out of the rain, read the timetable | sit in the shelter (20, three slots), read the timetable (5, not autonomous) | OK (labelled "sit", not "wait for the bus": no bus comes); the timetable is read from the wrong place, as a book (I-community-07) |
| `postBox` | post a letter or a card | post a card to a friend (6, `write`, $1) | OK; no card (I-held-01) |
| `noticeBoard` | read the notices, pin one up | read the notices (10) | OK; read as a book (I-community-07) |
| `phoneBooth` | make a call | call a friend (20, $1, not autonomous) | OK |
| `fountainTown` | sit on the rim, toss a coin, watch the water | toss a coin (5, $1), sit on the rim (25, four slots) | OK; no toss (I-community-11) |
| `townStatue` | look at it, meet beside it | admire (8) | OK |

## CM3. Food stalls and vending (5)

Family: `tools/art/blender/families/stalls.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `iceCreamCart` Ice cream cart | OK | OK | I-community-08, I-community-09, I-held-01 | I-community-09 S2 code; I-community-08 S3 code; I-held-01 S2 code (held ledger: no cone) | `community/sheets/obj-iceCreamCart.png`, `community/sheets/age-iceCreamCart.png`, `community/extra/night/night-iceCreamCart-iceCream-01.png` |
| `hotDogStand` Hot dog stand | OK | OK | I-community-08, I-community-09, I-community-10 | I-community-09 S2 code; I-community-08 S3 code; I-community-10 S3 decision | `community/sheets/obj-hotDogStand.png` |
| `vendingSnack` Snack machine | OK | OK | I-community-10, I-held-01 | I-community-10 S3 decision; I-held-01 S2 code (held ledger) | `community/sheets/obj-vendingSnack.png` |
| `vendingDrink` Drinks machine | OK | OK | I-held-01 | I-held-01 S2 code (held ledger: no can) | `community/sheets/obj-vendingDrink.png`, `community/extra/night/night-vendingDrink-drink-01.png` |
| `marketStall` Market stall | OK | OK | I-community-08, I-community-09, I-community-10 | I-community-09 S2 code; I-community-08 S3 code; I-community-10 S3 decision | `community/sheets/obj-marketStall.png` |

Notes:
- Tops: the ice cream cart 1.01 m and the hot dog stand 0.95 m (90–105 cm), the market stall 0.82 m (80–100 cm), all in range. The machines are 0.90 × 1.88 m (in range); their signs and the drinks machine's front glow at night (`extra/night/night-vendingDrink-drink-01.png`).
- Moving parts: the freezer lid slides, the hot plate's lid lifts (75°, a little under 80–110° but it reads as open), the snack and the can drop into the tray; each played in use (`play/play.jsonl`). Who opens them is I-community-09.
- Durations (10–15 minutes to buy and eat at a cart or machine against a real 1–3 to buy) are by design.

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `iceCreamCart` | buy a cone from the vendor, eat it walking or on a bench | buy an ice cream (10, $4, a feeling) | OK as a use; no vendor (I-community-09), eaten standing at the cart with no cone (I-held-01) |
| `hotDogStand` | buy a hot dog, add sauce, eat it nearby | buy a hot dog (15, $6) | as above (I-community-09, I-community-10) |
| `vendingSnack` | buy a snack, take it out of the tray | buy a snack (10, $4) | OK; eaten at the machine (I-community-10) |
| `vendingDrink` | buy a cold drink, take it out of the tray | buy a cold drink (8, $4) | OK |
| `marketStall` | browse, buy fruit and vegetables to take home, chat to the stallholder | browse (15, two slots), buy a bag of fruit (10, $5, `eat`) | the bag is eaten on the spot (I-community-10); no stallholder (I-community-09) |

## CM4. Shops and café (5)

Family: `tools/art/blender/families/shops.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `shopCounter` Shop counter | OK | OK | I-community-08, I-community-09 | I-community-09 S2 code; I-community-08 S3 code | `community/sheets/obj-shopCounter.png` |
| `bakeryCase` Bakery case | OK | OK | I-community-08, I-community-09, I-community-10 | I-community-09 S2 code; I-community-08 S3 code; I-community-10 S3 decision | `community/sheets/obj-bakeryCase.png` |
| `groceryShelf` Grocery shelves | OK | OK | I-community-08, I-community-10 | I-community-08 S3 code; I-community-10 S3 decision | `community/sheets/obj-groceryShelf.png` |
| `cafeTable` Café table | OK | n/a | OK | | `community/sheets/obj-cafeTable.png` |
| `cafeChair` Café chair | OK | OK | I-community-09, I-held-01 | I-community-09 S2 code; I-held-01 S2 code (held ledger: no cup, cake or plate) | `community/sheets/obj-cafeChair.png`, `community/extra/cafe/cafe-cafeChair-coffee-01.png`, `community/extra/cafe/cafe-cafeChair-cake-01.png` |

Notes:
- `shopCounter`: top 0.97 m (90–110 cm). `bakeryCase`: the sizes table's "top 0.78 < 0.90" is the display deck behind the glass, where a real case keeps its trays; the case is 1.29 m high, a counter's height. Measured against a counter's range (kind `shopCounter`), a tool artefact.
- `cafeTable` 0.75 m and `cafeChair` 0.47 m agree with the content (0.745, 0.47) and are in range (71–76, 42–48 cm).
- Lead (café chair coffee needs a table and a coffee source): on `review-stage` with a café table in front, sit, coffee and cake are all taken (`extra/cafe/`, the stage's kitchen has a coffee machine); on the park plot of `shops-town` coffee is refused for want of one, as the content says. The café lot in `houses.json:1126` has its machine. Working as designed; what it looks like is I-community-09 and I-held-01.

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `shopCounter` | pay at the till, buy sweets or a magazine | buy a little something (6, $6, two slots) | OK; nobody at the till (I-community-09) |
| `bakeryCase` | choose a pastry, the baker hands it over, eat it at a table or take it away | buy a pastry (10, $5) | eaten standing at the case (I-community-10); the baker's door opens by itself (I-community-09) |
| `groceryShelf` | browse, pick groceries, take them to the till | browse (10), grab a sandwich (15, $7) | the sandwich is eaten in the aisle (I-community-10); no groceries to take home |
| `cafeTable` | a coffee or a cake for two | none (its chairs' uses, `seatAt: table`) | OK |
| `cafeChair` | sit, have a coffee and a cake at the table | sit (20), coffee (20, needs a table and a coffee maker on the plot, $4), cake (15, needs a table, $5) | OK; the coffee and cake appear from nowhere (I-community-09, I-held-01) |

## CM5. Farm (8)

Family: `tools/art/blender/families/farm.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `farm.barn` Red barn | OK | n/a | OK | | `community/sheets/obj-farm.barn.png`, `community/play/town-things/town-things-farm-a.png` |
| `farm.silo` Grain silo | OK | n/a | OK | | `community/sheets/obj-farm.silo.png` |
| `farm.tractor` Little tractor | M-community-04 | n/a | OK | M-community-04 S3 model | `community/sheets/obj-farm.tractor.png` |
| `farm.hayBale` Round hay bale | M-community-04 | n/a | OK | M-community-04 S3 model | `community/sheets/obj-farm.hayBale.png` |
| `farm.hayStack` Stack of hay bales | OK | n/a | I-community-12 | I-community-12 S3 decision | `community/sheets/obj-farm.hayStack.png` |
| `farm.scarecrow` Scarecrow | OK | OK | OK | | `community/sheets/obj-farm.scarecrow.png` |
| `farm.trough` Water trough | OK | n/a | OK | | `community/sheets/obj-farm.trough.png` |
| `farm.coop` Chicken coop | OK | OK | I-community-12 | I-community-12 S3 decision | `community/sheets/obj-farm.coop.png` |

Notes (lead: the farm at real scale next to residents, `sheets/obj-farm.*.png`, `play/town-things/town-things-farm-a.png`):
- `farm.barn` 6.0 × 6.0 × 5.9 m: the small end of a barn (6–15 m); its double doors are about 2.7 m high, half again an adult, and it reads as a barn beside them. `farm.silo` 2.8 × 6.9 m (6–20 m), taller than the barn's ridge, as silos are. `farm.trough` 1.84 × 0.66 m, `farm.coop` 1.74 × 1.88 × 1.97 m, `farm.scarecrow` 2.13 m (it sways, A11) and `farm.hayStack` (small square bales about 0.9 × 0.45 × 0.45 m) are all in range and read at their size beside residents.
- `farm.tractor`: its rear wheels are 1.45 m across (`farm.py:399`) and the canopy 2.5 m up: the size of a real utility tractor, at scale beside residents, but wider than the reference's compact tractor and its name ("Little") (M-community-04).
- `farm.hayBale`: 1.0 m across, waist-high on an adult, where a round bale (1.2–1.8 m) stands chest to head high (M-community-04).
- `farm.coop`: its pop door shuts at night (`night close`), a nice touch with no hens to shut in.

### Real uses against the content's (I1)

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `farm.barn` | store hay and machinery, house animals | none | decor, no use needed |
| `farm.silo` | store grain | none | decor, no use needed |
| `farm.tractor` | drive it round the farm | none | decor, no use needed (driving is new behaviour) |
| `farm.hayBale` | feed livestock; sit on it | none | decor, no use needed |
| `farm.hayStack` | sit on it, climb it (children), feed animals | none | a seat its own description promises ("a step, a seat, a fort"): missing use (I-community-12) |
| `farm.scarecrow` | keep birds off the crops | none | decor, no use needed |
| `farm.trough` | water the animals | none | decor, no use needed (no animals) |
| `farm.coop` | collect eggs, feed the hens | none | missing use, if hens come (I-community-12) |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### A-community-01 — Climbers who start at other offsets can still meet at the pole

> Community wave: confirmed by reading the code; not seen in play (the tools drive one resident at a time).

- **Severity:** S3
- **Rubric:** A7
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters/clips/street.ts:127` (`CLIMB`, lanes), `:156-157` (`ROUND`: when each lane's round reaches the deck, waits, crosses and swings, as shares of the climber's own 13 s loop), `web/src/render/babylon/characters.ts:2838-2851` (`freeLane`, `stepSeq`: a lane is chosen on stepping in, the loop runs on the climber's own clock)
- **Evidence:** Two climbers on the frame have a lane each, but if they start at other offsets they can still meet at the fireman's pole. The second lane's `wait` (0.34 → 0.50 of the round) holds it back only if both rounds started together: nothing ties the second climber's clock to the first's, so one who steps in half a round later reaches the pole while the first is still on it. `check:clips` doesn't test two climbers, and `check.mjs play` uses one resident.
- **Status:** fixed (3872d23a)
- **After:** a climber stepping in while someone is going round the same frame waits at their lane, standing, until the other's round comes back to its start (`inTurn`, `LANE_SYNC` 0.4 s), so the two lanes always go round in step and the second waits on the deck while the first slides. Two climbers (a child and a teen, the second sent 6 s later) waited 7 s and then ran 0.016 s apart for the rest of the use. `bench-results/review/fixes/clips-standing/``climb/` (`climb2-gap6.json`, the pictures at the pole).
- **Source:** `docs/objects/open.md` Still open › Residents; community wave (code reading)

### M-community-01 — The ping-pong table is about two thirds of a real one

- **Severity:** S2
- **Rubric:** M1, M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/playground.py:438-441` (`pingPongTable`: `top = 0.78`, the top 1.9 × 0.94 m); `web/public/content/community.json:576` (footprint 2 × 1)
- **Evidence:** `tables/sizes-community.md`: 1.90 × 0.95 × 0.99 m against a table tennis table's 2.74 × 0.76 × 1.525 m (ITTF; `reference.json` `pingPongTable` 1.5–1.6 × 2.7–2.8 m): 69 % of the length, 62 % of the width, well outside ±10 %, and the top 2 cm over the fixed 0.76 m. `sheets/obj-pingPongTable.png`: beside the adult it is a dining table with a net. A full table needs a 3 × 2 footprint (with its ends clear for the players, I-community-02).
- **Status:** open
- **Source:** community wave (sizes table, gallery)

### M-community-02 — The basketball rim is 11 cm low and the backboard small

- **Severity:** S3
- **Rubric:** M1, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/playground.py:381` (`RIM = (2.94, 0.27)`), `:412` (`board` 0.95 × 0.62 m)
- **Evidence:** A rim is 3.05 m up everywhere (FIBA, and park and driveway hoops); this one is at 2.94 m. The backboard is 0.95 × 0.62 m, about half a regulation board (1.80 × 1.05) and small even for a park or portable one (1.1–1.4 m wide). `sheets/obj-basketballHoop.png`: it reads as a hoop, but a short, small one beside the adult. The overall height (3.50 m) is inside the reference.
- **Status:** open
- **Source:** community wave (family script, gallery)

### M-community-03 — Two playground seats off their range or the content: the spring rider's saddle 8 cm high, the merry-go-round's deck 2 cm under its seat

- **Severity:** S3
- **Rubric:** M2
- **Fix class:** model
- **Where:** `tools/art/blender/families/playground.py:336` (`saddle` at 0.755 + 0.025 = 0.78 m) and `web/public/content/community.json:465` (`springRider` `seat.height` 0.78); `playground.py:358-362` (`merryGoRound`: the platform's top 0.24 m, its stripes 0.243 m) against `community.json:500` (`seat.height` 0.26)
- **Evidence:** `tables/sizes-community.md`: the spring rider's seat 0.78 > 0.70 (EN 1176-6 riders, 40–70 cm), on a ride for toddlers to six-year-olds; the content agrees with the model, so both move. `sheets/age-springRider.png`, `extra/poses/rides-child.png`: the child sits high with the feet off the footrests' level. The merry-go-round's deck (0.24 m) is in range by the 1 cm tolerance, but the content seats its riders at 0.26 m, 2 cm above it (the three-way check); in the shots the thighs hide it.
- **Status:** open
- **Source:** community wave (sizes table, poses)

### M-community-04 — Five overall sizes just past the ±10 % band: the hay bale, the tractor, the park bench, the bus shelter

- **Severity:** S3
- **Rubric:** M1
- **Fix class:** model
- **Where:** `tools/art/blender/families/farm.py:485` (`BALE_R, BALE_L = 0.495, 0.86`), `:399` and `:732` (`tractor` 1.98 × 2.50 × 3.28 m); `tools/art/blender/families/street_furniture.py:141` (`parkBench` slats 2.86 m), `:217-219` (`busStop` 0.98 m deep)
- **Evidence:** `tables/sizes-community.md`:
  - `farm.hayBale` 0.88 × 1.0 × 1.0 m against 1.2–1.8 m: a round bale comes up to an adult's chest or head; this one is waist-high (`sheets/obj-farm.hayBale.png`), the most visible of the five.
  - `farm.tractor` W 1.98 > 1.76 m (a compact tractor, 1.0–1.6 m wide): the model is a utility tractor, right in its own scale beside residents (`sheets/obj-farm.tractor.png`), called "Little tractor". A decision as much as a model change: a narrower compact, or a utility tractor in the name and the reference.
  - `parkBench` W 2.86 > 2.75 m (1.8–2.5 m): three seats a metre apart on its three tiles.
  - `busStop` D 0.98 < 1.08 m (1.2–1.8 m): the bench fills the shelter's depth.
  The climbing frame's H (3.45 m, the flag) and the seesaw's W and D (the plank along the width) are tool artefacts, not in this finding.
- **Status:** open
- **Source:** community wave (sizes table, gallery)

### M-community-05 — The town fountain is called round and built square

- **Severity:** S3
- **Rubric:** M5
- **Fix class:** content
- **Where:** `web/public/content/community.json:297` (`fountainTown` description: "A round stone basin"); `tools/art/blender/families/street_furniture.py:380` ("a square stone basin")
- **Evidence:** `sheets/obj-fountainTown.png`: a square basin with a round two-tier fountain in it. The buy catalogue's text says round. Either word is fine; they should agree (the square rim is what seats four along its front).
- **Status:** fixed (b3f6b82d)
- **After:** "A square stone basin with a round fountain in the middle".
- **Source:** community wave (content against the model)

### M-community-06 — At night the street lamp's lantern glows but no light reaches the ground

- **Severity:** S3
- **Rubric:** M10
- **Fix class:** code
- **Where:** `web/public/content/community.json:27` (`streetLamp` `light`: range 8, intensity 1.3, height 3.5); `web/src/render/babylon/house.ts:198-235` (`placeLights`: the garden's lamps are placed last); `web/src/render/babylon/BabylonRenderer.ts:4704-4724` (`applyRoomLights`)
- **Evidence:** `extra/night/night-streetLamp-no-uses.png` (`review-stage-night`, 22:02, Medium, headless): the lantern's panes glow warm, the grass round the post is as dark as the rest of the garden, no pool of light under a 3.5 m lamp whose whole job is lighting the pavement. The cause isn't traced: Medium has clustered lamps (32), so the stage's rooms shouldn't crowd it out; a lamp's light from 3.5 m with the clustered inverse-square falloff may just be too weak to show. Not checked on a town lot at night (no night state of the park).
- **Status:** fixed (0eaa3ba7)
- **After:** the cause: the lamps never light the landscape, the lawn included (`excludeLandscape`), so a garden lamp lit only what stands round it. Letting the cluster light the ground cost 1.1–1.5 ms of GPU a frame at night on Medium (`fixes/lighting/perf/perf-ab`), so the garden's lamps now cast pools of light on the lawn as the street's lamps do (`gardenPools.ts`: one draw at night, on Low too), and the street lamp is brighter (`intensity` 1.3 → 3.0; the bulkhead 0.9 → 1.6): `fixes/lighting/webgpu-medium/street.png`, `firepit-lit.png`, and the same in the other three folders. Not checked on a town lot (no night state of the park).
- **Source:** community wave (night run)

### I-community-01 — "Go down the slide" is standing at the foot of the chute

- **Severity:** S1
- **Rubric:** I2, I3, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:477` (`ACTIONS.play`: `stand: ['spell']`, the hands-at-waist mime); `:193-199` (`STEP_IN`: the drinking fountain, the phone box and the climbing frame have a fitted round, the slide has none); `web/public/content/community.json:361` (`slide`: `anim: play`); the ladder at the back (`playground.py:185-191`)
- **Evidence:** `sheets/obj-slide.png` (`play/slide-slide-01.png`), `sheets/age-slide.png` (`play/child/slide-slide-01.png`): the adult and the child stand on the tile in front of the run-out for 15 minutes, hands moving at the waist by the chute's end. Nobody climbs the ladder behind the tower or comes down the chute: the use named on the label never happens, and it is done at the wrong end of the object. The lead (the adult and the child stand at the foot) is confirmed. The climbing frame shows what the fix is: a `STEP_IN` round fitted per height (round the tower to the ladder, up, sit on the deck, down the chute on the seat, off at the run-out).
- **Status:** open
- **Source:** community wave (Phase 0 play shots, "Seen (not reviewed)")

### I-community-02 — Seats and places only go side by side, facing the front: riders, players and diners can't face each other

> Consolidation (2026-10-21): closed as a duplicate of I-outdoor-07 (systems ledger), which takes its items and evidence.

- **Severity:** S2
- **Rubric:** I2, M8
- **Fix class:** code
- **Where:** `crates/sim-core/src/world.rs:322-333` (`slot_position`: slots spread across the width at one depth); `web/src/render/babylon/characters.ts:240-249` (`Seat`: one facing per object, `back` or `along`, not per slot); `tools/art/blender/families/playground.py:513` (the chess stools both at z = -0.27, side by side)
- **Evidence:** `extra/poses/rides-child.png` (`poses.html` with the real footprints and slots, at the riding clips): the seesaw's two riders sit on the end seats side-on, both facing the same way off the plank's side, not astride facing each other over the handles; the merry-go-round's four sit in a row along one chord, all facing the same way, not round the rim. `extra/poses/chess-adult.png`: the two chess players sit shoulder to shoulder behind the board, the man's arm across it; the content says "a stool either side" (`community.json:629`). `sheets/obj-pingPongTable.png`: the player stands at the long side by the net (and a second would stand beside them), where players stand at the two ends. One cause in the sim and the renderer: a slot is a place across the front, facing it. Other items it reaches: the base dining table, both diners on one side (M-kitchen-01, kitchen ledger, filed there as a model finding); by the content (2 slots, one `SEATS` entry each, not checked in shots) `diningOak`, `diningGlass`, `picnicTable` (`SEATS.picnicTable` fwd -0.68: both on one bench) and `romantic.candlelitTable`. A per-slot place and facing (or slots round a circle) is new behaviour; the chess stools then move to opposite sides.
- **Status:** closed (duplicate of I-outdoor-07)
- **Source:** community wave (poses with real footprints, play)

### I-community-03 — Two-player things played alone: the seesaw tips with one rider, ping-pong and park chess have no partner

- **Severity:** S2
- **Rubric:** I16, I5
- **Fix class:** code
- **Where:** `web/public/content/community.json:428` (`seesaw`, social 0.1), `:576` (`pingPongTable`, social 0.12), `:612` (`parkChessTable`, social 0.08); `tools/art/blender/families/playground.py:556` (`SEESAW`: the plank's `seesaw` loop plays for any use)
- **Evidence:** `sheets/obj-seesaw.png` (`clips/clips-seesaw-all-t99.png`, `play/seesaw-seesaw-01.png`): a lone rider goes up and down on a seesaw, which a seesaw can't do (the empty end drops and stays). `sheets/obj-pingPongTable.png`: a rally with nobody across the net. `sheets/obj-parkChessTable.png`: "Play chess in the park" alone, a pawn moving now and then. Each needs someone to play with (I16), and each gives `social` to a resident who is alone (I5). The content's own description says it ("Much better with two", `community.json:444`). Fix: a partner condition (both slots taken, or a second resident joining), as the pilot's I16 line asks; the home chess table's "Practise chess" alone is fine (I-hobbies-01).
- **Status:** open
- **Source:** community wave (play)

### I-community-04 — moved to systems.md

### I-community-05 — "Shoot some hoops" is jogging on the spot under the rim

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:481-493` (`ACTIONS.exercise`: `jog` for every def but bags, the fitness game, boulders and `streetball`; `basketballHoop` falls through to `jog`); `tools/art/blender/families/playground.py:423` (the ball a still part on the ground)
- **Evidence:** `sheets/obj-basketballHoop.png` (`play/basketballHoop-shoot-01.png`), `sheets/age-basketballHoop.png` (`play/child/basketballHoop-shoot-01.png`): the adult and the child jog on the spot at the ballast base, right under the backboard, for 40 minutes, the ball lying on the grass beside them; the net swishes every 3.2 s with no shot. The pack's `energetic.streetballHoop` already gets `pickUp` and `interact` (picking the ball up, a throw). Giving `basketballHoop` the same is a table change; a shot clip from 2–4 m out, with the ball held (I-held-01), is the real fix.
- **Status:** fixed (3cd0bf87, 0f613d59)
- **After:** `basketballHoop` takes the streetball hoop's `pickUp`, `interact` (`ACTIONS.exercise`): the ball picked up and reached with at the hoop, no jogging (`fixes/tables/play-a/a-basketballHoop-shoot-01.png`). A shot from 2–4 m out with the ball held is still the clip and props waves' (I-held-01). Then (fix: clips-reach) the shot: `STEP_IN.basketballHoop` steps the shooter back out on the court, 2.5 m from the hoop's middle (2.2 m short of the rim; a lane each for two; the walk back played in reverse, facing the hoop), and plays `hoopShot177`/`147`/`112` (clips/shot.ts): dribbles, the gather, the dip, a jump shot let go at the top, the follow-through held while the net swishes; a child heaves it two-handed from the chest (`fixes/clips-reach/play-hoop/hoop0-…`, `hoop2-` teen, `hoop3-` child, `hoop4-` elder; `compare/I-community-05.png`). The ball stays mimed: a ball in hand is I-held-01's.
- **Source:** community wave (play)

### I-community-06 — Ping-pong is the hands-at-the-waist mime, the bats left on the table

- **Severity:** S2
- **Rubric:** I3, I7, A1
- **Fix class:** clip
- **Where:** `web/src/render/babylon/characters.ts:477` (`ACTIONS.play`: `spell`); `tools/art/blender/families/playground.py:455-459` (the two bats are still parts of the table)
- **Evidence:** `sheets/obj-pingPongTable.png` (`play/pingPongTable-pingPong-01.png`), `sheets/age-pingPongTable.png`: the player stands at the table's side by the net and works the hands at waist height for 30 minutes; both bats lie on the table and the ball sits still. Table tennis is a quick forehand and backhand with a bat in the hand. A swing clip and a bat prop (as the weights bench lifts its bar, `held`) are needed; where the players stand is I-community-02.
- **Status:** fixed (4366ec55, 83d53185)
- **After:** a rally of forehands (the chest turning back, through and up across) and backhands, in the ready stance, knees bent (`pingPong`, `ACTIONS.play` by object; closed hand on the bat's line): `bench-results/review/fixes/clips-games/after-pingPongTable-pingPong-0*.png`. The bats in hand are G-70, the players at the ends G-62, a partner G-65.
- **Source:** community wave (play)

### I-community-07 — The notice board and the bus timetable are read like a book in the hands

- **Severity:** S3
- **Rubric:** I3, I2, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:475` (`ACTIONS.read`: `lantern`, the book held at the chest, head down); `tools/art/blender/families/street_furniture.py:235` (the timetable on the back glass at x = -0.95 m, 1.45 m up)
- **Evidence:** `sheets/obj-noticeBoard.png` (`play/shops-town/noticeBoard-read-01.png`): the reader stands before the board looking down at an open book that isn't there, not up at the notices. `sheets/obj-busStop.png` (`play/busStop-timetable-01.png`): the same pose on the tile in front of the shelter's middle, a metre to the side of the timetable and the bench between them. Reading a board is looking at it: `look` (or `read` chosen by object, as `look` is for the telescope) for boards; the timetable reader needs `approach` or a step in to the timetable's end.
- **Status:** fixed (0c5e889e)
- **After:** `ACTIONS.read` by def: the notice board and the timetable are read looking at them (`idle`), not at a book; `STEP_IN.busStop` steps the reader into the shelter to the timetable's end (lane -0.95).
- **Source:** community wave (play)

### I-community-08 — moved to systems.md

### I-community-09 — Nobody serves: carts, the stall, the counters and the café sell themselves

- **Severity:** S2
- **Rubric:** I1, I7, A10
- **Fix class:** code
- **Where:** `tools/art/blender/families/shops.py:306-312` (the bakery case's back door, on the baker's side, slides for the customer's `pastry`), `:230` (the till's drawer), `tools/art/blender/families/stalls.py:152-158` and `:200-206` (the freezer lid and the hot plate's lid, opened for the customer's use); `crates/sim-core/src/world.rs:3689-3712` (`requires`: a coffee maker anywhere on the plot lets a café chair serve coffee; nobody makes or brings it)
- **Evidence:** `sheets/obj-iceCreamCart.png`, `sheets/obj-hotDogStand.png`, `sheets/obj-marketStall.png`, `sheets/obj-shopCounter.png`, `sheets/obj-bakeryCase.png` (the shops-town and stalls-town runs): each sale is a lone customer at an empty cart, stall or counter; the vendor's parts open by themselves, the bakery case's from the far side. `extra/cafe/cafe-cafeChair-coffee-01.png`: a coffee is had at a bare table, the machine never visited. Buying from someone is what these things are for, and a careful player sees no one there. A vendor (a resident at work, or a stand-in figure) is new behaviour (§9 decision 3); a cheaper step is the coffee fetched from the plot's machine before sitting, as cooking walks to the fridge.
- **Status:** open
- **Source:** community wave (Phase 0 "Seen (not reviewed)", café run)

### I-community-10 — Food bought is eaten where it was bought, standing: in the shop aisle, at the bakery case, at the stall

- **Severity:** S3
- **Rubric:** I1, I3
- **Fix class:** decision
- **Where:** `web/public/content/community.json:888` (`groceryShelf` `sandwich`: `eat`, 15 min), `:857` (`bakeryCase` `pastry`, 10), `:780` (`marketStall` `fruit`: "Buy a bag of fruit", `eat`, 10), `:719` (`vendingSnack`), `:687` (`hotDogStand`)
- **Evidence:** `sheets/obj-groceryShelf.png` (`play/shops-town/groceryShelf-sandwich-01.png`): a sandwich eaten for a quarter of an hour in front of the shelf, before the till. `sheets/obj-marketStall.png` (`play/stalls-town/marketStall-fruit-01.png`): a bag of fruit eaten at the stall. `sheets/obj-bakeryCase.png`: a pastry at the case. Street food is eaten standing, but near the cart, a bench or a table, and groceries go home. Options: relabel (a piece of fruit), take the food to a seat (the café chair's cake already sits), or carry it home (new behaviour).
- **Status:** open
- **Source:** community wave (play)

### I-community-11 — The coin toss and the pigeons are not there: wishing is standing, feeding the birds is sitting with the hands in the lap

- **Severity:** S3
- **Rubric:** I3, I7
- **Fix class:** code
- **Where:** `web/public/content/community.json:280` (`fountainTown` `wish`: `anim: look`), `:37` (`parkBench` `feedBirds`: `anim: sit`); `web/src/render/babylon/characters.ts:523` (`ACTIONS.look`: `idle`, `foldArms`)
- **Evidence:** `sheets/obj-fountainTown.png` (`play/shops-town/fountainTown-wish-01.png`): the wisher stands at the rim, arms folded, for 5 minutes; no throw. `sheets/obj-parkBench.png` (`play/shops-town/parkBench-feedBirds-01.png`): "Feed the pigeons" is the bench's sit, hands in the lap, no birds on the ground. A toss (an `interact` throw toward the basin) is a table change; the coin and the crumbs are I-held-01, the birds new behaviour.
- **Status:** open
- **Source:** community wave (play)

### I-community-12 — The hay stack's promised seat and the coop's eggs: two farm things with a clear use and none

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `web/public/content/community.json:1090` (`farm.hayStack`: no interactions; its description, `:1102`, "a step, a seat, a fort"), `:1139` (`farm.coop`: no interactions)
- **Evidence:** `sheets/obj-farm.hayStack.png`: three bales at 0.46 and 0.92 m, as good a seat as the fountain's rim; the buy text offers a seat and a fort that the object doesn't have. `sheets/obj-farm.coop.png`: a coop with a ramp and a pop door that shuts at night, no hens and no eggs (`reference.json` `chickenCoop`: collect eggs, feed the hens). The rest of the farm (the barn, the silo, the tractor, the round bale, the scarecrow, the trough) is decor, as §9 decision 4 has it. Decide whether the hay stack gets a sit (content) and whether hens and eggs come with the coop (new behaviour).
- **Status:** fixed (198e90d9): a sit on the hay stack; the coop decor (Q-19)
- **After:** "Sit on the hay bales", two at once on the front edges (seat 0.45 m, forward 0.25: `bench-results/review/fixes/content/g12/fwd25-farm.hayStack-sit-00.png`); `chickenCoop` is decor (hens and eggs would be new behaviour).
- **Source:** community wave (I1)

### I-community-13 — moved to residents.md

### I-community-14 — The park chess players' knees meet under the table

- **Severity:** S3
- **Rubric:** I2, M6
- **Fix class:** model
- **Where:** `tools/art/blender/families/playground.py` (`parkChessTable`: a 1 × 1 tile, the stools at x = ±0.42 m), `web/public/content/community.json` (`parkChessTable` `places`)
- **Evidence:** `bench-results/review/fixes/slots/play/parkChessTable-side-0*.png`, `parkChessTable-0*.png`: since G-62 the two face each other across the board, but on one tile their hips are 0.84 m apart: the knees meet under the 0.48 m top and the hands at the board's middle, the heads about 0.4 m apart. Real park chess tables seat the players about 1.1 m apart (a 0.7 m top). A 2 × 1 footprint (the stools at ±0.6, moving the park lots' tables), or a narrower seated clip for board games.
- **Status:** open
- **Source:** fix: slots

### Pointers to findings in other ledgers

What this wave saw of causes filed elsewhere, carried into those findings at the consolidation (2026-10-21):

- **I-held-01** (held ledger, no props): the card at the post box, the cone at the ice cream cart, the snack and the can at the machines, the coffee cup, the cake and its plate at the café table (`extra/cafe/cafe-cafeChair-cake-01.png`: hands in the lap, nothing on the table), the coin at the fountain, the basketball held.
- **I-social-01** (social ledger, the phone mimed): the phone box's receiver stays on its hook while the caller holds a phone that isn't there to the ear.
- **I-kids-03** (systems ledger, a direct use skips free will's rules) and **I-systems-04** (systems ledger, no age rule): the playground (I-community-04).
- **I-bathroom-04** (systems ledger, noisy uses with no sound): no running water at either fountain, no ball on the court or the table, no clunk from the machines, no ring or dial tone in the phone box (`web/src/audio/world.ts:34`, `ACTION_SOUNDS`: none of these uses).
- **A-residents-10** (residents ledger): `check:clips` still prints the child's 1.3–1.9 cm and the elder woman's 1.1 cm foot slide in the climbing frame's rounds (`climb112a/b`, `climb172a`).
- **A-residents-09** (residents ledger): a child's feet off the floor on the park bench, the shelter and the café chair.
- **I-hobbies-01** (hobbies ledger) names the park chess table as the one with "a stool either side": its stools are side by side (I-community-02).
