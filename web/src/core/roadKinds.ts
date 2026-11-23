/**
 * What each kind of road is (`RoadKind`): one table, read by the generator, the land's grading, the junctions and
 * the Street tool alike.
 *
 * A motorway is two `motorway` roads, one carriageway each way about 9 m apart, each one-way; the barrier and the
 * verge between them are the renderer's alone. Its ramps are one-way `ramp` roads that `merge` with it.
 */

import type { RoadKind, RoadPlan } from './protocol';

export interface RoadKindInfo {
  /** Its width (m), kerb to kerb with the sidewalks (`RoadPlan.width`), the median in it too. */
  width: number;
  /** Each sidewalk's width with its kerb (m); 0: none (a country road's verge is the land's). */
  sidewalk: number;
  /** The median down its middle (m); 0: none (a motorway's carriageways are two roads, the verge between them not in either). */
  median: number;
  /** Its lanes each way (a one-way road's, its one way); 0: none for cars. */
  lanesEachWay: number;
  /** Whether cars go only one way along it, whatever `RoadPlan.oneway` says (a motorway's carriageway, a ramp). */
  oneway: boolean;
  /** The steepest it climbs (rise per m); a trail's is a measure only (it lies on the land). */
  grade: number;
  /** Whether people walk along it. */
  walkers: boolean;
  /** Whether its edges are painted with a line (and its lanes with dividers). */
  edgeLines: boolean;
  /** Whether the player draws it with the Street tool (a motorway and its ramps are the generator's only). */
  drawable: boolean;
}

export const ROAD_KIND: Record<RoadKind, RoadKindInfo> = {
  street: { width: 7, sidewalk: 1.35, median: 0, lanesEachWay: 1, oneway: false, grade: 0.08, walkers: true, edgeLines: false, drawable: true },
  lane: { width: 6, sidewalk: 1.35, median: 0, lanesEachWay: 1, oneway: false, grade: 0.08, walkers: true, edgeLines: false, drawable: true },
  country: { width: 7, sidewalk: 0, median: 0, lanesEachWay: 1, oneway: false, grade: 0.12, walkers: true, edgeLines: false, drawable: true },
  trail: { width: 2, sidewalk: 0, median: 0, lanesEachWay: 0, oneway: false, grade: 0.22, walkers: true, edgeLines: false, drawable: true },
  avenue: { width: 14, sidewalk: 1.35, median: 2.6, lanesEachWay: 2, oneway: false, grade: 0.08, walkers: true, edgeLines: true, drawable: true },
  // Two 3.5 m lanes and a 2 m shoulder.
  motorway: { width: 9, sidewalk: 0, median: 0, lanesEachWay: 2, oneway: true, grade: 0.05, walkers: false, edgeLines: true, drawable: false },
  ramp: { width: 5, sidewalk: 0, median: 0, lanesEachWay: 1, oneway: true, grade: 0.08, walkers: false, edgeLines: true, drawable: false },
};

/** Every kind, in the table's order. */
export const ROAD_KINDS = Object.keys(ROAD_KIND) as RoadKind[];

/** A road's kind (`street` when it says none). */
export const kindOf = (r: Pick<RoadPlan, 'kind'>): RoadKind => r.kind ?? 'street';

/** What a road's kind is (`ROAD_KIND`). */
export const kindInfo = (r: Pick<RoadPlan, 'kind'>): RoadKindInfo => ROAD_KIND[kindOf(r)];

/**
 * Which way cars go along a road: +1 the way its points run, -1 against them, 0 both ways. A kind that is one-way
 * (`RoadKindInfo.oneway`: a motorway's carriageway, a ramp) without `RoadPlan.oneway` goes the way its points run.
 */
export const onewayOf = (r: Pick<RoadPlan, 'kind' | 'oneway'>): 1 | -1 | 0 => r.oneway ?? (kindInfo(r).oneway ? 1 : 0);
