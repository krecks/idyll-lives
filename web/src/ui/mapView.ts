/**
 * The town map's view (`TownPanel.svelte`): an SVG whose `viewBox` is the ground in view (m),
 * drawn into its box as SVG does by default (`xMidYMid meet`): scaled to fit, the same scale both
 * ways, and centred, with bands to spare when the box isn't the view's shape. Clicks, the wheel and
 * label sizes go by the scale it's drawn at, not by the box's width.
 *
 * North up: the map shows the ground as the 3D view does from above (game space is x east, z north,
 * and `render/pc/space.ts` mirrors the world in z for the engine, so there x is to the right and z
 * up the screen). SVG's y runs down the page, so the map's y is −z: the ground is drawn in game
 * coordinates under `GROUND` (a flip in y), and what must read upright (the names) is placed in the
 * SVG's own coordinates (`mapY`) under `UPRIGHT`.
 */

import { layoutTags, type Box as TagBox, type TagSpot } from '../game/tagLayout';

/** The ground in view (m): the viewBox. */
interface MapView {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** A box on screen (px), as `getBoundingClientRect` gives it. */
interface Box {
  left: number;
  top: number;
  width: number;
  height: number;
}

/** The SVG's y of a ground point's `z` (m): north up the page. */
export const mapY = (z: number): number => -z;

/** The transform that draws game coordinates (x, z) north up: on the ground's layer. */
export const GROUND = 'scale(1 -1)';

/** The transform that sets a name upright again inside `GROUND`: its x and y are then the SVG's (`mapY`). */
export const UPRIGHT = 'scale(1 -1)';

/** The SVG's `viewBox` for the ground in view: x along, −z down (the view's north edge at its top). */
export function viewBox(view: MapView): string {
  return `${view.x} ${mapY(view.z + view.d)} ${view.w} ${view.d}`;
}

/** Pixels a metre the view is drawn at in a box `width` × `height` px. */
export function mapScale(width: number, height: number, view: MapView): number {
  if (!(view.w > 0) || !(view.d > 0)) return 1;
  const s = Math.min(width / view.w, height / view.d);
  return s > 0 ? s : 1;
}

/** Where the view is drawn in a box `width` × `height` px: its scale and its top left corner in the box (px). */
function fit(width: number, height: number, view: MapView): { s: number; left: number; top: number } {
  const s = mapScale(width, height, view);
  return { s, left: (width - view.w * s) / 2, top: (height - view.d * s) / 2 };
}

/** The point of the ground (m) at a point on screen (px): its top the view's north edge. */
export function screenToMap(clientX: number, clientY: number, box: Box, view: MapView): [number, number] {
  const { s, left, top } = fit(box.width, box.height, view);
  return [view.x + (clientX - box.left - left) / s, view.z + view.d - (clientY - box.top - top) / s];
}

/** The point on screen (px) of a point of the ground (m): `screenToMap` the other way. */
export function mapToScreen(x: number, z: number, box: Box, view: MapView): [number, number] {
  const { s, left, top } = fit(box.width, box.height, view);
  return [box.left + left + (x - view.x) * s, box.top + top + (view.z + view.d - z) * s];
}

/**
 * A name on the map: wanted centred on its point (m), `size` px high (and a smaller line under it,
 * `sub`: a lot's "2 here"). The more important first (`rank`, the lower first); a winner (`wins`, the
 * lower first: the household's own lot) is shown wherever it lands.
 */
export interface MapLabel<K> {
  key: K;
  x: number;
  z: number;
  text: string;
  size: number;
  /** Room for a second, smaller line under it (px high), shown or not. */
  sub?: number;
  rank: number;
  wins?: number | null;
}

/** Where a name goes, in the SVG's coordinates (m: x, and y as `mapY` has it): its baseline's middle, and its second line's. */
interface PlacedLabel {
  x: number;
  y: number;
  subY: number;
}

/** A name's width on screen (px), from its length (the map's labels are Inter, semibold or bolder), its halo in it. */
export function labelWidth(text: string, size: number): number {
  return size * 0.56 * text.length + 4;
}

/**
 * The names on the map laid out as the town view's tags are (`game/tagLayout.ts`): each takes the
 * free spot nearest its point inside the map's box, clear of `blocks` (px in the box: the zoom
 * buttons) and of the names placed before it; one with no room near its point isn't drawn (its lot
 * is still there to click; zoomed in, it has room). Names whose point is out of view aren't drawn;
 * one whose point is by the edge is drawn in from it.
 * Laid out in the box's pixels (`width` × `height`, the map drawn in it as `mapScale` says), so the
 * names stay apart at any zoom.
 */
export function layoutLabels<K>(labels: readonly MapLabel<K>[], width: number, height: number, view: MapView, blocks: readonly TagBox[] = []): Map<K, PlacedLabel> {
  const box = { left: 0, top: 0, width, height };
  const { s, left, top } = fit(width, height, view);
  const spots: (TagSpot & { label: MapLabel<K> })[] = [];
  for (const label of labels) {
    const [px, py] = mapToScreen(label.x, label.z, box, view);
    if (px < 0 || px > width || py < 0 || py > height) continue;
    // The box: the name's line (and the second's), centred on the point; its anchor is its foot.
    const h = label.size * 1.25 + (label.sub ? label.sub * 1.25 : 0);
    const w = labelWidth(label.text, label.size);
    // A place by the edge wants its name in from the edge as far as it needs, not moved there (which would count
    // against how far a name may move, and hide it).
    const sx = Math.min(width - w / 2, Math.max(w / 2, px));
    const sy = Math.min(height, Math.max(h, py + h / 2));
    spots.push({ label, sx, sy, w, h, rank: label.rank, capped: false, wins: label.wins ?? null, x: 0, y: 0, crowded: false });
  }
  layoutTags(spots, { l: 0, t: 0, r: width, b: height }, blocks);
  const out = new Map<K, PlacedLabel>();
  for (const t of spots) {
    if (t.crowded) continue;
    const { size, sub } = t.label;
    const boxTop = t.y - t.h;
    // (Inter's ascent is about 0.97 of its size: the baseline sits that far below the line's top.)
    const baseline = boxTop + size * 0.97;
    const subBaseline = boxTop + size * 1.25 + (sub ?? 0) * 0.97;
    const north = mapY(view.z + view.d);
    out.set(t.label.key, { x: view.x + (t.x - left) / s, y: north + (baseline - top) / s, subY: north + (subBaseline - top) / s });
  }
  return out;
}
