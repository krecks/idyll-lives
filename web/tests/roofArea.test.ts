/**
 * Where the storey above stands on the middle of this one, the roof left round it is an L, a U or a
 * ring. Roofed a rectangle at a time (`indoorRects`), each got its own hips, ridge and eaves, and
 * their eaves ran into the next rectangle: two surfaces over the same tile, butting together at the
 * corners (`addRoofArea` roofs the whole area as one instead).
 */
import { expect, test } from 'bun:test';
import type { WallEdge, WorldStructure } from '../src/core/protocol';
import { HousePlanner, type HousePlan } from '../src/render/shared/house';

/** Rows a storey. */
const DEPTH = 12;
const W = 12;
/** The house's walls: a box on both storeys. */
const X0 = 2;
const Z0 = 2;
const X1 = 8;
const Z1 = 7;
/** Where the storey above stands: a tile in from the walls, so the roof below is a rim one wide. */
const UX0 = 3;
const UZ0 = 3;
const UX1 = 7;
const UZ1 = 6;

/** A box house on two storeys: rooms all over the ground, the upper one on the middle tiles only. */
function twoStoreys(): WorldStructure {
  const walls: WallEdge[] = [];
  const rooms = new Array(W * DEPTH * 2).fill(0);
  for (let k = 0; k < 2; k++) {
    const z0 = Z0 + k * DEPTH;
    const z1 = Z1 + k * DEPTH;
    for (let x = X0; x < X1; x++) walls.push({ axis: 'h', x, z: z0 }, { axis: 'h', x, z: z1 });
    for (let z = z0; z < z1; z++) walls.push({ axis: 'v', x: X0, z }, { axis: 'v', x: X1, z });
  }
  for (let z = Z0; z < Z1; z++) for (let x = X0; x < X1; x++) rooms[z * W + x] = 1;
  for (let z = UZ0; z < UZ1; z++) for (let x = UX0; x < UX1; x++) rooms[(z + DEPTH) * W + x] = 2;
  return { width: W, depth: DEPTH * 2, storeyDepth: DEPTH, storeys: 2, walls, openings: [], diagonals: [], rooms, objects: [], plots: [], households: [], sims: [], exits: [] } as unknown as WorldStructure;
}

/** The ground storey's plan, roofed but where the storey above stands. */
function ring(): HousePlan {
  const world = twoStoreys();
  const room = (x: number, z: number) => (x >= 0 && z >= 0 && x < W && z < DEPTH * 2 ? world.rooms[z * W + x] : 0);
  const plan = new HousePlanner().build(world, { x: 0, z: 0, w: W, d: DEPTH }, { covered: (x, z) => room(x, z + DEPTH) !== 0 });
  if (!plan) throw new Error('no house');
  return plan;
}

/** The heights of the roof's faces looking up, over (x, z): one over a roof that is one surface. */
function tops(plan: HousePlan, x: number, z: number): number[] {
  const roof = plan.parts.find((p) => p.name === 'roof');
  if (!roof) throw new Error('no roof');
  const { pos, nor, idx } = roof.geo;
  const at = (v: number) => [pos[v * 3], pos[v * 3 + 1], pos[v * 3 + 2]];
  const out: number[] = [];
  for (let t = 0; t < idx.length; t += 3) {
    const [a, b, c] = [idx[t], idx[t + 1], idx[t + 2]];
    if (nor[a * 3 + 1] <= 0.5) continue;
    const [p, q, r] = [at(a), at(b), at(c)];
    // Barycentric, in plan: is (x, z) inside this triangle's footprint?
    const d = (q[0] - p[0]) * (r[2] - p[2]) - (r[0] - p[0]) * (q[2] - p[2]);
    if (Math.abs(d) < 1e-9) continue;
    const u = ((q[0] - x) * (r[2] - z) - (r[0] - x) * (q[2] - z)) / d;
    const v = ((r[0] - x) * (p[2] - z) - (p[0] - x) * (r[2] - z)) / d;
    const w = 1 - u - v;
    if (u < -1e-9 || v < -1e-9 || w < -1e-9) continue;
    out.push(u * p[1] + v * q[1] + w * r[1]);
  }
  return out;
}

test('round the storey above, the roof is one surface: over every tile of it, one face', () => {
  const plan = ring();
  const twice: string[] = [];
  let tiles = 0;
  for (let z = Z0; z < Z1; z++) {
    for (let x = X0; x < X1; x++) {
      if (x >= UX0 && x < UX1 && z >= UZ0 && z < UZ1) continue; // the storey above stands here
      tiles++;
      // (Off the lines the tile's four triangles meet on, its diagonals: a sample on one of them
      // lies in two of the triangles.)
      for (const [dx, dz] of [[0.45, 0.15], [0.55, 0.85], [0.15, 0.55], [0.85, 0.45], [0.2, 0.35], [0.8, 0.65]]) {
        const h = tops(plan, x + dx, z + dz);
        if (h.length !== 1) twice.push(`${x + dx}, ${z + dz}: ${h.length} faces`);
      }
    }
  }
  expect(tiles).toBe((X1 - X0) * (Z1 - Z0) - (UX1 - UX0) * (UZ1 - UZ0));
  expect(twice).toEqual([]);
});

test('the ring has no gable ends (they would cut across it): a hip roof all round', () => {
  const gables = ring().parts.find((p) => p.name === 'gables');
  expect(gables?.geo.idx.length ?? 0).toBe(0);
});

test('its eaves stand half a wall plus the overhang out from the walls, mitred at the corners', () => {
  const roof = ring().parts.find((p) => p.name === 'roof');
  const { pos } = roof!.geo;
  const [xs, zs] = [[], []] as [number[], number[]];
  for (let i = 0; i < pos.length; i += 3) {
    xs.push(pos[i]);
    zs.push(pos[i + 2]);
  }
  // WALL_HALF (0.07) + OVERHANG (0.4): one eave line, not two where a straight run carries on.
  const d = 0.47;
  expect(Math.min(...xs)).toBeCloseTo(X0 - d, 6);
  expect(Math.max(...xs)).toBeCloseTo(X1 + d, 6);
  expect(Math.min(...zs)).toBeCloseTo(Z0 - d, 6);
  expect(Math.max(...zs)).toBeCloseTo(Z1 + d, 6);
});

test('a rim one tile wide is a ridge, not a flat shelf at the eaves', () => {
  // The storey above stands a tile in from the walls, so every rim tile's four corners lie at the
  // eaves' height and only its middle rises.
  const plan = ring();
  // The rim tile in front of the storey above, between the wall's eave and the hole's.
  const corner = tops(plan, 5.25, Z0 + 0.15);
  const mid = tops(plan, 5.5, Z0 + 0.5);
  expect(corner.length).toBe(1);
  expect(mid.length).toBeGreaterThan(0);
  expect(Math.min(...mid)).toBeGreaterThan(corner[0] + 0.2);
});

test('a plain rectangle keeps the classic roof (a handful of faces, not a tile at a time)', () => {
  const world = twoStoreys();
  const plan = new HousePlanner().build(world, { x: 0, z: DEPTH, w: W, d: DEPTH }, { upper: true, covered: () => false });
  if (!plan) throw new Error('no house');
  const roof = plan.parts.find((p) => p.name === 'roof');
  expect(roof!.geo.idx.length / 3).toBeLessThan(24);
});
