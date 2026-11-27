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
// (The settings are a Svelte store, and the perf spans watch the page's frames: not for a test.)
mock.module('../src/settings/settings.svelte', () => ({ settings: { shading: 'soft' } }));
mock.module('../src/debug/perf', () => ({ perfTime: <T>(_: string, fn: () => T) => fn(), perfTimeAsync: <T>(_: string, fn: () => Promise<T>) => fn(), perfStart: () => ({}), perfEnd: () => 0 }));
// (A library that takes what the lamps give it.)
const lib = { setRoomBounce() {}, setRoomDay() {}, excludeFromLights() {}, rooms: { enabled: true, show() {}, setLight() {} } };
const materials = await import('../src/render/pc/materials');
mock.module('../src/render/pc/materials', () => ({ ...materials, MaterialLibrary: { for: () => lib } }));
const { Lamps } = await import('../src/render/pc/lamps');
const { LotContents } = await import('../src/render/pc/lotContents');

function lamps(drawn: { windows: boolean }, lot: { applyGlow: (light: never) => boolean } = { applyGlow: () => false }) {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const app = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = new pc.NullGraphicsDevice(canvas);
  options.componentSystems = [pc.LightComponentSystem];
  options.resourceHandlers = [];
  app.init(options);
  const view = { storey: () => 0, storeyOf: () => 0, rowOf: () => 0, town: () => false, whole: () => false };
  const hooks = { house: { setWindowGlow: () => drawn.windows }, lot, pools: () => false, casters: () => [] };
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

  test("a fire burning on the lot isn't told while a room elsewhere fades", () => {
    // The lot's one glow, a fire out of doors (no room, no switch), its flames swaying: `LotContents` with only what its
    // glows use.
    const draw = { destroyed: false, enabled: false, setAll() {}, update() {} };
    const fire = { draw, colors: new Float32Array(4), rooms: [0], lamps: [-1], ids: [5], screen: false, glass: false, base: new Float32Array(1) };
    const lot = { glows: [fire], motion: { burning: () => true }, flickerTime: 0 };
    const contents = LotContents.prototype as unknown as { applyGlow(light: never): boolean; animateFires(): boolean };
    const l = lamps({ windows: false }, { applyGlow: (light) => contents.applyGlow.call(lot as never, light) });
    expect(draw.enabled).toBe(true);
    let told = 0;
    l.onChanged = () => told++;
    const dark = new Set([7]);
    for (let i = 0; i < 10; i++) {
      lot.flickerTime = i * 16;
      contents.animateFires.call(lot as never);
      l.frame(1 / 60, dark, new Set(), 0, 0);
    }
    expect(told).toBe(0);
  });

  test("a fire's flicker is told while the game runs (the mirrors show it), and stands still while it's paused", () => {
    const draw = { destroyed: false, enabled: true, setAll() {}, update() {} };
    const fire = { draw, colors: new Float32Array(4), rooms: [0], lamps: [-1], ids: [5], screen: false, glass: false, base: new Float32Array([1]) };
    const lot = Object.setPrototypeOf(
      { glows: [fire], motion: { burning: () => true, update: () => ({ moved: false, ambient: false }) }, camera: { getPosition: () => ({ x: 0, y: 0, z: 0 }) }, eye: {}, screenOn: new Set(), flickerTime: 0, flickerAt: 0 },
      LotContents.prototype,
    ) as { update(frame: unknown): boolean };
    const frame = (now: number, speed: number) => ({ now, curr: [speed], layout: { header: { speed: 0 }, actions: [] } });
    lot.update(frame(0, 1));
    expect(lot.update(frame(16, 1))).toBe(true);
    // Paused: the flicker holds, and nothing is told (a paused frame stays held).
    lot.update(frame(32, 0));
    expect(lot.update(frame(48, 0))).toBe(false);
  });
});
