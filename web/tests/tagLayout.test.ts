/**
 * The menus' lot tags on screen (`game/tagLayout.ts`): ranked by kind (the selected and hovered lots', then as
 * the screen ranks the kinds: households before places on the new-game screens), each kept near its lot or hidden,
 * and no more than `MOST_HOMES` households' tags at once when the town is zoomed out.
 */
import { describe, expect, test } from 'bun:test';
import { layoutTags, MOST_HOMES, MOST_SHIFT, tagOrder, type Box, type TagSpot } from '../src/game/tagLayout';

/** The new-game screens' ranks (`TownStage` `TAG_ORDER`, less the player's own). */
const TAG_RANK = { home: 1, sale: 2, park: 3 } as const;

const AREA: Box = { l: 0, t: 0, r: 1400, b: 900 };

/** A tag 120 × 50 px (its stem in it) for a lot at (`sx`, `sy`). */
const tag = (sx: number, sy: number, rank: number = TAG_RANK.home, wins: 0 | 1 | null = null): TagSpot => ({ sx, sy, w: 120, h: 50, rank, capped: rank === TAG_RANK.home, wins, x: 0, y: 0, crowded: false });

describe('menu lot tags', () => {
  test('the selected and hovered lots first, then households, then places, each from the bottom of the screen up', () => {
    const order = [tag(100, 300, TAG_RANK.park), tag(100, 200), tag(100, 800, TAG_RANK.park), tag(100, 100, TAG_RANK.park, 1), tag(100, 700), tag(100, 50, TAG_RANK.park, 0)]
      .sort((a, b) => tagOrder(a) - tagOrder(b))
      .map((t) => [t.sy, t.rank, t.wins]);
    expect(order).toEqual([
      [50, TAG_RANK.park, 0],
      [100, TAG_RANK.park, 1],
      [700, TAG_RANK.home, null],
      [200, TAG_RANK.home, null],
      [800, TAG_RANK.park, null],
      [300, TAG_RANK.park, null],
    ]);
  });

  test('a household’s tag takes the room over its lot from a place’s', () => {
    const place = tag(500, 400, TAG_RANK.park);
    const home = tag(510, 405);
    layoutTags([place, home], AREA, []);
    expect(home.crowded).toBe(false);
    expect([home.x, home.y]).toEqual([510, 405]);
    expect(place.crowded).toBe(true);
  });

  test('a tag with no room near its lot waits rather than stacking up away from it', () => {
    // A street of lots 30 px apart along the screen's left: zoomed out.
    const row = Array.from({ length: 8 }, (_, i) => tag(160, 200 + i * 30));
    layoutTags(row, AREA, []);
    const shown = row.filter((t) => !t.crowded);
    expect(shown.length).toBeGreaterThan(0);
    expect(shown.length).toBeLessThan(row.length);
    for (const t of shown) expect(Math.abs(t.x - t.sx) * 1.4 + Math.abs(t.y - t.sy)).toBeLessThanOrEqual(MOST_SHIFT);
  });

  test('no more than MOST_HOMES households’ tags at once, the nearest; the hovered and selected ones besides', () => {
    // Lots well apart, room for all of them: only the cap hides any.
    const homes = Array.from({ length: MOST_HOMES + 6 }, (_, i) => tag(80 + (i % 8) * 160, 150 + Math.floor(i / 8) * 400));
    const hovered = tag(700, 600, TAG_RANK.home, 1);
    const selected = tag(1000, 600, TAG_RANK.home, 0);
    const places = [tag(300, 600, TAG_RANK.park), tag(1300, 850, TAG_RANK.park)];
    layoutTags([...homes, hovered, selected, ...places], AREA, []);
    const shown = homes.filter((t) => !t.crowded);
    expect(shown.length).toBe(MOST_HOMES);
    // The nearest (lowest on screen) win.
    expect(Math.min(...shown.map((t) => t.sy))).toBeGreaterThanOrEqual(Math.max(...homes.filter((t) => t.crowded).map((t) => t.sy)));
    expect(hovered.crowded).toBe(false);
    expect(selected.crowded).toBe(false);
    // Places are not households: the cap leaves them be.
    expect(places.every((t) => !t.crowded)).toBe(true);
  });

  test('tags keep clear of the panels over the town; a winner is pulled inside the stage', () => {
    const panel: Box = { l: 0, t: 700, r: 600, b: 900 };
    const under = tag(200, 800);
    const edge = tag(1395, 30, TAG_RANK.home, 1);
    layoutTags([under, edge], AREA, [panel]);
    expect(under.crowded).toBe(true);
    expect(edge.crowded).toBe(false);
    expect(edge.x + 60).toBeLessThanOrEqual(AREA.r);
    expect(edge.y - 50).toBeGreaterThanOrEqual(AREA.t);
  });
});
