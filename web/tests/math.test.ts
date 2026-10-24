// `render/shared/math.ts` against Babylon.js: the same numbers for the same calls (`bun test`). Babylon's
// numbers, and the calls' inputs, are fixtures recorded from Babylon 9.30 (`fixtures/babylon/math.json`,
// made by `fixtures/record-babylon.ts`, which needs a Babylon reference checkout to run again).
import { describe, expect, test } from 'bun:test';
import * as m from '../src/render/shared/math';
import { fixture } from './fixtures/babylon';

type Mat = number[];
type Quat = [number, number, number, number];
const B = fixture<{
  rotations: { a: number; x: Mat; y: Mat; z: Mat }[];
  translation: { at: [number, number, number]; translation: Mat; scaling: Mat };
  quaternions: { yaw: number; pitch: number; roll: number; yawPitchRoll: Quat; euler: Quat; axis: [number, number, number]; angle: number; axisAngle: Quat }[];
  compose: { s: [number, number, number]; q: Quat; t: [number, number, number]; m: Mat }[];
  multiply: { a: Mat; b: Mat; ab: Mat }[];
  invert: { a: Mat; inverse: Mat }[];
  transform: { m: Mat; p: [number, number, number]; point: number[]; normal: number[] }[];
  toLinear: [number, number][];
}>('math');

const close = (a: ArrayLike<number>, b: ArrayLike<number>, eps = 1e-5) => {
  expect(a.length).toBe(b.length);
  for (let i = 0; i < a.length; i++) expect(Math.abs(a[i] - b[i])).toBeLessThan(eps);
};

describe('shared/math matches Babylon', () => {
  test('rotations about x, y and z', () => {
    expect(B.rotations.length).toBe(50);
    for (const { a, x, y, z } of B.rotations) {
      close(m.rotationX(a, new Float32Array(16)), x);
      close(m.rotationY(a, new Float32Array(16)), y);
      close(m.rotationZ(a, new Float32Array(16)), z);
    }
  });

  test('translation and scaling', () => {
    const [x, y, z] = B.translation.at;
    close(m.translation(x, y, z, new Float32Array(16)), B.translation.translation);
    close(m.scaling(x, y, z, new Float32Array(16)), B.translation.scaling);
  });

  test('quaternions: yaw, pitch, roll; Euler angles; axis and angle', () => {
    for (const { yaw, pitch, roll, yawPitchRoll, euler, axis, angle, axisAngle } of B.quaternions) {
      close(m.quatYawPitchRoll(yaw, pitch, roll), yawPitchRoll);
      close(m.quatFromEuler(pitch, yaw, roll), euler);
      close(m.quatAxisAngle(axis[0], axis[1], axis[2], angle), axisAngle);
    }
  });

  test('compose: scaled, turned, moved', () => {
    for (const { s, q, t, m: expected } of B.compose) close(m.compose(s[0], s[1], s[2], q, t[0], t[1], t[2], new Float32Array(16)), expected);
  });

  test('multiply: a first, then b (in place too)', () => {
    for (const { a, b, ab } of B.multiply) {
      close(m.multiply(a, b, new Float32Array(16)), ab, 1e-3);
      const inPlace = a.slice();
      close(m.multiply(inPlace, b, inPlace), ab, 1e-3);
    }
  });

  test('invert', () => {
    for (const { a, inverse } of B.invert) close(m.invert(a, new Float32Array(16)), inverse, 1e-4);
  });

  test('points and directions through a matrix', () => {
    for (const { m: a, p, point, normal } of B.transform) {
      close(m.transformPoint(a, p[0], p[1], p[2]), point, 1e-3);
      close(m.transformNormal(a, p[0], p[1], p[2]), normal, 1e-3);
    }
  });

  test('sRGB to linear', () => {
    for (const [c, linear] of B.toLinear) expect(Math.abs(m.toLinear(c) - linear)).toBeLessThan(1e-6);
  });
});
