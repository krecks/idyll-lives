/**
 * The roaming camera's share of the world (`docs/design/open-world.md`): which 64 m chunks are
 * streamed in round the camera, and which lot is the focus, the one drawn in full (its house's
 * rooms, its things, the residents indoors).
 *
 * - Stream radius: `clamp(radius · 1.2 + 80, 120, 260)` m round the camera's target, the disc
 *   stretched ahead along the view (the far side of the frustum shows more of the town than the
 *   near side): a chunk whose nearest point is within the radius (ahead: 1.5 × the radius, inside
 *   the view's angle) comes in; it goes only once it is `HYSTERESIS` m further out, so a camera
 *   moving to and fro on a chunk edge doesn't stream it in and out.
 * - Focus lot: the plot under the target, else the nearest within `FOCUS_REACH` m; reported once
 *   the camera has settled on it for `SETTLE_MS` (a pan across the town doesn't build every lot it
 *   passes). While `pinned` (build mode, following a resident, the director's shots) the game
 *   chooses the focus itself and none is reported.
 *
 * Engine-neutral: the renderer feeds it the camera each frame (`update`).
 */

import { CHUNK, chunkCoords, chunkId } from './chunks';

/** Metres past the stream radius a chunk keeps streaming before it goes. */
export const HYSTERESIS = 32;
/** A lot this near the camera's target (m) can be the focus when no lot is under it. */
export const FOCUS_REACH = 12;
/** How long (ms) the camera stays on a lot before it becomes the focus. */
export const SETTLE_MS = 350;
/** The camera's target moving slower than this (m/s) has settled. */
const SETTLED_SPEED = 3;

/** The stream radius (m) for a camera this far from its target. */
export function streamRadius(radius: number): number {
  return Math.min(260, Math.max(120, radius * 1.2 + 80));
}

/** A lot as the focus needs it: its id and rectangle (tiles = m). */
export interface FocusPlot {
  id: number;
  x: number;
  z: number;
  w: number;
  d: number;
}

/** The camera as the stream sees it: its target on the ground, its distance, which way it looks (the eye's angle about the target) and its horizontal field of view. */
export interface StreamCamera {
  x: number;
  z: number;
  radius: number;
  /** The orbit's alpha: the eye stands at target + (cos α, sin α) · horizontal distance, so it looks along (-cos α, -sin α). */
  alpha: number;
  /** Horizontal field of view (radians). */
  fovH: number;
}

/** Distance (m) from a point to the nearest point of chunk `id`. */
function chunkDistance(id: number, x: number, z: number): number {
  const [cx, cz] = chunkCoords(id);
  const x0 = cx * CHUNK;
  const z0 = cz * CHUNK;
  const dx = Math.max(x0 - x, 0, x - (x0 + CHUNK));
  const dz = Math.max(z0 - z, 0, z - (z0 + CHUNK));
  return Math.hypot(dx, dz);
}

/** Whether chunk `id` is within `r` of the camera, or within `ahead` · `r` inside its view's angle. */
function inReach(id: number, cam: StreamCamera, r: number, ahead: number): boolean {
  const d = chunkDistance(id, cam.x, cam.z);
  if (d <= r) return true;
  if (d > r * ahead) return false;
  const [cx, cz] = chunkCoords(id);
  const mx = (cx + 0.5) * CHUNK - cam.x;
  const mz = (cz + 0.5) * CHUNK - cam.z;
  // The angle between the view (along -(cos α, sin α)) and the chunk's middle, widened by the chunk's own half-width as seen from there.
  const along = -(mx * Math.cos(cam.alpha) + mz * Math.sin(cam.alpha));
  const dist = Math.hypot(mx, mz) || 1;
  const angle = Math.acos(Math.max(-1, Math.min(1, along / dist)));
  const widen = Math.atan2(CHUNK * 0.71, dist);
  return angle <= cam.fovH / 2 + widen + 0.15;
}

export class Roam {
  /** The chunks streamed in (ids, `shared/chunks.ts`); a new set when it changes. */
  chunks: ReadonlySet<number> = new Set();
  /** The focus lot reported last (null: none yet). */
  focus: number | null = null;
  /** The game chooses the focus itself (build mode, following someone, the director): none is reported. */
  pinned = false;
  private candidate: number | null = null;
  private since = 0;
  private last: { x: number; z: number; t: number } | null = null;
  /** The camera's ground speed (m/s), smoothed, and its way (unit vector): where to prefetch. */
  speed = 0;
  readonly heading = { x: 0, z: 0 };
  /** The bounds of what the stream covers (the town's chunks, and some way round): chunks outside are never streamed. */
  private bounds: { x0: number; z0: number; x1: number; z1: number } | null = null;

  /** Chunks are streamed only within this rectangle (m; null: anywhere). */
  setBounds(b: { x0: number; z0: number; x1: number; z1: number } | null): void {
    this.bounds = b;
  }

  /**
   * One frame: the chunks round the camera (true when the set changed) and the lot under it,
   * reported through `onFocus` once settled. `now`: ms.
   */
  update(cam: StreamCamera, now: number, plotAt: (x: number, z: number, reach: number) => FocusPlot | null, onFocus: (plot: FocusPlot) => boolean): boolean {
    // How fast the target moves, and which way.
    if (this.last) {
      const dt = Math.max(1, now - this.last.t) / 1000;
      const dx = cam.x - this.last.x;
      const dz = cam.z - this.last.z;
      const v = Math.hypot(dx, dz) / dt;
      const k = 1 - Math.exp(-dt / 0.25);
      this.speed += (v - this.speed) * k;
      if (v > 0.5) {
        const l = Math.hypot(dx, dz);
        this.heading.x += (dx / l - this.heading.x) * k;
        this.heading.z += (dz / l - this.heading.z) * k;
      }
    }
    this.last = { x: cam.x, z: cam.z, t: now };

    const changed = this.stream(cam);

    // The focus: under the target (or nearest), held a moment once the camera slows.
    if (!this.pinned) {
      const plot = plotAt(cam.x, cam.z, FOCUS_REACH);
      const id = plot?.id ?? null;
      if (id !== this.candidate) {
        this.candidate = id;
        this.since = now;
      }
      // (Taken only if the game takes it: in build mode it keeps its own.)
      if (plot && id !== this.focus && this.speed < SETTLED_SPEED && now - this.since >= SETTLE_MS && onFocus(plot)) this.focus = id;
    } else {
      this.candidate = null;
    }
    return changed;
  }

  /** The game set the focus itself (a lot shown for build mode, a resident followed). */
  setFocus(id: number | null): void {
    this.focus = id;
    this.candidate = id;
  }

  /** The chunks round the camera, with hysteresis. True when the set changed. */
  private stream(cam: StreamCamera): boolean {
    const r = streamRadius(cam.radius);
    const reach = r * 1.5 + HYSTERESIS;
    const b = this.bounds;
    const x0 = Math.floor((Math.max(cam.x - reach, b ? b.x0 : -Infinity)) / CHUNK);
    const x1 = Math.floor((Math.min(cam.x + reach, b ? b.x1 : Infinity)) / CHUNK);
    const z0 = Math.floor((Math.max(cam.z - reach, b ? b.z0 : -Infinity)) / CHUNK);
    const z1 = Math.floor((Math.min(cam.z + reach, b ? b.z1 : Infinity)) / CHUNK);
    const next = new Set<number>();
    for (let cx = x0; cx <= x1; cx++) {
      for (let cz = z0; cz <= z1; cz++) {
        const id = chunkId(cx, cz);
        const had = this.chunks.has(id);
        if (inReach(id, cam, had ? r + HYSTERESIS : r, 1.5)) next.add(id);
      }
    }
    if (next.size === this.chunks.size && [...next].every((id) => this.chunks.has(id))) return false;
    this.chunks = next;
    return true;
  }
}

/**
 * The ground rectangle the camera's target may roam (m): a big world's whole land and 20 m round
 * it, else the town and 60 m round it.
 */
export function panBounds(world: { width: number; depth: number; storeyDepth?: number; meta?: { world?: { size?: string } } | null }): { x0: number; z0: number; x1: number; z1: number } {
  const big = world.meta?.world?.size === 'big';
  const m = big ? 20 : 60;
  const d = world.storeyDepth ?? world.depth;
  return { x0: -m, z0: -m, x1: world.width + m, z1: d + m };
}

/**
 * A smooth flight of the camera's target to a point (the town map's "look here"): eased out and in,
 * longer the further it goes (0.6 s, up to 2.2 s across a big world), rising on the way when far.
 */
export class Flight {
  private from = { x: 0, z: 0, radius: 0 };
  private to = { x: 0, z: 0, radius: 0 };
  private t = 0;
  private duration = 0;
  active = false;

  /** `cut`: Reduce motion is on: the camera is there on the next step, without the flight. */
  start(fromX: number, fromZ: number, fromRadius: number, toX: number, toZ: number, toRadius: number, cut = false): void {
    this.from = { x: fromX, z: fromZ, radius: fromRadius };
    this.to = { x: toX, z: toZ, radius: toRadius };
    const d = Math.hypot(toX - fromX, toZ - fromZ);
    this.duration = cut ? 0 : Math.min(2.2, 0.6 + d / 400);
    this.t = 0;
    this.active = true;
  }

  stop(): void {
    this.active = false;
  }

  /** Steps the flight by `dt` s: where the target and radius are now. */
  step(dt: number, out: { x: number; z: number; radius: number }): boolean {
    if (!this.active) return false;
    this.t = this.duration > 0 ? Math.min(1, this.t + dt / this.duration) : 1;
    const s = this.t * this.t * (3 - 2 * this.t);
    const d = Math.hypot(this.to.x - this.from.x, this.to.z - this.from.z);
    // Up and over a long way: the camera rises by up to a third of the distance, half way.
    const lift = this.t >= 1 ? 0 : Math.min(60, d / 3) * Math.sin(Math.PI * s);
    out.x = this.from.x + (this.to.x - this.from.x) * s;
    out.z = this.from.z + (this.to.z - this.from.z) * s;
    out.radius = this.from.radius + (this.to.radius - this.from.radius) * s + lift;
    if (this.t >= 1) this.active = false;
    return true;
  }
}

/** What `warmLots` reads of a world: its lots and what stands on them (rows of upper storeys below the ground's, `storeyDepth` apart). */
export interface WarmWorld {
  plots: readonly { x: number; z: number; w: number; d: number; house: unknown }[];
  storeyDepth?: number;
  depth: number;
  walls?: readonly { x: number; z: number; faces?: readonly number[]; form?: unknown }[];
  openings?: readonly { x: number; z: number; kind: string; style?: number }[];
  floors?: readonly (readonly number[])[];
  objects: readonly { x: number; z: number; def: string }[];
}

/**
 * Up to `max` lots with houses (besides `home`) that between them show the most of what a big
 * world's houses are made of (wall coverings, half walls, floors, door and window styles, things):
 * each in turn the one adding the most not yet seen (none once a lot adds nothing). The warm-up
 * builds each in full behind the loading veil, so a lot the roaming camera settles on later makes
 * few pipelines in play (`PlayCanvasRenderer.warmGame`).
 */
export function warmLots(world: WarmWorld, home: { x: number; z: number } | null, max: number): { x: number; z: number; w: number; d: number }[] {
  const D = world.storeyDepth ?? world.depth;
  const isHome = (p: { x: number; z: number }) => !!home && p.x === home.x && p.z === home.z;
  const lots = world.plots.filter((p) => p.house || isHome(p));
  const at = (p: { x: number; z: number; w: number; d: number }, x: number, z: number) => x >= p.x && x < p.x + p.w && z % D >= p.z && z % D < p.z + p.d;
  const what = lots.map((p) => {
    const s = new Set<string>();
    for (const e of world.walls ?? []) if (at(p, e.x, e.z)) for (const [i, f] of (e.faces ?? [0, 0]).entries()) s.add(`f${i}:${f}${e.form ? ':half' : ''}`);
    for (const o of world.openings ?? []) if (at(p, o.x, o.z)) s.add(`o${o.kind}:${o.style ?? 0}`);
    for (const f of world.floors ?? []) if (at(p, f[0], f[1])) s.add(`c${f[2]}`);
    for (const o of world.objects) if (at(p, o.x, o.z)) s.add(`t${o.def}`);
    return s;
  });
  // (The home's own is warmed anyway.)
  const seen = new Set<string>(lots.flatMap((p, i) => (isHome(p) ? [...what[i]] : [])));
  const out: { x: number; z: number; w: number; d: number }[] = [];
  while (out.length < max) {
    let best = -1;
    let gain = 0;
    what.forEach((s, i) => {
      if (isHome(lots[i])) return;
      let n = 0;
      for (const k of s) if (!seen.has(k)) n++;
      if (n > gain) [best, gain] = [i, n];
    });
    if (best < 0) break;
    for (const k of what[best]) seen.add(k);
    const p = lots[best];
    out.push({ x: p.x, z: p.z, w: p.w, d: p.d });
  }
  return out;
}
