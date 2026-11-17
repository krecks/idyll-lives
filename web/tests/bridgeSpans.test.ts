// Bridges over a wet mask (`game/bridges.ts`), as the street tool and the game's own bridges lay
// them: where a line crosses the water, whether it may be bridged there (45 m of deck at most,
// straight, no junction or lot on it), and the road split into the road before, a straight country
// bridge with one deck, and the road after, the deck walked dry. A drawn street's (`crossWater`):
// 70 m of deck, the straight through the water's middle and turned squarer where the line won't do,
// its deck carried on over a river's dip. `bun test`.
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { BRIDGE_IN, BRIDGE_SPAN, bridgeable, bridgeFits, chordOf, crossWater, DECK_PAST, deckSpanOf, DRAWN_SPAN, wetSpans, withBridge, withoutDeck } from '../src/game/bridges';
import { blankLand, LAND_CELL } from '../src/game/land';
import { deckOf } from '../src/game/macro';
import { lengths, type Pt } from '../src/game/roads';

const W = 200;
const D = 300;

/** A river across the world between z `z0` and `z1`, and dry land either side. */
function river(z0: number, z1: number, gaps: [number, number][] = []): Uint8Array {
  const wet = new Uint8Array(W * D);
  for (let z = z0; z < z1; z++) for (let x = 0; x < W; x++) if (!gaps.some(([a, b]) => z >= a && z < b)) wet[z * W + x] = 1;
  return wet;
}
const line = (x0: number, z0: number, x1: number, z1: number): Pt[] => {
  const n = Math.max(1, Math.ceil(Math.hypot(x1 - x0, z1 - z0)));
  return Array.from({ length: n + 1 }, (_, i): Pt => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);
};
const road = (points: Pt[]): RoadPlan => ({ points, width: 7, ends: ['join', 'join'], kind: 'street', area: 0, name: 'Mill Lane' });

describe('wet spans', () => {
  test('where a line goes into the water and comes out, along it', () => {
    const spans = wetSpans(line(100, 120, 100, 280), river(200, 230), W, D);
    expect(spans.length).toBe(1);
    expect(spans[0][0]).toBeCloseTo(80, 0);
    expect(spans[0][1]).toBeCloseTo(109.75, 1);
  });

  test('two rivers two spans; a sandbank a metre wide no gap', () => {
    expect(wetSpans(line(100, 20, 100, 280), (() => {
      const wet = river(200, 230);
      for (let z = 60; z < 70; z++) for (let x = 0; x < W; x++) wet[z * W + x] = 1;
      return wet;
    })(), W, D).length).toBe(2);
    expect(wetSpans(line(100, 120, 100, 280), river(200, 230, [[214, 215]]), W, D).length).toBe(1);
  });

  test('none off the world or on dry land', () => {
    expect(wetSpans(line(-50, 210, -10, 210), river(200, 230), W, D)).toEqual([]);
    expect(wetSpans(line(10, 10, 190, 10), river(200, 230), W, D)).toEqual([]);
  });
});

describe('whether a crossing may be bridged', () => {
  const across = line(100, 120, 100, 280);
  const span = (wet: Uint8Array) => wetSpans(across, wet, W, D)[0];

  test('a 30 m river, straight across, may', () => {
    expect(bridgeable(across, span(river(200, 230)), [], [])).toBeNull();
  });

  test(`a 60 m one may not: a deck is ${BRIDGE_SPAN} m at most`, () => {
    expect(bridgeable(across, span(river(200, 260)), [], [])).toContain('the water is 60 m across');
    // (41 m of water and its 2 m either side are the most.)
    expect(bridgeable(across, span(river(200, 200 + BRIDGE_SPAN - 2 * DECK_PAST)), [], [])).toBeNull();
  });

  test('a road meeting it there, or a lot standing there, may not', () => {
    const lane: RoadPlan = { points: line(60, 195, 96, 195), width: 7, ends: ['dead', 'join'], name: 'Kiln Lane' };
    expect(bridgeable(across, span(river(200, 230)), [lane], [])).toBe('Kiln Lane meets it where it would be a bridge');
    // Further back than the straight approach: it may.
    expect(bridgeable(across, span(river(200, 230)), [{ ...lane, points: line(60, 170, 96, 170) }], [])).toBeNull();
    expect(bridgeable(across, span(river(200, 230)), [], [{ x: 102, z: 236, w: 14, d: 18 }])).toBe('a lot stands where the bridge would be');
    // Its own points are not another road.
    expect(bridgeable(across, span(river(200, 230)), [road(across)].map((r) => ({ ...r, points: across })), [])).toBeNull();
  });

  test('a bend where it crosses may not', () => {
    const bent: Pt[] = [...line(100, 120, 100, 205), ...line(100, 205, 120, 225).slice(1), ...line(120, 225, 120, 280).slice(1)];
    expect(bridgeable(bent, wetSpans(bent, river(200, 230), W, D)[0], [], [])).toContain('it bends where it crosses the water');
  });
});

describe('a road bridged', () => {
  test('split into the road before, a straight country bridge with its deck, and the road after', () => {
    const wet = river(200, 230);
    // Slightly aslant, to be made straight.
    const pts: Pt[] = [...line(100, 120, 101, 210), ...line(101, 210, 100, 280).slice(1)];
    const roads = [{ points: line(0, 100, W, 100), width: 7, ends: ['out', 'out'] as RoadPlan['ends'] }, road(pts)];
    const span = wetSpans(pts, wet, W, D)[0];
    const { roads: out, walk } = withBridge(roads, 1, span, wet, W, D, { bridge: 'Mill Bridge', after: 'Brook Lane' });
    expect(out.length).toBe(4);
    const [before, bridge, after] = [out[1], out[2], out[3]];
    expect(before.name).toBe('Mill Lane');
    expect(before.ends).toEqual(['join', 'join']);
    expect(bridge).toMatchObject({ kind: 'country', name: 'Mill Bridge', ends: ['join', 'join'] });
    expect(bridge.area).toBeUndefined();
    expect(after).toMatchObject({ name: 'Brook Lane', ends: ['join', 'join'], kind: 'street', area: 0 });
    // Straight, from 12 m before the bank to 12 m after it, joined end to end.
    const [a, b] = [bridge.points[0], bridge.points.at(-1)!];
    for (const p of bridge.points) expect(Math.abs((p[0] - a[0]) * (b[1] - a[1]) - (p[1] - a[1]) * (b[0] - a[0])) / Math.hypot(b[0] - a[0], b[1] - a[1])).toBeLessThan(0.02);
    expect(a[1]).toBeCloseTo(200 - BRIDGE_IN, 0);
    expect(b[1]).toBeCloseTo(229.75 + BRIDGE_IN, 0);
    expect(before.points.at(-1)).toEqual(a);
    expect(after.points[0]).toEqual(b);
    // Its deck: the water and 2 m either side.
    const [s0, s1] = bridge.bridge!;
    expect(s0).toBeCloseTo(BRIDGE_IN - DECK_PAST, 0);
    expect(s1 - s0).toBeCloseTo(30 + 2 * DECK_PAST, 0);
    expect(s1).toBeLessThanOrEqual(lengths(bridge.points).at(-1)!);
    // Walked dry on the deck, wet beside it.
    expect(walk[215 * W + 100]).toBe(0);
    expect(walk[215 * W + 103]).toBe(0);
    expect(walk[215 * W + 110]).toBe(1);
    expect(wet[215 * W + 100]).toBe(1);
    // Taken away again, the water is back.
    const back = withoutDeck(bridge, walk, (_, z) => z >= 200 && z < 230, W, D);
    expect([...back]).toEqual([...wet]);
  });

  test('a road beginning by the bank: the bridge begins it', () => {
    const wet = river(200, 230);
    const pts = line(100, 195, 100, 280);
    const { roads: out } = withBridge([road(pts)], 0, wetSpans(pts, wet, W, D)[0], wet, W, D);
    expect(out.length).toBe(2);
    expect(out[0].bridge).toBeDefined();
    expect(out[0].points[0]).toEqual([100, 195]);
    expect(out[0].name).toBe('Mill Bridge');
    expect(out[1].name).toBe('Mill Lane');
  });
});

describe("a drawn street's bridges (`crossWater`)", () => {
  /** A river `w` m wide across the world, its banks at 45° (z = x + `z0` to z = x + `z0` + w·√2). */
  const diagonal = (z0: number, w: number) => {
    const wet = new Uint8Array(W * D);
    for (let z = 0; z < D; z++) for (let x = 0; x < W; x++) if (z - x >= z0 && z - x < z0 + w * Math.SQRT2) wet[z * W + x] = 1;
    return wet;
  };

  test('bridgeable takes a longer deck and a looser bend when asked; as it was otherwise', () => {
    const across = line(100, 120, 100, 280);
    const span = wetSpans(across, river(200, 260), W, D)[0];
    expect(bridgeable(across, span, [], [])).toContain('the water is 60 m across');
    expect(bridgeable(across, span, [], [], { most: DRAWN_SPAN })).toBeNull();
    expect(bridgeable(across, wetSpans(across, river(200, 280), W, D)[0], [], [], { most: DRAWN_SPAN })).toContain(`spans ${DRAWN_SPAN - 2 * DECK_PAST} m of water at most`);
    const bent: Pt[] = [...line(100, 120, 100, 205), ...line(100, 205, 120, 225).slice(1), ...line(120, 225, 120, 280).slice(1)];
    expect(bridgeable(bent, wetSpans(bent, river(200, 230), W, D)[0], [], [], { bend: 20 })).toBeNull();
  });

  test('deckSpanOf finds the deck deckOf does, drying nothing', () => {
    const wet = river(200, 230);
    for (const pts of [line(100, 188, 100, 242), line(90, 188, 112, 243), line(100, 250, 100, 300)]) {
      const r: RoadPlan = { ...road(pts), points: pts };
      const walk = wet.slice();
      expect(deckSpanOf(pts, wet, W, D)).toEqual(deckOf(r, wet, walk, W, D));
    }
    expect(deckSpanOf(line(100, 250, 100, 300), wet, W, D)).toEqual([0, 0]);
  });

  test('the fits tried: the chord first, then through the middle of the water, then turned either way', () => {
    const wet = diagonal(100, 50);
    const pts = line(100, 60, 100, 340);
    const span = wetSpans(pts, wet, W, D)[0];
    const fits = bridgeFits(pts, span, wet, W, D);
    expect(fits[0]).toMatchObject(chordOf(pts, span));
    expect(fits.length).toBeGreaterThan(5);
    // (Each straight from 12 m before the water to 12 m after it, through the middle of the water on the line.)
    for (const f of fits.slice(1)) {
      const deck = f.deck!;
      expect(deck[1]).toBeGreaterThan(deck[0]);
      expect(f.a).toBeGreaterThan(0);
      expect(f.b).toBeLessThan(lengths(pts).at(-1)!);
    }
    // (Turned squarer, the deck is shorter: about 50 m of water and 2 m either side when square.)
    const shortest = Math.min(...fits.map((f) => f.deck![1] - f.deck![0]));
    expect(shortest).toBeLessThan(60);
    expect(fits[0].deck![1] - fits[0].deck![0]).toBeGreaterThan(DRAWN_SPAN);
  });

  test('a crossing too long at its slant is squared; one too wide either way refused with the chord’s reason', () => {
    const pts = line(100, 60, 100, 340);
    const wet = diagonal(100, 50);
    const crossed = crossWater(pts, ['join', 'join'], wetSpans(pts, wet, W, D)[0], { wet, W, D, roads: [], plots: [] });
    if (typeof crossed === 'string') throw new Error(crossed);
    expect(crossed.fit.deck![1] - crossed.fit.deck![0]).toBeLessThanOrEqual(DRAWN_SPAN);
    const [a, b] = [crossed.bridge[0], crossed.bridge.at(-1)!];
    expect(Math.abs(Math.atan2(b[0] - a[0], b[1] - a[1]))).toBeGreaterThan(0.1);
    // (The road onto it and off it joined end to end, nothing of it over the water.)
    expect(crossed.before!.at(-1)).toEqual(a);
    expect(crossed.after![0]).toEqual(b);
    expect(wetSpans(crossed.before!, wet, W, D)).toEqual([]);
    expect(wetSpans(crossed.after!, wet, W, D)).toEqual([]);
    const wide = river(100, 180);
    expect(crossWater(pts, ['join', 'join'], wetSpans(pts, wide, W, D)[0], { wet: wide, W, D, roads: [], plots: [] })).toContain('the water is 80 m across');
    // (A straight drawn square across is the chord, as the generator's.)
    const square = crossWater(pts, ['join', 'join'], wetSpans(pts, river(200, 230), W, D)[0], { wet: river(200, 230), W, D, roads: [], plots: [] });
    if (typeof square === 'string') throw new Error(square);
    expect(square.fit).toMatchObject(chordOf(pts, wetSpans(pts, river(200, 230), W, D)[0]));
  });

  test('over a dip the deck reaches on to where the land is within 3 m of the line between the bridge’s ends', () => {
    const wet = river(200, 230);
    const land = blankLand(W, D);
    // (The land 10 m up, falling to 4 m within 6 m of the water; the water at 3 m.)
    for (let j = 0; j < land.rows; j++) {
      for (let i = 0; i < land.cols; i++) {
        const z = j * LAND_CELL;
        const off = z < 200 ? 200 - z : z >= 230 ? z - 229 : 0;
        const k = j * land.cols + i;
        if (!off) [land.h[k], land.wl[k]] = [100, 300];
        else land.h[k] = off <= 6 ? 400 : 1000;
      }
    }
    const pts = line(100, 120, 100, 280);
    const span = wetSpans(pts, wet, W, D)[0];
    const flat = crossWater(pts, ['join', 'join'], span, { wet, W, D, roads: [], plots: [] });
    const dipped = crossWater(pts, ['join', 'join'], span, { wet, W, D, roads: [], plots: [], land });
    if (typeof flat === 'string' || typeof dipped === 'string') throw new Error('not bridged');
    const [f, d] = [flat.fit.deck!, dipped.fit.deck!];
    expect(f[1] - f[0]).toBeCloseTo(33, 0);
    expect(d[1] - d[0]).toBeGreaterThan(f[1] - f[0] + 6);
    // (Its straight still runs on 10 m before it either end.)
    expect(d[0]).toBeGreaterThanOrEqual(BRIDGE_IN - DECK_PAST - 0.01);
    expect(lengths(dipped.bridge).at(-1)! - d[1]).toBeGreaterThanOrEqual(BRIDGE_IN - DECK_PAST - 0.01);
  });

  /** How far (m) the middle of a crossing's deck is from the middle of the water under it, along its bridge. */
  const offCentre = (crossed: ReturnType<typeof crossWater>, wet: Uint8Array) => {
    if (typeof crossed === 'string') throw new Error(crossed);
    const [w0, w1] = wetSpans(crossed.bridge, wet, W, D)[0];
    const [d0, d1] = crossed.fit.deck!;
    return Math.abs((d0 + d1) / 2 - (w0 + w1) / 2);
  };

  test('a river under a hill, a low bank in its dip on one side and a meadow at its level on the other: the deck centred on the water', () => {
    const wet = river(200, 230);
    const land = blankLand(W, D);
    // (The water at 3 m; north of it a meadow 0.4 m over it; south a shelf 6 m wide just over it, then the hill's
    // side rising to 10 m over 4 m.)
    for (let j = 0; j < land.rows; j++) {
      for (let i = 0; i < land.cols; i++) {
        const z = j * LAND_CELL;
        const k = j * land.cols + i;
        if (z >= 200 && z < 230) [land.h[k], land.wl[k]] = [100, 300];
        else if (z < 200) land.h[k] = 340;
        else land.h[k] = Math.round(325 + Math.max(0, Math.min(1, (z - 236) / 4)) * 675);
      }
    }
    const pts = line(100, 120, 100, 280);
    const span = wetSpans(pts, wet, W, D)[0];
    const crossed = crossWater(pts, ['join', 'join'], span, { wet, W, D, roads: [], plots: [], land });
    if (typeof crossed === 'string') throw new Error(crossed);
    const deck = crossed.fit.deck!;
    const z0 = crossed.bridge[0][1];
    // Over the shelf and onto the hill (past z 240), as far over the meadow; no longer than a drawn deck may be.
    expect(z0 + deck[1]).toBeGreaterThan(240);
    expect(offCentre(crossed, wet)).toBeLessThan(2);
    expect(deck[1] - deck[0]).toBeLessThanOrEqual(DRAWN_SPAN);
  });

  test('on flat land, and under a high bank on one side with the other low, the deck stays where the water is', () => {
    const wet = river(200, 230);
    const pts = line(100, 120, 100, 280);
    const span = wetSpans(pts, wet, W, D)[0];
    for (const north of [340, 340, 700]) {
      const land = blankLand(W, D);
      for (let j = 0; j < land.rows; j++) {
        for (let i = 0; i < land.cols; i++) {
          const z = j * LAND_CELL;
          const k = j * land.cols + i;
          if (z >= 200 && z < 230) [land.h[k], land.wl[k]] = [100, 300];
          else land.h[k] = z < 200 ? north : 340;
        }
      }
      const crossed = crossWater(pts, ['join', 'join'], span, { wet, W, D, roads: [], plots: [], land });
      if (typeof crossed === 'string') throw new Error(crossed);
      expect(offCentre(crossed, wet)).toBeLessThan(2);
      expect(crossed.fit.deck![1] - crossed.fit.deck![0]).toBeLessThan(30 + 2 * DECK_PAST + 2);
    }
  });
});
