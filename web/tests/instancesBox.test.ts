// An instanced draw's world box (`Instances` in render/pc/instancing.ts): round every copy, following the entity
// when it moves and the copies when they change, and not worked out again in a frame where neither did (PlayCanvas
// does it every frame for a custom box: 0.19 ms a frame on Low with its woods in squares). On the engine's null
// device: no GPU.
import { afterAll, describe, expect, mock, spyOn, test } from 'bun:test';
// (PlayCanvas as the renderer imports it: render/pc/engine.ts.)
import * as pc from '../src/render/pc/engine';

// (The renderer's modules read the page's address as they load: one put in for them, the real one back after.)
const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (Only the colour stream's attribute comes from the materials, whose module wants the game's settings.)
mock.module('../src/render/pc/materials', () => ({ INSTANCE_COLOR_SEMANTIC: pc.SEMANTIC_ATTR10 }));
const { Instances } = await import('../src/render/pc/instancing');

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
  // A unit box (−0.5..0.5).
  const mesh = pc.Mesh.fromGeometry(device, new pc.BoxGeometry());
  return { device, root, mesh };
}

const at = (x: number, z: number) => new pc.Mat4().setTranslate(x, 0, z).data;

describe("an instanced draw's world box", () => {
  test('round every copy, and after the entity or a copy moves', () => {
    const { device, root, mesh } = scene();
    const d = new Instances(device, root, mesh, new pc.StandardMaterial(), { capacity: 2 });
    d.setCount(2);
    d.setMatrix(0, at(0, 0));
    d.setMatrix(1, at(10, 4));
    d.update();
    const box = () => [d.meshInstance.aabb.getMin().toArray(), d.meshInstance.aabb.getMax().toArray()];
    expect(box()).toEqual([[-0.5, -0.5, -0.5], [10.5, 0.5, 4.5]]);
    root.setPosition(100, 0, 0);
    expect(box()).toEqual([[99.5, -0.5, -0.5], [110.5, 0.5, 4.5]]);
    d.setMatrix(1, at(2, 0));
    d.update();
    expect(box()).toEqual([[99.5, -0.5, -0.5], [102.5, 0.5, 0.5]]);
  });

  test('not worked out again while nothing moves', () => {
    const { device, root, mesh } = scene();
    const d = new Instances(device, root, mesh, new pc.StandardMaterial());
    d.setCount(1);
    d.setMatrix(0, at(3, 3));
    d.update();
    const spy = spyOn(pc.BoundingBox.prototype, 'setFromTransformedAabb');
    for (let frame = 0; frame < 10; frame++) void d.meshInstance.aabb;
    expect(spy).toHaveBeenCalledTimes(1);
    root.setPosition(1, 0, 0);
    void d.meshInstance.aabb;
    void d.meshInstance.aabb;
    expect(spy).toHaveBeenCalledTimes(2);
    spy.mockRestore();
  });
});
