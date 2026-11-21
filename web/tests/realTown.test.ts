// The real town's contracts (`core/protocol.ts`): a household's plot may be a flat, past the ground
// lots; `flatOf` and `lotOf` take it to its flat and its building's lot, and a ground lot to itself.
// The renderer's paving reads the fixture's town: its centre, court and car park paved, its bays drawn.
import { describe, expect, test } from 'bun:test';
import { flatOf, lotOf } from '../src/core/protocol';
import { townPaving } from '../src/render/shared/townPaving';
import { BUILDING, FLATS, realTown, STOREY, W } from './fixtures/realTown';

describe('the real town', () => {
  test('a home in a flat goes to its flat and its building', () => {
    const t = realTown();
    for (const h of t.households) {
      const flat = flatOf(t, h.plot!);
      expect(flat?.id).toBe(h.plot!);
      expect(lotOf(t, h.plot!)).toBe(BUILDING);
    }
    expect(flatOf(t, BUILDING)).toBeUndefined();
    expect(lotOf(t, BUILDING)).toBe(BUILDING);
    // (Flats out of order are found by id too; an id past them is no flat and its own lot.)
    const shuffled = { ...t, flats: [...t.flats!].reverse() };
    expect(flatOf(shuffled, 2)?.storey).toBe(0);
    expect(flatOf(t, 9)).toBeUndefined();
    expect(lotOf(t, 9)).toBe(9);
  });

  test('its flats follow the ground lots, a door on each one’s own storey', () => {
    const t = realTown();
    expect(t.flats!.map((f) => f.id)).toEqual([...FLATS]);
    expect(Math.min(...FLATS)).toBe(t.plots.length);
    for (const f of t.flats!) expect(Math.floor(f.door[1] / STOREY)).toBe(f.storey);
    expect(t.parking!.every((b) => b.park === 0)).toBe(true);
  });

  test('its centre, court and car park are paved; its bays are drawn with the car park’s P sign', () => {
    const p = townPaving(realTown());
    expect(p.pieces.map((q) => q.kind).sort()).toEqual(['carpark', 'centre', 'court', 'square']);
    expect(p.mask(W, STOREY).some((v) => v === 1)).toBe(true);
    expect(p.bays).toHaveLength(4);
    expect(p.signs).toHaveLength(1);
  });
});
