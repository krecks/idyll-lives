/**
 * The Street tool's live checks: what a street drawn through the player's points would be, worked
 * out on every pointer move cheaply enough to keep up with it (well under 5 ms on a city), with
 * where it would be refused and why, so the ghost shows it as the player draws.
 *
 * It takes the line and its ends exactly as `drawStreet` does (`streets.ts` `streetLine`,
 * `streetCourse`: the same curve, the same snapping, the same T-joins), then puts the generator's
 * rules to it in `drawStreet`'s own order, using the same checks for the verdict (`bends`,
 * `roomAt`, `clearOf`, `bridgeable`) and finding, besides, where along the line each one fails:
 * - a bend tighter than 9 m, where it bends;
 * - a junction within 14 m of another, by that junction;
 * - running into or alongside another road, or across a green, where it does;
 * - a crossing of the water: a bridge stretch, or red where it may not be bridged, as `bridges.ts`
 *   `crossWater` says (`drawStreet` lays its bridges with it too);
 * - a stretch that climbs more steeply than its kind may, or that the land would need more than
 *   `CUT_MAX` m of cut or fill under (`profileOf`, on the plan's land as the settle measures against
 *   it: the land as laid with its finer hills smoothed out, as `gentleUnder` smooths them under
 *   anything built). The climb is measured as `landFollow.ts` `measureLand` measures it (`climbOf`:
 *   steeper than `GRADE` by more than 1 %), and both are said in the settle's own words
 *   (`overWords`). This one is a forecast, not a refusal (`slope`, apart from `why`): the settle is
 *   the judge (decision 11) and refuses the street with its own reason; but a clear excess
 *   (`excess`: `CLEAR_CLIMB` or `CLEAR_CUT` past the limit) the tool refuses up front. Inside a
 *   town's or a village's box the settle lays the land to the area's plane first, so stretches there
 *   are not judged.
 *
 * What it leaves to `drawStreet` itself, run when the pointer rests and when the street is ended
 * (`verdictOf`): what is in the way, the roads it is split into at its bridges, the footpaths it
 * cuts and whether the network stays one. Two of its refusals are of a street not finished yet
 * rather than of one that may not be (`unfinished`): a bridge with no road on beyond it, and a
 * street joining no road yet. While drawing they are said, not drawn red.
 */

import type { RoadKind, RoadPlan } from '../../core/protocol';
import { bridgeStretch, crossWater, DRAWN_SPAN, wetSpans } from '../../game/bridges';
import type { Edited } from '../../game/edit/edited';
import { crossingsOf, drawStreet, madeKind, Meeting, ONE_WAY_ENDS, ROOM, streetCourse, streetLine, streetWidth, TRAIL_APART, type End, type StreetLine } from '../../game/edit/streets';
import { heightAt, NONE, type Land } from '../../game/land';
import { overWords, smoothHeights } from '../../game/landFollow';
import { unrle } from '../../game/landMasks';
import { bends, clearOf, inside, lengths, nearest, pointAt, roomAt, tangent, type Fit, type Pt } from '../../game/roads';
import { plotRect, type NeighbourhoodDraft, type Rect, type Templates } from '../../game/town';
import { CUT_MAX, CUT_MAX_MOUNTAIN_ROAD, GRADE, profileOf } from '../../render/shared/builtSurface';

/** What the player asks the street to be: as `drawStreet` takes it (`street`: a street of the area it runs in, else a country road, unless asked for). */
export type StrokeKind = RoadKind;

/** A climb this much (rise per m) past what `measureLand` allows, or cut or fill `CLEAR_CUT` m past it, is a clear excess: the tool refuses the street up front rather than leave it to the settle. */
export const CLEAR_CLIMB = 0.03;
export const CLEAR_CUT = 1;

/** A stretch along a live street's line (arc lengths, m) drawn otherwise than as a new street: refused, or a bridge. */
export interface Stretch {
  from: number;
  to: number;
  tone: 'bad' | 'bridge';
}

/** What the street drawn through some points would be, as far as the live checks tell (see the top of this file). */
export interface LiveStreet {
  /** Its line: the new stretch of road (`StreetCourse.part`), or, where it can't be made at all, the line as drawn. */
  line: Pt[];
  width: number;
  /** What it would be (a dead end it carries on: that road's kind), and the area it is a street of (-1: none). Null: it can't be made. */
  kind: RoadKind | null;
  area: number;
  /** How it starts and ends. Null: it can't be made. */
  start: End | null;
  end: End | null;
  /** Where it would join a road (a T, or the dead end it carries on), and whether there is room for a junction there. */
  joins: { at: Pt; ok: boolean }[];
  /** The stretches drawn red or as bridges, by arc length along `line`. */
  stretches: Stretch[];
  /** Why it would be refused (the first reason, in `drawStreet`'s order); null: as far as the live checks tell, it may be drawn. */
  why: string | null;
  /** Some of it looks too steep (red), and what the settle would say of it: a forecast, not a refusal (see the top of this file). */
  steep: boolean;
  slope: string | null;
  /** The steepest it would climb (rise per m, as `measureLand` measures it), out of the town's and the villages' boxes. */
  climb: number;
  /** It is clearly too steep (`CLEAR_CLIMB`, `CLEAR_CUT`): the tool refuses it with `slope`. */
  excess: boolean;
  /** Why it can't be ended where it ends yet, though nothing is wrong with it so far: it joins no road, or is a dead end longer than a court. */
  open: string | null;
  /** It ends on a road, or at the world's edge: it is finished there. */
  reached: boolean;
  /** The road its far end (by the last point drawn) joins or carries on; -1: none (the edge, or the open). */
  finishOn: number;
  /** Its bridges' lengths over the water (m), those it may have. */
  bridges: number[];
}

/** What the live checks keep of a plan draft (made once per draft: the pointer asks many times over). */
interface Context {
  draft: NeighbourhoodDraft;
  /** The water's 1 m tiles, unpacked. */
  wet: Uint8Array | null;
  /** The land the town is laid on as the settle measures a new road against it (`settledLand`; none in a classic world). */
  land: Land | null;
  mountains: boolean;
  plots: Rect[];
  /** The town's roads as a street of each width meets them (`meetingOf`). */
  meetings: Map<number, Meeting>;
}

const contexts = new WeakMap<NeighbourhoodDraft, Context>();
const smoothed = new WeakMap<Land, Land>();
const unpacked = new WeakMap<object, Uint8Array>();

/** The water's tiles unpacked (kept per run-length list: the edits keep a town's unless a bridge changes it). */
function wetOf(water: Parameters<typeof unrle>[0], size: number): Uint8Array {
  let out = unpacked.get(water);
  if (!out) unpacked.set(water, (out = unrle(water, size)));
  return out;
}

/**
 * `land` as the settle measures a new road's cut and fill against it: its dry heights with the finer
 * hills smoothed out (`landFollow.ts` `smoothHeights`), as `gentleUnder` smooths the land under
 * anything built before the town is graded in (kept per land: the plan drafts share theirs).
 */
function settledLand(land: Land): Land {
  let out = smoothed.get(land);
  if (!out) {
    const smooth = smoothHeights(land);
    const h = land.h.slice();
    for (let k = 0; k < h.length; k++) if (land.wl[k] === NONE || land.wl[k] <= land.h[k]) h[k] = Math.max(-32767, Math.min(32767, Math.round(smooth[k] * 100)));
    smoothed.set(land, (out = { ...land, h }));
  }
  return out;
}

/** What the live checks keep of `draft` (`t` for its lots' rectangles). */
function contextOf(draft: NeighbourhoodDraft, t: Templates): Context {
  let c = contexts.get(draft);
  if (!c) {
    const open = draft.open;
    const options = draft.world?.options as { hills?: string } | undefined;
    c = {
      draft,
      wet: open ? wetOf(open.water, draft.width * draft.depth) : null,
      land: open?.land ? settledLand(open.land) : null,
      mountains: options?.hills === 'mountains',
      plots: draft.slots.map((s) => plotRect(t, s)),
      meetings: new Map(),
    };
    contexts.set(draft, c);
  }
  return c;
}

/** What the live checks keep of `draft` made now (the smoothed land takes tens of milliseconds on a big world): before the first pointer move asks. */
export function warmLive(draft: NeighbourhoodDraft, t: Templates): void {
  contextOf(draft, t);
}

/** The town's roads as a street of `kind` meets them (a footpath's, or a road's of its width: an avenue keeps further off). */
function meetingOf(c: Context, kind: RoadKind): Meeting {
  const trail = kind === 'trail';
  const width = streetWidth(kind);
  const key = trail ? -1 : width;
  let m = c.meetings.get(key);
  if (!m) c.meetings.set(key, (m = new Meeting(c.draft.roads, trail, width)));
  return m;
}

/** A `drawStreet` refusal of a street that is not finished rather than one that may not be: it reaches no road yet, its dead end is longer than a court, or it is one-way with an end on no road yet. */
export const unfinished = (why: string) => /^it joins none of the roads$|past the last road it meets|^a bridge needs a road on at each end/.test(why) || why === ONE_WAY_ENDS;

/** Stretches merged where they meet or overlap (of one tone), and clamped to `[0, L]`. */
function merged(list: readonly Stretch[], L: number): Stretch[] {
  const out: Stretch[] = [];
  for (const tone of ['bridge', 'bad'] as const) {
    const of = list.filter((s) => s.tone === tone).map((s) => ({ from: Math.max(0, s.from), to: Math.min(L, s.to), tone })).filter((s) => s.to > s.from).sort((a, b) => a.from - b.from);
    for (const s of of) {
      const last = out[out.length - 1];
      if (last && last.tone === tone && s.from <= last.to + 0.5) last.to = Math.max(last.to, s.to);
      else out.push({ ...s });
    }
  }
  return out;
}

/** Where along `pts` it bends tighter than 9 m (arc lengths), stepping as `roads.ts` `bends` does. */
function tightAt(pts: readonly Pt[]): Stretch[] {
  const ls = lengths(pts);
  const L = ls[ls.length - 1];
  const out: Stretch[] = [];
  let prev: Pt | null = null;
  for (let s = 0; s + 3 <= L + 1e-6; s += 3) {
    const a = pointAt(pts, s, ls).p;
    const b = pointAt(pts, Math.min(L, s + 3), ls).p;
    const l = Math.hypot(b[0] - a[0], b[1] - a[1]) || 1;
    const t: Pt = [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
    if (prev && Math.abs(Math.atan2(prev[0] * t[1] - prev[1] * t[0], prev[0] * t[0] + prev[1] * t[1])) > 3 / 9) out.push({ from: s - 4, to: s + 4, tone: 'bad' });
    prev = t;
  }
  return out;
}

/** Where along `pts` it is not clear of the roads and greens (as `roads.ts` `clearOf` judges each point: arc lengths). */
function unclearAt(pts: readonly Pt[], width: number, roads: readonly RoadPlan[], m: Meeting, fit: Fit, from: number, to: number, apart: number): Stretch[] {
  const ls = lengths(pts);
  const L = ls[ls.length - 1];
  const widest = Math.max(width, ...roads.map((r) => r.width));
  const out: Stretch[] = [];
  for (let i = 0; i < pts.length; i++) {
    const [x, z] = pts[i];
    const [s, u] = [ls[i], L - ls[i]];
    let bad = s > 4 && u > 4 && (fit.greens ?? []).some((g) => inside(g, x, z));
    const t = tangent(pts, i);
    for (const [r, near] of m.index.around(x, z, widest / 2 + apart)) {
      if (bad) break;
      if (m.skip(r)) continue;
      const clear = roads[r].width / 2 + width / 2 + 1;
      const free = clear * 1.8 + 1;
      if (near.dist < clear && !((r === from && s < free) || (r === to && u < free))) bad = true;
      else if (s > 20 && u > 20 && near.dist < roads[r].width / 2 + apart && Math.abs(t[0] * near.t[0] + t[1] * near.t[1]) > Math.cos((35 * Math.PI) / 180)) {
        const q = pointAt(roads[r].points, near.s).p;
        if (Math.abs((x - q[0]) * near.t[0] + (z - q[1]) * near.t[1]) < near.dist * 0.5) bad = true;
      }
    }
    if (bad) out.push({ from: s - 2, to: s + 2, tone: 'bad' });
  }
  return out;
}

/** The lots under a line (within its width and 1.5 m, as `drawStreet` clears them), of those whose rectangles `plots` near `box`. */
function underLine(plots: readonly Rect[], pts: readonly Pt[], half: number): Set<number> {
  const xs = pts.map((p) => p[0]);
  const zs = pts.map((p) => p[1]);
  const box = [Math.min(...xs) - half, Math.min(...zs) - half, Math.max(...xs) + half, Math.max(...zs) + half];
  const out = new Set<number>();
  plots.forEach((r, i) => {
    if (r.x > box[2] || r.x + r.w < box[0] || r.z > box[3] || r.z + r.d < box[1]) return;
    for (const [x, z] of pts) if (Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d)) < half) return void out.add(i);
  });
  return out;
}

/** A road's height where a street meets it: the land's there (the town as laid has it graded under its roads). */
const heightOn = (land: Land, p: Pt) => heightAt(land, p[0], p[1]);

/** A live street that can't be made at all: the line as drawn (or shaped), red, and why. */
function cannot(line: Pt[], width: number, why: string): LiveStreet {
  const L = line.length > 1 ? lengths(line)[line.length - 1] : 0;
  return { line, width, kind: null, area: -1, start: null, end: null, joins: [], stretches: L > 0 ? [{ from: 0, to: L, tone: 'bad' }] : [], why, steep: false, slope: null, climb: 0, excess: false, open: null, reached: false, finishOn: -1, bridges: [] };
}

/**
 * The street drawn through `points` on `draft` as the live checks see it (see the top of this file):
 * of `kind` (as `drawStreet` takes it, `asked` too: `StreetStroke.asked`), clearing what is in its
 * way (`clear`) or not; one-way (`oneway`), it is open till it ends on a road at both ends.
 */
export function liveStreet(draft: NeighbourhoodDraft, t: Templates, points: readonly Pt[], kind: StrokeKind, clear = true, asked = false, oneway?: 1 | -1): LiveStreet {
  const c = contextOf(draft, t);
  const trail = kind === 'trail';
  const width = streetWidth(kind);
  const [W, D] = [draft.width, draft.depth];
  const shaped: StreetLine | string = streetLine(points, W, D, width);
  if (typeof shaped === 'string') return cannot(points.map(([x, z]): Pt => [x, z]), width, shaped);
  const meet = meetingOf(c, kind);
  const course = streetCourse(meet, shaped);
  if (typeof course === 'string') return cannot(shaped.points, width, course);
  const roads = draft.roads;
  const named = (r: number) => roads[r].name ?? 'a road';
  const { start, end, junctions, part, head, foot } = course;
  const ls = lengths(part);
  const L = ls[ls.length - 1];
  const list: Stretch[] = [];
  let why: string | null = null;
  const refuse = (reason: string, ...where: Stretch[]) => {
    why ??= reason;
    list.push(...where);
  };
  const near = (p: Pt) => (Math.hypot(p[0] - part[0][0], p[1] - part[0][1]) <= Math.hypot(p[0] - part[part.length - 1][0], p[1] - part[part.length - 1][1]) ? 0 : L);
  const by = (p: Pt, reach: number): Stretch => {
    const s = near(p);
    return { from: s - reach, to: s + reach, tone: 'bad' };
  };

  // ---- the generator's rules, in drawStreet's order ----
  if (bends([...head.slice(-20, -1), ...part, ...foot.slice(1, 20)]).tight) {
    const at = tightAt(part);
    refuse('it bends tighter than 9 m', ...(at.length ? at : [head.length ? { from: 0, to: 8, tone: 'bad' as const } : { from: L - 8, to: L, tone: 'bad' as const }]));
  }
  const joins: LiveStreet['joins'] = [];
  for (const j of junctions) {
    const ok = roomAt(meet.index, j.at, [j.road], meet.skip);
    joins.push({ at: j.at, ok });
    if (!ok) refuse(`it meets ${named(j.road)} within ${ROOM} m of another junction`, by(j.at, ROOM));
  }
  if (start.kind === 'extend') joins.push({ at: part[0], ok: true });
  if (end.kind === 'extend') joins.push({ at: part[part.length - 1], ok: true });
  if (junctions.length === 2 && Math.hypot(junctions[0].at[0] - junctions[1].at[0], junctions[0].at[1] - junctions[1].at[1]) < ROOM) refuse(`its two junctions are within ${ROOM} m of each other`, { from: 0, to: L, tone: 'bad' });
  const fromRoad = start.kind === 'join' || start.kind === 'extend' ? start.road : -1;
  const toRoad = end.kind === 'join' || end.kind === 'extend' ? end.road : -1;
  const fit: Fit = { box: { w: W, d: D }, margin: -0.5, greens: draft.greens };
  const apart = trail ? TRAIL_APART : 18;
  if (!clearOf(part, width, roads, meet.index, fit, fromRoad, toRoad, trail ? TRAIL_APART : undefined, meet.skip)) {
    const at = unclearAt(part, width, roads, meet, fit, fromRoad, toRoad, apart);
    const where = at.length ? at : [{ from: 0, to: L, tone: 'bad' as const }];
    if (part.some(([x, z], i) => ls[i] > 4 && L - ls[i] > 4 && draft.greens.some((g) => inside(g, x, z)))) refuse('it runs across a green', ...where);
    else if (!clearOf(part, width, roads, meet.index, fit, fromRoad, toRoad, 0, meet.skip)) refuse('it runs into another road', ...where);
    else refuse(`it runs closer than ${apart} m alongside another road`, ...where);
  }

  // ---- what it would be ----
  const it = start.kind === 'extend' ? { kind: roads[start.road].kind ?? 'street', area: roads[start.road].area } : madeKind(draft, part, kind, asked);
  const made: RoadKind = it.kind ?? 'street';
  const area = it.area ?? -1;

  // ---- the water: a bridge over each crossing (the last first, as drawStreet bridges them: `bridges.ts` `crossWater`) ----
  const bridges: number[] = [];
  const decks: Stretch[] = [];
  let bridgeOpen: string | null = null;
  if (c.wet) {
    const spans = wetSpans(part, c.wet, W, D);
    if (spans.length && trail) refuse('a footpath does not cross the water', ...spans.map(([a, b]) => ({ from: a, to: b, tone: 'bad' as const })));
    else if (spans.length) {
      // (Not the dead ends it carries on: they are of the road it makes.)
      const base = start.kind === 'extend' ? start.road : -1;
      const merging = start.kind === 'extend' && end.kind === 'extend' ? end.road : -1;
      const others = roads.filter((_, i) => i !== base && i !== merging && !meet.crossable(i));
      // (What the street clears is out of the bridge's way; what it doesn't, stays.)
      const gone = clear ? underLine(c.plots, part, width / 2 + 1.5) : new Set<number>();
      const plots = gone.size ? c.plots.filter((_, i) => !gone.has(i)) : c.plots;
      const ends: RoadPlan['ends'] = [start.kind === 'join' || start.kind === 'extend' ? 'join' : 'dead', end.kind === 'join' || end.kind === 'extend' ? 'join' : 'dead'];
      // (Each crossing laid on the line as drawn up to it, as drawStreet lays it on the road before the bridges after it.)
      let line: Pt[] = part;
      for (const span of crossingsOf(spans).reverse()) {
        const [a, b] = bridgeStretch(L, span);
        const crossed = crossWater(line, line === part ? ends : [ends[0], 'join'], span, { wet: c.wet, W, D, roads: others, plots, land: draft.open?.land, most: DRAWN_SPAN });
        if (typeof crossed === 'string') {
          // (One wanting a road on beyond it is a street not finished yet: said, not drawn red.)
          if (/^a bridge needs a road on at each end/.test(crossed)) bridgeOpen ??= crossed;
          else refuse(crossed, { from: a, to: b, tone: 'bad' });
          continue;
        }
        const { fit } = crossed;
        list.push({ from: fit.a, to: fit.b, tone: 'bridge' });
        // (Its deck, where the land under it is not judged: along the line as drawn, where the deck's ends are nearest.)
        const bl = lengths(crossed.bridge);
        const [d0, d1] = [pointAt(crossed.bridge, fit.deck![0], bl).p, pointAt(crossed.bridge, fit.deck![1], bl).p];
        const [s0, s1] = [nearest(part, d0[0], d0[1], ls).s, nearest(part, d1[0], d1[1], ls).s];
        decks.push({ from: Math.min(s0, s1), to: Math.max(s0, s1), tone: 'bridge' });
        const water = wetSpans(crossed.bridge, c.wet, W, D);
        bridges.unshift(Math.round(water.length ? water[water.length - 1][1] - water[0][0] : 0));
        if (crossed.before) line = crossed.before;
      }
    }
  }

  // ---- the land: no steeper than its kind may climb, and no more cut or fill under it than the settle allows ----
  let slope: string | null = null;
  let [climb, excess] = [0, false];
  if (c.land && made !== 'trail') {
    const most = c.mountains && made === 'country' ? CUT_MAX_MOUNTAIN_ROAD : CUT_MAX;
    const held = (e: End, p: Pt) => (e.kind === 'join' || e.kind === 'extend' ? heightOn(c.land!, p) : undefined);
    const profile = profileOf(part, made, c.land, { start: held(start, part[0]), end: held(end, part[part.length - 1]) });
    const boxes = draft.open?.areas ?? [];
    const judged = profile.s.map((_, i) => !boxes.some((b) => profile.x[i] >= b.x && profile.x[i] <= b.x + b.w && profile.z[i] >= b.z && profile.z[i] <= b.z + b.d));
    const climbs = climbOf(profile, made, judged);
    climb = climbs.steepest;
    const where: Stretch[] = climbs.over.map(([from, to]) => ({ from, to, tone: 'bad' as const }));
    let cut = 0;
    for (let i = 0; i < profile.s.length; i++) {
      const s = profile.s[i];
      if (!judged[i] || Math.abs(profile.cut[i]) <= most || decks.some((d) => s >= d.from && s <= d.to)) continue;
      cut = Math.max(cut, Math.abs(profile.cut[i]));
      where.push({ from: s - 1.5, to: s + 1.5, tone: 'bad' });
    }
    if (climbs.over.length) slope = overWords({ what: 'climb', by: climb, most: GRADE[made] }, 'it', made);
    else if (cut > 0) slope = overWords({ what: 'cut', by: cut, most }, 'it', made);
    excess = climb > GRADE[made] + 0.01 + CLEAR_CLIMB || cut > most + CLEAR_CUT;
    list.push(...where);
  }

  // ---- whether it may end where it ends ----
  const onRoad = (e: End) => e.kind !== 'free';
  let open: string | null = null;
  if (!onRoad(start) && !onRoad(end)) open = 'it joins no road yet: start or end it on one';
  else if (bridgeOpen) open = bridgeOpen;
  else if (oneway && !trail && (!onRoad(start) || !onRoad(end))) open = ONE_WAY_ENDS;
  // (Its far end is the one nearest the last point drawn: a dead end it carries on is its start, whichever end the player drew it from.)
  const last = points[points.length - 1];
  const far = near(last) === 0 ? start : end;
  const reached = onRoad(far);
  const finishOn = far.kind === 'join' || far.kind === 'extend' ? far.road : -1;
  return { line: part, width, kind: made, area, start, end, joins, stretches: merged(list, L), why, steep: slope !== null, slope, climb, excess, open, reached, finishOn, bridges };
}

/**
 * How steeply a road's profile (`s`, `y`: arc lengths and heights) climbs, as `landFollow.ts` `measureLand` measures
 * a road of `kind`: the steepest rise per m between samples more than half a metre apart, and the stretches between
 * them steeper than `GRADE[kind]` by more than 1 % (arc lengths). `judged`: per sample, whether it counts (a step
 * counts where both its samples do; all, if not given).
 */
export function climbOf(profile: { s: ArrayLike<number>; y: ArrayLike<number> }, kind: RoadKind, judged?: readonly boolean[]): { steepest: number; over: [number, number][] } {
  const { s, y } = profile;
  let steepest = 0;
  const over: [number, number][] = [];
  for (let k = 0; k + 1 < s.length; k++) {
    if (s[k + 1] - s[k] <= 0.5 || (judged && !(judged[k] && judged[k + 1]))) continue;
    const g = Math.abs(y[k + 1] - y[k]) / (s[k + 1] - s[k]);
    steepest = Math.max(steepest, g);
    if (g > GRADE[kind] + 0.01) over.push([s[k], s[k + 1]]);
  }
  return { steepest, over };
}

/** `drawStreet`'s own verdict on the street drawn through `points` (no homes along it: what it would be, not what it fills). */
export function verdictOf(draft: NeighbourhoodDraft, t: Templates, content: Parameters<typeof drawStreet>[2], points: readonly Pt[], kind: StrokeKind, clear: boolean, seed: number, asked = false, oneway?: 1 | -1): Edited {
  return drawStreet(draft, t, content, { points, kind, clear, fill: false, seed, asked, ...(oneway ? { oneway } : {}) });
}

/** The roads a street drawn on `before` made (`after.made`: its own pieces, not the footpaths it cut), else those `after.draft` has that `before` hasn't (by the road itself: the edits keep the roads they leave as they were). */
export function newRoads(before: NeighbourhoodDraft, after: { draft: NeighbourhoodDraft; made?: readonly number[] }): number[] {
  if (after.made) return [...after.made];
  const had = new Set(before.roads);
  return after.draft.roads.flatMap((r, i) => (had.has(r) ? [] : [i]));
}

/**
 * `line` split into runs by `stretches` (arc lengths): each run's points (the cut points put in)
 * and its tone, a refused stretch over a bridge's, a bridge's over the new street's.
 */
export function runsOf(line: readonly Pt[], stretches: readonly Stretch[]): { points: Pt[]; tone: 'new' | 'bad' | 'bridge' }[] {
  if (line.length < 2) return [];
  const ls = lengths(line);
  const L = ls[ls.length - 1];
  const toneAt = (s: number): 'new' | 'bad' | 'bridge' => (stretches.some((x) => x.tone === 'bad' && s >= x.from && s <= x.to) ? 'bad' : stretches.some((x) => x.tone === 'bridge' && s >= x.from && s <= x.to) ? 'bridge' : 'new');
  const cuts = [...new Set([0, L, ...stretches.flatMap((x) => [x.from, x.to]).filter((s) => s > 0 && s < L)])].sort((a, b) => a - b);
  const out: { points: Pt[]; tone: 'new' | 'bad' | 'bridge' }[] = [];
  for (let k = 0; k + 1 < cuts.length; k++) {
    const [a, b] = [cuts[k], cuts[k + 1]];
    if (b - a < 0.05) continue;
    const tone = toneAt((a + b) / 2);
    const pts: Pt[] = [pointAt(line, a, ls).p];
    for (let i = 0; i < line.length; i++) if (ls[i] > a + 0.01 && ls[i] < b - 0.01) pts.push(line[i]);
    pts.push(pointAt(line, b, ls).p);
    const last = out[out.length - 1];
    if (last && last.tone === tone) last.points.push(...pts.slice(1));
    else out.push({ points: pts, tone });
  }
  return out;
}

/** The stretches the last street drawn was made as (`streetChoice.drawn`), while `draft` still has them all as they were named; none for a street not split. */
export function drawnStretches(draft: Pick<NeighbourhoodDraft, 'roads'>, drawn: readonly { road: number; name: string }[]): { road: number; name: string; deck: boolean }[] {
  if (drawn.length < 2 || drawn.some((d) => !draft.roads[d.road] || (draft.roads[d.road].name ?? '') !== d.name)) return [];
  return drawn.map((d) => ({ ...d, deck: !!draft.roads[d.road].bridge }));
}
