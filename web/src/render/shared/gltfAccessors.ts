/**
 * A glTF file's accessors read into arrays (`pc/models.ts` reads a file's geometry and clips from its
 * buffer views itself): floats as they are, normalised integers to 0..1 or -1..1, triangles straight
 * into 32-bit integers. Through a typed array over the view where its bytes line up (most files), else
 * value by value through a DataView: the same numbers either way (`tests/gltfAccessors.test.ts`).
 */

/** The parts of a glTF file read here: its JSON's accessors and buffer views, and the views' bytes. */
export interface AccessorFile {
  gltf: { accessors?: GltfAccessor[]; bufferViews?: { byteStride?: number }[] };
  views: (Uint8Array | undefined)[];
}

export interface GltfAccessor {
  bufferView?: number;
  byteOffset?: number;
  componentType: number;
  normalized?: boolean;
  count: number;
  type: string;
  sparse?: unknown;
}

const COMPONENTS: Record<string, number> = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4, MAT2: 4, MAT3: 9, MAT4: 16 };
const SIZES: Record<number, number> = { 5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4 };
/** The typed array each component type is read through (`readAccessor`'s fast path). */
const TYPED: Record<number, new (buffer: ArrayBufferLike, byteOffset: number, length: number) => ArrayLike<number>> = {
  5120: Int8Array,
  5121: Uint8Array,
  5122: Int16Array,
  5123: Uint16Array,
  5125: Uint32Array,
  5126: Float32Array,
};
/** The page's typed arrays are little-endian, as glTF's buffers are (every browser's are; the fast path checks). */
const LITTLE_ENDIAN = new Uint8Array(new Uint16Array([1]).buffer)[0] === 1;

/**
 * An accessor's components as a typed array over its buffer view (`TYPED`), with its stride in
 * components: no copy. Null where the view's bytes don't line up with the type (an odd offset or
 * stride), which `readAccessor` reads value by value instead.
 */
function typedAccessor(file: AccessorFile, a: GltfAccessor, components: number, size: number): { src: ArrayLike<number>; step: number } | null {
  if (!LITTLE_ENDIAN || a.bufferView === undefined) return null;
  const bytes = file.views[a.bufferView];
  if (!bytes) return null;
  const stride = file.gltf.bufferViews?.[a.bufferView]?.byteStride ?? components * size;
  const start = bytes.byteOffset + (a.byteOffset ?? 0);
  if (start % size || stride % size) return null;
  const length = a.count ? ((a.count - 1) * stride) / size + components : 0;
  if (start + length * size > bytes.byteOffset + bytes.byteLength) return null;
  return { src: new TYPED[a.componentType](bytes.buffer, start, length), step: stride / size };
}

/** An accessor's values as floats (normalised integers to 0..1 or -1..1), with their count per element. */
export function readAccessor(file: AccessorFile, index: number): { values: Float32Array; components: number } {
  const a = file.gltf.accessors![index];
  const components = COMPONENTS[a.type];
  const size = SIZES[a.componentType];
  if (!components || !size) throw new Error(`accessor ${index}: unknown type ${a.type}/${a.componentType}`);
  if (a.sparse) throw new Error(`accessor ${index}: sparse accessors aren't read`);
  if (a.bufferView === undefined) return { values: new Float32Array(a.count * components), components };
  const bytes = file.views[a.bufferView];
  if (!bytes) throw new Error(`accessor ${index}: buffer view ${a.bufferView} wasn't loaded`);
  // Through a typed array over the view where its bytes line up (most files): the same numbers as the
  // DataView below, without a call per value.
  const typed = typedAccessor(file, a, components, size);
  if (typed) {
    const { src, step } = typed;
    const n = a.count * components;
    // (Floats packed tight: one copy.)
    if (a.componentType === 5126 && step === components) return { values: Float32Array.from(src), components };
    const values = new Float32Array(n);
    const scale = !a.normalized ? 0 : ({ 5120: 127, 5121: 255, 5122: 32767, 5123: 65535 } as Record<number, number>)[a.componentType] ?? 0;
    const signed = a.componentType === 5120 || a.componentType === 5122;
    for (let k = 0; k < a.count; k++) {
      for (let c = 0; c < components; c++) {
        const v = src[k * step + c];
        values[k * components + c] = !scale ? v : signed ? Math.max(v / scale, -1) : v / scale;
      }
    }
    return { values, components };
  }
  const values = new Float32Array(a.count * components);
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const stride = file.gltf.bufferViews?.[a.bufferView]?.byteStride ?? components * size;
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
  return { values, components };
}

/** An accessor of whole numbers (a primitive's triangles) as they are, straight into 32 bits. */
export function readIndices(file: AccessorFile, index: number): Uint32Array {
  const a = file.gltf.accessors![index];
  const typed = a.type === 'SCALAR' && SIZES[a.componentType] && !a.sparse ? typedAccessor(file, a, 1, SIZES[a.componentType]) : null;
  if (!typed) return Uint32Array.from(readAccessor(file, index).values);
  const out = new Uint32Array(a.count);
  const { src, step } = typed;
  for (let k = 0; k < a.count; k++) out[k] = src[k * step];
  return out;
}
