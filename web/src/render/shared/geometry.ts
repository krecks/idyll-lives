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

/** A colour channel (0..1, sRGB) as the byte uploaded for it (0..255): rounded, clamped, NaN as 0. */
function byte(v: number): number {
  return v <= 0 ? 0 : v >= 1 ? 255 : (v * 255 + 0.5) | 0;
}

/**
 * Builds vertex arrays in typed arrays that grow as it goes (positions, normals and UVs in 64-bit
 * floats, as the plain arrays it used to build in held them, so `meshData` rounds them to 32 bits
 * the same; colours and the cutaway a byte a channel, as uploaded): a vertex is 72 bytes while it's
 * built, not 128.
 */
export class Geo {
  private p = new Float64Array(0);
  private n = new Float64Array(0);
  private t = new Float64Array(0);
  private c = new Uint8Array(0);
  private k = new Uint8Array(0);
  private i = new Uint32Array(0);
  /** Vertices and triangle corners written. */
  private nv = 0;
  private ni = 0;
  /** Some vertex's alpha is under 1 (the mesh then blends by vertex alpha, `MeshData.vertexAlpha`). */
  private seeThrough = false;
  /** sRGB colour applied to added vertices. */
  r = 1;
  g = 1;
  b = 1;
  a = 1;
  maskA: number = LOOK.All;
  maskB: number = LOOK.All;
  kind: number = Cut.Keep;

  /** Positions (3 a vertex), as written: a view, valid until the next vertex is added. */
  get pos(): Float64Array {
    return this.p.subarray(0, this.nv * 3);
  }
  /** Normals (3 a vertex), as `pos`. */
  get nor(): Float64Array {
    return this.n.subarray(0, this.nv * 3);
  }
  /** UVs (2 a vertex), as `pos`. */
  get uv(): Float64Array {
    return this.t.subarray(0, this.nv * 2);
  }
  /** Colours (RGBA, a byte a channel), as `pos`. */
  get col(): Uint8Array {
    return this.c.subarray(0, this.nv * 4);
  }
  /** The cutaway attribute (4 a vertex), as `pos`. */
  get cut(): Uint8Array {
    return this.k.subarray(0, this.nv * 4);
  }
  /** Triangles (3 corners each), as `pos`. */
  get idx(): Uint32Array {
    return this.i.subarray(0, this.ni);
  }
  get vertexCount(): number {
    return this.nv;
  }
  get empty(): boolean {
    return this.ni === 0;
  }

  /**
   * Room for `vertices` more vertices and `corners` more triangle corners: a full array is replaced by
   * one half as big again (or as big as asked, if that's more), so a merge of a big geometry into an
   * empty one makes it its own size at once.
   */
  private room(vertices: number, corners: number): void {
    const nv = this.nv + vertices;
    if (nv * 3 > this.p.length) {
      const cap = Math.max(nv, Math.ceil((this.p.length / 3) * 1.5), 16);
      this.p = grown(this.p, cap * 3, this.nv * 3);
      this.n = grown(this.n, cap * 3, this.nv * 3);
      this.t = grown(this.t, cap * 2, this.nv * 2);
      this.c = grown(this.c, cap * 4, this.nv * 4);
      this.k = grown(this.k, cap * 4, this.nv * 4);
    }
    const ni = this.ni + corners;
    if (ni > this.i.length) this.i = grown(this.i, Math.max(ni, Math.ceil(this.i.length * 1.5), 48), this.ni);
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

  /** A vertex in the colour and cutaway set now; its index. */
  point(p: V3, n: V3, u: number, v: number): number {
    return this.vertex(p, n, u, v);
  }

  /** A triangle of vertices already added, as given (`poly` winds its own by their normal). */
  face(a: number, b: number, c: number): this {
    this.room(0, 3);
    const i = this.i;
    i[this.ni++] = a;
    i[this.ni++] = b;
    i[this.ni++] = c;
    return this;
  }

  private vertex(p: V3, n: V3, u: number, v: number, alpha = this.a, tint?: V3): number {
    this.room(1, 0);
    const at = this.nv++;
    this.p[at * 3] = p[0];
    this.p[at * 3 + 1] = p[1];
    this.p[at * 3 + 2] = p[2];
    this.n[at * 3] = n[0];
    this.n[at * 3 + 1] = n[1];
    this.n[at * 3 + 2] = n[2];
    this.t[at * 2] = u;
    this.t[at * 2 + 1] = v;
    const c = this.c;
    c[at * 4] = byte(tint ? this.r * tint[0] : this.r);
    c[at * 4 + 1] = byte(tint ? this.g * tint[1] : this.g);
    c[at * 4 + 2] = byte(tint ? this.b * tint[2] : this.b);
    c[at * 4 + 3] = byte(alpha);
    if (alpha < 1) this.seeThrough = true;
    const k = this.k;
    k[at * 4] = this.maskA;
    k[at * 4 + 1] = this.kind;
    k[at * 4 + 2] = this.maskB;
    k[at * 4 + 3] = 0;
    return at;
  }

  /**
   * Convex polygon (3 or 4 points, any order around the perimeter) facing `n`, with explicit
   * UVs or planar UVs projected along the dominant normal axis. `alphas` sets per-vertex alpha;
   * `tints` multiplies the colour per vertex (sRGB, as the colour is: a floor's tone over a room).
   */
  poly(points: readonly V3[], n: V3, uvs?: readonly (readonly [number, number])[], alphas?: readonly number[], tints?: readonly V3[]): this {
    const ids = points.map((p, i) => {
      const [u, v] = uvs?.[i] ?? planarUV(p, n);
      return this.vertex(p, n, u, v, alphas?.[i] ?? this.a, tints?.[i]);
    });
    for (let i = 1; i + 1 < ids.length; i++) this.tri(ids[0], ids[i], ids[i + 1], n);
    return this;
  }

  /** Adds `other`'s geometry (with its cutaway data) in this geometry's colour. */
  append(other: Geo): this {
    const base = this.nv;
    const n = other.nv;
    this.take(other, 0);
    const r = byte(this.r);
    const g = byte(this.g);
    const b = byte(this.b);
    const a = byte(this.a);
    const c = this.c;
    for (let v = base; v < base + n; v++) {
      c[v * 4] = r;
      c[v * 4 + 1] = g;
      c[v * 4 + 2] = b;
      c[v * 4 + 3] = a;
    }
    if (n && this.a < 1) this.seeThrough = true;
    return this;
  }

  /** Adds `other`'s geometry as it is, its own colours and cutaway data (`append` paints it in this one's colour), `dy` m higher. */
  merge(other: Geo, dy = 0): this {
    const base = this.nv;
    this.take(other, dy);
    this.c.set(other.col, base * 4);
    if (other.seeThrough) this.seeThrough = true;
    return this;
  }

  /** `other`'s vertices (`dy` m higher) and triangles after this one's, but for their colours (`append`, `merge`). */
  private take(other: Geo, dy: number): void {
    const base = this.nv;
    const n = other.nv;
    this.room(n, other.ni);
    this.p.set(other.pos, base * 3);
    if (dy) for (let i = base * 3 + 1; i < (base + n) * 3; i += 3) this.p[i] += dy;
    this.n.set(other.nor, base * 3);
    this.t.set(other.uv, base * 2);
    this.k.set(other.cut, base * 4);
    const from = other.i;
    const to = this.i;
    for (let i = 0; i < other.ni; i++) to[this.ni + i] = base + from[i];
    this.nv += n;
    this.ni += other.ni;
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
    const p = this.p;
    const ax = p[a * 3] - p[b * 3];
    const ay = p[a * 3 + 1] - p[b * 3 + 1];
    const az = p[a * 3 + 2] - p[b * 3 + 2];
    const cx = p[c * 3] - p[b * 3];
    const cy = p[c * 3 + 1] - p[b * 3 + 1];
    const cz = p[c * 3 + 2] - p[b * 3 + 2];
    const fx = ay * cz - az * cy;
    const fy = az * cx - ax * cz;
    const fz = ax * cy - ay * cx;
    if (fx * n[0] + fy * n[1] + fz * n[2] >= 0) this.face(a, b, c);
    else this.face(a, c, b);
  }

  /** Every vertex lifted onto the ground: `ground(x, z)` m added to its height (a decal laid flat at 0, draped over a land that slopes). */
  drape(ground: (x: number, z: number) => number): this {
    const p = this.p;
    for (let i = 0; i < this.nv * 3; i += 3) p[i + 1] += ground(p[i], p[i + 2]);
    return this;
  }

  /** The geometry's arrays, with its colours and its cutaway attribute if asked for. */
  meshData(options: { colors?: boolean; cut?: boolean } = {}): MeshData {
    return {
      positions: new Float32Array(this.pos),
      normals: new Float32Array(this.nor),
      uvs: new Float32Array(this.uv),
      indices: this.nv > 65535 ? this.idx.slice() : new Uint16Array(this.idx),
      colors: options.colors ? this.col.slice() : undefined,
      cut: options.cut ? this.cut.slice() : undefined,
      vertexAlpha: !!options.colors && this.seeThrough,
    };
  }
}

/** `from`'s first `used` values in a new array `size` long (the rest 0). */
function grown<T extends Float64Array | Uint8Array | Uint32Array>(from: T, size: number, used: number): T {
  const to = new (from.constructor as new (n: number) => T)(size);
  to.set(from.subarray(0, used));
  return to;
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
