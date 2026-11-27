// A gate swinging (`render/pc/fences.ts` `swingGate`): its leaf's vertices written into the fences' one mesh and that
// span alone sent up (the street's mesh holds every gate in town), the mesh's box made once round every leaf swung open
// either way; a land stroke under the town's gates lifts each (leaf and posts) by the ground's change, in place, as
// far open as it was. On the engine's null device: no GPU.
import { afterAll, describe, expect, mock, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';
import type { FenceStyleDef } from '../src/content/content';
import type { FenceEdge } from '../src/core/protocol';
import { swungGate } from '../src/render/shared/fences';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
mock.module('../src/render/pc/materials', () => ({ INSTANCE_COLOR_SEMANTIC: pc.SEMANTIC_ATTR10, MaterialLibrary: { for: () => ({ plainFinish: () => new pc.StandardMaterial() }) } }));
mock.module('../src/render/pc/shaders/chunks', () => ({ WALL_CUT_SEMANTIC: pc.SEMANTIC_ATTR7 }));
const { buildFences, regroundGates, swingGate } = await import('../src/render/pc/fences');

const STYLES = [{ id: 'picket', label: 'Picket', kind: 'picket', color: '#ffffff' }] as FenceStyleDef[];
const gate = (x: number, z: number): FenceEdge => ({ axis: 'h', x, z, kind: 'gate', style: 0 });

describe('a gate swinging', () => {
  test("writes its leaf alone, sends up only its span, and stays in the mesh's box", () => {
    const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
    const device = new pc.NullGraphicsDevice(canvas);
    const app = new pc.AppBase(canvas);
    const options = new pc.AppOptions();
    options.graphicsDevice = device;
    options.componentSystems = [pc.RenderComponentSystem];
    options.resourceHandlers = [];
    app.init(options);
    const fences = buildFences(app, app.root, [gate(0, 0), gate(10, 0)], STYLES)!;
    const vb = fences.mesh.vertexBuffer;
    const before = new Float32Array(vb.lock() as ArrayBuffer).slice();
    const sent: [number, number][] = [];
    const unlock = vb.impl.unlock.bind(vb.impl);
    vb.impl.unlock = (b: pc.VertexBuffer, offset?: number, length?: number) => (sent.push([offset ?? 0, length ?? vb.numBytes]), unlock(b, offset, length));
    const leaf = fences.gates[1];
    const box = fences.mesh.aabb;
    swingGate(fences, leaf, 0.5, 1);
    const n = leaf.rest.length / 3;
    const pos = vb.format.elements.find((e) => e.name === pc.SEMANTIC_POSITION)!;
    // Two spans (positions and normals), each the leaf's vertices only.
    expect(sent.length).toBe(2);
    for (const [, length] of sent) expect(length).toBe(n * 12);
    const after = new Float32Array(vb.lock() as ArrayBuffer);
    const at = (pos.offset + leaf.first * 12) / 4;
    expect(Array.from(after.subarray(at, at + n * 3))).toEqual(Array.from(swungGate(leaf, 0.5, 1).pos));
    // Nothing else moved.
    let changed = 0;
    for (let i = 0; i < after.length; i++) if (after[i] !== before[i]) changed++;
    expect(changed).toBeLessThanOrEqual(n * 6);
    // The box as it was, round both leaves standing open either way.
    expect(fences.mesh.aabb).toBe(box);
    for (const towards of [1, -1]) {
      const { pos: open } = swungGate(fences.gates[0], 1, towards);
      for (let i = 0; i < open.length; i += 3) expect(box.containsPoint(new pc.Vec3(open[i], open[i + 1], open[i + 2]))).toBe(true);
    }
  });

  test('the land moved under them: every gate and post lifted once by the ground change, in place, an open one still open', () => {
    const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
    const app = new pc.AppBase(canvas);
    const options = new pc.AppOptions();
    options.graphicsDevice = new pc.NullGraphicsDevice(canvas);
    options.componentSystems = [pc.RenderComponentSystem];
    options.resourceHandlers = [];
    app.init(options);
    // Two gates side by side (a post between them) and one apart, built on flat ground.
    const fences = buildFences(app, app.root, [gate(0, 0), gate(1, 0), gate(10, 0)], STYLES, { ground: () => 0 })!;
    swingGate(fences, fences.gates[1], 0.5, 1);
    const vb = fences.mesh.vertexBuffer;
    const pos = vb.format.elements.find((e) => e.name === pc.SEMANTIC_POSITION)!;
    const positions = () => {
      const f = new Float32Array(vb.lock() as ArrayBuffer);
      return Array.from({ length: vb.numVertices * 3 }, (_, k) => f[(pos.offset + Math.floor(k / 3) * pos.stride) / 4 + (k % 3)]);
    };
    const before = positions();
    const top = fences.mesh.aabb.getMax().y;
    // The land raised 1.5 m under the pair, not under the one apart.
    expect(regroundGates(fences, (x) => (x < 5 ? 1.5 : 0))).toBe(true);
    const after = positions();
    const apart = new Set<number>();
    for (const g of [fences.gates[2]]) for (let v = g.first; v < g.first + g.rest.length / 3; v++) apart.add(v);
    for (const p of fences.gates[2].posts) for (let v = p.first; v < p.first + p.count; v++) apart.add(v);
    for (let v = 0; v < vb.numVertices; v++) {
      const dy = apart.has(v) ? 0 : 1.5;
      expect(after[v * 3]).toBe(before[v * 3]);
      expect(after[v * 3 + 1]).toBeCloseTo(before[v * 3 + 1] + dy, 5);
      expect(after[v * 3 + 2]).toBe(before[v * 3 + 2]);
    }
    // The open leaf swings on from its new rest; the box holds the lifted ones.
    const leaf = fences.gates[1];
    const swung = Array.from(swungGate(leaf, 0.5, 1).pos);
    after.slice(leaf.first * 3, leaf.first * 3 + leaf.rest.length).forEach((v, k) => expect(v).toBeCloseTo(swung[k], 5));
    expect(fences.mesh.aabb.getMax().y).toBeGreaterThan(top + 1.4);
    expect(fences.mesh.aabb.getMax().y).toBeGreaterThanOrEqual(Math.max(...after.filter((_, k) => k % 3 === 1)));
    // The same ground again: nothing to move.
    expect(regroundGates(fences, (x) => (x < 5 ? 1.5 : 0))).toBe(false);
  });
});
