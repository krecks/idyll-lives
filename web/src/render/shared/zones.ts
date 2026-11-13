/**
 * A big world's zones (`meta.zones`: woods, meadows, water, hills and fields, polygons the world's
 * generator laid out, `game/macro.ts`) as the landscape reads them: each kind rasterised once, at
 * load, on a `ZONE_CELL` m grid over the world, as a signed distance to its edge (m; above 0
 * inside), so the land's height, its colours and what grows on it can follow the zones with soft
 * edges. Past the world's edge a zone carries on as at the edge (a sea along one side goes on to the
 * horizon). Engine-neutral.
 */

import type { WorldZone } from '../../core/protocol';

/** The grid's cells (m). */
export const ZONE_CELL = 4;
/** Cells this near (m) a zone's edge have their true distance to it (`exactNearEdges`); further ones the chamfer's. */
const EXACT_BAND = 32;
/** How far (cells) the farmland's edge is evened out each way (`evened`): past the generator's 10 m steps. */
const EVEN_CELLS = 2;

export type ZoneKind = WorldZone['kind'];

/** `ZoneField.share` from what `ZoneField.edge` gave there (metres inside the zones): 0..1, over a soft edge `soft` m each side. */
export function shareAt(edge: number, soft: number): number {
  if (edge === -Infinity) return 0;
  const t = Math.min(1, Math.max(0, (edge + soft) / (2 * soft)));
  return t * t * (3 - 2 * t);
}

export class ZoneField {
  private readonly gw: number;
  private readonly gd: number;
  /** Each kind's signed distance at the cells' middles (m, above 0 inside); missing: no zone of it. */
  private readonly fields = new Map<ZoneKind, Float32Array>();

  constructor(
    zones: readonly WorldZone[],
    private readonly w: number,
    private readonly d: number,
  ) {
    this.gw = Math.max(1, Math.ceil(w / ZONE_CELL));
    this.gd = Math.max(1, Math.ceil(d / ZONE_CELL));
    for (const [kind, list] of Map.groupBy(zones, (z) => z.kind)) {
      const polys = list.map((z) => z.poly);
      const inside = this.rasterise(polys);
      if (!inside.some((v) => v)) continue;
      const sd = this.exactNearEdges(this.signedDistance(inside), polys);
      // (Farmland's outline is the generator's 10 m cells', a staircase along every slanted edge: evened out, so its
      // fields end along a line, not in steps.)
      this.fields.set(kind, kind === 'fields' ? this.evened(sd, EVEN_CELLS) : sd);
    }
  }

  /** Whether the world has a zone of `kind`. */
  has(kind: ZoneKind): boolean {
    return this.fields.has(kind);
  }

  /**
   * Metres inside the zones of `kind` at (x, z) (below 0: outside, that far from the nearest), read
   * between the cells' middles; -Infinity without any. Outside the world, the edge's, carried on
   * outward: deeper in a zone that reaches the edge, further from one that doesn't.
   */
  edge(kind: ZoneKind, x: number, z: number): number {
    const f = this.fields.get(kind);
    if (!f) return -Infinity;
    const cx = Math.min(this.w, Math.max(0, x));
    const cz = Math.min(this.d, Math.max(0, z));
    const out = Math.hypot(x - cx, z - cz);
    const gx = Math.min(this.gw - 1, Math.max(0, cx / ZONE_CELL - 0.5));
    const gz = Math.min(this.gd - 1, Math.max(0, cz / ZONE_CELL - 0.5));
    const x0 = Math.floor(gx);
    const z0 = Math.floor(gz);
    const x1 = Math.min(this.gw - 1, x0 + 1);
    const z1 = Math.min(this.gd - 1, z0 + 1);
    const u = gx - x0;
    const v = gz - z0;
    const at = (i: number, j: number) => f[j * this.gw + i];
    const e = (at(x0, z0) * (1 - u) + at(x1, z0) * u) * (1 - v) + (at(x0, z1) * (1 - u) + at(x1, z1) * u) * v;
    return e > 0 ? e + out : e - out;
  }

  /** How much of a point is in the zones of `kind`, 0..1, over a soft edge `soft` m each side of it. */
  share(kind: ZoneKind, x: number, z: number, soft: number): number {
    return shareAt(this.edge(kind, x, z), soft);
  }

  /** The cells whose middle is inside an odd number of `polys` (even-odd, as the generator's `rasterize`). */
  private rasterise(polys: readonly (readonly [number, number][])[]): Uint8Array {
    const { gw, gd } = this;
    const mask = new Uint8Array(gw * gd);
    for (const poly of polys) {
      for (let j = 0; j < gd; j++) {
        const zc = (j + 0.5) * ZONE_CELL;
        const xs: number[] = [];
        for (let a = 0, b = poly.length - 1; a < poly.length; b = a++) {
          const [[xa, za], [xb, zb]] = [poly[a], poly[b]];
          if (za > zc !== zb > zc) xs.push(((xb - xa) * (zc - za)) / (zb - za) + xa);
        }
        xs.sort((p, q) => p - q);
        for (let k = 0; k + 1 < xs.length; k += 2) {
          const i0 = Math.max(0, Math.ceil(xs[k] / ZONE_CELL - 0.5));
          const i1 = Math.min(gw, Math.ceil(xs[k + 1] / ZONE_CELL - 0.5));
          for (let i = i0; i < i1; i++) mask[j * gw + i] ^= 1;
        }
      }
    }
    return mask;
  }

  /**
   * The cells within `EXACT_BAND` m of the edge given their true distance to the polygons' edges
   * (the chamfer's follows the cells: a shore drawn from it would step along the grid), its sign kept.
   */
  private exactNearEdges(sd: Float32Array, polys: readonly (readonly [number, number][])[]): Float32Array {
    const B = 16;
    const bw = Math.ceil(this.w / B) + 1;
    const bd = Math.ceil(this.d / B) + 1;
    const buckets = new Map<number, number[]>();
    const segs: number[] = [];
    for (const poly of polys) {
      for (let a = 0, b = poly.length - 1; a < poly.length; b = a++) {
        const [[xa, za], [xb, zb]] = [poly[b], poly[a]];
        const k = segs.length / 4;
        segs.push(xa, za, xb, zb);
        for (let j = Math.max(0, Math.floor(Math.min(za, zb) / B)); j <= Math.min(bd - 1, Math.floor(Math.max(za, zb) / B)); j++) {
          for (let i = Math.max(0, Math.floor(Math.min(xa, xb) / B)); i <= Math.min(bw - 1, Math.floor(Math.max(xa, xb) / B)); i++) {
            const list = buckets.get(j * bw + i);
            if (list) list.push(k);
            else buckets.set(j * bw + i, [k]);
          }
        }
      }
    }
    const reach = Math.ceil(EXACT_BAND / B);
    const seen = new Set<number>();
    for (let j = 0; j < this.gd; j++) {
      for (let i = 0; i < this.gw; i++) {
        const c = j * this.gw + i;
        if (Math.abs(sd[c]) > EXACT_BAND) continue;
        const x = (i + 0.5) * ZONE_CELL;
        const z = (j + 0.5) * ZONE_CELL;
        const bi = Math.floor(x / B);
        const bj = Math.floor(z / B);
        let best = Infinity;
        seen.clear();
        for (let v = Math.max(0, bj - reach); v <= Math.min(bd - 1, bj + reach); v++) {
          for (let u = Math.max(0, bi - reach); u <= Math.min(bw - 1, bi + reach); u++) {
            for (const k of buckets.get(v * bw + u) ?? []) {
              if (seen.has(k)) continue;
              seen.add(k);
              const [xa, za, xb, zb] = [segs[k * 4], segs[k * 4 + 1], segs[k * 4 + 2], segs[k * 4 + 3]];
              const dx = xb - xa;
              const dz = zb - za;
              const t = Math.max(0, Math.min(1, ((x - xa) * dx + (z - za) * dz) / (dx * dx + dz * dz || 1)));
              const d = Math.hypot(x - xa - dx * t, z - za - dz * t);
              if (d < best) best = d;
            }
          }
        }
        if (best < Infinity) sd[c] = sd[c] > 0 ? best : -best;
      }
    }
    return sd;
  }

  /**
   * `sd` with each cell the mean of the cells within `r` of it, twice over (a box blur each way, twice: nearly a
   * round one): the edge (where it is 0) loses corners smaller than that, and along a straight edge it is still the
   * distance to it.
   */
  private evened(sd: Float32Array, r: number): Float32Array {
    const { gw, gd } = this;
    let from: Float32Array = sd;
    let to: Float32Array = new Float32Array(sd.length);
    for (let pass = 0; pass < 4; pass++) {
      // (Along x on even passes, along z on odd ones; the edge's cells stand in for those past it.)
      const [n, m, step, row] = pass % 2 === 0 ? [gw, gd, 1, gw] : [gd, gw, gw, 1];
      for (let j = 0; j < m; j++) {
        const base = j * row;
        let sum = 0;
        for (let k = -r; k <= r; k++) sum += from[base + Math.min(n - 1, Math.max(0, k)) * step];
        for (let i = 0; i < n; i++) {
          to[base + i * step] = sum / (2 * r + 1);
          sum += from[base + Math.min(n - 1, i + r + 1) * step] - from[base + Math.max(0, i - r) * step];
        }
      }
      [from, to] = [to, from === sd ? new Float32Array(sd.length) : from];
    }
    return from;
  }

  /** Signed distance (m) from each cell's middle to the edge between inside and outside cells (two chamfer passes each way). */
  private signedDistance(inside: Uint8Array): Float32Array {
    const toInside = this.chamfer(inside, 1);
    const toOutside = this.chamfer(inside, 0);
    const out = new Float32Array(inside.length);
    const h = ZONE_CELL / 2;
    for (let i = 0; i < out.length; i++) out[i] = inside[i] ? toOutside[i] - h : -(toInside[i] - h);
    return out;
  }

  /** Distance (m) from each cell to the nearest cell whose `mask` is `value` (0 on those). */
  private chamfer(mask: Uint8Array, value: 0 | 1): Float32Array {
    const { gw, gd } = this;
    const dist = new Float32Array(gw * gd);
    // (No cell of that value at all: as far as the world is wide.)
    const far = (gw + gd) * ZONE_CELL;
    for (let i = 0; i < dist.length; i++) dist[i] = mask[i] === value ? 0 : far;
    const c = ZONE_CELL;
    const diag = c * Math.SQRT2;
    const relax = (i: number, j: number, cost: number) => {
      if (dist[j] + cost < dist[i]) dist[i] = dist[j] + cost;
    };
    for (let z = 0; z < gd; z++) {
      for (let x = 0; x < gw; x++) {
        const i = z * gw + x;
        if (x > 0) relax(i, i - 1, c);
        if (z > 0) {
          relax(i, i - gw, c);
          if (x > 0) relax(i, i - gw - 1, diag);
          if (x < gw - 1) relax(i, i - gw + 1, diag);
        }
      }
    }
    for (let z = gd - 1; z >= 0; z--) {
      for (let x = gw - 1; x >= 0; x--) {
        const i = z * gw + x;
        if (x < gw - 1) relax(i, i + 1, c);
        if (z < gd - 1) {
          relax(i, i + gw, c);
          if (x < gw - 1) relax(i, i + gw + 1, diag);
          if (x > 0) relax(i, i + gw - 1, diag);
        }
      }
    }
    return dist;
  }
}
