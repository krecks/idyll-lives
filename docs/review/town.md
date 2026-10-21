# Town: authenticity ledger

What it covers: what the town is drawn with outside the content objects: roads and pavements, cars, street props, fences and shops built in code (`street.ts`, `roads.ts`, `fences.ts`), the 9 `model.street.*` models, the 4 ground models and the hedges and bushes no content object names (`nature.ts`, `street.ts` `FOLIAGE`). Public places' own objects (benches, stalls, shops) are content: [community.md](community.md). Rubric M18 in particular.

- **Evidence:** `bench-results/review/town/` (gallery, clip, pose and play shots; shots stay out of git).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Never looked at before** (`docs/objects/open.md` "Not looked at"): the town overview at full night; far versions in the town overview.

## Roads and pavements (7)

`street.ts` (`SIDEWALK`, `KERB`), `roads.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Street road surface | — | — | — | | |
| Lane markings | — | — | — | M-town-03 S3 code (seeded) | |
| Pavements and kerbs | — | — | — | | |
| Driveways | — | — | — | | |
| Country roads out of town | — | — | — | | |
| Side roads between streets | — | — | — | | |
| "Welcome to …" sign | — | — | — | | |

## Cars (4)

`street.ts` `CARS`, `car()`; paints `CAR_PAINT`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Saloon (4.4 × 1.78 m) | — | — | — | | |
| Hatchback (4.0 × 1.74 m) | — | — | — | | |
| SUV (4.6 × 1.88 m) | — | — | — | | |
| Estate (4.6 × 1.8 m) | — | — | — | | |

## Street props built in code (4)

`street.ts` `streetlight`, `mailbox`, `hydrant`, `bin`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Streetlight | — | — | — | M-town-04 S3 code (seeded) | |
| Mailbox | — | — | — | M-town-04 S3 code (seeded) | |
| Hydrant | — | — | — | M-town-04 S3 code (seeded) | |
| Wheelie bin | — | — | — | M-town-04 S3 code (seeded) | |

## Fences (7)

`fences.ts` (`base.json` `fenceStyles`); lot edges in `street.ts` (`pickets`, `rails`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `picket` White picket | — | — | — | | |
| `rails` Ranch rails | — | — | — | | |
| `slats` Modern slats | — | — | — | | |
| `iron` Wrought iron | — | — | — | | |
| `stone` Low stone wall | — | — | — | | |
| Gates | — | — | — | | |
| Lot-edge pickets, rails and hedges along the street | — | — | — | | |

## Public places seen from another lot (2)

`street.ts` (`AWNINGS`, stand-ins).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Shop silhouettes with awnings | — | — | — | | |
| Park, playground and market things standing in the open | — | — | — | | |

## Street models (9)

Family: `tools/art/blender/families/street.py`; drawn by `street.ts` `PROP`, `FOLIAGE`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `model.street.bench` | — | — | — | | |
| `model.street.planter` | — | — | — | | |
| `model.street.flowerTub` | — | — | — | | |
| `model.street.sign` | — | — | — | | |
| `model.street.busSign` | — | — | — | | |
| `model.street.pot` | — | — | — | | |
| `model.street.pathLight` | — | — | — | | |
| `model.street.hedge` | — | — | — | M-town-02 S3 decision (seeded) | |
| `model.street.flowers` | — | — | — | | |

## Ground models (4)

Family: `tools/art/blender/families/ground.py`; drawn by `nature.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `model.grassTuft` | — | — | — | M-town-01 S3 code (seeded) | |
| `model.flowers` | — | — | — | M-town-01 S3 code (seeded) | |
| `model.rock` | — | — | — | M-town-01 S3 code (seeded) | |
| `model.marker` | — | — | — | | |

## Hedges and bushes not in content (3)

Family: `tools/art/blender/families/plants.py`; drawn by `nature.ts` and `street.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `model.bush.small` | — | — | — | | |
| `model.bush.flowering` | — | — | — | | |
| `model.hedge` | — | — | — | | |

## Landscape motion (2)

`nature.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Wind sway of plants and trees | — | — | — | A-town-01 S3 code (seeded) | |
| Clouds | — | — | — | | |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### M-town-01 — The ground's far versions are built but never used

- **Severity:** S3
- **Rubric:** M12
- **Fix class:** code
- **Where:** `web/src/render/babylon/nature.ts:580`
- **Evidence:** `nature.ts` never asks for the `.far` versions of the grass tufts, wildflowers and rocks.
- **Status:** seeded (from `docs/objects/open.md` L138)
- **Source:** `docs/objects/open.md` Still open › Rendering

### M-town-02 — `model.street.hedge` is built but unused

- **Severity:** S3
- **Rubric:** M13
- **Fix class:** decision
- **Where:** `web/src/render/babylon/street.ts:71` (`FOLIAGE.hedge` is `model.hedge`)
- **Evidence:** The street draws its hedges from `model.hedge`; the street pack's own hedge is never drawn. Use it or drop it.
- **Status:** seeded (from `docs/objects/open.md` L138)
- **Source:** `docs/objects/open.md` Still open › Rendering

### M-town-03 — A far road's lane markings change brightness between loads

- **Severity:** S3
- **Rubric:** M10
- **Fix class:** code
- **Where:** `web/src/render/babylon/roads.ts`
- **Evidence:** A far road's lane markings render brighter or darker between page loads (0.03–0.12 % of pixels). Proposed: thaw or refreeze the shared landscape materials on the switch from the overview to the game.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 P1-17)
- **Source:** `docs/release-0.2.0-review.md` §4 (P1-17); `plan/release-0.3.0.md` §3.3 (likely defer)

### M-town-04 — Plain finishes on code-built models may all show as rough metal

- **Severity:** S3
- **Rubric:** M9
- **Fix class:** code
- **Where:** `web/src/render/babylon/models.ts:460` (`finish`, `encodeFinish`), `web/src/render/babylon/street.ts`
- **Evidence:** The finish is carried in the vertex colour's alpha, but meshes with `hasVertexAlpha` false read an alpha of 1 (the cozy fridge's cream showed as khaki in the 0.2 look study). The street sets it false on its plain meshes on purpose. The Blender models don't use this path; the code-built street props, cars and fallbacks still do. Not checked in play.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 "Plain finishes")
- **Source:** `docs/release-0.2.0-review.md` §4 (new, look study), target 0.2.1 or v0.3

### A-town-01 — The household creator's preview sways its plants with reduce motion on

- **Severity:** S3
- **Rubric:** A11
- **Fix class:** code
- **Where:** `web/src/render/preview/stage.ts`, `web/src/render/babylon/nature.ts`
- **Evidence:** R1-15 stilled the scene's decorative movement with reduce motion, except in the creator's preview; and a change of the system setting reaches the renderer only at the next settings change.
- **Status:** seeded (from `docs/release-0.2.0-review.md` §4 R1-15 (rest))
- **Source:** `docs/release-0.2.0-review.md` §4 (R1-15 rest), target 0.2.1, not in the 0.3.0 notes
