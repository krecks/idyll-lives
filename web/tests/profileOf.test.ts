// The street tool's live profile of a road being drawn (`profileOf`, `render/shared/builtSurface.ts`): the same heights
// the surface's solver gives a road of that kind on its own (so the tint while drawing rarely disagrees with the
// settle), within its kind's grade, held at its ends where asked, its cut and fill measured against the land as it
// lies (none over water, none along a trail), and a road up a hill steeper than its grade cut past `CUT_MAX`.
// `bun test tests/profileOf.test.ts`.
import { describe, expect, test } from 'bun:test';
import type { RoadKind, RoadPlan } from '../src/core/protocol';
import { blankLand, heightAt, LAND_CELL, type Land } from '../src/game/land';
import { BuiltSurface, CUT_MAX, GRADE, profileOf } from '../src/render/shared/builtSurface';

const [W, D] = [400, 200];
/** The pond the second land has (x, m) and its water's level (m). */
const POND: [number, number] = [180, 220];

/** A land of rolling hills rising along x (a ridge at x 300 too steep for a street), with or without a pond across it. */
function makeLand(pond = false): Land {
  const land = blankLand(W, D, 'follow');
  for (let j = 0; j < land.rows; j++) {
    for (let i = 0; i < land.cols; i++) {
      const [x, z] = [i * LAND_CELL, j * LAND_CELL];
      const k = j * land.cols + i;
      if (pond && x >= POND[0] && x <= POND[1]) {
        land.wl[k] = 0;
        land.h[k] = -200;
        continue;
      }
      const ridge = 30 * Math.exp(-(((x - 300) / 25) ** 2));
      land.h[k] = Math.round((6 * Math.sin(x / 30) + 0.04 * x + 1.5 * Math.sin(z / 11) + ridge) * 100);
    }
  }
  return land;
}

/** A polyline from `a` to `b`, a point every 5 m, bending a little. */
function line(a: [number, number], b: [number, number]): [number, number][] {
  const n = Math.max(1, Math.round(Math.hypot(b[0] - a[0], b[1] - a[1]) / 5));
  return Array.from({ length: n + 1 }, (_, k): [number, number] => [a[0] + ((b[0] - a[0]) * k) / n, a[1] + ((b[1] - a[1]) * k) / n + 6 * Math.sin((k / n) * Math.PI)]);
}

describe('profileOf', () => {
  const land = makeLand();
  const points = line([10, 100], [250, 120]);

  for (const kind of ['street', 'lane', 'country'] as RoadKind[]) {
    test(`a ${kind} on its own: as the surface solves it`, () => {
      const p = profileOf(points, kind, land);
      const road: RoadPlan = { points, width: 7, kind, ends: ['dead', 'dead'] };
      const solved = new BuiltSurface(land, [road], []).profile(0);
      expect(p.s.length).toBe(solved.s.length);
      for (let i = 0; i < p.s.length; i++) {
        expect(p.s[i]).toBeCloseTo(solved.s[i], 4);
        expect(Math.abs(p.y[i] - solved.y[i])).toBeLessThan(0.01);
      }
    });
  }

  test('within its grade, its samples where they stand, its cut against the land', () => {
    const p = profileOf(points, 'street', land);
    for (let i = 0; i + 1 < p.s.length; i++) expect(Math.abs(p.y[i + 1] - p.y[i]) / (p.s[i + 1] - p.s[i])).toBeLessThanOrEqual(GRADE.street + 1e-3);
    expect(p.x[0]).toBeCloseTo(10, 4);
    expect(p.z[p.z.length - 1]).toBeCloseTo(120, 4);
    for (let i = 0; i < p.s.length; i++) expect(p.cut[i]).toBeCloseTo(p.y[i] - heightAt(land, p.x[i], p.z[i]), 4);
  });

  test('held at its ends where it joins a road', () => {
    const free = profileOf(points, 'street', land);
    const p = profileOf(points, 'street', land, { start: free.y[0] + 1.5, end: free.y[free.y.length - 1] - 1 });
    expect(p.y[0]).toBeCloseTo(free.y[0] + 1.5, 4);
    expect(p.y[p.y.length - 1]).toBeCloseTo(free.y[free.y.length - 1] - 1, 4);
    for (let i = 0; i + 1 < p.s.length; i++) expect(Math.abs(p.y[i + 1] - p.y[i]) / (p.s[i + 1] - p.s[i])).toBeLessThanOrEqual(GRADE.street + 1e-3);
  });

  test('over a ridge too steep for a street: cut past CUT_MAX, a country road less', () => {
    const over = line([240, 60], [360, 60]);
    const street = profileOf(over, 'street', land);
    const country = profileOf(over, 'country', land);
    const most = (cut: number[]) => Math.max(...cut.map(Math.abs));
    expect(most(street.cut)).toBeGreaterThan(CUT_MAX);
    expect(most(country.cut)).toBeLessThan(most(street.cut));
    // (Away from the ridge the same street keeps to the land.)
    const flat = profileOf(line([20, 60], [140, 60]), 'street', land);
    expect(most(flat.cut)).toBeLessThan(CUT_MAX);
  });

  test('a trail lies on the land; over water nothing is cut', () => {
    const trail = profileOf(points, 'trail', land);
    for (let i = 0; i < trail.s.length; i++) {
      expect(trail.y[i]).toBeCloseTo(heightAt(land, trail.x[i], trail.z[i]), 4);
      expect(trail.cut[i]).toBe(0);
    }
    const wet = makeLand(true);
    const across = profileOf(line([100, 150], [300, 150]), 'country', wet);
    const inPond = across.x.map((x, i) => [x, i] as const).filter(([x]) => x > POND[0] + LAND_CELL && x < POND[1] - LAND_CELL);
    expect(inPond.length).toBeGreaterThan(5);
    for (const [, i] of inPond) expect(across.cut[i]).toBe(0);
  });

  test('cheap enough for every pointer move', () => {
    const long = line([5, 20], [395, 180]);
    const t0 = performance.now();
    for (let k = 0; k < 20; k++) profileOf(long, 'street', land);
    expect((performance.now() - t0) / 20).toBeLessThan(5);
  });
});
