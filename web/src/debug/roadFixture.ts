/**
 * The road hierarchy's fixture drawn in a real Big world (`?bench=bigworld&big=roads`): until the generator lays them,
 * every new kind of road (`render/roadHierarchyFixture.ts`: the motorway's two carriageways over a street and a river,
 * its half-diamond's ramps, an avenue, a one-way loop) set down in the south-east of a flat countryside world with no
 * town (away from the bench village's home lot, which the game plays and draws where it is), on a land drawn flat
 * (no built surface: the decks climb on their own, `motorway.ts` `deckLevels`), the river's water with it, the
 * junctions worked out from the roads as the simulation joins them up (`tests/fixtures/junctions.ts`
 * `deriveJunctions`). Its area is the world's main town's, so the bench's town view is held over its middle (`over=centre`;
 * `tx`, `tz` move it: the overpass at `tx=-100&tz=44`, the avenue's junctions at `tx=-150&tz=-80` and `tx=-30&tz=-80`,
 * the bridge at `tx=99&tz=44`, the one-way loop at `tx=90&tz=-120`).
 *
 * Debug only: nothing outside the bench calls it.
 */

import { groundDepth, type RoadPlan, type WorldStructure } from '../core/protocol';
import { roadHierarchy } from '../render/roadHierarchyFixture';
import { deriveJunctions } from '../../tests/fixtures/junctions';

type Pt = [number, number];

/** How far round the fixture nothing of the world it is set in is kept (m). */
const CLEAR = 60;

/** `base` (a flat Big world) with its town taken away and the road hierarchy's fixture set in its middle. */
export function roadFixtureWorld(base: WorldStructure): WorldStructure {
  const h = roadHierarchy();
  const [W, D] = [base.width, groundDepth(base)];
  const [ox, oz] = [Math.round(W - h.width - CLEAR), Math.round(D - h.depth - CLEAR)];
  const shift = ([x, z]: Pt): Pt => [x + ox, z + oz];
  const roads: RoadPlan[] = h.roads.map((r) => ({ ...r, points: r.points.map(shift) }));
  const box = { x0: ox - CLEAR, z0: oz - CLEAR, x1: ox + h.width + CLEAR, z1: oz + h.depth + CLEAR };
  const clear = (pts: readonly Pt[]) => pts.every(([x, z]) => x < box.x0 || x > box.x1 || z < box.z0 || z > box.z1);
  const meta = base.meta!;
  const world: WorldStructure = {
    ...base,
    // (No lots: the fixture's roads alone, on the land.)
    plots: [],
    flats: [],
    junctions: deriveJunctions(roads),
    meta: {
      ...meta,
      roads,
      streets: [],
      paths: [],
      greens: [],
      facing: [],
      pads: {},
      lotLooks: {},
      places: [],
      paving: [],
      // (Its area the main town's, for the bench's camera: no centre, which would be paved.)
      areas: [{ name: 'The road hierarchy', kind: 'countryside', x: ox, z: oz, w: h.width, d: h.depth }],
      centre: [],
      land: undefined,
      relief: undefined,
      zones: [...(meta.zones ?? []).filter((z) => clear(z.poly)), { kind: 'water', poly: h.river.map(shift) }],
      world: meta.world && { ...meta.world, fields: [] },
    },
  };
  console.info('[roadFixture]', { world: [W, D], at: [ox, oz], roads: roads.length, junctions: world.junctions?.length });
  return world;
}
