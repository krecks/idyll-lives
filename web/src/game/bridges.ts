/**
 * Bridges over the water a world's land has (its wet mask: 1 m tiles, row-major over the world):
 * where a line crosses the water, whether it may be bridged there, and the road split so the
 * crossing is a straight `country` road with one deck span (`RoadPlan.bridge`), its deck kept dry
 * for walkers (as the generator's `macro.ts` `deckOf` keeps it). The street tool uses these
 * (`edit/streets.ts` `drawStreet`), and later the game's own bridges.
 *
 * A bridge is laid as the generator lays one: straight from `BRIDGE_IN` m before the bank to as far
 * past the other, its deck the water and `DECK_PAST` m either side, at most `BRIDGE_SPAN` m long.
 */

import type { RoadPlan } from '../core/protocol';
import { deckTiles } from './macro';
import { connector, lengths, pointAt, type Pt } from './roads';
import type { Rect } from './town';

/** The longest a bridge's deck may be (m), as the generator's (`macro.ts` `BRIDGE_SPAN`, copied: it is not exported). */
export const BRIDGE_SPAN = 45;
/** A bridge runs straight from this far before the bank to as far past the other (m: `macro.ts` `BRIDGE_IN`). */
export const BRIDGE_IN = 12;
/** A deck reaches this far past the water at each end (m: `macro.ts` `DECK_PAST`). */
export const DECK_PAST = 2;
/** A bridge's width (m): a country road's, as every road in a big world is (`macro.ts` `ROAD_WIDTH`). */
const BRIDGE_WIDTH = 7;
/** A line bent further than this off the straight between a bridge's ends (m) is not straightened into one. */
const BEND_MOST = 4;
/** Wet stretches closer than this (m) are one crossing (a sandbank in the middle of a river). */
const SAME_WATER = 2;
/** A stretch of road shorter than this (m) at either end of a bridge is taken into it. */
const SHORTEST_PIECE = 6;
/** How far before a bridge (m) the road turns to run straight onto it. */
const LEAD = 16;
/** Steps along a line where the water is looked for (m). */
const STEP = 0.25;

const round2 = (v: number) => Math.round(v * 100) / 100;
const unit = (x: number, z: number): Pt => {
  const l = Math.hypot(x, z) || 1;
  return [x / l, z / l];
};

/** Whether the tile under (x, z) is wet; off the world is dry. */
function wetAt(wet: Uint8Array, W: number, D: number, x: number, z: number): boolean {
  if (x < 0 || z < 0 || x >= W || z >= D) return false;
  return wet[Math.floor(z) * W + Math.floor(x)] === 1;
}

/** Where along `points` (arc lengths, m) it runs over the water of `wet` (`W` × `D` tiles), each span from where it goes in to where it comes out. */
export function wetSpans(points: readonly Pt[], wet: Uint8Array, W: number, D: number): [number, number][] {
  const out: [number, number][] = [];
  let [from, last] = [-1, -1];
  let s0 = 0;
  for (let i = 0; i + 1 < points.length; i++) {
    const [a, b] = [points[i], points[i + 1]];
    const l = Math.hypot(b[0] - a[0], b[1] - a[1]);
    const n = Math.max(1, Math.ceil(l / STEP));
    for (let k = i === 0 ? 0 : 1; k <= n; k++) {
      const f = k / n;
      const s = s0 + l * f;
      if (!wetAt(wet, W, D, a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f)) continue;
      if (from >= 0 && s - last > SAME_WATER) {
        out.push([round2(from), round2(last)]);
        from = -1;
      }
      if (from < 0) from = s;
      last = s;
    }
    s0 += l;
  }
  if (from >= 0) out.push([round2(from), round2(last)]);
  return out;
}

/** The stretch of a line `L` m long a bridge over `span` would straighten (arc lengths): `BRIDGE_IN` m before and after the water, within the line; an end within `SHORTEST_PIECE` m of the line's own end goes to it. */
export function bridgeStretch(L: number, span: readonly [number, number]): [number, number] {
  let [a, b] = [Math.max(0, span[0] - BRIDGE_IN), Math.min(L, span[1] + BRIDGE_IN)];
  if (a < SHORTEST_PIECE) a = 0;
  if (L - b < SHORTEST_PIECE) b = L;
  return [a, b];
}

/** The points of `pts` between arc lengths `a` and `b`, with the points at `a` and `b` themselves. */
function between(pts: readonly Pt[], a: number, b: number, ls = lengths(pts)): Pt[] {
  const out: Pt[] = [pointAt(pts, a, ls).p];
  for (let i = 0; i < pts.length; i++) if (ls[i] > a + 0.01 && ls[i] < b - 0.01) out.push(pts[i]);
  out.push(pointAt(pts, b, ls).p);
  return out;
}

/** How far a point is from a rectangle (0 inside it). */
const offRect = (r: Rect, x: number, z: number) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));

/** How far `p` is from the segment `a`–`b`, and how far along it (0..1) the nearest point is. */
function toSegment(p: Pt, a: Pt, b: Pt): { dist: number; f: number } {
  const [dx, dz] = [b[0] - a[0], b[1] - a[1]];
  const l2 = dx * dx + dz * dz || 1;
  const f = Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dz) / l2));
  return { dist: Math.hypot(p[0] - a[0] - dx * f, p[1] - a[1] - dz * f), f };
}

/**
 * Why the stretch `span` (arc lengths, m) of `points` may not be bridged (too long, a junction or a
 * lot on it, among `roads` and `plots`); null: it may be. Too long: the water and `DECK_PAST` m
 * either side over `BRIDGE_SPAN`. A bend: the line further than `BEND_MOST` m off straight over the
 * stretch a bridge would straighten (`BRIDGE_IN` m before the water to as far after). A junction:
 * another road coming onto that stretch, not where the line itself begins or ends (a road whose
 * points are `points` itself is not another). A lot: one within a metre of a bridge's width there.
 */
export function bridgeable(points: readonly Pt[], span: readonly [number, number], roads: readonly RoadPlan[], plots: readonly Rect[]): string | null {
  const ls = lengths(points);
  const L = ls[ls.length - 1];
  const across = span[1] - span[0];
  if (across + 2 * DECK_PAST > BRIDGE_SPAN) return `the water is ${Math.round(across)} m across here, and a bridge spans ${BRIDGE_SPAN - 2 * DECK_PAST} m of water at most`;
  const [a, b] = bridgeStretch(L, span);
  const stretch = between(points, a, b, ls);
  const [pa, pb] = [stretch[0], stretch[stretch.length - 1]];
  const off = Math.max(...stretch.map((p) => toSegment(p, pa, pb).dist));
  if (off > BEND_MOST) return `it bends where it crosses the water (${Math.round(off)} m off straight), and a bridge runs straight`;
  const half = BRIDGE_WIDTH / 2;
  const xs = stretch.map((p) => p[0]);
  const zs = stretch.map((p) => p[1]);
  const box = [Math.min(...xs), Math.min(...zs), Math.max(...xs), Math.max(...zs)];
  const sl = lengths(stretch);
  for (const r of roads) {
    if (r.points === points) continue;
    const reach = r.width / 2 + half + 1;
    for (const q of r.points) {
      if (q[0] < box[0] - reach || q[0] > box[2] + reach || q[1] < box[1] - reach || q[1] > box[3] + reach) continue;
      // The nearest point of the stretch, and how far along the line it is.
      let [best, s] = [Infinity, 0];
      for (let i = 0; i + 1 < stretch.length; i++) {
        const { dist, f } = toSegment(q, stretch[i], stretch[i + 1]);
        if (dist < best) [best, s] = [dist, a + sl[i] + (sl[i + 1] - sl[i]) * f];
      }
      if (best >= reach) continue;
      // (Where the line begins or ends on a road: that is its own join, not one on the bridge.)
      if ((a === 0 && s < reach) || (b === L && s > L - reach)) continue;
      return `${r.name ?? 'a road'} meets it where it would be a bridge`;
    }
  }
  for (const p of plots) {
    if (p.x > box[2] + half + 1 || p.x + p.w < box[0] - half - 1 || p.z > box[3] + half + 1 || p.z + p.d < box[1] - half - 1) continue;
    for (let i = 0; i + 1 < stretch.length; i++) {
      const [u, v] = [stretch[i], stretch[i + 1]];
      const n = Math.max(1, Math.ceil(Math.hypot(v[0] - u[0], v[1] - u[1])));
      for (let k = 0; k <= n; k++) if (offRect(p, u[0] + ((v[0] - u[0]) * k) / n, u[1] + ((v[1] - u[1]) * k) / n) < half + 1) return 'a lot stands where the bridge would be';
    }
  }
  return null;
}

/**
 * A bridge's or a causeway's deck over `water` (as the generator's `macro.ts` `deckOf`, copied: it
 * is not exported): its arc lengths from the first point over the water less `DECK_PAST` to the
 * last plus as much, the deck's tiles cleared in `walk` (the water the sim's walkers keep off).
 */
function deckOf(road: RoadPlan, water: Uint8Array, walk: Uint8Array, W: number, D: number): [number, number] {
  const ls = lengths(road.points);
  const L = ls[ls.length - 1];
  const wet = ([x, z]: Pt) => water[Math.min(D - 1, Math.max(0, Math.floor(z))) * W + Math.min(W - 1, Math.max(0, Math.floor(x)))] === 1;
  const first = road.points.findIndex(wet);
  if (first < 0) return [0, 0];
  let last = first;
  for (let i = first; i < road.points.length; i++) if (wet(road.points[i])) last = i;
  const span: [number, number] = [round2(Math.max(0, ls[first] - DECK_PAST)), round2(Math.min(L, ls[last] + DECK_PAST))];
  deckTiles(road, span, W, D, (tile) => (walk[tile] = 0));
  return span;
}

/** A polyline with points every metre or so from `a` to `b`. */
function straight(a: Pt, b: Pt): Pt[] {
  const n = Math.max(1, Math.ceil(Math.hypot(b[0] - a[0], b[1] - a[1])));
  return Array.from({ length: n + 1 }, (_, k): Pt => [a[0] + ((b[0] - a[0]) * k) / n, a[1] + ((b[1] - a[1]) * k) / n]);
}

const rounded = (pts: readonly Pt[]): Pt[] => pts.map(([x, z]): Pt => [round2(x), round2(z)]);

/**
 * `roads` with road `road` bridged over `span` (arc lengths, m, where it is over the water): the
 * stretch from `BRIDGE_IN` m before the water to as far after made straight (the road turning onto
 * it over the `LEAD` m before and off it over as many after), and the road split into the road
 * before (at index `road`), a straight `country` bridge with its deck, and the road after (those
 * two at the end of the list); and the walkers' mask, `wet` with the deck dry. A piece shorter than
 * `SHORTEST_PIECE` m is taken into the bridge, which then ends as the road did there. The road
 * after keeps the road's name unless `names` gives it one; the bridge is called `names.bridge`, else
 * after the road. Whether it may be bridged there is `bridgeable`'s to say.
 */
export function withBridge(roads: readonly RoadPlan[], road: number, span: readonly [number, number], wet: Uint8Array, W: number, D: number, names?: { bridge?: string; after?: string }): { roads: RoadPlan[]; walk: Uint8Array } {
  const r = roads[road];
  const ls = lengths(r.points);
  const L = ls[ls.length - 1];
  const [a, b] = bridgeStretch(L, span);
  const [pa, pb] = [pointAt(r.points, a, ls).p, pointAt(r.points, b, ls).p];
  const u = unit(pb[0] - pa[0], pb[1] - pa[1]);
  const { bridge: _deck, ...plain } = r;
  let before: RoadPlan | null = null;
  let after: RoadPlan | null = null;
  if (a > 0) {
    // Onto the bridge: the road as it was up to `LEAD` m before it, then turning to run straight onto it.
    const lead = Math.min(LEAD, a * 0.5);
    const from = pointAt(r.points, a - lead, ls);
    before = { ...plain, points: rounded([...between(r.points, 0, a - lead, ls).slice(0, -1), ...connector(from.p, from.t, pa, u)]), ends: [r.ends[0], 'join'] };
  }
  if (b < L) {
    const lead = Math.min(LEAD, (L - b) * 0.5);
    const to = pointAt(r.points, b + lead, ls);
    after = { ...plain, points: rounded([...connector(pb, u, to.p, to.t), ...between(r.points, b + lead, L, ls).slice(1)]), ends: ['join', r.ends[1]], ...(names?.after ? { name: names.after } : {}) };
  }
  const bridge: RoadPlan = {
    points: rounded(straight(pa, pb)),
    width: Math.max(BRIDGE_WIDTH, r.width),
    ends: [before ? 'join' : r.ends[0], after ? 'join' : r.ends[1]],
    kind: 'country',
    name: names?.bridge ?? `${(r.name ?? 'The').split(' ')[0]} Bridge`,
  };
  const walk = wet.slice();
  bridge.bridge = deckOf(bridge, wet, walk, W, D);
  const rest = before ? [bridge] : [];
  if (after) rest.push(after);
  return { roads: [...roads.map((x, i) => (i === road ? (before ?? bridge) : x)), ...rest], walk };
}

/** `walk` (the walkers' water) with the tiles under road `road`'s deck wet again where `water` (the water as it lies) is: what taking a bridge away gives back. */
export function withoutDeck(road: RoadPlan, walk: Uint8Array, water: (x: number, z: number) => boolean, W: number, D: number): Uint8Array {
  if (!road.bridge) return walk;
  const out = walk.slice();
  deckTiles(road, road.bridge, W, D, (tile) => {
    if (water((tile % W) + 0.5, Math.floor(tile / W) + 0.5)) out[tile] = 1;
  });
  return out;
}
