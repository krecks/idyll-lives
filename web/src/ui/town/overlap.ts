/**
 * The town tags kept from overlapping (docs/design/living-town.md, Town tags): when two meet on screen, the more
 * important one stays. Household > your people > places > homes for sale > street and area names; within a kind,
 * the one shown last frame stays (no flicker as the camera moves), then the nearer, then the first by key.
 */

/** What a tag is for, also its filter (the tag menu). */
export type TagKind = 'household' | 'people' | 'places' | 'sale' | 'names';

/** Most important first. */
export const TAG_ORDER: readonly TagKind[] = ['household', 'people', 'places', 'sale', 'names'];

/** A tag on screen: its box (CSS px, `x`, `y` its top left) and how far its point is from the camera (m). */
export interface TagBox {
  key: string;
  kind: TagKind;
  x: number;
  y: number;
  w: number;
  h: number;
  depth: number;
  /** Always kept (the tag the player clicked): it takes its place before any other. */
  pinned?: boolean;
  /**
   * Times it may step up out of the way (each step its height and the gap) when its place is taken, rather than go:
   * the household's tags over one another when two of them stand together.
   */
  climb?: number;
}

export interface KeepApartOptions {
  /** The screen (CSS px): a tag wholly off it isn't kept. */
  width: number;
  height: number;
  /** Space kept between two tags (px). */
  gap?: number;
  /** The tags shown last frame: within a kind they go first. */
  shown?: ReadonlySet<string>;
  /** Boxes no tag may cover (the HUD's panels), in the same pixels. */
  avoid?: readonly { x: number; y: number; w: number; h: number }[];
}

const CELL = 96;

/**
 * The tags to show, by key, each with how far it stepped up (px, 0 or less): each tag, most important first, unless
 * it would cover one already kept or a box to avoid (and can't step up clear of them), or lies wholly off the
 * screen. Greedy over a grid of 96 px cells, so a few hundred tags cost little.
 */
export function keepApart(tags: readonly TagBox[], { width, height, gap = 3, shown, avoid = [] }: KeepApartOptions): Map<string, number> {
  const rank = (t: TagBox) => (t.pinned ? -1 : TAG_ORDER.indexOf(t.kind));
  const order = tags
    .filter((t) => t.x < width && t.y < height && t.x + t.w > 0 && t.y + t.h > 0)
    .sort((a, b) => rank(a) - rank(b) || (shown ? Number(shown.has(b.key)) - Number(shown.has(a.key)) : 0) || a.depth - b.depth || (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
  type Box = Pick<TagBox, 'x' | 'y' | 'w' | 'h'>;
  const kept = new Map<string, number>();
  const grid = new Map<number, Box[]>();
  const cells = (t: Box, visit: (k: number) => boolean | void) => {
    const x0 = Math.floor((t.x - gap) / CELL);
    const x1 = Math.floor((t.x + t.w + gap) / CELL);
    const y0 = Math.floor((t.y - gap) / CELL);
    const y1 = Math.floor((t.y + t.h + gap) / CELL);
    for (let cy = y0; cy <= y1; cy++) for (let cx = x0; cx <= x1; cx++) if (visit(cy * 65536 + cx) === true) return true;
    return false;
  };
  const meets = (a: Box, b: Box) => a.x < b.x + b.w + gap && b.x < a.x + a.w + gap && a.y < b.y + b.h + gap && b.y < a.y + a.h + gap;
  const blocked = (t: Box) => cells(t, (k) => grid.get(k)?.some((o) => meets(t, o)));
  const take = (t: Box) =>
    cells(t, (k) => {
      const list = grid.get(k);
      if (list) list.push(t);
      else grid.set(k, [t]);
    });
  /** How far up `t` stands clear (0: in its own place; up to `climb` steps), or null: nowhere. */
  const room = (t: TagBox): number | null => {
    for (let step = 0; step <= (t.climb ?? 0); step++) {
      const dy = step ? -step * (t.h + gap) : 0;
      if (!blocked({ x: t.x, y: t.y + dy, w: t.w, h: t.h })) return dy;
    }
    return null;
  };
  for (const a of avoid) take(a);
  for (const t of order) {
    const dy = room(t);
    if (dy === null) continue;
    kept.set(t.key, dy);
    take({ x: t.x, y: t.y + dy, w: t.w, h: t.h });
  }
  return kept;
}
