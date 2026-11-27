// The lamps' changes (`render/pc/lamps.ts` `onChanged`, which draws the mirrors' held reflections again): the
// windows of every room in town fade as its residents fall asleep, nearly every frame of the night, but only a fade
// that changes something drawn (the viewed house's windows, the lot's shades, the garden's pools) is told. On the
// engine's null device: no GPU.
import { afterAll, describe, expect, mock, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';
import type { Lighting } from '../src/render/shared/environment';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (The materials' module reads the game's settings: a library that takes what the lamps give it.)
const lib = { setRoomBounce() {}, setRoomDay() {}, excludeFromLights() {}, rooms: { enabled: true, show() {}, setLight() {} } };
mock.module('../src/render/pc/materials', () => ({ MaterialLibrary: { for: () => lib }, ROOM_SPLIT: 0x4000, ROOM_SPLIT_DN: 0x2000 }));
const { Lamps } = await import('../src/render/pc/lamps');

function lamps(drawn: { windows: boolean }) {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const app = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = new pc.NullGraphicsDevice(canvas);
  options.componentSystems = [pc.LightComponentSystem];
  options.resourceHandlers = [];
  app.init(options);
  const view = { storey: () => 0, storeyOf: () => 0, rowOf: () => 0, town: () => false, whole: () => false };
  const hooks = { house: { setWindowGlow: () => drawn.windows }, lot: { applyGlow: () => false }, pools: () => false, casters: () => [] };
  const l = new Lamps(app, { clusteredLamps: false, lampShadows: 0 } as never, view, hooks as never);
  l.applyLighting({ lamps: 1, exposure: 1 } as Lighting);
  return l;
}

describe("the lamps' changes", () => {
  test("a room fading that nothing drawn shows isn't told; one whose windows are drawn is", () => {
    const drawn = { windows: false };
    const l = lamps(drawn);
    let told = 0;
    l.onChanged = () => told++;
    // A neighbour's room (not drawn here) falls asleep: its glow fades over a second or so.
    const dark = new Set([7]);
    for (let i = 0; i < 10; i++) l.frame(1 / 60, dark, new Set(), 0, 0);
    expect(told).toBe(0);
    // The viewed house's windows change with it: told, each frame it does.
    drawn.windows = true;
    l.frame(1 / 60, dark, new Set(), 0, 0);
    expect(told).toBe(1);
  });
});
