/**
 * Buy and Build mode input: placing and moving objects on the home lot (Buy), and drawing
 * walls and rooms and putting doors and windows into them (Build). The simulation is the
 * authority (it re-checks every edit and reports problems as errors); this module only shows
 * previews and sends commands.
 *
 * Walls, Room and Remove work by press on a corner → drag → release (with a live preview and its
 * cost); a press and release without moving sets the start corner instead and the next click
 * finishes (click-then-click). Dragging at about 45° draws a diagonal run across tiles; the Room
 * tool draws the four walls of the rectangle between the two corners. Esc or a right-click while
 * drawing cancels. While drawing, left-drag doesn't turn the camera.
 *
 * New walls, doors and windows take the looks picked in Build mode (`game.buildLook`: covering
 * and form, door and window style); drawing a wall in another form over a standing one rebuilds
 * it, and a door or window of another style replaces the one there. The Paint tool covers the
 * wall face on the pointer's side: click, or drag along walls; with Shift, every face of the
 * room (or, outdoors, the outside of the house) at once. The Floor tool covers the floor of the
 * tile under the pointer: click, or drag out a rectangle of tiles; with Shift, the whole room.
 * Remove on doors and windows alone walls
 * them up again (the wall stays); a run that takes in walls tears everything on it down.
 */

import type { ObjectDef } from '../content/content';
import { type EdgeEdit, type FacePaint, type PartColors } from '../core/protocol';
import { type BuildTool } from '../ui/state.svelte';

export type Point = { x: number; z: number };
/** A wall face to paint; `half` for a half wall (the preview's height). */
export type PaintFace = FacePaint & { half: boolean };
/** A wall's look as the structure sends it (doors and windows resized: sill and top, decimetres). */
export type Look = { faces?: [number, number]; form?: number; style?: number; sill?: number; head?: number };

/**
 * Doors or windows side by side in one straight wall, the same style and size: one wide window
 * or a double door (as the renderer draws them). `line`: the wall's z (`h`) or x (`v`); the run
 * covers edges `start .. start + len - 1` along it. Heights in centimetres.
 */
export interface OpeningRun {
  axis: 'h' | 'v';
  line: number;
  start: number;
  len: number;
  kind: 'door' | 'window';
  style: number;
  sill: number;
  head: number;
  /** Whether the player set them (else the style's own). */
  sized: boolean;
}
/** The widest window (edges) and door (a double door). */
export const MAX_WINDOW_RUN = 6;
export const MAX_DOOR_RUN = 2;
/** A door's own top (centimetres; house.ts `DOOR_HEIGHT`). */
export const DOOR_HEAD = 210;

/** What the eyedropper picked up: a build tool with its look set, or an object to buy again. */
export type Picked = { kind: 'tool'; tool: BuildTool } | { kind: 'object'; def: string; colors?: PartColors; rot: number; turn: number; size?: [number, number]; tall?: number };

/**
 * Things stand anywhere on their tiles (all but stairs): their middle snaps to quarter tiles (the
 * build grid's finer lines), or with Shift held to the centimetre.
 */
export const OFFSET_STEP = 0.25;
export const FREE_STEP = 0.01;
/** Things on a top snap finer than that: tops are small. */
export const TOP_STEP = 0.05;

/** The eyedropper takes a wall (rather than the floor) within this distance of it (tiles). */
export const PICK_WALL = 0.4;
export type EdgeState = EdgeEdit['kind'];
export type Axis = EdgeEdit['axis'];

/** A drag counts as diagonal within this angle of 45° (tan 22.5°: halfway to straight). */
export const DIAGONAL_SLOPE = Math.tan(Math.PI / 8);

/** Resize arrows sit this far (m) outside the edge they move, and the height arrow this far above the top. */
export const HANDLE_OUT = 0.3;
/**
 * Whether things can be resized in Buy mode (the arrows). Off for now: a model's moving parts
 * (a fridge's doors, a wall bed) don't follow a resize well. It comes back for the objects it suits.
 * Things resized before keep their size, and "Reset size" still takes them back to their own.
 */
export const RESIZING = false;

export type Point2 = { x: number; y: number };

/** An owned object being resized by one of its arrows (see `press`). */
export interface Resizing {
  id: number;
  def: ObjectDef;
  handle: string;
  /** As it was: its middle (tiles) and size along x and z (m), facing, turn and height (cm). */
  cx: number;
  cz: number;
  ex: number;
  ez: number;
  rot: number;
  turn: number;
  tall: number;
  lift: number;
  box: { height: number; bottom: number };
  /** Its tiles' corner and offset as it was. */
  was: { x: number; z: number; offset?: [number, number] };
  /** Height arrow: where the drag started (m above the floor). */
  from: number | null;
  /** What letting go now would make it: its middle and size, height, tiles (corner, turned size), offset and own size (cm before turning). */
  now: { cx: number; cz: number; ex: number; ez: number; tall: number; x: number; z: number; w: number; d: number; offset?: [number, number]; size?: [number, number]; valid: boolean };
}

/** An object's offset (centimetres off the middle of its footprint) as a key; `0:0` when centred. */
export function offsetKey(offset: [number, number] | undefined): string {
  return `${offset?.[0] ?? 0}:${offset?.[1] ?? 0}`;
}

/**
 * Pairs of neighbouring tiles the body of a thing on tiles `x, z, w, d` (turned) stands across,
 * `offset` off their middle and `extent` (cm, before turning) in size, as sim-core `body_edges`.
 */
export function bodyEdges(x: number, z: number, w: number, d: number, rot: number, offset: readonly [number, number], extent: readonly [number, number]): [[number, number], [number, number]][] {
  const [x0, x1, z0, z1] = bodyBox(x, z, w, d, rot, offset, extent);
  // The tiles it overlaps, and the tile lines strictly inside it.
  const lines = (lo: number, hi: number) => range(Math.floor(lo / 200) + 1, Math.floor((hi - 1) / 200) + 1);
  const edges: [[number, number], [number, number]][] = [];
  for (const lx of lines(x0, x1)) for (const tz of boxTiles(z0, z1)) edges.push([[lx - 1, tz], [lx, tz]]);
  for (const lz of lines(z0, z1)) for (const tx of boxTiles(x0, x1)) edges.push([[tx, lz - 1], [tx, lz]]);
  return edges;
}

export const range = (from: number, to: number) => Array.from({ length: Math.max(0, to - from) }, (_, i) => from + i);
/** The tiles a body's sides `lo`..`hi` (half centimetres) overlap along one axis. */
export const boxTiles = (lo: number, hi: number) => range(Math.floor(lo / 200), Math.floor((hi + 199) / 200));

/** A body's sides `[x0, x1, z0, z1]` in half centimetres (tile lines every 200), as sim-core `body_box`. */
export function bodyBox(x: number, z: number, w: number, d: number, rot: number, offset: readonly [number, number], extent: readonly [number, number]): [number, number, number, number] {
  const [ex, ez] = rot % 2 === 0 ? extent : [extent[1], extent[0]];
  const span = (at: number, tiles: number, off: number, size: number) => {
    const mid = 2 * (at * 100 + off) + tiles * 100;
    return [mid - size, mid + size];
  };
  return [...span(x, w, offset[0], ex), ...span(z, d, offset[1], ez)] as [number, number, number, number];
}

/** Whether a diagonal wall `axis` across tile `tx, tz` runs through a body (`bodyBox`), as sim-core `body_crosses`. */
export function bodyCrosses([x0, x1, z0, z1]: readonly [number, number, number, number], tx: number, tz: number, axis: 'dp' | 'dn'): boolean {
  // The part of the body on that tile; the wall runs through it when it passes strictly inside.
  const [a0, a1] = [Math.max(x0, tx * 200), Math.min(x1, tx * 200 + 200)];
  const [b0, b1] = [Math.max(z0, tz * 200), Math.min(z1, tz * 200 + 200)];
  if (a0 >= a1 || b0 >= b1) return false;
  // `dp` (/): x - z is that of corner (x, z) all along it; `dn` (\): x + z that of (x, z + 1).
  const on = axis === 'dp' ? 200 * (tx - tz) : 200 * (tx + tz + 1);
  const [lo, hi] = axis === 'dp' ? [a0 - b1, a1 - b0] : [a0 + b0, a1 + b1];
  return lo < on && on < hi;
}

