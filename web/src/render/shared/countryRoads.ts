/**
 * The roads that connect a town with the world outside, as laid out (engine-neutral; the renderer
 * draws them: `pc/roads.ts`): from wherever a town's road leads out of it (`townExits`; an older
 * town: both ends of its street, the middle one of several) a country road winds on across the
 * land to the horizon, levelled into the hills and kept off the water. An older town of several
 * streets also gets a side road at each end joining its streets, where the country road begins.
 *
 * Visual only and outside the town (residents leave town where its roads do): the landscape
 * (`nature.ts`) levels the ground under the roads (`near`) and keeps trees off them.
 */

import type { WorldStructure } from '../../core/protocol';
import { townExits } from './roads';

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
  /** `near`'s last question and answer. */
  private readonly asked = [NaN, NaN];
  private answer: NearRoad | null = null;

  constructor(
    private readonly land: RoadLand,
    world: WorldStructure,
    /** The town's name (its welcome signs). */
    readonly name: string,
  ) {
    const exits = townExits(world);
    // An older town of several streets: side roads joining their ends, the country roads beyond them.
    const streets = world.meta?.roads ? [] : (world.meta?.streets ?? []).filter((s) => s.w >= s.d).sort((a, b) => a.z - b.z);
    const several = streets.length > 1;
    if (several) {
      const [first, last] = [streets[0], streets[streets.length - 1]];
      this.links.push({ x: -TOWN_WIDTH, z: first.z, w: TOWN_WIDTH, d: last.z + last.d - first.z });
      this.links.push({ x: land.w, z: first.z, w: TOWN_WIDTH, d: last.z + last.d - first.z });
    }
    exits.forEach((e, k) => {
      const road = this.lay(e.x + (several ? e.dx * TOWN_WIDTH : 0), e.z, e.dx, e.dz, k);
      this.roads.push(road);
    });
  }

  /**
   * A road from (`x`, `z`) out of town, heading (`ux`, `uz`) (the `k`th way out): straight for a
   * little, then wandering gently, never turning back towards town and steering away from the
   * water, out to the end of the land. Its surface follows the ground, smoothed so it rises and
   * falls gently.
   */
  private lay(x: number, z: number, ux: number, uz: number, k: number): Point[] {
    const land = this.land;
    // An older town's two roads, west and east, wander as they always have.
    const rand = mulberry32(land.seed ^ (ux === -1 && k === 0 ? 0x7a0d1 : ux === 1 && k === 1 ? 0x7a0d2 : 0x7a0d3 + k * 977));
    // Wandering to the side of the way out: `a` turns towards `v` (towards larger z, or larger x heading along z).
    const [vx, vz] = ux > 0 || (ux === 0 && uz < 0) ? [-uz, ux] : [uz, -ux];
    const step = (h: number, len: number): [number, number] => [x + (ux * Math.cos(h) + vx * Math.sin(h)) * len, z + (uz * Math.cos(h) + vz * Math.sin(h)) * len];
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
        const ahead = (h: number) => land.ground(...step(h, 16));
        const lower = [a - 0.2, a + 0.2].filter((h) => Math.abs(h) <= 0.75).reduce((best, h) => (ahead(h) < ahead(best) - 0.8 ? h : best), a);
        a += (lower - a) * 0.6;
        // Water ahead: turn back towards the town's side of it.
        if (land.waterEdge(...step(a, 14)) > -10) a += ((land.w / 2 - x) * vx + (land.d / 2 - z) * vz > 0 ? 1 : -1) * 0.22;
      }
      [x, z] = step(a, STEP);
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

  /** Whether (x, z) is on a side road (`links`), or within `margin` m of one. */
  onLink(x: number, z: number, margin = 0): boolean {
    return this.links.some((r) => x > r.x - margin && x < r.x + r.w + margin && z > r.z - margin && z < r.z + r.d + margin);
  }

  /** The road nearest a point within its width and verge (null: none that close). */
  near(x: number, z: number): NearRoad | null {
    // (Asked twice at a point in a row, by the land's height and its colour: answered once.)
    if (x === this.asked[0] && z === this.asked[1]) return this.answer;
    this.asked[0] = x;
    this.asked[1] = z;
    return (this.answer = this.nearest(x, z));
  }

  /**
   * The road segments by 8 m cell (`GRID`): each in every cell its box, grown by its width and verge,
   * touches, in the roads' order, so the nearest is found as the full scan would find it (ties too).
   */
  private grid: Map<number, number[]> | null = null;

  private cells(): Map<number, number[]> {
    if (this.grid) return this.grid;
    const grid = new Map<number, number[]>();
    this.roads.forEach((road, r) => {
      for (let i = 0; i + 1 < road.length; i++) {
        const [ax, az, , aw] = road[i];
        const [bx, bz, , bw] = road[i + 1];
        const m = Math.max(aw, bw) / 2 + ROAD_VERGE;
        for (let cx = Math.floor((Math.min(ax, bx) - m) / GRID); cx <= Math.floor((Math.max(ax, bx) + m) / GRID); cx++) {
          for (let cz = Math.floor((Math.min(az, bz) - m) / GRID); cz <= Math.floor((Math.max(az, bz) + m) / GRID); cz++) {
            const k = cellKey(cx, cz);
            let list = grid.get(k);
            if (!list) grid.set(k, (list = []));
            list.push(r * SEGMENTS + i);
          }
        }
      }
    });
    return (this.grid = grid);
  }

  private nearest(x: number, z: number): NearRoad | null {
    const list = this.cells().get(cellKey(Math.floor(x / GRID), Math.floor(z / GRID)));
    if (!list) return null;
    let best: NearRoad | null = null;
    for (const s of list) {
      const road = this.roads[Math.floor(s / SEGMENTS)];
      const i = s % SEGMENTS;
      const [ax, az, ah, aw] = road[i];
      const [bx, bz, bh, bw] = road[i + 1];
      const [dx, dz] = [bx - ax, bz - az];
      const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz)));
      const dist = Math.hypot(x - (ax + dx * t), z - (az + dz * t));
      if (dist < (best?.dist ?? aw / 2 + ROAD_VERGE)) best = { dist, h: ah + (bh - ah) * t, width: aw + (bw - aw) * t };
    }
    return best;
  }
}

/** Side of `near`'s grid cells (m). */
const GRID = 8;
/** Segment ids: road index × this + segment index (a road has fewer points). */
const SEGMENTS = 1 << 16;
const cellKey = (cx: number, cz: number) => (cx + 32768) * 65536 + (cz + 32768);
