/**
 * Where the lot tags go on screen, the menus' (`menuScene.ts`) and the town view's in the game (`ui/town/TownTags.svelte`),
 * apart from the page so the rules can be tested: each tag wants to sit right above its lot (or resident), inside
 * the town's part of the screen and clear of the panels over it. In order of rank (the winners first: the selected
 * lot's, the hovered lot's, in the game the household's own; then by kind as the screen ranks them: `TownStage`'s
 * `order`, households before places on the new-game screens; each kind from the bottom of the screen up, nearer the
 * camera first, a tag shown last frame before the others of its kind) each takes the free spot nearest its lot. A
 * tag with no free spot near its lot waits, hidden, rather than covering one that ranks higher or floating off to
 * where it no longer says which lot it is: zoomed out, the tags of a whole town piled up round its middle and down
 * the left of the picture. Its lot is still there to hover and click, and listed.
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

/** At most this many households' tags are shown at once, the winners' apart. */
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
  /**
   * A winner: placed before every other tag, the lower first (the selected lot's 0, the hovered lot's 1 on the menus),
   * and shown however far it has to move and whatever it covers; null for the rest.
   */
  wins: number | null;
  /** Shown last frame: placed before the others of its rank, so two tags of a kind don't trade places as the camera moves. */
  held?: boolean;
  /** Out: where its stem's foot goes. */
  x: number;
  y: number;
  /** Out: no room near its lot (or too many households' tags): hidden. */
  crowded: boolean;
}

/** The order tags are placed in: the winners, then by rank, each rank the tags held from last frame first, then from the bottom of the screen up. */
export function tagOrder(t: Pick<TagSpot, 'sy' | 'rank' | 'wins' | 'held'>): number {
  return t.wins !== null ? (t.wins - 100) * 1e5 - t.sy : t.rank * 1e4 - t.sy - (t.held ? 5e3 : 0);
}

const overlaps = (a: Box, b: Box) => a.l < b.r && a.r > b.l && a.t < b.b && a.b > b.t;

/**
 * How far round its point (CSS pixels) a tag that doesn't win can stand: `MOST_SHIFT` sideways (counted 1.4 times)
 * and up or down, and a pixel more for rounding. Only what lies there can be in its way.
 */
const REACH_X = MOST_SHIFT / 1.4 + 1;
const REACH_Y = MOST_SHIFT + 1;

/**
 * Places `tags` inside `area` clear of `blocks` (the panels over the town), setting each one's `x`, `y` and
 * `crowded`. A winner is pulled inside the area and shown wherever it lands (unless the area has no room for it at
 * all); the others only where they fit, near their lot.
 *
 * (Run every frame the town view moves: a tag that doesn't win is checked only against the boxes within its reach,
 * and of the spots tried for it only those it may move to, which places every tag where trying them all would.)
 */
export function layoutTags(tags: TagSpot[], area: Box, blocks: readonly Box[]): void {
  const sorted = tags.slice().sort((a, c) => tagOrder(a) - tagOrder(c));
  const done: Box[] = [];
  /** What may be in the way of the tag being placed: everything for a winner, else what lies within its reach. */
  const near: Box[] = [];
  let homes = 0;
  const clampX = (x: number, w: number) => Math.min(area.r - w / 2, Math.max(area.l + w / 2, x));
  const clampY = (y: number, h: number) => Math.min(area.b, Math.max(area.t + h, y));
  for (const tag of sorted) {
    const { w, h, sx, sy } = tag;
    const wins = tag.wins !== null;
    // (The households' few are shown already: no need to look for room.)
    if (!wins && tag.capped && homes >= MOST_HOMES) {
      tag.x = sx;
      tag.y = sy;
      tag.crowded = true;
      continue;
    }
    near.length = 0;
    if (wins) {
      for (const o of done) near.push(o);
      for (const o of blocks) near.push(o);
    } else {
      const reach: Box = { l: sx - REACH_X - w / 2, t: sy - REACH_Y - h, r: sx + REACH_X + w / 2, b: sy + REACH_Y };
      for (const o of done) if (overlaps(o, reach)) near.push(o);
      for (const o of blocks) if (overlaps(o, reach)) near.push(o);
    }
    // The anchor (x, y) is the stem's foot: the box spans [x - w/2, x + w/2] x [y - h, y].
    const wx = clampX(sx, w);
    const wy = clampY(sy, h);
    // Inside the town's part of the screen (a tag taller or wider than that has nowhere to go).
    const inside = (x: number, y: number) => y - h >= area.t - 0.5 && y <= area.b + 0.5 && x - w / 2 >= area.l - 0.5 && x + w / 2 <= area.r + 0.5;
    const free = (x: number, y: number) => {
      if (!inside(x, y)) return false;
      const l = x - w / 2;
      const r = x + w / 2;
      const t = y - h;
      for (const o of near) if (l < o.r && r > o.l && t < o.b && y > o.t) return false;
      return true;
    };
    // Vertical moves read better than sideways ones (the tag stays over its column).
    const shift = (x: number, y: number) => Math.abs(x - sx) * 1.4 + Math.abs(y - sy);
    // Candidates: the wanted spot, and spots just beside every tag placed and every obstacle (above, below, left,
    // right and the diagonals); the free one nearest the lot wins.
    let x = wx;
    let y = wy;
    if (!free(wx, wy)) {
      let best = Infinity;
      /** Tries the spot (`cx`, `cy`), clamped: taken if free and nearer the lot than the best so far. */
      const tryAt = (cx0: number, cy0: number) => {
        const cx = clampX(cx0, w);
        const cy = clampY(cy0, h);
        const cost = shift(cx, cy);
        // (Only a winner may go further than `MOST_SHIFT`: the rest don't look there.)
        if (cost < best && (wins || cost <= MOST_SHIFT) && free(cx, cy)) {
          best = cost;
          x = cx;
          y = cy;
        }
      };
      const around = done.length ? [...done, ...blocks] : blocks;
      for (const o of around) {
        for (let i = 0; i < 3; i++) {
          const cx = i === 0 ? wx : i === 1 ? o.l - w / 2 - 3 : o.r + w / 2 + 3;
          // (A spot that far sideways costs more than the best, or than a tag that doesn't win may move, whatever its height.)
          const across = Math.abs(clampX(cx, w) - sx) * 1.4;
          if (across >= best || (!wins && across > MOST_SHIFT)) continue;
          tryAt(cx, wy);
          tryAt(cx, o.t - 3);
          tryAt(cx, o.b + h + 3);
        }
      }
      // Crowded corner: combine edges of different boxes (rare, a little more work). For a tag that doesn't win,
      // only the edges within `MOST_SHIFT` of its point (a spot further either way costs more than that).
      if (best === Infinity) {
        const xs = [wx];
        const ys = [wy];
        for (const o of around) xs.push(o.l - w / 2 - 3, o.r + w / 2 + 3);
        for (const o of around) ys.push(o.t - 3, o.b + h + 3);
        const xsIn = wins ? xs : xs.filter((cx) => Math.abs(clampX(cx, w) - sx) * 1.4 <= MOST_SHIFT);
        const ysIn = wins ? ys : ys.filter((cy) => Math.abs(clampY(cy, h) - sy) <= MOST_SHIFT);
        for (const cx of xsIn) for (const cy of ysIn) tryAt(cx, cy);
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
