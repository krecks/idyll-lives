// `render/shared/math.ts` against Babylon.js: the same numbers for the same calls (`bun test`).
import { describe, expect, test } from 'bun:test';
import { Color3 } from '@babylonjs/core/Maths/math.color';
import { Matrix, Quaternion, Vector3 } from '@babylonjs/core/Maths/math.vector';
import * as m from '../src/render/shared/math';

/** A fixed sequence of numbers in -4..4 (the same every run). */
function numbers(seed: number): () => number {
  let s = seed >>> 0;
  return () => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 2 ** 32) * 8 - 4;
}
const close = (a: ArrayLike<number>, b: ArrayLike<number>, eps = 1e-5) => {
  expect(a.length).toBe(b.length);
  for (let i = 0; i < a.length; i++) expect(Math.abs(a[i] - b[i])).toBeLessThan(eps);
};
const arr = (x: Matrix) => Array.from(x.asArray());
const quat = (q: Quaternion) => [q.x, q.y, q.z, q.w];
const CASES = 50;

describe('shared/math matches Babylon', () => {
  const r = numbers(7);

  test('rotations about x, y and z', () => {
    for (let i = 0; i < CASES; i++) {
      const a = r();
      close(m.rotationX(a, new Float32Array(16)), arr(Matrix.RotationX(a)));
      close(m.rotationY(a, new Float32Array(16)), arr(Matrix.RotationY(a)));
      close(m.rotationZ(a, new Float32Array(16)), arr(Matrix.RotationZ(a)));
    }
  });

  test('translation and scaling', () => {
    const [x, y, z] = [r(), r(), r()];
    close(m.translation(x, y, z, new Float32Array(16)), arr(Matrix.Translation(x, y, z)));
    close(m.scaling(x, y, z, new Float32Array(16)), arr(Matrix.Scaling(x, y, z)));
  });

  test('quaternions: yaw, pitch, roll; Euler angles; axis and angle', () => {
    for (let i = 0; i < CASES; i++) {
      const [y, p, ro] = [r(), r(), r()];
      close(m.quatYawPitchRoll(y, p, ro), quat(Quaternion.RotationYawPitchRoll(y, p, ro)));
      close(m.quatFromEuler(p, y, ro), quat(Quaternion.FromEulerAngles(p, y, ro)));
      const [ax, ay, az, angle] = [r(), r(), r(), r()];
      close(m.quatAxisAngle(ax, ay, az, angle), quat(Quaternion.RotationAxis(new Vector3(ax, ay, az), angle)));
    }
  });

  test('compose: scaled, turned, moved', () => {
    for (let i = 0; i < CASES; i++) {
      const s = [r(), r(), r()] as const;
      const q = m.quatYawPitchRoll(r(), r(), r());
      const t = [r(), r(), r()] as const;
      const expected = Matrix.Compose(new Vector3(...s), new Quaternion(...q), new Vector3(...t));
      close(m.compose(s[0], s[1], s[2], q, t[0], t[1], t[2], new Float32Array(16)), arr(expected));
    }
  });

  test('multiply: a first, then b (in place too)', () => {
    for (let i = 0; i < CASES; i++) {
      const a = Matrix.Compose(new Vector3(r(), r(), r()), Quaternion.RotationYawPitchRoll(r(), r(), r()), new Vector3(r(), r(), r()));
      const b = Matrix.Compose(new Vector3(r(), r(), r()), Quaternion.RotationYawPitchRoll(r(), r(), r()), new Vector3(r(), r(), r()));
      const expected = arr(a.multiply(b));
      close(m.multiply(arr(a), arr(b), new Float32Array(16)), expected, 1e-3);
      const inPlace = arr(a);
      close(m.multiply(inPlace, arr(b), inPlace), expected, 1e-3);
    }
  });

  test('invert', () => {
    for (let i = 0; i < CASES; i++) {
      const a = Matrix.Compose(new Vector3(1 + Math.abs(r()), 1 + Math.abs(r()), 1 + Math.abs(r())), Quaternion.RotationYawPitchRoll(r(), r(), r()), new Vector3(r(), r(), r()));
      close(m.invert(arr(a), new Float32Array(16)), arr(Matrix.Invert(a)), 1e-4);
    }
  });

  test('points and directions through a matrix', () => {
    for (let i = 0; i < CASES; i++) {
      const a = Matrix.Compose(new Vector3(r(), r(), r()), Quaternion.RotationYawPitchRoll(r(), r(), r()), new Vector3(r(), r(), r()));
      const [x, y, z] = [r(), r(), r()];
      const p = Vector3.TransformCoordinatesFromFloatsToRef(x, y, z, a, new Vector3());
      close(m.transformPoint(arr(a), x, y, z), [p.x, p.y, p.z], 1e-3);
      const n = Vector3.TransformNormalFromFloatsToRef(x, y, z, a, new Vector3());
      close(m.transformNormal(arr(a), x, y, z), [n.x, n.y, n.z], 1e-3);
    }
  });

  test('sRGB to linear', () => {
    for (const c of [0, 0.04, 0.2, 0.5, 0.73, 1]) expect(Math.abs(m.toLinear(c) - new Color3(c, c, c).toLinearSpace().r)).toBeLessThan(1e-6);
  });
});
