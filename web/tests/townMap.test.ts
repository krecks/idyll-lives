/**
 * The town map's view (`ui/mapView.ts`): a point on screen to the ground under it when the map
 * fits a box of another shape (the "Bigger" map), its names kept apart, and the place pill's words (`ui/place.ts`): where a
 * resident is off to, on foot or riding.
 */
import { describe, expect, test } from 'bun:test';
import { labelWidth, layoutLabels, mapScale, screenToMap, type MapLabel } from '../src/ui/mapView';
import { placeAt, rideText, tripText, wayText } from '../src/ui/place';

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

describe('town map names', () => {
  // 400 × 400 m drawn in a 400 px box: a metre a pixel.
  const view = { x: 0, z: 0, w: 400, d: 400 };
  const label = (key: string, x: number, z: number, text: string, rank: number, more: Partial<MapLabel<string>> = {}): MapLabel<string> => ({ key, x, z, text, size: 11, rank, ...more });
  const box = (l: MapLabel<string>, at: { x: number; y: number }) => {
    const w = labelWidth(l.text, l.size);
    return { l: at.x - w / 2, r: at.x + w / 2, t: at.y - l.size, b: at.y };
  };
  const apart = (a: ReturnType<typeof box>, b: ReturnType<typeof box>) => a.r <= b.l || b.r <= a.l || a.b <= b.t || b.b <= a.t;

  test('a hamlet and its orchard by it: the hamlet’s name where it is, the orchard’s moved clear of it or not drawn', () => {
    // (The "Bigger" map's "Maple Heights" over "Maple Heights Orchard".)
    const hamlet = label('hamlet', 200, 200, 'Maple Heights', 0, { size: 14 });
    const orchard = label('orchard', 230, 202, 'Maple Heights Orchard', 1);
    const placed = layoutLabels([orchard, hamlet], 400, 400, view);
    const h = placed.get('hamlet')!;
    expect(h.x).toBeCloseTo(200);
    const o = placed.get('orchard');
    if (o) expect(apart(box(hamlet, h), box(orchard, o))).toBe(true);
  });

  test('names with room each stay over their point, centred on it', () => {
    const a = label('a', 100, 100, 'Oakton', 0, { size: 14 });
    const b = label('b', 300, 300, 'Picnic Meadow', 1);
    const placed = layoutLabels([a, b], 400, 400, view);
    expect(placed.get('a')!.x).toBeCloseTo(100);
    expect(placed.get('b')!.x).toBeCloseTo(300);
    // The line's baseline a little below the point, its middle on it (11 px: 1.25 × 11 high).
    expect(placed.get('b')!.y).toBeCloseTo(300 - 11 * 0.625 + 11 * 0.97);
  });

  test('a pile of names: none drawn over another, the household’s own always', () => {
    const pile = Array.from({ length: 12 }, (_, i) => label(`p${i}`, 200 + (i % 3) * 4, 200 + i * 2, `Place number ${i}`, 2));
    const home = label('home', 202, 205, 'The Smiths', 2, { wins: 0 });
    const placed = layoutLabels([...pile, home], 400, 400, view);
    expect(placed.has('home')).toBe(true);
    const shown = [...pile, home].filter((l) => placed.has(l.key)).map((l) => box(l, placed.get(l.key)!));
    expect(shown.length).toBeLessThan(13);
    for (let i = 0; i < shown.length; i++) for (let j = i + 1; j < shown.length; j++) expect(apart(shown[i], shown[j])).toBe(true);
  });

  test('a name out of view isn’t drawn; one by the edge is kept inside the box', () => {
    const zoomed = { x: 100, z: 100, w: 200, d: 200 };
    const placed = layoutLabels([label('out', 50, 150, 'Far away', 0), label('edge', 102, 150, 'Cedarbury Lookout', 1)], 400, 400, zoomed);
    expect(placed.has('out')).toBe(false);
    // (2 px a metre: its half-width in metres in from the view's left.)
    expect(placed.get('edge')!.x).toBeCloseTo(100 + labelWidth('Cedarbury Lookout', 11) / 4);
  });

  test('clear of the zoom buttons', () => {
    const buttons = { l: 326, t: 360, r: 400, b: 400 };
    const placed = layoutLabels([label('lookout', 360, 385, 'Lookout', 1)], 400, 400, view, [buttons]);
    const at = placed.get('lookout');
    if (at) expect(apart(box(label('lookout', 0, 0, 'Lookout', 1), at), buttons)).toBe(true);
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

  test('a resident riding: cycling or on the e-scooter, to work, to school or home', () => {
    const time = (m: number) => `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}`;
    expect(rideText('bike', 'work')).toBe('Cycling to work');
    expect(rideText('bike', 'school')).toBe('Cycling to school');
    expect(rideText('scooter', 'work')).toBe('Riding the e-scooter to work');
    expect(tripText({ to: 1, at: 17 * 60 + 40 }, 1, plots, households, time, { kind: 'bike', to: 'home' })).toBe('Cycling home · back about 17:40');
    expect(tripText({ to: 1 }, 1, plots, households, time, { kind: 'scooter', to: 'home' })).toBe('Riding the e-scooter home');
    expect(rideText('bike', 'out')).toBe('Cycling out');
    expect(tripText({ to: 0 }, 1, plots, households, time, { kind: 'bike', to: 'out' })).toBe('Cycling to the Smiths');
  });

  test('a resident on their way: the trip, then the ride', () => {
    const time = (m: number) => `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}`;
    const way = (s: Parameters<typeof wayText>[0]) => wayText(s, 1, plots, households, time);
    const bike = { kind: 'bike', to: 'work' } as const;
    expect(way({ trip: { to: 0 }, ride: bike })).toBe('On the way to the Smiths');
    expect(way({ trip: { to: 1, at: 17 * 60 + 40 } })).toBe('On the way home · back about 17:40');
    expect(way({ ride: bike })).toBe('Cycling to work');
    expect(way({})).toBe('');
  });

  test('a resident and the work taxi: waiting at the kerb, in it to work or school, home in it', () => {
    const time = (m: number) => `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}`;
    const way = (s: Parameters<typeof wayText>[0]) => wayText(s, 1, plots, households, time);
    expect(way({ taxi: { state: 'wait' }, job: {} })).toBe('Waiting for the taxi');
    expect(way({ taxi: { state: 'work', at: 8 * 60 + 40 }, job: {} })).toBe('Taxi to work · there about 8:40');
    expect(way({ taxi: { state: 'work' }, job: null })).toBe('Taxi to school');
    expect(way({ taxi: { state: 'home', at: 18 * 60 }, job: {}, trip: { to: 1 } })).toBe('Taxi home');
  });
});
