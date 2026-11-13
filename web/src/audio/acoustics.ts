/**
 * Where a sound is heard from: how the camera's distance, the room a sound is made in and the room in
 * view change it. The rules only (no DOM or audio context); `space.ts` turns them into the mix's
 * filters, gains and reverb sends, `offline.ts` into samples for the tools and the tests.
 *
 * - Distance: at the game's own zoom (`DISTANCE.near`, where a game opens) and closer, a sound is as it
 *   was made. Further out it falls as a sound does over distance (half the level at twice as far) and
 *   loses its highs, down to a floor at the play camera's farthest so nothing goes silent; out in the
 *   town view the home's sounds sink further, to a murmur.
 * - Rooms: a sound made indoors goes to the reverb of a room its size (`rooms.ts`); outdoors it stays dry.
 * - Walls: zoomed in on a room, the sounds of the other rooms (another storey, the garden from indoors,
 *   indoors from the garden) come through the walls, duller and quieter (`OCCLUDE`). From the game's own
 *   zoom out the whole house is in view and nothing is muffled.
 */

import { DEFAULT_ORBIT, RADIUS_LIMITS, townBlend } from '../render/shared/lens';
import { roomSize, type RoomSize } from './rooms';

/**
 * The camera's distance: full level up to `near` m; the least left at the play camera's farthest (`floor`)
 * and in the full town view (`town`); the low-pass open (`open` Hz: over half any sample rate, the filter
 * passes everything) up to `near`, falling as distance to the power `dull`, never under `least` Hz.
 */
export const DISTANCE = { near: DEFAULT_ORBIT.radius, floor: DEFAULT_ORBIT.radius / RADIUS_LIMITS[1], town: 0.1, open: 24_000, dull: 1.3, least: 1800 };

/** The share of a sound's level left with the camera `radius` m from what it looks at (1 at the game's own zoom and closer). */
export function distanceGain(radius: number): number {
  const r = Number.isFinite(radius) && radius > 0 ? radius : DISTANCE.near;
  const fall = Math.max(DISTANCE.floor, Math.min(1, DISTANCE.near / r));
  return fall + (DISTANCE.town - DISTANCE.floor) * townBlend(r);
}

/** The low-pass on the world's sounds at that distance (Hz; `DISTANCE.open`: none). */
export function distanceCutoff(radius: number): number {
  const r = Number.isFinite(radius) && radius > 0 ? radius : DISTANCE.near;
  return Math.max(DISTANCE.least, Math.min(DISTANCE.open, DISTANCE.open * (DISTANCE.near / r) ** DISTANCE.dull));
}

/**
 * A sound from another room than the one in view: a low-pass at `cutoff` Hz and `gain` of its level (4 dB
 * down), in full with the camera within `near` m, easing to nothing by `far` m (the game's own zoom).
 */
export const OCCLUDE = { cutoff: 1500, gain: 0.63, near: 10, far: DEFAULT_ORBIT.radius };

/** How much the camera at `radius` m is in one room (1) rather than over the whole house (0). */
export function focus(radius: number): number {
  const u = Math.max(0, Math.min(1, (radius - OCCLUDE.near) / (OCCLUDE.far - OCCLUDE.near)));
  return 1 - u * u * (3 - 2 * u);
}

/** The low-pass (Hz) and the level of a sound muffled by `amount` (0 none, 1 through a wall, in between as the camera closes in). */
export function muffled(amount: number): { cutoff: number; gain: number } {
  const m = Math.max(0, Math.min(1, amount));
  return { cutoff: DISTANCE.open * (OCCLUDE.cutoff / DISTANCE.open) ** m, gain: OCCLUDE.gain ** m };
}

/** What of a world's structure says where its rooms are (`WorldStructure`). */
export interface RoomMap {
  width: number;
  depth: number;
  /** The room of each tile, row by row (0: outdoors), every storey's rows included. */
  rooms: ArrayLike<number>;
}

/** The world's rooms and the view, for the sounds: which room a point is in, how big it is, and whether it is the one looked into. */
export class Acoustics {
  private map: RoomMap | null = null;
  /** Tiles per room id. */
  private readonly areas = new Map<number, number>();
  private counted: ArrayLike<number> | null = null;
  private viewRoom = 0;
  private near = 0;
  /** The camera's distance (m). */
  radius: number = DISTANCE.near;

  /** The world's rooms (counted again only when they changed). */
  setWorld(map: RoomMap | null): void {
    this.map = map;
    if (!map || map.rooms === this.counted) return;
    this.counted = map.rooms;
    this.areas.clear();
    const rooms = map.rooms;
    for (let i = 0; i < rooms.length; i++) {
      const id = rooms[i];
      if (id) this.areas.set(id, (this.areas.get(id) ?? 0) + 1);
    }
  }

  /** The camera looks at tile point (`x`, `z`: the world's rows, the storey in view's) from `radius` m. */
  look(x: number, z: number, radius: number): void {
    this.radius = Number.isFinite(radius) && radius > 0 ? radius : DISTANCE.near;
    this.viewRoom = this.roomAt(x, z);
    this.near = focus(this.radius);
  }

  /** How far the camera is in on a room indoors (0..1): 0 looking at the garden or the street, or from the game's own zoom out. */
  get inside(): number {
    return this.viewRoom ? this.near : 0;
  }

  /** The room at a tile point (the world's rows; 0: outdoors, or off the map). */
  roomAt(x: number, z: number): number {
    const m = this.map;
    if (!m) return 0;
    const tx = Math.floor(x);
    const tz = Math.floor(z);
    if (!(tx >= 0 && tx < m.width && tz >= 0 && tz < m.depth)) return 0;
    return m.rooms[tz * m.width + tx] ?? 0;
  }

  /** The size of the room a sound at a tile point is made in; null outdoors (no reverb). */
  sizeAt(x: number, z: number): RoomSize | null {
    const room = this.roomAt(x, z);
    return room ? roomSize(this.areas.get(room) ?? 0) : null;
  }

  /** How muffled a sound made at a tile point is (0..1): in another room than the one in view, by how far the camera is in on it. */
  muffleAt(x: number, z: number): number {
    return this.roomAt(x, z) === this.viewRoom ? 0 : this.near;
  }
}

/** The game's (one world at a time): `world.ts` sets its world and its view, the voices read it too. */
export const acoustics = new Acoustics();
