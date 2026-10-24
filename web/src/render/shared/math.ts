// Ported from Babylon.js 9.30: the arithmetic of `Matrix`, `Quaternion` and `Vector3` (`ComposeToRef`,
// `RotationX/Y/Z`, `multiplyToRef`, `RotationYawPitchRollToRef`, `RotationAxis`, the transforms).
// Changed for Idyll Lives: free functions on plain arrays.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * Matrix and quaternion helpers in the game's conventions, engine-neutral: the same numbers
 * Babylon.js gives for the same calls (`Matrix.ComposeToRef`, `Quaternion.RotationYawPitchRollToRef`,
 * ...), so geometry and placements are the same whichever engine draws them. Checked against
 * Babylon's numbers (recorded from Babylon 9.30) by `web/tests/math.test.ts`.
 *
 * Layout: 16 floats, translation at 12, 13, 14 (Babylon's matrices, and PlayCanvas's `Mat4.data`).
 * Babylon multiplies row vectors: `multiply(a, b)` applies `a` first, then `b`, which is
 * PlayCanvas's `mul2(b, a)`. Game space is left-handed, y up (as Babylon's default scene).
 */

export type Mat4 = Float32Array | number[];
export type Quat = [number, number, number, number];

export function identity(out: Mat4): Mat4 {
  out.fill(0);
  out[0] = out[5] = out[10] = out[15] = 1;
  return out;
}

export function translation(x: number, y: number, z: number, out: Mat4): Mat4 {
  identity(out);
  out[12] = x;
  out[13] = y;
  out[14] = z;
  return out;
}

export function scaling(x: number, y: number, z: number, out: Mat4): Mat4 {
  identity(out);
  out[0] = x;
  out[5] = y;
  out[10] = z;
  return out;
}

/** Babylon's `Matrix.RotationX`. */
export function rotationX(angle: number, out: Mat4): Mat4 {
  const s = Math.sin(angle);
  const c = Math.cos(angle);
  identity(out);
  out[5] = c;
  out[6] = s;
  out[9] = -s;
  out[10] = c;
  return out;
}

/** Babylon's `Matrix.RotationY`: a turn of `angle` clockwise seen from above (game space). */
export function rotationY(angle: number, out: Mat4): Mat4 {
  const s = Math.sin(angle);
  const c = Math.cos(angle);
  identity(out);
  out[0] = c;
  out[2] = -s;
  out[8] = s;
  out[10] = c;
  return out;
}

/** Babylon's `Matrix.RotationZ`. */
export function rotationZ(angle: number, out: Mat4): Mat4 {
  const s = Math.sin(angle);
  const c = Math.cos(angle);
  identity(out);
  out[0] = c;
  out[1] = s;
  out[4] = -s;
  out[5] = c;
  return out;
}

/** Babylon's `Quaternion.RotationYawPitchRollToRef` (yaw about y, pitch about x, roll about z). */
export function quatYawPitchRoll(yaw: number, pitch: number, roll: number, out: Quat = [0, 0, 0, 1]): Quat {
  const sr = Math.sin(roll * 0.5);
  const cr = Math.cos(roll * 0.5);
  const sp = Math.sin(pitch * 0.5);
  const cp = Math.cos(pitch * 0.5);
  const sy = Math.sin(yaw * 0.5);
  const cy = Math.cos(yaw * 0.5);
  out[0] = cy * sp * cr + sy * cp * sr;
  out[1] = sy * cp * cr - cy * sp * sr;
  out[2] = cy * cp * sr - sy * sp * cr;
  out[3] = cy * cp * cr + sy * sp * sr;
  return out;
}

/** Babylon's `Quaternion.FromEulerAngles(x, y, z)`: `quatYawPitchRoll(y, x, z)`. */
export function quatFromEuler(x: number, y: number, z: number, out: Quat = [0, 0, 0, 1]): Quat {
  return quatYawPitchRoll(y, x, z, out);
}

/** Babylon's `Quaternion.RotationAxis` (the axis needn't be of unit length). */
export function quatAxisAngle(ax: number, ay: number, az: number, angle: number, out: Quat = [0, 0, 0, 1]): Quat {
  const len = Math.hypot(ax, ay, az) || 1;
  const s = Math.sin(angle / 2);
  out[0] = (ax / len) * s;
  out[1] = (ay / len) * s;
  out[2] = (az / len) * s;
  out[3] = Math.cos(angle / 2);
  return out;
}

/** Babylon's `Matrix.ComposeToRef(scale, rotation, translation)`: scaled, then turned, then moved. */
export function compose(sx: number, sy: number, sz: number, q: Readonly<Quat>, tx: number, ty: number, tz: number, out: Mat4): Mat4 {
  // (Read by index: a destructuring allocates an iterator until the function is optimised.)
  const x = q[0];
  const y = q[1];
  const z = q[2];
  const w = q[3];
  const x2 = x + x;
  const y2 = y + y;
  const z2 = z + z;
  const xx = x * x2;
  const xy = x * y2;
  const xz = x * z2;
  const yy = y * y2;
  const yz = y * z2;
  const zz = z * z2;
  const wx = w * x2;
  const wy = w * y2;
  const wz = w * z2;
  out[0] = (1 - (yy + zz)) * sx;
  out[1] = (xy + wz) * sx;
  out[2] = (xz - wy) * sx;
  out[3] = 0;
  out[4] = (xy - wz) * sy;
  out[5] = (1 - (xx + zz)) * sy;
  out[6] = (yz + wx) * sy;
  out[7] = 0;
  out[8] = (xz + wy) * sz;
  out[9] = (yz - wx) * sz;
  out[10] = (1 - (xx + yy)) * sz;
  out[11] = 0;
  out[12] = tx;
  out[13] = ty;
  out[14] = tz;
  out[15] = 1;
  return out;
}

/** Babylon's `a.multiply(b)`: `a` applied first, then `b`. `out` may be `a` or `b`. */
export function multiply(a: Readonly<Mat4>, b: Readonly<Mat4>, out: Mat4): Mat4 {
  const r = scratch;
  for (let i = 0; i < 4; i++) {
    const a0 = a[i * 4];
    const a1 = a[i * 4 + 1];
    const a2 = a[i * 4 + 2];
    const a3 = a[i * 4 + 3];
    for (let j = 0; j < 4; j++) r[i * 4 + j] = a0 * b[j] + a1 * b[4 + j] + a2 * b[8 + j] + a3 * b[12 + j];
  }
  for (let k = 0; k < 16; k++) out[k] = r[k];
  return out;
}
const scratch = new Float64Array(16);

/** The inverse of `m` (identity when it has none). `out` may be `m`. */
export function invert(m: Readonly<Mat4>, out: Mat4): Mat4 {
  const [m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33] = m;
  const b00 = m00 * m11 - m01 * m10;
  const b01 = m00 * m12 - m02 * m10;
  const b02 = m00 * m13 - m03 * m10;
  const b03 = m01 * m12 - m02 * m11;
  const b04 = m01 * m13 - m03 * m11;
  const b05 = m02 * m13 - m03 * m12;
  const b06 = m20 * m31 - m21 * m30;
  const b07 = m20 * m32 - m22 * m30;
  const b08 = m20 * m33 - m23 * m30;
  const b09 = m21 * m32 - m22 * m31;
  const b10 = m21 * m33 - m23 * m31;
  const b11 = m22 * m33 - m23 * m32;
  const det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
  if (det === 0) return identity(out);
  const d = 1 / det;
  out[0] = (m11 * b11 - m12 * b10 + m13 * b09) * d;
  out[1] = (m02 * b10 - m01 * b11 - m03 * b09) * d;
  out[2] = (m31 * b05 - m32 * b04 + m33 * b03) * d;
  out[3] = (m22 * b04 - m21 * b05 - m23 * b03) * d;
  out[4] = (m12 * b08 - m10 * b11 - m13 * b07) * d;
  out[5] = (m00 * b11 - m02 * b08 + m03 * b07) * d;
  out[6] = (m32 * b02 - m30 * b05 - m33 * b01) * d;
  out[7] = (m20 * b05 - m22 * b02 + m23 * b01) * d;
  out[8] = (m10 * b10 - m11 * b08 + m13 * b06) * d;
  out[9] = (m01 * b08 - m00 * b10 - m03 * b06) * d;
  out[10] = (m30 * b04 - m31 * b02 + m33 * b00) * d;
  out[11] = (m21 * b02 - m20 * b04 - m23 * b00) * d;
  out[12] = (m11 * b07 - m10 * b09 - m12 * b06) * d;
  out[13] = (m00 * b09 - m01 * b07 + m02 * b06) * d;
  out[14] = (m31 * b01 - m30 * b03 - m32 * b00) * d;
  out[15] = (m20 * b03 - m21 * b01 + m22 * b00) * d;
  return out;
}

/** Babylon's `Vector3.TransformCoordinatesFromFloatsToRef`: the point (x, y, z) through `m` (with the perspective divide). */
export function transformPoint(m: Readonly<Mat4>, x: number, y: number, z: number, out: number[] | Float32Array = [0, 0, 0]): number[] | Float32Array {
  const w = x * m[3] + y * m[7] + z * m[11] + m[15];
  const k = w !== 0 ? 1 / w : 1;
  out[0] = (x * m[0] + y * m[4] + z * m[8] + m[12]) * k;
  out[1] = (x * m[1] + y * m[5] + z * m[9] + m[13]) * k;
  out[2] = (x * m[2] + y * m[6] + z * m[10] + m[14]) * k;
  return out;
}

/** Babylon's `Vector3.TransformNormalFromFloatsToRef`: the direction (x, y, z) through `m`'s rotation and scale. */
export function transformNormal(m: Readonly<Mat4>, x: number, y: number, z: number, out: number[] | Float32Array = [0, 0, 0]): number[] | Float32Array {
  out[0] = x * m[0] + y * m[4] + z * m[8];
  out[1] = x * m[1] + y * m[5] + z * m[9];
  out[2] = x * m[2] + y * m[6] + z * m[10];
  return out;
}

/** An sRGB colour channel (0..1) in linear light, as Babylon's `Color3.toLinearSpace` (a 2.2 power). */
export function toLinear(c: number): number {
  return Math.pow(c, 2.2);
}
