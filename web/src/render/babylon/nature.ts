/**
 * Landscape around the town: terrain with rolling hills, forests, lawn details and a sky
 * dome. Everything is static and instanced (a draw call per model type, and in the game per area,
 * see `FOREST_SECTORS`), generated deterministically from the town's seed so a loaded game looks the same.
 *
 * Trees, bushes, hedges, lawn tufts and the garden catalog's plants are Blender-built files
 * (tools/art/blender/families/plants.py, trees.py): crowns on the leafy `foliage` finish over `bark`
 * limbs, each tree and shrub with a far version for the distant woods.
 * `installNature` adds vertex-shader wind and per-instance tint variation to those two finishes;
 * `natureDecor` turns the house builder's garden dressing into species (hedges or mixed borders,
 * mixed lot trees kept clear of the street and paths).
 */

import {
  Color3,
  Constants,
  MaterialPluginBase,
  Mesh,
  MeshBuilder,
  PBRMaterial,
  ShaderLanguage,
  StandardMaterial,
  Texture,
  Vector3,
  VertexBuffer,
  VertexData,
  type AbstractEngine,
  type Material,
  type MaterialDefines,
  type Nullable,
  type Scene,
  type SubMesh,
  type UniformBuffer,
} from './core';
import type { AssetRegistry } from '../../assets/registry';
import type { WorldStructure } from '../../core/protocol';
import type { Lighting } from '../shared/environment';
import { groupInstances, hideLawn, LandscapeLayout, lawnLayer, lawnMask, smoothstep, SPLIT_TRIANGLES, WIND, WIND_KIND, windSwayAt, type LawnLayer, type LawnMask } from '../shared/nature';
import { MaterialLibrary } from './materials';
import { buildModel } from './models';
import { CountryRoads } from './roads';
import { perfEnd, perfStart, perfTimeAsync } from '../../debug/perf';

export { decorPlants, hash, lawnMask, mulberry32, natureDecor, smoothstep, unlawned, windDirection, type LawnMask, type PlantPlacement } from '../shared/nature';

export interface NatureResult {
  meshes: Mesh[];
  /**
   * Meshes that should cast shadows: the woods near the town, whole in the game too (its shadow
   * map round the viewed lot would draw each group reaching into it, a draw each).
   */
  casters: Mesh[];
  /** Plants drawn only in the game (in groups) or only in the menus' town (whole); the rest are drawn in both. */
  only: Map<Mesh, 'game' | 'town'>;
}

// ---- Landscape ----------------------------------------------------------------------

/** The landscape as Babylon draws it (its layout and growth: `shared/nature.ts`). */
export class Landscape extends LandscapeLayout<CountryRoads> {
  /** Lawn tufts and flowers, per model, whole and in groups (see `TUFT_CELL`). */
  private lawn: LawnLayer<Mesh>[] = [];
  /** What `refreshLawn` last hid, for the lawn built again since (`rebuild`). */
  private lawnMask: LawnMask | null = null;
  /** Each model's plants as built, to build again when a file they were built without comes (`rebuild`). */
  private batches: Batch[] = [];

  constructor(world: WorldStructure, lawnDensity = 1, lawnLod: 0 | 1 = 0) {
    super(world, (land, w, name) => new CountryRoads(land, w, name), lawnDensity, lawnLod);
  }

  async build(scene: Scene, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>): Promise<NatureResult> {
    const meshes: Mesh[] = [];
    meshes.push(await perfTimeAsync('nature: terrain', async () => this.terrain(scene, pace)));
    if (this.watered) meshes.push(await perfTimeAsync('nature: water', async () => this.water(scene)));
    meshes.push(...(await perfTimeAsync('nature: roads', async () => this.roads.build(scene))));
    this.batches = [];
    await perfTimeAsync('nature: forests', () => this.forests(scene, assets, pace));
    await perfTimeAsync('nature: lawns', () => this.plantLawns(scene, assets, world, pace));
    const plants = gather(this.batches);
    return { meshes: [...meshes, ...plants.meshes], casters: plants.casters, only: plants.only };
  }

  /**
   * Builds again the plants built without the file at `url`, now that it has come: the new meshes
   * (on no layer yet, `layerMask` 0, like a first build's), the ones they replace, and `commit`,
   * which puts the new in the old ones' place here (the lawn's hiding goes over to them). The caller
   * draws the new meshes and disposes the old ones as it commits. Null when none was built without
   * it. Not while `build` or another `rebuild` runs.
   */
  async rebuild(scene: Scene, assets: AssetRegistry, url: string, pace?: () => Promise<void>): Promise<NatureRebuild | null> {
    const stale = this.batches.filter((b) => b.again && b.built.failed?.includes(url));
    if (!stale.length) return null;
    const fresh: Batch[] = [];
    for (const { again, casts } of stale) {
      const { key, matrices, lod, fixed, cell } = again!;
      fresh.push(batch(again!, casts, await instanced(scene, assets, key, matrices, lod, fixed, cell, pace)));
    }
    const replaced = gather(stale).meshes;
    const commit = () => {
      this.batches = this.batches.map((b) => fresh[stale.indexOf(b)] ?? b);
      const going = new Set(replaced);
      this.lawn = this.lawn.filter((layer) => !layer.draws.some((mesh) => going.has(mesh)));
      stale.forEach(({ again }, i) => {
        if (!again!.tiles) return;
        const layers = lawnLayers(fresh[i].built, again!.matrices, again!.tiles);
        this.lawn.push(...layers);
        if (this.lawnMask) hideLawn(layers, this.lawnMask, uploadLawn);
      });
    };
    return { ...gather(fresh), replaced, commit };
  }

  private async terrain(scene: Scene, pace?: () => Promise<void>): Promise<Mesh> {
    const { size, segments } = this.terrainGrid();
    const ground = MeshBuilder.CreateGround('terrain', { width: size, height: size, subdivisions: segments, updatable: false }, scene);
    ground.position.set(this.w / 2, 0, this.d / 2);
    ground.bakeCurrentTransformIntoVertices();

    const positions = ground.getVerticesData(VertexBuffer.PositionKind)!;
    const colors = new Float32Array((positions.length / 3) * 4);
    // sRGB vertex colours (converted on the GPU) multiply the grass detail texture.
    await this.shadeTerrain(positions, colors, pace);
    const normals: number[] = [];
    VertexData.ComputeNormals(positions, ground.getIndices()!, normals);
    ground.setVerticesData(VertexBuffer.PositionKind, positions);
    ground.setVerticesData(VertexBuffer.NormalKind, normals);
    ground.setVerticesData(VertexBuffer.ColorKind, colors);
    ground.refreshBoundingInfo();

    ground.material = MaterialLibrary.for(scene).surface('material.ground', { vertexColors: true });
    ground.receiveShadows = true;
    return ground;
  }

  /**
   * The sea or the lake: one flat sheet at the water line under the whole landscape (the land
   * covers it everywhere but in the water), reflecting the sky.
   */
  private water(scene: Scene): Mesh {
    const at = this.waterSheet();
    const sheet = MeshBuilder.CreateGround('water', { width: at.size, height: at.size }, scene);
    sheet.position.set(at.x, at.y, at.z);
    sheet.bakeCurrentTransformIntoVertices();
    const mat = new PBRMaterial('water', scene);
    mat.albedoColor = Color3.FromHexString('#2F6F86').toLinearSpace();
    mat.metallic = 0;
    mat.roughness = 0.12;
    mat.alpha = 0.9;
    mat.environmentIntensity = 0.9;
    sheet.material = mat;
    sheet.receiveShadows = true;
    sheet.isPickable = false;
    return sheet;
  }

  /** The woods (`woods`), each model drawn with thin instances, in groups by sector round the town. */
  private async forests(scene: Scene, assets: AssetRegistry, pace?: () => Promise<void>): Promise<void> {
    const { near, far, sector } = await this.woods(pace);
    for (const [bucket, casts] of [
      [near, true],
      [far, false],
    ] as const) {
      for (const [key, list] of bucket) {
        const lod = casts ? 0 : 1;
        this.batches.push(batch({ key, matrices: list, lod, fixed: true, cell: sector }, casts, await instanced(scene, assets, key, list, lod, true, sector, pace)));
      }
    }
  }

  /** The lawn (`lawns`), each model drawn with thin instances, the tufts in squares. */
  private async plantLawns(scene: Scene, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>): Promise<void> {
    this.lawn = [];
    for (const { key, matrices, tiles, cell } of await this.lawns(world, pace)) {
      const lod = this.lawnLod;
      const built = await instanced(scene, assets, key, matrices, lod, false, cell, pace);
      this.batches.push(batch({ key, matrices, lod, fixed: false, cell, tiles }, false, built));
      this.lawn.push(...lawnLayers(built, matrices, tiles));
    }
    this.refreshLawn(world);
  }

  /**
   * Hides the lawn on tiles that are now indoors or under furniture, and shows it again elsewhere.
   * On a tile split by a diagonal wall, each tuft follows the half it stands in.
   */
  refreshLawn(world: WorldStructure): void {
    this.lawnMask = lawnMask(world);
    hideLawn(this.lawn, this.lawnMask, uploadLawn);
  }
}

/** A lawn layer's matrices as drawn now, up to its meshes. */
function uploadLawn(layer: LawnLayer<Mesh>): void {
  for (const mesh of layer.draws) mesh.thinInstanceSetBuffer('matrix', layer.shown, 16, false);
}

/**
 * A model's meshes drawn with thin instances: the parts drawn whole in the game and the menus'
 * town (`shared`), and those also drawn in groups (see `SPLIT_TRIANGLES`): whole in the town
 * (`whole`), in groups in the game (`cells`; each keeps its instances in their order, `indices`
 * into `matrices`).
 */
interface Instanced {
  shared: Mesh[];
  whole: Mesh[];
  cells: { meshes: Mesh[]; indices: number[] }[];
  /** The model's files that failed to load (`ModelTemplate.failed`): it's built again when one comes (`Landscape.rebuild`). */
  failed?: string[];
}

/** One model's plants, as the landscape keeps them. */
interface Batch {
  built: Instanced;
  /** Its parts drawn whole cast shadows (the woods near the town). */
  casts: boolean;
  /** What it was built from, kept only while it's built without a file (`built.failed`), to build it again when that comes (`rebuild`). */
  again?: BatchSource;
}

/** `instanced`'s arguments for a model's plants. */
interface BatchSource {
  key: string;
  matrices: Float32Array[];
  lod: 0 | 1;
  fixed: boolean;
  cell?: (x: number, z: number) => string;
  /** The lawn's: each instance's town tile (-1 outside the town), for `refreshLawn`. */
  tiles?: readonly number[];
}

/** A batch of plants built from `source`, which it keeps only if it was built without a file (all of them are several MB a landscape). */
function batch(source: BatchSource, casts: boolean, built: Instanced): Batch {
  return { built, casts, ...(built.failed ? { again: source } : {}) };
}

/** A landscape's plants built again (`Landscape.rebuild`): the new meshes, the ones they replace, and the swap. */
export interface NatureRebuild extends NatureResult {
  replaced: Mesh[];
  commit: () => void;
}

/** The meshes of `batches`: which cast shadows, and which are drawn only in the game or only in the menus' town (see `NatureResult`). */
function gather(batches: readonly Batch[]): NatureResult {
  const meshes: Mesh[] = [];
  const casters: Mesh[] = [];
  const only = new Map<Mesh, 'game' | 'town'>();
  for (const { built, casts } of batches) {
    const groups = built.cells.flatMap((cell) => cell.meshes);
    meshes.push(...built.shared, ...built.whole, ...groups);
    for (const mesh of built.whole) only.set(mesh, 'town');
    for (const mesh of groups) only.set(mesh, 'game');
    if (casts) casters.push(...built.shared, ...built.whole);
  }
  return { meshes, casters, only };
}

/** The lawn layers of a model's tufts or flowers: every instance, in the parts drawn whole; then each group's. */
function lawnLayers(built: Instanced, matrices: readonly Float32Array[], tiles: readonly number[]): LawnLayer<Mesh>[] {
  const layers: LawnLayer<Mesh>[] = [];
  for (const { meshes, indices } of [{ meshes: [...built.shared, ...built.whole], indices: matrices.map((_, k) => k) }, ...built.cells]) {
    if (!meshes.length) continue;
    layers.push(lawnLayer(meshes, matrices, tiles, indices));
  }
  return layers;
}

/**
 * One template per model key, drawn with thin instances; static, so bounds are computed once.
 * `cell` groups the instances by where they stand: the parts with enough triangles get a copy per
 * group (sharing the template's materials), which the game's camera leaves out when out of view.
 */
async function instanced(
  scene: Scene,
  assets: AssetRegistry,
  key: string,
  matrices: Float32Array[],
  lod: 0 | 1 = 0,
  fixed = true,
  cell?: (x: number, z: number) => string,
  pace?: () => Promise<void>,
): Promise<Instanced> {
  if (matrices.length === 0) return { shared: [], whole: [], cells: [] };
  await pace?.();
  // Without its file, nothing, until the file comes (`Landscape.rebuild`).
  const template = await perfTimeAsync(`nature: model ${key}`, () => buildModel(scene, key, assets.get(key, 'model'), [1, 1], { lod }));
  // On no layer until its owner draws it: built again, it comes in beside what's drawn, unseen until it's whole.
  for (const mesh of template.meshes) mesh.layerMask = 0;
  await pace?.();
  const groups = groupInstances(matrices, cell);
  const split = groups.size > 1 ? template.meshes.filter((mesh) => (mesh.getTotalIndices() / 3) * matrices.length >= SPLIT_TRIANGLES) : [];
  // Copies are made before any instances are set, each with its own geometry: Babylon keeps a
  // mesh's instance buffer with its geometry's vertex buffers. New meshes, not clones: a clone
  // gets the template's fields as its own, an object of another shape, and that slowed Babylon's
  // loops over all the scene's meshes each frame (at 3x CPU slow-down, +0.7 ms a frame in the
  // menus' town, whose groups are switched off).
  const copy = (mesh: Mesh, c: number) => {
    const part = new Mesh(`${mesh.name}#${c}`, scene);
    mesh.geometry?.copy(`${mesh.geometry.id}#${c}`).applyToMesh(part);
    part.material = mesh.material;
    part.layerMask = 0;
    // Groups are flat and wide: their boxes fit them far better than the spheres Babylon tests by default.
    part.cullingStrategy = Constants.MESHES_CULLINGSTRATEGY_STANDARD;
    return part;
  };
  const cells = split.length ? [...groups.values()].map((indices, c) => ({ meshes: split.map((mesh) => copy(mesh, c)), indices })) : [];
  const span = perfStart(`nature: instances ${key}`);
  for (const { meshes, indices } of [{ meshes: template.meshes, indices: matrices.map((_, k) => k) }, ...cells]) {
    const buffer = new Float32Array(indices.length * 16);
    for (let i = 0; i < indices.length; i++) {
      if (i > 0 && i % 2048 === 0) await pace?.();
      buffer.set(matrices[indices[i]], i * 16);
    }
    for (const mesh of meshes) {
      mesh.thinInstanceSetBuffer('matrix', buffer, 16, fixed);
      mesh.thinInstanceRefreshBoundingInfo(false);
      mesh.receiveShadows = true;
      mesh.freezeWorldMatrix();
    }
    await pace?.();
  }
  perfEnd(span);
  return { shared: template.meshes.filter((mesh) => !split.includes(mesh)), whole: split, cells, ...(template.failed ? { failed: template.failed } : {}) };
}

// ---- Wind and foliage shading -----------------------------------------------------------

/** Wind shared by every foliage material of a scene; `time` advances per frame. */
const windState = new WeakMap<Scene, { time: number; strength: number }>();

/** A number as a shader's float literal. */
const glsl = (n: number) => (Number.isInteger(n) ? n.toFixed(1) : String(n));

/**
 * Vertex-shader wind for plants (parts on the `bark` and `foliage` finishes): a slow
 * height-weighted bend with gusts plus leaf flutter, phased per instance; and, on foliage, a small
 * per-instance colour variation so repeated trees don't look stamped. Short plants hardly move.
 */
class NatureWindPlugin extends MaterialPluginBase {
  constructor(
    material: Material,
    private readonly kind: [number, number, number, number],
    private readonly state: { time: number; strength: number },
    private readonly foliage: boolean,
  ) {
    super(material, 'NatureWind', 230, { NATUREWIND: false, NATURETINT: false }, true, false);
    this.registerForExtraEvents = true;
    this._enable(true);
  }

  override getClassName(): string {
    return 'NatureWindPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override prepareDefines(defines: MaterialDefines): void {
    defines.NATUREWIND = true;
    defines.NATURETINT = this.foliage;
  }

  override getUniforms(): { externalUniforms: string[] } {
    return { externalUniforms: ['natureWind', 'natureWindKind'] };
  }

  override hardBindForSubMesh(_ubo: UniformBuffer, _scene: Scene, _engine: AbstractEngine, subMesh: SubMesh): void {
    const effect = subMesh.effect;
    if (!effect) return;
    const k = this.kind;
    // A model's part grown from nothing (`windStill`, models.ts) bends but doesn't flutter: flat at rest, it stays hidden.
    const still = (subMesh.getMesh().metadata as { windStill?: boolean } | null)?.windStill;
    effect.setFloat4('natureWind', this.state.time, this.state.strength, WIND.dir[0], WIND.dir[1]);
    effect.setFloat4('natureWindKind', k[0], still ? 0 : k[1], k[2], k[3]);
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    if (shaderType !== 'vertex') return null;
    const wgsl = shaderLanguage === ShaderLanguage.WGSL;
    const definitions = wgsl
      ? '#ifdef NATUREWIND\nuniform natureWind: vec4f;\nuniform natureWindKind: vec4f;\n#endif\n'
      : '#ifdef NATUREWIND\nuniform vec4 natureWind;\nuniform vec4 natureWindKind;\n#endif\n';
    const u = (n: string) => (wgsl ? `uniforms.${n}` : n);
    const v = (n: string) => (wgsl ? `vertexInputs.${n}` : n);
    const f3 = wgsl ? 'vec3f' : 'vec3';
    const f2 = wgsl ? 'vec2f' : 'vec2';
    const decl = (name: string, type: string) => (wgsl ? `var ${name}: ${type === 'float' ? 'f32' : type === 'vec3' ? 'vec3f' : type}` : `${type} ${name}`);
    const { gust: g, phase, sway } = WIND;
    const body = `
#ifdef NATUREWIND
{
  ${decl('inst', 'vec3')} = ${f3}(0.0);
  ${decl('lw', 'vec3')} = ${f3}(${u('natureWind')}.z, 0.0, ${u('natureWind')}.w);
  // The instance's scale along each axis: flutter is in world metres, however far a moving part is scaled
  // (a part grown from nothing is 1000x along its height once grown).
  ${decl('scl', 'vec3')} = ${f3}(1.0);
#ifdef INSTANCES
  inst = ${v('world3')}.xyz;
  ${decl('s2', 'float')} = max(dot(${v('world0')}.xyz, ${v('world0')}.xyz), 0.0001);
  lw = ${f3}(dot(lw, ${v('world0')}.xyz), 0.0, dot(lw, ${v('world2')}.xyz)) / s2;
  scl = max(${f3}(length(${v('world0')}.xyz), length(${v('world1')}.xyz), length(${v('world2')}.xyz)), ${f3}(0.0001));
#endif
  ${decl('t', 'float')} = ${u('natureWind')}.x;
  ${decl('ph', 'float')} = dot(inst.xz, ${f2}(${glsl(phase[0])}, ${glsl(phase[1])}));
  ${decl('h', 'float')} = max(positionUpdated.y, 0.0);
  ${decl('gust', 'float')} = (${glsl(g.base)} + ${glsl(g.amp)} * sin(t * ${glsl(g.rate)} + ph * ${glsl(g.phase)})) * ${u('natureWind')}.y;
  ${decl('sway', 'float')} = (${sway.map((s) => `sin(t * ${glsl(s.rate)} + ph * ${glsl(s.phase)}) * ${glsl(s.weight)}`).join(' + ')}) * gust;
  ${decl('kind', 'vec4f')} = ${u('natureWindKind')};
  ${decl('fp', 'float')} = dot(positionUpdated, ${f3}(3.1, 1.7, 2.3)) + ph * 3.0;
  ${decl('fl', 'float')} = kind.y * min(h * kind.w, 1.0) * gust;
  positionUpdated = positionUpdated + lw * (sway * kind.x * h * h) + ${f3}(sin(t * kind.z + fp), 0.6 * sin(t * kind.z * 1.3 + fp * 1.3), cos(t * kind.z * 0.9 + fp)) * fl / scl;
#if defined(NATURETINT) && defined(VERTEXCOLOR)
  ${decl('rnd', 'float')} = fract(sin(dot(inst.xz, ${f2}(12.9898, 78.233))) * 43758.5453);
  colorUpdated = ${wgsl ? 'vec4f' : 'vec4'}(colorUpdated.rgb * mix(${f3}(1.07, 1.02, 0.84), ${f3}(0.9, 0.98, 1.04), rnd) * (0.92 + 0.14 * fract(rnd * 7.31)), colorUpdated.a);
#endif
}
#endif
`;
    return {
      CUSTOM_VERTEX_DEFINITIONS: definitions,
      CUSTOM_VERTEX_UPDATE_POSITION: wgsl ? body : body.replace(/vec4f/g, 'vec4'),
    };
  }
}

/**
 * How far the wind bends a plant standing at (`px`, `pz`) now, per m² of height up its model (m,
 * along `windDirection`): a point `h` m up (before the plant's scale) is where the vertex shader
 * draws it (the bend, not the leaves' flutter) this times `h²` away, so what sits on a branch sways
 * with it. Zero before the scene's wind exists.
 */
export function windSway(scene: Scene, px: number, pz: number): number {
  const state = windState.get(scene);
  if (!state) return 0;
  return windSwayAt(state.time, state.strength, px, pz);
}

/** How strongly the wind moves a scene's plants (1 as made; 0 still, for reduce motion). */
export function setWind(scene: Scene, strength: number): void {
  const state = windState.get(scene);
  if (state) state.strength = strength;
}

/**
 * Hooks the plant finishes as they are created (whoever builds the first plant): wind sway and the
 * per-instance tint. Also advances the wind clock. Call once per scene, before any model loads.
 */
export function installNature(scene: Scene): void {
  if (windState.has(scene)) return;
  const state = { time: 0, strength: 1 };
  windState.set(scene, state);
  scene.onBeforeRenderObservable.add(() => {
    state.time += Math.min(0.1, scene.getEngine().getDeltaTime() / 1000);
  });
  scene.onNewMaterialAddedObservable.add((material) => {
    const kind = WIND_KIND[material.name];
    if (!kind || !(material instanceof PBRMaterial)) return;
    new NatureWindPlugin(material, kind, state, material.name !== 'material.finish.bark');
  });
}


// ---- Sky ------------------------------------------------------------------------------

/**
 * Gradient sky dome with a glow around the sun (colours follow the time of day via vertex
 * colours, no custom shader) and a band of drifting clouds (`image.clouds`).
 */
export class Sky {
  private readonly dome: Mesh;
  private readonly dirs: Float32Array;
  private readonly colors: Float32Array;
  private readonly clouds: Mesh | null = null;
  private readonly cloudMaterial: StandardMaterial | null = null;
  private readonly zenith = new Color3();
  private readonly tmp = new Color3();
  private readonly glow = new Color3();

  constructor(scene: Scene, radius: number, cloudsUrl?: string) {
    this.dome = MeshBuilder.CreateSphere('sky', { diameter: radius * 2, segments: 32, sideOrientation: Mesh.BACKSIDE }, scene);
    const positions = this.dome.getVerticesData(VertexBuffer.PositionKind)!;
    this.dirs = new Float32Array(positions.length);
    for (let i = 0; i < positions.length; i++) this.dirs[i] = positions[i] / radius;
    this.colors = new Float32Array((positions.length / 3) * 4);
    this.dome.setVerticesData(VertexBuffer.ColorKind, this.colors, true);
    const mat = new StandardMaterial('sky', scene);
    mat.disableLighting = true;
    mat.emissiveColor = Color3.White();
    mat.fogEnabled = false;
    mat.backFaceCulling = false;
    this.dome.material = mat;
    this.dome.infiniteDistance = true;
    this.dome.isPickable = false;
    this.dome.applyFog = false;

    if (cloudsUrl) {
      // A slightly tapered open cylinder spanning ~3..32 degrees above the horizon.
      const r = radius * 0.94;
      const band = (this.clouds = MeshBuilder.CreateCylinder(
        'clouds',
        { diameterBottom: r * 2, diameterTop: r * 1.7, height: r * 0.55, tessellation: 48, cap: Mesh.NO_CAP, sideOrientation: Mesh.DOUBLESIDE },
        scene,
      ));
      band.position.y = r * 0.31;
      band.infiniteDistance = true;
      band.isPickable = false;
      band.applyFog = false;
      const cm = (this.cloudMaterial = new StandardMaterial('clouds', scene));
      const tex = new Texture(cloudsUrl, scene, { invertY: true });
      tex.hasAlpha = true;
      tex.uScale = 2;
      tex.wrapU = Texture.WRAP_ADDRESSMODE;
      tex.wrapV = Texture.CLAMP_ADDRESSMODE;
      cm.diffuseTexture = tex;
      cm.useAlphaFromDiffuseTexture = true;
      cm.disableLighting = true;
      cm.diffuseColor = Color3.Black();
      cm.specularColor = Color3.Black();
      cm.fogEnabled = false;
      cm.backFaceCulling = false;
      cm.disableDepthWrite = true;
      band.material = cm;
    }
  }

  get mesh(): Mesh {
    return this.dome;
  }

  /** The cloud band (when the sky has clouds): the mirror pass draws it too. */
  get band(): Mesh | null {
    return this.clouds;
  }

  /** Camera layers the sky is drawn on (see layers.ts). */
  setLayerMask(mask: number): void {
    this.dome.layerMask = mask;
    if (this.clouds) this.clouds.layerMask = mask;
  }

  /** Slow cloud drift; call per frame (moves a mesh, no re-recording needed). */
  drift(dt: number): void {
    if (this.clouds) this.clouds.rotation.y += dt * 0.000004;
  }

  /** Horizon = fog/clear colour, zenith = deeper sky colour, plus a glow around the sun. */
  update(l: Lighting<Color3, Vector3>): void {
    this.zenith.copyFrom(l.zenithColor);
    const sx = -l.sunDirection.x;
    const sy = -l.sunDirection.y;
    const sz = -l.sunDirection.z;
    // Strongest at golden hour, faint at noon, a dim halo around the moon at night.
    const strength = Math.min(1, l.sunIntensity / 2.5) * (0.35 + 0.65 * (1 - l.sunUp));
    const n = this.dirs.length / 3;
    for (let i = 0; i < n; i++) {
      const dx = this.dirs[i * 3];
      const dy = this.dirs[i * 3 + 1];
      const dz = this.dirs[i * 3 + 2];
      const t = smoothstep(0.02, 0.75, dy);
      Color3.LerpToRef(l.clearColor, this.zenith, t, this.tmp);
      const d = Math.max(0, dx * sx + dy * sy + dz * sz);
      const g = (Math.pow(d, 6) * 0.55 + Math.pow(d, 48) * 0.6) * strength * (dy > -0.05 ? 1 : 0);
      this.glow.copyFrom(l.sunColor).scaleInPlace(g);
      this.tmp.addInPlace(this.glow);
      // Rendering is linear (sRGB conversion happens in image processing).
      this.tmp.toLinearSpaceToRef(this.tmp);
      this.colors[i * 4] = this.tmp.r;
      this.colors[i * 4 + 1] = this.tmp.g;
      this.colors[i * 4 + 2] = this.tmp.b;
      this.colors[i * 4 + 3] = 1;
    }
    this.dome.updateVerticesData(VertexBuffer.ColorKind, this.colors);
    if (this.cloudMaterial) {
      // Lit side of the clouds: sky-bright by day, sunset-tinted at golden hour, dim at night.
      Color3.LerpToRef(l.clearColor, l.sunColor, 0.35 + 0.25 * (1 - l.sunUp), this.tmp);
      this.tmp.scaleInPlace(Math.min(1.15, 0.25 + l.sunIntensity * 0.28));
      this.tmp.toLinearSpaceToRef(this.cloudMaterial.emissiveColor);
    }
  }
}
