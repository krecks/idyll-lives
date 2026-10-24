/**
 * The landscape round the town on PlayCanvas: terrain with rolling hills, the sea or a lake, the
 * country roads (drawn by whoever is handed them, `buildRoads`), forests, lawn details, and the sky
 * dome with its band of clouds. Everything is static and instanced (`instancing.ts`: a draw call
 * per model type, and in the game per area: the groups of `shared/nature.ts`), generated
 * deterministically from the town's seed by the engine-neutral layout (`LandscapeLayout`), so a
 * town always grows the same plants in the same places.
 *
 * Trees, bushes, hedges, lawn tufts and the garden catalog's plants are Blender-built files
 * (tools/art/blender/families/plants.py, trees.py): crowns on the leafy `foliage` finish over `bark`
 * limbs, each tree and shrub with a far version for the distant woods. `installNature` gives those
 * two finishes the wind (`NATURE_WIND`, the port of `NatureWindPlugin`) and the foliage its
 * per-instance tint.
 *
 * Draws are entities under the world root (game space), made switched off: their owner shows them
 * (`lotContents.ts`'s landscapes, shared between the game and the overview), digs them away over a
 * basement and gives the woods near the town their shadows.
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { GAME_FRONT_FACE, MaterialLibrary, textureWithFallback } from './materials';
import { buildModel, disposeTemplate, type ModelMesh, type ModelTemplate } from './models';
import { registerShaderFeature, type ShaderFeature } from './shaders/chunks';
import type { AssetRegistry } from '../../assets/registry';
import type { WorldStructure } from '../../core/protocol';
import { CountryRoads } from '../shared/countryRoads';
import type { Lighting } from '../shared/environment';
import { toLinear } from '../shared/math';
import { computeNormals, sphereData } from '../shared/modelParts';
import { groupInstances, hideLawn, LandscapeLayout, lawnLayer, lawnMask, smoothstep, SPLIT_TRIANGLES, WIND, WIND_KIND, windSwayAt, type LawnLayer, type LawnMask } from '../shared/nature';
import { perfEnd, perfStart, perfTime, perfTimeAsync } from '../../debug/perf';

export { decorPlants, natureDecor, type PlantPlacement } from '../shared/nature';

/** What the landscape draws: entities under the world root, switched off until their owner shows them. */
export interface NatureResult {
  draws: pc.Entity[];
  /**
   * Draws that should cast shadows: the woods near the town, whole in the game too (its shadow map
   * round the viewed lot would draw each group reaching into it, a draw each).
   */
  casters: pc.Entity[];
  /** Plants drawn only in the game (in groups) or only in the menus' town (whole); the rest are drawn in both. */
  only: Map<pc.Entity, 'game' | 'town'>;
}

/**
 * A landscape's plants built again (`Landscape.rebuild`): the new draws, the ones they replace, the
 * swap (`commit`: the replaced ones and their templates go), or the new ones dropped (`discard`).
 */
export interface NatureRebuild extends NatureResult {
  replaced: pc.Entity[];
  commit: () => void;
  discard: () => void;
}

/** Draws the country roads (`pc/roads.ts`): their entities, under `parent`. */
export type RoadBuilder = (roads: CountryRoads, parent: pc.Entity) => pc.Entity[];

// ---- Landscape ----------------------------------------------------------------------

/** The landscape as PlayCanvas draws it (its layout and growth: `shared/nature.ts`). */
export class Landscape extends LandscapeLayout<CountryRoads> {
  /** Lawn tufts and flowers, per model, whole and in groups. */
  private lawn: LawnLayer<Instances>[] = [];
  /** What `refreshLawn` last hid, for the lawn built again since (`rebuild`). */
  private lawnMask: LawnMask | null = null;
  /** Each model's plants as built, to build again when a file they were built without comes (`rebuild`). */
  private batches: Batch[] = [];
  /** Its other draws (the terrain, the water, the roads). */
  private others: pc.Entity[] = [];

  constructor(world: WorldStructure, lawnDensity = 1, lawnLod: 0 | 1 = 0) {
    super(world, (land, w, name) => new CountryRoads(land, w, name), lawnDensity, lawnLod);
  }

  async build(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>, buildRoads?: RoadBuilder): Promise<NatureResult> {
    const draws: pc.Entity[] = [];
    draws.push(await perfTimeAsync('nature: terrain', () => this.terrain(app, parent, pace)));
    if (this.watered) draws.push(perfTime('nature: water', () => this.water(app, parent)));
    if (buildRoads) draws.push(...perfTime('nature: roads', () => buildRoads(this.roads, parent)));
    for (const e of draws) e.enabled = false;
    this.others = draws;
    this.batches = [];
    await perfTimeAsync('nature: forests', () => this.forests(app, parent, assets, pace));
    await perfTimeAsync('nature: lawns', () => this.plantLawns(app, parent, assets, world, pace));
    const plants = gather(this.batches);
    return { draws: [...draws, ...plants.draws], casters: plants.casters, only: plants.only };
  }

  /**
   * Builds again the plants built without the file at `url`, now that it has come: the new draws
   * (switched off, like a first build's), the ones they replace, and `commit`, which puts the new
   * in the old ones' place here (the lawn's hiding goes over to them) and takes the old ones down;
   * the caller shows the new draws as it commits, or `discard`s them. Null when none was built
   * without it. Not while `build` or another `rebuild` runs.
   */
  async rebuild(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, url: string, pace?: () => Promise<void>): Promise<NatureRebuild | null> {
    const stale = this.batches.filter((b) => b.again && b.built.failed?.includes(url));
    if (!stale.length) return null;
    const fresh: Batch[] = [];
    for (const { again, casts } of stale) {
      const { key, matrices, lod, cell } = again!;
      fresh.push(batch(again!, casts, await instanced(app, parent, assets, key, matrices, lod, cell, pace)));
    }
    const replaced = gather(stale).draws;
    const commit = () => {
      for (const b of stale) release(b.built);
      this.batches = this.batches.map((b) => fresh[stale.indexOf(b)] ?? b);
      const going = new Set(replaced);
      this.lawn = this.lawn.filter((layer) => !layer.draws.some((d) => going.has(d.entity)));
      stale.forEach(({ again }, i) => {
        if (!again!.tiles) return;
        const layers = lawnLayers(fresh[i].built, again!.matrices, again!.tiles);
        this.lawn.push(...layers);
        if (this.lawnMask) hideLawn(layers, this.lawnMask, uploadLawn);
      });
    };
    return { ...gather(fresh), replaced, commit, discard: () => fresh.forEach((b) => release(b.built)) };
  }

  /** Takes down everything it draws (the plants' templates with their draws). */
  dispose(): void {
    for (const b of this.batches) release(b.built);
    for (const e of this.others) e.destroy();
    this.batches = [];
    this.others = [];
    this.lawn = [];
  }

  /**
   * Hides the lawn on tiles that are now indoors or under furniture, and shows it again elsewhere.
   * On a tile split by a diagonal wall, each tuft follows the half it stands in.
   */
  refreshLawn(world: WorldStructure): void {
    this.lawnMask = lawnMask(world);
    hideLawn(this.lawn, this.lawnMask, uploadLawn);
  }

  /** The ground: a regular grid over the landscape's square, lifted to the land and coloured. */
  private async terrain(app: pc.AppBase, parent: pc.Entity, pace?: () => Promise<void>): Promise<pc.Entity> {
    const { size, segments } = this.terrainGrid();
    const n = segments + 1;
    const positions = new Float32Array(n * n * 3);
    const uvs = new Float32Array(n * n * 2);
    // (Rows from the far edge, moved to the town's middle.)
    for (let row = 0, v = 0; row <= segments; row++) {
      for (let col = 0; col <= segments; col++, v++) {
        positions[v * 3] = (col * size) / segments - size / 2 + this.w / 2;
        positions[v * 3 + 2] = ((segments - row) * size) / segments - size / 2 + this.d / 2;
        uvs[v * 2] = col / segments;
        uvs[v * 2 + 1] = 1 - row / segments;
      }
    }
    const indices = new Uint32Array(segments * segments * 6);
    for (let row = 0, k = 0; row < segments; row++) {
      for (let col = 0; col < segments; col++) {
        indices[k++] = col + 1 + (row + 1) * n;
        indices[k++] = col + 1 + row * n;
        indices[k++] = col + row * n;
        indices[k++] = col + (row + 1) * n;
        indices[k++] = col + 1 + (row + 1) * n;
        indices[k++] = col + row * n;
      }
    }
    const colors = new Float32Array(n * n * 4);
    // sRGB vertex colours (converted on the GPU) multiply the grass detail texture.
    await this.shadeTerrain(positions, colors, pace);
    const normals = new Float32Array(positions.length);
    computeNormals(positions, indices, normals);
    const mesh = new pc.Mesh(app.graphicsDevice);
    mesh.setPositions(positions);
    mesh.setNormals(normals);
    mesh.setUvs(0, uvs);
    mesh.setColors(colors);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    return drawOf(app, parent, 'terrain', mesh, MaterialLibrary.for(app).surface('material.ground', { vertexColors: true }));
  }

  /**
   * The sea or the lake: one flat sheet at the water line under the whole landscape (the land
   * covers it everywhere but in the water), reflecting the sky.
   */
  private water(app: pc.AppBase, parent: pc.Entity): pc.Entity {
    const at = this.waterSheet();
    const h = at.size / 2;
    const mesh = new pc.Mesh(app.graphicsDevice);
    // (Wound as the ground: its front up.)
    mesh.setPositions([at.x - h, at.y, at.z + h, at.x + h, at.y, at.z + h, at.x - h, at.y, at.z - h, at.x + h, at.y, at.z - h]);
    mesh.setNormals([0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0]);
    mesh.setUvs(0, [0, 1, 1, 1, 0, 0, 1, 0]);
    mesh.setIndices([3, 1, 0, 2, 3, 0]);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const mat = new pc.StandardMaterial();
    mat.name = 'water';
    mat.frontFace = GAME_FRONT_FACE;
    mat.useMetalness = true;
    mat.diffuse.fromString('#2F6F86');
    mat.metalness = 0;
    mat.glossInvert = true;
    mat.gloss = 0.12;
    mat.opacity = 0.9;
    mat.blendType = pc.BLEND_NORMAL;
    MaterialLibrary.for(app).adopt(mat);
    mat.update();
    return drawOf(app, parent, 'water', mesh, mat);
  }

  /** The woods (`woods`), each model instanced, in groups by sector round the town. */
  private async forests(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, pace?: () => Promise<void>): Promise<void> {
    const { near, far, sector } = await this.woods(pace);
    for (const [bucket, casts] of [
      [near, true],
      [far, false],
    ] as const) {
      for (const [key, list] of bucket) {
        const lod = casts ? 0 : 1;
        this.batches.push(batch({ key, matrices: list, lod, cell: sector }, casts, await instanced(app, parent, assets, key, list, lod, sector, pace)));
      }
    }
  }

  /** The lawn (`lawns`), each model instanced, the tufts in squares. */
  private async plantLawns(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>): Promise<void> {
    this.lawn = [];
    for (const { key, matrices, tiles, cell } of await this.lawns(world, pace)) {
      const lod = this.lawnLod;
      const built = await instanced(app, parent, assets, key, matrices, lod, cell, pace);
      this.batches.push(batch({ key, matrices, lod, cell, tiles }, false, built));
      this.lawn.push(...lawnLayers(built, matrices, tiles));
    }
    this.refreshLawn(world);
  }
}

/** An entity of `app` drawing `mesh` with `material`, switched off, under `parent`; it receives shadows and casts none. */
function drawOf(app: pc.AppBase, parent: pc.Entity, name: string, mesh: pc.Mesh, material: pc.Material): pc.Entity {
  const entity = new pc.Entity(name, app);
  entity.addComponent('render', { meshInstances: [new pc.MeshInstance(mesh, material)], castShadows: false, receiveShadows: true });
  entity.enabled = false;
  parent.addChild(entity);
  return entity;
}

/** A lawn layer's matrices as drawn now, up to its draws. */
function uploadLawn(layer: LawnLayer<Instances>): void {
  for (const draw of layer.draws) {
    draw.setMatrices(layer.shown);
    draw.update();
  }
}

/**
 * A model's meshes drawn instanced: the parts drawn whole in the game and the menus' town
 * (`shared`), and those also drawn in groups (see `SPLIT_TRIANGLES`): whole in the town (`whole`),
 * in groups in the game (`cells`; each keeps its instances in their order, `indices` into `matrices`).
 */
interface Instanced {
  shared: Instances[];
  whole: Instances[];
  cells: { draws: Instances[]; indices: number[] }[];
  /** The model's files that failed to load (`ModelTemplate.failed`): it's built again when one comes (`Landscape.rebuild`). */
  failed?: string[];
  /** The template they draw (theirs alone: it goes with them, `release`). */
  template?: ModelTemplate;
}

/** Takes down a model's plants: their draws, then the template's meshes. */
function release(built: Instanced): void {
  for (const d of [...built.shared, ...built.whole, ...built.cells.flatMap((c) => c.draws)]) d.destroy();
  if (built.template) disposeTemplate(built.template);
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
  cell?: (x: number, z: number) => string;
  /** The lawn's: each instance's town tile (-1 outside the town), for `refreshLawn`. */
  tiles?: readonly number[];
}

/** A batch of plants built from `source`, which it keeps only if it was built without a file (all of them are several MB a landscape). */
function batch(source: BatchSource, casts: boolean, built: Instanced): Batch {
  return { built, casts, ...(built.failed ? { again: source } : {}) };
}

/** The draws of `batches`: which cast shadows, and which are drawn only in the game or only in the menus' town (see `NatureResult`). */
function gather(batches: readonly Batch[]): NatureResult {
  const draws: pc.Entity[] = [];
  const casters: pc.Entity[] = [];
  const only = new Map<pc.Entity, 'game' | 'town'>();
  for (const { built, casts } of batches) {
    const groups = built.cells.flatMap((cell) => cell.draws.map((d) => d.entity));
    const shared = built.shared.map((d) => d.entity);
    const whole = built.whole.map((d) => d.entity);
    draws.push(...shared, ...whole, ...groups);
    for (const e of whole) only.set(e, 'town');
    for (const e of groups) only.set(e, 'game');
    if (casts) casters.push(...shared, ...whole);
  }
  return { draws, casters, only };
}

/** The lawn layers of a model's tufts or flowers: every instance, in the parts drawn whole; then each group's. */
function lawnLayers(built: Instanced, matrices: readonly Float32Array[], tiles: readonly number[]): LawnLayer<Instances>[] {
  const layers: LawnLayer<Instances>[] = [];
  for (const { draws, indices } of [{ draws: [...built.shared, ...built.whole], indices: matrices.map((_, k) => k) }, ...built.cells]) {
    if (!draws.length) continue;
    layers.push(lawnLayer(draws, matrices, tiles, indices));
  }
  return layers;
}

/**
 * One template per model key, drawn instanced; static, so the boxes are worked out once. `cell`
 * groups the instances by where they stand: the parts with enough triangles get a draw per group
 * (sharing the template's mesh and material), which the game's camera leaves out when out of view.
 */
async function instanced(
  app: pc.AppBase,
  parent: pc.Entity,
  assets: AssetRegistry,
  key: string,
  matrices: Float32Array[],
  lod: 0 | 1 = 0,
  cell?: (x: number, z: number) => string,
  pace?: () => Promise<void>,
): Promise<Instanced> {
  if (matrices.length === 0) return { shared: [], whole: [], cells: [] };
  await pace?.();
  // Without its file, nothing, until the file comes (`Landscape.rebuild`).
  const template = await perfTimeAsync(`nature: model ${key}`, () => buildModel(app, key, assets.get(key, 'model'), [1, 1], { lod }));
  await pace?.();
  const groups = groupInstances(matrices, cell);
  const split = groups.size > 1 ? template.meshes.filter((m) => (indexCount(m) / 3) * matrices.length >= SPLIT_TRIANGLES) : [];
  const span = perfStart(`nature: instances ${key}`);
  const draw = (m: ModelMesh, indices: readonly number[], name: string): Instances => {
    const d = new Instances(app.graphicsDevice, parent, m.mesh, m.material, { name, castShadows: false, receiveShadows: true, capacity: indices.length });
    d.setCount(indices.length);
    for (let i = 0; i < indices.length; i++) d.setMatrix(i, matrices[indices[i]]);
    d.update();
    windStill(d.meshInstance, m);
    d.enabled = false;
    return d;
  };
  const all = matrices.map((_, k) => k);
  const whole = template.meshes.map((m) => draw(m, all, m.name));
  await pace?.();
  const cells: Instanced['cells'] = [];
  if (split.length) {
    let c = 0;
    for (const indices of groups.values()) {
      cells.push({ draws: split.map((m) => draw(m, indices, `${m.name}#${c}`)), indices });
      c++;
      await pace?.();
    }
  }
  perfEnd(span);
  return {
    shared: whole.filter((_, i) => !split.includes(template.meshes[i])),
    whole: whole.filter((_, i) => split.includes(template.meshes[i])),
    cells,
    template,
    ...(template.failed ? { failed: template.failed } : {}),
  };
}

/** A mesh's triangle corners (its indices drawn). */
function indexCount(m: ModelMesh): number {
  return m.mesh.primitive[0]?.count ?? 0;
}

// ---- Wind and foliage shading -----------------------------------------------------------

/** A number as a shader's float literal. */
const f = (n: number) => (Number.isInteger(n) ? n.toFixed(1) : String(n));

/**
 * The wind (the port of `NatureWindPlugin`), on parts on the `bark` and `foliage` finishes: a slow
 * height-weighted bend with gusts plus leaf flutter, phased per instance by where it stands (its
 * matrix's translation; an instance's scale and turn are taken out of the flutter and the bend's
 * direction), and, on foliage, a small per-instance colour
 * variation so repeated trees don't look stamped. Short plants hardly move. The vertex's local
 * position (`vsLocal`), so the shadow pass could take it too; it doesn't: trees cast still shadows
 * (the sun's map is drawn again only when something in it changes, `sunShadows.ts`).
 *
 * - `GAME_WIND`: `game_wind` (global: time, strength, direction x, z), `game_windKind` (the
 *   material's: `WIND_KIND`), `game_windFlutter` (1, and 0 on a draw of a part grown from nothing,
 *   `windStill`: flat at rest, it stays hidden).
 * - `GAME_WIND_TINT`: the foliage's tint, on its vertex colour (an sRGB factor, so here its 2.2 power on the linear colour).
 *
 * No instance stream of its own: it reads only the instance's matrix (`instance_line1..4`).
 */
function windCode(lang: 'glsl' | 'wgsl'): ShaderFeature['glsl'] {
  const wgsl = lang === 'wgsl';
  const v3 = wgsl ? 'vec3f' : 'vec3';
  const v2 = wgsl ? 'vec2f' : 'vec2';
  const u = (n: string) => (wgsl ? `uniform.${n}` : n);
  const decl = (name: string, type: 'float' | 'vec3' | 'vec4', mutable = false) => (wgsl ? `${mutable ? 'var' : 'let'} ${name}` : `${type} ${name}`);
  const { gust: g, phase, sway } = WIND;
  const local = `
#if defined(GAME_WIND) && !defined(SHADOW_PASS)
	{
		${decl('windInst', 'vec3', true)} = ${v3}(0.0);
		${decl('windLw', 'vec3', true)} = ${v3}(${u('game_wind')}.z, 0.0, ${u('game_wind')}.w);
		// The instance's scale along each axis: flutter is in world metres, however far a moving part is scaled
		// (a part grown from nothing is 1000x along its height once grown).
		${decl('windScl', 'vec3', true)} = ${v3}(1.0);
		#ifdef INSTANCING
			windInst = instance_line4.xyz;
			${decl('windS2', 'float')} = max(dot(instance_line1.xyz, instance_line1.xyz), 0.0001);
			windLw = ${v3}(dot(windLw, instance_line1.xyz), 0.0, dot(windLw, instance_line3.xyz)) / windS2;
			windScl = max(${v3}(length(instance_line1.xyz), length(instance_line2.xyz), length(instance_line3.xyz)), ${v3}(0.0001));
		#endif
		${decl('windT', 'float')} = ${u('game_wind')}.x;
		${decl('windPh', 'float')} = dot(windInst.xz, ${v2}(${f(phase[0])}, ${f(phase[1])}));
		${decl('windH', 'float')} = max(localPos.y, 0.0);
		${decl('windGust', 'float')} = (${f(g.base)} + ${f(g.amp)} * sin(windT * ${f(g.rate)} + windPh * ${f(g.phase)})) * ${u('game_wind')}.y;
		${decl('windSway', 'float')} = (${sway.map((s) => `sin(windT * ${f(s.rate)} + windPh * ${f(s.phase)}) * ${f(s.weight)}`).join(' + ')}) * windGust;
		${decl('windKind', 'vec4')} = ${u('game_windKind')};
		${decl('windFp', 'float')} = dot(localPos, ${v3}(3.1, 1.7, 2.3)) + windPh * 3.0;
		${decl('windFl', 'float')} = windKind.y * ${u('game_windFlutter')} * min(windH * windKind.w, 1.0) * windGust;
		localPos = localPos + windLw * (windSway * windKind.x * windH * windH) + ${v3}(sin(windT * windKind.z + windFp), 0.6 * sin(windT * windKind.z * 1.3 + windFp * 1.3), cos(windT * windKind.z * 0.9 + windFp)) * windFl / windScl;
	}
#endif`;
  const color = wgsl ? 'output.vVertexColor' : 'vVertexColor';
  const end = `
#if defined(GAME_WIND_TINT) && defined(VERTEX_COLOR) && !defined(SHADOW_PASS)
	{
		${decl('tintInst', 'vec3', true)} = ${v3}(0.0);
		#ifdef INSTANCING
			tintInst = instance_line4.xyz;
		#endif
		${decl('tintRnd', 'float')} = fract(sin(dot(tintInst.xz, ${v2}(12.9898, 78.233))) * 43758.5453);
		${decl('tint', 'vec3')} = mix(${v3}(1.07, 1.02, 0.84), ${v3}(0.9, 0.98, 1.04), tintRnd) * (0.92 + 0.14 * fract(tintRnd * 7.31));
		${color} = ${wgsl ? 'vec4f' : 'vec4'}(${color}.rgb * pow(tint, ${v3}(2.2)), ${color}.a);
	}
#endif`;
  const vsDecl = wgsl
    ? `
#ifdef GAME_WIND
	uniform game_wind: vec4f;
	uniform game_windKind: vec4f;
	uniform game_windFlutter: f32;
#endif`
    : `
#ifdef GAME_WIND
	uniform vec4 game_wind;
	uniform vec4 game_windKind;
	uniform float game_windFlutter;
#endif`;
  return { vsDecl, vsLocal: local, vsEnd: end };
}

/** The plants' wind and tint (see `windCode`), at its place among the shader features (`chunks.ts`: lower runs first). */
export const NATURE_WIND: ShaderFeature = { name: 'natureWind', priority: 230, glsl: windCode('glsl'), wgsl: windCode('wgsl') };

// (Before any device's chunks are built: they're built with every feature registered, once per device.)
registerShaderFeature(NATURE_WIND);

/** The wind of an app's plants: its clock (seconds, advancing with the frames) and strength (1 as made; 0 still, for reduce motion). */
interface WindState {
  time: number;
  strength: number;
}

const windStates = new WeakMap<pc.AppBase, WindState>();

/**
 * Hooks the plant finishes as the library makes them (whoever builds the first plant): the wind,
 * and the tint on foliage. Also advances the wind clock with the app's frames and hands it to the
 * shaders. Call once per app, before any model loads.
 */
export function installNature(app: pc.AppBase): void {
  if (windStates.has(app)) return;
  const state: WindState = { time: 0, strength: 1 };
  windStates.set(app, state);
  MaterialLibrary.for(app).onCreate((key, material) => {
    const kind = WIND_KIND[key];
    if (!kind) return;
    material.setDefine('GAME_WIND', '');
    if (key !== 'material.finish.bark') material.setDefine('GAME_WIND_TINT', '');
    material.setParameter('game_windKind', new Float32Array(kind));
    material.setParameter('game_windFlutter', 1);
  });
  const id = app.graphicsDevice.scope.resolve('game_wind');
  const value = new Float32Array([0, 1, WIND.dir[0], WIND.dir[1]]);
  app.on('update', (dt: number) => {
    state.time += Math.min(0.1, dt);
  });
  app.on('prerender', () => {
    value[0] = state.time;
    value[1] = state.strength;
    id.setValue(value);
  });
}

/** A draw of model mesh `m` (on a plant finish) bends without fluttering when it's a part grown from nothing (`ModelMesh.windStill`). */
export function windStill(meshInstance: pc.MeshInstance, m: ModelMesh): void {
  if (m.windStill) meshInstance.setParameter('game_windFlutter', 0);
}

/** How strongly the wind moves an app's plants (1 as made; 0 still, for reduce motion and `?still`). */
export function setWind(app: pc.AppBase, strength: number): void {
  const state = windStates.get(app);
  if (state) state.strength = strength;
}

/**
 * How far the wind bends a plant standing at (`px`, `pz`) now, per m² of height up its model (m,
 * along `windDirection`): what sits on a branch sways with it. Zero before the app's wind exists.
 */
export function windSway(app: pc.AppBase, px: number, pz: number): number {
  const state = windStates.get(app);
  return state ? windSwayAt(state.time, state.strength, px, pz) : 0;
}

// ---- Sky ------------------------------------------------------------------------------

/** The dome's sphere: 32 segments (`sphereData`). */
const SKY_SEGMENTS = 32;

/**
 * Gradient sky dome with a glow around the sun (colours follow the time of day via vertex
 * colours) and a band of drifting clouds (`image.clouds`). Both follow the camera (`follow`), so
 * they are always as far away, unlit and out of the fog.
 *
 * Colours as the look was tuned with: the sky's colour is taken for gamma and linearised before the
 * image processing, so the dome's linear vertex colours are linearised once more here, and the
 * clouds' emissive colour is handed over as an sRGB one (PlayCanvas linearises it).
 */
export class Sky {
  readonly dome: pc.Entity;
  readonly band: pc.Entity | null = null;
  private readonly mesh: pc.Mesh;
  private readonly dirs: Float32Array;
  private readonly colors: Float32Array;
  private readonly cloudMaterial: pc.StandardMaterial | null = null;
  private readonly tmp = { r: 0, g: 0, b: 0 };
  private readonly glow = { r: 0, g: 0, b: 0 };
  private spin = 0;

  constructor(app: pc.AppBase, parent: pc.Entity, radius: number, cloudsUrl?: string) {
    const device = app.graphicsDevice;
    const { positions, indices } = sphereData(radius * 2, radius * 2, radius * 2, SKY_SEGMENTS);
    this.dirs = new Float32Array(positions.length);
    for (let i = 0; i < positions.length; i++) this.dirs[i] = positions[i] / radius;
    this.colors = new Float32Array((positions.length / 3) * 4);
    const mesh = (this.mesh = new pc.Mesh(device));
    mesh.setPositions(positions);
    mesh.setColors(this.colors);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const mat = new pc.StandardMaterial();
    mat.name = 'sky';
    mat.useLighting = false;
    mat.useSkybox = false;
    mat.useFog = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.set(1, 1, 1);
    mat.emissiveVertexColor = true;
    mat.cull = pc.CULLFACE_NONE;
    mat.update();
    this.dome = new pc.Entity('sky', app);
    this.dome.addComponent('render', { meshInstances: [new pc.MeshInstance(mesh, mat)], castShadows: false, receiveShadows: false });
    // (It moves with the camera, which is always inside it.)
    this.dome.render!.meshInstances[0].cull = false;
    parent.addChild(this.dome);

    if (cloudsUrl) {
      // A slightly tapered open cylinder spanning ~3..32 degrees above the horizon: 48 sides, its
      // faces doubled (each drawn twice, uncull'd, so the clouds blend in twice).
      const r = radius * 0.94;
      const [bottom, top, height, sides] = [r, r * 0.85, r * 0.55, 48];
      const p: number[] = [];
      const uv: number[] = [];
      for (let i = 0; i <= 1; i++) {
        const rr = i ? top : bottom;
        for (let j = 0; j <= sides; j++) {
          const a = (j * Math.PI * 2) / sides;
          p.push(Math.cos(-a) * rr, -height / 2 + i * height + r * 0.31, Math.sin(-a) * rr);
          // (`uScale` 2 baked in; the band's v runs up it, PlayCanvas's from the image's top.)
          uv.push((j / sides) * 2, 1 - i);
        }
      }
      const idx: number[] = [];
      for (let j = 0; j < sides; j++) {
        const [a, b, c, d] = [j, j + 1, j + sides + 1, j + sides + 2];
        idx.push(c, b, a, c, d, b);
      }
      const n = p.length / 3;
      const band = new pc.Mesh(device);
      band.setPositions([...p, ...p]);
      band.setUvs(0, [...uv, ...uv]);
      band.setIndices([...idx, ...idx.map((k) => k + n)]);
      band.update(pc.PRIMITIVE_TRIANGLES);
      const cm = (this.cloudMaterial = new pc.StandardMaterial());
      cm.name = 'clouds';
      cm.useLighting = false;
      cm.useSkybox = false;
      cm.useFog = false;
      cm.diffuse.set(0, 0, 0);
      cm.emissive.set(0, 0, 0);
      cm.blendType = pc.BLEND_NORMAL;
      cm.depthWrite = false;
      cm.cull = pc.CULLFACE_NONE;
      cm.opacity = 0;
      cm.update();
      void textureWithFallback(app, cloudsUrl).then((tex) => {
        if (!tex) return;
        tex.addressU = pc.ADDRESS_REPEAT;
        tex.addressV = pc.ADDRESS_CLAMP_TO_EDGE;
        cm.emissiveMap = tex;
        cm.opacityMap = tex;
        cm.opacityMapChannel = 'a';
        cm.opacity = 1;
        cm.update();
      });
      const entity = (this.band = new pc.Entity('clouds', app));
      entity.addComponent('render', { meshInstances: [new pc.MeshInstance(band, cm)], castShadows: false, receiveShadows: false });
      entity.render!.meshInstances[0].cull = false;
      this.dome.addChild(entity);
    }
  }

  /** Puts the sky round the camera (game space: its eye), so it never comes nearer. */
  follow(x: number, y: number, z: number): void {
    this.dome.setLocalPosition(x, y, z);
  }

  /** Slow cloud drift; call per frame with the frame's milliseconds. */
  drift(dt: number): void {
    if (!this.band) return;
    this.spin += dt * 0.000004;
    // (A turn about y in game space: degrees, the world root's mirror taken by the parent.)
    this.band.setLocalEulerAngles(0, (this.spin * 180) / Math.PI, 0);
  }

  /** Horizon = fog/clear colour, zenith = deeper sky colour, plus a glow around the sun. */
  update(l: Lighting): void {
    const sx = -l.sunDirection.x;
    const sy = -l.sunDirection.y;
    const sz = -l.sunDirection.z;
    // Strongest at golden hour, faint at noon, a dim halo around the moon at night.
    const strength = Math.min(1, l.sunIntensity / 2.5) * (0.35 + 0.65 * (1 - l.sunUp));
    const n = this.dirs.length / 3;
    const tmp = this.tmp;
    const glow = this.glow;
    const z = l.zenithColor;
    const c = l.clearColor;
    for (let i = 0; i < n; i++) {
      const dx = this.dirs[i * 3];
      const dy = this.dirs[i * 3 + 1];
      const dz = this.dirs[i * 3 + 2];
      const t = smoothstep(0.02, 0.75, dy);
      tmp.r = c.r + (z.r - c.r) * t;
      tmp.g = c.g + (z.g - c.g) * t;
      tmp.b = c.b + (z.b - c.b) * t;
      const d = Math.max(0, dx * sx + dy * sy + dz * sz);
      const g = (Math.pow(d, 6) * 0.55 + Math.pow(d, 48) * 0.6) * strength * (dy > -0.05 ? 1 : 0);
      glow.r = l.sunColor.r * g;
      glow.g = l.sunColor.g * g;
      glow.b = l.sunColor.b * g;
      // The linear colour, linearised again (see the class).
      this.colors[i * 4] = toLinear(toLinear(tmp.r + glow.r));
      this.colors[i * 4 + 1] = toLinear(toLinear(tmp.g + glow.g));
      this.colors[i * 4 + 2] = toLinear(toLinear(tmp.b + glow.b));
      this.colors[i * 4 + 3] = 1;
    }
    this.mesh.setColors(this.colors);
    this.mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    const cm = this.cloudMaterial;
    if (cm) {
      // Lit side of the clouds: sky-bright by day, sunset-tinted at golden hour, dim at night.
      const k = 0.35 + 0.25 * (1 - l.sunUp);
      const s = Math.min(1.15, 0.25 + l.sunIntensity * 0.28);
      cm.emissive.set(toLinear((c.r + (l.sunColor.r - c.r) * k) * s), toLinear((c.g + (l.sunColor.g - c.g) * k) * s), toLinear((c.b + (l.sunColor.b - c.b) * k) * s));
      cm.update();
    }
  }

  destroy(): void {
    this.dome.destroy();
  }
}
