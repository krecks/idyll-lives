/**
 * The neighbourhood around the viewed lot, as a lived-in street: the neighbour houses
 * (`HouseBuilder.silhouettes`, with porches, chimneys and garages), sidewalks with kerbs and
 * a dashed centre line, streetlights that light up at night (with pools of light on the
 * ground), mailboxes, hydrants and bins along the kerb, cars parked on neighbours' driveways,
 * picket fences or hedges around neighbours' back gardens, and flower beds along their house
 * fronts, with pots by the door and little lights along the garden path; the things standing in
 * their gardens. The town's public places seen from another lot: shops as silhouettes with an awning,
 * and the park, playground and market things standing in the open. At the ends of the street a bus
 * stop and street signs; benches, planters and bins along the kerb in front of the public places.
 *
 * Rules:
 * - Visual only, and kept off where Sims walk: front lawns (Sims cut across them on their way
 *   out of town), garden paths, driveway mouths, the road and the middle of the sidewalk. Kerb
 *   furniture stands on the road side of the sidewalk, fences only run from the house front line
 *   backwards, cars park beside the houses.
 * - The things outdoors on other lots (neighbours' garden furniture and planters, the park's
 *   benches; not what stands indoors, in the silhouettes) in the game, placed, turned and moved off
 *   their tiles' middle as the lot draws them: the lawn is cleared under every lot's things
 *   (`lawnMask`). The menus' town (`pc/overview.ts`) draws every lot's things itself.
 * - Nothing on the viewed lot: the player builds there (the mailbox stands on the sidewalk). In
 *   the menus' town the player's lot (`WorldMeta.home`) gets its house as a shell dressed as the
 *   game builds it (no porch, chimney or garden dressing), and nothing else. Its garage, if the
 *   town gave it one (`WorldStructure.garages`), stands on it in the game too, drawn apart
 *   (`StreetPlan.lot`): it goes with the lot's ground while its basement is in view.
 * - Cars: the world's (`WorldStructure.cars`), parked, in the menus' town; in the game they come
 *   and go, drawn by `pc/cars.ts`. A game saved before there were cars gets them by some neighbours'
 *   driveways (`parkCar`), parked for good.
 * - Static: generated once per town layout and kept across structure versions of the viewed
 *   lot (build-mode edits don't rebuild it), merged into a handful of meshes (one per material)
 *   plus thin-instanced foliage and props (manifest models: `model.street.*` and the town's
 *   things, see `assets/packs/street/manifest.json`), a few dozen draw calls for the whole street.
 * - Models built in Blender (`.glb` files, downloaded like the rest), plus a CC0 concrete texture for the sidewalks.
 *
 * Engine-neutral: `planStreet` lays the street out and writes its geometry and its models'
 * instances (`StreetPlan`); the renderer's `Street` (`pc/street.ts`) draws it.
 */

import type { ModelEntry } from '../../assets/types';
import { groundDepth, objectCentre, storeyOfRow, storeyRow, type ObjectPlacement, type Opening, type PlotInfo, type RoadPlan, type WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import { Geo, type V3 } from './geometry';
import { CARS, CAR_PAINT, garageFor, hash, paintCar, parkCar, pick, type CarBody } from './cars';
import { chunkAt, hashNumbers, hashString, RectGrid } from './chunks';
import type { HousePlanner, SilhouettePlan, SilhouetteSpec } from './house';
import { compose, quatYawPitchRoll, type Quat } from './math';
import { placementVariation, type StandIn } from './modelEntry';
import { centroid } from '../../game/roads';
import { Kerb, KERB, roadIndex, roadMeshes, SIDEWALK, townRoads, WALK_Y, type RoadIndex } from './roads';

/** Vertex-colour alpha for the plain finish, as the material libraries' `encodeFinish`: [0, 0.5) dielectric, [0.5, 1] metal; roughness scaled within. */
function encodeFinish(roughness: number, metallic: number): number {
  const r = Math.min(0.999, Math.max(0, roughness)) * 0.5;
  return metallic >= 0.5 ? 0.5 + r : r;
}

export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** Paths at least this wide are driveways (garden paths are 1 m; see `game/town.ts`). */
const DRIVEWAY_MIN_WIDTH = 1.8;
/** Half wall thickness (as in house.ts). */
const T = 0.07;

const MAILBOX = ['#22262A', '#2F4A3A', '#E8E6E0', '#8C2A26', '#2E4568'];
const BIN = ['#3C6B3A', '#55595E', '#2E5C8A'];
const FENCE_WHITE = '#F2F0EA';
/** Weathered wood of the rail fences round farms and their fields. */
const RAIL_WOOD = '#8A6F52';
/**
 * How far neighbours' fences (and hedges, their middle) keep from the viewed lot's edge: clear of
 * the pit dug round it while its basement is in view (`PIT_MARGIN` in PlayCanvasRenderer).
 */
const LOT_CLEARANCE = { picket: 0.4, rail: 0.4, hedge: 0.8 };

/** Model keys of the thin-instanced foliage (the street and trees packs). */
export const FOLIAGE = { bush: 'model.bush', flowers: 'model.street.flowers', hedge: 'model.hedge' } as const;
export type FoliageKind = keyof typeof FOLIAGE;
/** Props drawn from manifest models (the street and town packs); a missing one is left out. */
const PROP = {
  bench: 'model.street.bench',
  planter: 'model.street.planter',
  flowerTub: 'model.street.flowerTub',
  sign: 'model.street.sign',
  busSign: 'model.street.busSign',
  shelter: 'model.busStop',
  fountain: 'model.fountainTown',
  pot: 'model.street.pot',
  pathLight: 'model.street.pathLight',
  litterBin: 'model.litterBin',
} as const;
/** A thin instance: x, z, scale across, scale up, yaw, and how high it stands (a thing on a top; 0 if left out). */
export type Inst = [number, number, number, number, number, number?];
/** Shop awnings: stripe colours. */
const AWNINGS: [string, string][] = [['#2F6B4F', '#F2F0EA'], ['#B8322B', '#F2F0EA'], ['#2E5C8A', '#F2F0EA'], ['#C98F2E', '#F7EBD3'], ['#6B3E5E', '#F2E6EE']];
/** What a placed object looks like and how big it is (the game's content), for the things on other lots. */
export type ObjectLook = (def: string) => { model: string; footprint: [number, number]; standIn: StandIn } | undefined;

const overlaps = (a: Rect, b: Rect, m = 0) => a.x - m < b.x + b.w && a.x + a.w + m > b.x && a.z - m < b.z + b.d && a.z + a.d + m > b.z;

/*
 * A plot's dressing is laid out as if its street ran along x (`PlotLayout`'s coordinates): a plot
 * on a street along z is turned a quarter for it, the town's (x, z) at (z, -x), and its dressing
 * turned back onto the town.
 */
const localRect = (turned: boolean, r: Rect): Rect => (turned ? { x: r.z, z: -(r.x + r.w), w: r.d, d: r.w } : r);
const townRect = (turned: boolean, r: Rect): Rect => (turned ? { x: -(r.z + r.d), z: r.x, w: r.d, d: r.w } : r);
const townPoint = (turned: boolean, x: number, z: number): [number, number] => (turned ? [-z, x] : [x, z]);
const townYaw = (turned: boolean, yaw: number) => (turned ? yaw - Math.PI / 2 : yaw);
/** A box (x0, z0, x1, z1) as laid out, on the town. */
const townBox = (turned: boolean, b: { x0: number; z0: number; x1: number; z1: number }) => {
  const r = townRect(turned, { x: b.x0, z: b.z0, w: b.x1 - b.x0, d: b.z1 - b.z0 });
  return { x0: r.x, z0: r.z, x1: r.x + r.w, z1: r.z + r.d };
};

/**
 * Writes posed primitives into a `Geo`: a local frame (origin on the ground, +z forward)
 * placed at x/z with a yaw, and a plain finish (colour + roughness/metalness in the vertex
 * alpha, see `encodeFinish`).
 */
export class Painter {
  private ox = 0;
  private oy = 0;
  private oz = 0;
  private c = 1;
  private s = 0;
  constructor(readonly geo: Geo) {}

  at(x: number, z: number, yaw = 0, y = 0): this {
    this.ox = x;
    this.oy = y;
    this.oz = z;
    this.c = Math.cos(yaw);
    this.s = Math.sin(yaw);
    return this;
  }

  paint(hex: string, rough = 0.7, metal = 0): this {
    this.geo.color(hex, encodeFinish(rough, metal));
    return this;
  }

  p(x: number, y: number, z: number): V3 {
    return [this.ox + x * this.c + z * this.s, this.oy + y, this.oz - x * this.s + z * this.c];
  }

  n(x: number, y: number, z: number): V3 {
    return [x * this.c + z * this.s, y, -x * this.s + z * this.c];
  }

  poly(pts: readonly V3[], n: V3): this {
    this.geo.poly(
      pts.map((q) => this.p(q[0], q[1], q[2])),
      this.n(n[0], n[1], n[2]),
    );
    return this;
  }

  box(x0: number, y0: number, z0: number, x1: number, y1: number, z1: number, skip = ''): this {
    if (!skip.includes('py')) this.poly([[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]], [0, 1, 0]);
    if (!skip.includes('ny')) this.poly([[x0, y0, z0], [x0, y0, z1], [x1, y0, z1], [x1, y0, z0]], [0, -1, 0]);
    if (!skip.includes('nz')) this.poly([[x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0]], [0, 0, -1]);
    if (!skip.includes('pz')) this.poly([[x0, y0, z1], [x0, y1, z1], [x1, y1, z1], [x1, y0, z1]], [0, 0, 1]);
    if (!skip.includes('nx')) this.poly([[x0, y0, z0], [x0, y1, z0], [x0, y1, z1], [x0, y0, z1]], [-1, 0, 0]);
    if (!skip.includes('px')) this.poly([[x1, y0, z0], [x1, y0, z1], [x1, y1, z1], [x1, y1, z0]], [1, 0, 0]);
    return this;
  }

  /**
   * A tube (frustum) along `axis` from a0 (radius r0) to a1 (radius r1), centred on (cu, cv):
   * the other two axes in x, y, z order. `from`/`to` limit the sweep (radians, from the first
   * of the other axes towards the second); caps close the ends.
   */
  tube(axis: 'x' | 'y' | 'z', a0: number, a1: number, cu: number, cv: number, r0: number, r1: number, seg: number, caps = true, from = 0, to = Math.PI * 2): this {
    const P = (a: number, u: number, v: number): V3 => (axis === 'x' ? [a, u, v] : axis === 'y' ? [u, a, v] : [u, v, a]);
    const full = to - from >= Math.PI * 2 - 1e-6;
    const slope = (r0 - r1) / Math.max(1e-6, Math.abs(a1 - a0)) * Math.sign(a1 - a0);
    for (let i = 0; i < seg; i++) {
      const t0 = from + ((to - from) * i) / seg;
      const t1 = from + ((to - from) * (i + 1)) / seg;
      const tm = (t0 + t1) / 2;
      const [cu0, cv0, cu1, cv1] = [Math.cos(t0), Math.sin(t0), Math.cos(t1), Math.sin(t1)];
      const len = Math.hypot(1, slope);
      const n = P(slope / len, Math.cos(tm) / len, Math.sin(tm) / len);
      this.poly([P(a0, cu + cu0 * r0, cv + cv0 * r0), P(a0, cu + cu1 * r0, cv + cv1 * r0), P(a1, cu + cu1 * r1, cv + cv1 * r1), P(a1, cu + cu0 * r1, cv + cv0 * r1)], n);
    }
    if (caps) {
      for (const [a, r, sgn] of [[a0, r0, -Math.sign(a1 - a0)], [a1, r1, Math.sign(a1 - a0)]] as const) {
        if (r <= 0.001) continue;
        const pts: V3[] = [];
        if (!full) pts.push(P(a, cu, cv));
        for (let i = 0; i <= seg; i++) {
          if (full && i === seg) break;
          const t = from + ((to - from) * i) / seg;
          pts.push(P(a, cu + Math.cos(t) * r, cv + Math.sin(t) * r));
        }
        this.poly(pts, P(sgn, 0, 0));
      }
    }
    return this;
  }
}

// ---- street furniture -------------------------------------------------------------------

/** Cast-iron lantern post; the glass goes into `glow`. */
function streetlight(p: Painter, glow: Painter): void {
  p.paint('#2B302E', 0.45, 0);
  p.tube('y', 0, 0.12, 0, 0, 0.17, 0.16, 8);
  p.tube('y', 0.12, 0.55, 0, 0, 0.12, 0.075, 8);
  p.tube('y', 0.55, 3.3, 0, 0, 0.05, 0.045, 8, false);
  p.tube('y', 3.3, 3.4, 0, 0, 0.075, 0.075, 8);
  p.tube('y', 3.4, 3.5, 0, 0, 0.08, 0.13, 6);
  glow.tube('y', 3.5, 3.86, 0, 0, 0.13, 0.19, 6);
  for (let i = 0; i < 6; i++) {
    const t = (i / 6) * Math.PI * 2;
    const [x, z] = [Math.cos(t), Math.sin(t)];
    p.poly([[x * 0.135, 3.5, z * 0.135], [x * 0.195, 3.86, z * 0.195], [x * 0.205, 3.86, z * 0.205], [x * 0.145, 3.5, z * 0.145]], [Math.cos(t + Math.PI / 2), 0, Math.sin(t + Math.PI / 2)]);
  }
  p.tube('y', 3.86, 4.06, 0, 0, 0.26, 0.05, 6);
  p.tube('y', 4.06, 4.16, 0, 0, 0.03, 0.01, 6);
}

/** Kerbside mailbox on a post, its door towards +z (the road). */
function mailbox(p: Painter, colour: string): void {
  p.paint('#6E5A44', 0.8, 0).box(-0.04, 0, -0.04, 0.04, 1.0, 0.04);
  p.box(-0.07, 0.95, -0.2, 0.07, 1.0, 0.2);
  p.paint(colour, 0.35, 0).box(-0.1, 1.0, -0.24, 0.1, 1.12, 0.24, 'py');
  p.tube('z', -0.24, 0.24, 0, 1.12, 0.1, 0.1, 8, true, 0, Math.PI);
  p.paint('#C0302A', 0.4, 0).box(0.1, 1.06, 0.02, 0.115, 1.3, 0.05);
  p.box(0.1, 1.22, 0.05, 0.115, 1.3, 0.18);
}

function hydrant(p: Painter, colour: string): void {
  p.paint(colour, 0.35, 0);
  p.tube('y', 0, 0.06, 0, 0, 0.14, 0.13, 10);
  p.tube('y', 0.06, 0.52, 0, 0, 0.1, 0.095, 10, false);
  p.tube('y', 0.52, 0.56, 0, 0, 0.12, 0.12, 10);
  p.tube('y', 0.56, 0.66, 0, 0, 0.1, 0.04, 10);
  p.tube('y', 0.66, 0.71, 0, 0, 0.025, 0.025, 6);
  p.tube('x', 0.09, 0.19, 0.38, 0, 0.045, 0.045, 8);
  p.tube('x', -0.09, -0.19, 0.38, 0, 0.045, 0.045, 8);
  p.paint('#D9D6CE', 0.3, 1).tube('z', 0.08, 0.18, 0, 0.33, 0.06, 0.06, 8);
}

/** Wheelie bin, lid hinge at the back (-z). */
function bin(p: Painter, colour: string): void {
  p.paint(colour, 0.55, 0);
  p.box(-0.27, 0.06, -0.31, 0.27, 0.98, 0.31);
  p.box(-0.29, 0.98, -0.35, 0.29, 1.03, 0.34);
  p.box(-0.2, 0.86, 0.31, 0.2, 0.94, 0.34);
  p.paint('#232426', 0.7, 0).box(-0.22, 0.92, -0.4, 0.22, 0.96, -0.36);
  for (const s of [-1, 1]) p.tube('x', s * 0.22, s * 0.3, 0.1, -0.26, 0.1, 0.1, 10);
}

/** A white picket fence along an axis-aligned line (posts every ~2 m). */
function pickets(p: Painter, x0: number, z0: number, x1: number, z1: number): void {
  const alongX = Math.abs(x1 - x0) >= Math.abs(z1 - z0);
  const len = alongX ? Math.abs(x1 - x0) : Math.abs(z1 - z0);
  if (len < 0.5) return;
  // Local frame: +x along the fence.
  p.at(x0, z0, alongX ? (x1 > x0 ? 0 : Math.PI) : z1 > z0 ? -Math.PI / 2 : Math.PI / 2);
  p.paint(FENCE_WHITE, 0.6, 0);
  const posts = Math.max(1, Math.round(len / 2));
  for (let i = 0; i <= posts; i++) {
    const x = (len * i) / posts;
    p.box(x - 0.045, 0, -0.045, x + 0.045, 0.98, 0.045, 'ny');
    p.box(x - 0.06, 0.98, -0.06, x + 0.06, 1.02, 0.06, 'ny');
  }
  for (const y of [0.25, 0.7]) p.box(0, y, -0.035, len, y + 0.06, -0.012, 'ny');
  for (let x = 0.09; x < len - 0.05; x += 0.15) {
    if (Math.abs(((x + 1) % (len / posts)) - 1) < 0.07) continue;
    for (const [z, n] of [[0.009, 1], [-0.009, -1]] as const) {
      p.poly([[x - 0.036, 0.05, z], [x + 0.036, 0.05, z], [x + 0.036, 0.84, z], [x, 0.92, z], [x - 0.036, 0.84, z]], [0, 0, n]);
    }
  }
}

/** A rustic rail fence along an axis-aligned line (round-ish posts every ~2.4 m, two split rails). */
function rails(p: Painter, x0: number, z0: number, x1: number, z1: number): void {
  const alongX = Math.abs(x1 - x0) >= Math.abs(z1 - z0);
  const len = alongX ? Math.abs(x1 - x0) : Math.abs(z1 - z0);
  if (len < 0.5) return;
  p.at(x0, z0, alongX ? (x1 > x0 ? 0 : Math.PI) : z1 > z0 ? -Math.PI / 2 : Math.PI / 2);
  p.paint(RAIL_WOOD, 0.9, 0);
  const posts = Math.max(1, Math.round(len / 2.4));
  for (let i = 0; i <= posts; i++) {
    const x = (len * i) / posts;
    p.box(x - 0.06, 0, -0.06, x + 0.06, 1.08, 0.06, 'ny');
  }
  for (const y of [0.42, 0.84]) p.box(0, y, -0.035, len, y + 0.1, 0.035, 'ny');
}

/**
 * Farm fields as ground layers (`WorldLook.fields`): tilled earth, with darker heaped rows under
 * the crops (every other tile across, from the field's second tile; `town.ts` keeps fields an odd
 * number of tiles wide so this holds on lots turned round too). `prefix` names the meshes.
 */
export function fieldLayers(fields: Rect[] | undefined, prefix: string): [string, Rect[], string, number][] {
  if (!fields?.length) return [];
  const rows: Rect[] = [];
  for (const f of fields) for (let x = f.x + 1; x < f.x + f.w - 1; x += 2) rows.push({ x: x + 0.1, z: f.z + 0.5, w: 0.8, d: f.d - 1 });
  const name = (n: string) => (prefix ? `${prefix}${n[0].toUpperCase()}${n.slice(1)}` : n);
  return [
    [name('fields'), fields, 'material.soil', 0.013],
    [name('furrows'), rows, 'material.furrow', 0.0135],
  ];
}

/** Soft elliptical contact shadow (alpha falls off to the rim). */
export function blob(g: Geo, x: number, z: number, rx: number, rz: number, yaw: number, alpha: number, y = 0.02): void {
  const c = Math.cos(yaw);
  const s = Math.sin(yaw);
  const seg = 14;
  const P = (t: number): V3 => {
    const [lx, lz] = [Math.cos(t) * rx, Math.sin(t) * rz];
    return [x + lx * c + lz * s, y, z - lx * s + lz * c];
  };
  for (let i = 0; i < seg; i++) {
    g.poly([[x, y, z], P(((i + 1) / seg) * Math.PI * 2), P((i / seg) * Math.PI * 2)], [0, 1, 0], undefined, [alpha, 0, 0]);
  }
}

// ---- layout ---------------------------------------------------------------------------

/**
 * A plot as its dressing is laid out: in the town's coordinates, or turned a quarter (`turned`;
 * see `localRect`) so that its street runs along x either way.
 */
export interface PlotLayout {
  /** The plot (its rectangle as laid out). */
  plot: PlotInfo;
  /** The plot in the town's own coordinates. */
  info: PlotInfo;
  /** Laid out turned a quarter (its street runs along z in the town). */
  turned: boolean;
  /** `shop`: a public place with walls; `park`: one in the open; `home`: the focus lot (see `TownStreet`). */
  kind: 'home' | 'house' | 'park' | 'shop' | 'empty';
  /** Its kind whichever lot is the focus. */
  base: 'house' | 'park' | 'shop' | 'empty';
  /** +1: the street is on the plot's max-z side; -1: on its min-z side (as laid out). */
  front: 1 | -1;
  /** z of the plot edge on the street (as laid out). */
  edge: number;
  /** Wall bounding box. */
  house: { x0: number; z0: number; x1: number; z1: number } | null;
  /** Centre x of the garden path to the front door. */
  door: number | null;
  /** The garden path to the front door, and the driveway. */
  path: Rect | null;
  drive: Rect | null;
}


/**
 * A placed thing's kind and where and how it stands, hashed: the street draws other lots' things as
 * they stand, so one moved, turned or swapped for another builds its chunk again.
 */
function poseHash(o: ObjectPlacement): number {
  let h = Math.imul(o.x, 73856093) ^ Math.imul(o.z, 19349663) ^ Math.imul(o.rot * 90 + (o.turn ?? 0) + 1, 83492791);
  h ^= Math.imul((o.offset?.[0] ?? 0) + 1000, 40503) ^ Math.imul((o.offset?.[1] ?? 0) + 1000, 2246822519 | 0) ^ Math.imul(Math.round((o.lift ?? 0) * 100), 3266489917 | 0);
  for (let i = 0; i < o.def.length; i++) h = Math.imul(h ^ o.def.charCodeAt(i), 16777619);
  return h;
}

/** What `planStreet` needs to know of the game: its models, and its public places' things. */
export interface StreetSource {
  /** Whether the manifests have model `key` (a missing one is left out). */
  has(key: string): boolean;
  /** Model `key`'s manifest entry. */
  entry(key: string): ModelEntry | undefined;
  /** Neighbours' windows glow only while someone is up in there (`StreetOptions.lived`). */
  lived: boolean;
  /** The outdoor things of the lots outside the viewed one, public places' and neighbours' (`StreetOptions.look`). */
  look?: ObjectLook;
  /** Also the ground layers (`ChunkPlan.ground`: the road, garden paths, fields; the game's street). */
  ground?: boolean;
}

/** The street as laid out, to be drawn. */
export interface StreetPlan {
  /** The neighbours' houses (`HousePlanner.silhouettes`). */
  silhouettes: SilhouettePlan;
  /** Their wall boxes and garages, for what stands round them (x0, z0, x1, z1). */
  specs: SilhouetteSpec[];
  /** Sidewalks and kerbs (the sidewalk surface, vertex-coloured). */
  walk: Geo;
  /** Street furniture, cars, fences, flower beds, awnings (the plain finish). */
  props: Geo;
  /** Streetlights' and porch lamps' glass (unlit, glowing at night). */
  glow: Geo;
  /** Pools of lamplight on the ground (0..1 UVs over a radial gradient, added). */
  pools: Geo;
  /** Soft contact shadows (black, by vertex alpha). */
  shadows: Geo;
  /**
   * The viewed lot's own (the game's): its garage and the garage's coach lamp (its cap, glass and
   * pool of light), apart from the street's meshes so they hide with the lot's ground while its
   * basement is in view. Null when it has no garage.
   */
  lot: { silhouettes: SilhouettePlan; props: Geo; glow: Geo; pools: Geo } | null;
  /** Instanced models in the order they're drawn: the foliage first, then props and things; `cast`: they cast shadows. */
  batches: [key: string, items: Inst[], cast: boolean][];
  /** Other lots' things: their footprints and the boxes they're drawn as without their files, by model key. */
  footprints: Map<string, [number, number]>;
  standIns: Map<string, StandIn>;
}

/** Ground layers a chunk draws under the street (the game's, not the menus' town): the road, garden paths, farm fields. */
export interface GroundLayer {
  name: string;
  material: string;
  geo: Geo;
}

/** One stream chunk of the street (`shared/chunks.ts`), as planned for a focus lot: what `StreetPlan` holds, for its square. */
export interface ChunkPlan extends Omit<StreetPlan, 'batches' | 'footprints' | 'standIns'> {
  id: number;
  /** What it was planned from: a chunk with the same key needn't be built again. */
  key: string;
  /** Instanced models by key (see `StreetPlan.batches`; whether they cast is `castsShadow`). */
  batches: Map<string, Inst[]>;
  /** Road surface, garden paths and fields off the focus lot (game only). */
  ground: GroundLayer[];
}

/** Whether a model the street instances casts the sun's shadows (as `StreetPlan.batches`). */
export const castsShadow = (key: string): boolean => key !== FOLIAGE.bush && key !== FOLIAGE.flowers;

/** What a part of the street writes: the plain-finish props, lamp glass, pools, contact shadows, sidewalks, models' instances. */
class Parts {
  readonly props = new Painter(new Geo());
  readonly glow = new Painter(new Geo());
  readonly pools = new Geo();
  readonly shadows = new Geo().color('#000000');
  readonly walk = new Geo();
  readonly plants: Record<FoliageKind, Inst[]> = { bush: [], flowers: [], hedge: [] };
  readonly things = new Map<string, Inst[]>();
  constructor(private readonly has: (key: string) => boolean) {}
  put(key: string, x: number, z: number, yaw: number, s = 1, sy = s, y = 0): void {
    if (!this.has(key)) return;
    let list = this.things.get(key);
    if (!list) this.things.set(key, (list = []));
    list.push(y ? [x, z, s, sy, yaw, y] : [x, z, s, sy, yaw]);
  }
  /** A hash of what was written (for parts that are planned again and may well come out the same). */
  signature(): number {
    let h = hashNumbers(this.props.geo.pos);
    h = hashNumbers(this.glow.geo.pos, h);
    h = hashNumbers(this.pools.pos, h);
    h = hashNumbers(this.shadows.pos, h);
    for (const kind of Object.keys(this.plants) as FoliageKind[]) h = hashNumbers(this.plants[kind].flat() as number[], h);
    for (const [k, list] of this.things) h = hashNumbers(list.flat() as number[], hashString(k, h));
    return h;
  }
}

/** A plot's share of the street: its house (or garage) and garden, a shop's awning, the things outdoors on it. */
interface PlotOut {
  parts: Parts;
  silhouettes: SilhouettePlan | null;
  spec: SilhouetteSpec | null;
  lot: StreetPlan['lot'];
}

/** A road side's kerbside furniture, by chunk, with what each chunk's share looks like (`Parts.signature`). */
interface KerbOut {
  chunks: Map<number, { parts: Parts; sig: number }>;
  /** Plots whose ground or things the furniture was checked against (see `TownStreet.free`). */
  touched: Set<number>;
  /** Cells of the blocked grid it looked in. */
  cells: Rect[];
}

/** Something on the street that blocks props: a thing's footprint or a garage, and the plot it's on (-1: none). */
interface Blocker extends Rect {
  plot: number;
  /**
   * 'thing': a placed thing (the game draws the focus lot's itself: not in the street's way there);
   * 'legacyGarage': a garage the street placed itself, a game from before garages (none on the
   * focus lot); 'garage': the town's.
   */
  kind: 'thing' | 'legacyGarage' | 'garage';
  hash: number;
}

/** No plot: what the menus' town and a game with no lot in view plan with. */
export const NO_FOCUS = -1;

/** Merges a part into a chunk being assembled. */
function mergeParts(into: ChunkPlan, p: Parts): void {
  into.props.merge(p.props.geo);
  into.glow.merge(p.glow.geo);
  into.pools.merge(p.pools);
  into.shadows.merge(p.shadows);
  into.walk.merge(p.walk);
  for (const kind of Object.keys(FOLIAGE) as FoliageKind[]) {
    if (!p.plants[kind].length) continue;
    const list = into.batches.get(FOLIAGE[kind])!;
    for (const i of p.plants[kind]) list.push(i);
  }
  for (const [k, items] of p.things) {
    let list = into.batches.get(k);
    if (!list) into.batches.set(k, (list = []));
    for (const i of items) list.push(i);
  }
}

/** Silhouette parts (one per material) merged by name. */
function mergeSilhouettes(into: SilhouettePlan, s: SilhouettePlan): void {
  for (const part of s.parts) {
    const same = into.parts.find((p) => p.name === part.name);
    if (same) same.geo.merge(part.geo);
    else into.parts.push({ ...part, geo: new Geo().merge(part.geo) });
  }
  into.homes.push(...s.homes);
  into.lamps.push(...s.lamps);
}

/** A part of `TownStreet.layoutKey`, strung once per object. */
const keys = new WeakMap<object, string>();
function keyOf(of: object, make: () => string): string {
  let k = keys.get(of);
  if (k === undefined) keys.set(of, (k = make()));
  return k;
}

/** Mixes numbers into a 32-bit hash. */
const mix = (h: number, v: number) => Math.imul(h ^ (v | 0), 16777619);

/**
 * The whole town's street, laid out once, view-independent, and planned per 64 m chunk for a focus
 * lot (`chunk`): the focus lot's house and garden are the game's own (only its garage here, apart in
 * `lot`), and the lots beside it keep their fences clear of it. Each part of the street (a plot, a
 * road side's kerb, a field's fence) is planned on its own and kept, planned again only when what it
 * stands among changes (a thing moved near it, the focus next to it), so a focus change or an edit
 * builds one or two chunks again, not the street.
 *
 * `mode` 'town': the menus' town (`WorldMeta.home` the player's lot, every window lit, the world's
 * cars parked, no other lots' things: the overview draws them).
 */
export class TownStreet {
  readonly plots: PlotLayout[];
  private world: WorldStructure;
  private readonly roads: RoadPlan[];
  private readonly index: RoadIndex;
  private readonly paths: Rect[];
  private readonly fields: Rect[];
  private readonly W: number;
  private readonly D: number;
  private readonly byId = new Map<number, PlotLayout>();
  /** Plots by where they are (cells of 16 m). */
  private readonly plotGrid = new RectGrid<Rect & { L: PlotLayout }>(16);
  /** The plots each side of a plot in its row (same street edge, edges meeting). */
  private readonly sides = new Map<PlotLayout, { left?: PlotLayout; right?: PlotLayout }>();
  private readonly fronting: ({ L: PlotLayout; road: number; side: 1 | -1; at: (u: number) => [number, number] } | null)[];
  private readonly meeting: { p: Rect; road: number; side: 1 | -1 }[];
  /** Openings by where they lie on the ground (cells of 8 m). */
  private openingGrid = new RectGrid<Rect & { o: Opening; i: number }>(8);
  private openingsOf: Opening[] | undefined | null = null;
  private objectsOf: ObjectPlacement[] | null = null;
  private roomsOf: ArrayLike<number> | null = null;
  private garagesOf: WorldStructure['garages'] | null = null;
  private blocked = new RectGrid<Blocker>(4);
  private objectsByPlot = new Map<number, ObjectPlacement[]>();
  private garages = new Map<number, SilhouetteSpec['garage']>();
  /** Things on no plot's ground (on the street, on lots not laid out). */
  private readonly plotUnits = new Map<string, { hash: number; out: PlotOut }>();
  private readonly kerbUnits = new Map<string, { hash: number; out: KerbOut }>();
  private readonly fieldUnits = new Map<string, { hash: number; out: Parts }>();
  /** The street's own: sidewalks, the road's surface, fountains, the menus' parked cars (by chunk). */
  private statics: Map<number, { parts: Parts; road: Geo }> | null = null;
  /** Kerbs by road side, and which plots front each. */
  private readonly rows = new Map<string, number[]>();
  readonly footprints = new Map<string, [number, number]>();
  readonly standIns = new Map<string, StandIn>();

  constructor(
    private readonly planner: HousePlanner,
    world: WorldStructure,
    private readonly options: StreetSource,
    readonly mode: 'game' | 'town',
  ) {
    this.world = world;
    const streets = (world.meta?.streets ?? []).filter((s) => s.w >= s.d);
    this.roads = townRoads(world);
    this.index = roadIndex(this.roads);
    this.paths = world.meta?.paths ?? [];
    this.fields = world.meta?.world?.fields ?? [];
    this.W = world.width;
    this.D = groundDepth(world);
    this.plots = layout(world, streets, this.paths);
    for (const L of this.plots) {
      this.byId.set(L.info.id, L);
      this.plotGrid.add({ x: L.info.x, z: L.info.z, w: L.info.w, d: L.info.d, L });
    }
    // Rows of plots along one street edge, in order: who is left and right of whom.
    const rowsOf = new Map<string, PlotLayout[]>();
    for (const L of this.plots) {
      const k = `${L.turned}|${L.edge}`;
      let row = rowsOf.get(k);
      if (!row) rowsOf.set(k, (row = []));
      row.push(L);
    }
    for (const row of rowsOf.values()) {
      row.sort((a, b) => a.plot.x - b.plot.x);
      row.forEach((L, i) => {
        const left = row[i - 1] && Math.abs(row[i - 1].plot.x + row[i - 1].plot.w - L.plot.x) < 0.01 ? row[i - 1] : undefined;
        const right = row[i + 1] && Math.abs(L.plot.x + L.plot.w - row[i + 1].plot.x) < 0.01 ? row[i + 1] : undefined;
        this.sides.set(L, { left, right });
      });
    }
    this.fronting = this.plots.map((L) => {
      const at = (u: number) => townPoint(L.turned, u, L.edge);
      const [cx, cz] = at(L.plot.x + L.plot.w / 2);
      const near = this.index.nearest(cx, cz);
      return near && { L, road: near.road, side: near.side, at };
    });
    this.meeting = this.paths.flatMap((p) => {
      const near = this.index.nearest(p.x + p.w / 2, p.z + p.d / 2);
      return near && near.dist <= this.roads[near.road].width / 2 + Math.max(p.w, p.d) / 2 + 0.6 ? [{ p, road: near.road, side: near.side }] : [];
    });
    // Every road side (its furniture even where no lot fronts it), and the plots fronting it.
    this.roads.forEach((_, r) => {
      for (const side of [1, -1]) this.rows.set(`${r}|${side}`, []);
    });
    this.fronting.forEach((f, i) => {
      if (f) this.rows.get(`${f.road}|${f.side}`)!.push(i);
    });
    this.setWorld(world);
  }

  /**
   * What the street is laid out from (roads, paths, fields, the plots' places and faces, garages),
   * beyond what each part of it keeps track of itself (houses, things, openings): a town with
   * another layout key needs a new `TownStreet`.
   */
  static layoutKey(world: WorldStructure, mode: 'game' | 'town'): string {
    const m = world.meta;
    // (Each part strung once per object: a world message brings the same meta and lists again.)
    return [
      mode,
      world.width,
      world.depth,
      world.storeyDepth,
      world.storeys,
      world.basement,
      m ? keyOf(m, () => JSON.stringify([m.seed ?? 0, m.streets, m.roads, m.paths, m.facing, m.world?.fields, m.home])) : '-',
      world.garages ? keyOf(world.garages, () => JSON.stringify(world.garages)) : '-',
      mode === 'town' ? (world.cars ? keyOf(world.cars, () => JSON.stringify(world.cars)) : '-') : !!world.cars,
      keyOf(world.plots, () => JSON.stringify(world.plots.map((p) => [p.id, p.x, p.z, p.w, p.d, p.public, p.entry]))),
    ].join('|');
  }

  /** Takes the world's things, houses and rooms as they are now (the layout stays: see `layoutKey`). */
  setWorld(world: WorldStructure): void {
    this.world = world;
    // (Rooms decide which things are outdoors: another rooms array, keys worked out again.)
    if (world.rooms !== this.roomsOf) {
      this.roomsOf = world.rooms;
      this.version++;
    }
    // Doors and windows (a window put in on the focus lot): found again by where they are; each
    // plot's share notices those round its house (`plotOut`).
    if (world.openings !== this.openingsOf) {
      this.openingsOf = world.openings;
      const grid = new RectGrid<Rect & { o: Opening; i: number }>(8);
      (world.openings ?? []).forEach((o, i) => {
        // (On the ground's rows: an opening upstairs on the face of the house below it; none of the basement's shows.)
        const k = storeyOfRow(world, o.z);
        if (k < 0) return;
        const z = k > 0 ? o.z - storeyRow(world, k) : o.z;
        grid.add({ x: o.x - 1, z: z - 1, w: 2, d: 2, o, i });
      });
      this.openingGrid = grid;
      this.version++;
    }
    const index = new Map(world.plots.map((p, i) => [p.id, i]));
    let relaid = false;
    const same = (a: PlotInfo, b: PlotInfo) => a.public === b.public && a.storeys === b.storeys && String(a.house) === String(b.house) && String(a.roof) === String(b.roof);
    this.plots.forEach((L) => {
      const i = index.get(L.info.id);
      const info = i === undefined ? undefined : world.plots[i];
      if (!info || info === L.info) return;
      if (same(info, L.info)) {
        L.info = info;
        return;
      }
      // A house built, grown or re-roofed: laid out again (its place and its paths are the layout's).
      const fresh = layout({ ...world, plots: [info] } as WorldStructure, (world.meta?.streets ?? []).filter((s) => s.w >= s.d), this.paths, world.meta?.facing?.[i!])[0];
      if (fresh) Object.assign(L, { info: fresh.info, house: fresh.house, plot: fresh.plot, base: fresh.base, kind: fresh.base });
      relaid = true;
    });
    // (The same things, the same garages: their grid stands.)
    if (!relaid && world.objects === this.objectsOf && world.garages === this.garagesOf) return;
    this.version++;
    this.objectsOf = world.objects;
    this.garagesOf = world.garages;
    const blocked = new RectGrid<Blocker>(4);
    const byPlot = new Map<number, ObjectPlacement[]>();
    for (const o of world.objects) {
      const L = this.plotAtTile(o.x, o.z);
      const plot = L ? L.info.id : -1;
      blocked.add({ x: o.x, z: o.z, w: o.w, d: o.d, plot, kind: 'thing', hash: poseHash(o) });
      let list = byPlot.get(plot);
      if (!list) byPlot.set(plot, (list = []));
      list.push(o);
    }
    this.objectsByPlot = byPlot;
    // Garages: the town's (`WorldStructure.garages`), else (a game from before them) the street places the
    // neighbours' itself, clear of the things (those of a focus lot beside one left out: `legacyGarage`).
    this.blocked = blocked;
    this.legacy.clear();
    const garages = new Map<number, SilhouetteSpec['garage']>();
    for (const L of this.plots) {
      if (!L.house || L.base === 'shop' || L.base === 'park') continue;
      if (world.garages) {
        const g = world.garages.find((g) => g.plot === L.plot.id);
        if (g) garages.set(L.info.id, { x0: g.x0, z0: g.z0, x1: g.x1, z1: g.z1 });
      } else if (L.base === 'house') {
        garages.set(L.info.id, this.legacyGarage(L, NO_FOCUS));
      }
    }
    for (const [plot, g] of garages) {
      if (g) blocked.add({ x: g.x0 - T, z: g.z0 - T, w: g.x1 - g.x0 + 2 * T, d: g.z1 - g.z0 + 2 * T, plot, kind: world.garages ? 'garage' : 'legacyGarage', hash: mix(mix(g.x0 * 100, g.z0 * 100), mix(g.x1 * 100, g.z1 * 100)) });
    }
    this.garages = garages;
  }

  /**
   * The garage a game from before garages gets beside a neighbour's house (`garageFor`), clear of the
   * things round it but those of the focus lot when that is beside it (the game draws its own).
   */
  private legacyGarage(L: PlotLayout, focus: number): SilhouetteSpec['garage'] {
    const variant = this.mode === 'game' && this.plotsNear(L.info, 2).some((n) => n.info.id === focus) ? focus : NO_FOCUS;
    const key = `${L.info.id}|${variant}`;
    const known = this.legacy.get(key);
    if (known !== undefined) return known;
    const things = this.blocked.near(L.info, 4).filter((b) => b.kind === 'thing' && !this.gone(b, variant));
    const g = garage(L, things);
    const out = g && townBox(L.turned, g);
    this.legacy.set(key, out);
    return out;
  }
  private readonly legacy = new Map<string, SilhouetteSpec['garage']>();

  private plotAtTile(x: number, z: number): PlotLayout | undefined {
    for (const { L } of this.plotGrid.near({ x: x + 0.5, z: z + 0.5, w: 0, d: 0 })) {
      const r = L.info;
      if (x >= r.x && z >= r.z && x < r.x + r.w && z < r.z + r.d) return L;
    }
    return undefined;
  }

  /** Whether a blocker is out of the street's way with lot `focus` in view (see `Blocker.kind`). */
  private gone(b: Blocker, focus: number): boolean {
    return b.plot === focus && focus !== NO_FOCUS && (b.kind === 'legacyGarage' || (b.kind === 'thing' && this.mode === 'game'));
  }

  /** A plot's kind with lot `focus` in view (the viewed lot is the game's own: 'home'). */
  private kindOf(L: PlotLayout, focus: number): PlotLayout['kind'] {
    return L.info.id === focus ? 'home' : L.base;
  }

  /** The plot under a ground point, or the nearest within `reach` m (null: none). */
  plotNear(x: number, z: number, reach = 0): PlotInfo | null {
    let best: PlotInfo | null = null;
    let bestD = reach;
    for (const { L } of this.plotGrid.near({ x, z, w: 0, d: 0 }, reach)) {
      const r = L.info;
      const dx = Math.max(r.x - x, 0, x - (r.x + r.w));
      const dz = Math.max(r.z - z, 0, z - (r.z + r.d));
      const d = Math.hypot(dx, dz);
      if (d <= bestD) [best, bestD] = [r, d];
    }
    return best;
  }

  /**
   * Whether a rectangle is clear of the street's blockers (things and garages, those of the focus
   * lot left out in the game, where the game draws its own) and, in the game, of the focus lot.
   * `touched` collects the plots whose ground or blockers it was checked against, `cells` where it looked.
   */
  private free(r: Rect, m: number, focus: number, touched?: Set<number>, cells?: Rect[]): boolean {
    const game = this.mode === 'game';
    if (cells) cells.push({ x: r.x - m, z: r.z - m, w: r.w + 2 * m, d: r.d + 2 * m });
    let clear = true;
    for (const b of this.blocked.near(r, m)) {
      if (!overlaps(r, b, m)) continue;
      if (touched && b.plot >= 0) touched.add(b.plot);
      if (this.gone(b, focus)) continue;
      clear = false;
    }
    if (game) {
      for (const { L } of this.plotGrid.near(r)) {
        if (!overlaps(r, L.info)) continue;
        touched?.add(L.info.id);
        if (L.info.id === focus) clear = false;
      }
    }
    return clear;
  }

  /** The blockers near a rectangle as a list (the garages' check, the fences'), the focus lot's left out in the game. */
  private blockersNear(r: Rect, m: number, focus: number): Rect[] {
    return this.blocked.near(r, m).filter((b) => !this.gone(b, focus));
  }

  /** A hash of the blockers in rectangles (those of the focus lot left out in the game). */
  private blockersHash(rects: readonly Rect[], focus: number, h = 0): number {
    for (const r of rects) {
      for (const b of this.blocked.near(r)) {
        if (this.gone(b, focus)) continue;
        h = mix(h, b.hash);
        h = mix(h, b.w * 31 + b.d);
      }
    }
    return h;
  }

  /** The plots within `m` of a rectangle. */
  private plotsNear(r: Rect, m: number): PlotLayout[] {
    return this.plotGrid.near(r, m).filter(({ L }) => overlaps(r, L.info, m)).map(({ L }) => L);
  }

  /** The chunk a plot's share of the street goes into: where its middle is. */
  private plotChunk(L: PlotLayout): number {
    return chunkAt(L.info.x + L.info.w / 2, L.info.z + L.info.d / 2);
  }

  /** The ground layers' rectangles (the game's): garden paths, fields and their furrows; name, rectangles, material, height. */
  private groundRects(): [string, Rect[], string, number][] {
    return (this.ground ??= [['paths', this.paths, 'material.path', 0.014], ...fieldLayers(this.fields, '')]);
  }
  private ground: [string, Rect[], string, number][] | null = null;

  /** The plots by the chunk their share goes into. */
  private plotsByChunk(): Map<number, PlotLayout[]> {
    if (this.byChunk) return this.byChunk;
    const out = new Map<number, PlotLayout[]>();
    for (const L of this.plots) {
      const id = this.plotChunk(L);
      let list = out.get(id);
      if (!list) out.set(id, (list = []));
      list.push(L);
    }
    return (this.byChunk = out);
  }
  private byChunk: Map<number, PlotLayout[]> | null = null;

  /** Every chunk with something of the street in it. */
  chunkIds(): number[] {
    const ids = new Set<number>();
    for (const L of this.plots) ids.add(this.plotChunk(L));
    for (const id of this.staticParts().keys()) ids.add(id);
    for (const r of this.rows.keys()) for (const id of this.kerbOf(r, NO_FOCUS).chunks.keys()) ids.add(id);
    for (const f of this.fields) ids.add(chunkAt(f.x + f.w / 2, f.z + f.d / 2));
    if (this.options.ground) for (const [, rects] of this.groundRects()) for (const r of rects) ids.add(chunkAt(r.x + r.w / 2, r.z + r.d / 2));
    return [...ids].sort((a, b) => a - b);
  }

  /** The chunk the focus lot's own share (`lot`) and its shell's absence are in. */
  focusChunk(focus: number): number | null {
    const L = this.byId.get(focus);
    return L ? this.plotChunk(L) : null;
  }

  // ---- the parts, each planned on its own and kept ----------------------------------------

  /** The plots a plot's share depends on with the focus there: itself and those beside it (their fences keep clear of it). */
  private plotDeps(L: PlotLayout): PlotLayout[] {
    return this.plotsNear(L.info, 2);
  }

  private plotOut(L: PlotLayout, focus: number): { key: string; out: PlotOut } {
    const deps = this.plotDeps(L);
    const variant = deps.some((d) => d.info.id === focus) ? focus : NO_FOCUS;
    const id = `${L.info.id}|${variant}`;
    // What it's planned from: the plot, its neighbours' houses (their fronts), the things and garages round it, the openings on its house.
    let h = hashString(JSON.stringify([L.info.house, L.info.roof, L.info.storeys, L.info.public, this.sides.get(L)?.left?.info.house, this.sides.get(L)?.right?.info.house]));
    h = this.blockersHash([{ x: L.info.x - 1, z: L.info.z - 1, w: L.info.w + 2, d: L.info.d + 2 }], variant, h);
    if (!this.world.garages && L.base === 'house') {
      const g = this.legacyGarage(L, variant);
      h = mix(h, g ? hashNumbers([g.x0, g.z0, g.x1, g.z1]) : 7);
    }
    if (L.info.house) {
      const [x0, z0, x1, z1] = L.info.house;
      for (const { o } of this.openingGrid.near({ x: x0, z: z0, w: x1 - x0, d: z1 - z0 }, 1)) h = mix(mix(mix(h, o.x), o.z * 7 + (o.axis === 'h' ? 1 : 2)), o.kind.length);
    }
    // Indoors or out: the things on it, by the rooms under them.
    const W = this.W;
    for (const o of this.objectsByPlot.get(L.info.id) ?? []) h = mix(h, this.world.rooms[o.z * W + o.x] ? 1 : 0);
    const cached = this.plotUnits.get(id);
    if (cached && cached.hash === h) return { key: `p${id}:${h}`, out: cached.out };
    const out = this.planPlot(L, variant);
    this.plotUnits.set(id, { hash: h, out });
    return { key: `p${id}:${h}`, out };
  }

  /** A plot's share of the street with lot `focus` in view (see the rules at the top). */
  private planPlot(L: PlotLayout, focus: number): PlotOut {
    const world = this.world;
    const game = this.mode === 'game';
    const parts = new Parts(this.options.has);
    const kind = this.kindOf(L, focus);
    const home = kind === 'home';
    let silhouettes: SilhouettePlan | null = null;
    let spec: SilhouetteSpec | null = null;
    let lot: StreetPlan['lot'] = null;

    // ---- its house (the player's in the menus' town a shell; the viewed lot's only its garage) ----
    if ((kind === 'house' || kind === 'shop' || home) && L.house) {
      const g = kind === 'shop' ? null : world.garages ? (this.garages.get(L.info.id) ?? null) : home ? null : this.legacyGarage(L, focus);
      if (!(home && game && !g)) {
        const [x0, z0, x1, z1] = L.info.house!;
        const front: [number, number] = L.turned ? [-L.front, 0] : [0, L.front];
        spec = { x0, z0, x1, z1, lot: L.info, home, garageOnly: home && game, front, door: L.door, garage: g, roof: L.plot.roof, storeys: L.plot.storeys, lived: kind === 'house' && this.options.lived };
        // The openings on and round its walls, in the town's order (upstairs ones found on the ground's rows).
        const openings = this.openingGrid
          .near({ x: x0, z: z0, w: x1 - x0, d: z1 - z0 }, 2)
          .sort((a, b) => a.i - b.i)
          .map((e) => e.o);
        const plan = this.planner.silhouettes([spec], openings, world);
        if (spec.garageOnly) {
          lot = { silhouettes: plan, props: new Geo(), glow: new Geo(), pools: new Geo() };
          lamps(plan.lamps, new Painter(lot.props), new Painter(lot.glow), lot.pools);
        } else {
          silhouettes = plan;
          lamps(plan.lamps, parts.props, parts.glow, parts.pools);
        }
      }
    }

    // ---- its garden: a car, fences or hedges, flower beds ----
    if (kind === 'house' && L.house) this.garden(L, focus, parts);

    // ---- a shop's awning; the things outdoors on it ----
    if (kind === 'shop' && L.house) awning(L, parts);
    if (game && this.options.look && !home) {
      const W = this.W;
      const D = this.D;
      for (const o of this.objectsByPlot.get(L.info.id) ?? []) {
        if (o.z + o.d > D) continue;
        let indoors = false;
        for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) indoors ||= !!world.rooms[z * W + x];
        const look = this.options.look(o.def);
        if (indoors || !look) continue;
        this.footprints.set(look.model, look.footprint);
        this.standIns.set(look.model, look.standIn);
        const c = objectCentre(o);
        // (As `lotContents.ts` places it: its model's own variation by tile, and a thing on a top on it.)
        const { turn, size } = placementVariation(this.options.entry(look.model)?.vary ?? 0, o.x, o.z);
        parts.put(look.model, c.x, c.z, (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn, size, size, o.lift ?? 0);
      }
    }
    return { parts, silhouettes, spec, lot };
  }

  /** A neighbour's garden (see the rules at the top), with lot `focus` in view. */
  private garden(L: PlotLayout, focus: number, parts: Parts): void {
    const world = this.world;
    const { plot, front, turned } = L;
    const h = L.house!;
    const props = parts.props;
    const neighbour = (o: PlotLayout | undefined) => !!o && this.kindOf(o, focus) === 'house';
    const free = (r: Rect, m = 0) => this.free(r, m, focus);
    const near = this.blockersNear({ x: L.info.x - 1, z: L.info.z - 1, w: L.info.w + 2, d: L.info.d + 2 }, 0.5, focus);
    // Laid out as the plot is (see `PlotLayout`), drawn on the town.
    const at = (x: number, z: number, yaw = 0, y = 0) => props.at(...townPoint(turned, x, z), townYaw(turned, yaw), y);
    const isFree = (r: Rect, m = 0) => free(townRect(turned, r), m);
    const plant = (kind: FoliageKind, [x, z, sx, sy, yaw]: Inst) => parts.plants[kind].push([...townPoint(turned, x, z), sx, sy, townYaw(turned, yaw)]);
    const putAt = (key: string, x: number, z: number, yaw: number, size?: number) => parts.put(key, ...townPoint(turned, x, z), townYaw(turned, yaw), size);
    const vz = (v: number) => L.edge - front * v; // depth into the plot -> z
    const frontWall = front > 0 ? h.z1 + T : h.z0 - T;
    const backEdge = front > 0 ? plot.z : plot.z + plot.d;

    // A game without cars of its own (none in `WorldStructure.cars`: towns of winding roads, whose
    // cars stay parked for now, and games from before cars): a car on some driveways, for good.
    const end = !world.cars && L.drive ? { x: L.drive.x + L.drive.w / 2, z: front > 0 ? L.drive.z : L.drive.z + L.drive.d, front, street: 0 } : null;
    const car = end ? parkCar(plot, end) : null;
    if (car) parked(parts, ...townPoint(turned, car.x, car.z), townYaw(turned, car.yaw), CARS[car.body], CAR_PAINT[car.paint]);

    // Fences or hedges: sides from the house front line back, and the back edge. A side
    // shared with another neighbour is drawn once, on the boundary, by the plot on its left.
    // Farms have rail fences.
    const farm = this.fields.some((f) => overlaps(f, L.info, -0.01));
    const style = farm ? 'rail' : hash(plot.x, plot.z, 30) < 0.5 ? 'picket' : hash(plot.x, plot.z, 31) < 0.75 ? 'hedge' : 'none';
    if (style !== 'none') {
      const { left, right } = this.sides.get(L) ?? {};
      const inset = style === 'hedge' ? 0.45 : 0.15;
      const vStart = Math.max(L.edge * front - frontWall * front, ...[left, right].filter(neighbour).map((o) => (o!.house ? o!.edge * front - (front > 0 ? o!.house.z1 + T : o!.house.z0 - T) * front : 0))) + 0.4;
      const vEnd = plot.d - inset;
      const lines: [number, number, number, number][] = [];
      // Further in beside the viewed lot (see `LOT_CLEARANCE`).
      const homeAt = (o: PlotLayout | undefined) => !!o && this.kindOf(o, focus) === 'home';
      const [inL, inR] = [homeAt(left) ? LOT_CLEARANCE[style] : inset, homeAt(right) ? LOT_CLEARANCE[style] : inset];
      if (!neighbour(left)) lines.push([plot.x + inL, vz(vStart), plot.x + inL, vz(vEnd)]);
      lines.push(neighbour(right) ? [plot.x + plot.w, vz(vStart), plot.x + plot.w, vz(vEnd)] : [plot.x + plot.w - inR, vz(vStart), plot.x + plot.w - inR, vz(vEnd)]);
      lines.push([plot.x + inL, backEdge + front * inset, plot.x + plot.w - inR, backEdge + front * inset]);
      for (const [x0, z0, x1, z1] of lines) edgeRun(style, props, parts.plants.hedge, ...townPoint(turned, x0, z0), ...townPoint(turned, x1, z1), near);
    }

    // Flower beds along the house front, either side of the porch (one along all of it when no path
    // leads to a door).
    const door = L.door;
    const depth = 0.85;
    const bedRect = (a: number, b: number): Rect => ({ x: a, z: Math.min(frontWall, frontWall + front * depth), w: b - a, d: depth });
    const runs: [number, number][] =
      door === null
        ? [[h.x0 + 0.1, h.x1 - 0.1]]
        : [
            [h.x0 + 0.1, door - 1.3],
            [door + 1.3, h.x1 - 0.1],
          ];
    for (const [a0, a1] of runs) {
      if (a1 - a0 < 1.2) continue;
      const r = bedRect(a0, a1);
      if (!isFree(r, 0.1)) continue;
      const [z0, z1] = [r.z, r.z + r.d];
      at(0, 0).paint('#3E2C20', 0.95, 0).box(a0, 0, z0, a1, 0.045, z1, 'ny');
      const ez = front > 0 ? z1 : z0;
      at(0, 0).paint('#B5AEA2', 0.85, 0).box(a0 - 0.04, 0, Math.min(ez, ez + front * 0.08), a1 + 0.04, 0.08, Math.max(ez, ez + front * 0.08), 'ny');
      for (let x = a0 + 0.35; x < a1 - 0.2; x += 0.62) {
        const k = hash(x, plot.z, 40);
        const back = frontWall + front * 0.3;
        const fore = frontWall + front * 0.62;
        if (k < 0.45) plant('bush', [x, back, 0.42 + hash(x, 1, 41) * 0.18, 0.4 + hash(x, 2, 41) * 0.25, hash(x, 3, 42) * 6.28]);
        plant('flowers', [x + 0.15, k < 0.45 ? fore : (back + fore) / 2, 0.85 + hash(x, 4, 43) * 0.35, 0.8 + hash(x, 5, 43) * 0.4, hash(x, 6, 44) * 6.28]);
      }
    }
    // A shrub at each front corner of the house.
    for (const cx of [h.x0 - 0.55, h.x1 + 0.55]) {
      const cz = frontWall + front * 0.2;
      const r = { x: cx - 0.5, z: cz - 0.5, w: 1, d: 1 };
      if (isFree(r) && !(L.drive && overlaps(r, L.drive, 0.2))) plant('bush', [cx, cz, 0.7, 0.75, hash(cx, cz, 45) * 6.28]);
    }
    // Pots either side of the front door, and on some plots little lights along the garden path.
    if (L.door !== null && hash(plot.x, plot.z, 46) < 0.8) {
      for (const dx of [-0.85, 0.85]) {
        const [px, pz] = [L.door + dx, frontWall + front * 0.32];
        if (isFree({ x: px - 0.2, z: pz - 0.2, w: 0.4, d: 0.4 })) putAt(PROP.pot, px, pz, hash(px, pz, 47) * 6.28, 0.85 + hash(px, pz, 48) * 0.3);
      }
    }
    const path = L.path ?? undefined;
    if (path && hash(plot.x, plot.z, 49) < 0.55) {
      const [a, b] = [Math.min(L.edge, frontWall) + 1.0, Math.max(L.edge, frontWall) - 1.0];
      for (let z = a, i = 0; z <= b; z += 2.2, i++) {
        const px = i % 2 ? path.x - 0.22 : path.x + path.w + 0.22;
        if (isFree({ x: px - 0.1, z: z - 0.1, w: 0.2, d: 0.2 })) putAt(PROP.pathLight, px, z, 0);
      }
    }
  }

  /** A road side's kerbside furniture with lot `focus` in view: planned again only when the focus is on its row or by what it checked. */
  private kerbOf(rowKey: string, focus: number): KerbOut {
    const row = this.rows.get(rowKey) ?? [];
    // (What the lots along it are: a house built on an empty lot gets its mailbox.)
    const rowHash = hashString(row.map((i) => this.fronting[i]!.L.base).join());
    const unit = (variant: number): KerbOut => {
      const id = `${rowKey}|${variant}`;
      const cached = this.kerbUnits.get(id);
      const hashOf = (out: KerbOut) => this.blockersHash(out.cells, variant, rowHash);
      if (cached && cached.hash === hashOf(cached.out)) return cached.out;
      const [r, side] = rowKey.split('|').map(Number) as [number, 1 | -1];
      const out = this.planKerb(r, side, row, variant);
      this.kerbUnits.set(id, { hash: hashOf(out), out });
      return out;
    };
    // Planned for no focus first: what it checks says which focus lots it depends on.
    const plain = unit(NO_FOCUS);
    if (focus === NO_FOCUS) return plain;
    const onRow = row.some((i) => this.fronting[i]!.L.info.id === focus);
    return onRow || plain.touched.has(focus) ? unit(focus) : plain;
  }

  private planKerb(r: number, side: 1 | -1, rowIds: number[], focus: number): KerbOut {
    const world = this.world;
    const road = this.roads[r];
    const index = this.index;
    const out: KerbOut = { chunks: new Map(), touched: new Set(), cells: [] };
    const at = (x: number, z: number): Parts => {
      const id = chunkAt(x, z);
      let c = out.chunks.get(id);
      if (!c) out.chunks.set(id, (c = { parts: new Parts(this.options.has), sig: 0 }));
      return c.parts;
    };
    const free = (r: Rect, m = 0) => this.free(r, m, focus, out.touched, out.cells);
    const kerb = new Kerb(road, r, side, index);
    const along = (u: [number, number]) => kerb.along(u[0], u[1]);
    // The plots fronting this side, in order along it: their fronts (s0..s1), front doors and driveways.
    const row = rowIds
      .map((i) => {
        const { L, at } = this.fronting[i]!;
        const [a, b] = [along(at(L.plot.x)), along(at(L.plot.x + L.plot.w))];
        const drive = L.drive && [along(at(L.drive.x)), along(at(L.drive.x + L.drive.w))].sort((p, q) => p - q);
        return { L, kind: this.kindOf(L, focus), s0: Math.min(a, b), s1: Math.max(a, b), door: L.door === null ? null : along(at(L.door)), drive };
      })
      .sort((a, b) => a.s0 - b.s0);
    // Mouths of garden paths and driveways on this side: keep them clear.
    const mouths: [number, number][] = [];
    for (const { p } of this.meeting.filter((m) => m.road === r && m.side === side)) {
      const corners = [[p.x, p.z], [p.x + p.w, p.z], [p.x, p.z + p.d], [p.x + p.w, p.z + p.d]].map(([x, z]) => kerb.along(x, z));
      mouths.push([Math.min(...corners), Math.max(...corners)]);
    }
    const clear = (s: number, m: number) => s > 1 && s < kerb.length - 1 && kerb.open(s) && !mouths.some(([a, b]) => s > a - m && s < b + m);
    const used: number[] = [];
    const near = (s: number, m: number) => used.some((u) => Math.abs(u - s) < m);
    const kerbD = SIDEWALK - KERB - 0.3;
    const on = (s: number, d = kerbD) => kerb.at(s, d);

    // Streetlights, one per plot width, staggered between the two sides.
    const lights = row.map((f) => f.s0 + (side > 0 ? 1.2 : (f.s1 - f.s0) / 2));
    if (row.length) lights.push(row[row.length - 1].s1 + (side > 0 ? 1.2 : -1.2));
    for (const s0 of lights) {
      const s = [0, 1.5, -1.5, 3, -3].map((d) => s0 + d).find((s) => clear(s, 1.2) && !near(s, 1.5));
      if (s === undefined) continue;
      used.push(s);
      const [x, z] = on(s);
      const P = at(x, z);
      streetlight(P.props.at(x, z, kerb.yaw(s), WALK_Y), P.glow.at(x, z, kerb.yaw(s), WALK_Y));
      blob(P.shadows, x, z, 0.32, 0.32, 0, 0.45, WALK_Y + 0.004);
      pool(P.pools, ...on(s, kerbD + 0.5), 4.2);
    }
    // Mailboxes in front of every house (beside the path, away from the driveway).
    for (const f of row) {
      if (f.door === null || (f.kind !== 'house' && f.kind !== 'home')) continue;
      const away = f.drive && (f.drive[0] + f.drive[1]) / 2 > f.door ? -1 : 1;
      const s = [0.95 * away, -0.95 * away, 1.6 * away].map((d) => f.door! + d).find((s) => clear(s, 0.25) && !near(s, 0.6));
      if (s === undefined) continue;
      used.push(s);
      const [x, z] = on(s);
      const P = at(x, z);
      // (None of them quite straight.)
      mailbox(P.props.at(x, z, kerb.yaw(s) + (hash(x, z, 9) - 0.5) * 0.5, WALK_Y), pick(MAILBOX, f.L.plot.x, f.L.plot.z, 3));
      blob(P.shadows, x, z, 0.2, 0.3, kerb.yaw(s), 0.35, WALK_Y + 0.004);
    }
    // A hydrant every other plot.
    row.forEach((f, i) => {
      if ((i + (side > 0 ? 0 : 1)) % 2) return;
      const s = [6, 8, 4, 10].map((d) => f.s0 + d).find((s) => s < f.s1 && clear(s, 0.8) && !near(s, 1.2));
      if (s === undefined) return;
      used.push(s);
      const [x, z] = on(s);
      const P = at(x, z);
      hydrant(P.props.at(x, z, kerb.yaw(s) + hash(x, z, 5) * 0.6, WALK_Y), hash(x, z, 4) < 0.75 ? '#B32B24' : '#D9A520');
      blob(P.shadows, x, z, 0.22, 0.22, 0, 0.4, WALK_Y + 0.004);
    });
    // Bins out by some driveways, on the garden side of the sidewalk.
    for (const f of row) {
      const L = f.L;
      if ((f.kind !== 'house' && f.kind !== 'home') || !f.drive || hash(L.plot.x, L.plot.z, 12) > 0.6) continue;
      const outer = (f.drive[0] + f.drive[1]) / 2 > (f.s0 + f.s1) / 2 ? 1 : -1;
      for (let k = 0; k < (hash(L.plot.x, L.plot.z, 13) < 0.5 ? 2 : 1); k++) {
        const s = (outer > 0 ? f.drive[1] + 0.45 : f.drive[0] - 0.45) + outer * k * 0.7;
        if (!clear(s, 0.35) || near(s, 0.5) || s > f.s1 - 0.3 || s < f.s0 + 0.3) continue;
        used.push(s);
        const [x, z] = on(s, 0.4);
        const P = at(x, z);
        bin(P.props.at(x, z, kerb.yaw(s) + Math.PI + (hash(x, z, 6) - 0.5) * 0.3, WALK_Y), BIN[(k + Math.floor(hash(L.plot.x, 1, 14) * 3)) % BIN.length]);
        blob(P.shadows, x, z, 0.36, 0.4, 0, 0.4, WALK_Y + 0.004);
      }
    }
    // In front of the public places: planters with a little tree, benches facing the place
    // (backs to the road), a litter bin by each bench.
    for (const f of row) {
      if (f.kind !== 'park' && f.kind !== 'shop') continue;
      let k = Math.floor(hash(f.L.plot.x, f.L.plot.z, 70) * 2);
      for (let s0 = f.s0 + 2.2; s0 < f.s1 - 2; s0 += 3.6) {
        const bench = k++ % 2 === 1;
        const s = [0, 0.6, -0.6].map((d) => s0 + d).find((s) => clear(s, bench ? 1.3 : 0.8) && !near(s, bench ? 1.5 : 1.0));
        if (s === undefined) continue;
        used.push(s);
        const [x, z] = on(s);
        const P = at(x, z);
        const yaw = kerb.yaw(s);
        if (bench) {
          P.put(PROP.bench, x, z, yaw + Math.PI);
          blob(P.shadows, x, z, 0.95, 0.3, yaw, 0.35, WALK_Y + 0.004);
          const bs = s + (hash(x, z, 71) < 0.5 ? -1.2 : 1.2);
          if (clear(bs, 0.3) && !near(bs, 0.5)) {
            used.push(bs);
            P.put(PROP.litterBin, ...on(bs), kerb.yaw(bs) + (hash(x, z, 72) - 0.5) * 0.5, 0.9);
          }
        } else {
          P.put(hash(x, z, 73) < 0.75 ? PROP.planter : PROP.flowerTub, x, z, hash(x, z, 74) * 6.28, 0.9 + hash(x, z, 75) * 0.15);
          blob(P.shadows, x, z, 0.5, 0.5, 0, 0.4, WALK_Y + 0.004);
        }
      }
    }
    // At the ends of the row (past the last lots): a bus shelter facing the road, its stop sign
    // at the kerb, and a street-name sign on the corner.
    if (row.length) {
      const first = side > 0;
      const corner = first ? row[0].s0 : row[row.length - 1].s1;
      const outward = first ? -1 : 1;
      const cs = corner + outward * 4;
      const [cx, cz] = kerb.at(cs, -0.62);
      const shelter = { x: cx - 1.6, z: cz - 1.6, w: 3.2, d: 3.2 };
      if (clear(cs, 1.6) && free(shelter, 0.2) && !index.onRoad(cx, cz, 1)) {
        const P = at(cx, cz);
        P.put(PROP.shelter, cx, cz, kerb.yaw(cs));
        blob(P.shadows, cx, cz, 1.7, 0.6, kerb.yaw(cs), 0.35, 0.02);
        const ss = cs - outward * 2.1;
        if (clear(ss, 0.3) && !near(ss, 0.6)) {
          used.push(ss);
          const [x, z] = on(ss);
          at(x, z).put(PROP.busSign, x, z, kerb.yaw(ss));
        }
      }
      // Street-name signs at both ends, on one side of the road.
      const ends = first ? [row[0].s0 - 0.9, row[row.length - 1].s1 + 0.9] : [];
      for (const s of ends) {
        if (!clear(s, 0.3) || near(s, 0.8)) continue;
        used.push(s);
        const [x, z] = on(s);
        at(x, z).put(PROP.sign, x, z, kerb.yaw(s) + Math.PI / 4);
      }
    }
    // Where nobody lives along the road: on the water's side (the side its lots aren't on) a
    // promenade (benches looking out over the water, a lamp between each two), elsewhere now and
    // then a bench with a view.
    const empty = (s: number) => !row.some((f) => s > f.s0 - 6 && s < f.s1 + 6);
    const onVerge = (x: number, z: number) => x > 1 && z > 1 && x < world.width - 1 && z < this.D - 1 && !index.onRoad(x, z, 0.4, r) && free({ x: x - 1, z: z - 1, w: 2, d: 2 }, 0.2);
    const bench = (s: number, d: number) => {
      const [x, z] = kerb.at(s, d);
      if (!onVerge(x, z)) return false;
      used.push(s);
      const P = at(x, z);
      P.put(PROP.bench, x, z, kerb.yaw(s) + Math.PI);
      blob(P.shadows, x, z, 0.95, 0.3, kerb.yaw(s), 0.35, WALK_Y + 0.004);
      return true;
    };
    if (road.lots && side !== road.lots) {
      for (let s = 6, k = 0; s < kerb.length - 6; s += 7, k++) {
        if (!empty(s) || !clear(s, 1.4) || near(s, 2)) continue;
        if (k % 2 === 0) bench(s, -1.1);
        else {
          const [x, z] = on(s, 0.35);
          used.push(s);
          const P = at(x, z);
          streetlight(P.props.at(x, z, kerb.yaw(s) + Math.PI, WALK_Y), P.glow.at(x, z, kerb.yaw(s) + Math.PI, WALK_Y));
          pool(P.pools, ...on(s, -0.6), 4.2);
        }
      }
    } else {
      for (let s = 12, run = 0; s < kerb.length - 12; s++) {
        run = empty(s) && kerb.open(s) ? run + 1 : 0;
        if (run < 40 || hash(s, r * 7 + side, 81) > 0.6) continue;
        if (bench(s - 20, -1.3)) run = -40;
      }
    }
    for (const c of out.chunks.values()) c.sig = c.parts.signature();
    return out;
  }

  /** A farm field's rail fence round it, a gate on the side towards the farmhouse (none round a field on the viewed lot). */
  private fieldOut(i: number, focus: number): { key: string; out: Parts } {
    const f = this.fields[i];
    const deps = this.plotsNear(f, 1);
    const variant = deps.some((d) => d.info.id === focus) ? focus : NO_FOCUS;
    const id = `${i}|${variant}`;
    const h = this.blockersHash([{ x: f.x - 1, z: f.z - 1, w: f.w + 2, d: f.d + 2 }], variant, hashString(JSON.stringify(deps.map((d) => d.info.house))));
    const cached = this.fieldUnits.get(id);
    if (cached && cached.hash === h) return { key: `f${id}:${h}`, out: cached.out };
    const parts = new Parts(this.options.has);
    const F = this.byId.get(variant);
    if (!(this.mode === 'game' && F && overlaps(f, F.info, -0.01))) {
      const house = this.plots.find((L) => L.info.house && overlaps(f, L.info, -0.01))?.info.house;
      const gateWest = !house || (house[0] + house[2]) / 2 < f.x + f.w / 2;
      const [x0, z0, x1, z1] = [f.x + 0.3, f.z + 0.3, f.x + f.w - 0.3, f.z + f.d - 0.3];
      const [gx, ox] = gateWest ? [x0, x1] : [x1, x0];
      const gate = (z0 + z1) / 2;
      const near = this.blockersNear({ x: f.x - 1, z: f.z - 1, w: f.w + 2, d: f.d + 2 }, 0.5, variant);
      for (const [a, b, c, d] of [
        [x0, z0, x1, z0],
        [x0, z1, x1, z1],
        [ox, z0, ox, z1],
        [gx, z0, gx, gate - 1.3],
        [gx, gate + 1.3, gx, z1],
      ] as const) {
        edgeRun('rail', parts.props, parts.plants.hedge, a, b, c, d, near);
      }
    }
    this.fieldUnits.set(id, { hash: h, out: parts });
    return { key: `f${id}:${h}`, out: parts };
  }

  /** Sidewalks and kerbs, the road's surface (the game's), the fountains on roundabouts, the menus' parked cars: by chunk, planned once. */
  private staticParts(): Map<number, { parts: Parts; road: Geo }> {
    if (this.statics) return this.statics;
    const out = new Map<number, { parts: Parts; road: Geo }>();
    const at = (x: number, z: number) => {
      const id = chunkAt(x, z);
      let c = out.get(id);
      if (!c) out.set(id, (c = { parts: new Parts(this.options.has), road: new Geo() }));
      return c;
    };
    // Sidewalks, kerbs and the road, each piece in the chunk its middle is in.
    const walk = new RoutedGeo((x, z) => at(x, z).parts.walk);
    const shadows = new RoutedGeo((x, z) => at(x, z).parts.shadows).color('#000000');
    roadMeshes(this.roads, null, 0, { walk, shadows }, 'sidewalks');
    if (this.options.ground) roadMeshes(this.roads, null, 0, { surface: new RoutedGeo((x, z) => at(x, z).road) }, 'surface');
    // A fountain in the middle of every roundabout, flowers round it.
    for (const road of this.roads) {
      if (road.island !== 'fountain') continue;
      const [cx, cz] = centroid(road.points);
      const P = at(cx, cz).parts;
      P.put(PROP.fountain, cx, cz, hash(cx, cz, 90) * 6.28, 0.85);
      for (let k = 0; k < 10; k++) {
        const a = (k / 10) * Math.PI * 2;
        P.plants.flowers.push([cx + Math.cos(a) * 2.6, cz + Math.sin(a) * 2.6, 0.9 + hash(cx, k, 91) * 0.3, 0.85 + hash(cz, k, 92) * 0.3, a]);
      }
    }
    // The world's own cars, parked, in the menus' town (the game draws them coming and going: `pc/cars.ts`).
    if (this.world.cars && this.mode === 'town') {
      for (const c of this.world.cars) parked(at(c.x, c.z).parts, c.x, c.z, c.yaw, CARS[c.body % CARS.length], CAR_PAINT[c.paint % CAR_PAINT.length]);
    }
    this.statics = out;
    return out;
  }

  /**
   * Chunk `id` far off (the far LOD, past the chunks streamed in): its houses as boxes with their
   * roofs (`HousePlanner.farHouse`), the focus lot's left out, in one plain-finish geometry, with
   * what it was made from. Null when it has none.
   */
  farChunk(id: number, focus: number): { key: string; geo: Geo } | null {
    const houses = (this.plotsByChunk().get(id) ?? []).filter((L) => L.info.house && L.info.id !== focus && (L.base === 'house' || L.base === 'shop'));
    if (!houses.length) return null;
    const key = `${id}|${houses.map((L) => `${L.info.id}:${L.info.house!.join(',')}:${L.info.roof?.join(',') ?? ''}:${L.info.storeys ?? 1}`).join(';')}`;
    const cached = this.far.get(id);
    if (cached?.key === key) return cached;
    const geo = new Geo();
    for (const L of houses) this.planner.farHouse(geo, { x: L.info.x, z: L.info.z, house: L.info.house!, roof: L.info.roof, storeys: L.info.storeys });
    const out = { key, geo };
    this.far.set(id, out);
    return out;
  }
  private readonly far = new Map<number, { key: string; geo: Geo }>();

  /** What chunk `id` is planned from with lot `focus` in view (`ChunkPlan.key`), without planning it. */
  chunkKey(id: number, focus: number): string {
    // (Kept until the world changes what the street stands among: `setWorld` moves `version` on.)
    const memo = `${id}|${focus}`;
    const known = this.keys.get(memo);
    if (known && known.version === this.version) return known.key;
    const key = this.gather(id, focus).key;
    this.keys.set(memo, { version: this.version, key });
    return key;
  }
  private readonly keys = new Map<string, { version: number; key: string }>();
  /** Moved on whenever `setWorld` takes a world that changes anything the street is planned from. */
  private version = 0;

  private gather(id: number, focus: number): { key: string; plots: PlotOut[]; kerbs: Parts[]; fields: Parts[]; ground: GroundLayer[] } {
    const keys: string[] = [];
    const plots: PlotOut[] = [];
    for (const L of this.plotsByChunk().get(id) ?? []) {
      const { key, out } = this.plotOut(L, focus);
      keys.push(key);
      plots.push(out);
    }
    const kerbs: Parts[] = [];
    for (const r of this.rows.keys()) {
      const c = this.kerbOf(r, focus).chunks.get(id);
      if (!c) continue;
      keys.push(`k${r}:${c.sig}`);
      kerbs.push(c.parts);
    }
    const fields: Parts[] = [];
    this.fields.forEach((f, i) => {
      if (chunkAt(f.x + f.w / 2, f.z + f.d / 2) !== id) return;
      const { key, out } = this.fieldOut(i, focus);
      keys.push(key);
      fields.push(out);
    });
    // Ground layers: garden paths and fields off the focus lot (the lot draws its own, on its ground).
    const ground: GroundLayer[] = [];
    if (this.options.ground) {
      const F = this.mode === 'game' ? this.byId.get(focus)?.info : undefined;
      const onLot = (r: Rect) => !!F && r.x >= F.x && r.z >= F.z && r.x + r.w <= F.x + F.w && r.z + r.d <= F.z + F.d;
      const mine = (r: Rect) => chunkAt(r.x + r.w / 2, r.z + r.d / 2) === id && !onLot(r);
      const road = this.staticParts().get(id)?.road;
      if (road && !road.empty) ground.push({ name: 'streets', material: 'material.road', geo: road });
      for (const [name, all, material, y] of this.groundRects()) {
        const list = all.filter(mine);
        if (!list.length) continue;
        const geo = new Geo();
        for (const r of list) geo.poly([[r.x, y, r.z], [r.x + r.w, y, r.z], [r.x + r.w, y, r.z + r.d], [r.x, y, r.z + r.d]], [0, 1, 0], [[0, 0], [1, 0], [1, 1], [0, 1]]);
        ground.push({ name, material, geo });
        keys.push(`${name}:${list.length}:${hashNumbers(list.flatMap((r) => [r.x, r.z, r.w, r.d]))}`);
      }
    }
    return { key: `${id}|${this.options.lived}|${keys.join(';')}`, plots, kerbs, fields, ground };
  }

  /** Plans chunk `id` with lot `focus` in view (`NO_FOCUS`: none; in the menus' town the player's lot is the focus). */
  chunk(id: number, focus: number): ChunkPlan {
    const g = this.gather(id, focus);
    const plan: ChunkPlan = {
      id,
      key: g.key,
      silhouettes: { parts: [], homes: [], lamps: [] },
      specs: [],
      walk: new Geo(),
      props: new Geo(),
      glow: new Geo(),
      pools: new Geo(),
      shadows: new Geo(),
      lot: null,
      batches: new Map((Object.keys(FOLIAGE) as FoliageKind[]).map((k) => [FOLIAGE[k], []])),
      ground: g.ground,
    };
    const st = this.staticParts().get(id);
    if (st) mergeParts(plan, st.parts);
    for (const p of g.plots) {
      if (p.spec) plan.specs.push(p.spec);
      if (p.silhouettes) mergeSilhouettes(plan.silhouettes, p.silhouettes);
      if (p.lot) plan.lot = p.lot;
      mergeParts(plan, p.parts);
    }
    for (const p of g.kerbs) mergeParts(plan, p);
    for (const p of g.fields) mergeParts(plan, p);
    return plan;
  }
}


/** Geometry that goes into the chunk each polygon's middle is in. */
class RoutedGeo extends Geo {
  constructor(private readonly target: (x: number, z: number) => Geo) {
    super();
  }
  override poly(points: readonly V3[], n: V3, uvs?: readonly (readonly [number, number])[], alphas?: readonly number[]): this {
    let x = 0;
    let z = 0;
    for (const p of points) {
      x += p[0];
      z += p[2];
    }
    const g = this.target(x / points.length, z / points.length);
    g.r = this.r;
    g.g = this.g;
    g.b = this.b;
    g.a = this.a;
    g.kind = this.kind;
    g.maskA = this.maskA;
    g.maskB = this.maskB;
    g.poly(points, n, uvs, alphas);
    return this;
  }
}

/** A parked car on a driveway, its contact shadow. */
function parked(parts: Parts, x: number, z: number, yaw: number, body: CarBody, paint: string): void {
  paintCar(parts.props.at(x, z, yaw, 0.015), body, paint);
  blob(parts.shadows, x, z, body.wid * 0.62, body.len * 0.56, yaw, 0.55, 0.03);
}

/** Porch and coach lamps: a cap, the glass, a pool of light under it. */
function lamps(list: SilhouettePlan['lamps'], props: Painter, glow: Painter, pools: Geo): void {
  for (const [x, y, z, nx, nz] of list) {
    glow.at(x, z).box(-0.06 + nx * 0.02, y - 0.12, -0.06 + nz * 0.02, 0.06 + nx * 0.02, y + 0.08, 0.06 + nz * 0.02);
    props.at(x, z).paint('#26282A', 0.5, 0).box(-0.08, y + 0.08, -0.08, 0.08, y + 0.12, 0.08);
    pool(pools, x + nx * 0.9, z + nz * 0.9, 1.7);
  }
}

/** A striped awning along a shop front, over the windows and the door. */
function awning(L: PlotLayout, parts: Parts): void {
  const props = parts.props;
  const h = L.house!;
  const frontWall = L.front > 0 ? h.z1 + T : h.z0 - T;
  const [x0, x1] = [h.x0 + 0.5, h.x1 - 0.5];
  const [a, b] = pick(AWNINGS, L.plot.x, L.plot.z, 76);
  props.at(...townPoint(L.turned, (x0 + x1) / 2, frontWall), townYaw(L.turned, L.front > 0 ? 0 : Math.PI));
  const half = (x1 - x0) / 2;
  const n = Math.max(2, Math.round((x1 - x0) / 0.5));
  for (let i = 0; i < n; i++) {
    const [u0, u1] = [-half + ((x1 - x0) * i) / n, -half + ((x1 - x0) * (i + 1)) / n];
    props.paint(i % 2 ? b : a, 0.85, 0);
    props.poly([[u0, 2.55, 0], [u1, 2.55, 0], [u1, 2.2, 0.95], [u0, 2.2, 0.95]], [0, 0.94, 0.35]);
    props.poly([[u0, 2.04, 0.95], [u0, 2.2, 0.95], [u1, 2.2, 0.95], [u1, 2.04, 0.95]], [0, 0, 1]);
    props.poly([[u0, 2.04, 0.95], [u1, 2.04, 0.95], [u1, 2.55, 0.0], [u0, 2.55, 0.0]], [0, -0.94, -0.35]);
  }
  props.paint('#3A3D41', 0.5, 0.5);
  for (const u of [-half, half]) props.box(u - 0.02, 2.2, 0, u + 0.02, 2.24, 0.95);
  blob(parts.shadows, ...townPoint(L.turned, (x0 + x1) / 2, frontWall + L.front * 0.6), half, 0.6, townYaw(L.turned, 0), 0.25, 0.02);
}

/**
 * Lays the whole street out and writes it, with the viewed lot (`view`; the menus' town: null, its
 * player's lot `WorldMeta.home`) as the focus, as one plan: see the rules above. (The renderer plans
 * it chunk by chunk, `TownStreet`.) `pace` lets a long build give the frame back between its parts.
 */
export async function planStreet(planner: HousePlanner, world: WorldStructure, view: ViewRect | null, options: StreetSource, pace?: () => Promise<void>): Promise<StreetPlan> {
  const street = new TownStreet(planner, world, options, view ? 'game' : 'town');
  const focus = view ? (world.plots.find((p) => overlaps(p, view, -0.01))?.id ?? NO_FOCUS) : (world.meta?.home ?? NO_FOCUS);
  const all: ChunkPlan[] = [];
  for (const id of street.chunkIds()) {
    all.push(street.chunk(id, focus));
    await pace?.();
  }
  return mergePlans(all, street);
}

/** Chunk plans as one street plan (the batches' order: the foliage first, then the rest as they come). */
export function mergePlans(chunks: readonly ChunkPlan[], street: TownStreet): StreetPlan {
  const silhouettes: SilhouettePlan = { parts: [], homes: [], lamps: [] };
  const [walk, props, glow, pools, shadows] = [new Geo(), new Geo(), new Geo(), new Geo(), new Geo()];
  const batches = new Map<string, Inst[]>((Object.keys(FOLIAGE) as FoliageKind[]).map((k) => [FOLIAGE[k], []]));
  let lot: StreetPlan['lot'] = null;
  const specs: SilhouetteSpec[] = [];
  for (const c of chunks) {
    mergeSilhouettes(silhouettes, c.silhouettes);
    specs.push(...c.specs);
    walk.merge(c.walk);
    props.merge(c.props);
    glow.merge(c.glow);
    pools.merge(c.pools);
    shadows.merge(c.shadows);
    lot ??= c.lot;
    for (const [k, items] of c.batches) {
      let list = batches.get(k);
      if (!list) batches.set(k, (list = []));
      list.push(...items);
    }
  }
  return {
    silhouettes,
    specs,
    walk,
    props,
    glow,
    pools,
    shadows,
    lot,
    batches: [...batches].map(([k, items]): [string, Inst[], boolean] => [k, items, castsShadow(k)]),
    footprints: street.footprints,
    standIns: street.standIns,
  };
}

/**
 * An instance's matrix (game space, in the layout of `shared/math.ts`) into `out` at `offset`: at
 * (x, y, z), turned by `yaw`, scaled `s` across and `sy` up (`flat`: not along z, a hedge stretched
 * along its run).
 */
export function instanceMatrix(item: Inst, flat: boolean, out: Float32Array, offset: number): void {
  const [x, z, s, sy, yaw, y = 0] = item;
  compose(s, sy, flat ? 1 : s, quatYawPitchRoll(yaw, 0, 0, scratchQ), x, y, z, scratchM);
  out.set(scratchM, offset);
}
const scratchQ: Quat = [0, 0, 0, 1];
const scratchM = new Float32Array(16);

/** Every plot laid out (`PlotLayout`; a plot the town gives no street side is left out). `facing`: one plot's, given. */
function layout(world: WorldStructure, streets: Rect[], paths: Rect[], facingOf?: number): PlotLayout[] {
  const out: PlotLayout[] = [];
  world.plots.forEach((info, i) => {
    // The side its street is on: as the town says (`meta.facing`: 0 north, 1 west, 2 south,
    // 3 east), else (older saves) the street one of its z edges lies on.
    let facing = facingOf ?? world.meta?.facing?.[i];
    if (facing === undefined) {
      for (const st of streets) {
        if (info.x + info.w <= st.x || info.x >= st.x + st.w) continue;
        if (Math.abs(info.z + info.d - st.z) < 0.05) facing = 2;
        else if (Math.abs(info.z - (st.z + st.d)) < 0.05) facing = 0;
      }
    }
    if (facing === undefined) return;
    const turned = facing % 2 === 1;
    const plot = { ...info, ...localRect(turned, info) };
    // Turned a quarter, the street west of a plot is on its max-z side as laid out, east of it on its min-z side.
    const front: 1 | -1 = facing === 2 || facing === 1 ? 1 : -1;
    const inside = (r: Rect) => r.x >= info.x - 0.01 && r.x + r.w <= info.x + info.w + 0.01 && r.z >= info.z - 0.01 && r.z + r.d <= info.z + info.d + 0.01;
    const own = paths.filter(inside).map((r) => localRect(turned, r));
    const path = own.find((r) => r.w < DRIVEWAY_MIN_WIDTH) ?? null;
    const drive = own.find((r) => r.w >= DRIVEWAY_MIN_WIDTH) ?? null;
    const house = info.house && localRect(turned, { x: info.house[0], z: info.house[1], w: info.house[2] - info.house[0], d: info.house[3] - info.house[1] });
    const base = info.public ? (info.house ? 'shop' : 'park') : info.house ? 'house' : 'empty';
    out.push({
      plot,
      info,
      turned,
      kind: base,
      base,
      front,
      edge: front > 0 ? plot.z + plot.d : plot.z,
      house: house ? { x0: house.x, z0: house.z, x1: house.x + house.w, z1: house.z + house.d } : null,
      door: path ? path.x + path.w / 2 : null,
      path,
      drive,
    });
  });
  return out;
}

/** A garage at the inner end of the driveway, beside the house, when it fits clear of `blocked` (the town's; `garageFor`): as laid out. */
function garage(L: PlotLayout, blocked: Rect[]): SilhouetteSpec['garage'] {
  return L.drive && L.house ? garageFor(L.plot, L.house, L.drive, L.front, blocked.map((b) => localRect(L.turned, b))) : null;
}

/** A fence or hedge along a line, skipping stretches that would run through objects. */
function edgeRun(style: 'picket' | 'hedge' | 'rail', props: Painter, hedges: Inst[], x0: number, z0: number, x1: number, z1: number, blocked: Rect[]): void {
  const len = Math.hypot(x1 - x0, z1 - z0);
  if (len < 1) return;
  const step = 2;
  const n = Math.max(1, Math.round(len / step));
  let runStart: number | null = null;
  const flush = (end: number) => {
    if (runStart === null) return;
    const [a, b] = [runStart / n, end / n];
    const [ax, az, bx, bz] = [x0 + (x1 - x0) * a, z0 + (z1 - z0) * a, x0 + (x1 - x0) * b, z0 + (z1 - z0) * b];
    if (style === 'picket') pickets(props, ax, az, bx, bz);
    else if (style === 'rail') rails(props, ax, az, bx, bz);
    else {
      const l = Math.hypot(bx - ax, bz - az);
      const k = Math.max(1, Math.round(l / 1.05));
      const yaw = Math.abs(bx - ax) >= Math.abs(bz - az) ? 0 : Math.PI / 2;
      for (let i = 0; i < k; i++) {
        const t = (i + 0.5) / k;
        const [x, z] = [ax + (bx - ax) * t, az + (bz - az) * t];
        hedges.push([x, z, (l / k) / 1.05 * (1 + hash(x, z, 60) * 0.06), 0.85 + hash(x, z, 61) * 0.2, yaw + (hash(x, z, 62) < 0.5 ? 0 : Math.PI)]);
      }
    }
    runStart = null;
  };
  for (let i = 0; i < n; i++) {
    const [a, b] = [i / n, (i + 1) / n];
    const xs = [x0 + (x1 - x0) * a, x0 + (x1 - x0) * b];
    const zs = [z0 + (z1 - z0) * a, z0 + (z1 - z0) * b];
    const r = { x: Math.min(...xs) - 0.1, z: Math.min(...zs) - 0.1, w: Math.abs(xs[1] - xs[0]) + 0.2, d: Math.abs(zs[1] - zs[0]) + 0.2 };
    if (blocked.some((o) => overlaps(r, o))) flush(i);
    else runStart ??= i;
  }
  flush(n);
}

/** A soft pool of lamplight on the ground. */
function pool(g: Geo, x: number, z: number, r: number): void {
  const y = 0.055;
  g.poly([[x - r, y, z - r], [x + r, y, z - r], [x + r, y, z + r], [x - r, y, z + r]], [0, 1, 0], [[0, 0], [1, 0], [1, 1], [0, 1]]);
}
