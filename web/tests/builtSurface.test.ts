// Built land on a land that slopes (`render/shared/builtSurface.ts`), on a small made-up world: a steep
// street with a bridge over a channel, a lane in a T off it, a street crossing it, a country road carrying
// it on, a trail off it, two lots on paths and one off any road. Every road within its kind's grade (a
// trail, a footpath, on the land as it lies);
// each junction one height over its arms; each pad within a short drive of its road where its way in
// meets it, at the middle of the land under it where it may be, and near the pads beside it; the
// bridge clear of its water; the land graded within the cones and the retaining edge, and under a road's edges; `update` after an
// edit the same as a surface made afresh, its chunks' versions moved; `at` without a jump along the
// roads. `bun test tests/builtSurface.test.ts`.
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { blankLand, heightAt, LAND_CELL, NONE, type Land } from '../src/game/land';
import { bridgeSpans, lengths, pointAt, type Pt } from '../src/game/roads';
import { BRIDGE_CLEAR, BuiltSurface, builtSurfaceOf, CUT_MAX, ENTRY_MAX, GRADE, RETAIN_MAX, ROAD_SINK, surfacePlots, type SurfaceWorld } from '../src/render/shared/builtSurface';
import { chunkAt } from '../src/render/shared/chunks';

const [W, D] = [160, 120];
/** The channel the street bridges (x, m), its water's level and its bed (m). */
const CHANNEL: [number, number] = [70, 84];
const WATER = 0;

/** A hilly land: steep along x, rolling along z, the channel cut across it. */
function makeLand(): Land {
  const land = blankLand(W, D, 'follow');
  for (let j = 0; j < land.rows; j++) {
    for (let i = 0; i < land.cols; i++) {
      const [x, z] = [i * LAND_CELL, j * LAND_CELL];
      const k = j * land.cols + i;
      if (x >= CHANNEL[0] && x <= CHANNEL[1]) {
        land.wl[k] = WATER * 100;
        land.h[k] = -150;
      } else land.h[k] = Math.round((10 * Math.sin(x / 25) + 0.15 * z + 2 * Math.sin(z / 7)) * 100);
    }
  }
  return land;
}

/** A road from `a` to `b`, a point a metre. */
function line(a: Pt, b: Pt, width: number, kind: RoadPlan['kind'], ends: RoadPlan['ends'], bridge?: [number, number]): RoadPlan {
  const n = Math.max(1, Math.round(Math.hypot(b[0] - a[0], b[1] - a[1])));
  const points = Array.from({ length: n + 1 }, (_, k): [number, number] => [a[0] + ((b[0] - a[0]) * k) / n, a[1] + ((b[1] - a[1]) * k) / n]);
  return { points, width, kind, ends, ...(bridge ? { bridge } : {}) };
}

const roads: RoadPlan[] = [
  line([0, 40], [130, 40], 7, 'street', ['out', 'join'], [66, 88]),
  // A lane begun at the street's kerb.
  line([30, 42.15], [30, 110], 7, 'lane', ['join', 'dead']),
  // The street carried on as a country road.
  line([130, 40], [159, 40], 7, 'country', ['join', 'out']),
  // A trail from the street's edge.
  line([110, 43.5], [110, 100], 2, 'trail', ['join', 'dead']),
  // A street across it.
  line([55, 5], [55, 75], 7, 'street', ['dead', 'dead']),
];
const plots = [
  // On the street, facing it (south), a driveway across the verge.
  { x: 10, z: 20, w: 15, d: 15 },
  // On the lane, facing it (west), a path across the verge.
  { x: 34, z: 60, w: 15, d: 15 },
  // Off any road.
  { x: 130, z: 90, w: 10, d: 10 },
];
const world: SurfaceWorld = {
  plots,
  meta: {
    roads,
    paths: [
      { x: 20, z: 35, w: 3, d: 1.5 },
      { x: 33.5, z: 66, w: 0.5, d: 1 },
    ],
    facing: [2, 1, 0],
    world: { landscape: { slope: 0.4 } },
  },
};

describe('built land on a slope', () => {
  const land = makeLand();
  const surface = builtSurfaceOf(world, land);

  test('every road within its grade, a trail on the land as it lies', () => {
    roads.forEach((r, i) => {
      const { s, y } = surface.profile(i);
      if (r.kind === 'trail') {
        // (A footpath: no grade, the land under it.)
        const ls = lengths(r.points);
        for (let k = 0; k < s.length; k++) expect(y[k]).toBeCloseTo(heightAt(land, ...pointAt(r.points, s[k], ls).p), 4);
        return;
      }
      expect(s.length).toBeGreaterThan(1);
      for (let k = 0; k + 1 < s.length; k++) expect(Math.abs(y[k + 1] - y[k]) / (s[k + 1] - s[k])).toBeLessThanOrEqual(GRADE[r.kind ?? 'street'] + 1e-3);
    });
    // (The land is steeper than a street may climb: the street doesn't just follow it.)
    const { s, y } = surface.profile(0);
    expect(Math.abs(y[s.length - 1] - y[0])).toBeLessThan(10 * 0.9);
  });

  test('a junction is one height over its arms', () => {
    const junctions = surface.junctionHeights;
    // The lane's T and the crossing street.
    expect(junctions.length).toBe(2);
    for (const j of junctions) {
      expect(j.arms.length).toBeGreaterThanOrEqual(2);
      for (const a of j.arms) {
        expect(a.to - a.from).toBeGreaterThan(4);
        for (let s = a.from; s <= a.to; s += 0.5) expect(surface.roadY(a.road, s)).toBeCloseTo(j.y, 4);
      }
    }
    // A join carries the street on into the country road at one height.
    expect(surface.roadY(2, 0)).toBeCloseTo(surface.roadY(0, 130), 4);
  });

  test("a lot's pad stands near its road's height where its way in meets it", () => {
    const found = surfacePlots(world);
    expect(found[0].entry?.road).toBe(0);
    expect(found[0].entry!.s).toBeCloseTo(21.5, 1);
    expect(found[0].front![0]).toBeCloseTo(10, 1);
    expect(found[0].front![1]).toBeCloseTo(25, 1);
    expect(found[1].entry?.road).toBe(1);
    expect(found[2].entry).toBeNull();
    /** The middle of the land under a lot. */
    const middle = (p: { x: number; z: number; w: number; d: number }) => {
      const under: number[] = [];
      // (Inside its edges.)
      for (let z = (Math.floor(p.z / LAND_CELL) + 1) * LAND_CELL; z < p.z + p.d; z += LAND_CELL) for (let x = (Math.floor(p.x / LAND_CELL) + 1) * LAND_CELL; x < p.x + p.w; x += LAND_CELL) under.push(land.h[(z / LAND_CELL) * land.cols + x / LAND_CELL] / 100);
      return (Math.min(...under) + Math.max(...under)) / 2;
    };
    found.forEach((p, i) => {
      if (!p.entry) return;
      // The entry is the road's height there; the pad within a short drive of it, and at the land's middle where it may be.
      expect(surface.entryY(i)).toBe(surface.roadY(p.entry.road, p.entry.s));
      expect(Math.abs(surface.padY(i) - surface.entryY(i))).toBeLessThanOrEqual(ENTRY_MAX + 1e-9);
      if (Math.abs(middle(p) - surface.entryY(i)) < ENTRY_MAX - 0.01) expect(surface.padY(i)).toBeCloseTo(middle(p), 4);
    });
    // Off any road: the middle of the land under it, its entry its pad.
    expect(surface.padY(2)).toBeCloseTo(middle(plots[2]), 6);
    expect(surface.entryY(2)).toBe(surface.padY(2));
    // The surface in a lot is its pad, its land graded to it.
    expect(surface.at(17, 27)).toBe(surface.padY(0));
    const graded = surface.graded();
    expect(graded[(26 / LAND_CELL) * land.cols + 16 / LAND_CELL]).toBe(Math.round(surface.padY(0) * 100));
  });

  test('lots side by side on a steep street step no more than a retaining edge', () => {
    // A street climbing at 8 %, the land with it; two lots on it, side by side, their entries 1.6 m apart.
    const slope = blankLand(120, 60, 'follow');
    for (let j = 0; j < slope.rows; j++) for (let i = 0; i < slope.cols; i++) slope.h[j * slope.cols + i] = Math.round(0.08 * i * LAND_CELL * 100);
    const street = [line([0, 20], [119, 20], 7, 'street', ['out', 'out'])];
    const lots = [
      { x: 20, z: 0, w: 20, d: 16, entry: { road: 0, s: 30 }, front: [20, 40] as [number, number] },
      { x: 40, z: 0, w: 20, d: 16, entry: { road: 0, s: 50 }, front: [40, 60] as [number, number] },
    ];
    const both = new BuiltSurface(slope, street, lots, [], 0.4);
    expect(Math.abs(both.entryY(1) - both.entryY(0))).toBeGreaterThan(RETAIN_MAX);
    expect(Math.abs(both.padY(1) - both.padY(0))).toBeLessThanOrEqual(RETAIN_MAX);
    for (const i of [0, 1]) expect(Math.abs(both.padY(i) - both.entryY(i))).toBeLessThanOrEqual(ENTRY_MAX + 1e-9);
  });

  test('the bridge clears its water', () => {
    const { s, y } = surface.profile(0);
    let over = 0;
    for (let k = 0; k < s.length; k++) {
      if (s[k] < CHANNEL[0] || s[k] > CHANNEL[1]) continue;
      over++;
      expect(y[k]).toBeGreaterThanOrEqual(WATER + BRIDGE_CLEAR - 1e-4);
    }
    expect(over).toBeGreaterThan(4);
    // A little higher in its middle (its floor there: the clearance and a hump of 1 cm a metre of its 22 m span).
    expect(surface.roadY(0, 77)).toBeGreaterThanOrEqual(WATER + BRIDGE_CLEAR + 0.22 - 1e-3);
    // The water under it is left as it was.
    const k = (40 / LAND_CELL) * land.cols + 76 / LAND_CELL;
    expect(surface.conformed[k]).toBe(land.h[k] / 100);
  });

  test('the land is graded between the cones, under a road a little below it', () => {
    const bank = 0.4;
    const k = (40 / LAND_CELL) * land.cols + 20 / LAND_CELL;
    expect(surface.conformed[k]).toBeCloseTo(surface.roadY(0, 20) - ROAD_SINK, 4);
    // Beside a lot, within a retaining edge of its pad; out from the street, no steeper than the bank from it.
    for (let x = 10; x <= 25; x += LAND_CELL) {
      const behind = surface.conformed[(18 / LAND_CELL) * land.cols + x / LAND_CELL];
      expect(Math.abs(behind - surface.padY(0))).toBeLessThanOrEqual(RETAIN_MAX + 1e-4);
    }
    for (let z = 46; z <= 56; z += LAND_CELL) {
      const at = surface.conformed[(z / LAND_CELL) * land.cols + 120 / LAND_CELL];
      expect(Math.abs(at - surface.roadY(0, 120))).toBeLessThanOrEqual(bank * (z - 40) + 1e-3);
    }
  });

  test("the land drawn between its samples stays under a road's edges, the land beside it higher or lower", () => {
    // (Just inside each edge of every way for cars, off its decks and their ends, where it is that road's alone and no
    // lot is within a sample (a pad is drawn as a slab of its own): the land there, between the samples round it, as
    // the terrain draws it, no higher than the road. Where the land beside rose from a sample just past the edge, it
    // stood over the road's last metre: a notch in its edges where a stroke raised the land beside it.)
    const nearLot = (x: number, z: number) => plots.some((p) => x > p.x - LAND_CELL && x < p.x + p.w + LAND_CELL && z > p.z - LAND_CELL && z < p.z + p.d + LAND_CELL);
    let worst = -Infinity;
    let checked = 0;
    roads.forEach((r, road) => {
      if (r.kind === 'trail') return;
      const ls = lengths(r.points);
      const decks = bridgeSpans(r);
      for (let s = 0; s <= ls[ls.length - 1]; s += 0.5) {
        if (decks.some(([a, b]) => s >= a - 2 && s <= b + 2)) continue;
        const { p, t } = pointAt(r.points, s, ls);
        for (const side of [-1, 1]) {
          const o = side * (r.width / 2 - 0.25);
          const [x, z] = [p[0] - t[1] * o, p[1] + t[0] * o];
          if (x < 0 || z < 0 || x > W - LAND_CELL || z > D - LAND_CELL || nearLot(x, z) || surface.roadAt(x, z)?.road !== road) continue;
          worst = Math.max(worst, surface.landAt(x, z) - surface.at(x, z));
          checked++;
        }
      }
    });
    expect(checked).toBeGreaterThan(500);
    expect(worst).toBeLessThanOrEqual(0.005);
  });

  test('at() has no jump along a road', () => {
    roads.forEach((r) => {
      if (r.kind === 'trail') return;
      const ls = lengths(r.points);
      let last: number | null = null;
      for (let s = 0; s <= ls[ls.length - 1]; s += 0.25) {
        const { p } = pointAt(r.points, s, ls);
        const y = surface.at(p[0], p[1]);
        if (last !== null) expect(Math.abs(y - last)).toBeLessThanOrEqual(GRADE[r.kind ?? 'street'] * 0.25 + 0.01);
        last = y;
      }
    });
  });

  test('update after an edit is the surface made afresh, its chunks moved on', () => {
    const edited = makeLand();
    const before = builtSurfaceOf(world, edited);
    const lane = before.profile(1).y.slice();
    const rect = { i0: 10, j0: 25, i1: 30, j1: 45 };
    for (let j = rect.j0; j < rect.j1; j++) for (let i = rect.i0; i < rect.i1; i++) edited.h[j * edited.cols + i] += 400;
    const change = before.update(rect);
    const fresh = builtSurfaceOf(world, edited);
    expect(Array.from(before.conformed)).toEqual(Array.from(fresh.conformed));
    roads.forEach((_, i) => expect(Array.from(before.profile(i).y)).toEqual(Array.from(fresh.profile(i).y)));
    plots.forEach((_, i) => expect(before.padY(i)).toBe(fresh.padY(i)));
    expect(Array.from(before.graded())).toEqual(Array.from(fresh.graded()));
    // The lane climbed with the land; its lot with it; the land round it changed.
    expect(change.roads).toContain(1);
    expect(Math.max(...Array.from(before.profile(1).y, (y, k) => y - lane[k]))).toBeGreaterThan(0.2);
    expect(change.plots).toContain(1);
    expect(change.rect).not.toBeNull();
    expect(before.version(chunkAt(30, 80))).toBe(1);
    expect(fresh.version(chunkAt(30, 80))).toBe(0);
    // Nothing changed: nothing moves.
    const again = before.update(rect);
    expect(again).toEqual({ roads: [], plots: [], rect: null });
    expect(before.version(chunkAt(30, 80))).toBe(1);
  });

  test("a road's list of bridges (the land's brushes') is solved as its own bridge is, the old one read as one", () => {
    const listed = roads.map((r, i) => (i === 0 ? { ...r, bridge: undefined, bridges: [r.bridge!] } : r));
    const other = builtSurfaceOf({ ...world, meta: { ...world.meta, roads: listed } }, land);
    expect(Array.from(other.profile(0).y)).toEqual(Array.from(surface.profile(0).y));
    expect(Array.from(other.conformed)).toEqual(Array.from(surface.conformed));
    // Two spans: each its own deck, the road between them on the land.
    expect(bridgeSpans({ ...roads[0], bridges: [[100, 110], [10, 5]] })).toEqual([[5, 10], [66, 88], [100, 110]]);
    expect(bridgeSpans({ ...roads[0], bridge: undefined, bridges: [[120, 200]] })).toEqual([[120, 130]]);
  });

  test('the land raised under a street is settled within a cut of it and a retaining edge of a lot, in a few passes', () => {
    const edited = makeLand();
    const s = builtSurfaceOf(world, edited);
    // A 12 m hill under the street west of the channel and the first lot's front.
    const rect = { i0: 5, j0: 12, i1: 20, j1: 24 };
    for (let j = rect.j0; j < rect.j1; j++) for (let i = rect.i0; i < rect.i1; i++) edited.h[j * edited.cols + i] += 1200;
    const before = edited.h.slice();
    const { rect: more, change } = s.settleCuts(rect);
    expect(more).not.toBeNull();
    expect(change.roads).toContain(0);
    // (Settled: solved again, the land everywhere within a cut or a fill of the land graded to what is built.)
    const fresh = builtSurfaceOf(world, edited);
    const off = (sf: BuiltSurface, l: Land, k: number) => (l.wl[k] !== NONE && l.wl[k] > l.h[k] ? 0 : Math.abs(l.h[k] / 100 - sf.conformed[k]));
    let worst = 0;
    for (let j = rect.j0 - 4; j < rect.j1 + 4; j++) for (let i = rect.i0 - 4; i < rect.i1 + 4; i++) worst = Math.max(worst, off(fresh, edited, j * edited.cols + i));
    expect(worst).toBeLessThanOrEqual(CUT_MAX + 0.02);
    // Only round the hill: the land far off as it was.
    const far = (40 / LAND_CELL) * edited.cols + 140 / LAND_CELL;
    expect(edited.h[far]).toBe(before[far]);
  });

  test('a surface made directly is the one a world gives', () => {
    const direct = new BuiltSurface(land, roads, surfacePlots(world), [], 0.4);
    expect(Array.from(direct.conformed)).toEqual(Array.from(surface.conformed));
  });
});
