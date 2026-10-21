# Town: authenticity ledger

What it covers: what the town is drawn with outside the content objects: roads and pavements, cars, street props, fences and shops built in code (`street.ts`, `roads.ts`, `fences.ts`), the 9 `model.street.*` models, the 4 ground models and the hedges and bushes no content object names (`nature.ts`, `street.ts` `FOLIAGE`). Public places' own objects (benches, stalls, shops) are content: [community.md](community.md). Rubric M18 in particular.

- **Evidence:** `bench-results/review/town/` (gallery, close-ups, `views/` of the bench village by day and night, `extra/`: contact sheets of the views and the gallery, fences and a gate walked through on `review-stage`; shots stay out of git). The house-style runs that also show fences are in `houses/extra/` (`houses.mjs`, `houses.jsonl`).
- **Rubric:** [plan/authenticity-review.md §3](../../plan/authenticity-review.md#3-rubrics); real-world ranges in [reference.md](reference.md). Finding format and severities: [README.md](README.md#writing-a-finding).
- **Columns:** M, A and I take `OK`, a finding id or `n/a`; `—` is pending. Findings lists the ids filed against the row, with severity and fix class; Shots names the evidence files.
- **Seeded findings** (`Status: seeded`) are known issues copied in from earlier docs. Confirm or close them; don't file them again.
- **Never looked at before** (`docs/objects/open.md` "Not looked at"): the town overview at full night (now seen: `views/11-overview-h22.png`, streetlight pools and lit windows read right); far versions in the town overview (the street's models' far versions are drawn there, `street.ts:1197`; not compared side by side).
- **Wave: town** (2026-10-21). Everything here is scenery: residents use none of it (the street is drawn outside the lots, `street.ts:13-19`), so I is `n/a` unless a use is shown (lamps lighting up) or one would be expected and the miss shows.

## Roads and pavements (7)

`street.ts` (`SIDEWALK`, `KERB`), `roads.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Street road surface | M-town-05 | n/a | n/a | M-town-05 S2 decision: 3.3 m between the kerbs with a centre line | `town/views/02-street-cars-h13.png`, `town/views/09-willow-street-h13.png` |
| Lane markings | M-town-05, M-town-03 | n/a | n/a | M-town-03 S3 code (seeded; not checked); dashes 2.4 m every 5 m, 12 cm wide (`street.ts:754`): stylised, fine; they split a 3.3 m road into two 1.65 m lanes (M-town-05) | `town/views/02-street-cars-h13.png` |
| Pavements and kerbs | M-town-07 | n/a | n/a | M-town-07 S3 decision: the kerb stands 3.3 cm over the road. Sidewalk 1.21 m plus a 0.14 m kerb: OK | `town/views/05-kerb-mailbox-hydrant-bin-h13.png` |
| Driveways | OK | n/a | n/a | 2.3–3.0 m wide, from the street to 5.4 m past the house front, 0.3–0.6 m off its wall (`town.ts:582-596`): real single drives are 2.4–3 m | `town/views/04-neighbour-4-sycamore-garage-h13.png`, `town/views/09-willow-street-h13.png` |
| Country roads out of town | OK | n/a | n/a | 6 m at the town's end narrowing to 5 m (`roads.ts:18-19`), a country lane's width; seen leaving the street's end (code, `views/10`) | `town/views/10-streetlights-along-h13.png` |
| Side roads between streets | OK | n/a | n/a | As the country roads, 6 m (`roads.ts:18`); code only: the village has one street | none (code only) |
| "Welcome to …" sign | OK | n/a | n/a | A 3 × 1.1 m green board on two posts, 1.0–2.1 m up, 1.6 m off the road's edge on the arrivals' right (`roads.ts:219-275`): code only, not in a view | none (code only) |

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| Road | drive along it; cross it | residents walk out of town along it at the street's ends; no traffic | n/a (scenery) |
| Pavements | walk along them | walked by residents leaving town | OK |
| Driveways | park the car; walk to the door | a parked car on most (`street.ts:775`) | OK |

## Cars (4)

`street.ts` `CARS`, `car()`; paints `CAR_PAINT`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Saloon (4.4 × 1.78 m) | M-town-06 | n/a | n/a | 4.4 × 1.78 × 1.42 m, wheels 0.64 m: in the range (saloon 4.4–4.95 × 1.75–1.87 × 1.4–1.5); M-town-06 S3 code: the body's underside 0.30 m up | `town/views/03-neighbour-2-sycamore-h13.png` |
| Hatchback (4.0 × 1.74 m) | M-town-06 | n/a | n/a | 4.0 × 1.74 × 1.46, wheels 0.62: in the range; M-town-06 | `town/views/02-street-cars-h13.png` |
| SUV (4.6 × 1.88 m) | M-town-06 | n/a | n/a | 4.6 × 1.88 × 1.76, wheels 0.74: in the range (h within the 10 %); M-town-06 S3 code: underside 0.40 m | `town/views/01-home-front-resident-h13.png` |
| Estate (4.6 × 1.8 m) | M-town-06 | n/a | n/a | 4.6 × 1.8 × 1.45, wheels 0.64: in the range; M-town-06 | `town/views/09-willow-street-h13.png` |

Phase 0's "car parked against the house wall" (`views/04`): closed. The car stands on its driveway, centred, with 0.37 m from its mirror to the wall face at the least (`town.ts:596`, `street.ts:775-781`); from that camera the house hides its near side. A door couldn't open on that side, but it is a parked prop. A is `n/a`: cars never move, and their doors and boot don't open.

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| Cars (all four) | drive to work and the shops; load and unload; park on the drive | parked on neighbours' drives for good; residents walk out of town | n/a (scenery; no driving in the game's design) |

## Street props built in code (4)

`street.ts` `streetlight`, `mailbox`, `hydrant`, `bin`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Streetlight | OK | OK | OK | 4.16 m cast-iron lantern post, lantern at 3.5–3.86 m (`street.ts:324-339`): in the range (3–8 m), reads at scale beside residents; lit from dusk with a pool of light 4.2 m across (`setNight`). M-town-04 closed | `town/views/10-streetlights-along-h13.png`, `town/views/10-streetlights-along-h22.png`, `town/extra/views-night.png` |
| Mailbox | OK | n/a | n/a | US kerbside box on a 1 m post, box 0.2 × 0.22 × 0.48 m, top 1.22 m, red flag on the side (`street.ts:342-349`): in the range; door towards the road. M-town-04 closed | `town/views/01-home-front-resident-h13.png`, `town/views/03-neighbour-2-sycamore-h13.png` |
| Hydrant | OK | n/a | n/a | 0.71 m pillar, red or yellow, caps and nozzles (`street.ts:351-361`): in the range. M-town-04 closed | `town/views/02-street-cars-h13.png`, `town/views/09-willow-street-h13.png` |
| Wheelie bin | OK | n/a | n/a | 0.58 × 1.03 × 0.69 m, wheels and handle at the hinge side (`street.ts:364-371`): in the range (240 L). It stands 0.05–0.75 m into the 1.21 m walk (`street.ts:694`), across its middle; whether residents walk through it was not seen. M-town-04 closed | `town/views/04-neighbour-4-sycamore-garage-h13.png`, `town/views/09-willow-street-h13.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| Streetlight | light the pavement at night | lights at dusk, pools of light on the ground | OK |
| Mailbox | post and collect letters | none: no post in the game | n/a (scenery) |
| Hydrant | firefighters connect a hose | none | n/a |
| Wheelie bin | put the rubbish out; wheel it to the kerb | none: the home's `trashCan` takes the rubbish | n/a (scenery) |

## Fences (7)

`fences.ts` (`base.json` `fenceStyles`); lot edges in `street.ts` (`pickets`, `rails`).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `picket` White picket | OK | n/a | OK | 1.0 m, pointed pickets every 0.15 m on two rails (`fences.ts:79-86`): a picket fence's height | `town/extra/fences-all.png` |
| `rails` Ranch rails | OK | n/a | OK | 1.1 m, three rails | `town/extra/fences-all.png` |
| `slats` Modern slats | OK | n/a | OK | 1.4 m horizontal slats with gaps | `town/extra/fences-all.png` |
| `iron` Wrought iron | OK | n/a | OK | 1.2 m bars every 0.1 m with finials | `town/extra/fences-all.png` |
| `stone` Low stone wall | OK | n/a | OK | 0.7 m dry-stone wall with a cap, 0.32 m thick | `town/extra/fences-all.png` |
| Gates | OK | A-town-02 | A-town-02 | A-town-02 S2 code: gates never open; residents walk through the shut leaf. Fixed (e04bc6c0): the leaf swings open for the one going through (`fixes/town/doors/gate-in-1.png`). The leaf (0.82 m, braced from the hinge foot to the latch head) is right | `town/extra/sheet-fences.png`, `town/extra/gate-walk-1.png` |
| Lot-edge pickets, rails and hedges along the street | OK | n/a | n/a | Pickets 0.92–0.98 m, farm rails 1.08 m, hedges from `model.hedge` 0.87–1.07 m (`street.ts:374-409`, `:1162-1170`); from the house front line back only | `town/views/08-back-gardens-fences-h13.png`, `town/extra/views-05-08.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| Fences | mark the lot; keep children and pets in or out | block the way like a wall (they make no rooms) | OK |
| Gate | open it, walk through, shut it behind you | walked through, shut | A-town-02 |

## Public places seen from another lot (2)

`street.ts` (`AWNINGS`, stand-ins).

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Shop silhouettes with awnings | OK | n/a | n/a | Striped awning 2.55 m at the wall to 2.2 m at 0.95 m out, valance to 2.04 m (`street.ts:879-897`): clear of a resident's head, over the shop windows; a porch and lamp like the houses' | `town/views/06-bakery-shop-h13.png`, `town/views/06-bakery-shop-h22.png` |
| Park, playground and market things standing in the open | OK | n/a | n/a | The content's own models at their places (community ledger judges them) | `town/views/07-park-playground-h13.png`, `town/views/09-willow-street-h13.png` |

## Street models (9)

Family: `tools/art/blender/families/street.py`; drawn by `street.ts` `PROP`, `FOLIAGE`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `model.street.bench` | OK | n/a | n/a | 1.70 × 0.91 × 0.51; seat slats' top 0.45 m (`street.py:63-83`, in the 0.42–0.48 range; the sizes table's "no seat to measure" is because no content seat names it), arms 0.21 m over the seat. On the kerb facing the public place, back to the road. Nobody sits on it (scenery) | `town/sheets/obj-street.bench.png`, `town/views/06-bakery-shop-h13.png` |
| `model.street.planter` | OK | n/a | n/a | 0.70 m concrete tub, 0.42 high, young tree to 2.42 m | `town/sheets/obj-street.planter.png`, `town/extra/town-gallery-a.png` |
| `model.street.flowerTub` | OK | n/a | n/a | the same tub with a flower mound, 0.57 m | `town/sheets/obj-street.flowerTub.png` |
| `model.street.sign` | OK | n/a | n/a | 2.66 m pole, two 0.9 m blades at 2.25 and 2.45 m: in the range; set 45° on the corner | `town/extra/town-gallery-a.png` |
| `model.street.busSign` | OK | n/a | n/a | 2.63 m, 0.46 m flag at 2.4 m, timetable at 1.5 m (eye height) | `town/sheets/obj-street.busSign.png` |
| `model.street.pot` | OK | n/a | n/a | terracotta pot 0.4 × 0.35 m with a clipped ball, 0.87 m in all; either side of front doors | `town/sheets/obj-street.pot.png`, `town/views/03-neighbour-2-sycamore-h13.png` |
| `model.street.pathLight` | OK | OK | OK | 0.56 m bollard, lens glows at night with the streetlights (`street.ts:1083-1087`) | `town/sheets/obj-street.pathLight.png`, `town/extra/views-night.png` |
| `model.street.hedge` | M-town-02 | n/a | n/a | M-town-02 S3 decision (seeded; confirmed): never drawn | `town/sheets/obj-street.hedge.png` |
| `model.street.flowers` | OK | OK | n/a | 0.59 × 0.34 × 0.53 clump in the beds along house fronts; sways on the foliage finish | `town/sheets/obj-street.flowers.png`, `town/views/04-neighbour-4-sycamore-garage-h13.png` |

### Real uses against the content's (I1)

| Item | Real uses | Content's | Verdict |
|---|---|---|---|
| `model.street.bench` | sit down; wait; meet someone | none: kerb dressing outside the lots (the community `bench` has the uses) | n/a (decor) |
| `model.street.planter`, `flowerTub`, `pot`, `flowers`, `hedge` | look nice; be watered and trimmed | none | n/a (decor) |
| `model.street.sign` | read the street name | none | n/a (decor) |
| `model.street.busSign` | check the timetable; wait for the bus | none; the bus shelter (`model.busStop`) beside it is scenery too | n/a (decor) |
| `model.street.pathLight` | light the garden path at night | glows at night | OK |

## Ground models (4)

Family: `tools/art/blender/families/ground.py`; drawn by `nature.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `model.grassTuft` | M-town-01 | OK | n/a | 0.12 × 0.29 × 0.14 tuft; M-town-01 S3 code (seeded; confirmed: drawn at full detail everywhere) | `town/sheets/obj-grassTuft.png`, `town/extra/town-gallery-b.png` |
| `model.flowers` | M-town-01 | OK | n/a | 0.27 m wildflower; M-town-01 S3 code (confirmed) | `town/sheets/obj-flowers.png`, `town/extra/town-gallery-b.png` |
| `model.rock` | OK | n/a | n/a | 1.14 × 0.55 × 0.83 boulder, scaled 0.6–1.3; its far version is drawn (M-town-01 closed for the rock) | `town/sheets/obj-rock.png` |
| `model.marker` | OK | OK | OK | selection star 0.23 × 0.34 (a UI marker, in the 0.2–0.4 range); floats over the selected resident facing the camera with a slow bob, still with reduce motion (`BabylonRenderer.ts:4508-4513`) | `town/sheets/obj-marker.png` |

## Hedges and bushes not in content (3)

Family: `tools/art/blender/families/plants.py`; drawn by `nature.ts` and `street.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| `model.bush.small` | OK | OK | n/a | 0.83 × 0.71 × 0.78 clipped shrub in the woods (`nature.ts:494`) | `town/sheets/obj-bush.small.png` |
| `model.bush.flowering` | OK | OK | n/a | 1.21 × 1.02 × 1.27 with blooms, in gardens' dressing (`nature.ts:881`) | `town/sheets/obj-bush.flowering.png` |
| `model.hedge` | OK | OK | n/a | 1.55 × 1.02 × 0.85 block, stretched along each run (`street.ts:1162-1170`); the street's hedges and field hedgerows | `town/sheets/obj-hedge.png`, `town/views/08-back-gardens-fences-h13.png` |

## Landscape motion (2)

`nature.ts`.

| Item | M | A | I | Findings | Shots |
|---|---|---|---|---|---|
| Wind sway of plants and trees | n/a | A-town-01 | n/a | A-town-01 S3 code (seeded; not checked). The sway itself: a bend of 0.0022 m per m² of height (22 cm at the top of a 10 m tree) with gusts, periods of 3–6 s, leaves fluttering 1.2 cm, on real time, not game speed (`nature.ts:900-990`): believable | — (motion; stills can't show it) |
| Clouds | n/a | OK | n/a | A cloud band 3–32° above the horizon drifting slowly; still with reduce motion (`BabylonRenderer.ts:352`) | `town/views/10-streetlights-along-h13.png` |

## Findings

Seeded from earlier docs first; add new findings after them in the same form.

### M-town-01 — The ground's far versions are built but never used

- **Severity:** S3
- **Rubric:** M12
- **Fix class:** code
- **Where:** `web/src/render/babylon/nature.ts:580-582` (the lawn's `model.grassTuft` and `model.flowers` are instanced at `lod` 0 whatever the distance)
- **Evidence:** `nature.ts` never asks for the `.far` versions of the grass tufts and wildflowers. Wave town: the rock's far version is drawn: rocks always go to the far bucket (`nature.ts:505`, `close` leaves rocks out) and that bucket is built at `lod` 1 (`nature.ts:517`, `models.ts:248`).
- **Status:** fixed (13ab25bf)
- **After:** on Low (`streetLod` 1) the lawn's grass tufts and wildflowers are drawn from their far versions, as the street's models are (`nature.ts` `lawnLod`); Medium and High keep the full ones. The bench village on Low, same draws (orbit 280-283 against 278, the difference the shadow and mirror redraws that come and go; wide 323.4 against 323.5), triangles 0.94-0.99 M against 1.13 M in the orbit and 1.18 M against 1.51 M wide (`fixes/town/counts-before.jsonl`, `counts-after.jsonl`; headless, counts only).
- **Source:** `docs/objects/open.md` Still open › Rendering; wave: town

### M-town-02 — `model.street.hedge` is built but unused

- **Severity:** S3
- **Rubric:** M13
- **Fix class:** decision
- **Where:** `web/src/render/babylon/street.ts:71` (`FOLIAGE.hedge` is `model.hedge`)
- **Evidence:** The street draws its hedges from `model.hedge`; the street pack's own hedge is never drawn. Use it or drop it. Wave town: no file under `web/src` names `model.street.hedge`; the two look alike (1.28 × 1.02 × 0.63 against 1.55 × 1.02 × 0.85, `town/extra/town-gallery-a.png`), so dropping it loses nothing.
- **Status:** fixed (13ab25bf): dropped (Q-09)
- **After:** the street pack's own hedge is gone from `families/street.py`, `packs/street/manifest.json`, `models/street/` (`hedge.glb`, `hedge.far.glb`), `centres.json` and `reference.json`; the street's hedges are `model.hedge` (plants pack) as before, so nothing drawn changes.
- **Source:** `docs/objects/open.md` Still open › Rendering; wave: town

### M-town-03 — A far road's lane markings change brightness between loads

- **Severity:** S3
- **Rubric:** M10
- **Fix class:** code
- **Where:** `web/src/render/babylon/roads.ts`
- **Evidence:** A far road's lane markings render brighter or darker between page loads (0.03–0.12 % of pixels). Proposed: thaw or refreeze the shared landscape materials on the switch from the overview to the game.
- **Status:** not checked (needs the same view over several page loads compared pixel by pixel; the village views were taken once)
- **Source:** `docs/release-0.2.0-review.md` §4 (P1-17); `plan/release-0.3.0.md` §3.3 (likely defer)

### M-town-04 — Plain finishes on code-built models may all show as rough metal

- **Severity:** S3
- **Rubric:** M9
- **Fix class:** code
- **Where:** `web/src/render/babylon/models.ts:460` (`finish`, `encodeFinish`), `web/src/render/babylon/street.ts`
- **Evidence:** The finish is carried in the vertex colour's alpha, but meshes with `hasVertexAlpha` false read an alpha of 1 (the cozy fridge's cream showed as khaki in the 0.2 look study). The street sets it false on its plain meshes on purpose. Wave town: no longer so. `PlainFinishPlugin` passes the colour's alpha to the fragment in a varying of its own, not Babylon's `vColor.a` (`web/src/render/babylon/materials.ts:505-545`), so the street's props (`street.ts:925`, `:934`) and the fences (`fences.ts:50-52`) keep their finishes. By day the white car and the white mailboxes read white, the hydrants red, the iron fence dark grey (`town/views/03-neighbour-2-sycamore-h13.png`, `town/views/09-willow-street-h13.png`, `town/extra/fences-all.png`).
- **Status:** closed (fixed: `materials.ts:505-545` reads the finish from its own varying)
- **Source:** `docs/release-0.2.0-review.md` §4 (new, look study), target 0.2.1 or v0.3; wave: town

### A-town-01 — The household creator's preview sways its plants with reduce motion on

- **Severity:** S3
- **Rubric:** A11
- **Fix class:** code
- **Where:** `web/src/render/preview/stage.ts`, `web/src/render/babylon/nature.ts`
- **Evidence:** R1-15 stilled the scene's decorative movement with reduce motion, except in the creator's preview; and a change of the system setting reaches the renderer only at the next settings change. Wave town: the preview's item scene installs the wind (`web/src/render/preview/itemScene.ts:34`) and nothing there calls `setWind` (only `BabylonRenderer.ts:1379` does); not seen in a running preview.
- **Status:** not checked (needs the creator's preview running with reduce motion on; code points the same way)
- **Source:** `docs/release-0.2.0-review.md` §4 (R1-15 rest), target 0.2.1, not in the 0.3.0 notes; wave: town

### M-town-05 — The street is 3.3 m between the kerbs, with a centre line

- **Severity:** S2
- **Rubric:** M18, M1
- **Fix class:** decision
- **Where:** `web/src/game/town.ts:182` (`STREET = 6`, the whole street tile row), `web/src/render/babylon/street.ts:47` (`SIDEWALK = 1.35` each side), `street.ts:754-757` (the dashed centre line)
- **Evidence:** 6 m less two sidewalks of 1.35 m leaves a carriageway of 3.3 m, which the centre line splits into two lanes of 1.65 m: narrower than any of the street's cars (1.74–1.88 m, `street.ts:205-214`). A real two-way residential street is 5.5–7.3 m between kerbs, a lane 2.75–3.5 m; 3.3 m is one lane. In the views the road reads as a lane next to the houses and cars (`town/views/02-street-cars-h13.png`, `town/views/09-willow-street-h13.png`, `town/extra/views-overview.png`). Options: widen the street row to about 8.5–9 m (moves every lot of every town, so the owner's call), or keep 6 m and drop the centre line so it reads as a one-lane mews.
- **Status:** fixed (76d38b46): the centre line dropped (Q-13); the street keeps its 6 m
- **After:** without the dashed line the 3.3 m between the kerbs reads as one lane, a mews (`street.ts`; `fixes/town/views/02-street-cars-h13.png`, `09-willow-street-h13.png`, against `town/views/02-street-cars-h13.png`). Widening the street to 8.5-9 m would move every lot of every town: left for a new town layout, as Q-13 says. The country roads past the town (`roads.ts`, 5-6 m) keep their centre lines.
- **Source:** wave: town

### M-town-06 — The cars ride high: 0.30 m (SUV 0.40 m) under the body

- **Severity:** S3
- **Rubric:** M3
- **Fix class:** code
- **Where:** `web/src/render/babylon/street.ts:207-213` (`CARS[].low`: each side profile's lowest points are at y 0.3, the SUV's at 0.4)
- **Evidence:** The body's underside, between the wheels, is 0.30 m off the ground on the saloon, hatchback and estate and 0.40 m on the SUV; real ground clearance is 0.13–0.16 m for a saloon and 0.18–0.22 m for a compact SUV, sills a little higher. With wheels 0.62–0.74 m across the cars look jacked up from low angles (`town/views/03-neighbour-2-sycamore-h13.png`, the beige car; `town/views/09-willow-street-h13.png`, the white estate). Lowering the profiles' bottom edge to about 0.18 m (SUV 0.24 m) keeps the wheel arches.
- **Status:** fixed (76d38b46)
- **After:** the sills are at 0.17 m (the SUV's 0.23 m), the bumpers and lights staying by the wheels' height (`street.ts` `CARS`, `car`): `fixes/town/views/13-garage-willow-h13.png` (the white estate), against `town/views/09-willow-street-h13.png`.
- **Source:** wave: town

### M-town-07 — The kerb stands 3.3 cm over the road

- **Severity:** S3
- **Rubric:** M1, M18
- **Fix class:** decision
- **Where:** `web/src/render/babylon/street.ts:49-51` (`WALK_Y` 0.03, `KERB_Y` 0.045, `ROAD_Y` 0.012)
- **Evidence:** A real kerb's upstand is 10–15 cm (UK 125 mm, US 150 mm); here it is 3.3 cm, with the walk 1.5 cm below the kerb's top, so the kerb reads as a white painted strip rather than a step (`town/views/05-kerb-mailbox-hydrant-bin-h13.png`). Residents walk the street at ground level, so a taller kerb would sink their feet into the sidewalk unless the walkers are lifted onto it: a decision between a real step and keeping it flat.
- **Status:** won't fix (Q-13: the kerb stays flat)
- **After:** the owner kept the flat kerb: a real 10-15 cm step would sink the residents' feet into the sidewalk unless walkers were lifted onto it, and the sim walks them at ground level.
- **Source:** wave: town

### A-town-02 — moved to systems.md

### M-town-08 — Surfaces alike but for planar UVs may share one shader

- **Severity:** S3
- **Rubric:** M9
- **Fix class:** code
- **Where:** `web/src/render/babylon/materials.ts:70-104` (`VertexTweaksPlugin`: `planarUV` and `srgbColors` change its code but it declares no defines)
- **Evidence:** Babylon caches an effect by shader name and defines. A plugin whose code differs between materials without a define of its own gets one material's shader reused for another with the same defines: G-57's `PatternPlugin` showed it (its stripes never appeared until it had `PATTERN_*` defines). `VertexTweaksPlugin` is built that way, so a surface with planar UVs (roads, the ground: `projection` `xz`) and one without, with otherwise the same defines, could draw with each other's UVs. Not seen in a view; found reading the code.
- **Status:** not checked (needs two such materials in one scene, and a look at which shader each draws with)
- **Source:** fix: town

