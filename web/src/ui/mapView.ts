/**
 * The town map's view (`TownPanel.svelte`): an SVG whose `viewBox` is the ground in view (m),
 * drawn into its box as SVG does by default (`xMidYMid meet`): scaled to fit, the same scale both
 * ways, and centred, with bands to spare when the box isn't the view's shape. Clicks, the wheel and
 * label sizes go by the scale it's drawn at, not by the box's width.
 */

/** The ground in view (m): the viewBox. */
export interface MapView {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** A box on screen (px), as `getBoundingClientRect` gives it. */
export interface Box {
  left: number;
  top: number;
  width: number;
  height: number;
}

/** Pixels a metre the view is drawn at in a box `width` × `height` px. */
export function mapScale(width: number, height: number, view: MapView): number {
  if (!(view.w > 0) || !(view.d > 0)) return 1;
  const s = Math.min(width / view.w, height / view.d);
  return s > 0 ? s : 1;
}

/** The point of the ground (m) at a point on screen (px). */
export function screenToMap(clientX: number, clientY: number, box: Box, view: MapView): [number, number] {
  const s = mapScale(box.width, box.height, view);
  const left = box.left + (box.width - view.w * s) / 2;
  const top = box.top + (box.height - view.d * s) / 2;
  return [view.x + (clientX - left) / s, view.z + (clientY - top) / s];
}
