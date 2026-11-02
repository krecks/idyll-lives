// `render/shared/contactShadow.ts`: contact shadows shaped like what casts them (`bun test`).
import { describe, expect, test } from 'bun:test';
import { CONTACT_TILE, contactAtlas, contactShape, type ContactPiece, type ContactShape } from '../src/render/shared/contactShadow';

const T = CONTACT_TILE;

/** A flat quad from (x0, z0) to (x1, z1) at height y. */
function quad(x0: number, z0: number, x1: number, z1: number, y: number): ContactPiece {
  return { positions: [x0, y, z0, x1, y, z0, x1, y, z1, x0, y, z1], indices: [0, 1, 2, 0, 2, 3] };
}

/** The texel of a shape's tile over model point (x, z). */
function at(shape: ContactShape, x: number, z: number): number {
  const i = Math.floor(((x - shape.minX) / (shape.maxX - shape.minX)) * T);
  const j = Math.floor(((z - shape.minZ) / (shape.maxZ - shape.minZ)) * T);
  return shape.alpha[j * T + i];
}

describe('contactShape', () => {
  test('nothing to lay flat: no shape', () => {
    expect(contactShape([])).toBeUndefined();
    expect(contactShape([{ positions: [0, 0, 0], indices: [] }])).toBeUndefined();
  });

  test('covers the outline with room round it, its edge clear', () => {
    const shape = contactShape([quad(-0.5, -0.3, 0.5, 0.3, 0)])!;
    expect(shape.minX).toBeLessThan(-0.5);
    expect(shape.maxZ).toBeGreaterThan(0.3);
    for (let k = 0; k < T; k++) {
      expect(shape.alpha[k]).toBe(0);
      expect(shape.alpha[(T - 1) * T + k]).toBe(0);
      expect(shape.alpha[k * T]).toBe(0);
      expect(shape.alpha[k * T + T - 1]).toBe(0);
    }
  });

  test('darkest where it touches the floor, lighter the higher it is', () => {
    const floor = contactShape([quad(-0.5, -0.5, 0.5, 0.5, 0)])!;
    // (The model's bottom is its lowest point: a foot under the raised top.)
    const table = contactShape([quad(-0.5, -0.5, 0.5, 0.5, 0.75), quad(0.4, 0.4, 0.45, 0.45, 0)])!;
    expect(at(floor, 0, 0)).toBeGreaterThan(140);
    expect(at(table, 0, 0)).toBeLessThan(at(floor, 0, 0) * 0.6);
    expect(at(table, 0, 0)).toBeGreaterThan(0);
  });

  test('keeps its shape: rows along +z, columns along +x', () => {
    // An L: a long arm along x at the near z, a short one up z at the low x.
    const shape = contactShape([quad(-1, -1, 1, -0.6, 0), quad(-1, -0.6, -0.6, 1, 0)])!;
    expect(at(shape, 0.8, -0.8)).toBeGreaterThan(100);
    expect(at(shape, -0.8, 0.8)).toBeGreaterThan(100);
    expect(at(shape, 0.6, 0.6)).toBe(0);
  });

  test('either winding', () => {
    const q = quad(-0.5, -0.5, 0.5, 0.5, 0);
    const flipped = { positions: q.positions, indices: [0, 2, 1, 0, 3, 2] };
    expect(contactShape([flipped])!.alpha).toEqual(contactShape([q])!.alpha);
  });
});

describe('contactAtlas', () => {
  test('tiles side by side, each where its rectangle says', () => {
    const tiles = [new Uint8Array(T * T).fill(5), new Uint8Array(T * T).fill(7), new Uint8Array(T * T).fill(9)];
    const atlas = contactAtlas(tiles);
    expect([atlas.width, atlas.height]).toEqual([2 * T, 2 * T]);
    atlas.rects.forEach((rect, n) => {
      const [x, y] = [Math.round(rect[0] * atlas.width), Math.round(rect[1] * atlas.height)];
      expect([rect[2], rect[3]]).toEqual([0.5, 0.5]);
      expect(atlas.texels[(y + 5) * atlas.width + x + 3]).toBe(tiles[n][5 * T + 3]);
    });
  });
});
