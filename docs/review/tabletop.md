# Tabletop: authenticity ledger

What it covers: the 39 objects of [docs/objects/tabletop.md](../objects/tabletop.md), in its groups and order.

- **Evidence:** `bench-results/review/tabletop/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## TT1. Kitchenware (11)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `kettle` Electric kettle | — | — | — | I-tabletop-01 S3 code (seeded) | |
| `toaster` Toaster | — | — | — | | |
| `blender` Blender | — | — | — | | |
| `standMixer` Stand mixer | — | — | — | | |
| `espressoMachineSmall` Espresso machine | — | — | — | | |
| `knifeBlock` Knife block | — | — | — | | |
| `utensilCrock` Utensil pot | — | — | — | | |
| `cuttingBoard` Chopping board | — | — | — | | |
| `breadBin` Bread bin | — | — | — | I-tabletop-02 S3 code (seeded) | |
| `spiceJars` Spice jars | — | — | — | | |
| `cookieJar` Cookie jar | — | — | — | | |

## TT2. Tableware (7)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `fruitBowl` Fruit bowl | — | — | — | | |
| `plateSet` Place setting | — | — | — | | |
| `cakeStand` Cake on a stand | — | — | — | | |
| `teapotSet` Tea set | — | — | — | | |
| `mugs` Mugs | — | — | — | | |
| `wineGlasses` Wine and glasses | — | — | — | | |
| `candlesticks` Candlesticks | — | — | — | | |

## TT3. Books and small plants (7)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `bookStack` Stack of books | — | — | — | | |
| `booksUpright` Books and bookends | — | — | — | | |
| `vaseFlowers` Vase of flowers | — | — | — | | |
| `vaseSmall` Bud vase | — | — | — | | |
| `potSucculent` Succulent | — | — | — | | |
| `potCactus` Cactus | — | — | — | | |
| `potSmall` Small potted plant | — | — | — | | |

## TT4. Small ornaments (6)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `photoFrame` Photo frame | — | — | — | | |
| `tableClock` Alarm clock | — | — | — | | |
| `globe` Globe | — | — | — | | |
| `snowGlobe` Snow globe | — | — | — | | |
| `trophy` Trophy | — | — | — | | |
| `sculptureSmall` Small sculpture | — | — | — | | |

## TT5. Bath, desk and toys (8)

Family: `tools/art/blender/families/tabletop.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `soapSet` Soap set | — | — | — | | |
| `towelStack` Folded towels | — | — | — | | |
| `perfumeTray` Perfume tray | — | — | — | | |
| `deskOrganiser` Desk organiser | — | — | — | | |
| `laptopClosed` Laptop | — | — | — | I-tabletop-03 S3 code (seeded) | |
| `monitor` Monitor and keyboard | — | — | I-tabletop-05 | I-tabletop-05 S2 content: fits on no counter and almost no desk | |
| `stuffedToy` Teddy | — | — | — | | |
| `toyBlocks` Toy blocks | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

Not tied to one row: I-tabletop-04.

### I-tabletop-01 — The kettle has no use

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** code
- **Where:** `tools/art/blender/families/tabletop.py:226`, `web/public/content/catalog2/tabletop.json:5`
- **Evidence:** Content rules forbid uses on things that stand on a top; lifting that needs sim work. Left decorative for 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L141)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-tabletop-02 — The bread bin has no use

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** code
- **Where:** `tools/art/blender/families/tabletop.py:349`, `web/public/content/catalog2/tabletop.json:141`
- **Evidence:** As I-tabletop-01: no uses on things on a top. Left decorative for 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L141)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-tabletop-03 — The laptop has no use

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** code
- **Where:** `tools/art/blender/families/tabletop.py:805`, `web/public/content/catalog2/tabletop.json:600`
- **Evidence:** As I-tabletop-01: no uses on things on a top. Left decorative for 0.3.0.
- **Status:** seeded (from `docs/objects/open.md` L141)
- **Source:** `docs/objects/open.md` Still open › Content and sim; `plan/release-0.3.0.md` §2.5

### I-tabletop-04 — Every generated house has small things on tops, and none of them can be used

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `web/src/game/housegen.ts`
- **Evidence:** Every one of the 180 sample houses has small things on tops. With the rule against uses on things on a top (I-tabletop-01), they are decor in every home.
- **Status:** seeded (from `docs/objects/open.md` L147)
- **Source:** `docs/objects/open.md` Still open › Content and sim

### I-tabletop-05 — The monitor fits on no counter and on almost no desk

- **Severity:** S2
- **Rubric:** I2, M4
- **Fix class:** content
- **Where:** `monitor` in `web/public/content/catalog2/tabletop.json` (`radius`); the disc test for things on tops in `crates/sim-core/src/surface.rs`
- **Evidence:** Things on tops are discs. The monitor's radius is 0.30 plus a 0.04 margin, but the stage's counter top is 0.62 deep (0.31 each way), so it fits at no offset ("would hang over the edge"). It also fits on no standard 0.66 counter and on none of the computer, writing, laptop, standing or kids' desks, though its description says it stands on a counter or desk; only `deskSimple` takes it. `laptopClosed` (radius 0.19) fits. Shots: `tabletop/play/c3-monitor/monitor-on-deskSimple.png`.
- **Status:** open (found in the Phase 0 tooling round)
- **Source:** play runner agent, tooling round
