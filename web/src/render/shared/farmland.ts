/**
 * A world's farmland as fields: the patchwork laid out in rows of fields (`Patchwork`: each row its own fields'
 * length and shifted along, so no two rows line up), what each field grows and which way its rows run, which fields
 * are farmed (`included`: those the farmland covers, as the landscape asks), the hedges and the grass margins between
 * them; and how they look from above (`paintFields`, into the ground's colour maps: `pc/groundTiles.ts`). The
 * landscape grows the hedges' bushes and trees and the hay bales from the same fields (`nature.ts` `farmPlants`).
 */

import { hash } from '../../game/noise';

/** The patchwork as the landscape gives it (`LandscapeParams.fields`): fields about `size` m, the rows along `angle` (radians). */
export interface FieldGrid {
  size: number;
  angle: number;
}

/** What a field grows: ripe wheat, a green crop in rows, ploughed earth, stubble after the harvest, mown hay, rapeseed in flower, or grazing. */
export type Crop = 'wheat' | 'rows' | 'plough' | 'stubble' | 'hay' | 'rape' | 'pasture';

/** Each crop's share of the fields. */
const CROPS: readonly (readonly [Crop, number])[] = [
  ['wheat', 0.22],
  ['rows', 0.17],
  ['plough', 0.15],
  ['stubble', 0.13],
  ['hay', 0.11],
  ['rape', 0.09],
  ['pasture', 0.13],
];

/** The grass left round a field inside its edge, before its crop (m). */
export const MARGIN = 1.4;
/** A hedge's width as the map paints it (m), and where its bushes stand either side of its line (m). */
export const HEDGE_WIDTH = 2.2;
/** How far apart a hedge's bushes stand (m), and a hedgerow tree at most every so many metres along it. */
export const HEDGE_STEP = 1.9;
export const HEDGE_TREE_EVERY = 24;
/** A field's rows of hay bales: this far apart across them and along them (m). */
const BALE_ROWS = 11;
const BALE_STEP = 9;

/** A field: its row and its place in it, where it starts along and across the rows (m, the patchwork's frame), its size, crop and rows' way. */
export interface Field {
  row: number;
  col: number;
  u0: number;
  v0: number;
  len: number;
  depth: number;
  crop: Crop;
  /** Its crop's rows run across the patchwork's rows (along v) rather than along them. */
  across: boolean;
}

/** A field boundary's stretch (the patchwork's frame, m), and whether a hedge grows along it. */
export interface FieldEdge {
  a: readonly [number, number];
  b: readonly [number, number];
  hedge: boolean;
}

/** Where a point is in the patchwork: its field (farmed or not), how far along and across it, and how far from its nearest edge and nearest hedge (m). */
export interface FieldPoint {
  row: number;
  col: number;
  farmed: boolean;
  crop: Crop;
  /** How far in from the field's nearest edge (m). */
  edge: number;
  /** How far from the nearest hedge's line round the field (m; Infinity: none round it). */
  hedge: number;
}

/**
 * What decides a field is farmed, asked at points across it (a 3 × 3 grid and its middle): `'in'` the farmland,
 * `'out'` of it, or `'never'` (water, the town: no field there whatever the rest).
 */
export type FieldTest = (x: number, z: number) => 'in' | 'out' | 'never';

/** At least this many of a field's nine points in the farmland for it to be farmed, its middle one of them. */
const FARMED_FROM = 5;

/**
 * The patchwork: rows `depth` m deep across the grid's angle, each cut into fields of its own length (1.15 to 2 times
 * the grid's size) from its own shift. Fields are farmed whole (`included`): their edges are straight lines, wherever
 * the farmland's own edge runs.
 */
export class Patchwork {
  readonly depth: number;
  private readonly c: number;
  private readonly s: number;
  private readonly rows = new Map<number, { len: number; shift: number }>();
  private readonly farmed = new Map<number, boolean>();

  constructor(
    readonly grid: FieldGrid,
    private readonly seed: number,
    private readonly test: FieldTest,
  ) {
    this.depth = grid.size * 0.8;
    this.c = Math.cos(grid.angle);
    this.s = Math.sin(grid.angle);
  }

  /** The patchwork's frame at a point: along the rows (u) and across them (v), m. */
  uv(x: number, z: number): [number, number] {
    return [x * this.c + z * this.s, -x * this.s + z * this.c];
  }

  /** A point of the patchwork's frame in the world (x, z). */
  xz(u: number, v: number): [number, number] {
    return [u * this.c - v * this.s, u * this.s + v * this.c];
  }

  /** The canvas transform from the patchwork's frame to the world's (`a, b, c, d` of `transform`). */
  get turn(): [number, number, number, number] {
    return [this.c, this.s, -this.s, this.c];
  }

  /** A row's fields' length and where its fields start (m). */
  row(row: number): { len: number; shift: number } {
    let r = this.rows.get(row);
    if (!r) {
      const len = this.grid.size * (1.15 + 0.85 * hash(row, 11, this.seed + 211));
      r = { len, shift: hash(row, 12, this.seed + 211) * len };
      this.rows.set(row, r);
    }
    return r;
  }

  /** The field of a row at `u`. */
  colAt(row: number, u: number): number {
    const { len, shift } = this.row(row);
    return Math.floor((u - shift) / len);
  }

  /** A field by its row and place in it. */
  field(row: number, col: number): Field {
    const { len, shift } = this.row(row);
    const crop = cropOf(hash(col, row, this.seed + 77));
    return { row, col, u0: shift + col * len, v0: row * this.depth, len, depth: this.depth, crop, across: hash(col, row, this.seed + 78) < 0.3 };
  }

  /** Whether a field is farmed: the farmland under most of it and its middle, and no water or town anywhere on it. */
  included(row: number, col: number): boolean {
    const key = (row + 50_000) * 100_000 + col + 50_000;
    let known = this.farmed.get(key);
    if (known === undefined) {
      const f = this.field(row, col);
      let inside = 0;
      let never = false;
      let middle = false;
      for (const a of [0.12, 0.5, 0.88]) {
        for (const b of [0.12, 0.5, 0.88]) {
          const [x, z] = this.xz(f.u0 + f.len * a, f.v0 + f.depth * b);
          const t = this.test(x, z);
          if (t === 'never') never = true;
          else if (t === 'in') {
            inside++;
            if (a === 0.5 && b === 0.5) middle = true;
          }
        }
      }
      known = !never && middle && inside >= FARMED_FROM;
      this.farmed.set(key, known);
    }
    return known;
  }

  /** The hedge along the start of field (row, col), between it and the one before it in its row. */
  hedgeBefore(row: number, col: number): boolean {
    const [a, b] = [this.included(row, col - 1), this.included(row, col)];
    if (!a && !b) return false;
    return hash(col, row, this.seed + 307) < (a && b ? 0.45 : 0.85);
  }

  /** The hedge along the stretch of row `row`'s near edge between field `below` of the row before and field `above` of this one. */
  hedgeUnder(row: number, below: number, above: number): boolean {
    const [a, b] = [this.included(row - 1, below), this.included(row, above)];
    if (!a && !b) return false;
    return hash(below * 31 + above, row, this.seed + 401) < (a && b ? 0.45 : 0.85);
  }

  /** Where (x, z) is in the patchwork (its field, farmed or not, and how far from its edges and hedges). */
  at(x: number, z: number): FieldPoint {
    const [u, v] = this.uv(x, z);
    const row = Math.floor(v / this.depth);
    const col = this.colAt(row, u);
    const f = this.field(row, col);
    const [lu, lv] = [u - f.u0, v - f.v0];
    const edge = Math.min(lu, f.len - lu, lv, f.depth - lv);
    let hedge = Infinity;
    if (this.hedgeBefore(row, col)) hedge = Math.min(hedge, lu);
    if (this.hedgeBefore(row, col + 1)) hedge = Math.min(hedge, f.len - lu);
    if (this.hedgeUnder(row, this.colAt(row - 1, u), col)) hedge = Math.min(hedge, lv);
    if (this.hedgeUnder(row + 1, col, this.colAt(row + 1, u))) hedge = Math.min(hedge, f.depth - lv);
    return { row, col, farmed: this.included(row, col), crop: f.crop, edge, hedge };
  }

  /** The patchwork's frame's box round a world box (x0, z0, x1, z1). */
  private uvBox(box: readonly [number, number, number, number]): [number, number, number, number] {
    const pts = [this.uv(box[0], box[1]), this.uv(box[2], box[1]), this.uv(box[2], box[3]), this.uv(box[0], box[3])];
    return [Math.min(...pts.map((p) => p[0])), Math.min(...pts.map((p) => p[1])), Math.max(...pts.map((p) => p[0])), Math.max(...pts.map((p) => p[1]))];
  }

  /** The farmed fields reaching into a world box. */
  fields(box: readonly [number, number, number, number]): Field[] {
    const [u0, v0, u1, v1] = this.uvBox(box);
    const out: Field[] = [];
    for (let row = Math.floor(v0 / this.depth); row * this.depth < v1; row++) {
      for (let col = this.colAt(row, u0); col <= this.colAt(row, u1); col++) if (this.included(row, col)) out.push(this.field(row, col));
    }
    return out;
  }

  /**
   * The edges of the farmed fields reaching into a world box: each field's start in its row, and each row's near
   * edge in stretches between the fields either side of it (the rows' fields don't line up), with their hedges.
   */
  edges(box: readonly [number, number, number, number]): FieldEdge[] {
    const [u0, v0, u1, v1] = this.uvBox(box);
    const out: FieldEdge[] = [];
    for (let row = Math.floor(v0 / this.depth); row * this.depth < v1 + this.depth; row++) {
      const v = row * this.depth;
      // (Each field's start in its row, if the row is in the box.)
      if (v < v1) {
        for (let col = this.colAt(row, u0); col <= this.colAt(row, u1) + 1; col++) {
          if (!this.included(row, col - 1) && !this.included(row, col)) continue;
          const u = this.field(row, col).u0;
          out.push({ a: [u, v], b: [u, v + this.depth], hedge: this.hedgeBefore(row, col) });
        }
      }
      // (The row's near edge, cut wherever a field of either row starts: the same stretches whatever the box.)
      const [lo, hi] = [this.row(row - 1), this.row(row)];
      const cuts = new Set<number>();
      for (const r of [lo, hi]) for (let k = Math.floor((u0 - r.shift) / r.len); r.shift + k * r.len <= u1 + r.len; k++) cuts.add(r.shift + k * r.len);
      const at = [...cuts].sort((p, q) => p - q);
      for (let k = 0; k + 1 < at.length; k++) {
        if (at[k + 1] <= u0 || at[k] >= u1) continue;
        const mid = (at[k] + at[k + 1]) / 2;
        const [below, above] = [this.colAt(row - 1, mid), this.colAt(row, mid)];
        if (!this.included(row - 1, below) && !this.included(row, above)) continue;
        out.push({ a: [at[k], v], b: [at[k + 1], v], hedge: this.hedgeUnder(row, below, above) });
      }
    }
    return out;
  }

  /** The hay bales of a stubble field (the patchwork's frame, m; and each one's turn along its rows): in rows, a few missing. */
  bales(f: Field): [number, number, number][] {
    const out: [number, number, number][] = [];
    const inset = MARGIN + 3;
    const [along, across] = f.across ? [f.depth, f.len] : [f.len, f.depth];
    for (let r = 0; inset + r * BALE_ROWS < across - inset; r++) {
      for (let k = 0; inset + k * BALE_STEP < along - inset; k++) {
        const h = hash(k + r * 64, f.col * 97 + f.row, this.seed + 503);
        if (h < 0.3) continue;
        const a = inset + k * BALE_STEP + (h - 0.5) * 3;
        const b = inset + r * BALE_ROWS + (hash(k, r, this.seed + f.col + 509) - 0.5) * 2;
        const [lu, lv] = f.across ? [b, a] : [a, b];
        out.push([f.u0 + lu, f.v0 + lv, (f.across ? Math.PI / 2 : 0) + (h - 0.5) * 0.6]);
      }
    }
    return out;
  }
}

/** The crop for a hash 0..1, by `CROPS`' shares. */
function cropOf(h: number): Crop {
  let at = 0;
  for (const [crop, share] of CROPS) {
    at += share;
    if (h < at) return crop;
  }
  return CROPS[CROPS.length - 1][0];
}

/**
 * How each crop looks from above (sRGB): its ground (`base`) and the lines across it (`line`, `width` m wide every
 * `period` m, along its rows), its tramlines (the tractor's wheel marks, a pair every `tram.every` m) and a third colour in
 * broad patches (`patch`: the crop thinner or riper there). Pasture is grass: the land's own lawn shows (none).
 */
export const CROP_LOOK: Record<Exclude<Crop, 'pasture'>, { base: string; line: string; width: number; period: number; tram?: { every: number; colour: string }; patch: string }> = {
  wheat: { base: '#CFAE55', line: '#B4943F', width: 0.7, period: 2, tram: { every: 20, colour: '#A88C3E' }, patch: '#D8BD68' },
  rows: { base: '#775538', line: '#5F8E34', width: 1.2, period: 2.2, patch: '#6E7F3A' },
  plough: { base: '#755237', line: '#4E3524', width: 0.9, period: 2, patch: '#68472F' },
  stubble: { base: '#D7C891', line: '#BCA86C', width: 2.2, period: 4.4, patch: '#CDBB7E' },
  hay: { base: '#8DAA4C', line: '#CBC47C', width: 1.1, period: 5.5, patch: '#9BB55A' },
  rape: { base: '#DDC638', line: '#C8B12C', width: 0.6, period: 2.2, tram: { every: 20, colour: '#B9A42A' }, patch: '#E6D24A' },
};

/** A field's average colour from above (sRGB 0..1): its ground and its lines together, as far off it looks. Null for pasture. */
export function cropColour(crop: Crop): [number, number, number] | null {
  if (crop === 'pasture') return null;
  const look = CROP_LOOK[crop];
  const [b, l] = [rgb(look.base), rgb(look.line)];
  const t = look.width / look.period;
  return [b[0] + (l[0] - b[0]) * t, b[1] + (l[1] - b[1]) * t, b[2] + (l[2] - b[2]) * t];
}

/** The hedges' colour from above (sRGB). */
export const HEDGE_LOOK = { leaf: '#3E5A28', shade: '#2E4520' };

function rgb(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
}

/** What `paintFields` draws with: a 2D canvas's context (or a stand-in that records what is drawn, in the tests). */
export type FieldCanvas = Pick<
  OffscreenCanvasRenderingContext2D,
  'fillStyle' | 'strokeStyle' | 'lineWidth' | 'lineCap' | 'globalAlpha' | 'save' | 'restore' | 'transform' | 'beginPath' | 'rect' | 'clip' | 'fillRect' | 'moveTo' | 'lineTo' | 'stroke' | 'fill' | 'arc' | 'createRadialGradient'
>;

/**
 * The farmed fields reaching into a world box onto `g` (its transform the world's metres; the patchwork's turn is
 * added here): each field's crop inside its grass margin, its rows, tramlines and broad patches, then the hedges
 * along the edges that have them. The margins and pasture are left clear: the land's lawn shows there.
 */
export function paintFields(g: FieldCanvas, pw: Patchwork, box: readonly [number, number, number, number]): void {
  g.save();
  g.transform(...pw.turn, 0, 0);
  for (const f of pw.fields(box)) {
    if (f.crop === 'pasture') continue;
    const look = CROP_LOOK[f.crop];
    const [x0, z0, w, d] = [f.u0 + MARGIN, f.v0 + MARGIN, f.len - 2 * MARGIN, f.depth - 2 * MARGIN];
    if (w <= 0 || d <= 0) continue;
    g.save();
    g.beginPath();
    g.rect(x0, z0, w, d);
    g.clip();
    g.fillStyle = look.base;
    g.fillRect(x0, z0, w, d);
    // Broad patches, a little riper or thinner: a few soft blots.
    for (let k = 0; k < 3; k++) {
      const h = hash(f.col * 7 + k, f.row, 613);
      const [px, pz, r] = [x0 + w * hash(k, f.col, f.row + 617), z0 + d * hash(f.row, k, f.col + 619), 6 + 10 * h];
      const blot = g.createRadialGradient(px, pz, 0, px, pz, r);
      blot.addColorStop(0, look.patch);
      blot.addColorStop(1, `${look.patch}00`);
      g.globalAlpha = 0.55;
      g.fillStyle = blot;
      g.fillRect(px - r, pz - r, 2 * r, 2 * r);
    }
    g.globalAlpha = 1;
    // Its rows (along or across the patchwork's), then its tramlines.
    g.fillStyle = look.line;
    const [span, reach] = f.across ? [w, d] : [d, w];
    for (let k = look.period / 2; k < span; k += look.period) (f.across ? g.fillRect(x0 + k - look.width / 2, z0, look.width, d) : g.fillRect(x0, z0 + k - look.width / 2, w, look.width));
    if (look.tram) {
      g.fillStyle = look.tram.colour;
      for (let k = look.tram.every / 2; k < span; k += look.tram.every) {
        for (const o of [-0.95, 0.95]) (f.across ? g.fillRect(x0 + k + o - 0.2, z0, 0.4, reach) : g.fillRect(x0, z0 + k + o - 0.2, reach, 0.4));
      }
    }
    g.restore();
  }
  // The hedges: a dark band, its shaded side darker.
  g.lineCap = 'round';
  for (const [colour, width] of [
    [HEDGE_LOOK.shade, HEDGE_WIDTH],
    [HEDGE_LOOK.leaf, HEDGE_WIDTH * 0.62],
  ] as const) {
    g.strokeStyle = colour;
    g.lineWidth = width;
    g.beginPath();
    for (const e of pw.edges(box)) {
      if (!e.hedge) continue;
      g.moveTo(e.a[0], e.a[1]);
      g.lineTo(e.b[0], e.b[1]);
    }
    g.stroke();
  }
  g.restore();
}
