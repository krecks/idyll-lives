/**
 * The benchmark's big world, made from a scene of today's size (`?bench=bigworld`, or `&tile=N` on
 * any scene): the town as the simulation has it, laid out `n × n` times side by side and given to the
 * renderer as a big world (`meta.world.size` 'big': the street streamed round the camera, its far
 * LOD, the far plane), so a ~1 km world can be drawn and measured before the simulation holds one.
 * Nothing of it goes in git: it is made from the scene in the page.
 *
 * Only the renderer sees the copies. The game plays the town it has: its residents, cars and the
 * viewed lot's walls and floors (`meshes`) are the first copy's, so a lot of another copy in view has
 * its house's shell from `walls` but no wall or floor meshes.
 */

import { groundDepth, type ObjectPlacement, type PlotInfo, type WorldStructure } from '../core/protocol';

/** Ids of the copies' things and plots: the original's plus this times the copy's index. */
const ID_STRIDE = 100_000;

/** The world `n × n` times (memoised: the same world given twice is the same copy). */
export function tileWorld(world: WorldStructure, n: number): WorldStructure {
  if (n <= 1) return world;
  if (last?.from === world && last.n === n) return last.to;
  const W = world.width;
  const D = groundDepth(world);
  const blocks = Math.max(1, Math.round(world.depth / Math.max(1, D)));
  const D2 = D * n;
  // A row of storey block `b` (row `b·D + zz`) in copy row `tz`: block `b` of the big world, `tz·D` further.
  const row = (z: number, tz: number) => {
    const b = Math.floor(z / D);
    return b * D2 + tz * D + (z - b * D);
  };
  const copies: [number, number, number][] = [];
  for (let tz = 0; tz < n; tz++) for (let tx = 0; tx < n; tx++) copies.push([tx, tz, tz * n + tx]);
  const many = <T>(list: readonly T[] | undefined, move: (item: T, tx: number, tz: number, k: number) => T): T[] => copies.flatMap(([tx, tz, k]) => (list ?? []).map((item) => move(item, tx, tz, k)));

  const plots = many(world.plots, (p, tx, tz, k): PlotInfo => ({
    ...p,
    id: p.id + k * ID_STRIDE,
    x: p.x + tx * W,
    z: p.z + tz * D,
    entry: p.entry && [p.entry[0] + tx * W, p.entry[1] + tz * D],
    house: p.house && [p.house[0] + tx * W, p.house[1] + tz * D, p.house[2] + tx * W, p.house[3] + tz * D],
  }));
  // Plot ids are their indices in the town: the copies' are their place in the big list.
  plots.forEach((p, i) => (p.id = i));
  const P = world.plots.length;
  const objects = many(world.objects, (o, tx, tz, k): ObjectPlacement => ({ ...o, id: o.id + k * ID_STRIDE, x: o.x + tx * W, z: row(o.z, tz) }));
  // (A typed array, row by row: the open world's sim sends rooms as a Uint16Array too.)
  const rooms = new Uint16Array(W * n * D2 * blocks);
  const src = Uint16Array.from(world.rooms);
  for (let z = 0; z < world.depth; z++) {
    const line = src.subarray(z * W, z * W + W);
    for (const [tx, tz] of copies) rooms.set(line, row(z, tz) * W * n + tx * W);
  }
  const m = world.meta;
  const shift = <R extends { x: number; z: number }>(r: R, tx: number, tz: number): R => ({ ...r, x: r.x + tx * W, z: r.z + tz * D });
  const meta = m && {
    ...m,
    streets: many(m.streets, shift),
    paths: many(m.paths, shift),
    roads: m.roads && many(m.roads, (r, tx, tz) => ({ ...r, points: r.points.map(([x, z]): [number, number] => [x + tx * W, z + tz * D]) })),
    greens: m.greens && many(m.greens, (g, tx, tz) => g.map(([x, z]): [number, number] => [x + tx * W, z + tz * D])),
    facing: m.facing && copies.flatMap(() => m.facing!),
    home: null,
    world: m.world && { ...m.world, size: 'big' as const, fields: m.world.fields && many(m.world.fields, shift) },
  };
  const to: WorldStructure = {
    ...world,
    width: W * n,
    depth: D2 * blocks,
    storeyDepth: world.storeyDepth === undefined ? undefined : D2,
    plots,
    objects,
    // (`rooms` is typed `number[]` until the sim-grid branch makes it the contract's Uint16Array; read by index alike.)
    rooms: rooms as unknown as number[],
    meta,
    // The player's household on the first copy (its plot index is the original's).
    households: world.households,
    exits: world.exits,
    walls: many(world.walls, (e, tx, tz) => ({ ...e, x: e.x + tx * W, z: row(e.z, tz) })),
    openings: many(world.openings, (o, tx, tz) => ({ ...o, x: o.x + tx * W, z: row(o.z, tz) })),
    diagonals: world.diagonals && many(world.diagonals, (d, tx, tz) => ({ ...d, x: d.x + tx * W, z: row(d.z, tz) })),
    fences: world.fences && many(world.fences, (f, tx, tz) => ({ ...f, x: f.x + tx * W, z: row(f.z, tz) })),
    floors: world.floors && many(world.floors, ([x, z, c], tx, tz): [number, number, number] => [x + tx * W, row(z, tz), c]),
    garages: world.garages && many(world.garages, (g, tx, tz, k) => ({ ...g, plot: g.plot + k * P, x0: g.x0 + tx * W, x1: g.x1 + tx * W, z0: g.z0 + tz * D, z1: g.z1 + tz * D })),
    porches: world.porches && many(world.porches, (p, tx, tz, k) => ({ ...p, plot: p.plot + k * P, x: p.x + tx * W, z: p.z + tz * D })),
    porchPieces: world.porchPieces && many(world.porchPieces, (p, tx, tz, k) => ({ ...p, plot: p.plot + k * P, x: p.x + tx * W, z: p.z + tz * D })),
  };
  last = { from: world, n, to };
  return to;
}
let last: { from: WorldStructure; n: number; to: WorldStructure } | null = null;
