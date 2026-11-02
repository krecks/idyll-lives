/**
 * Build mode's Garage tool without the simulation (`game/garage.ts`): a garage placed freely on whole
 * tiles, whether it fits on the home lot (as sim-core `garages.rs` checks, bar who it would shut in),
 * and the one at the end of a driveway (`drivewayGarage`, where the town puts garages), turned with
 * a lot that faces its street on another side.
 */
import { describe, expect, test } from 'bun:test';
import type { PlotInfo, WorldStructure } from '../src/core/protocol';
import { freeGarage, GARAGE_TILES, garageFits, garageTiles, onGarage, type GarageSite } from '../src/game/garage';

type Rect = { x: number; z: number; w: number; d: number };

/** A lot with nothing on it (x 0..30, z 0..22), its driveway (x 23..25, z 15.5..22) where the car parks. */
function site(over: Partial<GarageSite> = {}): GarageSite {
  return {
    plot: { x: 0, z: 0, w: 30, d: 22 },
    hasGarage: false,
    indoors: () => false,
    walled: () => false,
    thingAt: () => undefined,
    drive: { x: 23, z: 15.5, w: 2, d: 6.5 },
    affords: true,
    ...over,
  };
}

describe('a garage placed freely', () => {
  test('snaps to whole tiles, its walls in the middle of them, and turns with its door', () => {
    const g = freeGarage(10.2, 10.7, 0);
    expect(g.rot).toBe(0);
    // 4 tiles across its door (x 8..12), 6 deep (z 8..14), its 3.14 × 5.4 m in the middle.
    expect(g.x0).toBeCloseTo(8.43, 5);
    expect(g.x1).toBeCloseTo(11.57, 5);
    expect(g.z0).toBeCloseTo(8.3, 5);
    expect(g.z1).toBeCloseTo(13.7, 5);
    const tiles = garageTiles(g);
    expect(tiles.length).toBe(GARAGE_TILES[0] * GARAGE_TILES[1]);
    expect(Math.min(...tiles.map(([x]) => x))).toBe(8);
    expect(Math.max(...tiles.map(([, z]) => z))).toBe(13);
    // Turned a quarter: 6 tiles along x, 4 along z, round the same middle.
    const t = freeGarage(10.2, 10.7, 1);
    expect(t.x1 - t.x0).toBeCloseTo(5.4, 5);
    expect(t.z1 - t.z0).toBeCloseTo(3.14, 5);
    expect(garageTiles(t).length).toBe(24);
    expect(onGarage(g, 10, 10)).toBe(true);
    expect(onGarage(g, 12.5, 10)).toBe(false);
  });
});

describe('whether a garage fits on the home lot', () => {
  const g = freeGarage(10, 10, 0);
  test('on open ground it does', () => {
    expect(garageFits(g, site())).toBe(true);
  });
  test('one a home, on the lot, outdoors, clear of walls, things and the car, paid for', () => {
    expect(garageFits(g, site({ hasGarage: true }))).toBe('Your home has a garage already');
    expect(garageFits(freeGarage(1, 10, 0), site())).toBe('Has to go on your lot');
    expect(garageFits(freeGarage(10, 20, 0), site())).toBe('Has to go on your lot');
    expect(garageFits(g, site({ indoors: (x, z) => x === 9 && z === 12 }))).toBe('Goes outdoors, not in a room');
    // A fence across it, a diagonal in it; a wall along its tiles' edge, off its walls, is no matter.
    expect(garageFits(g, site({ walled: (a, x, z) => a === 'v' && x === 10 && z === 12 }))).toBe('A wall or a fence is in the way');
    expect(garageFits(g, site({ walled: (a, x, z) => a === 'h' && x === 9 && z === 7 }))).toBe(true);
    expect(garageFits(g, site({ walled: (a, x, z) => a === 'd' && x === 9 && z === 9 }))).toBe('A wall or a fence is in the way');
    expect(garageFits(g, site({ walled: (a, x) => a === 'v' && x === 8 }))).toBe(true);
    expect(garageFits(g, site({ thingAt: (x, z) => (x === 10 && z === 9 ? 'potted plant' : undefined) }))).toBe('The potted plant is in the way');
    expect(garageFits(g, site({ affords: false }))).toBe('Not enough money');
    expect(garageFits(freeGarage(24, 15, 0), site())).toBe("That's where the car parks");
    // Its door just on the driveway's end, where the town builds them, is fine.
    const end = { x0: 22.43, z0: 10.1 - 0.06, x1: 25.57, z1: 15.5 - 0.06, rot: 0 };
    expect(garageFits(end, site())).toBe(true);
  });
});

/** Two houses north of a street running along x (`turn`: the town turned a quarter, the street along z), the second with a thing where its garage would go. */
function town(turn: boolean): WorldStructure {
  const pt = (x: number, z: number): [number, number] => (turn ? [-z, x] : [x, z]);
  const rect = (r: Rect): Rect => (turn ? { x: -(r.z + r.d), z: r.x, w: r.d, d: r.w } : r);
  const box = ([x0, z0, x1, z1]: [number, number, number, number]): [number, number, number, number] => {
    const r = rect({ x: x0, z: z0, w: x1 - x0, d: z1 - z0 });
    return [r.x, r.z, r.x + r.w, r.z + r.d];
  };
  const lots = [
    { r: { x: 0, z: 0, w: 20, d: 20 }, house: [3, 5, 14, 14] as [number, number, number, number], door: 8, drive: { x: 14.6, z: 9.6, w: 3, d: 10.4 } },
    { r: { x: 20, z: 0, w: 18, d: 20 }, house: [24, 6, 33, 15] as [number, number, number, number], door: 28, drive: { x: 33.6, z: 10.6, w: 3, d: 9.4 } },
  ];
  const plots: PlotInfo[] = lots.map((l, id) => ({ id, name: `${id}`, ...rect(l.r), public: false, entry: pt(l.door + 0.5, 18), house: box(l.house) }));
  const paths = lots.flatMap((l) => [rect({ x: l.door, z: l.house[3], w: 1, d: 20 - l.house[3] }), rect(l.drive)]);
  const [w, d] = turn ? [26, 38] : [38, 26];
  return {
    width: w,
    depth: d,
    objects: [],
    plots,
    rooms: new Array(w * d).fill(0),
    households: [],
    sims: [],
    exits: [],
    meta: { kind: 'town', name: 'Test', streets: [], paths, facing: lots.map(() => (turn ? 1 : 2)), seed: 1 },
  } as unknown as WorldStructure;
}

describe('the garage at the end of the driveway', async () => {
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: {} });
  const { drivewayGarage } = await import('../src/render/shared/street');
  test('stands where the drive ends, beside the house, its door towards the street', () => {
    const at = drivewayGarage(town(false), 0, []);
    expect(at).not.toBeNull();
    const { garage: g, rot, drive } = at!;
    expect(rot).toBe(0);
    expect(drive).toEqual({ x: 14.6, z: 9.6, w: 3, d: 10.4 });
    // As wide as the drive and a little more, off the house, up to the drive's end.
    expect(g.z1).toBeCloseTo(9.6, 5);
    expect(g.z1 - g.z0).toBeCloseTo(5.4, 5);
    expect(g.x0).toBeGreaterThanOrEqual(14.35 - 1e-6);
    expect(g.x1 - g.x0).toBeCloseTo(3.3, 5);
    // It fits there.
    const plan = { ...g, rot };
    expect(garageFits(plan, site({ plot: { x: 0, z: 0, w: 20, d: 20 }, drive }))).toBe(true);
  });
  test('none where something stands in its way', () => {
    expect(drivewayGarage(town(false), 1, [{ x: 34, z: 7, w: 1, d: 1 }])).toBeNull();
    expect(drivewayGarage(town(false), 1, [])).not.toBeNull();
  });
  test('turned with a lot facing its street on another side', () => {
    const a = drivewayGarage(town(false), 0, [])!;
    const b = drivewayGarage(town(true), 0, [])!;
    // The street west of the turned lot: its door faces -x.
    expect(b.rot).toBe(3);
    const [s, t] = [a.garage, b.garage];
    expect([t.x0, t.z0, t.x1, t.z1].map((v) => Math.round(v * 1000) / 1000)).toEqual([-s.z1, s.x0, -s.z0, s.x1].map((v) => Math.round(v * 1000) / 1000));
  });
});
