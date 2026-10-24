/**
 * Where the things residents hold go (`residents.ts` `writeHeld`, `writeCarried`, `writeTool`): the palms as
 * just posed, and a prop's matrix in its row of the prop's instances (`ResidentProp.matrices`).
 */

import { DEFAULT_GRIP } from './actions';
import type { PoseScratch } from './pose';
import type { SimRig } from './rig';

/** The palm centre of hand bone `b` (`side` 1 right, -1 left) as last posed, into `out`. */
export function palmAt(scratch: PoseScratch, rig: SimRig, b: number, side: number, out: Float32Array): void {
  const t = scratch.worldT;
  const q = scratch.worldQ;
  const g = rig.body.grip ?? DEFAULT_GRIP;
  const s = rig.scale;
  // (The grip is the left palm's offset from the wrist in the hand's rest frame; the right mirrors x.)
  const gx = -g[0] * side * s;
  const gy = g[1] * s;
  const gz = g[2] * s;
  const qx = q[b * 4];
  const qy = q[b * 4 + 1];
  const qz = q[b * 4 + 2];
  const qw = q[b * 4 + 3];
  // Rotated by the hand: v + 2 w (q x v) + 2 q x (q x v).
  const cx = qy * gz - qz * gy;
  const cy = qz * gx - qx * gz;
  const cz = qx * gy - qy * gx;
  out[0] = t[b * 3] + gx + 2 * (qw * cx + qy * cz - qz * cy);
  out[1] = t[b * 3 + 1] + gy + 2 * (qw * cy + qz * cx - qx * cz);
  out[2] = t[b * 3 + 2] + gz + 2 * (qw * cz + qx * cy - qy * cx);
}

/** Writes a prop placed by the frame `f` (its x, y and z axes, then its origin; scaled by `scale`) into row `i` of `m`. */
export function placeFrame(m: Float32Array, i: number, f: Float32Array, scale: number): void {
  const o = i * 16;
  for (let r = 0; r < 3; r++) {
    m[o + r * 4] = f[r * 3] * scale;
    m[o + r * 4 + 1] = f[r * 3 + 1] * scale;
    m[o + r * 4 + 2] = f[r * 3 + 2] * scale;
    m[o + r * 4 + 3] = 0;
  }
  m[o + 12] = f[9];
  m[o + 13] = f[10];
  m[o + 14] = f[11];
  m[o + 15] = 1;
}

/** Writes a level prop (turned to `yaw`) at a point into row `i` of `m`. */
export function placeProp(m: Float32Array, i: number, x: number, y: number, z: number, yaw: number, scale: number): void {
  const c = Math.cos(yaw) * scale;
  const s = Math.sin(yaw) * scale;
  const o = i * 16;
  m.fill(0, o, o + 16);
  m[o] = c;
  m[o + 2] = -s;
  m[o + 5] = scale;
  m[o + 8] = s;
  m[o + 10] = c;
  m[o + 12] = x;
  m[o + 13] = y;
  m[o + 14] = z;
  m[o + 15] = 1;
}
