/**
 * The parked cars' paint copy by copy (`pc/cars.ts`: one draw per body and age, each copy tinted to its car's paint):
 * a body painted in `TINT_PAINT` and tinted where it is white is the body painted in the car's paint as its age
 * shows it, byte for byte, for every body, age and paint (so nothing else on a car is pure white, and an old car's
 * fade is all in its tint).
 */
import { describe, expect, test } from 'bun:test';
import { CAR_PAINT, CARS, carPaintColour, paintCar, TINT_PAINT, type CarAge } from '../src/render/shared/cars';
import { Geo } from '../src/render/shared/geometry';
import { Painter } from '../src/render/shared/street';

/** A parked car's body as `pc/cars.ts` draws it (shut, without its wheels). */
function body(k: number, paint: string, age: CarAge) {
  const geo = new Geo();
  paintCar(new Painter(geo).at(0, 0), CARS[k], paint, false, false, age);
  return geo.meshData({ colors: true });
}

describe('the parked cars’ paint', () => {
  test('an old car’s faded, the others’ as they are; the tint paint never fades', () => {
    expect(carPaintColour('#1E2328', 1)).toBe('#1E2328');
    expect(carPaintColour('#1E2328', 2)).toBe('#1E2328');
    expect(carPaintColour('#1E2328', 0)).not.toBe('#1E2328');
    for (const age of [0, 1, 2] as const) expect(carPaintColour(TINT_PAINT, age)).toBe(TINT_PAINT);
  });

  test('tinted where white, every body at every age in every paint is the body painted so', () => {
    for (let k = 0; k < CARS.length; k++) {
      for (const age of [0, 1, 2] as const) {
        const white = body(k, TINT_PAINT, age);
        const colors = white.colors!;
        let tinted = 0;
        for (const paint of CAR_PAINT) {
          const want = body(k, paint, age);
          expect(white.positions).toEqual(want.positions);
          const v = parseInt(carPaintColour(paint, age).slice(1), 16);
          const tint = [(v >> 16) & 255, (v >> 8) & 255, v & 255];
          const got = new Uint8Array(colors);
          tinted = 0;
          for (let i = 0; i < got.length; i += 4) {
            if (got[i] !== 255 || got[i + 1] !== 255 || got[i + 2] !== 255) continue;
            got.set(tint, i);
            tinted++;
          }
          expect(Array.from(got)).toEqual(Array.from(want.colors!));
        }
        // (Its paint is there to be tinted.)
        expect(tinted).toBeGreaterThan(0);
      }
    }
  });
});
