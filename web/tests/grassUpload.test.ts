// The lawn's textures sent to the GPU as the lawn changes (`render/pc/grass.ts` `Grass.refresh`): only the rectangle
// round the changed texels of the town's mask (WebGPU's `writeTexture`, WebGL2's `texSubImage2D` with the copy's row
// length, put back after), the bytes of the CPU copy there; nothing before the texture's first draw sends it whole;
// the lawn's map only when it changed; a town of another size, textures of its own. On the engine's null device, its
// GPU calls recorded.
import { afterAll, describe, expect, mock, test } from 'bun:test';
import type { WorldMeta, WorldStructure } from '../src/core/protocol';
import * as pc from '../src/render/pc/engine';
import { GRASS, grassField, type GrassField, type GrassLand } from '../src/render/shared/grass';
import { keepBareTiles, lawnChanges, lawnMask } from '../src/render/shared/nature';
import { bigWorld } from './support/landFixtures';

// (The render code reads the page's flags as it loads.)
const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (The settings are a Svelte store, not for a test.)
mock.module('../src/settings/settings.svelte', () => ({ settings: { shading: 'soft' } }));
const { Grass } = await import('../src/render/pc/grass');
type Grass = InstanceType<typeof Grass>;

const flat: GrassLand = {
  heightAt: () => 0,
  terrainColour: (_x, _z, _h, out) => ((out.r = 0.5), (out.g = 0.7), (out.b = 0.3), 1),
  distanceToTown: () => 0,
};

type Inner = { app: unknown; field: GrassField; quality: unknown; grounds: unknown[]; townSize: Float32Array; town: pc.Texture; lawn: pc.Texture; lawnSent: Uint8Array; townTexture(): pc.Texture; lawnTexture(): pc.Texture };
type Internals = { impl: Record<string, unknown>; _needsUpload: boolean; device: unknown };

/** A grass on the null device for `world` (no draws: only what `refresh` reads), as the landscape keeps it. */
async function grassOf(world: WorldStructure) {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const field = await grassField(world, flat);
  const grass = Object.create(Grass.prototype) as Grass;
  const inner = grass as unknown as Inner;
  Object.assign(inner, { app: { graphicsDevice: new pc.NullGraphicsDevice(canvas) }, field, quality: { density: 1 }, grounds: [], townSize: new Float32Array(4) });
  inner.town = inner.townTexture();
  inner.lawn = inner.lawnTexture();
  inner.lawnSent = field.lawn.slice();
  let state = { world, mask: lawnMask(world) };
  keepBareTiles(world, state.mask);
  /** `next` as the landscape's `refreshLawn` gives it. */
  const refresh = (next: WorldStructure) => {
    const rects = lawnChanges(state.world, next);
    const change = rects && { was: state.world, rects };
    const mask = lawnMask(next, state.mask, rects);
    grass.refresh(next, mask, keepBareTiles(next, mask, change), change);
    state = { world: next, mask };
  };
  return { grass, inner, field, refresh };
}

/** The town's texels (`[mask, shade]` each) as grown afresh for `world`. */
async function townBytes(world: WorldStructure): Promise<Uint8Array> {
  const f = await grassField(world, flat);
  const out = new Uint8Array(f.town.length * 2);
  for (let i = 0; i < f.town.length; i++) [out[i * 2], out[i * 2 + 1]] = [f.town[i], f.shade[i]];
  return out;
}

/** The rows `h` of `w` texels (2 bytes each) at `offset` in `data`, `stride` bytes apart. */
const rows = (data: Uint8Array, offset: number, stride: number, w: number, h: number) => Array.from({ length: h }, (_, r) => [...data.subarray(offset + r * stride, offset + r * stride + w * 2)]);

const thing = (id: number, def: string, x: number, z: number) => ({ id, def, x, z, rot: 0, w: 1, d: 1 }) as WorldStructure['objects'][number];
const first = () => ({ ...bigWorld({ seed: 3 } as WorldMeta, [], 24), walls: [{ axis: 'h', x: 8, z: 8 }] }) as WorldStructure;

describe("the lawn's textures sent as it changes", () => {
  test('WebGPU: the rectangle round the change, from the CPU copy; the lawn map only when it changed; none before the first draw', async () => {
    const world = first();
    const { inner, refresh } = await grassOf(world);
    const sent: { origin: number[]; data: Uint8Array; layout: GPUTexelCopyBufferLayout; size: { width: number; height: number } }[] = [];
    const device = inner.app as { graphicsDevice: pc.GraphicsDevice };
    Object.defineProperty(device.graphicsDevice, 'isWebGPU', { value: true });
    Object.assign(device.graphicsDevice, { wgpu: { queue: { writeTexture: (dst: { origin: number[] }, data: Uint8Array, layout: GPUTexelCopyBufferLayout, size: { width: number; height: number }) => sent.push({ origin: dst.origin, data, layout, size }) } } });
    const town = inner.town as pc.Texture & Internals;
    [town._needsUpload, town.impl.gpuTexture] = [false, {}];
    let lawnSends = 0;
    const unlock = inner.lawn.unlock.bind(inner.lawn);
    inner.lawn.unlock = () => (lawnSends++, unlock());
    // A bench on the lawn: the town's mask, its shade and the lawn's worn earth under it.
    const bench = { ...world, objects: [thing(1, 'parkBench', 12, 12)] };
    refresh(bench);
    expect(sent.length).toBe(1);
    const [{ origin, data, layout, size }] = sent;
    const n = GRASS.townTexels;
    expect(origin[0]).toBeGreaterThan(0);
    expect(origin[0] + size.width).toBeLessThan(24 * n);
    expect(layout.bytesPerRow).toBe(town.width * 2);
    expect(data).toBe(town.getSource() as unknown as Uint8Array);
    const fresh = await townBytes(bench);
    expect(rows(data, layout.offset!, layout.bytesPerRow!, size.width, size.height)).toEqual(rows(fresh, layout.offset!, layout.bytesPerRow!, size.width, size.height));
    expect(Buffer.from(data).equals(Buffer.from(fresh))).toBe(true);
    expect(lawnSends).toBe(1);
    expect(Buffer.from(inner.lawnSent).equals(Buffer.from(inner.field.lawn))).toBe(true);
    // A stool (shade, no worn earth): the mask's rectangle sent, the lawn's map not.
    const stool = { ...bench, objects: [...bench.objects, thing(2, 'stool', 4, 16)] };
    refresh(stool);
    expect(sent.length).toBe(2);
    expect(lawnSends).toBe(1);
    // Nothing changed: nothing sent.
    refresh({ ...stool, version: 2 });
    expect(sent.length).toBe(2);
    // Not drawn yet: its first draw sends it whole, the CPU copy made as the world is.
    town._needsUpload = true;
    const moved = { ...stool, objects: [thing(1, 'parkBench', 16, 4), stool.objects[1]] };
    refresh(moved);
    expect(sent.length).toBe(2);
    expect(Buffer.from(town.getSource() as unknown as Uint8Array).equals(Buffer.from(await townBytes(moved)))).toBe(true);
  });

  test('WebGL2: the rectangle at the copy\'s row length, put back to 0 after', async () => {
    const world = first();
    const { inner, refresh } = await grassOf(world);
    const calls: unknown[][] = [];
    const gl = { TEXTURE_2D: 1, RG: 2, UNSIGNED_BYTE: 3, UNPACK_ROW_LENGTH: 4, pixelStorei: (...a: unknown[]) => calls.push(['pixelStorei', ...a]), texSubImage2D: (...a: unknown[]) => calls.push(['texSubImage2D', ...a]) };
    const device = Object.assign(Object.create(pc.WebglGraphicsDevice.prototype), { gl, activeTexture() {}, bindTexture() {}, setUnpackFlipY() {}, setUnpackPremultiplyAlpha() {}, setUnpackAlignment() {} });
    const town = inner.town as pc.Texture & Internals;
    [town._needsUpload, town.impl._glTexture, town.device] = [false, {}, device];
    const bench = { ...world, objects: [thing(1, 'parkBench', 12, 12)] };
    refresh(bench);
    expect(calls.map((c) => c[0])).toEqual(['pixelStorei', 'texSubImage2D', 'pixelStorei']);
    expect(calls[0].slice(1)).toEqual([4, town.width]);
    expect(calls[2].slice(1)).toEqual([4, 0]);
    const [, target, level, x, z, w, h, format, type, data, offset] = calls[1] as [string, number, number, number, number, number, number, number, number, Uint8Array, number];
    expect([target, level, format, type]).toEqual([1, 0, 2, 3]);
    expect(offset).toBe((z * town.width + x) * 2);
    const fresh = await townBytes(bench);
    expect(rows(data, offset, town.width * 2, w, h)).toEqual(rows(fresh, offset, town.width * 2, w, h));
    expect(Buffer.from(data).equals(Buffer.from(fresh))).toBe(true);
  });

  test('a town of another size: textures of its own, sized to it, the lawn map kept as sent', async () => {
    const { inner, refresh } = await grassOf(first());
    const [town, lawn] = [inner.town, inner.lawn];
    const bigger = { ...bigWorld({ seed: 3 } as WorldMeta, [], 32), walls: [{ axis: 'h', x: 8, z: 8 }] } as WorldStructure;
    refresh(bigger);
    expect(inner.town).not.toBe(town);
    expect(inner.lawn).not.toBe(lawn);
    expect([inner.town.width, inner.town.height]).toEqual([inner.field.townW, inner.field.townD]);
    expect(inner.field.townW).toBe(32 * GRASS.townTexels);
    expect(Buffer.from(inner.town.getSource() as unknown as Uint8Array).equals(Buffer.from(await townBytes(bigger)))).toBe(true);
    expect(Buffer.from(inner.lawnSent).equals(Buffer.from(inner.field.lawn))).toBe(true);
  });
});
