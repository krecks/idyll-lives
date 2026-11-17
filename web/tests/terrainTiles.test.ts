// A big world's ground tiles (`render/pc/groundTiles.ts`, built by `nature.ts`): each has three levels of detail and draws one;
// taking the landscape down destroys the ones not drawn too (the one drawn goes with its entity), or about
// 40 MB of vertex buffers stay behind with every new game or load. A land drawn from a height field draws again, at a
// brush's stamp, only the tiles round the samples it changed (and past the world's edge where it reaches it), with the
// colours they had. Each tile makes its finest level only while near enough to draw it. On the engine's null device: no GPU.
import { afterAll, describe, expect, mock, spyOn, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';
import { disposeTiles, type GroundTile } from '../src/render/pc/groundTiles';
import { bakeLand } from '../src/render/shared/landBake';
import { GROUND_TILE } from '../src/render/shared/nature';
import { plainWorld, SIDE } from './support/landFixtures';

function device() {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  return new pc.NullGraphicsDevice(canvas);
}

describe("a big world's ground tiles", () => {
  test('the levels not drawn are destroyed; the one drawn is left to its entity', () => {
    const d = device();
    const meshes = [0, 1, 2].map(() => pc.Mesh.fromGeometry(d, new pc.PlaneGeometry()));
    const mi = new pc.MeshInstance(meshes[1], new pc.StandardMaterial());
    // (Drawn at its far level, then its near one: the level swapped as the camera moves.)
    mi.mesh = meshes[2];
    mi.mesh = meshes[0];
    const destroyed = meshes.map((m) => spyOn(m, 'destroy'));
    disposeTiles([{ mi, meshes, x: 64, z: 64, lod: 0 }]);
    expect(destroyed.map((s) => s.mock.calls.length)).toEqual([0, 1, 1]);
    // Its draw's going (with its entity) takes the one drawn.
    mi.destroy();
    expect(destroyed[0].mock.calls.length).toBe(1);
  });
});

describe("a land drawn from a height field: a stamp's tiles", async () => {
  // (The landscape reads the page's flags as it loads.)
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  afterAll(() => Object.assign(globalThis, { location: realLocation }));
  // (The settings are a Svelte store, not for a test: the soft look's only.)
  mock.module('../src/settings/settings.svelte', () => ({ settings: { shading: 'soft' } }));
  // (Nor the perf spans, which watch the page's frames.)
  mock.module('../src/debug/perf', () => ({
    perfTime: <T>(_: string, fn: () => T) => fn(),
    perfTimeAsync: <T>(_: string, fn: () => Promise<T>) => fn(),
    perfStart: () => ({}),
    perfEnd: () => 0,
  }));
  const { Landscape } = await import('../src/render/pc/nature');

  /** A landscape on the null device with its ground tiles laid (no plants, no grass), drawn from its own survey. */
  function shaped() {
    const d = device();
    const world = plainWorld();
    const scape = new Landscape(world, 1, 0, 0, { radius: 60, density: 1, nearest: 0 }, { stream: 1, farCull: 450, woodsFar: 400, woodsCell: 256, flowersFar: 120, tiles: [150, 350], far: 1800 });
    const field = bakeLand(scape, SIDE, SIDE);
    scape.setLand(field);
    const parent = new pc.Entity('root');
    const inner = scape as unknown as { host: unknown; tiles: GroundTile[]; terrainGrid(): { size: number } };
    inner.host = { app: { graphicsDevice: d }, parent };
    const count = Math.ceil(inner.terrainGrid().size / GROUND_TILE);
    const kept = new Uint8Array(67 * 67 * 4).fill(200);
    inner.tiles = Array.from({ length: count * count }, () => {
      const meshes = [0, 1, 2].map(() => pc.Mesh.fromGeometry(d, new pc.PlaneGeometry()));
      return { mi: new pc.MeshInstance(meshes[1], new pc.StandardMaterial()), meshes, x: 0, z: 0, lod: 1, kept };
    });
    return { scape, field, tiles: inner.tiles, count };
  }

  /** What each tile draws now: its meshes and how often each was made or written (a tile drawn again changes it). */
  const drawn = (tiles: readonly GroundTile[]) => tiles.map((t) => t.meshes.map((m) => (m ? `${m.id}:${(m as unknown as { _aabbVer: number })._aabbVer}` : '-')).join());
  const changed = (before: string[], after: string[]) => after.map((v, k) => (v !== before[k] ? k : -1)).filter((k) => k >= 0);

  test('only the tiles round the changed samples are drawn again, at the land as it is now', () => {
    const { scape, field, tiles, count } = shaped();
    const before = tiles.map((t) => [...t.meshes]);
    // A mound round (180, 180): samples 70..110.
    for (let j = 70; j < 110; j++) for (let i = 70; i < 110; i++) field.h[j * field.cols + i] += 300;
    const count2 = scape.reground({ i0: 70, j0: 70, i1: 110, j1: 110 });
    const again = tiles.map((t, k) => t.mi.mesh !== before[k][1]);
    // (The landscape's square from -260, 9 tiles across: the samples' box, 136..222 m with their neighbours and the
    // grids' rings, is in tile 3 (124..252 m) each way.)
    expect(count2).toBe(1);
    expect(again.filter(Boolean).length).toBe(1);
    expect(again[3 * count + 3]).toBe(true);
    // The tile drawn again stands on the new land: its levels made anew (the old ones destroyed), the level drawn kept.
    const t = tiles[3 * count + 3];
    expect(t.mi.mesh).toBe(t.meshes[1]);
    expect(before[3 * count + 3].every((m) => m!.vertexBuffer === null)).toBe(true);
    const ps: number[] = [];
    t.meshes[1]!.getPositions(ps);
    const at = [...Array(ps.length / 3).keys()].find((v) => ps[v * 3] === 180 && ps[v * 3 + 2] === 180)!;
    expect(ps[at * 3 + 1]).toBeCloseTo(scape.heightAt(180, 180), 5);
    expect(scape.heightAt(180, 180)).toBeCloseTo(field.h[90 * field.cols + 90] / 100, 5);
    // Stamped again: written into the same mesh (kept on the CPU for it), no new one.
    const live = t.meshes[1];
    const was = drawn(tiles);
    field.h[90 * field.cols + 90] += 100;
    scape.reground({ i0: 90, j0: 90, i1: 91, j1: 91 });
    expect(t.meshes[1]).toBe(live);
    expect(changed(was, drawn(tiles))).toEqual([3 * count + 3]);
    t.meshes[1]!.getPositions(ps);
    expect(ps[at * 3 + 1]).toBeCloseTo(scape.heightAt(180, 180), 5);
  });

  test('the finest level is let go far off, and made again from the heights kept as the tile comes near', () => {
    const { scape, field, tiles, count } = shaped();
    for (let j = 70; j < 110; j++) for (let i = 70; i < 110; i++) field.h[j * field.cols + i] += 300;
    scape.reground({ i0: 70, j0: 70, i1: 110, j1: 110 });
    const t = tiles[3 * count + 3];
    // (Every fixture tile's middle at 0, 0: the camera there draws them all at their finest.)
    scape.frameTiles(0, 0);
    expect(t.lod).toBe(0);
    expect(t.mi.mesh).toBe(t.meshes[0]);
    const ps: number[] = [];
    t.meshes[0]!.getPositions(ps);
    const at = [...Array(ps.length / 3).keys()].find((v) => ps[v * 3] === 180 && ps[v * 3 + 2] === 180)!;
    expect(ps[at * 3 + 1]).toBeCloseTo(scape.heightAt(180, 180), 5);
    // Far off: its coarsest drawn, its finest gone.
    scape.frameTiles(5000, 5000);
    expect(t.lod).toBe(2);
    expect(t.meshes[0]).toBe(null);
    expect(t.mi.mesh).toBe(t.meshes[2]);
    // Near again: made again from its heights.
    scape.frameTiles(0, 0);
    t.meshes[0]!.getPositions(ps);
    expect(ps[at * 3 + 1]).toBeCloseTo(scape.heightAt(180, 180), 5);
  });

  test('the ground stops at the landscape’s square, though its last tiles would reach past it', () => {
    const { scape, count } = shaped();
    const inner = scape as unknown as { tileGrid(k: number): Float32Array; terrainGrid(): { size: number } };
    // (The square from -260 to 772 m each way: 9 tiles would reach to 892 m.)
    const size = inner.terrainGrid().size;
    const [lo, hi] = [SIDE / 2 - size / 2, SIDE / 2 + size / 2];
    expect(lo + count * GROUND_TILE).toBeGreaterThan(hi);
    let [x0, x1, z0, z1] = [Infinity, -Infinity, Infinity, -Infinity];
    for (let k = 0; k < count * count; k++) {
      const g = inner.tileGrid(k);
      for (let v = 0; v < g.length; v += 3) [x0, x1, z0, z1] = [Math.min(x0, g[v]), Math.max(x1, g[v]), Math.min(z0, g[v + 2]), Math.max(z1, g[v + 2])];
    }
    // (Its grids' ring of one past each tile reaches out on the near sides, for the normals there.)
    expect(x1).toBe(hi);
    expect(z1).toBe(hi);
    expect(x0).toBeGreaterThanOrEqual(lo - GROUND_TILE / 64);
    expect(z0).toBeGreaterThanOrEqual(lo - GROUND_TILE / 64);
  });

  test('samples on a tile’s edge draw both tiles; at the world’s edge, the tiles past it too', () => {
    const { scape, tiles, count } = shaped();
    const before = drawn(tiles);
    // x 124 m is where tiles 2 and 3 meet (-260 + 3 × 128).
    scape.reground({ i0: 62, j0: 130, i1: 63, j1: 131 });
    expect(changed(before, drawn(tiles))).toEqual([4 * count + 2, 4 * count + 3]);
    const edge = drawn(tiles);
    scape.reground({ i0: 0, j0: 130, i1: 1, j1: 131 });
    // (The edge's land carries 60 m past it: tile 1 (-132..-4) and tile 2 (-4..124).)
    expect(changed(edge, drawn(tiles))).toEqual([4 * count + 1, 4 * count + 2]);
  });
});
