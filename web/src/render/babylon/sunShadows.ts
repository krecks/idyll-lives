/**
 * The sun's shadow map in the game. It covers a square round the viewed lot (see
 * `BabylonRenderer.applyLighting`), but Babylon draws every caster of its list into it, wherever
 * it stands: the street's far houses, the woods round the town. Only casters whose bounds reach
 * into the square are drawn; the rest would add no texel.
 */

import { Frustum, Plane, type AbstractMesh, type ShadowGenerator } from './core';

/** Metres a caster's bounds may lie past the square's sides and still be drawn (bounds that lag behind a thing springing into place). */
const MARGIN = 1;

export class SunShadows {
  private readonly planes = Array.from({ length: 6 }, () => new Plane(0, 0, 0, 0));
  private readonly inside: AbstractMesh[] = [];
  /** The menus' town: its map is fitted round all its casters, so every one is drawn. */
  private all = false;

  constructor(private readonly generator: ShadowGenerator) {
    generator.getShadowMap()!.getCustomRenderList = (_pass, casters, count) => (this.all || !casters ? null : this.cull(casters, count));
  }

  /** Draws every caster (the overview) or only those reaching into the map (the game). */
  drawAll(on: boolean): void {
    this.all = on;
  }

  private cull(casters: readonly AbstractMesh[], count: number): AbstractMesh[] {
    // (The light's transform for this frame is made just before the list is asked for.)
    Frustum.GetPlanesToRef(this.generator.getTransformMatrix(), this.planes);
    const inside = this.inside;
    inside.length = 0;
    for (let i = 0; i < count; i++) {
      const mesh = casters[i];
      // Meshes the camera never culls (Sims: posed on the GPU, their bounds say nothing) are always drawn.
      if (mesh.alwaysSelectAsActiveMesh || this.reaches(mesh)) inside.push(mesh);
    }
    return inside;
  }

  /** Whether a caster's bounds reach in past the map's four sides (its depth range is left to the GPU). */
  private reaches(mesh: AbstractMesh): boolean {
    const box = mesh.getBoundingInfo().boundingBox;
    const lo = box.minimumWorld;
    const hi = box.maximumWorld;
    for (let p = 2; p < 6; p++) {
      const { normal: n, d } = this.planes[p];
      const far = n.x * (n.x >= 0 ? hi.x : lo.x) + n.y * (n.y >= 0 ? hi.y : lo.y) + n.z * (n.z >= 0 ? hi.z : lo.z) + d;
      if (far < -MARGIN) return false;
    }
    return true;
  }
}
