# Living town: the shared contract

This file is the contract for the living town view. You play inside the home, as before. Zoom out, and the
town around the household becomes a dense, lived-in place, with tags for your people, the places you can go
and the homes for sale. Click a tag, and the camera takes you there.

The inspiration is Paralives' open-world town and town map, and The Sims' neighbourhood map view, with the
density of a real town seen from above. We take their ideas, not their look or names.

The household stays the game. Nothing here changes how residents live, decide or feel.

Five streams build the feature, each on its own branch. The integrator merges them into `feat/living-town`
in this order:

1. `lt/worldgen`: the dense town. More lots (terraces and semi-detached pairs), a backdrop of unsimulated
   buildings, a town centre, and an optional river.
2. `lt/sim-scale`: the 09:30 job-handout spike, `within_earshot`, and the resident caps raised.
3. `lt/camera`: one zoom from the play orbit up to the town view, glides, and click-to-go from tags.
4. `lt/render`: drawing at town scale. Backdrop and far houses instanced, far woods, a ground colour map at
   altitude, and far movers.
5. `lt/hud`: the town tags, the clock and day bar, and the "Today in <town>" card.

This file and the types it names in `web/src/core/protocol.ts` are **pinned** (`tools/pins/pins.json`). A
stream that needs them changed asks the integrator; it does not change them itself.

## World data (worldgen writes it; render and hud read it)

All the fields are optional: a town without them draws and plays as today. The types live in
`web/src/core/protocol.ts`.

### `WorldMeta.backdrop?: BackdropBuilding[]`

The buildings that make a town dense without being lots. Nobody lives in them and the sim never sees them,
so they cost no sim time and no save space beyond the town file.

```ts
interface BackdropBuilding {
  /**
   * Footprint corners, town metres, 4 to 8 points (most are rectangles), counter-clockwise seen from above:
   * the signed area Σ(x·z′ − x′·z) is negative.
   */
  poly: [number, number][];
  /** Ground height of the footprint's lowest corner (m). */
  y: number;
  /** Storeys (1 to 6); each storey is 3 m. */
  storeys: number;
  roof: 'gable' | 'hip' | 'flat' | 'mansard';
  /**
   * The ridge's direction for gable and mansard roofs (rad, 0 = along x): a turn around +y (y up), so the ridge
   * runs along (cos r, −sin r) in (x, z). It runs along the footprint's longer side.
   */
  ridge?: number;
  /** What it is: drives its look and which tags it gets. */
  use: BackdropUse;
  /** A wall and roof colour pair: an index into the renderer's backdrop palette, taken modulo its length. */
  look: number;
  /** Places with a name and opening hours (`use` a place kind): its index into `WorldMeta.places`. */
  place?: number;
  /** The area (`TownArea` index) it stands in. */
  area: number;
}

type BackdropUse =
  | 'flats' | 'terrace' | 'house'                  // homes nobody plays
  | 'shop' | 'cafe' | 'restaurant' | 'office'      // places
  | 'school' | 'church' | 'hall' | 'workshop' | 'farm' | 'barn';
```

The rules worldgen keeps (`tools/worlds/sample.ts` checks them):
- A backdrop building never overlaps a lot, a road with its pavements, a junction pad, water, or another
  backdrop building. It keeps at least 1 m from a lot's boundary.
- Every main town has at least 60 backdrop buildings, and every hamlet at least 8. The town centre (see
  below) is the densest part.
- The buildings face the nearest road: the long side or the gable end runs parallel to the road.

### `WorldMeta.places?: TownPlace[]`

A place is somewhere the tags can name: a shop, café, restaurant, workplace, school or church, whether it
is a backdrop building or a public lot (the shops and parks the sim has already).

```ts
interface TownPlace {
  name: string;
  kind: 'shop' | 'cafe' | 'restaurant' | 'office' | 'school' | 'church' | 'park' | 'outdoor' | 'workshop' | 'farm';
  /** Where its tag stands (town metres; at the door, or the middle of a park). */
  x: number;
  z: number;
  /** A public lot: its plot index (the sim's own place: visitors, who is there). Absent: a backdrop place. */
  plot?: number;
  /** Opening hours, minutes of the day [open, close) and the days (bit 0 = Monday). Absent: always open (parks). */
  hours?: { open: number; close: number; days: number };
  /** Career categories (`JobView.category`) whose workers are shown as working here (see Workplaces). */
  work?: string[];
  area: number;
}
```

### `WorldMeta.centre?: { area: number; poly: [number, number][] }[]`

Each main town's centre: the square and the high street. Worldgen fills it densest (terraces, shops, flats
of 2 to 4 storeys). The renderer may pave it, and the HUD names it.

### Denser lots

Main towns get terrace rows and semi-detached pairs as **real lots** (`plots`), on narrow plots of 6 to 9 m
frontage. They are ordinary lots to the sim: a house, a household, a price. Whether the houses of a row
share walls depends on what `tools/housegen` can build. If it can't, they are narrow detached houses with
gaps of 1 m or less. A world has about 1.5 times today's homes: worldgen reports the count per seed.

### Optional river: `WorldZone.kind: 'water'` along a polyline

A river is drawn and blocked like today's water (`terrain.water`, a `water` zone). Roads cross it on
bridges only where worldgen lays a road over it at right angles, and a bridge is part of the road.

### Save format

Save format 37 becomes 38: the town file's new meta fields go into the save. An older save has none of
them and plays as today. Sample saves may break: re-bless them.

## Sim scale (sim-scale writes it)

- Find the cause of the frame spike at 09:30 with about 170 residents, and fix it. The spike comes when the
  town's jobs are handed out (`docs/design/open-world.md`, Caps).
- `conversation::within_earshot`: drop the per-pair 12 m Dijkstra over a SipHash map. Either cache it per
  tick, bound it by a cheap distance check first, or use a faster hasher. Its answers must stay the same.
- Raise `NEIGHBOUR_CAP` (`web/src/game/town.ts`) from 140 to about 220, and `MAX_SIMS`
  (`crates/sim-core/src/world.rs`) from 160 to about 240. Raise the snapshot layout and the renderer's
  `MAX_CHARACTERS` with them (`render/shared/characters/rig.ts`).
- What residents choose and how they live stays the same. The household's needs, AI, careers, cooking and
  social modules are pinned. `grid_identity` is re-blessed only when what changed is written in the commit
  message.

## Camera modes (camera writes it; render and hud read it)

`web/src/render/shared/lens.ts` exports:

`TOWN_FROM` (120 m of orbit radius), `TOWN_FULL` (400 m), `TOWN_MAX` (900 m: the farthest the camera
zooms out) and `townBlend(radius)` (0..1, smoothstep from `TOWN_FROM` to `TOWN_FULL`).

- The zoom is one continuous movement: there is no mode switch or button. Up to `TOWN_FROM` it behaves
  exactly as today, so `RADIUS_LIMITS[1]` is reached as before inside a lot. Past it, the orbit's radius
  limit is `TOWN_MAX`.
- As `townBlend` rises from 0 to 1:
  - beta is eased into the range 0.6 to 1.0;
  - the target is held over land (`panBounds`);
  - the fog's density falls to about a quarter, and the fog begins 1.5 times the radius out, past the town
    (`townFogFrom`), thick enough by the land's edge to fade it into the sky (`edgeFog`);
  - the far plane covers the whole world.
- `web/src/render/shared/townView.ts` `townView` is the shared state: `blend`, `radius`, `target` and
  `project` (a world point to CSS pixels), written by the renderer every frame. It also holds `goTo({ x, z,
  radius?, select? })`, set once. The camera stream wires all of them in the renderer. The render and hud
  streams only read them. Before the camera stream is merged, they may wire a stand-in for their own
  testing, marked `// lt-temp`, which the integrator removes.
- `goTo` flies to a point using the existing glides (`shared/focus.ts` `Flight`, `cameraRig.ts`), and it is
  what a tag's click calls. With `select: simId`, the resident is selected and followed when the glide
  ends.

## Town-scale drawing (render)

- `render/pc/townView.ts` draws, whenever `townBlend > 0`:
  - every backdrop building in one instanced pass for the whole world: walls, roofs and a few window
    bands, about one draw per material;
  - the far LOD houses of real lots, merged into the same pass.
- Far woods are instanced crown clusters or impostors past the near woods' range.
- A ground colour map is used at altitude: fields, roads, the centre's paving, water. It is a texture over
  the whole world, baked once per town.
- Movers past the near range are tiny instanced meshes: cars, taxis, bikes and residents, in their own
  colours. Use the snapshot's positions; don't add sim state.
- Culling in the town view comes from altitude and the frustum, not from the preset's `farCull`. Up to
  `TOWN_FROM` the play camera draws exactly as today: blind review 8 guards it.
- There are no performance budgets in this run. Measure and report draws, triangles, GPU ms and frame
  times (`tools/bench/render.mjs`).

## Town tags (hud)

The tags are HTML labels over the canvas, positioned from world points each frame (`ui/town/*`). They fade
in with `townBlend` (from about 0.2; the household's own from the start of the town view, so the family and
its home are marked at any distance past `TOWN_FROM`), are filtered, and are kept from overlapping: when two
meet, the more important one stays. The order of importance is: household > your people > places > homes for
sale > street names.

| Tag | Shows | Click |
|---|---|---|
| Household member | Name, what they're doing, where ("Mara · at work · Bakery · home 17:10") | Select them and glide down |
| Your people | Family, partners and friends (by relationship) elsewhere in town, where they are | Glide there; menu: visit, call (existing actions) |
| Place | Name, open or closed (and till when), which of your people are there, how many are inside | Glide there |
| Home for sale | Price and size | Glide there and open the home picker |
| Street / area | The street's and the hamlet's names at the right zoom | — |

- The household's home has a tag of its own ("Home") while nobody of the household is in, so the home is marked
  from anywhere in the town view; a click glides down to it.
- The tags read only what the sim's views already carry: `SimView.plot`, `visiting`, `trip`, `awayUntil`
  and `job`; relationships; the `PlotInfo`s; and `WorldMeta.places`. No new sim state is added.
- **Workplaces:** a resident at work (`plot` null, `awayUntil` set) is shown at the place whose `work` lists
  their `job.category`. When several places do, the place is picked stably by resident id. When none do,
  the tag says "at work" with no place.
- **The top bar** (`TopBar.svelte`): the clock and date as today, and a day bar from sunrise to sunset
  (`render/pc/daylight.ts`'s sun times) with the speed.
- **"Today in <town>":** a card with the household's next event: a shift, school, a planned visit or a
  birthday. It comes from the plan and the job views.
- **Filters** in a small tag menu: household, people, places, for sale, names.

## Pins

`tools/pins/pins.json` pins the accepted parts by sha256. `bun tools/pins/check.ts` (and
`tools/ci-local.sh pins`) fails when a pinned file differs from its hash. Only the integrator may change an
entry, and when it does it writes `reason` and `by` into the entry. The end report lists every unpin.
