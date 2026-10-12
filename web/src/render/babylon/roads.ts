/**
 * The roads that connect a town with the world outside: from both ends of its street (the middle
 * one, in a town of several streets) a country road winds out across the land to the horizon,
 * levelled into the hills and kept off the water, with a "Welcome to …" sign by the way into
 * town. A town of several streets also gets a side road at each end joining its streets, where
 * the country road begins.
 *
 * Visual only and outside the town (residents leave town at the ends of its streets): the
 * landscape (`nature.ts`) levels the ground under the roads, keeps trees off them and draws them.
 */

import { Color3, DynamicTexture, Mesh, MeshBuilder, StandardMaterial, type Scene } from './core';
import type { WorldStructure } from '../../core/protocol';
import { Geo, type V3 } from './geometry';
import { MaterialLibrary } from './materials';

/** Road widths (m): as wide as the town's streets where they leave it, narrower out in the country. */
const TOWN_WIDTH = 6;
const COUNTRY_WIDTH = 5;
/** Metres between the points of a road. */
const STEP = 4;
/** Straight out of town for this many points before it starts to wind. */
const STRAIGHT = 5;
/** Road surface above the levelled ground (m). */
const LIFT = 0.06;
/** How far the ground is graded to the road on either side, past its edge (m). */
export const ROAD_VERGE = 7;

interface Rect {
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
type Point = [x: number, z: number, h: number, width: number];

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
    private readonly name: string,
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

  /** The road surfaces, their centre lines and the welcome signs. */
  build(scene: Scene): Mesh[] {
    const lib = MaterialLibrary.for(scene);
    const surface = new Geo();
    const lines = new Geo().color('#E8E2CC');
    const out: Mesh[] = [];
    const quad = (g: Geo, p: V3[]) => g.poly(p, [0, 1, 0]);

    for (const r of this.links) {
      const y = 0.012;
      quad(surface, [[r.x, y, r.z], [r.x + r.w, y, r.z], [r.x + r.w, y, r.z + r.d], [r.x, y, r.z + r.d]]);
      const xc = r.x + r.w / 2;
      for (let z = r.z + 1; z + 2.4 < r.z + r.d - 1; z += 5) quad(lines, [[xc - 0.06, y + 0.005, z], [xc + 0.06, y + 0.005, z], [xc + 0.06, y + 0.005, z + 2.4], [xc - 0.06, y + 0.005, z + 2.4]]);
    }
    for (const road of this.roads) {
      // Each point's sideways direction (across the road), from its neighbours.
      const side = road.map((_, i) => {
        const [ax, az] = road[Math.max(0, i - 1)];
        const [bx, bz] = road[Math.min(road.length - 1, i + 1)];
        const l = Math.hypot(bx - ax, bz - az) || 1;
        return [-(bz - az) / l, (bx - ax) / l];
      });
      const edge = (i: number, s: number, y: number, half = road[i][3] / 2): V3 => [road[i][0] + side[i][0] * half * s, road[i][2] + y, road[i][1] + side[i][1] * half * s];
      for (let i = 0; i + 1 < road.length; i++) {
        quad(surface, [edge(i, 1, LIFT), edge(i + 1, 1, LIFT), edge(i + 1, -1, LIFT), edge(i, -1, LIFT)]);
        // A dash along the middle of every stretch (2.4 m of each 4).
        const [ax, az, ah] = road[i];
        const [bx, bz, bh] = road[i + 1];
        const t = 2.4 / STEP;
        const [cx, cz, ch] = [ax + (bx - ax) * t, az + (bz - az) * t, ah + (bh - ah) * t];
        const [sx, sz] = side[i].map((v) => v * 0.06);
        const y = LIFT + 0.006;
        quad(lines, [[ax + sx, ah + y, az + sz], [cx + sx, ch + y, cz + sz], [cx - sx, ch + y, cz - sz], [ax - sx, ah + y, az - sz]]);
      }
      const sign = this.sign(scene, road);
      if (sign) out.push(...sign);
    }
    const roads = surface.toMesh('countryRoads', scene, { colors: false });
    if (roads) {
      roads.material = lib.surface('material.road', { doubleSided: true });
      roads.receiveShadows = true;
      out.push(roads);
    }
    const marks = lines.toMesh('countryRoadLines', scene, { colors: true });
    if (marks) {
      marks.material = lib.finish('material.finish.matte');
      marks.receiveShadows = true;
      out.push(marks);
    }
    for (const m of out) m.isPickable = false;
    return out;
  }

  /** A "Welcome to …" sign on the right of the road into town, a little way out, facing arrivals. */
  private sign(scene: Scene, road: Point[]): Mesh[] | null {
    const i = Math.min(road.length - 1, STRAIGHT);
    if (!this.name || i < 1) return null;
    const [x, z, h, width] = road[i];
    // Arrivals drive towards the town (point i towards point i - 1); the sign stands on their right.
    const [bx, bz] = [road[i - 1][0] - x, road[i - 1][1] - z];
    const l = Math.hypot(bx, bz) || 1;
    const [fx, fz] = [bx / l, bz / l];
    const right = [-fz, fx];
    const off = width / 2 + 1.6;
    const [px, pz] = [x + right[0] * off, z + right[1] * off];

    const g = new Geo();
    g.color('#6B5236');
    g.box(-1.35, 0, -0.06, -1.2, 1.9, 0.06);
    g.box(1.2, 0, -0.06, 1.35, 1.9, 0.06);
    g.color('#2F5D3A');
    g.box(-1.5, 1.0, -0.05, 1.5, 2.1, 0.02);
    const frame = g.toMesh('welcomeSign', scene, { colors: true });
    if (!frame) return null;
    frame.material = MaterialLibrary.for(scene).finish('material.finish.satin');

    const tex = new DynamicTexture('welcomeSignText', { width: 512, height: 192 }, scene, true);
    const ctx = tex.getContext() as CanvasRenderingContext2D;
    ctx.fillStyle = '#2F5D3A';
    ctx.fillRect(0, 0, 512, 192);
    ctx.strokeStyle = '#F2EEE2';
    ctx.lineWidth = 6;
    ctx.strokeRect(10, 10, 492, 172);
    ctx.fillStyle = '#F2EEE2';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.font = '600 34px sans-serif';
    ctx.fillText('Welcome to', 256, 62);
    let size = 64;
    do ctx.font = `700 ${size}px sans-serif`;
    while (ctx.measureText(this.name).width > 460 && (size -= 4) > 24);
    ctx.fillText(this.name, 256, 124);
    tex.update();
    const mat = new StandardMaterial('welcomeSignText', scene);
    mat.diffuseTexture = tex;
    mat.specularColor = Color3.Black();
    mat.emissiveColor = new Color3(0.25, 0.25, 0.25);
    const text = MeshBuilder.CreatePlane('welcomeSignText', { width: 2.9, height: 1.04 }, scene);
    text.material = mat;
    text.parent = frame;
    // The plane faces -z; the board's front is +z.
    text.rotation.y = Math.PI;
    text.position.set(0, 1.55, 0.03);

    frame.position.set(px, h, pz);
    frame.rotation.y = Math.atan2(fx, fz) + Math.PI;
    frame.receiveShadows = true;
    text.isPickable = false;
    return [frame, text];
  }
}
