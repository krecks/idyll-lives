/**
 * The town's paved ground, one source for near and far: the centre's outlines and squares, the
 * courts and yards the town lays (`WorldMeta.paving`), the car parks' aprons and their bays' lines
 * (`WorldStructure.carParks`, `parking`), and the P signs. The far colour map
 * (`pc/groundTiles.ts`), the near ground (`streetGround.ts`) and what is no lawn (`nature.ts`
 * `unlawned`) all read it, so the centre is paved at every distance.
 *
 * The rule: a centre's outline is paved, but for its home and farm lots, its greens and its
 * water; a place's or a flats building's lot in it is paved through its `court` pieces. Nothing
 * here guesses what the town did not say.
 */

import type { PavedPiece, WorldStructure } from '../../core/protocol';

/** A rectangle on the ground (town metres, x and z its corner). */
export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** What `townPaving` reads: the lots (to leave homes out), the meta's centres and paving, the bays and car parks. */
export type PavingWorld = Pick<WorldStructure, 'plots' | 'meta' | 'parking' | 'carParks'>;

/**
 * A piece to pave: what it is (a car park's apron is `carpark`), its outline (counter-clockwise
 * seen from above), the lots cut out of it (`holes`), and how far apart its courses' joints are (m).
 */
export interface PavingDraw {
  kind: PavedPiece['kind'] | 'carpark';
  poly: [number, number][];
  holes: Rect[];
  course: number;
}

/** A parking bay as drawn: its corners (the bay's own rectangle) and its painted lines (each a segment). */
export interface BayDraw {
  corners: [number, number][];
  lines: [[number, number], [number, number]][];
}

/** The town's paving as drawn (`townPaving`): the pieces, the bays' lines, the P signs (x, z, yaw) and a mask of what is paved. */
export interface TownPaving {
  pieces: PavingDraw[];
  bays: BayDraw[];
  signs: [number, number, number][];
  /** One per tile over `w` × `d` (row-major from the world's corner): 1 where the ground is paved. */
  mask(w: number, d: number): Uint8Array;
}

/** Nothing paved yet: what `townPaving` gives until the town's paving is drawn from (the ground stream fills it in). */
const NONE: TownPaving = { pieces: [], bays: [], signs: [], mask: (w, d) => new Uint8Array(w * d) };

/** The town's paved ground (see the top of this file), the same object for the same meta. */
export function townPaving(world: PavingWorld): TownPaving {
  void world;
  return NONE;
}
