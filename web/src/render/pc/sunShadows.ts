/**
 * The sun's shadow map in the game, as the Babylon renderer's (`babylon/sunShadows.ts`): it covers
 * a fixed square round the viewed lot, so its texels keep their size and place while the camera
 * turns and the sun steps, and it is drawn again only when something in it changed (`moved`): the
 * sun's step, a resident's pose (throttled to the preset's `shadowRate`), a thing placed, the
 * walls. Casters outside the square aren't drawn. The menus' town is fitted round the camera's
 * view instead, and drawn every frame.
 *
 * PlayCanvas fits a directional light's map to the camera's view. For a light with a square, the
 * fit is handed the square instead: the directional shadow renderer's `cull` asks the camera for
 * the corners of its view, and gets eight points of a sphere round the square's middle (its
 * radius the square's half side), seen from the camera. Everything else (the light's camera
 * turned with the sun, snapped to whole texels, its depth fitted to the casters, the casters culled
 * to it) is PlayCanvas's own.
 */

import * as pc from './engine';
import { toEngine } from './space';

interface Square {
  /** The middle (PlayCanvas space) and the half side (m). */
  centre: pc.Vec3;
  radius: number;
}

/** The squares of the lights that have one (by PlayCanvas's scene light). */
const squares = new WeakMap<object, Square>();
const patched = new WeakSet<object>();

interface DirectionalShadows {
  cull(light: object, comp: object, camera: pc.Camera, casters?: object, mask?: number): void;
}

/** Hands the directional shadow renderer of `app` the squares of lights that have one. */
function patch(app: pc.AppBase): void {
  // SAFETY: the forward renderer keeps its directional shadow renderer in `_shadowRendererDirectional`
  // (PlayCanvas 2.23, pinned); `check:chunks` checks the name is still there.
  const sr = (app.renderer as unknown as { _shadowRendererDirectional?: DirectionalShadows })._shadowRendererDirectional;
  if (!sr || patched.has(sr)) return;
  patched.add(sr);
  const cull = sr.cull.bind(sr);
  const corners = Array.from({ length: 8 }, () => new pc.Vec3());
  const inverse = new pc.Mat4();
  sr.cull = (light, comp, camera, casters, mask) => {
    const square = squares.get(light);
    if (!square) return cull(light, comp, camera, casters, mask);
    const own = camera.getFrustumCorners;
    camera.getFrustumCorners = () => {
      // Eight points whose middle is the square's and whose farthest is its half side away, in the camera's space (`cull` turns them back into the world).
      inverse.copy(camera.node.getWorldTransform()).invert();
      const { centre: c, radius: r } = square;
      const offsets: [number, number, number][] = [[r, 0, 0], [-r, 0, 0], [0, r, 0], [0, -r, 0], [0, 0, r], [0, 0, -r], [0, 0, 0], [0, 0, 0]];
      offsets.forEach(([x, y, z], i) => inverse.transformPoint(corners[i].set(c.x + x, c.y + y, c.z + z), corners[i]));
      return corners;
    };
    try {
      return cull(light, comp, camera, casters, mask);
    } finally {
      camera.getFrustumCorners = own;
    }
  };
}

export class SunShadows {
  private readonly light: pc.LightComponent;
  /** Milliseconds between pose-driven redraws (0: every change), and when the last one was made. */
  private readonly interval: number;
  private lastDraw = -Infinity;
  private pending = false;
  private overview = false;
  private readonly square: Square = { centre: new pc.Vec3(), radius: 30 };

  constructor(app: pc.AppBase, sun: pc.Entity, rate: number) {
    patch(app);
    this.light = sun.light!;
    this.interval = rate > 0 ? 1000 / Math.min(240, rate) : 0;
    this.light.shadowUpdateMode = pc.SHADOWUPDATE_NONE;
    squares.set(this.light.light, this.square);
  }

  /** The square round the viewed lot: its middle (game space, on the ground) and half side (m). */
  setSquare(x: number, z: number, radius: number): void {
    toEngine(x, 0, z, this.square.centre);
    this.square.radius = radius;
    this.moved();
  }

  /** The menus' town (fitted round the view, drawn every frame) or the game. */
  setOverview(on: boolean): void {
    this.overview = on;
    if (on) squares.delete(this.light.light);
    else squares.set(this.light.light, this.square);
    this.light.shadowUpdateMode = on ? pc.SHADOWUPDATE_REALTIME : pc.SHADOWUPDATE_NONE;
    this.moved();
  }

  /**
   * Something in the map changed: it is drawn again in the next frame. `urgent` changes (the sun,
   * the walls, a thing placed) draw at once; a resident's pose may wait for the shadow rate.
   */
  moved(urgent = true): void {
    if (this.overview) return;
    const now = performance.now();
    if (this.interval <= 0 || urgent || now - this.lastDraw >= this.interval) {
      this.pending = false;
      this.lastDraw = now;
      this.light.shadowUpdateMode = pc.SHADOWUPDATE_THISFRAME;
    } else {
      this.pending = true;
    }
  }

  /** Each frame: a pose redraw held back draws once the interval has passed. */
  frame(): void {
    if (!this.pending || performance.now() - this.lastDraw < this.interval) return;
    this.pending = false;
    this.lastDraw = performance.now();
    this.light.shadowUpdateMode = pc.SHADOWUPDATE_THISFRAME;
  }
}
