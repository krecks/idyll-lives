/**
 * The lawn's grass on PlayCanvas: blades built in the vertex shader (`shaders/grass.ts`), drawn a
 * 2 m patch at a time (`shared/grass.ts` says where it grows and how dense with distance).
 *
 * One mesh per draw of `GRASS.lods` (a patch's blades, each only its t along the blade, its side
 * and its number), drawn instanced: each frame the patches with grass within the preset's reach of
 * the eye and in view are sorted into the draws by how far their nearest point is (`grassLod`), a
 * copy each, its matrix moving it to its corner on the ground, its `GRASS_SLOPE` the ground's tilt
 * under it. Its own material (the textures and the eye are this town's): both faces, lit alike,
 * casting no shadow, receiving the sun's. The textures: the field's colour (and grassiness), the
 * town's mask (`refresh`, when the lot changes) and the gusts' noise.
 */

import * as pc from './engine';
import { Instances, type InstanceStream } from './instancing';
import { GAME_FRONT_FACE, MaterialLibrary } from './materials';
import { registerShaderFeature } from './shaders/chunks';
import { GRASS_SHADER } from './shaders/grass';
import type { WorldStructure } from '../../core/protocol';
import { identity } from '../shared/math';
import { GRASS, GRASS_PUSHERS, grassLod, gustNoise, townGrass, type GrassField, type GrassPusher, type GrassQuality } from '../shared/grass';

export type { GrassPusher, GrassQuality } from '../shared/grass';

// (Before any device's chunks are built.)
registerShaderFeature(GRASS_SHADER);

/**
 * How far the eye may move (m), and its view turn (its planes' normals: about a third of a degree,
 * 0.4 m at 60 m), before the patches are chosen again: they're chosen with that much room round them.
 */
const SLACK = 0.4;
const TURN = 0.006;

/** The ground's slope (x, z) under a patch, per copy. */
const GRASS_SLOPE: InstanceStream = { semantic: pc.SEMANTIC_ATTR8, components: 2 };

const gustTextures = new WeakMap<pc.GraphicsDevice, pc.Texture>();

/** The gusts' noise (`gustNoise`): one per device, wrapping. */
function gustTexture(device: pc.GraphicsDevice): pc.Texture {
  let tex = gustTextures.get(device);
  if (!tex) {
    const n = GRASS.gust.size;
    tex = new pc.Texture(device, { name: 'grassGust', width: n, height: n, format: pc.PIXELFORMAT_R8, mipmaps: false, minFilter: pc.FILTER_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_REPEAT, addressV: pc.ADDRESS_REPEAT, levels: [gustNoise()] });
    gustTextures.set(device, tex);
  }
  return tex;
}

const grounds = new WeakSet<pc.Material>();
const noGrass = new WeakMap<pc.GraphicsDevice, { colour: pc.Texture; town: pc.Texture }>();

/**
 * The ground's gusts (`GAME_GRASS_GROUND`) on `material` (the terrain's, shared by every town's):
 * the gusts' noise, and no grass until a town's draw says where it grows (`Grass.onGround`).
 */
export function grassGround(device: pc.GraphicsDevice, material: pc.StandardMaterial): void {
  if (grounds.has(material)) return;
  grounds.add(material);
  let none = noGrass.get(device);
  if (!none) {
    const blank = (name: string, format: number, bytes: number) => new pc.Texture(device, { name, width: 1, height: 1, format, mipmaps: false, levels: [new Uint8Array(bytes)] });
    noGrass.set(device, (none = { colour: blank('noGrass', pc.PIXELFORMAT_RGBA8, 4), town: blank('noTownGrass', pc.PIXELFORMAT_R8, 1) }));
  }
  material.setDefine('GAME_GRASS_GROUND', '');
  material.setParameter('game_grassGust', gustTexture(device));
  material.setParameter('game_grassColour', none.colour);
  material.setParameter('game_grassTown', none.town);
  material.setParameter('game_grassArea', new Float32Array([0, 0, 1, 1]));
  material.setParameter('game_grassEye', new Float32Array(4));
  // (No town: its mask isn't read, nor the lawn's tone drawn.)
  material.setParameter('game_grassTownSize', new Float32Array([0, 0, -2, -1]));
  material.update();
}

/** A patch's blades: `blades` of `segments` each, (t, side, number) per vertex. */
function patchMesh(device: pc.GraphicsDevice, blades: number, segments: number): pc.Mesh {
  const per = segments * 2 + 1;
  const positions = new Float32Array(blades * per * 3);
  const normals = new Float32Array(blades * per * 3);
  const indices = new Uint16Array(blades * (segments * 2 - 1) * 3);
  for (let b = 0, v = 0, k = 0; b < blades; b++) {
    const first = v;
    for (let s = 0; s < segments; s++) {
      for (const side of [-1, 1]) {
        positions.set([s / segments, side, b], v * 3);
        normals[v * 3 + 1] = 1;
        v++;
      }
    }
    positions.set([1, 0, b], v * 3);
    normals[v * 3 + 1] = 1;
    v++;
    // A strip up the blade: a quad a segment, the last one a triangle to the tip.
    for (let s = 0; s < segments - 1; s++) {
      const a = first + s * 2;
      indices.set([a, a + 1, a + 3, a, a + 3, a + 2], k);
      k += 6;
    }
    const top = first + (segments - 1) * 2;
    indices.set([top, top + 1, v - 1], k);
    k += 3;
  }
  const mesh = new pc.Mesh(device);
  mesh.setPositions(positions);
  mesh.setNormals(normals);
  mesh.setIndices(indices);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  // (The blades reach past their patch, leaning and in the wind: the copies are culled here, not by the box.)
  const p = GRASS.patch;
  mesh.aabb = new pc.BoundingBox(new pc.Vec3(p / 2, 0.2, p / 2), new pc.Vec3(p / 2 + 0.4, 0.4, p / 2 + 0.4));
  return mesh;
}

/** The meshes of `GRASS.lods`, one set per device (shared by every town's grass). */
const meshSets = new WeakMap<pc.GraphicsDevice, pc.Mesh[]>();
function lodMeshes(device: pc.GraphicsDevice): pc.Mesh[] {
  let set = meshSets.get(device);
  if (!set) {
    set = GRASS.lods.map((l) => patchMesh(device, l.blades, l.segments));
    // (Shared and kept: a draw let go never takes them with it.)
    for (const m of set) m.incRefCount();
    meshSets.set(device, set);
  }
  return set;
}

export class Grass {
  /** A draw per `GRASS.lods`, under the world root (game space), switched off until shown. */
  readonly draws: Instances[];
  private readonly material: pc.StandardMaterial;
  private readonly colour: pc.Texture;
  private town: pc.Texture;
  /** The residents pushing it (`setPushers`: x, z, strength each). */
  private readonly push = new Float32Array(GRASS_PUSHERS * 4);
  /**
   * The eye (game space) and how far the grass reaches (the blades shrink away towards it; the
   * ground, `onGround`, blends into the lawn's colour). Its uniforms hold these arrays: they're
   * changed in place.
   */
  private readonly eye = new Float32Array(4);
  /** The colour texture's corner and 1 / its size (m). */
  private readonly area: Float32Array;
  /** The town's ground in metres (the mask's size), and from how far round it to how far the lawn's tone fades out on the ground. */
  private readonly townSize = new Float32Array(4);
  /** The town's ground draw (`onGround`): it reads the mask too. */
  private ground: pc.MeshInstance | null = null;
  /** A patch's bounds, with room for the eye to move by `SLACK` before they're chosen again. */
  private readonly sphere = new pc.BoundingSphere(new pc.Vec3(), GRASS.patch * 0.75 + 0.4 + SLACK);
  private readonly matrix = new Float32Array(16);
  private readonly plane = new pc.Plane();
  private readonly counts: number[];
  /** What the copies were last chosen for (the frustum's planes and the eye), to choose again only when it changes. */
  private readonly last = new Float32Array(6 * 4 + 3);
  /** The patches with grass changed (`refresh`): chosen again whether or not anything moved. */
  private stale = true;

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
    private readonly field: GrassField,
    private readonly quality: GrassQuality,
  ) {
    const device = app.graphicsDevice;
    this.colour = new pc.Texture(device, { name: 'grassColour', width: field.w, height: field.d, format: pc.PIXELFORMAT_RGBA8, mipmaps: false, minFilter: pc.FILTER_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, levels: [field.colour] });
    this.town = this.townTexture();
    this.area = new Float32Array([field.x0, field.z0, 1 / field.w, 1 / field.d]);
    this.townSize[2] = field.margin;
    this.townSize[3] = field.margin + GRASS.toneFade;
    this.eye[3] = quality.radius;
    const mat = (this.material = new pc.StandardMaterial());
    mat.name = 'grass';
    mat.frontFace = GAME_FRONT_FACE;
    // Matte: no highlight (a blade is a few pixels; the specular path was a third of their cost,
    // 0.35 ms on Medium, WebGL2). The sky stays: it is the ambient light, which keeps the night dark.
    mat.useMetalness = false;
    mat.specular.set(0, 0, 0);
    mat.diffuse.set(1, 1, 1);
    // Both faces, lit alike (the shader turns the normal to the eye).
    mat.cull = pc.CULLFACE_NONE;
    mat.twoSidedLighting = false;
    mat.setAttribute('game_grassSlope', GRASS_SLOPE.semantic);
    MaterialLibrary.for(app).adopt(mat);
    mat.setDefine('GAME_GRASS', '');
    mat.setParameter('game_grassArea', this.area);
    mat.setParameter('game_grassEye', this.eye);
    mat.setParameter('game_grassDensity', quality.density);
    mat.setParameter('game_grassPush[0]', this.push);
    mat.setParameter('game_grassColour', this.colour);
    mat.setParameter('game_grassGust', gustTexture(device));
    this.setTown();
    mat.update();
    this.draws = lodMeshes(device).map((mesh, k) => {
      const d = new Instances(device, parent, mesh, mat, { name: `grass#${k}`, streams: [GRASS_SLOPE], castShadows: false, receiveShadows: true, cull: false, capacity: 64 });
      d.enabled = false;
      return d;
    });
    this.counts = this.draws.map(() => 0);
  }

  /** The town's mask as a texture (one byte a texel). */
  private townTexture(): pc.Texture {
    const f = this.field;
    return new pc.Texture(this.app.graphicsDevice, { name: 'grassTown', width: Math.max(1, f.townW), height: Math.max(1, f.townD), format: pc.PIXELFORMAT_R8, mipmaps: false, minFilter: pc.FILTER_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, levels: [f.town.length ? f.town : new Uint8Array(1)] });
  }

  /** The town's mask and its size on the material. */
  private setTown(): void {
    const f = this.field;
    const n = GRASS.townTexels;
    this.townSize[0] = f.townW / n;
    this.townSize[1] = f.townD / n;
    for (const on of [this.material, this.ground]) {
      on?.setParameter('game_grassTown', this.town);
      on?.setParameter('game_grassTownSize', this.townSize);
    }
  }

  /**
   * The town's ground (`meshInstance`, on a `grassGround` material) shows the gusts where this grass
   * grows, and its shade near the eye (the town's mask too, `refresh`); the lawn's tone and gusts
   * go on past the field, fading out (`GRASS.toneFade`).
   */
  onGround(meshInstance: pc.MeshInstance): void {
    this.ground = meshInstance;
    meshInstance.setParameter('game_grassEye', this.eye);
    meshInstance.setParameter('game_grassColour', this.colour);
    meshInstance.setParameter('game_grassArea', this.area);
    this.setTown();
  }

  /** The lawn grows as the lot is now (rooms and furniture changed): the town's mask again. */
  refresh(world: WorldStructure): void {
    townGrass(this.field, world);
    // (A lot of another shape, even of the same area, needs a texture of its own.)
    if (this.field.townW !== this.town.width || this.field.townD !== this.town.height || !this.field.town.length) {
      this.town.destroy();
      this.town = this.townTexture();
    } else {
      const data = this.town.lock() as Uint8Array;
      data.set(this.field.town);
      this.town.unlock();
    }
    this.setTown();
    this.stale = true;
  }

  /** The residents pushing the grass aside (the nearest first; at most `GRASS_PUSHERS`). */
  setPushers(pushers: readonly GrassPusher[]): void {
    const push = this.push;
    push.fill(0);
    for (let i = 0; i < Math.min(pushers.length, GRASS_PUSHERS); i++) {
      const p = pushers[i];
      push[i * 4] = p.x;
      push[i * 4 + 1] = p.z;
      push[i * 4 + 2] = p.strength;
    }
  }

  /**
   * Each frame: the patches in view near `eye` (game space) into their draws. `frustum` is the
   * camera's as last drawn (PlayCanvas space: z mirrored); chosen again only when it or the eye moved.
   */
  frame(eye: { x: number; y: number; z: number }, frustum: pc.Frustum): void {
    this.eye[0] = eye.x;
    this.eye[1] = eye.y;
    this.eye[2] = eye.z;
    if (!this.moved(frustum) && !this.stale) return;
    this.stale = false;
    const f = this.field;
    const p = GRASS.patch;
    const r = this.quality.radius;
    const i0 = Math.max(0, Math.floor((eye.x - r - f.x0) / p));
    const i1 = Math.min(f.pw - 1, Math.floor((eye.x + r - f.x0) / p));
    const j0 = Math.max(0, Math.floor((eye.z - r - f.z0) / p));
    const j1 = Math.min(f.pd - 1, Math.floor((eye.z + r - f.z0) / p));
    const counts = this.counts;
    counts.fill(0);
    const nearest = this.quality.nearest;
    const density = this.quality.density;
    const centre = this.sphere.center;
    const m = identity(this.matrix);
    for (let j = j0; j <= j1; j++) {
      const z = f.z0 + j * p;
      const dz = Math.max(z - eye.z, 0, eye.z - (z + p));
      for (let i = i0; i <= i1; i++) {
        const k = j * f.pw + i;
        if (!f.patches[k]) continue;
        const x = f.x0 + i * p;
        const dx = Math.max(x - eye.x, 0, eye.x - (x + p));
        const y = f.ground[k * 3] + (f.ground[k * 3 + 1] + f.ground[k * 3 + 2]) * (p / 2);
        const dy = eye.y - y;
        const d = Math.sqrt(dx * dx + dy * dy + dz * dz);
        if (d > r) continue;
        // (PlayCanvas space: the world root mirrors z.)
        centre.set(x + p / 2, y + 0.15, -(z + p / 2));
        if (!frustum.containsSphere(this.sphere)) continue;
        // (As near as the eye may come before they're chosen again.)
        const lod = Math.max(nearest, grassLod(d - SLACK, density));
        const draw = this.draws[lod];
        const n = counts[lod]++;
        draw.reserve(n + 1);
        m[12] = x;
        m[13] = f.ground[k * 3];
        m[14] = z;
        draw.setMatrix(n, m);
        draw.set(GRASS_SLOPE, n, f.ground, k * 3 + 1);
      }
    }
    for (let k = 0; k < this.draws.length; k++) {
      this.draws[k].setCount(counts[k]);
      this.draws[k].update(false);
    }
  }

  /** Whether the frustum or the eye changed since the copies were last chosen. */
  private moved(frustum: pc.Frustum): boolean {
    const last = this.last;
    const plane = this.plane;
    let moved = false;
    // (A turn of a third of a degree, or the planes or the eye moved by `SLACK`: the margin the patches are chosen with.)
    for (let i = 0; i < 6 && !moved; i++) {
      const p = frustum.getPlane(i, plane);
      const o = i * 4;
      moved = Math.abs(last[o] - p.normal.x) > TURN || Math.abs(last[o + 1] - p.normal.y) > TURN || Math.abs(last[o + 2] - p.normal.z) > TURN || Math.abs(last[o + 3] - p.distance) > SLACK;
    }
    const e = this.eye;
    if (!moved) moved = Math.abs(last[24] - e[0]) > SLACK || Math.abs(last[25] - e[1]) > SLACK || Math.abs(last[26] - e[2]) > SLACK;
    if (!moved) return false;
    for (let i = 0; i < 6; i++) {
      const p = frustum.getPlane(i, plane);
      last[i * 4] = p.normal.x;
      last[i * 4 + 1] = p.normal.y;
      last[i * 4 + 2] = p.normal.z;
      last[i * 4 + 3] = p.distance;
    }
    last.set(e.subarray(0, 3), 24);
    return true;
  }

  dispose(): void {
    for (const d of this.draws) d.destroy();
    MaterialLibrary.for(this.app).release(this.material);
    this.material.destroy();
    this.colour.destroy();
    this.town.destroy();
  }
}
