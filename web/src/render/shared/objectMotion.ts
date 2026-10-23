/**
 * The engine-neutral parts of placed objects' motion (`babylon/objectMotion.ts`): where a crop's
 * growth clip stands at each stage, where the residents using an object are (its slots), and the
 * moving parts' poses at rest.
 */

import { objectCentre, type ObjectPlacement } from '../../core/protocol';

/**
 * Where a crop's growth clip stands (shares of it) for each stage, planted to picked: ripe at
 * rest (the catalog's picture), then picked (the crop gone), growing (smaller) and just planted
 * (seedlings). Each stage moves on to the next along it: picking plays forward to picked, planting
 * on to planted, growing back through growing and picked (no crop yet) to ripe.
 */
export const GROWTH_TIMES = [1, 2 / 3, 0, 1 / 3];

/** What `slotPlace` needs of an object's kind. */
type SlotDef = { slots?: number; bunk?: boolean; footprint?: [number, number]; places?: { at: [number, number]; facing?: number }[] };

/**
 * Where the resident in slot `k` of `o` is and which way they face (sim coordinates, yaw as sim-core's), as
 * sim-core's `slot_position` and `slot_yaw` have it: at its place (content `places`, turned and sized with it),
 * else spread along the object's width (a bunk's two a hair apart, the upper one right), facing its front.
 */
export function slotPlace(o: ObjectPlacement, def: SlotDef | undefined, k: number): { x: number; z: number; yaw: number } {
  const c = objectCentre(o);
  const place = def?.places?.[k];
  if (place) {
    const a = (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180;
    const [fw, fd] = def?.footprint ?? [1, 1];
    const x = place.at[0] * (o.size ? o.size[0] / 100 / fw : 1);
    const z = place.at[1] * (o.size ? o.size[1] / 100 / fd : 1);
    return { x: c.x + Math.cos(a) * x + Math.sin(a) * z, z: c.z - Math.sin(a) * x + Math.cos(a) * z, yaw: a + ((place.facing ?? 0) * Math.PI) / 180 };
  }
  const n = Math.max(1, def?.slots ?? 1);
  const yaw = (o.rot * Math.PI) / 2;
  const width = o.size ? o.size[0] / 100 : o.rot % 2 ? o.d : o.w;
  const across = (k - (n - 1) / 2) * (def?.bunk ? 0.04 : width / n);
  return { x: c.x + Math.cos(yaw) * across, z: c.z - Math.sin(yaw) * across, yaw };
}

/** Which of `o`'s slots (content `slots`, from 0) a resident at `x`, `z` is in: the nearest of their places (`slotPlace`). */
export function slotOf(o: ObjectPlacement, def: SlotDef | undefined): (x: number, z: number) => number {
  const places = Array.from({ length: Math.max(1, def?.slots ?? 1) }, (_, k) => slotPlace(o, def, k));
  return (x, z) => {
    let best = 0;
    for (let k = 1; k < places.length; k++) if (Math.hypot(places[k].x - x, places[k].z - z) < Math.hypot(places[best].x - x, places[best].z - z)) best = k;
    return best;
  };
}

/** Poses for `parts` moving parts, all at rest. */
export function restPoses(parts: number): Float32Array {
  const poses = new Float32Array(parts * 16);
  for (let k = 0; k < parts; k++) identityAt(poses, k * 16);
  return poses;
}

/** The identity matrix into `out` at `offset`. */
export function identityAt(out: Float32Array, offset: number): void {
  out.fill(0, offset, offset + 16);
  for (let i = 0; i < 4; i++) out[offset + i * 5] = 1;
}
