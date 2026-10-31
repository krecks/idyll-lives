/**
 * The selection marker over the selected resident: `model.marker`, a small soft four-point sparkle
 * star, unlit, its faces shaded by vertex colours (a hot core, deeper towards the ray tips), with a
 * thin deeper-toned outline (its back faces pushed out along their normals: an inverted hull) and a
 * faint additive glow disc behind it. All three face the camera, float clear of the head with a
 * slow bob and a breathing pulse (none with reduced motion), grow a little with the camera's
 * distance, and take the resident's mood colour (`LOOK.marker`, by `EMOTION_MOOD`).
 *
 * One copy each, drawn instanced from a matrix (game space, under the world root); the colour is a
 * per-draw `material_emissive` (linear), so nothing is uploaded but the three matrices. Without its
 * file there is no star (the glow stays) until the file comes. No allocations per frame.
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { GAME_FRONT_FACE } from './materials';
import { buildModel, onModelFileArrived, uploadData } from './models';
import type { AssetRegistry } from '../../assets/registry';
import { LOOK } from '../look';
import { toLinear } from '../shared/math';

/** Marker colour slot (0 = great .. 4 = bad) per emotion id; no emotion = slot 1. */
const EMOTION_MOOD: Record<string, number> = { happy: 0, flirty: 0, confident: 0, focused: 0, inspired: 0, energized: 0, relaxed: 0, embarrassed: 3, sad: 3, tense: 3, angry: 4 };

/** How far the outline stands off the star (m, along its normals). */
const OUTLINE = 0.009;
/**
 * The star's brightest vertex shade: PlayCanvas clamps an emissive vertex colour to 1, so the shades
 * are stored divided by it and the emission scaled up by it.
 */
const HOT = 1.45;

export class SelectionMarker {
  private readonly device: pc.GraphicsDevice;
  private readonly starMaterial: pc.StandardMaterial;
  private readonly outlineMaterial: pc.StandardMaterial;
  private star: Instances[] = [];
  private outline: Instances[] = [];
  private readonly halo: Instances;
  private readonly haloTexture: pc.Texture;
  private readonly haloMaterial: pc.StandardMaterial;
  /** The files the star was built without: it's built again when one comes. */
  private failed: readonly string[] = [];
  private builds: Promise<void> = Promise.resolve();
  private readonly unlisten: () => void;
  private destroyed = false;
  private shown = false;
  /** Mood colours (linear, a little hot), and the colour slot per emotion code (index + 1; 0 = none). */
  private readonly palette = LOOK.marker.map((hex) => [toLinear(parseInt(hex.slice(1, 3), 16) / 255) * 1.2, toLinear(parseInt(hex.slice(3, 5), 16) / 255) * 1.2, toLinear(parseInt(hex.slice(5, 7), 16) / 255) * 1.2]);
  private mood: number[] = [1];
  private readonly matrix = new Float32Array(16);
  private readonly haloMatrix = new Float32Array(16);
  private readonly starColor = new Float32Array(3);
  private readonly outlineColor = new Float32Array(3);
  private readonly haloColor = new Float32Array(3);
  private readonly axes = [new pc.Vec3(), new pc.Vec3(), new pc.Vec3()];

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
    private readonly assets: AssetRegistry,
  ) {
    this.device = app.graphicsDevice;
    const unlit = (name: string, cull: number) => {
      const m = new pc.StandardMaterial();
      m.name = name;
      m.useLighting = false;
      m.useSkybox = false;
      m.useFog = false;
      m.diffuse.set(0, 0, 0);
      m.emissive.set(1, 1, 1);
      m.emissiveVertexColor = true;
      m.frontFace = GAME_FRONT_FACE;
      m.cull = cull;
      m.update();
      return m;
    };
    this.starMaterial = unlit('marker', pc.CULLFACE_BACK);
    this.starMaterial.emissiveIntensity = HOT;
    this.starMaterial.update();
    // (Only the back faces: the hull round the star.)
    this.outlineMaterial = unlit('markerOutline', pc.CULLFACE_FRONT);

    // Glow: a radial gradient disc, added on top of the scene (reads at night, subtle by day).
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = 64;
    const ctx = canvas.getContext('2d')!;
    const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
    g.addColorStop(0, 'rgba(255,255,255,0.9)');
    g.addColorStop(0.25, 'rgba(255,255,255,0.45)');
    g.addColorStop(0.6, 'rgba(255,255,255,0.12)');
    g.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, 64, 64);
    this.haloTexture = new pc.Texture(this.device, { name: 'markerGlow', width: 64, height: 64, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
    this.haloTexture.setSource(canvas);
    const h = (this.haloMaterial = new pc.StandardMaterial());
    h.name = 'markerHalo';
    h.useLighting = false;
    h.useSkybox = false;
    h.useFog = false;
    h.diffuse.set(0, 0, 0);
    h.emissive.set(1, 1, 1);
    h.opacityMap = this.haloTexture;
    h.opacityMapChannel = 'a';
    // (The colour times its alpha, added.)
    h.blendType = pc.BLEND_ADDITIVEALPHA;
    h.depthWrite = false;
    h.cull = pc.CULLFACE_NONE;
    h.update();
    // A 1 m plane (in x and y, facing -z).
    const plane = new pc.Mesh(this.device);
    plane.setPositions([-0.5, -0.5, 0, 0.5, -0.5, 0, 0.5, 0.5, 0, -0.5, 0.5, 0]);
    plane.setNormals([0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1]);
    plane.setUvs(0, [0, 0, 1, 0, 1, 1, 0, 1]);
    plane.setIndices([0, 1, 2, 0, 2, 3]);
    plane.update(pc.PRIMITIVE_TRIANGLES);
    this.halo = this.draw('markerHalo', plane, h, this.haloColor);
    plane.decRefCount();

    // Listened for before it's built: an app nobody listens to doesn't try a failed file again.
    this.unlisten = onModelFileArrived(app, (url) => {
      this.builds = this.builds.then(() => (this.failed.includes(url) ? this.build() : undefined)).catch((err) => console.error(`[render] building the marker again with ${url} failed`, err));
    });
    this.builds = this.build().catch((err) => console.error('[render] the selection marker failed', err));
  }

  /** The marker colour slot per emotion code (content `emotions`, in snapshot order). */
  setEmotions(ids: readonly string[]): void {
    this.mood = [1, ...ids.map((id) => EMOTION_MOOD[id] ?? 1)];
  }

  /** One copy of `mesh`, coloured by `color` (linear, per draw), hidden. */
  private draw(name: string, mesh: pc.Mesh, material: pc.Material, color: Float32Array): Instances {
    const d = new Instances(this.device, this.parent, mesh, material, { name, castShadows: false, receiveShadows: false, cull: false });
    d.meshInstance.setParameter('material_emissive', color);
    d.setCount(1);
    d.enabled = false;
    return d;
  }

  /** The star and its outline from `model.marker`, in place of the last ones; none without the file, until it comes. */
  private async build(): Promise<void> {
    const template = await buildModel(this.app, 'model.marker', this.assets.get('model.marker', 'model'), [1, 1]);
    const made: pc.Mesh[] = [];
    const star: Instances[] = [];
    const outline: Instances[] = [];
    if (!this.destroyed) {
      for (const m of template.meshes) {
        const n = m.mesh.vertexBuffer?.getNumVertices() ?? 0;
        const positions = new Float32Array(n * 3);
        const normals = new Float32Array(n * 3);
        const indices: number[] = [];
        m.mesh.getPositions(positions);
        m.mesh.getNormals(normals);
        m.mesh.getIndices(indices);
        // Smooth, pillowy shading: a hot core that blooms, brightest on the faces turned to the viewer, softly deeper towards the ray tips.
        const colors = new Float32Array(n * 4);
        for (let i = 0; i < n; i++) {
          const r = Math.hypot(positions[i * 3], positions[i * 3 + 1]);
          const face = Math.abs(normals[i * 3 + 2]) * 0.75 + Math.max(0, normals[i * 3 + 1]) * 0.25;
          const k = ((0.7 + 0.3 * face) * (1.45 - 0.6 * Math.min(1, r / 0.16))) / HOT;
          colors.set([k, k, k, 1], i * 4);
        }
        const tris = Uint32Array.from(indices);
        const body = uploadData(this.device, { positions, normals, colors, indices: tris });
        const hull = uploadData(this.device, { positions: positions.map((p, i) => p + normals[i] * OUTLINE), normals: normals.slice(), colors: new Float32Array(n * 4).fill(1), indices: tris.slice() });
        made.push(body, hull);
        star.push(this.draw('marker', body, this.starMaterial, this.starColor));
        outline.push(this.draw('markerOutline', hull, this.outlineMaterial, this.outlineColor));
      }
    }
    for (const m of template.meshes) m.mesh.destroy();
    for (const g of template.glows ?? []) g.mesh.mesh.destroy();
    // (The draws hold their meshes: destroying a draw lets go of its mesh.)
    for (const m of made) m.decRefCount();
    if (this.destroyed) return;
    for (const d of [...this.star, ...this.outline]) d.destroy();
    this.star = star;
    this.outline = outline;
    this.failed = template.failed ?? [];
    this.shown = false;
  }

  /**
   * Per drawn frame, after the camera took its input: over the selected resident's head top (`x`, `headY`, `z`,
   * game space) facing `camera`, in the colour of `emotion` (snapshot code); null hides it. `now` (ms) drives the
   * bob and the breathing (none when `still`).
   */
  place(head: { x: number; y: number; z: number } | null, emotion: number, camera: pc.Entity, now: number, still: boolean): void {
    if (!head) {
      if (this.shown) this.show(false);
      return;
    }
    const { x, y: headY, z } = head;
    const t = still ? 0 : now * 0.001;
    const breath = Math.sin(t * 2.4);
    const eye = camera.getPosition();
    // (The camera in game space: z turned round.)
    const dist = Math.hypot(eye.x - x, eye.y - headY, -eye.z - z);
    const grow = 1 + Math.min(2.2, Math.max(0, dist - 5) * 0.06);
    const s = grow * (1 + breath * 0.05);
    // Float clear of the head: the lower ray's tip stays ~8 cm above it at any size.
    const by = headY + 0.07 + 0.19 * grow + Math.sin(t * 1.5) * 0.03;
    // The camera's right, up and forward (the way it looks) in game space.
    const [right, up, forward] = this.axes;
    right.copy(camera.right);
    up.copy(camera.up);
    forward.copy(camera.forward);
    const hs = 0.62 * s * (1 + breath * 0.06);
    const m = this.matrix;
    const hm = this.haloMatrix;
    for (let r = 0; r < 3; r++) {
      const a = this.axes[r];
      m[r * 4] = a.x * s;
      m[r * 4 + 1] = a.y * s;
      m[r * 4 + 2] = -a.z * s;
      hm[r * 4] = a.x * hs;
      hm[r * 4 + 1] = a.y * hs;
      hm[r * 4 + 2] = -a.z * hs;
      m[r * 4 + 3] = hm[r * 4 + 3] = 0;
    }
    m[12] = x;
    m[13] = by;
    m[14] = z;
    m[15] = hm[15] = 1;
    // The halo nudged just behind the star (away from the camera).
    hm[12] = x + forward.x * 0.04;
    hm[13] = by + forward.y * 0.04;
    hm[14] = z - forward.z * 0.04;

    const c = this.palette[this.mood[emotion | 0] ?? 1] ?? this.palette[0];
    if (c) {
      const glow = 1 + breath * 0.12;
      this.starColor[0] = c[0] * glow;
      this.starColor[1] = c[1] * glow;
      this.starColor[2] = c[2] * glow;
      // Outline: the mood colour, much deeper (amber -> warm brown, periwinkle -> ink blue).
      this.outlineColor[0] = c[0] * 0.16;
      this.outlineColor[1] = c[1] * 0.13;
      this.outlineColor[2] = c[2] * 0.12;
      const h = 0.22 * (1 + breath * 0.25);
      this.haloColor[0] = c[0] * h;
      this.haloColor[1] = c[1] * h;
      this.haloColor[2] = c[2] * h;
    }
    for (const d of this.star) this.write(d, m);
    for (const d of this.outline) this.write(d, m);
    this.write(this.halo, hm);
    if (!this.shown) this.show(true);
  }

  private write(d: Instances, m: Float32Array): void {
    d.setMatrix(0, m);
    d.update(false);
  }

  private show(on: boolean): void {
    this.shown = on;
    for (const d of [this.halo, ...this.star, ...this.outline]) d.enabled = on;
  }

  destroy(): void {
    this.destroyed = true;
    this.unlisten();
    for (const d of [this.halo, ...this.star, ...this.outline]) d.destroy();
    this.star = [];
    this.outline = [];
    this.haloTexture.destroy();
    for (const m of [this.starMaterial, this.outlineMaterial, this.haloMaterial]) m.destroy();
  }
}
