/**
 * Bridges the land's water brushes make in the game (Build mode's Land tab): where a stroke's water now crosses a
 * road, the road is bridged there, along it as it runs (not made straight nor split, as the street tool's bridges
 * are: the simulation's roads stay as they are), if it may be (`bridges.ts` `bridgeable`: 45 m of deck at most,
 * straight, no junction or lot on it); else the stroke is refused with why. A road's new decks join its
 * `RoadPlan.bridges`; the command carries each changed road's whole list (`ShapeLandMeta.roads`), and the decks'
 * tiles stay dry in the water mask the simulation walks (`LandEditor.water`, once given the roads).
 */

import type { RoadPlan } from '../core/protocol';
import { bridgeable, DECK_PAST, wetSpans } from './bridges';
import { bridgeSpans, lengths, type Pt } from './roads';
import type { Rect } from './town';

/** A road whose bridges a stroke changed: its list before and after. */
export interface BridgeChange {
  road: number;
  before: [number, number][];
  after: [number, number][];
}

const round2 = (v: number) => Math.round(v * 100) / 100;

/**
 * The bridges the water in `wet` (1 m tiles, `W` × `D`, the walkers' mask: the decks there are already dry) asks of
 * `roads`: for each road it crosses where it has no deck, a deck over the water and `DECK_PAST` m either side; or
 * why one of them may not be bridged (thrown: `bridgesOrWhy`). Trails are left: a footpath through water is a ford.
 */
function bridgesFor(roads: readonly RoadPlan[], plots: readonly Rect[], wet: Uint8Array, W: number, D: number): { roads: RoadPlan[]; changes: BridgeChange[] } {
  const changes: BridgeChange[] = [];
  const out = roads.slice();
  roads.forEach((r, road) => {
    if (r.kind === 'trail' || r.points.length < 2) return;
    const decks = bridgeSpans(r);
    const fresh: [number, number][] = [];
    for (const span of wetSpans(r.points as Pt[], wet, W, D)) {
      if (decks.some(([a, b]) => span[0] >= a - 0.5 && span[1] <= b + 0.5)) continue;
      const why = bridgeable(r.points as Pt[], span, roads, plots);
      if (why) throw new Refused(`Water can't cross ${r.name ?? 'the road'} there: ${why}.`);
      const ls = lengths(r.points as Pt[]);
      fresh.push([round2(Math.max(0, span[0] - DECK_PAST)), round2(Math.min(ls[ls.length - 1], span[1] + DECK_PAST))]);
    }
    if (!fresh.length) return;
    const before = r.bridges ? r.bridges.map(([a, b]): [number, number] => [a, b]) : [];
    const after = [...before, ...fresh].sort((a, b) => a[0] - b[0]);
    changes.push({ road, before, after });
    out[road] = { ...r, bridges: after };
  });
  return { roads: out, changes };
}

/** A stroke's water may not cross a road there (`bridgesFor`). */
class Refused extends Error {}

/** `bridgesFor`, a refusal as its reason. */
export function bridgesOrWhy(roads: readonly RoadPlan[], plots: readonly Rect[], wet: Uint8Array, W: number, D: number): { roads: RoadPlan[]; changes: BridgeChange[] } | { refused: string } {
  try {
    return bridgesFor(roads, plots, wet, W, D);
  } catch (err) {
    if (err instanceof Refused) return { refused: err.message };
    throw err;
  }
}

/** `roads` with each changed road's bridges as `changes` had them (`side`): what undo and redo put back. */
export function withBridges(roads: readonly RoadPlan[], changes: readonly BridgeChange[], side: 'before' | 'after'): RoadPlan[] {
  const out = roads.slice();
  for (const c of changes) {
    const { bridges: _old, ...rest } = out[c.road];
    out[c.road] = c[side].length ? { ...rest, bridges: c[side] } : rest;
  }
  return out;
}

/** The command's `meta.roads` for `changes` (`side`): each changed road's whole list. */
export function bridgesMeta(changes: readonly BridgeChange[], side: 'before' | 'after'): Record<number, { bridges: [number, number][] }> {
  return Object.fromEntries(changes.map((c) => [c.road, { bridges: c[side] }]));
}
