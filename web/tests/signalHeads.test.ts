// The traffic lights' draws (`render/pc/signals.ts`): a lamp changing writes the glows where the poles stand, without
// looking the ground up again (each look walks the roads round the head); the poles' heights taken when they're written.
// On the engine's null device: no GPU.
import { afterAll, describe, expect, mock, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';
import type { AssetRegistry } from '../src/assets/registry';
import type { FrameState } from '../src/core/bridge';
import { layout, LAYOUT } from './snapshotLayout';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
mock.module('../src/render/pc/materials', () => ({ INSTANCE_COLOR_SEMANTIC: pc.SEMANTIC_ATTR10, MaterialLibrary: { for: () => ({ excludeFromLights: () => {} }) } }));
mock.module('../src/render/pc/shaders/chunks', () => ({ WALL_CUT_SEMANTIC: pc.SEMANTIC_ATTR7 }));
// (The model is built by `load`, left out here: its draws are put in.)
mock.module('../src/render/pc/models', () => ({ buildModel: () => {}, disposeTemplate: () => {} }));
const { Instances } = await import('../src/render/pc/instancing');
const { Signals } = await import('../src/render/pc/signals');

const F = LAYOUT.light;

/** Two heads, the first showing `first` (the sim's state), the second red. */
function snapshot(first: number): Float32Array {
  const snap = new Float32Array(layout.capacity);
  snap[LAYOUT.header.lightCount] = 2;
  [
    [10, 20, first],
    [30, 40, F.states.red],
  ].forEach(([x, z, state], k) => {
    const o = LAYOUT.lightBase + k * LAYOUT.lightStride;
    [snap[o + F.shown], snap[o + F.x], snap[o + F.z], snap[o + F.yaw], snap[o + F.state]] = [1, x, z, 0, state];
  });
  return snap;
}

describe('the traffic lights drawn', () => {
  test('a lamp changing lights the glow where its pole stands, not looking the ground up again', () => {
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
    const mesh = pc.Mesh.fromGeometry(device, new pc.BoxGeometry());
    const draw = () => new Instances(device, root, mesh, new pc.StandardMaterial(), { capacity: 2 });
    const signals = new Signals(app, root, {} as AssetRegistry, { shadows: false, lod: 0 });
    const [body, red, amber, green] = [draw(), draw(), draw(), draw()];
    // (The model as `load` builds it: its draws put in.)
    Object.assign(signals, { drawn: { near: {}, far: null, body: [body], farBody: [], glows: [red, amber, green], ped: null } });
    let looks = 0;
    signals.setGround((x) => (looks++, x / 10));
    const frame = (curr: Float32Array): FrameState => ({ prev: curr, curr, alpha: 1, steps: 1, layout, now: 0 });
    signals.update(frame(snapshot(F.states.green)));
    expect(looks).toBe(2);
    looks = 0;
    signals.update(frame(snapshot(F.states.amber)));
    expect(looks).toBe(0);
    // The first head's amber glow where its pole stands, on the ground there (1 m up at x 10).
    expect(Array.from(amber.matrices())).toEqual(Array.from(body.matrices().subarray(0, 16)));
    expect(amber.matrices()[13]).toBeCloseTo(1, 6);
  });
});
