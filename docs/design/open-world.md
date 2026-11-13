# Open world: the shared contract

Bigger worlds (up to 1024×1024 m) with a main town, hamlets and usable countryside, joined by
country roads residents travel, and a camera that roams the whole world. The work is split
across four branches that merge into `feat/open-world` in this order:

1. `feat/open-world-sim-grid` — chunked tile storage, incremental reachability, bounded A*, caps, save v35, wasm views by chunk.
2. `feat/open-world-sim-travel` — road graph, router, ETA and commutes, cars on roads, nearby choices, outings, countryside content.
3. `feat/open-world-worldgen` — sizes Cosy/Normal/Big (Cosy and Normal were dropped in 0.8.0: every world is Big now), the Big macro layout, the town file fields below, the pan/zoom town map, sample/map tools.
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
- `MAX_SIMS` 240, `MAX_STAFF` 48, `MAX_CARS` 128 (sim-grid). The renderer's `MAX_CHARACTERS`
  is the snapshot layout's maxSims + maxStaff (288, `render/shared/characters/rig.ts`; a test holds
  them together); the speech bubbles follow the layout's `maxSims`.
- Neighbours: `NEIGHBOUR_CAP = 220` in every world; a world starts with ~120 (`BIG_START = 118`:
  the rest of its homes are for sale) and grows to `MAX_SIMS`. `MAX_SIMS` and `NEIGHBOUR_CAP` were
  raised from 160 / 140 in the living-town run (2026-11-12), once the two costs that had held them
  down were gone. `BIG_START` was tried at 168 and put back: with ~170 residents from the first
  morning in the denser town, one of seed base 0's 24 sample worlds (mountain 209466) jammed at its
  centre by 11:00 on day 1 (a cyclist getting off by the Bakery and a car arriving there waited for
  each other for good, the town's taxis queued behind them, and 11 residents were on their bikes
  for over 2 hours: `generated_worlds_cars_keep_to_the_roads`); with 118 the same world and seed
  bases 0, 17 and 22 are green. It can go up once in-view jams are finished (`BIG_PREVIEW`).
  - **The 09:30 spike.** A real Big game with ~170 had 16 frames over 33 ms (up to 299 ms) as the
    town got its jobs at 09:30. The cause: each hire worked out their commute on their next turn
    (`life::refresh_commute`), never waiting for the search budget, and nearly every commute is an
    A* over the town's tiles (the walk by road gives up when a waypoint along the road lands on a
    tile nobody can stand on: `world::road_route`). With the town at full detail they came in one
    tick: 35 hires, 70-88 ms natively. Now a commute not needed for six hours waits for a tick
    nobody else's is worked out in (`life::unhurried`): that tick is 22 ms.
  - **Earshot.** `conversation::within_earshot` searched 12 m of floor to the end for every pair,
    over a SipHash map; it stops at the first way within earshot now, and someone choosing what to
    do searches once for everyone they ask about (`conversation::Earshot`). The full-town babies
    test went from 191 s (at 240) to 4 s.
  - The work taxis' kerbs (`taxi::kerb_at`): the cheap tests first, 82 ms to 4 ms as a game starts.

  The big-world bench at 1024 m (4 game hours, 3 runs, M3 Pro): the synthetic town
  (`SIZE=1024 RESIDENTS=240`) 8,000–8,900 ticks a second, p99 0.35–0.49 ms, the slowest tick
  7.3–8.0 ms, heap 60 MB (160: 15,100–16,400, p99 0.26–0.30 ms, 10–10.7 ms, 60 MB); a real Big
  town (`WORLDS=`, seaside 104736 filled to the cap, 222 residents) 4,800–5,000 a second, p99
  3.4–3.6 ms, the slowest tick 41–42 ms (98–134 ms before), heap 76 MB. The slowest ticks left are
  the work taxis' plans (`taxi::plan_group`, `book_backs`), people done at work and the traffic.

## Sizes

One size now: `big`. Cosy and normal were dropped in 0.8.0 (Create World has no size option);
a save or town file that still says `cosy` or `normal` reads as `big`. A world is 896–1024 m: a
main town (the kind's shape, its streets reaching all of its box) + 5–7 hamlets of 16–24 lots,
farmsteads and cottages along the country roads, ~120 starting residents, growing to `MAX_SIMS`.
Every town has the open world's roads, outings, commutes timed by the walk, cars and taxis; a
save without the open-world fields takes its network from its meta's roads, or its streets.

## Town file additions

All optional; absent means today's behaviour. Types in `web/src/core/protocol.ts`
(`OpenWorldTownFields`, `RoadPlan`, `WorldMeta`); the sim reads them in `lot.rs` `LotFile`.

- `size: 'big'` (cosy and normal read as `big`)
- `roads: RoadPlan[]` (top level, for the sim; same objects as `meta.roads`). `RoadPlan.kind`:
  `'street'` (default) · `'lane'` · `'country'` (between areas) · `'trail'` (2 m, foot only).
  `RoadPlan.area`: index into `areas` (absent for country roads and trails between areas).
  A Big world's country roads (`game/macro.ts`) leave an area by a street's end straight on, by
  a T square off one of its streets or off a country road leaving it already, or run straight on
  from one area to a street of the next; out in the country they never turn more than 70° over
  40 m of their own line (`COUNTRY_TURN`: no hairpins; where two roads meet doesn't count),
  checked by `tools/worlds/sample.ts`. Homes along them stand only where they run straight
  (`BEND`), and an area's lots keep 18 m from its junctions and off its tight bends (`CORNER`,
  `TIGHT_BEND`), so cars turn in and out on the paving.
  The sim builds its graph from the polylines: endpoints, plus joins snapped within
  `max(width) / 2 + 1` m of the other road's middle. A world's roads are one
  network by that rule (`game/roads.ts` `connect`: lanes go on to another road; a court or two
  per area at most, none longer than 45 m), checked by `tools/worlds/sample.ts` and
  `generated_worlds.rs`.
- `areas: { name, kind: 'hamlet' | 'countryside', x, z, w, d }[]` — area 0 is the main town.
- `plots[].area?: number`, `plots[].kind?: 'home' | 'farm' | 'public' | 'shop' | 'countryside'`.
  Countryside plots are public, unfenced, with an `entry` at their trail end.
- `exits: ([x, z] | { x, z, kind: 'road' })[]` — the roads out of town. (Town files and saves
  written while 0.7.0's bus was in development may list bus stops too, `kind: 'stop'`: the bus is
  gone, and they're no ways out; the sim leaves them out as the town loads. A shared work taxi
  takes its place.) A big world's roads are laid 7 m wide (`macro.ts` `ROAD_WIDTH`), wide enough
  for two cars to pass in their lanes.
- `terrain: { water?: Rle, blocked?: Rle }` — run-length masks over the ground grid in row-major
  order: `[runLength, value, runLength, value, ...]` with value 0/1, starting at tile (0, 0).
  The sim treats set tiles as a base blocked layer; the renderer draws water from `meta.zones`.
- `parking: { x, z, rot, area }[]` — public parking spots for car trips between areas.
- `drives[].road?: number`, `drives[].s?: number` — the road a driveway meets and the distance
  along its polyline (m): where the driveway's line, on across the verge, comes onto it.
  `drives[].frontX?: 1 | -1` — a driveway running along x (its house's street east or west of
  it; `front` then 0). Absent: along z, as before.
- `drives` and `cars` in towns (`game/town.ts`): every house's driveway, and the
  households' cars on them where the street drew its parked cars (`shared/cars.ts` `houseCar`:
  the same houses, about four in five, the same bodies and paints; the player's home always has
  one when its driveway reaches the road). The renderer draws the sim's cars and no others when
  the structure has `cars`; towns and saves without them keep the street's parked ones.
  Cars drive the road network in every town, as the open world does: the sim builds the graph
  for its cars too (`cars::settle_all`) where a town has none of its own. A driveway meeting its road more than 60° from
  square (alongside it rather than onto it) keeps its car parked (`Car::no_way`). Cars turn onto
  the road at the kerb, not across the sidewalk; a big world's parking bays are turned into nose
  first and backed out of like driveways. Garden fences keep a metre off every driveway's sides
  (room for the doors) and start behind where a garage could go at its end (Build mode's Garage
  tool); kerbside furniture keeps 1.2 m off a driveway's mouth where the world has cars, and on
  a 6 m lane (3.3 m between the kerbs, too narrow for two cars in their lanes) stands 0.5 m back
  from the kerb (`pullOverStreets`, `cars::LANE_KERBSIDE`), for two cars meeting there to pull over
  onto the pavement and pass. Cars take turns where their ways cross or meet (junctions' boxes,
  where two roads cross or touch between their ends too, driveway and bay mouths, a 6 m lane's
  tight bend): one queue of claims, granted in the order asked (`sim-core/src/traffic.rs`).
- Junctions giving way (the owner's call, 2026-11-13: lights and zebras at every junction had cars
  stopping at a quiet side street's crosswalks for nobody). A junction is where `SIGNAL_ARMS` (3)
  roads for cars or more meet; only a big one is lit (`Signal::lit`, `Travel::signals`): where
  `LIT_ARMS` (4) arms or more meet, or `LIT_MAIN_ARMS` (3) of them are streets' or country roads'
  (two main roads meeting: a street through it has two arms, a road meeting it the third), and a
  roundabout's. In the sample worlds of seed base 0 that's 408 of 1860 junctions (17 a world, 77
  before). Every other one gives way (`Travel::give_way`): cars take turns through its box by the
  claims as at any junction, and its zebras are on a street's arm only, one of a street's two
  through it (the first round; a lane meeting a street is crossed as anywhere): 573 in all, and 662
  at the lit ones (4606 before). A car stops at such a zebra's stop line only for someone on foot on
  it or stepping onto it (`ZEBRA_LOOK` 1.5 m along their way: `traffic::on_crossings`), or on the
  zebra of the arm it goes out by (`traffic::zebra_stops`, `cars::zebra_yield`), waiting for them by
  name (`Car::on_foot`: they don't wait for it in turn); else it drives on. Those only stepping onto
  it it waits for `ZEBRA_KERB_WAIT` 6 s at most (a stream of them, the next wait for it to pass, as
  anywhere), those on it `ZEBRA_WAIT` 30 s (then it comes on up to them as to anyone in its way,
  `go_on`). The junction's box takes in its zebras as a lit one's crosswalks (no queue stands on
  one).
- Traffic lights (`traffic::Signal`, `traffic::settle`, worked out again on load), at the big
  junctions. Its arms make phases: two more than
  150° apart share one, an arm with none across from it has its own (a crossroads two, a T two:
  the road through, then the side road; `SignalArm::phase`); of those, one road's two arms through
  it first, then two of a kind, each the most nearly opposite first (`phases_of`: a country road
  16° off a street, straighter on than the street's own way, took the street's arm's place, and
  the street's two arms were never green together, Canalside's j35). One phase is green at a time,
  actuated as the arms were (`MIN_GREEN` 5 s, its share of `CYCLE` 50 s, `AMBER` 2 s, `ALL_RED`
  4 s); a car within `APPROACH` (2 s of drive time) of its stop line counts as coming to its green
  (`Traffic::approaching`), so a green isn't cut short in front of it; and nobody waiting anywhere
  nor coming, the lights go back to the main road's phase and rest there (`Signal::rest`: its
  streets' and country roads' arms; a roundabout's stays where it is). A turn across those coming
  the other way on the same green is left to the claims. Only a
  car's way through a junction's box (within `GATE_THROUGH` 7 m of its middle) goes by its lights,
  on the phase of the arm it comes in by, of the first junction it goes through (`traffic::gate`;
  not one it only comes near, out of a driveway by it). A lit junction's box takes in its
  crosswalks (`traffic::settle`: its zone out to their far side), so nobody drives onto one without
  room to clear it beyond ("don't block the box"). Where a car can't wait between two lit
  junctions (the second's stop line before it's out of the first's box), the way through both is
  one group, asked for at the first's stop line on both greens (`Claim::gate2`); not round a
  roundabout, whose junctions are lit as ever (on two of their greens at once, cars round one
  waited for hours). Of the two, the
  one listed first leads and the other follows: the leader on its green for one waiting through
  both holds it, and the follower's phase that one needs is wanted there (it keeps that green,
  ends another after `MIN_GREEN`, and turns to it next, but not twice running). Each light holding
  its green for one waiting at the other's red, two in step kept two cars crossing each other's
  way at red for hours (Hollyglade). A hold lasts `CYCLE` at most from when it began, and the
  second's green is held for those granted till they're past its stop line; one is granted only
  with `GREEN_HELD` 10 s of the hold still to come. A way that comes back into the same lit
  junction by another arm too soon to wait between is through it on the green it came in on (asked
  for on two of its phases' greens at once, never both, it waited for good). A light whose stop
  line a car is past already (in its box, asking again for the rest of its way through after a
  taxi's stand there) neither holds it nor is called by it (`Claim::lights`): it waited in the box
  at red for good while the light was held for one waiting for the box to clear. A work taxi
  waiting at a red light isn't held still on its way: it doesn't give its run up there. Each
  arm has a crosswalk from its corner (where the kerb's `KERB_RETURN` 3 m return round from the
  next arm ends, or the box's edge, whichever is further out) `CROSSWALK` 3 m wide, and a stop
  line `STOP_GAP` 0.5 m past it, no further out than half way to the next junction: a car's claim
  through the lights begins with its nose at the stop line, so it waits there on red, and one
  granted but still short of the line when the light stops being green lets the grant go
  (`Claim::stop_at`). The pole stands at the stop line on the kerb right of those coming in; where
  that's on another road's carriageway, further out along the kerb (onto the island between it
  and a road leaving at a slant), else back towards the crosswalk, else on the left (`signal_at`:
  `POLE_CLEAR` off every road's carriageway). Two lit junctions so near that an arm's crosswalk
  and stop line, and the other's back along it, can't both be out of the other's box are one
  junction there: the stretch between them has no crosswalk (`walk_from` = `walk_to`: no zebra,
  no pedestrians' head) and its stop lines are half way. A
  walk phase follows the cars' (`Signal::walk_phase`): brought round only by someone on foot
  waiting at the junction's crosswalks now (one who took a turn this tick and didn't wait at the
  kerb in it no longer asks: a request standing on for a far resident's turn brought walks round
  for nobody), every arm red, the crosswalks open for `WALK_OPEN` 5 s at most, then its clearance
  (the widest arm at `WALK_PACE` 1 m/s, 3 s at least), then `ALL_RED`; the open time is over once
  nobody waits at the kerbs nor is on the crosswalks (after `WALK_LEAST` 1 s), and the clearance
  once nobody's on them (`Traffic::crossing`). Walk timing stays on the real clock (people don't
  hurry for the cars' clock). Those on foot cross a
  lit junction on its zebras only: its box, and each arm's carriageway out past its stop line by
  `KEEP_OFF` 3 m but for the zebra (an arm without one, out to the box's edge), are blocked to
  them for good (`Signal::off_limits`, `Travel::junction_tiles`, in `ground_blocked`), so their
  way goes over a zebra or crosses the road further off. Kept off is only stepping onto them:
  one standing on them (set down there, or there before they were kept off) gets off them the
  nearest way, over them (`NavGrid::kept_off`, `NavGrid::way_out`), and anyone standing in a
  road's carriageway with nothing to do walks off it to the nearest ground off it
  (`world::off_the_road`): Canalside's Mila, saved in a T's box, stood there for a day, every
  way out shut to her, a taxi and two cars held up round her. The zebra and the road by the stop line
  are measured from the arm's own road there (`SignalArm::span`, its middle at the crosswalk and
  `KEEP_OFF` past the stop line, where that goes out within 30° of the arm's way and about as far
  out; else along the way), not along the arm's way out from the junction's middle: a road
  bending away, or a junction's middle merged off it, left that metres off over the pavement,
  where it shut a house's gate in. And a tile is kept off only on a carriageway: within its kerb
  of one of the junction's roads' middle lines (a box along a way off its road took in a garden's
  planter). Nor do they ever cut ground off: a pocket of pavement walled
  round but for its side onto that road, or a gate on it, gets the shortest way over them back
  to the ground round it left open (`traffic::left_open`, looked at within `KEEP_OFF_ROOM` 4 m
  round each junction). Out of the walk phase a zebra is closed
  to them (a bike or an e-scooter rides on with the traffic): a step onto it, into the
  carriageway (`KERB_IN` 0.25 m in from the kerb; one walking past on the pavement isn't held),
  stops at the kerb (`traffic::kerb_stop`, a 16 m grid of the junctions, `Travel::crossings`), and
  their wait is what brings the walk round; someone already on them goes on across (over a junction next to
  it too), and so does someone a car waits for (`cars::holding_up`: the two waited for each other
  for good at a corner); nor does anyone a car waits for by name wait for it in turn (one backing
  out of a bay by a junction and a cyclist behind it waited for each other for good:
  `cars::in_the_way`). Only an arm into a street or a lane has a crosswalk
  (`SignalArm::crosswalk`, sent to the renderer for its zebra and pedestrians' head): a country
  road's arm is crossed as a road anywhere else, and where only country roads meet there is no walk
  phase. Crossing a road away from the junctions is as before. A roundabout's junctions (a road with `island`) keep an arm a
  phase, the pole just past the box, cars waiting at the box's edge, no crosswalks and no walk.
  A T where a road ends at another's kerb has its node on that road's middle line (`Graph::build`),
  so its box, lights and slow zone are in the middle of the road; and junctions closer than
  `MERGE` 8 m along a road between them are one, with all their arms (`Graph::merge_junctions`:
  each within the other's box; two side roads a metre apart either side of a road are a
  crossroads, not two T's each with its lights in the middle of the other), not on a roundabout's
  ring. The worlds' layout doesn't lay a country road from a street's end that another area's
  street comes up to (`macro.ts` `continued`: it ran on beside that street and crossed it).
- The structure's `junctions` (`JunctionInfo`, `web/src/core/protocol.ts`): every lit junction in
  the snapshot's signal head rows' order, then those giving way, `{x, z, roundabout?, arms: {road,
  dir, width, walkFrom, walkTo, stop, crosswalk, light}[]}` (the arm's `meta.roads` index, its unit
  way out, the carriageway kerb to kerb, its crosswalk and stop line in m out from the middle,
  whether there's a zebra there, its head's snapshot row or null: the renderer draws a zebra and
  its stop line where `crosswalk` is, a stop line alone on a lit arm without one);
  `lightPoles` stays for the kerbside furniture. The snapshot's light rows are as before; every
  head of a junction shows `STATE` 3 (walk) while its walk phase lets people step out, red in its
  clearance. `snapshot::MAX_LIGHTS` is 512, a row for every head (the generated worlds have up to
  367 heads at 121 junctions, over seed bases 0 and 16–35; 256 left some unlit); past it a debug
  build fails, the generated-world check too.
- `garages[].rot?: 0 | 1 | 2 | 3` — a garage the player built (Build mode's Garage tool,
  `Command::BuildGarage`): the way its door faces, as a thing's facing (0 towards +z, 1 +x,
  2 -z, 3 -x). Absent: towards the street, as the town places garages. The same field is in the
  save's `garages` and the structure's `WorldStructure.garages` (`GarageInfo`); a save also keeps
  `cars[].driveSpot?: [x, z, yaw]`, where a car parked before it parked in front of such a garage at
  the end of its driveway. Both are left out when absent, so towns and saves without a garage of
  the player's stay byte for byte as they were.
- `fences: { line: [x0, z0, x1, z1], style?: string, gate?: true }[]` — garden fences, hedges and
  field fences the town puts up (`game/gardenFences.ts`), runs along grid lines on the ground storey
  in a `fenceStyles` id, never on a wall: real `Edge::Fence` / `Edge::Gate` edges, saved like the
  player's. With them `meta.gardenFences: true`; a town or save without it gets the same plan drawn
  only (`render/shared/shownFences.ts`).
- `meta.zones: { kind: 'woods' | 'meadow' | 'water' | 'hill' | 'fields', poly: [x, z][] }[]`
- `meta.pads: Record<plotIndex, y>` — the height an off-town outdoor plot is levelled to (m).
- `meta.world.size` mirrors `size` for the renderer.

## Worker protocol (sim worker ↔ main thread)

- `setView(rect)` — the focus lot: its wall/floor meshes and full structure, as today.
- `setWatch(rect)` — new: the camera's stream rectangle; `World::set_view` uses it for the
  sim's full-detail area (LOD), the view rect when never sent. It is also what's drawn
  (`World::set_watch`): cars further than 32 m out of it drive through each other, claiming
  nothing (`cars::sight`, the owner's call for big towns); the snapshot leaves them out. A car
  the camera comes to on top of another is put clear along its way before it's drawn, and one
  held up out of the picture (8 m and more from the watch) with all it waits for is put on after
  3 game minutes. Never sent (drawn whole, as before), every car is in sight.
- `rooms` is sent as a transferable `Uint16Array` (same indexing as today's `number[]`).
- Structure is versioned per sim chunk; `structure_rect(x0, z0, x1, z1)` returns the walls,
  doors and objects inside a rectangle, and `chunk_versions()` lists the versions to diff.

## Game feel

Walking is 1.5 m a real second (`WALK_SPEED`), about 1.9 m a game minute at the 48× game clock
(a game minute is 1.25 real seconds), so 1 km takes about 9 game hours on foot. Vehicles go on
the traffic's own clock (`DRIVE_SCALE`, 4): everything about how they move and take turns (their
speeds and acceleration, bends, look-aheads, claims, jams, the traffic lights' timing, the taxi's
way weighed in seconds) is timed in seconds of drive time, four to a real second, so it keeps its
shape at four times the pace. Cars drive at 9 m a second of it in the hamlets (`TOWN_SPEED`, 32
km/h) and 16 on country roads (`COUNTRY_SPEED`, content `cars.countrySpeed`), speed up and slow
down at 3 m/s² (`cars::ACCEL`), go straight through a junction at their speed (the lights and the
stop lines stop them) and turn through one at 3.5, and slow round a bend no more than it pulls them
sideways 6.5 m/s² (`cars::SIDEWAYS`): from a big world's road furthest south to its road furthest
west (one to two km) in about 20 to 25 game minutes with the roads to itself, 50 at most, where it
took two to five hours at real speeds (`generated_worlds`' drive across). Its etas follow (`travel::DRIVE_MINUTES`). A bike or an e-scooter is ridden at its content `vehicle.speed` times
walking (the city bike 3, the kids' bike 2.2, the e-scooter 4) on the same clock, so four times
that (`content::Vehicle::pace`, `ride.rs`), along the same way as the walk, and the way to work is
timed by the ride (`ride::commute_bike`). Walking, getting in and out of a car, its doors and the
taxi's stands for its riders stay real time, and schedules game time. With the taxi that quick,
visits and outings out of walking reach go by it far more often: six leisure taxis at a time, there
and back (`taxi::LEISURE_FLEET`), keep a big world's evening from jamming.

- Areas are 170 m or more apart (best ~230 m from the nearest, every quarter of the world with
  one or with homes out in the country); trips between them are by car. The household setting off with
  the driver for somewhere the car parks near goes in it too (`Car::riders`, saved with the car
  as `cars[].riders`): it waits in the driveway for them, they ride there and get out at the
  parking spot, and when one of them goes home they all go back to it and home together.
  Renderer contract unchanged: the car is a snapshot car row as ever, and everyone in it (the
  driver, and now the passengers) is a resident sitting in their seat (`pos`, pose sit, not
  away), so no new snapshot field.
- Countryside spots are within ~150 m of their hamlet, reached by trail.
- `careerRules.commuteMinutes` means only the off-map part; the on-map part comes from the ETA.
- The way to work or school is chosen once (`world.rs`, `TaskKind::Work`): already riding, else a
  seat in the household's car, a neighbour's lift (`cars::lift_seat`: a friend of 30 or more whose
  car stands within 25 m of the door, setting off with a seat free), the household's bike or
  e-scooter, or on foot. Late for work is
  judged by when that way gets them there.

## Life on every lot

Residents outdoors are drawn on every lot within the draw radius, and window glow follows who
is home. The fences round gardens are the world's (`WorldStructure.fences`), so residents on every
lot walk round them; they are drawn the same whichever lot is the focus (the focus lot's with the
lot, the rest merged into the street's chunks, every gate by the game so it swings). Full interiors are drawn for the focus lot (the one under the camera) plus 2 cached ones.

## Integration rules

- Each branch keeps every existing test passing and today's towns identical.
- New sim tests are separate files under `crates/sim-core/tests/`, written to move into
  `tests/sim/` unchanged once PR #43 lands.
- Before merging, run every `ci.yml` step locally.
- Big was behind the dev flag `?big=1` (UI) until all four branches were in; it is the only size since 0.8.0.
