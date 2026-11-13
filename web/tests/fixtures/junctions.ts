/**
 * Junctions worked out from the roads, for tests: as the simulation joins them up (sim-core
 * `travel::Graph::build`) and lights them (`traffic::signal_at`). Production takes the
 * simulation's own list (`WorldStructure.junctions`, sim-core `traffic::Signal`); this is the
 * fallback the renderer once had, kept so tests can build a structure from a few roads.
 */

import type { Junction, RoadPlan } from '../../src/core/protocol';
import { lengths, pointAt, SIDEWALK, type Pt } from '../../src/game/roads';
import { carriageway, CROSSWALK, hasSidewalks, STOP_PAST } from '../../src/render/shared/junctions';

/** The box round a junction (sim-core `traffic::boxes`): how far out along its arms (m) at least and at most, and how far apart two arms' lanes are at its edge. */
const BOX_LEAST = 6;
const BOX_MOST = 14;
const ARMS_APART = 4.6;
/** How far along an arm's road its way out is taken (m, sim-core `ARM_LOOK`). */
const ARM_LOOK = 5;
/** Two roads' meeting places this near are one junction (m). */
const SAME_JUNCTION = 1.5;
/** A junction is lit where this many arms meet (not only country roads), or this many of streets and country roads, a country road's counted only with a street there (sim-core `traffic::LIT_ARMS`, `LIT_MAIN_ARMS`, `lit_of`). */
const LIT_ARMS = 4;
const LIT_MAIN_ARMS = 3;
/** Junctions giving way this near each other along a street (m) have one zebra, between them where it has room, its crosswalk no more than `ZEBRA_SQUEEZE` m further in (sim-core `traffic::ZEBRA_APART`). */
const ZEBRA_APART = 40;
const ZEBRA_SQUEEZE = 1.5;
/** Two arms whose ways out are further apart than 150° (the cosine) are across from each other (sim-core `traffic::OPPOSITE`). */
const OPPOSITE = -0.866;

/** Whether cars use a road (not a trail). */
const drivable = (r: RoadPlan | undefined) => !!r && r.kind !== 'trail' && r.points.length >= 2;

/** Each set of roads' junctions, worked out once. */
const derived = new WeakMap<readonly RoadPlan[], Junction[]>();

/**
 * The junctions of `roads` as the simulation joins them up (sim-core `travel::Graph::build`): a
 * road's end on another road's middle line (within half the wider one's width and a metre), two
 * roads crossing, places this near one another one junction; those where three arms or more meet.
 * Its middle is on the road met (a lane begins at that road's kerb: `trimJoins`). Each arm's
 * crosswalk is just out past the junction's box (sim-core `traffic::boxes`), its stop line past
 * that. A junction is lit as the simulation lights them (sim-core `traffic::signal_at`: a big one,
 * where four arms or more meet or three of streets and country roads, and a roundabout's), its
 * heads numbered in the order worked out, not the snapshot's; any other gives way. Its zebras as
 * the simulation has them too: a lit one's on its streets' and lanes' arms, one giving way's on a
 * street's arm, one of two through it (the first round from the way out along z, as the
 * simulation goes round them); of junctions giving way within `ZEBRA_APART` of each other along a
 * street, only the first along it has one, on the stretch to the next where that has room (none
 * by a lit one).
 */
export function deriveJunctions(roads: readonly RoadPlan[]): Junction[] {
  const had = derived.get(roads);
  if (had) return had;
  const lens = roads.map((r) => lengths(r.points));
  const bounds = roads.map((r) => boundsOf(r.points));
  const ring = (r: RoadPlan) => r.ends[0] === 'loop';
  /** Where two roads meet: the point, and where along each. */
  const touches: { p: Pt; on: [road: number, s: number][] }[] = [];
  const near = (q: number, x: number, z: number) => nearestOn(roads[q].points, lens[q], x, z);
  roads.forEach((r, i) => {
    if (!drivable(r) || ring(r)) return;
    for (const e of [0, 1] as const) {
      const p = e === 0 ? r.points[0] : r.points[r.points.length - 1];
      const se = e === 0 ? 0 : lens[i][lens[i].length - 1];
      roads.forEach((o, q) => {
        if (q === i || !drivable(o)) return;
        const snap = Math.max(o.width, r.width) / 2 + 1;
        if (!boxNear(bounds[q], p, snap)) return;
        const { dist, s } = near(q, p[0], p[1]);
        if (dist > snap) return;
        const len = lens[q][lens[q].length - 1];
        // (Near the other's end: their ends meet there.)
        const sq = !ring(o) && s < snap ? 0 : !ring(o) && s > len - snap ? len : s;
        touches.push({ p: pointAt(o.points, sq, lens[q]).p, on: [[i, se], [q, sq]] });
      });
    }
  });
  for (const c of crossings(roads, lens)) touches.push(c);
  // Touches this near each other: one junction.
  const groups: (typeof touches)[] = [];
  for (const t of touches) {
    const g = groups.find((g) => g.some((u) => Math.hypot(u.p[0] - t.p[0], u.p[1] - t.p[1]) < SAME_JUNCTION));
    if (g) g.push(t);
    else groups.push([t]);
  }
  type Way = { road: number; dir: Pt; k: number; s: number };
  const found: { x: number; z: number; roundabout: boolean; lit: boolean; ways: Way[] }[] = [];
  for (const g of groups) {
    const x = g.reduce((n, t) => n + t.p[0], 0) / g.length;
    const z = g.reduce((n, t) => n + t.p[1], 0) / g.length;
    // Each road's arms: one out along it from its end, two from its middle.
    const along = new Map<number, number[]>();
    for (const t of g) for (const [road, s] of t.on) along.set(road, [...(along.get(road) ?? []), s]);
    const ways: { road: number; dir: Pt; k: number; s: number }[] = [];
    for (const [road, ss] of along) {
      const r = roads[road];
      const s = ss.reduce((n, v) => n + v, 0) / ss.length;
      const len = lens[road][lens[road].length - 1];
      for (const k of [1, -1]) {
        let at = s + k * ARM_LOOK;
        if (ring(r)) at = ((at % len) + len) % len;
        else if (at < 0 || at > len) {
          if (Math.abs(s - (k > 0 ? len : 0)) < 0.5) continue;
          at = Math.max(0, Math.min(len, at));
        }
        const q = pointAt(r.points, at, lens[road]).p;
        const l = Math.hypot(q[0] - x, q[1] - z);
        if (l < 0.5) continue;
        const dir: Pt = [(q[0] - x) / l, (q[1] - z) / l];
        if (ways.some((w) => w.dir[0] * dir[0] + w.dir[1] * dir[1] > 0.995)) continue;
        ways.push({ road, dir, k, s });
      }
    }
    if (ways.length < 3) continue;
    const roundabout = ways.some((w) => roads[w.road].island);
    ways.sort((a, b) => Math.atan2(a.dir[1], a.dir[0]) - Math.atan2(b.dir[1], b.dir[0]));
    const kind = (road: number) => roads[road].kind ?? 'street';
    const streets = ways.filter((w) => kind(w.road) === 'street').length;
    const country = streets > 0 ? ways.filter((w) => kind(w.road) === 'country').length : 0;
    const allCountry = ways.every((w) => kind(w.road) === 'country');
    const lit = roundabout || (ways.length >= LIT_ARMS && !allCountry) || streets + country >= LIT_MAIN_ARMS;
    found.push({ x, z, roundabout, lit, ways });
  }
  const kind = (road: number) => roads[road].kind ?? 'street';
  // (Along a road from `s` its way `k`: the next junction on it, and how far.)
  const next = (j: number, road: number, s: number, k: number) => {
    let best: { j: number; d: number } | null = null;
    found.forEach((o, i) => {
      if (i === j) return;
      for (const w of o.ways) {
        if (w.road !== road) continue;
        const d = (w.s - s) * k;
        if (d > 0.5 && (!best || d < best.d)) best = { j: i, d };
      }
    });
    return best as { j: number; d: number } | null;
  };
  const out: Junction[] = [];
  let light = 0;
  found.forEach(({ x, z, roundabout, lit, ways }, j) => {
    const dirs = ways.map((w) => w.dir);
    const reach = ways.map((w) => boxReach(w.dir, dirs));
    // (Giving way, of a street's two arms through it the first round as the simulation goes, by
    // the angle from z towards x.)
    const round = (d: Pt) => Math.atan2(d[0], d[1]);
    const across = (w: Way) => ways.some((o) => o !== w && kind(o.road) === 'street' && o.dir[0] * w.dir[0] + o.dir[1] * w.dir[1] < OPPOSITE && round(o.dir) < round(w.dir));
    // (And of junctions giving way close together along a street, one zebra: sim-core `signal_at`.)
    const near = ways.map((w) => {
      if (lit || kind(w.road) !== 'street') return { at: 'far' as const, d: Infinity };
      const n = next(j, w.road, w.s, w.k);
      if (!n || n.d > ZEBRA_APART) return { at: 'far' as const, d: Infinity };
      return { at: found[n.j].lit ? ('lit' as const) : w.k < 0 ? ('before' as const) : ('after' as const), d: n.d };
    });
    const shared = near.some((n) => n.at === 'lit' || n.at === 'before');
    const order = ways.map((_, i) => i).sort((a, b) => round(ways[a].dir) - round(ways[b].dir));
    const fits = (i: number) => reach[i] + CROSSWALK + STOP_PAST <= near[i].d / 2 + ZEBRA_SQUEEZE;
    let toward = order.find((i) => near[i].at === 'after' && fits(i));
    if (toward === undefined && near.some((n) => n.at === 'after')) toward = order.find((i) => kind(ways[i].road) === 'street' && near[i].at === 'far');
    const street = (i: number) => !shared && (toward === undefined ? !across(ways[i]) : toward === i);
    const zebra = (i: number) => !roundabout && (kind(ways[i].road) === 'street' ? lit || street(i) : kind(ways[i].road) === 'lane' && lit);
    out.push({
      x,
      z,
      ...(roundabout ? { roundabout: true as const } : {}),
      arms: ways.map(({ road, dir }, i) => {
        const walkFrom = reach[i];
        return { road, dir, width: carriageway(roads[road]), walkFrom, walkTo: walkFrom + CROSSWALK, stop: walkFrom + CROSSWALK + STOP_PAST, crosswalk: zebra(i), light: lit && !roundabout ? light++ : null };
      }),
    });
  });
  derived.set(roads, out);
  return out;
}

/** How far out along arm `a` a junction's box reaches (m, sim-core `traffic::boxes`): as far as its lane is `ARMS_APART` from another's going the same way. */
function boxReach(a: Pt, all: readonly Pt[]): number {
  let r = BOX_LEAST;
  for (const b of all) {
    if (b === a) continue;
    const cos = a[0] * b[0] + a[1] * b[1];
    if (cos > 0) r = Math.max(r, ARMS_APART / Math.max(1e-3, Math.sqrt(Math.max(0, 1 - cos * cos))));
  }
  return Math.min(BOX_MOST, r);
}

/** A polyline's bounding box (x0, z0, x1, z1). */
function boundsOf(pts: readonly Pt[]): [number, number, number, number] {
  const b: [number, number, number, number] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const [x, z] of pts) {
    if (x < b[0]) b[0] = x;
    if (x > b[2]) b[2] = x;
    if (z < b[1]) b[1] = z;
    if (z > b[3]) b[3] = z;
  }
  return b;
}

/** Whether `p` is within `m` m of a bounding box. */
const boxNear = ([x0, z0, x1, z1]: readonly number[], p: Pt, m: number) => p[0] > x0 - m && p[0] < x1 + m && p[1] > z0 - m && p[1] < z1 + m;

/** The nearest point of a polyline (its distance and arc length). */
function nearestOn(pts: readonly Pt[], ls: readonly number[], x: number, z: number): { dist: number; s: number } {
  let best = { dist: Infinity, s: 0 };
  for (let i = 0; i + 1 < pts.length; i++) {
    const [[ax, az], [bx, bz]] = [pts[i], pts[i + 1]];
    const [dx, dz] = [bx - ax, bz - az];
    const l2 = dx * dx + dz * dz || 1;
    const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / l2));
    const dist = Math.hypot(x - (ax + dx * t), z - (az + dz * t));
    if (dist < best.dist) best = { dist, s: ls[i] + t * (ls[i + 1] - ls[i]) };
  }
  return best;
}

/** Where two roads for cars cross away from their ends (sim-core `travel::crossings`): segments filed by 8 m cells, each pair crossing once. */
function crossings(roads: readonly RoadPlan[], lens: readonly number[][]): { p: Pt; on: [number, number][] }[] {
  const CELL = 8;
  const cells = new Map<number, [number, number][]>();
  roads.forEach((r, i) => {
    if (!drivable(r)) return;
    for (let k = 0; k + 1 < r.points.length; k++) {
      const [[ax, az], [bx, bz]] = [r.points[k], r.points[k + 1]];
      for (let cz = Math.floor(Math.min(az, bz) / CELL); cz <= Math.floor(Math.max(az, bz) / CELL); cz++) {
        for (let cx = Math.floor(Math.min(ax, bx) / CELL); cx <= Math.floor(Math.max(ax, bx) / CELL); cx++) {
          const key = cx * 65536 + cz;
          const list = cells.get(key);
          if (list) list.push([i, k]);
          else cells.set(key, [[i, k]]);
        }
      }
    }
  });
  const out: { p: Pt; on: [number, number][] }[] = [];
  const seen = new Set<string>();
  for (const list of cells.values()) {
    for (let u = 0; u < list.length; u++) {
      for (let v = u + 1; v < list.length; v++) {
        const [[i, k], [j, l]] = [list[u], list[v]];
        if (i === j) continue;
        const key = i < j ? `${i}:${k}:${j}:${l}` : `${j}:${l}:${i}:${k}`;
        if (seen.has(key)) continue;
        seen.add(key);
        const [a0, a1, b0, b1] = [roads[i].points[k], roads[i].points[k + 1], roads[j].points[l], roads[j].points[l + 1]];
        const hit = cross(a0, a1, b0, b1);
        if (!hit) continue;
        const sa = lens[i][k] + (lens[i][k + 1] - lens[i][k]) * hit[0];
        const sb = lens[j][l] + (lens[j][l + 1] - lens[j][l]) * hit[1];
        const snap = Math.max(roads[i].width, roads[j].width) / 2 + 1;
        const inside = (r: number, s: number) => roads[r].ends[0] === 'loop' || (s >= snap && s <= lens[r][lens[r].length - 1] - snap);
        if (!inside(i, sa) || !inside(j, sb)) continue;
        out.push({ p: [a0[0] + (a1[0] - a0[0]) * hit[0], a0[1] + (a1[1] - a0[1]) * hit[0]], on: [[i, sa], [j, sb]] });
      }
    }
  }
  return out;
}

/** Where segments a0–a1 and b0–b1 cross (how far along each, 0..1), or null. */
function cross(a0: Pt, a1: Pt, b0: Pt, b1: Pt): [number, number] | null {
  const [ux, uz, vx, vz] = [a1[0] - a0[0], a1[1] - a0[1], b1[0] - b0[0], b1[1] - b0[1]];
  const d = ux * vz - uz * vx;
  if (Math.abs(d) < 1e-9) return null;
  const [wx, wz] = [b0[0] - a0[0], b0[1] - a0[1]];
  const t = (wx * vz - wz * vx) / d;
  const s = (wx * uz - wz * ux) / d;
  return t >= 0 && t < 1 && s >= 0 && s < 1 ? [t, s] : null;
}
