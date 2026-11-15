/**
 * The land brush's cursor (`Renderer.setLandBrush`): two rings draped over the land round where the brush is, its
 * reach and (where it works at full strength) its inner ring, each `SEGMENTS` segments of a flat band
 * `CURSOR_Y` m over the ground, drawn unlit and clear of the fog in the colour of what the brush does (red where
 * it can't). Made again as it moves (a few hundred vertices: its buffer written in place). An entity under the
 * world root (game space), on the layer the camera in use draws.
 */

import * as pc from './engine';
import { GAME_FRONT_FACE } from './materials';
import type { LandCursor } from '../types';

/** Segments a ring. */
const SEGMENTS = 64;
/** How high over the ground the rings hang (m). */
const CURSOR_Y = 0.15;
/** The bands' width: this share of the brush's reach, between these (m). */
const BAND = { share: 0.025, min: 0.25, max: 3 };
/** The colours (sRGB) of what the brush does. */
const TONES: Record<LandCursor['tone'], string> = {
  up: '#FFD166',
  down: '#B39DDB',
  water: '#4FC3F7',
  cover: '#8BD48B',
  bad: '#EF5350',
};

/** Vertices of a ring's band: its inner and outer edge, the first pair again at the end. */
const RING_VERTICES = (SEGMENTS + 1) * 2;

export class LandCursorDraw {
  private readonly entity: pc.Entity;
  private readonly mesh: pc.Mesh;
  private readonly materials = new Map<LandCursor['tone'], pc.StandardMaterial>();
  private readonly positions = new Float32Array(RING_VERTICES * 2 * 3);
  private readonly mi: pc.MeshInstance;

  constructor(app: pc.AppBase, parent: pc.Entity) {
    const mesh = (this.mesh = new pc.Mesh(app.graphicsDevice));
    const indices = new Uint16Array(SEGMENTS * 6 * 2);
    for (let ring = 0, k = 0; ring < 2; ring++) {
      const base = ring * RING_VERTICES;
      for (let s = 0; s < SEGMENTS; s++) {
        const [a, b, c, d] = [base + s * 2, base + s * 2 + 1, base + s * 2 + 2, base + s * 2 + 3];
        indices.set([a, c, b, b, c, d], k);
        k += 6;
      }
    }
    mesh.setPositions(this.positions);
    mesh.setNormals(new Float32Array(this.positions.length).map((_, q) => (q % 3 === 1 ? 1 : 0)));
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    this.mi = new pc.MeshInstance(mesh, this.material('up'));
    // (Its box is made again as it moves; never left out by an old one.)
    this.mi.cull = false;
    this.entity = new pc.Entity('landCursor', app);
    this.entity.addComponent('render', { meshInstances: [this.mi], castShadows: false, receiveShadows: false });
    this.entity.enabled = false;
    parent.addChild(this.entity);
  }

  /** The unlit, fogless material of a tone, made once. */
  private material(tone: LandCursor['tone']): pc.StandardMaterial {
    let mat = this.materials.get(tone);
    if (mat) return mat;
    mat = new pc.StandardMaterial();
    mat.name = `landCursor.${tone}`;
    mat.frontFace = GAME_FRONT_FACE;
    mat.useLighting = false;
    mat.useFog = false;
    mat.useSkybox = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.fromString(TONES[tone]);
    mat.cull = pc.CULLFACE_NONE;
    mat.update();
    this.materials.set(tone, mat);
    return mat;
  }

  /**
   * Shows the cursor round (`cursor.x`, `cursor.z`), draped over `height`, on `layer` (null hides it). The inner
   * ring is left out where it is no smaller than the reach or not above 0.
   */
  set(cursor: LandCursor | null, height: (x: number, z: number) => number, layer: number): void {
    if (!cursor) {
      this.entity.enabled = false;
      return;
    }
    const band = Math.min(BAND.max, Math.max(BAND.min, cursor.radius * BAND.share));
    const p = this.positions;
    const inner = cursor.inner > 0 && cursor.inner < cursor.radius - band ? cursor.inner : 0;
    for (const [ring, r] of [
      [0, cursor.radius],
      [1, inner],
    ] as const) {
      for (let s = 0; s <= SEGMENTS; s++) {
        const a = (s / SEGMENTS) * Math.PI * 2;
        const [cx, sz] = [Math.cos(a), Math.sin(a)];
        for (const [side, rr] of [
          [0, Math.max(0, r - band / 2)],
          [1, r + band / 2],
        ] as const) {
          const o = ((ring * RING_VERTICES + s * 2 + side) * 3) | 0;
          // (No inner ring: its band folded to a point under the middle, drawing nothing.)
          const [x, z] = r > 0 ? [cursor.x + cx * rr, cursor.z + sz * rr] : [cursor.x, cursor.z];
          p[o] = x;
          p[o + 1] = height(x, z) + CURSOR_Y;
          p[o + 2] = z;
        }
      }
    }
    this.mesh.setPositions(p);
    this.mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    const mat = this.material(cursor.tone);
    if (this.mi.material !== mat) this.mi.material = mat;
    const render = this.entity.render!;
    if (render.layers.length !== 1 || render.layers[0] !== layer) render.layers = [layer];
    this.entity.enabled = true;
  }

  destroy(): void {
    this.entity.destroy();
    for (const m of this.materials.values()) m.destroy();
  }
}
