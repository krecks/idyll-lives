// `Geo` (render/shared/geometry.ts) builds in typed arrays, its colours and cutaway a byte a channel: what it
// hands the renderer (`meshData`) is what it was when it built in plain arrays, number for number. `Before` is
// that builder as it was (kept here, word for word, as the reference), and both build the same geometries.
import { describe, expect, test } from 'bun:test';
import { Cut, Geo, LOOK, planarUV, type MeshData, type V3 } from '../src/render/shared/geometry';

// The builder as it was before it built in typed arrays (the reference).
class Before {
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

  private vertex(p: V3, n: V3, u: number, v: number, alpha = this.a, tint?: V3): number {
    this.pos.push(p[0], p[1], p[2]);
    this.nor.push(n[0], n[1], n[2]);
    this.uv.push(u, v);
    if (tint) this.col.push(this.r * tint[0], this.g * tint[1], this.b * tint[2], alpha);
    else this.col.push(this.r, this.g, this.b, alpha);
    this.cut.push(this.maskA, this.kind, this.maskB, 0);
    return this.pos.length / 3 - 1;
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
  append(other: Before): this {
    const base = this.pos.length / 3;
    const n = other.pos.length / 3;
    this.copy(this.pos, other.pos);
    this.copy(this.nor, other.nor);
    this.copy(this.uv, other.uv);
    this.copy(this.cut, other.cut);
    this.grow(this.col, n * 4);
    const colAt = this.col.length - n * 4;
    for (let i = 0; i < n; i++) {
      const at = colAt + i * 4;
      this.col[at] = this.r;
      this.col[at + 1] = this.g;
      this.col[at + 2] = this.b;
      this.col[at + 3] = this.a;
    }
    this.grow(this.idx, other.idx.length);
    const idxAt = this.idx.length - other.idx.length;
    for (let i = 0; i < other.idx.length; i++) this.idx[idxAt + i] = base + other.idx[i];
    return this;
  }

  /** Adds `other`'s geometry as it is, its own colours and cutaway data (`append` paints it in this one's colour), `dy` m higher. */
  merge(other: Before, dy = 0): this {
    const base = this.pos.length / 3;
    if (dy) {
      const from = other.pos;
      const at = this.pos.length;
      this.grow(this.pos, from.length);
      for (let i = 0; i < from.length; i += 3) {
        this.pos[at + i] = from[i];
        this.pos[at + i + 1] = from[i + 1] + dy;
        this.pos[at + i + 2] = from[i + 2];
      }
    } else this.copy(this.pos, other.pos);
    this.copy(this.nor, other.nor);
    this.copy(this.uv, other.uv);
    this.copy(this.col, other.col);
    this.copy(this.cut, other.cut);
    this.grow(this.idx, other.idx.length);
    const idxAt = this.idx.length - other.idx.length;
    for (let i = 0; i < other.idx.length; i++) this.idx[idxAt + i] = base + other.idx[i];
    return this;
  }

  /** Adds `from`'s numbers to the end of `to`, one by one (a spread would overflow the stack on a big mesh). */
  private copy(to: number[], from: readonly number[]): void {
    for (let i = 0; i < from.length; i++) to.push(from[i]);
  }

  /** Makes `to` `n` numbers longer, the new ones 0, for the caller to fill in place. (Pushed, not a longer `length`: that would leave the array with holes, which V8 keeps slower for good.) */
  private grow(to: number[], n: number): void {
    for (let i = 0; i < n; i++) to.push(0);
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

  /** Every vertex lifted onto the ground: `ground(x, z)` m added to its height (a decal laid flat at 0, draped over a land that slopes). */
  drape(ground: (x: number, z: number) => number): this {
    const p = this.pos;
    for (let i = 0; i < p.length; i += 3) p[i + 1] += ground(p[i], p[i + 2]);
    return this;
  }

  /** The geometry's arrays, with its colours and its cutaway attribute if asked for. */
  meshData(options: { colors?: boolean; cut?: boolean } = {}): MeshData {
    // One pass for the colours, which are a byte a channel: `Uint8Array.from(col, fn)` calls
    // back per number and allocates a wrapper per call, and the alpha scan was a second pass.
    let colors: Uint8Array | undefined;
    let vertexAlpha = false;
    if (options.colors) {
      const col = this.col;
      colors = new Uint8Array(col.length);
      for (let i = 0; i < col.length; i++) {
        const v = col[i];
        colors[i] = v <= 0 ? 0 : v >= 1 ? 255 : (v * 255 + 0.5) | 0;
        if ((i & 3) === 3 && v < 1) vertexAlpha = true;
      }
    }
    return {
      positions: new Float32Array(this.pos),
      normals: new Float32Array(this.nor),
      uvs: new Float32Array(this.uv),
      indices: this.pos.length / 3 > 65535 ? new Uint32Array(this.idx) : new Uint16Array(this.idx),
      colors,
      cut: options.cut ? new Uint8Array(this.cut) : undefined,
      vertexAlpha,
    };
  }
}


/** The same steps on either builder: boxes and polygons in several colours, see-through, tinted, cut, with explicit UVs. */
function build<G extends Geo | Before>(g: G, seed: number): G {
  for (let k = 0; k < 6; k++) {
    const x = seed + k * 1.5;
    g.color(`#${(0x102030 * (k + 1)).toString(16).padStart(6, '0')}`, k === 3 ? 0.4 : 1).box(x, 0, 0, x + 1, 1 + k * 0.25, 1, k % 2 ? 'py' : '');
  }
  g.cutting(Cut.Clamp, LOOK.PosX | LOOK.NegZ, LOOK.NegX).color('#7a5c3e');
  const quad: V3[] = [[seed, 0, 2], [seed + 1.3, 0.1, 2], [seed + 1.2, 0.9, 2.2], [seed, 1, 2.1]];
  g.poly(quad, [0, 0, 1], [[0, 0], [1, 0], [1, 1], [0, 1]], [1, 0.5, 0.999, 1], [[0.9, 0.8, 1], [1, 1, 1], [1.3, -0.2, 0.5], [0.5, 0.5, 0.5]]);
  g.cutting(Cut.Hide).poly([[seed, 2, 0], [seed + 2, 2, 0], [seed + 1, 3, 0.5]], [0, 0.5, 1]);
  g.cutting(Cut.Keep);
  return g;
}

/** What the renderer is given, as plain numbers. */
const handed = (d: MeshData) => ({
  positions: Array.from(d.positions),
  normals: Array.from(d.normals),
  uvs: Array.from(d.uvs),
  indices: [d.indices.constructor.name, ...d.indices],
  colors: d.colors && Array.from(d.colors),
  cut: d.cut && Array.from(d.cut),
  vertexAlpha: d.vertexAlpha,
});

/** `fn` on a `Geo` and on a `Before` (the same steps: the reference is driven as a `Geo`), and what each hands over compared. */
const both = (fn: (g: Geo, ...others: Geo[]) => Geo, ...seeds: number[]) => {
  const now = fn(new Geo(), ...seeds.map((s) => build(new Geo(), s)));
  const was = fn(new Before() as unknown as Geo, ...seeds.map((s) => build(new Before(), s) as unknown as Geo)) as unknown as Before;
  for (const options of [{}, { colors: true }, { colors: true, cut: true }]) expect(handed(now.meshData(options))).toEqual(handed(was.meshData(options)));
  return { now, was };
};

describe('Geo builds what it built in plain arrays', () => {
  test('polygons and boxes', () => {
    const { now, was } = both((g) => build(g, 0));
    expect(Array.from(now.pos)).toEqual(was.pos);
    expect(Array.from(now.idx)).toEqual(was.idx);
    expect(now.vertexCount).toBe(was.pos.length / 3);
    expect(now.empty).toBe(false);
    expect(new Geo().empty).toBe(true);
  });

  test('append paints the other geometry in this one’s colour', () => {
    both((g, a, b) => build(g, 5).color('#abcdef', 0.5).append(a).color('#123456').append(b), 20, 40);
  });

  test('merge keeps the other geometry’s colours and lifts it by dy', () => {
    for (const dy of [0, 2.5]) both((g, a, b) => build(g, 0).merge(a, dy).merge(b), 30, 60);
    // (Into an empty one too: made its size at once.)
    both((g, a) => g.merge(a), 7);
  });

  test('drape lifts every vertex onto the ground', () => {
    both((g, a) => g.merge(a).drape((x, z) => Math.sin(x) * 0.3 + z * 0.01), 3);
  });

  test('see-through only where a vertex is', () => {
    const opaque = new Geo().color('#808080').box(0, 0, 0, 1, 1, 1);
    expect(opaque.meshData({ colors: true }).vertexAlpha).toBe(false);
    expect(opaque.meshData().vertexAlpha).toBe(false);
    expect(new Geo().merge(opaque).append(new Geo().box(0, 0, 0, 1, 1, 1)).meshData({ colors: true }).vertexAlpha).toBe(false);
    expect(new Geo().color('#808080', 0.5).append(opaque).meshData({ colors: true }).vertexAlpha).toBe(true);
    expect(new Geo().merge(new Geo().color('#808080', 0.2).box(0, 0, 0, 1, 1, 1)).meshData({ colors: true }).vertexAlpha).toBe(true);
  });

  test('a big geometry has 32-bit triangles, as before', () => {
    both((g) => {
      for (let i = 0; i < 2800; i++) g.box(i, 0, 0, i + 1, 1, 1);
      return g;
    });
  });

  test('vertices added one by one, and triangles as given', () => {
    const g = new Geo().color('#FFFFFF', 0.45);
    const a = g.point([0, 0, 0], [0, 1, 0], 0, 0);
    const b = g.point([1, 0, 0], [0, 1, 0], 0, 0);
    const c = g.point([0, 0, 1], [0, 1, 0], 0, 0);
    g.face(a, c, b);
    const d = g.meshData({ colors: true, cut: true });
    expect(Array.from(d.indices)).toEqual([0, 2, 1]);
    expect(Array.from(d.colors!)).toEqual([255, 255, 255, 115, 255, 255, 255, 115, 255, 255, 255, 115]);
    expect(Array.from(d.cut!)).toEqual([15, 0, 15, 0, 15, 0, 15, 0, 15, 0, 15, 0]);
    expect(d.vertexAlpha).toBe(true);
  });
});
