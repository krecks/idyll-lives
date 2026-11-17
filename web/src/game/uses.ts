/**
 * What the land is to be used for, as the player paints it in step 2 (the UI calls these "zones";
 * in code they are uses, since `WorldZone`, the backdrop's zones and `render/shared/zones.ts` have
 * the word already). A grid of `USE_CELL` m cells over the world from its corner, row-major, each
 * holding one use's code (0: none), kept run-length as `[count, code, count, code, …]`.
 *
 * The layer rides on the draft as `OpenWorldDraft.uses` while the new game is set up. It never goes
 * into the town file nor the world's code: Build turns it into lots, and then it is done with.
 */

import type { NeighbourhoodDraft } from './town';

/** A use cell's side (m). */
export const USE_CELL = 4;

/** The uses, in their codes' order (homes is 1). */
export const USE_KINDS = ['homes', 'rows', 'shops', 'green', 'public'] as const;
/**
 * What a cell is for: houses (`homes`), terraces and flats (`rows`), shops and work (`shops`), parks and
 * green (`green`), or public places such as the hall, the church and the school (`public`).
 */
export type UseKind = (typeof USE_KINDS)[number];

/** A cell with no use. */
export const NO_USE = 0;

/** Each use's code in a layer's cells. */
export const USE_CODE: Readonly<Record<UseKind, number>> = { homes: 1, rows: 2, shops: 3, green: 4, public: 5 };

/** The use a cell's code stands for; null for none, or a code no use has. */
export function useOfCode(code: number): UseKind | null {
  return USE_KINDS[code - 1] ?? null;
}

/** Each use's colour (sRGB), on the overlay, the brush and the card's chips. */
export const USE_COLORS: Readonly<Record<UseKind, string>> = {
  homes: '#F2C94C',
  rows: '#F2994A',
  shops: '#9B6BDF',
  green: '#5DBB63',
  public: '#E05C9A',
};

/** Each use's name for the player. */
export const USE_LABELS: Readonly<Record<UseKind, string>> = {
  homes: 'Homes',
  rows: 'Terraces & flats',
  shops: 'Shops & work',
  green: 'Parks & green',
  public: 'Public places',
};

/** The uses over a world: `cols` × `rows` cells of `USE_CELL` m, row-major, run-length `[count, code, …]`. */
export interface UseLayer {
  v: 1;
  cols: number;
  rows: number;
  cells: number[];
}

/** How many cells across and down a world `width` × `depth` m takes (the last ones may hang over its edge). */
export function useGrid(width: number, depth: number): { cols: number; rows: number } {
  return { cols: Math.max(0, Math.ceil(width / USE_CELL)), rows: Math.max(0, Math.ceil(depth / USE_CELL)) };
}

/** A layer of `cols` × `rows` cells with no use anywhere. */
export function usesEmpty(cols: number, rows: number): UseLayer {
  const n = cols * rows;
  return { v: 1, cols, rows, cells: n > 0 ? [n, NO_USE] : [] };
}

/** A layer of the empty use for `draft`'s world. */
export function usesFor(draft: Pick<NeighbourhoodDraft, 'width' | 'depth'>): UseLayer {
  const { cols, rows } = useGrid(draft.width, draft.depth);
  return usesEmpty(cols, rows);
}

/** `codes` (a code a cell, row-major, `cols` × `rows` of them) as a layer; a code no use has is taken as none. */
export function packUses(codes: ArrayLike<number>, cols: number, rows: number): UseLayer {
  const n = cols * rows;
  const cells: number[] = [];
  let run = 0;
  let code = -1;
  for (let k = 0; k < n; k++) {
    const c = useOfCode(codes[k] ?? NO_USE) ? codes[k] : NO_USE;
    if (c === code) {
      run++;
      continue;
    }
    if (run) cells.push(run, code);
    [run, code] = [1, c];
  }
  if (run) cells.push(run, code);
  return { v: 1, cols, rows, cells };
}

/** Layers unpacked, by layer (a layer is never changed in place: an edit makes a new one). */
const unpacked = new WeakMap<UseLayer, Uint8Array>();

/**
 * `layer`'s cells, a code each, row-major (cell (`i`, `j`) at `j * cols + i`). Kept with the layer
 * and handed out again: read it, never write it (copy it first to make a new layer).
 */
export function unpackUses(layer: UseLayer): Uint8Array {
  let out = unpacked.get(layer);
  if (out) return out;
  const n = layer.cols * layer.rows;
  out = new Uint8Array(n);
  let k = 0;
  for (let r = 0; r + 1 < layer.cells.length && k < n; r += 2) {
    const [count, code] = [layer.cells[r], layer.cells[r + 1]];
    const end = Math.min(n, k + count);
    if (useOfCode(code)) out.fill(code, k, end);
    k = end;
  }
  unpacked.set(layer, out);
  return out;
}

/** The use at a town point (`x`, `z`, m); null where there is none, or off the layer. */
export function useAt(layer: UseLayer, x: number, z: number): UseKind | null {
  const [i, j] = [Math.floor(x / USE_CELL), Math.floor(z / USE_CELL)];
  if (i < 0 || j < 0 || i >= layer.cols || j >= layer.rows) return null;
  return useOfCode(unpackUses(layer)[j * layer.cols + i]);
}

/**
 * A fingerprint of `layer`'s cells (none: ''): the same uses give the same hash however their runs
 * were split, and a cell changed gives another.
 */
export function usesHash(layer: UseLayer | null | undefined): string {
  if (!layer) return '';
  const h = new TextHash();
  // (The runs as `packUses` would write them: next runs of one code joined, empty ones left out.)
  let run = 0;
  let code = -1;
  for (let r = 0; r + 1 < layer.cells.length; r += 2) {
    const [count, c] = [layer.cells[r], useOfCode(layer.cells[r + 1]) ? layer.cells[r + 1] : NO_USE];
    if (count <= 0) continue;
    if (c === code) {
      run += count;
      continue;
    }
    if (run) h.number(run).number(code);
    [run, code] = [count, c];
  }
  if (run) h.number(run).number(code);
  return `${layer.cols}x${layer.rows}:${h.value()}`;
}

/**
 * A fingerprint of a town's layout (the plan step 2 edits): its size, its roads, its areas and the main
 * town's centre, and its uses. What Build makes the town from, so a layout whose hash is the proposal's
 * builds the proposal's town as it is. Lots, backdrop, places and parking don't count.
 */
export function layoutHash(plan: NeighbourhoodDraft): string {
  const open = plan.open;
  const text = JSON.stringify([plan.width, plan.depth, plan.roads, plan.greens, open?.areas ?? null, open?.centre ?? null]);
  return `${new TextHash().text(text).value()}:${usesHash(open?.uses)}`;
}

/** Two 32-bit FNV-style hashes run side by side over what is fed in (as `land.ts` `landHash`). */
class TextHash {
  private a = 0x811c9dc5;
  private b = 0x1b873593;

  private mix(v: number): void {
    this.a = Math.imul(this.a ^ (v & 0xffff), 0x01000193);
    this.b = Math.imul(this.b ^ (v & 0xffff), 0x85ebca6b) ^ (this.b >>> 13);
  }

  number(v: number): this {
    this.mix(v);
    this.mix(v >>> 16);
    return this;
  }

  text(s: string): this {
    for (let k = 0; k < s.length; k++) this.mix(s.charCodeAt(k));
    return this;
  }

  value(): string {
    return `${(this.a >>> 0).toString(36)}${(this.b >>> 0).toString(36)}`;
  }
}
