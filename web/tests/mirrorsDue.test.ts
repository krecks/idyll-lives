// A held mirror reflection (`render/pc/mirrors.ts` `due`), on High drawn again in the frame something in it changed: a
// resident walking into the view of a mirror nobody was in is such a change, though the picking that finds who a view
// shows runs only in the frames that draw.
import { afterAll, expect, mock, test } from 'bun:test';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (The settings are a Svelte store, not for a test.)
mock.module('../src/settings/settings.svelte', () => ({ settings: { shading: 'soft' } }));
const { Mirrors } = await import('../src/render/pc/mirrors');

type Due = { due(now: number, viewProj: Float32Array, residents: unknown): boolean };

test('a resident walking into a held reflection nobody was in draws it again', () => {
  const viewProj = new Float32Array(16);
  // The view: in front of the glass (z > 0), one plane is enough here.
  const planes = new Float32Array(20);
  planes.set([0, 0, 1, 0]);
  const r = { due: 0, dirty: false, next: { slot: 0 }, drawnViewProj: viewProj.slice(), showsAnyone: false, shows: new Float32Array(4), planes, planeCount: 1 };
  const mirrors = { pool: [r], residentCount: 1, lo: new Float32Array(3), hi: new Float32Array(3) };
  Object.setPrototypeOf(mirrors, Mirrors.prototype);
  const due = (z: number) => (mirrors as unknown as Due).due(1000, viewProj, { heads: new Float32Array([0, 1.7, z]), visible: new Uint8Array([1]), posed: true });
  // Walking behind the glass, out of its view: nothing to draw.
  expect(due(-5)).toBe(false);
  // Walking in front of it: drawn again.
  expect(due(2)).toBe(true);
});
