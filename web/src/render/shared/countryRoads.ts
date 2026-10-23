/**
 * The roads that connect a town with the world outside, as laid out (engine-neutral; each renderer
 * draws them: `babylon/roads.ts`, `pc/roads.ts`): from both ends of its street (the middle one, in a
 * town of several streets) a country road winds out across the land to the horizon, levelled into
 * the hills and kept off the water. A town of several streets also gets a side road at each end
 * joining its streets, where the country road begins.
 *
 * Visual only and outside the town (residents leave town at the ends of its streets): the
 * landscape (`nature.ts`) levels the ground under the roads (`near`) and keeps trees off them.
 */

import type { WorldStructure } from '../../core/protocol';

/** Road widths (m): as wide as the town's streets where they leave it, narrower out in the country. */
export const TOWN_WIDTH = 6;
export const COUNTRY_WIDTH = 5;
/** Metres between the points of a road. */
export const STEP = 4;
/** Straight out of town for this many points before it starts to wind. */
export const STRAIGHT = 5;
/** Road surface above the levelled ground (m). */
export const LIFT = 0.06;
/** How far the ground is graded to the road on either side, past its edge (m). */
export const ROAD_VERGE = 7;

export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** What the roads need to know about the land. */
export interface RoadLand {
  seed: number;
  /** Town size (m). */
  w: number;
  d: number;
  /** How far the land reaches past the town. */
  ring: number;
  /** Ground height before the roads are cut in. */
  ground(x: number, z: number): number;
  /** Metres past the shoreline (above 0 in the water). */
  waterEdge(x: number, z: number): number;
  distanceToTown(x: number, z: number): number;
}

/** A point on a road: where it is, the road's surface height there and its width. */
export type Point = [x: number, z: number, h: number, width: number];

export interface NearRoad {
  /** Distance from the road's middle (m). */
  dist: number;
  /** Surface height of the road there. */
  h: number;
  width: number;
}

function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export class CountryRoads {
  /** Country roads, each from the town's edge outwards. */
  readonly roads: Point[][] = [];
  /** Side roads joining the ends of a town's streets (flat, on the apron round the town). */
  readonly links: Rect[] = [];
  /** Each road's bounding box (grown by its verge), for quick misses in `near`. */
  private readonly boxes: [number, number, number, number][] = [];

  constructor(
    private readonly land: RoadLand,
    world: WorldStructure,
    /** The town's name (its welcome signs). */
    readonly name: string,
  ) {
    const streets = (world.meta?.streets ?? []).filter((s) => s.w >= s.d).sort((a, b) => a.z - b.z);
    if (!streets.length) return;
    const first = streets[0];
    const last = streets[streets.length - 1];
    const several = streets.length > 1;
    if (several) {
      this.links.push({ x: -TOWN_WIDTH, z: first.z, w: TOWN_WIDTH, d: last.z + last.d - first.z });
      this.links.push({ x: land.w, z: first.z, w: TOWN_WIDTH, d: last.z + last.d - first.z });
    }
    const main = streets[Math.floor((streets.length - 1) / 2)];
    const z = main.z + main.d / 2;
    for (const dir of [-1, 1] as const) {
      const x = dir < 0 ? (several ? -TOWN_WIDTH : 0) : land.w + (several ? TOWN_WIDTH : 0);
      const road = this.lay(x, z, dir);
      this.roads.push(road);
      const xs = road.map((p) => p[0]);
      const zs = road.map((p) => p[1]);
      const m = TOWN_WIDTH / 2 + ROAD_VERGE;
      this.boxes.push([Math.min(...xs) - m, Math.min(...zs) - m, Math.max(...xs) + m, Math.max(...zs) + m]);
    }
  }

  /**
   * A road from (`x`, `z`) out of town (`dir`: -1 west, +1 east): straight for a little, then
   * wandering gently, never turning back towards town and steering away from the water, out to
   * the end of the land. Its surface follows the ground, smoothed so it rises and falls gently.
   */
  private lay(x: number, z: number, dir: -1 | 1): Point[] {
    const land = this.land;
    const rand = mulberry32(land.seed ^ (dir < 0 ? 0x7a0d1 : 0x7a0d2));
    const flat: [number, number][] = [];
    let a = 0;
    let bend = (rand() - 0.5) * 0.04;
    for (let i = 0; i < 220 && land.distanceToTown(x, z) < land.ring + 25; i++) {
      flat.push([x, z]);
      if (i >= STRAIGHT) {
        // A slowly changing bend; it straightens out again towards the way out of town.
        if (rand() < 0.12) bend = (rand() - 0.5) * 0.1;
        a = Math.max(-0.6, Math.min(0.6, (a + bend + (rand() - 0.5) * 0.04) * 0.985));
        // Where the land climbs, the road keeps to the lower ground (valleys, passes).
        const ahead = (h: number) => land.ground(x + dir * Math.cos(h) * 16, z + Math.sin(h) * 16);
        const lower = [a - 0.2, a + 0.2].filter((h) => Math.abs(h) <= 0.75).reduce((best, h) => (ahead(h) < ahead(best) - 0.8 ? h : best), a);
        a += (lower - a) * 0.6;
        // Water ahead: turn back towards the town's side of it.
        const [ax, az] = [x + dir * Math.cos(a) * 14, z + Math.sin(a) * 14];
        if (land.waterEdge(ax, az) > -10) a += (land.d / 2 - z > 0 ? 1 : -1) * 0.22;
      }
      x += dir * Math.cos(a) * STEP;
      z += Math.sin(a) * STEP;
    }
    // Heights: the ground along the way, smoothed, kept above the water.
    const raw = flat.map(([px, pz]) => land.ground(px, pz));
    return flat.map(([px, pz], i) => {
      let sum = 0;
      let n = 0;
      for (let k = Math.max(0, i - 3); k <= Math.min(raw.length - 1, i + 3); k++, n++) sum += raw[k];
      const h = i < 2 ? 0 : Math.max(0.1, sum / n);
      const width = TOWN_WIDTH + (COUNTRY_WIDTH - TOWN_WIDTH) * Math.min(1, i / 6);
      return [px, pz, h, width];
    });
  }

  /** The road nearest a point within its width and verge (null: none that close). */
  near(x: number, z: number): NearRoad | null {
    let best: NearRoad | null = null;
    this.roads.forEach((road, r) => {
      const [x0, z0, x1, z1] = this.boxes[r];
      if (x < x0 || x > x1 || z < z0 || z > z1) return;
      for (let i = 0; i + 1 < road.length; i++) {
        const [ax, az, ah, aw] = road[i];
        const [bx, bz, bh, bw] = road[i + 1];
        const [dx, dz] = [bx - ax, bz - az];
        const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz)));
        const dist = Math.hypot(x - (ax + dx * t), z - (az + dz * t));
        if (dist < (best?.dist ?? aw / 2 + ROAD_VERGE)) best = { dist, h: ah + (bh - ah) * t, width: aw + (bw - aw) * t };
      }
    });
    return best;
  }
}
