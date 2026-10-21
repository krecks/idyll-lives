# Tabletop: authenticity ledger

What it covers: the 39 objects of [docs/objects/tabletop.md](../objects/tabletop.md), in its groups and order.

- **Evidence:** `bench-results/review/tabletop/` (gallery, clip, pose and play shots; shots stay out of git). Shots cells are paths under `bench-results/review/` in the review worktree. The models were also read from their previews (`tools/art/blender/previews/tabletop/<id>.png`, four contact sheets in the review log's folder).
- **Reviewed** 2026-10-21 in the activities wave (decor, tabletop and lighting by one reviewer); its log is `bench-results/review/decor/review-log.md`.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **How these were judged.** No tabletop thing has a use (content rule, `crates/sim-core/src/content.rs:2738`: things on a surface are 1 × 1 "with one slot and no interactions"), so A is `n/a` but for the clock's hands, and I is about coverage (I1) alone. M5 was judged on the play shots on the stage's counter (`tabletop/play/`), which are about Build-mode zoom: at the default Live camera (radius 24) a 9 cm mug is a few pixels, as the real thing would be. The gallery stands them on the ground beside the residents, which is why the index found the espresso machine and the mixer "knee high"; on the counter they are right.
- **Sizes** (`cargo review sizes`): five outside ±10 %. Artefacts: `standMixer` D 0.17 (it stands side-on, profile to the front, so W and D swap; across, 0.17 against a real 0.20–0.24 is 1 cm under the band), `cuttingBoard` H 0.106 (the loaf and tomatoes on it), `candlesticks` D 0.066 (the bases, 6 mm), `monitor` D 0.335 (5 mm). Real: `tableClock` H 0.228 (M-tabletop-02).

## TT1. Kitchenware (11)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `kettle` Electric kettle | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code (seeded; confirmed and widened) | `tabletop/sheets/obj-kettle.png` |
| `toaster` Toaster | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-toaster.png` |
| `blender` Blender | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-blender.png` |
| `standMixer` Stand mixer | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-standMixer.png` |
| `espressoMachineSmall` Espresso machine | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-espressoMachineSmall.png` |
| `knifeBlock` Knife block | OK | n/a | OK | | `tabletop/sheets/obj-knifeBlock.png` |
| `utensilCrock` Utensil pot | OK | n/a | OK | | `tabletop/sheets/obj-utensilCrock.png` |
| `cuttingBoard` Chopping board | OK | n/a | OK | | `tabletop/sheets/obj-cuttingBoard.png` |
| `breadBin` Bread bin | OK | n/a | I-tabletop-01 | I-tabletop-02 S3 code (seeded; closed: merged into I-tabletop-01); I-tabletop-01 S2 code | `tabletop/sheets/obj-breadBin.png` |
| `spiceJars` Spice jars | OK | n/a | OK | | `tabletop/sheets/obj-spiceJars.png` |
| `cookieJar` Cookie jar | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-cookieJar.png` |

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `kettle` | boil water for tea or coffee | none | missing (I-tabletop-01) |
| `toaster` | make toast | none | missing (I-tabletop-01) |
| `blender` | blend a smoothie or soup | none | missing (I-tabletop-01) |
| `standMixer` | mix dough or batter (baking) | none | missing (I-tabletop-01) |
| `espressoMachineSmall` | pull a shot, steam milk | none, though the stage's floor coffee machine has "make a coffee" | missing (I-tabletop-01) |
| `knifeBlock` | take a knife | none | OK (decor; cooking's chopping is the counter's) |
| `utensilCrock` | take a spoon or spatula | none | OK (decor) |
| `cuttingBoard` | chop vegetables | none | OK: not plainly missing, chopping is the counter station's step in cooking (I13) |
| `breadBin` | take out bread for a sandwich or toast | none | missing (I-tabletop-01) |
| `spiceJars` | season food | none | OK (decor) |
| `cookieJar` | sneak a biscuit | none | missing (I-tabletop-01) |

## TT2. Tableware (7)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `fruitBowl` Fruit bowl | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-fruitBowl.png` |
| `plateSet` Place setting | OK | n/a | OK | | `tabletop/sheets/obj-plateSet.png` |
| `cakeStand` Cake on a stand | OK | n/a | OK | | `tabletop/sheets/obj-cakeStand.png` |
| `teapotSet` Tea set | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-teapotSet.png` |
| `mugs` Mugs | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-mugs.png` |
| `wineGlasses` Wine and glasses | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-wineGlasses.png` |
| `candlesticks` Candlesticks | M-tabletop-01 | n/a | OK | M-tabletop-01 S3 model | `tabletop/sheets/obj-candlesticks.png`, `tabletop/extra/candlesticks-night.png` |

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `fruitBowl` | take a piece of fruit | none | missing (I-tabletop-01) |
| `plateSet` | lay the table | none | OK (decor) |
| `cakeStand` | cut a slice | none | OK (decor, as the reference has it) |
| `teapotSet` | pour a cup of tea | none | missing (I-tabletop-01) |
| `mugs` | have a hot drink | none | missing (I-tabletop-01) |
| `wineGlasses` | pour a glass | none | missing (I-tabletop-01) |
| `candlesticks` | light the candles | none; drawn lit, day and night | OK (decor); the lit look is M-tabletop-01 |

## TT3. Books and small plants (7)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bookStack` Stack of books | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-bookStack.png` |
| `booksUpright` Books and bookends | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-booksUpright.png` |
| `vaseFlowers` Vase of flowers | OK | n/a | OK | | `tabletop/sheets/obj-vaseFlowers.png` |
| `vaseSmall` Bud vase | OK | n/a | OK | | `tabletop/sheets/obj-vaseSmall.png` |
| `potSucculent` Succulent | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-potSucculent.png` |
| `potCactus` Cactus | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-potCactus.png` |
| `potSmall` Small potted plant | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-potSmall.png` |

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `bookStack` | pick one up and read it | none | missing (I-tabletop-01; the bookshelf has read) |
| `booksUpright` | pick one up and read it | none | missing (I-tabletop-01) |
| `vaseFlowers` | change the water | none | OK (decor) |
| `vaseSmall` | change the water | none | OK (decor) |
| `potSucculent` | water it now and then | none | missing (I-tabletop-01; the floor `plant` and `hangingPlant` have water) |
| `potCactus` | water it now and then | none | missing (I-tabletop-01) |
| `potSmall` | water it | none | missing (I-tabletop-01) |

## TT4. Small ornaments (6)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `photoFrame` Photo frame | OK | n/a | OK | | `tabletop/sheets/obj-photoFrame.png` |
| `tableClock` Alarm clock | M-tabletop-02 | OK | OK | M-tabletop-02 S3 model | `tabletop/sheets/obj-tableClock.png`, `tabletop/play/c2/tableClock-no-uses.png` |
| `globe` Globe | OK | n/a | OK | | `tabletop/sheets/obj-globe.png` |
| `snowGlobe` Snow globe | OK | n/a | OK | | `tabletop/sheets/obj-snowGlobe.png` |
| `trophy` Trophy | OK | n/a | OK | | `tabletop/sheets/obj-trophy.png` |
| `sculptureSmall` Small sculpture | OK | n/a | OK | | `tabletop/sheets/obj-sculptureSmall.png` |

Notes: the snow globe's preview is an opaque grey ball (the preview's glass), but in the game its glass shows the tree inside (`tabletop/gallery/gallery-snowGlobe.png`). The alarm clock's hands tell the game's time (`motions.clock`).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `photoFrame` | look at a photo | none | OK (decor) |
| `tableClock` | set the alarm; switch it off in the morning | none; the hands tell the time | OK: not plainly missing, the sim has no alarms or wake times to set |
| `globe` | spin it and find a country | none | OK: not plainly missing (a toy-like gesture, decor-level) |
| `snowGlobe` | shake it | none | OK (decor) |
| `trophy` | show off a win | none | OK (decor) |
| `sculptureSmall` | look at it | none | OK (decor) |

## TT5. Bath, desk and toys (8)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `soapSet` Soap set | OK | n/a | OK | | `tabletop/sheets/obj-soapSet.png` |
| `towelStack` Folded towels | OK | n/a | OK | | `tabletop/sheets/obj-towelStack.png` |
| `perfumeTray` Perfume tray | OK | n/a | OK | | `tabletop/sheets/obj-perfumeTray.png` |
| `deskOrganiser` Desk organiser | OK | n/a | OK | | `tabletop/sheets/obj-deskOrganiser.png` |
| `laptopClosed` Laptop | OK | n/a | I-tabletop-01 | I-tabletop-03 S3 code (seeded; closed: merged into I-tabletop-01); I-tabletop-01 S2 code | `tabletop/sheets/obj-laptopClosed.png` |
| `monitor` Monitor and keyboard | OK | n/a | I-tabletop-05, I-tabletop-01 | I-tabletop-05 S2 content (confirmed); I-tabletop-01 S2 code | `tabletop/sheets/obj-monitor.png`, `tabletop/play/c3-monitor/monitor-on-deskSimple.png` |
| `stuffedToy` Teddy | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-stuffedToy.png` |
| `toyBlocks` Toy blocks | OK | n/a | I-tabletop-01 | I-tabletop-01 S2 code | `tabletop/sheets/obj-toyBlocks.png` |

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `soapSet` | wash hands with it | none | OK (decor; washing is the basin's) |
| `towelStack` | take a towel | none | OK (decor) |
| `perfumeTray` | dab on some scent | none | OK (decor) |
| `deskOrganiser` | take a pen, file a paper | none | OK (decor) |
| `laptopClosed` | open it and work, browse, play | none | missing (I-tabletop-01) |
| `monitor` | work or play at the screen | none, and it fits on almost no top (I-tabletop-05) | missing (I-tabletop-01) |
| `stuffedToy` | cuddle it, play with it (children) | none | missing (I-tabletop-01) |
| `toyBlocks` | stack them (children) | none | missing (I-tabletop-01) |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

Not tied to one row: I-tabletop-04.

### I-tabletop-01 — No thing on a top can be used, though 20 of them are plainly for using

- **Severity:** S2 (raised from S3: a missing obvious use, §4)
- **Rubric:** I1
- **Fix class:** code
- **Where:** the content rule `crates/sim-core/src/content.rs:2738` (things on a surface are 1 × 1 "with one slot and no interactions"); the objects in `web/public/content/catalog2/tabletop.json`: `kettle` `:5`, `toaster` `:22`, `blender` `:39`, `standMixer` `:56`, `espressoMachineSmall` `:73`, `breadBin` `:141`, `cookieJar` `:175`, `fruitBowl` `:192`, `teapotSet` `:243`, `mugs` `:260`, `wineGlasses` `:277`, `bookStack` `:311`, `booksUpright` `:328`, `potSucculent` `:379`, `potCactus` `:396`, `potSmall` `:413`, `laptopClosed` `:600`, `monitor` `:617`, `stuffedToy` `:634`, `toyBlocks` `:651`; models `tools/art/blender/families/tabletop.py:226` (kettle), `:349` (bread bin), `:805` (laptop)
- **Evidence:** Seeded for the kettle: "Content rules forbid uses on things that stand on a top; lifting that needs sim work. Left decorative for 0.3.0."
  Activities wave: of the 23 things `cargo review uses` flags "no use", 20 have a real use that is plainly missing, by what they are for:
  - appliances and screens, whose only reason is a use (the reference marks these kinds "not decor"): `kettle` (boil water for tea), `toaster` (toast), `blender` (a smoothie), `standMixer` (baking), `espressoMachineSmall` (a coffee: the floor `coffeeMachine` has one), `laptopClosed` (work, browse), `monitor` (work at the screen);
  - food and drink to take: `breadBin`, `cookieJar`, `fruitBowl` (a snack), `teapotSet`, `mugs`, `wineGlasses` (a drink);
  - books, small plants and toys: `bookStack`, `booksUpright` (read), `potSucculent`, `potCactus`, `potSmall` (water: the floor `plant` and `hangingPlant` have it), `stuffedToy`, `toyBlocks` (a child's play).
  The 3 others are not plainly missing: `tableClock` (the sim has no alarms), `globe` (spinning it is a gesture), `cuttingBoard` (chopping is the counter's step in cooking). The 16 with no flag are decor by the reference. Every shot is a "no uses" record (`tabletop/play/c1/`, `c2/`). The appliances are the worst: a kettle, toaster and espresso machine on every generated kitchen's counter (I-tabletop-04) and never touched. Owner decision 3 makes the sim work (using a thing on a top, standing at its counter) a `code` finding.
- **Status:** confirmed and widened (activities wave: one cause, the content rule; I-tabletop-02 and I-tabletop-03 merged into it)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5; widened in wave: tabletop

### I-tabletop-02 — The bread bin has no use

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** code
- **Where:** `tools/art/blender/families/tabletop.py:349`, `web/public/content/catalog2/tabletop.json:141`
- **Evidence:** As I-tabletop-01: no uses on things on a top. Left decorative for 0.3.0.
- **Status:** closed (one cause with I-tabletop-01, which now names the bread bin with the other food and drink to take; activities wave)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-tabletop-03 — The laptop has no use

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** code
- **Where:** `tools/art/blender/families/tabletop.py:805`, `web/public/content/catalog2/tabletop.json:600`
- **Evidence:** As I-tabletop-01: no uses on things on a top. Left decorative for 0.3.0.
- **Status:** closed (one cause with I-tabletop-01, which now names the laptop with the other appliances and screens; activities wave)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-tabletop-04 — Every generated house has small things on tops, and none of them can be used

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `web/src/game/housegen.ts`
- **Evidence:** Every one of the 180 sample houses has small things on tops. With the rule against uses on things on a top (I-tabletop-01), they are decor in every home.
  Activities wave: confirmed by the rule (`content.rs:2738`); not re-counted in houses. It is the reach of I-tabletop-01: whether generated kitchens should keep a kettle and a toaster nobody uses, or the fix of I-tabletop-01 comes first.
- **Status:** confirmed (activities wave, by the content rule)
- **Source:** `docs/objects/open.md` Still open › Content and sim

### I-tabletop-05 — The monitor fits on no counter and on almost no desk

- **Severity:** S2
- **Rubric:** I2, M4
- **Fix class:** content
- **Where:** `monitor` in `web/public/content/catalog2/tabletop.json` (`radius`); the disc test for things on tops in `crates/sim-core/src/surface.rs` (refused at `crates/sim-core/src/world.rs:1652`)
- **Evidence:** Things on tops are discs. The monitor's radius is 0.30 plus a 0.04 margin, but the stage's counter top is 0.62 deep (0.31 each way), so it fits at no offset ("would hang over the edge"). It also fits on no standard 0.66 counter and on none of the computer, writing, laptop, standing or kids' desks, though its description says it stands on a counter or desk; only `deskSimple` takes it. `laptopClosed` (radius 0.19) fits. Shots: `tabletop/play/c3-monitor/monitor-on-deskSimple.png`.
  Activities wave: the play run's two buys on the counter were refused (`tabletop/index.md` Problems), the desk shot places it. The model itself is right (0.54 × 0.46 × 0.335, a 24-inch screen with keyboard and mouse); the disc is the screen's width, not its 0.33 m depth.
- **Status:** fixed (543c6121)
- **After:** Radius 0.23 (half its 0.46 m depth with the keyboard): bought on the stage counter (`bench-results/review/fixes/content/g07/after-monitor-no-uses.png`), on the standing, simple and executive desks at their tops' middles; refused on the 0.38–0.42 m deep computer and writing desks. Its text says so.
- **Source:** play runner agent, tooling round

### M-tabletop-01 — The candles are drawn lit all day, and at night they don't glow

- **Severity:** S3
- **Rubric:** M10, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/tabletop.py:98` (`flame=('#FFD27A', 'gloss'),  # gloss, never emissive`), `:509`, `:888`
- **Evidence:** `tabletop/sheets/obj-candlesticks.png` (by day: two yellow flames on the counter), `tabletop/extra/candlesticks-night.png` (22:00 by the sim's clock: the flames are dark grey, nothing lit round them). Real candles are lit for an evening, and then they are the brightest thing on the table. Either unlit wicks (decor) or a flame that glows at night as lamp shades do (`lighting.py:33` gives the lantern's flame a `glow`).
- **Status:** fixed (1c55d5ef)
- **After:** by day the candles show dark wicks, no flame (`fixes/blender-decor/clips/clips-candlesticks-all-t0.png`); from dusk the two flames rise out of the candles' tops (`lit`, the model's `night` clip, held open dusk to dawn: `clips-candlesticks-all-t99.png`) and glow (`#FFC46A`, as the lantern's flame) while the room's lights are on (`candlesticks-night-room-lit.png`). In a room whose lights are off the flames stand unlit (`candlesticks-night.png`): the glow follows the room, the night clip the clock. Two draws (the flames a part of their own).
- **Source:** wave: tabletop

### M-tabletop-02 — The alarm clock is half again a real one's height

- **Severity:** S3
- **Rubric:** M1
- **Fix class:** model
- **Where:** `tools/art/blender/families/tabletop.py:658` (`tableClock`), `:903` (size 0.153 × 0.228 × 0.063)
- **Evidence:** `cargo review sizes`: H 0.228 against the reference's 0.08–0.15 (0.165 with the ±10 %); a twin-bell alarm clock stands 12–18 cm with its bells. On the counter it is as tall as the kettle (`tabletop/play/c2/tableClock-no-uses.png`). Its case is 0.148 across (`R = 0.074`, `:659`), a real one's 9–12 cm; the case drawn about 0.11 across would bring it to 0.17.
- **Status:** fixed (1c55d5ef)
- **After:** built at 0.72 of its first size: the case 10.7 cm across, 0.11 × 0.16 × 0.05 m in all (`cargo review sizes`: ok, was H 0.228; `fixes/blender-decor/gallery/gallery-tableClock.png`). Its hands still tell the time (`clips-tableClock-all-t99.png`).
- **Source:** wave: tabletop

### M-tabletop-03 — At night the candles' flames stand on their wicks in a dark room

- **Severity:** S3
- **Rubric:** M10, I15
- **Fix class:** code
- **Where:** `web/src/render/babylon/objectMotion.ts:225` (a `night` clip is held open by the lamps' level, dusk to dawn, whatever the room); `tools/art/blender/families/tabletop.py` (`candlesticks`, `motions.night: lit`)
- **Evidence:** after M-tabletop-01's fix the flames rise out of the candles from dusk and glow while the room's lights are on (`fixes/blender-decor/candlesticks-night-room-lit.png`); in a room whose lights are out (nobody awake there, `review-stage-night`) the flames still stand on the wicks, unlit and dull (`fixes/blender-decor/candlesticks-night.png`). Real candles are lit by someone and blown out when they leave or go to bed. A night clip that follows the room's lights (as the shade glow does), or the candles lit as a fire is (content `light.fire`, a use), would do.
- **Status:** open
- **Source:** fix: blender-decor
