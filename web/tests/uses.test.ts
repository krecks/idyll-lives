// The uses the player paints in step 2 (`game/uses.ts`): packed run-length and back as they were,
// an empty layer with no use anywhere, a cell read at a town point, and a hash that follows the
// cells (not how their runs were split).
import { describe, expect, test } from 'bun:test';
import { layoutHash, NO_USE, packUses, unpackUses, USE_CELL, USE_CODE, USE_COLORS, USE_KINDS, USE_LABELS, useAt, useGrid, useOfCode, usesEmpty, usesHash } from '../src/game/uses';
import type { NeighbourhoodDraft } from '../src/game/town';

/** A layer's cells, `cols` × `rows`, painted with a few blocks of each use. */
function painted(cols: number, rows: number): Uint8Array {
  const codes = new Uint8Array(cols * rows);
  for (let j = 0; j < rows; j++) for (let i = 0; i < cols; i++) codes[j * cols + i] = ((i >> 2) + (j >> 3)) % 6;
  return codes;
}

describe('the uses', () => {
  test('each use has its code, from 1, a colour and a name; 0 is none', () => {
    expect(USE_KINDS.map((k) => USE_CODE[k])).toEqual([1, 2, 3, 4, 5]);
    for (const k of USE_KINDS) {
      expect(useOfCode(USE_CODE[k])).toBe(k);
      expect(USE_COLORS[k]).toMatch(/^#[0-9A-F]{6}$/);
      expect(USE_LABELS[k].length).toBeGreaterThan(0);
    }
    expect(useOfCode(NO_USE)).toBeNull();
    expect(useOfCode(6)).toBeNull();
    expect(USE_COLORS.homes).toBe('#F2C94C');
  });

  test('packed and unpacked, the cells are as they were, in runs of one code', () => {
    const [cols, rows] = [37, 23];
    const codes = painted(cols, rows);
    const layer = packUses(codes, cols, rows);
    expect(layer).toMatchObject({ v: 1, cols, rows });
    expect(layer.cells.length % 2).toBe(0);
    expect(layer.cells.filter((_, k) => k % 2 === 0).reduce((a, b) => a + b, 0)).toBe(cols * rows);
    for (let k = 3; k < layer.cells.length; k += 2) expect(layer.cells[k]).not.toBe(layer.cells[k - 2]);
    expect([...unpackUses(layer)]).toEqual([...codes]);
    // (Kept with the layer: the same array again.)
    expect(unpackUses(layer)).toBe(unpackUses(layer));
    // (Through JSON too, as a draft crosses to the worker.)
    expect([...unpackUses(JSON.parse(JSON.stringify(layer)))]).toEqual([...codes]);
  });

  test('a code no use has is packed as none', () => {
    const layer = packUses([1, 9, 2, -1], 4, 1);
    expect([...unpackUses(layer)]).toEqual([1, 0, 2, 0]);
  });

  test('an empty layer has no use anywhere, and is what packing no uses gives', () => {
    const layer = usesEmpty(10, 6);
    expect(layer).toEqual({ v: 1, cols: 10, rows: 6, cells: [60, NO_USE] });
    expect(unpackUses(layer).every((c) => c === NO_USE)).toBe(true);
    expect(packUses(new Uint8Array(60), 10, 6)).toEqual(layer);
    expect(usesEmpty(0, 0).cells).toEqual([]);
    expect(useGrid(1001, 400)).toEqual({ cols: Math.ceil(1001 / USE_CELL), rows: 400 / USE_CELL });
  });

  test('the use at a town point is its cell\'s; none off the layer', () => {
    const codes = new Uint8Array(4 * 3);
    codes[1 * 4 + 2] = USE_CODE.shops;
    const layer = packUses(codes, 4, 3);
    expect(useAt(layer, 2 * USE_CELL + 0.5, 1 * USE_CELL + 3.9)).toBe('shops');
    expect(useAt(layer, 0.5, 0.5)).toBeNull();
    expect(useAt(layer, -0.1, 1)).toBeNull();
    expect(useAt(layer, 4 * USE_CELL, 1)).toBeNull();
  });

  test('the hash: the same cells the same, however their runs were split; a cell changed another', () => {
    const [cols, rows] = [40, 30];
    const codes = painted(cols, rows);
    const layer = packUses(codes, cols, rows);
    const hash = usesHash(layer);
    expect(usesHash(packUses(codes, cols, rows))).toBe(hash);
    expect(usesHash(JSON.parse(JSON.stringify(layer)))).toBe(hash);
    // (The first run split in two, and an empty run put in.)
    const split = { ...layer, cells: [1, layer.cells[1], layer.cells[0] - 1, layer.cells[1], 0, 3, ...layer.cells.slice(2)] };
    expect(usesHash(split)).toBe(hash);
    const changed = codes.slice();
    changed[cols * rows - 1] = (changed[cols * rows - 1] + 1) % 6;
    expect(usesHash(packUses(changed, cols, rows))).not.toBe(hash);
    expect(usesHash(usesEmpty(cols, rows))).not.toBe(usesHash(usesEmpty(rows, cols)));
    expect(usesHash(null)).toBe('');
  });

  test("a layout's hash follows its roads and uses, not its lots", () => {
    const plan = {
      width: 400,
      depth: 400,
      roads: [{ points: [[0, 10], [200, 10]], width: 8 }],
      greens: [],
      slots: [],
      open: { areas: [], uses: usesEmpty(100, 100) },
    } as unknown as NeighbourhoodDraft;
    const hash = layoutHash(plan);
    expect(layoutHash(structuredClone(plan))).toBe(hash);
    expect(layoutHash({ ...plan, slots: [{} as NeighbourhoodDraft['slots'][number]] })).toBe(hash);
    expect(layoutHash({ ...plan, roads: [{ ...plan.roads[0], width: 6 }] })).not.toBe(hash);
    const uses = unpackUses(plan.open!.uses!).slice();
    uses[5] = USE_CODE.homes;
    expect(layoutHash({ ...plan, open: { ...plan.open!, uses: packUses(uses, 100, 100) } })).not.toBe(hash);
  });
});
