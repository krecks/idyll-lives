/**
 * Living in a flat, as the game's views and Build mode have it (the real town: Mill Court, its flats 1 to 4, the
 * player's household in flat 1): the flats for sale and what they cost (never the building itself), their tags over
 * their own rectangles, and the build tools the simulation refuses in a flat, in its words.
 */
import { describe, expect, test } from 'bun:test';
import { flatRefuses } from '../src/game/flatBuild';
import { FLAT_FLOOR } from '../src/game/price';
import type { BuildTool } from '../src/ui/state.svelte';
import { homesForSale } from '../src/ui/town/places';
import { saleTags } from '../src/ui/town/tags';
import { BUILDING, FLATS, realTown } from './fixtures/realTown';

const RATES = { perTile: 10, perFloorTile: 100, round: 10 };

describe('flats for sale', () => {
  const world = realTown();
  const households = world.households.filter((h) => h.plot !== FLATS[1] && h.plot !== FLATS[3]);

  test('the vacant flats, each its share of the land and its own floor at a flat’s rate; the building is no home itself', () => {
    const sale = homesForSale(world.plots, households, RATES, world.flats);
    expect(sale.map((h) => h.plot)).toEqual([FLATS[1], FLATS[3]]);
    expect(sale.every((h) => h.flat && h.storeys === 1 && h.floor === 63 && h.w === 7 && h.d === 9)).toBe(true);
    expect(sale[0].price).toBe(Math.round((10 * ((20 * 26) / 4) + FLAT_FLOOR * 100 * 63) / 10) * 10);
    // (Without the flats in the views, the building of flats isn't put up for sale whole.)
    expect(homesForSale(world.plots, [], RATES).some((h) => h.plot === BUILDING)).toBe(false);
  });

  test("a flat's tag stands over its own rectangle and says it is a flat", () => {
    const [tag] = saleTags(homesForSale(world.plots, households, RATES, world.flats), world, (n) => `$${n}`);
    expect(tag.anchor).toEqual({ x: 15 + 3.5, z: 8 + 4.5 });
    expect(tag.detail).toBe('flat · 63 m²');
    expect(tag.action).toEqual({ kind: 'sale', plot: FLATS[1] });
  });
});

describe('Build mode in a flat', () => {
  const world = realTown();
  const refused: BuildTool[] = ['roof', 'garage', 'porch', 'blueprint', 'move', 'stairs', 'fence', 'gate'];
  const allowed: BuildTool[] = ['wall', 'room', 'paint', 'floor', 'door', 'window', 'remove'];

  test("the tools the simulation refuses in a flat say why, as it does; inside walls, paint, floors, doors and windows stay", () => {
    for (const tool of refused) expect(flatRefuses(tool, FLATS[0], world)).toBeTruthy();
    for (const tool of allowed) expect(flatRefuses(tool, FLATS[0], world)).toBeNull();
    expect(flatRefuses('roof', FLATS[0], world)).toBe('In a flat you build inside it only: no roof');
    expect(flatRefuses('stairs', FLATS[2], world)).toBe('A flat is all on one storey: no stairs in it');
  });

  test('in a house, or with no home, none is refused', () => {
    for (const tool of [...refused, ...allowed]) {
      expect(flatRefuses(tool, BUILDING, world)).toBeNull();
      expect(flatRefuses(tool, null, world)).toBeNull();
    }
  });
});
