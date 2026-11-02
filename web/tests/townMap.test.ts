/**
 * The town map's view (`ui/mapView.ts`): a point on screen to the ground under it when the map
 * fits a box of another shape (the "Bigger" map), and the place pill's words (`ui/place.ts`).
 */
import { describe, expect, test } from 'bun:test';
import { mapScale, screenToMap } from '../src/ui/mapView';
import { placeAt, tripText } from '../src/ui/place';

describe('town map view', () => {
  const view = { x: -6, z: -6, w: 412, d: 212 };

  test('a box the map’s shape: corners to corners', () => {
    const box = { left: 100, top: 50, width: 824, height: 424 };
    expect(mapScale(box.width, box.height, view)).toBe(2);
    expect(screenToMap(100, 50, box, view)).toEqual([-6, -6]);
    expect(screenToMap(924, 474, box, view)).toEqual([406, 206]);
    expect(screenToMap(512, 262, box, view)).toEqual([200, 100]);
  });

  test('a box taller than the map: bands above and below, the scale from its width', () => {
    // 824 px wide, 700 px tall: drawn 824 × 424, centred (138 px bands).
    const box = { left: 0, top: 0, width: 824, height: 700 };
    expect(mapScale(box.width, box.height, view)).toBe(2);
    expect(screenToMap(0, 138, box, view)).toEqual([-6, -6]);
    expect(screenToMap(412, 350, box, view)).toEqual([200, 100]);
    // (The box's own height would have put this 68 m too far down.)
    expect(screenToMap(824, 562, box, view)).toEqual([406, 206]);
  });

  test('a box wider than the map: bands at the sides, the scale from its height', () => {
    const box = { left: 0, top: 0, width: 1200, height: 424 };
    expect(mapScale(box.width, box.height, view)).toBe(2);
    const band = (1200 - 824) / 2;
    expect(screenToMap(band, 0, box, view)).toEqual([-6, -6]);
    expect(screenToMap(600, 212, box, view)).toEqual([200, 100]);
  });

  test('an empty box or view doesn’t divide by zero', () => {
    expect(mapScale(0, 0, view)).toBe(1);
    expect(mapScale(100, 100, { x: 0, z: 0, w: 0, d: 0 })).toBe(1);
  });
});

describe('place names', () => {
  const plots = [
    { id: 0, name: '1 Elm Row', x: 0, z: 0, w: 20, d: 30, public: false },
    { id: 1, name: '3 Elm Row', x: 20, z: 0, w: 20, d: 30, public: false },
    { id: 2, name: 'Village green', x: 60, z: 0, w: 40, d: 40, public: true },
  ];
  const households = [{ name: 'Smith', plot: 0 }];
  const areas = [{ name: 'Millbrook', x: 0, z: 0, w: 120, d: 60 }];

  test('the lot under the camera and the hamlet round it', () => {
    expect(placeAt(10, 10, plots, households, areas)).toBe('The Smiths · Millbrook');
    expect(placeAt(30, 10, plots, households, areas)).toBe('3 Elm Row · Millbrook');
    expect(placeAt(80, 20, plots, households, areas)).toBe('Village green · Millbrook');
  });

  test('just off a lot names it; far off, only the hamlet (or nothing)', () => {
    expect(placeAt(50, 10, plots, households, areas)).toBe('3 Elm Row · Millbrook');
    expect(placeAt(110, 55, plots, households, areas)).toBe('Millbrook');
    expect(placeAt(500, 500, plots, households, areas)).toBe('');
    expect(placeAt(10, 10, plots, households, [])).toBe('The Smiths');
  });

  test('a resident on the way: where to, and when', () => {
    const time = (m: number) => `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}`;
    expect(tripText({ to: 1, at: 17 * 60 + 40 }, 1, plots, households, time)).toBe('On the way home · back about 17:40');
    expect(tripText({ to: 0, at: 14 * 60 + 20 }, 1, plots, households, time)).toBe('On the way to the Smiths · there about 14:20');
    expect(tripText({ to: 2, at: 9 * 60 }, 1, plots, households, time)).toBe('On the way to Village green · there about 9:00');
    // (The time only comes with a resident's details.)
    expect(tripText({ to: 1 }, 1, plots, households, time)).toBe('On the way home');
    expect(tripText({ to: 0 }, 1, plots, households, time)).toBe('On the way to the Smiths');
  });
});
