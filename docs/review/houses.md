# Houses: authenticity ledger

What it covers: the house parts built in code: walls, doors and windows, coverings, roofs, stairwells and storeys, and the neighbours' shells with their porches, chimneys and garages. Built by `web/src/render/babylon/house.ts` (door swing: `BabylonRenderer.swingDoors`); styles from the `base.json` lists. The staircase itself is a content object (`stairs`, [living.md](living.md)). Rubric M17 in particular.

- **Evidence:** `bench-results/review/houses/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## Walls (7)

`house.ts` `build`; `WALL_HEIGHT`, `PLINTH`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Straight wall | — | — | — | | |
| Diagonal wall | — | — | — | | |
| Half wall (`form` 1) | — | — | — | | |
| Wall height and storeys | — | — | — | | |
| Plinth and foundation | — | — | — | | |
| Basement walls (stone-faced earth) | — | — | — | | |
| Wall cut-away (lowered walls) | — | — | — | M-houses-01 S3 code (seeded) | |

## Doors (11)

`DOOR_HEIGHT` 2.1, `LEAF_W` 0.8; swing in `BabylonRenderer.ts` `swingDoors`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `panel` Painted panel | — | — | — | | |
| `panel.white` White panel | — | — | — | | |
| `panel.red` Red panel | — | — | — | | |
| `panel.blue` Blue panel | — | — | — | | |
| `oak` Natural oak | — | — | — | | |
| `halfGlass` Half-glazed | — | — | — | | |
| `glass` French glass | — | — | — | | |
| Front door swing | — | — | — | A-houses-01 S3 code (seeded) | |
| Bathroom door swing | — | — | — | A-houses-01 S3 code (seeded) | |
| A door beside a diagonal wall (part swing, or slides into the wall) | — | — | — | | |
| Door frames, sills and handles (all styles) | — | — | — | | |

## Windows (8)

`SILL` 0.9, `HEAD` 2.1; `muntins`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `classic` Classic | — | — | — | | |
| `cottage` Cottage | — | — | — | | |
| `picture` Picture | — | — | — | | |
| `tall` Floor-length | — | — | — | | |
| `high` High light | — | — | — | | |
| `transom` Transom | — | — | — | | |
| Window glass and sunlight through it | — | — | — | | |
| Windows glowing at night | — | — | — | | |

## Wall coverings (25)

`base.json` `wallCoverings`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `paint.white` White paint | — | — | — | | |
| `paint.cream` Cream paint | — | — | — | | |
| `paint.butter` Butter paint | — | — | — | | |
| `paint.sage` Sage paint | — | — | — | | |
| `paint.sky` Sky-blue paint | — | — | — | | |
| `paint.blush` Blush paint | — | — | — | | |
| `paint.terracotta` Terracotta paint | — | — | — | | |
| `paint.navy` Navy paint | — | — | — | | |
| `paint.charcoal` Charcoal paint | — | — | — | | |
| `wallpaper.linen` Linen wallpaper | — | — | — | | |
| `wallpaper.mint` Mint wallpaper | — | — | — | | |
| `wallpaper.lilac` Lilac wallpaper | — | — | — | | |
| `wallpaper.rose` Rose wallpaper | — | — | — | | |
| `siding.white` White siding | — | — | — | | |
| `siding.blue` Blue siding | — | — | — | | |
| `siding.green` Green siding | — | — | — | | |
| `siding.yellow` Yellow siding | — | — | — | | |
| `brick.red` Red brick | — | — | — | | |
| `brick.smoked` Smoked brick | — | — | — | | |
| `wood.oak` Oak panelling | — | — | — | | |
| `wood.walnut` Walnut panelling | — | — | — | | |
| `stone` Fieldstone | — | — | — | | |
| `tile.white` White tiles | — | — | — | | |
| `tile.blue` Blue tiles | — | — | — | | |
| `tile.green` Green tiles | — | — | — | | |

## Floor coverings (13)

`base.json` `floorCoverings`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `wood.oak` Oak boards | — | — | — | | |
| `wood.honey` Honey pine | — | — | — | | |
| `wood.walnut` Walnut boards | — | — | — | | |
| `wood.whitewashed` Whitewashed wood | — | — | — | | |
| `tile.white` White tiles | — | — | — | | |
| `tile.sand` Sand tiles | — | — | — | | |
| `tile.terracotta` Terracotta tiles | — | — | — | | |
| `tile.slate` Slate tiles | — | — | — | | |
| `carpet.oatmeal` Oatmeal carpet | — | — | — | | |
| `carpet.sage` Sage carpet | — | — | — | | |
| `carpet.blue` Blue carpet | — | — | — | | |
| `carpet.rose` Rose carpet | — | — | — | | |
| `stone.flag` Flagstone | — | — | — | | |

## Roofs (7)

`addRoof`, `PITCH` 34°, `OVERHANG` 0.4; `base.json` `roofStyles`, `roofColors`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `gable` Gable | — | — | — | | |
| `hip` Hip | — | — | — | | |
| `steepGable` Steep gable | — | — | — | | |
| `lowHip` Low hip | — | — | — | | |
| `flat` Flat | — | — | — | | |
| Roof colours (slate, brown, terracotta, moss, charcoal, taupe, navy, pearl) | — | — | — | | |
| Eaves, fascia and soffit | — | — | — | | |

## Stairwells and storeys (4)

`house.ts` (railing round each stairwell, landings).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Stairwell opening | — | — | — | | |
| Railing round a stairwell | — | — | — | | |
| Landings | — | — | — | | |
| Ceilings and floors between storeys | — | — | — | | |

## Neighbour shells (8)

`HouseBuilder.silhouettes`, `porch`, `chimney` (`house.ts`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Shell walls and roof | — | — | — | | |
| Shell windows, lit and curtained at night | — | — | — | | |
| Porch: gabled canopy over the front door | — | — | — | | |
| Chimney | — | — | — | | |
| Garage (`GARAGE_HEIGHT` 2.5) | — | — | — | | |
| Garage door facing the street | — | — | — | | |
| Porch and garage lamps | — | — | — | | |
| Shell front door | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### M-houses-01 — Wall things shown while one is held float over lowered walls

- **Severity:** S3
- **Rubric:** M4
- **Fix class:** code
- **Where:** `web/src/game/buildmode.ts`
- **Evidence:** While the player holds a wall thing, the ones already hung show at full height over walls the cut-away has lowered, pictures on outside walls too.
- **Status:** seeded (from `docs/objects/open.md` L134)
- **Source:** `docs/objects/open.md` Still open › Rendering

### A-houses-01 — House doors open by nearness, not by hand

- **Severity:** S3
- **Rubric:** A12, A10
- **Fix class:** code
- **Where:** `web/src/render/babylon/BabylonRenderer.ts:3778` (`swingDoors`)
- **Evidence:** Front and bathroom doors swing when someone walks near them; nobody reaches for the handle. Doors opened by hand are new behaviour: filed as `code`, planned separately (§9, decision 3).
- **Status:** seeded (from `plan/authenticity-review.md` §3.2 A12)
- **Source:** `plan/authenticity-review.md` §3.2 (A12) and §9 (decision 3)
