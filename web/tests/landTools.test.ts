// The land palette's brush (`ui/land/landTools.ts`): each group keeps its own size, a river is held to 4–30 m,
// the size slider on a log scale (whole metres there and back, "one point" at 4 m and under, the keyboard's steps),
// the editor's brush is half the size across with the mountains' height; which tools have a strength or a height;
// the cursor's tone (red where the brush would do nothing); what the cursor tag says a stroke costs (free in
// Creative, the price, how much short, a refund); and where a stroke stops at the funds.
// `bun test tests/landTools.test.ts`.
import { describe, expect, test } from 'bun:test';
import { brushOf, costLabel, cursorOf, hasHeight, hasStrength, INNER, LAND_GROUPS, POINT, sizeAt, sizeOf, sizeRange, SIZES, sizeText, sliderAt, startBrush, stepSize, stopsAtFunds, toneOf, toolInfo } from '../src/ui/land/landTools';
import { falloff, TOOL_KIND } from '../src/game/landBrush';

describe('the brush in hand', () => {
  test('every tool is in one group, and its group is the kind of thing it does', () => {
    const tools = LAND_GROUPS.flatMap((g) => g.tools.map((t) => t.id));
    expect(new Set(tools).size).toBe(tools.length);
    expect([...tools].sort()).toEqual(Object.keys(TOOL_KIND).sort() as typeof tools);
    expect(LAND_GROUPS.map((g) => g.id)).toEqual(['shape', 'water', 'cover', 'mountains']);
    expect(toolInfo('lake').group).toBe('water');
    expect(toolInfo('level').group).toBe('mountains');
  });

  test('each group keeps its own size; the editor gets half of it as the radius', () => {
    let b = startBrush('raise');
    expect(sizeOf(b)).toBe(SIZES.shape.start);
    b = { ...b, sizes: { ...b.sizes, shape: 50 } };
    expect(brushOf(b)).toEqual({ tool: 'raise', radius: 25, strength: 0.5 });
    b = { ...b, tool: 'peak' };
    expect(sizeOf(b)).toBe(SIZES.mountains.start);
    expect(brushOf(b)).toEqual({ tool: 'peak', radius: SIZES.mountains.start / 2, strength: 0.5, height: b.height });
    b = { ...b, tool: 'lower' };
    expect(sizeOf(b)).toBe(50);
  });

  test('the ranges: shape 1–120, water 2–120, cover 4–120, mountains 60–400, a river 4–30', () => {
    const range = (t: string) => sizeRange(t as never);
    expect([range('raise'), range('lake'), range('woods'), range('peak'), range('river')].map(({ min, max }) => ({ min, max }))).toEqual([
      { min: 1, max: 120 },
      { min: 2, max: 120 },
      { min: 4, max: 120 },
      { min: 60, max: 400 },
      { min: 4, max: 30 },
    ]);
  });

  test('a river is 4 to 30 m wide whatever the water group says; sizes are held to their range', () => {
    const b = startBrush('river');
    expect(sizeOf({ ...b, sizes: { ...b.sizes, water: 120 } })).toBe(30);
    expect(sizeOf({ ...b, sizes: { ...b.sizes, water: 2 } })).toBe(4);
    expect(sizeOf({ ...b, tool: 'lake', sizes: { ...b.sizes, water: 2 } })).toBe(2);
    expect(sizeOf({ ...b, tool: 'raise', sizes: { ...b.sizes, shape: 0.2 } })).toBe(1);
    expect(sizeOf({ ...b, tool: 'lake', sizes: { ...b.sizes, water: 120 } })).toBe(120);
    expect(sizeOf({ ...b, tool: 'woods', sizes: { ...b.sizes, cover: 1000 } })).toBe(SIZES.cover.max);
    expect(brushOf({ ...b, strength: 3 }).strength).toBe(1);
  });

  test('strength for what works by degrees; height for the peak and the range', () => {
    expect(['raise', 'lower', 'smooth', 'flatten', 'lake', 'river', 'level'].every((t) => hasStrength(t as never))).toBe(true);
    expect(['dry', 'woods', 'meadow', 'fields', 'clear', 'peak', 'range'].some((t) => hasStrength(t as never))).toBe(false);
    expect(hasHeight('peak') && hasHeight('range') && !hasHeight('level')).toBe(true);
  });

  test('the cursor: its inner ring where the brush works at half strength, its tone by what it does, red where refused', () => {
    expect(falloff(INNER * 10, 10)).toBeCloseTo(0.5, 6);
    expect(cursorOf({ tool: 'woods', radius: 20, strength: 1 }, 5, 6, false)).toEqual({ x: 5, z: 6, radius: 20, inner: 20 * INNER, tone: 'cover' });
    expect(toneOf('raise', false)).toBe('up');
    expect(toneOf('lower', false)).toBe('down');
    expect(toneOf('river', false)).toBe('water');
    expect(toneOf('dry', false)).toBe('water');
    expect(toneOf('peak', false)).toBe('up');
    expect(toneOf('raise', true)).toBe('bad');
  });
});

describe('the size slider', () => {
  const ranges = [SIZES.shape, SIZES.water, SIZES.cover, SIZES.mountains, sizeRange('river')];

  test('on a log scale: its ends are the range, its middle the geometric middle, and every whole size there and back', () => {
    for (const range of ranges) {
      expect(sizeAt(0, range)).toBe(range.min);
      expect(sizeAt(1, range)).toBe(range.max);
      expect(sizeAt(0.5, range)).toBe(Math.round(Math.sqrt(range.min * range.max)));
      for (let size = range.min; size <= range.max; size++) {
        const t = sliderAt(size, range);
        expect(t).toBeGreaterThanOrEqual(0);
        expect(t).toBeLessThanOrEqual(1);
        expect(sizeAt(t, range)).toBe(size);
        // (And the slider's own steps, a thousandth, land on it too.)
        expect(sizeAt(Math.round(t * 1000) / 1000, range)).toBe(size);
      }
      // (Slider to size and back moves it no more than a whole metre's share.)
      for (let t = 0; t <= 1; t += 0.01) expect(Math.abs(sliderAt(sizeAt(t, range), range) - t)).toBeLessThanOrEqual(Math.log((range.min + 0.5) / range.min) / Math.log(range.max / range.min) + 1e-9);
    }
    // (Half the shape slider is under 11 m.)
    expect(sizeAt(0.5, SIZES.shape)).toBe(11);
  });

  test('the size is written "one point" at 4 m and under, whole metres above', () => {
    expect(POINT).toBe(4);
    expect([1, 2, 4].map(sizeText)).toEqual(['one point', 'one point', 'one point']);
    expect([5, 30, 120].map(sizeText)).toEqual(['5 m', '30 m', '120 m']);
  });

  test("the keyboard's steps: about a tenth, at least a metre, held to the range", () => {
    expect(stepSize(1, 1, SIZES.shape)).toBe(2);
    expect(stepSize(2, -1, SIZES.shape)).toBe(1);
    expect(stepSize(1, -1, SIZES.shape)).toBe(1);
    expect(stepSize(50, 1, SIZES.shape)).toBe(55);
    expect(stepSize(55, -1, SIZES.shape)).toBe(50);
    expect(stepSize(118, 1, SIZES.shape)).toBe(120);
    // (From the bottom to the top in a few dozen presses.)
    let [size, n] = [1, 0];
    while (size < 120) [size, n] = [stepSize(size, 1, SIZES.shape), n + 1];
    expect(n).toBeLessThan(60);
  });

  test("a point brush's cursor sits on the sample it changes, a ring as wide as it; a cover tool's on its cell", () => {
    expect(cursorOf({ tool: 'raise', radius: 0.5, strength: 1 }, 10.9, 13.2, false)).toEqual({ x: 10, z: 14, radius: 1, inner: 0, tone: 'up' });
    expect(cursorOf({ tool: 'lake', radius: 1, strength: 1 }, 3.1, 2.9, true)).toEqual({ x: 4, z: 2, radius: 1, inner: 0, tone: 'bad' });
    expect(cursorOf({ tool: 'woods', radius: 2, strength: 1 }, 7.9, 8.1, false)).toEqual({ x: 6, z: 10, radius: 2, inner: 0, tone: 'cover' });
    // (Over 4 m: the pointer itself.)
    expect(cursorOf({ tool: 'raise', radius: 2.5, strength: 1 }, 10.9, 13.2, false).x).toBe(10.9);
  });
});

describe('what a stroke costs, at the cursor', () => {
  const money = (n: number) => `$${n.toLocaleString('en-US')}`;
  test('free in Creative; the price in Living, red and how much short past the funds; a refund back', () => {
    expect(costLabel(5000, true, 0, money)).toEqual({ text: 'Free', bad: false });
    expect(costLabel(1234, false, 5000, money)).toEqual({ text: '$1,234', bad: false });
    expect(costLabel(6000, false, 5000, money)).toEqual({ text: '$1,000 short', bad: true });
    expect(costLabel(-300, false, 0, money)).toEqual({ text: '$300 back', bad: false });
    expect(costLabel(0, false, 0, money)).toEqual({ text: '$0', bad: false });
  });

  test('a stroke stops before a stamp at its going rate would pass the funds (never in Creative)', () => {
    expect(stopsAtFunds(900, 9, 1000, false)).toBe(false);
    expect(stopsAtFunds(950, 9, 1000, false)).toBe(true);
    expect(stopsAtFunds(950, 9, 1000, true)).toBe(false);
    expect(stopsAtFunds(0, 0, 0, false)).toBe(false);
  });
});

test('a few stamps ahead, for a frame that stamps several times', () => {
  expect(stopsAtFunds(800, 8, 1000, false, 1)).toBe(false);
  expect(stopsAtFunds(800, 8, 1000, false, 4)).toBe(true);
});
