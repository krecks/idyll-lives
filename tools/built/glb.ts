/**
 * A minimal glTF binary writer for the built world's dump (`samples.ts`): one node and mesh a part, one primitive
 * each (POSITION, NORMAL, TEXCOORD_0, COLOR_0 and, where the part follows the cutaway, `_CUT`), one material a
 * name. Plain arrays in, the same bytes out for the same arrays: nothing here depends on the machine or the
 * time, so two dumps of one build are the same files.
 */

/** A part as dumped: what one draw call draws. */
export interface DumpMesh {
  /** The part's name as the game has it (`wallsExterior`, `silhouetteTrim`, `road`). */
  name: string;
  /** What it is drawn with: a manifest key (`material.wall.brick`) or one of `BUILT_MATERIALS`. */
  material: string;
  /** Three a vertex, metres, as PlayCanvas draws them (the game's z mirrored). */
  positions: Float32Array;
  normals: Float32Array;
  /** Two a vertex, in metres over the surface (a material's `uvScale` repeats its texture over them). */
  uvs: Float32Array;
  /** sRGB vertex colours, RGBA, a byte each. */
  colors: Uint8Array;
  /** The cutaway attribute (`Geo`'s: mask A, kind, mask B, 0), on parts that follow the cutaway. */
  cut?: Uint8Array;
  indices: Uint32Array;
  /** What the checks need to know of the part besides (the node's `extras`). */
  extras?: Record<string, string | number | boolean>;
}

const FLOAT = 5126;
const UBYTE = 5121;
const UINT = 5125;

/** The meshes as one `.glb` file. */
export function glbOf(meshes: readonly DumpMesh[]): Uint8Array {
  const chunks: Uint8Array[] = [];
  let length = 0;
  const bufferViews: object[] = [];
  const accessors: object[] = [];
  /** An accessor over `data` (`width` numbers an element), its bytes on a 4-byte boundary. */
  const accessor = (data: Float32Array | Uint8Array | Uint32Array, width: number, target: number | null, extra: object = {}): number => {
    const bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    bufferViews.push({ buffer: 0, byteOffset: length, byteLength: bytes.length, ...(target ? { target } : {}) });
    chunks.push(bytes);
    length += bytes.length;
    const pad = (4 - (length % 4)) % 4;
    if (pad) {
      chunks.push(new Uint8Array(pad));
      length += pad;
    }
    const componentType = data instanceof Float32Array ? FLOAT : data instanceof Uint8Array ? UBYTE : UINT;
    accessors.push({ bufferView: bufferViews.length - 1, componentType, count: data.length / width, type: width === 1 ? 'SCALAR' : `VEC${width}`, ...extra });
    return accessors.length - 1;
  };
  const materials: string[] = [];
  const nodes: object[] = [];
  const out: object[] = [];
  for (const m of meshes) {
    if (!m.indices.length) continue;
    let material = materials.indexOf(m.material);
    if (material < 0) material = materials.push(m.material) - 1;
    const [min, max] = [[Infinity, Infinity, Infinity], [-Infinity, -Infinity, -Infinity]];
    for (let i = 0; i < m.positions.length; i++) {
      min[i % 3] = Math.min(min[i % 3], m.positions[i]);
      max[i % 3] = Math.max(max[i % 3], m.positions[i]);
    }
    const attributes: Record<string, number> = {
      POSITION: accessor(m.positions, 3, 34962, { min, max }),
      NORMAL: accessor(m.normals, 3, 34962),
      TEXCOORD_0: accessor(m.uvs, 2, 34962),
      COLOR_0: accessor(m.colors, 4, 34962, { normalized: true }),
    };
    if (m.cut) attributes._CUT = accessor(m.cut, 4, 34962);
    out.push({ primitives: [{ attributes, indices: accessor(m.indices, 1, 34963), material }] });
    nodes.push({ name: m.name, mesh: out.length - 1, ...(m.extras && Object.keys(m.extras).length ? { extras: m.extras } : {}) });
  }
  const gltf = {
    asset: { version: '2.0', generator: 'tools/built/dump.ts' },
    scene: 0,
    scenes: [{ nodes: nodes.map((_, i) => i) }],
    nodes,
    meshes: out,
    materials: materials.map((name) => ({ name })),
    accessors,
    bufferViews,
    buffers: [{ byteLength: length }],
  };
  let json = new TextEncoder().encode(JSON.stringify(gltf));
  if (json.length % 4) {
    const padded = new Uint8Array(json.length + 4 - (json.length % 4)).fill(0x20);
    padded.set(json);
    json = padded;
  }
  const file = new Uint8Array(12 + 8 + json.length + 8 + length);
  const view = new DataView(file.buffer);
  view.setUint32(0, 0x46546c67, true);
  view.setUint32(4, 2, true);
  view.setUint32(8, file.length, true);
  view.setUint32(12, json.length, true);
  view.setUint32(16, 0x4e4f534a, true);
  file.set(json, 20);
  let at = 20 + json.length;
  view.setUint32(at, length, true);
  view.setUint32(at + 4, 0x004e4942, true);
  at += 8;
  for (const c of chunks) {
    file.set(c, at);
    at += c.length;
  }
  return file;
}
