/**
 * The sun's shadow map in the game. It covers a square round the viewed lot (see
 * `BabylonRenderer.applyLighting`), but Babylon draws every caster of its list into it, wherever
 * it stands: the street's far houses, the woods round the town. Only casters whose bounds reach
 * into the square are drawn; the rest would add no texel.
 *
 * And the square stays put while the camera turns, so the map is drawn again only when something
 * in it changed (`moved`): the sun's step, a Sim's pose, a thing placed or hidden, the walls. While
 * the game is paused (Buy and Build mode too) it isn't drawn at all.
 *
 * A Sim's every breath would draw it anew, so the poses (and the street's moving doors) are
 * throttled to the quality's `shadowRate` (0: every change; `?shadowrate=<Hz>` sets it).
 */

import { Frustum, Plane, RenderTargetTexture, type AbstractMesh, type Scene, type ShadowGenerator } from './core';

/** Metres a caster's bounds may lie past the square's sides and still be drawn (bounds that lag behind a thing springing into place). */
const MARGIN = 1;

export class SunShadows {
  private readonly planes = Array.from({ length: 6 }, () => new Plane(0, 0, 0, 0));
  private readonly inside: AbstractMesh[] = [];
  private readonly map: RenderTargetTexture;
  /** The menus' town: its map is fitted round all its casters, and drawn every frame. */
  private overview = false;
  /** Milliseconds between pose-driven redraws (0: every change), and when the last one was made. */
  private readonly interval: number;
  private lastDraw = -Infinity;
  private pending = false;

  constructor(
    private readonly generator: ShadowGenerator,
    scene: Scene,
    rate: number,
  ) {
    this.map = generator.getShadowMap()!;
    this.map.refreshRate = RenderTargetTexture.REFRESHRATE_RENDER_ONCE;
    this.map.getCustomRenderList = (_pass, casters, count) => (this.overview || !casters ? null : this.cull(casters, count));
    this.interval = rate > 0 ? 1000 / Math.min(240, rate) : 0;
    // A pose redraw held back waits for the interval to pass, then draws on the next frame.
    scene.onBeforeRenderObservable.add(() => {
      if (!this.pending || performance.now() - this.lastDraw < this.interval) return;
      this.pending = false;
      this.lastDraw = performance.now();
      this.map.resetRefreshCounter();
    });
  }

  /** The menus' town (every caster, every frame) or the game. */
  setOverview(on: boolean): void {
    this.overview = on;
    this.map.refreshRate = on ? 1 : RenderTargetTexture.REFRESHRATE_RENDER_ONCE;
    this.moved();
  }

  /**
   * Something in the map changed: it is drawn again in the next frame. `urgent` changes (the sun,
   * the walls, a thing placed) draw at once; a Sim's pose may wait for the shadow rate.
   */
  moved(urgent = true): void {
    const now = performance.now();
    if (this.overview || this.interval <= 0 || urgent || now - this.lastDraw >= this.interval) {
      this.pending = false;
      this.lastDraw = now;
      this.map.resetRefreshCounter();
    } else {
      this.pending = true;
    }
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
