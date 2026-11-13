/**
 * Where the menus' lot tags go on screen (`menuScene.ts`), apart from the page so the rules can be tested: each tag
 * wants to sit right above its lot, inside the town's part of the screen and clear of the panels over it. In order
 * of rank (the selected lot's, the hovered lot's, then by kind as the screen ranks them: `TownStage`'s `order`,
 * households before places on the new-game screens; each kind from the bottom of the screen up, nearer the camera
 * first) each takes the free spot nearest its lot. A tag with no free spot near its lot waits, hidden, rather than covering one that ranks
 * higher or floating off to where it no longer says which lot it is: zoomed out, the tags of a whole town piled up
 * round its middle and down the left of the picture. Its lot is still there to hover and click, and listed.
 *
 * And no more than `MOST_HOMES` households' tags at once (the nearest): a zoomed-out town shows a few, the others
 * come with the pointer on their lot or their row, or nearer.
 *
 * Pure; allocates only the sort and the placed boxes.
 */

/** A rectangle on screen (CSS pixels): left, top, right, bottom. */
export interface Box {
  l: number;
  t: number;
  r: number;
  b: number;
}

/** At most this many households' tags are shown at once, the selected and hovered lots' apart. */
export const MOST_HOMES = 8;
/**
 * How far (CSS pixels, sideways moves counted 1.4 times) a tag may be moved from right above its lot to find room:
 * a nudge, less than a tag's height. Allowed a tag's height, the lots along a street stacked theirs into a column
 * down the left of the picture, each one's stem pointing at a lot it no longer stood over.
 */
export const MOST_SHIFT = 28;

/** One tag to place: in, its lot's point and size; out, where it goes. */
export interface TagSpot {
  /** The lot's point on screen: the stem's foot wants to be there. */
  sx: number;
  sy: number;
  /** The tag's size, its stem in the height. */
  w: number;
  h: number;
  /** Its kind's rank (the lower first). */
  rank: number;
  /** A household's tag: counted against `MOST_HOMES`. */
  capped: boolean;
  /** The selected lot's tag (0) or the hovered lot's (1): it is placed first and shown whatever it covers. */
  wins: 0 | 1 | null;
  /** Out: where its stem's foot goes. */
  x: number;
  y: number;
  /** Out: no room near its lot (or too many households' tags): hidden. */
  crowded: boolean;
}

/** The order tags are placed in: the winners, then by rank, each rank from the bottom of the screen up. */
export function tagOrder(t: Pick<TagSpot, 'sy' | 'rank' | 'wins'>): number {
  return t.wins === 0 ? -2e5 : t.wins === 1 ? -1e5 : t.rank * 1e4 - t.sy;
}

const overlaps = (a: Box, b: Box) => a.l < b.r && a.r > b.l && a.t < b.b && a.b > b.t;

/**
 * Places `tags` inside `area` clear of `blocks` (the panels over the town), setting each one's `x`, `y` and
 * `crowded`. A winner is pulled inside the area and shown wherever it lands (unless the area has no room for it at
 * all); the others only where they fit, near their lot.
 */
export function layoutTags(tags: TagSpot[], area: Box, blocks: readonly Box[]): void {
  const sorted = tags.slice().sort((a, c) => tagOrder(a) - tagOrder(c));
  const done: Box[] = [];
  let homes = 0;
  const clampX = (x: number, w: number) => Math.min(area.r - w / 2, Math.max(area.l + w / 2, x));
  const clampY = (y: number, h: number) => Math.min(area.b, Math.max(area.t + h, y));
  for (const tag of sorted) {
    const { w, h } = tag;
    // The anchor (x, y) is the stem's foot: the box spans [x - w/2, x + w/2] x [y - h, y].
    const wx = clampX(tag.sx, w);
    const wy = clampY(tag.sy, h);
    // Inside the town's part of the screen (a tag taller or wider than that has nowhere to go).
    const inside = (x: number, y: number) => y - h >= area.t - 0.5 && y <= area.b + 0.5 && x - w / 2 >= area.l - 0.5 && x + w / 2 <= area.r + 0.5;
    const free = (x: number, y: number) => {
      const box = { l: x - w / 2, t: y - h, r: x + w / 2, b: y };
      return inside(x, y) && !blocks.some((o) => overlaps(box, o)) && !done.some((o) => overlaps(box, o));
    };
    // Vertical moves read better than sideways ones (the tag stays over its column).
    const shift = (x: number, y: number) => Math.abs(x - tag.sx) * 1.4 + Math.abs(y - tag.sy);
    // Candidates: the wanted spot, and spots just beside every tag placed and every obstacle (above, below, left,
    // right and the diagonals); the free one nearest the lot wins.
    let x = wx;
    let y = wy;
    if (!free(wx, wy)) {
      let best = Infinity;
      const tryAt = (cx0: number, cy0: number) => {
        const cx = clampX(cx0, w);
        const cy = clampY(cy0, h);
        const cost = shift(cx, cy);
        if (cost < best && free(cx, cy)) {
          best = cost;
          x = cx;
          y = cy;
        }
      };
      const around = [...done, ...blocks];
      for (const o of around) {
        for (const cx of [wx, o.l - w / 2 - 3, o.r + w / 2 + 3]) for (const cy of [wy, o.t - 3, o.b + h + 3]) tryAt(cx, cy);
      }
      // Crowded corner: combine edges of different boxes (rare, a little more work).
      if (best === Infinity) {
        const xs = [wx, ...around.flatMap((o) => [o.l - w / 2 - 3, o.r + w / 2 + 3])];
        const ys = [wy, ...around.flatMap((o) => [o.t - 3, o.b + h + 3])];
        for (const cx of xs) for (const cy of ys) tryAt(cx, cy);
      }
    }
    tag.x = x;
    tag.y = y;
    tag.crowded = tag.wins !== null ? !inside(x, y) : !free(x, y) || shift(x, y) > MOST_SHIFT || (tag.capped && homes >= MOST_HOMES);
    if (tag.crowded) continue;
    if (tag.wins === null && tag.capped) homes++;
    done.push({ l: x - w / 2, t: y - h, r: x + w / 2, b: y });
  }
}
