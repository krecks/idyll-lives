/**
 * Tree impostors: the woods far from the eye drawn as pictures of their trees on cards, two triangles a
 * tree instead of the model's thousand or so, in the town view.
 *
 * Baked once per tree model, the first time the town view wants it (`TreeImpostors.want`): the model is drawn
 * from `IMPOSTOR.columns` sides at each of `IMPOSTOR.elevations` (the town view's camera looks down at about
 * 33° to 56°), orthographically, into one target in a frame, as its base colour by day and by night (the leaves'
 * tint, `nature.ts` `LEAVES`: their tops warm by day, a moonlit grey by night) and its normals (the diag view's
 * `GAME_DIAG_ALBEDO` and `GAME_DIAG_NORMALS`, `shaders/diagView.ts`: no light, no haze, the wind and the veil left
 * out), and its occlusion and glossiness (`GAME_DIAG_SURFACE`). They are read back and made four pictures: the colour by day and by night, the card drawing the one the
 * daylight says (`game_daylight`, blending between), and the normals across each view, which light the card texel by
 * texel as the leaves are lit (`metalness` slot, after `getNormal`), and the occlusion and glossiness, the card's
 * own as the leaves' finish has them. Each picture has its edges bled outwards so
 * the mipmaps don't darken them, and is uploaded with mipmaps.
 *
 * A card is a quad facing the eye (`IMPOSTOR_SHADER`: `vsLocal` turns it to face `game_eye` from the
 * tree's middle, `uv0VS` (`chunks.ts`) picks the picture of the side and the height the eye sees it from, by the copy's
 * own turn), on the game's lit material (`MaterialLibrary.adopt`: the sun, the sky, the haze, the night,
 * the veil's sinking), each card in its tree's own leaf tint (the pictures
 * are baked in the origin's, and each leaf takes it as far as the real one does: `nature.ts` `windCode`). Instanced by the same matrices as the
 * trees it stands for, one draw per woods square and model (`nature.ts` `frameImpostors`).
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { MaterialLibrary } from './materials';
import type { ModelTemplate } from './models';
import { withoutContact } from './post';
import { codeFor, float, registerShaderFeature, type Language, type ShaderFeature } from './shaders/chunks';
import { IMPOSTOR_COLUMNS, IMPOSTOR_ROWS } from './shaders/impostorLayout';

/**
 * The pictures: `columns` sides round the tree at each of `elevations` (rad above the horizon), `tile` texels
 * square each; a woods square further than `switchDistance` m from the eye (its nearest tree) draws them.
 */
export const IMPOSTOR = { columns: IMPOSTOR_COLUMNS, elevations: [0.6, 0.95], tile: 128, switchDistance: 250 } as const;

/** Where the bake's cameras stand from the tree (m: far enough that the leaves' cut-out near the eye is off, `shaders/cutout.ts`). */
const BAKE_DISTANCE = 120;
/** The picture's margin round the tree's outline (a share of its size). */
const MARGIN = 1.06;
/** A first bake whose tree needs less than this share of its card is drawn again, framed to it (`fit`). */
const FRAMED = 0.85;
/** How fast a card's glossiness falls as the leaves' blended normals disagree (the power of their average's length). */
const TOKSVIG = 8;
/** A card's texels drawn from this opacity (its leaves' edges thinning in the far mipmaps). */
const ALPHA_TEST = 0.4;
/** How far (texels) a picture's edges are bled into the clear texels round them. */
const BLEED = 4;
/**
 * How long a bake waits for its frame (ms). No frame comes while the browser's GPU process compiles (a machine's first
 * game: most of a minute before the warm-up made its pipelines side by side, `parallelPipelines.ts`), so long.
 */
const FRAME_WAIT_MS = 30_000;
/** How long the pictures' read-back may take once the frame is drawn (ms). */
const READ_WAIT_MS = 10_000;
/** A bake that failed is asked for again after this (ms), times the tries so far; given up after `BAKE_TRIES`. */
const RETRY_MS = 20_000;
const BAKE_TRIES = 3;

const ROWS = IMPOSTOR_ROWS;
/** What the bake draws, a band of rows each, from the top: the colour by day, by night, the normals, the occlusion and glossiness. */
const PASSES = ['day', 'night', 'normals', 'surface'] as const;

/** The shader's code for a card (see the file's header): the tree's frame and the eye's way from its middle. */
function frame(lang: Language): string {
  const { v3, u, decl } = codeFor(lang);
  return `
		${decl('impAx', 'vec3')} = normalize(instance_line1.xyz);
		${decl('impAz', 'vec3')} = normalize(instance_line3.xyz);
		${decl('impC', 'vec3')} = instance_line4.xyz + ${v3}(0.0, ${u('game_impostor')}.y * length(instance_line2.xyz), 0.0);
		${decl('impF', 'vec3')} = normalize(${u('game_eye')} - impC);`;
}

function impostorCode(lang: Language): ShaderFeature['glsl'] {
  const { wgsl, v3, u, decl } = codeFor(lang);
  const vsDecl = wgsl
    ? `
#ifdef GAME_IMPOSTOR
	uniform game_eye: vec3f;
	uniform game_impostor: vec4f;
#endif`
    : `
#ifdef GAME_IMPOSTOR
	uniform vec3 game_eye;
	uniform vec4 game_impostor;
#endif`;
  // The card faces the eye from the tree's middle: right and up across the eye's view (the world root's mirror turns
  // them round, so right is f × up here), in the copy's own frame (its turn undone; its scale stays the matrix's).
  const vsLocal = `
#if defined(GAME_IMPOSTOR) && defined(INSTANCING)
	{${frame(lang)}
		${decl('impR', 'vec3')} = normalize(cross(impF, ${v3}(0.0, 1.0, 0.0)));
		${decl('impU', 'vec3')} = cross(impR, impF);
		${decl('impO', 'vec3')} = (impR * localPos.x + impU * localPos.y) * ${u('game_impostor')}.x + ${v3}(0.0, ${u('game_impostor')}.y, 0.0);
		localPos = ${v3}(dot(impO, impAx), impO.y, dot(impO, impAz));
	}
#endif`;
  // Lit as a crown facing up and a little towards the eye.
  const vsNormal = {
    when: 'defined(GAME_IMPOSTOR) && defined(INSTANCING)',
    code: `${frame(lang)}
		${decl('impN', 'vec3')} = normalize(impF * 0.35 + ${v3}(0.0, 1.0, 0.0));
		return normalize(dNormalMatrix * ${v3}(dot(impN, impAx), impN.y, dot(impN, impAz)));`,
  };
  // The pictures, blended between the side before the eye's and the next and between the two heights (`vVertexColor`'s
  // alpha: `nature.ts` `windCode`'s `GAME_IMPOSTOR`, whole 256ths of the way to the next side plus the share of the
  // way to the higher height), so a card never jumps from one picture to another as the camera moves.
  const cols = float(IMPOSTOR_COLUMNS);
  const rows = float(IMPOSTOR_ROWS);
  const psDecl = wgsl
    ? `
#ifdef GAME_IMPOSTOR
	var game_impostorDay: texture_2d<f32>;
	var game_impostorDaySampler: sampler;
	var game_impostorNight: texture_2d<f32>;
	var game_impostorNightSampler: sampler;
	var game_impostorNormal: texture_2d<f32>;
	var game_impostorNormalSampler: sampler;
	var game_impostorSurface: texture_2d<f32>;
	var game_impostorSurfaceSampler: sampler;
	uniform game_daylight: f32;
	fn gameImpostorTex(tex: texture_2d<f32>, smp: sampler) -> vec4f {
		let impW = floor(vVertexColor.a) / 256.0;
		let impT = fract(vVertexColor.a) / 0.99;
		let impCol = floor(vUv0.x * ${cols});
		let impB = vec2f(vUv0.x + select(1.0, 1.0 - ${cols}, impCol >= ${cols} - 1.0) / ${cols}, vUv0.y);
		let impUp = vec2f(0.0, 1.0 / ${rows});
		return mix(mix(textureSample(tex, smp, vUv0), textureSample(tex, smp, impB), impW), mix(textureSample(tex, smp, vUv0 + impUp), textureSample(tex, smp, impB + impUp), impW), impT);
	}
	// (The normals at their finest level: a far tree's pixel shows one leaf, facing its own way, as the real leaves'
	// do; the mipmaps' average, made unit again, faced the sky more than the leaves under it, and the crowns caught the
	// moon and the sky as the trees don't.)
	fn gameImpostorFine(tex: texture_2d<f32>, smp: sampler) -> vec4f {
		let impW = floor(vVertexColor.a) / 256.0;
		let impT = fract(vVertexColor.a) / 0.99;
		let impCol = floor(vUv0.x * ${cols});
		let impB = vec2f(vUv0.x + select(1.0, 1.0 - ${cols}, impCol >= ${cols} - 1.0) / ${cols}, vUv0.y);
		let impUp = vec2f(0.0, 1.0 / ${rows});
		return mix(mix(textureSampleLevel(tex, smp, vUv0, 0.0), textureSampleLevel(tex, smp, impB, 0.0), impW), mix(textureSampleLevel(tex, smp, vUv0 + impUp, 0.0), textureSampleLevel(tex, smp, impB + impUp, 0.0), impW), impT);
	}
#endif`
    : `
#ifdef GAME_IMPOSTOR
	uniform sampler2D game_impostorDay;
	uniform sampler2D game_impostorNight;
	uniform sampler2D game_impostorNormal;
	uniform sampler2D game_impostorSurface;
	uniform float game_daylight;
	vec4 gameImpostorTex(sampler2D tex) {
		float impW = floor(vVertexColor.a) / 256.0;
		float impT = fract(vVertexColor.a) / 0.99;
		float impCol = floor(vUv0.x * ${cols});
		vec2 impB = vec2(vUv0.x + (impCol >= ${cols} - 1.0 ? 1.0 - ${cols} : 1.0) / ${cols}, vUv0.y);
		vec2 impUp = vec2(0.0, 1.0 / ${rows});
		return mix(mix(texture2D(tex, vUv0), texture2D(tex, impB), impW), mix(texture2D(tex, vUv0 + impUp), texture2D(tex, impB + impUp), impW), impT);
	}
	vec4 gameImpostorFine(sampler2D tex) {
		float impW = floor(vVertexColor.a) / 256.0;
		float impT = fract(vVertexColor.a) / 0.99;
		float impCol = floor(vUv0.x * ${cols});
		vec2 impB = vec2(vUv0.x + (impCol >= ${cols} - 1.0 ? 1.0 - ${cols} : 1.0) / ${cols}, vUv0.y);
		vec2 impUp = vec2(0.0, 1.0 / ${rows});
		return mix(mix(texture2DLod(tex, vUv0, 0.0), texture2DLod(tex, impB, 0.0), impW), mix(texture2DLod(tex, vUv0 + impUp, 0.0), texture2DLod(tex, impB + impUp, 0.0), impW), impT);
	}
#endif`;
  const tex = (name: string) => (wgsl ? `gameImpostorTex(${name}, ${name}Sampler)` : `gameImpostorTex(${name})`);
  const fine = (name: string) => (wgsl ? `gameImpostorFine(${name}, ${name}Sampler)` : `gameImpostorFine(${name})`);
  // Its opacity, its glossiness and occlusion (as the leaves' finish has them) from the blended pictures; the colour is
  // the trees' wind feature's, which knows their leaf tint (`nature.ts` `windCode`'s albedo).
  const opacity = `
#ifdef GAME_IMPOSTOR
	dAlpha = ${tex('game_impostorDay')}.a;
#endif`;
  // (Rougher only where the pictures a texel blends between disagree, read at their finest level as the lighting
  // normal is: a picture's own leaves disagree in its mipmaps, and from 250 m out the fetch is coarse and the average
  // short, so the gloss was gone on every far card — its crown catching the sky's blue at night, not the moon's sheen.)
  const gloss = wgsl
    ? `
#ifdef GAME_IMPOSTOR
	{
		let impL = clamp(length(${fine('game_impostorNormal')}.rgb * 2.0 - vec3f(1.0)), 0.0, 1.0);
		dGlossiness = ${tex('game_impostorSurface')}.g * pow(impL, ${float(TOKSVIG)});
	}
#endif`
    : `
#ifdef GAME_IMPOSTOR
	{
		float impL = clamp(length(${fine('game_impostorNormal')}.rgb * 2.0 - vec3(1.0)), 0.0, 1.0);
		dGlossiness = ${tex('game_impostorSurface')}.g * pow(impL, ${float(TOKSVIG)});
	}
#endif`;
  const emissive = `
#if defined(GAME_IMPOSTOR) && defined(STD_AO)
	litArgs_ao = ${tex('game_impostorSurface')}.r;
#endif`;
  // Lit by the tree's own normals (`compose`: across the picture, up it and towards the eye that drew it), the card's
  // frame worked out again from the eye (engine space: the world root's mirror turns right round, `vsLocal`). After
  // `getNormal`: the slot after it that every card runs (`useMetalness`).
  const metalness = wgsl
    ? `
#ifdef GAME_IMPOSTOR
	{
		let impF = normalize(uniform.view_position - vPositionW);
		let impR = normalize(cross(vec3f(0.0, 1.0, 0.0), impF));
		let impU = cross(impF, impR);
		let impN = ${fine('game_impostorNormal')}.rgb * 2.0 - vec3f(1.0);
		litArgs_worldNormal = normalize(impR * impN.x + impU * impN.y + impF * impN.z);
	}
#endif`
    : `
#ifdef GAME_IMPOSTOR
	{
		vec3 impF = normalize(view_position - vPositionW);
		vec3 impR = normalize(cross(vec3(0.0, 1.0, 0.0), impF));
		vec3 impU = cross(impF, impR);
		vec3 impN = ${fine('game_impostorNormal')}.rgb * 2.0 - vec3(1.0);
		litArgs_worldNormal = normalize(impR * impN.x + impU * impN.y + impF * impN.z);
	}
#endif`;
  return { vsDecl, vsLocal, vsNormal, psDecl, opacity, gloss, emissive, metalness };
}

/** The cards' shader (see the file's header); before the veil's sinking (`fog.ts`, 239), which then sinks the card. */
const IMPOSTOR_SHADER: ShaderFeature = { name: 'impostor', priority: 225, glsl: impostorCode('glsl'), wgsl: impostorCode('wgsl') };

registerShaderFeature(IMPOSTOR_SHADER);

/** The eye (game space) the cards face: every card's, set each frame the town view draws them (`setImpostorEye`). */
const eyeValues = new WeakMap<pc.GraphicsDevice, Float32Array>();

export function setImpostorEye(device: pc.GraphicsDevice, eye: { x: number; y: number; z: number }): void {
  let v = eyeValues.get(device);
  if (!v) eyeValues.set(device, (v = new Float32Array(3)));
  v[0] = eye.x;
  v[1] = eye.y;
  v[2] = eye.z;
  device.scope.resolve('game_eye').setValue(v);
}

/** A model's impostor: its picture, the card's mesh (its box the card's reach) and material. */
export interface Impostor {
  /** The pictures by day and by night (the leaves' tops warm, or a moonlit grey). */
  texture: pc.Texture;
  night: pc.Texture;
  /** Its normals (`compose`), lighting the card as the tree's leaves are lit. */
  normal: pc.Texture;
  /** Its occlusion (red) and glossiness (green), as the leaves' finish has them, and how much of its tree's tint each leaf takes (blue). */
  surface: pc.Texture;
  mesh: pc.Mesh;
  material: pc.StandardMaterial;
}

interface Wanted {
  key: string;
  template: ModelTemplate;
  parent: pc.Entity;
}

/** Every tree model's impostor, baked one at a time as the town view first wants each. */
export class TreeImpostors {
  private readonly made = new Map<string, Impostor | 'baking' | 'failed'>();
  /** Each model's failed bakes so far (`RETRY_MS`). */
  private readonly tries = new Map<string, number>();
  private readonly retries = new Set<ReturnType<typeof setTimeout>>();
  private readonly queue: Wanted[] = [];
  private busy = false;
  private destroyed = false;

  constructor(
    private readonly app: pc.AppBase,
    /** An impostor was baked: what draws them chooses again. */
    private readonly onBaked: () => void,
  ) {}

  /**
   * `key`'s state: its impostor once baked, `'baking'` while its bake runs, `'failed'` if that gave up (`'missing'`
   * again `RETRY_MS` later, so the next ask bakes it again, until `BAKE_TRIES` have failed), `'missing'` until it is asked for.
   */
  state(key: string): Impostor | 'baking' | 'failed' | 'missing' {
    return this.made.get(key) ?? 'missing';
  }

  /** Asks for `key`'s impostor (baked from `template`, its draws under `parent`) and says what came of asking. */
  want(key: string, template: ModelTemplate, parent: pc.Entity): Impostor | 'baking' | 'failed' {
    const have = this.state(key);
    if (have !== 'missing') return have;
    this.made.set(key, 'baking');
    this.queue.push({ key, template, parent });
    void this.next();
    return 'baking';
  }

  /** `key`'s impostor if it is baked; else it is baked (from `template`, its bake's draws under `parent`) and null for now. */
  get(key: string, template: ModelTemplate, parent: pc.Entity): Impostor | null {
    const have = this.want(key, template, parent);
    return typeof have === 'object' ? have : null;
  }

  private async next(): Promise<void> {
    if (this.busy || this.destroyed) return;
    const job = this.queue.shift();
    if (!job) return;
    this.busy = true;
    try {
      const made = await bake(this.app, job.key, job.template, job.parent);
      if (this.destroyed) {
        if (made) disposeImpostor(made);
        return;
      }
      if (made) {
        this.made.set(job.key, made);
        this.onBaked();
      } else this.failed(job.key);
    } catch (err) {
      console.warn(`[render] impostor of ${job.key} failed`, err);
      this.failed(job.key);
    } finally {
      this.busy = false;
    }
    void this.next();
  }

  /** `key`'s bake gave up: asked for again later (a frame withheld for long, a read-back lost), up to `BAKE_TRIES` in all. */
  private failed(key: string): void {
    this.made.set(key, 'failed');
    const tries = (this.tries.get(key) ?? 0) + 1;
    this.tries.set(key, tries);
    if (tries >= BAKE_TRIES) {
      console.warn(`[render] impostor of ${key}: given up after ${tries} tries (its woods drawn as trees)`);
      return;
    }
    const retry = setTimeout(() => {
      this.retries.delete(retry);
      if (!this.destroyed && this.made.get(key) === 'failed') this.made.delete(key);
    }, RETRY_MS * tries);
    this.retries.add(retry);
  }

  destroy(): void {
    this.destroyed = true;
    for (const r of this.retries) clearTimeout(r);
    this.retries.clear();
    this.queue.length = 0;
    for (const m of this.made.values()) if (typeof m === 'object') disposeImpostor(m);
    this.made.clear();
  }
}

function disposeImpostor(m: Impostor): void {
  m.material.destroy();
  m.mesh.destroy();
  m.texture.destroy();
  m.night.destroy();
  m.normal.destroy();
  m.surface.destroy();
}

/** A card for `count` copies of `impostor`, their matrices `matrices` (16 floats each), under `parent` (drawn while it is). */
export function impostorCards(impostor: Impostor, parent: pc.Entity, matrices: Float32Array, count: number, name: string): Instances {
  // SAFETY: an entity keeps its app in `_app` (the constructor's `app`; tools/render/chunks.json).
  const device = (parent as unknown as { _app: pc.AppBase })._app.graphicsDevice;
  // (A matrix each, 16 floats: never read past what was given.)
  const cards = Math.min(count, Math.floor(matrices.length / 16));
  if (cards < count) console.warn(`[render] ${name}: ${count} cards asked for, ${cards} matrices given`);
  const d = new Instances(device, parent, impostor.mesh, impostor.material, { name, castShadows: false, receiveShadows: true, capacity: cards });
  d.setMatrices(matrices, cards);
  d.update();
  return d;
}

/**
 * Bakes `template`'s impostor (see the file's header): its draws on layers of their own, under `parent` (the world
 * root: the mirror of the game's draws), the cameras over a frame; null if the frame never came.
 */
async function bake(app: pc.AppBase, key: string, template: ModelTemplate, parent: pc.Entity): Promise<Impostor | null> {
  const device = app.graphicsDevice;
  const { columns, elevations, tile } = IMPOSTOR;
  // The model's box (its meshes', model space): how high its middle is, how far it reaches round its trunk and up and down.
  const min = new pc.Vec3(Infinity, Infinity, Infinity);
  const max = new pc.Vec3(-Infinity, -Infinity, -Infinity);
  let any = false;
  for (const m of template.meshes) {
    const b = m.mesh.aabb;
    min.min(b.getMin());
    max.max(b.getMax());
    any = true;
  }
  if (!any) {
    console.warn(`[render] impostor of ${key} failed: its model has no mesh with a box`);
    return null;
  }
  const centre = (min.y + max.y) / 2;
  const tallHalf = Math.max(0.25, (max.y - min.y) / 2);
  const radius = Math.max(0.25, Math.hypot(Math.max(-min.x, max.x), Math.max(-min.z, max.z)));
  // Half the card at first: the tree's box seen from any side at either height.
  const loose = Math.max(radius, ...elevations.map((e) => tallHalf * Math.cos(e) + radius * Math.sin(e))) * MARGIN;

  // One target: the colour by day's rows, the colour by night's, the normals'.
  const width = tile * columns;
  const passes = PASSES.length;
  const tall = tile * ROWS * passes;
  const color = new pc.Texture(device, { name: `impostorBake:${key}`, width, height: tall, format: pc.PIXELFORMAT_RGBA8, mipmaps: false, minFilter: pc.FILTER_NEAREST, magFilter: pc.FILTER_NEAREST });
  const target = new pc.RenderTarget({ name: `impostorBake:${key}`, colorBuffer: color, depth: true });
  const layers = PASSES.map((pass) => {
    const layer = new pc.Layer({ name: `impostorBake:${pass}` });
    app.scene.layers.push(layer);
    return { pass, layer };
  });
  const materials: pc.Material[] = [];
  const draws: Instances[] = [];
  const identity = new pc.Mat4().data;
  for (const { pass, layer } of layers) {
    for (const m of template.meshes) {
      if (!(m.material instanceof pc.StandardMaterial)) continue;
      const mat = m.material.clone();
      mat.setDefine('GAME_DIAG_ALBEDO', pass === 'day' || pass === 'night' ? '' : false);
      mat.setDefine('GAME_DIAG_NORMALS', pass === 'normals' ? '' : false);
      mat.setDefine('GAME_DIAG_SURFACE', pass === 'surface' ? '' : false);
      mat.setDefine('GAME_VEIL_SINK', false);
      mat.update();
      materials.push(mat);
      const d = new Instances(device, parent, m.mesh, mat, { name: `impostorBake:${key}`, castShadows: false, receiveShadows: false, layers: [layer.id], capacity: 1, cull: false });
      d.setMatrices(identity, 1);
      d.update();
      // Still: no sway, no flutter; the leaves' tint by day or by night (`nature.ts` `NATURE_WIND`: their tops a moonlit grey).
      d.meshInstance.setParameter('game_wind', [0, 0, 1, 0]);
      d.meshInstance.setParameter('game_windFlutter', 0);
      d.meshInstance.setParameter('game_daylight', pass === 'night' ? 0 : 1);
      draws.push(d);
    }
  }
  /** The pictures drawn with the card `half` m across its middle: the target's pixels, null if no frame came. */
  const render = async (half: number, middle: number): Promise<Uint8Array | null> => {
    // The cameras, a tile each (rows from the top, by `PASSES`; a camera's rect from the bottom).
    const cameras: pc.Entity[] = [];
    // (Game space through the world root's own transform, where the draws are: it mirrors z and may stand off the origin.)
    const root = parent.getWorldTransform();
    const at = new pc.Vec3();
    const aim = root.transformPoint(new pc.Vec3(0, middle, 0));
    layers.forEach(({ layer }, band) => {
      elevations.forEach((e, row) => {
        for (let col = 0; col < columns; col++) {
          const a = (col / columns) * Math.PI * 2;
          const top = band * ROWS + row;
          const rect = new pc.Vec4(col / columns, 1 - (top + 1) / (ROWS * passes), 1 / columns, 1 / (ROWS * passes));
          const cam = new pc.Entity('impostorBake');
          cam.addComponent('camera', {
            projection: pc.PROJECTION_ORTHOGRAPHIC,
            orthoHeight: half,
            aspectRatioMode: pc.ASPECT_MANUAL,
            aspectRatio: 1,
            nearClip: 1,
            farClip: BAKE_DISTANCE * 2,
            clearColor: new pc.Color(0, 0, 0, 0),
            layers: [layer.id],
            priority: -100,
            renderTarget: target,
            rect,
            scissorRect: rect,
            toneMapping: pc.TONEMAP_LINEAR,
            gammaCorrection: pc.GAMMA_SRGB,
          });
          root.transformPoint(at.set(Math.cos(a) * Math.cos(e) * BAKE_DISTANCE, middle + Math.sin(e) * BAKE_DISTANCE, Math.sin(a) * Math.cos(e) * BAKE_DISTANCE), at);
          cam.setPosition(at);
          cam.lookAt(aim);
          cam.enabled = false;
          app.root.addChild(cam);
          cameras.push(cam);
        }
      });
    });
    // Drawn in the next frame drawn, read once after it (as the save's picture, `thumbnail.ts`). Each wait has its
    // own deadline and says which it was: no frame (`FRAME_WAIT_MS`), or the read-back (`READ_WAIT_MS`).
    const pixels = await new Promise<Uint8Array | null>((resolve) => {
      let armed = false;
      let done = false;
      let timer = setTimeout(() => finish(null, `no frame was drawn within ${FRAME_WAIT_MS / 1000} s`), FRAME_WAIT_MS);
      const prerender = () => {
        if (armed) return;
        armed = true;
        for (const c of cameras) c.enabled = true;
      };
      /** The cameras off and the frame's hooks gone (once the frame is drawn, or the bake gives up). */
      const unhook = () => {
        for (const c of cameras) c.enabled = false;
        app.off('prerender', prerender);
        app.off('postrender', postrender);
      };
      const postrender = () => {
        if (!armed) return;
        unhook();
        clearTimeout(timer);
        timer = setTimeout(() => finish(null, `its read-back took over ${READ_WAIT_MS / 1000} s`), READ_WAIT_MS);
        const pending = color.read(0, 0, width, tall, { renderTarget: target }) as Promise<Uint8Array> | undefined;
        if (!pending) finish(null, 'the device cannot read the target back');
        else pending.then(finish, (err: unknown) => finish(null, `its read-back failed (${String(err)})`));
      };
      function finish(p: Uint8Array | null, why?: string) {
        if (done) return;
        done = true;
        if (why) console.warn(`[render] impostor of ${key}: ${why}`);
        clearTimeout(timer);
        unhook();
        resolve(p);
      }
      app.on('prerender', prerender);
      app.on('postrender', postrender);
    });
    for (const c of cameras) c.destroy();
    return pixels;
  };
  const cleanup = () => {
    for (const d of draws) d.destroy();
    for (const m of materials) m.destroy();
    for (const { layer } of layers) app.scene.layers.remove(layer);
    target.destroy();
    color.destroy();
  };
  // Drawn once with room for any tree of its size, then again framed to the tree as those pictures show it.
  let half = loose;
  let middle = centre;
  let pixels = await render(half, middle);
  const framed = pixels && fit(pixels, width, tall, tile, !device.isWebGPU, half, elevations);
  if (framed && framed.half < half * FRAMED) {
    middle -= framed.drop;
    half = framed.half;
    pixels = await render(half, middle);
  }
  cleanup();
  if (!pixels) return null;
  const picture = (band: number, name: string, normals = false, linear = normals) =>
    new pc.Texture(device, {
      name: `impostor:${key}:${name}`,
      width,
      height: tile * ROWS,
      // (Colour in sRGB, read back linear; normals as they are.)
      format: linear ? pc.PIXELFORMAT_RGBA8 : pc.PIXELFORMAT_SRGBA8,
      mipmaps: true,
      // (The nearer mipmap, not a blend of two: a crown's clumps stay sharper at a little shimmer's cost.)
      minFilter: pc.FILTER_LINEAR_MIPMAP_NEAREST,
      magFilter: pc.FILTER_LINEAR,
      addressU: pc.ADDRESS_CLAMP_TO_EDGE,
      addressV: pc.ADDRESS_CLAMP_TO_EDGE,
      levels: [compose(pixels, width, tall, !device.isWebGPU, band, normals)],
    });
  const texture = picture(PASSES.indexOf('day'), 'day');
  const night = picture(PASSES.indexOf('night'), 'night');
  const normal = picture(PASSES.indexOf('day'), 'normals', true);
  const surface = picture(PASSES.indexOf('surface'), 'surface', false, true);
  // The card: a quad from -1 to 1, its top's texels at v 0; its box what the card can reach (`vsLocal`), for culling.
  const mesh = new pc.Mesh(device);
  mesh.setPositions([-1, -1, 0, 1, -1, 0, 1, 1, 0, -1, 1, 0]);
  mesh.setNormals([0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1]);
  mesh.setUvs(0, [0, 1, 1, 1, 1, 0, 0, 0]);
  // (White: the vertex colour carries each tree's own leaf tint, `nature.ts` `windCode`.)
  mesh.setColors32([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
  mesh.setIndices([0, 1, 2, 0, 2, 3]);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  mesh.aabb = new pc.BoundingBox(new pc.Vec3(0, middle, 0), new pc.Vec3(half, half, half));
  const material = new pc.StandardMaterial();
  material.name = `impostor:${key}`;
  material.diffuse.set(1, 1, 1);
  material.diffuseMap = texture;
  material.diffuseVertexColor = true;
  material.opacityMap = texture;
  material.opacityMapChannel = 'a';
  material.alphaTest = ALPHA_TEST;
  material.cull = pc.CULLFACE_NONE;
  material.useMetalness = true;
  material.metalness = 0;
  // The leaves' occlusion and glossiness as their finish has them (texel by texel: the sky's light kept out of the crown, its sheen).
  material.aoMap = surface;
  material.aoMapChannel = 'r';
  material.glossMap = surface;
  material.glossMapChannel = 'g';
  material.glossInvert = false;
  material.gloss = 1;
  material.setDefine('GAME_IMPOSTOR', '');
  material.setDefine('GAME_VEIL_SINK', '');
  // (Half the card, its middle's height, the pictures' two heights; and the pictures themselves, `IMPOSTOR_SHADER`.)
  material.setParameter('game_impostor', [half, middle, elevations[0], elevations[1]]);
  material.setParameter('game_impostorDay', texture);
  material.setParameter('game_impostorSurface', surface);
  // (By night the night's picture, as the daylight goes: `IMPOSTOR_SHADER`'s `albedo`.)
  material.setParameter('game_impostorNight', night);
  // (Lit by the tree's own normals: `IMPOSTOR_SHADER`'s `metalness`.)
  material.setParameter('game_impostorNormal', normal);
  MaterialLibrary.for(app).adopt(material);
  // (No contact shading: a picture of a crown has no creases to shade, `post.ts`.)
  withoutContact(material);
  material.update();
  return { texture, night, normal, surface, mesh, material };
}

/**
 * Where the tree is in its first pictures (`pixels`, drawn with the card `half` m across): how far the aim must come
 * down the trunk (m) for the tree to stand in its tiles' middles, and the half (m) that then holds it in every tile. The
 * colour's rows only; each height's own pixels a metre (a tile is `2 half` m across), its drop the aim's by its cosine.
 */
function fit(pixels: Uint8Array, width: number, tall: number, tile: number, bottomUp: boolean, half: number, elevations: readonly number[]): { drop: number; half: number } | null {
  const rows = tile * elevations.length;
  const mid = tile / 2;
  const perMetre = tile / (2 * half);
  const bands = elevations.map(() => ({ top: Infinity, bottom: -Infinity, across: 0 }));
  for (let y = 0; y < rows; y++) {
    const at = (bottomUp ? tall - 1 - y : y) * width * 4;
    const band = bands[Math.floor(y / tile)];
    const ty = (y % tile) + 0.5;
    for (let x = 0; x < width; x++) {
      if (!pixels[at + x * 4 + 3]) continue;
      band.top = Math.min(band.top, ty);
      band.bottom = Math.max(band.bottom, ty);
      band.across = Math.max(band.across, Math.abs((x % tile) + 0.5 - mid));
    }
  }
  if (bands.some((b) => !Number.isFinite(b.top))) return null;
  // (Below the middle in the picture: the aim too high, by that over its height's cosine.)
  const drops = bands.map((b, i) => ((b.top + b.bottom) / 2 - mid) / perMetre / Math.cos(elevations[i]));
  const drop = drops.reduce((a, d) => a + d, 0) / drops.length;
  // Each band's reach once its middle comes to the tile's: up and down by its own spread, across as it was.
  const reach = Math.max(...bands.map((b) => Math.max((b.bottom - b.top) / 2, b.across))) / perMetre;
  return { drop, half: reach * MARGIN };
}

/**
 * A picture from the bake's pixels (`width` × `tall`, a band of rows per pass, `PASSES`; rows from the bottom on
 * WebGL2), opaque where the tree is, its edges bled: `band`'s colour as drawn, or (`normals`) each texel's normal
 * across its tile's view, up it and towards the eye that drew it (`compose`'s cameras: game space, the world root's
 * mirror turning right round, as the cards' `vsLocal`), as `n * 0.5 + 0.5`, for the cards' lighting.
 */
/** A band's rows: the target's height over the passes. */
const rowsPerPass = (tall: number) => tall / PASSES.length;

/** `v` · (`x`, `y`, `z`): a texel's normal across the picture against one of its view's axes. */
const dot = (v: readonly number[], x: number, y: number, z: number) => v[0] * x + v[1] * y + v[2] * z;

function compose(pixels: Uint8Array, width: number, tall: number, bottomUp: boolean, band: number, normals = false): Uint8Array {
  const rows = rowsPerPass(tall);
  const { columns, elevations, tile } = IMPOSTOR;
  const out = new Uint8Array(width * rows * 4);
  const row = (y: number) => (bottomUp ? tall - 1 - y : y) * width * 4;
  const views = elevations.map((e) =>
    Array.from({ length: columns }, (_, col) => {
      const a = (col / columns) * Math.PI * 2;
      const f = [Math.cos(a) * Math.cos(e), Math.sin(e), Math.sin(a) * Math.cos(e)];
      // right = f × up, up' = right × f
      const l = Math.hypot(f[2], f[0]) || 1;
      const r = [-f[2] / l, 0, f[0] / l];
      const u = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]];
      return { f, r, u };
    }),
  );
  const nBand = PASSES.indexOf('normals');
  for (let y = 0; y < rows; y++) {
    const a = row(y + band * rows);
    const n = row(y + nBand * rows);
    const view = views[Math.floor(y / tile)];
    for (let x = 0; x < width; x++) {
      const i = a + x * 4;
      const o = (y * width + x) * 4;
      if (pixels[i + 3] === 0) continue;
      if (normals) {
        const { f, r, u } = view[Math.floor(x / tile)];
        const gx = (pixels[n + x * 4] / 255) * 2 - 1;
        const gy = (pixels[n + x * 4 + 1] / 255) * 2 - 1;
        const gz = (pixels[n + x * 4 + 2] / 255) * 2 - 1;
        out[o] = Math.round((dot(r, gx, gy, gz) * 0.5 + 0.5) * 255);
        out[o + 1] = Math.round((dot(u, gx, gy, gz) * 0.5 + 0.5) * 255);
        out[o + 2] = Math.round((dot(f, gx, gy, gz) * 0.5 + 0.5) * 255);
      } else {
        out[o] = pixels[i];
        out[o + 1] = pixels[i + 1];
        out[o + 2] = pixels[i + 2];
      }
      out[o + 3] = 255;
    }
  }
  bleed(out, width, rows, tile);
  return out;
}

/** The clear texels next to the picture take its colour (still clear), `BLEED` texels out, within each tile. */
function bleed(px: Uint8Array, width: number, height: number, tile: number): void {
  const filled = new Uint8Array(width * height);
  for (let i = 0; i < filled.length; i++) filled[i] = px[i * 4 + 3] ? 1 : 0;
  for (let pass = 0; pass < BLEED; pass++) {
    const next = filled.slice();
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const i = y * width + x;
        if (filled[i]) continue;
        let r = 0;
        let g = 0;
        let b = 0;
        let n = 0;
        for (let dy = -1; dy <= 1; dy++) {
          for (let dx = -1; dx <= 1; dx++) {
            const xx = x + dx;
            const yy = y + dy;
            if (xx < 0 || yy < 0 || xx >= width || yy >= height) continue;
            if (Math.floor(xx / tile) !== Math.floor(x / tile) || Math.floor(yy / tile) !== Math.floor(y / tile)) continue;
            const j = yy * width + xx;
            if (!filled[j]) continue;
            r += px[j * 4];
            g += px[j * 4 + 1];
            b += px[j * 4 + 2];
            n++;
          }
        }
        if (!n) continue;
        px[i * 4] = r / n;
        px[i * 4 + 1] = g / n;
        px[i * 4 + 2] = b / n;
        next[i] = 1;
      }
    }
    filled.set(next);
  }
}
