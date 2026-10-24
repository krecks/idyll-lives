/**
 * The residents drawn by Babylon: what they do and how they are posed is engine-neutral
 * (`shared/characters/residents.ts`); this is the `ResidentView` that draws them (`shared/characters/view.ts`).
 *
 * - One mesh per (body, part), drawn with thin instances (one per Sim wearing it; `setPartRows`, `partRows`).
 * - Skinning on the GPU through Babylon's baked-vertex-animation path: the pose texture (`simPoses`, one row
 *   per Sim) is the VAT manager's texture, and each instance's settings (start frame = end frame = its row,
 *   speed 0) read "its" row; `CharacterPlugin` (`characters/material.ts`) adds the face morphs before
 *   skinning and paints the appearance texels. A skeleton per body family only declares the bone count.
 * - Shadow-only copies of the low-detail meshes on `LAYER_SHADOW_ONLY` (the sun's shadow map draws its own list).
 * - Quilts (`characters/blanket.ts`), props (`models.ts` templates) and the selection ring as thin instances
 *   whose matrices the residents' code writes.
 */

import { Bone, Buffer, Color3, Constants, DynamicTexture, MeshBuilder, StandardMaterial, Mesh, PBRMaterial, RawTexture, Skeleton, Matrix, VertexData, type Engine, type Scene, type WebGPUEngine } from './core';
import { BakedVertexAnimationManager } from '@babylonjs/core/BakedVertexAnimation/bakedVertexAnimationManager';
import type { AssetRegistry } from '../../assets/registry';
import type { ModelEntry } from '../../assets/types';
import type { Content } from '../../content/content';
import type { PartColors } from '../../core/protocol';
import type { BodyData, CharacterSet } from '../shared/characters/data';
import type { QuiltGeometry } from '../shared/characters/blanket';
import { partColors, surfaceOf } from '../shared/characters/parts';
import { Characters as SharedCharacters, MAX_CHARACTERS } from '../shared/characters/residents';
import type { ResidentProp, ResidentView, RowChange } from '../shared/characters/view';
import { CharacterPlugin, type MorphBinding } from './characters/material';
import { blanketMaterial, buildBlanket, type BlanketShape } from './characters/blanket';
import type { MaterialLibrary } from './materials';
import { LAYER_SHADOW_ONLY } from '../shared/layers';
import { buildModel, fileColour, onModelFileArrived, type ModelTemplate } from './models';
import { ITEM_STAND_IN, painted } from '../shared/modelEntry';
import { TemplateCache } from './templateCache';

export { EXPRESSIONS, FRIENDLY, GAME_RATE, MAX_CHARACTERS, QUIET_SPEED, STANCE, type Face, type StageDirection } from '../shared/characters/residents';

/** A prop's meshes (one thin instance per Sim row), its matrices and its top. */
interface Prop extends ResidentProp {
  meshes: Mesh[];
}

/** The residents' meshes, materials and textures in a Babylon scene. */
class BabylonResidents implements ResidentView<Mesh, BlanketShape, Prop> {
  private texture!: RawTexture;
  private data!: Float32Array;
  private width = 0;
  private bones = 0;
  private vat!: BakedVertexAnimationManager;
  /** Face morphs (null for sets without them). */
  private morph: MorphBinding | null = null;
  private set: CharacterSet | null = null;
  private readonly materials = new Map<string, PBRMaterial>();
  private readonly skeletons = new Map<string, Skeleton>();
  /** The quilts' shared material, for the life stages' quilts made later. */
  private quilt: PBRMaterial | null = null;
  private ring: Mesh | null = null;
  readonly props: TemplateCache<Prop | null>;

  constructor(
    private readonly scene: Scene,
    private readonly assets: AssetRegistry,
    private readonly lib: MaterialLibrary,
  ) {
    this.props = new TemplateCache(scene);
  }

  init(set: CharacterSet, data: Float32Array, width: number, rows: number): void {
    this.set = set;
    this.data = data;
    this.width = width;
    this.bones = set.bones.length;
    this.texture = new RawTexture(data, width, rows, Constants.TEXTUREFORMAT_RGBA, this.scene, false, false, Constants.TEXTURE_NEAREST_SAMPLINGMODE, Constants.TEXTURETYPE_FLOAT);
    this.texture.name = 'simPoses';
    this.vat = new BakedVertexAnimationManager(this.scene);
    this.vat.texture = this.texture;
    this.vat.isEnabled = true;
    const morphs = set.morphs;
    if (morphs) {
      const texture = new RawTexture(
        morphs.data,
        morphs.width,
        morphs.height,
        Constants.TEXTUREFORMAT_RGBA,
        this.scene,
        false,
        false,
        Constants.TEXTURE_NEAREST_SAMPLINGMODE,
        Constants.TEXTURETYPE_HALF_FLOAT,
      );
      texture.name = 'simMorphs';
      this.morph = { texture, width: morphs.width, header: morphs.headerTexels };
    }
  }

  uploadPoses(): void {
    this.texture.update(this.data);
  }

  uploadPoseRows(from: number, count: number): void {
    const texture = this.texture.getInternalTexture();
    if (!texture) return;
    const stride = this.width * 4;
    (this.scene.getEngine() as Engine | WebGPUEngine).updateTextureData(texture, this.data.subarray(from * stride, (from + count) * stride), 0, from, this.width, count);
  }

  createPart(body: BodyData, part: string, shadow: boolean): Mesh {
    const data = body.parts.get(part)!;
    const mesh = new Mesh(`${shadow ? 'simShadow' : 'sim'}:${body.name}:${part}`, this.scene);
    const vd = new VertexData();
    vd.positions = data.positions;
    vd.normals = data.normals;
    vd.indices = data.indices;
    vd.matricesIndices = data.joints;
    vd.matricesWeights = data.weights;
    vd.applyToMesh(mesh, false);
    // Fields, morph slots and tone interleaved in one buffer: WebGPU allows 8 vertex buffers per
    // draw (here: position, normal, joints, weights, this, the instance matrices and settings).
    // Parts no morph moves get slot -1 (and no `simMorph` attribute).
    const n = data.fields.length / 4;
    const all = new Float32Array(n * 12);
    for (let v = 0; v < n; v++) {
      all.set(data.fields.subarray(v * 4, v * 4 + 4), v * 12);
      if (data.morph) all.set(data.morph.subarray(v * 4, v * 4 + 4), v * 12 + 4);
      else all[v * 12 + 4] = -1;
      all.set(data.tone.subarray(v * 4, v * 4 + 4), v * 12 + 8);
    }
    const buffer = new Buffer(this.scene.getEngine(), all, false, 12);
    mesh.setVerticesBuffer(buffer.createVertexBuffer('simField', 0, 4));
    if (data.morph && this.morph) mesh.setVerticesBuffer(buffer.createVertexBuffer('simMorph', 4, 4));
    mesh.setVerticesBuffer(buffer.createVertexBuffer('simTone', 8, 4));
    mesh.skeleton = this.skeleton(body);
    mesh.numBoneInfluencers = 4;
    mesh.bakedVertexAnimationManager = this.vat;
    mesh.material = this.material(part);
    mesh.isPickable = false;
    mesh.receiveShadows = !shadow;
    // Shadow copies: on a layer no camera draws (the shadow map draws its own list of meshes).
    if (shadow) mesh.layerMask = LAYER_SHADOW_ONLY;
    // Sims move every frame (bones carry the placement): skip culling and bounds work.
    mesh.alwaysSelectAsActiveMesh = true;
    mesh.doNotSyncBoundingInfo = true;
    mesh.freezeWorldMatrix();
    return mesh;
  }

  partEnabled(mesh: Mesh, on: boolean): void {
    mesh.setEnabled(on);
  }

  setPartRows(mesh: Mesh, rows: Float32Array, perFrame: boolean): void {
    const n = rows.length / 4;
    const identity = new Float32Array(n * 16);
    for (let j = 0; j < n; j++) identity[j * 16] = identity[j * 16 + 5] = identity[j * 16 + 10] = identity[j * 16 + 15] = 1;
    mesh.thinInstanceSetBuffer('matrix', identity, 16, true);
    mesh.thinInstanceSetBuffer('bakedVertexAnimationSettingsInstanced', rows, 4, !perFrame);
    mesh.isVisible = n > 0;
  }

  partRows(mesh: Mesh, rows: Float32Array, count: number, change: RowChange): void {
    if (change === 'new') mesh.thinInstanceSetBuffer('bakedVertexAnimationSettingsInstanced', rows, 4, true);
    mesh.thinInstanceCount = count;
    if (change === 'rows') mesh.thinInstanceBufferUpdated('bakedVertexAnimationSettingsInstanced');
    // (With no thin instances Babylon would draw the mesh once, uninstanced: unposed and untinted.)
    mesh.isVisible = count > 0;
  }

  quiltMaterial(toeZ: number, footZ: number): void {
    this.quilt = blanketMaterial(this.scene, toeZ, footZ);
    this.lib.adopt(this.quilt);
  }

  createQuilt(name: string, geometry: QuiltGeometry, matrices: Float32Array, ext: Float32Array, tint: Float32Array): BlanketShape {
    const shape = buildBlanket(this.scene, name, geometry, this.quilt!);
    shape.mesh.thinInstanceSetBuffer('matrix', matrices, 16, false);
    shape.mesh.thinInstanceSetBuffer('blanketExt', ext, 2, false);
    shape.mesh.thinInstanceSetBuffer('blanketTint', tint, 3, false);
    shape.mesh.thinInstanceCount = 0;
    shape.mesh.isVisible = false;
    shape.mesh.setEnabled(false);
    return shape;
  }

  quiltRows(quilt: BlanketShape, count: number, upload: boolean): void {
    quilt.mesh.thinInstanceCount = count;
    // (None: not drawn at all; Babylon draws a mesh with no thin instances once as it is, at the origin.)
    quilt.mesh.isVisible = count > 0;
    if (!upload) return;
    quilt.mesh.thinInstanceBufferUpdated('matrix');
    quilt.mesh.thinInstanceBufferUpdated('blanketExt');
    quilt.mesh.thinInstanceBufferUpdated('blanketTint');
  }

  quiltEnabled(quilt: BlanketShape, on: boolean): void {
    quilt.mesh.setEnabled(on);
  }

  /** A soft glowing ring on the floor around the selected Sim (one thin instance, moved per frame). */
  createRing(matrix: Float32Array): void {
    const size = 256;
    const tex = new DynamicTexture('simRing', size, this.scene, true);
    const ctx = tex.getContext() as CanvasRenderingContext2D;
    const g = ctx.createRadialGradient(size / 2, size / 2, size * 0.3, size / 2, size / 2, size / 2);
    g.addColorStop(0, 'rgba(255,255,255,0)');
    g.addColorStop(0.55, 'rgba(255,255,255,0.0)');
    g.addColorStop(0.72, 'rgba(255,255,255,0.85)');
    g.addColorStop(0.8, 'rgba(255,255,255,0.55)');
    g.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.clearRect(0, 0, size, size);
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, size, size);
    tex.update();
    tex.hasAlpha = true;
    const mat = new StandardMaterial('simRing', this.scene);
    mat.disableLighting = true;
    mat.diffuseColor = Color3.Black();
    mat.specularColor = Color3.Black();
    mat.emissiveColor = Color3.FromHexString('#7BE38A');
    mat.opacityTexture = tex;
    mat.disableDepthWrite = true;
    mat.zOffset = -4;
    const ring = MeshBuilder.CreateGround('simRing', { width: 1, height: 1 }, this.scene);
    ring.material = mat;
    ring.isPickable = false;
    ring.alwaysSelectAsActiveMesh = true;
    ring.thinInstanceSetBuffer('matrix', matrix, 16, false);
    this.ring = ring;
  }

  ringMoved(): void {
    this.ring?.thinInstanceBufferUpdated('matrix');
  }

  /** Builds prop `key`; without its file, a small block (it's built again when the file comes). */
  async buildProp(key: string): Promise<Prop | null> {
    const at = key.indexOf('#');
    const p =
      at < 0
        ? await buildModel(this.scene, `prop:${key}`, this.assets.get(key, 'model'), [1, 1], { standIn: ITEM_STAND_IN }).then((t) => ({ meshes: t.meshes, top: [0, 0, 0] as const, failed: t.failed }))
        : await this.partProp(key.slice(0, at), ...partColors(key.slice(at + 1)));
    if (!p) return null;
    const matrices = new Float32Array(MAX_CHARACTERS * 16);
    for (const mesh of p.meshes) {
      mesh.isPickable = false;
      mesh.thinInstanceSetBuffer('matrix', matrices, 16, false);
      mesh.alwaysSelectAsActiveMesh = true;
    }
    return { meshes: p.meshes, matrices, top: p.top, failed: p.failed };
  }

  /**
   * Moving part `part` of model `model` on its own (a held instrument), in the part's own frame (its node's
   * origin and axes at rest), painted in `colors` as the object is; and its top (the highest point: a mop's
   * handle); null if the model has no such part (nothing is drawn, and nothing is hidden on
   * the object either). Without the model's file, a small block in the hand.
   */
  private async partProp(model: string, part: string, colors?: PartColors): Promise<{ meshes: Mesh[]; top: readonly [number, number, number]; failed?: string[] } | null> {
    const t: ModelTemplate = await buildModel(this.scene, `prop:${model}#${part}`, painted(this.assets.get(model, 'model'), colors), [1, 1], { standIn: ITEM_STAND_IN });
    if (t.failed) return { meshes: t.meshes, top: [0, t.height, 0], failed: t.failed };
    const motion = t.motion;
    const j = motion?.names?.indexOf(part) ?? -1;
    const keep = t.meshes.filter((_, k) => j >= 0 && motion!.parts[k] === j);
    for (const mesh of t.meshes) if (!keep.includes(mesh)) mesh.dispose(false, false);
    for (const glow of t.glows ?? []) glow.mesh.dispose(false, false);
    if (j < 0 || !motion?.rests) return null;
    const local = motion.rests[j].clone().invert();
    const top: [number, number, number] = [0, -Infinity, 0];
    for (const mesh of keep) {
      mesh.bakeTransformIntoVertices(local);
      const p = mesh.getVerticesData('position') ?? [];
      for (let v = 1; v < p.length; v += 3) if (p[v] > top[1]) [top[0], top[1], top[2]] = [p[v - 1], p[v], p[v + 1]];
    }
    return { meshes: keep, top };
  }

  showProp(prop: Prop, count: number): void {
    for (const mesh of prop.meshes) {
      mesh.isVisible = count > 0;
      if (count > 0) {
        mesh.thinInstanceCount = count;
        mesh.thinInstanceBufferUpdated('matrix');
      }
    }
  }

  onModelFile(listener: (url: string) => void): () => void {
    return onModelFileArrived(this.scene, listener);
  }

  fileColour(entry: ModelEntry | undefined, name: string): Promise<string | undefined> {
    return fileColour(this.scene, entry, name);
  }

  camera(out: Float32Array): boolean {
    const cam = this.scene.activeCamera;
    if (!cam) return false;
    const c = cam.globalPosition;
    out[0] = c.x;
    out[1] = c.y;
    out[2] = c.z;
    out[3] = this.scene.getEngine().getRenderHeight();
    out[4] = cam.mode === 0 ? cam.fov : NaN;
    return true;
  }

  onDispose(fn: () => void): void {
    this.scene.onDisposeObservable.addOnce(fn);
  }

  /** A skeleton only declares the bone count to the shaders; poses come from the texture. */
  private skeleton(body: BodyData): Skeleton {
    let sk = this.skeletons.get(body.family);
    if (sk) return sk;
    const set = this.set!;
    sk = new Skeleton(`sim:${body.family}`, `sim:${body.family}`, this.scene);
    const bones: Bone[] = [];
    set.bones.forEach((name, b) => {
      const p = set.parents[b];
      bones.push(new Bone(name, sk!, p >= 0 ? bones[p] : null, Matrix.Identity()));
    });
    this.skeletons.set(body.family, sk);
    return sk;
  }

  private material(part: string): PBRMaterial {
    const kind = surfaceOf(part);
    const key = kind === 'mouth' ? `${kind}:${part}` : kind;
    const cached = this.materials.get(key);
    if (cached) return cached;
    const mat = new PBRMaterial(`sim:${key}`, this.scene);
    mat.metallic = 0;
    mat.maxSimultaneousLights = 6;
    // No textures: every colour comes from the Sim's appearance texels and the vertex tone (see the plugin).
    mat.albedoColor = new Color3(1, 1, 1);
    if (kind === 'body') {
      mat.roughness = 0.55;
    } else if (kind === 'cloth') {
      // As rough as the clothes painted on the body (so shells and paint match).
      mat.roughness = 0.85;
      // Open hems and collars show their inside.
      mat.backFaceCulling = false;
      mat.twoSidedLighting = true;
    } else if (kind === 'hair') {
      // No PBR sheen: linked to the albedo it takes the diffuse colour down to (1 - intensity)^5 (8 % at
      // 0.4), so every colour drew near black. The plugin draws a sheen band into the colour instead.
      mat.roughness = 0.5;
      mat.backFaceCulling = false;
      mat.twoSidedLighting = true;
    } else if (kind === 'brow' || kind === 'lash') {
      // Opaque: brows in the hair colour (see the plugin), lashes dark brown.
      if (kind === 'lash') mat.albedoColor = new Color3(0.16, 0.12, 0.1);
      mat.roughness = 0.6;
      mat.backFaceCulling = false;
      mat.twoSidedLighting = true;
    } else if (kind === 'mouth') {
      // Warm, soft and hardly reflecting the sky (the mouth hides most of it).
      mat.albedoColor = part === 'teeth' ? new Color3(1, 0.96, 0.9) : new Color3(0.85, 0.8, 0.8);
      mat.roughness = 0.5;
      mat.environmentIntensity = 0.35;
    } else {
      // Eyes: drawn by the plugin (sclera, iris, pupil); wet and glossy (a clear coat for the catchlight).
      mat.roughness = 0.25;
      mat.clearCoat.isEnabled = true;
      mat.clearCoat.intensity = 1;
      mat.clearCoat.roughness = 0.03;
    }
    new CharacterPlugin(mat, kind, this.bones * 4, this.texture, this.morph);
    this.lib.adopt(mat);
    this.materials.set(key, mat);
    return mat;
  }
}

/** The residents in a Babylon scene (`shared/characters/residents.ts`, drawn by `BabylonResidents`). */
export class Characters extends SharedCharacters<Mesh, BlanketShape, Prop> {
  constructor(
    scene: Scene,
    assets: AssetRegistry,
    lib: MaterialLibrary,
    /** For seats content describes (`seat`) and what can be sat or lain on. */
    content?: Content,
    options: {
      /**
       * Builds again the props drawn without a model file that came (the game's): the scene then tries
       * failed files in the background (`onModelFileArrived`). Off for the preview studio and the debug pages.
       */
      refreshProps?: boolean;
      /** Sims cast sun shadows (the shadow-only copies are built); off where they cast none (`QualitySettings.characterShadows`). */
      castsShadows?: boolean;
    } = {},
  ) {
    super(new BabylonResidents(scene, assets, lib), assets, content, options);
  }
}
