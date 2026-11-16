/**
 * The land brushes' debug hook (`window.__land`, with `?debug`): scripted strokes through the same
 * editor, drawing and (in the game) sending as the player's, for benches (`tools/bench/land.mjs`)
 * and tests in the browser. Step 1's Shape panel and Build mode's Land tab each put theirs up while
 * they have a land to shape.
 *
 * - `stroke(tool, points, radius, strength, height?)`: a stroke along `points` (town metres), a
 *   point a frame; resolves with the frames' times, each frame's stamps' time and what it changed.
 * - `undo()`, `redo()`: as the palette's buttons.
 * - `stats()`: the editor's stacks, the world's size, what has changed.
 * - `free(x, z)`, `heightAt(x, z)`, `wet(x, z)`: the land at a point; `pick(x, y)`: the town point
 *   under a screen point (CSS pixels).
 * - `findFree(radius, near?)`: the nearest point to `near` (else the world's middle) where a brush of
 *   `radius` lies wholly on free land.
 * - `findWater(near?)`: the nearest water to `near`.
 * - `simWet(x, z)` (in the game): whether the simulation walks round the 1 m tile there as water.
 * - `pan(dx, dz)`, `zoom(factor)` (step 1): the town's camera moved as the pointer moves it.
 */

import { heightAt, LAND_CELL, wetAt, type LandPatch } from '../../game/land';
import type { LandTool } from '../../game/landBrush';
import type { LandEditor } from '../../game/landEdit';
import type { LandSurface } from './landStroke';

export interface LandHookHost {
  editor(): LandEditor | null;
  surface(): LandSurface | null;
  /** The land changed by a scripted stroke (what shows the editor's stacks reads them again). */
  changed?(): void;
  /** A scripted stroke ended with `patch` (the game sends it; resolves once taken or refused). */
  ended?(patch: LandPatch): Promise<unknown>;
  undo(): Promise<unknown> | unknown;
  redo(): Promise<unknown> | unknown;
  /** Step 1's camera: panned (m over the ground) and zoomed (a factor). */
  camera?: { pan(dx: number, dz: number): void; zoom(factor: number): void };
  /** The simulation's water mask (the game's). */
  simWater?(): Promise<{ water: number[]; width: number; depth: number }>;
}

declare global {
  interface Window {
    __land?: unknown;
  }
}

const DEBUG = typeof location !== 'undefined' && new URLSearchParams(location.search).has('debug');

const frame = () => new Promise<number>((resolve) => requestAnimationFrame(resolve));

/** Puts `window.__land` up for `host` (with `?debug`); returns what takes it down again. */
export function installLandHook(host: LandHookHost): () => void {
  if (!DEBUG) return () => {};
  const need = (): LandEditor => {
    const e = host.editor();
    if (!e) throw new Error('no land to shape yet');
    return e;
  };
  const sample = (e: LandEditor, x: number, z: number) => {
    const { cols, rows } = e.land;
    return [Math.min(cols - 1, Math.max(0, Math.round(x / LAND_CELL))), Math.min(rows - 1, Math.max(0, Math.round(z / LAND_CELL)))] as const;
  };
  const api = {
    async stroke(tool: LandTool, points: [number, number][], radius: number, strength = 0.5, height?: number) {
      const e = need();
      const surface = host.surface();
      if (!points.length) throw new Error('no points');
      const frames: number[] = [];
      const stamps: number[] = [];
      let before = await frame();
      let t = performance.now();
      let rect = e.begin({ tool, radius, strength, ...(height ? { height } : {}) }, points[0][0], points[0][1]);
      if (rect) surface?.changed(rect, false);
      stamps.push(performance.now() - t);
      for (let k = 1; k < points.length; k++) {
        const now = await frame();
        frames.push(now - before);
        t = performance.now();
        rect = e.move(points[k][0], points[k][1], Math.min(0.1, (now - before) / 1000));
        if (rect) surface?.changed(rect, false);
        stamps.push(performance.now() - t);
        before = now;
      }
      const patch = e.end();
      if (patch) surface?.changed(patch.rect, true);
      host.changed?.();
      const after = await frame();
      frames.push(after - before);
      let refused: string | null = null;
      if (patch && host.ended) {
        try {
          await host.ended(patch);
        } catch (err) {
          refused = err instanceof Error ? err.message : String(err);
        }
      }
      return { changed: !!patch, rect: patch?.rect ?? null, hint: e.hint, refused, frames, stamps };
    },
    async undo() {
      await host.undo();
      return need().stats();
    },
    async redo() {
      await host.redo();
      return need().stats();
    },
    stats() {
      const e = need();
      return { ...e.stats(), cols: e.land.cols, rows: e.land.rows, changed: e.changed() };
    },
    pick(x: number, y: number) {
      return host.surface()?.pick(x, y) ?? null;
    },
    free(x: number, z: number) {
      const e = need();
      return e.free(...sample(e, x, z));
    },
    heightAt(x: number, z: number) {
      return heightAt(need().land, x, z);
    },
    wet(x: number, z: number) {
      const e = need();
      return wetAt(e.land, ...sample(e, x, z));
    },
    findFree(radius: number, near?: [number, number]) {
      const e = need();
      const { cols, rows } = e.land;
      const [cx, cz] = near ?? [(cols * LAND_CELL) / 2, (rows * LAND_CELL) / 2];
      const r = Math.ceil(radius / LAND_CELL);
      let best: [number, number] | null = null;
      let bestD = Infinity;
      const stride = Math.max(2, Math.round(r / 2));
      for (let j = r; j < rows - r; j += stride) {
        for (let i = r; i < cols - r; i += stride) {
          const d = Math.hypot(i * LAND_CELL - cx, j * LAND_CELL - cz);
          if (d >= bestD) continue;
          let ok = true;
          for (let b = -r; b <= r && ok; b += 2) for (let a = -r; a <= r && ok; a += 2) if (a * a + b * b <= r * r && (e.free(i + a, j + b) < 1 || wetAt(e.land, i + a, j + b))) ok = false;
          if (!ok) continue;
          best = [i * LAND_CELL, j * LAND_CELL];
          bestD = d;
        }
      }
      return best;
    },
    pan(dx: number, dz: number) {
      host.camera?.pan(dx, dz);
    },
    zoom(factor: number) {
      host.camera?.zoom(factor);
    },
    async simWet(x: number, z: number) {
      const mask = await host.simWater?.();
      if (!mask) return null;
      const k = Math.floor(z) * mask.width + Math.floor(x);
      let at = 0;
      for (let p = 0; p < mask.water.length; p += 2) {
        if (k < at + mask.water[p]) return mask.water[p + 1] !== 0;
        at += mask.water[p];
      }
      return false;
    },
    findWater(near?: [number, number]) {
      const e = need();
      const { cols, rows } = e.land;
      const [cx, cz] = near ?? [(cols * LAND_CELL) / 2, (rows * LAND_CELL) / 2];
      let best: [number, number] | null = null;
      let bestD = Infinity;
      for (let j = 0; j < rows; j += 2) {
        for (let i = 0; i < cols; i += 2) {
          if (!wetAt(e.land, i, j)) continue;
          const d = Math.hypot(i * LAND_CELL - cx, j * LAND_CELL - cz);
          if (d < bestD) [best, bestD] = [[i * LAND_CELL, j * LAND_CELL], d];
        }
      }
      return best;
    },
  };
  window.__land = api;
  return () => {
    if (window.__land === api) delete window.__land;
  };
}
