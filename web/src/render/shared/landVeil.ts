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

/**
 * While the curtain closing the landscape's cut edge is drawn (`curtain`), the veil rises to whole over the landscape's
 * last this many metres whatever its strength (m; `shaders/fog.ts`): from a camera out over the band at its edge, the
 * veil at a quarter of its strength there, the curtain stood as a straight wall of the sky's colour across land and water
 * barely hazed. Near the eye over more of it (`VEIL_EDGE_SOFT`).
 */
export const VEIL_EDGE = 60;

/**
 * Within the first of these from the eye (m), the veil rises to whole over the whole band (`VEIL_FADE`) while the curtain
 * is drawn, past the second over its last `VEIL_EDGE` m, between them over as much between as it is (`veilShown`): from
 * a camera low in the band looking along it, the veil rose from 0.44 to whole over its last 60 m, a few hundred pixels
 * across, and what it took stood as a pale wall (the plane 60 m in from the edge, seen edge on) over the land and the
 * water. Further off, the ring's far hills and the mountains round a mountain place are in no more of the haze than they
 * were, the band a sliver of the picture there.
 */
export const VEIL_EDGE_SOFT = [500, 1000] as const;

/**
 * How thick the veil is `out` m past its clear rectangle (its `fade` m to whole), at `strength` and, while the curtain
 * is drawn, rising to whole over its last `edge` m (0: not) or more of it near the eye (`VEIL_EDGE_SOFT`; `eye` m from
 * it): as the shaders work it out (`shaders/fog.ts` `veilStrength`), but for the veil's clearing at the eye.
 */
export function veilShown(fade: number, strength: number, edge: number, out: number, eye = Infinity): number {
  const over = fade + (Math.max(edge, 1e-3) - fade) * smoothstep(VEIL_EDGE_SOFT[0], VEIL_EDGE_SOFT[1], eye);
  const most = edge > 1e-6 ? smoothstep(fade - over, fade, out) : 0;
  return Math.max(strength, most) * smoothstep(0, fade, out);
}

/** A box of the world (m). */
interface VeilBox {
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

/** `cutInView` is whole once the cut edge is this share of the half picture in from the picture's edge. */
const CUT_EASE = 0.15;

/**
 * How far the cut edge of the landscape `box`, closed by its curtain (`curtain`'s `positions`: each sample's top, then
 * each one's foot), is in the picture of a camera whose eye is at (`ex`, `ey`, `ez`), looking as an orbit at `alpha`,
 * `beta` does (towards -(cos α sin β, cos β, sin α sin β)), its vertical field of view `fov` (rad) and its `aspect`: 0
 * while none of it on a side the eye stands past shows (over the landscape, or past it but looking down on it steeply
 * enough that the side under it is below the picture: a side's face shows only from past it), over 0 once some does
 * (the curtain drawn then closes it), whole once it is `CUT_EASE` of the half picture in. The game's town view from far
 * out over a small land's corner. Allocation-free; nothing to work out from over the landscape.
 */
export function cutInView(box: VeilBox, positions: Float32Array, ex: number, ey: number, ez: number, alpha: number, beta: number, fov: number, aspect: number): number {
  const past0 = ex < box.x0;
  const past1 = ex > box.x1;
  const past2 = ez < box.z0;
  const past3 = ez > box.z1;
  if (!past0 && !past1 && !past2 && !past3) return 0;
  // (The view's way in, right and up: a left-handed look-at, up y, as `overviewCamera.ts` `viewShift`'s.)
  const [sa, ca, sb, cb] = [Math.sin(alpha), Math.cos(alpha), Math.sin(beta), Math.cos(beta)];
  const [fx, fy, fz] = [-ca * sb, -cb, -sa * sb];
  const rl = Math.hypot(fz, fx) || 1;
  const [rx, rz] = [fz / rl, -fx / rl];
  const [ux, uy, uz] = [fy * rz, fz * rx - fx * rz, -fy * rx];
  const tanY = Math.tan(fov / 2);
  const tanX = tanY * aspect;
  let most = -Infinity;
  const n = positions.length / 6;
  for (let i = 0; i < n * 2; i++) {
    const [x, y, z] = [positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]];
    // (Only the sides the eye is past: the curtain's on the others is turned away from it.)
    if (!((past0 && x < box.x0) || (past1 && x > box.x1) || (past2 && z < box.z0) || (past3 && z > box.z1))) continue;
    const [dx, dy, dz] = [x - ex, y - ey, z - ez];
    const depth = dx * fx + dy * fy + dz * fz;
    if (depth <= 0) continue;
    const across = Math.abs(dx * rx + dz * rz) / (depth * tanX);
    const up = Math.abs(dx * ux + dy * uy + dz * uz) / (depth * tanY);
    most = Math.max(most, 1 - Math.max(across, up));
  }
  return most <= 0 ? 0 : smoothstep(0, CUT_EASE, most);
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
