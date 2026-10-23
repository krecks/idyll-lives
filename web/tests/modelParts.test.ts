// `render/shared/modelParts.ts` against Babylon.js: the same vertices for the same parts (`bun test`),
// so a primitive part looks the same whichever engine draws it.
import { describe, expect, test } from 'bun:test';
import { NullEngine } from '@babylonjs/core/Engines/nullEngine';
import { Scene } from '@babylonjs/core/scene';
import { VertexBuffer } from '@babylonjs/core/Buffers/buffer';
import { VertexData } from '@babylonjs/core/Meshes/mesh.vertexData';
import { CreateCylinderVertexData } from '@babylonjs/core/Meshes/Builders/cylinderBuilder';
import { CreateSphereVertexData } from '@babylonjs/core/Meshes/Builders/sphereBuilder';
import { CreateTorusVertexData } from '@babylonjs/core/Meshes/Builders/torusBuilder';
import { CreateCapsuleVertexData } from '@babylonjs/core/Meshes/Builders/capsuleBuilder';
import { CreateIcoSphereVertexData } from '@babylonjs/core/Meshes/Builders/icoSphereBuilder';
import type { PrimitivePart } from '../src/assets/types';
import * as parts from '../src/render/shared/modelParts';

// (The Babylon renderer's models module sets up its loaders against the page's address.)
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '' }, $state: <T>(v: T) => v, $effect: Object.assign(() => {}, { root: () => () => {} }) });
const { mirrorSurface: babylonMirror, partMesh } = await import('../src/render/babylon/models');

const close = (a: ArrayLike<number> | null | undefined, b: ArrayLike<number> | null | undefined, eps = 1e-5) => {
  expect(a?.length).toBe(b?.length);
  if (!a || !b) return;
  let worst = 0;
  for (let i = 0; i < a.length; i++) worst = Math.max(worst, Math.abs(a[i] - b[i]));
  expect(worst).toBeLessThan(eps);
};
const same = (ours: parts.VData, theirs: VertexData) => {
  close(ours.positions, theirs.positions);
  close(ours.normals, theirs.normals);
  expect(Array.from(ours.indices)).toEqual(Array.from(theirs.indices ?? []));
};

describe('shapes match Babylon builders', () => {
  test('cylinders and cones', () => {
    for (const [h, top, bottom, n] of [
      [0.8, 1, 1, 20],
      [0.5, 0, 1, 16],
      [1.2, 0.4, 1, 8],
    ]) {
      same(parts.cylinderData(h, top, bottom, n), CreateCylinderVertexData({ height: h, diameterTop: top, diameterBottom: bottom, tessellation: n }));
    }
  });
  test('spheres', () => {
    for (const [x, y, z, n] of [
      [0.3, 0.3, 0.3, 12],
      [0.5, 0.2, 0.8, 6],
    ]) {
      same(parts.sphereData(x, y, z, n), CreateSphereVertexData({ diameterX: x, diameterY: y, diameterZ: z, segments: n }));
    }
  });
  test('tori', () => {
    same(parts.torusData(0.3, 0.05, 24), CreateTorusVertexData({ diameter: 0.3, thickness: 0.05, tessellation: 24 }));
    same(parts.torusData(0.6, 0.1, 12), CreateTorusVertexData({ diameter: 0.6, thickness: 0.1, tessellation: 12 }));
  });
  test('capsules', () => {
    same(parts.capsuleData(0.9, 0.12, 16, 6), CreateCapsuleVertexData({ height: 0.9, radius: 0.12, tessellation: 16, capSubdivisions: 6 }));
  });
  test('icospheres and their normals', () => {
    for (const n of [1, 2, 3]) same(parts.icoSphereData(0.5, n), CreateIcoSphereVertexData({ radius: 0.5, subdivisions: n, flat: false }));
    const ico = parts.icoSphereData(0.5, 2);
    const ours: number[] = [];
    const theirs: number[] = [];
    parts.computeNormals(ico.positions, ico.indices, ours);
    VertexData.ComputeNormals(ico.positions, ico.indices, theirs);
    close(ours, theirs);
  });
});

describe('primitive parts match the Babylon renderer', () => {
  const scene = new Scene(new NullEngine());
  const PARTS: PrimitivePart[] = [
    { shape: 'box', size: [0.8, 0.4, 0.5], at: [0, 0.2, 0], color: '#B8B2AA' },
    { shape: 'box', size: [0.1, 1.2, 0.04], at: [0.3, 0.6, -0.2], color: '#3A6EA5', rotate: [10, 35, -5], radius: 0.02 },
    { shape: 'cylinder', size: [0.3, 0.7, 0.2], at: [0.1, 0.35, 0], color: '#C08A64', taper: 0.7 },
    { shape: 'cone', size: [0.5, 0.6, 0.5], at: [0, 1, 0], color: '#2F3236', rotate: [90, 0, 0] },
    { shape: 'sphere', size: [0.3, 0.2, 0.4], at: [-0.2, 0.5, 0.1], color: '#FFFFFF', segments: 8 },
    { shape: 'capsule', size: [0.2, 0.8, 0.3], at: [0, 0.4, 0], color: '#123456', rotate: [0, 0, 90] },
    { shape: 'blob', size: [0.9, 0.7, 0.8], at: [0, 1.4, 0], color: '#4F7A2E' },
    { shape: 'torus' as PrimitivePart['shape'], size: [0.4, 0.05, 0.3], at: [0, 0.9, 0], color: '#AA8800' },
    { shape: 'lathe' as PrimitivePart['shape'], size: [0.3, 0.4, 0.3], at: [0, 0.2, 0], color: '#DDDDDD', ...{ profile: [[0, -0.5], [0.4, -0.5], [0.5, 0.2], [0.3, 0.5]] } },
  ];
  for (const [i, p] of PARTS.entries()) {
    for (const lod of [0, 1] as const) {
      test(`${p.shape} ${i} (lod ${lod})`, () => {
        const mesh = partMesh(scene, p, lod, i, 0.25);
        const ours = parts.partData(p, lod, i, 0.25);
        close(ours.positions, mesh.getVerticesData(VertexBuffer.PositionKind));
        close(ours.normals, mesh.getVerticesData(VertexBuffer.NormalKind));
        close(ours.uvs, mesh.getVerticesData(VertexBuffer.UVKind));
        close(ours.colors, mesh.getVerticesData(VertexBuffer.ColorKind));
        expect(Array.from(ours.indices)).toEqual(Array.from(mesh.getIndices() ?? []));
        mesh.dispose();
      });
    }
  }
  test('mirror faces', () => {
    for (const p of [
      { shape: 'box', size: [0.6, 0.9, 0.02], at: [0, 1.2, -0.2], color: '#FFFFFF', mirror: 'front', rotate: [0, 15, 0] },
      { shape: 'cylinder', size: [0.5, 0.02, 0.5], at: [0, 1.4, 0], color: '#FFFFFF', mirror: 'back', rotate: [90, 0, 0] },
      { shape: 'box', size: [0.02, 0.5, 0.4], at: [0.3, 1, 0], color: '#FFFFFF' },
    ] as PrimitivePart[]) {
      const ours = parts.mirrorSurface(p);
      const theirs = babylonMirror(p);
      expect(ours.round).toBe(theirs.round);
      close(ours.matrix, theirs.matrix.asArray());
    }
  });
});
