/**
 * The dense pass: what makes a town as built up as its size wants, every building a real lot on a
 * road's frontage (`LotPlacer`). It took the place of the backdrop, whose buildings nobody lived or
 * worked in. Along every street it lays, by the kinds its zone draws (`KINDS`: the centre's shops,
 * cafés, flats and offices shoulder to shoulder; the rest of a main town's homes, terraces, flats and
 * workshops with gaps; a hamlet's homes and workshops further apart):
 *
 * - a home on a `tight` garden (no driveway, so no car of its own): a small generated house;
 * - a terrace row of two to four lots (`macro.ts` `rowHouse`, `LotPlacer.placeRow`);
 * - a flats building (`flats.ts` `flatsFor`; a terrace row stands in where flats are not made yet);
 * - a named place from the compact layouts (`places.ts` `ZONE`'s `compact`: a shop or a café 8 m
 *   wide, a restaurant, an office or a workshop of 12 to 14).
 *
 * The generator runs it after the homes, the named places, the parking and the orchards, from the
 * `dense` stream (`macro.ts`); Build and the build-up tool make one from a draft (`DenseDraft`):
 * every lot standing is kept, the new ones are added after them.
 */

import type { Content } from '../content/content';
import type { BuildingUse, TownPlace, TownSize } from '../core/protocol';
import type { Centre } from './centre';
import type { Pt, RoadPlan } from './roads';
import type { LotPlacer, NeighbourhoodDraft, PlotSlot, Rect, Templates } from './town';

type Rand = () => number;

/** Where a lot stands, which sets its kinds, gaps and the corner it keeps: a main town's centre, the rest of a main town, a hamlet. */
export type Zone = 'centre' | 'town' | 'hamlet';

/** What a pass over a town being generated is made from. */
export interface DenseInput {
  content: Content;
  t: Templates;
  W: number;
  D: number;
  roads: readonly RoadPlan[];
  /** The areas (0: the main town): name, box, middle, roads (indices into `roads`). */
  areas: readonly { name: string; rect: Rect; centre: Pt; roads: readonly number[] }[];
  /** The lots so far, and the ground they take: the pass adds its own to them. */
  lots: LotPlacer;
  centres: readonly Centre[];
  /** How much of a main town's land is built over, 0..1 (a dense town's most). */
  density: number;
  /** How big a town the main town is (its places' names: `Namer.size`). */
  civic: TownSize;
}

/** What a pass over a town laid already is made from (Build, the build-up tool): its draft, the house templates its lots are measured by, the content its places come from. */
export interface DenseDraft {
  draft: NeighbourhoodDraft;
  t: Templates;
  content: Content;
}

/**
 * Lots of a row along a road (`DensePass.rowsAlong`): their slots' indices, the road, how far along it
 * the row's middle is and on which side, how far back from the road its back is, its zone, and the
 * kinds the row was limited to (if it was).
 */
export interface Row {
  slots: number[];
  r: number;
  s: number;
  side: 1 | -1;
  back: number;
  zone: Zone;
  kinds?: readonly BuildingUse[];
}

/**
 * Where along its road a row goes (m, `from` to `to`; the whole road if not given) and what its lots may be
 * (`kinds`: any of its zone's if not given; `kindsAt`: those the land a metre along and 6 m back from the
 * sidewalk asks for, none there if null, in place of `kinds`, with no gaps but where it asks for none).
 */
export interface RowLimits {
  from?: number;
  to?: number;
  kinds?: readonly BuildingUse[];
  kindsAt?: (p: Pt) => readonly BuildingUse[] | null;
}

/** The dense pass (see the top of this file). */
export class DensePass {
  /** What else a lot must be to go down (by hand only: Build keeps each to the land painted its use); anything if unset. */
  allow?: (rect: Rect, use: BuildingUse) => boolean;

  constructor(input: DenseInput | DenseDraft) {
    void input;
  }

  /** How many lots there are so far (a draft's among them). */
  get count(): number {
    return 0;
  }

  /**
   * A row of lots along road `road` (of area `area`) on each of `sides`: the centre's shoulder to
   * shoulder, the rest with gaps, each of its zone's kinds (`zone`: that zone all along; else the
   * centre's where it is in one, the main town's in area 0, a hamlet's elsewhere). `limits`: see
   * `RowLimits`. The rows as laid.
   */
  rowsAlong(area: number, road: number, sides: readonly (1 | -1)[], rand: Rand, zone?: Zone, limits: RowLimits = {}): Row[] {
    void [area, road, sides, rand, zone, limits];
    return [];
  }

  /**
   * A lot of area `area` near each of `points` (set by hand), on the frontage of the road nearest it,
   * of the kinds the land there asks for (`kindsAt`: none if null). The slots laid, by index.
   */
  fillPoints(area: number, points: readonly Pt[], rand: Rand, kindsAt: (p: Pt) => readonly BuildingUse[] | null): number[] {
    void [area, points, rand, kindsAt];
    return [];
  }

  /** Every lot (a draft's first, then the new) and the places of the public ones. */
  result(): { slots: PlotSlot[]; places: TownPlace[] } {
    return { slots: [], places: [] };
  }
}
