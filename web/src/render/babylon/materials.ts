/**
 * Physically based materials and image-based lighting.
 *
 * - Lot surfaces (walls, floors, roads, terrain) and placeholder "finishes" (fabric, wood,
 *   ceramic, metal, ...) are `PBRMaterial`s built from manifest `material` entries: albedo,
 *   normal and packed AO/roughness/metal maps tiled in metres.
 * - Manifest colours are sRGB; PBR shades in linear space. Vertex and per-instance colours
 *   (placeholder parts, Sim tints) are converted on the GPU by `VertexTweaksPlugin`.
 * - The environment (`.env`, prefiltered offline from a CC0 HDRI) lights everything with sky
 *   ambient and reflections; its intensity follows the time of day.
 *
 * - House surfaces can carry `WallCutPlugin`: walls drop to stubs in the vertex shader when the
 *   cutaway state says so (driven by a per-vertex attribute, see `geometry.ts`), at no CPU cost.
 *
 * Every PBR material made or adopted here is shaded in the game's soft look (`softLook.ts`):
 * the light PBR computed, redrawn as a wide soft ramp in the light's own colour. The Settings'
 * Shading option (`classic`) draws the PBR shading instead; it applies live, by preparing the
 * materials again (`refreshSoftLook`).
 *
 * All materials are frozen once their textures are loaded. Changing the environment intensity
 * (or an emissive glow, via `touch`) thaws them for a couple of frames so the new value reaches
 * their uniform buffers (the renderer re-records its WebGPU snapshot on lighting changes anyway).
 */

import {
  Color3,
  CubeTexture,
  MaterialPluginBase,
  PBRMaterial,
  ShaderLanguage,
  StandardMaterial,
  Texture,
  Vector4,
  type AbstractEngine,
  type AbstractMesh,
  type Light,
  type Material,
  type MaterialDefines,
  type Mesh,
  type Nullable,
  type Scene,
  type SubMesh,
  type UniformBuffer,
} from './core';
import { withBuild } from '../../assets/build';
import { optimizedUrl, type AssetRegistry } from '../../assets/registry';
import type { MaterialEntry } from '../../assets/types';
import { applySoftLook } from './softLook';

/** Default finish for placeholder parts without a `material`. */
export const DEFAULT_FINISH = 'material.finish.matte';

const FALLBACK_FINISH: MaterialEntry = { type: 'material', color: '#FFFFFF', roughness: 0.75 };

/**
 * Small vertex-shader tweaks shared by GLSL (WebGL2) and WGSL (WebGPU):
 * - `srgbColors`: vertex/instance colours are authored in sRGB; convert them to linear.
 * - `planarUV`: UVs from the (baked, metre-scale) XZ position, for flat meshes built with
 *   0..1 UVs such as roads, so textures keep a realistic size.
 */
interface Tweaks {
  srgbColors?: boolean;
  planarUV?: boolean;
}
let pendingTweaks: Tweaks = {};

class VertexTweaksPlugin extends MaterialPluginBase {
  private readonly tweaks: Tweaks;

  constructor(material: Material, tweaks: Tweaks) {
    // The base constructor already asks for the shader code, before `this.tweaks` exists.
    pendingTweaks = tweaks;
    super(material, 'VertexTweaks', 200, undefined, true, false);
    this.tweaks = tweaks;
    this._enable(true);
  }

  override getClassName(): string {
    return 'VertexTweaksPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    if (shaderType !== 'vertex') return null;
    const wgsl = shaderLanguage === ShaderLanguage.WGSL;
    const tweaks = this.tweaks ?? pendingTweaks;
    const code: { [pointName: string]: string } = {};
    if (tweaks.planarUV) {
      code.CUSTOM_VERTEX_UPDATE_POSITION = '#ifdef UV1\nuvUpdated = positionUpdated.xz;\n#endif\n';
    }
    if (tweaks.srgbColors) {
      const body = wgsl
        ? 'vertexOutputs.vColor = vec4f(pow(max(vertexOutputs.vColor.rgb, vec3f(0.0)), vec3f(2.2)), vertexOutputs.vColor.a);'
        : 'vColor = vec4(pow(max(vColor.rgb, vec3(0.0)), vec3(2.2)), vColor.a);';
      code.CUSTOM_VERTEX_MAIN_END = `#if defined(VERTEXCOLOR) || defined(INSTANCESCOLOR) && defined(INSTANCES)\n${body}\n#endif\n`;
    }
    return code;
  }
}

/** Cutaway state shared by every wall material of a scene (set by the renderer). */
export class WallCutState {
  /** 1 where the camera looks along +x, -x, +z, -z (matches `LOOK` bits in geometry.ts). */
  readonly look = new Vector4(0, 0, 0, 0);
  /** x: all walls down (0/1); y: stub height in metres. */
  readonly params = new Vector4(0, 0.35, 0, 0);
  /**
   * A stretch of wall hidden while Build mode's preview stands in for it: everything within
   * `holeSpan.x` of the segment `hole` (x0, z0, x1, z1), between heights `holeSpan.y` and `.z`.
   * A zero width hides nothing.
   */
  readonly hole = new Vector4(0, 0, 0, 0);
  readonly holeSpan = new Vector4(0, 0, 0, 0);
}

/**
 * Lowers wall vertices in the vertex shader. Per vertex `wallCut` = [mask A, kind, mask B, 0]:
 * the vertex is cut when the camera looks along a direction set in both masks (or all walls are
 * down); kind 1 clamps it to the stub height, kind 2 hides it below the ground (window frames,
 * lintels). Fragments in the state's `hole` are dropped. Uniforms are bound per draw, so frozen
 * materials still follow the state.
 */
class WallCutPlugin extends MaterialPluginBase {
  constructor(
    material: Material,
    private readonly state: WallCutState,
  ) {
    super(material, 'WallCut', 210, { WALLCUT: false }, true, false);
    this.registerForExtraEvents = true;
    this._enable(true);
  }

  override getClassName(): string {
    return 'WallCutPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override prepareDefines(defines: MaterialDefines, _scene: Scene, mesh: AbstractMesh): void {
    defines.WALLCUT = mesh.isVerticesDataPresent('wallCut');
  }

  override getAttributes(attributes: string[], _scene: Scene, mesh: AbstractMesh): void {
    if (mesh.isVerticesDataPresent('wallCut')) attributes.push('wallCut');
  }

  override getUniforms(): { externalUniforms: string[] } {
    return { externalUniforms: ['wallCutLook', 'wallCutParams', 'wallHole', 'wallHoleSpan'] };
  }

  override hardBindForSubMesh(_ubo: UniformBuffer, _scene: Scene, _engine: AbstractEngine, subMesh: SubMesh): void {
    const effect = subMesh.effect;
    if (!effect) return;
    const { look, params, hole, holeSpan } = this.state;
    effect.setFloat4('wallCutLook', look.x, look.y, look.z, look.w);
    effect.setFloat4('wallCutParams', params.x, params.y, params.z, params.w);
    effect.setFloat4('wallHole', hole.x, hole.y, hole.z, hole.w);
    effect.setFloat4('wallHoleSpan', holeSpan.x, holeSpan.y, holeSpan.z, holeSpan.w);
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    if (shaderType === 'fragment') {
      const wgsl = shaderLanguage === ShaderLanguage.WGSL;
      const [u, p, v4, f] = wgsl ? ['uniforms.', 'fragmentInputs.vPositionW', 'vec4f', 'let'] : ['', 'vPositionW', 'vec4', 'float'];
      return {
        CUSTOM_FRAGMENT_DEFINITIONS: `
#ifdef WALLCUT
${wgsl ? `uniform wallHole: ${v4};\nuniform wallHoleSpan: ${v4};` : `uniform ${v4} wallHole;\nuniform ${v4} wallHoleSpan;`}
#endif
`,
        CUSTOM_FRAGMENT_MAIN_BEGIN: `
#ifdef WALLCUT
if (${u}wallHoleSpan.x > 0.0) {
  ${wgsl ? 'let ab: vec2f' : 'vec2 ab'} = ${u}wallHole.zw - ${u}wallHole.xy;
  ${f} t = dot(${p}.xz - ${u}wallHole.xy, ab) / max(dot(ab, ab), 1e-6);
  ${wgsl ? 'let off: vec2f' : 'vec2 off'} = ${p}.xz - (${u}wallHole.xy + ab * t);
  if (t >= 0.0 && t <= 1.0 && dot(off, off) < ${u}wallHoleSpan.x * ${u}wallHoleSpan.x && ${p}.y > ${u}wallHoleSpan.y && ${p}.y < ${u}wallHoleSpan.z) { discard; }
}
#endif
`,
      };
    }
    if (shaderType !== 'vertex') return null;
    if (shaderLanguage === ShaderLanguage.WGSL) {
      return {
        CUSTOM_VERTEX_DEFINITIONS: `
#ifdef WALLCUT
attribute wallCut: vec4f;
uniform wallCutLook: vec4f;
uniform wallCutParams: vec4f;
#endif
`,
        CUSTOM_VERTEX_UPDATE_POSITION: `
#ifdef WALLCUT
{
  let cutA = step(0.5, dot(floor(fract(vertexInputs.wallCut.x / vec4f(2.0, 4.0, 8.0, 16.0)) * 2.0), uniforms.wallCutLook));
  let cutB = step(0.5, dot(floor(fract(vertexInputs.wallCut.z / vec4f(2.0, 4.0, 8.0, 16.0)) * 2.0), uniforms.wallCutLook));
  if (max(uniforms.wallCutParams.x, cutA * cutB) > 0.5) {
    if (vertexInputs.wallCut.y > 1.5) { positionUpdated.y = -2.0; }
    else if (vertexInputs.wallCut.y > 0.5) { positionUpdated.y = min(positionUpdated.y, uniforms.wallCutParams.y); }
  }
}
#endif
`,
      };
    }
    return {
      CUSTOM_VERTEX_DEFINITIONS: `
#ifdef WALLCUT
attribute vec4 wallCut;
uniform vec4 wallCutLook;
uniform vec4 wallCutParams;
#endif
`,
      CUSTOM_VERTEX_UPDATE_POSITION: `
#ifdef WALLCUT
{
  float cutA = step(0.5, dot(floor(fract(wallCut.x / vec4(2.0, 4.0, 8.0, 16.0)) * 2.0), wallCutLook));
  float cutB = step(0.5, dot(floor(fract(wallCut.z / vec4(2.0, 4.0, 8.0, 16.0)) * 2.0), wallCutLook));
  if (max(wallCutParams.x, cutA * cutB) > 0.5) {
    if (wallCut.y > 1.5) { positionUpdated.y = -2.0; }
    else if (wallCut.y > 0.5) { positionUpdated.y = min(positionUpdated.y, wallCutParams.y); }
  }
}
#endif
`,
    };
  }
}

/** Rectangles of windowless rooms the materials test (see `DigState.rects`). */
export const WINDOWLESS_RECTS = 8;

/**
 * The ground dug away over the lot while its basement is in view: `rect` (x0, z0, x1, z1, in
 * metres) of the ground's materials isn't drawn above `below` metres while `on`.
 */
export class DigState {
  readonly rect = new Vector4(0, 0, 0, 0);
  on = false;
  below = -0.5;
  /**
   * The basement itself is in view: everything lit inside `rect` is underground then, out of the
   * sun (see `UndergroundPlugin`), with `lit` the share of the sun, the sky fill and the
   * environment (sky ambient and reflections) that still reaches it.
   */
  under = false;
  readonly lit = new Vector4(0, 0.1, 0.1, 0);
  /**
   * Rooms without windows on the storey in view, as dark as the basement by day: `rects` holds
   * up to `WINDOWLESS_RECTS` rectangles (x0, z0, x1, z1, in metres), `band` the heights they
   * reach (y0, y1) and in `band.z` how many there are.
   */
  readonly rects = new Float32Array(WINDOWLESS_RECTS * 4);
  readonly band = new Vector4(0, 0, 0, 0);
}

/**
 * Keeps daylight out of a basement in view (`DigState.under`), and out of the rooms without
 * windows on the storey in view (`DigState.rects`): inside them the sun and the sky fill (the
 * directional and hemispheric lights) and the environment are scaled down to `DigState.lit`, so
 * only the lamps (point lights) light them, day and night.
 */
class UndergroundPlugin extends MaterialPluginBase {
  constructor(
    material: Material,
    private readonly state: DigState,
  ) {
    super(material, 'Underground', 212, undefined, true, false);
    this.registerForExtraEvents = true;
    this._enable(true);
  }

  override getClassName(): string {
    return 'UndergroundPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override getUniforms(): { externalUniforms: string[] } {
    return { externalUniforms: ['undergroundRect', 'undergroundLit', 'windowlessBand', 'windowlessRects'] };
  }

  override hardBindForSubMesh(_ubo: UniformBuffer, _scene: Scene, _engine: AbstractEngine, subMesh: SubMesh): void {
    const effect = subMesh.effect;
    if (!effect) return;
    const { rect, under, lit, band, rects } = this.state;
    effect.setFloat4('undergroundRect', rect.x, rect.y, rect.z, rect.w);
    effect.setFloat4('undergroundLit', lit.x, lit.y, lit.z, under ? 1 : 0);
    effect.setFloat4('windowlessBand', band.x, band.y, band.z, 0);
    effect.setFloatArray4('windowlessRects', rects);
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    if (shaderType !== 'fragment') return null;
    const wgsl = shaderLanguage === ShaderLanguage.WGSL;
    const [u, p, v3, v4] = wgsl ? ['uniforms.', 'fragmentInputs.vPositionW', 'vec3f', 'vec4f'] : ['', 'vPositionW', 'vec3', 'vec4'];
    return {
      CUSTOM_FRAGMENT_DEFINITIONS: wgsl
        ? `uniform undergroundRect: ${v4};\nuniform undergroundLit: ${v4};\nuniform windowlessBand: ${v4};\nuniform windowlessRects: array<${v4}, ${WINDOWLESS_RECTS}>;`
        : `uniform ${v4} undergroundRect;\nuniform ${v4} undergroundLit;\nuniform ${v4} windowlessBand;\nuniform ${v4} windowlessRects[${WINDOWLESS_RECTS}];`,
      CUSTOM_FRAGMENT_BEFORE_LIGHTS: `
${wgsl ? 'var underground' : 'vec3 underground'} = ${v3}(1.0);
if (${u}undergroundLit.w > 0.5 && ${p}.x > ${u}undergroundRect.x && ${p}.x < ${u}undergroundRect.z && ${p}.z > ${u}undergroundRect.y && ${p}.z < ${u}undergroundRect.w) { underground = ${u}undergroundLit.xyz; }
if (${p}.y > ${u}windowlessBand.x && ${p}.y < ${u}windowlessBand.y) {
  for (${wgsl ? 'var i = 0' : 'int i = 0'}; i < ${WINDOWLESS_RECTS}; i++) {
    if (${wgsl ? 'f32(i)' : 'float(i)'} >= ${u}windowlessBand.z) { break; }
    ${wgsl ? 'let' : 'vec4'} r = ${u}windowlessRects[i];
    if (${p}.x > r.x && ${p}.x < r.z && ${p}.z > r.y && ${p}.z < r.w) { underground = ${u}undergroundLit.xyz; }
  }
}
`,
      // Each light's colour as it's read (lamps, the point lights, keep theirs).
      '!#define CUSTOM_LIGHT(\\d+)_COLOR': `
#if defined(DIRLIGHT$1)
diffuse$1 = ${v4}(diffuse$1.rgb * underground.x, diffuse$1.a);
#elif defined(HEMILIGHT$1)
diffuse$1 = ${v4}(diffuse$1.rgb * underground.y, diffuse$1.a);
#endif
#define CUSTOM_LIGHT$1_COLOR`,
      CUSTOM_FRAGMENT_BEFORE_FINALCOLORCOMPOSITION: `
#if !defined(UNLIT) && defined(REFLECTION)
finalIrradiance *= underground.z;
finalRadianceScaled *= underground.z;
#endif
`,
    };
  }
}

/** Drops the fragments of the landscape (terrain, lawn) inside the dug-out rectangle (`DigState`). */
class DigPlugin extends MaterialPluginBase {
  constructor(
    material: Material,
    private readonly state: DigState,
  ) {
    super(material, 'Dig', 211, undefined, true, false);
    this.registerForExtraEvents = true;
    this._enable(true);
  }

  override getClassName(): string {
    return 'DigPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override getUniforms(): { externalUniforms: string[] } {
    return { externalUniforms: ['digRect', 'digSpan'] };
  }

  override hardBindForSubMesh(_ubo: UniformBuffer, _scene: Scene, _engine: AbstractEngine, subMesh: SubMesh): void {
    const effect = subMesh.effect;
    if (!effect) return;
    const { rect, on, below } = this.state;
    effect.setFloat4('digRect', rect.x, rect.y, rect.z, rect.w);
    effect.setFloat4('digSpan', on ? 1 : 0, below, 0, 0);
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    if (shaderType !== 'fragment') return null;
    const wgsl = shaderLanguage === ShaderLanguage.WGSL;
    const [u, p, v4] = wgsl ? ['uniforms.', 'fragmentInputs.vPositionW', 'vec4f'] : ['', 'vPositionW', 'vec4'];
    return {
      CUSTOM_FRAGMENT_DEFINITIONS: wgsl ? `uniform digRect: ${v4};\nuniform digSpan: ${v4};` : `uniform ${v4} digRect;\nuniform ${v4} digSpan;`,
      CUSTOM_FRAGMENT_MAIN_BEGIN: `
if (${u}digSpan.x > 0.5 && ${p}.y > ${u}digSpan.y && ${p}.x > ${u}digRect.x && ${p}.x < ${u}digRect.z && ${p}.z > ${u}digRect.y && ${p}.z < ${u}digRect.w) { discard; }
`,
    };
  }
}

/** How far up a metal's reflection goes for one that would go down (1: as far up as it went down). */
const SKY_FOR_GROUND = '0.3';

/**
 * One material for every untextured opaque finish (matte, satin, gloss, metal, chrome, skin,
 * hair): metalness and roughness come from the vertex colour's alpha (see `encodeFinish`), so a
 * placeholder model with five finishes is one draw call instead of five. The alpha goes to the
 * fragment in a varying of its own: Babylon's `vColor.a` is 1 unless the mesh has vertex alpha,
 * and vertex alpha would make the material blend (which put every plain finish at rough metal).
 *
 * Metals don't reflect the ground. The game looks down on everything, so a mirror reflection off
 * anything upright (a handle, a steel fridge) comes from the sky map's lower half, the dark field
 * outside, which no room has: a reflection that would go down goes as far up instead, flattened
 * towards the horizon (`SKY_FOR_GROUND`), and the low sky stands in for the room.
 */
class PlainFinishPlugin extends MaterialPluginBase {
  constructor(material: Material) {
    super(material, 'PlainFinish', 220, undefined, true, true);
  }

  override getClassName(): string {
    return 'PlainFinishPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    const wgsl = shaderLanguage === ShaderLanguage.WGSL;
    // Matte where a mesh has no vertex colours.
    const matte = encodeFinish(0.75, 0).toFixed(4);
    if (shaderType === 'vertex') {
      return wgsl
        ? {
            CUSTOM_VERTEX_DEFINITIONS: 'varying vFinish: f32;',
            CUSTOM_VERTEX_MAIN_END: `#ifdef VERTEXCOLOR\nvertexOutputs.vFinish = vertexInputs.color.a;\n#else\nvertexOutputs.vFinish = ${matte};\n#endif\n`,
          }
        : {
            CUSTOM_VERTEX_DEFINITIONS: 'varying float vFinish;',
            CUSTOM_VERTEX_MAIN_END: `#ifdef VERTEXCOLOR\nvFinish = color.a;\n#else\nvFinish = ${matte};\n#endif\n`,
          };
    }
    if (shaderType !== 'fragment') return null;
    if (wgsl) {
      const a = 'fragmentInputs.vFinish';
      return {
        CUSTOM_FRAGMENT_DEFINITIONS: 'varying vFinish: f32;',
        '!(var\\s+reflectionVector\\s*:\\s*vec3f\\s*=\\s*computeReflectionCoords\\(\\s*vec4f\\(\\s*vPositionW\\s*,\\s*1\\.0\\s*\\)\\s*,\\s*normalW\\s*\\)\\s*;)': `$1\nif (${a} >= 0.5 && reflectionVector.y < 0.0) { reflectionVector.y *= -${SKY_FOR_GROUND}; }`,
        '!reflectivityBlock\\(\\s*uniforms\\.vReflectivityColor': `reflectivityBlock(vec4f(step(0.5, ${a}), clamp((${a} - 0.5 * step(0.5, ${a})) * 2.0, 0.03, 1.0), uniforms.vReflectivityColor.zw)`,
      };
    }
    return {
      CUSTOM_FRAGMENT_DEFINITIONS: 'varying float vFinish;',
      '!(vec3\\s+reflectionVector\\s*=\\s*computeReflectionCoords\\(\\s*vec4\\(\\s*vPositionW\\s*,\\s*1\\.0\\s*\\)\\s*,\\s*normalW\\s*\\)\\s*;)': `$1\nif (vFinish >= 0.5 && reflectionVector.y < 0.0) { reflectionVector.y *= -${SKY_FOR_GROUND}; }`,
      '!reflectivityBlock\\(\\s*vReflectivityColor': `reflectivityBlock(vec4(step(0.5, vFinish), clamp((vFinish - 0.5 * step(0.5, vFinish)) * 2.0, 0.03, 1.0), vReflectivityColor.ba)`,
    };
  }
}

/** Vertex-colour alpha for the plain finish: [0, 0.5) dielectric, [0.5, 1] metal; roughness scaled within. */
export function encodeFinish(roughness: number, metallic: number): number {
  const r = Math.min(0.999, Math.max(0, roughness)) * 0.5;
  return metallic >= 0.5 ? 0.5 + r : r;
}

/** Point lights (interior lamps) a lit material must handle besides the sun and the sky. */
const MAX_LIGHTS = 6;
/**
 * Smallest change (share of the value, and absolute) passed on to frozen materials over the day:
 * each one thaws them, and every mesh drawn with them then redoes its shader checks for a few
 * frames. Lighting updates come every half game minute, mostly with far smaller changes.
 */
const LIGHT_CHANGE = 0.02;
const LIGHT_CHANGE_MIN = 0.004;

export interface SurfaceOptions {
  /** Multiply albedo by (sRGB) vertex / instance colours. */
  vertexColors?: boolean;
  /** Render both faces (generated lot geometry does not guarantee winding). */
  doubleSided?: boolean;
  /** Derive UVs from the XZ position (metres) instead of the mesh's own UVs. */
  planarUV?: boolean;
  /** Follow the wall cutaway state (meshes need the `wallCut` attribute). */
  cutaway?: boolean;
  /** A separate copy of the surface under this name (e.g. one per room, for its own emissive glow). */
  variant?: string;
}

export class MaterialLibrary {
  private static readonly byScene = new WeakMap<Scene, MaterialLibrary>();

  /** The library for `scene`; created on first use (without a registry, finishes fall back to matte). */
  static for(scene: Scene): MaterialLibrary {
    let lib = MaterialLibrary.byScene.get(scene);
    if (!lib) MaterialLibrary.byScene.set(scene, (lib = new MaterialLibrary(scene)));
    return lib;
  }

  private assets: AssetRegistry | null = null;
  private readonly cache = new Map<string, PBRMaterial>();
  private readonly textures = new Map<string, Texture>();
  /** Every PBR material whose uniforms depend on the environment intensity. */
  private readonly lit = new Set<PBRMaterial>();
  private readonly thawed = new Set<PBRMaterial>();
  /** Thawed only for their uniforms to be uploaded again (see `setEnvironmentIntensity`). */
  private readonly thawedUniforms = new Set<PBRMaterial>();
  private refreezeFrames = 0;
  private shadowMaterial: StandardMaterial | null = null;
  /** Anisotropic filtering of the surfaces' textures (the graphics preset's `anisotropy`; set before the first surface). */
  anisotropy = 8;
  /** Wall cutaway state read by every `cutaway` material. */
  readonly wallCut = new WallCutState();
  /** The ground dug away over a basement in view (see `digInto`). */
  readonly dig = new DigState();
  private readonly dug = new WeakSet<Material>();

  /** Makes `material` (the landscape's ground and lawn) follow `dig`. */
  digInto(material: Material | null): void {
    if (!material || this.dug.has(material)) return;
    this.dug.add(material);
    const parts = (material as Material & { subMaterials?: (Material | null)[] }).subMaterials;
    if (parts) return parts.forEach((m) => this.digInto(m));
    if (!(material instanceof PBRMaterial || material instanceof StandardMaterial)) return;
    new DigPlugin(material, this.dig);
    // (It may have been compiled without it.)
    material.markDirty(true);
  }

  /** Called after thawed materials were frozen again (a WebGPU snapshot must re-record then). */
  onRefrozen: (() => void) | null = null;

  private constructor(private readonly scene: Scene) {
    scene.onAfterRenderObservable.add(() => {
      if (this.refreezeFrames === 0 || --this.refreezeFrames > 0) return;
      for (const m of this.thawed) m.freeze();
      for (const m of this.thawedUniforms) if (!this.thawed.has(m)) m.checkReadyOnlyOnce = true;
      this.thawed.clear();
      this.thawedUniforms.clear();
      this.onRefrozen?.();
    });
  }

  setAssets(assets: AssetRegistry): this {
    this.assets = assets;
    return this;
  }

  /** A textured surface from a manifest `material` entry (walls, floors, roads, terrain). */
  surface(key: string, options: SurfaceOptions = {}): PBRMaterial {
    const id = `${key}|${options.vertexColors ? 'vc' : ''}|${options.doubleSided ? '2s' : ''}|${options.planarUV ? 'xz' : ''}|${options.cutaway ? 'cut' : ''}|${options.variant ?? ''}`;
    let mat = this.cache.get(id);
    if (!mat) {
      const entry = this.assets?.get(key, 'material') ?? { type: 'material', color: '#DDDDDD' };
      mat = this.create(key, entry, { ...options, planarUV: options.planarUV || entry.projection === 'xz' });
      this.cache.set(id, mat);
    }
    return mat;
  }

  /** A vertex-coloured finish for placeholder parts (`material.finish.*`). */
  finish(key: string = DEFAULT_FINISH): PBRMaterial {
    const id = `finish|${key}`;
    let mat = this.cache.get(id);
    if (!mat) {
      const entry = (this.assets && this.assets.get(key, 'material')) || FALLBACK_FINISH;
      mat = this.create(key, entry, { vertexColors: true });
      this.cache.set(id, mat);
    }
    return mat;
  }

  /** Whether the manifests have finish `key` (`material.finish.*`). */
  hasFinish(key: string): boolean {
    return !!this.assets?.has(key, 'material');
  }

  /**
   * Whether a finish is untextured and opaque (it can share the plain material), with its
   * roughness and metalness.
   */
  finishInfo(key: string): { plain: boolean; roughness: number; metallic: number } {
    const entry = (this.assets && this.assets.has(key, 'material') && this.assets.get(key, 'material')) || FALLBACK_FINISH;
    return {
      plain: !entry.texture && (entry.alpha ?? 1) >= 1,
      roughness: entry.roughness ?? 0.8,
      metallic: entry.metallic ?? 0,
    };
  }

  /** The shared material for plain finishes (vertex colour alpha = encoded roughness/metalness). */
  plainFinish(): PBRMaterial {
    const id = 'finish|plain';
    let mat = this.cache.get(id);
    if (!mat) {
      mat = this.create('material.finish.plain', { type: 'material', color: '#FFFFFF', roughness: 1, metallic: 0 }, { vertexColors: true });
      new PlainFinishPlugin(mat);
      this.cache.set(id, mat);
    }
    return mat;
  }

  /** Adopts a material created elsewhere (e.g. by the glTF loader) into env updates, basements and freezing. */
  adopt(material: Material | null): void {
    if (!(material instanceof PBRMaterial) || this.lit.has(material)) return;
    material.maxSimultaneousLights = MAX_LIGHTS;
    new UndergroundPlugin(material, this.dig);
    applySoftLook(material);
    // (It may have been compiled without it.)
    material.markDirty(true);
    this.lit.add(material);
    this.freezeWhenReady(material);
  }

  /**
   * Unlit black with per-vertex alpha: soft contact shadows / ambient occlusion decals.
   * Follows the wall cutaway like the walls it darkens.
   */
  contactShadow(): StandardMaterial {
    if (this.shadowMaterial) return this.shadowMaterial;
    const mat = (this.shadowMaterial = new StandardMaterial('contactShadow', this.scene));
    mat.disableLighting = true;
    mat.diffuseColor = Color3.Black();
    mat.specularColor = Color3.Black();
    mat.emissiveColor = Color3.Black();
    mat.zOffset = -2;
    mat.disableDepthWrite = true;
    new WallCutPlugin(mat, this.wallCut);
    return mat;
  }

  /** A material's emissive colour (a frozen one is thawed only for a change one could see). */
  setEmissive(material: PBRMaterial, color: Color3): void {
    const e = material.emissiveColor;
    if (!visibleChange(e.r, color.r) && !visibleChange(e.g, color.g) && !visibleChange(e.b, color.b)) return;
    e.copyFrom(color);
    this.touch(material);
  }

  /** Re-uploads a frozen material's uniforms after a property change (e.g. emissive glow). */
  touch(material: Material): void {
    if (!(material instanceof PBRMaterial)) return;
    // Thawed for its uniforms only: frozen again in full, its shaders checked then.
    if (this.thawedUniforms.has(material)) this.thawed.add(material);
    if (!material.isFrozen) return;
    material.unfreeze();
    this.thawed.add(material);
    this.refreezeFrames = 3;
  }

  /**
   * Leaves `meshes` out of `lights` (replacing earlier entries of the same meshes). A frozen
   * material keeps the lights its shader was compiled with, and on WebGPU a mesh drawn with fewer
   * lights than its shader expects fails to draw, so meshes left out now (perhaps already drawn
   * while their models were still loading) get their materials thawed to recompile.
   */
  excludeFromLights(lights: readonly Light[], meshes: ReadonlySet<Mesh>): void {
    for (const light of lights) {
      // The list is changed in place: assigning a new one makes Babylon check every mesh of the
      // scene against all of it, per light (most of a lot change's hitch); its `push` checks just
      // the meshes added. Disposed ones are just dropped (nothing of them is left to update).
      const list = light.excludedMeshes;
      let kept = 0;
      for (const m of list) if (!m.isDisposed()) list[kept++] = m;
      list.length = kept;
      const before = new Set(list);
      const added = [...meshes].filter((m) => !before.has(m));
      if (added.length) list.push(...added);
      // Meshes not drawn yet get their shaders set up on their first frame, frozen or not
      // (thawing a shared material would redo that for every mesh drawn with it).
      for (const mesh of added) if (mesh.material && mesh.subMeshes?.some((sm) => sm.effect)) this.touch(mesh.material);
    }
  }

  /**
   * Loads a prefiltered `.env` environment; resolves (without throwing) even when it fails. The
   * file is fetched here rather than by Babylon, so the download `index.html` starts with the page
   * (`earlySky`) can be the one used.
   */
  loadEnvironment(url: string): Promise<boolean> {
    return new Promise((resolve) => {
      let done = false;
      const finish = (ok: boolean) => {
        if (done) return;
        done = true;
        if (!ok) console.warn(`[render] environment ${url} unavailable; using lights only`);
        resolve(ok);
      };
      setTimeout(() => finish(false), 15000);
      const download = () => fetch(url).then((res) => (res.ok ? res.arrayBuffer() : null));
      (earlySky(url)?.then((bytes) => bytes ?? download()) ?? download())
        .then((bytes) => {
          if (!bytes) throw new Error('not loaded');
          // (Also after the time limit: the sky lights the scene once it's there.)
          const tex = new CubeTexture(url, this.scene, {
            buffer: new Uint8Array(bytes),
            prefiltered: true,
            forcedExtension: '.env',
            onLoad: () => {
              this.scene.environmentTexture = tex;
              finish(true);
            },
            onError: () => finish(false),
          });
          tex.rotationY = Math.PI * 0.35;
        })
        .catch(() => finish(false));
    });
  }

  /** Scales image-based lighting (sky ambient + reflections), e.g. for the time of day (in steps too small to see). */
  setEnvironmentIntensity(value: number): void {
    if (!visibleChange(this.scene.environmentIntensity, value)) return;
    this.scene.environmentIntensity = value;
    // Only a uniform changes, not the shaders: a frozen material uploads its uniforms again while
    // `isFrozen` is false, without `unfreeze` and `freeze` (each walks every mesh of the scene, for
    // every material: milliseconds a lighting step, ten steps a second at dusk at fast speeds, and
    // the first frozen frame after them checked every mesh's shader again).
    for (const m of this.lit) {
      if (!m.isFrozen) continue;
      m.checkReadyOnlyOnce = false;
      this.thawedUniforms.add(m);
    }
    // Keep them thawed for the frame that re-records the snapshot, then freeze again.
    if (this.thawedUniforms.size) this.refreezeFrames = 3;
  }

  private create(key: string, entry: MaterialEntry, options: SurfaceOptions): PBRMaterial {
    if (new URLSearchParams(location.search).has('profile')) console.log(`[material] create ${key} ${options.variant ?? ''}`);
    const mat = new PBRMaterial(key, this.scene);
    mat.albedoColor = Color3.FromHexString(entry.color).toLinearSpace();
    const scale = entry.uvScale ?? 1;
    const base = entry.texture;
    if (base) {
      mat.albedoTexture = this.texture(base, scale, true);
      if (entry.normal) {
        const bump = this.texture(new URL(entry.normal, base).href, scale, false);
        mat.bumpTexture = bump;
        bump.level = entry.normalStrength ?? 1;
        // Same convention as Babylon's glTF loader for OpenGL-style maps in a left-handed scene.
        mat.invertNormalMapX = true;
        mat.invertNormalMapY = false;
      }
      if (entry.arm) {
        mat.metallicTexture = this.texture(new URL(entry.arm, base).href, scale, false);
        mat.useAmbientOcclusionFromMetallicTextureRed = true;
        mat.useRoughnessFromMetallicTextureGreen = true;
        mat.useMetallnessFromMetallicTextureBlue = true;
        mat.useRoughnessFromMetallicTextureAlpha = false;
      }
    }
    // With a packed map the scalars multiply its channels.
    mat.roughness = entry.roughness ?? (entry.arm ? 1 : 0.8);
    mat.metallic = entry.metallic ?? (entry.arm ? 1 : 0);
    if (entry.alpha !== undefined && entry.alpha < 1) {
      mat.alpha = entry.alpha;
      mat.transparencyMode = PBRMaterial.MATERIAL_ALPHABLEND;
      // Keep glass reflections strong while the body is see-through.
      mat.useRadianceOverAlpha = true;
      mat.useSpecularOverAlpha = true;
    }
    mat.backFaceCulling = !options.doubleSided;
    if (options.doubleSided) mat.twoSidedLighting = true;
    mat.maxSimultaneousLights = MAX_LIGHTS;
    if (options.vertexColors || options.planarUV) new VertexTweaksPlugin(mat, { srgbColors: options.vertexColors, planarUV: options.planarUV });
    if (options.cutaway) new WallCutPlugin(mat, this.wallCut);
    new UndergroundPlugin(mat, this.dig);
    applySoftLook(mat);
    this.lit.add(mat);
    this.freezeWhenReady(mat);
    return mat;
  }

  private texture(url: string, scale: number, srgb: boolean): Texture {
    const id = `${url}|${scale}`;
    let tex = this.textures.get(id);
    if (!tex) {
      tex = textureWithFallback(url, this.scene, srgb);
      tex.uScale = tex.vScale = scale;
      tex.anisotropicFilteringLevel = this.anisotropy;
      this.textures.set(id, tex);
    }
    return tex;
  }

  private freezeWhenReady(mat: PBRMaterial): void {
    const textures = mat.getActiveTextures().filter((t): t is Texture => t instanceof Texture);
    Texture.WhenAllReady(textures, () => {
      if (!this.thawed.has(mat) && !this.thawedUniforms.has(mat)) mat.freeze();
    });
  }
}

/**
 * A texture from the build's compressed copy (KTX2, which carries its own colour space) when
 * there is one, else from the file; a copy that fails to load falls back to the file.
 */
export function textureWithFallback(source: string, scene: Scene, srgb: boolean, anisotropy?: number): Texture {
  // Normal and ARM maps sit next to the albedo (`new URL(entry.normal, base)`), without the build's query.
  const url = withBuild(source);
  const copy = optimizedUrl(url);
  const tex: Texture = new Texture(copy, scene, {
    invertY: false,
    samplingMode: Texture.TRILINEAR_SAMPLINGMODE,
    gammaSpace: srgb,
    onError: () => {
      if (tex.url === url) return;
      console.warn(`[render] compressed ${copy} failed, loading ${url}`);
      tex.updateURL(url);
    },
  });
  tex.gammaSpace = srgb;
  if (anisotropy !== undefined) tex.anisotropicFilteringLevel = anisotropy;
  return tex;
}

/**
 * The sky's lighting as `index.html` started fetching it with the page (`earlyEnvironment` in
 * vite.config.ts), if that's `url`. Taken once: later loads (the portrait studios) fetch it again,
 * from the browser's cache.
 */
function earlySky(url: string): Promise<ArrayBuffer | null> | null {
  const page = self as { __skyEnv?: { url: string; bytes: Promise<ArrayBuffer | null> } };
  const early = page.__skyEnv;
  if (!early || new URL(early.url, location.href).href !== url) return null;
  delete page.__skyEnv;
  return early.bytes;
}

/** Whether a lighting value moved enough to pass on (see `LIGHT_CHANGE`). */
function visibleChange(from: number, to: number): boolean {
  // Going off (or coming on) always counts.
  if (from === 0 || to === 0) return from !== to;
  return Math.abs(to - from) >= Math.max(LIGHT_CHANGE_MIN, Math.max(Math.abs(from), Math.abs(to)) * LIGHT_CHANGE);
}
