/**
 * The menus' lot tags on screen (`game/tagLayout.ts`): ranked by kind (the selected and hovered lots', then as
 * the screen ranks the kinds: households before places on the new-game screens), each kept near its lot or hidden,
 * and no more than `MOST_HOMES` households' tags at once when the town is zoomed out.
 */
import { describe, expect, test } from 'bun:test';
import { mulberry32 } from '../src/game/noise';
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

/**
 * The layout as it was before it looked only within each tag's reach (every spot tried against every box): the
 * town view's tags lay out every frame, and the faster one has to place every tag where this one does.
 */
function layoutEveryBox(tags: TagSpot[], area: Box, blocks: readonly Box[]): void {
  const overlaps = (a: Box, b: Box) => a.l < b.r && a.r > b.l && a.t < b.b && a.b > b.t;
  const sorted = tags.slice().sort((a, c) => tagOrder(a) - tagOrder(c));
  const done: Box[] = [];
  let homes = 0;
  const clampX = (x: number, w: number) => Math.min(area.r - w / 2, Math.max(area.l + w / 2, x));
  const clampY = (y: number, h: number) => Math.min(area.b, Math.max(area.t + h, y));
  for (const tag of sorted) {
    const { w, h } = tag;
    const wins = tag.wins !== null;
    if (!wins && tag.capped && homes >= MOST_HOMES) {
      tag.x = tag.sx;
      tag.y = tag.sy;
      tag.crowded = true;
      continue;
    }
    const wx = clampX(tag.sx, w);
    const wy = clampY(tag.sy, h);
    const inside = (x: number, y: number) => y - h >= area.t - 0.5 && y <= area.b + 0.5 && x - w / 2 >= area.l - 0.5 && x + w / 2 <= area.r + 0.5;
    const free = (x: number, y: number) => {
      const box = { l: x - w / 2, t: y - h, r: x + w / 2, b: y };
      return inside(x, y) && !blocks.some((o) => overlaps(box, o)) && !done.some((o) => overlaps(box, o));
    };
    const shift = (x: number, y: number) => Math.abs(x - tag.sx) * 1.4 + Math.abs(y - tag.sy);
    let x = wx;
    let y = wy;
    if (!free(wx, wy)) {
      let best = Infinity;
      const tryAt = (cx0: number, cy0: number) => {
        const cx = clampX(cx0, w);
        const cy = clampY(cy0, h);
        const cost = shift(cx, cy);
        if (cost < best && (wins || cost <= MOST_SHIFT) && free(cx, cy)) {
          best = cost;
          x = cx;
          y = cy;
        }
      };
      const around = [...done, ...blocks];
      for (const o of around) {
        for (const cx of [wx, o.l - w / 2 - 3, o.r + w / 2 + 3]) for (const cy of [wy, o.t - 3, o.b + h + 3]) tryAt(cx, cy);
      }
      if (best === Infinity) {
        const xs = [wx, ...around.flatMap((o) => [o.l - w / 2 - 3, o.r + w / 2 + 3])];
        const ys = [wy, ...around.flatMap((o) => [o.t - 3, o.b + h + 3])];
        for (const cx of xs) for (const cy of ys) tryAt(cx, cy);
      }
    }
    tag.x = x;
    tag.y = y;
    tag.crowded = wins ? !inside(x, y) : !free(x, y) || shift(x, y) > MOST_SHIFT;
    if (tag.crowded) continue;
    if (!wins && tag.capped) homes++;
    done.push({ l: x - w / 2, t: y - h, r: x + w / 2, b: y });
  }
}

describe('the town view’s tags, laid out every frame', () => {
  test('each tag goes where trying every spot against every box would put it', () => {
    const rand = mulberry32(11);
    const at = (lo: number, hi: number) => lo + rand() * (hi - lo);
    let crowded = 0;
    let moved = 0;
    for (let round = 0; round < 120; round++) {
      // A town view's worth: a few winners, households, places, names; bunched round a few points (a town's
      // middle), some by the screen's edges and under the panels.
      const area: Box = { l: 8, t: 8, r: at(600, 1912), b: at(400, 1072) };
      const blocks: Box[] = Array.from({ length: Math.floor(at(0, 5)) }, () => {
        const l = at(0, area.r);
        const t = at(0, area.b);
        return { l, t, r: l + at(40, 400), b: t + at(30, 200) };
      });
      const centres = Array.from({ length: 1 + Math.floor(at(0, 4)) }, () => [at(-50, area.r + 50), at(-50, area.b + 50)]);
      const n = Math.floor(at(1, 120));
      const make = (): TagSpot[] =>
        Array.from({ length: n }, (_, i) => {
          const [cx, cy] = centres[i % centres.length];
          const spread = at(10, 400);
          const roll = rand();
          const rank = Math.floor(at(0, 6));
          return {
            sx: Math.round(cx + at(-spread, spread)),
            sy: Math.round(cy + at(-spread, spread)),
            w: Math.round(at(30, 260)),
            h: Math.round(at(14, 40)),
            rank,
            capped: rank === 1,
            wins: roll < 0.04 ? 0 : roll < 0.12 ? 1 : roll < 0.15 ? 2 : null,
            held: rand() < 0.5,
            x: 0,
            y: 0,
            crowded: false,
          };
        });
      const tags = make();
      const again = tags.map((t) => ({ ...t }));
      layoutTags(tags, area, blocks);
      layoutEveryBox(again, area, blocks);
      for (let i = 0; i < n; i++) {
        expect([tags[i].x, tags[i].y, tags[i].crowded]).toEqual([again[i].x, again[i].y, again[i].crowded]);
        if (tags[i].crowded) crowded++;
        else if (tags[i].x !== tags[i].sx || tags[i].y !== tags[i].sy) moved++;
      }
    }
    // (The rounds crowd: many tags hidden, many moved to find room.)
    expect(crowded).toBeGreaterThan(400);
    expect(moved).toBeGreaterThan(400);
  });
});
