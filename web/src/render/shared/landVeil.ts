/**
 * The haze the menus' land fades into at its edge (`pc/landHaze.ts`), as numbers: where it begins and how thick it is
 * at a point. Kept apart from the engine for the tests.
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
