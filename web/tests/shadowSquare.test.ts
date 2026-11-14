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
  return { shadows, drawn, device };
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
    // A caster moving right after: held back for the rate, drawn once it has passed.
    shadows.moved(false);
    expect(drawn()).toBe(false);
    shadows.frame();
    expect(drawn()).toBe(false);
    await Bun.sleep(110);
    shadows.frame();
    expect(drawn()).toBe(true);
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

// The still casters' picture (`SunShadows.still`): a redraw for what moves alone (`posed`) copies it into the map and
// draws only what moves over it; anything else (`moved`, the light's view, a still caster that came, went or moved)
// draws the picture again first. On the null device: the passes' draws and copies recorded, not made.
describe("the sun's map over the still casters' picture", () => {
  function split() {
    const { shadows, device } = sun();
    const cam = new pc.Camera(device);
    cam.node = new pc.GraphNode('light');
    cam.projection = pc.PROJECTION_ORTHOGRAPHIC;
    cam.aspectRatioMode = pc.ASPECT_MANUAL;
    cam.aspectRatio = 1;
    cam.orthoHeight = 30;
    cam.node.setPosition(0, 50, 0);
    cam.node.setEulerAngles(-90, 0, 0);
    const depth = new pc.Texture(device, { width: 256, height: 256, format: pc.PIXELFORMAT_DEPTH, mipmaps: false });
    const map = new pc.RenderTarget({ name: 'map', depthBuffer: depth });
    cam.renderTarget = map;
    const caster = (x: number, cull = true) => {
      const aabb = new pc.BoundingBox(new pc.Vec3(x, 0.5, 0), new pc.Vec3(0.5, 0.5, 0.5));
      return { cull, aabb, instancingCount: 1 } as unknown as pc.MeshInstance;
    };
    const house = caster(0);
    const tree = caster(10);
    const body = caster(0, false);
    const visibleCasters = [house, body, tree];
    const light = { numCascades: 1, _shadowMap: {}, _shadowCascadesInvalidated: false, shadowUpdateOverrides: null, getRenderData: () => ({ shadowCamera: cam, visibleCasters }) };
    const log: string[] = [];
    const name = (mi: pc.MeshInstance) => (mi === house ? 'house' : mi === tree ? 'tree' : 'body');
    const renderer = { renderFace: (_l: unknown, _c: unknown, _f: number, clear: boolean) => log.push(`${cam.renderTarget === map ? 'map' : 'still'}${clear ? ' cleared' : ''}: ${visibleCasters.map(name).join(' ')}`) };
    const d = device as unknown as Record<string, unknown>;
    d.startRenderPass = () => {};
    d.endRenderPass = () => {};
    d.copyRenderTarget = (from: pc.RenderTarget, to: pc.RenderTarget) => log.push(`copy ${from === map ? 'map' : 'still'} → ${to === map ? 'map' : 'still'}`);
    const pass = { light, camera: cam, renderTarget: map, shadowRenderer: renderer, depthStencilOps: { clearDepth: true }, before() {}, execute() {} };
    shadows.movers = (out) => out.push(body);
    const draw = () => {
      log.length = 0;
      shadows.drawStill(pass as unknown as Parameters<typeof shadows.drawStill>[0]);
      shadows.drawPass(pass as unknown as Parameters<typeof shadows.drawPass>[0]);
      return log.slice();
    };
    return { shadows, cam, house, tree, body, visibleCasters, draw };
  }

  test('the first redraw draws the still casters into their picture; a pose only copies it and draws the residents', () => {
    const { shadows, draw, visibleCasters, house, body, tree } = split();
    expect(draw()).toEqual(['still: house tree', 'copy still → map', 'map: body']);
    shadows.posed();
    expect(draw()).toEqual(['copy still → map', 'map: body']);
    expect(visibleCasters).toEqual([house, body, tree]);
    expect(shadows.draws).toEqual({ whole: 1, moving: 1 });
  });

  test('anything else drawn again first: a thing moved, a still caster moved or went, the light turned', () => {
    const { shadows, draw, tree, visibleCasters, cam } = split();
    draw();
    shadows.moved(false);
    expect(draw()[0]).toBe('still: house tree');
    tree.aabb.center.x += 1;
    shadows.posed();
    expect(draw()[0]).toBe('still: house tree');
    visibleCasters.pop();
    shadows.posed();
    expect(draw()[0]).toBe('still: house');
    cam.node.setPosition(1, 50, 0);
    shadows.posed();
    expect(draw()[0]).toBe('still: house');
    // The fit's float noise (as the camera turns) is no change.
    cam.node.setPosition(1 + 1e-9, 50, 0);
    shadows.posed();
    expect(draw()).toEqual(['copy still → map', 'map: body']);
  });

  test("without what moves to ask (`movers`), or the picture off, the pass is PlayCanvas's own", () => {
    const { shadows } = split();
    expect(shadows.splits({ numCascades: 1 } as never)).toBe(true);
    shadows.movers = null;
    expect(shadows.splits({ numCascades: 1 } as never)).toBe(false);
  });
});
