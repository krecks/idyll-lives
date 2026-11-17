// Bridges over a wet mask (`game/bridges.ts`), as the street tool and the game's own bridges lay
// them: where a line crosses the water, whether it may be bridged there (45 m of deck at most,
// straight, no junction or lot on it), and the road split into the road before, a straight country
// bridge with one deck, and the road after, the deck walked dry. `bun test`.
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { BRIDGE_IN, BRIDGE_SPAN, bridgeable, DECK_PAST, wetSpans, withBridge, withoutDeck } from '../src/game/bridges';
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
