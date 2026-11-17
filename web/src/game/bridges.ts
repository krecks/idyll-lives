/**
 * Bridges over the water a world's land has (its wet mask: 1 m tiles, row-major over the world):
 * where a line crosses the water, whether it may be bridged there, and the road split so the
 * crossing is a straight `country` road with one deck span (`RoadPlan.bridge`), its deck kept dry
 * for walkers (as the generator's `macro.ts` `deckOf` keeps it). The street tool uses these
 * (`edit/streets.ts` `drawStreet`), and later the game's own bridges.
 *
 * A bridge is laid as the generator lays one: straight from `BRIDGE_IN` m before the bank to as far
 * past the other, its deck the water and `DECK_PAST` m either side, at most `BRIDGE_SPAN` m long.
 *
 * A street drawn over the water (`crossWater`) is let off more: its deck may be `DRAWN_SPAN` m long,
 * and where the line drawn won't make a bridge as it runs (it bends there, or only clips the water,
 * or the crossing is too long at its slant) the straight is laid through the water's middle, and
 * then turned up to 50° to cross squarer (`bridgeFits`). On a land with heights, its deck reaches
 * on over the dip a river lies in (`grownDeck`), so the road is not dug down into it nor built up,
 * and as far over the low ground on the water's other side, so it stays centred on the water.
 */

import type { RoadPlan } from '../core/protocol';
import { BRIDGE_CLEAR } from '../render/shared/builtSurface';
import { heightAt, LAND_CELL, NONE, type Land } from './land';
import { BRIDGE_IN, BRIDGE_SPAN, DECK_PAST, deckOf, deckTiles, ROAD_WIDTH } from './macro';
import { bends, connector, lengths, nearest, pointAt, type Pt } from './roads';

/** The generator's bridge limits (`macro.ts`): its longest deck, how far before the bank it runs straight, how far past the water its deck reaches. */
export { BRIDGE_IN, BRIDGE_SPAN, DECK_PAST };
import type { Rect } from './town';

/** The longest deck of a street the player draws over the water (m); the generator's are `BRIDGE_SPAN` at most. */
export const DRAWN_SPAN = 70;
/** A drawn bridge's straight is turned this much at most (degrees), in steps of `TURN_STEP`, to cross the water squarer. */
const TURN_MOST = 50;
const TURN_STEP = 10;
/** A drawn bridge's deck reaches on over dry land lying this far (m) under the line between the bridge's ends: the dip a river lies in. */
const DIP = 3;
/** A grown deck leaves this much of its bridge straight before it either end (m), as the generator's leaves; and reaches this far onto the bank past the dip (m). */
const APPROACH_KEEP = BRIDGE_IN - DECK_PAST;
const ONTO = 6;
/** A deck's stone reaches this far (m) under its road (`render/shared/street.ts` `BRIDGE.under`): a deck stands clear of land lying this far under its floor. */
const SLAB = 0.6;

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
export function bridgeable(points: readonly Pt[], span: readonly [number, number], roads: readonly RoadPlan[], plots: readonly Rect[], opts: { most?: number; bend?: number } = {}): string | null {
  const { most = BRIDGE_SPAN, bend = BEND_MOST } = opts;
  const ls = lengths(points);
  const L = ls[ls.length - 1];
  const across = span[1] - span[0];
  if (across + 2 * DECK_PAST > most) return `the water is ${Math.round(across)} m across here, and a bridge spans ${most - 2 * DECK_PAST} m of water at most`;
  const [a, b] = bridgeStretch(L, span);
  const stretch = between(points, a, b, ls);
  const [pa, pb] = [stretch[0], stretch[stretch.length - 1]];
  const off = Math.max(...stretch.map((p) => toSegment(p, pa, pb).dist));
  if (off > bend) return `it bends where it crosses the water (${Math.round(off)} m off straight), and a bridge runs straight`;
  return blockedOn(points, stretch, a, b, L, roads, plots);
}

/**
 * Why `stretch` (of the line `points`, from `a` to `b` m along it, `L` m long) may not be a bridge's: another road
 * meeting it (not where the line itself begins or ends), or a lot within a metre of a bridge's width of it; null: none.
 */
function blockedOn(points: readonly Pt[], stretch: readonly Pt[], a: number, b: number, L: number, roads: readonly RoadPlan[], plots: readonly Rect[]): string | null {
  const half = ROAD_WIDTH / 2;
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

/** A polyline with points every metre or so from `a` to `b`. */
function straight(a: Pt, b: Pt): Pt[] {
  const n = Math.max(1, Math.ceil(Math.hypot(b[0] - a[0], b[1] - a[1])));
  return Array.from({ length: n + 1 }, (_, k): Pt => [a[0] + ((b[0] - a[0]) * k) / n, a[1] + ((b[1] - a[1]) * k) / n]);
}

const rounded = (pts: readonly Pt[]): Pt[] => pts.map(([x, z]): Pt => [round2(x), round2(z)]);

/**
 * Where a bridge over a line's crossing is laid: the road leaves the line `a` m along it (0: the
 * bridge begins the line) turning over its last `lead[0]` m to run straight onto the bridge at `pa`,
 * and comes back onto it from `pb` over `lead[1]` m to `b` m along it (the line's length: the bridge
 * ends it). Its deck along the bridge from `pa` (arc lengths), if worked out (`withBridge` finds it
 * on the water else, as the generator does). The chord (`chordOf`) is the stretch of the line itself.
 */
export interface BridgeFit {
  a: number;
  b: number;
  pa: Pt;
  pb: Pt;
  lead: [number, number];
  deck?: [number, number];
  /** The straight is a chord of the line (between two of its points), as the generator's are. */
  chord?: boolean;
}

/** The fit a bridge over `span` of `points` has made straight where it is (as the generator's are): from `BRIDGE_IN` m before the water to as far after it, along the line (`bridgeStretch`). */
export function chordOf(points: readonly Pt[], span: readonly [number, number], ls = lengths(points)): BridgeFit {
  const [a, b] = bridgeStretch(ls[ls.length - 1], span);
  return { a, b, pa: pointAt(points, a, ls).p, pb: pointAt(points, b, ls).p, lead: [LEAD, LEAD], chord: true };
}

/**
 * The road `points` made with a bridge as `fit` lays it (see `BridgeFit`): the road before it (none if the bridge
 * begins it), the bridge's straight, the road after it (none if it ends it); and the turns onto it and off it (the
 * last points of the road before, the first of the road after: none where there is no such road).
 */
function piecesOf(points: readonly Pt[], fit: BridgeFit, ls = lengths(points)): { before: Pt[] | null; bridge: Pt[]; after: Pt[] | null; turns: [Pt[], Pt[]] } {
  const L = ls[ls.length - 1];
  const { a, b, pa, pb } = fit;
  const u = unit(pb[0] - pa[0], pb[1] - pa[1]);
  let [before, after]: (Pt[] | null)[] = [null, null];
  const turns: [Pt[], Pt[]] = [[], []];
  if (a > 0) {
    // Onto the bridge: the road as it was up to its lead before it, then turning to run straight onto it.
    const lead = Math.min(fit.lead[0], a * 0.5);
    const from = pointAt(points, a - lead, ls);
    turns[0] = rounded(connector(from.p, from.t, pa, u));
    before = [...rounded(between(points, 0, a - lead, ls).slice(0, -1)), ...turns[0]];
  }
  if (b < L) {
    const lead = Math.min(fit.lead[1], (L - b) * 0.5);
    const to = pointAt(points, b + lead, ls);
    turns[1] = rounded(connector(pb, u, to.p, to.t));
    after = [...turns[1], ...rounded(between(points, b + lead, L, ls).slice(1))];
  }
  return { before, bridge: rounded(straight(pa, pb)), after, turns };
}

/** Whether the 1 m tile under (x, z) is wet, as `macro.ts` `deckOf` reads it: clamped into the world. */
const deckWet = (wet: Uint8Array, W: number, D: number, [x, z]: Pt) => wet[Math.min(D - 1, Math.max(0, Math.floor(z))) * W + Math.min(W - 1, Math.max(0, Math.floor(x)))] === 1;

/** A bridge's deck along `points` (its straight) as `macro.ts` `deckOf` finds it over `wet`, without drying anything: from its first point over the water less `DECK_PAST` m to its last plus as much; [0, 0] over no water. */
export function deckSpanOf(points: readonly Pt[], wet: Uint8Array, W: number, D: number, ls = lengths(points)): [number, number] {
  const L = ls[ls.length - 1];
  const first = points.findIndex((p) => deckWet(wet, W, D, p));
  if (first < 0) return [0, 0];
  let last = first;
  for (let i = first; i < points.length; i++) if (deckWet(wet, W, D, points[i])) last = i;
  return [round2(Math.max(0, ls[first] - DECK_PAST)), round2(Math.min(L, ls[last] + DECK_PAST))];
}

/** The land's sample nearest (x, z). */
function sampleAt(land: Land, x: number, z: number): number {
  const i = Math.min(land.cols - 1, Math.max(0, Math.round(x / LAND_CELL)));
  const j = Math.min(land.rows - 1, Math.max(0, Math.round(z / LAND_CELL)));
  return j * land.cols + i;
}

/** The land's height at (x, z) (m), its water's level where it is wet. */
function groundAt(land: Land, x: number, z: number): number {
  const k = sampleAt(land, x, z);
  return land.wl[k] !== NONE && land.wl[k] > land.h[k] ? land.wl[k] / 100 : heightAt(land, x, z);
}

/**
 * `deck` (along the bridge's straight `points`) reaching on over the dip its water lies in: while the
 * land past its end lies more than `DIP` m under the line between the land at the bridge's ends, a
 * metre at a time, and then `ONTO` m onto the bank past the dip (the road's height is worked out from
 * the land round it: `builtSurface.ts`), no longer than `most` m and leaving `APPROACH_KEEP` m of
 * straight before it either end. Then, centred on its water: the end reaching less far past the water
 * carried on as far as the other does, over land lying under the deck (`SLAB` m under its floor, the
 * water's level and `BRIDGE_CLEAR`: a low bank, a meadow at the water's level), never over higher
 * ground (a bank's top, a footpath along it), where it would lie on the land. `short`: per end,
 * whether it would have reached on past that (the straight is too short for it).
 */
export function grownDeck(points: readonly Pt[], deck: readonly [number, number], land: Land, most: number, ls = lengths(points)): { deck: [number, number]; short: [boolean, boolean] } {
  const L = ls[ls.length - 1];
  const at = (s: number) => {
    const { p } = pointAt(points, s, ls);
    return groundAt(land, p[0], p[1]);
  };
  const [h0, h1] = [at(0), at(L)];
  const dip = (s: number) => at(s) < h0 + ((h1 - h0) * s) / L - DIP;
  const [lo, hi] = [APPROACH_KEEP, L - APPROACH_KEEP];
  let [s0, s1] = deck;
  const short: [boolean, boolean] = [false, false];
  for (const e of [0, 1] as const) {
    const step = e ? 1 : -1;
    const room = (s: number) => s >= lo && s <= hi && (e ? s - s0 : s1 - s) <= most;
    let end = e ? s1 : s0;
    let s = end + step;
    while (room(s) && dip(s)) [end, s] = [s, s + step];
    if (end === (e ? s1 : s0)) {
      short[e] = !room(s) && dip(s);
      continue;
    }
    // (Past the dip, onto the bank: as far as there is room.)
    let onto = end;
    for (let k = 1; k <= ONTO && room(end + step * k); k++) onto = end + step * k;
    short[e] = !room(s) || onto !== end + step * ONTO;
    if (e) s1 = onto;
    else s0 = onto;
  }
  // Centred on its water: the end reaching less far past it carried on over the low ground there.
  const past = [deck[0] - s0, s1 - deck[1]];
  const e = past[0] < past[1] ? 0 : 1;
  const want = Math.floor(Math.abs(past[1] - past[0]));
  if (want >= 1) {
    let level = -Infinity;
    for (let s = deck[0]; s <= deck[1]; s += 1) {
      const { p } = pointAt(points, s, ls);
      const k = sampleAt(land, p[0], p[1]);
      if (land.wl[k] !== NONE && land.wl[k] > land.h[k]) level = Math.max(level, land.wl[k] / 100);
    }
    const under = level + BRIDGE_CLEAR - SLAB;
    const step = e ? 1 : -1;
    let end = e ? s1 : s0;
    for (let k = 0; k < want; k++) {
      const s = end + step;
      if (!(at(s) < under)) break;
      if (s < lo || s > hi || (e ? s - s0 : s1 - s) > most) {
        short[e] = true;
        break;
      }
      end = s;
    }
    if (e) s1 = end;
    else s0 = end;
  }
  return { deck: [round2(s0), round2(s1)], short };
}

/** What `crossWater` lays a bridge with: the water (the walkers' mask: 1 m tiles, `W` × `D`), the roads and lots a bridge keeps off, the land (its dips: `grownDeck`), and the longest deck (m). */
export interface WaterWay {
  wet: Uint8Array;
  W: number;
  D: number;
  roads: readonly RoadPlan[];
  plots: readonly Rect[];
  land?: Land | null;
  most?: number;
}

/** `v` turned by `deg` degrees. */
const turned = (v: Pt, deg: number): Pt => {
  const [c, s] = [Math.cos((deg * Math.PI) / 180), Math.sin((deg * Math.PI) / 180)];
  return [v[0] * c - v[1] * s, v[0] * s + v[1] * c];
};

/**
 * The fits `crossWater` tries for a bridge over `span` of `points`, in order (see the top of this
 * file): the chord (`chordOf`, so what was bridged before is bridged the same; carried on along the
 * line over a dip, and then as it was); then straights
 * through the middle of the water the line crosses (where it crosses its middle), on the chord's
 * heading and turned from it by `TURN_STEP`° and more up to `TURN_MOST`° either way, each from
 * `BRIDGE_IN` m before the water to as far after it (a straight with no line left before or after
 * it, or whose middle is dry, is none). With `land`, each with its deck grown over the dip
 * (`grownDeck`), its straight carried on where the deck wants more of it.
 */
export function bridgeFits(points: readonly Pt[], span: readonly [number, number], wet: Uint8Array, W: number, D: number, land?: Land | null, most = DRAWN_SPAN): BridgeFit[] {
  const ls = lengths(points);
  const L = ls[ls.length - 1];
  const chord = chordOf(points, span, ls);
  const out: BridgeFit[] = [withDeck(chord, (f, more) => {
    // (An end within `SHORTEST_PIECE` m of the line's own goes to it, as `bridgeStretch` has it.)
    let [a, b] = [f.a > 0 ? Math.max(0, f.a - more[0]) : 0, f.b < L ? Math.min(L, f.b + more[1]) : L];
    if (a < SHORTEST_PIECE) a = 0;
    if (L - b < SHORTEST_PIECE) b = L;
    return { ...f, a, b, pa: pointAt(points, a, ls).p, pb: pointAt(points, b, ls).p };
  })];
  // (Carried on over a dip: the chord as it was next, should the longer one not do.)
  if (out[0].a !== chord.a || out[0].b !== chord.b) out.push(withDeck(chord, () => null));
  const mid = pointAt(points, (span[0] + span[1]) / 2, ls).p;
  if (!wetAt(wet, W, D, mid[0], mid[1])) return out;
  const u0 = unit(chord.pb[0] - chord.pa[0], chord.pb[1] - chord.pa[1]);
  const turns = [0];
  for (let d = TURN_STEP; d <= TURN_MOST; d += TURN_STEP) turns.push(d, -d);
  for (const turn of turns) {
    const u = turned(u0, turn);
    // The water along it either way from the middle, sandbanks no gap.
    const reach = (dir: 1 | -1) => {
      let [t, last] = [0, 0];
      while (t - last <= SAME_WATER && t <= most) {
        t += STEP;
        if (wetAt(wet, W, D, mid[0] + u[0] * t * dir, mid[1] + u[1] * t * dir)) last = t;
      }
      return last;
    };
    const [w0, w1] = [reach(-1), reach(1)];
    if (w0 + w1 + 2 * DECK_PAST > most) continue;
    const fitOf = (e0: number, e1: number): BridgeFit | null => {
      const pa: Pt = [mid[0] - u[0] * e0, mid[1] - u[1] * e0];
      const pb: Pt = [mid[0] + u[0] * e1, mid[1] + u[1] * e1];
      const [na, nb] = [nearest(points, pa[0], pa[1], ls), nearest(points, pb[0], pb[1], ls)];
      // (Its turns onto and off it longer the further it stands off the line.)
      const lead: [number, number] = [LEAD + 2 * na.dist, LEAD + 2 * nb.dist];
      if (na.s - lead[0] < 1 || nb.s + lead[1] > L - 1 || nb.s - na.s < 1) return null;
      return { a: na.s, b: nb.s, pa, pb, lead };
    };
    const first = fitOf(w0 + BRIDGE_IN, w1 + BRIDGE_IN);
    if (!first) continue;
    const fit = withDeck(first, (f, more) => {
      const e0 = Math.hypot(f.pa[0] - mid[0], f.pa[1] - mid[1]) + more[0];
      const e1 = Math.hypot(f.pb[0] - mid[0], f.pb[1] - mid[1]) + more[1];
      return fitOf(e0, e1);
    });
    if (!out.some((o) => near(o.pa, fit.pa) && near(o.pb, fit.pb))) out.push(fit);
  }
  return out;

  /** `fit` with its deck (grown over the land's dip, its straight carried on by `longer` where the deck wants more of it). */
  function withDeck(fit: BridgeFit, longer: (f: BridgeFit, more: [number, number]) => BridgeFit | null): BridgeFit {
    let f = fit;
    for (let round = 0; ; round++) {
      const pts = rounded(straight(f.pa, f.pb));
      const bl = lengths(pts);
      const deck = deckSpanOf(pts, wet, W, D, bl);
      if (!land || !(deck[1] > deck[0])) return { ...f, deck };
      const grown = grownDeck(pts, deck, land, most, bl);
      const more: [number, number] = [grown.short[0] ? GROW_ON : 0, grown.short[1] ? GROW_ON : 0];
      const next = round < GROW_ROUNDS && (more[0] || more[1]) ? longer(f, more) : null;
      if (!next || (near(next.pa, f.pa) && near(next.pb, f.pb))) return { ...f, deck: grown.deck };
      f = next;
    }
  }
}

/** A drawn bridge's straight is carried on this far (m) at a time where its deck would reach further over a dip, `GROW_ROUNDS` times at most. */
const GROW_ON = 8;
const GROW_ROUNDS = 4;
const near = (p: Pt, q: Pt) => Math.hypot(p[0] - q[0], p[1] - q[1]) < 0.05;

/** What `crossWater` lays: the fit chosen, and the road it makes (see `piecesOf`). */
export interface Crossed {
  fit: BridgeFit;
  before: Pt[] | null;
  bridge: Pt[];
  after: Pt[] | null;
}

/**
 * The bridge a drawn street (`points`, its `ends`) is laid over the crossing `span` with (see the top
 * of this file), or why it may not be: the first of `bridgeFits` whose water is no wider than
 * `way.most` (`DRAWN_SPAN`, unless given) allows, with no road meeting it nor a lot on it
 * (`bridgeable`; for a straight off the line, the new road from where it leaves the line to where it
 * comes back), a road on at each end, a deck over the water (else it only clips its edge) no longer
 * than `way.most`, and the road turning onto it and off it neither over the water nor tighter than
 * it may bend. None: why the chord may not be. The street tool's live checks and `drawStreet` both
 * lay a street's bridges with it, so they agree.
 */
export function crossWater(points: readonly Pt[], ends: RoadPlan['ends'], span: readonly [number, number], way: WaterWay): Crossed | string {
  const { wet, W, D, roads, plots, land = null, most = DRAWN_SPAN } = way;
  const ls = lengths(points);
  const L = ls[ls.length - 1];
  const [a0, b0] = bridgeStretch(L, span);
  let first: string | null = null;
  for (const fit of bridgeFits(points, span, wet, W, D, land, most)) {
    const pieces = piecesOf(points, fit, ls);
    const why = fitWhy(fit, !!fit.chord, pieces);
    if (!why) return { fit, before: pieces.before, bridge: pieces.bridge, after: pieces.after };
    first ??= why;
  }
  return first ?? 'it can not be bridged there';

  function fitWhy(fit: BridgeFit, chord: boolean, pieces: ReturnType<typeof piecesOf>): string | null {
    if (chord) {
      const no = bridgeable(points, span, roads, plots, { most });
      if (no) return no;
      // (Carried on over a dip: the rest of its stretch too.)
      if (fit.a < a0 || fit.b > b0) {
        const more = blockedOn(points, between(points, fit.a, fit.b, ls), fit.a, fit.b, L, roads, plots);
        if (more) return more;
      }
    } else {
      // (The new road from where it leaves the line to where it comes back onto it.)
      const [into, outOf] = pieces.turns;
      const stretch = [...into, ...pieces.bridge.slice(1, -1), ...outOf];
      const no = blockedOn(points, stretch, fit.a - fit.lead[0], fit.b + fit.lead[1], L, roads, plots);
      if (no) return no;
    }
    if ((fit.a === 0 && ends[0] !== 'join') || (fit.b === L && ends[1] !== 'join')) return 'a bridge needs a road on at each end: carry it on past the water';
    const deck = fit.deck!;
    if (!(deck[1] > deck[0])) return 'it only clips the edge of the water: draw it clear of the water, or on across it';
    if (deck[1] - deck[0] > most + 0.01) return `its bridge would be ${Math.round(deck[1] - deck[0])} m long (${most} m at most)`;
    for (const turn of pieces.turns) if (turn.length > 1 && wetSpans(turn, wet, W, D).length) return 'it runs over the water where it would not be a bridge';
    for (const piece of [pieces.before, pieces.after]) if (piece && bends(piece).tight) return 'it would turn onto its bridge tighter than 9 m';
    return null;
  }
}

/**
 * `roads` with road `road` bridged over `span` (arc lengths, m, where it is over the water): the
 * stretch from `BRIDGE_IN` m before the water to as far after made straight (the road turning onto
 * it over the `LEAD` m before and off it over as many after), and the road split into the road
 * before (at index `road`), a straight `country` bridge with its deck, and the road after (those
 * two at the end of the list); and the walkers' mask, `wet` with the deck dry. A piece shorter than
 * `SHORTEST_PIECE` m is taken into the bridge, which then ends as the road did there. The road
 * after keeps the road's name unless `names` gives it one; the bridge is called `names.bridge`, else
 * after the road. Whether it may be bridged there is `bridgeable`'s to say. `fit`: the bridge laid
 * otherwise (`crossWater`'s choice), its deck as the fit has it.
 */
export function withBridge(roads: readonly RoadPlan[], road: number, span: readonly [number, number], wet: Uint8Array, W: number, D: number, names?: { bridge?: string; after?: string }, fit?: BridgeFit): { roads: RoadPlan[]; walk: Uint8Array } {
  const r = roads[road];
  const ls = lengths(r.points);
  const f = fit ?? chordOf(r.points, span, ls);
  const pieces = piecesOf(r.points, f, ls);
  const { bridge: _deck, ...plain } = r;
  const before: RoadPlan | null = pieces.before ? { ...plain, points: pieces.before, ends: [r.ends[0], 'join'] } : null;
  const after: RoadPlan | null = pieces.after ? { ...plain, points: pieces.after, ends: ['join', r.ends[1]], ...(names?.after ? { name: names.after } : {}) } : null;
  const bridge: RoadPlan = {
    points: pieces.bridge,
    width: Math.max(ROAD_WIDTH, r.width),
    ends: [before ? 'join' : r.ends[0], after ? 'join' : r.ends[1]],
    kind: 'country',
    name: names?.bridge ?? `${(r.name ?? 'The').split(' ')[0]} Bridge`,
  };
  const walk = wet.slice();
  if (f.deck) {
    bridge.bridge = [f.deck[0], f.deck[1]];
    if (f.deck[1] > f.deck[0]) deckTiles(bridge, f.deck, W, D, (tile) => (walk[tile] = 0));
  } else bridge.bridge = deckOf(bridge, wet, walk, W, D);
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
