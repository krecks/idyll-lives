/**
 * Neighbours' houses on lots that face a street on any side (`WorldMeta.facing`): a street of
 * houses turned a quarter, the whole town with it, is dressed the same way turned a quarter
 * (gardens, fences, cars, flower beds; the houses' fronts, doors and garages).
 */
import { describe, expect, test } from 'bun:test';
import type { PlotInfo, WorldStructure } from '../src/core/protocol';
import type { StreetPlan } from '../src/render/shared/street';

type Rect = { x: number; z: number; w: number; d: number };

/** Two houses each side of a street running along x (`turn`: the town turned a quarter, the street along z). */
function town(turn: boolean): WorldStructure {
  // Turned: the town's (x, z) at (-z, x), so the street dressing (laid out turned back) comes out where it was.
  const pt = (x: number, z: number): [number, number] => (turn ? [-z, x] : [x, z]);
  const rect = (r: Rect): Rect => (turn ? { x: -(r.z + r.d), z: r.x, w: r.d, d: r.w } : r);
  const box = ([x0, z0, x1, z1]: [number, number, number, number]): [number, number, number, number] => {
    const r = rect({ x: x0, z: z0, w: x1 - x0, d: z1 - z0 });
    return [r.x, r.z, r.x + r.w, r.z + r.d];
  };
  // North of the street (z 0..20, street on its max-z side: facing 2) and south of it (z 26..46: facing 0).
  const lots: { r: Rect; house: [number, number, number, number]; door: number; drive: Rect; facing: number }[] = [
    { r: { x: 0, z: 0, w: 20, d: 20 }, house: [3, 5, 14, 14], door: 8, drive: { x: 14.6, z: 9.6, w: 3, d: 10.4 }, facing: 2 },
    { r: { x: 20, z: 0, w: 18, d: 20 }, house: [24, 6, 33, 15], door: 28, drive: { x: 33.6, z: 10.6, w: 3, d: 9.4 }, facing: 2 },
    { r: { x: 0, z: 26, w: 20, d: 20 }, house: [4, 31, 15, 40], door: 9, drive: { x: 15.6, z: 26, w: 3, d: 10.4 }, facing: 0 },
    { r: { x: 20, z: 26, w: 18, d: 20 }, house: [23, 30, 32, 40], door: 27, drive: { x: 32.6, z: 26, w: 3, d: 9.4 }, facing: 0 },
  ];
  const plots: PlotInfo[] = lots.map((l, id) => ({ id, name: `${id}`, ...rect(l.r), public: false, entry: pt(l.door + 0.5, l.facing === 2 ? 18 : 28), house: box(l.house) }));
  const paths = lots.flatMap((l) => [
    rect(l.facing === 2 ? { x: l.door, z: l.house[3], w: 1, d: 20 - l.house[3] } : { x: l.door, z: 26, w: 1, d: l.house[1] - 26 }),
    rect(l.drive),
  ]);
  // Turned a quarter, a lot whose street was south of it has it west of it (1), north of it: east (3).
  const facing = lots.map((l) => (turn ? (l.facing === 2 ? 1 : 3) : l.facing));
  const [w, d] = turn ? [46, 38] : [38, 46];
  return {
    width: w,
    depth: d,
    objects: [],
    plots,
    rooms: new Array(w * d).fill(0),
    openings: [],
    walls: [],
    households: [],
    sims: [],
    exits: [],
    meta: { kind: 'town', name: 'Test', streets: [], paths, facing, seed: 1 },
  } as unknown as WorldStructure;
}

const options = { has: () => true, entry: () => undefined, lived: false };

/** Vertex positions (x, z) of a mesh, sorted, rounded to the centimetre. */
const points = (pos: number[], turn: boolean) => {
  const out: string[] = [];
  for (let i = 0; i < pos.length; i += 3) {
    // Turned back: the town's (x, z) from (-z, x).
    const [x, y, z] = turn ? [pos[i + 2], pos[i + 1], -pos[i]] : [pos[i], pos[i + 1], pos[i + 2]];
    out.push([x, y, z].map((v) => (Math.round(v * 100) / 100 + 0).toFixed(2)).join(','));
  }
  return out.sort();
};

/** Instances' places, turned back (sizes and yaw left out: hedges are sized by where they stand). */
const places = (plan: StreetPlan, turn: boolean) =>
  plan.batches.map(([key, items]) => [key, items.map(([x, z]) => (turn ? [z, -x] : [x, z]).map((v) => (Math.round(v * 100) / 100 + 0).toFixed(2)).join(',')).sort()]);

describe('street dressing on lots facing any side', async () => {
  // The renderer's modules read the page's address (and its frames' timing) as they load.
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: {} });
  const { HousePlanner } = await import('../src/render/shared/house');
  const { planStreet } = await import('../src/render/shared/street');
  test('a street turned a quarter is dressed the same, turned', async () => {
    const [a, b] = [await planStreet(new HousePlanner(), town(false), null, options), await planStreet(new HousePlanner(), town(true), null, options)];
    expect(a.specs.length).toBe(4);
    expect(b.specs.length).toBe(4);
    // The houses' fronts, doors and garages, turned.
    for (const [s, t] of a.specs.map((s, i) => [s, b.specs[i]] as const)) {
      expect([t.front[0], t.front[1]]).toEqual([-s.front[1], s.front[0]].map((v) => v + 0));
      expect(t.door).toBe(s.door);
      expect(!!t.garage).toBe(!!s.garage);
      if (s.garage && t.garage) expect([t.garage.x0, t.garage.z0, t.garage.x1, t.garage.z1]).toEqual([-s.garage.z1, s.garage.x0, -s.garage.z0, s.garage.x1]);
    }
    expect(a.specs.some((s) => s.garage)).toBe(true);
    // Fences, cars, flower beds, shrubs and pots: the same, turned.
    expect(points(b.props.pos, true)).toEqual(points(a.props.pos, false));
    expect(points(b.shadows.pos, true)).toEqual(points(a.shadows.pos, false));
    expect(places(b, true)).toEqual(places(a, false));
    expect(a.props.pos.length).toBeGreaterThan(0);
  });
});
