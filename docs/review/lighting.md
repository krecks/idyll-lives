# Lighting: authenticity ledger

What it covers: the 21 objects of [docs/objects/lighting.md](../objects/lighting.md), in its groups and order.

- **Evidence:** `bench-results/review/lighting/` (gallery, clip, pose and play shots; shots stay out of git). Night shots: `night/c1`, `night/c2` (the renderer's light at 22:00, the sim's clock at 10:00) and the reviewer's `lighting/extra/` (on `review-stage-night`, the sim's own 22:00).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Activities wave rows** (2026-10-21, decor, tabletop and lighting by one reviewer): Shots are paths under `bench-results/review/`; the log is `bench-results/review/decor/review-log.md`.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **How these were judged.** A lamp has no use but its switch (the player's, `switchLamp`; residents switch a room's lamps together on coming and going, `lights.rs`), which counts as its use (owner decision). For each: where the light comes from (M7: the content `light.height` and `at` inside the shade or bulb; `cargo check-glb`'s heights warnings name two), the night look (M10), the switch (I15). Every lamp is drawn as an omnidirectional point light with a warm floor pool (M-lighting-02), which sets most of the night look. Sizes: all inside ±10 % but `ceilingLight` (M-lighting-04).

## LT1. Ceiling lights (7)

Family: `tools/art/blender/families/lighting.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `ceilingPendant` Pendant lamp | M-lighting-05 | OK | I-lighting-01 | M-lighting-05 S3 decision; I-lighting-01 S3 decision | `lighting/sheets/obj-ceilingPendant.png`, `lighting/extra/ceiling-after-wall-open.png` |
| `ceilingLight` Ceiling light | M-lighting-04 | OK | I-lighting-01 | M-lighting-04 S3 decision; I-lighting-01 S3 decision | `lighting/sheets/obj-ceilingLight.png` |
| `chandelier` Ring chandelier | M-lighting-05 | OK | I-lighting-01 | M-lighting-05 S3 decision (borderline); I-lighting-01 S3 decision | `lighting/sheets/obj-chandelier.png` |
| `pendantIndustrial` Industrial pendant | M-lighting-05 | OK | I-lighting-01 | M-lighting-05 S3 decision; I-lighting-01 S3 decision | `lighting/sheets/obj-pendantIndustrial.png` |
| `ceilingFan` Ceiling fan | OK | A-lighting-01 | A-lighting-01, I-lighting-01 | A-lighting-01 S3 code; I-lighting-01 S3 decision | `lighting/sheets/obj-ceilingFan.png`, `lighting/extra/ceilingFan-night-bought.png` |
| `trackLights` Track lights | OK | OK | I-lighting-01 | I-lighting-01 S3 decision | `lighting/sheets/obj-trackLights.png` |
| `paperLantern` Paper lantern | M-lighting-05 | A-lighting-01 | I-lighting-01 | M-lighting-05 S3 decision; A-lighting-01 S3 code; I-lighting-01 S3 decision | `lighting/sheets/obj-paperLantern.png` |

Notes: the light heights sit in the shades (pendant 1.9 in a dome 1.86–2.1, ceiling light 2.3, chandelier 2.1 at its ring, fan 2.4 under the motor, track 2.5, lantern 2.05 at the globe's middle). The industrial pendant is dark from the game camera at night apart from its rim and the floor pool (an opaque enamel dome from above: right). The fan's light is faint (power 0.8): a fan's light kit is. Seen with the walls cut away, the ceiling things hang in the open room with no ceiling drawn; that is the cutaway.

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `ceilingPendant` | switch it on and off | its switch; the room's lamps by residents | OK |
| `ceilingLight` | switch it on and off | its switch | OK |
| `chandelier` | switch it on and off | its switch | OK |
| `pendantIndustrial` | switch it on and off | its switch | OK |
| `ceilingFan` | switch the fan on or off, and its light | one switch, for the light only; the blades always turn | the fan's own switch missing (A-lighting-01) |
| `trackLights` | switch them on; aim the spots | its switch | OK |
| `paperLantern` | switch it on and off | its switch | OK |

## LT2. Wall lights (3)

Family: `tools/art/blender/families/lighting.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `wallSconce` Wall sconce | M-lighting-02 | OK | I-lighting-01 | M-lighting-02 S3 code; I-lighting-01 S3 decision | `lighting/sheets/obj-wallSconce.png`, `lighting/night/c1/wallSconce-no-uses.png` |
| `wallLampArm` Swing-arm wall lamp | M-lighting-02 | OK | I-lighting-01 | M-lighting-02 S3 code; I-lighting-01 S3 decision | `lighting/sheets/obj-wallLampArm.png`, `lighting/night/c1/wallLampArm-no-uses.png` |
| `wallLightOutdoor` Bulkhead light | M-lighting-02 | OK | OK | M-lighting-02 S3 code | `lighting/sheets/obj-wallLightOutdoor.png`, `lighting/night/extra/wallLightOutdoor-outside-on.png` |

Notes: the bulkhead is a wall thing with no `outdoors`, so it can hang inside too (as a real one does in a garage or a porch). Outside, nobody switches it ("Outdoors … is the player's", `lights.rs:13`): bought on, it burns dusk to dawn, as an outdoor light on a sensor would. With the walls cut away its fixture on the outside face isn't drawn, but its light lands on the inside floor (M-lighting-02).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `wallSconce` | switch it on and off | its switch | OK |
| `wallLampArm` | switch it on; swing it over to read | its switch; the arm is still | OK (the swing is a gesture, not a use) |
| `wallLightOutdoor` | light the door or the path at night | its switch; outdoors nobody else's | OK |

## LT3. Floor lamps (5)

Family: `tools/art/blender/families/lighting.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `lamp` Floor lamp | OK | OK | I-lighting-01 | I-lighting-01 S3 decision (activities wave; all lamps) | `sheets/obj-lamp.png`, `play-night/lamp-no-uses.png`, `gallery-all/gallery-lamp.png` |
| `floorLampSimple` Simple floor lamp | OK | OK | I-lighting-01 | I-lighting-01 S3 decision | `lighting/sheets/obj-floorLampSimple.png` |
| `lampFloorArc` Arc floor lamp | M-lighting-06 | OK | I-lighting-01 | M-lighting-06 S3 model; I-lighting-01 S3 decision | `lighting/sheets/obj-lampFloorArc.png` |
| `lampFloorTripod` Tripod floor lamp | OK | OK | I-lighting-01 | I-lighting-01 S3 decision | `lighting/sheets/obj-lampFloorTripod.png` |
| `lampTorchiere` Uplighter | M-lighting-02 | OK | I-lighting-01 | M-lighting-02 S3 code; I-lighting-01 S3 decision | `lighting/sheets/obj-lampTorchiere.png`, `lighting/extra/lampTorchiere-night-wallsup.png` |

Notes: the arc lamp's light (1.50) is 2 cm under its shade's rim (1.52–1.78, the heights warning): inside the tolerance, and the bulb is there. The switch was shown both ways on `lamp` (`lighting/night/extra/lamp-switched-off.png`, `…-on.png`); the light fades on and off (A12).

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `lamp` | switch it on and off, read by it | its switch in the pie menu (`session.ts:1163`); residents switch a room's lamps together at dusk and on leaving (`lights.rs`) | OK (the switch is its use) |
| `floorLampSimple` | switch it on and off | its switch | OK |
| `lampFloorArc` | light a sofa or a table from above | its switch | OK; it can't reach over a sofa (M-lighting-06) |
| `lampFloorTripod` | switch it on and off | its switch | OK |
| `lampTorchiere` | wash the ceiling with light | its switch | OK; it lights the floor (M-lighting-02) |

## LT4. Table lamps (6)

Family: `tools/art/blender/families/lighting.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `tableLamp` Table lamp | M-lighting-07 | OK | I-lighting-01 | M-lighting-07 S3 content; I-lighting-01 S3 decision | `lighting/sheets/obj-tableLamp.png` |
| `lampTableRound` Ginger-jar lamp | OK | OK | I-lighting-01 | I-lighting-01 S3 decision | `lighting/sheets/obj-lampTableRound.png` |
| `lampTableSquare` Square table lamp | OK | OK | I-lighting-01 | I-lighting-01 S3 decision | `lighting/sheets/obj-lampTableSquare.png` |
| `lampBanker` Banker's lamp | M-lighting-03 | OK | I-lighting-01 | M-lighting-03 S3 model; I-lighting-01 S3 decision | `lighting/sheets/obj-lampBanker.png`, `lighting/extra/lampBanker-night-bought.png` |
| `lampMushroom` Mushroom lamp | M-lighting-03 | OK | I-lighting-01 | M-lighting-03 S3 model; I-lighting-01 S3 decision | `lighting/sheets/obj-lampMushroom.png` |
| `lampLantern` Candle lantern | OK | OK | I-lighting-01 | I-lighting-01 S3 decision | `lighting/sheets/obj-lampLantern.png` |

Notes: the lamps on a top shine from their own height above it (0.15–0.42, in each shade or glass); each throws a disc up the wall behind the counter (M-lighting-02), most of all the banker's lamp, whose real light goes down onto the desk.

**Real uses against the content's (I1)**

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `tableLamp` | switch it on and off | its switch | OK |
| `lampTableRound` | switch it on and off; read by it | its switch | OK |
| `lampTableSquare` | switch it on and off | its switch | OK |
| `lampBanker` | light the desk to work | its switch | OK |
| `lampMushroom` | switch it on and off | its switch | OK |
| `lampLantern` | light the candle | its switch | OK |

The pilot's `lamp` row of the I1 table moved into LT3's table (its I cell now names I-lighting-01, filed in the activities wave for every lamp).

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

Not tied to one row: M-lighting-01, M-lighting-02 (every lamp).

### M-lighting-01 — moved to systems.md

### M-lighting-02 — moved to systems.md

### M-lighting-03 — Coloured shades glow the same cream at night

- **Severity:** S3
- **Rubric:** M10, M9
- **Fix class:** model
- **Where:** `tools/art/blender/families/lighting.py:26` (`GLOW = {'glow': '#FFD9A0'}`, one glow colour for every shade); `web/src/render/babylon/BabylonRenderer.ts:4627` (`applyGlow`)
- **Evidence:** `lighting/sheets/obj-lampBanker.png` (day: a green glass shade; night: cream), `lighting/sheets/obj-lampMushroom.png` (orange by day, cream at night), `lighting/extra/lampBanker-night-bought.png`. A lit coloured glass or enamel shade keeps its colour, brighter (a banker's lamp is green lit); a white fabric or paper shade goes cream, as these all do. A glow tinted by the shade's own colour for the coloured ones would do.
- **Status:** fixed (4029f842)
- **After:** the coloured glass shades glow in their own colour: the banker's lamp's green shade `#4FB27A` over its white liner (cream), the mushroom lamp's coral cap `#FFAA78` (`fixes/blender-decor/lampBanker-night.png`, `lampMushroom-night.png`; before: `lighting/extra/lampBanker-night-bought.png`). The white and paper shades keep the cream `GLOW`.
- **Source:** wave: lighting

### M-lighting-04 — The "ceiling light" is a lantern pendant 0.6 m deep, not a ceiling light

- **Severity:** S3
- **Rubric:** M1, M5
- **Fix class:** decision
- **Where:** `tools/art/blender/families/lighting.py:185` (`ceilingLight`: a 0.36 m frosted cube on a 0.28 m stem), `web/public/content/furniture.json:4142`
- **Evidence:** `lighting/sheets/obj-ceilingLight.png`; `cargo review sizes`: H 0.60 against the reference's flush ceiling light 0.05–0.30. What is drawn is a square lantern pendant (its docstring says so); what a buyer of a "Ceiling light" expects is a flush or semi-flush fitting. Either rename it (and its kind to `pendantLamp`, inside whose range it is) or build a flush one.
- **Status:** fixed (b3f6b82d)
- **After:** "Lantern pendant", kind `pendantLamp`: lighting 1 → 0 outside.
- **Source:** wave: lighting

### M-lighting-05 — Pendants hang at head height where residents walk under them

- **Severity:** S3
- **Rubric:** M4, M2
- **Fix class:** decision
- **Where:** `tools/art/blender/families/lighting.py:173` (`ceilingPendant`, dome bottom about 1.86 m), `:235` (`pendantIndustrial`, 1.93), `:297` (`paperLantern`, globe bottom 1.80), `:205` (`chandelier`, ring 2.04); `crates/sim-core/src/home.rs:491` ("Residents walk under pictures and lamps"); the adult man is 1.77 m (`crates/tools/src/characters/rig.rs:81`)
- **Evidence:** `cargo review sizes` (H from the 2.8 m ceiling: 0.95, 0.87, 1.00, 0.76), `lighting/gallery/gallery-ceilingPendant.png`, `lighting/gallery/gallery-paperLantern.png`. The reference (`pendant`) gives at least 2.1 m clear where people walk, or 0.75–0.9 m over a dining table (bottom 1.5–1.65 m); these hang between, 3–16 cm over a man's crown, and nothing keeps residents from walking under (they are not `hangsLow`). Decide per lamp: a shorter drop (walk-under) or `hangsLow` (over a table only, as the hanging plant).
- **Status:** fixed (05970ca3, 4029f842)
- **After:** Q-12: the ring chandelier and the industrial pendant are `headHeight` (content): nobody walks under them, furniture still goes under (a dining table, a sofa), not furniture as tall as the wall (sim test `layers.rs` `a_lamp_at_head_height_hangs_over_a_table_not_over_heads`; generated houses take their floor from walking and use another kind where one would block the way). Blender wave (4029f842): the pendant lamp and the paper lantern hang a 0.69 m drop, their lowest point 2.11 and 2.12 m up (the renderer's `objectBox` bottoms; was 1.85 and 1.80), 0.34 m over the adult man's 1.77 m, the reference's 2.1 m where people walk; `light.height` 2.16 and 2.37 (`fixes/blender-decor/ceilingPendant-under.png`, `paperLantern-under.png`, at night `ceilingPendant-night.png`, `paperLantern-night.png`; `cargo review sizes` H 0.69 and 0.68, ok).
- **Source:** wave: lighting

### M-lighting-06 — The arc lamp's arc doesn't reach past its own tile

- **Severity:** S3
- **Rubric:** M6, M3
- **Fix class:** model
- **Where:** `tools/art/blender/families/lighting.py:423` (`ARC`: from the base at `(-0.3, -0.3)` to the shade at `(0.22, 0.18)`), `:427`; `web/public/content/catalog2/lighting.json:120` (footprint 1 × 1, `light.at` `[0.245, 0.2]`)
- **Evidence:** `lighting/sheets/obj-lampFloorArc.png`: the shade hangs about 0.7 m from the base, over the same tile. An arc lamp exists to reach 1.2–2 m over a sofa or a table from behind it (the reference's use, "light a sofa or a table from above", W up to 1.8 m); this one has to stand beside the sofa, where a plain floor lamp would. A 2-tile reach (footprint 1 × 1 with the shade over the next tile, as things hung over others are) needs the sim to allow it.
- **Status:** fixed in the sim (2b7cf0d8); the model's half later (recorded in `tools/art/blender/families/lighting.md`)
- **After:** content `overhang` (`[right, front]` tiles from a 1×1 floor thing) lets a shade reach over the next tile: taken from walking and from anything tall, hanging low or at head height (furniture may stand under it, as under a lamp at head height), no wall across the arm, not off the lot or over the stairs (`layers::an_arc_lamp_reaches_over_the_sofa_in_front_of_it`). Not set on `lampFloorArc` until its model reaches over the next tile (then `overhang: [[0, 1]]`, `light.at` and build mode's and `housegen.ts`'s placing).
- **Source:** wave: lighting

### M-lighting-07 — The table lamp's light is 10 cm above its shade

- **Severity:** S3
- **Rubric:** M7, M2
- **Fix class:** content
- **Where:** `web/public/content/furniture.json:4337` (`tableLamp` `light.height` 0.95)
- **Evidence:** `cargo check-glb` heights warning: "light 0.95 m, outside the lamp there (its model 0.13 to 0.85 m at x -0.12)": the shade's top is 0.85 m, so the light shines from the air above it. At night the pool looks right (`lighting/night/c1/tableLamp-no-uses.png`), but the light's height sets where the wall disc lands. About 0.75 (inside the shade) fixes it.
- **Status:** fixed (a920f174)
- **After:** `tableLamp` `light.height` 0.75, inside the shade (0.13–0.85 m); its heights warning gone.
- **Source:** wave: lighting

### I-lighting-01 — moved to systems.md

### A-lighting-01 — The ceiling fan turns with its light off; the paper lantern sways in still air

- **Severity:** S3
- **Rubric:** A11, A12, I15
- **Fix class:** code
- **Where:** `web/src/render/babylon/objectMotion.ts:239` (an `idle` clip plays whenever the camera is near, whatever the object's state); `tools/art/blender/families/lighting.py:248` (`ceilingFan`, `blades_turn` idle), `:297` (`paperLantern`, `sway` idle)
- **Evidence:** `lighting/extra/ceilingFan-night-bought.png`; after `switchLamp` off, the fan's `blades_turn` clip still runs (its time 0.2 then 0.9 s, 0.7 s apart: `decor/extra/extra.json`), and it turns by day in an empty house. A real fan has its own switch (pull cord or wall) and stops; the content has one switch, for the light. The paper lantern's sway goes on day and night indoors, where a paper globe hangs still without a draught. The fan's clip following its switch (on: turning) and the lantern's sway kept for open windows or dropped would do.
- **Status:** fixed (0eaa3ba7, 4029f842)
- **After:** a lamp's `idle` clip goes round only while it is switched on (`objectMotion.ts`): the fan's blades stop with its light off (in the browser, `blades_turn` absent bought off, turning 1.5 → 2.2 s once switched on, gone when off again). Blender wave (4029f842): the paper lantern hangs still, one joined node with no clip (`sway`, `IDLE_SWAY` and its manifest `motions` gone).
- **Source:** wave: lighting

### M-lighting-08 — A lamp's light passes through walls into the next room

- **Severity:** S3
- **Rubric:** M10
- **Fix class:** code
- **Where:** `web/src/render/babylon/BabylonRenderer.ts` (`setupLights`: the room lights have no shadows and no room of their own), `web/src/render/babylon/house.ts` (`placeLights`)
- **Evidence:** the part of M-lighting-02 its fix left: a lamp near a wall lights the floor and furniture of the room behind it within its reach (4–7 m); aimed lamps (`light.aim`, G-60) leak less, all-round ones (a floor lamp, a fire) as before. Keeping a light to its room needs a per-room mask in the cluster's shader or shadows per lamp, both new renderer work with a frame cost to measure first.
- **Status:** fixed (ae10185e) for the clustered lamps (Medium, High); Low's forward lamps not masked
- **After:** every lit fragment and every clustered lamp get their room from a mask of the lot in view (a texel per tile on each storey shown, outdoors its own; a fragment classed 0.1 m out along its normal), and Babylon's clustered lighting loop skips a lamp in another room (`web/src/render/babylon/roomLights.ts`, patched into `pbrClusteredLightingFunctions` for WGSL and GLSL). office-night, before and after (`?roomlights=0`), WebGPU and WebGL2: the light that reached the bedroom's and the bathroom's walls and floors from the living room's chandeliers, and the outside of the walls, is gone; each room keeps its own (`fixes/sweep-sim/M-lighting-08/before-*.png`, `after-*.png`, `diff-*.png`). Cost, village orbit at night, window, no vsync, median of 3 (M3 Pro): Medium WebGPU GPU 3.35 → 3.47 ms (196 → 194 fps), WebGL2 6.31 → 6.30 ms; Low 2.50 / 3.27 ms either way (`fixes/sweep-sim/M-lighting-08/perf-roomlights-0`, `-1`). Not masked: Low's four forward lamps, tiles split by a diagonal wall; light now also stops at an open doorway.
- **Source:** fix: lighting
