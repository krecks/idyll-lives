/**
 * The neighbourhood around the viewed lot, as a lived-in street: the neighbour houses
 * (`HouseBuilder.silhouettes`, with porches and garages), sidewalks with kerbs turning
 * the junctions' corners, the junctions' stop lines and crosswalks (`roads.ts`), streetlights that light up at night (with pools of light on the
 * ground), mailboxes, hydrants and bins along the kerb, cars parked on neighbours' driveways,
 * the fences and hedges round gardens and fields, and flower beds along their house
 * fronts, with pots by the door and little lights along the garden path; the things standing in
 * their gardens. The town's public places seen from another lot: shops as silhouettes with an awning,
 * a lot with a look (`WorldMeta.lotLooks`: a school, a church, a shop on the high street, a flats building) closed as
 * the building its look says, as tall as its real storeys (`buildingDress.ts`, its door where its own is; open, the
 * game draws its real walls), and the park,
 * playground and market things standing in the open. At the ends of the street a taxi
 * rank and street signs; benches, planters and bins along the kerb in front of the public places.
 *
 * Rules:
 * - Visual only, and kept off where Sims walk: front lawns (Sims cut across them on their way
 *   out of town), garden paths, driveway mouths, the road and the middle of the sidewalk. Kerb
 *   furniture stands on the road side of the sidewalk, cars park beside the houses.
 * - The things outdoors on other lots (neighbours' garden furniture and planters, the park's
 *   benches; not what stands indoors, in the silhouettes) in the game, placed, turned and moved off
 *   their tiles' middle as the lot draws them: the lawn is cleared under every lot's things
 *   (`lawnMask`). The menus' town (`pc/overview.ts`) draws every lot's things itself.
 * - Nothing on the viewed lot: the player builds there (the mailbox stands on the sidewalk). In
 *   the menus' town the player's lot (`WorldMeta.home`) gets its house as a shell dressed as the
 *   game draws it (its porch, no garden dressing), and nothing else. Its garage, if the
 *   town gave it one (`WorldStructure.garages`), its house's porches (the simulation's,
 *   `WorldStructure.porches`, as on every silhouette: `porchDoors`) and its porch pieces stand on it
 *   in the game too, drawn apart (`StreetPlan.lot`): they go with the lot's ground while its
 *   basement is in view, each porch with its wall in the cutaway. A house looks the same from the street as in the game:
 *   the lot's look on every storey, the roof the player chose.
 * - Fences: the world's (`shownFences`: the ones a town puts up round its gardens and fields,
 *   `game/gardenFences.ts`, the player's, and in an older town the ones it would have put up) on
 *   other lots, in their styles (`shared/fences.ts`), merged into each chunk's meshes, hedges with
 *   its plants; the game draws the focus lot's the same way, and every gate (they swing). The same
 *   fences whichever lot is in view.
 * - Cars: the world's (`WorldStructure.cars`), parked, in the menus' town; in the game they come
 *   and go, drawn by `pc/cars.ts`. A game saved before there were cars gets them by some neighbours'
 *   driveways (`parkCar`), parked for good.
 * - Static: generated once per town layout and kept across structure versions of the viewed
 *   lot (build-mode edits don't rebuild it), merged into a handful of meshes (one per material)
 *   plus thin-instanced foliage and props (manifest models: `model.street.*` and the town's
 *   things, see `assets/packs/street/manifest.json`), a few dozen draw calls for the whole street.
 * - Generated models (`.glb` files, downloaded like the rest), plus a CC0 concrete texture for the sidewalks.
 *
 * Engine-neutral: `planStreet` lays the street out and writes its geometry and its models'
 * instances (`StreetPlan`); the renderer's `Street` (`pc/street.ts`) draws it.
 */

import type { ModelEntry } from '../../assets/types';
import type { FenceStyleDef } from '../../content/content';
import { groundDepth, objectCentre, storeyOfRow, storeyRow, type FenceEdge, type GarageInfo, type LotLook, type ObjectPlacement, type Opening, type PlotInfo, type RoadPlan, type TownPlace, type WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import { EASE, easedBox, encodeFinish, facing, Geo, type V3 } from './geometry';
import { CARS, CAR_PAINT, carAge, garageAt, garageFor, hash, paintCar, parkCar, pick, type CarAge, type CarBody } from './cars';
import { chunkAt, chunkCoords, chunkId, distanceToRect, hashNumbers, hashString, RectGrid } from './chunks';
import { canopyBack, type HousePart, type HousePlanner, type SilhouettePlan, type SilhouetteSpec } from './house';
import { compose, quatYawPitchRoll, type Quat } from './math';
import { placementVariation, type StandIn } from './modelEntry';
import { wallCorners, writeFences } from './fences';
import { shownFences } from './shownFences';
import { centroid, lengths, pointAt } from '../../game/roads';
import { isSeparated, kindInfo } from '../../core/roadKinds';
import { junctionStretches } from './markings';
import { allBarriers, barrier, deckLevels, liftedStretches, MOTORWAY_SIGN, motorwayPairs, OVERPASS, OVERPASS_CONCRETE, signSpots, without } from './motorway';
import { pavedRects } from './paving';
import { streetGround, streetGroundChunks, streetGroundKey } from './streetGround';
import { bridgeSpans, Kerb, KERB, levelStops, raisedSpans, ROAD_Y, roadIndex, roadMeshes, SIDEWALK, stretchOf, townRoads, WALK_Y, walkStops, type RoadIndex, type RoadLevels } from './roads';
import { carriageway, hasSidewalks, JunctionCuts, junctionsOf, type JunctionPlan } from './junctions';
import { Levels, levelsOf } from './landSurface';
import type { BuiltSurface } from './builtSurface';
import { dressBuilding } from './buildingDress';
import { farBuilding, lotLookBuilding, type TownBuilding } from './townBuildings';

export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** Paths at least this wide are driveways (garden paths are 1 m; see `game/town.ts`). */
const DRIVEWAY_MIN_WIDTH = 1.8;
/** How far kerbside furniture reaches from where it stands, towards the road (m: a bench's depth, a lamp's base): on a 6 m lane it stands this far back from what cars pulling over keep clear (`PullOverStreetInfo.keep`). */
const KERBSIDE_REACH = 0.3;

/** How far either side of a driveway's mouth a car coming or going swings across the sidewalk (m): no kerbside furniture there. */
const CAR_SWING = 1.2;
/**
 * The taxi ranks at the rows' ends (`TownStreet.ranks`): one a street at most, none nearer another
 * than this (m), the town's own (a lot's `busStop`, the Market Square's) among them.
 */
export const RANK_APART = 60;
/** The rank's shelter: the content object it is, how far out past the row's last lot along the kerb, and how far behind the pavement its middle stands (m). */
const RANK_OBJECT = 'busStop';
const RANK_OUT = 4;
const RANK_BACK = 0.62;
/** Half the shelter's length along the kerb (m, `model.busStop` 2.98 m), and the clear gap every kerbside thing keeps from it and its sign. */
export const RANK_HALF = 1.49;
export const RANK_GAP = 1.2;
/** How far along the kerb from a rank `rankKerb` reports what stands near it (m). */
const RANK_SPAN = 10;
/** Half wall thickness (as in house.ts). */
const T = 0.07;

const MAILBOX = ['#22262A', '#2F4A3A', '#E8E6E0', '#8C2A26', '#2E4568'];
const BIN = ['#3C6B3A', '#55595E', '#2E5C8A'];

/** Model keys of the thin-instanced foliage (the street and trees packs). */
export const FOLIAGE = { bush: 'model.bush', flowers: 'model.street.flowers', hedge: 'model.hedge' } as const;
export type FoliageKind = keyof typeof FOLIAGE;
/** Props drawn from manifest models (the street and town packs); a missing one is left out. */
const PROP = {
  bench: 'model.street.bench',
  planter: 'model.street.planter',
  flowerTub: 'model.street.flowerTub',
  sign: 'model.street.sign',
  taxiSign: 'model.street.taxiSign',
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
  private rise = 0;
  /** `ground`: on a land the town follows, the ground (m) under a frame placed at (x, z), added to its height (`at`). */
  constructor(
    readonly geo: Geo,
    private readonly ground?: (x: number, z: number) => number,
  ) {}

  /** The frame at (x, z) turned `yaw`, `y` up (on the ground there, `ground`); `rise`: its heights sheared along its +z by this much a metre (a deck that climbs). */
  at(x: number, z: number, yaw = 0, y = 0, rise = 0): this {
    this.ox = x;
    this.oy = y + (this.ground ? this.ground(x, z) : 0);
    this.oz = z;
    this.c = Math.cos(yaw);
    this.s = Math.sin(yaw);
    this.rise = rise;
    return this;
  }

  paint(hex: string, rough = 0.7, metal = 0): this {
    this.geo.color(hex, encodeFinish(rough, metal));
    return this;
  }

  p(x: number, y: number, z: number): V3 {
    return [this.ox + x * this.c + z * this.s, this.oy + y + z * this.rise, this.oz - x * this.s + z * this.c];
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
    const put = (pts: V3[], n: V3, cut?: readonly (readonly [number, number, number])[]) => {
      if (!cut) return this.poly(pts, n);
      this.geo.polyCut(
        pts.map((q) => this.p(q[0], q[1], q[2])),
        this.n(n[0], n[1], n[2]),
        cut,
      );
    };
    if (easedBox(x0, y0, z0, x1, y1, z1, skip, this.geo.ease, put)) return this;
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
    if (this.geo.ease > 0 && caps && this.easedTube(axis, a0, a1, cu, cv, r0, r1, seg, from, to)) return this;
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

  /**
   * A whole tube with its caps (`tube`) on a mesh that eases (`Geo.easing`): each cap's rim the gate would find
   * hard (its sides 5 cm or more round, it and the cap 12 mm or more across) drawn back to a 45° shoulder, as a
   * box's edges are chamfered. False, nothing written, where no rim is.
   */
  private easedTube(axis: 'x' | 'y' | 'z', a0: number, a1: number, cu: number, cv: number, r0: number, r1: number, seg: number, from = 0, to = Math.PI * 2): boolean {
    const ease = this.geo.ease;
    const d = Math.sign(a1 - a0);
    const length = Math.abs(a1 - a0);
    const full = to - from >= Math.PI * 2 - 1e-6;
    // (A part-round one, a half pipe's ends: its rim shows as one arris round, however short each of its sides.)
    const shows = (r: number) => r >= EASED_RIM && length >= EASED_RIM && (full ? 2 * r * Math.sin(Math.PI / seg) >= 0.05 : r * (to - from) >= 0.05);
    const [e0, e1] = [r0, r1].map((r) => (shows(r) ? Math.min(ease, r * 0.3, length * 0.3) : 0));
    if (!e0 && !e1) return false;
    // (Its radius `a` along it.)
    const at = (a: number) => r0 + ((r1 - r0) * (a - a0)) / (a1 - a0);
    const [s0, s1] = [a0 + d * e0, a1 - d * e1];
    // (Drawn plain from here on: the shoulders are what makes it eased.)
    this.geo.easing(0);
    if (e0) this.tube(axis, a0, s0, cu, cv, r0 - e0, at(s0), seg, false, from, to);
    this.tube(axis, s0, s1, cu, cv, at(s0), at(s1), seg, false, from, to);
    if (e1) this.tube(axis, s1, a1, cu, cv, at(s1), r1 - e1, seg, false, from, to);
    this.geo.easing(ease);
    const P = (a: number, u: number, v: number): V3 => (axis === 'x' ? [a, u, v] : axis === 'y' ? [u, a, v] : [u, v, a]);
    for (const [a, r, n] of [[a0, r0 - e0, -d], [a1, r1 - e1, d]] as const) {
      if (r <= 0.001) continue;
      // (Round as the tube's own rings are; a part-round one's a slice from its middle.)
      const ring: V3[] = full ? [] : [P(a, cu, cv)];
      for (let i = 0; i < (full ? seg : seg + 1); i++) {
        const t = from + ((to - from) * i) / seg;
        ring.push(P(a, cu + Math.cos(t) * r, cv + Math.sin(t) * r));
      }
      this.poly(ring, P(n, 0, 0));
    }
    return true;
  }
}

/** A round part's rim shows hard (`cargo check-glb`'s edges) only where it and its cap are this wide or more (m). */
const EASED_RIM = 0.012;

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
  /** The content's fence styles, for the fences on other lots (`WorldStructure.fences`). */
  fenceStyles?: readonly FenceStyleDef[];
}

/** The street as laid out, to be drawn. */
export interface StreetPlan {
  /** The neighbours' houses (`HousePlanner.silhouettes`). */
  silhouettes: SilhouettePlan;
  /** Their wall boxes and garages, for what stands round them (x0, z0, x1, z1); the viewed lot's too (`dressingOnly`: its garage and porch). */
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
   * The viewed lot's own (the game's): its garage and porch (`SilhouetteSpec.dressingOnly`) and
   * their lamps (caps, glass and pools of light), apart from the street's meshes so they hide with
   * the lot's ground while its basement is in view. The porches' lamps apart again (`porches`, one
   * set for each wall they're on, with the cutaway's look bits that drop it): they go with that
   * wall. Null when it has no house, no garage and no porch pieces.
   */
  lot: { silhouettes: SilhouettePlan; props: Geo; glow: Geo; pools: Geo; porches: { look: number; props: Geo; glow: Geo; pools: Geo }[] } | null;
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
  /** Road paint (the markings): vertex-coloured in the plain finish, drawn over the road (`MaterialLibrary.roadPaint`; `material` unused). */
  paint?: boolean;
  /** Toned by vertex (`paving.ts`: worn asphalt, paving darker at its sides): drawn with its vertex colours. */
  toned?: boolean;
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
  /** The lots' buildings drawn closed in it (`lotDressed`): each as the town view has it (its lights, `litAt`), and its window glass by set (`PANE_SETS`). */
  buildings: { t: TownBuilding; panes: Geo[] }[];
}

/** Whether a model the street instances casts the sun's shadows (as `StreetPlan.batches`). */
export const castsShadow = (key: string): boolean => key !== FOLIAGE.bush && key !== FOLIAGE.flowers;

/** What a part of the street writes: the plain-finish props, lamp glass, pools, contact shadows, sidewalks, models' instances. */
class Parts {
  readonly props: Painter;
  readonly glow: Painter;
  readonly pools = new Geo();
  readonly shadows = new Geo().color('#000000');
  readonly walk = new Geo();
  readonly plants: Record<FoliageKind, Inst[]> = { bush: [], flowers: [], hedge: [] };
  readonly things = new Map<string, Inst[]>();
  /**
   * `ground`: on a land the town follows, the ground under what is put down here (`BuiltSurface.at`: the kerbside
   * things on the sidewalk, a road's height): each thing and frame stands on it where it is put, the pools of light
   * and the contact shadows laid over it (`drape`). None: at 0, or lifted as a whole where merged (a lot's pad).
   */
  constructor(
    private readonly has: (key: string) => boolean,
    readonly ground?: (x: number, z: number) => number,
  ) {
    this.props = new Painter(new Geo().easing(EASE), ground);
    this.glow = new Painter(new Geo(), ground);
  }
  /** The contact shadows laid over the ground (`ground`), once all are written. */
  drape(): void {
    if (this.ground) this.shadows.drape(this.ground);
  }
  put(key: string, x: number, z: number, yaw: number, s = 1, sy = s, y = 0): void {
    if (!this.has(key)) return;
    if (this.ground) y += this.ground(x, z);
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
  /** A lot with a look closed: the building its look says, dressed, on its pad (`TownStreet.lotDressed`). */
  dressed: { t: TownBuilding; parts: HousePart[]; panes: Geo[] } | null;
}

/**
 * How a lot with a look (`WorldMeta.lotLooks`: a named place's, a flats building's) looks from outside, its place's
 * hours (its windows lit while it is open), and whether a place has flats over it (`WorldStructure.flats` of its lot).
 */
export interface LotDress {
  look: LotLook;
  hours?: TownPlace['hours'];
  flats?: true;
}

/** A road side's kerbside furniture, by chunk, with what each chunk's share looks like (`Parts.signature`). */
interface KerbOut {
  chunks: Map<number, { parts: Parts; sig: number }>;
  /** Plots whose ground or things the furniture was checked against (see `TownStreet.free`). */
  touched: Set<number>;
  /** Cells of the blocked grid it looked in. */
  cells: Rect[];
  /**
   * The taxi rank at its row's end, if it has one (`TownStreet.chooseRanks`): its shelter's middle and
   * its sign's (null: none) along the kerb, and where every other kerbside thing near it stands (`RANK_SPAN`):
   * for checks (`TownStreet.rankKerb`).
   */
  rank?: { shelter: number; sign: number | null; others: number[] };
}

/** What a plot's share is planned from (`TownStreet.plotKey`): the plot, the focus lot it is planned for (or `NO_FOCUS`), its unit's id and a hash. */
interface PlotKey {
  L: PlotLayout;
  variant: number;
  id: string;
  hash: number;
  /** Who it is in a chunk's key: where it is and where the focus lot it depends on is (not their indexes, which a lot removed before them moves). */
  key: string;
}

/** What a chunk's fences are planned from (`TownStreet.fenceKey`): its fences, those round it (for the posts) and a hash. */
interface FenceKey {
  id: number;
  list: FenceEdge[];
  round: FenceEdge[];
  hash: number;
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

/** An instance `dy` m higher. */
const lifted = (i: Inst, dy: number): Inst => (dy ? [i[0], i[1], i[2], i[3], i[4], (i[5] ?? 0) + dy] : i);

/** Merges a part into a chunk being assembled, `dy` m higher (a lot's share, on its pad). */
function mergeParts(into: ChunkPlan, p: Parts, dy = 0): void {
  into.props.merge(p.props.geo, dy);
  into.glow.merge(p.glow.geo, dy);
  into.pools.merge(p.pools, dy);
  into.shadows.merge(p.shadows, dy);
  into.walk.merge(p.walk, dy);
  for (const kind of Object.keys(FOLIAGE) as FoliageKind[]) {
    if (!p.plants[kind].length) continue;
    const list = into.batches.get(FOLIAGE[kind])!;
    for (const i of p.plants[kind]) list.push(lifted(i, dy));
  }
  for (const [k, items] of p.things) {
    let list = into.batches.get(k);
    if (!list) into.batches.set(k, (list = []));
    for (const i of items) list.push(lifted(i, dy));
  }
}

/** Silhouette parts (one per material) merged by name, `dy` m higher (a lot's houses, on its pad). */
function mergeSilhouettes(into: SilhouettePlan, s: SilhouettePlan, dy = 0): void {
  for (const part of s.parts) {
    const same = into.parts.find((p) => p.name === part.name);
    if (same) same.geo.merge(part.geo, dy);
    else into.parts.push({ ...part, geo: new Geo().merge(part.geo, dy) });
  }
  into.homes.push(...(dy ? s.homes.map((h) => ({ spec: h.spec, panes: new Geo().merge(h.panes, dy) })) : s.homes));
  into.lamps.push(...(dy ? s.lamps.map((l): (typeof s.lamps)[number] => [l[0], l[1] + dy, l[2], l[3], l[4], ...(l.length > 5 ? [l[5]] : [])] as (typeof s.lamps)[number]) : s.lamps));
}

/** How far either side of a traffic light's pole along the kerb nothing else stands (m: `WorldStructure.lightPoles`). */
export const POLE_CLEAR = 1.5;

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
 * lot (`chunk`): the focus lot's house and garden are the game's own (only its garage and porch here, apart in
 * `lot`), and its fences the game's too (the world's fences round it are the same whichever lot is
 * the focus). Each part of the street (a plot, a road side's kerb, a chunk's fences) is planned on its own and kept, planned again only when what it
 * stands among changes (a thing moved near it, the focus next to it), so a focus change or an edit
 * builds one or two chunks again, not the street.
 *
 * `mode` 'town': the menus' town (`WorldMeta.home` the player's lot, every window lit, the world's
 * cars parked, no other lots' things: the overview draws them).
 */
/** The street's own in a chunk (`TownStreet.staticParts`): its things, the road's surface, the trails, the markings, the medians' grass. */
interface StaticPart {
  parts: Parts;
  road: Geo;
  trail: Geo;
  marks: Geo;
  median: Geo;
}

/**
 * An avenue's median: its trees (a model the woods have), how far apart and how far from a junction's pad; its grass's
 * material (the land's, a detail texture toned by its vertex colour, `roads.ts` `MEDIAN_GRASS`).
 */
const MEDIAN_TREE = 'model.tree.maple';
const MEDIAN_TREE_EVERY = 12;
const MEDIAN_TREE_CLEAR = 5;
const MEDIAN_MATERIAL = 'material.ground';

export class TownStreet {
  readonly plots: PlotLayout[];
  private world: WorldStructure;
  /** The porches and porch pieces last given (`setWorld`), and what they were. */
  private porchesOf: WorldStructure['porches'];
  private piecesOf: WorldStructure['porchPieces'];
  private porchesKey = '';
  private readonly roads: RoadPlan[];
  /** A big world's ways between its areas: country roads (a surface, no sidewalks) and trails (a narrow path). */
  private readonly ways: { country: RoadPlan[]; trail: RoadPlan[] };
  private readonly index: RoadIndex;
  /** The roads that stop a sidewalk (`roads.ts` `walkStops`): the streets and lanes (`roads`, at the same indexes), then the country roads. */
  private readonly walkStops: RoadIndex;
  /** The junctions as drawn (`junctions.ts`), and where their corners stop the sidewalks. */
  private readonly junctions: JunctionPlan[];
  /**
   * On a land the town follows, the heights everything stands at (`landSurface.ts`): the roads climbing, each
   * lot's share on its pad, the kerbside things on the sidewalk. Null: everything at 0, as ever.
   */
  private levels: Levels | null;
  /** Every road of the town as laid out (what `levels` finds them by). */
  private readonly allRoads: RoadPlan[];
  /**
   * The roads with a surface drawn: `roads`, then the country roads. One array for the street's life, so what is worked
   * out once per set of roads (the motorway's pairs, its gores, its signs: `motorway.ts`, `markings.ts`) is when the land
   * moves (`landMoved`) too.
   */
  private readonly surfaced: RoadPlan[];
  /** Each plot's index among the world's (its pad's, `BuiltSurface.padY`), by id. */
  private readonly plotIndex = new Map<number, number>();
  private readonly cuts: JunctionCuts;
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
  /** The things' poses and the garages as last taken (`setWorld`). */
  private poses: Int32Array | null = null;
  private garagesKey = '';
  private garagesOf: WorldStructure['garages'] | null = null;
  private blocked = new RectGrid<Blocker>(4);
  private objectsByPlot = new Map<number, ObjectPlacement[]>();
  private garages = new Map<number, SilhouetteSpec['garage']>();
  /** Things on no plot's ground (on the street, on lots not laid out). */
  private readonly plotUnits = new Map<string, { hash: number; out: PlotOut; chunk: number; variant: number }>();
  private readonly kerbUnits = new Map<string, { hash: number; out: KerbOut; carry: string }>();
  /** The world's fences by the chunk their middle is in, as last taken (`setWorld`), and each chunk's share of the street. */
  private fencesOf: FenceEdge[] | null = null;
  private fencesByChunk = new Map<number, FenceEdge[]>();
  /** Which chunk draws the post on each grid corner (`x:z`): the first fence's there (in the world's order). */
  private postOwner = new Map<string, number>();
  /** The grid corners walls end or turn at (`wallCorners`), and the walls they're from: a fence meeting a wall there has no post. */
  private wallCornerSet = new Set<string>();
  private wallsOf: unknown = null;
  /** The edges walls (and the doors and windows in them) stand on, `axis:x:z`, made when first asked for after the walls or openings changed (`walled`). */
  private wallEdges: Set<string> | null = null;
  private readonly fenceUnits = new Map<number, { hash: number; out: Parts }>();
  /** The lots' looks by plot id (`lotDresses`), what they were taken from (`setWorld`), and as a key. */
  private dresses = new Map<number, LotDress>();
  private dressesOf: unknown[] = [];
  private dressesKey = '';

  /** The street's own: sidewalks, the road's surface, fountains, bridges (by chunk); and what it was planned from (`staticKey`). */
  private statics: Map<number, StaticPart> | null = null;
  private staticKey: string | null = null;
  /** The menus' parked cars, by chunk (`carParts`): apart from `statics`, which a lot edit (and the car it takes) keeps. */
  private cars: Map<number, Parts> | null = null;
  /** What the kerbs are planned from beyond their own row (`carryKey`): the same, a street of the town before may hand its kerbs on. */
  private kerbsKey: string | null = null;
  /** Kerbs by road side, and which plots front each. */
  private readonly rows = new Map<string, number[]>();
  /**
   * The road sides with a taxi rank at their row's end (`chooseRanks`), and where: chosen once for the
   * town as it's laid out, whatever is streamed in or in view, so none comes or goes as the camera moves.
   */
  private ranks = new Map<string, [number, number]>();
  readonly footprints = new Map<string, [number, number]>();
  readonly standIns = new Map<string, StandIn>();

  /**
   * `previous`: the street of the town shown before (step 2's town before an edit), if it is of the same mode: what of
   * it this town plans just as it did is taken over (`inherit`), not planned again.
   */
  constructor(
    private readonly planner: HousePlanner,
    world: WorldStructure,
    private readonly options: StreetSource,
    readonly mode: 'game' | 'town',
    previous?: TownStreet | null,
  ) {
    this.world = world;
    const streets = (world.meta?.streets ?? []).filter((s) => s.w >= s.d);
    // Streets and lanes get sidewalks and kerbside furniture; a big world's country roads only their
    // surface, its trails a narrow path (`ways`).
    const all = townRoads(world);
    this.roads = all.filter((r) => r.kind !== 'country' && r.kind !== 'trail');
    this.ways = { country: all.filter((r) => r.kind === 'country'), trail: all.filter((r) => r.kind === 'trail') };
    this.index = roadIndex(this.roads);
    this.walkStops = roadIndex(walkStops(all));
    this.junctions = junctionsOf(world, all);
    this.cuts = JunctionCuts.of(this.junctions);
    this.levels = levelsOf(world, all);
    this.allRoads = all;
    this.surfaced = [...this.roads, ...this.ways.country];
    world.plots.forEach((p, i) => this.plotIndex.set(p.id, i));
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
    this.ranks = this.chooseRanks(world);
    if (previous && previous.mode === mode) this.inherit(previous);
  }

  /**
   * Takes over from `previous` (the street of the town shown before) what this one would plan just as it did: the
   * street's own (`staticParts`), when the roads, their junctions and heights and the bridges' beds are the same; and
   * each road side's kerbside furniture (`kerbOf`), when the roads are the same, the side's row of lots too, and the
   * heights in every chunk it stands in (`BuiltSurface.chunkSig`). (What the kerbs checked of the things and garages
   * round them is checked again as ever: `kerbOf`.) A lot removed, added or changed then plans only its own row's.
   */
  private inherit(previous: TownStreet): void {
    if (previous.statics && previous.staticKeyOf() === this.staticKeyOf()) {
      this.statics = previous.statics;
      for (const [id, sig] of previous.staticSigs) this.staticSigs.set(id, sig);
      this.handedOn.statics = true;
    }
    if (previous.kerbsKeyOf() !== this.kerbsKeyOf()) return;
    const [was, now] = [previous.levels?.surface, this.levels?.surface];
    for (const [id, unit] of previous.kerbUnits) {
      const [r, side, variant] = id.split('|');
      const rowKey = `${r}|${side}`;
      if (!this.rows.has(rowKey) || unit.carry !== this.rowCarry(rowKey, Number(variant))) continue;
      if ([...unit.out.chunks.keys()].some((c) => (was?.chunkSig(c) ?? 0) !== (now?.chunkSig(c) ?? 0))) continue;
      this.kerbUnits.set(id, unit);
      this.handedOn.kerbs++;
    }
  }

  /** What the street before handed on (`inherit`): its street's own, and how many road sides' kerbs (for the perf log and the tests). */
  readonly handedOn = { statics: false, kerbs: 0 };

  /** What every kerb and the street's own are planned from beyond their own row: the roads, their junctions, the lights' poles, the lanes cars pull over on, the world's size and whether it has cars. */
  private kerbsKeyOf(): string {
    const w = this.world;
    return (this.kerbsKey ??= hashString(JSON.stringify([this.walkStops.roads, this.ways, this.junctions, w.lightPoles ?? null, w.pullOverStreets ?? null, !!w.cars, this.W, this.D, !!this.options.ground, this.options.has(PROP.fountain)])).toString(36));
  }

  /** What the street's own is planned from (`staticParts`): what the kerbs are, and the heights it stands at (`levelsKey`). */
  private staticKeyOf(): string {
    return (this.staticKey ??= `${this.kerbsKeyOf()}|${this.levelsKey()}`);
  }

  /** The heights the street's own stands at: the roads' (`roadsSig`), the junctions', and the beds and the ground under the decks (every 2 m). */
  private levelsKey(): string {
    const levels = this.levels;
    if (!levels) return '-';
    const beds: number[] = [];
    for (const road of this.surfaced) {
      for (const [s0, s1] of bridgeSpans(road)) {
        const ls = lengths(road.points);
        for (let u = s0; u <= s1; u += 2) {
          const { p } = pointAt(road.points, u, ls);
          beds.push(Math.round(levels.surface.landAt(p[0], p[1]) * 1000));
        }
      }
      for (const [s0, s1] of raisedSpans(road)) {
        const ls = lengths(road.points);
        for (let u = s0; u <= s1; u += 2) {
          const { p } = pointAt(road.points, u, ls);
          beds.push(Math.round(levels.groundUnder(p[0], p[1]) * 1000));
        }
      }
    }
    return `${levels.surface.roadsSig()}|${hashString(JSON.stringify(levels.surface.junctionHeights))}|${hashNumbers(beds)}`;
  }

  /** What road side `rowKey`'s kerbside furniture is planned from of its own (with lot `focus` in view): its row of lots, as each fronts it, the paths and driveways meeting it, and its taxi rank. */
  private rowCarry(rowKey: string, focus: number): string {
    const row = (this.rows.get(rowKey) ?? []).map((i) => {
      const { L } = this.fronting[i]!;
      return [L.plot.x, L.plot.z, L.plot.w, L.plot.d, L.turned, L.edge, L.door, L.drive, this.kindOf(L, focus)];
    });
    if (!this.mouths) {
      this.mouths = new Map();
      for (const { p, road, side } of this.meeting) {
        const k = `${road}|${side}`;
        this.mouths.set(k, `${this.mouths.get(k) ?? ''}${p.x},${p.z},${p.w},${p.d};`);
      }
    }
    return `${hashString(JSON.stringify(row))}|${hashString(this.mouths.get(rowKey) ?? '')}|${this.ranks.get(rowKey)?.join(',') ?? '-'}`;
  }
  /** The paths and driveways meeting each road side (`meeting`), written out for `rowCarry`. */
  private mouths: Map<string, string> | null = null;

  /** The taxi ranks at the rows' ends, where they stand (x, z): for checks. */
  rankSpots(): [number, number][] {
    return [...this.ranks.values()];
  }

  /** The taxi rank of road `r`'s `side` and what stands round it along the kerb (`KerbOut.rank`): for checks. */
  rankKerb(r: number, side: 1 | -1): KerbOut['rank'] {
    return this.kerbOf(`${r}|${side}`, NO_FOCUS).rank;
  }

  /**
   * Which road sides get a taxi rank at their row's end: of those with room for one (`rankSpot`), in
   * an order of the town's own (by where each stands), one a road at most, and none within
   * `RANK_APART` of another or of a rank the town has on a lot (the Market Square's).
   */
  private chooseRanks(world: WorldStructure): Map<string, [number, number]> {
    const spots: { key: string; road: number; at: [number, number]; order: number }[] = [];
    for (const [key, ids] of this.rows) {
      const [r, side] = key.split('|').map(Number) as [number, 1 | -1];
      const at = ids.length ? this.rankSpot(r, side, ids) : null;
      if (at) spots.push({ key, road: r, at, order: hash(at[0], at[1], 81) });
    }
    spots.sort((a, b) => a.order - b.order || (a.key < b.key ? -1 : 1));
    const taken: [number, number][] = (world.objects ?? []).filter((o) => o.def === RANK_OBJECT).map((o) => {
      const c = objectCentre(o);
      return [c.x, c.z];
    });
    const roads = new Set<number>();
    const out = new Map<string, [number, number]>();
    for (const { key, road, at } of spots) {
      if (roads.has(road) || taken.some(([x, z]) => Math.hypot(x - at[0], z - at[1]) < RANK_APART)) continue;
      roads.add(road);
      taken.push(at);
      out.set(key, at);
    }
    return out;
  }

  /**
   * Where the taxi rank at the end of road `r`'s row on `side` would stand (its shelter's middle),
   * if there's room: past the row's last lot (the first one's on the road's left), behind the
   * pavement, clear of the other roads, of every lot and of what stands there as the town was laid out.
   */
  private rankSpot(r: number, side: 1 | -1, rowIds: number[]): [number, number] | null {
    const kerb = new Kerb(this.roads[r], r, side, this.walkStops, this.cuts);
    const spans = rowIds.map((i) => {
      const { L, at } = this.fronting[i]!;
      const [a, b] = [kerb.along(...at(L.plot.x)), kerb.along(...at(L.plot.x + L.plot.w))];
      return [Math.min(a, b), Math.max(a, b)] as const;
    });
    const first = side > 0;
    const corner = first ? Math.min(...spans.map((sp) => sp[0])) : Math.max(...spans.map((sp) => sp[1]));
    const cs = corner + (first ? -1 : 1) * RANK_OUT;
    if (!(cs > 1 && cs < kerb.length - 1 && kerb.open(cs))) return null;
    // (Clear of the garden paths' and driveways' mouths and the light poles by 1.6 m, as `planKerb` keeps
    // its kerbside things: what it checks again where it plans the rank.)
    const world = this.world;
    const blocked = (a: number, b: number) => cs > a - 1.6 && cs < b + 1.6;
    for (const { p } of this.meeting.filter((m) => m.road === r && m.side === side)) {
      const corners = [[p.x, p.z], [p.x + p.w, p.z], [p.x, p.z + p.d], [p.x + p.w, p.z + p.d]].map(([x, z]) => kerb.along(x, z));
      const swing = world.cars && Math.min(p.w, p.d) >= DRIVEWAY_MIN_WIDTH ? CAR_SWING : 0;
      if (blocked(Math.min(...corners) - swing, Math.max(...corners) + swing)) return null;
    }
    for (const [x, z] of world.lightPoles ?? []) {
      const near = this.index.nearest(x, z, 8);
      if (near?.road !== r || near.side !== side) continue;
      const at = kerb.along(x, z);
      if (blocked(at - POLE_CLEAR, at + POLE_CLEAR)) return null;
    }
    const [cx, cz] = kerb.at(cs, -RANK_BACK);
    const shelter = { x: cx - 1.6, z: cz - 1.6, w: 3.2, d: 3.2 };
    if (this.index.onRoad(cx, cz, 1, r) || !this.free(shelter, 0.2, NO_FOCUS)) return null;
    for (const { L } of this.plotGrid.near(shelter)) if (overlaps(shelter, L.info)) return null;
    return [cx, cz];
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
      m ? keyOf(m, () => JSON.stringify([m.seed ?? 0, m.streets, m.roads, m.paths, m.facing, m.world?.fields, m.home, m.gardenFences])) : '-',
      // (Whether the town has them: which garages and where is `setWorld`'s, so one built or sold
      // plans only the chunks round it again.)
      world.garages ? 'garages' : '-',
      mode === 'town' ? (world.cars ? keyOf(world.cars, () => JSON.stringify(world.cars)) : '-') : !!world.cars,
      world.pullOverStreets ? keyOf(world.pullOverStreets, () => JSON.stringify(world.pullOverStreets)) : '-',
      world.lightPoles ? keyOf(world.lightPoles, () => JSON.stringify(world.lightPoles)) : '-',
      world.junctions ? keyOf(world.junctions, () => JSON.stringify(world.junctions)) : '-',
      keyOf(world.plots, () => JSON.stringify(world.plots.map((p) => [p.id, p.x, p.z, p.w, p.d, p.public, p.entry]))),
    ].join('|');
  }

  /**
   * The land under the town moved (the game's land shaped by the brushes): stands on `surface` from now on, the one its
   * landscape solves (one surface for the land and what stands on it). Each chunk's key is worked out again; the next
   * build plans again only the chunks whose heights moved (`chunkSig`). False on a town on flat land (no surface).
   */
  landMoved(surface: BuiltSurface): boolean {
    if (!this.levels) return false;
    // (Not one solved for other roads: a road the land's water bridged, which the world brought back laid out again.)
    if (surface.roads.length !== this.allRoads.length || surface.roads.some((r, i) => r !== this.allRoads[i])) return false;
    if (this.levels.surface !== surface) this.levels = new Levels(surface, this.allRoads);
    this.ground_ = null;
    this.statics = null;
    this.version++;
    return true;
  }

  /** Whether the street stands on `surface` (`landMoved`). */
  standsOn(surface: BuiltSurface): boolean {
    return this.levels?.surface === surface;
  }

  /** The height (m) the street stands at at (`x`, `z`), on its surface (null: a town on flat land). */
  levelAt(x: number, z: number): number | null {
    return this.levels ? this.levels.at(x, z) : null;
  }

  /** How many bridge decks the street draws (every road's spans, `bridgeSpans`): for the benches. */
  bridgeCount(): number {
    return this.surfaced.reduce((n, r) => n + bridgeSpans(r).length, 0);
  }

  /** Takes the world's things, houses and rooms as they are now (the layout stays: see `layoutKey`). */
  setWorld(world: WorldStructure): void {
    this.world = world;
    // The lots' looks, their places' hours and their flats (a place's lot added or given up): each plot's share notices
    // its own (`plotKey`).
    const dressesOf = [world.meta?.lotLooks, world.meta?.places, world.plots, world.flats];
    if (dressesOf.some((v, i) => v !== this.dressesOf[i])) {
      this.dressesOf = dressesOf;
      this.dresses = lotDresses(world);
      const key = JSON.stringify([...this.dresses]);
      if (key !== this.dressesKey) {
        this.dressesKey = key;
        this.version++;
      }
    }
    // Fences (one put up or taken down): found again by chunk; each chunk's share notices its own (`fenceKey`).
    const fences = shownFences(world, this.options.fenceStyles ?? []);
    if (fences !== this.fencesOf) {
      this.fencesOf = fences;
      const by = new Map<number, FenceEdge[]>();
      const owner = new Map<string, number>();
      for (const f of fences) {
        if (f.z > this.D) continue;
        const id = f.axis === 'h' ? chunkAt(f.x + 0.5, f.z) : chunkAt(f.x, f.z + 0.5);
        let list = by.get(id);
        if (!list) by.set(id, (list = []));
        list.push(f);
        for (const k of [`${f.x}:${f.z}`, f.axis === 'h' ? `${f.x + 1}:${f.z}` : `${f.x}:${f.z + 1}`]) if (!owner.has(k)) owner.set(k, id);
      }
      this.fencesByChunk = by;
      this.postOwner = owner;
      this.version++;
    }
    // Walls built or taken down: the fences meeting them are drawn again (with or without their posts there).
    if (world.walls !== this.wallsOf) {
      this.wallsOf = world.walls;
      this.wallCornerSet = wallCorners(world.walls);
      this.wallEdges = null;
      this.fenceUnits.clear();
      this.version++;
    }
    // (Rooms decide which things are outdoors: another rooms array, keys worked out again.)
    if (world.rooms !== this.roomsOf) {
      this.roomsOf = world.rooms;
      this.version++;
    }
    // Doors and windows (a window put in on the focus lot): found again by where they are; each
    // plot's share notices those round its house (`plotKey`).
    if (world.openings !== this.openingsOf) {
      this.openingsOf = world.openings;
      this.wallEdges = null;
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
    // Porches and porch pieces (one built or taken down): each plot's share notices its own (`plotKey`).
    if (world.porches !== this.porchesOf || world.porchPieces !== this.piecesOf) {
      this.porchesOf = world.porches;
      this.piecesOf = world.porchPieces;
      const key = JSON.stringify([world.porches ?? null, world.porchPieces ?? null]);
      if (key !== this.porchesKey) {
        this.porchesKey = key;
        this.version++;
      }
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
    // (A new message with the same things where they were, the same garages: the grid stands too.)
    const poses = new Int32Array(world.objects.length * 3);
    world.objects.forEach((o, i) => {
      poses[i * 3] = poseHash(o);
      poses[i * 3 + 1] = o.w * 1024 + o.d;
      poses[i * 3 + 2] = o.id;
    });
    const garagesKey = JSON.stringify(world.garages ?? null);
    const samePoses = !!this.poses && this.poses.length === poses.length && this.poses.every((v, i) => v === poses[i]);
    this.poses = poses;
    if (!relaid && samePoses && garagesKey === this.garagesKey) {
      this.objectsOf = world.objects;
      this.garagesOf = world.garages;
      return;
    }
    this.garagesKey = garagesKey;
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
      if (L.base === 'shop' || L.base === 'park') continue;
      if (world.garages) {
        // (One the player built stands on a lot with no house yet too, its door any way round.)
        const g = world.garages.find((g) => g.plot === L.plot.id);
        if (g && (L.house || g.rot !== undefined)) garages.set(L.info.id, { x0: g.x0, z0: g.z0, x1: g.x1, z1: g.z1, ...(g.rot !== undefined ? { door: garageDoor(g.rot) } : {}) });
      } else if (L.base === 'house' && L.house) {
        garages.set(L.info.id, this.legacyGarage(L, NO_FOCUS));
      }
    }
    for (const [plot, g] of garages) {
      if (g) blocked.add({ x: g.x0 - T, z: g.z0 - T, w: g.x1 - g.x0 + 2 * T, d: g.z1 - g.z0 + 2 * T, plot, kind: world.garages ? 'garage' : 'legacyGarage', hash: mix(mix(mix(g.x0 * 100, g.z0 * 100), mix(g.x1 * 100, g.z1 * 100)), g.door ? g.door[0] * 3 + g.door[1] + 5 : 0) });
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
      const d = distanceToRect(x, z, r.x, r.z, r.w, r.d);
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

  /** A plot's pad (m): the level its share of the street stands at (0 without a surface). */
  private padOf(L: PlotLayout): number {
    if (!this.levels) return 0;
    const i = this.plotIndex.get(L.info.id);
    return i === undefined ? this.levels.at(L.info.x + L.info.w / 2, L.info.z + L.info.d / 2) : this.levels.surface.padY(i);
  }

  /** The ground (m) at a point: the surface's (`BuiltSurface.at`), or none without one (everything at 0). */
  private get groundAt(): ((x: number, z: number) => number) | undefined {
    const levels = this.levels;
    return levels ? (this.ground_ ??= (x, z) => levels.at(x, z)) : undefined;
  }
  private ground_: ((x: number, z: number) => number) | null = null;

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

  /**
   * Plans, a part at a time with `pace` between, what `chunkIds` and the chunks' keys need of the whole town: the
   * street's own (`staticParts`) and every road side's kerbside furniture. Without it they are planned when first
   * asked for, all at once (a big town's in one stall of a few hundred milliseconds).
   */
  async prepare(pace?: () => Promise<void>): Promise<void> {
    this.staticParts();
    await pace?.();
    for (const r of this.rows.keys()) {
      this.kerbOf(r, NO_FOCUS);
      await pace?.();
    }
  }

  /** Every chunk with something of the street in it. */
  chunkIds(): number[] {
    const ids = new Set<number>();
    for (const L of this.plots) ids.add(this.plotChunk(L));
    for (const id of this.staticParts().keys()) ids.add(id);
    for (const id of this.carParts().keys()) ids.add(id);
    for (const r of this.rows.keys()) for (const id of this.kerbOf(r, NO_FOCUS).chunks.keys()) ids.add(id);
    for (const f of this.fields) ids.add(chunkAt(f.x + f.w / 2, f.z + f.d / 2));
    for (const id of this.fencesByChunk.keys()) ids.add(id);
    if (this.options.ground) for (const [, rects] of this.groundRects()) for (const r of rects) ids.add(chunkAt(r.x + r.w / 2, r.z + r.d / 2));
    for (const id of streetGroundChunks(this.world)) ids.add(id);
    return [...ids].sort((a, b) => a - b);
  }

  /** The chunk the focus lot's own share (`lot`) and its shell's absence are in. */
  focusChunk(focus: number): number | null {
    const L = this.byId.get(focus);
    return L ? this.plotChunk(L) : null;
  }

  // ---- the parts, each planned on its own and kept ----------------------------------------

  /** The plots a plot's share depends on with the focus there: itself and those beside it (what stands round it is the game's there). */
  private plotDeps(L: PlotLayout): PlotLayout[] {
    return this.plotsNear(L.info, 2);
  }

  /** Whether a wall (or a door or a window in one) stands on an edge (`SilhouetteSpec.walled`). */
  private readonly walled = (axis: 'h' | 'v', x: number, z: number): boolean => {
    if (!this.wallEdges) {
      this.wallEdges = new Set();
      for (const e of [...this.world.walls, ...(this.world.openings ?? [])]) this.wallEdges.add(`${e.axis}:${e.x}:${e.z}`);
    }
    return this.wallEdges.has(`${axis}:${x}:${z}`);
  };

  /** Whether a porch piece is a canopy against a wall (it goes with that wall in the cutaway on the viewed lot). */
  private backed(p: { kind: string; x: number; z: number; w: number; d: number; rot: number }): boolean {
    return p.kind === 'canopy' && canopyBack(p).some(([axis, x, z]) => this.walled(axis, x, z));
  }

  /** What a plot's share is planned from with lot `focus` in view (its unit's id and a hash), without planning it. */
  private plotKey(L: PlotLayout, focus: number): PlotKey {
    const deps = this.plotDeps(L);
    const variant = deps.some((d) => d.info.id === focus) ? focus : NO_FOCUS;
    const id = `${L.info.id}|${variant}`;
    const at = this.byId.get(variant)?.info;
    const key = `${placeOf(L.info)}|${at ? placeOf(at) : '-'}`;
    // What it's planned from: the plot, its neighbours' houses (their fronts), the things and garages round it, the openings on its house.
    let h = hashString(JSON.stringify([L.info.house, L.info.roof, L.info.storeys, L.info.public, this.sides.get(L)?.left?.info.house, this.sides.get(L)?.right?.info.house]));
    // (A named place's lot: its look and its hours.)
    const dress = this.dresses.get(L.info.id);
    if (dress) h = mix(h, hashString(JSON.stringify(dress)));
    // (And how it is laid out, its pad and whether the town has its own cars and garages: what a chunk planned for
    // another town of the same lot here may differ by. Its place is in `key`.)
    h = mix(h, hashString(JSON.stringify([L.turned, L.front, L.edge, L.door, L.path, L.drive, L.base, L.plot.x, L.plot.z, L.plot.w, L.plot.d, L.info.entry])));
    h = mix(mix(h, Math.round(this.padOf(L) * 1000)), (this.world.cars ? 1 : 2) + (this.world.garages ? 4 : 8));
    h = this.blockersHash([{ x: L.info.x - 1, z: L.info.z - 1, w: L.info.w + 2, d: L.info.d + 2 }], variant, h);
    if (!this.world.garages && L.base === 'house') {
      const g = this.legacyGarage(L, variant);
      h = mix(h, g ? hashNumbers([g.x0, g.z0, g.x1, g.z1]) : 7);
    }
    if (L.info.house) {
      const [x0, z0, x1, z1] = L.info.house;
      for (const { o } of this.openingGrid.near({ x: x0, z: z0, w: x1 - x0, d: z1 - z0 }, 1)) h = mix(mix(mix(h, o.x), o.z * 7 + (o.axis === 'h' ? 1 : 2)), o.kind.length);
      h = mix(h, this.coverOf(L));
    }
    // Its porches and porch pieces.
    for (const p of this.world.porches ?? []) if (p.plot === L.info.id) h = mix(mix(mix(h, p.x), p.z * 7 + (p.axis === 'h' ? 1 : 2)), p.out + 3);
    for (const p of this.world.porchPieces ?? []) if (p.plot === L.info.id) h = mix(mix(mix(mix(mix(h, p.x), p.z), p.w * 64 + p.d * 8 + p.rot), p.kind.length), this.backed(p) ? 1 : 0);
    // Indoors or out: the things on it, by the rooms under them.
    const W = this.W;
    for (const o of this.objectsByPlot.get(L.info.id) ?? []) h = mix(h, this.world.rooms[o.z * W + o.x] ? 1 : 0);
    return { L, variant, id, hash: h, key };
  }

  /** A plot's share as `plotKey` found it: the one kept, or planned now. */
  private plotOut({ L, variant, id, hash }: PlotKey): PlotOut {
    const cached = this.plotUnits.get(id);
    if (cached && cached.hash === hash) return cached.out;
    const out = this.planPlot(L, variant);
    this.plotUnits.set(id, { hash, out, chunk: this.plotChunk(L), variant });
    return out;
  }

  /**
   * Lets go of the parts kept that the street won't soon plan a chunk from: the plots' and the
   * fences' of the chunks not in `chunks` (null: every chunk's are kept), and the plots' for another
   * focus lot than `focus`. A chunk's key needs none of them (`chunkKey`): a chunk streamed in with
   * the key it was built with plans nothing; one whose key changed meanwhile plans its plots and
   * fences afresh.
   */
  keepUnits(chunks: ReadonlySet<number> | null, focus: number): void {
    for (const [id, u] of this.plotUnits) if ((chunks && !chunks.has(u.chunk)) || (u.variant !== NO_FOCUS && u.variant !== focus)) this.plotUnits.delete(id);
    if (chunks) for (const id of this.fenceUnits.keys()) if (!chunks.has(id)) this.fenceUnits.delete(id);
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
    let dressed: PlotOut['dressed'] = null;
    // (A lot with a look not in view, a named place's or a flats building's: the building its look says, in place of a
    // silhouette, an awning and a garden.)
    const dress = (kind === 'shop' || kind === 'house') && L.house ? this.dresses.get(L.info.id) : undefined;

    // ---- its house (the player's in the menus' town a shell; the viewed lot's only its garage and porch) ----
    // (A garage or porch pieces the player built on their lot before any house: they stand there alone.)
    const porches = world.porches?.filter((p) => p.plot === L.info.id);
    const pieces = world.porchPieces?.filter((p) => p.plot === L.info.id);
    const loneGarage = home && game && !L.house ? (this.garages.get(L.info.id) ?? null) : null;
    const lone = loneGarage ?? (home && game && !L.house && pieces?.length ? { x0: pieces[0].x, z0: pieces[0].z, x1: pieces[0].x + pieces[0].w, z1: pieces[0].z + pieces[0].d } : null);
    if ((kind === 'house' || kind === 'shop' || home) && (L.house || lone)) {
      const g = kind === 'shop' || dress ? null : world.garages ? (this.garages.get(L.info.id) ?? null) : home ? null : this.legacyGarage(L, focus);
      const [x0, z0, x1, z1] = L.info.house ?? [lone!.x0, lone!.z0, lone!.x1, lone!.z1];
      const front: [number, number] = L.turned ? [-L.front, 0] : [0, L.front];
      spec = { x0, z0, x1, z1, lot: L.info, home, dressingOnly: home && game, front, door: L.door, garage: g, porches, pieces, roof: L.plot.roof, cover: this.coverOf(L), storeys: L.plot.storeys, lived: kind === 'house' && this.options.lived };
      // (A canopy against the viewed house's wall goes with it in the cutaway.)
      if (spec.dressingOnly && pieces?.some((p) => p.kind === 'canopy')) spec.walled = this.walled;
      if (dress) return { parts: this.outdoors(L, parts), silhouettes, spec, lot, dressed: this.lotDressed(L, dress) };
      // The openings on and round its walls, in the town's order (upstairs ones found on the ground's rows).
      const openings = this.openingGrid
        .near({ x: x0, z: z0, w: x1 - x0, d: z1 - z0 }, 2)
        .sort((a, b) => a.i - b.i)
        .map((e) => e.o);
      const plan = this.planner.silhouettes([spec], openings, world);
      if (spec.dressingOnly) {
        lot = { silhouettes: plan, props: new Geo(), glow: new Geo(), pools: new Geo(), porches: [] };
        // (The porches' lamps go with their walls in the cutaway: drawn apart, a set a wall, `pc/street.ts` `cutLot`.)
        for (const look of new Set(plan.lamps.map((l) => l[5]).filter((l) => l !== undefined))) {
          const porch = { look, props: new Geo(), glow: new Geo(), pools: new Geo() };
          lamps(plan.lamps.filter((l) => l[5] === look), new Painter(porch.props), new Painter(porch.glow), porch.pools);
          lot.porches.push(porch);
        }
        lamps(plan.lamps.filter((l) => !l[5]), new Painter(lot.props), new Painter(lot.glow), lot.pools);
      } else {
        silhouettes = plan;
        lamps(plan.lamps, parts.props, parts.glow, parts.pools);
      }
    }

    // ---- its garden: a car, flower beds ----
    if (kind === 'house' && L.house) this.garden(L, focus, parts);

    // ---- a shop's awning; the things outdoors on it ----
    if (kind === 'shop' && L.house) awning(L, parts);
    if (!home) this.outdoors(L, parts);
    return { parts, silhouettes, spec, lot, dressed };
  }

  /** The things outdoors on a lot not in view (the game's: `StreetSource.look`), into `parts`. */
  private outdoors(L: PlotLayout, parts: Parts): Parts {
    const world = this.world;
    if (this.mode === 'game' && this.options.look) {
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
    return parts;
  }

  /**
   * A lot with a look as drawn closed (`lotLookBuilding`): the building its look says on its walls' box (their outer
   * faces), on its pad, as tall as its real storeys (the walls on each storey of its lot). Open (the focus), the game
   * draws its own walls: the same storeys.
   */
  private lotBuilding(L: PlotLayout, dress: LotDress): TownBuilding {
    const [x0, z0, x1, z1] = L.info.house!;
    const pad = this.padOf(L);
    return { ...lotLookBuilding([x0 - T, z0 - T, x1 + T, z1 + T], dress.look, L.info.storeys ?? 1, pad, () => pad, dress.hours, dress.flats), chunk: this.plotChunk(L) };
  }

  /**
   * `lotBuilding` dressed (`dressBuilding`): its door where the lot's own front door is, on the side its street is on;
   * a place with flats over it, its flats' street door where the next door on that wall is.
   */
  private lotDressed(L: PlotLayout, dress: LotDress): NonNullable<PlotOut['dressed']> {
    const t = this.lotBuilding(L, dress);
    const [x0, z0, x1, z1] = L.info.house!;
    const [fx, fz] = L.turned ? [-L.front, 0] : [0, L.front];
    const [door = null, flats = null] = this.frontDoors(L, fx, fz);
    const [mx, mz] = door ?? [(x0 + x1) / 2 + (fx * (x1 - x0)) / 2, (z0 + z1) / 2 + (fz * (z1 - z0)) / 2];
    const { parts, panes } = dressBuilding(t, dress.look.use, [mx + fx * 6, mz + fz * 6], door, flats);
    return { t, parts, panes };
  }

  /**
   * The middles of a lot's front doors (town metres, on its house's wall towards the street, (`fx`, `fz`)): the doors on
   * the ground floor, those side by side as one (a double door), the nearest the wall's middle first. None on that wall:
   * empty.
   */
  private frontDoors(L: PlotLayout, fx: number, fz: number): [number, number][] {
    const [x0, z0, x1, z1] = L.info.house!;
    const alongX = fz !== 0;
    const line = fz > 0 ? z1 : fz < 0 ? z0 : fx > 0 ? x1 : x0;
    const tiles = new Set<number>();
    for (const { o } of this.openingGrid.near({ x: x0, z: z0, w: x1 - x0, d: z1 - z0 }, 1)) {
      if (o.kind !== 'door' || storeyOfRow(this.world, o.z) !== 0) continue;
      if (alongX ? o.axis === 'h' && o.z === line && o.x >= x0 && o.x < x1 : o.axis === 'v' && o.x === line && o.z >= z0 && o.z < z1) tiles.add(alongX ? o.x : o.z);
    }
    const mid = alongX ? (x0 + x1) / 2 : (z0 + z1) / 2;
    const out: [number, number][] = [];
    for (const a of [...tiles].sort((p, q) => Math.abs(p + 0.5 - mid) - Math.abs(q + 0.5 - mid))) {
      if (!tiles.has(a)) continue;
      let [lo, hi] = [a, a];
      while (tiles.has(lo - 1)) lo--;
      while (tiles.has(hi + 1)) hi++;
      for (let k = lo; k <= hi; k++) tiles.delete(k);
      const c = (lo + hi + 1) / 2;
      out.push(alongX ? [c, line] : [line, c]);
    }
    return out;
  }

  /** A neighbour's garden (see the rules at the top), with lot `focus` in view. */
  private garden(L: PlotLayout, focus: number, parts: Parts): void {
    const world = this.world;
    const { plot, front, turned } = L;
    const h = L.house!;
    const props = parts.props;
    const free = (r: Rect, m = 0) => this.free(r, m, focus);
    // Laid out as the plot is (see `PlotLayout`), drawn on the town.
    const at = (x: number, z: number, yaw = 0, y = 0) => props.at(...townPoint(turned, x, z), townYaw(turned, yaw), y);
    const isFree = (r: Rect, m = 0) => free(townRect(turned, r), m);
    const plant = (kind: FoliageKind, [x, z, sx, sy, yaw]: Inst) => parts.plants[kind].push([...townPoint(turned, x, z), sx, sy, townYaw(turned, yaw)]);
    const putAt = (key: string, x: number, z: number, yaw: number, size?: number) => parts.put(key, ...townPoint(turned, x, z), townYaw(turned, yaw), size);
    const frontWall = front > 0 ? h.z1 + T : h.z0 - T;

    // A game without cars of its own (none in `WorldStructure.cars`: towns of winding roads, whose
    // cars stay parked for now, and games from before cars): a car on some driveways, for good.
    const end = !world.cars && L.drive ? { x: L.drive.x + L.drive.w / 2, z: front > 0 ? L.drive.z : L.drive.z + L.drive.d, front, street: 0 } : null;
    const car = end ? parkCar(plot, end) : null;
    if (car) parked(parts, ...townPoint(turned, car.x, car.z), townYaw(turned, car.yaw), CARS[car.body], CAR_PAINT[car.paint]);

    // (Its fences and hedges are the world's: `fenceOut`.)
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
      this.kerbUnits.set(id, { hash: hashOf(out), out, carry: this.rowCarry(rowKey, variant) });
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
    // (A motorway's and a ramp's sides have no sidewalk: nothing stands on them.)
    if (!hasSidewalks(road)) return out;
    // (On a land the town follows, everything put down on the sidewalk and behind it stands on the ground there.)
    const ground = this.groundAt;
    const at = (x: number, z: number): Parts => {
      const id = chunkAt(x, z);
      let c = out.chunks.get(id);
      if (!c) out.chunks.set(id, (c = { parts: new Parts(this.options.has, ground), sig: 0 }));
      return c.parts;
    };
    const free = (r: Rect, m = 0) => this.free(r, m, focus, out.touched, out.cells);
    const kerb = new Kerb(road, r, side, this.walkStops, this.cuts);
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
    // Mouths of garden paths and driveways on this side: keep them clear (a driveway's wider where
    // the world's cars come and go: they swing across the sidewalk either side of it, `CAR_SWING`).
    const mouths: [number, number][] = [];
    for (const { p } of this.meeting.filter((m) => m.road === r && m.side === side)) {
      const corners = [[p.x, p.z], [p.x + p.w, p.z], [p.x, p.z + p.d], [p.x + p.w, p.z + p.d]].map(([x, z]) => kerb.along(x, z));
      // (`world.cars` there or not, as `layoutKey` keys it: a car bought later finds the mouths clear.)
      const swing = world.cars && Math.min(p.w, p.d) >= DRIVEWAY_MIN_WIDTH ? CAR_SWING : 0;
      mouths.push([Math.min(...corners) - swing, Math.max(...corners) + swing]);
    }
    // The traffic lights' poles on this side (sim-core `traffic::Signal`): nothing else stands there.
    for (const [x, z] of world.lightPoles ?? []) {
      const near = index.nearest(x, z, 8);
      if (near?.road !== r || near.side !== side) continue;
      const s = kerb.along(x, z);
      mouths.push([s - POLE_CLEAR, s + POLE_CLEAR]);
    }
    const clear = (s: number, m: number) => s > 1 && s < kerb.length - 1 && kerb.open(s) && !mouths.some(([a, b]) => s > a - m && s < b + m);
    const used: number[] = [];
    const near = (s: number, m: number) => used.some((u) => Math.abs(u - s) < m);
    // (On a 6 m lane, back from the kerb: cars pull over onto the pavement to pass each other
    // there, no nearer the road's edge than `keep`, `WorldStructure.pullOverStreets`.)
    const keep = world.pullOverStreets?.find((b) => b.road === r)?.keep;
    const kerbD = keep === undefined ? SIDEWALK - KERB - 0.3 : Math.min(SIDEWALK - KERB - 0.3, keep - KERBSIDE_REACH);
    const on = (s: number, d = kerbD) => kerb.at(s, d);

    // The taxi rank at the row's end, on the road sides the town chose (`chooseRanks`), first: past the
    // row's last lot a shelter facing the road behind the pavement (`model.busStop`, as the bus stop it
    // was), its TAXI sign at the kerb past the shelter's far end (away from the row and the street-name
    // sign on its corner); every kerbside thing after keeps `RANK_GAP` clear of both.
    let rank: { shelter: number; sign: number | null } | null = null;
    if (row.length && this.ranks.has(`${r}|${side}`)) {
      const first = side > 0;
      const outward = first ? -1 : 1;
      const cs = (first ? row[0].s0 : row[row.length - 1].s1) + outward * RANK_OUT;
      const [cx, cz] = kerb.at(cs, -RANK_BACK);
      const shelter = { x: cx - 1.6, z: cz - 1.6, w: 3.2, d: 3.2 };
      // (Just behind its own road's pavement: clear of the other roads, a metre at least.)
      if (clear(cs, 1.6) && free(shelter, 0.2) && !index.onRoad(cx, cz, 1, r)) {
        const P = at(cx, cz);
        P.put(PROP.shelter, cx, cz, kerb.yaw(cs));
        blob(P.shadows, cx, cz, 1.7, 0.6, kerb.yaw(cs), 0.35, 0.02);
        rank = { shelter: cs, sign: null };
        const ss = cs + outward * (RANK_HALF + RANK_GAP + 0.1);
        if (clear(ss, 0.3) && !near(ss, RANK_GAP)) {
          used.push(ss);
          const [x, z] = on(ss);
          at(x, z).put(PROP.taxiSign, x, z, kerb.yaw(ss));
          rank.sign = ss;
          mouths.push([ss - RANK_GAP, ss + RANK_GAP]);
        }
        mouths.push([cs - RANK_HALF - RANK_GAP, cs + RANK_HALF + RANK_GAP]);
      }
    }

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
      pool(P.pools, ...on(s, kerbD + 0.5), 4.2, ground);
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
    // A street-name sign on the corner at the row's ends (the taxi rank's clear of it, above).
    if (row.length) {
      const first = side > 0;
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
          pool(P.pools, ...on(s, -0.6), 4.2, ground);
        }
      }
    } else {
      for (let s = 12, run = 0; s < kerb.length - 12; s++) {
        run = empty(s) && kerb.open(s) ? run + 1 : 0;
        if (run < 40 || hash(s, r * 7 + side, 81) > 0.6) continue;
        if (bench(s - 20, -1.3)) run = -40;
      }
    }
    if (rank) {
      const { shelter, sign } = rank;
      out.rank = { shelter, sign, others: used.filter((s) => s !== sign && Math.abs(s - shelter) < RANK_SPAN) };
    }
    for (const c of out.chunks.values()) {
      c.parts.drape();
      c.sig = c.parts.signature();
    }
    return out;
  }

  /**
   * The world's fences in chunk `id` with lot `focus` in view, in their styles: in the game, not the
   * focus lot's (on its edges too) nor the gates (the game draws those, swinging); in the menus'
   * town all of them.
   */
  private fenceKey(id: number, focus: number): FenceKey | null {
    const all = this.fencesByChunk.get(id);
    if (!all?.length) return null;
    const F = this.mode === 'game' ? this.byId.get(focus)?.info : undefined;
    const onLot = (f: FenceEdge) => !!F && f.x >= F.x && f.z >= F.z && f.x + (f.axis === 'h' ? 1 : 0) <= F.x + F.w && f.z + (f.axis === 'v' ? 1 : 0) <= F.z + F.d;
    const list = all.filter((f) => !onLot(f) && (this.mode === 'town' || f.kind !== 'gate'));
    if (!list.length) return null;
    // Each post once: by the chunk of the first fence at its corner (the game draws the focus lot's and the gates' own), in the tallest style there.
    const [cx, cz] = chunkCoords(id);
    const round: FenceEdge[] = [];
    for (let dz = -1; dz <= 1; dz++) for (let dx = -1; dx <= 1; dx++) round.push(...(this.fencesByChunk.get(chunkId(cx + dx, cz + dz)) ?? []));
    const edge = (h: number, f: FenceEdge) => mix(mix(mix(h, f.x * 2 + (f.axis === 'h' ? 1 : 0)), f.z), (f.style ?? 0) * 2 + (f.kind === 'gate' ? 1 : 0));
    let h = list.reduce(edge, 0);
    h = round.reduce(edge, mix(h, round.length));
    return { id, list, round, hash: h };
  }

  /** A chunk's fences as `fenceKey` found them: those kept, or planned now. */
  private fenceOut({ id, list, round, hash }: FenceKey): Parts {
    const cached = this.fenceUnits.get(id);
    if (cached && cached.hash === hash) return cached.out;
    const parts = new Parts(this.options.has);
    const postHere = (x: number, z: number) => this.postOwner.get(`${x}:${z}`) === id;
    const wallAt = (x: number, z: number) => this.wallCornerSet.has(`${x}:${z}`);
    // (On a land the town follows, each on its lot's pad: the higher of two on their shared edge, `BuiltSurface.at`.)
    writeFences(parts.props, list, this.options.fenceStyles ?? [], { gates: null, hedges: parts.plants.hedge, postsFrom: round, postHere, wallAt, ground: this.groundAt });
    this.fenceUnits.set(id, { hash, out: parts });
    return parts;
  }

  /**
   * Sidewalks and kerbs, the road's surface (the game's) and its markings, the avenues' medians and their trees, the
   * motorways' barriers and direction signs, the fountains on roundabouts, the bridges and overpasses: by chunk, planned once.
   */
  private staticParts(): Map<number, StaticPart> {
    if (this.statics) return this.statics;
    const out = new Map<number, StaticPart>();
    const at = (x: number, z: number) => {
      const id = chunkAt(x, z);
      let c = out.get(id);
      if (!c) out.set(id, (c = { parts: new Parts(this.options.has), road: new Geo(), trail: new Geo(), marks: new Geo(), median: new Geo() }));
      return c;
    };
    // Sidewalks, kerbs and the road, the junctions' pads, corners and markings, each piece in the
    // chunk its middle is in. (The streets' and lanes' sidewalks, stopped by the country roads too.)
    const walk = new RoutedGeo((x, z) => at(x, z).parts.walk);
    const shadows = new RoutedGeo((x, z) => at(x, z).parts.shadows).color('#000000');
    // (On a land the town follows, climbing with it: `levels`. Elsewhere a road raised over another climbs to its deck
    // on its own: `deckLevels`.)
    const levels = this.levels;
    const roadLevels: RoadLevels | null = levels ?? deckLevels(this.allRoads);
    const median = new RoutedGeo((x, z) => at(x, z).median);
    roadMeshes(this.walkStops.roads, null, 0, { walk, shadows, ...(this.options.ground ? { median } : {}) }, 'sidewalks', this.junctions, undefined, roadLevels);
    const all = this.surfaced;
    if (this.options.ground) {
      // (Worn between the kerbs; a road without them, from edge to edge.)
      const [road, marks] = [new RoutedGeo((x, z) => at(x, z).road), new RoutedGeo((x, z) => at(x, z).marks)];
      roadMeshes(all, null, 0, { surface: road, marks }, 'surface', this.junctions, (ri) => carriageway(all[ri]) / 2, roadLevels);
      roadMeshes(this.ways.trail, null, 0, { surface: new RoutedGeo((x, z) => at(x, z).trail) }, 'surface', [], undefined, roadLevels);
    }
    // A fountain in the middle of every roundabout, flowers round it.
    for (const road of this.roads) {
      if (road.island !== 'fountain') continue;
      const [cx, cz] = centroid(road.points);
      const P = at(cx, cz).parts;
      // (On its ring's height, on a land the town follows.)
      const y = levels ? levels.road(road, 0) : 0;
      P.put(PROP.fountain, cx, cz, hash(cx, cz, 90) * 6.28, 0.85, 0.85, y);
      for (let k = 0; k < 10; k++) {
        const a = (k / 10) * Math.PI * 2;
        const flower: Inst = [cx + Math.cos(a) * 2.6, cz + Math.sin(a) * 2.6, 0.9 + hash(cx, k, 91) * 0.3, 0.85 + hash(cz, k, 92) * 0.3, a];
        P.plants.flowers.push(lifted(flower, y));
      }
    }
    // Trees down the middle of each avenue's median, none near its junctions.
    for (const road of this.roads) {
      if (!kindInfo(road).median || road.points.length < 2) continue;
      const ls = lengths(road.points);
      const total = ls[ls.length - 1];
      const cut = junctionStretches(road, this.junctions, ls);
      for (let s = MEDIAN_TREE_EVERY / 2; s < total - 2; s += MEDIAN_TREE_EVERY) {
        if (cut.some(([a, b]) => s > a - MEDIAN_TREE_CLEAR && s < b + MEDIAN_TREE_CLEAR)) continue;
        const { p } = pointAt(road.points, s, ls);
        const scale = 0.62 + hash(p[0], p[1], 131) * 0.12;
        at(p[0], p[1]).parts.put(MEDIAN_TREE, p[0], p[1], hash(p[0], p[1], 132) * 6.28, scale, scale, (roadLevels ? roadLevels.road(road, s) : 0) + WALK_Y);
      }
    }
    // The concrete barrier between a motorway's two carriageways, at their height.
    const props = new RoutedGeo((x, z) => at(x, z).parts.props.geo);
    for (const { a, b, runs } of allBarriers(all)) {
      for (const run of runs) barrier(props, run, (q) => (roadLevels ? (roadLevels.road(all[a], q.sa) + roadLevels.road(all[b], q.sb)) / 2 : 0) + ROAD_Y);
    }
    // The motorway's direction signs by its exits and ramps (`signSpots`).
    for (const spot of signSpots(all, roadLevels)) at(spot.x, spot.z).parts.put(MOTORWAY_SIGN, spot.x, spot.z, spot.yaw, 1, 1, spot.y);
    // Bridges and causeways: a stone deck under each span over the water, parapets along it (a country road that is
    // a bridge, and the bridges the land's brushes made on any road); a motorway's and a ramp's in concrete, as their
    // overpasses (`RoadPlan.raised`) are; on a world with no built surface, walls down the sides of the ramps up to
    // them (`deckLevels`).
    const pairs = motorwayPairs(all);
    all.forEach((road, ri) => {
      const concrete = isSeparated(road.kind);
      // (On a land the town follows, its deck as the road climbs over it, its piers down to the bed or the ground.)
      const heightsOf = (s0: number) => {
        if (!roadLevels) return undefined;
        const stops = levelStops(road).flatMap((s) => (s > s0 && s < s0 + 1e9 ? [s - s0] : []));
        return { deck: (u: number) => roadLevels.road(road, s0 + u), bed: (x: number, z: number) => (levels ? levels.surface.landAt(x, z) : 0), under: (x: number, z: number) => (levels ? levels.groundUnder(x, z) : 0), stops };
      };
      const pair = pairs.get(ri);
      const inner = pair && { inner: pair.side, innerApron: pair.verge / 2 };
      const deck = (s0: number, s1: number, solid = false) => {
        const h = heightsOf(s0);
        // (Its piers and abutments on the ground with every deck taken away: `BuiltSurface.groundUnder`.)
        // (No pier on the roads it passes over: each its width and a margin either side, more where it crosses at a slant.)
        const clear = (road.crosses ?? []).flatMap((c): [number, number][] => {
          const half = ((all[c.road]?.width ?? 7) / 2 + 2) * 1.6;
          return c.s > s0 && c.s < s1 ? [[c.s - s0 - half, c.s - s0 + half]] : [];
        });
        overpass(stretchOf(road, s0, s1), road.width, (x, z) => at(x, z).parts.props, h && { deck: h.deck, ground: h.under, stops: h.stops.filter((u) => u < s1 - s0) }, { ...inner, solid, clear });
      };
      for (const [s0, s1] of bridgeSpans(road)) {
        if (concrete) deck(s0, s1);
        else {
          const h = heightsOf(s0);
          bridge(stretchOf(road, s0, s1), road.width, (x, z) => at(x, z).parts.props, h && { deck: h.deck, bed: h.bed, stops: h.stops.filter((u) => u < s1 - s0) });
        }
      }
      const raised = raisedSpans(road);
      for (const [s0, s1] of raised) deck(s0, s1);
      // (No built surface: the ramps up to a deck stand on walls, `deckLevels`.)
      if (!levels && roadLevels) for (const [s0, s1] of without(liftedStretches(road, roadLevels), [...raised, ...bridgeSpans(road)])) deck(s0, s1, true);
    });
    this.statics = out;
    return out;
  }

  /** The world's own cars, parked, in the menus' town (the game draws them coming and going: `pc/cars.ts`), by chunk. */
  private carParts(): Map<number, Parts> {
    if (this.cars) return this.cars;
    const out = new Map<number, Parts>();
    const levels = this.levels;
    if (this.world.cars && this.mode === 'town') {
      for (const c of this.world.cars) {
        const id = chunkAt(c.x, c.z);
        let P = out.get(id);
        if (!P) out.set(id, (P = new Parts(this.options.has)));
        parked(P, c.x, c.z, c.yaw, CARS[c.body % CARS.length], CAR_PAINT[c.paint % CAR_PAINT.length], levels ? levels.at(c.x, c.z) : 0, carAge(c.age));
      }
    }
    return (this.cars = out);
  }

  /** A hash of the street's own in chunk `id` (`staticParts`: what it looks like), or null if it has none. */
  private staticSig(id: number): number | null {
    const known = this.staticSigs.get(id);
    if (known !== undefined) return this.withCars(id, known);
    const st = this.staticParts().get(id);
    let sig: number | null = null;
    if (st) {
      sig = hashNumbers(st.parts.walk.pos, st.parts.signature());
      for (const geo of [st.road, st.trail, st.marks, st.median]) sig = hashNumbers(geo.pos, sig);
    }
    this.staticSigs.set(id, sig);
    return this.withCars(id, sig);
  }

  /** `sig` with chunk `id`'s parked cars in it (none: as it is). */
  private withCars(id: number, sig: number | null): number | null {
    const cars = this.carParts().get(id);
    return cars ? cars.signature() ^ (sig ?? 0x5bd1e995) : sig;
  }
  private readonly staticSigs = new Map<number, number | null>();

  /**
   * Chunk `id` far off (the far LOD, past the chunks streamed in): its houses as boxes with their
   * roofs (`HousePlanner.farHouse`), the focus lot's left out, in one plain-finish geometry, with
   * what it was made from. Null when it has none.
   */
  farChunk(id: number, focus: number): { key: string; geo: Geo } | null {
    const houses = (this.plotsByChunk().get(id) ?? []).filter((L) => L.info.house && L.info.id !== focus && (L.base === 'house' || L.base === 'shop'));
    if (!houses.length) return null;
    const key = `${id}|${houses.map((L) => `${L.info.id}:${L.info.house!.join(',')}:${L.info.roof?.join(',') ?? ''}:${L.info.storeys ?? 1}:${this.coverOf(L)}${this.dresses.has(L.info.id) ? `:${hashString(JSON.stringify(this.dresses.get(L.info.id)))}` : ''}`).join(';')}${this.levels ? `|v${this.levels.surface.version(id)}` : ''}`;
    const cached = this.far.get(id);
    if (cached?.key === key) return cached;
    const geo = new Geo();
    for (const L of houses) {
      // (A named place's lot as its look has it, on its pad: `lotBuilding`.)
      const dress = this.dresses.get(L.info.id);
      if (dress) {
        farBuilding(geo, this.lotBuilding(L, dress));
        continue;
      }
      // (On a land the town follows, each on its pad.)
      const dy = this.padOf(L);
      const into = dy ? new Geo() : geo;
      this.planner.farHouse(into, { x: L.info.x, z: L.info.z, house: L.info.house!, roof: L.info.roof, storeys: L.info.storeys, cover: this.coverOf(L) });
      if (dy) geo.merge(into, dy);
    }
    const out = { key, geo };
    this.far.set(id, out);
    return out;
  }
  private readonly far = new Map<number, { key: string; geo: Geo }>();

  /**
   * Every lot's house the far LOD draws (`farChunk`'s, the focus lot's left out), with the chunk it is planned in and
   * its pad: the town view draws them all at once (`pc/townView.ts`), those of the chunks not streamed in.
   */
  townHouses(focus: number): { chunk: number; plot: PlotInfo; pad: number; cover: number; dress?: LotDress }[] {
    const out: { chunk: number; plot: PlotInfo; pad: number; cover: number; dress?: LotDress }[] = [];
    for (const [chunk, list] of this.plotsByChunk()) {
      for (const L of list) {
        if (!L.info.house || L.info.id === focus || (L.base !== 'house' && L.base !== 'shop')) continue;
        const dress = this.dresses.get(L.info.id);
        out.push({ chunk, plot: L.info, pad: this.padOf(L), cover: this.coverOf(L), ...(dress ? { dress } : {}) });
      }
    }
    return out;
  }

  /** The wall covering most of a lot's house's outside wears (`HousePlanner.exteriorCover`: 0, its own look), for its far versions. */
  private coverOf(L: PlotLayout): number {
    return L.info.house ? this.planner.exteriorCover(this.world, L.info.house, L.info.storeys ?? 1) : 0;
  }

  /** The named places' lots' looks as a key (`townHouses` made again when it changes). */
  get dressKey(): string {
    return this.dressesKey;
  }

  /** How many times the land under the town has moved its roads or pads (`BuiltSurface.moves`; 0 on flat land). */
  get surfaceMoves(): number {
    return this.levels?.surface.moves ?? 0;
  }

  /** What chunk `id` is planned from with lot `focus` in view (`ChunkPlan.key`), without planning it. */
  chunkKey(id: number, focus: number): string {
    // (Kept until the world changes what the street stands among: `setWorld` moves `version` on.)
    const memo = `${id}|${focus}`;
    const known = this.keys.get(memo);
    // (On a land the town follows, kept too until the heights in it move: `BuiltSurface.version`.)
    const moved = this.levels?.surface.version(id) ?? 0;
    if (known && known.version === this.version && known.moved === moved) return known.key;
    const gathered = this.gather(id, focus);
    this.keys.set(memo, { version: this.version, moved, key: gathered.key });
    // (A chunk's key is most often asked for just before it is planned: `chunk` takes what was gathered.)
    this.gathered = { memo, version: this.version, gathered };
    return gathered.key;
  }
  private readonly keys = new Map<string, { version: number; moved: number; key: string }>();
  /** What the last `chunkKey` gathered, for `chunk` (once). */
  private gathered: { memo: string; version: number; gathered: ReturnType<TownStreet['gather']> } | null = null;
  /** Moved on whenever `setWorld` takes a world that changes anything the street is planned from. */
  private version = 0;

  private gather(id: number, focus: number): { key: string; plots: PlotKey[]; kerbs: Parts[]; fence: FenceKey | null; ground: () => GroundLayer[] } {
    const keys: string[] = [];
    const plots: PlotKey[] = [];
    for (const L of this.plotsByChunk().get(id) ?? []) {
      const k = this.plotKey(L, focus);
      keys.push(`p${k.key}:${k.hash}`);
      plots.push(k);
    }
    const kerbs: Parts[] = [];
    for (const r of this.rows.keys()) {
      const c = this.kerbOf(r, focus).chunks.get(id);
      if (!c) continue;
      // (By what it looks like, not by its road's index: a road removed before it moves that.)
      keys.push(`k${c.sig}`);
      kerbs.push(c.parts);
    }
    const fence = this.fenceKey(id, focus);
    if (fence) keys.push(`e${id}:${fence.hash}`);
    // The street's own in it (sidewalks, the road, fountains, bridges, parked cars), by what they look like.
    const own = this.staticSig(id);
    if (own !== null) keys.push(`s${own}`);
    // The town's paved ground in it (`streetGround.ts`).
    const paved = streetGroundKey(this.world, id, focus, !!this.options.ground);
    if (paved) keys.push(`g${paved}`);
    // Ground layers: garden paths and fields off the focus lot (the lot draws its own, on its ground). Their key now,
    // their geometry only when the chunk is planned (`chunk`): most keys are asked for to find the chunk unchanged.
    const layers: [string, Rect[], string, number][] = [];
    if (this.options.ground) {
      const F = this.mode === 'game' ? this.byId.get(focus)?.info : undefined;
      const onLot = (r: Rect) => !!F && r.x >= F.x && r.z >= F.z && r.x + r.w <= F.x + F.w && r.z + r.d <= F.z + F.d;
      const mine = (r: Rect) => chunkAt(r.x + r.w / 2, r.z + r.d / 2) === id && !onLot(r);
      for (const [name, all, material, y] of this.groundRects()) {
        const list = all.filter(mine);
        if (!list.length) continue;
        layers.push([name, list, material, y]);
        keys.push(`${name}:${list.length}:${hashNumbers(list.flatMap((r) => [r.x, r.z, r.w, r.d]))}`);
      }
    }
    const ground = (): GroundLayer[] => {
      const out: GroundLayer[] = [];
      if (!this.options.ground) return out;
      const road = this.staticParts().get(id)?.road;
      if (road && !road.empty) out.push({ name: 'streets', material: 'material.road', geo: road, toned: true });
      const trail = this.staticParts().get(id)?.trail;
      // (Toned white, which draws them as they are untoned, so they go into one draw with the garden paths: `groundDraw`.)
      if (trail && !trail.empty) out.push({ name: 'trails', material: 'material.path', geo: trail, toned: true });
      const marks = this.staticParts().get(id)?.marks;
      if (marks && !marks.empty) out.push({ name: 'roadMarks', material: 'paint', geo: marks, paint: true });
      // (An avenue's median's grass: the land's grass material.)
      const median = this.staticParts().get(id)?.median;
      if (median && !median.empty) out.push({ name: 'medians', material: MEDIAN_MATERIAL, geo: median, toned: true });
      const all = new Map(this.groundRects().map(([name, rects]) => [name, rects]));
      for (const [name, list, material, y] of layers) {
        const geo = new Geo();
        const toned = name === 'paths';
        if (toned) pavedRects(geo, list, all.get(name)!, y);
        else for (const r of list) geo.poly([[r.x, y, r.z], [r.x + r.w, y, r.z], [r.x + r.w, y, r.z + r.d], [r.x, y, r.z + r.d]], [0, 1, 0], [[0, 0], [1, 0], [1, 1], [0, 1]]);
        // (On a land the town follows, laid over the ground: a path on its lot's pad, across the verge from it to
        // its road's height where it meets it, a field on its farm's pad.)
        if (this.groundAt) geo.drape(this.groundAt);
        out.push({ name, material, geo, toned });
      }
      return out;
    };
    // (On a land the town follows, the heights in it: `BuiltSurface.chunkSig`. Not how often they moved, which a
    // surface solved afresh for another town of the same land counts from nothing again.)
    const moved = this.levels ? `|h${this.levels.surface.chunkSig(id)}` : '';
    return { key: `${id}|${this.options.lived}${moved}|${keys.join(';')}`, plots, kerbs, fence, ground };
  }

  /** Plans chunk `id` with lot `focus` in view (`NO_FOCUS`: none; in the menus' town the player's lot is the focus). */
  chunk(id: number, focus: number): ChunkPlan {
    const kept = this.gathered;
    this.gathered = null;
    const g = kept && kept.memo === `${id}|${focus}` && kept.version === this.version ? kept.gathered : this.gather(id, focus);
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
      ground: g.ground(),
      buildings: [],
    };
    const st = this.staticParts().get(id);
    if (st) mergeParts(plan, st.parts);
    const cars = this.carParts().get(id);
    if (cars) mergeParts(plan, cars);
    for (const k of g.plots) {
      const p = this.plotOut(k);
      // (On a land the town follows, each lot's share on its pad; the viewed lot's own, `lot`, stays where the game
      // draws its lot, at its own 0: `pc/space.ts` `setDatum`.)
      const dy = this.padOf(k.L);
      if (p.spec) plan.specs.push(p.spec);
      if (p.silhouettes) mergeSilhouettes(plan.silhouettes, p.silhouettes, dy);
      // (A lot with a look closed: built on its pad already, its windows lit by the hour, `litAt`.)
      if (p.dressed) {
        mergeSilhouettes(plan.silhouettes, { parts: p.dressed.parts, homes: [], lamps: [] });
        plan.buildings.push({ t: p.dressed.t, panes: p.dressed.panes });
      }
      if (p.lot) plan.lot = p.lot;
      mergeParts(plan, p.parts, dy);
    }
    for (const p of g.kerbs) mergeParts(plan, p);
    if (g.fence) mergeParts(plan, this.fenceOut(g.fence));
    // The town's paved ground: the centre, the courts and yards, the car parks and their bays (`streetGround.ts`).
    streetGround(plan, this.world, id, this.levels?.surface, focus, !!this.options.ground);
    return plan;
  }
}


/** A lot's place (its rectangle), as chunk keys name it: the same lot in another town of the same layout is the same. */
function placeOf(p: { x: number; z: number; w: number; d: number }): string {
  return `${p.x},${p.z},${p.w},${p.d}`;
}

/** Geometry that goes into the chunk each polygon's middle is in. */
class RoutedGeo extends Geo {
  constructor(private readonly target: (x: number, z: number) => Geo) {
    super();
  }
  override poly(points: readonly V3[], n: V3, uvs?: readonly (readonly [number, number])[], alphas?: readonly number[], tints?: readonly V3[]): this {
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
    g.poly(points, n, uvs, alphas, tints);
    return this;
  }
}

/**
 * A stone bridge's measures (m): its deck reaches `apron` past the road's edges each side, from `under` (below
 * the water line) to just under the road; its parapets stand `parapet` high along its edges, `wall` thick, with a
 * coping on top and a post at each end; a pier's nose (a cutwater, carried up the parapet's outside as a
 * buttress: the deck stands only a few hand-widths over the water) juts `nose` out of each side every `pier` m,
 * `noseWide` across, its top `noseDrop` lower at its tip than under the coping.
 */
export const BRIDGE = { apron: 2, under: -0.6, top: ROAD_Y - 0.004, parapet: 0.9, wall: 0.4, coping: 0.08, post: 0.55, postUp: 1.1, pier: 8, nose: 0.9, noseWide: 1.4, noseDrop: 0.35 } as const;
const BRIDGE_STONE = '#A1998C';
/** A pier reaches this far (m) into the bed under it, on a land the town follows. */
const PIER_SINK = 0.3;
const BRIDGE_COPING = '#BCB4A6';

/**
 * A bridge's (or a causeway's) stone along `deck` (its middle line, `bridgeDeck`) for a road `width` wide: the
 * deck, its parapets and their end posts, and a pier's nose each side every `BRIDGE.pier` m or a little less (its
 * length shared out evenly; none at its ends, on the banks), in pieces at most that long, each written into the
 * painter `into` gives for where it is (the street's chunks). A deck that bends is drawn a straight run at a time
 * (a generated road's points are a metre apart), each run's pieces reaching a little past its ends round a bend.
 */
export function bridge(
  deck: readonly (readonly [number, number])[],
  width: number,
  into: (x: number, z: number) => Painter,
  /**
   * On a land the town follows: the road's height `u` m along the deck from its start, and the land's (the water's
   * bed) at a point; the deck climbs with the road, the piers' noses reach down to the bed. `stops`: where along the
   * deck (m from its start) the road's surface takes its heights, running straight between them (`levelStops`): the
   * deck is cut there too, so its top follows the road's and never stands over it where the road bends up or down
   * (a piece from pier to pier, straight, stood over the asphalt where the road came down off a high bank onto it).
   * None: at 0.
   */
  heights?: { deck: (u: number) => number; bed: (x: number, z: number) => number; stops?: readonly number[] },
): void {
  const B = BRIDGE;
  const half = width / 2 + B.apron;
  deckPieces(deck, B.pier, into, heights?.deck, heights?.stops, ({ p, L, mx, mz, dx, dz, ya, yb, first, last, opens, cut, u0 }) => {
    // The deck (no underside: it is under the water), closed at the bridge's two ends.
    const ends = `${first ? '' : 'nz'}${last ? '' : 'pz'}`;
    p.paint(BRIDGE_STONE, 0.9).box(-half, B.under, -L / 2, half, B.top, L / 2, `ny${ends}`);
    for (const side of [-1, 1]) {
      const [o0, o1] = side < 0 ? [-half, -half + B.wall] : [half - B.wall, half];
      p.paint(BRIDGE_STONE, 0.9).box(o0, B.top, -L / 2, o1, B.top + B.parapet - B.coping, L / 2, `ny${ends}`);
      p.paint(BRIDGE_COPING, 0.8).box(o0 - 0.05, B.top + B.parapet - B.coping, -L / 2, o1 + 0.05, B.top + B.parapet, L / 2, ends);
      // A post at each end of the parapet.
      for (const [end, at] of [[first, -L / 2], [last, L / 2]] as const) {
        if (!end) continue;
        const x = side * (half - B.post / 2);
        p.paint(BRIDGE_COPING, 0.85).box(x - B.post / 2, B.top, at - B.post / 2, x + B.post / 2, B.top + B.postUp, at + B.post / 2, 'ny');
      }
      // A pier's nose where the piece between two piers begins (not at the bridge's start: its abutment is on the
      // bank), down to the bed under it on a land the town follows.
      if (opens && cut > 1e-6) {
        const at = cut - u0 - L / 2;
        let down: number = BRIDGE.under;
        if (heights) {
          const [nx, nz] = [mx + dx * at + dz * side * (half + BRIDGE.nose / 2), mz + dz * at - dx * side * (half + BRIDGE.nose / 2)];
          down = Math.min(BRIDGE.under, heights.bed(nx, nz) - PIER_SINK - ((ya + yb) / 2 + (at * (yb - ya)) / (L || 1)));
        }
        nose(p, side, half, at, down);
      }
    }
  }, { bend: half });
}

/** An overpass's parapet's concrete. */
const OVERPASS_PARAPET = '#BCB9B1';

/**
 * An overpass (`RoadPlan.raised`: a road over another; a motorway's bridge over the water too) along `deck` (its middle
 * line, `stretchOf`) for a road `width` wide: its concrete slab, its parapets, a pier every `OVERPASS.pier` m or a little
 * less (its length shared out evenly; none at its ends) down to the ground, an abutment under each end, in pieces at
 * most that long, each written into the painter `into` gives for where it is (the street's chunks), a bend drawn a
 * straight run at a time as `bridge` draws one.
 *
 * `heights`: the road's height `u` m along the deck from its start, and the ground's at a point (none: both at 0, the
 * deck on the ground and nothing under it). `stops`: where the road's surface takes its heights (`levelStops`): the deck
 * is cut there too. `inner`: the side (+1: the deck's left, looking along it) its pair runs on (a motorway's
 * carriageway): no parapet there (the barrier between them stands on the deck), the slab reaching only `innerApron`
 * past the road's edge (half the verge between them: the two slabs meet). `solid`: a ramp up to a deck on a world with
 * no built surface to bank it (`motorway.ts` `deckLevels`): walls down its sides to the ground instead of what stands
 * under a deck. `clear`: stretches along it (m from its start) where no pier stands (the roads it passes over): one
 * that would stands each side of them instead (`pierStations`).
 */
export function overpass(
  deck: readonly (readonly [number, number])[],
  width: number,
  into: (x: number, z: number) => Painter,
  heights?: { deck: (u: number) => number; ground: (x: number, z: number) => number; stops?: readonly number[] },
  options: { inner?: 1 | -1; innerApron?: number; solid?: boolean; clear?: readonly (readonly [number, number])[] } = {},
): void {
  const O = OVERPASS;
  const solid = !!options.solid;
  // (Where its piers stand, once its length is known: `pierStations`.)
  let piers: readonly number[] = [];
  const isPier = (u: number) => u > 1e-6 && piers.some((v) => Math.abs(v - u) < 1e-6);
  // (Local +x is the deck's left: `Painter.at` turns +z to (sin yaw, cos yaw), +x to (cos yaw, -sin yaw).)
  const reach = (side: 1 | -1) => width / 2 + (options.inner === side ? (options.innerApron ?? 0) : O.apron);
  const [left, right] = [reach(1), reach(-1)];
  const groundAt = (x: number, z: number) => (heights ? heights.ground(x, z) : 0);
  deckPieces(deck, O.pier, into, heights?.deck, heights?.stops, ({ p, L, mx, mz, dx, dz, ya, yb, first, last, opens, cut, run }) => {
    const ends = `${first ? '' : 'nz'}${last ? '' : 'pz'}`;
    // (`under`: the ground `at` m along the piece and `x` across, from the deck's road.)
    const under = (at: number, x: number) => {
      const [wx, wz] = [mx + dx * at + dz * x, mz + dz * at - dx * x];
      return groundAt(wx, wz) - ((ya + yb) / 2 + (at * (yb - ya)) / (L || 1));
    };
    // The slab (its top under the road's surface, which covers it); on a solid ramp, a wall down each outer side.
    // (Its top: a lid `O.lid` under the road, unseen, for its shadow. Open, the sun's map saw only its sides, its
    // underside facing away, culled: the shadow on the ground under it was its outline, zig-zagging over the banks.)
    if (!solid) {
      p.paint(OVERPASS_CONCRETE, 0.92).box(-right, -O.slab, -L / 2, left, ROAD_Y - 0.004, L / 2, `py${ends}`);
      p.poly([[-right, -O.lid, -L / 2], [left, -O.lid, -L / 2], [left, -O.lid, L / 2], [-right, -O.lid, L / 2]], [0, 1, 0]);
    }
    else {
      const foot = Math.min(under(-L / 2, 0), under(L / 2, 0)) - O.sink;
      for (const side of [-1, 1] as const) {
        if (options.inner === side) continue;
        const [w0, w1] = side > 0 ? [left - O.foot, left] : [-right, -right + O.foot];
        p.paint(OVERPASS_CONCRETE, 0.92).box(w0, foot, -L / 2, w1, ROAD_Y - 0.004, L / 2, `pyny${ends}`);
      }
    }
    // The parapets, on its outer edges: a foot, the face above it.
    for (const side of [-1, 1] as const) {
      if (options.inner === side) continue;
      const edge = side > 0 ? left : right;
      const span = (a: number, b: number): [number, number] => (side > 0 ? [edge - b, edge - a] : [-edge + a, -edge + b]);
      const [f0, f1] = span(0, O.foot);
      p.paint(OVERPASS_PARAPET, 0.9).box(f0, ROAD_Y - 0.004, -L / 2, f1, O.footHigh, L / 2, `ny${ends}`);
      const [t0, t1] = span(0.06, 0.06 + O.top);
      p.box(t0, O.footHigh, -L / 2, t1, O.parapet, L / 2, `ny${ends}`);
    }
    // What stands under it down to the ground: a pier where a piece begins at one (`pierStations`), an abutment
    // under each end.
    if (solid) return;
    if (opens && isPier(run + cut)) {
      const at = -L / 2;
      const xs = width >= O.wide ? [-width / 4, width / 4] : [0];
      if (xs.every((x) => under(at, x) < -O.low)) {
        p.paint(OVERPASS_CONCRETE, 0.92).box(-right + 0.3, -O.slab - O.capDeep, at - O.cap / 2, left - 0.3, -O.slab, at + O.cap / 2, 'py');
        for (const x of xs) p.box(x - O.column / 2, under(at, x) - O.sink, at - O.column / 2, x + O.column / 2, -O.slab - O.capDeep, at + O.column / 2, 'py');
      }
    }
    for (const [end, at, s] of [[first, -L / 2, 1], [last, L / 2, -1]] as const) {
      if (!end || Math.max(under(at, -right), under(at, left)) > -O.low) continue;
      const foot = Math.min(under(at, -right), under(at, 0), under(at, left)) - O.sink;
      const [z0, z1] = s > 0 ? [at, at + O.abutment] : [at - O.abutment, at];
      p.paint(OVERPASS_CONCRETE, 0.92).box(-right, foot, z0, left, -O.slab, z1, 'py');
    }
  }, { piers: (total, n) => (piers = pierStations(total, n, options.clear ?? [])) });
}

/**
 * Where an overpass `total` m long stands on piers (m along it): every `total / n` m, none at its ends; one that falls on
 * a road it passes over (`clear`: the stretches over them) stands beside that road instead, one each side of it (each
 * where it is still clear of the others and `PIER_END` m in from the deck's ends). A deck over a road with nothing
 * beside it ran from bank to bank unheld.
 */
export function pierStations(total: number, n: number, clear: readonly (readonly [number, number])[]): number[] {
  const free = (u: number) => u >= PIER_END && u <= total - PIER_END && !clear.some(([a, b]) => u > a + 1e-6 && u < b - 1e-6);
  const out: number[] = [];
  for (let k = 1; k < n; k++) {
    const u = (k * total) / n;
    const over = clear.find(([a, b]) => u > a + 1e-6 && u < b - 1e-6);
    if (!over) out.push(u);
    else for (const v of over) if (free(v)) out.push(v);
  }
  out.sort((a, b) => a - b);
  return out.filter((u, k) => k === 0 || u - out[k - 1] > OVERPASS.cap * 2);
}
/** A pier stands at least this far (m) in from either end of its deck. */
const PIER_END = 2;

/** A piece of a deck as `deckPieces` walks it. */
interface DeckPiece {
  /** Its painter: at its middle, at the road's height there, sheared to the road's heights at its ends. */
  p: Painter;
  /** Its length (m), its middle, the way along it (a unit), the road's heights at its ends (m). */
  L: number;
  mx: number;
  mz: number;
  dx: number;
  dz: number;
  ya: number;
  yb: number;
  /** Whether it is the deck's first piece, its last. */
  first: boolean;
  last: boolean;
  /**
   * Whether it is the first piece of its cut (from a pier, or a run's start, to the next), where along its straight run
   * that cut begins (m), where along the deck its run begins (m) and where along its run the piece itself begins (m:
   * before the cut's start, round a bend).
   */
  opens: boolean;
  cut: number;
  run: number;
  u0: number;
}

/**
 * A deck along `deck` (its middle line) in pieces, each to `piece` with the painter `into` gives for where it is: a
 * straight run at a time (a generated road's points are a metre apart), each run cut every `every` m or a little less
 * (the deck's length shared out evenly), where `piers` stand (given the deck's length and how many lengths of `every`
 * it is shared into), and where the road's surface takes its heights (`stops`, m along the deck: `levelStops`). Each
 * piece climbs with the road (`deckY`, m along the deck; none: at 0). `bend`: how far out from its middle the deck
 * reaches, its runs' pieces reaching a little past their ends round a bend to close the gap on its outside (0: not).
 */
function deckPieces(
  deck: readonly (readonly [number, number])[],
  every: number,
  into: (x: number, z: number) => Painter,
  deckY: ((u: number) => number) | undefined,
  stops: readonly number[] | undefined,
  piece: (q: DeckPiece) => void,
  { piers, bend = 0 }: { piers?: (total: number, n: number) => readonly number[]; bend?: number } = {},
): void {
  const runs = straightRuns(deck, 0.03);
  const ls = [0];
  for (let i = 1; i < runs.length; i++) ls.push(ls[i - 1] + Math.hypot(runs[i][0] - runs[i - 1][0], runs[i][1] - runs[i - 1][1]));
  const total = ls[ls.length - 1];
  if (total < 0.01) return;
  const n = Math.max(1, Math.ceil(total / every - 1e-6));
  const P = total / n;
  const stations = piers?.(total, n) ?? [];
  const dirOf = (i: number): [number, number] => {
    const l = ls[i + 1] - ls[i];
    return [(runs[i + 1][0] - runs[i][0]) / l, (runs[i + 1][1] - runs[i][1]) / l];
  };
  // (How far a run's pieces reach past its end round the bend into the next: the gap on the bend's outside, half each.)
  const past = (i: number): number => {
    if (!bend || i <= 0 || i >= runs.length - 1) return 0;
    const [a, b] = [dirOf(i - 1), dirOf(i)];
    const turn = Math.acos(Math.max(-1, Math.min(1, a[0] * b[0] + a[1] * b[1])));
    return bend * Math.tan(Math.min(turn, 1) / 2) + 0.01;
  };
  for (let i = 0; i + 1 < runs.length; i++) {
    const len = ls[i + 1] - ls[i];
    const [dx, dz] = dirOf(i);
    // (Local +z along the deck: `Painter.at` turns +z to (sin yaw, cos yaw).)
    const yaw = Math.atan2(dx, dz);
    // (Cut every `P` m, the pieces no longer than that, and where a pier stands.)
    const inner: number[] = [];
    for (let k = Math.floor(ls[i] / P + 1e-6) + 1; k * P < ls[i + 1] - 1e-6; k++) inner.push(k * P - ls[i]);
    for (const v of stations) if (v > ls[i] + 1e-6 && v < ls[i + 1] - 1e-6) inner.push(v - ls[i]);
    inner.sort((a, b) => a - b);
    const cuts = [0, ...inner.filter((u, k) => k === 0 || u - inner[k - 1] > 0.05), len];
    for (let c = 0; c + 1 < cuts.length; c++) {
      const [from, to] = [cuts[c] - (c === 0 ? past(i) : 0), cuts[c + 1] + (c + 2 === cuts.length ? past(i + 1) : 0)];
      // (Cut again where the road's surface takes its heights, on a land the town follows.)
      const bounds = [from, ...(stops ?? []).map((u) => u - ls[i]).filter((u) => u > from + 0.05 && u < to - 0.05), to];
      for (let k = 0; k + 1 < bounds.length; k++) {
        const [u0, u1] = [bounds[k], bounds[k + 1]];
        const L = u1 - u0;
        const [mx, mz] = [runs[i][0] + dx * (u0 + L / 2), runs[i][1] + dz * (u0 + L / 2)];
        // (Climbing with the road: at its height at the piece's middle, sheared to its heights at its ends.)
        const [ya, yb] = deckY ? [deckY(ls[i] + u0), deckY(ls[i] + u1)] : [0, 0];
        const p = into(mx, mz).at(mx, mz, yaw, (ya + yb) / 2, L > 0 ? (yb - ya) / L : 0);
        const [first, last] = [i === 0 && c === 0 && k === 0, i + 2 === runs.length && c + 2 === cuts.length && k + 2 === bounds.length];
        piece({ p, L, mx, mz, dx, dz, ya, yb, first, last, opens: k === 0, cut: cuts[c], run: ls[i], u0 });
      }
    }
  }
}

/** A polyline with the points dropped where it runs on straight: each run's points within `most` m of the line from its start to its end. */
function straightRuns(pts: readonly (readonly [number, number])[], most: number): [number, number][] {
  const out: [number, number][] = [[pts[0][0], pts[0][1]]];
  let from = 0;
  const fits = (to: number) => {
    const [[ax, az], [bx, bz]] = [pts[from], pts[to]];
    const l = Math.hypot(bx - ax, bz - az) || 1;
    for (let k = from + 1; k < to; k++) if (Math.abs((pts[k][0] - ax) * (bz - az) - (pts[k][1] - az) * (bx - ax)) / l > most) return false;
    return true;
  };
  for (let i = 1; i < pts.length; i++) {
    if (i + 1 < pts.length && fits(i + 1)) continue;
    const [x, z] = pts[i];
    const [px, pz] = out[out.length - 1];
    if (Math.hypot(x - px, z - pz) > 1e-6) out.push([x, z]);
    from = i;
  }
  return out;
}

/** A pier's cutwater on `side` of a deck `half` m wide, at `at` along it: a three-sided prism jutting out of the deck's side up to the parapet's coping, its top sloping down to its tip. */
function nose(p: Painter, side: number, half: number, at: number, down: number = BRIDGE.under): void {
  const B = BRIDGE;
  const [y0, y1, drop] = [down, B.top + B.parapet - B.coping, B.noseDrop];
  const w = B.noseWide / 2;
  const [x0, x1] = [side * half, side * (half + B.nose)];
  p.paint(BRIDGE_STONE, 0.9);
  // Out from the deck's side to its point and back, its top falling `drop` to the point; its point cut and its two
  // faces' arrises eased (`easedPrism`), as the street's boxes are. (Round as `Geo.poly` winds it: any way.)
  const outline: [number, number][] = [
    [x0, at - w],
    [x1, at],
    [x0, at + w],
  ];
  const top = (x: number) => y1 - (drop * (x - x0)) / (x1 - x0);
  easedPrism(p, outline, y0, top, [true, true, false], [false, true, false]);
}

/** How far a cut corner of an eased prism is cut back along each side (m): a pier's point, blunted. */
const PRISM_CUT = 0.03;

/**
 * An upright prism over the convex `outline` (x, z, round it either way) from `y0` up to `top(x, z)`, its top a plane:
 * side `i` (from point `i` to the next) eased at its top where `eased[i]` (its face stops `EASE` short, a strip up to
 * the top drawn in from it), corner `i` cut across `PRISM_CUT` back where `cut[i]` (its arris eased the other way).
 * No bottom. Drawn by `p` in its paint.
 */
function easedPrism(p: Painter, outline: readonly [number, number][], y0: number, top: (x: number, z: number) => number, eased: readonly boolean[], cut: readonly boolean[]): void {
  const c = EASE;
  // The corners cut: each a short side of its own, eased if either side beside it is.
  const pts: [number, number][] = [];
  const easedSide: boolean[] = [];
  const n = outline.length;
  for (let i = 0; i < n; i++) {
    const [prev, at, next] = [outline[(i + n - 1) % n], outline[i], outline[(i + 1) % n]];
    if (!cut[i]) {
      pts.push(at);
      easedSide.push(eased[i]);
      continue;
    }
    const toward = (q: readonly [number, number]): [number, number] => {
      const [dx, dz] = [q[0] - at[0], q[1] - at[1]];
      const l = Math.sqrt(dx * dx + dz * dz) || 1;
      return [at[0] + (dx / l) * PRISM_CUT, at[1] + (dz / l) * PRISM_CUT];
    };
    pts.push(toward(prev), toward(next));
    easedSide.push(eased[(i + n - 1) % n] || eased[i], eased[i]);
  }
  const m = pts.length;
  // Which way is out: the outline's winding.
  let area = 0;
  for (let i = 0; i < m; i++) area += pts[i][0] * pts[(i + 1) % m][1] - pts[(i + 1) % m][0] * pts[i][1];
  const turn = area > 0 ? -1 : 1;
  const outOf = (i: number): [number, number] => {
    const [a, b] = [pts[i], pts[(i + 1) % m]];
    const [dx, dz] = [b[0] - a[0], b[1] - a[1]];
    const l = Math.sqrt(dx * dx + dz * dz) || 1;
    return [(turn * dz) / l, (-turn * dx) / l];
  };
  // The top drawn in `c` from each eased side (the lines either side of a corner, met).
  const inner = pts.map((at, i): [number, number] => {
    const j = (i + m - 1) % m;
    const [na, nb] = [outOf(j), outOf(i)];
    const [da, db] = [easedSide[j] ? c : 0, easedSide[i] ? c : 0];
    // Solve (q - at) · na = -da, (q - at) · nb = -db.
    const det = na[0] * nb[1] - na[1] * nb[0];
    if (Math.abs(det) < 1e-9) return [at[0] - na[0] * Math.max(da, db), at[1] - na[1] * Math.max(da, db)];
    const [ra, rb] = [-da, -db];
    return [at[0] + (ra * nb[1] - rb * na[1]) / det, at[1] + (rb * na[0] - ra * nb[0]) / det];
  });
  const P = (q: readonly [number, number], y: number): V3 => [q[0], y, q[1]];
  for (let i = 0; i < m; i++) {
    const k = (i + 1) % m;
    const [a, b] = [pts[i], pts[k]];
    const [o0, o1] = outOf(i);
    const sink = (q: readonly [number, number], s: number) => top(q[0], q[1]) - (s ? c : 0);
    // The face, up to where its top's chamfer begins (or the top, if not eased), and at a corner where an eased side
    // meets one that is not, the triangle up to the top between them.
    const ea = easedSide[i] || easedSide[(i + m - 1) % m];
    const eb = easedSide[i] || easedSide[k];
    p.poly([P(a, y0), P(b, y0), P(b, sink(b, eb ? 1 : 0)), P(a, sink(a, ea ? 1 : 0))], [o0, 0, o1]);
    if (!easedSide[i]) {
      if (ea) p.poly([P(a, sink(a, 1)), P(a, top(a[0], a[1])), P(inner[i], top(inner[i][0], inner[i][1]))], [o0, 0, o1]);
      if (eb) p.poly([P(b, sink(b, 1)), P(inner[k], top(inner[k][0], inner[k][1])), P(b, top(b[0], b[1]))], [o0, 0, o1]);
      continue;
    }
    const q: V3[] = [P(a, sink(a, 1)), P(b, sink(b, 1)), P(inner[k], top(inner[k][0], inner[k][1])), P(inner[i], top(inner[i][0], inner[i][1]))];
    p.poly(q, facing(q[0], q[1], q[2], [o0, 1, o1]));
  }
  const cap = inner.map((q) => P(q, top(q[0], q[1])));
  p.poly(cap, facing(cap[0], cap[1], cap[2], [0, 1, 0]));
}

/** A parked car on a driveway, its contact shadow. */
function parked(parts: Parts, x: number, z: number, yaw: number, body: CarBody, paint: string, y = 0, age: CarAge = 1): void {
  paintCar(parts.props.at(x, z, yaw, 0.015 + y), body, paint, false, true, age);
  blob(parts.shadows, x, z, body.wid * 0.62, body.len * 0.56, yaw, 0.55, 0.03 + y);
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

/**
 * The lots' looks (`WorldMeta.lotLooks`, by plot index: a named place's lot's, a flats building's) by plot id, each with
 * its place's hours (the `WorldMeta.places` entry on that plot), and whether flats stand on it (`WorldStructure.flats`).
 * A home's lot has none (it is drawn as the house it is).
 */
export function lotDresses(world: Pick<WorldStructure, 'plots' | 'meta' | 'flats'>): Map<number, LotDress> {
  const out = new Map<number, LotDress>();
  const looks = world.meta?.lotLooks;
  if (!looks) return out;
  const hours = new Map<number, NonNullable<TownPlace['hours']>>();
  for (const p of world.meta?.places ?? []) if (p.hours) hours.set(p.plot, p.hours);
  const flats = new Set((world.flats ?? []).map((f) => f.of));
  for (const [k, look] of Object.entries(looks)) {
    const i = Number(k);
    const plot = world.plots[i];
    // (A place's lot, or a flats building's.)
    if (!plot || !look || !(plot.public || plot.kind === 'flats')) continue;
    const h = hours.get(i);
    out.set(plot.id, { look, ...(h ? { hours: h } : {}), ...(flats.has(plot.id) ? { flats: true as const } : {}) });
  }
  return out;
}

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

/** The outward normal (x, z) of a garage's door facing `rot` (as a thing's: 0 towards +z, 1 +x, 2 -z, 3 -x). */
export function garageDoor(rot: number): [number, number] {
  return ([[0, 1], [1, 0], [0, -1], [-1, 0]] as [number, number][])[((rot % 4) + 4) % 4];
}

/** A garage door's outward normal as a facing (`garageDoor` back). */
function doorRot([x, z]: readonly [number, number]): number {
  return z > 0 ? 0 : x > 0 ? 1 : z < 0 ? 2 : 3;
}

/**
 * The garage plot `plotId` could have at the inner end of its driveway, beside its house, its door
 * towards the street: where the town puts one (`garageAt`, on any house, not only seven in ten),
 * clear of `blocked` (things, on the town) by a tile. On the town, with the way its door faces
 * (`rot`) and the driveway. Null: no house, no driveway, or no room for it there.
 */
export function drivewayGarage(world: WorldStructure, plotId: number, blocked: readonly Rect[]): { garage: Omit<GarageInfo, 'plot' | 'rot'>; rot: number; drive: Rect } | null {
  const i = world.plots.findIndex((p) => p.id === plotId);
  if (i < 0) return null;
  const streets = (world.meta?.streets ?? []).filter((s) => s.w >= s.d);
  const L = layout({ ...world, plots: [world.plots[i]] } as WorldStructure, streets, world.meta?.paths ?? [], world.meta?.facing?.[i])[0];
  if (!L?.drive || !L.house) return null;
  const g = garageAt(L.plot, L.house, L.drive, L.front, blocked.map((b) => localRect(L.turned, b)));
  if (!g) return null;
  return { garage: townBox(L.turned, g), rot: doorRot(L.turned ? [-L.front, 0] : [0, L.front]), drive: townRect(L.turned, L.drive) };
}

/** A soft pool of lamplight on the ground (on a land the town follows, `ground`: laid over it, in `POOL_CELLS` × `POOL_CELLS` pieces). */
function pool(g: Geo, x: number, z: number, r: number, ground?: (x: number, z: number) => number): void {
  const y = 0.055;
  if (!ground) {
    g.poly([[x - r, y, z - r], [x + r, y, z - r], [x + r, y, z + r], [x - r, y, z + r]], [0, 1, 0], [[0, 0], [1, 0], [1, 1], [0, 1]]);
    return;
  }
  const n = POOL_CELLS;
  const P = (i: number, j: number): V3 => {
    const [px, pz] = [x - r + (2 * r * i) / n, z - r + (2 * r * j) / n];
    return [px, y + ground(px, pz), pz];
  };
  for (let j = 0; j < n; j++) {
    for (let i = 0; i < n; i++) g.poly([P(i, j), P(i + 1, j), P(i + 1, j + 1), P(i, j + 1)], [0, 1, 0], [[i / n, j / n], [(i + 1) / n, j / n], [(i + 1) / n, (j + 1) / n], [i / n, (j + 1) / n]]);
  }
}
const POOL_CELLS = 4;
