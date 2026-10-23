/**
 * Chairs pulled up to a table. A seat with a `seatAt` facing a top of a kind it tucks under (sim-core
 * `faced_table`) stands a tile from it: drawn there, the sitter's hands stop short of the top. While
 * someone sits on it, the chair is drawn pulled in with them (`Characters.pulled`, the renderer moves
 * its instance) as far as the top's near edge and the room under it for their legs allow.
 *
 * What is in front of each such seat is read once per world from what is drawn of the table (its
 * triangles, as boxes in the seat's own frame), so round tops, an L-shaped desk, drawer pedestals,
 * a hanging drawer box or a counter's cabinet all count as they are modelled. Engine-neutral: each
 * renderer hands over what it draws of an object (`DrawnObject`).
 */

import type { ObjectPlacement, WorldStructure } from '../../core/protocol';
import type { Content } from '../../content/content';
import { facedTable } from '../../game/surfaces';
import { transformPoint, type Mat4 } from './math';

/** What a seat pulled up to a table has in front of it, in the seat's frame (from its middle, on its floor). */
export interface TableRoom {
  /** The table (object id). */
  host: number;
  /** The top's height above the seat's floor (m). */
  top: number;
  /** The table's triangles as boxes, 6 floats each: right0, right1, up0, up1, ahead0, ahead1 (m). */
  boxes: Float32Array;
}

/** A mesh of an object as drawn: its vertices, its triangles, and where they are drawn (a vertex times `matrix`: game space). */
export interface DrawnGeometry {
  positions: ArrayLike<number>;
  indices: ArrayLike<number>;
  matrix: Readonly<Mat4>;
}

/** An object as drawn: where its middle is (its instance's place, game space), and its meshes (its moving parts' too). */
export interface DrawnObject {
  x: number;
  z: number;
  geometry(): DrawnGeometry[];
}

/** Only what is this close to the seat's line (m, to either side) and this far ahead is kept. */
const KEEP_SIDE = 0.6;
const KEEP_AHEAD = 2.5;

/**
 * The tables the seats in `world` face, with what of each is in front of the seat (`TableRoom`), by
 * seat id. `drawn`: an object as it is drawn, if it is; `floor`: the floor height of its row.
 */
export function tableRooms(world: WorldStructure, content: Content, drawn: (id: number) => DrawnObject | undefined, floor: (o: ObjectPlacement) => number): Map<number, TableRoom> {
  const rooms = new Map<number, TableRoom>();
  const seats = world.objects.filter((o) => content.object(o.def)?.seatAt?.length);
  if (!seats.length) return rooms;
  const walls = new Set(world.walls.map((w) => `${w.axis}:${w.x}:${w.z}`));
  const wallBetween = (ax: number, az: number, bx: number, bz: number) => walls.has(bx !== ax ? `v:${Math.max(ax, bx)}:${az}` : `h:${ax}:${Math.max(az, bz)}`);
  const v = [0, 0, 0];
  for (const seat of seats) {
    const table = facedTable(content, world.objects, seat, wallBetween, world.storeyDepth ?? 0) as ObjectPlacement | null;
    const surface = table && content.object(table.def)?.surface;
    const there = table && drawn(table.id);
    const own = drawn(seat.id);
    if (!table || !surface || !there || !own) continue;
    // The seat's frame: its middle (its instance's place), ahead along its facing, right across it.
    const [cx, cz] = [own.x, own.z];
    const y0 = floor(seat);
    const yaw = (seat.rot * Math.PI) / 2;
    const [ax, az] = [Math.sin(yaw), Math.cos(yaw)];
    const [rx, rz] = [Math.cos(yaw), -Math.sin(yaw)];
    const boxes: number[] = [];
    for (const { positions, indices, matrix } of there.geometry()) {
      const n = positions.length / 3;
      const right = new Float32Array(n);
      const up = new Float32Array(n);
      const ahead = new Float32Array(n);
      for (let i = 0; i < n; i++) {
        transformPoint(matrix, positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2], v);
        const [dx, dz] = [v[0] - cx, v[2] - cz];
        right[i] = dx * rx + dz * rz;
        up[i] = v[1] - y0;
        ahead[i] = dx * ax + dz * az;
      }
      for (let t = 0; t + 2 < indices.length; t += 3) {
        const [a, b, c] = [indices[t], indices[t + 1], indices[t + 2]];
        const r0 = Math.min(right[a], right[b], right[c]);
        const r1 = Math.max(right[a], right[b], right[c]);
        const f0 = Math.min(ahead[a], ahead[b], ahead[c]);
        const f1 = Math.max(ahead[a], ahead[b], ahead[c]);
        if (r1 < -KEEP_SIDE || r0 > KEEP_SIDE || f1 < 0 || f0 > KEEP_AHEAD) continue;
        boxes.push(r0, r1, Math.min(up[a], up[b], up[c]), Math.max(up[a], up[b], up[c]), f0, f1);
      }
    }
    if (boxes.length) rooms.set(seat.id, { host: table.id, top: surface.height + (table.tall ?? 0) / 100, boxes: new Float32Array(boxes) });
  }
  return rooms;
}

/**
 * How far ahead of the seat's middle (m) the nearest of the table is within `right` ± `half` across
 * and between `low` and `high` above the floor (Infinity: nothing there).
 */
export function nearestAhead(room: TableRoom, right: number, half: number, low: number, high: number): number {
  const b = room.boxes;
  let near = Infinity;
  for (let i = 0; i < b.length; i += 6) {
    if (b[i + 1] < right - half || b[i] > right + half || b[i + 3] < low || b[i + 2] > high) continue;
    if (b[i + 4] < near) near = b[i + 4];
  }
  return near;
}

/** How far ahead of the seat's middle (m) the table reaches at its furthest within `right` ± `half` across and between `low` and `high` (-Infinity: nothing there). */
export function furthestAhead(room: TableRoom, right: number, half: number, low: number, high: number): number {
  const b = room.boxes;
  let far = -Infinity;
  for (let i = 0; i < b.length; i += 6) {
    if (b[i + 1] < right - half || b[i] > right + half || b[i + 3] < low || b[i + 2] > high) continue;
    if (b[i + 5] > far) far = b[i + 5];
  }
  return far;
}
