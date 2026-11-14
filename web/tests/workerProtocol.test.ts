// The sim worker's messages on the main thread: world updates merged (`core/bridge.ts` `mergeWorld`),
// the shared snapshot's triple buffer (`core/snapshot.ts`), and the newest thing a snapshot's
// residents use (`newestObjectUsed`: a world the main thread lacks goes at once).
import { describe, expect, test } from 'bun:test';
import { mergeWorld } from '../src/core/bridge';
import { newestObjectUsed, type ObjectPlacement, type WorldUpdate } from '../src/core/protocol';
import { createSharedSnapshot, SharedSnapshotReader, SharedSnapshotWriter, simFields, type SnapshotLayout } from '../src/core/snapshot';

const thing = (id: number, open = false): ObjectPlacement => ({ id, def: 'door', x: id, z: 0, rot: 0, w: 1, d: 1, quality: 0, sellValue: null, open });
const meshes = () => {
  const m = () => ({ positions: new Float32Array(3), normals: new Float32Array(3), uvs: new Float32Array(2), indices: new Uint32Array(3) });
  return { walls: m(), wallsLow: m(), floors: m() };
};
const first = (): WorldUpdate => ({
  version: 1,
  lotVersion: 1,
  mode: 'living',
  width: 4,
  depth: 4,
  objects: [thing(1), thing(2), thing(3)],
  sims: [],
  households: [],
  plots: [],
  exits: [],
  walls: [],
  openings: [],
  meta: null,
  garages: [{ plot: 0, x0: 0, z0: 0, x1: 1, z1: 1 }],
});

describe('mergeWorld', () => {
  test('the first update is the whole world, its rooms and meshes beside it', () => {
    const rooms = new Uint16Array(16);
    const w = mergeWorld(null, null, { world: first(), rooms, meshes: meshes() });
    expect(w.objects.length).toBe(3);
    expect(w.rooms).toBe(rooms);
    expect(w.meshes.walls.indices.length).toBe(3);
  });

  test('what an update leaves out stays the same object; changed things by index', () => {
    const shown = mergeWorld(null, null, { world: first(), rooms: new Uint16Array(16), meshes: meshes() });
    const next = mergeWorld(shown, shown, { world: { version: 2, lotVersion: 1, objectChanges: [[1, thing(2, true)]] } });
    expect(next).not.toBe(shown);
    expect(next.version).toBe(2);
    expect(next.walls).toBe(shown.walls);
    expect(next.rooms).toBe(shown.rooms);
    expect(next.meshes).toBe(shown.meshes);
    expect(next.meta).toBe(shown.meta);
    expect(next.sims).toBe(shown.sims);
    // The shown world's list is left as it was; the unchanged things are the same objects.
    expect(next.objects).not.toBe(shown.objects);
    expect(shown.objects[1].open).toBe(false);
    expect(next.objects[1].open).toBe(true);
    expect(next.objects[0]).toBe(shown.objects[0]);
    expect(next.objects[2]).toBe(shown.objects[2]);
  });

  test('updates nobody has seen yet add up in place', () => {
    const shown = mergeWorld(null, null, { world: first(), rooms: new Uint16Array(16), meshes: meshes() });
    const a = mergeWorld(shown, shown, { world: { version: 2, lotVersion: 1, objectChanges: [[0, thing(1, true)]] } });
    const b = mergeWorld(a, shown, { world: { version: 3, lotVersion: 1, objectChanges: [[2, thing(3, true)]] } });
    expect(b).toBe(a);
    expect(b.version).toBe(3);
    expect(b.objects.map((o) => o.open)).toEqual([true, false, true]);
    expect(shown.objects.map((o) => o.open)).toEqual([false, false, false]);
  });

  test('a part unset is gone; a new view brings meshes only', () => {
    const shown = mergeWorld(null, null, { world: first(), rooms: new Uint16Array(16), meshes: meshes() });
    const sold = mergeWorld(shown, shown, { world: { version: 2, lotVersion: 1, unset: ['garages'] } });
    expect('garages' in sold).toBe(false);
    expect(shown.garages?.length).toBe(1);
    const moved = meshes();
    const viewed = mergeWorld(sold, sold, { world: { version: 2, lotVersion: 1 }, meshes: moved });
    expect(viewed.meshes).toBe(moved);
    expect(viewed.walls).toBe(shown.walls);
    expect(viewed.objects).toBe(shown.objects);
  });

  test('a new list of things replaces the old', () => {
    const shown = mergeWorld(null, null, { world: first(), rooms: new Uint16Array(16), meshes: meshes() });
    const next = mergeWorld(shown, shown, { world: { version: 2, lotVersion: 1, objects: [thing(1)] } });
    expect(next.objects.length).toBe(1);
    expect(shown.objects.length).toBe(3);
  });

  test('a splice adds and takes away things, the others kept as they were', () => {
    const shown = mergeWorld(null, null, { world: first(), rooms: new Uint16Array(16), meshes: meshes() });
    // Thing 2 sold, things 4 and 5 bought in its place, then thing 5 (now at index 2) opened.
    const next = mergeWorld(shown, shown, { world: { version: 2, lotVersion: 1, objectSplice: [1, 1, [thing(4), thing(5)]], objectChanges: [[2, thing(5, true)]] } });
    expect(next.objects.map((o) => o.id)).toEqual([1, 4, 5, 3]);
    expect(next.objects[2].open).toBe(true);
    expect(next.objects[0]).toBe(shown.objects[0]);
    expect(next.objects[3]).toBe(shown.objects[2]);
    expect(shown.objects.map((o) => o.id)).toEqual([1, 2, 3]);
    // Bought at the end, on a world nobody has seen yet: changed in place.
    const end = mergeWorld(next, shown, { world: { version: 3, lotVersion: 1, objectSplice: [4, 0, [thing(6)]] } });
    expect(end).toBe(next);
    expect(end.objects.map((o) => o.id)).toEqual([1, 4, 5, 3, 6]);
  });
});

describe('shared snapshot', () => {
  const layout = { capacity: 8 } as SnapshotLayout;

  test('the reader gets the newest snapshot, once', () => {
    const buffer = createSharedSnapshot(layout);
    const writer = new SharedSnapshotWriter(buffer, layout.capacity);
    const reader = new SharedSnapshotReader(buffer, layout.capacity);
    const into = new Float32Array(layout.capacity);
    expect(reader.read(into)).toBe(false);
    for (let step = 1; step <= 5; step++) {
      writer.write(new Float32Array(layout.capacity).fill(step), step);
      if (step === 3) continue;
      expect(reader.read(into)).toBe(true);
      expect(reader.step).toBe(step);
      expect([...into]).toEqual(new Array(layout.capacity).fill(step));
      expect(reader.read(into)).toBe(false);
    }
  });

  test('several writes between reads: the newest, whole', () => {
    const buffer = createSharedSnapshot(layout);
    const writer = new SharedSnapshotWriter(buffer, layout.capacity);
    const reader = new SharedSnapshotReader(buffer, layout.capacity);
    const into = new Float32Array(layout.capacity);
    for (let round = 1; round < 50; round++) {
      for (let k = 0; k < (round % 4) + 1; k++) {
        const step = round * 10 + k;
        writer.write(new Float32Array(layout.capacity).fill(step), step);
      }
      expect(reader.read(into)).toBe(true);
      expect(new Set(into).size).toBe(1);
      expect(into[0]).toBe(reader.step);
      expect(reader.step).toBe(round * 10 + (round % 4));
    }
  });
});

describe('newestObjectUsed', () => {
  const layout = { headerLen: 2, simStride: 3, header: { simCount: 0 }, sim: simFields(['id', 'x', 'object']) } as SnapshotLayout;
  /** A snapshot of residents using these things (-1: none), with a stale row past the count. */
  const snapshot = (...objects: number[]) => [objects.length, 0, ...objects.flatMap((object, id) => [id, 0, object]), 0, 0, 99];

  test('the highest id any resident uses or walks to', () => {
    expect(newestObjectUsed(snapshot(4, -1, 17, 9), layout)).toBe(17);
  });

  test('none: -1 (nobody uses anything, or nobody is there); rows past the count are not read', () => {
    expect(newestObjectUsed(snapshot(-1, -1), layout)).toBe(-1);
    expect(newestObjectUsed(snapshot(), layout)).toBe(-1);
  });
});
