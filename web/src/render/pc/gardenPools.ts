/**
 * Pools of lamplight on the garden's ground on PlayCanvas (ported from the Babylon renderer, removed: see git history up to f15b839b):
 * under a lamp post, a street lamp, a fire pit or a bulkhead on the outside wall, as the street
 * draws them under its own lamps. The lamps' lights light what stands in the garden, but not the
 * ground (the landscape is kept out of the lamps). A pool is a disc on the ground, added to what's
 * there, unlit and without fog: one draw for all of them (a quad each, its brightness in its
 * vertex colour), none by day.
 */

import * as pc from './engine';
import { GAME_FRONT_FACE } from './materials';
import { drawMesh } from './meshes';
import { gradientTexture } from './street';
import type { RGB } from '../shared/environment';
import type { GardenPool } from '../shared/gardenPools';

export { gardenPool, type GardenPool } from '../shared/gardenPools';

/** A hair over the ground, so the pool isn't lost in it. */
const LIFT = 0.03;

export class GardenPools {
  private readonly material: pc.StandardMaterial;
  private readonly texture: pc.Texture;
  private entity: pc.Entity | null = null;
  private mesh: pc.Mesh | null = null;
  private pools: GardenPool[] = [];
  /** The pools' vertex colours (four vertices each), rewritten as they brighten and dim. */
  private colors = new Float32Array(0);
  private lit = false;

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
  ) {
    this.texture = gradientTexture(app.graphicsDevice, 'gardenLightPool', [
      [0, 255],
      [0.3, 150],
      [0.65, 40],
      [1, 0],
    ]);
    const m = (this.material = new pc.StandardMaterial());
    m.name = 'gardenLightPools';
    m.frontFace = GAME_FRONT_FACE;
    m.useLighting = false;
    m.useSkybox = false;
    m.useFog = false;
    m.diffuse.set(0, 0, 0);
    // The vertex colour times the gradient (as opacity), added: a pool switched off adds nothing. (Taken
    // as sRGB, linearised: the Babylon renderer's standard material's output was, the lamps' linear
    // colour times the strength with it.)
    m.emissive.set(1, 1, 1);
    m.emissiveVertexColor = true;
    (m as pc.StandardMaterial & { vertexColorGamma: boolean }).vertexColorGamma = true;
    m.opacityMap = this.texture;
    m.opacityMapChannel = 'r';
    m.blendType = pc.BLEND_ADDITIVEALPHA;
    m.depthWrite = false;
    m.slopeDepthBias = -4;
    m.update();
  }

  /** The viewed lot's garden lamps (their pools' places), on every change of it. */
  set(pools: GardenPool[]): void {
    this.pools = pools;
    this.entity?.destroy();
    this.entity = this.mesh = null;
    this.colors = new Float32Array(pools.length * 16);
    this.lit = false;
    if (!pools.length) return;
    const n = pools.length * 4;
    const positions = new Float32Array(n * 3);
    const normals = new Float32Array(n * 3);
    const uvs = new Float32Array(n * 2);
    const indices = new Uint16Array(pools.length * 6);
    pools.forEach((p, i) => {
      const corners = [
        [-1, -1],
        [1, -1],
        [1, 1],
        [-1, 1],
      ];
      corners.forEach(([u, v], k) => {
        const j = i * 4 + k;
        positions.set([p.x + u * p.radius, p.y + LIFT, p.z + v * p.radius], j * 3);
        normals.set([0, 1, 0], j * 3);
        uvs.set([(u + 1) / 2, (v + 1) / 2], j * 2);
      });
      // Facing up, wound as `Geo` winds an upward face.
      indices.set([i * 4, i * 4 + 1, i * 4 + 2, i * 4, i * 4 + 2, i * 4 + 3], i * 6);
    });
    const mesh = (this.mesh = new pc.Mesh(this.app.graphicsDevice));
    mesh.clear(true, false);
    mesh.setPositions(positions);
    mesh.setNormals(normals);
    mesh.setUvs(0, uvs);
    mesh.setColors(this.colors);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    this.entity = drawMesh(this.parent, 'gardenPools', mesh, this.material, { receiveShadows: false });
    this.entity.enabled = false;
  }

  /**
   * How bright they are now: `level` the lamps' night level (0 by day), `on` each lamp's
   * switch (eased, 0..1), `color` the lamps' (linear). Off altogether when none adds anything.
   */
  update(level: number, on: (lamp: number | undefined) => number, color: RGB): void {
    let lit = false;
    const c = this.colors;
    for (let i = 0; i < this.pools.length; i++) {
      const p = this.pools[i];
      const f = level * on(p.lamp) * p.strength;
      for (let k = 0; k < 4; k++) {
        const o = (i * 4 + k) * 4;
        c[o] = color.r * f;
        c[o + 1] = color.g * f;
        c[o + 2] = color.b * f;
        c[o + 3] = 1;
      }
      lit ||= f > 0;
    }
    if (this.mesh && (lit || this.lit)) {
      this.mesh.setColors(c);
      this.mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    }
    this.lit = lit;
    if (this.entity && this.entity.enabled !== lit) this.entity.enabled = lit;
  }

  destroy(): void {
    this.entity?.destroy();
    this.entity = this.mesh = null;
    this.material.destroy();
    this.texture.destroy();
  }
}
