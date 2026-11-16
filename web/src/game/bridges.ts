/**
 * Bridges over the water a world's land has (its wet mask: 1 m tiles, row-major over the world):
 * where a line crosses the water, whether it may be bridged there, and the road split so the
 * crossing is a straight `country` road with one deck span (`RoadPlan.bridge`), its deck kept dry
 * for walkers (`macro.ts` `deckOf`). The street tool uses these, and later the game's own bridges.
 * Only their shape is here so far: each throws until it is built.
 */

import type { RoadPlan } from '../core/protocol';
import type { Rect } from './town';
import type { Pt } from './roads';

/** Where along `points` (arc lengths, m) it runs over the water of `wet` (`W` × `D` tiles), each span from where it goes in to where it comes out. */
export function wetSpans(_points: readonly Pt[], _wet: Uint8Array, _W: number, _D: number): [number, number][] {
  throw new Error('wetSpans: not built yet');
}

/** Why the stretch `span` (arc lengths, m) of `points` may not be bridged (too long, a junction or a lot on it, among `roads` and `plots`); null: it may be. */
export function bridgeable(_points: readonly Pt[], _span: readonly [number, number], _roads: readonly RoadPlan[], _plots: readonly Rect[]): string | null {
  throw new Error('bridgeable: not built yet');
}

/**
 * `roads` with road `road` bridged over `span` (arc lengths, m): split into the road before, a
 * straight bridge with its deck, and the road after; and the walkers' mask with the deck dry.
 */
export function withBridge(_roads: readonly RoadPlan[], _road: number, _span: readonly [number, number], _wet: Uint8Array, _W: number, _D: number): { roads: RoadPlan[]; walk: Uint8Array } {
  throw new Error('withBridge: not built yet');
}
