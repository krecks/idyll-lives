import { describe, expect, test } from 'bun:test';
import { Geo } from '../src/render/shared/geometry';

/** A few boxes and a see-through panel in several colours: enough of every attribute to compare. */
function sample(seed: number): Geo {
  const g = new Geo();
  for (let k = 0; k < 6; k++) {
    const x = seed + k * 1.5;
    g.color(`#${(0x102030 * (k + 1)).toString(16).padStart(6, '0')}`, k === 3 ? 0.4 : 1).box(x, 0, 0, x + 1, 1 + k * 0.25, 1, k % 2 ? 'py' : '');
  }
  return g;
}

/** The arrays as `append` and `merge` wrote them before they filled in place (spread and pushes). */
function before(into: Geo, other: Geo, how: 'append' | 'merge', dy = 0) {
  const pos = [...into.pos];
  const nor = [...into.nor];
  const uv = [...into.uv];
  const col = [...into.col];
  const cut = [...into.cut];
  const idx = [...into.idx];
  const base = pos.length / 3;
  const n = other.pos.length / 3;
  if (how === 'append') {
    pos.push(...other.pos);
    for (let i = 0; i < n; i++) col.push(into.r, into.g, into.b, into.a);
  } else {
    for (let i = 0; i < other.pos.length; i += 3) pos.push(other.pos[i], other.pos[i + 1] + dy, other.pos[i + 2]);
    col.push(...other.col);
  }
  nor.push(...other.nor);
  uv.push(...other.uv);
  cut.push(...other.cut);
  for (const i of other.idx) idx.push(base + i);
  return { pos, nor, uv, col, cut, idx };
}

const arrays = (g: Geo) => ({ pos: g.pos, nor: g.nor, uv: g.uv, col: g.col, cut: g.cut, idx: g.idx });

describe('Geo filled in place', () => {
  test('append paints the other geometry in this one’s colour, as before', () => {
    const into = sample(0).color('#abcdef', 0.5);
    const other = sample(20);
    const want = before(into, other, 'append');
    expect(arrays(into.append(other))).toEqual(want);
  });

  test('merge keeps the other geometry’s colours and lifts it by dy, as before', () => {
    for (const dy of [0, 2.5]) {
      const into = sample(0);
      const other = sample(30);
      const want = before(into, other, 'merge', dy);
      expect(arrays(into.merge(other, dy))).toEqual(want);
    }
  });

  test('meshData packs the colours a byte a channel and finds see-through vertices, as before', () => {
    const g = sample(0).merge(sample(10));
    g.col.push(-0.2, 1.3, Number.NaN, 0.999);
    g.pos.push(0, 0, 0);
    const data = g.meshData({ colors: true });
    expect(data.colors).toEqual(Uint8Array.from(g.col, (v) => Math.min(255, Math.max(0, Math.round(v * 255)))));
    expect(data.vertexAlpha).toBe(true);
    const opaque = new Geo().color('#808080').box(0, 0, 0, 1, 1, 1);
    expect(opaque.meshData({ colors: true }).vertexAlpha).toBe(false);
    expect(opaque.meshData().vertexAlpha).toBe(false);
  });
});
