/**
 * The land's debug fixture (`?landFixture=valley,river,island` on the dev server): until worldgen writes them, a
 * big world gets a relief grid, a river with bridges or an island's sea here, hand-made round its main town, so
 * the renderer's side of them can be seen in the game:
 *
 * - `valley`: a relief grid (`WorldMeta.relief`, 16 m cells, decimetres) of a valley along the main town, its
 *   sides climbing from 70 m off its floor to 150 m at the world's edge (the near, low-z side 0.45 as high), and
 *   the land's `slope` (0.75 among mountains, else 0.4).
 * - `river`: a stretch of river (a water zone 18 m wide, 90 m long) where a country road crosses open land
 *   nearest the main town, and a `bridge` span on the road over it.
 * - `island`: the sea all round (a water zone touching all four edges: the world's square with a superellipse of
 *   land cut out of it), a `bridge` (a causeway) on each way out over it.
 *
 * Only the drawing sees it (the simulation keeps its land and water). Debug only: nothing outside
 * `?landFixture` calls it; each world's meta is changed once.
 */

import { groundDepth, type RoadPlan, type WorldMeta, type WorldStructure, type WorldZone } from '../core/protocol';
import { inside, lengths, pointAt } from '../game/roads';

const params = typeof location === 'undefined' ? new URLSearchParams() : new URLSearchParams(location.search);

/** Which fixtures the URL asks for (none outside the dev server). */
export const LAND_FIXTURE: ReadonlySet<string> = new Set(import.meta.env.DEV ? (params.get('landFixture') ?? '').split(',').filter(Boolean) : []);

type Pt = [number, number];
const done = new WeakSet<WorldMeta>();

/** Gives a big world the fixtures `which` names (the URL's: `LAND_FIXTURE`; once; a world that isn't big is left alone). */
export function applyLandFixture(world: WorldStructure, which: ReadonlySet<string> = LAND_FIXTURE): void {
  const meta = world.meta;
  if (!meta?.world || meta.world.size !== 'big' || done.has(meta)) return;
  done.add(meta);
  const [W, D] = [world.width, groundDepth(world)];
  // The main town's middle: its square's, else its area's (as the bench's town view has it).
  const square = meta.centre?.find((c) => c.area === 0)?.poly ?? meta.centre?.[0]?.poly;
  const main = meta.areas?.[0];
  const [cx, cz] = square?.length ? [square.reduce((s, p) => s + p[0], 0) / square.length, square.reduce((s, p) => s + p[1], 0) / square.length] : main ? [main.x + main.w / 2, main.z + main.d / 2] : [W / 2, D / 2];
  meta.zones ??= [];
  if (which.has('river')) river(world, meta, cx, cz);
  if (which.has('island')) island(meta, W, D);
  if (which.has('valley')) {
    meta.relief = valley(W, D, cz, meta.zones.filter((z) => z.kind === 'water'));
    meta.world.landscape = { ...meta.world.landscape, slope: meta.world.landscape.mountains ? 0.75 : 0.4 };
  }
  console.info('[landFixture]', [...which].join(', '), { relief: !!meta.relief, bridges: (meta.roads ?? []).filter((r) => r.bridge).map((r) => [r.name, r.bridge]) });
}

/** The valley's relief grid round the line z = `cz`. */
function valley(W: number, D: number, cz: number, water: WorldZone[]): NonNullable<WorldMeta['relief']> {
  const cell = 16;
  const [cols, rows] = [Math.ceil(W / cell), Math.ceil(D / cell)];
  const h: number[] = [];
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const [x, z] = [(i + 0.5) * cell, (j + 0.5) * cell];
      const reach = z < cz ? cz : D - cz;
      const t = Math.min(1, Math.max(0, (Math.abs(z - cz) - 70) / Math.max(1, reach - 70))) ** 1.4;
      const wet = water.some((w) => inside(w.poly, x, z));
      h.push(wet ? 0 : Math.round(150 * t * (z < cz ? 0.45 : 1) * 10));
    }
  }
  return { cell, cols, rows, h };
}

/** Arc lengths along `road` where it first enters and last leaves `wet` (null: it doesn't), 2 m wider each way. */
function span(road: RoadPlan, wet: (x: number, z: number) => boolean): [number, number] | null {
  const ls = lengths(road.points);
  const total = ls[ls.length - 1];
  let [s0, s1] = [Infinity, -Infinity];
  for (let s = 0; s <= total; s += 0.5) {
    const { p } = pointAt(road.points, s, ls);
    if (!wet(p[0], p[1])) continue;
    s0 = Math.min(s0, s);
    s1 = Math.max(s1, s);
  }
  return s1 < s0 ? null : [Math.max(0, s0 - 2), Math.min(total, s1 + 2)];
}

/**
 * A stretch of river (18 m wide, 90 m long, square to the road) where a country road crosses open land nearest the
 * main town (clear of every lot, building and other road), and a bridge on the road over it.
 */
function river(world: WorldStructure, meta: WorldMeta, cx: number, cz: number): void {
  const [wide, long] = [18, 90];
  const roads = meta.roads ?? [];
  const boxes = [...world.plots.map((p) => [p.x, p.z, p.x + p.w, p.z + p.d]), ...(meta.backdrop ?? []).map((b) => [Math.min(...b.poly.map((q) => q[0])), Math.min(...b.poly.map((q) => q[1])), Math.max(...b.poly.map((q) => q[0])), Math.max(...b.poly.map((q) => q[1]))])];
  let best: { d: number; road: RoadPlan; poly: Pt[] } | null = null;
  for (const road of roads) {
    if (road.kind !== 'country') continue;
    const ls = lengths(road.points);
    for (let s = 30; s < ls[ls.length - 1] - 30; s += 6) {
      const { p, t } = pointAt(road.points, s, ls);
      // (Within 190 m of the main town's middle each way, as the bench's camera can be moved, `tx`, `tz`, if any is.)
      const d = Math.hypot(p[0] - cx, p[1] - cz) + (Math.max(Math.abs(p[0] - cx), Math.abs(p[1] - cz)) > 190 ? 1e4 : 0);
      if (best && d >= best.d) continue;
      // Square to the road's way, as near as the grid's axes go.
      const alongX = Math.abs(t[0]) > Math.abs(t[1]);
      const [hx, hz] = alongX ? [wide / 2, long / 2] : [long / 2, wide / 2];
      const [x0, z0, x1, z1] = [p[0] - hx, p[1] - hz, p[0] + hx, p[1] + hz];
      if (boxes.some(([a, b, c, e]) => c > x0 - 4 && a < x1 + 4 && e > z0 - 4 && b < z1 + 4)) continue;
      const others = roads.some((r) => r !== road && r.points.some(([x, z]) => x > x0 - 3 && x < x1 + 3 && z > z0 - 3 && z < z1 + 3));
      if (others) continue;
      best = {
        d,
        road,
        poly: [
          [x0, z0],
          [x1, z0],
          [x1, z1],
          [x0, z1],
        ],
      };
    }
  }
  if (!best) return console.warn('[landFixture] no open land for a river by a country road');
  const poly = best.poly;
  meta.zones!.push({ kind: 'water', poly });
  const s = span(best.road, (x, z) => inside(poly, x, z));
  if (s) {
    best.road.bridge = s;
    best.road.name ??= 'Fixture Bridge';
  }
}

/** The sea round an island, and a causeway on each way out over it. */
function island(meta: WorldMeta, W: number, D: number): void {
  const n = 96;
  const coast: Pt[] = Array.from({ length: n }, (_, k) => {
    const a = (k / n) * Math.PI * 2;
    const [c, s] = [Math.cos(a), Math.sin(a)];
    const r = W * 0.44 * (Math.abs(c) ** 3 + Math.abs(s) ** 3) ** (-1 / 3) * (1 + 0.05 * Math.sin(a * 5 + 1));
    return [W / 2 + c * r, D / 2 + s * r];
  });
  meta.zones = [
    ...meta.zones!.filter((z) => z.kind !== 'water'),
    {
      kind: 'water',
      poly: [
        [-40, -40],
        [W + 40, -40],
        [W + 40, D + 40],
        [-40, D + 40],
      ],
    },
    { kind: 'water', poly: coast },
  ];
  for (const r of meta.roads ?? []) {
    if (r.kind !== 'country' || !r.ends.includes('out')) continue;
    const s = span(r, (x, z) => !inside(coast, x, z));
    if (s) {
      r.bridge = s;
      r.name ??= 'Fixture Causeway';
    }
  }
}
