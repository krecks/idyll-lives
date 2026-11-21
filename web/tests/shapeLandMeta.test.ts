// A `shapeLand` stroke merged into the page's copy of `meta` (`mergeShapeLand`) as the simulation merges it into its
// own (sim-core `land.rs`, tests/sim/shape_land.rs): the land's tiles by index over the old ones, the rest of the land
// and the zones replaced, a new `meta` with the old one left as it was; with the land's decoder reading the merged tiles.
// `bun test tests/shapeLandMeta.test.ts`.
import { describe, expect, test } from 'bun:test';
import { mergeShapeLand, type WorldMeta } from '../src/core/protocol';
import { blankLand, decodeLand, encodeTiles } from '../src/game/land';

describe('mergeShapeLand', () => {
  test('the first stroke makes the land; a later one merges its tiles and replaces the zones', () => {
    const meta: WorldMeta = { name: 'Elmbury' };
    const header = { v: 1 as const, cell: 2 as const, cols: 200, rows: 100, tile: 64 as const, cover: [5000, 0], seeded: 'drawn' as const };
    const first = mergeShapeLand(meta, { land: { ...header, h: { 0: 'AAA', 5: 'BBB' }, wl: { 5: 'WWW' } }, zones: [] })!;
    expect(first.land!.h).toEqual(['AAA', '', '', '', '', 'BBB', '', '']);
    expect(first.land!.wl).toEqual(['', '', '', '', '', 'WWW', '', '']);
    expect(first.name).toBe('Elmbury');
    expect(meta.land).toBeUndefined();
    const second = mergeShapeLand(first, { land: { h: { 5: 'CCC', 7: 'DDD' }, wl: { 5: '' }, cover: [5000, 1] } })!;
    expect(second.land!.h).toEqual(['AAA', '', '', '', '', 'CCC', '', 'DDD']);
    expect(second.land!.wl.every((c) => c === '')).toBe(true);
    expect(second.land!.cover).toEqual([5000, 1]);
    expect(second.zones).toEqual([]);
    expect(first.land!.h[5]).toBe('BBB');
  });

  test("a river's way merges by tile once there is one; a land without leaves none", () => {
    const header = { v: 1 as const, cell: 2 as const, cols: 200, rows: 100, tile: 64 as const, cover: [5000, 0], seeded: 'drawn' as const };
    const first = mergeShapeLand(null, { land: { ...header, h: { 0: 'AAA' }, wl: {} } })!;
    expect(first.land!.flow).toBeUndefined();
    const ran = mergeShapeLand(first, { land: { wl: { 2: 'WWW' }, flow: { 2: 'FFF' } } })!;
    expect(ran.land!.flow).toEqual(['', '', 'FFF', '', '', '', '', '']);
    const kept = mergeShapeLand(ran, { land: { h: { 3: 'GGG' } } })!;
    expect(kept.land!.flow).toEqual(['', '', 'FFF', '', '', '', '', '']);
    const moved = mergeShapeLand(kept, { land: { flow: { 2: '', 6: 'HHH' } } })!;
    expect(moved.land!.flow).toEqual(['', '', '', '', '', '', 'HHH', '']);
    expect(ran.land!.flow![2]).toBe('FFF');
  });

  test('tiles coded by encodeTiles decode from the merged land', () => {
    const land = blankLand(256, 256);
    land.h[land.cols * 70 + 40] = 1234;
    const header = { v: 1 as const, cell: 2 as const, cols: land.cols, rows: land.rows, tile: 64 as const, cover: [land.cover.length, 0], seeded: land.seeded };
    const merged = mergeShapeLand(null, { land: { ...header, ...encodeTiles(land, [2]) } })!;
    expect(decodeLand(merged.land!).h[land.cols * 70 + 40]).toBe(1234);
  });

  test("a stroke's bridges replace its roads' lists (as the simulation merges them), the other roads the same objects", () => {
    const street = { points: [[0, 0], [100, 0]] as [number, number][], width: 7, ends: ['join', 'join'] as const, kind: 'street' as const };
    const other = { ...street, points: [[0, 50], [100, 50]] as [number, number][], bridge: [10, 20] as [number, number] };
    const meta: WorldMeta = { name: 'Elmbury', roads: [{ ...street, ends: ['join', 'join'] }, { ...other, ends: ['join', 'join'] }] };
    const bridged = mergeShapeLand(meta, { roads: { 0: { bridges: [[40, 60]] } } })!;
    expect(bridged.roads![0].bridges).toEqual([[40, 60]]);
    expect(bridged.roads![1]).toBe(meta.roads![1]);
    expect(meta.roads![0].bridges).toBeUndefined();
    const gone = mergeShapeLand(bridged, { roads: { 0: { bridges: [] } } })!;
    expect('bridges' in gone.roads![0]).toBe(false);
  });

  test('nothing to merge leaves meta as it is', () => {
    const meta: WorldMeta = { name: 'Elmbury' };
    expect(mergeShapeLand(meta, {})).toBe(meta);
  });
});
