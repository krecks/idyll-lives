// Held frames (`render/pc/heldFrames.ts`): a view standing still is drawn once and then held, drawn again when
// it changes; the paused game's (`settleMs`) only once a while has passed since the last change; with water flowing
// in view (`motionMs`) drawn again at that rate. On the
// engine's null device: no GPU.
import { afterAll, describe, expect, mock, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';

// (The model files' arrivals, which draw a held view again, aren't what is tested: their module, with the settings it reads, is left out.)
mock.module('../src/render/pc/models', () => ({ onModelFileArrived: () => () => {} }));

const real = { location: globalThis.location, document: globalThis.document };
Object.assign(globalThis, {
  location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' },
  document: { addEventListener() {}, removeEventListener() {} },
});
afterAll(() => Object.assign(globalThis, real));
const { HeldFrames } = await import('../src/render/pc/heldFrames');
const { fixFrameClock } = await import('../src/render/shared/frameClock');
// The renderer's own clock, fixed and moved by hand: a slow machine can't change what is drawn.
let clock = 1000;
const wait = (ms: number) => fixFrameClock((clock += ms));
fixFrameClock(clock);
afterAll(() => fixFrameClock(null));

function view(enabled = true) {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const device = new pc.NullGraphicsDevice(canvas);
  const app = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = device;
  options.componentSystems = [pc.CameraComponentSystem, pc.LightComponentSystem];
  options.resourceHandlers = [];
  app.init(options);
  const camera = new pc.Entity('camera', app);
  camera.addComponent('camera');
  app.root.addChild(camera);
  const sun = new pc.Entity('sun', app);
  sun.addComponent('light', { type: 'directional' });
  app.root.addChild(sun);
  const held = new HeldFrames(app, camera.camera!, sun, enabled);
  /** The next `n` frames: whether each is drawn. */
  const frames = (n: number) => Array.from({ length: n }, () => held.draw());
  return { held, camera, frames };
}

describe('held frames', () => {
  test('a still view is drawn a few frames, then held until it changes', () => {
    const { held, camera, frames } = view();
    held.active = true;
    expect(frames(6)).toEqual([true, true, true, false, false, false]);
    expect(held.holding).toBe(true);
    // The camera moves: drawn again, then held.
    camera.setPosition(1, 2, 3);
    expect(frames(5)).toEqual([true, true, true, false, false]);
    // Told of a change it can't see (a highlight, the thing in hand): the same.
    held.changed();
    expect(frames(4)).toEqual([true, true, true, false]);
    // A frame something waits for is drawn.
    expect(held.draw(true)).toBe(true);
  });

  test('not active, or the preset without it: every frame is drawn', () => {
    const off = view();
    expect(off.frames(5).every(Boolean)).toBe(true);
    const preset = view(false);
    preset.held.active = true;
    expect(preset.frames(5).every(Boolean)).toBe(true);
    expect(preset.held.holding).toBe(false);
  });

  test("the paused game's change is drawn for `settleMs` (what is drawn at a rate of its own comes a little later)", () => {
    const { held, frames } = view();
    held.active = true;
    held.settleMs = 60;
    frames(4);
    held.changed();
    expect(frames(6).every(Boolean)).toBe(true);
    wait(80);
    expect(frames(2)).toEqual([false, false]);
  });

  test('water flowing in view: a held view is drawn again every `motionMs`, its clocks going on between', () => {
    const { held, frames } = view();
    held.active = true;
    frames(4);
    expect(held.holding).toBe(true);
    held.motionMs = 40;
    // (The last drawn some frames ago: drawn now, then not until 40 ms on, not holding meanwhile.)
    wait(45);
    expect(frames(3)).toEqual([true, false, false]);
    expect(held.holding).toBe(false);
    wait(45);
    expect(frames(2)).toEqual([true, false]);
    // No water in view: held again.
    held.motionMs = 0;
    expect(frames(2)).toEqual([false, false]);
    expect(held.holding).toBe(true);
  });
});
