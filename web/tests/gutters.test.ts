import { describe, expect, test } from 'bun:test';
import type { Opening, WallEdge, WorldStructure } from '../src/core/protocol';
import { Geo } from '../src/render/shared/geometry';
import { HousePlanner, houseScheme, type HousePart } from '../src/render/shared/house';
import { WALL_HEIGHT } from '../src/render/shared/walls';

/**
 * Gutters and downpipes: a gutter hangs along every level eave over the open (a hip's all round, a gable's along
 * its two long sides), and a downpipe comes down from it at two of the house's corners, on plain wall only. They are
 * the roof's trim: drawn with the roof, in its draw. The far house has none.
 */

const [W, D] = [24, 20];
/** A box house (wall centre lines x0..x1, z0..z1) on a lot at (`lot`, 0), with `windows` in its walls; `upper`: a storey on it, over the tiles `upper` says. */
function world(lot: number, windows: Opening[] = [], upper?: (x: number, z: number) => boolean): WorldStructure {
  const [x0, z0, x1, z1] = [lot + 4, 5, lot + 12, 11];
  const walls: WallEdge[] = [];
  const rooms = new Array(W * D * (upper ? 2 : 1)).fill(0);
  const box = (inside: (x: number, z: number) => boolean, row: number, id: number) => {
    for (let z = 0; z < D; z++) {
      for (let x = 0; x < W; x++) {
        if (inside(x, z)) rooms[(z + row) * W + x] = id;
        if (inside(x, z) !== inside(x, z - 1)) walls.push({ axis: 'h', x, z: z + row });
        if (inside(x, z) !== inside(x - 1, z)) walls.push({ axis: 'v', x, z: z + row });
      }
    }
  };
  const ground = (x: number, z: number) => x >= x0 && x < x1 && z >= z0 && z < z1;
  box(ground, 0, 1);
  if (upper) box((x, z) => ground(x, z) && upper(x, z), D, 2);
  const plots = [{ id: 0, name: 'Home', x: lot, z: 0, w: 16, d: D, public: false, entry: null, house: [x0, z0, x1, z1], ...(upper ? { storeys: 2 } : {}) }];
  return { width: W, depth: D * (upper ? 2 : 1), ...(upper ? { storeys: 2, storeyDepth: D } : {}), walls, openings: windows, diagonals: [], rooms, objects: [], plots, households: [], sims: [], exits: [] } as unknown as WorldStructure;
}
const lotWith = (look: (s: ReturnType<typeof houseScheme>) => boolean): number => {
  for (let x = 0; x < 8; x++) if (look(houseScheme(x, 0))) return x;
  throw new Error('no such lot');
};
const HIP = lotWith((s) => s.hip);
const GABLE = lotWith((s) => !s.hip);

function trimOf(w: WorldStructure, lot: number, storey = 0): HousePart['geo'] {
  const covered = (x: number, z: number) => storey === 0 && (w.storeys ?? 1) > 1 && w.rooms[(z + D) * W + x] !== 0;
  const built = new HousePlanner().build(w, { x: lot, z: storey * D, w: 16, d: D }, storey ? { upper: true } : { covered });
  return built!.parts.find((p) => p.name === 'roofTrim')!.geo;
}
/** The gutters' troughs: the roof trim's faces looking up, each as its box in plan [x0, z0, x1, z1] and its height. */
function troughs(trim: HousePart['geo']): { box: number[]; y: number }[] {
  const { pos, nor, idx } = trim;
  const out: { box: number[]; y: number }[] = [];
  for (let t = 0; t < idx.length; t += 6) {
    if (nor[idx[t] * 3 + 1] < 0.99) continue;
    const vs = [...idx.subarray(t, t + 6)];
    out.push({ box: [Math.min(...vs.map((v) => pos[v * 3])), Math.min(...vs.map((v) => pos[v * 3 + 2])), Math.max(...vs.map((v) => pos[v * 3])), Math.max(...vs.map((v) => pos[v * 3 + 2]))], y: pos[vs[0] * 3 + 1] });
  }
  return out;
}
/** The downpipes: what of the roof's trim hangs under the eaves, as each pipe's place (x, z, to the centimetre) and its foot. */
function pipes(trim: HousePart['geo'], eaves = WALL_HEIGHT - 0.6): { x: number; z: number; foot: number }[] {
  const { pos } = trim;
  const out: { x: number; z: number; foot: number }[] = [];
  for (let i = 0; i < pos.length; i += 3) {
    if (pos[i + 1] > eaves) continue;
    // (A pipe's six corners are within its radius of its middle.)
    const at = out.find((p) => Math.hypot(p.x - pos[i], p.z - pos[i + 2]) < 0.15);
    if (at) at.foot = Math.min(at.foot, pos[i + 1]);
    else out.push({ x: pos[i], z: pos[i + 2], foot: pos[i + 1] });
  }
  return out;
}

describe('gutters', () => {
  test('a hip roof has one along each of its four eaves, a gable roof along its two long ones', () => {
    const hip = troughs(trimOf(world(HIP), HIP));
    expect(hip).toHaveLength(4);
    const gable = troughs(trimOf(world(GABLE), GABLE));
    expect(gable).toHaveLength(2);
    // (The house is longer along x: the gable's gutters run along x, past its walls at both ends.)
    for (const g of gable) expect(g.box[2] - g.box[0]).toBeGreaterThan(8.5);
    // Under the roof's edge, outside the walls.
    for (const g of [...hip, ...gable]) expect(g.y).toBeGreaterThan(WALL_HEIGHT - 0.3);
  });

  test('none far off', () => {
    const far = new Geo();
    new HousePlanner().farHouse(far, { x: HIP, z: 0, house: [HIP + 4, 5, HIP + 12, 11] });
    expect(far.idx.length / 3).toBe(14);
  });

  test('a roof round a storey above has gutters over the open only, not along that storey\'s walls', () => {
    const w = world(HIP, [], (x, z) => x >= HIP + 6 && x < HIP + 10 && z >= 7 && z < 9);
    const all = troughs(trimOf(w, HIP));
    // The ring's four outer eaves; nothing inside the storey above (x 6..10, z 7..9 of the lot).
    expect(all).toHaveLength(4);
    for (const g of all) expect(g.box[0] < HIP + 4 || g.box[2] > HIP + 12 || g.box[1] < 5 || g.box[3] > 11).toBe(true);
  });
});

describe('downpipes', () => {
  test('two, at opposite corners of the house, down to the ground', () => {
    const found = pipes(trimOf(world(GABLE), GABLE));
    expect(found).toHaveLength(2);
    const [a, b] = found.sort((p, q) => p.x - q.x);
    expect(a.x).toBeLessThan(GABLE + 5);
    expect(b.x).toBeGreaterThan(GABLE + 11);
    expect(Math.abs(a.z - b.z)).toBeGreaterThan(5.5);
    for (const p of found) expect(p.foot).toBeCloseTo(0.1, 6);
  });

  test('not across a window: at the other end of its eave, or not at all', () => {
    // A window in the corner tile where the first pipe would stand (the wall along x at z 5, its first tile).
    const other = pipes(trimOf(world(GABLE, [{ axis: 'h', x: GABLE + 4, z: 5, kind: 'window' }]), GABLE));
    expect(other).toHaveLength(2);
    expect(other.filter((p) => p.z < 5).every((p) => p.x > GABLE + 11)).toBe(true);
    const none = pipes(trimOf(world(GABLE, [{ axis: 'h', x: GABLE + 4, z: 5, kind: 'window' }, { axis: 'h', x: GABLE + 11, z: 5, kind: 'window' }]), GABLE));
    expect(none).toHaveLength(1);
    expect(none[0].z).toBeGreaterThan(11);
  });

  test('from a storey above: down past the storey below to the ground, but not across its windows; onto its roof where it stands out under the pipe', () => {
    const whole = () => true;
    const box = pipes(trimOf(world(GABLE, [], whole), GABLE, 1));
    expect(box).toHaveLength(2);
    for (const p of box) expect(p.foot).toBeCloseTo(0.1 - WALL_HEIGHT, 6);
    // A window downstairs under the first corner: that pipe stands at the other end.
    const moved = pipes(trimOf(world(GABLE, [{ axis: 'h', x: GABLE + 4, z: 5, kind: 'window' }], whole), GABLE, 1));
    expect(moved.filter((p) => p.z < 5).every((p) => p.x > GABLE + 11)).toBe(true);
    // The storey above set back from the walls below: its pipes end on the roof below.
    const back = pipes(trimOf(world(GABLE, [], (x, z) => x >= GABLE + 6 && x < GABLE + 10 && z >= 7 && z < 9), GABLE, 1));
    expect(back.length).toBeGreaterThan(0);
    for (const p of back) expect(p.foot).toBeCloseTo(0.2, 6);
  });

  test("a neighbour's silhouette has its gutters and its pipes, clear of its windows", () => {
    const planner = new HousePlanner();
    const spec = { x0: GABLE + 4, z0: 5, x1: GABLE + 12, z1: 11, lot: { x: GABLE, z: 0 }, front: [0, -1] as [number, number], door: null, garage: null };
    const trim = (openings: Opening[]) => planner.silhouettes([spec], openings).parts.find((p) => p.name === 'silhouetteTrim')!.geo;
    const count = (g: HousePart['geo']) => {
      // (Six corners a pipe's ring, at its foot: 0.1 over the ground.)
      let n = 0;
      for (let i = 1; i < g.pos.length; i += 3) if (Math.abs(g.pos[i] - 0.1) < 1e-9) n++;
      return n / 12;
    };
    expect(count(trim([]))).toBe(2);
    expect(count(trim([{ axis: 'h', x: GABLE + 4, z: 5, kind: 'window' }, { axis: 'h', x: GABLE + 11, z: 5, kind: 'window' }]))).toBe(1);
  });
});
