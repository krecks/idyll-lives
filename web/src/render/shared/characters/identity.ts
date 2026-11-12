/**
 * A resident's own face: the identity channels (`id.<name>` morphs after the expressions', made by
 * `cargo characters`, crates/tools/src/characters/identity.rs) at the weights of its look
 * (`Appearance.face`, household.ts; a look without one gets a face from the resident's id, so nobody
 * shares one).
 *
 * The weights are static: `writeIdentity` puts them once, when the resident is built, into the row's
 * identity texels (`layout.ts`), which the vertex shader's morph loop reads after the expression's.
 * The eyeballs turn on bones, so what the face moves or sizes of them comes back as the bones' offsets
 * (`pose.ts` `skin`, `Placement.eyes`).
 */

import { type Appearance, faceFromId } from '../../../game/household';
import type { BodyData, MorphSet } from './data';
import { IDENTITY_PAIRS } from './layout';

/** The face of a look: its own, or one made from `id` (older saves, residents made without one). */
export function faceOf(appearance: Appearance | undefined, id: number): Record<string, number> {
  return appearance?.face ?? faceFromId(id);
}

/**
 * Writes `face` on `body` as (channel, weight) pairs at `out[o..]` (`IDENTITY_PAIRS * 2` floats; the weights
 * scaled as far as the body's life stage takes them, `BodyData.faceScale`), and the eyeballs' share into
 * `eyes` (4: the right eye's centre x, y, z in metres, the left's x mirrored, and how much larger they are, of 1).
 */
export function writeIdentity(morphs: MorphSet, body: BodyData, face: Record<string, number>, out: Float32Array, o: number, eyes: Float32Array): void {
  const first = morphs.expressions;
  const moved = morphs.eyes[body.family];
  eyes.fill(0);
  for (let k = 0; k < IDENTITY_PAIRS; k++) {
    const name = morphs.channels[first + k];
    const v = name ? (face[name.slice(3)] ?? 0) : 0;
    const w = Math.max(-1, Math.min(1, Number.isFinite(v) ? v : 0)) * body.faceScale;
    out[o + k * 2] = name ? first + k : 0;
    out[o + k * 2 + 1] = w;
    if (moved && w !== 0) for (let c = 0; c < 4; c++) eyes[c] += moved[k * 4 + c] * w;
  }
}
