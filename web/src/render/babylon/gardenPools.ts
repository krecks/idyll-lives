/**
 * Pools of lamplight on the garden's ground: under a lamp post, a street lamp, a fire pit or a
 * bulkhead on the outside wall, as the street draws them under its own lamps (`street.ts`).
 * The lamps' lights light what stands in the garden, but not the ground: the landscape is kept
 * out of the lamp cluster (lit by it, the ground alone cost 1.1–1.5 ms of GPU a frame at night on
 * Medium), and Low's four forward lights go to the rooms first. A pool is a disc on the ground,
 * added to what's there, unlit and without fog: one draw for all of them, none by day.
 */

import { Color3, Constants, DynamicTexture, Matrix, MeshBuilder, StandardMaterial, type Mesh, type Scene } from './core';
import type { GardenPool } from '../shared/gardenPools';

export { gardenPool, type GardenPool } from '../shared/gardenPools';

/** A hair over the ground, so the pool isn't lost in it. */
const LIFT = 0.03;

export class GardenPools {
  private readonly mesh: Mesh;
  private pools: GardenPool[] = [];
  private colors = new Float32Array(0);

  constructor(scene: Scene) {
    const tex = new DynamicTexture('gardenLightPool', 128, scene, true);
    const ctx = tex.getContext();
    const g = ctx.createRadialGradient(64, 64, 0, 64, 64, 64);
    g.addColorStop(0, 'rgb(255,255,255)');
    g.addColorStop(0.3, 'rgb(150,150,150)');
    g.addColorStop(0.65, 'rgb(40,40,40)');
    g.addColorStop(1, 'rgb(0,0,0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, 128, 128);
    tex.update();
    tex.getAlphaFromRGB = true;
    const material = new StandardMaterial('gardenLightPools', scene);
    material.disableLighting = true;
    material.diffuseColor = Color3.Black();
    material.specularColor = Color3.Black();
    // The instance colour times the gradient (as opacity), added: a pool switched off adds nothing.
    material.emissiveColor = Color3.White();
    material.opacityTexture = tex;
    material.alphaMode = Constants.ALPHA_ADD;
    material.disableDepthWrite = true;
    material.fogEnabled = false;
    material.zOffset = -4;
    this.mesh = MeshBuilder.CreateGround('gardenPools', { width: 1, height: 1 }, scene);
    this.mesh.material = material;
    this.mesh.isPickable = false;
    this.mesh.alwaysSelectAsActiveMesh = true;
    this.mesh.setEnabled(false);
  }

  /** The viewed lot's garden lamps (their pools' places), on every change of it. */
  set(pools: GardenPool[]): void {
    this.pools = pools;
    const matrices = new Float32Array(pools.length * 16);
    pools.forEach((p, i) => Matrix.Scaling(p.radius * 2, 1, p.radius * 2).multiply(Matrix.Translation(p.x, p.y + LIFT, p.z)).copyToArray(matrices, i * 16));
    this.colors = new Float32Array(pools.length * 4);
    this.mesh.thinInstanceSetBuffer('matrix', pools.length ? matrices : null, 16, true);
    this.mesh.thinInstanceSetBuffer('color', pools.length ? this.colors : null, 4, false);
  }

  /**
   * How bright they are now: `level` the lamps' night level (0 by day), `on` each lamp's
   * switch (eased, 0..1), `color` the lamps' (linear). Off altogether when none adds anything.
   */
  update(level: number, on: (lamp: number | undefined) => number, color: Color3): void {
    let lit = false;
    this.pools.forEach((p, i) => {
      const f = level * on(p.lamp) * p.strength;
      this.colors[i * 4] = color.r * f;
      this.colors[i * 4 + 1] = color.g * f;
      this.colors[i * 4 + 2] = color.b * f;
      this.colors[i * 4 + 3] = 1;
      lit ||= f > 0;
    });
    if (this.pools.length) this.mesh.thinInstanceBufferUpdated('color');
    if (this.mesh.isEnabled() !== lit) this.mesh.setEnabled(lit);
  }
}
