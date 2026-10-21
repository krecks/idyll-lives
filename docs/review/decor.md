# Decor: authenticity ledger

What it covers: the 22 objects of [docs/objects/decor.md](../objects/decor.md), in its groups and order.

- **Evidence:** `bench-results/review/decor/` (gallery, clip, pose and play shots; shots stay out of git). Shots cells are paths under `bench-results/review/` in the review worktree; `decor/extra/` holds the reviewer's own shots (`extra.mjs`, its log `extra.json`).
- **Reviewed** 2026-10-21 in the activities wave (decor, tabletop and lighting by one reviewer, headless, one core); its log is `bench-results/review/decor/review-log.md`.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Sizes** (`cargo review sizes`): all inside ±10 % but three artefacts: `coatRack` D 0.355 against 0.36 (its feet's spread, 5 mm), `hangingPlant` H 1.83 (the cord from the ceiling, by design: `hangsLow` takes the floor under it) and `fireplace` H 2.80 (its chimney breast to the ceiling; the reference range is for the surround, 1.96 × 1.29 m to the mantel).

## D1. Wall art and clocks (6)

Family: `tools/art/blender/families/decor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `printFramed` Framed print | OK | n/a | OK | | `decor/sheets/obj-printFramed.png` |
| `photoPrint` Photo print | OK | n/a | OK | | `decor/sheets/obj-photoPrint.png` |
| `paintingClassic` Oil painting | OK | OK | OK | | `decor/sheets/obj-paintingClassic.png`, `decor/play/c1/paintingClassic-admire-01.png` |
| `canvasAbstract` Abstract canvas | OK | OK | OK | | `decor/sheets/obj-canvasAbstract.png`, `decor/play/c1/canvasAbstract-admire-01.png` |
| `wallClock` Wall clock | OK | OK | OK | | `decor/sheets/obj-wallClock.png`, `decor/clips/clips-wallClock-all-t99.png` |
| `wallShelf` Floating shelves | OK | n/a | OK | | `decor/sheets/obj-wallShelf.png` |

Notes: the admire use stands the resident at the middle of the tile in front (sim `(10.5, 16.5)` for a picture at `(10, 17, 2)`, `play/c1/play.jsonl`), 1.5 m from the wall, facing it (`ACTIONS.look`: idle and folded arms); the "a tile to the side" note in the evidence index is the camera's 0.7 rad side angle. The clocks' hands show the game's time (`objectMotion.ts:14`).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `printFramed` | look at it | none | OK (decor) |
| `photoPrint` | look at it | none | OK (decor) |
| `paintingClassic` | look at it | admire (5) | OK |
| `canvasAbstract` | look at it | admire (5) | OK |
| `wallClock` | check the time | none; the hands tell the game's time | OK (decor) |
| `wallShelf` | show books and ornaments | none | OK (decor) |

## D2. Ornaments (10)

Family: `tools/art/blender/families/decor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `grandfatherClock` Grandfather clock | OK | A-decor-01, I-decor-02 | I-decor-02 | A-decor-01 S3 motion; I-decor-02 S2 renderer-table | `decor/sheets/obj-grandfatherClock.png`, `decor/play/c2/grandfatherClock-wind-01.png` |
| `vaseTall` Tall vase | OK | n/a | OK | | `decor/sheets/obj-vaseTall.png` |
| `urnPorcelain` Porcelain urn | OK | n/a | OK | | `decor/sheets/obj-urnPorcelain.png` |
| `vasePedestal` Vase on a pedestal | OK | n/a | OK | | `decor/sheets/obj-vasePedestal.png` |
| `bustMarble` Marble bust | OK | OK | OK | | `decor/sheets/obj-bustMarble.png`, `decor/play/c2/bustMarble-admire-01.png` |
| `statueCat` Cat statue | OK | n/a | OK | | `decor/sheets/obj-statueCat.png` |
| `statueHorse` Bronze horse | M-decor-01 | n/a | OK | M-decor-01 S3 model | `decor/sheets/obj-statueHorse.png`, `decor/play/c2/statueHorse-no-uses.png` |
| `coatRack` Coat stand | OK | n/a | I-decor-04 | I-decor-04 S3 decision | `decor/sheets/obj-coatRack.png` |
| `hangingPlant` Hanging plant | OK | OK | I-held-01 | I-held-01 S2 code (seeded; held ledger: no watering can) | `decor/sheets/obj-hangingPlant.png`, `decor/play/c2/hangingPlant-water-01.png` |
| `fireplace` Fireplace | I-decor-01 | I-decor-01 | I-decor-01, I-decor-03 | I-decor-01 S1 model; I-decor-03 S3 code | `decor/sheets/obj-fireplace.png`, `decor/extra/fireplace-relax-front.png`, `decor/extra/fireplace-night-unused.png` |

Notes: the grandfather clock's wind (5 min) is flagged against the `tinker` median (52.5 min) by `cargo review uses`; the reference's wind is 1–5 min, so it is right and the flag is the shared tag. The hanging plant's sway plays on the watering (`hold` 0.8) and its pot hangs at about 1.0–1.4 m: low, but it is `hangsLow` and takes the floor under it (`world.rs:1585`), so nobody walks into it. The fireplace's flames play only while someone warms up by it (`uses relax→flames`).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `grandfatherClock` | wind it once a week; check the time | wind the clock (5, standing, `tinker`) | OK; the pose is I-decor-02 |
| `vaseTall` | decorate a corner | none | OK (decor) |
| `urnPorcelain` | decorate a corner | none | OK (decor) |
| `vasePedestal` | look at it; change the flowers | none | OK (decor; the bust's admire would fit it too) |
| `bustMarble` | look at it | admire (5) | OK |
| `statueCat` | look at it; pat it for luck | none | OK (decor) |
| `statueHorse` | look at it | none | OK (decor, as `vasePedestal`) |
| `coatRack` | hang up a coat and a hat coming in, take them going out | none (the uses table flags "no use") | missing in life, but residents have no coats to hang: I-decor-04 |
| `hangingPlant` | water it | water the plant (5) | OK (no can: I-held-01) |
| `fireplace` | light a fire; warm up by it; read by it | warm up by the fire (30, sitting on the hearth) | lighting it missing (I-decor-03); the seat is I-decor-01; reading by it is a sibling's (`bookworm.firesideLounge`) |

## D3. Textiles (6)

Family: `tools/art/blender/families/decor.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `curtains` Curtains | M-decor-02 | OK | OK | M-decor-02 S3 model | `decor/sheets/obj-curtains.png`, `decor/extra/curtains-pair-straight.png` |
| `curtainsLong` Floor-length curtains | M-decor-02 | OK | OK | M-decor-02 S3 model | `decor/sheets/obj-curtainsLong.png`, `decor/extra/curtainsLong-straight.png` |
| `rugRound` Round rug | OK | n/a | OK | | `decor/sheets/obj-rugRound.png` |
| `rugLarge` Large area rug | OK | n/a | OK | | `decor/sheets/obj-rugLarge.png` |
| `rugRunner` Hall runner | OK | n/a | OK | | `decor/sheets/obj-rugRunner.png` |
| `rugPattern` Patterned rug | OK | n/a | OK | | `decor/sheets/obj-rugPattern.png` |

Notes: the lead "curtains off-centre on the stage window" is closed. The stage's window is two edges wide (`x = 4`, `z = 14` and `15`); `curtainsLong` (2 × 1, rot 1) hangs on both (`world.rs:372`), and straight on its rail is centred on the window (`decor/extra/curtainsLong-straight.png`): the off-centre look in `play/c1/curtainsLong-no-uses.png` is the camera's side angle. `curtains` (1 × 1) hangs on one edge, so on this window it covers half (M-decor-02). The curtains are drawn from dusk to dawn by the lamps' level (`objectMotion.ts:137`, `close` 1.2 s), with nobody at them: code-driven, as the house doors (A12).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `curtains` | draw them at night, open them in the morning | none; drawn by themselves from dusk to dawn (`motions.night`) | OK (the night clip does it; the uses table's "no use" is this) |
| `curtainsLong` | draw them at night, open them in the morning | none; drawn by themselves from dusk to dawn | OK (as `curtains`) |
| `rugRound` | soften the floor; sit or play on it | none | OK (decor) |
| `rugLarge` | soften the floor; sit or play on it | none | OK (decor) |
| `rugRunner` | soften the floor | none | OK (decor) |
| `rugPattern` | soften the floor; sit or play on it | none | OK (decor) |

## Findings

### I-decor-01 — Warming up by the fire, the resident sits in the firebox's mouth with her back in the flames

- **Severity:** S1
- **Rubric:** I3, A1, I2, M7
- **Fix class:** model
- **Where:** `tools/art/blender/families/decor.py:697` (`hearth`: 0.57 deep from the wall face `WALL = -0.43`, `decor.py:120`, so only 0.14 m proud of the surround, whose front is the tile's middle) and `:889`; the seat `web/public/content/furniture.json:4094` (`seat`: height 0.42, `forward` 0.02 from the footprint's middle); no `fireplace` row in `web/src/render/babylon/characters/furniture.json` (the default seat)
- **Evidence:** `decor/extra/fireplace-relax-front.png` (straight on: the flames either side of her torso), `decor/play/c2/fireplace-relax-01.png`, `decor/poses/poses-seats-fireplace.png` (both sitters' hips in the opening). The sim puts the pelvis at `(10, 17.5)` facing out (`extra.json`): the firebox opening's plane is at the tile's middle, so the seat 2 cm in front of it has the body's back inside the opening, against burning logs. A real raised hearth runs 0.4–0.5 m in front of the opening and people sit on its edge or its end, side-on to the fire. Fix with the model (a deeper hearth bench, or a seat at the hearth's end) and the content `seat.forward`.
- **Status:** open
- **Source:** wave: decor (lead from the evidence index's "Seen" notes, confirmed)

### I-decor-02 — The grandfather clock is wound at arm's length, hands at the chest, the dial 0.6 m higher

- **Severity:** S2
- **Rubric:** I3, A1
- **Fix class:** renderer-table
- **Where:** `web/src/render/babylon/characters.ts:518` (`ACTIONS.tinker`: `interact`, `spell`, `pickUp` for every object); `web/public/content/furniture.json:3891` (`wind`, `anim: tinker`); the dial at `DIAL_Y = 1.77` m (`tools/art/blender/families/decor.py:371`)
- **Evidence:** `decor/play/c2/grandfatherClock-wind-01.png`: the resident stands at the middle of the tile in front (sim `(10.5, 16.5)`, the case's front at about 17.6), about 1.1 m from the case, hands out at chest height and touching nothing; `pickUp` stoops towards the floor. Winding a longcase clock is done at the dial (a crank key into the winding holes, about 1.6–1.8 m) or by pulling the weights' chains inside the trunk door. Needs a `tinker` choice by def (reach up at the hood, near), as `look` has for the telescope.
- **Status:** fixed (0c5e889e)
- **After:** `STAND_BACK.grandfatherClock` -0.6 and `ACTIONS.tinker` by def (`interact`, `spell`, no `pickUp`): wound at the trunk door (the weights' chains), the hands on it (`fixes/tables/play-a/a-grandfatherClock-wind-01.png`). A crank key at the 1.77 m dial needs a reach-up clip (resident-clip wave).
- **Source:** wave: decor (lead from the evidence index's "Seen" notes, confirmed)

### I-decor-03 — The fire has no "light it": firelight each evening, flames only while someone sits at it

- **Severity:** S3
- **Rubric:** I1, I15, M10
- **Fix class:** code
- **Where:** `web/public/content/furniture.json:4087` (`light`, `switch: false`); `web/src/render/babylon/house.ts:222` (a light with `switch: false` has no lamp id, so it follows the room's fill rule, `BabylonRenderer.ts:3622`); `tools/art/blender/families/decor.py:889` (`motions.uses.relax → flames`)
- **Evidence:** By code: the firelight (range 4, power 0.8) comes up at dusk in any room with someone awake and goes out when they sleep, while the flames are drawn only during "Warm up by the fire"; between, the grate shows embers (`decor/extra/fireplace-night-unused.png`, a room whose only resident, the baby, sleeps: no firelight). A real fire is lit by someone, burns for hours, and its light and flames go together. A "light the fire" use (or the flames following the light) is missing. Not shot: an awake room at night with nobody at the fire.
- **Status:** open
- **Source:** wave: decor

### I-decor-04 — The coat stand has nothing to hang

- **Severity:** S3
- **Rubric:** I1
- **Fix class:** decision
- **Where:** `web/public/content/furniture.json:4022` (`coatRack`, `interactions: []`)
- **Evidence:** A coat stand's one use is hanging a coat and a hat coming in; the reference doesn't mark the kind as decor (`coatStand`: "hang up a coat and hat"), and `cargo review uses` flags it. Residents have no outerwear to take off, so a use would be a mime (I-held-01). Decide: decor (it carries a coat and a hat already, `decor.py:611`) or a "hang up your coat" use on coming home.
- **Status:** open
- **Source:** wave: decor

### A-decor-01 — The grandfather clock's pendulum hangs still, and its trunk door never opens

- **Severity:** S3
- **Rubric:** A11, A8
- **Fix class:** motion
- **Where:** `tools/art/blender/families/decor.py:8-9` ("the grandfather clock's trunk door and pendulum are still, so the hands keep it within 4 draw calls") and `:374`
- **Evidence:** `decor/clips/clips-grandfatherClock-all-t99.png`: the hands turn with the game's time, the pendulum (behind the glazed trunk door) never swings; a running longcase clock's pendulum swings all the time (about 1 s a beat). The model's own docstring says the door opens to wind it, but no clip does (`motions` has only `clock`). The draw-call budget is the reason given; a pendulum swing is one more rotating part.
- **Status:** open
- **Source:** wave: decor

### M-decor-01 — The bronze horse reads as dark wood

- **Severity:** S3
- **Rubric:** M9
- **Fix class:** model
- **Where:** `tools/art/blender/families/decor.py:90` (`bronze=('#6E4F35', 'metal')`), `:568`
- **Evidence:** `decor/play/c2/statueHorse-no-uses.png`, `decor/gallery/gallery-statueHorse.png`: on its oak plinth the horse is a flat dark brown, the same family as the wood, with no metallic highlight at the game camera. Bronze statuary reads with a greener or golden patina and bright edges.
- **Status:** open
- **Source:** wave: decor (lead from the evidence index's "Seen" notes, confirmed)

### M-decor-02 — The curtains hang inside the window: open, they cover its sides; the short pair covers one edge of a wide window

- **Severity:** S3
- **Rubric:** M3, M4
- **Fix class:** model
- **Where:** `tools/art/blender/families/decor.py:753` (`_curtains`), `:781` (`curtains`: `w` 0.86, rail 0.92), `:785` (`curtainsLong`: `w` 1.8, rail 1.86); footprints `web/public/content/furniture.json:3777`, `:3794`
- **Evidence:** `decor/extra/curtainsLong-straight.png`: on the stage's two-edge window the long pair's rail is barely wider than the frame, so the open panels (0.34 m gathered each) hang over a fifth of the glass on each side; `decor/extra/curtains-pair-straight.png`: the window-height `curtains` are one edge wide, so a two-edge window needs two pairs, and two of the four panels hang in the middle of the glass. A real rail runs 15–25 cm past the frame each side so the open curtains stack off the glass; window-height curtains come in the window's width. Wider rails (about the footprint plus 0.3 m) and a two-edge window-height pair would do.
- **Status:** open
- **Source:** wave: decor (the "off-centre" lead, closed as the camera angle; this is what the straight-on shots show)
