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
  Matrix,
  Mesh,
  MeshBuilder,
  PBRMaterial,
  Quaternion,
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
import { DEFAULT_LANDSCAPE, groundDepth, type LandscapeParams, type WorldStructure } from '../../core/protocol';
import type { Lighting } from './environment';
import { MaterialLibrary } from './materials';
import { buildModel } from './models';
import { CountryRoads, ROAD_VERGE } from './roads';
import { perfEnd, perfStart, perfTimeAsync } from '../../debug/perf';

/**
 * How far the landscape extends beyond the town (metres): towns of older code, and worlds (which
 * reach further, to a farther horizon). The valley's rim rises over `RING` either way.
 */
const RING = 150;
const WORLD_RING = 260;
/** Flat apron around the town before the hills start. */
const APRON = 6;
const TERRAIN_STEP = 2.5;
/** At most this many terrain quads across (a wider landscape gets coarser ones). */
const TERRAIN_SEGMENTS = 260;
/** Trees within this distance of the town cast shadows; further ones don't. */
const SHADOW_RANGE = 28;
/** Mountains: no trees above about this height (m). */
const TREELINE = 12;
/** Height of the sea or a lake (m); the shore is where the land meets it. */
const WATER_Y = -0.35;
/** Sand from the shoreline up the bank (m). */
const BEACH = 8;
/**
 * In the game, plants are drawn in groups the camera leaves out when they're out of view (a model
 * drawn as one mesh over the whole landscape never is): the woods in this many sectors round the
 * town, lawn tufts in squares of this size (m). A model part with fewer triangles in all than
 * `SPLIT_TRIANGLES` (bark, flowers) stays one mesh: the draw calls more would cost the main thread
 * more than the triangles they save. Each group is a mesh of its own, and every mesh costs the main
 * thread a little each frame whether drawn or not, so the squares are wide. The menus' town, which
 * has the whole landscape in view, draws each part whole (see `NatureResult.only`).
 */
const FOREST_SECTORS = 8;
const TUFT_CELL = 64;
const SPLIT_TRIANGLES = 36_000;

/** A tree, bush or hedge as placed (the garden's, the street's, the woods'): its model, where it stands, its scale (x, up, z) and turn. */
export interface PlantPlacement {
  key: string;
  x: number;
  y: number;
  z: number;
  sx: number;
  sy: number;
  sz: number;
  yaw: number;
}

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

// ---- Deterministic noise ------------------------------------------------------------

export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function hash(x: number, z: number, seed: number): number {
  let h = (Math.imul(x, 374761393) + Math.imul(z, 668265263) + Math.imul(seed, 2147483647)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

function valueNoise(x: number, z: number, seed: number): number {
  const xi = Math.floor(x);
  const zi = Math.floor(z);
  const xf = x - xi;
  const zf = z - zi;
  const u = xf * xf * (3 - 2 * xf);
  const v = zf * zf * (3 - 2 * zf);
  const a = hash(xi, zi, seed);
  const b = hash(xi + 1, zi, seed);
  const c = hash(xi, zi + 1, seed);
  const d = hash(xi + 1, zi + 1, seed);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}

/** Ridged noise, 0..1: sharp crests where plain noise crosses its middle (mountain ridges). */
function ridged(x: number, z: number, seed: number, octaves = 5): number {
  let sum = 0;
  let amp = 0.5;
  let freq = 1;
  let weight = 1;
  for (let i = 0; i < octaves; i++) {
    const r = Math.pow(1 - Math.abs(valueNoise(x * freq, z * freq, seed + i * 31) * 2 - 1), 2) * weight;
    sum += r * amp;
    // Detail mostly on the ridges, little in the valleys.
    weight = Math.min(1, r * 1.6);
    amp *= 0.5;
    freq *= 2.1;
  }
  return Math.min(1, sum / 0.9);
}

function fbm(x: number, z: number, seed: number, octaves = 4): number {
  let sum = 0;
  let amp = 0.5;
  let freq = 1;
  for (let i = 0; i < octaves; i++) {
    sum += amp * valueNoise(x * freq, z * freq, seed + i * 17);
    amp *= 0.5;
    freq *= 2;
  }
  return sum;
}

export const smoothstep = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

// ---- Landscape ----------------------------------------------------------------------

export class Landscape {
  private readonly seed: number;
  private readonly w: number;
  private readonly d: number;
  /** How the land looks (the world's kind rolled it; older towns: the classic valley). */
  private readonly land: LandscapeParams;
  /** How far the land reaches past the town (m). */
  private readonly ring: number;
  /** Where the road leaves town (z of the main street; the mountains part there). */
  private readonly passZ: number | null;
  /** The roads out of town (the ground is levelled under them). */
  private readonly roads: CountryRoads;
  /** Lawn tufts and flowers, per model, whole and in groups (see `TUFT_CELL`). */
  private lawn: LawnLayer[] = [];
  /** What `refreshLawn` last hid, for the lawn built again since (`rebuild`). */
  private lawnMask: LawnMask | null = null;
  /** Each model's plants as built, to build again when a file they were built without comes (`rebuild`). */
  private batches: Batch[] = [];
  /** The woods' trees and bushes near the town (full models), where birds may live (`woodsNear`). */
  private nearWoods: PlantPlacement[] = [];
  /** `woodsNear`'s last answer, and where it was asked (the same array while that's unchanged). */
  private woodsAsked: { at: string; plants: readonly PlantPlacement[] } | null = null;

  /**
   * `lawnDensity`: the share of the lawn's tufts and flowers drawn (Low: half); `lawnLod` 1 draws
   * them in their far versions (Low, as the street's models: `QualitySettings.streetLod`).
   */
  constructor(
    world: WorldStructure,
    private readonly lawnDensity = 1,
    private readonly lawnLod: 0 | 1 = 0,
  ) {
    this.seed = world.meta?.seed ?? 1;
    this.w = world.width;
    this.d = groundDepth(world);
    this.land = { ...DEFAULT_LANDSCAPE, ...world.meta?.world?.landscape };
    this.ring = world.meta?.world ? WORLD_RING : RING;
    const streets = (world.meta?.streets ?? []).filter((s) => s.w >= s.d).sort((a, b) => a.z - b.z);
    const main = streets[Math.floor((streets.length - 1) / 2)];
    this.passZ = main ? main.z + main.d / 2 : null;
    this.roads = new CountryRoads(
      { seed: this.seed, w: this.w, d: this.d, ring: this.ring, ground: (x, z) => this.ground(x, z), waterEdge: (x, z) => this.waterEdge(x, z), distanceToTown: (x, z) => this.distanceToTown(x, z) },
      world,
      world.meta?.name ?? '',
    );
  }

  /** The woods' trees and bushes within `range` m of a point (those near the town only), for the birds: the same array when asked the same again. */
  woodsNear(x: number, z: number, range: number): readonly PlantPlacement[] {
    const at = `${x},${z},${range}`;
    if (this.woodsAsked?.at !== at) this.woodsAsked = { at, plants: this.nearWoods.filter((p) => Math.hypot(p.x - x, p.z - z) < range) };
    return this.woodsAsked.plants;
  }

  /** Whether mountains rise round the town (the haze is thinner, so they're seen). */
  get mountainous(): boolean {
    return !!this.land.mountains;
  }

  /** Identity of a town's landscape (built once per town and kept while it is shown). */
  static key(world: WorldStructure): string {
    const look = world.meta?.world;
    const streets = (world.meta?.streets ?? []).map((s) => `${s.z}`).join(',');
    return `${world.meta?.seed ?? 1}:${world.width}x${groundDepth(world)}:${world.meta?.name ?? ''}:${streets}${look ? `:${look.kind}:${JSON.stringify(look.landscape)}` : ''}`;
  }

  /**
   * Metres past the shoreline of the sea or the lake (above 0 in the water, below 0 on land);
   * -Infinity without water. The shore wanders a little but never comes into the town.
   */
  waterEdge(x: number, z: number): number {
    const water = this.land.water;
    if (!water) return -Infinity;
    if (water.kind === 'sea') {
      const past = water.side === 'zmin' ? -z : z - this.d;
      return past - water.shore - fbm(x * 0.02, 3.7, this.seed + 61, 3) * 8;
    }
    const dx = (x - water.x) / water.rx;
    const dz = (z - water.z) / water.rz;
    const r = Math.hypot(dx, dz) * (1 + (fbm(Math.atan2(dz, dx) * 1.3 + 9, 0.5, this.seed + 67, 2) - 0.5) * 0.25);
    return (1 - r) * Math.min(water.rx, water.rz);
  }

  /** Distance from the town rectangle (0 inside). */
  distanceToTown(x: number, z: number): number {
    const dx = Math.max(0 - x, 0, x - this.w);
    const dz = Math.max(0 - z, 0, z - this.d);
    return Math.hypot(dx, dz);
  }

  /** Height of the land, levelled under the roads out of town. */
  heightAt(x: number, z: number): number {
    const h = this.ground(x, z);
    const road = this.roads.near(x, z);
    if (!road) return h;
    // Level with the road across it, then graded back into the land beside it.
    const t = 1 - smoothstep(road.width / 2 + 0.4, road.width / 2 + ROAD_VERGE, road.dist);
    return h + (road.h - 0.02 - h) * t;
  }

  /** How high the mountains rise on this side, 0..1: the valley opens out flat towards low z, where the menus look at the town from. */
  private openness(z: number): number {
    return smoothstep(-0.9, 0.1, (z - this.d / 2) / (this.d / 2 + 90));
  }

  /** Height the mountains (if any) add at a point `dist` from the town. */
  private mountain(x: number, z: number, dist: number): number {
    const top = this.land.mountains;
    if (!top) return 0;
    const ramp = smoothstep(APRON + 10, APRON + 130, dist);
    if (ramp <= 0) return 0;
    const open = this.openness(z);
    // A pass where the road leaves town at each end.
    const past = Math.max(-x, x - this.w);
    const pass = this.passZ === null || past <= 0 ? 1 : 1 - smoothstep(0, 50, past) * (1 - smoothstep(14, 70, Math.abs(z - this.passZ))) * 0.8;
    const crest = ridged(x * 0.0075, z * 0.0075, this.seed + 211);
    return top * ramp * open * pass * (0.25 + 0.75 * crest);
  }

  /** Height of the land before the roads are cut in. */
  ground(x: number, z: number): number {
    const dist = this.distanceToTown(x, z);
    const L = this.land;
    let h = 0;
    if (dist > APRON) {
      const rise = smoothstep(APRON, APRON + L.rise, dist);
      // Among mountains the near side of the valley (towards the menus' camera) stays low.
      const open = L.mountains ? this.openness(z) : 1;
      const hills = (fbm(x * 0.018, z * 0.018, this.seed) * L.hills - (L.hills * 4) / 14) * (0.15 + 0.85 * open);
      const rim = Math.pow(Math.max(0, dist - L.rimFrom) / RING, 2) * L.rim * open;
      // By water the land stays above it (no puddles in the hollows).
      h = Math.max(L.water ? 0 : -0.4, rise * hills + rim) + this.mountain(x, z, dist);
    }
    if (!L.water || dist === 0) return h;
    // Down a sandy bank to the shore, then under the water.
    const e = this.waterEdge(x, z);
    if (e < -BEACH * 1.5) return h;
    if (e > 0) return WATER_Y - Math.min(4, e * 0.22 + e * e * 0.004);
    const bank = WATER_Y - e * 0.09;
    return h + (Math.min(h, bank) - h) * smoothstep(-BEACH * 1.5, -BEACH * 0.3, e);
  }

  async build(scene: Scene, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>): Promise<NatureResult> {
    const meshes: Mesh[] = [];
    meshes.push(await perfTimeAsync('nature: terrain', async () => this.terrain(scene, pace)));
    if (this.land.water) meshes.push(await perfTimeAsync('nature: water', async () => this.water(scene)));
    meshes.push(...(await perfTimeAsync('nature: roads', async () => this.roads.build(scene))));
    this.batches = [];
    await perfTimeAsync('nature: forests', () => this.forests(scene, assets, pace));
    await perfTimeAsync('nature: lawns', () => this.lawns(scene, assets, world, pace));
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
      this.lawn = this.lawn.filter((layer) => !layer.meshes.some((mesh) => going.has(mesh)));
      stale.forEach(({ again }, i) => {
        if (!again!.tiles) return;
        const layers = lawnLayers(fresh[i].built, again!.matrices, again!.tiles);
        this.lawn.push(...layers);
        if (this.lawnMask) this.hideLawn(layers, this.lawnMask);
      });
    };
    return { ...gather(fresh), replaced, commit };
  }

  private async terrain(scene: Scene, pace?: () => Promise<void>): Promise<Mesh> {
    const size = Math.max(this.w, this.d) + this.ring * 2;
    const segments = Math.min(TERRAIN_SEGMENTS, Math.round(size / TERRAIN_STEP));
    const ground = MeshBuilder.CreateGround('terrain', { width: size, height: size, subdivisions: segments, updatable: false }, scene);
    ground.position.set(this.w / 2, 0, this.d / 2);
    ground.bakeCurrentTransformIntoVertices();

    const positions = ground.getVerticesData(VertexBuffer.PositionKind)!;
    const colors = new Float32Array((positions.length / 3) * 4);
    // sRGB vertex colours (converted on the GPU) multiply the grass detail texture.
    // Lush, slightly yellow-green lawn (the classic life-sim look); greener on the hills. Each
    // world has its own greens.
    const [lush, grass, meadow, hill] = this.land.grass.map((c) => Color3.FromHexString(c));
    const dirt = Color3.FromHexString('#7C6748');
    const sand = Color3.FromHexString('#D8C89A');
    const wetSand = Color3.FromHexString('#A99A72');
    const rock = Color3.FromHexString('#7A7064');
    const snow = Color3.FromHexString('#F1F3F4');
    const alpine = Color3.FromHexString('#8FA266');
    const crops = ['#C9B45C', '#7E6142', '#7A9C44', '#B5A35A', '#6C8A3C'].map((c) => Color3.FromHexString(c));
    const furrow = Color3.FromHexString('#5F4733');
    const tmp = new Color3();
    let seen = 0;
    for (let i = 0, c = 0; i < positions.length; i += 3, c += 4) {
      if (++seen % 4096 === 0) await pace?.();
      const x = positions[i];
      const z = positions[i + 2];
      const h = this.heightAt(x, z);
      positions[i + 1] = h;
      const patch = fbm(x * 0.06, z * 0.06, this.seed + 99, 3);
      const fine = fbm(x * 0.21, z * 0.21, this.seed + 31, 2);
      Color3.LerpToRef(lush, grass, smoothstep(0.3, 0.55, fine), tmp);
      Color3.LerpToRef(tmp, meadow, smoothstep(0.5, 0.75, patch), tmp);
      Color3.LerpToRef(tmp, hill, smoothstep(1, 10, h), tmp);
      // Worn, earthy patches: sparse, and more common on the flat ground near the town.
      const bare = smoothstep(0.66, 0.8, fbm(x * 0.09 + 13, z * 0.09 - 7, this.seed + 5, 3)) * (1 - smoothstep(2, 8, h));
      Color3.LerpToRef(tmp, dirt, bare * 0.85, tmp);
      // Farmland: a patchwork of fields (wheat, ploughed earth, green crops) round the town.
      const cell = this.fieldCell(x, z);
      if (cell) {
        const crop = crops[Math.floor(cell.kind * crops.length)];
        Color3.LerpToRef(tmp, crop, cell.weight * 0.9, tmp);
        // Ploughed and planted fields show their furrows.
        if (crop !== crops[0]) Color3.LerpToRef(tmp, furrow, cell.weight * 0.35 * Math.max(0, Math.sin(cell.v * Math.PI * 1.4)), tmp);
      }
      // Mountains: bare rock above the trees and on steep faces, snow on the tops.
      const top = this.land.mountains;
      if (top) {
        const steep = Math.hypot(this.heightAt(x + 1.5, z) - h, this.heightAt(x, z + 1.5) - h) / 1.5;
        // Pale alpine meadow above the trees, then rock, then snow.
        Color3.LerpToRef(tmp, alpine, smoothstep(TREELINE * 0.8, TREELINE * 1.25, h + fine * 6) * 0.8, tmp);
        Color3.LerpToRef(tmp, rock, Math.max(smoothstep(top * 0.16, top * 0.28, h + fine * 12), smoothstep(0.7, 1.2, steep) * smoothstep(8, 18, h)) * 0.92, tmp);
        Color3.LerpToRef(tmp, snow, smoothstep(top * 0.33, top * 0.45, h + patch * 20) * (1 - smoothstep(1.5, 2.6, steep) * 0.5), tmp);
      }
      // A worn verge of dirt and gravel along the roads.
      const road = this.roads.near(x, z);
      if (road) Color3.LerpToRef(tmp, dirt, 0.45 * (1 - smoothstep(road.width / 2, road.width / 2 + 2.2, road.dist)), tmp);
      // Sand along the water, darker where it is wet.
      const e = this.waterEdge(x, z);
      if (e > -BEACH) {
        Color3.LerpToRef(tmp, e > -1.5 ? wetSand : sand, smoothstep(-BEACH, -BEACH * 0.45, e), tmp);
      }
      colors[c] = tmp.r;
      colors[c + 1] = tmp.g;
      colors[c + 2] = tmp.b;
      colors[c + 3] = 1;
    }
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
    const size = Math.max(this.w, this.d) + this.ring * 2;
    const sheet = MeshBuilder.CreateGround('water', { width: size, height: size }, scene);
    sheet.position.set(this.w / 2, WATER_Y, this.d / 2);
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

  /**
   * The farm field a point lies in (farm worlds, outside the town): the field's kind (0..1),
   * how far across it the point is along the furrows (`v`, m), how strongly it shows (fading out
   * with distance and up the hills) and whether the point is by a hedgerow at its edge. Null elsewhere.
   */
  private fieldCell(x: number, z: number): { kind: number; v: number; weight: number; hedge: boolean } | null {
    const f = this.land.fields;
    if (!f) return null;
    const dist = this.distanceToTown(x, z);
    if (dist < APRON + 3) return null;
    const weight = (1 - smoothstep(this.ring * 0.6, this.ring * 0.85, dist)) * smoothstep(APRON + 3, APRON + 6, dist);
    if (weight <= 0) return null;
    const c = Math.cos(f.angle);
    const s = Math.sin(f.angle);
    const u = (x * c + z * s) / f.size;
    const v = (-x * s + z * c) / (f.size * 0.7);
    const [cu, cv] = [Math.floor(u), Math.floor(v)];
    // Some cells are left as meadow.
    const kind = hash(cu, cv, this.seed + 77);
    if (kind > 0.82) return null;
    const edge = Math.min(u - cu, cu + 1 - u, v - cv, cv + 1 - v) * f.size * 0.7;
    return { kind: kind / 0.82, v: (v - cv) * f.size * 0.7, weight, hedge: edge < 1.6 };
  }

  /**
   * Woods outside the town: mixed deciduous / birch / conifer stands with bushes and rocks. Trees
   * near the town are the full models and cast shadows; further ones use the low level of detail
   * (a file's own far version), which don't.
   */
  private async forests(scene: Scene, assets: AssetRegistry, pace?: () => Promise<void>): Promise<void> {
    const rand = mulberry32(this.seed ^ 0x5eed);
    this.nearWoods = [];
    this.woodsAsked = null;
    const near = new Map<string, Matrix[]>();
    const far = new Map<string, Matrix[]>();
    const spacing = 4.2;
    const size = Math.max(this.w, this.d) + this.ring * 2;
    const x0 = this.w / 2 - size / 2;
    const z0 = this.d / 2 - size / 2;
    for (let gx = 0; gx * spacing < size; gx++) {
      if (gx % 4 === 0) await pace?.();
      for (let gz = 0; gz * spacing < size; gz++) {
        const x = x0 + (gx + rand()) * spacing;
        const z = z0 + (gz + rand()) * spacing;
        const dist = this.distanceToTown(x, z);
        const L = this.land;
        if (dist < APRON + L.treeline) continue;
        const forest = fbm(x * 0.025, z * 0.025, this.seed + 7, 3);
        let density = smoothstep(0.42 - L.forest, 0.62 - L.forest, forest) * 0.85 + 0.06 + L.forest * 0.25 - (dist < 14 ? 0.05 : 0);
        // Fields are open, with hedgerows and the odd tree along their edges; nothing grows in the water or on the beach.
        const cell = this.fieldCell(x, z);
        if (cell) density = cell.hedge ? 0.55 : density * (1 - cell.weight);
        if (L.water && this.waterEdge(x, z) > -BEACH * 0.8) continue;
        // Bare tops on mountains.
        if (L.mountains && this.heightAt(x, z) > TREELINE + forest * 10) continue;
        // Nothing on the roads or right by them.
        const road = this.roads.near(x, z);
        if (road && road.dist < road.width / 2 + 2.5) continue;
        // The far woods of a wide world thinner (seen through haze; fewer trees to draw).
        if (this.ring > RING && dist > RING && rand() < 0.45) continue;
        if (rand() > density) continue;
        // Species: conifers in the dense cores, birches in groves at the edges, broadleaves elsewhere.
        const grove = fbm(x * 0.06 + 40, z * 0.06 - 15, this.seed + 3, 2);
        const roll = rand();
        let kind: string;
        if (roll < 0.08) kind = 'model.rock';
        else if (roll < 0.2) kind = 'model.bush';
        else if (roll < 0.26) kind = 'model.bush.small';
        else if ((forest > 0.56 && rand() < Math.min(1, (0.62 * L.conifers) / 0.5)) || (L.conifers > 0.5 && rand() < (L.conifers - 0.5) * 1.6)) kind = rand() < 0.4 ? 'model.spruce' : 'model.pine';
        else if (grove > 0.55 - Math.max(0, L.birches - 0.5) * 0.3 && rand() < Math.min(1, (0.7 * L.birches) / 0.5)) kind = 'model.tree.birch';
        else kind = rand() < 0.35 ? 'model.tree.maple' : 'model.tree';
        const tree = kind.startsWith('model.tree') || kind === 'model.pine' || kind === 'model.spruce';
        const scale = kind === 'model.rock' ? 0.6 + rand() * 0.7 : tree ? 0.75 + rand() * 0.6 : 0.8 + rand() * 0.6;
        // (Drawn in this order, as ever: the same woods from the same seed.)
        const up = scale * (0.9 + rand() * 0.25);
        const yaw = rand() * Math.PI * 2;
        const y = this.heightAt(x, z) - 0.05;
        const m = Matrix.Compose(new Vector3(scale, up, scale), Quaternion.RotationAxis(Vector3.Up(), yaw), new Vector3(x, y, z));
        const close = dist < SHADOW_RANGE && kind !== 'model.rock';
        if (close) this.nearWoods.push({ key: kind, x, y, z, sx: scale, sy: up, sz: scale, yaw });
        const bucket = close ? near : far;
        bucket.set(kind, [...(bucket.get(kind) ?? []), m]);
      }
    }
    const [cx, cz] = [this.w / 2, this.d / 2];
    const sector = (x: number, z: number) => `${Math.floor((Math.atan2(z - cz, x - cx) / (Math.PI * 2) + 1) * FOREST_SECTORS) % FOREST_SECTORS}`;
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

  /**
   * Grass tufts and wildflowers on the town's tiles. Every unpaved tile gets some; the ones
   * indoors or under furniture are hidden by `refreshLawn` (rooms and furniture change in build mode).
   */
  private async lawns(scene: Scene, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>): Promise<void> {
    const rand = mulberry32(this.seed ^ 0x1a3d);
    const covered = unlawned(world);
    const onPavement = (x: number, z: number) => covered.some((r) => x >= r.x && x < r.x + r.w && z >= r.z && z < r.z + r.d);
    const tufts: Matrix[] = [];
    const flowers: Matrix[] = [];
    const tuftTiles: number[] = [];
    const flowerTiles: number[] = [];
    for (let z = 0; z < groundDepth(world); z++) {
      if (z % 4 === 0) await pace?.();
      for (let x = 0; x < world.width; x++) {
        const i = z * world.width + x;
        if (onPavement(x + 0.5, z + 0.5)) continue;
        // Tufts are a few thin blades each: a dense, soft lawn.
        const n = rand() < 0.5 ? 2 : 1;
        for (let k = 0; k < n; k++) {
          tufts.push(scatter(x, z, rand, 0.7 + rand() * 0.6));
          tuftTiles.push(i);
        }
        if (rand() < 0.06) {
          flowers.push(scatter(x, z, rand, 0.8 + rand() * 0.5));
          flowerTiles.push(i);
        }
      }
    }
    // Lawn detail around the town too, thinning out with distance.
    for (let n = 0; n < 5000; n++) {
      if (n % 1000 === 0) await pace?.();
      const x = -APRON - 30 + rand() * (world.width + APRON * 2 + 60);
      const z = -APRON - 30 + rand() * (groundDepth(world) + APRON * 2 + 60);
      const dist = this.distanceToTown(x, z);
      if (dist === 0 || rand() < dist / 40 || this.waterEdge(x, z) > -BEACH * 0.6) continue;
      const road = this.roads.near(x, z);
      if (road && road.dist < road.width / 2 + 0.3) continue;
      const m = scatter(x - 0.5, z - 0.5, rand, 0.8 + rand() * 0.7, this.heightAt(x, z));
      if (rand() < 0.1) {
        flowers.push(m);
        flowerTiles.push(-1);
      } else {
        tufts.push(m);
        tuftTiles.push(-1);
      }
    }
    // A lighter lawn keeps every so many of them, the same ones each time (the placement above is unchanged).
    const d = this.lawnDensity;
    const thin = <T>(list: T[]) => (d >= 1 ? list : list.filter((_, k) => Math.floor((k + 1) * d) > Math.floor(k * d)));
    this.lawn = [];
    // About `TUFT_CELL` squares over the town; the tufts scattered round it join the squares at its edge
    // (on their own they would be draws of a few tufts each).
    const [tw, td] = [world.width, groundDepth(world)];
    const [nx, nz] = [Math.max(1, Math.round(tw / TUFT_CELL)), Math.max(1, Math.round(td / TUFT_CELL))];
    const square = (x: number, z: number) => `${Math.min(nx - 1, Math.max(0, Math.floor((x / tw) * nx)))},${Math.min(nz - 1, Math.max(0, Math.floor((z / td) * nz)))}`;
    for (const [key, matrices, tiles, cell] of [
      ['model.grassTuft', thin(tufts), thin(tuftTiles), square],
      // (Flowers are sparse: too few triangles for groups to pay.)
      ['model.flowers', thin(flowers), thin(flowerTiles), undefined],
    ] as const) {
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
    this.hideLawn(this.lawn, this.lawnMask);
  }

  /** Hides the tufts and flowers of `layers` on the tiles `mask` hides them on (`refreshLawn`), and shows the rest. */
  private hideLawn(layers: LawnLayer[], { width, hidden, splitAxis }: LawnMask): void {
    for (const layer of layers) {
      let changed = false;
      for (let k = 0; k < layer.tiles.length; k++) {
        const t = layer.tiles[k];
        let hide = t >= 0 ? hidden[t] : 0;
        if (hide > 1) {
          // The tuft's position within its tile decides its half.
          const fx = layer.base[k * 16 + 12] - (t % width);
          const fz = layer.base[k * 16 + 14] - Math.floor(t / width);
          const axis = splitAxis.get(t) ?? 'dp';
          const half = axis === 'dp' ? (fz < fx ? 0 : 1) : fx + fz < 1 ? 0 : 1;
          // Tufts right at the wall would poke through it.
          const nearWall = (axis === 'dp' ? Math.abs(fx - fz) : Math.abs(fx + fz - 1)) < 0.17;
          hide = half === hide - 2 || nearWall ? 1 : 0;
        }
        if (layer.hidden[k] === hide) continue;
        layer.hidden[k] = hide;
        changed = true;
        // An all-zero matrix hides an instance; `base` brings it back once the tile is outdoors again.
        if (hide) layer.shown.fill(0, k * 16, k * 16 + 16);
        else layer.shown.set(layer.base.subarray(k * 16, k * 16 + 16), k * 16);
      }
      if (!changed) continue;
      for (const mesh of layer.meshes) mesh.thinInstanceSetBuffer('matrix', layer.shown, 16, false);
    }
  }
}

function scatter(x: number, z: number, rand: () => number, scale: number, y = 0): Matrix {
  return Matrix.Compose(
    new Vector3(scale, scale, scale),
    Quaternion.RotationAxis(Vector3.Up(), rand() * Math.PI * 2),
    new Vector3(x + rand(), y, z + rand()),
  );
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
  matrices: Matrix[];
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

/** Lawn tufts or flowers drawn by `meshes`: each instance's original matrix and town tile (-1 outside the town), its matrix as drawn (`shown`) and whether it's hidden. */
interface LawnLayer {
  meshes: Mesh[];
  base: Float32Array;
  shown: Float32Array;
  tiles: Int32Array;
  hidden: Uint8Array;
}

/** Where the lawn is hidden (`refreshLawn`): per town tile, and the split tiles' diagonals. */
export interface LawnMask {
  width: number;
  /** 1 indoors or under furniture; on a tile split by a diagonal wall, 2 or 3: only its half 0 or 1 indoors. */
  hidden: Uint8Array;
  splitAxis: Map<number, 'dp' | 'dn'>;
}

/** The town's ground where no lawn grows at all: its streets, paths and fields (the birds forage on the lawn). */
export function unlawned(world: WorldStructure): readonly { x: number; z: number; w: number; d: number }[] {
  return [...(world.meta?.streets ?? []), ...(world.meta?.paths ?? []), ...(world.meta?.world?.fields ?? [])];
}

/** Where the lawn that grows is hidden now: the tiles indoors (or the half of a split tile that is) and under furniture. */
export function lawnMask(world: WorldStructure): LawnMask {
  const hidden = new Uint8Array(world.width * world.depth);
  for (let i = 0; i < hidden.length; i++) if (world.rooms[i] !== 0) hidden[i] = 1;
  // Split tiles: 2 = half 0 indoors, 3 = half 1 indoors, 4 = both (bit 1 marks "per half").
  const splitAxis = new Map<number, 'dp' | 'dn'>();
  for (const d of world.diagonals ?? []) {
    const i = d.z * world.width + d.x;
    const [a, b] = [d.rooms[0] !== 0, d.rooms[1] !== 0];
    hidden[i] = a && b ? 1 : a ? 2 : b ? 3 : 0;
    splitAxis.set(i, d.axis);
  }
  for (const o of world.objects) {
    for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) hidden[z * world.width + x] = 1;
  }
  return { width: world.width, hidden, splitAxis };
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
function lawnLayers(built: Instanced, matrices: readonly Matrix[], tiles: readonly number[]): LawnLayer[] {
  const layers: LawnLayer[] = [];
  for (const { meshes, indices } of [{ meshes: [...built.shared, ...built.whole], indices: matrices.map((_, k) => k) }, ...built.cells]) {
    if (!meshes.length) continue;
    const base = new Float32Array(indices.length * 16);
    indices.forEach((k, i) => matrices[k].copyToArray(base, i * 16));
    layers.push({ meshes, base, shown: new Float32Array(base), tiles: Int32Array.from(indices, (k) => tiles[k]), hidden: new Uint8Array(indices.length) });
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
  matrices: Matrix[],
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
  const groups = new Map<string, number[]>();
  if (cell) {
    matrices.forEach((m, i) => {
      const k = cell(m.m[12], m.m[14]);
      const group = groups.get(k);
      if (group) group.push(i);
      else groups.set(k, [i]);
    });
  }
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
      matrices[indices[i]].copyToArray(buffer, i * 16);
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

// ---- Garden dressing ------------------------------------------------------------------

type Decor = [number, number, number, number][];

/** Clear distances for lot trees (trunk to edge, metres). */
const TREE_CLEAR = { street: 3.0, path: 1.3, house: 2.2, object: 1.0 };

/**
 * Moves a lot tree (by up to 2.5 m, staying on the viewed lot) until its trunk is clear of the
 * street (road, sidewalk, kerb furniture and streetlights all stand inside the street rects, see
 * street.ts), garden paths and driveways, the house and placed objects; null when no spot is.
 */
function clearTreeSpot(x: number, z: number, world: WorldStructure, view: { x: number; z: number; w: number; d: number } | null): [number, number] | null {
  const streets = world.meta?.streets ?? [];
  const paths = world.meta?.paths ?? [];
  const gap = (r: { x: number; z: number; w: number; d: number }, px: number, pz: number) =>
    Math.hypot(Math.max(r.x - px, 0, px - (r.x + r.w)), Math.max(r.z - pz, 0, pz - (r.z + r.d)));
  const indoorNear = (px: number, pz: number, r: number) => {
    for (let tz = Math.floor(pz - r); tz <= Math.floor(pz + r); tz++) {
      for (let tx = Math.floor(px - r); tx <= Math.floor(px + r); tx++) {
        if (tx < 0 || tz < 0 || tx >= world.width || tz >= world.depth || world.rooms[tz * world.width + tx] === 0) continue;
        if (gap({ x: tx, z: tz, w: 1, d: 1 }, px, pz) < r) return true;
      }
    }
    return false;
  };
  const ok = (px: number, pz: number) =>
    (!view || (px > view.x + 0.8 && px < view.x + view.w - 0.8 && pz > view.z + 0.8 && pz < view.z + view.d - 0.8)) &&
    streets.every((r) => gap(r, px, pz) >= TREE_CLEAR.street) &&
    paths.every((r) => gap(r, px, pz) >= TREE_CLEAR.path) &&
    world.objects.every((o) => gap(o, px, pz) >= TREE_CLEAR.object) &&
    !indoorNear(px, pz, TREE_CLEAR.house);
  for (let r = 0; r <= 2.5; r += 0.5) {
    const steps = r === 0 ? 1 : 12;
    for (let k = 0; k < steps; k++) {
      const a = (k / steps) * Math.PI * 2;
      const px = x + Math.cos(a) * r;
      const pz = z + Math.sin(a) * r;
      if (ok(px, pz)) return [px, pz];
    }
  }
  return null;
}

/** The garden dressing (`natureDecor`'s) as placed, for the birds. */
export function decorPlants(decor: readonly [string, Decor][]): PlantPlacement[] {
  return decor.flatMap(([key, items]) => items.map(([x, z, s, yaw]) => ({ key, x, y: 0, z, sx: s, sy: s, sz: s, yaw })));
}

/**
 * Splits the house builder's garden dressing (`model.tree` lot trees, `model.bush` foundation
 * shrubs; items are [x, z, scale, yaw]) into species. Lot trees mix oaks, maples, blossom, fruit,
 * birch, magnolia and Japanese maple; each house gets either a clipped hedge along its walls or a
 * mixed border of shrubs, hydrangeas, roses, lavender and clipped balls. Deterministic per position.
 */
export function natureDecor(key: string, items: Decor, world?: WorldStructure, view?: { x: number; z: number; w: number; d: number } | null): [string, Decor][] {
  const out = new Map<string, Decor>();
  const put = (k: string, item: [number, number, number, number]) => out.set(k, [...(out.get(k) ?? []), item]);
  if (key === 'model.tree') {
    for (const item of items) {
      const h = hash(Math.floor(item[0] * 7), Math.floor(item[1] * 7), 91);
      // Crowns are ~2 m in radius: keep trunks well off the sidewalk, its streetlights and props.
      const spot = world ? clearTreeSpot(item[0], item[1], world, view ?? null) : [item[0], item[1]];
      if (!spot) continue;
      const species =
        h < 0.24 ? 'model.tree' : h < 0.4 ? 'model.tree.maple' : h < 0.55 ? 'model.tree.blossom' : h < 0.68 ? 'model.tree.fruit' : h < 0.8 ? 'model.tree.birch' : h < 0.9 ? 'model.tree.magnolia' : 'model.tree.japaneseMaple';
      put(species, [spot[0], spot[1], item[2], item[3]]);
    }
  } else if (key === 'model.bush' && items.length) {
    // One border style per house (the first shrub's position stands in for the lot).
    const style = hash(Math.floor(items[0][0] / 8), Math.floor(items[0][1] / 8), 92);
    for (const item of items) {
      const [x, z, s] = item;
      const h = hash(Math.floor(x * 5), Math.floor(z * 5), 93);
      if (style < 0.4) {
        // Hedge segments run along the wall: shrubs sit at x + 0.5 (wall along x) or z + 0.5.
        const alongX = Math.abs(x - Math.floor(x) - 0.5) < 0.05;
        put('model.hedge', [x, z, 0.95 + (s - 0.55) * 0.2, alongX ? 0 : Math.PI / 2]);
      } else {
        // A mixed border: green shrubs with hydrangeas, roses, lavender and clipped balls among them.
        const kind =
          h < 0.3 ? 'model.bush' : h < 0.45 ? 'model.bush.flowering' : h < 0.55 ? 'model.bush.small' : h < 0.67 ? 'model.garden.hydrangea' : h < 0.78 ? 'model.garden.rosebush' : h < 0.89 ? 'model.garden.lavender' : 'model.garden.boxwood';
        put(kind, [x, z, s * (kind.startsWith('model.garden') ? 1.25 : 1.1), item[3]]);
      }
    }
  } else if (items.length) {
    out.set(key, items);
  }
  return [...out];
}

// ---- Wind and foliage shading -----------------------------------------------------------

/** Wind shared by every foliage material of a scene; `time` advances per frame. */
const windState = new WeakMap<Scene, { time: number; strength: number }>();

/**
 * The wind's bend, as the vertex shader draws it and `windSway` follows it for what sits on a
 * branch: a sway of two sines along `dir`, in gusts, phased by where each plant stands, bending a
 * point by `bend` × its height in its model squared (m).
 */
const WIND = {
  /** Which way it blows (x, z). */
  dir: [0.8, 0.6],
  /** Phase per metre (x, z), so neighbours sway out of step. */
  phase: [0.21, 0.17],
  /** Gusts: `base + amp · sin(t · rate + phase · ph)`. */
  gust: { base: 0.6, amp: 0.4, rate: 0.37, phase: 0.5 },
  /** The sway: `Σ weight · sin(t · rate + phase · ph)`. */
  sway: [
    { weight: 0.7, rate: 1.1, phase: 1 },
    { weight: 0.3, rate: 2.3, phase: 1.7 },
  ],
  /** Bend (m per m² of height, per unit of sway). */
  bend: 0.0022,
} as const;

/**
 * Per finish (primitive parts' `material`): x bend (per m^2 of height), y flutter amplitude (m),
 * z flutter speed, w flutter ramp (1/m).
 */
const WIND_KIND: Record<string, [number, number, number, number]> = {
  'material.finish.bark': [WIND.bend, 0, 0, 0],
  'material.finish.foliage': [WIND.bend, 0.012, 3.1, 1.5],
};

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
  const t = state.time;
  const ph = px * WIND.phase[0] + pz * WIND.phase[1];
  const g = WIND.gust;
  const gust = (g.base + g.amp * Math.sin(t * g.rate + ph * g.phase)) * state.strength;
  let sway = 0;
  for (const s of WIND.sway) sway += Math.sin(t * s.rate + ph * s.phase) * s.weight;
  return sway * gust * WIND.bend;
}

/**
 * The way (x, z) the wind bends a plant instance turned `yaw` and scaled `sx` across and `sz` deep:
 * the wind's direction in the world, but for a plant scaled unevenly (a hedge), as the shader bends
 * it in the model's own space (its `lw`).
 */
export function windDirection(yaw: number, sx: number, sz: number, out: { x: number; z: number }): void {
  // The instance's x and z axes in the world (`Quaternion.RotationAxis(Up, yaw)`), and the wind along each.
  const [c, s] = [Math.cos(yaw), Math.sin(yaw)];
  const [dx, dz] = WIND.dir;
  const along = dx * c - dz * s;
  const across = (dx * s + dz * c) * (sz / sx) ** 2;
  out.x = along * c + across * s;
  out.z = -along * s + across * c;
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
  update(l: Lighting): void {
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
