// The copy of a mesh PlayCanvas keeps on the CPU (`meshFrom`, `gpuOnly` in render/pc/meshes.ts): a static mesh lets
// it go once uploaded, and its going still takes its bytes off the device's count of GPU memory; one whose vertices
// change later or are read back keeps it. On the engine's null device, its buffers made to say they're uploaded.
import { afterAll, describe, expect, mock, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';
import type { MeshUse } from '../src/render/pc/meshes';

// (The renderer's modules read the page's address as they load: one put in for them, the real one back after.)
const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (Only the cutaway's attribute comes from the shader chunks, whose module wants the game's settings.)
mock.module('../src/render/pc/shaders/chunks', () => ({ WALL_CUT_SEMANTIC: pc.SEMANTIC_ATTR7 }));
const { gpuOnly, meshFrom } = await import('../src/render/pc/meshes');

function device() {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const d = new pc.NullGraphicsDevice(canvas);
  // (The null device's buffers never say they're on the GPU: these do, and count their uploads.)
  const impl = () => ({ initialized: true, uploads: 0, unlock() { this.uploads++; }, destroy() {}, loseContext() {} });
  d.createVertexBufferImpl = impl;
  d.createIndexBufferImpl = impl;
  return d;
}

/** Two triangles, a metre square, lifted 2 m. */
const QUAD = {
  positions: new Float32Array([0, 2, 0, 1, 2, 0, 1, 2, 1, 0, 2, 1]),
  normals: new Float32Array([0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0]),
  uvs: new Float32Array([0, 0, 1, 0, 1, 1, 0, 1]),
  indices: new Uint32Array([0, 1, 2, 0, 2, 3]),
};

function made(use: MeshUse) {
  const d = device();
  const vram = { vb: d._vram.vb, ib: d._vram.ib };
  const mesh = meshFrom(d, QUAD, use)!;
  return { d, vram, mesh, buffers: [mesh.vertexBuffer, mesh.indexBuffer[0]] };
}

describe('the CPU copy of a mesh', () => {
  test('a static mesh lets it go once uploaded, keeping its box and its count of GPU memory', () => {
    const { d, vram, mesh, buffers } = made('static');
    for (const b of buffers) expect(b.storage.byteLength).toBe(0);
    expect(d._vram.vb - vram.vb).toBe(mesh.vertexBuffer.numBytes);
    expect(d._vram.ib - vram.ib).toBe(mesh.indexBuffer[0].numBytes);
    expect([...mesh.aabb.getMin().toArray(), ...mesh.aabb.getMax().toArray()]).toEqual([0, 2, 0, 1, 2, 1]);
    // (Destroyed, its bytes come off the count as they went on.)
    mesh.destroy();
    expect(d._vram).toMatchObject(vram);
  });

  test('a device coming back has nothing to upload again for it', () => {
    const { buffers } = made('static');
    for (const b of buffers) {
      const uploads = b.impl.uploads;
      expect(() => b.restoreContext()).not.toThrow();
      expect(b.impl.uploads).toBe(uploads);
    }
  });

  test('a dynamic mesh keeps it, to write its changed vertices into', () => {
    const { mesh } = made('dynamic');
    expect(mesh.vertexBuffer.storage.byteLength).toBe(mesh.vertexBuffer.numBytes);
    mesh.setPositions(QUAD.positions.map((v, i) => (i % 3 === 1 ? v + 1 : v)));
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const read: number[] = [];
    mesh.getPositions(read);
    expect(read[1]).toBe(3);
  });

  test('a mesh read back keeps it', () => {
    const { mesh } = made('read');
    const read: number[] = [];
    expect(mesh.getPositions(read)).toBe(4);
    expect(read).toEqual([...QUAD.positions]);
    const indices: number[] = [];
    mesh.getIndices(indices);
    expect(indices).toEqual([...QUAD.indices]);
  });

  test('a buffer not uploaded yet keeps it', () => {
    const d = new pc.NullGraphicsDevice({ width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement);
    const mesh = gpuOnly(meshFrom(d, QUAD, 'read')!);
    expect(mesh.vertexBuffer.storage.byteLength).toBe(mesh.vertexBuffer.numBytes);
  });
});
