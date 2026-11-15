/**
 * The land's debug hook (`window.__landDebug`, dev server with `?debug`): scripted strokes on the land in view,
 * written straight into its height field and drawn through the renderer's land methods (`setLand`,
 * `landChanged`, `pickLand`, `setLandBrush`), stamp by stamp as a brush would, so the drawing's side can be seen
 * and timed without the brushes (`game/landBrush.ts`). Nothing outside `?debug` calls it.
 *
 * - `survey()`: the land in view as a height field (`landOf`), drawn from from now on (`setLand`).
 * - `mound`, `pond`, `river`, `paint`, `peak`: one stroke each (see each).
 * - `demo(x, z)`: all of them round a point: a mound with a pond on its top at its own level, a river, woods
 *   painted then half cleared, a peak; each stroke's stamps timed.
 * - `settled()`: once the strokes' work (shading, grass, woods) is done.
 * - `times()`: the land's perf spans so far.
 */

import { COVER, COVER_CELL, LAND_CELL, NONE, heightAt, type Land, type LandRect } from '../../game/land';
import { perfReport } from '../../debug/perf';
import type { LandCursor } from '../types';

/** What the hook drives: the renderer's land methods. */
interface LandHost {
  landOf(): Promise<Land>;
  setLand(land: Land | null): void;
  landChanged(rect: LandRect, done: boolean): void;
  pickLand(x: number, y: number): { x: number; y: number; z: number } | null;
  setLandBrush(cursor: LandCursor | null): void;
}

declare global {
  interface Window {
    __landDebug?: unknown;
  }
}

/** The kernel: 1 at the middle to 0 at `r`. */
const falloff = (d: number, r: number) => (d >= r ? 0 : (1 - (d / r) ** 2) ** 2);

export function installLandDebug(host: LandHost, queue: () => Promise<void>): void {
  let land: Land | null = null;
  const stamps: number[] = [];
  const need = (): Land => {
    if (!land) throw new Error('survey() first');
    return land;
  };
  /** The samples within `r` m of (x, z). */
  const around = (x: number, z: number, r: number): LandRect => {
    const L = need();
    return { i0: Math.max(0, Math.floor((x - r) / LAND_CELL)), j0: Math.max(0, Math.floor((z - r) / LAND_CELL)), i1: Math.min(L.cols, Math.ceil((x + r) / LAND_CELL) + 1), j1: Math.min(L.rows, Math.ceil((z + r) / LAND_CELL) + 1) };
  };
  const union = (a: LandRect | null, b: LandRect): LandRect => (a ? { i0: Math.min(a.i0, b.i0), j0: Math.min(a.j0, b.j0), i1: Math.max(a.i1, b.i1), j1: Math.max(a.j1, b.j1) } : { ...b });
  /** Each sample in `rect`: its index, where it stands and how far from (x, z). */
  const each = (rect: LandRect, x: number, z: number, fn: (k: number, px: number, pz: number, d: number) => void) => {
    const L = need();
    for (let j = rect.j0; j < rect.j1; j++) for (let i = rect.i0; i < rect.i1; i++) fn(j * L.cols + i, i * LAND_CELL, j * LAND_CELL, Math.hypot(i * LAND_CELL - x, j * LAND_CELL - z));
  };
  /** A stamp drawn and timed. */
  const stamp = (rect: LandRect) => {
    const t = performance.now();
    host.landChanged(rect, false);
    stamps.push(performance.now() - t);
  };
  const frame = () => new Promise((resolve) => requestAnimationFrame(resolve));
  /** A stroke: `steps` stamps of `fn` (each told its share), then done over all they reached. */
  const stroke = async (steps: number, fn: (step: number) => LandRect): Promise<LandRect> => {
    let all: LandRect | null = null;
    for (let s = 0; s < steps; s++) {
      const rect = fn(s);
      stamp(rect);
      all = union(all, rect);
      await frame();
    }
    host.landChanged(all!, true);
    return all!;
  };

  const api = {
    async survey(): Promise<{ cols: number; rows: number; ms: number }> {
      const t = performance.now();
      land = await host.landOf();
      const ms = performance.now() - t;
      host.setLand(land);
      await queue();
      return { cols: land.cols, rows: land.rows, ms };
    },
    /** Raises the land by `height` m round (x, z) within `r` m, over `steps` stamps. */
    mound(x: number, z: number, r: number, height: number, steps = 10): Promise<LandRect> {
      const L = need();
      return stroke(steps, () => {
        const rect = around(x, z, r);
        each(rect, x, z, (k, _px, _pz, d) => (L.h[k] = Math.min(32767, L.h[k] + Math.round((height * 100 * falloff(d, r)) / steps))));
        return rect;
      });
    },
    /**
     * A pond at its own level round (x, z): its level the land's at the middle less 0.1 m, its bed the sheet's
     * (0.22 m a metre in, more further), `depth` at most; a low bank raised round it where the land is lower.
     */
    pond(x: number, z: number, r: number, depth = 2): Promise<LandRect> {
      const L = need();
      const level = Math.round((heightAt(L, x, z) - 0.1) * 100);
      return stroke(1, () => {
        const rect = around(x, z, r + 8);
        each(rect, x, z, (k, _px, _pz, d) => {
          if (d < r) {
            const e = r - d;
            L.wl[k] = level;
            L.h[k] = Math.min(L.h[k], level - Math.round(Math.min(depth, 0.22 * e + 0.004 * e * e) * 100));
          } else if (L.h[k] < level + 20) L.h[k] = d < r + 3 ? level + 20 : Math.round(L.h[k] + (level + 20 - L.h[k]) * falloff(d - r - 3, 5));
        });
        return rect;
      });
    },
    /** A river along `points` (m), `width` wide, cut `depth` into the land, its level never rising along the way. */
    async river(points: [number, number][], width: number, depth = 1.5): Promise<LandRect> {
      const L = need();
      let level = Infinity;
      let all: LandRect | null = null;
      for (let p = 0; p + 1 < points.length; p++) {
        const [[ax, az], [bx, bz]] = [points[p], points[p + 1]];
        const n = Math.max(1, Math.ceil(Math.hypot(bx - ax, bz - az) / (width / 2)));
        for (let s = 0; s < n; s++) {
          const [x, z] = [ax + ((bx - ax) * s) / n, az + ((bz - az) * s) / n];
          level = Math.min(level, Math.round((heightAt(L, x, z) - 0.4) * 100));
          const rect = around(x, z, width / 2 + 2);
          each(rect, x, z, (k, _px, _pz, d) => {
            if (d >= width / 2) return;
            // (Meeting water, it takes that water's level.)
            if (L.wl[k] !== NONE && L.wl[k] > L.h[k]) level = Math.min(level, L.wl[k]);
            L.wl[k] = level;
            L.h[k] = Math.min(L.h[k], level - Math.round(depth * 100 * (1 - (d / (width / 2)) ** 2)));
          });
          stamp(rect);
          all = union(all, rect);
          await frame();
        }
      }
      host.landChanged(all!, true);
      return all!;
    },
    /** The cover (`COVER`'s name) painted within `r` m of (x, z). */
    paint(x: number, z: number, r: number, cover: keyof typeof COVER): Promise<LandRect> {
      const L = need();
      const cw = Math.ceil((L.cols * LAND_CELL) / COVER_CELL);
      const cd = L.cover.length / cw;
      return stroke(1, () => {
        for (let j = Math.max(0, Math.floor((z - r) / COVER_CELL)); j < Math.min(cd, Math.ceil((z + r) / COVER_CELL)); j++) {
          for (let i = Math.max(0, Math.floor((x - r) / COVER_CELL)); i < Math.min(cw, Math.ceil((x + r) / COVER_CELL)); i++) {
            if (Math.hypot((i + 0.5) * COVER_CELL - x, (j + 0.5) * COVER_CELL - z) < r) L.cover[j * cw + i] = COVER[cover];
          }
        }
        return around(x, z, r);
      });
    },
    /** A peak `height` m high round (x, z), `r` m across at its foot, rugged (placed whole, as on release). */
    peak(x: number, z: number, r: number, height: number): Promise<LandRect> {
      const L = need();
      return stroke(1, () => {
        const rect = around(x, z, r);
        each(rect, x, z, (k, px, pz, d) => {
          const g = Math.max(0, 1 - d / r) ** 1.6;
          const rough = 0.6 + 0.4 * Math.abs(Math.sin(px * 0.05) * Math.cos(pz * 0.043));
          L.h[k] = Math.min(32767, L.h[k] + Math.round(height * 100 * g * rough));
        });
        return rect;
      });
    },
    /** Each stroke round (x, z) (see the file's header); the stamps' times (ms) and the land's spans. */
    async demo(x: number, z: number): Promise<{ stamps: { n: number; mean: number; max: number }; spans: Record<string, { n: number; total: number; max: number }> }> {
      if (!land) await api.survey();
      stamps.length = 0;
      await api.mound(x - 70, z, 45, 9);
      await api.pond(x - 70, z, 13);
      await api.river([[x + 10, z + 70], [x + 90, z + 95], [x + 200, z + 150]], 14);
      await api.paint(x + 70, z - 70, 34, 'woods');
      await queue();
      await api.paint(x + 85, z - 70, 18, 'open');
      await api.peak(x - 170, z - 150, 85, 62);
      await queue();
      return { stamps: { n: stamps.length, mean: stamps.reduce((a, b) => a + b, 0) / Math.max(1, stamps.length), max: Math.max(0, ...stamps) }, spans: api.times() };
    },
    settled: (): Promise<void> => queue(),
    /** The land's perf spans so far, by name: how many, their total and the longest (ms). */
    times(): Record<string, { n: number; total: number; max: number }> {
      const out: Record<string, { n: number; total: number; max: number }> = {};
      for (const e of perfReport().entries) {
        if (!e.name.startsWith('land:')) continue;
        // (A stamp's by how many ground tiles it drew again.)
        const o = (out[e.name === 'land: stamp' ? `${e.name} (${e.detail})` : e.name] ??= { n: 0, total: 0, max: 0 });
        o.n++;
        o.total += e.ms;
        o.max = Math.max(o.max, e.ms);
      }
      return out;
    },
    pick: (x: number, y: number) => host.pickLand(x, y),
    brush: (cursor: LandCursor | null) => host.setLandBrush(cursor),
    get land(): Land | null {
      return land;
    },
  };
  window.__landDebug = api;
}
