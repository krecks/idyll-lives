// Hand-made big worlds for the landscape's tests (`render/shared/nature.ts`, the street's bridges, the ground map):
// a town in a valley with hamlets, roads, woods, meadows, fields, an orchard and a pond; the same with a relief grid
// (a valley whose sides climb 60 to 150 m); a river with a bridge over it; an island whose sea touches all four edges.
// Small enough to lay out in a test, made of nothing the generator writes.
import { DEFAULT_LANDSCAPE, type LandscapeParams, type PlotInfo, type RoadPlan, type WorldMeta, type WorldStructure, type WorldZone } from '../../src/core/protocol';

/** The fixtures' side (m). */
export const SIDE = 512;

/** A plot `w` × `d` at (x, z). */
function plot(id: number, x: number, z: number, w: number, d: number): PlotInfo {
  return { id, name: `Plot ${id}`, x, z, w, d, public: false, entry: null, house: null };
}

/** A rectangle as a zone's outline. */
function rect(x0: number, z0: number, x1: number, z1: number): [number, number][] {
  return [
    [x0, z0],
    [x1, z0],
    [x1, z1],
    [x0, z1],
  ];
}

/** A rough circle (`n` corners) as a zone's outline. */
function blob(cx: number, cz: number, r: number, n = 24): [number, number][] {
  return Array.from({ length: n }, (_, k) => {
    const a = (k / n) * Math.PI * 2;
    const rr = r * (1 + 0.12 * Math.sin(a * 3));
    return [cx + Math.cos(a) * rr, cz + Math.sin(a) * rr] as [number, number];
  });
}

/** A world of `meta` round `plots`, with nothing indoors (the landscape reads none of that). */
export function bigWorld(meta: WorldMeta, plots: PlotInfo[], side = SIDE): WorldStructure {
  return {
    version: 1,
    lotVersion: 1,
    mode: 'living',
    width: side,
    depth: side,
    objects: [],
    sims: [],
    households: [],
    plots,
    exits: [],
    rooms: new Array(side * side).fill(0),
    walls: [],
    openings: [],
    meta,
    meshes: {} as WorldStructure['meshes'],
  } as WorldStructure;
}

/** The town's plots: two rows along its high street (z 240..272), and a hamlet's at (120, 400). */
function townPlots(): PlotInfo[] {
  const out: PlotInfo[] = [];
  let id = 0;
  for (let x = 200; x < 312; x += 16) {
    out.push(plot(id++, x, 222, 14, 18));
    out.push(plot(id++, x, 272, 14, 18));
  }
  for (let x = 100; x < 148; x += 16) out.push(plot(id++, x, 384, 14, 14));
  return out;
}

/** The roads: the high street, a country road to the hamlet, and a way out east (past the edge). */
function townRoads(): RoadPlan[] {
  return [
    { points: [[180, 256], [330, 256]], width: 12, ends: ['join', 'join'], kind: 'street', area: 0 },
    { points: [[180, 256], [140, 300], [124, 380]], width: 7, ends: ['join', 'dead'], kind: 'country' },
    { points: [[330, 256], [420, 240], [SIDE, 236]], width: 7, ends: ['join', 'out'], kind: 'country' },
  ];
}

/** Woods, meadows, a hill, fields, an orchard and a pond, as a big world has them. */
function townZones(): WorldZone[] {
  return [
    { kind: 'woods', poly: blob(400, 120, 80) },
    { kind: 'woods', poly: rect(0, 0, 90, 200) },
    { kind: 'meadow', poly: blob(120, 120, 60) },
    { kind: 'hill', poly: blob(420, 420, 90) },
    { kind: 'fields', poly: rect(300, 320, 480, 380) },
    { kind: 'orchard', poly: rect(220, 320, 260, 350) },
    { kind: 'water', poly: blob(60, 300, 30) },
  ];
}

/** A big world's look: the countryside's land, with `extra` over it. */
function look(extra: Partial<LandscapeParams> = {}, kind = 'country'): NonNullable<WorldMeta['world']> {
  return { kind, size: 'big', landscape: { ...DEFAULT_LANDSCAPE, hills: 11, rise: 44, rim: 28, rimFrom: 60, ...extra } };
}

/** A big world as saves before relief have it: zones, no relief. */
export function plainWorld(extra: Partial<LandscapeParams> = {}, kind = 'country'): WorldStructure {
  return bigWorld({ seed: 4242, name: 'Fixton', roads: townRoads(), zones: townZones(), world: look(extra, kind) }, townPlots());
}

/** The mountains' look (`mountains` 140 m), as a classic mountain world has it. */
export const MOUNTAINS: Partial<LandscapeParams> = { hills: 12, rise: 24, rim: 40, rimFrom: 50, forest: 0.1, treeline: 2, conifers: 0.9, mountains: 140 };

/**
 * A relief grid as the generator writes one (`WorldMeta.relief`): 16 m cells, decimetres over the built level at
 * each cell's middle, 0 on the water; here a valley along z = SIDE / 2 whose sides climb from 0 at 70 m off its
 * floor to `top` m at the world's edge (60 to 150 m), the valley's low-z side `low` as high (the generator's 0.45).
 */
export function valleyRelief(top: number, low = 0.45, side = SIDE, water: WorldZone[] = townZones().filter((z) => z.kind === 'water')): NonNullable<WorldMeta['relief']> {
  const cell = 16;
  const cols = Math.ceil(side / cell);
  const rows = cols;
  const h: number[] = [];
  const wet = (x: number, z: number) => water.some((zone) => inside(zone.poly, x, z));
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const [x, z] = [(i + 0.5) * cell, (j + 0.5) * cell];
      const off = Math.abs(z - side / 2);
      const t = Math.min(1, Math.max(0, (off - 70) / (side / 2 - 70))) ** 1.4;
      const m = top * t * (z < side / 2 ? low : 1);
      h.push(wet(x, z) ? 0 : Math.round(m * 10));
    }
  }
  return { cell, cols, rows, h };
}

/** Whether (x, z) is inside `poly` (even-odd). */
function inside(poly: readonly [number, number][], x: number, z: number): boolean {
  let c = false;
  for (let a = 0, b = poly.length - 1; a < poly.length; b = a++) {
    const [[xa, za], [xb, zb]] = [poly[a], poly[b]];
    if (za > z !== zb > z && x < ((xb - xa) * (z - za)) / (zb - za) + xa) c = !c;
  }
  return c;
}

/** The town in a valley: `plainWorld` with a relief grid (sides `top` m high), `slope` and the rest given. */
export function reliefWorld(top = 120, extra: Partial<LandscapeParams> = { slope: 0.4 }, kind = 'country'): WorldStructure {
  const world = plainWorld(extra, kind);
  world.meta!.relief = valleyRelief(top);
  return world;
}

/** The river's band (z), across the world from west to east. */
export const RIVER = { z0: 150, z1: 168 } as const;

/**
 * A river across the world (a water zone `RIVER` wide) with a bridge over it: a country road from the town's high
 * street north over the water to a hamlet, its deck (`bridge`) from 2 m before the water to 2 m past it; and a
 * second country road (the way on) joining the bridge's far end at a two-arm joint.
 */
export function riverWorld(): WorldStructure {
  const world = plainWorld({ beach: 3 }, 'river');
  const meta = world.meta!;
  meta.zones = [...meta.zones!.filter((z) => z.kind !== 'water'), { kind: 'water', poly: rect(-20, RIVER.z0, SIDE + 20, RIVER.z1) }];
  // The bridge: straight, from 12 m inland of the south bank (z 180) to 12 m inland of the north bank (z 138).
  const [south, north] = [RIVER.z1 + 12, RIVER.z0 - 12];
  const s0 = south - (RIVER.z1 + 2);
  const s1 = south - (RIVER.z0 - 2);
  meta.roads = [
    ...meta.roads!,
    { points: [[256, 222], [256, south]], width: 7, ends: ['join', 'join'], kind: 'country' },
    { points: [[256, south], [256, north]], width: 7, ends: ['join', 'join'], kind: 'country', name: 'Fix Bridge', bridge: [s0, s1] },
    { points: [[256, north], [250, 100], [220, 60]], width: 7, ends: ['join', 'dead'], kind: 'country' },
  ];
  // (Two-arm joints at the bridge's ends, as if the simulation sent them: it doesn't, `traffic::SIGNAL_ARMS`.)
  const n = meta.roads.length;
  const arm = (road: number, dz: number) => ({ road, dir: [0, dz] as [number, number], width: 7, walkFrom: 3, walkTo: 6, stop: 6.5, light: null });
  world.junctions = [
    { x: 256, z: south, arms: [arm(n - 3, 1), arm(n - 2, -1)] },
    { x: 256, z: north, arms: [arm(n - 2, 1), arm(n - 1, -1)] },
  ];
  return world;
}

/**
 * An island: the sea all round (a water zone touching all four edges: the square past the world's edge with the
 * island cut out of it, as two outlines read even-odd), the town on it, and a causeway east out over the sea to
 * past the edge (`bridge` over its wet stretch).
 */
export function islandWorld(): WorldStructure {
  const world = plainWorld({}, 'island');
  const meta = world.meta!;
  const n = 64;
  const coast: [number, number][] = Array.from({ length: n }, (_, k) => {
    const a = (k / n) * Math.PI * 2;
    const [c, s] = [Math.cos(a), Math.sin(a)];
    // A superellipse (power 3), its half-axes 0.44 of the side.
    const r = SIDE * 0.44 * (Math.abs(c) ** 3 + Math.abs(s) ** 3) ** (-1 / 3);
    return [SIDE / 2 + c * r, SIDE / 2 + s * r];
  });
  meta.zones = [...meta.zones!.filter((z) => z.kind !== 'water'), { kind: 'water', poly: rect(-40, -40, SIDE + 40, SIDE + 40) }, { kind: 'water', poly: coast }];
  const coastX = Math.max(...coast.map((p) => p[0]));
  const from = 330;
  meta.roads = [
    ...meta.roads!.filter((r) => !r.ends.includes('out')),
    { points: [[330, 256], [SIDE, 256]], width: 7, ends: ['join', 'out'], kind: 'country', name: 'Fix Causeway', bridge: [coastX - 2 - from, SIDE - from] },
  ];
  return world;
}
