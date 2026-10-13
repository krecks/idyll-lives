/**
 * Live mirror reflections. Every mirror's glass (a model's `mirrors`) gets a flat stand-in just in
 * front of it; the few nearest the camera that face it show the room in a `MirrorTexture` (the
 * scene drawn once more, reflected in the glass's plane, at a fraction of the screen's size).
 * The rest keep their glossy glass. A reflection is only drawn while its mirror is on screen.
 *
 * Mirrors follow their object's instance matrix each frame, so things popping in, hidden storeys
 * and objects being moved need nothing extra. A reflection's render target exists only while a
 * mirror uses it (and a little after, as mirrors come and go with the camera's turns).
 */

import { Color3, Constants, Matrix, Mesh, MirrorTexture, Plane, StandardMaterial, Vector3, VertexData, type AbstractMesh, type Camera, type Scene } from './core';
import type { MirrorSurface } from './models';

/** A placed object with mirror glass. */
export interface MirrorInstance {
  surfaces: MirrorSurface[];
  /** Its template's instance matrices (16 floats each) and its index there. */
  matrices: Float32Array;
  index: number;
}

/** Resolution of a reflection, as a share of the screen's. */
const RATIO = 0.75;
/** Mirrors further from the camera than this (m) aren't worth a reflection. */
const RANGE = 40;
/** Milliseconds a reflection's render target is kept once no mirror uses it. */
const KEEP_MS = 10_000;

interface Reflection {
  texture: MirrorTexture;
  material: StandardMaterial;
  /** When a mirror last used it (`performance.now()`). */
  used: number;
}

interface Item {
  instance: MirrorInstance;
  meshes: Mesh[];
  /** Per mesh: its world matrix (Babylon keeps the one a mesh is frozen with). */
  worlds: Matrix[];
  /** Pool slot showing its reflection, or -1. */
  slot: number;
  /** Its object's instance matrix this frame. */
  matrix: Matrix;
  centre: Vector3;
  normal: Vector3;
  distance: number;
}

export class Mirrors {
  /** The reflections that may be live at once (null: not made, or let go). */
  private readonly pool: (Reflection | null)[];
  private readonly square: Mesh;
  private readonly disc: Mesh;
  private readonly overlays = new Set<AbstractMesh>();
  private items: Item[] = [];
  private readonly world = new Matrix();
  private readonly back = new Vector3();
  private readonly ranked: Item[] = [];

  /**
   * `count`: how many mirrors reflect at once (0: none). `reflected`: whether a mesh shows in
   * reflections (the overlays never do).
   */
  /** A reflection made (the benchmark watches what each draws). */
  onCreated: ((texture: MirrorTexture) => void) | null = null;

  constructor(
    private readonly scene: Scene,
    count: number,
    private readonly reflected: (mesh: AbstractMesh) => boolean,
  ) {
    this.square = unitMesh(scene, 'mirrorSquare', false);
    this.disc = unitMesh(scene, 'mirrorDisc', true);
    this.pool = Array.from({ length: count }, () => null);
  }

  private reflection(i: number): Reflection {
    const known = this.pool[i];
    if (known) return known;
    const texture = new MirrorTexture(`mirror${i}`, { ratio: RATIO }, this.scene, false, Constants.TEXTURETYPE_HALF_FLOAT);
    texture.renderListPredicate = (mesh) => !this.overlays.has(mesh) && this.reflected(mesh);
    // Multisampled: at this size, unsmoothed edges show as stairs.
    texture.samples = 4;
    // A hint of the glass's own tint and loss.
    texture.level = 0.92;
    const material = new StandardMaterial(`mirror${i}`, this.scene);
    material.diffuseColor = Color3.Black();
    material.specularColor = Color3.Black();
    material.emissiveColor = new Color3(0.006, 0.008, 0.009);
    material.disableLighting = true;
    material.reflectionTexture = texture;
    this.onCreated?.(texture);
    return (this.pool[i] = { texture, material, used: 0 });
  }

  /** Lets go of the reflections no mirror has used for `KEEP_MS` (all of them with `now` Infinity). */
  private release(now: number): void {
    this.pool.forEach((r, i) => {
      if (!r || now - r.used < KEEP_MS || this.items.some((item) => item.slot === i)) return;
      for (const item of this.items) for (const mesh of item.meshes) if (mesh.material === r.material) mesh.material = null;
      r.material.dispose();
      r.texture.dispose();
      this.pool[i] = null;
    });
  }

  /** The mirrors of the world being shown (replaces the last ones). */
  set(instances: MirrorInstance[]): void {
    for (const item of this.items) for (const mesh of item.meshes) mesh.dispose();
    this.overlays.clear();
    this.items = [];
    // A world without mirrors needs none of the reflections.
    if (!instances.some((i) => i.surfaces.length)) this.release(Infinity);
    if (!this.pool.length) return;
    for (const instance of instances) {
      const meshes = instance.surfaces.map((s, i) => {
        const mesh = (s.round ? this.disc : this.square).clone(`mirror:${instance.index}:${i}`);
        mesh.setEnabled(false);
        mesh.isPickable = false;
        this.overlays.add(mesh);
        return mesh;
      });
      this.items.push({ instance, meshes, worlds: meshes.map(() => new Matrix()), slot: -1, matrix: new Matrix(), centre: new Vector3(), normal: new Vector3(), distance: 0 });
    }
  }

  /** Per frame: places the mirrors and gives the reflections to the nearest ones facing `camera`. */
  update(camera: Camera): void {
    if (!this.items.length) return;
    const now = performance.now();
    const eye = camera.globalPosition;
    const ranked = this.ranked;
    ranked.length = 0;
    for (const item of this.items) {
      const { matrices, index, surfaces } = item.instance;
      Matrix.FromArrayToRef(matrices, index * 16, item.matrix);
      surfaces[0].matrix.multiplyToRef(item.matrix, this.world);
      Vector3.TransformCoordinatesFromFloatsToRef(0, 0, 0, this.world, item.centre);
      Vector3.TransformNormalFromFloatsToRef(0, 0, 1, this.world, item.normal);
      const length = item.normal.length();
      item.distance = Vector3.Distance(eye, item.centre);
      // Hidden (scaled away), turned away from the camera, or too far to matter.
      if (length < 1e-6) continue;
      item.normal.scaleInPlace(1 / length);
      const facing = (eye.x - item.centre.x) * item.normal.x + (eye.y - item.centre.y) * item.normal.y + (eye.z - item.centre.z) * item.normal.z;
      if (facing > 0 && item.distance < RANGE) ranked.push(item);
    }
    ranked.sort((a, b) => a.distance - b.distance);
    const chosen = new Set(ranked.slice(0, this.pool.length));
    // Keep each chosen mirror's slot where it can (no flicker as the order changes).
    const free = new Set(this.pool.keys());
    for (const item of this.items) {
      if (chosen.has(item) && item.slot >= 0) free.delete(item.slot);
      else item.slot = -1;
    }
    for (const item of this.items) {
      if (chosen.has(item) && item.slot < 0) {
        item.slot = free.values().next().value!;
        free.delete(item.slot);
      }
      const on = item.slot >= 0;
      item.meshes.forEach((mesh, i) => {
        if (mesh.isEnabled() !== on) mesh.setEnabled(on);
        if (!on) return;
        const reflection = this.reflection(item.slot);
        reflection.used = now;
        const { material, texture } = reflection;
        if (mesh.material !== material) mesh.material = material;
        item.instance.surfaces[i].matrix.multiplyToRef(item.matrix, item.worlds[i]);
        mesh.freezeWorldMatrix(item.worlds[i]);
        // The plane faces into the glass: what is behind it is clipped from the reflection.
        if (i === 0) Plane.FromPositionAndNormalToRef(item.centre, item.normal.negateToRef(this.back), texture.mirrorPlane);
      });
    }
    this.release(now);
  }

  /** How many reflections may be live at once, the render targets of those reflecting now, mirrors in the world and those reflecting now (benchmark reports). */
  get capacity(): number {
    return this.pool.length;
  }

  get reflecting(): MirrorTexture[] {
    return this.items.filter((item) => item.slot >= 0).map((item) => this.pool[item.slot]?.texture).filter((t): t is MirrorTexture => !!t);
  }

  /** The reflections' render targets made now. */
  get textures(): MirrorTexture[] {
    return this.pool.filter((r): r is Reflection => !!r).map((r) => r.texture);
  }

  get count(): number {
    return this.items.length;
  }

  get live(): number {
    return this.items.filter((item) => item.slot >= 0).length;
  }

  dispose(): void {
    this.set([]);
    this.release(Infinity);
    this.square.dispose();
    this.disc.dispose();
  }
}

/** The unit square, or the unit-diameter disc, in the xy plane facing +z (never drawn itself; its clones are). */
function unitMesh(scene: Scene, name: string, round: boolean): Mesh {
  const mesh = new Mesh(name, scene);
  const positions: number[] = [];
  const indices: number[] = [];
  if (round) {
    const n = 40;
    positions.push(0, 0, 0);
    for (let i = 0; i <= n; i++) {
      const a = (i / n) * Math.PI * 2;
      positions.push(Math.cos(a) * 0.5, Math.sin(a) * 0.5, 0);
      if (i > 0) indices.push(0, i + 1, i);
    }
  } else {
    positions.push(-0.5, -0.5, 0, 0.5, -0.5, 0, 0.5, 0.5, 0, -0.5, 0.5, 0);
    indices.push(0, 2, 1, 0, 3, 2);
  }
  const normals = positions.map((_, i) => (i % 3 === 2 ? 1 : 0));
  const data = new VertexData();
  data.positions = positions;
  data.indices = indices;
  data.normals = normals;
  data.applyToMesh(mesh);
  mesh.setEnabled(false);
  return mesh;
}
