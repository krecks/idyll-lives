// A big world's ground tiles (`render/pc/groundTiles.ts`, built by `nature.ts`): each has three levels of detail and draws one;
// taking the landscape down destroys the two not drawn too (the one drawn goes with its entity), or about
// 40 MB of vertex buffers stay behind with every new game or load. On the engine's null device: no GPU.
import { describe, expect, spyOn, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';
import { disposeTiles } from '../src/render/pc/groundTiles';

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
