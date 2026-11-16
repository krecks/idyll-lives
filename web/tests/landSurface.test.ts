// The renderer on a land the town follows (`render/shared/landSurface.ts`, `pads.ts`, `roads.ts` `roadMeshes`'s
// levels, `nature.ts`'s land, `pc/space.ts` `setDatum`), on a small made-up world: a street climbing a slope with a
// lane in a T off it and lots on both. The roads drawn on their heights (the sidewalks' outer edges a lip below),
// the junction's pad at its one height; a part merged onto its pad; each lot masked out of the terrain and drawn as
// a slab at its pad with a retaining edge where the land beside it is higher or lower; the land's heights the pads'
// in a lot and the land's, not a slope to the pad, beside it; and the datum keeping the viewed lot at the engine's
// 0. A world its town doesn't follow draws as ever. `bun test tests/landSurface.test.ts`.
import { afterAll, describe, expect, test } from 'bun:test';
import type { PlotInfo, RoadPlan, WorldStructure } from '../src/core/protocol';
import { blankLand, encodeLand, LAND_CELL } from '../src/game/land';
import { Geo } from '../src/render/shared/geometry';
import { follows, Levels, levelsOf, surfaceOf } from '../src/render/shared/landSurface';
import { EDGE_FOOT, padGeometry, padMask, padsOf, STEP_LEAST } from '../src/render/shared/pads';
import { ROAD_Y, roadMeshes, WALK_LIP, WALK_Y } from '../src/render/shared/roads';
import { junctionsOf } from '../src/render/shared/junctions';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));

const [W, D] = [200, 120];

/** A street along x at z = 40 climbing 6 % with the land, a lane south off it at x = 100, lots either side of each. */
function world(seeded: 'follow' | 'drawn' | null = 'follow'): WorldStructure {
  const roads: RoadPlan[] = [
    { points: [[0, 40], [200, 40]], width: 11, kind: 'street', ends: ['out', 'out'] },
    { points: [[100, 45.5], [100, 118]], width: 9, kind: 'lane', ends: ['join', 'dead'] },
  ];
  const plots: PlotInfo[] = [];
  const paths: { x: number; z: number; w: number; d: number }[] = [];
  const facing: number[] = [];
  for (let i = 0; i < 8; i++) {
    const x = i * 24 + 2;
    if (x + 20 > 95 && x < 106) continue;
    plots.push({ id: plots.length, name: '', x, z: 12, w: 20, d: 22, public: false } as PlotInfo);
    paths.push({ x: x + 9, z: 34, w: 2, d: 0.5 });
    facing.push(2);
  }
  plots.push({ id: plots.length, name: '', x: 112, z: 60, w: 20, d: 24, public: false } as PlotInfo);
  facing.push(1);
  const base = { width: W, depth: D, plots, objects: [], rooms: [], walls: [], openings: [], households: [], sims: [], exits: [] } as unknown as WorldStructure;
  const meta: Record<string, unknown> = { seed: 1, name: 'Slope', roads, paths, facing, world: { landscape: { slope: 0.5 } } };
  if (seeded) {
    const land = blankLand(W, D, seeded);
    for (let j = 0; j < land.rows; j++) for (let i = 0; i < land.cols; i++) land.h[j * land.cols + i] = Math.round((0.06 * i * LAND_CELL + 2 * Math.sin((j * LAND_CELL) / 9)) * 100);
    meta.land = encodeLand(land);
  }
  return { ...base, meta } as unknown as WorldStructure;
}

describe('the renderer on a land the town follows', () => {
  const w = world();
  const surface = surfaceOf(w)!;

  test('a world has a surface only where its town follows its land, made once', () => {
    expect(follows(w)).toBe(true);
    expect(surface).not.toBeNull();
    expect(surfaceOf(w)).toBe(surface);
    expect(surfaceOf(world('drawn'))).toBeNull();
    expect(surfaceOf(world(null))).toBeNull();
    expect(levelsOf(world(null))).toBeNull();
  });

  test('the roads drawn on their heights, the sidewalks’ outer edges a lip below, the junction’s pad at its one height', () => {
    const roads = w.meta!.roads!;
    const levels = new Levels(surface, roads);
    const junctions = junctionsOf(w, roads);
    const { surface: road, walk } = roadMeshes(roads, null, 0, {}, 'both', junctions, undefined, levels);
    // Every vertex of the street's surface on its height there (`ROAD_Y` over it), a vertex at least every 2 m.
    const street = roads[0];
    let seen = 0;
    for (let k = 0; k < road.pos.length; k += 3) {
      const [x, y, z] = [road.pos[k], road.pos[k + 1], road.pos[k + 2]];
      if (Math.abs(z - 40) > 5.6 || x < 1 || x > 199) continue;
      seen++;
      expect(Math.abs(y - (levels.road(street, x) + ROAD_Y))).toBeLessThan(0.02);
    }
    expect(seen).toBeGreaterThan(200);
    // The climb shows: 6 % over 200 m.
    expect(levels.road(street, 190) - levels.road(street, 10)).toBeGreaterThan(5);
    // The sidewalks' tops at `WALK_Y` over the road; their outer edges reach `WALK_LIP` below it.
    let lowest = Infinity;
    for (let k = 0; k < walk.pos.length; k += 3) {
      const [x, y, z] = [walk.pos[k], walk.pos[k + 1], walk.pos[k + 2]];
      if (Math.abs(z - 40) > 5.6 || x < 20 || x > 80) continue;
      const over = y - levels.road(street, x);
      expect(over).toBeLessThanOrEqual(WALK_Y + 0.016);
      lowest = Math.min(lowest, over);
    }
    expect(lowest).toBeCloseTo(-WALK_LIP, 2);
    // The T's junction (the sim's pads none here): one height, the street's there and the lane's where it begins.
    const jy = levels.junction({ x: 100, z: 40 });
    expect(surface.junctionHeights.length).toBe(1);
    expect(jy).toBeCloseTo(surface.roadY(0, 100), 3);
    expect(jy).toBeCloseTo(surface.roadY(1, 0), 3);
  });

  test('without levels the roads are drawn as ever', () => {
    const roads = w.meta!.roads!;
    const a = roadMeshes(roads, null, 0, {}, 'both', [], () => 3);
    const b = roadMeshes(roads, null, 0, {}, 'both', [], () => 3, null);
    expect(b.surface.pos).toEqual(a.surface.pos);
    expect(b.walk.pos).toEqual(a.walk.pos);
    // (And at 0: the street's surface at `ROAD_Y`.)
    for (let k = 1; k < a.surface.pos.length; k += 3) expect([ROAD_Y, ROAD_Y + 0.002]).toContain(a.surface.pos[k]);
  });

  test('a part merged onto its pad: lifted, its colours and its shape as they were', () => {
    const part = new Geo().color('#336699').box(0, 0, 0, 1, 2, 1);
    const flat = new Geo().merge(part);
    const lifted = new Geo().merge(part, 7.5);
    expect(lifted.col).toEqual(flat.col);
    expect(lifted.idx).toEqual(flat.idx);
    for (let k = 0; k < flat.pos.length; k++) expect(lifted.pos[k]).toBeCloseTo(flat.pos[k] + (k % 3 === 1 ? 7.5 : 0), 6);
  });

  test('each lot masked out of the terrain and drawn as a slab at its pad, a retaining edge where the land beside it differs', async () => {
    const pads = padsOf(surface, w.plots.length);
    expect(pads.length).toBe(w.plots.length);
    const mask = padMask(pads, W, D);
    for (const p of pads) {
      expect(mask[Math.floor(p.z + 1) * W + Math.floor(p.x + 1)]).toBe(255);
      expect(mask[Math.floor(p.z + p.d - 1) * W + Math.floor(p.x + p.w - 1)]).toBe(255);
    }
    // (Not on the street.)
    expect(mask[40 * W + 50]).toBe(0);
    const { LandscapeLayout } = await import('../src/render/shared/nature');
    const { CountryRoads } = await import('../src/render/shared/countryRoads');
    const layout = new LandscapeLayout(w, (l, ww, n) => new CountryRoads(l, ww, n));
    const land = (x: number, z: number) => layout.terrainAt(x, z);
    const geos = padGeometry(land, pads, () => 0, (_x, _z, _y, out) => ((out.r = out.g = out.b = 0.5), 1), (x, z) => [x, z]);
    const g = geos.get(0)!;
    // The slabs: every vertex at its lot's pad.
    for (let k = 0; k < g.slabs.pos.length; k += 3) {
      const [x, y, z] = [g.slabs.pos[k], g.slabs.pos[k + 1], g.slabs.pos[k + 2]];
      const p = pads.find((q) => x >= q.x - 1e-6 && x <= q.x + q.w + 1e-6 && z >= q.z - 1e-6 && z <= q.z + q.d + 1e-6)!;
      expect(y).toBeCloseTo(p.y, 5);
    }
    // The edges: from the lower of the two (less `EDGE_FOOT`) to the higher; none where the step is less than `STEP_LEAST`.
    expect(g.edges.pos.length).toBeGreaterThan(0);
    for (let k = 0; k < g.edges.pos.length; k += 12) {
      const ys = [1, 4, 7, 10].map((o) => g.edges.pos[k + o]);
      expect(Math.max(...ys) - Math.min(...ys)).toBeGreaterThanOrEqual(STEP_LEAST + EDGE_FOOT - 1e-6);
    }
  });

  test('a world its town doesn’t follow has no pads, and its land as ever', async () => {
    const { LandscapeLayout } = await import('../src/render/shared/nature');
    const { CountryRoads } = await import('../src/render/shared/countryRoads');
    const flat = new LandscapeLayout(world(null), (l, ww, n) => new CountryRoads(l, ww, n));
    expect(flat.surface).toBeNull();
    expect(flat.padAt).toBeUndefined();
    // (Built land at 0, as ever.)
    expect(flat.heightAt(50, 20)).toBe(0);
    expect(flat.heightAt(50, 40)).toBe(0);
  });

  test('the land’s heights: a lot’s pad inside it, the road on it, beside a lot the land’s own, not a slope to the pad', async () => {
    const { LandscapeLayout } = await import('../src/render/shared/nature');
    const { CountryRoads } = await import('../src/render/shared/countryRoads');
    const layout = new LandscapeLayout(w, (l, ww, n) => new CountryRoads(l, ww, n));
    expect(layout.surface).toBe(surface);
    const pads = padsOf(surface, w.plots.length);
    for (const [i, p] of pads.entries()) {
      expect(layout.heightAt(p.x + p.w / 2, p.z + p.d / 2)).toBeCloseTo(surface.padY(i), 5);
      expect(layout.padAt!(p.x + 1, p.z + 1)).toBeCloseTo(surface.padY(i), 5);
    }
    expect(layout.padAt!(50, 40)).toBeNull();
    expect(layout.heightAt(50, 40)).toBeCloseTo(surface.roadY(0, 50), 3);
    // The terrain under a road sunk a little below it, inside a lot whatever the land round it is (masked).
    expect(layout.terrainAt(50, 40)).toBeLessThan(surface.roadY(0, 50));
    // Beside the lot off the lane (x 112..132): the land west of it at its edge is the land carried in, not its pad.
    const p = pads[pads.length - 1];
    const inner = layout.terrainAt(p.x + p.w + 0.01, p.z + p.d / 2);
    const outer = layout.terrainAt(p.x + p.w + 2, p.z + p.d / 2);
    expect(Math.abs(inner - outer)).toBeLessThan(Math.abs(p.y - outer) + 0.01);
  });

  test('the datum keeps the viewed lot at the engine’s 0 and lowers the world by it', async () => {
    const pc = await import('../src/render/pc/engine');
    const { createWorldRoot, setDatum } = await import('../src/render/pc/space');
    const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
    const app = new pc.AppBase(canvas);
    const options = new pc.AppOptions();
    options.graphicsDevice = new pc.NullGraphicsDevice(canvas);
    options.componentSystems = [];
    options.resourceHandlers = [];
    app.init(options);
    const root = createWorldRoot(app);
    const lotRoot = new pc.Entity('lot');
    root.addChild(lotRoot);
    // A thing on the viewed lot (lot-relative: at its own 0) and a tree on the land beside it (game space: 14.25 m up).
    const thing = new pc.Entity('thing');
    thing.setLocalPosition(3, 0.5, 4);
    lotRoot.addChild(thing);
    const tree = new pc.Entity('tree');
    tree.setLocalPosition(10, 14.25, 20);
    root.addChild(tree);
    for (const datum of [0, 12.5, -3]) {
      setDatum(root, lotRoot, datum);
      const t = thing.getPosition();
      // (The engine mirrors z: `space.ts`.)
      expect([t.x, t.y, t.z].map((v) => +v.toFixed(5))).toEqual([3, 0.5, -4]);
      const g = tree.getPosition();
      expect(g.y).toBeCloseTo(14.25 - datum, 5);
      expect(g.z).toBeCloseTo(-20, 5);
    }
    // A camera's target on the land shifted by the old datum less the new keeps where it looks: game = engine + datum.
    const [old, next, engineY] = [12.5, 30, 1.75];
    const shifted = engineY + old - next;
    expect(shifted + next).toBeCloseTo(engineY + old, 9);
    app.destroy();
  });
});

