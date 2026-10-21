# Lighting: authenticity ledger

What it covers: the 21 objects of [docs/objects/lighting.md](../objects/lighting.md), in its groups and order.

- **Evidence:** `bench-results/review/lighting/` (gallery, clip, pose and play shots; shots stay out of git).
- **Pilot rows** (the 28 `base.json` objects and the residents, 2026-10-21): Shots are relative to `bench-results/review/pilot/` in the review worktree; how they were taken is in its `log.md`, what was learnt in [pilot-notes.md](pilot-notes.md).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.

## LT1. Ceiling lights (7)

Family: `tools/art/blender/families/lighting.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `ceilingPendant` Pendant lamp | — | — | — | | |
| `ceilingLight` Ceiling light | — | — | — | | |
| `chandelier` Ring chandelier | — | — | — | | |
| `pendantIndustrial` Industrial pendant | — | — | — | | |
| `ceilingFan` Ceiling fan | — | — | — | | |
| `trackLights` Track lights | — | — | — | | |
| `paperLantern` Paper lantern | — | — | — | | |

## LT2. Wall lights (3)

Family: `tools/art/blender/families/lighting.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `wallSconce` Wall sconce | — | — | — | | |
| `wallLampArm` Swing-arm wall lamp | — | — | — | | |
| `wallLightOutdoor` Bulkhead light | — | — | — | | |

## LT3. Floor lamps (5)

Family: `tools/art/blender/families/lighting.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `lamp` Floor lamp | OK | OK | OK | | `sheets/obj-lamp.png`, `play-night/lamp-no-uses.png`, `gallery-all/gallery-lamp.png` |
| `floorLampSimple` Simple floor lamp | — | — | — | | |
| `lampFloorArc` Arc floor lamp | — | — | — | | |
| `lampFloorTripod` Tripod floor lamp | — | — | — | | |
| `lampTorchiere` Uplighter | — | — | — | | |

## LT4. Table lamps (6)

Family: `tools/art/blender/families/lighting.py`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `tableLamp` Table lamp | — | — | — | | |
| `lampTableRound` Ginger-jar lamp | — | — | — | | |
| `lampTableSquare` Square table lamp | — | — | — | | |
| `lampBanker` Banker's lamp | — | — | — | | |
| `lampMushroom` Mushroom lamp | — | — | — | | |
| `lampLantern` Candle lantern | — | — | — | | |

## Real uses against the content's (I1)

Filled by the pilot for its rows.

| Item | Real uses | Content uses (minutes) | Verdict |
|---|---|---|---|
| `lamp` | switch it on and off, read by it | its switch in the pie menu (`session.ts:1163`); residents switch a room's lamps together at dusk and on leaving (`lights.rs`) | OK (the switch is its use) |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

Not tied to one row: M-lighting-01.

### M-lighting-01 — Removing a wall can leave a ceiling lamp outdoors

- **Severity:** S3
- **Rubric:** M4
- **Fix class:** code
- **Where:** `crates/sim-core/src/home.rs:475` (the "needs a ceiling" refusal)
- **Evidence:** Building lets a wall's removal leave a ceiling lamp outdoors (or something upstairs outside a room). It hangs there until the game is loaded again, which leaves it out and loses its value.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 "Ceiling lamps and building")
- **Source:** `docs/release-0.2.0-review.md` §4 (new, Z1), target 0.2.1, not in the 0.3.0 notes
