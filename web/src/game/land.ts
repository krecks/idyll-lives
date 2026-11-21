/**
 * The land as a height field: a height and a water level every `LAND_CELL` m, and what covers the
 * ground every `COVER_CELL` m, over the whole world from its corner. Sample (i, j) stands at
 * (i × `LAND_CELL`, j × `LAND_CELL`) m; past the last one the land holds the edge's height.
 *
 * The town file and saves carry it as `meta.land` (`LandFile`): `encodeLand` cuts it into tiles of
 * `TILE` × `TILE` samples, each a short base64 string, so a stroke of the land brushes sends only
 * the tiles it touched (`tilesOf`), and `decodeLand` reads it back.
 *
 * Where a river was drawn (the River brush), each of its samples says which way the water runs
 * (`Land.flow`, `flowOf`): the renderer's flowing water (`render/shared/waterFlow.ts`) reads it, and
 * works the rest out (a generated river, water stepping down from one level to the next).
 */

import type { LandFile } from '../core/protocol';
import { rle } from './landMasks';
import * as pmath from './pmath';

/** The height field's grain (m): the renderer's (its built cells, its finest ground quads). */
export const LAND_CELL = 2;
/** The cover's grain (m): woods, meadow and fields. */
export const COVER_CELL = 4;
/** A water level where there is no water (dry land). */
export const NONE = -32768;
/** The sea's level (cm): the water sheet the renderer draws (`WATER_Y`); water at this level is drawn by it. */
export const SEA_CM = -35;
/** Samples a side of a tile of `LandFile`. */
export const TILE = 64;

/** What covers the ground in a cover cell (`Land.cover`). */
export const COVER = { open: 0, woods: 1, meadow: 2, fields: 3 } as const;

/** The land of a world (see the file's comment). Every array is row-major from the world's corner. */
export interface Land {
  /** Samples across (x) and down (z). */
  cols: number;
  rows: number;
  /** The heights (cm). */
  h: Int16Array;
  /** The water levels (cm), `NONE` where dry: a sample is wet where its water stands over its ground (`wetAt`). */
  wl: Int16Array;
  /** What covers the ground (`COVER`), `coverSpan(cols)` × `coverSpan(rows)` cells of `COVER_CELL` m. */
  cover: Uint8Array;
  /**
   * Which way the water runs where a river was drawn: 0 none (still, or not said), else the way as an angle
   * (`flowCode`, `flowOf`). Missing: none anywhere (a land no river was drawn on, a generated one).
   */
  flow?: Uint8Array;
  /** How it was first filled (`LandFile.seeded`). */
  seeded: 'drawn' | 'follow';
  /** Shaped since it was made or read. */
  edited: boolean;
}

/** A rectangle of samples (or of cover cells): `i0 <= i < i1` across, `j0 <= j < j1` down. */
export interface LandRect {
  i0: number;
  j0: number;
  i1: number;
  j1: number;
}

/** What a rectangle of the land held before a stroke (row-major over `rect`): its undo. */
export interface LandPatch {
  rect: LandRect;
  h?: Int16Array;
  wl?: Int16Array;
  flow?: Uint8Array;
  cover?: { rect: LandRect; data: Uint8Array };
}

/** How many ways `Land.flow` tells apart: codes 1 to `FLOW_WAYS`, a full turn. */
export const FLOW_WAYS = 255;

/** The code (`Land.flow`) of the way (`dx`, `dz`) (game space; any length but 0: 0, no way). */
export function flowCode(dx: number, dz: number): number {
  if (!dx && !dz) return 0;
  const turn = pmath.atan2(dz, dx) / (2 * Math.PI);
  return 1 + (Math.round((turn - Math.floor(turn)) * FLOW_WAYS) % FLOW_WAYS);
}

/** The way (a unit vector, game space) `code` says, or null for 0. */
export function flowOf(code: number): [number, number] | null {
  if (!code) return null;
  const a = ((code - 1) / FLOW_WAYS) * 2 * Math.PI;
  return [pmath.cos(a), pmath.sin(a)];
}

/** `land.flow`, made (all still) if it had none. */
export function flowsOf(land: Land): Uint8Array {
  return (land.flow ??= new Uint8Array(land.cols * land.rows));
}

/** The cover cells over `samples` samples of the height field. */
export const coverSpan = (samples: number) => Math.ceil((samples * LAND_CELL) / COVER_CELL);

/** A flat, dry, open land for a `width` × `depth` m world (`LAND_CELL` samples over it, the edge held past the last). */
export function blankLand(width: number, depth: number, seeded: Land['seeded'] = 'drawn'): Land {
  const cols = Math.max(1, Math.ceil(width / LAND_CELL));
  const rows = Math.max(1, Math.ceil(depth / LAND_CELL));
  return { cols, rows, h: new Int16Array(cols * rows), wl: new Int16Array(cols * rows).fill(NONE), cover: new Uint8Array(coverSpan(cols) * coverSpan(rows)), seeded, edited: false };
}

/** The land's height (m) at world point (`x`, `z`): between the four samples round it (bilinear), the edge's past the edge. */
export function heightAt(land: Land, x: number, z: number): number {
  const { cols, rows, h } = land;
  const fx = Math.min(cols - 1, Math.max(0, x / LAND_CELL));
  const fz = Math.min(rows - 1, Math.max(0, z / LAND_CELL));
  const i = Math.min(Math.floor(fx), Math.max(0, cols - 2));
  const j = Math.min(Math.floor(fz), Math.max(0, rows - 2));
  const [tx, tz] = [fx - i, fz - j];
  const [i1, j1] = [Math.min(i + 1, cols - 1), Math.min(j + 1, rows - 1)];
  const top = h[j * cols + i] * (1 - tx) + h[j * cols + i1] * tx;
  const bottom = h[j1 * cols + i] * (1 - tx) + h[j1 * cols + i1] * tx;
  return (top * (1 - tz) + bottom * tz) / 100;
}

/**
 * A short fingerprint of the land's contents (FNV-1a over its heights, water and cover): two
 * lands with the same one are, as near as matters, the same (the setup screens tell a shaped world
 * from the world as it was by it). '' for none.
 */
export function landHash(land: Land | null | undefined): string {
  if (!land) return '';
  let a = 0x811c9dc5;
  let b = 0x1b873593;
  const mix = (v: number) => {
    a = Math.imul(a ^ (v & 0xffff), 0x01000193);
    b = Math.imul(b ^ (v & 0xffff), 0x85ebca6b) ^ (b >>> 13);
  };
  for (let k = 0; k < land.h.length; k++) mix(land.h[k]);
  for (let k = 0; k < land.wl.length; k++) mix(land.wl[k]);
  for (let k = 0; k < land.cover.length; k++) mix(land.cover[k]);
  // (Only where a river says which way it runs: a land with none hashes as it did before there was flow.)
  if (land.flow) {
    for (let k = 0; k < land.flow.length; k++) {
      if (!land.flow[k]) continue;
      mix(k);
      mix(land.flow[k]);
    }
  }
  return `${land.cols}x${land.rows}:${(a >>> 0).toString(36)}${(b >>> 0).toString(36)}`;
}

/** Whether sample (`i`, `j`) is under water (its level over its ground); false off the land. */
export function wetAt(land: Land, i: number, j: number): boolean {
  if (i < 0 || j < 0 || i >= land.cols || j >= land.rows) return false;
  const k = j * land.cols + i;
  return land.wl[k] !== NONE && land.wl[k] > land.h[k];
}

/** The tiles (`LandFile` indices: row-major, `ceil(cols / TILE)` a row) a rectangle of samples reaches. */
export function tilesOf(rect: LandRect, cols: number): number[] {
  const across = Math.ceil(cols / TILE);
  const [ti0, tj0] = [Math.floor(Math.max(0, rect.i0) / TILE), Math.floor(Math.max(0, rect.j0) / TILE)];
  const [ti1, tj1] = [Math.ceil(Math.min(cols, rect.i1) / TILE), Math.ceil(rect.j1 / TILE)];
  const out: number[] = [];
  for (let tj = tj0; tj < tj1; tj++) for (let ti = ti0; ti < ti1; ti++) out.push(tj * across + ti);
  return out;
}

/**
 * The land as the town file carries it. With `only`, `h`, `wl` and `flow` hold just those tiles (the rest of each
 * array left holes); the cover is always whole. `flow` only when some sample of the land has a way (none: the file
 * as it was before rivers ran).
 */
export function encodeLand(land: Land, only?: ReadonlySet<number>): LandFile {
  const count = Math.ceil(land.cols / TILE) * Math.ceil(land.rows / TILE);
  const h: string[] = [];
  const wl: string[] = [];
  const flows = !!land.flow?.some((v) => v !== 0);
  const flow: string[] = [];
  for (let t = 0; t < count; t++) {
    if (only && !only.has(t)) continue;
    h[t] = codeTile(land.h, land.cols, land.rows, t);
    wl[t] = hasWater(land, t) ? codeTile(land.wl, land.cols, land.rows, t) : '';
    if (flows) flow[t] = flowTile(land, t);
  }
  return { v: 1, cell: LAND_CELL, cols: land.cols, rows: land.rows, tile: TILE, h, wl, ...(flows ? { flow } : {}), cover: rle(land.cover), seeded: land.seeded };
}

/**
 * `tiles` of the land, coded as `encodeLand` codes them, by tile index: what a `shapeLand` command
 * sends (`ShapeLandMeta.land`). `flow` only on a land that has one (`Land.flow`): a tile with no way
 * anywhere in it '' (the way it had taken away).
 */
export function encodeTiles(land: Land, tiles: Iterable<number>): { h: Record<number, string>; wl: Record<number, string>; flow?: Record<number, string> } {
  const h: Record<number, string> = {};
  const wl: Record<number, string> = {};
  const flow: Record<number, string> | undefined = land.flow ? {} : undefined;
  for (const t of tiles) {
    h[t] = codeTile(land.h, land.cols, land.rows, t);
    wl[t] = hasWater(land, t) ? codeTile(land.wl, land.cols, land.rows, t) : '';
    if (flow) flow[t] = flowTile(land, t);
  }
  return flow ? { h, wl, flow } : { h, wl };
}

/** The land a town file carried (a tile missing from `h` reads as flat at 0, from `wl` as dry, from `flow` as still). */
export function decodeLand(file: LandFile): Land {
  const { cols, rows } = file;
  const land: Land = { cols, rows, h: new Int16Array(cols * rows), wl: new Int16Array(cols * rows).fill(NONE), cover: new Uint8Array(coverSpan(cols) * coverSpan(rows)), seeded: file.seeded, edited: false };
  const count = Math.ceil(cols / TILE) * Math.ceil(rows / TILE);
  for (let t = 0; t < count; t++) {
    if (file.h[t]) readTile(file.h[t], land.h, cols, rows, t);
    if (file.wl[t]) readTile(file.wl[t], land.wl, cols, rows, t);
    if (file.flow?.[t]) readTile(file.flow[t], flowsOf(land), cols, rows, t);
  }
  let at = 0;
  for (let k = 0; k + 1 < file.cover.length; k += 2) {
    const end = Math.min(land.cover.length, at + file.cover[k]);
    if (file.cover[k + 1]) land.cover.fill(file.cover[k + 1], at, end);
    at += file.cover[k];
  }
  return land;
}

// --- The tiles' coding -------------------------------------------------------------------------

/** Tile `t`'s samples: the first column and row, and how many across and down (cut short at the land's edge). */
function tileBox(cols: number, rows: number, t: number): [number, number, number, number] {
  const across = Math.ceil(cols / TILE);
  const [i0, j0] = [(t % across) * TILE, Math.floor(t / across) * TILE];
  return [i0, j0, Math.min(TILE, cols - i0), Math.min(TILE, rows - j0)];
}

/** Tile `t` of `land.flow` coded, or '' with no way anywhere in it. */
function flowTile(land: Land, t: number): string {
  const flow = land.flow;
  if (!flow) return '';
  const [i0, j0, w, d] = tileBox(land.cols, land.rows, t);
  for (let j = j0; j < j0 + d; j++) for (let k = j * land.cols + i0, end = k + w; k < end; k++) if (flow[k]) return codeTile(flow, land.cols, land.rows, t);
  return '';
}

/** Whether any sample in tile `t` has a water level. */
function hasWater(land: Land, t: number): boolean {
  const [i0, j0, w, d] = tileBox(land.cols, land.rows, t);
  for (let j = j0; j < j0 + d; j++) for (let k = j * land.cols + i0, end = k + w; k < end; k++) if (land.wl[k] !== NONE) return true;
  return false;
}

/**
 * Each sample of the tile less the one before it: the one left of it, or for the first of a row,
 * the first of the row above (the tile's first sample less 0). A flat tile is all zeros but one.
 */
function deltas(values: Int16Array | Uint8Array, cols: number, rows: number, t: number, each: (d: number) => void): void {
  const [i0, j0, w, d] = tileBox(cols, rows, t);
  let first = 0;
  for (let j = j0; j < j0 + d; j++) {
    const row = j * cols + i0;
    each(values[row] - first);
    first = values[row];
    for (let k = row + 1; k < row + w; k++) each(values[k] - values[k - 1]);
  }
}

/** Grows as the tiles need it (a tile is at most 3 bytes a sample). */
let scratch = new Uint8Array(TILE * TILE * 3 + 8);

/** Tile `t` of `values`: zigzag varints of its deltas (`deltas`), a run of zeros as 0x00 then the run's length, in base64. */
function codeTile(values: Int16Array | Uint8Array, cols: number, rows: number, t: number): string {
  let n = 0;
  let zeros = 0;
  const varint = (v: number) => {
    while (v >= 0x80) {
      scratch[n++] = (v & 0x7f) | 0x80;
      v >>>= 7;
    }
    scratch[n++] = v;
  };
  const flush = () => {
    if (!zeros) return;
    scratch[n++] = 0;
    varint(zeros);
    zeros = 0;
  };
  deltas(values, cols, rows, t, (d) => {
    if (d === 0) {
      zeros++;
      return;
    }
    flush();
    varint(d < 0 ? -2 * d - 1 : 2 * d);
  });
  flush();
  let text = '';
  for (let k = 0; k < n; k += 0x2000) text += String.fromCharCode(...scratch.subarray(k, Math.min(n, k + 0x2000)));
  return btoa(text);
}

/** Tile `t` of `values` read back from `codeTile`'s string. */
function readTile(code: string, values: Int16Array | Uint8Array, cols: number, rows: number, t: number): void {
  const text = atob(code);
  if (scratch.length < text.length) scratch = new Uint8Array(text.length);
  for (let k = 0; k < text.length; k++) scratch[k] = text.charCodeAt(k);
  let at = 0;
  const varint = () => {
    let [v, shift] = [0, 0];
    for (;;) {
      const b = scratch[at++];
      v += (b & 0x7f) * 2 ** shift;
      if (b < 0x80) return v;
      shift += 7;
    }
  };
  let zeros = 0;
  const next = (): number => {
    if (zeros) {
      zeros--;
      return 0;
    }
    if (at >= text.length) return 0;
    const v = varint();
    if (v === 0) {
      zeros = varint() - 1;
      return 0;
    }
    return v & 1 ? -(v + 1) / 2 : v / 2;
  };
  const [i0, j0, w, d] = tileBox(cols, rows, t);
  let first = 0;
  for (let j = j0; j < j0 + d; j++) {
    const row = j * cols + i0;
    values[row] = first += next();
    for (let k = row + 1; k < row + w; k++) values[k] = values[k - 1] + next();
  }
}
