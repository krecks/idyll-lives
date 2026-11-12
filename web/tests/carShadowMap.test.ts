// The cars and the sun's shadow map (`render/pc/cars.ts` `castsMoved`; the map is drawn again only when something
// that casts into it moved, `SunShadows.reaches`): a car that moves, comes or goes draws it again when it casts
// into it where it is now or where it was the frame before. At the fastest speed a car goes tens of metres a
// frame: one driving off from in the map to out of it left its shadow behind on the ground (Canalside, game
// speed 5). On the engine's null device: no GPU.
import { afterAll, describe, expect, mock, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';
import type { FrameState } from '../src/core/bridge';
import type { WorldStructure } from '../src/core/protocol';
import { carRowAt } from '../src/render/shared/cars';
import { layout, LAYOUT } from './snapshotLayout';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (The materials' module reads the game's settings: plain materials in their place.)
const material = () => new pc.StandardMaterial();
mock.module('../src/render/pc/materials', () => ({
  INSTANCE_COLOR_SEMANTIC: pc.SEMANTIC_ATTR10,
  MaterialLibrary: { for: () => ({ plainFinish: material, contactShadow: material, excludeFromLights: () => {} }) },
}));
// (The shader chunks' module reads the settings too: only the cutaway's attribute, which the meshes name.)
mock.module('../src/render/pc/shaders/chunks', () => ({ WALL_CUT_SEMANTIC: pc.SEMANTIC_ATTR7 }));
const { Cars } = await import('../src/render/pc/cars');

function scene() {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const device = new pc.NullGraphicsDevice(canvas);
  const app = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = device;
  options.componentSystems = [pc.RenderComponentSystem];
  options.resourceHandlers = [];
  app.init(options);
  const root = new pc.Entity('root', app);
  app.root.addChild(root);
  return { app, root };
}

const F = LAYOUT.car;

describe("the cars and the sun's shadow map", () => {
  test('a car driving off out of the map, or out of town, draws it again: its shadow goes with it', () => {
    const { app, root } = scene();
    const cars = new Cars(app, root, true);
    cars.build({ cars: [{ plot: 0, x: 10, z: 10, yaw: 0, body: 0, paint: 0 }] } as unknown as WorldStructure);
    // The map's square and the reach round it: x below 40.
    cars.setShadowTest((x) => x < 40);
    let prev = new Float32Array(layout.capacity);
    let curr = new Float32Array(layout.capacity);
    let now = 0;
    /** The next snapshot: the car at (x, 10), shown as `shown` (2 driving, 0 out of town); whether the map is to be drawn again. */
    const step = (x: number, shown: number): boolean => {
      prev = curr;
      curr = new Float32Array(layout.capacity);
      curr[layout.header.speed] = 5;
      curr[LAYOUT.header.carCount] = 1;
      const o = carRowAt(layout, 0);
      [curr[o + F.shown], curr[o + F.x], curr[o + F.z], curr[o + F.yaw]] = [shown, x, 10, Math.PI / 2];
      now += 1000 / 60;
      const frame: FrameState = { prev, curr, alpha: 1, steps: 1, layout, now };
      cars.update(frame);
      return cars.moved;
    };
    expect(step(10, 2)).toBe(true);
    step(10, 2);
    // Standing (its tilt settled): nothing to draw again.
    for (let i = 0; i < 300; i++) step(10, 2);
    expect(step(10, 2)).toBe(false);
    // Moving within the map: drawn again.
    expect(step(11, 2)).toBe(true);
    // Out of it in one frame (60 m at the fastest speed): drawn again, from where it was.
    expect(step(71, 2)).toBe(true);
    // On out there: the map as it is.
    expect(step(73, 2)).toBe(false);
    // Back in, then out of town where it stands in the map: drawn again.
    step(20, 2);
    expect(step(20, 0)).toBe(true);
  });
});
