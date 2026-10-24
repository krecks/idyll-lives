/**
 * A resident's part meshes (`residents.ts`): which parts every resident has and which their clothes show, the
 * shells' categories, the surface each part is drawn with, and which parts are drawn by size on screen (the
 * low-detail body far away, the small face parts close up).
 */

import type { PartColors } from '../../../core/protocol';
import type { BodyData } from './data';
import { SHELL_BITS } from './outfit';

/** Life stages without facial hair. */
export const YOUNG_STAGES = new Set(['baby', 'child', 'teen']);
/** Parts every Sim has (besides the body's skin, hair and clothes). */
export const FACE_PARTS: readonly string[] = ['eyes', 'brows', 'lashes', 'teeth', 'tongue'];
/** A part's shell category as a `SHELL_BITS` bit (`top.jacket`, `lod.bottom.skirt`, `shoes.boots` ...); 0 for anything else. */
export function shellBit(part: string): number {
  const kind = /^(?:lod\.)?(top|bottom|shoes)\./.exec(part)?.[1] as keyof typeof SHELL_BITS | undefined;
  return kind ? SHELL_BITS[kind] : 0;
}
/** The body's skin chunks a Sim wearing `parts` shows (those no worn garment hides completely). */
export function skinChunks(body: BodyData, parts: readonly string[]): string[] {
  return Object.entries(body.bodyChunks)
    .filter(([, hiders]) => !hiders.some((h) => parts.includes(h)))
    .map(([chunk]) => chunk);
}
/** A held part's name and its colours from the end of its prop key (`<part>@<colorsKey>`, see `PlacedObject.held`). */
export function partColors(ref: string): [string, PartColors?] {
  const at = ref.indexOf('@');
  if (at < 0) return [ref];
  return [ref.slice(0, at), Object.fromEntries(ref.slice(at + 1).split(',').map((p) => [p.slice(0, p.indexOf(':')), p.slice(p.indexOf(':') + 1)]))];
}
/** A part mesh (`ResidentView.createPart`, engine `M`) and the rows it draws. */
export interface PartMesh<M> {
  mesh: M;
  /** Enabled (`ResidentView.partEnabled`). */
  enabled: boolean;
  /** Instance settings (the Sims' rows, 4 numbers each) currently drawn, `count` of them. */
  settings: Float32Array;
  count: number;
  /** Every Sim wearing it; small face parts draw only those close enough (`detail`). */
  all: Float32Array;
  /** Projected size (px) of a 5 cm face feature below which this part isn't drawn (0: always). */
  detail: number;
  /** Drawn on Sims near the camera, far from it (the low-detail body and clothes), or always. */
  range: Range;
  /** A shell's category (`SHELL_BITS`; 0: not a shell), left out for the Sims whose clothes now don't draw it (`bare`). */
  shell: number;
}
export const Range = { Always: 0, Near: 1, Far: 2 } as const;
export type Range = (typeof Range)[keyof typeof Range];

/** The surface a part is drawn with (the engine's resident material, `pc/characterMaterial.ts`). */
export type CharacterSurface = 'body' | 'cloth' | 'hair' | 'brow' | 'lash' | 'mouth' | 'eye';

/** The surface part `part` is drawn with. */
export function surfaceOf(part: string): CharacterSurface {
  return part === 'body' || part === 'lod.body' || part.startsWith('body.')
    ? 'body'
    : part === 'eyes'
      ? 'eye'
      : part === 'brows'
        ? 'brow'
        : part === 'lashes'
          ? 'lash'
          : part === 'teeth' || part === 'tongue'
            ? 'mouth'
            : /^(lod\.)?(top|bottom|shoes)\./.test(part)
              ? 'cloth'
              : 'hair';
}
/**
 * Sims switch to the low-detail body and clothes below this projected size (px per 5 cm of face)
 * and back above the second: the gap keeps a Sim at the threshold from flickering between them.
 */
export const LOD_FAR_PX = 6;
export const LOD_NEAR_PX = 7;
/**
 * Small face parts are skipped on faces too small on screen to show them: the projected size (px)
 * of a 5 cm feature (a mouth, an eye with its brow) below which each isn't drawn.
 */
export const DETAIL_PX: Record<string, number> = { teeth: 8, tongue: 8, lashes: 3, brows: 2 };
/** Below this (same measure), a face doesn't run its morphs at all (expressions can't be seen). */
export const MORPH_PX = 3;
