/**
 * Placing things in build and buy mode on PlayCanvas (the Babylon renderer's `setPlacementGhost`,
 * `setHeldObject`, `buildEffect` and what they drive), for `PlayCanvasRenderer.ts`:
 *
 * - The thing in hand (`placementGhost.ts`), and, while it's something for a wall, the things on
 *   walls shown even with their walls down (`cutWallItems`: what's in its way shows).
 * - A placed object picked up (`setHeldObject`): hidden where it stands, with what stands on its
 *   top, until it's put down (moved: until the next world shows it in its new place, or briefly
 *   if the move is refused). The chairs residents pull up leave it be (`held`).
 * - The effects (`buildFx.ts`): rings, sparkles and dust, and the pop of what was placed.
 *
 * The renderer calls `worldBuilding` as a world build starts and `objectsBuilt` once its objects
 * are drawn (their buffers are new: the hand and the pops let go of the old ones), and `frame`
 * each game frame.
 */

import type * as pc from './engine';
import { BuildEffects } from './buildFx';
import type { LotContents } from './lotContents';
import { PlacementGhostView } from './placementGhost';
import type { PopTarget } from './objectMotion';
import type { Lighting } from '../shared/environment';
import { WALL_HEIGHT } from '../shared/walls';
import type { BuildEffect, PlacementGhost, RendererDeps } from '../types';

/** What placing needs of the renderer. */
export interface PlacingView {
  /** The storey a lot row is on, and the first row of storey `k`. */
  storeyOf(z: number): number;
  rowOf(k: number): number;
  /** The menus' town is shown (no effects there). */
  town(): boolean;
  /** The sun's shadows draw again. */
  shadowsMoved(): void;
}

export class Placing {
  private readonly ghost: PlacementGhostView;
  private readonly fx: BuildEffects;
  /** The thing for a wall in hand: the things on walls show. */
  private ghostOnWall = false;
  /** The cutaway as last given (`cutWallItems`). */
  private cutBits = 0;
  private wallsDown = false;
  /** The object in hand (`setHeldObject`), and what of it is hidden: each instance with its matrix to show it again. */
  private heldId: number | null = null;
  private hidden: { id: number; saved: { target: PopTarget; matrix: Float32Array }[] } | null = null;
  private release: ReturnType<typeof setTimeout> | null = null;

  constructor(
    app: pc.AppBase,
    root: pc.Entity,
    storeyLayer: pc.Entity,
    camera: pc.CameraComponent,
    deps: RendererDeps,
    private readonly lot: LotContents,
    private readonly view: PlacingView,
  ) {
    this.ghost = new PlacementGhostView(app, storeyLayer, camera, deps);
    this.fx = new BuildEffects(app, root, camera);
  }

  set reducedMotion(on: boolean) {
    this.ghost.reducedMotion = on;
    this.fx.reducedMotion = on;
  }

  /** The object in hand: the chairs residents pull up leave it where it is (`LotContents.pullChairs`). */
  get held(): number | null {
    return this.hidden?.id ?? null;
  }

  setPlacementGhost(ghost: PlacementGhost | null): void {
    // Something for a wall in hand: the things on walls show, so what's in its way does.
    const onWall = !!ghost?.wall;
    if (onWall !== this.ghostOnWall) {
      this.ghostOnWall = onWall;
      this.applyWallItems();
    }
    this.ghost.set(ghost);
  }

  /**
   * Things on a wall go with their wall (`LotContents.cutWallItems`), but the one in hand, and none
   * while something for a wall is held: then they show, as what may be in its way.
   */
  cutWallItems(bits: number, down: boolean): void {
    this.cutBits = bits;
    this.wallsDown = down;
    this.applyWallItems();
  }

  setHeldObject(id: number | null, moved = false): void {
    if (id === this.heldId) return;
    if (this.release) clearTimeout(this.release);
    this.release = null;
    this.heldId = id;
    if (id === null && moved) {
      // Put down elsewhere: the next world shows it there; a refused move shows it here again.
      this.release = setTimeout(() => ((this.release = null), this.showHeld()), 1500);
      return;
    }
    this.hideHeld();
  }

  buildEffect(fx: BuildEffect): void {
    if (this.view.town()) return;
    const k = this.view.storeyOf(fx.z);
    this.fx.play({ ...fx, z: fx.z - this.view.rowOf(k), y: k * WALL_HEIGHT });
    const placed = fx.objectId === undefined ? undefined : this.lot.placed.find((p) => p.id === fx.objectId);
    if (placed && placed.id !== this.heldId) this.fx.pop(placed.pop);
  }

  /** A world build starts: the hand and the pops keep off the buffers it replaces (a moved thing then shows in its new place). */
  worldBuilding(): void {
    this.fx.clearPops();
    this.hidden = null;
    if (this.release) clearTimeout(this.release);
    this.release = null;
  }

  /** The world's objects are drawn anew: the thing still in hand is hidden in them. */
  objectsBuilt(): void {
    this.fx.clearPops();
    this.hidden = null;
    this.hideHeld();
  }

  /** Each game frame (`dtMs`): the effects, and the thing in hand. */
  frame(dtMs: number): void {
    // Things springing into place cast as they grow.
    if (this.fx.busy) this.view.shadowsMoved();
    this.fx.frame(dtMs);
    this.ghost.frame(dtMs);
  }

  /** The time of day's light (each lighting update): the red ghost is shaded in it. */
  light(l: Lighting): void {
    this.ghost.light(l);
  }

  /** A model file that failed has come: the thing in hand is built with it (`placementGhost.ts`). */
  fileArrived(url: string): void {
    this.ghost.fileArrived(url);
  }

  /** The session ended: nothing in hand, no effects on things that go. */
  clear(): void {
    this.setPlacementGhost(null);
    this.fx.clearPops();
    this.hidden = null;
    this.heldId = null;
    if (this.release) clearTimeout(this.release);
    this.release = null;
  }

  dispose(): void {
    this.clear();
    this.ghost.destroy();
    this.fx.dispose();
  }

  /** The things on walls as the cutaway, the thing in hand and the ghost have them. */
  private applyWallItems(): void {
    const show = this.ghostOnWall;
    this.lot.cutWallItems(show ? 0 : this.cutBits, !show && this.wallsDown, this.hidden?.id ?? null);
  }

  /** Hides the object in hand where it stands, with what stands on its top (and their contact shadows). */
  private hideHeld(): void {
    if (this.hidden && this.hidden.id === this.heldId) return;
    this.showHeld();
    const placed = this.lot.placed;
    const p = this.heldId === null ? undefined : placed.find((o) => o.id === this.heldId);
    if (!p) return;
    // What stands on its top goes with it.
    const riders = placed.filter((o) => o.host === p.id);
    const targets = [p, ...riders].flatMap((o) => (o.shadow ? [o.pop, o.shadow] : [o.pop]));
    this.hidden = { id: p.id, saved: targets.map((target) => ({ target, matrix: target.matrices.slice(target.index * 16, target.index * 16 + 16) })) };
    // Scaled to nothing where it stands (an all-zero matrix would put w = 0).
    for (const { target, matrix } of this.hidden.saved) {
      const m = matrix.slice();
      m.fill(0, 0, 12);
      this.lot.rewrite(target, m);
    }
    this.view.shadowsMoved();
  }

  private showHeld(): void {
    if (!this.hidden) return;
    for (const { target, matrix } of this.hidden.saved) this.lot.rewrite(target, matrix);
    this.hidden = null;
    this.view.shadowsMoved();
  }
}
