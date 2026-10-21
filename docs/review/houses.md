# Houses: authenticity ledger

What it covers: the house parts built in code: walls, doors and windows, coverings, roofs, stairwells and storeys, and the neighbours' shells with their porches, chimneys and garages. Built by `web/src/render/babylon/house.ts` (door swing: `BabylonRenderer.swingDoors`); styles from the `base.json` lists. The staircase itself is a content object (`stairs`, [living.md](living.md)). Rubric M17 in particular.

- **Evidence:** `bench-results/review/houses/extra/` (shots on `review-stage` from `houses.mjs`: every door style with Ada beside it, every window style, the front door walked through at speed 3 and 1 and walked past, the eight roof styles and colours, 25 wall and 13 floor coverings, a bathroom door walked past; the log is `houses.jsonl`, the contact sheets `sheet-*.png`), the neighbours' shells in `town/views/` (bench village), the stairs in `pilot/scenes/stairs-*.png` (sheet: `houses/extra/stairs.png`); shots stay out of git.
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Wave: houses** (2026-10-21). Measured from code against the house standards in `reference.md`: wall 2.8 m (`walls.ts:7`), 0.14 m thick; door opening 0.86 × 2.10 m (a 1 m edge less two wall halves; `DOOR_HEIGHT`), leaf 0.80 m standing open or 0.83 m when it swings shut, 2.06 m high, 45 mm thick (`house.ts:123-124`, `:1835`, `:1957`); windows 0.76 m wide in their 1 m edge (`WIN_INSET` 0.12), sills and heads by style (`base.json` `windowStyles`); stairs 14 risers of 200 mm over 4 m (going about 286 mm, 2R + G 686 mm, about 35°: inside every range; `stairs` row in the living ledger).

## Walls (7)

`house.ts` `build`; `WALL_HEIGHT`, `PLINTH`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Straight wall | OK | n/a | n/a | 2.8 m high, 0.14 m thick, siding or brick outside, plaster or wallpaper inside, corner boards; thin for an outside wall (real 0.25–0.4 m) but reads as one at the game's distance | `houses/extra/sheet-doors.png`, `houses/extra/sheet-windows.png` |
| Diagonal wall | OK | n/a | n/a | Corner to corner across a tile with mitred posts (`slanted.ts`); code only, no state has one | none (code only) |
| Half wall (`form` 1) | OK | n/a | n/a | 1.05 m (`house.ts:52`), a pony wall's height (0.9–1.1 m); code only | none (code only) |
| Wall height and storeys | OK | n/a | n/a | 2.8 m a storey: inside the storey range (2.6–3.1 m) and at the top of the ceiling range (2.4–2.7 new, 2.7–3.0 older houses); 0.7 m of wall over a 2.1 m door | `houses/extra/door-panel.white.png`, `houses/extra/stairs.png` |
| Plinth and foundation | OK | n/a | n/a | 0.32 m stone band (`PLINTH`); the floor is at ground level, no step at the door (Sims walk in level) | `houses/extra/sheet-doors.png` |
| Basement walls (stone-faced earth) | OK | n/a | n/a | Faced in stone all the way up (`house.ts:251`, `:542`); code only, no basement in the states driven | none (code only) |
| Wall cut-away (lowered walls) | M-houses-01 | n/a | n/a | M-houses-01 S3 code (seeded; not checked). The cut-away itself lowers the walls towards the camera cleanly (`houses/extra/bath-occupied-shut.png`) | `houses/extra/sheet-bath.png` |

## Doors (11)

`DOOR_HEIGHT` 2.1, `LEAF_W` 0.8; swing in `BabylonRenderer.ts` `swingDoors`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `panel` Painted panel | M-houses-02 | n/a | OK | 0.83 × 2.06 m leaf in a 0.86 × 2.1 m opening: in the range; M-houses-02 S3 code: a flat slab, no panels. A and I: the swing rows | `houses/extra/door-panel.png`, `houses/extra/sheet-doors.png` |
| `panel.white` White panel | M-houses-02 | n/a | OK | as `panel`; M-houses-02 | `houses/extra/door-panel.white.png` |
| `panel.red` Red panel | M-houses-02 | n/a | OK | as `panel`; M-houses-02 | `houses/extra/door-panel.red.png` |
| `panel.blue` Blue panel | M-houses-02 | n/a | OK | as `panel`; M-houses-02 | `houses/extra/door-panel.blue.png` |
| `oak` Natural oak | M-houses-03 | n/a | OK | M-houses-03 S3 code: dark walnut-brown, not oak | `houses/extra/door-oak.png`, `houses/extra/textures.png` |
| `halfGlass` Half-glazed | OK | n/a | OK | solid to 1.05 m, glazed above between 0.1 m stiles | `houses/extra/door-halfGlass.png` |
| `glass` French glass | OK | n/a | OK | glazed from 0.22 m in a dark 0.08 m frame with a rail at 1.1 m; a single French door | `houses/extra/door-glass.png` |
| Front door swing | n/a | A-houses-01, A-houses-02 | A-houses-01 | A-houses-01 S2 code (seeded; confirmed and widened); A-houses-02 S3 code; both fixed (e04bc6c0): opens only for someone going through, ahead of them at any speed. Hinge on the jamb, swings 90° into the house (real 90–110°), eased, 0.45 s: right at speed 1 (fully open 0.6 m before the resident) | `houses/extra/swing1-in-1.png`, `houses/extra/swing1-past-0.png`, `houses/extra/swing-in-0.png` |
| Bathroom door swing | n/a | A-houses-01 | A-houses-01 | A-houses-01 S2 code: shut while the bathroom is in use, but swings open for anyone walking past. Fixed (e04bc6c0): passers-by leave it shut (`fixes/town/doors/bath-passer-0.png`) | `houses/extra/sheet-bath.png`, `houses/extra/bath-passer-open-0.png` |
| A door beside a diagonal wall (part swing, or slides into the wall) | OK | OK | OK | Opens as far as the diagonal lets it; under 60° it slides into the wall as a pocket door, its edge showing at the jamb (`house.ts:1879-1901`): what a builder would do; code only, no state has one | none (code only) |
| Door frames, sills and handles (all styles) | OK | n/a | n/a | Casings 0.1 m round the opening, head board to 2.32 m; a brass handle on both faces at 1.0–1.05 m, 7–12 cm from the edge away from the hinge (`house.ts:1973-1975`; real lever 0.9–1.05 m); no threshold (level entry) | `houses/extra/door-panel.white.png`, `houses/extra/door-glass.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| Front door | open it and walk through; shut it behind you; lock it; visitors knock or ring | swings open as anyone comes near (or walks past) and shuts behind them; visitors knock at it (`base.json` `visits`, judged in the systems ledger); no lock, no bell | A-houses-01 |
| Bathroom door | shut it for privacy while in there | shut in the drawing only while the room is occupied (privacy in the sim: I-systems-05, systems ledger); opens for passers-by | A-houses-01 |
| Inside doors | open, shut | stand open against the wall for good (`swingDoors`: only front and bathroom doors shut) | OK (a common way to live) |

## Windows (8)

`SILL` 0.9, `HEAD` 2.1; `muntins`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `classic` Classic | OK | n/a | n/a | 0.76 × 1.2 m glass, sill 0.9, head 2.1 (in both ranges), cross bars, the house's shutters | `houses/extra/window-classic.png`, `houses/extra/window-inside-classic.png` |
| `cottage` Cottage | OK | n/a | n/a | 0.72 × 0.95 m, sill 1.0, head 1.95 (just under the 2.0 m head range, as cottage windows are), grid bars, shutters | `houses/extra/window-cottage.png` |
| `picture` Picture | OK | n/a | n/a | 0.88 × 1.5 m, sill 0.7, head 2.2, no bars | `houses/extra/window-picture.png` |
| `tall` Floor-length | OK | n/a | n/a | sill 0.25, head 2.3, one bar: reads as a French window; below 0.6 m it would need safety glass or a guard upstairs (`reference.md` sill row), which a stylised pane doesn't show | `houses/extra/window-tall.png` |
| `high` High light | OK | n/a | n/a | sill 1.55 (eye height), head 2.2: a bathroom or hall window | `houses/extra/window-high.png` |
| `transom` Transom | OK | n/a | n/a | sill 0.6, head 2.3, a transom bar | `houses/extra/window-transom.png` |
| Window glass and sunlight through it | OK | n/a | n/a | Clear glass shows the room from outside by day (a deliberate see-through), sunlight falls through onto the floor (`pilot/scenes/stairs-10.png`) | `houses/extra/sheet-windows.png`, `houses/extra/stairs.png` |
| Windows glowing at night | OK | OK | n/a | Lit rooms' windows glow warm and fade with the room lights (`setWindowGlow`) | `town/views/02-street-cars-h22.png`, `town/extra/views-night.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| Windows (all styles) | look out; let light in; open for air; draw the curtains | let light in; no use (no looking out, no opening, curtains only on the neighbours' shells) | n/a (decor; not a clear missing use for a life sim at this scale) |

## Wall coverings (25)

`base.json` `wallCoverings`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `paint.white` White paint | OK | n/a | n/a | flat plaster, the colour reads | `houses/extra/covers-west-a.png` |
| `paint.cream` Cream paint | OK | n/a | n/a | as above | `houses/extra/covers-west-a.png` |
| `paint.butter` Butter paint | OK | n/a | n/a | as above | `houses/extra/covers-west-a.png` |
| `paint.sage` Sage paint | OK | n/a | n/a | as above | `houses/extra/covers-west-a.png` |
| `paint.sky` Sky-blue paint | OK | n/a | n/a | as above | `houses/extra/covers-west-a.png` |
| `paint.blush` Blush paint | OK | n/a | n/a | as above | `houses/extra/covers-west-a.png` |
| `paint.terracotta` Terracotta paint | OK | n/a | n/a | as above | `houses/extra/covers-west-b.png` |
| `paint.navy` Navy paint | OK | n/a | n/a | as above | `houses/extra/covers-west-b.png` |
| `paint.charcoal` Charcoal paint | OK | n/a | n/a | as above | `houses/extra/covers-west-b.png` |
| `wallpaper.linen` Linen wallpaper | M-houses-04 | n/a | n/a | M-houses-04 S3 content: no pattern, reads as paint | `houses/extra/covers-west-b.png`, `houses/extra/textures.png` |
| `wallpaper.mint` Mint wallpaper | M-houses-04 | n/a | n/a | M-houses-04 | `houses/extra/covers-west-b.png` |
| `wallpaper.lilac` Lilac wallpaper | M-houses-04 | n/a | n/a | M-houses-04 | `houses/extra/covers-west-b.png` |
| `wallpaper.rose` Rose wallpaper | M-houses-04 | n/a | n/a | M-houses-04 | `houses/extra/covers-east-a.png` |
| `siding.white` White siding | OK | n/a | n/a | horizontal boards | `houses/extra/covers-east-a.png` |
| `siding.blue` Blue siding | OK | n/a | n/a | as above | `houses/extra/covers-east-a.png` |
| `siding.green` Green siding | OK | n/a | n/a | as above | `houses/extra/covers-east-a.png` |
| `siding.yellow` Yellow siding | OK | n/a | n/a | as above | `houses/extra/covers-east-a.png` |
| `brick.red` Red brick | OK | n/a | n/a | brick courses at about real size | `houses/extra/covers-east-a.png`, `houses/extra/sheet-doors.png` |
| `brick.smoked` Smoked brick | OK | n/a | n/a | as above, darker | `houses/extra/covers-east-b.png` |
| `wood.oak` Oak panelling | OK | n/a | n/a | vertical boards, golden | `houses/extra/covers-east-b.png` |
| `wood.walnut` Walnut panelling | OK | n/a | n/a | vertical boards, dark | `houses/extra/covers-east-b.png` |
| `stone` Fieldstone | OK | n/a | n/a | rubble stone | `houses/extra/covers-east-b.png` |
| `tile.white` White tiles | OK | n/a | n/a | square tiles with joints | `houses/extra/covers-east-b.png` |
| `tile.blue` Blue tiles | OK | n/a | n/a | as above | `houses/extra/covers-front-corner.png` |
| `tile.green` Green tiles | OK | n/a | n/a | as above | `houses/extra/covers-front-corner.png` |

## Floor coverings (13)

`base.json` `floorCoverings`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `wood.oak` Oak boards | OK | n/a | n/a | boards at plank width, golden pine-oak | `houses/extra/floors-all.png` |
| `wood.honey` Honey pine | OK | n/a | n/a | as above, deeper | `houses/extra/floors-all.png` |
| `wood.walnut` Walnut boards | OK | n/a | n/a | dark | `houses/extra/floors-all.png` |
| `wood.whitewashed` Whitewashed wood | M-houses-04 | n/a | n/a | M-houses-04 S3 content: reads as plain pine | `houses/extra/floors-all.png`, `houses/extra/textures.png` |
| `tile.white` White tiles | OK | n/a | n/a | the same look as the default tile floor (`material.floor.tile`) | `houses/extra/floors-all.png` |
| `tile.sand` Sand tiles | OK | n/a | n/a | as above, warmer | `houses/extra/floors-all.png` |
| `tile.terracotta` Terracotta tiles | OK | n/a | n/a | terracotta squares | `houses/extra/floors-all.png` |
| `tile.slate` Slate tiles | OK | n/a | n/a | grey-blue squares | `houses/extra/floors-all.png` |
| `carpet.oatmeal` Oatmeal carpet | OK | n/a | n/a | pile texture; greyer than its oatmeal swatch (the carpet texture is grey) | `houses/extra/floors-all.png`, `houses/extra/floors-close.png` |
| `carpet.sage` Sage carpet | OK | n/a | n/a | pile, green | `houses/extra/floors-all.png` |
| `carpet.blue` Blue carpet | OK | n/a | n/a | pile, blue | `houses/extra/floors-all.png` |
| `carpet.rose` Rose carpet | OK | n/a | n/a | pile, rose | `houses/extra/floors-close.png` |
| `stone.flag` Flagstone | OK | n/a | n/a | stone slabs, mauve-grey | `houses/extra/floors-all.png` |

## Roofs (7)

`addRoof`, `PITCH` 34°, `OVERHANG` 0.4; `base.json` `roofStyles`, `roofColors`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `gable` Gable | OK | n/a | n/a | 34°, closed gable ends in the wall's cladding, ridge along the longer side | `houses/extra/roof-gable-slate.png`, `houses/extra/sheet-roofs.png` |
| `hip` Hip | OK | n/a | n/a | 34°, a pyramid on a square plan | `houses/extra/roof-hip-brown.png` |
| `steepGable` Steep gable | OK | n/a | n/a | 46° | `houses/extra/roof-steepGable-terracotta.png` |
| `lowHip` Low hip | OK | n/a | n/a | 20° | `houses/extra/roof-lowHip-moss.png` |
| `flat` Flat | OK | n/a | n/a | slab just over the walls, 0.16 m overhang, fascia | `houses/extra/roof-flat-charcoal.png` |
| Roof colours (slate, brown, terracotta, moss, charcoal, taupe, navy, pearl) | OK | n/a | n/a | the eight read as roofing colours on the shingle texture | `houses/extra/sheet-roofs.png` |
| Eaves, fascia and soffit | OK | n/a | n/a | 0.4 m overhang (real 0.3–0.6 m), eaves resting on the wall's outer top edge, fascia boards, soffit as the roof's white underside, ridge cap (`house.ts:2228-2316`); the close-up hid the roof (camera too close), so judged from the wide shots | `houses/extra/sheet-roofs.png` |

## Stairwells and storeys (4)

`house.ts` (railing round each stairwell, landings).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Stairwell opening | OK | n/a | n/a | the floor above left open over the stairs' tiles | `houses/extra/stairs.png` |
| Railing round a stairwell | M-houses-07 | n/a | n/a | 0.95 m handrail (UK guarding in a home 0.9 m), posts at its ends; M-houses-07 S3 code: balusters 116 mm apart | `houses/extra/stairs.png` |
| Landings | OK | n/a | n/a | the railing leaves the landing side open | `houses/extra/stairs.png` |
| Ceilings and floors between storeys | OK | n/a | n/a | the upper storey's floor at 2.8 m; seen from above as a floor (no ceiling drawn: the camera looks down) | `houses/extra/stairs.png` |

## Neighbour shells (8)

`HouseBuilder.silhouettes`, `porch`, `chimney` (`house.ts`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Shell walls and roof | OK | n/a | n/a | the viewed house's look: plinth, siding or brick, frieze, corner boards, the same roofs, 2.8 m a storey | `town/views/02-street-cars-h13.png`, `town/views/04-neighbour-4-sycamore-garage-h13.png` |
| Shell windows, lit and curtained at night | OK | OK | n/a | curtains drawn to the sides with a valance, glass lit while someone in there is up; all lit at 22:00 in the village is that rule, not a fault | `town/views/03-neighbour-2-sycamore-h22.png`, `town/extra/views-night.png` |
| Porch: gabled canopy over the front door | OK | n/a | n/a | 1.9 × 1.3 m landing 5 cm high, posts to 2.34 m, gabled canopy, pots either side (`house.ts:1743-1767`) | `town/views/03-neighbour-2-sycamore-h13.png`, `town/views/04-neighbour-4-sycamore-garage-h13.png` |
| Chimney | M-houses-05 | n/a | n/a | 0.6 × 0.76 m brick stack, 0.75 m over the ridge with a stone cap: right on one-storey shells; M-houses-05 S3 code: two-storey shells never show one | `town/views/03-neighbour-2-sycamore-h13.png`, `town/extra/views-05-08.png` |
| Garage (`GARAGE_HEIGHT` 2.5) | M-houses-06 | n/a | n/a | 2.5 m walls, 5.4 m deep (real single garage 3.0 × 5.5–6 m), its own low roof; M-houses-06 S3 code: 2.7–3.3 m wide | `town/views/09-willow-street-h13.png`, `town/extra/views-05-08.png` |
| Garage door facing the street | M-houses-06 | n/a | n/a | sectional door 2.12 m high with three joints, white or the house colour; M-houses-06: 2.0–2.6 m wide | `town/views/09-willow-street-h13.png` |
| Porch and garage lamps | OK | OK | OK | wall lanterns at 1.75 m beside the front door and 1.9 m beside the garage door, lit at dusk with a pool of light | `town/views/03-neighbour-2-sycamore-h22.png` |
| Shell front door | OK | n/a | n/a | 0.82 × 2.1 m leaf with four raised panels, brass knob at 1.0–1.06 m (`house.ts:1661-1672`) | `town/views/03-neighbour-2-sycamore-h13.png` |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### M-houses-01 — Wall things shown while one is held float over lowered walls

- **Severity:** S3
- **Rubric:** M4
- **Fix class:** code
- **Where:** `web/src/game/buildmode.ts`
- **Evidence:** While the player holds a wall thing, the ones already hung show at full height over walls the cut-away has lowered, pictures on outside walls too.
- **Status:** not checked (needs Build mode with a wall object held; the wave's harness drives Live mode only)
- **Source:** `docs/objects/open.md` Still open › Rendering

### A-houses-01 — moved to systems.md

### A-houses-02 — moved to systems.md

### M-houses-02 — The "Painted panel" doors are flat slabs

- **Severity:** S3
- **Rubric:** M6, M13
- **Fix class:** code
- **Where:** `web/src/render/babylon/house.ts:1960` (`panel`: one box the size of the leaf)
- **Evidence:** All four panel styles (`panel`, `panel.white`, `panel.red`, `panel.blue`) are a plain slab with a knob (`houses/extra/door-panel.png`, `door-panel.white.png`, `door-panel.red.png`, `door-panel.blue.png`), while the neighbours' shell doors right across the street have four raised panels (`house.ts:1666-1671`, `town/views/03-neighbour-2-sycamore-h13.png`). The style's name promises the panels the shells have; the same four raised boards on both faces of the leaf would do.
- **Status:** fixed (8e0f83c2)
- **After:** the painted and oak leaves have four raised panels on both faces, a shade lighter than the leaf, as the neighbours' doors (`house.ts` `leaf`): `fixes/town/houses/door-panel.white.png`, `door-panel.red.png`, `door-panel.blue.png`, against `houses/extra/door-panel.white.png`. Triangles only: the panels are in the leaf's own bucket.
- **Source:** wave: houses

### M-houses-03 — The "Natural oak" door is dark walnut

- **Severity:** S3
- **Rubric:** M9
- **Fix class:** code
- **Where:** `web/src/render/babylon/house.ts:1959` (the oak leaf on the door wood, untinted), `web/public/assets/manifest.json:1164` (`material.door`: `textures/door_wood`)
- **Evidence:** The door wood texture is a dark brown walnut-like grain (`houses/extra/textures.png`, second tile), so the oak door reads dark red-brown, darker than the walnut floor (`houses/extra/door-oak.png`, `houses/extra/sheet-doors.png`). Natural oak is light gold, as the oak panelling and oak boards show. The game has an oak texture (`textures/wood_oak`); the oak leaf could use it, or the door wood could be lightened and stained per style.
- **Status:** fixed (8e0f83c2)
- **After:** the oak leaf is drawn on the oak grain the oak panelling uses (`material.finish.wood`, `textures/wood_oak`), in a light golden oak (`OAK` #D4B48A), and in the panelling's bucket (one draw fewer where both are used): `fixes/town/houses/door-oak.png` against `houses/extra/door-oak.png`. `material.door` and `textures/door_wood` are now unused: M-houses-08.
- **Source:** wave: houses

### M-houses-04 — Wallpapers read as paint, whitewashed wood as plain pine

- **Severity:** S3
- **Rubric:** M9, M6
- **Fix class:** content
- **Where:** `web/public/assets/manifest.json:1144` (`material.wall.interior`: `textures/wallpaper/albedo.jpg`), `web/public/content/base.json:3600-3603` (the four wallpapers), `base.json:3621` (`wood.whitewashed`, `#F4EFE6` on the floor wood)
- **Evidence:** The wallpaper texture is a pale grey mottle with no pattern, the same kind as the plaster's (`houses/extra/textures.png`, first and last tiles), so linen, mint, lilac and rose wallpaper look like four more paints (`houses/extra/covers-west-b.png`, `covers-east-a.png`). And a covering's colour multiplies its texture, so the near-white tint of "Whitewashed wood" over the mid-tone pine floor (`textures.png`, third tile) leaves it looking like the oak boards beside it (`houses/extra/floors-all.png`, the fourth wood square). A wallpaper texture with a faint stripe or print, and a pale wood texture (or a whitewash blend) for the whitewashed floor, fix both.
- **Status:** fixed (8e0f83c2)
- **After:** without a new texture: wallpaper is drawn with faint stripes 12.5 cm apart along the wall (`materials.ts` `PatternPlugin` `stripes`, over the wallpaper texture, fading out where they would be finer than a pixel or two), and "Whitewashed wood" has a finish of its own (`whitewash`: the floor wood limed pale, its boards still showing; `base.json`, `content.ts`, the swatch in `LookPicker.svelte`). The wallpaper bucket was already its own draw; a whitewashed floor is one more draw only in a house that has one. `fixes/town/covers/wallpaper-in.png`, `wallpaper-out-a.png`, `floors-whitewash.png` (oak, whitewashed, walnut, whitewashed). Note: the wave's `houses.mjs` sent covering indices 0-based where `paint` and `paintFloor` take them 1-based (0 is the automatic look), so its covering and floor shots show each covering one place later than their labels say; the finding stands (the wallpaper texture has no pattern).
- **Source:** wave: houses

### M-houses-05 — Two-storey neighbours' chimneys are built inside the house

- **Severity:** S3
- **Rubric:** M7, M13
- **Fix class:** code
- **Where:** `web/src/render/babylon/house.ts:1776` and `:1783` (`chimney`: from `WALL_HEIGHT` to one storey's ridge, with the 34° `PITCH`), against `:1612` and `:1679` (the shell's roof starts at `tall`, the storeys' height)
- **Evidence:** The chimney's height is worked out for a one-storey house. On a two-storey shell the roof's eaves are at 5.6 m, so a stack topping out at the first storey's ridge plus 0.85 m stands inside the house: none of the two-storey shells in the village has a chimney (`town/views/04-neighbour-4-sycamore-garage-h13.png`, `town/views/09-willow-street-h13.png`, `town/extra/views-05-08.png`), while about 60 % of the one-storey ones do (`house.ts:1680`) (`town/views/03-neighbour-2-sycamore-h13.png`). Passing the shell's base height and the scheme's pitch fixes it.
- **Status:** fixed (8e0f83c2)
- **After:** the chimney is worked out from the shell's own eaves (`tall`) and its roof's pitch (`chimney(…, base)`, `s.pitch`): two-storey neighbours get one too, standing 0.75 m over their ridge (`fixes/town/views/04-neighbour-4-sycamore-garage-h13.png`, top right, against `town/views/04-neighbour-4-sycamore-garage-h13.png`).
- **Source:** wave: houses

### M-houses-06 — Garages are too narrow for the cars parked in front of them

- **Severity:** S3
- **Rubric:** M18, M1
- **Fix class:** code
- **Where:** `web/src/render/babylon/street.ts:1131-1140` (`garage`: the driveway's width plus 0.3 m, at least 2.7 m), `web/src/render/babylon/house.ts:1690` (door: the garage's width less 0.7 m)
- **Evidence:** A garage here is 2.7–3.3 m wide outside (2.56–3.16 m inside) with a door 2.0–2.6 m wide; a real single garage is at least 3.0 m inside with a 2.3–2.5 m door (2.13 m at the least). The street's cars are 1.74–1.88 m wide, 2.0–2.1 m over their mirrors (`street.ts:205-214`, `:308`), so in the narrowest garages they wouldn't go through the door (`town/views/09-willow-street-h13.png`, the brown garage with the white estate in front). A floor of 3.2 m outside (door 2.5 m), or no garage where the drive is too narrow, keeps them believable.
- **Status:** fixed (76d38b46)
- **After:** a garage is at least `GARAGE_WIDTH` 3.14 m between its wall lines (3 m inside), moved off the drive's middle if it must to clear the house and the plot's edge, and not built where there is no room for that (`street.ts` `garage`); its door is 2.4-2.5 m, centred (`house.ts`). The white estate now has room to drive in (`fixes/town/views/13-garage-willow-h13.png`, `09-willow-street-h13.png`, against `town/views/09-willow-street-h13.png`); a narrow drive keeps its car and loses the garage (`04-neighbour-4-sycamore-garage-h13.png`). Renderer only: no lot moves.
- **Source:** wave: houses

### M-houses-07 — A stairwell railing's balusters stand 116 mm apart

- **Severity:** S3
- **Rubric:** M17, M3
- **Fix class:** code
- **Where:** `web/src/render/babylon/house.ts:1348` (balusters 24 mm thick every 0.14 m)
- **Evidence:** The gaps between balusters are 116 mm; a guard in a home lets no 100 mm sphere through (Approved Document K; IRC 4 in). Only visible close up (`houses/extra/stairs.png`, second tile). Every 0.12 m (96 mm gaps) is enough.
- **Status:** fixed (8e0f83c2)
- **After:** the balusters are spaced evenly between the posts, as many as keep the gaps under 100 mm: seven a metre, 92 mm apart (`house.ts` railing). Checked in code; the stairwell close-up wasn't taken again.
- **Source:** wave: houses

### M-houses-08 — The door wood texture and `material.door` are left unused

- **Severity:** S3
- **Rubric:** M13
- **Fix class:** code
- **Where:** `crates/tools/src/tex.rs:462` (`door_wood`, Wood066, `Op::Rot90`) and `:625` (`material.door`), `web/public/assets/manifest.json:1164`, `web/public/assets/textures/door_wood/`, `web/public/assets/CREDITS.md:29`
- **Evidence:** Since G-57 the oak door leaf is drawn on the oak panelling's grain (`material.finish.wood`); nothing in `web/src` names `material.door` any more, so the 1024 px door wood set is built and shipped for nothing (never loaded). Dropping it goes through the texture tool: its `HOUSE_SETS` entry and `material.door` (and the fixture `tests/fixtures/tex/materials.json`), then `Op::Rot90`, which only it uses.
- **Status:** open
- **Source:** fix: town

