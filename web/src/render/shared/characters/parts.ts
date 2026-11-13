/**
 * A resident's part meshes (`residents.ts`): which parts every resident has and which their clothes show, the
 * shells' categories, the surface each part is drawn with, and which parts are drawn by size on screen (the
 * low-detail body far away, the small face parts and the hair's wisps close up).
 */

import type { PartColors } from '../../../core/protocol';
import { CLOSE_UP, type BodyData } from './data';
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

/** The part of a hair part's wisps (`hair.long` → `hair.long.wisps`: its soft edges, drawn only close up). */
export const wispsOf = (hair: string): string => `${hair}.wisps`;
/** Whether a part is a hair part's wisps. */
export const isWisps = (part: string): boolean => part.startsWith('hair.') && part.endsWith('.wisps');

/** The surface a part is drawn with (the engine's resident material, `pc/characterMaterial.ts`). */
export type CharacterSurface = 'body' | 'cloth' | 'cut' | 'hair' | 'wisp' | 'brow' | 'lash' | 'mouth' | 'eye';
/**
 * The shells the shader cuts to the resident's own garment (`cut`: cloth, with the cut as its opacity): the trousers,
 * one mesh for every bottom with legs (`outfit.ts` `LEGGED`), and the long top, one mesh for every regular top
 * (`REGULAR`). They have no `lod.` copy: far off the low-detail body is painted.
 */
export const CUT_SHELLS: ReadonlySet<string> = new Set(['bottom.trousers', 'top.long']);

/** The surface part `part` is drawn with. */
export function surfaceOf(part: string): CharacterSurface {
  return part === 'body' || part === 'lod.body' || part === CLOSE_UP || part.startsWith('body.')
    ? 'body'
    : part === 'eyes'
      ? 'eye'
      : part === 'brows'
        ? 'brow'
        : part === 'lashes'
          ? 'lash'
          : part === 'teeth' || part === 'tongue'
            ? 'mouth'
            : CUT_SHELLS.has(part)
              ? 'cut'
              : /^(lod\.)?(top|bottom|shoes)\./.test(part)
                ? 'cloth'
                : isWisps(part)
                  ? 'wisp'
                  : 'hair';
}
/**
 * Sims switch to the low-detail body and clothes below this projected size (px per 5 cm of face, at
 * `LOD_HEIGHT_PX` tall: `faceDetail`) and back above the second: the gap keeps a Sim at the threshold
 * from flickering between them.
 *
 * The thresholds are set by where the game's camera looks from (`shared/lens.ts`): at its default 24 m
 * a face is about 3.1 px per 5 cm, so a 6 px threshold kept the *whole* default view on `lod.body` (the
 * cage at its limit, 1.3k triangles) while the hair on the same head drew at 2.8k. 2.4 px brings the
 * full body back from 31 m in, and 1.8 px leaves it from 41 m out: the default view (and the close one,
 * 6 m) clear of the gap, so everyone there draws the same detail whichever way they came to it, and
 * the low-detail body to the far town and the shadows.
 *
 * Measured at a fixed height (not the drawing's: a window's size, the device's pixel ratio and the
 * resolution setting all change that), so the same camera shows the same detail on every screen.
 */
export const LOD_FAR_PX = 1.8;
export const LOD_NEAR_PX = 2.4;
/** The drawing's height (px) `LOD_FAR_PX` and `LOD_NEAR_PX` are measured at. */
export const LOD_HEIGHT_PX = 1080;
/**
 * Far enough off to be posed in full only every few frames (`residents.ts` `FAR_EVERY`), the fingers left
 * in the clips' fists, below this projected size (px per 5 cm of face, as drawn) and back above the second.
 * Apart from the mesh's (`LOD_FAR_PX`): at a few pixels a pose a frame or two old and a fist don't show on
 * the full body any more than on the low-detail one, and posing every resident in the default view every
 * frame would cost the slowest machines (an 8 GB M1) what nobody sees.
 */
export const POSE_FAR_PX = 6;
export const POSE_NEAR_PX = 7;
/**
 * Small face parts are skipped on faces too small on screen to show them: the projected size (px)
 * of a 5 cm feature (a mouth, an eye with its brow) below which each isn't drawn.
 */
export const DETAIL_PX: Record<string, number> = { teeth: 8, tongue: 8, lashes: 3, brows: 2 };
/**
 * The hair's wisps (same measure): not drawn below the first, and thinned out by their shader from the
 * second down to it (`pc/shaders/character.ts` `RESIDENT_WISP_OPACITY`), so they come and go without a
 * step. At the game's default 24 m a face is about 3.1 px per 5 cm (no wisps, no draw); zoomed all the
 * way in, 6 m, about 12 (all of them).
 */
export const WISPS_PX = [4, 7] as const;
/** The projected size (px per 5 cm of face) below which part `part` isn't drawn (0: always drawn). */
export const detailOf = (part: string): number => DETAIL_PX[part] ?? (isWisps(part) ? WISPS_PX[0] : 0);
/** Below this (same measure), a face doesn't run its morphs at all (expressions can't be seen). */
export const MORPH_PX = 3;
