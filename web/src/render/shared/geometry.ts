/**
 * Small mesh builder for generated geometry (houses, roofs, helpers): flat-shaded polygons
 * with explicit normals, metre-scale UVs, sRGB vertex colours and an optional per-vertex
 * cutaway attribute (see `WallCutPlugin` in materials.ts). Winding is derived from the normal,
 * so meshes work with back-face culling. Engine-neutral: `meshData` hands the arrays to the
 * renderer, which uploads them (`pc/meshes.ts` `meshFrom`).
 */

export type V3 = readonly [number, number, number];

/** Custom vertex attribute: [mask A, kind, mask B, 0]; see `WallCutPlugin`. */
export const WALL_CUT_KIND = 'wallCut';

/** Cutaway behaviour of a vertex when its wall is cut: keep, clamp to stub height, hide. */
export const Cut = { Keep: 0, Clamp: 1, Hide: 2 } as const;

/** Cut mask bits: the wall hides what is behind it when the camera looks along +x, -x, +z, -z. */
export const LOOK = { PosX: 1, NegX: 2, PosZ: 4, NegZ: 8, All: 15 } as const;

/**
 * A mesh's vertex arrays as a renderer uploads them (`Geo.meshData`). The colours and the cutaway
 * attribute are a byte a channel (40 bytes a vertex, not 64): the colours come from hex colours (a
 * ridge cap's darkened by a fifth: within half a step of 255), the cutaway's are small whole numbers.
 */
export interface MeshData {
  positions: Float32Array;
  normals: Float32Array;
  uvs: Float32Array;
  indices: Uint16Array | Uint32Array;
  /** sRGB vertex colours (RGBA, 0..255 for 0..1), if asked for. */
  colors?: Uint8Array;
  /** The cutaway attribute (`WALL_CUT_KIND`, 4 per vertex: masks 0..15, kind 0..2), if asked for. */
  cut?: Uint8Array;
  /** Whether a vertex colour is translucent (the mesh then blends by vertex alpha). */
  vertexAlpha: boolean;
}

export class Geo {
  readonly pos: number[] = [];
  readonly nor: number[] = [];
  readonly uv: number[] = [];
  readonly col: number[] = [];
  readonly cut: number[] = [];
  readonly idx: number[] = [];
  /** sRGB colour applied to added vertices. */
  r = 1;
  g = 1;
  b = 1;
  a = 1;
  maskA: number = LOOK.All;
  maskB: number = LOOK.All;
  kind: number = Cut.Keep;

  get empty(): boolean {
    return this.idx.length === 0;
  }

  color(hex: string, alpha = 1): this {
    const v = parseInt(hex.slice(1), 16);
    this.r = ((v >> 16) & 255) / 255;
    this.g = ((v >> 8) & 255) / 255;
    this.b = (v & 255) / 255;
    this.a = alpha;
    return this;
  }

  /** Cutaway data for following vertices. */
  cutting(kind: number, maskA: number = LOOK.All, maskB: number = LOOK.All): this {
    this.kind = kind;
    this.maskA = maskA;
    this.maskB = maskB;
    return this;
  }

  private vertex(p: V3, n: V3, u: number, v: number, alpha = this.a): number {
    this.pos.push(p[0], p[1], p[2]);
    this.nor.push(n[0], n[1], n[2]);
    this.uv.push(u, v);
    this.col.push(this.r, this.g, this.b, alpha);
    this.cut.push(this.maskA, this.kind, this.maskB, 0);
    return this.pos.length / 3 - 1;
  }

  /**
   * Convex polygon (3 or 4 points, any order around the perimeter) facing `n`, with explicit
   * UVs or planar UVs projected along the dominant normal axis. `alphas` sets per-vertex alpha.
   */
  poly(points: readonly V3[], n: V3, uvs?: readonly (readonly [number, number])[], alphas?: readonly number[]): this {
    const ids = points.map((p, i) => {
      const [u, v] = uvs?.[i] ?? planarUV(p, n);
      return this.vertex(p, n, u, v, alphas?.[i] ?? this.a);
    });
    for (let i = 1; i + 1 < ids.length; i++) this.tri(ids[0], ids[i], ids[i + 1], n);
    return this;
  }

  /** Adds `other`'s geometry (with its cutaway data) in this geometry's colour. */
  append(other: Geo): this {
    const base = this.pos.length / 3;
    const n = other.pos.length / 3;
    this.pos.push(...other.pos);
    this.nor.push(...other.nor);
    this.uv.push(...other.uv);
    this.cut.push(...other.cut);
    for (let i = 0; i < n; i++) this.col.push(this.r, this.g, this.b, this.a);
    for (const i of other.idx) this.idx.push(base + i);
    return this;
  }

  /** Adds `other`'s geometry as it is, its own colours and cutaway data (`append` paints it in this one's colour). */
  merge(other: Geo): this {
    const base = this.pos.length / 3;
    const push = (to: number[], from: readonly number[]) => {
      for (let i = 0; i < from.length; i++) to.push(from[i]);
    };
    push(this.pos, other.pos);
    push(this.nor, other.nor);
    push(this.uv, other.uv);
    push(this.col, other.col);
    push(this.cut, other.cut);
    for (let i = 0; i < other.idx.length; i++) this.idx.push(base + other.idx[i]);
    return this;
  }

  /** Axis-aligned box; `skip` lists faces to omit ('px', 'nx', 'py', 'ny', 'pz', 'nz'). */
  box(x0: number, y0: number, z0: number, x1: number, y1: number, z1: number, skip = ''): this {
    if (!skip.includes('py')) this.poly([[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]], [0, 1, 0]);
    if (!skip.includes('ny')) this.poly([[x0, y0, z0], [x0, y0, z1], [x1, y0, z1], [x1, y0, z0]], [0, -1, 0]);
    if (!skip.includes('nz')) this.poly([[x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0]], [0, 0, -1]);
    if (!skip.includes('pz')) this.poly([[x0, y0, z1], [x0, y1, z1], [x1, y1, z1], [x1, y0, z1]], [0, 0, 1]);
    if (!skip.includes('nx')) this.poly([[x0, y0, z0], [x0, y1, z0], [x0, y1, z1], [x0, y0, z1]], [-1, 0, 0]);
    if (!skip.includes('px')) this.poly([[x1, y0, z0], [x1, y0, z1], [x1, y1, z1], [x1, y1, z0]], [1, 0, 0]);
    return this;
  }

  /**
   * Front faces follow the convention used by `models.ts`: (a - b) x (c - b) points along
   * the normal.
   */
  private tri(a: number, b: number, c: number, n: V3): void {
    const p = this.pos;
    const ax = p[a * 3] - p[b * 3];
    const ay = p[a * 3 + 1] - p[b * 3 + 1];
    const az = p[a * 3 + 2] - p[b * 3 + 2];
    const cx = p[c * 3] - p[b * 3];
    const cy = p[c * 3 + 1] - p[b * 3 + 1];
    const cz = p[c * 3 + 2] - p[b * 3 + 2];
    const fx = ay * cz - az * cy;
    const fy = az * cx - ax * cz;
    const fz = ax * cy - ay * cx;
    if (fx * n[0] + fy * n[1] + fz * n[2] >= 0) this.idx.push(a, b, c);
    else this.idx.push(a, c, b);
  }

  /** The geometry's arrays, with its colours and its cutaway attribute if asked for. */
  meshData(options: { colors?: boolean; cut?: boolean } = {}): MeshData {
    return {
      positions: new Float32Array(this.pos),
      normals: new Float32Array(this.nor),
      uvs: new Float32Array(this.uv),
      indices: this.pos.length / 3 > 65535 ? new Uint32Array(this.idx) : new Uint16Array(this.idx),
      colors: options.colors ? Uint8Array.from(this.col, (v) => Math.min(255, Math.max(0, Math.round(v * 255)))) : undefined,
      cut: options.cut ? new Uint8Array(this.cut) : undefined,
      vertexAlpha: !!options.colors && this.col.some((v, i) => i % 4 === 3 && v < 1),
    };
  }
}

/** Planar UVs in metres along the dominant normal axis (v points up on walls). */
export function planarUV(p: V3, n: V3): [number, number] {
  const ax = Math.abs(n[0]);
  const ay = Math.abs(n[1]);
  const az = Math.abs(n[2]);
  if (ay >= ax && ay >= az) return [p[0], p[2]];
  if (ax >= az) return [p[2], p[1]];
  return [p[0], p[1]];
}

export function normalOf(a: V3, b: V3, c: V3): V3 {
  const ux = b[0] - a[0];
  const uy = b[1] - a[1];
  const uz = b[2] - a[2];
  const vx = c[0] - a[0];
  const vy = c[1] - a[1];
  const vz = c[2] - a[2];
  const x = uy * vz - uz * vy;
  const y = uz * vx - ux * vz;
  const z = ux * vy - uy * vx;
  const len = Math.hypot(x, y, z) || 1;
  return [x / len, y / len, z / len];
}

/** Normal of a polygon, flipped (if needed) to point towards `towards` (e.g. up or outwards). */
export function facing(a: V3, b: V3, c: V3, towards: V3): V3 {
  const n = normalOf(a, b, c);
  return n[0] * towards[0] + n[1] * towards[1] + n[2] * towards[2] >= 0 ? n : [-n[0], -n[1], -n[2]];
}
