/**
 * A roof over a plan with inner corners (`addRoofArea`): an L, a U, a T, a plus, steps, a house round
 * a courtyard. Each eave's slope was taken wherever it was the lowest, on past the eave's own ends: a
 * wing's roof went on down the slope of the eave at the inner corner, through the rooms to the floor
 * and under it. Here the roof is held to what it is meant to be over every plan: over the eave line by
 * the pitch times how far in a point is from the nearest eave, square to the walls.
 */
import { expect, test } from 'bun:test';
import type { RoofStyleDef } from '../src/content/content';
import type { WallEdge, WorldStructure } from '../src/core/protocol';
import { HousePlanner, WALL_HALF, WALL_HEIGHT, type HousePart } from '../src/render/shared/house';

/** The world's size, and where a plan's first tile stands on it. */
const W = 16;
const DEPTH = 16;
const AT = 3;
/** house.ts `OVERHANG`, `ROOF_THICKNESS`, `PITCH`, `FLAT_PITCH` and a flat roof's overhang. */
const OVERHANG = 0.4;
const THICK = 0.14;
const PITCH = Math.tan((34 * Math.PI) / 180);
const FLAT = { pitch: 0.03, overhang: 0.12 };

/** The plans, a row of tiles a line (`#`: indoors). */
const PLANS: Record<string, string[]> = {
  L: ['######', '######', '######', '###...', '###...', '###...'],
  U: ['###..###', '###..###', '###..###', '########', '########'],
  T: ['#######', '#######', '..###..', '..###..', '..###..'],
  plus: ['..##..', '..##..', '######', '######', '..##..', '..##..'],
  steps: ['####....', '####....', '######..', '######..', '########'],
  courtyard: ['#######', '#######', '##...##', '##...##', '#######', '#######'],
  // (A yard a tile wide: the eaves across it all but meet.)
  well: ['###', '#.#', '###'],
};

const ROOFS: RoofStyleDef[] = [
  { id: 'steep', label: 'Steep', shape: 'gable', pitch: 46 },
  { id: 'low', label: 'Low', shape: 'hip', pitch: 20 },
  { id: 'flat', label: 'Flat', shape: 'flat' },
];

type Plan = (x: number, z: number) => boolean;
const planOf = (name: string): Plan => (x, z) => PLANS[name][z - AT]?.[x - AT] === '#';

/** The plan as a one-storey house: a room over its tiles, walls round it; `style`: the roof the player chose (`ROOFS`), else the lot's own look. */
function roofOf(plan: Plan, style?: number): HousePart['geo'] {
  const walls: WallEdge[] = [];
  const rooms = new Array(W * DEPTH).fill(0);
  for (let z = 0; z < DEPTH; z++) {
    for (let x = 0; x < W; x++) {
      if (plan(x, z)) rooms[z * W + x] = 1;
      if (plan(x, z) !== plan(x, z - 1)) walls.push({ axis: 'h', x, z });
      if (plan(x, z) !== plan(x - 1, z)) walls.push({ axis: 'v', x, z });
    }
  }
  const plots = [{ id: 0, name: 'Home', x: 0, z: 0, w: W, d: DEPTH, public: false, entry: null, house: [AT, AT, W - AT, DEPTH - AT], ...(style === undefined ? {} : { roof: [style, 0] }) }];
  const world = { width: W, depth: DEPTH, walls, openings: [], diagonals: [], rooms, objects: [], plots, households: [], sims: [], exits: [] } as unknown as WorldStructure;
  const planner = new HousePlanner();
  planner.setLooks({ coverings: [], floors: [], doors: [], windows: [], roofs: ROOFS, roofColors: [{ id: 'slate', label: 'Slate', color: '#5C626A' }] });
  const built = planner.build(world, { x: 0, z: 0, w: W, d: DEPTH });
  const roof = built?.parts.find((p) => p.name === 'roof');
  if (!roof) throw new Error('no roof');
  // (No gable ends: a plan with inner corners is hipped all round.)
  expect(built?.parts.find((p) => p.name === 'gables')?.geo.idx.length ?? 0).toBe(0);
  return roof.geo;
}

type V3 = [number, number, number];
/** The roof's faces looking up, each a triangle. */
function faces(roof: HousePart['geo']): [V3, V3, V3][] {
  const { pos, nor, idx } = roof;
  const at = (v: number): V3 => [pos[v * 3], pos[v * 3 + 1], pos[v * 3 + 2]];
  const out: [V3, V3, V3][] = [];
  for (let t = 0; t < idx.length; t += 3) if (nor[idx[t] * 3 + 1] > 0.5) out.push([at(idx[t]), at(idx[t + 1]), at(idx[t + 2])]);
  return out;
}

/** The heights of the faces over (x, z): one over a roof that is one surface. */
function tops(tris: [V3, V3, V3][], x: number, z: number): number[] {
  const out: number[] = [];
  for (const [p, q, r] of tris) {
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

/** How far (x, z) is from the nearest tile `of` says yes to, square to the walls (the greater of its distances along x and z). */
function square(of: Plan, x: number, z: number): number {
  let least = Infinity;
  for (let tz = -1; tz <= DEPTH; tz++) {
    for (let tx = -1; tx <= W; tx++) {
      if (of(tx, tz)) least = Math.min(least, Math.max(tx - x, x - tx - 1, tz - z, z - tz - 1, 0));
    }
  }
  return least;
}

/** How far in from the eave line (x, z) is: the eaves stand `out` from the plan's edge. Under nought: past them. */
function inFrom(plan: Plan, out: number, x: number, z: number): number {
  return plan(Math.floor(x), Math.floor(z)) ? out + square((tx, tz) => !plan(tx, tz), x, z) : out - square(plan, x, z);
}

/** Places in a tile, off the lines its facets meet on (its diagonals and middle lines) and off an eave's mitre. */
const SPOTS = [[0.13, 0.31], [0.71, 0.58], [0.37, 0.86], [0.92, 0.17], [0.44, 0.07], [0.08, 0.66]];

/** The roofs held to: each plan under the lot's own pitch, and one plan under each roof a player can choose. */
const CASES: [name: string, style: number | undefined, pitch: number, overhang: number][] = [
  ...Object.keys(PLANS).map((name): [string, undefined, number, number] => [name, undefined, PITCH, OVERHANG]),
  ['U', 0, Math.tan((46 * Math.PI) / 180), OVERHANG],
  ['courtyard', 1, Math.tan((20 * Math.PI) / 180), OVERHANG],
  ['L', 2, FLAT.pitch, FLAT.overhang],
];

for (const [name, style, pitch, overhang] of CASES) {
  const about = `${name}${style === undefined ? '' : ` under a ${ROOFS[style].id} roof`}`;
  const plan = planOf(name);
  const out = WALL_HALF + overhang;
  /** The top of the roof at its eave line. */
  const eave = WALL_HEIGHT - overhang * pitch + THICK;

  test(`${about}: no part of the roof is under its eaves`, () => {
    const { pos } = roofOf(plan, style);
    let low = Infinity;
    for (let i = 1; i < pos.length; i += 3) low = Math.min(low, pos[i]);
    expect(low).toBeGreaterThan(eave - 1e-4);
  });

  test(`${about}: one surface over the plan and its overhang, as high as the pitch times the way in from the nearest eave`, () => {
    const tris = faces(roofOf(plan, style));
    const wrong: string[] = [];
    let [over, past] = [0, 0];
    for (let tz = AT - 1; tz <= AT + PLANS[name].length; tz++) {
      for (let tx = AT - 1; tx <= AT + PLANS[name][0].length; tx++) {
        for (const [dx, dz] of SPOTS) {
          const [x, z] = [tx + dx, tz + dz];
          const run = inFrom(plan, out, x, z);
          if (Math.abs(run) < 0.01) continue;
          const h = tops(tris, x, z);
          if (run < 0) {
            past++;
            if (h.length) wrong.push(`${x}, ${z}: a face past the eaves`);
            continue;
          }
          over++;
          if (h.length !== 1) wrong.push(`${x}, ${z}: ${h.length} faces`);
          else if (Math.abs(h[0] - (eave + pitch * run)) > 1e-4) wrong.push(`${x}, ${z}: at ${h[0].toFixed(3)}, not ${(eave + pitch * run).toFixed(3)}`);
        }
      }
    }
    expect(wrong).toEqual([]);
    expect(over).toBeGreaterThan(6 * PLANS[name].join('').split('#').length - 6);
    expect(past).toBeGreaterThan(0);
  });

  test(`${about}: every slope has the pitch`, () => {
    const off: string[] = [];
    for (const [p, q, r] of faces(roofOf(plan, style))) {
      const [ux, uy, uz, vx, vy, vz] = [q[0] - p[0], q[1] - p[1], q[2] - p[2], r[0] - p[0], r[1] - p[1], r[2] - p[2]];
      const n = [uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx];
      const rise = Math.hypot(n[0], n[2]) / Math.abs(n[1]);
      if (Math.abs(rise - pitch) > 1e-3) off.push(`${rise.toFixed(4)} at ${p.map((c) => c.toFixed(2)).join(', ')}`);
    }
    expect(off).toEqual([]);
  });

  test(`${about}: closed along its hips, valleys and ridges (every edge of a face is another's too, but along the eave line)`, () => {
    const key = (p: V3) => p.map((c) => Math.round(c * 1e4)).join(',');
    const edges = new Map<string, { count: number; a: V3; b: V3 }>();
    for (const tri of faces(roofOf(plan, style))) {
      for (let k = 0; k < 3; k++) {
        const [a, b] = [tri[k], tri[(k + 1) % 3]];
        const id = [key(a), key(b)].sort().join(' ');
        const e = edges.get(id) ?? { count: 0, a, b };
        e.count++;
        edges.set(id, e);
      }
    }
    const open: string[] = [];
    let rim = 0;
    for (const { count, a, b } of edges.values()) {
      if (count === 2) continue;
      const onEaves = count === 1 && [a, b].every((p) => Math.abs(p[1] - eave) < 1e-4 && Math.abs(inFrom(plan, out, p[0], p[2])) < 1e-4);
      if (onEaves) rim++;
      else open.push(`${count} faces on ${a.map((c) => c.toFixed(2)).join(', ')} to ${b.map((c) => c.toFixed(2)).join(', ')}`);
    }
    expect(open).toEqual([]);
    expect(rim).toBeGreaterThan(0);
  });
}

test('at an inner corner the slopes meet in a valley that runs up from it, at an outer one in a hip', () => {
  const plan = planOf('L');
  const tris = faces(roofOf(plan));
  const at = (x: number, z: number) => {
    const h = tops(tris, x, z);
    expect(h.length).toBeGreaterThan(0);
    // (On the line two facets meet on, both are over the point, at one height.)
    expect(Math.max(...h) - Math.min(...h)).toBeLessThan(1e-4);
    return h[0];
  };
  // The L's inner corner, its wings beyond it along x and along z, the house's body on its -x, -z side.
  const [cx, cz] = [AT + 3, AT + 3];
  const wallTop = WALL_HEIGHT + WALL_HALF * PITCH + THICK;
  expect(at(cx, cz)).toBeCloseTo(wallTop, 4);
  for (const t of [0.4, 0.8, 1.1]) {
    // Up the valley: a metre in is the pitch higher; either side of it the roof is higher still.
    expect(at(cx - t, cz - t)).toBeCloseTo(wallTop + t * PITCH, 4);
    expect(at(cx - t - 0.3, cz - t + 0.3)).toBeCloseTo(wallTop + (t + 0.3) * PITCH, 4);
    expect(at(cx - t + 0.3, cz - t - 0.3)).toBeCloseTo(wallTop + (t + 0.3) * PITCH, 4);
  }
  // Up the hip from the outer corner across the house from it: either side of it the roof is lower.
  for (const t of [0.5, 1, 1.4]) {
    expect(at(AT + t, AT + t)).toBeCloseTo(wallTop + t * PITCH, 4);
    expect(at(AT + t + 0.3, AT + t - 0.3)).toBeCloseTo(wallTop + (t - 0.3) * PITCH, 4);
    expect(at(AT + t - 0.3, AT + t + 0.3)).toBeCloseTo(wallTop + (t - 0.3) * PITCH, 4);
  }
});

test('a wing\'s roof is its own: a ridge along its middle at the height half its width gives, whatever the eaves round the corner', () => {
  // (The wing along z of the L, three tiles wide: where the roof went down to the floor.)
  const tris = faces(roofOf(planOf('L')));
  const ridge = WALL_HEIGHT - OVERHANG * PITCH + THICK + (1.5 + WALL_HALF + OVERHANG) * PITCH;
  for (const z of [AT + 3.6, AT + 4.2]) {
    const h = tops(tris, AT + 1.5, z);
    expect(Math.min(...h)).toBeCloseTo(ridge, 4);
    expect(Math.max(...h)).toBeCloseTo(ridge, 4);
  }
});

test('a courtyard\'s eaves drain into it: the roof falls to the yard from a ridge round it', () => {
  const plan = planOf('courtyard');
  const tris = faces(roofOf(plan));
  const eave = WALL_HEIGHT - OVERHANG * PITCH + THICK;
  // Across the two tiles between the outer wall and the yard, along x: up from the outer eave to the middle, down to the yard's.
  const z = AT + 3.3;
  const across = [-0.3, 0.4, 1, 1.6, 2.3].map((dx) => tops(tris, AT + dx, z)[0]);
  expect(across[1]).toBeGreaterThan(across[0] + 0.3);
  expect(across[2]).toBeGreaterThan(across[1] + 0.3);
  expect(across[3]).toBeLessThan(across[2] - 0.3);
  expect(across[4]).toBeLessThan(across[3] - 0.3);
  expect(across[4]).toBeGreaterThan(eave);
  // The yard itself stays open, but for the eaves' overhang.
  expect(tops(tris, AT + 3.5, AT + 3)).toEqual([]);
});
