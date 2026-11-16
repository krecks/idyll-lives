// The land palette's brush (`ui/land/landTools.ts`): each group keeps its own size, a river is held to 6–30 m,
// the editor's brush is half the size across with the mountains' height; which tools have a strength or a height;
// the cursor's tone (red where the brush would do nothing); what the cursor tag says a stroke costs (free in
// Creative, the price, how much short, a refund); and where a stroke stops at the funds.
// `bun test tests/landTools.test.ts`.
import { describe, expect, test } from 'bun:test';
import { brushOf, costLabel, cursorOf, hasHeight, hasStrength, INNER, LAND_GROUPS, sizeOf, SIZES, startBrush, stopsAtFunds, toneOf, toolInfo } from '../src/ui/land/landTools';
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

  test('a river is 6 to 30 m wide whatever the water group says; sizes are held to their range', () => {
    const b = startBrush('river');
    expect(sizeOf({ ...b, sizes: { ...b.sizes, water: 120 } })).toBe(30);
    expect(sizeOf({ ...b, sizes: { ...b.sizes, water: 2 } })).toBe(6);
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
