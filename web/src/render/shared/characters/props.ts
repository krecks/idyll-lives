/**
 * Where the things residents hold go (`residents.ts` `writeHeld`, `writeCarried`, `writeTool`, `writeHand`): the
 * palms as just posed, and a prop's matrix in its row of the prop's instances (`ResidentProp.matrices`).
 */

import { DEFAULT_GRIP, type HandProp } from './actions';
import { BALL_R, ballAt } from './clips/shot';
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

/** The hands as just posed, for `placeHand`: each palm's centre and its hand's world rotation, the head, the facing and the floor. */
export interface Hands {
  /** Palm centres (x, y, z): the right's, then the left's. */
  palms: Float32Array;
  /** The hands' world rotations (quaternions x, y, z, w): the right's, then the left's. */
  quats: Float32Array;
  /** The head joint (x, y, z). */
  head: Float32Array;
  yaw: number;
  floorY: number;
  /** The body's scale (m per m of the reference), and the props' (the hands' size: `min(1, scale)`). */
  scale: number;
  size: number;
  /** For the basketball (`ball`): the round's clip and how far into it (s), the rim's middle and the shooter's feet (x, z). */
  ball?: { clip: string; t: number; rim: Float32Array; feet: Float32Array };
}

const D = Math.PI / 180;
const A = new Float32Array(9);
const V = new Float32Array(3);
const F = new Float32Array(12);

/** `(x, y, z)` turned by the quaternion at `qo` of `q`, into `out` at `o`. */
function turn(q: Float32Array, qo: number, x: number, y: number, z: number, out: Float32Array, o: number): void {
  const qx = q[qo];
  const qy = q[qo + 1];
  const qz = q[qo + 2];
  const qw = q[qo + 3];
  const cx = qy * z - qz * y;
  const cy = qz * x - qx * z;
  const cz = qx * y - qy * x;
  out[o] = x + 2 * (qw * cx + qy * cz - qz * cy);
  out[o + 1] = y + 2 * (qw * cy + qz * cx - qx * cz);
  out[o + 2] = z + 2 * (qw * cz + qx * cy - qy * cx);
}

/** The axes (x, y, z columns, into `A`) of turns `t` degrees about x, then y, then z (`Rz Ry Rx`), mirrored for the left hand. */
function eulerAxes(t: readonly [number, number, number] | undefined, left: boolean): void {
  const x = (t?.[0] ?? 0) * D;
  const y = (t?.[1] ?? 0) * D * (left ? -1 : 1);
  const z = (t?.[2] ?? 0) * D * (left ? -1 : 1);
  const cx = Math.cos(x);
  const sx = Math.sin(x);
  const cy = Math.cos(y);
  const sy = Math.sin(y);
  const cz = Math.cos(z);
  const sz = Math.sin(z);
  A[0] = cz * cy;
  A[1] = sz * cy;
  A[2] = -sy;
  A[3] = cz * sy * sx - sz * cx;
  A[4] = sz * sy * sx + cz * cx;
  A[5] = cy * sx;
  A[6] = cz * sy * cx + sz * sx;
  A[7] = sz * sy * cx - cz * sx;
  A[8] = cy * cx;
}

function unitInto(out: Float32Array, o: number, x: number, y: number, z: number): void {
  const n = Math.hypot(x, y, z) || 1;
  out[o] = x / n;
  out[o + 1] = y / n;
  out[o + 2] = z / n;
}

/** `F`'s third axis from its first two (x × y), so the frame turns and doesn't mirror. */
function third(): void {
  F[6] = F[1] * F[5] - F[2] * F[4];
  F[7] = F[2] * F[3] - F[0] * F[5];
  F[8] = F[0] * F[4] - F[1] * F[3];
}

function smoothstep(t: number): number {
  const u = Math.min(1, Math.max(0, t));
  return u * u * (3 - 2 * u);
}

/** `F`'s origin: `(x, y, z)` moved `a` along its own axes. */
function along(x: number, y: number, z: number, a: Float32Array): void {
  F[9] = x + F[0] * a[0] + F[3] * a[1] + F[6] * a[2];
  F[10] = y + F[1] * a[0] + F[4] * a[1] + F[7] * a[2];
  F[11] = z + F[2] * a[0] + F[5] * a[1] + F[8] * a[2];
}

/**
 * Writes prop `p` (actions.ts `HandProp`) held in the hands `h` as just posed into row `i` of `m`, by its grip, from the
 * palm of the hand that holds it (or both).
 */
export function placeHand(m: Float32Array, i: number, p: HandProp, h: Hands): void {
  const left = p.hand === 'l';
  // This hand's palm and rotation, and the other palm.
  const b = left ? 3 : 0;
  const o = left ? 0 : 3;
  const qo = left ? 4 : 0;
  const P = h.palms;
  const k = h.size;
  const at = p.at ?? [0, 0, 0];
  V[0] = (left ? -at[0] : at[0]) * k;
  V[1] = at[1] * k;
  V[2] = at[2] * k;
  const fx = Math.sin(h.yaw);
  const fz = Math.cos(h.yaw);
  switch (p.grip) {
    case 'hand': {
      eulerAxes(p.turn, left);
      for (let c = 0; c < 3; c++) turn(h.quats, qo, A[c * 3], A[c * 3 + 1], A[c * 3 + 2], F, c * 3);
      turn(h.quats, qo, V[0], V[1], V[2], F, 9);
      F[9] += P[b];
      F[10] += P[b + 1];
      F[11] += P[b + 2];
      break;
    }
    case 'level': {
      // Upright, turned the way they face; a cup tipped to the lips as the hand comes up to the mouth, about where the
      // hand holds it (`at` along their right and ahead of the palm; its base `at[1]` up from there along it).
      const a = p.sip ? 55 * D * smoothstep((P[b + 1] - (h.head[1] - 0.3 * h.scale)) / (0.2 * h.scale)) : 0;
      const c = Math.cos(a);
      const s = Math.sin(a);
      F[0] = fz;
      F[1] = 0;
      F[2] = -fx;
      F[3] = -fx * s;
      F[4] = c;
      F[5] = -fz * s;
      third();
      F[9] = P[b] + fz * V[0] + fx * V[2] + F[3] * V[1];
      F[10] = P[b + 1] + F[4] * V[1];
      F[11] = P[b + 2] - fx * V[0] + fz * V[2] + F[5] * V[1];
      break;
    }
    case 'face': {
      // Across the palms, left to right; its face to the eyes (a little ahead of the head joint).
      const mx = (P[0] + P[3]) / 2;
      const my = (P[1] + P[4]) / 2;
      const mz = (P[2] + P[5]) / 2;
      unitInto(F, 0, P[0] - P[3], P[1] - P[4], P[2] - P[5]);
      let nx = h.head[0] + fx * 0.08 * h.scale - mx;
      let ny = h.head[1] - my;
      let nz = h.head[2] + fz * 0.08 * h.scale - mz;
      const d = nx * F[0] + ny * F[1] + nz * F[2];
      nx -= d * F[0];
      ny -= d * F[1];
      nz -= d * F[2];
      unitInto(F, 3, nx, ny, nz);
      third();
      along(mx, my, mz, V);
      break;
    }
    case 'line': {
      // Up the line from this palm to the other, its front the way they face, its x their right.
      unitInto(F, 3, P[o] - P[b], P[o + 1] - P[b + 1], P[o + 2] - P[b + 2]);
      const d = fx * F[3] + fz * F[5];
      unitInto(F, 6, fx - d * F[3], -d * F[4], fz - d * F[5]);
      F[0] = F[4] * F[8] - F[5] * F[7];
      F[1] = F[5] * F[6] - F[3] * F[8];
      F[2] = F[3] * F[7] - F[4] * F[6];
      along(P[b], P[b + 1], P[b + 2], V);
      break;
    }
    case 'bite': {
      // From the palm to the mouth (a little ahead of and under the head joint), its face up, moved `at` along itself.
      const s = h.scale;
      unitInto(F, 6, h.head[0] + fx * 0.1 * s - P[b], h.head[1] - 0.03 * s - P[b + 1], h.head[2] + fz * 0.1 * s - P[b + 2]);
      const d = fz * F[6] - fx * F[8];
      unitInto(F, 0, fz - d * F[6], -d * F[7], -fx - d * F[8]);
      F[3] = F[7] * F[2] - F[8] * F[1];
      F[4] = F[8] * F[0] - F[6] * F[2];
      F[5] = F[6] * F[1] - F[7] * F[0];
      along(P[b], P[b + 1], P[b + 2], V);
      break;
    }
    case 'cue': {
      // Out from the right palm through the left; upright in the left hand, its butt to the floor, with the right above it.
      unitInto(A, 0, P[3] - P[0], P[4] - P[1], P[5] - P[2]);
      const up = smoothstep((-A[1] - 0.2) / 0.5);
      unitInto(F, 6, A[0] * (1 - up), A[1] * (1 - up) + up, A[2] * (1 - up));
      // Its side: their right, square to it; y = z × x.
      const d = fz * F[6] - fx * F[8];
      unitInto(F, 0, fz - d * F[6], -d * F[7], -fx - d * F[8]);
      F[3] = F[7] * F[2] - F[8] * F[1];
      F[4] = F[8] * F[0] - F[6] * F[2];
      F[5] = F[6] * F[1] - F[7] * F[0];
      // The butt `at[2]` back from the right palm along it; or, upright, on the floor under the left hand.
      const reach = (P[4] - h.floorY) / Math.max(0.3, F[7]);
      for (let c = 0; c < 3; c++) F[9 + c] = (P[c] - F[6 + c] * V[2]) * (1 - up) + (P[3 + c] - F[6 + c] * reach) * up;
      break;
    }
    case 'ball': {
      // Upright, its middle where the round has it; its origin is at its foot.
      const b = h.ball;
      if (!b || !ballAt(b.clip, b.t, P, b.rim, b.feet, h.yaw, h.floorY, h.scale, k, A)) return;
      F.fill(0, 0, 9);
      F[0] = F[4] = F[8] = 1;
      F[9] = A[0];
      F[10] = A[1] - BALL_R * k;
      F[11] = A[2];
      break;
    }
  }
  placeFrame(m, i, F, k);
}
