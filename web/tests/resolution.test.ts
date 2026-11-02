// The render resolution (`render/shared/resolution.ts`): `auto` draws up to a budget of pixels (the preset's
// share of 1920×1080), a number that share of the screen along each side.
import { describe, expect, test } from 'bun:test';
import { PIXEL_BUDGET, renderScale } from '../src/render/shared/resolution';
import { QUALITY } from '../src/render/quality';

describe('the render resolution', () => {
  test('auto draws a 1080p screen whole and a 4K one at the budget', () => {
    expect(renderScale(1920, 1080, 'auto')).toBe(1);
    expect(renderScale(3840, 2160, 'auto')).toBeCloseTo(0.5);
    expect(renderScale(1280, 720, 'auto')).toBe(1);
  });

  test("Low's budget is three quarters of 1080p along each side; Medium's and High's the whole", () => {
    const budget = (p: keyof typeof QUALITY) => PIXEL_BUDGET * QUALITY[p].pixelBudget;
    expect(renderScale(1920, 1080, 'auto', budget('low'))).toBeCloseTo(0.75);
    expect(renderScale(3840, 2160, 'auto', budget('low'))).toBeCloseTo(0.375);
    expect(renderScale(1920, 1080, 'auto', budget('medium'))).toBe(1);
    expect(renderScale(1920, 1080, 'auto', budget('high'))).toBe(1);
  });

  test('a number is that share, within 0.5..1, whatever the budget', () => {
    expect(renderScale(1920, 1080, 0.8, PIXEL_BUDGET * 0.5)).toBe(0.8);
    expect(renderScale(1920, 1080, 0.2)).toBe(0.5);
    expect(renderScale(1920, 1080, 2)).toBe(1);
  });
});
