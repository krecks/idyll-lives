/**
 * The land brushes: what one stamp of a tool does to the height field (`land.ts`), as the player
 * drags it over the land, in step 1 and in Build mode's Land tab. The editor (`landEdit.ts`) lays
 * the stamps along a stroke and keeps its undo; these work on a bare `Land` too, headless.
 *
 * Every tool works through a falloff `(1 - (r/R)²)²` over its radius R, scaled at each sample by
 * how far the sample may change (`free`: 0 on built land, 1 well clear of it). The water tools
 * also ask whether a sample may be wet (`wettable`): water keeps off lots and roads. After every
 * stamp the water settles (`settleWater`): a level stays only where it stands over its ground,
 * and dry ground lowered beside water under its level fills from it.
 *
 * A brush `POINT` m across or narrower is a point brush: the falloff is left out, and a stamp
 * changes the one sample nearest the pointer (a cover tool, the one cover cell under it), so the
 * player can touch the land one point at a time.
 */

import { COVER, COVER_CELL, coverSpan, heightAt, LAND_CELL, NONE, SEA_CM, wetAt, type Land, type LandRect } from './land';
import { ridged, smoothstep } from './noise';

/**
 * A land tool. Shape: `raise`, `lower`, `smooth`, `flatten`. Water: `lake` (or the sea pushed
 * inland, started on it), `river`, `dry`. Cover: `woods`, `meadow`, `fields`, `clear`. Mountains:
 * `peak`, `range`, `level`.
 */
export type LandTool = 'raise' | 'lower' | 'smooth' | 'flatten' | 'lake' | 'river' | 'dry' | 'woods' | 'meadow' | 'fields' | 'clear' | 'peak' | 'range' | 'level';

/** A tool as the palette set it: its radius (m), its strength (0..1) and, for `peak` and `range`, its height (m). */
export interface Brush {
  tool: LandTool;
  radius: number;
  strength: number;
  height?: number;
}

/** A brush this wide (m across, twice its radius) or narrower is a point brush. */
export const POINT = 4;

/** Whether `brush` is a point brush (`POINT`). */
export const isPoint = (brush: Pick<Brush, 'radius'>) => brush.radius * 2 <= POINT;

/**
 * The grid a point brush of `tool` snaps to: the samples (every `LAND_CELL` m from 0) for most
 * tools, the cover cells' middles (every `COVER_CELL` m from half a cell in) for the cover tools.
 */
export function pointGrid(tool: LandTool): { unit: number; offset: number } {
  return TOOL_KIND[tool] === 'cover' ? { unit: COVER_CELL, offset: COVER_CELL / 2 } : { unit: LAND_CELL, offset: 0 };
}

/** Where `brush` works with the pointer at (`x`, `z`): a point brush's grid point nearest it, else the pointer itself. */
export function aimOf(brush: Brush, x: number, z: number): [number, number] {
  if (!isPoint(brush)) return [x, z];
  const { unit, offset } = pointGrid(brush.tool);
  return [offset + Math.round((x - offset) / unit) * unit, offset + Math.round((z - offset) / unit) * unit];
}

/** One stamp along a stroke: where (world m) and the seconds since the last one. */
export interface Stamp {
  x: number;
  z: number;
  dt: number;
}

/** How far sample (i, j) may change: 0 on built land, 1 well clear of it. */
export type Free = (i: number, j: number) => number;
/** Whether sample (i, j) may be under water. */
export type Wettable = (i: number, j: number) => boolean;

/** The tools' groups: what they change, and what they are charged at (`landEdit.ts` `cost`). */
export const TOOL_KIND: Record<LandTool, 'earth' | 'water' | 'cover' | 'mountain'> = {
  raise: 'earth',
  lower: 'earth',
  smooth: 'earth',
  flatten: 'earth',
  lake: 'water',
  river: 'water',
  dry: 'water',
  woods: 'cover',
  meadow: 'cover',
  fields: 'cover',
  clear: 'cover',
  peak: 'mountain',
  range: 'mountain',
  level: 'mountain',
};

/** The cover each cover tool paints. */
export const COVER_OF: Partial<Record<LandTool, number>> = { woods: COVER.woods, meadow: COVER.meadow, fields: COVER.fields, clear: COVER.open };

/** Raise and lower move the land this fast (m/s) at the brush's middle, at strength 0 and 1. */
export const RATE: [number, number] = [0.5, 8];
/** A lake's level, started on dry land: the ground at the press less this (m). */
export const LAKE_BELOW = 0.1;
/** A river's level: never above the ground under it less this (m). */
export const RIVER_BELOW = 0.4;
/** A river stops where keeping its level would cut deeper into the land than this (m). */
export const RIVER_CUT = 6;
/** A lake's or a river's bank: dry ground round its water is raised this far over its level (m), in a ring this wide (m, at least; a quarter of the radius on wide ones). */
export const BANK = 0.25;
export const BANK_RING = 2.5;
/** The deepest a lake or a river is dug, at strength 0 and 1 (m; the sea sheet's own bed goes no deeper than 4). */
export const DEPTH: [number, number] = [0.5, 4];
/** `dry` fills a bed this far over its old water's level (m). */
export const DRY_OVER = 0.05;
/** A mountain's height (m) when the brush gives none. */
export const MOUNTAIN_HEIGHT = 40;

/** Why a stroke couldn't do something, for the palette's hint line. */
export const HINTS = {
  water: 'Water keeps 3 m off lots and clear of roads.',
  river: `The river would cut more than ${RIVER_CUT} m into the land, so it stops here.`,
} as const;

/**
 * What a stroke keeps from one stamp to the next (`strokeOf` makes one; `stamp` fills it in).
 * `touched` is the samples changed so far; `level` the water's level (cm) for `lake` and
 * `river`; `target` the height at the press (cm) for `flatten`; `path` the stamps' points (the
 * dragged line of `range`); `stopped` and `hint` say why it stopped or left something out.
 */
export interface StrokeState {
  brush: Brush;
  /** Everything the stroke has touched so far (samples), or null before its first stamp. */
  touched: LandRect | null;
  /** Stamps so far. */
  stamps: number;
  /** The heights and water levels before the stroke: a river reads the ground it cuts into from them, and meets the water there was. */
  base: { h: Int16Array; wl: Int16Array } | null;
  /** `flatten`: the height at the press (cm). */
  target?: number;
  /** `lake`, `river`: the water's level (cm), never rising along a river. */
  level?: number;
  /** `lake`: started on the sea, so it pushes the sea inland. */
  sea?: boolean;
  /** The points stamped (m): a range's line, a peak's last point. */
  path: [number, number][];
  /** Set once the stroke goes no further (a river cut too deep), with why. */
  stopped?: string;
  /** Something the stroke refused (water by a lot or a road), with why. */
  hint?: string;
  /** The mountains' ridged noise seed, from where the stroke began. */
  seed: number;
}

/**
 * A new stroke of `brush`. `base`: the land before it (the editor passes its copy, kept anyway
 * for undo); a river with none copies `land`'s heights and water itself.
 */
export function strokeOf(brush: Brush, land?: Land, base?: { h: Int16Array; wl: Int16Array }): StrokeState {
  const own = !base && brush.tool === 'river' && land ? { h: land.h.slice(), wl: land.wl.slice() } : null;
  return { brush, touched: null, stamps: 0, base: base ?? own, path: [], seed: 0 };
}

const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
/** A height or level (cm) kept within an Int16 and off `NONE`. */
const cmOf = (v: number) => Math.max(-32767, Math.min(32767, Math.round(v)));
/** The brush's falloff at `r` of radius `R`. */
export const falloff = (r: number, R: number) => (r >= R ? 0 : (1 - (r / R) ** 2) ** 2);

/** The union of two rectangles (either may be null). */
export function unionRect(a: LandRect | null, b: LandRect | null): LandRect | null {
  if (!a) return b;
  if (!b) return a;
  return { i0: Math.min(a.i0, b.i0), j0: Math.min(a.j0, b.j0), i1: Math.max(a.i1, b.i1), j1: Math.max(a.j1, b.j1) };
}

/** The samples within `reach` m of (`x`, `z`), as a rectangle cut to the land; null if none. */
export function rectAround(land: Land, x: number, z: number, reach: number): LandRect | null {
  const i0 = Math.max(0, Math.ceil((x - reach) / LAND_CELL));
  const j0 = Math.max(0, Math.ceil((z - reach) / LAND_CELL));
  const i1 = Math.min(land.cols, Math.floor((x + reach) / LAND_CELL) + 1);
  const j1 = Math.min(land.rows, Math.floor((z + reach) / LAND_CELL) + 1);
  return i0 < i1 && j0 < j1 ? { i0, j0, i1, j1 } : null;
}

/** A sample's index, the nearest to world point (`x`, `z`). */
const nearestSample = (land: Land, x: number, z: number) => {
  const i = Math.min(land.cols - 1, Math.max(0, Math.round(x / LAND_CELL)));
  const j = Math.min(land.rows - 1, Math.max(0, Math.round(z / LAND_CELL)));
  return j * land.cols + i;
};

/** A height (m) between the samples of `h` (bilinear, as `heightAt`). */
function heightIn(land: Land, h: Int16Array, x: number, z: number): number {
  return h === land.h ? heightAt(land, x, z) : heightAt({ ...land, h }, x, z);
}

/**
 * One stamp of `brush` at `at` on `land`: changes the land in place and returns the rectangle of
 * samples it changed (null: none). `free(i, j)` is how far sample (i, j) may change (0 on built
 * land, 1 well clear of it); `wettable(i, j)` whether it may be under water (every sample if not
 * given). `peak` and `range` only note the point here: they are raised once, on release
 * (`placeMountain`), with `mountainGhost` to draw while dragging. A point brush works at its grid
 * point nearest `at` (`aimOf`), on that one sample or cover cell at full strength.
 */
export function stamp(land: Land, brush: Brush, at: Stamp, stroke: StrokeState, free: Free, wettable?: Wettable): LandRect | null {
  if (stroke.stopped) return null;
  const [x, z] = aimOf(brush, at.x, at.z);
  if (!stroke.stamps) stroke.seed = (Math.imul(Math.round(x), 73856093) ^ Math.imul(Math.round(z), 19349663)) >>> 0;
  stroke.stamps++;
  stroke.path.push([x, z]);
  const tool = brush.tool;
  // (A point brush's reach is half a sample, so the one sample at its middle is all it finds, at
  // a falloff of 1; a cover tool's paints the cell under it alone.)
  const R = isPoint(brush) ? LAND_CELL / 2 : Math.max(LAND_CELL, brush.radius);
  const s = clamp01(brush.strength);
  let changed: LandRect | null;
  switch (tool) {
    case 'raise':
    case 'lower':
    case 'flatten':
      changed = shift(land, x, z, R, s, Math.max(0, at.dt), tool, stroke, free);
      break;
    case 'smooth':
    case 'level':
      changed = blend(land, x, z, R, s, Math.max(0, at.dt), tool === 'level', free);
      break;
    case 'lake':
    case 'river':
      changed = water(land, x, z, R, s, tool, stroke, free, wettable);
      break;
    case 'dry':
      changed = dry(land, x, z, R, free);
      break;
    case 'peak':
    case 'range':
      return null;
    default:
      return paint(land, x, z, isPoint(brush) ? 0 : R, COVER_OF[tool] ?? COVER.open, stroke, free);
  }
  if (!changed) return null;
  settleWater(land, changed, wettable);
  stroke.touched = unionRect(stroke.touched, changed);
  return changed;
}

/** `raise`, `lower` and `flatten` (towards the height at the press), at the strength's rate. */
function shift(land: Land, x: number, z: number, R: number, s: number, dt: number, tool: 'raise' | 'lower' | 'flatten', stroke: StrokeState, free: Free): LandRect | null {
  const rect = rectAround(land, x, z, R);
  if (!rect || dt <= 0) return null;
  const rate = (RATE[0] + (RATE[1] - RATE[0]) * s) * dt * 100;
  if (tool === 'flatten') stroke.target ??= Math.round(heightAt(land, x, z) * 100);
  const { cols, h } = land;
  let any = false;
  for (let j = rect.j0; j < rect.j1; j++) {
    for (let i = rect.i0; i < rect.i1; i++) {
      const f = falloff(Math.hypot(i * LAND_CELL - x, j * LAND_CELL - z), R);
      if (f <= 0) continue;
      const fr = free(i, j);
      if (fr <= 0) continue;
      const k = j * cols + i;
      const step = rate * f * fr;
      const next = tool === 'raise' ? h[k] + step : tool === 'lower' ? h[k] - step : h[k] + Math.max(-step, Math.min(step, stroke.target! - h[k]));
      const v = cmOf(next);
      if (v !== h[k]) {
        h[k] = v;
        any = true;
      }
    }
  }
  return any ? rect : null;
}

/**
 * `smooth` (towards the mean of each sample's 3 × 3) and `level` (towards the mean of the land
 * within the radius round each sample: its low-passed surroundings, from a summed-area table).
 */
function blend(land: Land, x: number, z: number, R: number, s: number, dt: number, wide: boolean, free: Free): LandRect | null {
  const rect = rectAround(land, x, z, R);
  if (!rect || dt <= 0) return null;
  const { cols, rows, h } = land;
  const reach = wide ? Math.max(1, Math.round(R / LAND_CELL)) : 1;
  // The heights round the rectangle (as far as the window reaches), summed from its corner.
  const [a0, b0] = [Math.max(0, rect.i0 - reach), Math.max(0, rect.j0 - reach)];
  const [a1, b1] = [Math.min(cols, rect.i1 + reach), Math.min(rows, rect.j1 + reach)];
  const w = a1 - a0 + 1;
  const sat = new Float64Array(w * (b1 - b0 + 1));
  for (let j = b0; j < b1; j++) {
    let row = 0;
    for (let i = a0; i < a1; i++) {
      row += h[j * cols + i];
      sat[(j - b0 + 1) * w + (i - a0 + 1)] = sat[(j - b0) * w + (i - a0 + 1)] + row;
    }
  }
  const mean = (i: number, j: number) => {
    const [l, t] = [Math.max(a0, i - reach) - a0, Math.max(b0, j - reach) - b0];
    const [r, b] = [Math.min(a1, i + reach + 1) - a0, Math.min(b1, j + reach + 1) - b0];
    return (sat[b * w + r] - sat[t * w + r] - sat[b * w + l] + sat[t * w + l]) / ((r - l) * (b - t));
  };
  const pull = Math.min(1, (wide ? 0.5 + 2.5 * s : 2 + 18 * s) * dt);
  let any = false;
  for (let j = rect.j0; j < rect.j1; j++) {
    for (let i = rect.i0; i < rect.i1; i++) {
      const f = falloff(Math.hypot(i * LAND_CELL - x, j * LAND_CELL - z), R);
      if (f <= 0) continue;
      const fr = free(i, j);
      if (fr <= 0) continue;
      const k = j * cols + i;
      const v = cmOf(h[k] + (mean(i, j) - h[k]) * pull * f * fr);
      if (v !== h[k]) {
        h[k] = v;
        any = true;
      }
    }
  }
  return any ? rect : null;
}

/**
 * `lake` and `river`: water at the stroke's level within the radius, its bed dug as the sea
 * sheet's is (deeper the further from the shore: `0.22e + 0.004e²` m, `e` m in, down to the
 * strength's depth), and a low bank raised round it where the ground falls away below its level.
 * A lake's level is the water's it began on (the sea's: the sea pushed inland), else the ground
 * at the press less `LAKE_BELOW`. A river's never rises: each stamp the lower of the last and the
 * ground under it (before the stroke) less `RIVER_BELOW`; meeting lower water it takes that
 * water's level; where it would stand more than `RIVER_CUT` m under the ground, it stops.
 */
function water(land: Land, x: number, z: number, R: number, s: number, tool: 'lake' | 'river', stroke: StrokeState, free: Free, wettable?: Wettable): LandRect | null {
  const { cols, h, wl } = land;
  const base = stroke.base ?? { h, wl };
  const press = nearestSample(land, x, z);
  const wetBefore = (k: number) => base.wl[k] !== NONE && base.wl[k] > base.h[k];
  if (stroke.level === undefined) {
    if (wetBefore(press)) {
      stroke.level = base.wl[press];
      stroke.sea = tool === 'lake' && base.wl[press] === SEA_CM;
    } else stroke.level = cmOf(heightIn(land, base.h, x, z) * 100 - (tool === 'lake' ? LAKE_BELOW : RIVER_BELOW) * 100);
  }
  const ring = Math.max(BANK_RING, R / 4);
  const rect = rectAround(land, x, z, R + ring);
  if (!rect) return null;
  if (tool === 'river') {
    // (Over water there was, no lower than its level; over dry land, under the ground.)
    const ground = Math.round(heightIn(land, base.h, x, z) * 100);
    let level = Math.min(stroke.level, wetBefore(press) ? base.wl[press] : ground - RIVER_BELOW * 100);
    // (Water there was, lower than the river, within it: the river runs into it at its level.)
    for (let j = rect.j0; j < rect.j1; j++) {
      for (let i = rect.i0; i < rect.i1; i++) {
        const k = j * cols + i;
        if (wetBefore(k) && base.wl[k] < level && Math.hypot(i * LAND_CELL - x, j * LAND_CELL - z) <= R) level = base.wl[k];
      }
    }
    if (!wetBefore(press) && ground - level > RIVER_CUT * 100) {
      stroke.stopped = HINTS.river;
      return null;
    }
    stroke.level = level;
  }
  const L = stroke.level;
  const deepest = DEPTH[0] + (DEPTH[1] - DEPTH[0]) * s;
  let any = false;
  for (let j = rect.j0; j < rect.j1; j++) {
    for (let i = rect.i0; i < rect.i1; i++) {
      const r = Math.hypot(i * LAND_CELL - x, j * LAND_CELL - z);
      if (r > R + ring) continue;
      const fr = free(i, j);
      if (fr <= 0) continue;
      const k = j * cols + i;
      const [h0, w0] = [h[k], wl[k]];
      if (r <= R) {
        if (wettable && !wettable(i, j)) {
          stroke.hint = HINTS.water;
          continue;
        }
        const e = R - r;
        const bed = L - Math.round(Math.min(deepest, 0.22 * e + 0.004 * e * e) * 100);
        if (h[k] > bed) h[k] = cmOf(h[k] + (bed - h[k]) * fr);
        if (h[k] < L) wl[k] = L;
      } else if (!wetAt(land, i, j) && h[k] < L + BANK * 100) h[k] = cmOf(h[k] + (L + BANK * 100 - h[k]) * fr);
      if (h[k] !== h0 || wl[k] !== w0) any = true;
    }
  }
  return any ? rect : null;
}

/** `dry`: the water within the radius gone, its bed filled to just over its old level. */
function dry(land: Land, x: number, z: number, R: number, free: Free): LandRect | null {
  const rect = rectAround(land, x, z, R);
  if (!rect) return null;
  const { cols, h, wl } = land;
  let any = false;
  for (let j = rect.j0; j < rect.j1; j++) {
    for (let i = rect.i0; i < rect.i1; i++) {
      const k = j * cols + i;
      if (wl[k] === NONE || Math.hypot(i * LAND_CELL - x, j * LAND_CELL - z) > R) continue;
      const fr = free(i, j);
      if (fr <= 0) continue;
      const top = wl[k] + DRY_OVER * 100;
      if (h[k] < top) h[k] = cmOf(h[k] + (top - h[k]) * fr);
      if (h[k] >= wl[k]) wl[k] = NONE;
      any = true;
    }
  }
  return any ? rect : null;
}

/**
 * The cover tools: `cover` on every cover cell where the falloff is over a half, and always on the
 * cell under the brush's middle (so a small brush never paints nothing; `R` 0 paints that cell
 * alone), where the land is free to change.
 */
function paint(land: Land, x: number, z: number, R: number, cover: number, stroke: StrokeState, free: Free): LandRect | null {
  const [cw, cd] = [coverSpan(land.cols), coverSpan(land.rows)];
  const [mc, mr] = [Math.floor(x / COVER_CELL), Math.floor(z / COVER_CELL)];
  const c0 = Math.max(0, Math.min(mc, Math.floor((x - R) / COVER_CELL)));
  const r0 = Math.max(0, Math.min(mr, Math.floor((z - R) / COVER_CELL)));
  const c1 = Math.min(cw, Math.max(mc + 1, Math.ceil((x + R) / COVER_CELL)));
  const r1 = Math.min(cd, Math.max(mr + 1, Math.ceil((z + R) / COVER_CELL)));
  let out: LandRect | null = null;
  for (let r = r0; r < r1; r++) {
    for (let c = c0; c < c1; c++) {
      const [cx, cz] = [(c + 0.5) * COVER_CELL, (r + 0.5) * COVER_CELL];
      if ((c !== mc || r !== mr) && falloff(Math.hypot(cx - x, cz - z), R) <= 0.5) continue;
      const [i, j] = [Math.min(land.cols - 1, Math.round(cx / LAND_CELL)), Math.min(land.rows - 1, Math.round(cz / LAND_CELL))];
      if (free(i, j) <= 0.5 || land.cover[r * cw + c] === cover) continue;
      land.cover[r * cw + c] = cover;
      out = unionRect(out, coverSamples(land, { i0: c, j0: r, i1: c + 1, j1: r + 1 }));
    }
  }
  stroke.touched = unionRect(stroke.touched, out);
  return out;
}

/** The samples a rectangle of cover cells lies over (each cell's corners). */
export function coverSamples(land: Land, cells: LandRect): LandRect {
  const k = COVER_CELL / LAND_CELL;
  return { i0: cells.i0 * k, j0: cells.j0 * k, i1: Math.min(land.cols, cells.i1 * k + 1), j1: Math.min(land.rows, cells.j1 * k + 1) };
}

/** The cover cells a rectangle of samples reaches. */
export function coverCells(land: Land, rect: LandRect): LandRect {
  const k = COVER_CELL / LAND_CELL;
  return { i0: Math.floor(rect.i0 / k), j0: Math.floor(rect.j0 / k), i1: Math.min(coverSpan(land.cols), Math.ceil(rect.i1 / k)), j1: Math.min(coverSpan(land.rows), Math.ceil(rect.j1 / k)) };
}

/**
 * The water in `rect` (and only there) settled after its ground changed: a level stays only where
 * it stands over its ground (raised sea or lake dries out), and dry ground under the level of the
 * water beside it fills from it (lowered by the sea, it becomes sea), spreading through the
 * rectangle and no further, and never onto ground `wettable` refuses. Returns `rect`.
 */
export function settleWater(land: Land, rect: LandRect, wettable?: Wettable): LandRect {
  const { cols, rows, h, wl } = land;
  const { i0, j0, i1, j1 } = rect;
  for (let j = j0; j < j1; j++) for (let k = j * cols + i0, end = j * cols + i1; k < end; k++) if (wl[k] !== NONE && wl[k] <= h[k]) wl[k] = NONE;
  // From the wet samples in it and round it, into the dry ground in it lower than their water.
  const queue: number[] = [];
  for (let j = Math.max(0, j0 - 1); j < Math.min(rows, j1 + 1); j++) {
    for (let i = Math.max(0, i0 - 1); i < Math.min(cols, i1 + 1); i++) if (wetAt(land, i, j)) queue.push(j * cols + i);
  }
  for (let q = 0; q < queue.length; q++) {
    const k = queue[q];
    const [i, j] = [k % cols, Math.floor(k / cols)];
    for (const [ni, nj] of [[i - 1, j], [i + 1, j], [i, j - 1], [i, j + 1]] as const) {
      if (ni < i0 || nj < j0 || ni >= i1 || nj >= j1) continue;
      const n = nj * cols + ni;
      if (wl[n] !== NONE || h[n] >= wl[k] || (wettable && !wettable(ni, nj))) continue;
      wl[n] = wl[k];
      queue.push(n);
    }
  }
  return rect;
}

// ---- Mountains -------------------------------------------------------------------------------

/** How a mountain falls away from its crest (`t`: the distance over the radius, 0..1). */
const profile = (t: number) => (t >= 1 ? 0 : 1 - smoothstep(0, 1, t));

/** A peak (one point) or a range (along the dragged line): the height it adds (m) at a point, and the samples it reaches. */
export interface Mountain {
  lift(x: number, z: number): number;
  /** World bounds (m): x0, z0, x1, z1. */
  bounds: [number, number, number, number];
}

/**
 * The mountain a `peak` or `range` stroke raises: `H × g(d/R) × (0.6 + 0.4 × ridged)`, `d` the
 * distance from the peak (the stroke's last point) or from the range's line, `H` the brush's
 * height, `g` falling from 1 at the crest to 0 at the radius. Null before the stroke has a point.
 */
export function mountainOf(stroke: StrokeState): Mountain | null {
  const { brush, path } = stroke;
  if (!path.length) return null;
  const line = brush.tool === 'range' ? path : [path[path.length - 1]];
  const R = Math.max(LAND_CELL * 2, brush.radius);
  const H = brush.height ?? MOUNTAIN_HEIGHT;
  const k = 1.6 / R;
  const seed = stroke.seed;
  const xs = line.map((p) => p[0]);
  const zs = line.map((p) => p[1]);
  const distance = (x: number, z: number) => {
    let best = Math.hypot(x - line[0][0], z - line[0][1]);
    for (let n = 1; n < line.length; n++) {
      const [[ax, az], [bx, bz]] = [line[n - 1], line[n]];
      const [dx, dz] = [bx - ax, bz - az];
      const t = clamp01(((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz || 1));
      best = Math.min(best, Math.hypot(x - ax - dx * t, z - az - dz * t));
    }
    return best;
  };
  return {
    lift: (x, z) => {
      const g = profile(distance(x, z) / R);
      return g > 0 ? H * g * (0.6 + 0.4 * ridged(x * k, z * k, seed)) : 0;
    },
    bounds: [Math.min(...xs) - R, Math.min(...zs) - R, Math.max(...xs) + R, Math.max(...zs) + R],
  };
}

/** A mountain's ghost to draw while dragging: the height it would add (m) every `step` m over `cols` × `rows` points from (`x0`, `z0`). */
export interface MountainGhost {
  x0: number;
  z0: number;
  step: number;
  cols: number;
  rows: number;
  lift: Float32Array;
}

/** The ghost of the mountain a `peak` or `range` stroke would raise (`mountainOf`), every `step` m; null before it has a point. */
export function mountainGhost(stroke: StrokeState, step = 4): MountainGhost | null {
  const m = mountainOf(stroke);
  if (!m) return null;
  const [x0, z0, x1, z1] = m.bounds;
  const cols = Math.ceil((x1 - x0) / step) + 1;
  const rows = Math.ceil((z1 - z0) / step) + 1;
  const lift = new Float32Array(cols * rows);
  for (let r = 0; r < rows; r++) for (let c = 0; c < cols; c++) lift[r * cols + c] = m.lift(x0 + c * step, z0 + r * step);
  return { x0, z0, step, cols, rows, lift };
}

/** Raises the mountain of a `peak` or `range` stroke on `land` (scaled by `free`), on release; the samples it changed, or null. */
export function placeMountain(land: Land, stroke: StrokeState, free: Free, wettable?: Wettable): LandRect | null {
  const m = mountainOf(stroke);
  if (!m) return null;
  const [x0, z0, x1, z1] = m.bounds;
  const i0 = Math.max(0, Math.ceil(x0 / LAND_CELL));
  const j0 = Math.max(0, Math.ceil(z0 / LAND_CELL));
  const i1 = Math.min(land.cols, Math.floor(x1 / LAND_CELL) + 1);
  const j1 = Math.min(land.rows, Math.floor(z1 / LAND_CELL) + 1);
  if (i0 >= i1 || j0 >= j1) return null;
  let any = false;
  for (let j = j0; j < j1; j++) {
    for (let i = i0; i < i1; i++) {
      const fr = free(i, j);
      if (fr <= 0) continue;
      const lift = m.lift(i * LAND_CELL, j * LAND_CELL) * fr;
      if (lift <= 0) continue;
      const k = j * land.cols + i;
      const v = cmOf(land.h[k] + lift * 100);
      if (v !== land.h[k]) {
        land.h[k] = v;
        any = true;
      }
    }
  }
  if (!any) return null;
  const rect = settleWater(land, { i0, j0, i1, j1 }, wettable);
  stroke.touched = unionRect(stroke.touched, rect);
  return rect;
}
