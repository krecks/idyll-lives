import { expect, test } from 'bun:test';
import type { Opening, WallEdge, WorldStructure } from '../src/core/protocol';
import { HousePlanner, WALL_HALF, type HousePart } from '../src/render/shared/house';

/**
 * The house's boards are built with their arrises eased: a chamfer of 4 mm at 45° on the long edges that show (frames,
 * sills, casings, corner boards, baseboards, friezes, the plinth's ledge, door leaves), each a flat face with its own
 * normals. A chamfer stays inside its board: nothing of the house stands further out than it did.
 */

const [W, D] = [20, 16];
const [X0, Z0, X1, Z1] = [4, 4, 12, 10];
function house(): { parts: HousePart[]; doors: HousePart[] } {
  const walls: WallEdge[] = [];
  const rooms = new Array(W * D).fill(0);
  const inside = (x: number, z: number) => x >= X0 && x < X1 && z >= Z0 && z < Z1;
  for (let z = 0; z < D; z++) {
    for (let x = 0; x < W; x++) {
      if (inside(x, z)) rooms[z * W + x] = 1;
      if (inside(x, z) !== inside(x, z - 1)) walls.push({ axis: 'h', x, z });
      if (inside(x, z) !== inside(x - 1, z)) walls.push({ axis: 'v', x, z });
    }
  }
  const openings: Opening[] = [{ axis: 'h', x: 6, z: Z0, kind: 'door' }, { axis: 'h', x: 9, z: Z0, kind: 'window' }, { axis: 'v', x: X1, z: 6, kind: 'window' }];
  const world = { width: W, depth: D, walls, openings, diagonals: [], rooms, objects: [], plots: [], households: [], sims: [], exits: [] } as unknown as WorldStructure;
  const built = new HousePlanner().build(world, { x: 0, z: 0, w: W, d: D })!;
  return { parts: built.parts, doors: built.doors.flatMap((d) => d.parts) };
}
/** A geometry's triangles as (normal, shortest side). */
function faces(geo: HousePart['geo']): { n: number[]; short: number }[] {
  const { pos, nor, idx } = geo;
  const out = [];
  for (let t = 0; t < idx.length; t += 3) {
    const p = [0, 1, 2].map((k) => [pos[idx[t + k] * 3], pos[idx[t + k] * 3 + 1], pos[idx[t + k] * 3 + 2]]);
    const side = (a: number[], b: number[]) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    out.push({ n: [nor[idx[t] * 3], nor[idx[t] * 3 + 1], nor[idx[t] * 3 + 2]], short: Math.min(side(p[0], p[1]), side(p[1], p[2]), side(p[2], p[0])) });
  }
  return out;
}
/** A chamfer's face: its normal halfway between two of the axes. */
const eased = (n: number[]) => n.filter((c) => Math.abs(Math.abs(c) - Math.SQRT1_2) < 1e-6).length === 2;

test('the trim, the plinth and a door leaf have chamfers: flat faces at 45°, 4 mm of each face they ease', () => {
  const { parts, doors } = house();
  for (const part of [parts.find((p) => p.name === 'trim')!, parts.find((p) => p.name === 'plinth')!, doors.find((p) => !p.geo.empty)!]) {
    const chamfers = faces(part.geo).filter((f) => eased(f.n));
    expect(chamfers.length).toBeGreaterThan(part.name === 'trim' ? 150 : 8);
    // (Across it a chamfer is 4 mm × √2; its triangles' short sides are that, or its mitred end's, 4 mm × √3.)
    for (const f of chamfers) expect(f.short).toBeLessThan(0.004 * Math.sqrt(3) + 1e-6);
    expect(chamfers.some((f) => Math.abs(f.short - 0.004 * Math.SQRT2) < 1e-6)).toBe(true);
  }
});

test('a chamfer is cut into its board: the house stands no further out than its sills and its plinth\'s ledge', () => {
  const { parts } = house();
  const reach = (name: string) => {
    const { pos } = parts.find((p) => p.name === name)!.geo;
    let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
    for (let i = 0; i < pos.length; i += 3) [x0, z0, x1, z1] = [Math.min(x0, pos[i]), Math.min(z0, pos[i + 2]), Math.max(x1, pos[i]), Math.max(z1, pos[i + 2])];
    return [X0 - x0, Z0 - z0, x1 - X1, z1 - Z1].map((v) => Math.round((v - WALL_HALF) * 1e6) / 1e6);
  };
  // The sills, 7 cm out on the two walls with a window; elsewhere the casings and the corner boards, 2.4 cm.
  expect(reach('trim')).toEqual([0.025, 0.07, 0.07, 0.025]);
  expect(reach('plinth')).toEqual([0.03, 0.03, 0.03, 0.03]);
});
