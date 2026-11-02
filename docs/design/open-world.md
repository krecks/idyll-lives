# Open world: the shared contract

Bigger worlds (up to 1024×1024 m) with a main town, hamlets and usable countryside, joined by
country roads residents travel, and a camera that roams the whole world. The work is split
across four branches that merge into `feat/open-world` in this order:

1. `feat/open-world-sim-grid` — chunked tile storage, incremental reachability, bounded A*, caps, save v35, wasm views by chunk.
2. `feat/open-world-sim-travel` — road graph, router, ETA and commutes, cars on roads, nearby choices, outings, countryside content.
3. `feat/open-world-worldgen` — sizes Cosy/Normal/Big, the Big macro layout, the town file fields below, the size option, the pan/zoom town map, sample/map tools.
4. `feat/open-world-render` — view-independent chunked street, roaming camera, residents by distance, chunked terrain/woods/grass, streaming by distance.

This file is the interface between them. Change it only together (through the coordinator).

## Chunks

| What | Size | Owner |
|---|---|---|
| Sim tile storage | 16 × 16 tiles | sim-grid (`lot.rs`) |
| Sim spatial/object index cells | 64 m | sim-travel (`index.rs`) |
| Render stream chunks | 64 m | render (`render/shared/chunks.ts`) |
| Render far-LOD superchunks | 128 m | render |
| Terrain tiles | 128 m | render |

Chunk `(cx, cz)` covers tiles `cx*S .. cx*S+S`, `cz*S .. cz*S+S` in town (world) coordinates,
ground storey. Row blocks for upper storeys and the basement keep today's stacked-rows layout
(`storeys.rs`): storey `k` at rows `k*depth ..`; only their chunks are allocated lazily.

## Caps

- Town file: `width`, `depth` ≤ 1024. Saved grid: width ≤ 1024, depth ≤ 1024 × 5.
- `MAX_SIMS` 160, `MAX_STAFF` 48, `MAX_CARS` 128 (sim-grid). The renderer's `MAX_CHARACTERS`
  comes from `snapshot_layout` (maxSims + maxStaff = 208), not a constant.
- Neighbours per size: `neighbourCap('cosy' | 'normal') = 48`, `neighbourCap('big') = 140`.

## Sizes

| Size | Box | Lots | Sim change needed |
|---|---|---|---|
| `cosy` | today's recipe, unchanged | today's | none |
| `normal` | ≤ 256 m, ~1.6× cosy's box area | ~1.6× | none |
| `big` | 768–1024 m | main town (kind's shape at normal scale) + 2–4 hamlets/farmsteads, ~120 starting residents | all sim branches |

Saves and towns without `size` are `cosy` and must look and behave exactly as before: every
new land rule is gated on `size === 'big'`.

## Town file additions

All optional; absent means today's behaviour. Types in `web/src/core/protocol.ts`
(`OpenWorldTownFields`, `RoadPlan`, `WorldMeta`); the sim reads them in `lot.rs` `LotFile`.

- `size: 'cosy' | 'normal' | 'big'`
- `roads: RoadPlan[]` (top level, for the sim; same objects as `meta.roads`). `RoadPlan.kind`:
  `'street'` (default) · `'lane'` · `'country'` (between areas) · `'trail'` (2 m, foot only).
  `RoadPlan.area`: index into `areas` (absent for country roads and trails between areas).
  The sim builds its graph from the polylines: endpoints, plus joins snapped within
  `width / 2 + 1` m.
- `areas: { name, kind: 'hamlet' | 'countryside', x, z, w, d }[]` — area 0 is the main town.
- `plots[].area?: number`, `plots[].kind?: 'home' | 'farm' | 'public' | 'shop' | 'countryside'`.
  Countryside plots are public, unfenced, with an `entry` at their trail end.
- `exits: ([x, z] | { x, z, kind: 'road' | 'stop' })[]` — a `stop` is a bus stop inside an area
  that counts as an exit for people on foot.
- `terrain: { water?: Rle, blocked?: Rle }` — run-length masks over the ground grid in row-major
  order: `[runLength, value, runLength, value, ...]` with value 0/1, starting at tile (0, 0).
  The sim treats set tiles as a base blocked layer; the renderer draws water from `meta.zones`.
- `parking: { x, z, rot, area }[]` — public parking spots for car trips between areas.
- `drives[].road?: number`, `drives[].s?: number` — the road a driveway meets and the distance
  along its polyline (m).
- `meta.zones: { kind: 'woods' | 'meadow' | 'water' | 'hill' | 'fields', poly: [x, z][] }[]`
- `meta.pads: Record<plotIndex, y>` — the height an off-town outdoor plot is levelled to (m).
- `meta.world.size` mirrors `size` for the renderer.

## Worker protocol (sim worker ↔ main thread)

- `setView(rect)` — the focus lot: its wall/floor meshes and full structure, as today.
- `setWatch(rect)` — new: the camera's stream rectangle; `World::set_view` uses it for the
  sim's full-detail area (LOD). Defaults to the view rect when never sent.
- `rooms` is sent as a transferable `Uint16Array` (same indexing as today's `number[]`).
- Structure is versioned per sim chunk; `structure_rect(x0, z0, x1, z1)` returns the walls,
  doors and objects inside a rectangle, and `chunk_versions()` lists the versions to diff.

## Game feel

Walking is 1.5 m per game minute (`WALK_SPEED`), so 1 km takes ~11 game hours on foot.

- Hamlets are 250–450 m apart; trips between them are by car. The household setting off with
  the driver for somewhere the car parks near goes in it too (`Car::riders`, saved with the car
  as `cars[].riders`): it waits in the driveway for them, they ride there and get out at the
  parking spot, and when one of them goes home they all go back to it and home together.
  Renderer contract unchanged: the car is a snapshot car row as ever, and everyone in it (the
  driver, and now the passengers) is a resident sitting in their seat (`pos`, pose sit, not
  away), so no new snapshot field.
- Every house is within ~120 m of a stop or exit.
- Countryside spots are within ~150 m of their hamlet, reached by trail.
- `careerRules.commuteMinutes` means only the off-map part; the on-map part comes from the ETA.

## Life on every lot

Residents outdoors are drawn on every lot within the draw radius, and window glow follows who
is home. Full interiors are drawn for the focus lot (the one under the camera) plus 2 cached ones.

## Integration rules

- Each branch keeps every existing test passing and today's towns identical.
- New sim tests are separate files under `crates/sim-core/tests/`, written to move into
  `tests/sim/` unchanged once PR #43 lands.
- Before merging, run every `ci.yml` step locally.
- Big was behind the dev flag `?big=1` (UI) until all four branches were in; it is now offered on Create World.
