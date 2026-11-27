/**
 * Walk mode: the player walks as one of their household, seen from their own eyes. W/S go forward
 * and back, A/D turn, Q/E step sideways (World of Warcraft's keys), a drag looks round, F uses what's
 * in the middle of the view or talks to whoever is there. The resident chooses nothing on their own
 * meanwhile (sim-core `Command::Hold`); each move is a `steer` to a point just ahead of them.
 */

import type { SimBridge } from '../core/bridge';
import type { Renderer } from '../render/types';
import * as pmath from './pmath';

/** How far ahead of the walker each steer aims (tiles): far enough to keep walking between steers. */
const AHEAD = 1.2;
/** How often a held key steers again (ms). */
const STEER_MS = 150;
/** A/D's turn (rad/s). */
const TURN = (120 * Math.PI) / 180;
/** A drag's look (rad per CSS px). */
const LOOK = 0.005;
const PITCH_MOST = Math.PI / 3;

export type WalkKey = 'w' | 'a' | 's' | 'd' | 'q' | 'e';
/** The keys that move (F, besides, uses or talks). */
export const WALK_KEYS: ReadonlySet<string> = new Set<WalkKey>(['w', 'a', 's', 'd', 'q', 'e']);

/** The way the held keys walk (unit, game x/z), or null standing; and the turn (-1 left, 1 right, 0). */
export function moveDir(held: ReadonlySet<WalkKey>, heading: number): { dir: [number, number] | null; turn: number } {
  const ahead = (held.has('w') ? 1 : 0) - (held.has('s') ? 1 : 0);
  // (Game z is the engine's -z: to the right of the heading is -π/2, so D turns the heading down.)
  const side = (held.has('e') ? 1 : 0) - (held.has('q') ? 1 : 0);
  const turn = (held.has('d') ? 1 : 0) - (held.has('a') ? 1 : 0);
  const [fx, fz] = [pmath.cos(heading), pmath.sin(heading)];
  const [rx, rz] = [fz, -fx];
  const x = fx * ahead + rx * side;
  const z = fz * ahead + rz * side;
  const len = pmath.hypot(x, z);
  return { dir: len > 0 ? [x / len, z / len] : null, turn };
}

/** Resident `id`'s snapshot row and where they stand (tiles), or null when not on the map (away at work, gone). */
export function whereIs(bridge: SimBridge, id: number): { row: number; x: number; z: number } | null {
  const snap = bridge.latest();
  const { layout } = bridge;
  for (let i = 0; i < snap[layout.header.simCount]; i++) {
    const o = layout.headerLen + i * layout.simStride;
    if (snap[o + layout.sim.id] === id) return snap[o + layout.sim.away] ? null : { row: i, x: snap[o + layout.sim.x], z: snap[o + layout.sim.z] };
  }
  return null;
}

/** Whether (`x`, `z`) is within `reach` tiles of `me` and within 30° of the way they face (`heading`). */
export function inFront(me: { x: number; z: number }, heading: number, x: number, z: number, reach: number): boolean {
  const [dx, dz] = [x - me.x, z - me.z];
  const d = pmath.hypot(dx, dz);
  return d <= reach && (d < 0.3 || (dx * pmath.cos(heading) + dz * pmath.sin(heading)) / d >= pmath.cos(Math.PI / 6));
}

interface WalkDeps {
  bridge: SimBridge;
  renderer: Renderer;
  /** F: use or talk to what's at this point of the screen (CSS px). */
  useAt(x: number, y: number): void;
  /** The resident is gone from the lot (to work, away): walk mode ends. */
  lost(): void;
  /** Keys go to a text box or a menu instead (the chat, a pie menu). */
  busy(): boolean;
  /** A point a steer may aim at (on the lot: one off it is refused). */
  fits(x: number, z: number): boolean;
}

export class WalkMode {
  sim: number | null = null;
  private heading = 0;
  private pitch = 0;
  private readonly held = new Set<WalkKey>();
  private lastSteer = 0;
  private lastDir: [number, number] | null = null;
  private frame = 0;
  private last = 0;
  private drag: { x: number; y: number } | null = null;
  private readonly off: (() => void)[] = [];

  constructor(private readonly deps: WalkDeps) {}

  enter(sim: number, canvas: HTMLElement): void {
    this.exit();
    const me = whereIs(this.deps.bridge, sim);
    if (!me) return;
    this.sim = sim;
    this.deps.bridge.send({ type: 'hold', sim, on: true });
    this.heading = this.deps.renderer.setWalkView(me.row);
    this.pitch = 0;
    this.deps.renderer.setWalkLook(this.heading, this.pitch);
    const on = <K extends keyof WindowEventMap>(t: EventTarget, type: K, f: (e: WindowEventMap[K]) => void) => {
      t.addEventListener(type, f as EventListener);
      this.off.push(() => t.removeEventListener(type, f as EventListener));
    };
    on(window, 'keydown', (e) => this.key(e, true));
    on(window, 'keyup', (e) => this.key(e, false));
    on(window, 'blur', () => this.held.clear());
    on(canvas, 'pointerdown', (e) => (this.drag = { x: e.clientX, y: e.clientY }));
    on(window, 'pointerup', () => (this.drag = null));
    on(window, 'pointermove', (e) => {
      if (!this.drag) return;
      this.heading -= (e.clientX - this.drag.x) * LOOK;
      this.pitch = Math.max(-PITCH_MOST, Math.min(PITCH_MOST, this.pitch - (e.clientY - this.drag.y) * LOOK));
      this.drag = { x: e.clientX, y: e.clientY };
      this.deps.renderer.setWalkLook(this.heading, this.pitch);
    });
    this.last = performance.now();
    this.frame = requestAnimationFrame(this.tick);
  }

  exit(): void {
    if (this.sim === null) return;
    const sim = this.sim;
    this.sim = null;
    cancelAnimationFrame(this.frame);
    for (const f of this.off.splice(0)) f();
    this.held.clear();
    this.drag = null;
    if (this.lastDir) this.stop(sim);
    this.lastDir = null;
    this.deps.renderer.setWalkView(null);
    this.deps.bridge.send({ type: 'hold', sim, on: false });
  }

  /** The other resident nearest in front of the walker within `reach` tiles (`inFront`), or null. */
  facing(reach: number): number | null {
    const me = this.sim === null ? null : whereIs(this.deps.bridge, this.sim);
    if (!me) return null;
    const snap = this.deps.bridge.latest();
    const { layout } = this.deps.bridge;
    let best: number | null = null;
    let bestD = Infinity;
    for (let i = 0; i < snap[layout.header.simCount]; i++) {
      const o = layout.headerLen + i * layout.simStride;
      const [id, x, z] = [snap[o + layout.sim.id], snap[o + layout.sim.x], snap[o + layout.sim.z]];
      const d = pmath.hypot(x - me.x, z - me.z);
      if (id === this.sim || snap[o + layout.sim.away] || d >= bestD || !inFront(me, this.heading, x, z, reach)) continue;
      best = id;
      bestD = d;
    }
    return best;
  }

  private stop(sim: number): void {
    const me = whereIs(this.deps.bridge, sim);
    if (me) this.deps.bridge.send({ type: 'steer', sim, x: me.x, z: me.z });
  }

  private key(e: KeyboardEvent, down: boolean): void {
    const k = e.key.toLowerCase();
    if (down && (e.repeat || e.ctrlKey || e.metaKey || e.altKey)) return;
    if (down && this.deps.busy()) return;
    // (Typed into a text box: words, not steps.)
    const t = e.target;
    if (down && (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || t instanceof HTMLSelectElement || (t instanceof HTMLElement && t.isContentEditable))) return;
    if (k === 'f') {
      if (down) this.deps.useAt(innerWidth / 2, innerHeight / 2);
      return;
    }
    if (!WALK_KEYS.has(k)) return;
    if (down) this.held.add(k as WalkKey);
    else this.held.delete(k as WalkKey);
  }

  private readonly tick = (now: number): void => {
    this.frame = requestAnimationFrame(this.tick);
    const sim = this.sim;
    if (sim === null) return;
    const dt = Math.min(0.1, (now - this.last) / 1000);
    this.last = now;
    const me = whereIs(this.deps.bridge, sim);
    if (!me) return this.deps.lost();
    const { dir, turn } = moveDir(this.held, this.heading);
    if (turn) {
      this.heading -= turn * TURN * dt;
      this.deps.renderer.setWalkLook(this.heading, this.pitch);
    }
    if (!dir) {
      if (this.lastDir) this.stop(sim);
      this.lastDir = null;
      return;
    }
    // (Turning while walking bends the way a little each frame: a new steer only once it's bent ~8°.)
    const changed = !this.lastDir || this.lastDir[0] * dir[0] + this.lastDir[1] * dir[1] < 0.99;
    if (!changed && now - this.lastSteer < STEER_MS) return;
    const [x, z] = [me.x + dir[0] * AHEAD, me.z + dir[1] * AHEAD];
    if (!this.deps.fits(x, z)) return;
    this.lastSteer = now;
    this.lastDir = dir;
    this.deps.bridge.send({ type: 'steer', sim, x, z });
  };
}
