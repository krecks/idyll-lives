// The main menu's showcase kept from visit to visit (`game/showcaseCache.ts`): an entry read back is
// the same town and the same houses, and the renderer finds its (empty) meshes. What is kept goes
// through JSON, which is why the meshes are left out and a lot without a generated house goes as null.
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import type { HouseTemplate, NeighbourhoodDraft } from '../src/game/town';
import { keepEntry, readEntry, type Showcase } from '../src/game/showcaseCache';

const mesh = () => ({ positions: new Float32Array([1, 2, 3]), normals: new Float32Array([0, 1, 0]), uvs: new Float32Array([0, 0]), indices: new Uint32Array([0, 1, 2]) });

const draft = { name: 'Testvale', width: 200, depth: 160, slots: [], roads: [], greens: [], households: [], seed: 7 } as unknown as NeighbourhoodDraft;
const preview = { version: 1, lotVersion: 1, mode: 'menu', width: 200, depth: 160, objects: [], sims: [], households: [], plots: [], exits: [], rooms: [], walls: [], wallsLow: [], floors: [], openings: [], fences: [], meta: {}, meshes: { walls: mesh(), wallsLow: mesh(), floors: mesh() } } as unknown as WorldStructure;
const house = { id: 'gen:one', rooms: [] } as unknown as HouseTemplate;

const showcase: Showcase = {
  draft,
  houses: [
    ['gen:one', house],
    ['plot:2', undefined],
  ],
  preview,
};

/** As Cache Storage keeps it: JSON, then back. */
const roundTrip = (entry: unknown) => readEntry(JSON.parse(JSON.stringify(entry)));

describe('the showcase kept between visits', () => {
  test('a returning visit gets the same town, its houses and an empty preview', () => {
    const back = roundTrip(keepEntry(showcase));
    if (!back) throw new Error('not read back');
    expect(back.draft).toEqual(draft);
    expect(back.houses.map(([id]) => id)).toEqual(['gen:one', 'plot:2']);
    expect(back.houses[0][1]).toEqual(house);
    // (A lot the land generated no house for: JSON's null comes back as none.)
    expect(back.houses[1][1]).toBeUndefined();
    const { meshes, ...rest } = back.preview;
    const { meshes: before, ...was } = preview;
    expect(rest).toEqual(was);
    expect(before.walls.positions.length).toBe(3);
    // (The renderer expects the three meshes; a menu preview has none.)
    expect(meshes.walls.positions).toBeInstanceOf(Float32Array);
    expect(meshes.walls.indices).toBeInstanceOf(Uint32Array);
    expect([meshes.walls.positions.length, meshes.wallsLow.positions.length, meshes.floors.positions.length]).toEqual([0, 0, 0]);
  });

  test('an entry this code cannot use is no showcase', () => {
    expect(readEntry(null)).toBeNull();
    expect(readEntry({ version: 0, draft, preview })).toBeNull();
    expect(readEntry({ version: 1, preview })).toBeNull();
    expect(readEntry({ version: 1, draft })).toBeNull();
  });
});
