// The sun's shadow square (`render/pc/sunShadows.ts`): drawn again when something in it moved, not when a car or
// a resident moved across town (`movedIn`: a rectangle round what moved, or true/false as before). On the
// engine's null device: no GPU.
import { afterAll, describe, expect, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
const { CASTER_REACH, SunShadows } = await import('../src/render/pc/sunShadows');

function sun() {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const device = new pc.NullGraphicsDevice(canvas);
  const app = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = device;
  options.componentSystems = [pc.RenderComponentSystem, pc.LightComponentSystem];
  options.resourceHandlers = [];
  app.init(options);
  const light = new pc.Entity('sun', app);
  light.addComponent('light', { type: 'directional', castShadows: true });
  app.root.addChild(light);
  // (Every pose redraws at once: the rate isn't what's looked at.)
  const shadows = new SunShadows(app, light, { size: 256, rate: 0 });
  const drawn = () => {
    const now = light.light!.shadowUpdateMode === pc.SHADOWUPDATE_THISFRAME;
    light.light!.shadowUpdateMode = pc.SHADOWUPDATE_NONE;
    return now;
  };
  return { shadows, drawn };
}

describe("the sun's shadow square", () => {
  test('hands out the ground it covers', () => {
    const { shadows } = sun();
    shadows.setSquare(100, 200, 40);
    expect({ ...shadows.rect }).toEqual({ x0: 60, z0: 160, x1: 140, z1: 240 });
  });

  test('draws again for what moved in it or near it, not for what moved across town', () => {
    const { shadows, drawn } = sun();
    shadows.setSquare(100, 200, 40);
    drawn();
    shadows.movedIn({ x0: 500, z0: 500, x1: 510, z1: 505 });
    expect(drawn()).toBe(false);
    shadows.movedIn({ x0: 130, z0: 230, x1: 135, z1: 235 });
    expect(drawn()).toBe(true);
    // Just outside, within reach of a long shadow; further out, not.
    shadows.movedIn({ x0: 140 + CASTER_REACH - 1, z0: 200, x1: 150 + CASTER_REACH, z1: 201 });
    expect(drawn()).toBe(true);
    shadows.movedIn({ x0: 140 + CASTER_REACH + 1, z0: 200, x1: 150 + CASTER_REACH, z1: 201 });
    expect(drawn()).toBe(false);
    // The booleans as ever.
    shadows.movedIn(false);
    expect(drawn()).toBe(false);
    shadows.movedIn(true);
    expect(drawn()).toBe(true);
  });

  test("in the menus' town everything counts (the map follows the view)", () => {
    const { shadows } = sun();
    shadows.setOverview(true);
    expect(shadows.rect).toBeNull();
    expect(shadows.touches({ x0: 9000, z0: 9000, x1: 9001, z1: 9001 })).toBe(true);
  });
});
