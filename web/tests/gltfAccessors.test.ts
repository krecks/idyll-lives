// A glTF accessor read through a typed array over its view (where its bytes line up) gives the numbers
// `readAccessor` read value by value through a DataView before it had that path (`reference`, kept here as it
// was), and so does the same accessor laid out one byte off (where the typed path can't be taken for wider
// types). Every component type, normalised or not, packed or strided (meshopt's quantised files are normalised
// integers with padded strides), and triangles straight into 32 bits.
import { describe, expect, test } from 'bun:test';
import { readAccessor, readIndices, type AccessorFile, type GltfAccessor } from '../src/render/shared/gltfAccessors';

const SIZE: Record<number, number> = { 5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4 };
const COMPONENTS: Record<string, number> = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4 };

/** Random values of the type, written little-endian, `count` elements `stride` bytes apart, after `lead` bytes and an offset of `offset`. */
function file(componentType: number, type: string, count: number, stride: number | undefined, offset: number, lead: number, normalized: boolean, seed: number): AccessorFile {
  const size = SIZE[componentType];
  const components = COMPONENTS[type];
  const step = stride ?? size * components;
  const bytes = new Uint8Array(lead + offset + step * count + 8);
  const view = new DataView(bytes.buffer, lead);
  let s = seed;
  const rand = () => ((s = (Math.imul(s, 1103515245) + 12345) >>> 0) / 4294967296);
  for (let k = 0; k < count; k++) {
    for (let c = 0; c < components; c++) {
      const at = offset + k * step + c * size;
      const r = rand();
      if (componentType === 5120) view.setInt8(at, Math.floor(r * 256) - 128);
      else if (componentType === 5121) view.setUint8(at, Math.floor(r * 256));
      else if (componentType === 5122) view.setInt16(at, Math.floor(r * 65536) - 32768, true);
      else if (componentType === 5123) view.setUint16(at, Math.floor(r * 65536), true);
      else if (componentType === 5125) view.setUint32(at, Math.floor(r * 2 ** 24), true);
      else view.setFloat32(at, (r - 0.5) * 1000, true);
    }
  }
  const accessor: GltfAccessor = { bufferView: 0, byteOffset: offset, componentType, normalized, count, type };
  return { gltf: { accessors: [accessor], bufferViews: [stride ? { byteStride: stride } : {}] }, views: [new Uint8Array(bytes.buffer, lead, bytes.length - lead)] };
}

/** The accessor read value by value through a DataView, as `readAccessor` read every accessor before. */
function reference(f: AccessorFile): number[] {
  const a = f.gltf.accessors![0];
  const components = COMPONENTS[a.type];
  const size = SIZE[a.componentType];
  const values = new Float32Array(a.count * components);
  const bytes = f.views[0]!;
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const stride = f.gltf.bufferViews?.[0]?.byteStride ?? components * size;
  const read: (at: number) => number = {
    5120: (at: number) => (a.normalized ? Math.max(view.getInt8(at) / 127, -1) : view.getInt8(at)),
    5121: (at: number) => (a.normalized ? view.getUint8(at) / 255 : view.getUint8(at)),
    5122: (at: number) => (a.normalized ? Math.max(view.getInt16(at, true) / 32767, -1) : view.getInt16(at, true)),
    5123: (at: number) => (a.normalized ? view.getUint16(at, true) / 65535 : view.getUint16(at, true)),
    5125: (at: number) => view.getUint32(at, true),
    5126: (at: number) => view.getFloat32(at, true),
  }[a.componentType]!;
  const base = a.byteOffset ?? 0;
  for (let k = 0; k < a.count; k++) for (let c = 0; c < components; c++) values[k * components + c] = read(base + k * stride + c * size);
  return Array.from(values);
}

const cases: [number, string, number | undefined, boolean][] = [];
for (const t of [5120, 5121, 5122, 5123]) for (const type of ['VEC2', 'VEC3', 'VEC4']) for (const normalized of [false, true]) for (const stride of [undefined, 8, 12]) cases.push([t, type, stride, normalized]);
for (const type of ['SCALAR', 'VEC2', 'VEC3', 'VEC4']) for (const stride of [undefined, 16, 20]) cases.push([5126, type, stride, false]);
cases.push([5125, 'SCALAR', undefined, false]);

describe('accessors read through a typed array are read as the DataView reads them', () => {
  test.each(cases)('type %i %s stride %p normalised %p', (componentType, type, stride, normalized) => {
    const size = SIZE[componentType];
    // (A stride too short for the element isn't a file's: skip it.)
    if (stride && stride < size * COMPONENTS[type]) return;
    for (const offset of [0, 4]) {
      const fast = file(componentType, type, 37, stride, offset, 8, normalized, 7);
      // One byte off: the typed path doesn't line up, so the DataView reads it.
      const slow = file(componentType, type, 37, stride, offset, 9, normalized, 7);
      const a = readAccessor(fast, 0);
      const b = readAccessor(slow, 0);
      expect(a.components).toBe(COMPONENTS[type]);
      expect(Array.from(a.values)).toEqual(reference(fast));
      expect(Array.from(b.values)).toEqual(reference(slow));
      expect(Array.from(a.values)).toEqual(Array.from(b.values));
    }
  });

  test.each([5121, 5123, 5125])('triangles of type %i, packed and strided', (componentType) => {
    for (const stride of [undefined, 8]) {
      const fast = file(componentType, 'SCALAR', 61, stride, 4, 8, false, 3);
      const slow = file(componentType, 'SCALAR', 61, stride, 4, 9, false, 3);
      const a = readIndices(fast, 0);
      expect(a).toBeInstanceOf(Uint32Array);
      expect(Array.from(a)).toEqual(Array.from(readIndices(slow, 0)));
      expect(Array.from(a)).toEqual(reference(slow));
    }
  });
});
