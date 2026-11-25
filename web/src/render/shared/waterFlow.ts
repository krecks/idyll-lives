/**
 * The water's flow as the renderer draws it (`pc/water.ts`): for each wet sample of a height field (`game/land.ts`),
 * which way its water runs and how fast, and how much foam it carries. Engine-neutral, and pure.
 *
 * - The way: a drawn river's own (`Land.flow`, laid by the River brush); on a generated river (the water at the
 *   sea's level that crosses the world from its west edge to its east and reaches neither of the others:
 *   `riverCourse`), along its course towards its lower end; else, water stepping down from one level to the next
 *   runs from the higher to the lower, through the water on the higher level towards where it drops. A sample of
 *   the same water with no way of its own takes its neighbours' (a bank the water filled after the river was drawn).
 *   Lakes and the sea have none: they only ripple.
 * - The speed (0..1): fast where the water is narrow and where its level drops (a rapid), slower in wide water, near
 *   nothing at the shore. None without a way; a river's never under `FLOW.slowest`, but water whose way is only
 *   its steps' (a lake with a river running out of it) slows to nearly nothing where it is wide.
 * - The foam (0..1): where the level drops between neighbouring samples, and a little where the water runs fast.
 *
 * `waterFlow` works out a rectangle of samples (a brush's stamp, a stroke) from the land within `FLOW_MARGIN`
 * samples of it; `riverCourse` is the whole land's, worked out when the land is taken on and kept.
 */

import { chamfer } from './landBake';
import { flowOf, LAND_CELL, NONE, SEA_CM, type Land, type LandRect } from '../../game/land';
import { clamp01, smoothstep } from '../../game/noise';

/**
 * How the speed is worked out: `narrow`, the half-width (m) at and under which water with a way runs at full speed
 * (wider, slower: as `narrow / half-width`), never under `slowest`; `widthReach`, how far (samples) round a sample
 * the water's half-width is looked for; at the shore it slows to `shoreSpeed` of itself over `shore` m; a drop between
 * neighbours from `drop[0]` m is a rapid, a full one from `drop[1]` (a river drawn down a slope steps down a little
 * every few metres: it runs fast there, and foams only where it falls further), felt `rapidReach` samples up and
 * down from it.
 */
export const FLOW = {
  narrow: 6,
  slowest: 0.3,
  widthReach: 10,
  shore: 3,
  shoreSpeed: 0.15,
  drop: [0.2, 0.8],
  rapidReach: 2,
  /** Foam where the water runs this fast and more, at most `fastFoam`. */
  fast: 0.9,
  fastFoam: 0.02,
} as const;

/** How far round a rectangle (samples) `waterFlow` reads the land: the reach of everything it looks at, and room for the steps' ways. */
export const FLOW_MARGIN = 16;

/** The flow of a land's water, by sample (row-major, as the land). */
export interface WaterFlow {
  cols: number;
  rows: number;
  /** The way (game space, unit length; 0, 0 where none). */
  wx: Float32Array;
  wz: Float32Array;
  /** How fast (0..1). */
  speed: Float32Array;
  /** How much foam (0..1). */
  foam: Float32Array;
}

/**
 * A generated river's course (`riverCourse`): the samples of its water (`member`), and by column of samples the way
 * it runs there (unit length, game space: along its middle line, towards its lower end).
 */
export interface RiverCourse {
  member: Uint8Array;
  tx: Float32Array;
  tz: Float32Array;
}


/** A still land's flow (every sample none). */
export function stillFlow(cols: number, rows: number): WaterFlow {
  const n = cols * rows;
  return { cols, rows, wx: new Float32Array(n), wz: new Float32Array(n), speed: new Float32Array(n), foam: new Float32Array(n) };
}

/**
 * The river a generated world has across it, if it has one: the water at the sea's level that reaches both the
 * land's west and east edges and neither its north nor its south (the sea along a coast, an island's sea, reach one
 * of those). Its middle line by column, smoothed; it runs towards the end whose banks stand lower, or, as level,
 * one way or the other as the course leans (the same way every time for the same land). Null without one.
 */
export function riverCourse(land: Land): RiverCourse | null {
  const { cols, rows, h, wl } = land;
  const sea = (k: number) => wl[k] === SEA_CM && wl[k] > h[k];
  const seen = new Uint8Array(cols * rows);
  const member = new Uint8Array(cols * rows);
  const queue: number[] = [];
  let any = false;
  for (let s = 0; s < seen.length; s++) {
    if (seen[s] || !sea(s)) continue;
    // (One body of water: which edges it reaches.)
    queue.length = 0;
    queue.push(s);
    seen[s] = 1;
    let edges = 0;
    for (let q = 0; q < queue.length; q++) {
      const k = queue[q];
      const [i, j] = [k % cols, Math.floor(k / cols)];
      if (i === 0) edges |= 1;
      if (i === cols - 1) edges |= 2;
      if (j === 0) edges |= 4;
      if (j === rows - 1) edges |= 8;
      if (i > 0 && !seen[k - 1] && sea(k - 1)) (seen[k - 1] = 1), queue.push(k - 1);
      if (i < cols - 1 && !seen[k + 1] && sea(k + 1)) (seen[k + 1] = 1), queue.push(k + 1);
      if (j > 0 && !seen[k - cols] && sea(k - cols)) (seen[k - cols] = 1), queue.push(k - cols);
      if (j < rows - 1 && !seen[k + cols] && sea(k + cols)) (seen[k + cols] = 1), queue.push(k + cols);
    }
    if (edges !== 3) continue;
    any = true;
    for (const k of queue) member[k] = 1;
  }
  if (!any) return null;
  // Its middle by column (in samples), and its banks' height either side.
  const mid = new Float64Array(cols).fill(NaN);
  const bank = new Float64Array(cols).fill(NaN);
  for (let i = 0; i < cols; i++) {
    let [sum, n, lo, hi] = [0, 0, rows, -1];
    for (let j = 0; j < rows; j++) {
      if (!member[j * cols + i]) continue;
      sum += j;
      n++;
      lo = Math.min(lo, j);
      hi = Math.max(hi, j);
    }
    if (!n) continue;
    mid[i] = sum / n;
    let [banks, m] = [0, 0];
    for (const j of [lo - 4, hi + 4]) {
      if (j < 0 || j >= rows || wl[j * cols + i] !== NONE) continue;
      banks += h[j * cols + i];
      m++;
    }
    if (m) bank[i] = banks / m;
  }
  // (A column the water misses, as a meander's loop may: its neighbours' middle.)
  for (let i = 0; i < cols; i++) {
    if (!Number.isNaN(mid[i])) continue;
    let [a, b] = [i - 1, i + 1];
    while (a >= 0 && Number.isNaN(mid[a])) a--;
    while (b < cols && Number.isNaN(mid[b])) b++;
    mid[i] = a < 0 ? mid[b] : b >= cols ? mid[a] : mid[a] + ((mid[b] - mid[a]) * (i - a)) / (b - a);
  }
  const SMOOTH = 8;
  const smooth = new Float64Array(cols);
  for (let i = 0; i < cols; i++) {
    let [sum, n] = [0, 0];
    for (let d = -SMOOTH; d <= SMOOTH; d++) {
      const c = Math.min(cols - 1, Math.max(0, i + d));
      sum += mid[c];
      n++;
    }
    smooth[i] = sum / n;
  }
  // Which way: towards the lower banks, the westmost sixth against the eastmost; level, as the course leans.
  const sixth = Math.max(1, Math.floor(cols / 6));
  const mean = (from: number, to: number) => {
    let [sum, n] = [0, 0];
    for (let i = from; i < to; i++) if (!Number.isNaN(bank[i])) (sum += bank[i]), n++;
    return n ? sum / n : 0;
  };
  const [west, east] = [mean(0, sixth), mean(cols - sixth, cols)];
  const sign = Math.abs(west - east) >= 20 ? (east < west ? 1 : -1) : smooth[0] >= smooth[cols - 1] ? 1 : -1;
  const tx = new Float32Array(cols);
  const tz = new Float32Array(cols);
  const SPAN = 3;
  for (let i = 0; i < cols; i++) {
    const [a, b] = [Math.max(0, i - SPAN), Math.min(cols - 1, i + SPAN)];
    const slope = b > a ? (smooth[b] - smooth[a]) / (b - a) : 0;
    const l = Math.hypot(1, slope);
    tx[i] = (sign * 1) / l;
    tz[i] = (sign * slope) / l;
  }
  return { member, tx, tz };
}

/**
 * The flow of `land`'s water over `rect` (samples; the whole land if not given), written into `into` (a new one if
 * not given; its other samples left as they were) and returned. `course`: the land's river (`riverCourse`), if it
 * has one. Read from the land within `FLOW_MARGIN` samples of `rect`.
 */
export function waterFlow(land: Land, course: RiverCourse | null, rect?: LandRect, into?: WaterFlow): WaterFlow {
  const { cols, rows, h, wl } = land;
  const out = into && into.cols === cols && into.rows === rows ? into : stillFlow(cols, rows);
  const r = rect ?? { i0: 0, j0: 0, i1: cols, j1: rows };
  const [i0, j0] = [Math.max(0, r.i0 - FLOW_MARGIN), Math.max(0, r.j0 - FLOW_MARGIN)];
  const [i1, j1] = [Math.min(cols, r.i1 + FLOW_MARGIN), Math.min(rows, r.j1 + FLOW_MARGIN)];
  const [w, d] = [i1 - i0, j1 - j0];
  if (w <= 0 || d <= 0) return out;
  const n = w * d;
  // The window's samples: wet or not, and each one's level.
  const wet = new Uint8Array(n);
  const level = new Int16Array(n);
  let anyWet = false;
  for (let j = 0; j < d; j++) {
    for (let i = 0; i < w; i++) {
      const k = (j0 + j) * cols + i0 + i;
      if (wl[k] === NONE || wl[k] <= h[k]) continue;
      wet[j * w + i] = 1;
      level[j * w + i] = wl[k];
      anyWet = true;
    }
  }
  if (!anyWet) {
    clear(out, r);
    return out;
  }
  // (Same water: two wet samples side by side, whatever their levels.)
  const near = (q: number, i: number, j: number, fn: (p: number, di: number, dj: number) => void) => {
    if (i > 0 && wet[q - 1]) fn(q - 1, -1, 0);
    if (i < w - 1 && wet[q + 1]) fn(q + 1, 1, 0);
    if (j > 0 && wet[q - w]) fn(q - w, 0, -1);
    if (j < d - 1 && wet[q + w]) fn(q + w, 0, 1);
  };

  // How far each wet sample is from the shore (m: to the nearest dry sample, the shore half a sample short of it), and
  // the water's half-width round it (the furthest from the shore nearby).
  const shore = chamfer(wet, w, d, 0);
  const half = widest(shore, wet, w, d, FLOW.widthReach);

  // The ways (`has`: 1 a river's, a drawn one's or a generated one's; 2 only the steps').
  const wx = new Float32Array(n);
  const wz = new Float32Array(n);
  const has = new Uint8Array(n);
  const flow = land.flow;
  for (let j = 0; j < d; j++) {
    for (let i = 0; i < w; i++) {
      const q = j * w + i;
      if (!wet[q]) continue;
      const k = (j0 + j) * cols + i0 + i;
      const way = flow ? flowOf(flow[k]) : null;
      if (way) {
        [wx[q], wz[q]] = way;
        has[q] = 1;
      } else if (course?.member[k]) {
        [wx[q], wz[q]] = [course.tx[i0 + i], course.tz[i0 + i]];
        has[q] = 1;
      }
    }
  }
  stepWays(wet, level, has, wx, wz, w, d, near);
  // A sample of the same water at the same level with no way takes its neighbours', a few samples in.
  for (let pass = 0; pass < 4; pass++) {
    const take: number[] = [];
    for (let j = 0; j < d; j++) {
      for (let i = 0; i < w; i++) {
        const q = j * w + i;
        if (!wet[q] || has[q]) continue;
        let [sx, sz, kind] = [0, 0, 2];
        near(q, i, j, (p) => {
          if (!has[p] || level[p] !== level[q]) return;
          [sx, sz] = [sx + wx[p], sz + wz[p]];
          kind = Math.min(kind, has[p]);
        });
        const l = Math.hypot(sx, sz);
        if (l > 1e-3) take.push(q, sx / l, sz / l, kind);
      }
    }
    if (!take.length) break;
    for (let t = 0; t < take.length; t += 4) [wx[take[t]], wz[take[t]], has[take[t]]] = [take[t + 1], take[t + 2], take[t + 3]];
  }
  // Smoothed a little (a drawn way's steps, the steps' corners), through the same water.
  for (let pass = 0; pass < 2; pass++) {
    const [sx, sz] = [wx.slice(), wz.slice()];
    for (let j = 0; j < d; j++) {
      for (let i = 0; i < w; i++) {
        const q = j * w + i;
        if (!has[q]) continue;
        let [ax, az] = [sx[q] * 2, sz[q] * 2];
        near(q, i, j, (p) => {
          if (has[p]) [ax, az] = [ax + sx[p], az + sz[p]];
        });
        const l = Math.hypot(ax, az);
        if (l > 1e-3) [wx[q], wz[q]] = [ax / l, az / l];
      }
    }
  }

  // Rapids: where the level drops between neighbours, felt a little up and down from there.
  const rapid = new Float32Array(n);
  for (let j = 0; j < d; j++) {
    for (let i = 0; i < w; i++) {
      const q = j * w + i;
      if (!wet[q]) continue;
      let drop = 0;
      near(q, i, j, (p) => {
        drop = Math.max(drop, Math.abs(level[q] - level[p]) / 100);
      });
      rapid[q] = smoothstep(FLOW.drop[0], FLOW.drop[1], drop);
    }
  }
  const rapids = spread(rapid, wet, w, d, FLOW.rapidReach);

  // Written out over `rect`.
  for (let j = Math.max(0, r.j0); j < Math.min(rows, r.j1); j++) {
    for (let i = Math.max(0, r.i0); i < Math.min(cols, r.i1); i++) {
      const k = j * cols + i;
      const q = (j - j0) * w + (i - i0);
      if (!wet[q]) {
        out.wx[k] = out.wz[k] = out.speed[k] = out.foam[k] = 0;
        continue;
      }
      const rp = rapids[q];
      let speed = 0;
      if (has[q]) {
        const narrow = Math.min(1, FLOW.narrow / Math.max(0.1, half[q]));
        speed = has[q] === 1 ? Math.max(FLOW.slowest, narrow) : narrow * narrow;
        speed *= FLOW.shoreSpeed + (1 - FLOW.shoreSpeed) * smoothstep(0, FLOW.shore, shore[q] - LAND_CELL / 2);
        if (rp > 0) speed = Math.max(speed, 0.5 + 0.5 * rp);
      }
      out.wx[k] = has[q] ? wx[q] : 0;
      out.wz[k] = has[q] ? wz[q] : 0;
      out.speed[k] = speed;
      out.foam[k] = clamp01(rp + FLOW.fastFoam * smoothstep(FLOW.fast, 1, speed));
    }
  }
  return out;
}

/** Every sample of `rect` still. */
function clear(out: WaterFlow, rect: LandRect): void {
  for (let j = Math.max(0, rect.j0); j < Math.min(out.rows, rect.j1); j++) {
    for (let i = Math.max(0, rect.i0); i < Math.min(out.cols, rect.i1); i++) {
      const k = j * out.cols + i;
      out.wx[k] = out.wz[k] = out.speed[k] = out.foam[k] = 0;
    }
  }
}

/**
 * The ways of water that steps down: on a level with lower water beside it, from where it drops (each such sample
 * towards the lower water) back through the samples of that level (each towards the nearer of its neighbours to the
 * drop). Samples with a way already are left as they are, and not gone through.
 */
function stepWays(wet: Uint8Array, level: Int16Array, has: Uint8Array, wx: Float32Array, wz: Float32Array, w: number, d: number, near: (q: number, i: number, j: number, fn: (p: number, di: number, dj: number) => void) => void): void {
  const n = w * d;
  const dist = new Int32Array(n).fill(-1);
  const queue: number[] = [];
  for (let j = 0; j < d; j++) {
    for (let i = 0; i < w; i++) {
      const q = j * w + i;
      if (!wet[q] || has[q]) continue;
      let [sx, sz] = [0, 0];
      near(q, i, j, (p, di, dj) => {
        const drop = level[q] - level[p];
        if (drop > 0) [sx, sz] = [sx + di * drop, sz + dj * drop];
      });
      const l = Math.hypot(sx, sz);
      if (l <= 0) continue;
      [wx[q], wz[q]] = [sx / l, sz / l];
      has[q] = 2;
      dist[q] = 0;
      queue.push(q);
    }
  }
  if (!queue.length) return;
  for (let t = 0; t < queue.length; t++) {
    const q = queue[t];
    const [i, j] = [q % w, Math.floor(q / w)];
    near(q, i, j, (p) => {
      if (dist[p] >= 0 || has[p] || level[p] !== level[q]) return;
      dist[p] = dist[q] + 1;
      queue.push(p);
    });
  }
  // Down the distance to the drop (the steepest way among the neighbours on the same level).
  const at = (p: number, q: number) => (dist[p] >= 0 && level[p] === level[q] ? dist[p] : dist[q]);
  for (let j = 0; j < d; j++) {
    for (let i = 0; i < w; i++) {
      const q = j * w + i;
      if (dist[q] <= 0) continue;
      const gx = (i > 0 && wet[q - 1] ? at(q - 1, q) : dist[q]) - (i < w - 1 && wet[q + 1] ? at(q + 1, q) : dist[q]);
      const gz = (j > 0 && wet[q - w] ? at(q - w, q) : dist[q]) - (j < d - 1 && wet[q + w] ? at(q + w, q) : dist[q]);
      const l = Math.hypot(gx, gz);
      if (l <= 0) continue;
      [wx[q], wz[q]] = [gx / l, gz / l];
      has[q] = 2;
    }
  }
}

/** For each wet sample, the largest of `field` within `reach` samples (along each axis, through the wet). */
function widest(field: Float32Array, wet: Uint8Array, w: number, d: number, reach: number): Float32Array {
  const across = new Float32Array(w * d);
  for (let j = 0; j < d; j++) {
    for (let i = 0; i < w; i++) {
      let m = 0;
      for (let a = Math.max(0, i - reach); a <= Math.min(w - 1, i + reach); a++) if (wet[j * w + a]) m = Math.max(m, field[j * w + a]);
      across[j * w + i] = m;
    }
  }
  const out = new Float32Array(w * d);
  for (let j = 0; j < d; j++) {
    for (let i = 0; i < w; i++) {
      if (!wet[j * w + i]) continue;
      let m = 0;
      for (let b = Math.max(0, j - reach); b <= Math.min(d - 1, j + reach); b++) m = Math.max(m, across[b * w + i]);
      out[j * w + i] = m;
    }
  }
  return out;
}

/** `field` spread out through the wet samples `reach` samples each way, falling off with the distance. */
function spread(field: Float32Array, wet: Uint8Array, w: number, d: number, reach: number): Float32Array {
  const out = field.slice();
  for (let j = 0; j < d; j++) {
    for (let i = 0; i < w; i++) {
      const v = field[j * w + i];
      if (!v) continue;
      for (let b = Math.max(0, j - reach); b <= Math.min(d - 1, j + reach); b++) {
        for (let a = Math.max(0, i - reach); a <= Math.min(w - 1, i + reach); a++) {
          const p = b * w + a;
          if (!wet[p]) continue;
          const f = v * (1 - Math.hypot(a - i, b - j) / (reach + 1));
          if (f > out[p]) out[p] = f;
        }
      }
    }
  }
  return out;
}

/**
 * The flow of `rect` (samples) as texels of the flow map (`pc/water.ts`), RGBA8 row-major into `into`: R and G the way
 * times the speed (x and z, 128 still), B the foam, A 255. `border` texels of still water round the land (the map is
 * `cols + 2 border` across): read clamped at its edge, the map has no flow past the land.
 */
export function flowTexels(flow: WaterFlow, rect: LandRect, into: Uint8Array, border = 0): void {
  const { cols } = flow;
  const across = cols + 2 * border;
  for (let j = Math.max(0, rect.j0); j < Math.min(flow.rows, rect.j1); j++) {
    for (let i = Math.max(0, rect.i0); i < Math.min(cols, rect.i1); i++) {
      const k = j * cols + i;
      const t = ((j + border) * across + i + border) * 4;
      const s = flow.speed[k];
      into[t] = Math.round(128 + 127 * flow.wx[k] * s);
      into[t + 1] = Math.round(128 + 127 * flow.wz[k] * s);
      into[t + 2] = Math.round(255 * flow.foam[k]);
      into[t + 3] = 255;
    }
  }
}

/** The samples `waterFlow` reads for `rect`, and so the ones whose flow a change there may change: `rect` out by `FLOW_MARGIN`, cut to the land. */
export function flowReach(land: Pick<Land, 'cols' | 'rows'>, rect: LandRect): LandRect {
  return { i0: Math.max(0, rect.i0 - FLOW_MARGIN), j0: Math.max(0, rect.j0 - FLOW_MARGIN), i1: Math.min(land.cols, rect.i1 + FLOW_MARGIN), j1: Math.min(land.rows, rect.j1 + FLOW_MARGIN) };
}

