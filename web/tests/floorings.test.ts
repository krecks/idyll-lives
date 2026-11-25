// The floors households lay in their homes (`game/floorings.ts`, `housegen.json` `floorings`): every
// covering one the content has, the entrance hall never laid like the living room, the same floors for the
// same town, every floor inside a house's walls, and the new coverings in use about the town.
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('floorings', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  usePublicFetch();
  const { FLOOR_FINISHES } = await import('../src/content/content');
  const { generateWorld, layoutTown, useTownContent } = await import('../src/game/town');
  const { floorPicker, laidFloors } = await import('../src/game/floorings');
  const { mulberry32 } = await import('../src/game/noise');

  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);
  const spec = t.gen!.floorings!;
  const ids = new Set(content.floorCoverings.map((c) => c.id));
  const finishOf = (id: string) => content.floorCoverings.find((c) => c.id === id)!.finish;
  const classes = [undefined, ...Object.values(t.gen!.classes).map((c) => c.label)];

  test('the content has a covering of every finish', () => {
    const finishes = new Set(content.floorCoverings.map((c) => c.finish));
    for (const f of FLOOR_FINISHES) expect(finishes.has(f)).toBe(true);
  });

  test('every covering the floorings name is one the content has', () => {
    const named = [spec.rooms, spec.below ?? {}, ...Object.values(spec.classes ?? {})].flatMap((byKind) => Object.values(byKind).flatMap((w) => Object.keys(w)));
    expect(named.filter((id) => !ids.has(id))).toEqual([]);
  });

  test('the entrance hall is never laid like the living room, nor in wood', () => {
    for (const cls of classes) {
      for (let seed = 1; seed <= 300; seed++) {
        const pick = floorPicker(spec, cls, ids, mulberry32(seed));
        // Whichever is asked first.
        const [entry, living] = seed % 2 ? [pick('entry'), pick('living')] : [pick('living'), pick('entry')].reverse();
        expect(entry).not.toBeNull();
        expect(entry).not.toBe(living);
        expect(['wood', 'whitewash', 'parquet', 'laminate']).not.toContain(finishOf(entry!));
      }
    }
  });

  test('a house lays one covering a kind, the entrance hall first', () => {
    const pick = floorPicker(spec, 'villa', ids, mulberry32(7));
    const rooms = [
      { kind: 'living', x: 0, z: 0, w: 4, d: 4 },
      { kind: 'bathroom', x: 4, z: 0, w: 2, d: 2 },
      { kind: 'entry', x: 4, z: 2, w: 2, d: 2 },
      { kind: 'bathroom', x: 6, z: 0, w: 2, d: 2 },
      { kind: 'stockroom', x: 6, z: 2, w: 2, d: 2 },
    ];
    const laid = laidFloors(rooms, pick);
    expect(laid.map((f) => f.x * 10 + f.z)).toEqual([42, 0, 40, 60]);
    expect(laid[2].covering).toBe(laid[3].covering);
  });

  const town = generateWorld(content, t, 'country', 4242, undefined, undefined, { town: 'hamlet', villages: 'none' });
  const layout = layoutTown(content, t, town, null);

  test('the town lays its homes floors, the same each time, inside their walls', () => {
    expect(layout.floors.length).toBeGreaterThan(50);
    expect(layoutTown(content, t, town, null).floors).toEqual(layout.floors);
    for (const f of layout.floors) {
      expect(ids.has(f.covering)).toBe(true);
      expect([f.x, f.z, f.w, f.d].every(Number.isInteger)).toBe(true);
      expect(f.w > 0 && f.d > 0).toBe(true);
    }
    // The ground storey's: each inside the box of a lot's walls.
    const boxes = layout.built.filter((b) => b !== null);
    for (const f of layout.floors.filter((f) => f.z < town.depth)) {
      expect(boxes.some((b) => f.x >= b.x && f.z >= b.z && f.x + f.w <= b.x + b.w && f.z + f.d <= b.z + b.d)).toBe(true);
    }
    // No tile laid twice.
    const seen = new Set<string>();
    for (const f of layout.floors) for (let z = f.z; z < f.z + f.d; z++) for (let x = f.x; x < f.x + f.w; x++) {
      const key = `${x}:${z}`;
      expect(seen.has(key)).toBe(false);
      seen.add(key);
    }
  });

  test('the homes about the town use the new floors', () => {
    const used = new Set(layout.floors.map((f) => finishOf(f.covering)));
    const fresh = FLOOR_FINISHES.slice(5).filter((f) => used.has(f));
    expect(fresh.length).toBeGreaterThanOrEqual(6);
  });
});
