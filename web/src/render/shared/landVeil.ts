/**
 * The haze the menus' land fades into at its edge (`pc/landHaze.ts`), as numbers: where it begins and how thick it is
 * at a point, and the curtain that closes the landscape's cut edge. Kept apart from the engine for the tests.
 *
 * The veil is the haze laid over whatever stands at a point, by where it stands (its x and z): clear at the rectangle
 * `VEIL_FADE` m in from the landscape's edge, whole at the edge and past it, the same distance from that rectangle all
 * the way round (round at its corners). The game's lit materials take it in their haze (`shaders/fog.ts`), so the land,
 * its woods and its water fade by the ground each pixel shows, from any camera.
 */

import { smoothstep } from '../../game/noise';

/** How far in from the landscape's edge the veil begins (m). */
export const VEIL_FADE = 240;
/**
 * The veil is clear at the eye and whole from this far from it (m): a camera out over the landscape's edge sees the
 * trees and the ground round it as they are, and the haze beyond them. From the whole land's shot, all it sees is
 * further than this.
 */
export const VEIL_NEAR = 200;

/** A box of the world (m). */
export interface VeilBox {
  x0: number;
  z0: number;
  x1: number;
  z1: number;
}

/** Where the veil is clear: the rectangle's middle and half its sides (m), and how far past it the veil is whole (m). */
export interface VeilRect {
  cx: number;
  cz: number;
  hx: number;
  hz: number;
  fade: number;
}

/** The veil for the landscape `box`: `VEIL_FADE` in from its edge (less in a box too small for it, never under 1 m). */
export function veilRect(box: VeilBox): VeilRect {
  const fade = Math.max(1, Math.min(VEIL_FADE, (box.x1 - box.x0) / 2, (box.z1 - box.z0) / 2));
  return { cx: (box.x0 + box.x1) / 2, cz: (box.z0 + box.z1) / 2, hx: Math.max(0, (box.x1 - box.x0) / 2 - fade), hz: Math.max(0, (box.z1 - box.z0) / 2 - fade), fade };
}

/**
 * How thick the veil is at (`x`, `z`), 0 (clear) .. 1 (whole), and with `eye` (m from the eye) as seen from there:
 * as the shaders work it out (`shaders/fog.ts`).
 */
export function veilAt(rect: VeilRect, x: number, z: number, eye = Infinity): number {
  const dx = Math.max(0, Math.abs(x - rect.cx) - rect.hx);
  const dz = Math.max(0, Math.abs(z - rect.cz) - rect.hz);
  return smoothstep(0, rect.fade, Math.hypot(dx, dz)) * smoothstep(0, VEIL_NEAR, eye);
}

/** The curtain's samples along the landscape's edge (m). */
const CURTAIN_STEP = 4;
/** How far past the landscape's edge the curtain hangs (m): in front of the ground's own skirt there, not in it. */
const CURTAIN_OUT = 0.5;
/** How far over the ground at the edge its top reaches (m), and under the lowest of it (and the sea's sheet) its foot. */
const CURTAIN_OVER = 0.3;
const CURTAIN_UNDER = 25;

/**
 * The curtain round the landscape `box`: a band just past its edge from over the ground there down to well under it
 * and the sea's sheet (`floor`, m), so from a camera past the edge the landscape ends in the haze rather than in its
 * cut (the ground tiles' skirts, and under the hills the sea's sheet the whole square has). Its top at each sample is
 * over the highest ground within a sample either side of it (the ground's coarser levels go straight between their
 * points, under that). Positions (x, y, z) of each sample's top and then its foot, round the box once (closed), and
 * the triangles' indices.
 */
export function curtain(box: VeilBox, height: (x: number, z: number) => number, floor: number): { positions: Float32Array; indices: Uint32Array } {
  const { x0, z0, x1, z1 } = box;
  // Round the box: each side from its corner to the next, its samples (the next corner is the next side's first).
  const corners: [number, number][] = [
    [x0, z0],
    [x1, z0],
    [x1, z1],
    [x0, z1],
  ];
  const at: [number, number][] = [];
  for (let k = 0; k < 4; k++) {
    const [ax, az] = corners[k];
    const [bx, bz] = corners[(k + 1) % 4];
    const n = Math.max(1, Math.ceil(Math.hypot(bx - ax, bz - az) / CURTAIN_STEP));
    for (let i = 0; i < n; i++) at.push([ax + ((bx - ax) * i) / n, az + ((bz - az) * i) / n]);
  }
  const n = at.length;
  const ground = at.map(([x, z]) => height(x, z));
  const top = ground.map((h, i) => Math.max(h, ground[(i + n - 1) % n], ground[(i + 1) % n]) + CURTAIN_OVER);
  const foot = Math.min(floor, ...ground) - CURTAIN_UNDER;
  const positions = new Float32Array(n * 6);
  at.forEach(([x, z], i) => {
    // (Out past whichever side it is on; a corner's both ways.)
    const ox = x <= x0 ? -CURTAIN_OUT : x >= x1 ? CURTAIN_OUT : 0;
    const oz = z <= z0 ? -CURTAIN_OUT : z >= z1 ? CURTAIN_OUT : 0;
    const [px, pz] = [x + ox, z + oz];
    positions.set([px, top[i], pz], i * 3);
    positions.set([px, foot, pz], (n + i) * 3);
  });
  const indices = new Uint32Array(n * 6);
  for (let i = 0; i < n; i++) {
    const j = (i + 1) % n;
    indices.set([i, j, n + j, i, n + j, n + i], i * 6);
  }
  return { positions, indices };
}
