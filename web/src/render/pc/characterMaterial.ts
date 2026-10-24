/**
 * The residents' materials on PlayCanvas (the port of `babylon/characters/material.ts` and the
 * material half of the Babylon renderer's characters): standard materials without textures, every
 * colour the resident's own from its row of the pose texture and the part's per-vertex fields and
 * tone, skinned on the GPU from the same row (`shaders/character.ts`). The bed quilts' material too
 * (`blanketMaterial`, `shaders/blanket.ts`).
 *
 * PlayCanvas's own skinning and morphs aren't used: a skinned mesh instance can't be instanced, and
 * its morphs take the instancing attributes. A part is one instanced draw for everyone wearing it.
 *
 * What a part's draw needs:
 * - Its mesh in the resident layout (`residentMesh`, one interleaved vertex buffer, `RESIDENT_VERTEX`):
 *   position, normal, four bone indices (bytes) and weights (normalised bytes), the fields, the
 *   face morph block (x -1: none) and the tone (`MeshPart`, `shared/characters/data.ts`).
 * - Per copy, beside its matrix (the identity: the bones carry the placement), the resident's row
 *   (`RESIDENT_ROW`), as `Instances` (`instancing.ts`) draws it; never culled (`cull: false`: the
 *   mesh's bounds are its bind pose's at the origin).
 * - The app's pose texture and the set's morph texture (`setResidentTextures`, once).
 * - Its material, `characterMaterial(app, part)`: one per surface (and per mouth part), shared by
 *   every body; `morph` for parts face morphs move; `shadow` for the shadow-only copies (no morphs,
 *   and out of the library's lit materials: switching the look never compiles them again).
 *
 * The lit materials are the library's (`MaterialLibrary.adopt`): the soft look, the lamps in their
 * rooms, basements and windowless rooms, fog and the diag views read the skinned position and normal.
 */

import * as pc from './engine';
import type { InstanceStream } from './instancing';
import { GAME_FRONT_FACE, MaterialLibrary } from './materials';
import { BLANKET } from './shaders/blanket';
import { RESIDENT, RESIDENT_INSTANCING, RESIDENT_NORMAL } from './shaders/character';
import { engineChunk, gameChunks, installGameChunks, registerShaderFeature, replaceOnce, type Language } from './shaders/chunks';
import { BLANKET_WIDTH, COVER, SHEET } from '../shared/characters/blanket';
import type { MeshPart, MorphSet } from '../shared/characters/data';
import { surfaceOf, type CharacterSurface } from '../shared/characters/parts';

export { quiltTint } from '../shared/characters/blanket';
export { surfaceOf, type CharacterSurface } from '../shared/characters/parts';

// (Before any device's chunks are built: they're built with every feature registered, once per device.)
registerShaderFeature(RESIDENT);
registerShaderFeature(BLANKET);

/** The vertex attributes of a resident part (one interleaved buffer; see `residentVertexFormat`). */
export const RESIDENT_VERTEX = {
  joints: pc.SEMANTIC_BLENDINDICES,
  weights: pc.SEMANTIC_BLENDWEIGHT,
  fields: pc.SEMANTIC_ATTR7,
  morph: pc.SEMANTIC_ATTR8,
  tone: pc.SEMANTIC_ATTR9,
} as const;

/** The resident's row of the pose texture (0..63, a float), per copy beside its matrix. */
export const RESIDENT_ROW: InstanceStream = { semantic: pc.SEMANTIC_ATTR10, components: 1 };
/** A quilt's stretch and pull (m, local; `shaders/blanket.ts`), per copy. */
export const BLANKET_EXT: InstanceStream = { semantic: pc.SEMANTIC_ATTR8, components: 2 };
/** A quilt's bedding colour (linear rgb, `quiltTint`), per copy. */
export const BLANKET_TINT: InstanceStream = { semantic: pc.SEMANTIC_ATTR9, components: 3 };

/** The matrix's columns (`instance_line1..4`), as `instancing.ts` writes them: named by hand, the instancing transform being ours. */
const INSTANCE_LINES: [string, string][] = [
  ['instance_line1', pc.SEMANTIC_ATTR11],
  ['instance_line2', pc.SEMANTIC_ATTR12],
  ['instance_line3', pc.SEMANTIC_ATTR14],
  ['instance_line4', pc.SEMANTIC_ATTR15],
];

const formats = new WeakMap<pc.GraphicsDevice, pc.VertexFormat>();

/**
 * A resident part's vertex format, interleaved (80 bytes a vertex): POSITION and NORMAL (3 floats),
 * the bone indices (BLENDINDICES, 4 bytes) and weights (BLENDWEIGHT, 4 normalised bytes), then the
 * fields (ATTR7), the morph block (ATTR8) and the tone (ATTR9), 4 floats each.
 */
export function residentVertexFormat(device: pc.GraphicsDevice): pc.VertexFormat {
  let format = formats.get(device);
  if (!format) {
    const f32 = (semantic: string, components: number) => ({ semantic, components, type: pc.TYPE_FLOAT32 });
    format = new pc.VertexFormat(device, [
      f32(pc.SEMANTIC_POSITION, 3),
      f32(pc.SEMANTIC_NORMAL, 3),
      { semantic: RESIDENT_VERTEX.joints, components: 4, type: pc.TYPE_UINT8 },
      { semantic: RESIDENT_VERTEX.weights, components: 4, type: pc.TYPE_UINT8, normalize: true },
      f32(RESIDENT_VERTEX.fields, 4),
      f32(RESIDENT_VERTEX.morph, 4),
      f32(RESIDENT_VERTEX.tone, 4),
    ]);
    formats.set(device, format);
  }
  return format;
}

/** A resident part's mesh (`residentVertexFormat`), its bounds the part's at rest. The caller owns it. */
export function residentMesh(device: pc.GraphicsDevice, part: MeshPart): pc.Mesh {
  const format = residentVertexFormat(device);
  const n = part.positions.length / 3;
  const stride = format.size;
  const bytes = new ArrayBuffer(n * stride);
  const f = new Float32Array(bytes);
  const b = new Uint8Array(bytes);
  const at: Record<string, number> = {};
  for (const e of format.elements) at[e.name] = e.offset;
  const fs = stride / 4;
  for (let v = 0; v < n; v++) {
    const o = v * fs;
    const put = (semantic: string, src: ArrayLike<number>, k: number, count: number) => {
      const p = o + at[semantic] / 4;
      for (let c = 0; c < count; c++) f[p + c] = src[k + c];
    };
    put(pc.SEMANTIC_POSITION, part.positions, v * 3, 3);
    put(pc.SEMANTIC_NORMAL, part.normals, v * 3, 3);
    const jb = v * stride + at[RESIDENT_VERTEX.joints];
    const wb = v * stride + at[RESIDENT_VERTEX.weights];
    for (let c = 0; c < 4; c++) {
      b[jb + c] = part.joints[v * 4 + c];
      b[wb + c] = Math.round(Math.min(1, Math.max(0, part.weights[v * 4 + c])) * 255);
    }
    put(RESIDENT_VERTEX.fields, part.fields, v * 4, 4);
    if (part.morph) put(RESIDENT_VERTEX.morph, part.morph, v * 4, 4);
    else f[o + at[RESIDENT_VERTEX.morph] / 4] = -1;
    put(RESIDENT_VERTEX.tone, part.tone, v * 4, 4);
  }
  const mesh = new pc.Mesh(device);
  mesh.vertexBuffer = new pc.VertexBuffer(device, format, n, { data: bytes });
  mesh.indexBuffer[0] = new pc.IndexBuffer(device, pc.INDEXFORMAT_UINT16, part.indices.length, pc.BUFFER_STATIC, part.indices);
  mesh.primitive[0] = { type: pc.PRIMITIVE_TRIANGLES, base: 0, baseVertex: 0, count: part.indices.length, indexed: true };
  const box = new pc.BoundingBox();
  box.setMinMax(new pc.Vec3(...part.min), new pc.Vec3(...part.max));
  mesh.aabb = box;
  return mesh;
}

/** The set's face morphs as a texture (half floats, RGBA), for `setResidentTextures`. */
export function createMorphTexture(device: pc.GraphicsDevice, morphs: MorphSet): pc.Texture {
  return new pc.Texture(device, {
    name: 'simMorphs',
    width: morphs.width,
    height: morphs.height,
    format: pc.PIXELFORMAT_RGBA16F,
    mipmaps: false,
    minFilter: pc.FILTER_NEAREST,
    magFilter: pc.FILTER_NEAREST,
    addressU: pc.ADDRESS_CLAMP_TO_EDGE,
    addressV: pc.ADDRESS_CLAMP_TO_EDGE,
    levels: [morphs.data],
  });
}

export interface ResidentTextures {
  /** The pose texture: RGBA32F, a row per resident (`shared/characters/layout.ts`), nearest, no mipmaps (`PoseTexture`). */
  poses: pc.Texture;
  /** Bones in the set: a row's appearance texels start at `bones * 4`. */
  bones: number;
  /** The set's face morphs (`createMorphTexture`), null without. */
  morphs: pc.Texture | null;
  /** `MorphSet.width` and `headerTexels`. */
  morphWidth: number;
  morphHeader: number;
}

/** What an app's resident materials read, and the materials made so far. */
interface AppResidents {
  poses: pc.Texture;
  morphs: pc.Texture;
  /** `game_simLayout`: first appearance texel, morph width, morph header (read at every draw: changed in place). */
  layout: Float32Array;
  materials: Map<string, pc.StandardMaterial>;
}

const apps = new WeakMap<pc.AppBase, AppResidents>();

function residentsOf(app: pc.AppBase): AppResidents {
  let r = apps.get(app);
  if (!r) {
    const device = app.graphicsDevice;
    const none = (format: number, levels: Float32Array[] | Uint16Array[]) =>
      new pc.Texture(device, { name: 'simNone', width: 1, height: 1, format, mipmaps: false, minFilter: pc.FILTER_NEAREST, magFilter: pc.FILTER_NEAREST, levels });
    r = {
      poses: none(pc.PIXELFORMAT_RGBA32F, [new Float32Array(4)]),
      morphs: none(pc.PIXELFORMAT_RGBA16F, [new Uint16Array(4)]),
      layout: new Float32Array([0, 1, 0, 0]),
      materials: new Map(),
    };
    apps.set(app, r);
  }
  return r;
}

function bind(r: AppResidents, material: pc.StandardMaterial): void {
  material.setParameter('game_simPoses', r.poses);
  material.setParameter('game_simMorphs', r.morphs);
  material.setParameter('game_simLayout', r.layout);
}

/** The textures every resident material of `app` reads, now and later (once a set is loaded; no shader compiles). */
export function setResidentTextures(app: pc.AppBase, textures: ResidentTextures): void {
  const r = residentsOf(app);
  r.poses = textures.poses;
  if (textures.morphs) r.morphs = textures.morphs;
  r.layout[0] = textures.bones * 4;
  r.layout[1] = Math.max(1, textures.morphWidth);
  r.layout[2] = textures.morphHeader;
  for (const m of r.materials.values()) bind(r, m);
}

export interface CharacterMaterialOptions {
  /** The part has face morphs (`MeshPart.morph`): they move it. */
  morph?: boolean;
  /** For the shadow-only copies (`shadowOnlyLayer`): skinned, no morphs, not one of the library's lit materials. */
  shadow?: boolean;
}

/** Linear to PlayCanvas's sRGB colour properties. */
const srgb = (v: number): number => Math.pow(v, 1 / 2.2);

/**
 * The material of `part` (of any body) for `app`, made the first time. Every resident's colours come
 * from its row; the material's own are Babylon's (white, the teeth's and tongue's).
 */
export function characterMaterial(app: pc.AppBase, part: string, options: CharacterMaterialOptions = {}): pc.StandardMaterial {
  const kind = surfaceOf(part);
  const shadow = !!options.shadow;
  const morph = !shadow && !!options.morph;
  const key = `${shadow ? 'shadow:' : ''}${kind === 'mouth' ? `mouth:${part}` : kind}${morph ? ':morph' : ''}`;
  const r = residentsOf(app);
  const have = r.materials.get(key);
  if (have) return have;
  const device = app.graphicsDevice;
  if (!gameChunks(device).glsl.litUserDeclarationVS.includes('GAME_RESIDENT')) throw new Error('[render] resident shader feature registered after the first material was made');

  const mat = new pc.StandardMaterial();
  mat.name = `sim:${key}`;
  mat.frontFace = GAME_FRONT_FACE;
  mat.useMetalness = true;
  mat.metalness = 0;
  // Roughness, as PlayCanvas's inverted gloss.
  mat.glossInvert = true;
  const rough: Record<CharacterSurface, number> = { body: 0.55, cloth: 0.85, hair: 0.5, brow: 0.6, lash: 0.6, mouth: 0.5, eye: 0.25 };
  mat.gloss = rough[kind];
  // (Babylon's albedo colours, linear.)
  const albedo = part === 'teeth' ? [1, 0.96, 0.9] : part === 'tongue' ? [0.85, 0.8, 0.8] : kind === 'lash' ? [0.16, 0.12, 0.1] : [1, 1, 1];
  mat.diffuse.set(srgb(albedo[0]), srgb(albedo[1]), srgb(albedo[2]));
  if (kind === 'cloth' || kind === 'hair' || kind === 'brow' || kind === 'lash') {
    // Open hems and collars, hair cards and lashes show their insides.
    mat.cull = pc.CULLFACE_NONE;
    mat.twoSidedLighting = true;
  }
  if (kind === 'mouth') {
    // Warm, soft and hardly lit by the sky (the mouth hides most of it): Babylon's environment intensity.
    mat.reflectivity = 0.35;
    mat.ambient.set(srgb(0.35), srgb(0.35), srgb(0.35));
  }
  if (kind === 'eye') {
    // Wet and glossy: a clear coat for the catchlight.
    mat.clearCoat = 1;
    mat.clearCoatGloss = 0.97;
  }

  mat.setAttribute('game_simJoints', RESIDENT_VERTEX.joints);
  mat.setAttribute('game_simWeights', RESIDENT_VERTEX.weights);
  mat.setAttribute('game_simField', RESIDENT_VERTEX.fields);
  mat.setAttribute('game_simMorph', RESIDENT_VERTEX.morph);
  mat.setAttribute('game_simTone', RESIDENT_VERTEX.tone);
  mat.setAttribute('game_simRow', RESIDENT_ROW.semantic);
  for (const [name, semantic] of INSTANCE_LINES) mat.setAttribute(name, semantic);
  for (const lang of ['glsl', 'wgsl'] as const) {
    const chunks = mat.getShaderChunks(lang === 'glsl' ? pc.SHADERLANGUAGE_GLSL : pc.SHADERLANGUAGE_WGSL);
    chunks.set('transformInstancingVS', RESIDENT_INSTANCING[lang]);
    chunks.set('normalCoreVS', residentNormalCore(device, lang));
  }
  mat.setDefine('GAME_RESIDENT', '');
  mat.setDefine(`GAME_SIM_${kind.toUpperCase()}`, '');
  if (morph) mat.setDefine('GAME_SIM_MORPH', '');
  bind(r, mat);
  if (shadow) {
    installGameChunks(mat, device);
    mat.update();
  } else {
    MaterialLibrary.for(app).adopt(mat);
  }
  r.materials.set(key, mat);
  return mat;
}

/** `normalCoreVS` with `getLocalNormal` wrapped (the face morphs' normals), per language and device. */
function residentNormalCore(device: pc.GraphicsDevice, lang: Language): string {
  const { anchor, code } = RESIDENT_NORMAL[lang];
  const engine = engineChunk(device, lang, 'normalCoreVS');
  return `${replaceOnce(engine, 'normalCoreVS', anchor, anchor.replace('getLocalNormal(', 'getLocalNormalEngine('))}\n${code}`;
}

/**
 * The quilts' material for `app` (Babylon's `blanketMaterial`): a soft quilted cover with a
 * turned-down white sheet band at the top, the cover grey so each copy takes its bed's colour over
 * it (`BLANKET_TINT`, `quiltTint`). `toeZ` and `footZ` (local z) are the stretch's range (`BLANKET_EXT`).
 * Its draws: position, normal and uv (u across, v metres from the top edge) per vertex, the matrix,
 * `BLANKET_EXT` and `BLANKET_TINT` per copy.
 */
export function blanketMaterial(app: pc.AppBase, toeZ: number, footZ: number): pc.StandardMaterial {
  const device = app.graphicsDevice;
  const size = 512;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = size;
  const ctx = canvas.getContext('2d')!;
  // Drawn as Babylon's (whose texture rows run bottom-up in v), turned over for PlayCanvas's (top-down).
  ctx.translate(0, size);
  ctx.scale(1, -1);
  // v (texture y) spans 2 m; the top `SHEET` is the sheet.
  const perM = size / 2;
  const cover = COVER.toString(16);
  ctx.fillStyle = `#${cover}${cover}${cover}`;
  ctx.fillRect(0, 0, size, size);
  // Quilting: diamond stitch lines with soft puffed shading between them.
  const step = 0.16 * perM;
  for (let k = -size; k < size * 2; k += step) {
    for (const dir of [1, -1]) {
      ctx.strokeStyle = 'rgba(40,40,40,0.35)';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(k, 0);
      ctx.lineTo(k + dir * size, size);
      ctx.stroke();
      ctx.strokeStyle = 'rgba(255,255,255,0.12)';
      ctx.lineWidth = 6;
      ctx.beginPath();
      ctx.moveTo(k + step / 2, 0);
      ctx.lineTo(k + step / 2 + dir * size, size);
      ctx.stroke();
    }
  }
  // Turned-down sheet with a piped edge.
  ctx.fillStyle = '#EEEBE4';
  ctx.fillRect(0, size - SHEET * perM, size, SHEET * perM);
  ctx.fillStyle = '#C9C3B8';
  ctx.fillRect(0, size - SHEET * perM - 3, size, 3);
  ctx.fillStyle = '#D9D4CB';
  ctx.fillRect(0, size - 0.03 * perM - 2, size, 2);
  // (Linear bytes, decoded by the material as the game's colour maps are: `textureWithFallback`.)
  const tex = new pc.Texture(device, { name: 'simBlanket', width: size, height: size, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
  tex.setSource(canvas);
  // (Babylon's textures' default anisotropic filtering.)
  tex.anisotropy = 4;

  const mat = new pc.StandardMaterial();
  mat.name = 'simBlanket';
  mat.frontFace = GAME_FRONT_FACE;
  mat.diffuseMap = tex;
  // Babylon's `vScale` 0.5 (v' = v · 0.5: PlayCanvas's tiling is from the top, `1 - tiling - offset`).
  mat.diffuseMapTiling = new pc.Vec2(1 / BLANKET_WIDTH, 0.5);
  mat.diffuseMapOffset = new pc.Vec2(0, 0.5);
  mat.useMetalness = true;
  mat.metalness = 0;
  mat.glossInvert = true;
  mat.gloss = 0.88;
  // Babylon's sheen: white at 0.45, as rough as the cloth.
  mat.useSheen = true;
  mat.sheen.set(srgb(0.45), srgb(0.45), srgb(0.45));
  mat.sheenGloss = 1 - 0.88;
  mat.cull = pc.CULLFACE_NONE;
  mat.twoSidedLighting = true;
  mat.setAttribute('game_blanketExt', BLANKET_EXT.semantic);
  mat.setAttribute('game_blanketTint', BLANKET_TINT.semantic);
  mat.setDefine('GAME_BLANKET', '');
  mat.setParameter('game_blanketRange', new Float32Array([toeZ, footZ]));
  MaterialLibrary.for(app).adopt(mat);
  return mat;
}
