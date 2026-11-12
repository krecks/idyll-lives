/**
 * The town view's buildings, engine-neutral (docs/design/living-town.md, "Town-scale drawing"; drawn by
 * `pc/townView.ts`): the backdrop palette, how a backdrop building (`WorldMeta.backdrop`) or a far lot's
 * house is fitted to the unit meshes (a rectangle along its ridge or its longest side, its walls from
 * under its ground to its storeys, its roof's rise), and the unit meshes themselves.
 */

import type { BackdropBuilding, BackdropUse } from '../../core/protocol';
import { Geo, facing, type V3 } from './geometry';

/** Window styles: none (a barn), a house's, a shop's (a shopfront on the ground floor), flats' and offices' (closer bays). */
export const TOWN_STYLE = { none: 0, house: 1, shop: 2, flats: 3 } as const;

/** The roofs the unit meshes have, in their draws' order. */
export const ROOFS = ['gable', 'hip', 'flat', 'mansard'] as const;
export type RoofKind = (typeof ROOFS)[number];

/** A storey (m), as the contract has it. */
const STOREY = 3;
/** A pitched roof's pitch (as the game's houses: 34°), a mansard's rise at most (m), a flat roof's parapet (m). */
const PITCH = Math.tan((34 * Math.PI) / 180);
const MANSARD_RISE = 3.2;
const PARAPET = 0.7;
/** How far walls go under the lowest corner's ground (m): no gap under a building on a slope. */
const SINK = 0.4;

/**
 * The backdrop palette: wall and roof colour pairs (sRGB), taken by a building's `look` modulo its
 * length. Warm town tones beside the game's own houses (`shared/house.ts` SIDING, BRICK, ROOF): most
 * roofs red and orange tile, some brown, a few grey slate; walls cream, sand, ochre, render and brick.
 */
export const BACKDROP_PALETTE: readonly (readonly [string, string])[] = [
  ['#E9DFCB', '#A9503A'],
  ['#F1E7D3', '#B8623E'],
  ['#D8C39C', '#9C4634'],
  ['#E2C79A', '#8C4E3C'],
  ['#C9A27A', '#B45A3A'],
  ['#B66A4F', '#5C626A'],
  ['#9C5A43', '#A9553A'],
  ['#ECE6D8', '#6B4A3A'],
  ['#DCC9A8', '#BE6E44'],
  ['#E6D2B0', '#94442F'],
  ['#C8B79A', '#4F5258'],
  ['#EADBA8', '#B25438'],
  ['#D9A88A', '#A04C38'],
  ['#E8E4DA', '#B05E3E'],
  ['#C9D8C4', '#8C4E3C'],
  ['#E3CFA0', '#C27448'],
  ['#E6C9B8', '#9E4A36'],
  ['#BCD0E0', '#7A6A5A'],
];
/** Some uses keep their own look: a barn's boards and tin roof, a church's stone and slate, a flat roof's felt and gravel. */
const BARN = [['#8A3F2E', '#7B7F80'], ['#6E5040', '#6A6E70'], ['#9A8A70', '#7E5A44']] as const;
const CHURCH = ['#BDB5A5', '#55595F'] as const;
const FLAT_ROOF = '#8C8780';

/** A building's window style by what it is, and the share of its windows lit at night. */
const STYLE: Record<BackdropUse, [number, number]> = {
  house: [TOWN_STYLE.house, 0.42],
  terrace: [TOWN_STYLE.house, 0.45],
  flats: [TOWN_STYLE.flats, 0.5],
  shop: [TOWN_STYLE.shop, 0.55],
  cafe: [TOWN_STYLE.shop, 0.6],
  restaurant: [TOWN_STYLE.shop, 0.7],
  office: [TOWN_STYLE.flats, 0.18],
  school: [TOWN_STYLE.flats, 0.08],
  church: [TOWN_STYLE.house, 0.3],
  hall: [TOWN_STYLE.flats, 0.25],
  workshop: [TOWN_STYLE.house, 0.15],
  farm: [TOWN_STYLE.house, 0.4],
  barn: [TOWN_STYLE.none, 0],
};

/** A building as drawn: a rectangle along (ux, uz) `w` long and `d` deep round (cx, cz), its walls from `base` to `top`, its roof `rise` over them. */
export interface TownBuilding {
  roof: RoofKind;
  cx: number;
  cz: number;
  ux: number;
  uz: number;
  w: number;
  d: number;
  base: number;
  top: number;
  rise: number;
  /** sRGB, 0..1. */
  wall: [number, number, number];
  roofColor: [number, number, number];
  /** Its floor's height (m): the walls under it are its plinth. */
  floor: number;
  style: number;
  lit: number;
  seed: number;
}

/** A far lot's house as the street has it (`Street.townHouses`) and how it looks (`HousePlanner.farLook`). */
export interface TownLotHouse {
  /** The lot's id and the chunk it is planned in. */
  id: number;
  chunk: number;
  /** Its walls' box (m, x0 z0 x1 z1, round the walls' outer faces). */
  box: readonly [number, number, number, number];
  storeys: number;
  wall: string;
  roof: string;
  shape: 'gable' | 'hip' | 'flat';
  /** Its roof's rise over the walls (m). */
  rise: number;
}

const hex = (h: string): [number, number, number] => {
  const v = parseInt(h.slice(1), 16);
  return [((v >> 16) & 255) / 255, ((v >> 8) & 255) / 255, (v & 255) / 255];
};

/** A number from 0 to 1 for integers `a`, `b` (stable: a building's variation). */
function hash2(a: number, b: number): number {
  let h = Math.imul(a ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul(b + 0x632be5ab, 0xc2b2ae35);
  h = Math.imul(h ^ (h >>> 15), 0x2c1b3c6d);
  h ^= h >>> 13;
  return (h >>> 0) / 4294967296;
}

/** `c` a little lighter or darker and warmer or cooler by `t` and `s` (0..1 each). */
function vary(c: readonly number[], t: number, s: number, amount = 0.07): [number, number, number] {
  const k = 1 + (t - 0.5) * 2 * amount;
  const warm = (s - 0.5) * amount * 0.5;
  return [Math.min(1, Math.max(0, c[0] * k + warm)), Math.min(1, Math.max(0, c[1] * k)), Math.min(1, Math.max(0, c[2] * k - warm))];
}

/**
 * A backdrop building as drawn: its footprint's rectangle (exact for a rectangle, the one round it along its
 * ridge or its longest side otherwise), its walls from under its lowest corner's ground (`ground`) to its
 * storeys' height over its floor (`y`), its roof and its colours.
 */
export function backdropBuilding(b: BackdropBuilding, index: number, ground: (x: number, z: number) => number): TownBuilding {
  const p = b.poly;
  // The rectangle's direction: the ridge's, else the longest side's.
  let ux = 1;
  let uz = 0;
  if (b.ridge !== undefined && (b.roof === 'gable' || b.roof === 'mansard')) [ux, uz] = [Math.cos(b.ridge), Math.sin(b.ridge)];
  else {
    let best = -1;
    for (let i = 0; i < p.length; i++) {
      const [a, c] = [p[i], p[(i + 1) % p.length]];
      const l = Math.hypot(c[0] - a[0], c[1] - a[1]);
      if (l > best) [best, ux, uz] = [l, (c[0] - a[0]) / l, (c[1] - a[1]) / l];
    }
  }
  const [vx, vz] = [-uz, ux];
  let [u0, u1, v0, v1] = [Infinity, -Infinity, Infinity, -Infinity];
  for (const [x, z] of p) {
    const u = x * ux + z * uz;
    const v = x * vx + z * vz;
    [u0, u1, v0, v1] = [Math.min(u0, u), Math.max(u1, u), Math.min(v0, v), Math.max(v1, v)];
  }
  const [cu, cv] = [(u0 + u1) / 2, (v0 + v1) / 2];
  const cx = cu * ux + cv * vx;
  const cz = cu * uz + cv * vz;
  let [w, d] = [Math.max(1, u1 - u0), Math.max(1, v1 - v0)];
  let lowest = Infinity;
  for (const [x, z] of p) lowest = Math.min(lowest, ground(x, z));
  const floor = Number.isFinite(b.y) ? b.y : lowest;
  const base = Math.min(floor, lowest) - SINK;
  const storeys = Math.max(1, Math.min(6, Math.round(b.storeys)));
  const top = floor + storeys * STOREY;
  // A hip's ridge runs along its longer side.
  if (b.roof === 'hip' && d > w) {
    [ux, uz] = [vx, vz];
    [w, d] = [d, w];
  }
  const roof = b.roof;
  const rise = roof === 'flat' ? PARAPET : roof === 'mansard' ? Math.min(MANSARD_RISE, d * 0.4) : (d / 2) * PITCH;
  // Its colours: the palette's (or its use's own), each building a little different.
  const t = hash2(index, b.look);
  const s = hash2(b.look, index + 7);
  let [wallHex, roofHex] = BACKDROP_PALETTE[((b.look % BACKDROP_PALETTE.length) + BACKDROP_PALETTE.length) % BACKDROP_PALETTE.length];
  if (b.use === 'barn') [wallHex, roofHex] = BARN[b.look % BARN.length];
  else if (b.use === 'church') [wallHex, roofHex] = CHURCH;
  let roofColor = hex(roofHex);
  if (roof === 'flat') {
    const g = hex(FLAT_ROOF);
    roofColor = [g[0] * 0.8 + roofColor[0] * 0.2, g[1] * 0.8 + roofColor[1] * 0.2, g[2] * 0.8 + roofColor[2] * 0.2];
  }
  const [style, lit] = STYLE[b.use] ?? STYLE.house;
  return { roof, cx, cz, ux, uz, w, d, base, top, rise, wall: vary(hex(wallHex), t, s), roofColor: vary(roofColor, s, t, 0.1), floor, style, lit, seed: hash2(index * 31 + 5, b.storeys) };
}

/** A far lot's house as drawn: its walls' box on the ground at (`ground`), the street's far LOD's look. */
export function lotBuilding(h: TownLotHouse, ground: (x: number, z: number) => number): TownBuilding {
  const [x0, z0, x1, z1] = h.box;
  const along = x1 - x0 >= z1 - z0;
  const [w, d] = along ? [x1 - x0, z1 - z0] : [z1 - z0, x1 - x0];
  const cx = (x0 + x1) / 2;
  const cz = (z0 + z1) / 2;
  const lowest = Math.min(ground(x0, z0), ground(x1, z0), ground(x0, z1), ground(x1, z1));
  const top = h.storeys * STOREY;
  return {
    roof: h.shape,
    cx,
    cz,
    ux: along ? 1 : 0,
    uz: along ? 0 : 1,
    w,
    d,
    base: Math.min(0, lowest) - SINK,
    top,
    rise: h.shape === 'flat' ? 0.25 : h.rise,
    wall: hex(h.wall),
    roofColor: hex(h.roof),
    floor: 0,
    style: TOWN_STYLE.house,
    lit: 0.4,
    seed: hash2(h.id, 11),
  };
}

// ---- the unit meshes ------------------------------------------------------------------------

/** Vertex colour of a wall part (red 1) or a roof part (red 0), toned by `tone` (green; sRGB). */
const part = (wall: boolean, tone = 1) => `#${wall ? 'FF' : '00'}${Math.round(tone * 255).toString(16).padStart(2, '0')}00`;
/** The plain finish's rough, not metal (`farHouse`'s). */
export const FINISH = 0.4;
/** A roof's overhang (unit: a share of the footprint's side). */
const EAVE = 0.035;
/** A hip's ridge end and a mansard's break, a share of the side in from the eaves. */
const HIP_RIDGE = 0.2;
const MANSARD_IN = 0.13;
const MANSARD_BREAK = 0.72;

/**
 * The unit meshes (`walls` and each roof): a footprint from -0.5 to 0.5 in x (along the ridge) and z,
 * walls from 0 to 1 up, a roof from 0 (the eaves) to 1 (its rise), scaled per copy.
 */
export function unitMeshes(): Record<'walls' | RoofKind, Geo> {
  const walls = new Geo().color(part(true), FINISH).box(-0.5, 0, -0.5, 0.5, 1, 0.5, 'ny py');
  const e = 0.5 + EAVE;
  // A gable: two slopes, the ends wall.
  const gable = new Geo().color(part(false), FINISH);
  for (const s of [-1, 1]) {
    const q: V3[] = [[-e, 0, s * e], [e, 0, s * e], [e, 1, 0], [-e, 1, 0]];
    gable.poly(q, facing(q[0], q[1], q[2], [0, 1, s]));
  }
  gable.color(part(true, 0.97), FINISH);
  for (const s of [-1, 1]) gable.poly([[s * 0.5, 0, -0.5], [s * 0.5, 0, 0.5], [s * 0.5, 1, 0]], [s, 0, 0]);
  // A hip: four slopes to a short ridge, its ends a touch darker (they face the sun less often from above).
  const hip = new Geo().color(part(false), FINISH);
  for (const s of [-1, 1]) {
    const q: V3[] = [[-e, 0, s * e], [e, 0, s * e], [HIP_RIDGE, 1, 0], [-HIP_RIDGE, 1, 0]];
    hip.poly(q, facing(q[0], q[1], q[2], [0, 1, s]));
  }
  hip.color(part(false, 0.93), FINISH);
  for (const s of [-1, 1]) {
    const q: V3[] = [[s * e, 0, -e], [s * e, 0, e], [s * HIP_RIDGE, 1, 0]];
    hip.poly(q, facing(q[0], q[1], q[2], [s, 1, 0]));
  }
  // A flat roof: a parapet round a felt deck.
  const flat = new Geo();
  const t = 0.025;
  flat.color(part(true), FINISH).box(-0.5, 0, -0.5, 0.5, 1, 0.5, 'ny py');
  flat.color(part(true, 0.9), FINISH);
  flat.poly([[-0.5, 1, -0.5], [0.5, 1, -0.5], [0.5, 1, -0.5 + t], [-0.5, 1, -0.5 + t]], [0, 1, 0]);
  flat.poly([[-0.5, 1, 0.5 - t], [0.5, 1, 0.5 - t], [0.5, 1, 0.5], [-0.5, 1, 0.5]], [0, 1, 0]);
  flat.poly([[-0.5, 1, -0.5], [-0.5 + t, 1, -0.5], [-0.5 + t, 1, 0.5], [-0.5, 1, 0.5]], [0, 1, 0]);
  flat.poly([[0.5 - t, 1, -0.5], [0.5, 1, -0.5], [0.5, 1, 0.5], [0.5 - t, 1, 0.5]], [0, 1, 0]);
  flat.color(part(true, 0.75), FINISH);
  const i = 0.5 - t;
  flat.box(-i, 0.4, -i, i, 1, i, 'ny py').poly([[-i, 0.4, -i], [i, 0.4, -i], [i, 1, -i], [-i, 1, -i]], [0, 0, 1]);
  // (The inner faces of the parapet face inwards: its own box's faces point out, so a second set, turned.)
  for (const [n, q] of [
    [[0, 0, 1], [[-i, 0.4, -i], [i, 0.4, -i], [i, 1, -i], [-i, 1, -i]]],
    [[0, 0, -1], [[-i, 0.4, i], [i, 0.4, i], [i, 1, i], [-i, 1, i]]],
    [[1, 0, 0], [[-i, 0.4, -i], [-i, 0.4, i], [-i, 1, i], [-i, 1, -i]]],
    [[-1, 0, 0], [[i, 0.4, -i], [i, 0.4, i], [i, 1, i], [i, 1, -i]]],
  ] as [V3, V3[]][])
    flat.poly(q, n);
  flat.color(part(false), FINISH).poly([[-i, 0.4, -i], [i, 0.4, -i], [i, 0.4, i], [-i, 0.4, i]], [0, 1, 0]);
  // A mansard: steep lower slopes all round to a break, then a low hip.
  const mansard = new Geo().color(part(false, 0.95), FINISH);
  const b = 0.5 - MANSARD_IN;
  const y = MANSARD_BREAK;
  for (const s of [-1, 1]) {
    const qz: V3[] = [[-e, 0, s * e], [e, 0, s * e], [b, y, s * b], [-b, y, s * b]];
    mansard.poly(qz, facing(qz[0], qz[1], qz[2], [0, 0.4, s]));
    const qx: V3[] = [[s * e, 0, -e], [s * e, 0, e], [s * b, y, b], [s * b, y, -b]];
    mansard.poly(qx, facing(qx[0], qx[1], qx[2], [s, 0.4, 0]));
  }
  mansard.color(part(false), FINISH);
  for (const s of [-1, 1]) {
    const q: V3[] = [[-b, y, s * b], [b, y, s * b], [HIP_RIDGE * 0.6, 1, 0], [-HIP_RIDGE * 0.6, 1, 0]];
    mansard.poly(q, facing(q[0], q[1], q[2], [0, 1, s]));
    const r: V3[] = [[s * b, y, -b], [s * b, y, b], [s * HIP_RIDGE * 0.6, 1, 0]];
    mansard.poly(r, facing(r[0], r[1], r[2], [s, 1, 0]));
  }
  return { walls, gable, hip, flat, mansard };
}

