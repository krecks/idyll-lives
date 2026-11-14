// The sun's shadow square (`render/pc/sunShadows.ts`): drawn again when something in it moved, not when a car or
// a resident moved across town (`reaches`, asked of each that moved). On the engine's null device: no GPU.
import { afterAll, describe, expect, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
const { CASTER_REACH, SunShadows } = await import('../src/render/pc/sunShadows');

function sun(rate = 0) {
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
  // (Rate 0: every pose redraws at once.)
  const shadows = new SunShadows(app, light, { size: 256, rate });
  const drawn = () => {
    const now = light.light!.shadowUpdateMode === pc.SHADOWUPDATE_THISFRAME;
    light.light!.shadowUpdateMode = pc.SHADOWUPDATE_NONE;
    return now;
  };
  return { shadows, drawn };
}

describe("the sun's shadow square", () => {
  test('a caster reaches it in the square or near it, not across town (the cars and residents ask for each that moved)', () => {
    const { shadows } = sun();
    shadows.setSquare(100, 200, 40);
    expect(shadows.reaches(130, 230)).toBe(true);
    expect(shadows.reaches(60, 160)).toBe(true);
    // Just outside, within reach of a long shadow; further out, not.
    expect(shadows.reaches(140 + CASTER_REACH - 1, 200)).toBe(true);
    expect(shadows.reaches(140 + CASTER_REACH + 1, 200)).toBe(false);
    expect(shadows.reaches(100, 160 - CASTER_REACH - 1)).toBe(false);
    expect(shadows.reaches(500, 500)).toBe(false);
  });

  test('a moved caster draws it again (what the cars and residents trigger), a pose at the shadow rate', async () => {
    const { shadows, drawn } = sun(10);
    shadows.setSquare(100, 200, 40);
    expect(drawn()).toBe(true);
    // A caster moving right after: held back for the rate, drawn once it has passed (a held game frame waits for it).
    shadows.moved(false);
    expect(shadows.waiting).toBe(true);
    expect(drawn()).toBe(false);
    shadows.frame();
    expect(drawn()).toBe(false);
    await Bun.sleep(110);
    shadows.frame();
    expect(shadows.waiting).toBe(true);
    expect(drawn()).toBe(true);
    expect(shadows.waiting).toBe(false);
    // The sun, the walls, a thing placed: at once.
    shadows.moved();
    expect(drawn()).toBe(true);
  });

  test("in the menus' town everything counts (the map follows the view)", () => {
    const { shadows } = sun();
    shadows.setOverview(true);
    expect(shadows.reaches(9000, 9000)).toBe(true);
  });
});
