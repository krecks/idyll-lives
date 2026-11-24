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
  /** The steepest it climbs (rise per m); a trail's is a measure only (it lies on the land). */
  grade: number;
  /**
   * Whether it is the motorway's: a carriageway or a ramp, apart from every other road (`isSeparated`). Cars go only
   * one way along it, whatever `RoadPlan.oneway` says (`onewayOf`); nobody walks along it; it joins only its own kind
   * (a carriageway its ramps); the generator lays it, the Street tool never draws it.
   */
  separated: boolean;
  /** How far (m) from its middle it keeps its land to itself, nothing else laid there (`roads.ts` `keepOf`): a carriageway's embankment and verge, a ramp's own; 0, its edge enough. */
  keep: number;
  /** What it is called on its own ("Country road"), and in a sentence ("a country road"). */
  label: string;
  article: string;
}

export const ROAD_KIND: Record<RoadKind, RoadKindInfo> = {
  street: { width: 7, sidewalk: 1.35, median: 0, lanesEachWay: 1, grade: 0.08, separated: false, keep: 0, label: 'Street', article: 'a street' },
  lane: { width: 6, sidewalk: 1.35, median: 0, lanesEachWay: 1, grade: 0.08, separated: false, keep: 0, label: 'Lane', article: 'a lane' },
  country: { width: 7, sidewalk: 0, median: 0, lanesEachWay: 1, grade: 0.12, separated: false, keep: 0, label: 'Country road', article: 'a country road' },
  trail: { width: 2, sidewalk: 0, median: 0, lanesEachWay: 0, grade: 0.22, separated: false, keep: 0, label: 'Footpath', article: 'a footpath' },
  avenue: { width: 14, sidewalk: 1.35, median: 2.6, lanesEachWay: 2, grade: 0.08, separated: false, keep: 0, label: 'Avenue', article: 'an avenue' },
  // Two 3.5 m lanes and a 2 m shoulder.
  motorway: { width: 9, sidewalk: 0, median: 0, lanesEachWay: 2, grade: 0.05, separated: true, keep: 25, label: 'Motorway', article: 'a motorway' },
  ramp: { width: 5, sidewalk: 0, median: 0, lanesEachWay: 1, grade: 0.08, separated: true, keep: 10, label: 'Slip road', article: 'a slip road' },
};

/** Every kind, in the table's order. */
export const ROAD_KINDS = Object.keys(ROAD_KIND) as RoadKind[];

/** A road's kind (`street` when it says none). */
export const kindOf = (r: Pick<RoadPlan, 'kind'>): RoadKind => r.kind ?? 'street';

/** What a road's kind is (`ROAD_KIND`). */
export const kindInfo = (r: Pick<RoadPlan, 'kind'>): RoadKindInfo => ROAD_KIND[kindOf(r)];

/** Whether a kind is the motorway's, a carriageway or a ramp (`RoadKindInfo.separated`); no kind: a street, not. */
export const isSeparated = (kind: RoadKind | undefined): boolean => ROAD_KIND[kind ?? 'street'].separated;

/**
 * Which way cars go along a road: +1 the way its points run, -1 against them, 0 both ways. A kind that is one-way
 * (`RoadKindInfo.separated`: a motorway's carriageway, a ramp) without `RoadPlan.oneway` goes the way its points run.
 */
export const onewayOf = (r: Pick<RoadPlan, 'kind' | 'oneway'>): 1 | -1 | 0 => r.oneway ?? (kindInfo(r).separated ? 1 : 0);
