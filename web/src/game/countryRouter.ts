/**
 * The country roads' router, to be moved out of `macro.ts` `generateFromSpec` (its `route`,
 * `countryRoad` and `alongExisting`, and the cost functions they read) so a village placed by hand
 * is joined to the roads the way a generated one is. Only its shape is here so far: every method
 * throws until the move is made, byte for byte (the classic worlds' hashes hold through it).
 */

import type { RoadPlan, TownArea } from '../core/protocol';
import type { Way } from './macro';
import type { Pt } from './roads';
import type { WorldSpec } from './world';

/** The 8 m grid the router reads, as the generator keeps it: per cell, its height (m), whether it is water, whether it is wooded. */
export interface Cells {
  readonly gw: number;
  readonly gd: number;
  readonly height: Float32Array;
  readonly water: Uint8Array;
  readonly woods: Uint8Array;
  cell(x: number, z: number): number;
  centre(i: number): Pt;
}

/** A way found over the grid: its cells, which of the ways it was asked from it leaves by, the heading it ends on, and whether it was steered (else found by A*). */
export interface Route {
  cells: number[];
  from: number;
  h?: number;
  steered: boolean;
}

/** A stretch of a country road's course: its ends, its cells, and whether it is the course's first or last (`shared`). */
export interface Piece {
  a: Way;
  cells: number[];
  b: Way | null;
  first: boolean;
  last: boolean;
}

/** What a cell the route might end on costs to end there (-1: it may not), heading `h`, more loosely if `loose`. */
export type Goal = (i: number, h?: number, loose?: boolean) => number;

export class CountryRouter {
  constructor(
    readonly cells: Cells,
    readonly areas: readonly TownArea[],
    readonly roads: readonly RoadPlan[],
    readonly spec: WorldSpec,
  ) {}

  /** The way a country road goes from any of `from` to a cell `goal` accepts, never heading back against `toward`; null if there is none. */
  route(_from: readonly Way[], _goal: Goal, _toward?: Pt): Route | null {
    throw new Error('CountryRouter.route: not built yet');
  }

  /** The country road from `a` along `cells` to `b` (or a dead end), its ends and its name. */
  road(_a: Way, _cells: readonly number[], _b: Way | null, _ends: RoadPlan['ends'], _name: string): RoadPlan {
    throw new Error('CountryRouter.road: not built yet');
  }

  /** The course from `a` along `cells` to `b` in pieces, sharing the roads it runs along (`alongExisting`). */
  shared(_a: Way, _cells: readonly number[], _b: Way | null): Piece[] {
    throw new Error('CountryRouter.shared: not built yet');
  }
}
