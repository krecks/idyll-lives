// `render/shared/modelParts.ts` against Babylon.js: the same vertices for the same parts (`bun test`), so a
// primitive part looks as it did when Babylon drew it. Babylon's vertices (its mesh builders, and the Babylon
// renderer's primitive parts and mirror faces, for the shapes and parts recorded with them) are fixtures
// recorded from Babylon 9.30 (`fixtures/babylon/modelParts.json`, made by `fixtures/record-babylon.ts`, which
// needs a Babylon reference checkout to run again).
import { describe, expect, test } from 'bun:test';
import type { PrimitivePart } from '../src/assets/types';
import * as parts from '../src/render/shared/modelParts';
import { fixture, numbers, type Numbers } from './fixtures/babylon';

interface Shape {
  args: number[];
  positions: Numbers;
  normals: Numbers;
  indices: Numbers;
}
const B = fixture<{
  cylinders: Shape[];
  spheres: Shape[];
  tori: Shape[];
  capsules: Shape[];
  icospheres: Shape[];
  computeNormals: { positions: Numbers; indices: Numbers; normals: Numbers };
  parts: { part: PrimitivePart; lod: 0 | 1; index: number; alpha: number; positions: Numbers | null; normals: Numbers | null; uvs: Numbers | null; colors: Numbers | null; indices: Numbers }[];
  mirrors: { part: PrimitivePart; round: boolean; matrix: Numbers }[];
}>('modelParts');

const or = (n: Numbers | null) => (n ? numbers(n) : null);
const close = (a: ArrayLike<number> | null | undefined, b: ArrayLike<number> | null | undefined, eps = 1e-5) => {
  expect(a?.length).toBe(b?.length);
  if (!a || !b) return;
  let worst = 0;
  for (let i = 0; i < a.length; i++) worst = Math.max(worst, Math.abs(a[i] - b[i]));
  expect(worst).toBeLessThan(eps);
};
const same = (ours: parts.VData, theirs: Shape) => {
  close(ours.positions, numbers(theirs.positions));
  close(ours.normals, numbers(theirs.normals));
  expect(Array.from(ours.indices)).toEqual(Array.from(numbers(theirs.indices)));
};

describe('shapes match Babylon builders', () => {
  test('cylinders and cones', () => {
    for (const s of B.cylinders) same(parts.cylinderData(...(s.args as [number, number, number, number])), s);
  });
  test('spheres', () => {
    for (const s of B.spheres) same(parts.sphereData(...(s.args as [number, number, number, number])), s);
  });
  test('tori', () => {
    for (const s of B.tori) same(parts.torusData(...(s.args as [number, number, number])), s);
  });
  test('capsules', () => {
    for (const s of B.capsules) same(parts.capsuleData(...(s.args as [number, number, number, number])), s);
  });
  test('icospheres and their normals', () => {
    for (const s of B.icospheres) same(parts.icoSphereData(...(s.args as [number, number])), s);
    const ours: number[] = [];
    parts.computeNormals(numbers(B.computeNormals.positions), numbers(B.computeNormals.indices), ours);
    close(ours, numbers(B.computeNormals.normals));
  });
});

describe('primitive parts match the Babylon renderer', () => {
  for (const p of B.parts) {
    test(`${p.part.shape} ${p.index} (lod ${p.lod})`, () => {
      const ours = parts.partData(p.part, p.lod, p.index, p.alpha);
      close(ours.positions, or(p.positions));
      close(ours.normals, or(p.normals));
      close(ours.uvs, or(p.uvs));
      close(ours.colors, or(p.colors));
      expect(Array.from(ours.indices)).toEqual(Array.from(numbers(p.indices)));
    });
  }
  test('mirror faces', () => {
    expect(B.mirrors.length).toBe(5);
    for (const { part, round, matrix } of B.mirrors) {
      const ours = parts.mirrorSurface(part);
      expect(ours.round).toBe(round);
      close(ours.matrix, numbers(matrix));
    }
  });
});
