/**
 * Tops that things stand on, and seats at tables: the rules of sim-core `surface.rs`, for
 * previews in Buy mode and for laying out houses. The simulation re-checks everything.
 *
 * A thing on the `surface` layer stands at its tile's middle moved by its offset; the disc it
 * takes (its `radius`) lies wholly on one top (an object's `surface`: a rectangle of its `size`
 * at its `at`, turned and moved with the object), inside the margin along the edges, overlapping
 * nothing else standing there. Its host follows from where it stands. A seat with a `seatAt`
 * facing a top of one of those kinds is a seat at that table; seats used from `around` are used
 * from any free tile beside them.
 */

import type { ObjectDef } from '../content/content';

/** What these rules read of an object type. */
export type Shape = Pick<ObjectDef, 'footprint' | 'layer' | 'surface' | 'radius' | 'approach' | 'seatAt'> & Partial<Pick<ObjectDef, 'interactions'>>;

/** Object types by id, and the disc a thing on a top takes (the game's `Content`, or a house generator's view of it). */
export interface Shapes {
  object(id: string): Shape | undefined;
  radius(def: Shape | undefined): number;
}

/** Something placed: as the structure lists it (`ObjectPlacement`) or as a house layout does. */
export interface Placed {
  id?: number;
  def: string;
  x: number;
  z: number;
  rot: number;
  turn?: number;
  offset?: [number, number];
  /** Resized: its footprint turned (tiles along x and z, as the structure lists it); absent: its kind's. */
  w?: number;
  d?: number;
  /** Resized: its own size (centimetres across and deep before turning); absent: its footprint's. */
  size?: [number, number];
  /** Centimetres taller (negative: shorter) than its kind. */
  tall?: number;
}

/** Default margin along a top's edges (m). */
const DEFAULT_INSET = 0.04;
/** Slack when checking a disc stays on a top: offsets are whole centimetres. */
const SLACK = 0.006;

/** Surface kinds as bits (sim-core `SurfaceKind::bit`). */
const KINDS = ['table', 'bar', 'counter', 'desk', 'low', 'shelf'] as const;

/** A top in the world: its middle, half its size, which way it's turned, and what it is. */
export interface Top {
  host: Placed;
  cx: number;
  cz: number;
  hw: number;
  hd: number;
  cos: number;
  sin: number;
  height: number;
  kind: (typeof KINDS)[number];
  inset: number;
}

/** Rotated footprint (tiles) of a thing of type `def` (`size`: its own, before turning). */
export function sizeOf(def: Shape | undefined, rot: number, size?: readonly [number, number]): [number, number] {
  const [w, d] = size ?? def?.footprint ?? [1, 1];
  return rot % 2 === 0 ? [w, d] : [d, w];
}

/** Rotated footprint (tiles) of a placed thing, resized or not. */
export function placedSize(content: Shapes, o: Placed): [number, number] {
  return o.w && o.d ? [o.w, o.d] : sizeOf(content.object(o.def), o.rot);
}

/** A placed thing's own footprint before turning (tiles across and deep). */
export function footprintOf(content: Shapes, o: Placed): [number, number] {
  const [w, d] = placedSize(content, o);
  return o.rot % 2 === 0 ? [w, d] : [d, w];
}

/** A placed thing's own size before turning (centimetres across and deep). */
export function ownSize(content: Shapes, o: Placed): [number, number] {
  if (o.size) return [o.size[0], o.size[1]];
  const [w, d] = footprintOf(content, o);
  return [w * 100, d * 100];
}

/** Where a placed thing's middle is, its offset included (tiles). */
export function middleOf(content: Shapes, o: Placed): [number, number] {
  const [w, d] = placedSize(content, o);
  return [o.x + w / 2 + (o.offset?.[0] ?? 0) / 100, o.z + d / 2 + (o.offset?.[1] ?? 0) / 100];
}

/** The top of a placed thing with a `surface`. */
export function topOf(content: Shapes, o: Placed): Top | null {
  const def = content.object(o.def);
  const s = def?.surface;
  if (!s) return null;
  const a = (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180;
  const [cos, sin] = [Math.cos(a), Math.sin(a)];
  const [mx, mz] = middleOf(content, o);
  // Resized, the top grows with the footprint (its margins stay) and rises with the height (sim-core `Top::of`).
  const kind = def.footprint ?? [1, 1];
  const own = ownSize(content, o).map((v) => v / 100);
  const [w, d] = (s.size ?? kind).map((v, i) => Math.max(0.1, v + own[i] - kind[i]));
  const [ax, az] = (s.at ?? [0, 0]).map((v, i) => (v * own[i]) / kind[i]);
  const height = Math.max(0.05, s.height + (o.tall ?? 0) / 100);
  return { host: o, cx: mx + ax * cos + az * sin, cz: mz - ax * sin + az * cos, hw: w / 2, hd: d / 2, cos, sin, height, kind: s.kind, inset: s.inset ?? DEFAULT_INSET };
}

/** A point in a top's own frame (x to its right, z to its front). */
export function localOf(top: Top, x: number, z: number): [number, number] {
  const [dx, dz] = [x - top.cx, z - top.cz];
  return [dx * top.cos - dz * top.sin, dx * top.sin + dz * top.cos];
}

/** Whether a point is over a top at all. */
export function covers(top: Top, x: number, z: number): boolean {
  const [lx, lz] = localOf(top, x, z);
  return Math.abs(lx) < top.hw && Math.abs(lz) < top.hd;
}

/** Whether a disc of `radius` at a point lies on a top, inside the margin along its edges. */
export function holds(top: Top, x: number, z: number, radius: number): boolean {
  const [lx, lz] = localOf(top, x, z);
  const m = top.inset + radius;
  return Math.abs(lx) <= top.hw - m + SLACK && Math.abs(lz) <= top.hd - m + SLACK;
}


/** Whether a placed thing stands on a top (the `surface` layer). */
export function onTop(content: Shapes, o: Placed): boolean {
  return content.object(o.def)?.layer === 'surface';
}

/** The top a thing on the `surface` layer stands on (from where it stands). */
export function hostOf(content: Shapes, objects: readonly Placed[], o: Placed): Top | null {
  if (!onTop(content, o)) return null;
  const [x, z] = middleOf(content, o);
  for (const h of objects) {
    if (h === o || (h.id !== undefined && h.id === o.id)) continue;
    const top = topOf(content, h);
    if (top && covers(top, x, z)) return top;
  }
  return null;
}

/** Why a thing can't stand somewhere on a surface. */
export type Misfit = 'noTop' | 'overEdge' | 'crowded';

/**
 * Where a thing of `radius` with its middle at `(x, z)` would stand: the top taking the whole
 * disc, with nothing else on a top in the way (`skip`: the thing being moved).
 */
export function fitOnTop(content: Shapes, objects: readonly Placed[], x: number, z: number, radius: number, skip?: Placed | number | null): Top | Misfit {
  let under: Top | null = null;
  let host: Top | null = null;
  for (const o of objects) {
    const top = topOf(content, o);
    if (!top || !covers(top, x, z)) continue;
    under ??= top;
    if (holds(top, x, z, radius)) {
      host = top;
      break;
    }
  }
  if (!host) return under ? 'overEdge' : 'noTop';
  const skipped = (o: Placed) => o === skip || (typeof skip === 'number' && o.id === skip) || (typeof skip === 'object' && skip !== null && skip.id !== undefined && skip.id === o.id);
  for (const o of objects) {
    if (skipped(o) || !onTop(content, o)) continue;
    const [ox, oz] = middleOf(content, o);
    if (Math.hypot(ox - x, oz - z) < content.radius(content.object(o.def)) + radius - SLACK) return 'crowded';
  }
  return host;
}

/** A point as a tile and an offset (centimetres off its middle), as things on tops are placed. */
export function spotAt(x: number, z: number): { x: number; z: number; offset: [number, number] } {
  const [tx, tz] = [Math.floor(x), Math.floor(z)];
  const cm = (v: number) => Math.max(-50, Math.min(50, Math.round(v * 100)));
  return { x: tx, z: tz, offset: [cm(x - tx - 0.5), cm(z - tz - 0.5)] };
}

/**
 * Free places on a top for a thing of `radius`, nearest its middle first, `step` metres apart:
 * for laying out tops (a vase in the middle of the table, books along a shelf).
 */
export function freeSpots(content: Shapes, objects: readonly Placed[], top: Top, radius: number, step = 0.1): { x: number; z: number; offset: [number, number] }[] {
  const m = top.inset + radius;
  const [rx, rz] = [top.hw - m, top.hd - m];
  if (rx < 0 || rz < 0) return [];
  const found: { d: number; spot: { x: number; z: number; offset: [number, number] } }[] = [];
  for (let iz = -Math.floor(rz / step); iz <= Math.floor(rz / step); iz++) {
    for (let ix = -Math.floor(rx / step); ix <= Math.floor(rx / step); ix++) {
      const [lx, lz] = [ix * step, iz * step];
      const spot = spotAt(top.cx + lx * top.cos + lz * top.sin, top.cz - lx * top.sin + lz * top.cos);
      const [x, z] = [spot.x + 0.5 + spot.offset[0] / 100, spot.z + 0.5 + spot.offset[1] / 100];
      if (isTop(fitOnTop(content, objects, x, z, radius), top)) found.push({ d: Math.hypot(lx, lz), spot });
    }
  }
  return found.sort((a, b) => a.d - b.d).map((f) => f.spot);
}

function isTop(fit: Top | Misfit, top: Top): boolean {
  return typeof fit !== 'string' && fit.host === top.host;
}

/** The tile in front of a footprint (min corner, rotated size `w`×`d`), where it's used from. */
export function frontTile(x: number, z: number, w: number, d: number, rot: number): [number, number] {
  const mx = x + Math.floor((w - 1) / 2);
  const mz = z + Math.floor((d - 1) / 2);
  const fronts: [number, number][] = [
    [mx, z + d],
    [x + w, mz],
    [mx, z - 1],
    [x - 1, mz],
  ];
  return fronts[rot % 4];
}

/**
 * The tiles beside a footprint a resident could use it from, best first (sim-core
 * `approach_tiles`): the front (and, along an even front edge, the other middle tile: the
 * simulation uses that one should the front be taken), then (seats used from `around`) behind,
 * left and right of its middle. Each comes with the tile of its own it's next to; the caller
 * checks it's free and open.
 */
export function approachCandidates(def: Shape | undefined, x: number, z: number, rot: number, size?: readonly [number, number]): { tile: [number, number]; inner: [number, number] }[] {
  const [w, d] = sizeOf(def, rot, size);
  const front = frontTile(x, z, w, d, rot);
  if (def?.approach !== 'around') {
    const inner: [number, number] = [front[0] - [0, 1, 0, -1][rot % 4], front[1] - [1, 0, -1, 0][rot % 4]];
    const along = rot % 2 === 0 ? [1, 0, w] : [0, 1, d];
    if (along[2] % 2 !== 0) return [{ tile: front, inner }];
    return [
      { tile: front, inner },
      { tile: [front[0] + along[0], front[1] + along[1]], inner: [inner[0] + along[0], inner[1] + along[1]] },
    ];
  }
  const [cx, cz] = [x + Math.floor((w - 1) / 2), z + Math.floor((d - 1) / 2)];
  // Beside each side (+z, +x, -z, -x).
  const sides: { tile: [number, number]; inner: [number, number] }[] = [
    { tile: [cx, z + d], inner: [cx, z + d - 1] },
    { tile: [x + w, cz], inner: [x + w - 1, cz] },
    { tile: [cx, z - 1], inner: [cx, z] },
    { tile: [x - 1, cz], inner: [x, cz] },
  ];
  const r = rot % 4;
  return [r, (r + 2) % 4, (r + 3) % 4, (r + 1) % 4].map((i) => sides[i]);
}

/**
 * Where a resident will use a footprint from as things stand (sim-core `approach_tile`): the first
 * of its `approachCandidates` that can be stood on (`standable`: free, and open to the tile of its
 * own it's next to), so the other middle tile when the front is taken. Null: none can be. `out`
 * is which way the tile lies from the footprint, in quarter turns as `rot`.
 */
export function approachTile(
  def: Shape | undefined,
  x: number,
  z: number,
  rot: number,
  size: readonly [number, number] | undefined,
  standable: (c: { tile: [number, number]; inner: [number, number] }) => boolean,
): { tile: [number, number]; out: number } | null {
  const found = approachCandidates(def, x, z, rot, size).find(standable);
  if (!found) return null;
  const [dx, dz] = [found.tile[0] - found.inner[0], found.tile[1] - found.inner[1]];
  return { tile: found.tile, out: dz > 0 ? 0 : dx > 0 ? 1 : dz < 0 ? 2 : 3 };
}

/**
 * The top a seat with a `seatAt` faces (sim-core `faced_table`): the one over the tile in front of
 * it, of a kind it tucks under, on the same storey (`storeyDepth`: rows per storey, 0 for one)
 * with no wall between (`wallBetween`).
 */
/** A top that is sat at by a use of its own (a dining set with its chairs, a desk with its chair built in): no other seat is pulled up to it (sim-core `seats_itself`). */
export function seatsItself(def: Shape | undefined): boolean {
  return !!def?.interactions?.some((i) => i.pose === 'sit');
}

export function facedTable(content: Shapes, objects: readonly Placed[], seat: Placed, wallBetween: (ax: number, az: number, bx: number, bz: number) => boolean, storeyDepth = 0): Placed | null {
  const def = content.object(seat.def);
  const kinds = def?.seatAt ?? [];
  if (!def || !kinds.length) return null;
  const [{ tile, inner }] = approachCandidates({ ...def, approach: 'front' }, seat.x, seat.z, seat.rot, footprintOf(content, seat));
  const storey = (z: number) => (storeyDepth > 0 ? Math.floor(z / storeyDepth) : 0);
  if (storey(tile[1]) !== storey(inner[1]) || wallBetween(inner[0], inner[1], tile[0], tile[1])) return null;
  for (const o of objects) {
    const host = content.object(o.def);
    const s = host?.surface;
    if (!s || !kinds.includes(s.kind)) continue;
    const [ow, od] = placedSize(content, o);
    // (A table or desk with seats of its own, a dining set or a desk with its chair built in: none.)
    if (tile[0] >= o.x && tile[1] >= o.z && tile[0] < o.x + ow && tile[1] < o.z + od) return seatsItself(host) ? null : o;
  }
  return null;
}
