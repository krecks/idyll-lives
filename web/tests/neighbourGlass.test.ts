// The neighbours' window glass (`pc/street.ts` `drawWindows`), drawn again whenever a light goes on or off: from each set
// of panes' vertex arrays, made once (`paneData`) and put together (`concatMeshData`), the very mesh the planner's one
// geometry of them all gives (`HousePlanner.neighbourPanes`), so nothing drawn changes for the time it saves.
import { afterAll, expect, mock, test } from 'bun:test';
import { Geo } from '../src/render/shared/geometry';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (The settings are a Svelte store, not for a test.)
mock.module('../src/settings/settings.svelte', () => ({ settings: { shading: 'soft' } }));
// (Nor the perf spans, which watch the page's frames.)
mock.module('../src/debug/perf', () => ({ perfTime: <T>(_: string, fn: () => T) => fn(), perfTimeAsync: <T>(_: string, fn: () => Promise<T>) => fn(), perfStart: () => ({}), perfEnd: () => 0 }));
const { HousePlanner } = await import('../src/render/shared/house');
const { concatMeshData, paneData } = await import('../src/render/pc/street');

test("the neighbours' glass from each set's arrays is the planner's one mesh of them", () => {
  // Three sets of panes, coloured and cut their own ways (the glass paints them in its one colour), one empty.
  const sets = [0, 1, 2].map((k) => {
    const g = new Geo().color(k ? '#336699' : '#ffffff');
    for (let i = 0; i <= k; i++) g.poly([[i, 0, k], [i + 1, 0, k], [i + 1, 1, k], [i, 1, k]], [0, 0, 1]);
    return g;
  });
  sets.push(new Geo());
  const whole = new HousePlanner().neighbourPanes(sets, true).geo.meshData({ colors: true, cut: true });
  const joined = concatMeshData(sets.map(paneData))!;
  for (const k of ['positions', 'normals', 'uvs', 'indices', 'colors', 'cut'] as const) expect([...joined[k]!]).toEqual([...whole[k]!]);
  expect(joined.indices.constructor).toBe(whole.indices.constructor);
  expect(joined.vertexAlpha).toBe(whole.vertexAlpha);
  // (Made once a set.)
  expect(paneData(sets[1])).toBe(paneData(sets[1]));
});
