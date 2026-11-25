// The street dressing's tests: a small town along one street, and the plan options they dress it with.
import type { PlotInfo, WorldStructure } from '../../src/core/protocol';

/** Every lot has its things, no entry overrides, not lived in. */
export const streetOptions = { has: () => true, entry: () => undefined, lived: false };

/** Two houses each side of a street along x, its middle at z = `mid`, `width` m wide; `extra`: more of the world (its light poles, its pull-over streets). */
export function streetTown(mid: number, width: number, extra: Partial<WorldStructure> = {}): WorldStructure {
  const [north, south] = [mid - width / 2, mid + width / 2];
  const lots = [
    { x: 0, w: 20, z: 0, d: north, door: 8, facing: 2 },
    { x: 20, w: 18, z: 0, d: north, door: 28, facing: 2 },
    { x: 0, w: 20, z: south, d: 46 - south, door: 9, facing: 0 },
    { x: 20, w: 18, z: south, d: 46 - south, door: 27, facing: 0 },
  ];
  const plots: PlotInfo[] = lots.map((l, id) => ({
    id,
    name: `${id}`,
    x: l.x,
    z: l.z,
    w: l.w,
    d: l.d,
    public: false,
    entry: [l.door + 0.5, l.facing === 2 ? north - 1 : south + 1],
    house: l.facing === 2 ? [l.x + 3, 5, l.x + 14, 13] : [l.x + 4, south + 5, l.x + 15, south + 13],
  }));
  const paths = lots.map((l) => (l.facing === 2 ? { x: l.door, z: 13, w: 1, d: north - 13 } : { x: l.door, z: south, w: 1, d: 5 }));
  return {
    width: 38,
    depth: 46,
    objects: [],
    plots,
    rooms: new Array(38 * 46).fill(0),
    openings: [],
    walls: [],
    households: [],
    sims: [],
    exits: [],
    meta: {
      kind: 'town',
      name: 'Test',
      streets: [],
      roads: [{ points: [[0, mid], [38, mid]], width, ends: ['out', 'out'] }],
      paths,
      facing: lots.map((l) => l.facing),
      seed: 1,
    },
    ...extra,
  } as unknown as WorldStructure;
}
