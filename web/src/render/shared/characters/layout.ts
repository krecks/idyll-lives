/**
 * A resident's row in the pose texture: what the CPU writes (`characters.ts`) and the character
 * shader reads (`characters/material.ts`), one row per resident.
 *
 * Row layout (RGBA32F texels): bones * 4 matrix texels, then `APPEARANCE_TEXELS` texels:
 *   0: skin colour rgb (linear), top: sleeve cut
 *   1: top colour rgb, top: hem height
 *   2: bottom colour rgb, top: crew neckline cut
 *   3: hair colour rgb, top: scoop neckline cut
 *   4: r = 1 if the top is painted over the bottom (1 + n + y / 4: shirt n shows in an open
 *      jacket's V, `SHIRTS`, below its neck line y m), 0 if not, -1 if not and plain (no bands: the
 *      towel, swimwear); g = age (deeper creases: elders 1), b = how far the shells take their
 *      seated shape (characters.ts `writeSit`), bottom: waist height
 *   5: skin reference colour rgb (the skin is tinted relative to it; 1), bottom: leg cut
 *   6: shoe colour rgb, shoes: top height
 *   7: eye colour rgb, 1 if the Sim wears hair (the scalp takes the hair colour)
 * Disabled cuts are -10 (or +10 for "below" cuts) so their predicate always / never holds.
 * Then `MORPH_TEXELS` texels of (channel, weight, channel, weight); weight 0 = unused.
 */

/** Appearance texels per resident, after its bone matrices. */
export const APPEARANCE_TEXELS = 8;
/** Face morph weights per Sim: (channel, weight) pairs, two per texel. */
export const MORPH_TEXELS = 12;
export const MORPH_PAIRS = MORPH_TEXELS * 2;
