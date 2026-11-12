// The render resolution (`render/shared/resolution.ts`): 720p, 1080p, 1440p or 2160p lines of pixels,
// never more than the canvas has.
import { describe, expect, test } from 'bun:test';
import { DEFAULT_RESOLUTION, isResolution, renderScale } from '../src/render/shared/resolution';

describe('the render resolution', () => {
  test('1080p is the default', () => {
    expect(DEFAULT_RESOLUTION).toBe(1080);
  });

  test('a canvas taller than the setting is drawn at its lines, scaled up', () => {
    // A Retina window 1600 device pixels tall.
    expect(renderScale(1600, 1080)).toBeCloseTo(0.675);
    expect(renderScale(1600, 720)).toBeCloseTo(0.45);
    expect(renderScale(1600, 1440)).toBeCloseTo(0.9);
    // A 4K screen.
    expect(renderScale(2160, 1080)).toBe(0.5);
    expect(renderScale(2160, 2160)).toBe(1);
  });

  test('never more than the canvas has', () => {
    expect(renderScale(1080, 1440)).toBe(1);
    expect(renderScale(1080, 2160)).toBe(1);
    expect(renderScale(700, 1080)).toBe(1);
  });

  test('no canvas yet, or a setting that is none of them, draws as 1080p would', () => {
    expect(renderScale(0, 1080)).toBe(1);
    expect(renderScale(2160, 0.8 as never)).toBe(0.5);
  });

  test('only the four resolutions are settings', () => {
    expect([720, 1080, 1440, 2160].every(isResolution)).toBe(true);
    expect(isResolution(1)).toBe(false);
    expect(isResolution('auto')).toBe(false);
    expect(isResolution(900)).toBe(false);
  });
});
