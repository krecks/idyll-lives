/**
 * A resident's row in the pose texture: what the CPU writes (`characters.ts`) and the character
 * shader reads (`characters/material.ts`), one row per resident.
 *
 * Row layout (RGBA32F texels): `BONE_TEXELS` a bone (its skinning matrix's first three rows; the fourth is
 * 0 0 0 1), then `APPEARANCE_TEXELS` texels:
 *   0: skin colour rgb (linear), top: sleeve cut
 *   1: top colour rgb, top: hem height
 *   2: bottom colour rgb, top: crew neckline cut
 *   3: hair colour rgb, top: scoop neckline cut
 *   4: r = 1 if the top is painted over the bottom (1 + n + y / 4: shirt n shows in an open
 *      jacket's V, `SHIRT`, below its neck line y m; n 4: a white shirt over the whole top, under an
 *      apron), 0 if not, -1 if not and plain (no bands: the
 *      towel, swimwear); g = age (deeper creases: elders 1), b = how far the shells take their
 *      seated shape (characters.ts `writeSit`), bottom: waist height
 *   5: skin reference colour rgb (the skin is tinted relative to it; 1), bottom: leg cut
 *   6: shoe colour rgb, shoes: top height
 *   7: eye colour rgb, 1 if the Sim wears hair (the scalp takes the hair colour)
 *   8: r, g = how loose the top and the bottom are (0..1: how far the cloth stands off the skin it is painted on,
 *      the character shader's `EASE`; 0 for what clings or lies under a shell), b = 1 if the bottom has a shell
 *      round the hips (a skirt: the top lies flat under it)
 * Disabled cuts are -10 (or +10 for "below" cuts) so their predicate always / never holds.
 * Then `MORPH_TEXELS` texels of (channel, weight, channel, weight); weight 0 = unused: first the
 * `EXPRESSION_TEXELS` of the face's expression, written every frame (`face.ts` `FaceMixer`; a first
 * channel of -1: no morphs at all on this face this frame), then the `IDENTITY_TEXELS` of the resident's
 * own face (`identity.ts`), written once when the resident is built. Their weights are signed.
 */

/**
 * Texels a bone: its skinning matrix's rows 0, 1 and 2 (an affine matrix: the fourth is 0 0 0 1), so the
 * shader reads three a bone, not four (`pose.ts` `skin`, `shaders/character.ts` `gameSimSkin`).
 */
export const BONE_TEXELS = 3;
/** Floats a bone in a row (`BONE_TEXELS` RGBA texels). */
export const BONE_FLOATS = BONE_TEXELS * 4;

/** Appearance texels per resident, after its bone matrices. */
export const APPEARANCE_TEXELS = 9;

/**
 * The shirts of texel 4 r (`n`; their colours: the character shader's `SHIRTS`): what shows in an open jacket's V, white,
 * charcoal or grey; `apron`, a white shirt over the whole top (the apron shell alone in the top's colour).
 */
export const SHIRT = { white: 1, charcoal: 2, grey: 3, apron: 4 } as const;
/** Face morph weights per Sim: (channel, weight) pairs, two per texel; the expression's, written every frame. */
export const EXPRESSION_TEXELS = 12;
export const MORPH_PAIRS = EXPRESSION_TEXELS * 2;
/** The resident's own face after them (the identity channels, one pair each): written once. */
export const IDENTITY_TEXELS = 8;
export const IDENTITY_PAIRS = IDENTITY_TEXELS * 2;
/** Every pair texel the shader reads. */
export const MORPH_TEXELS = EXPRESSION_TEXELS + IDENTITY_TEXELS;
