/**
 * The town view's buildings, engine-neutral (drawn by `pc/townView.ts`): the town's palette, how a lot's building
 * closed (a named place's, a flats building's: `WorldMeta.lotLooks`) or a far lot's house is fitted to the unit meshes
 * (a rectangle along its longer side, its walls from under its ground to its storeys, its roof's rise), and the unit
 * meshes themselves.
 */

import type { BuildingUse, LotLook, TownPlace } from '../../core/protocol';
import { Geo, facing, type V3 } from './geometry';

/** Window styles: none (no windows), a house's, a shop's (a shopfront on the ground floor), flats' and offices' (closer bays). */
export const TOWN_STYLE = { none: 0, house: 1, shop: 2, flats: 3 } as const;

/** The roofs the unit meshes have, in their draws' order. */
export const ROOFS = ['gable', 'hip', 'flat', 'mansard'] as const;
export type RoofKind = (typeof ROOFS)[number];

/** A storey (m), as the contract has it. */
export const STOREY = 3;
/** A pitched roof's pitch (as the game's houses: 34°), a mansard's rise at most (m), a flat roof's parapet (m). */
export const PITCH = Math.tan((34 * Math.PI) / 180);
const MANSARD_RISE = 3.2;
export const PARAPET = 0.7;
/** How far walls go under the lowest corner's ground (m): no gap under a building on a slope. */
const SINK = 0.4;

/**
 * The town's palette: wall and roof colour pairs (sRGB), taken by a building's `look` modulo its
 * length. Warm town tones beside the game's own houses (`shared/house.ts` SIDING, BRICK, ROOF): most
 * roofs red and orange tile, some brown, a few grey slate; walls cream, sand, ochre, render and brick.
 */
export const PALETTE: readonly (readonly [string, string])[] = [
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
/** Some uses keep their own look: a church's stone and slate, a flat roof's felt and gravel. */
const CHURCH = ['#BDB5A5', '#55595F'] as const;
const FLAT_ROOF = '#8C8780';

/**
 * How much of their palette colour the town's pitched roofs keep (sRGB): as bright as the palette has them they
 * stood out beside the game's own houses' roofs as if lit from inside, most of all at dusk and by night.
 */
const ROOF_TONE = 0.72;

/**
 * A building's window style by what it is, and the share of its windows lit at night: a place's while it is open
 * (`litAt`: an open shop's front is lit, the windows over it half as often), a home's in the evening (`homeLights`).
 */
export const STYLE: Record<BuildingUse, [number, number]> = {
  house: [TOWN_STYLE.house, 0.45],
  terrace: [TOWN_STYLE.house, 0.48],
  flats: [TOWN_STYLE.flats, 0.5],
  shop: [TOWN_STYLE.shop, 0.95],
  cafe: [TOWN_STYLE.shop, 0.95],
  restaurant: [TOWN_STYLE.shop, 0.95],
  office: [TOWN_STYLE.flats, 0.18],
  school: [TOWN_STYLE.flats, 0.08],
  church: [TOWN_STYLE.house, 0.3],
  hall: [TOWN_STYLE.flats, 0.25],
  workshop: [TOWN_STYLE.house, 0.15],
  farm: [TOWN_STYLE.house, 0.4],
};
/** Homes: their lights go by the hour. */
const HOMES: ReadonlySet<BuildingUse> = new Set<BuildingUse>(['flats', 'farm']);
/** What a closed place keeps of its lights (a night light, a lit sign), and the share of a home's lights on in the small hours and at breakfast. */
const CLOSED = 0.06;
const SMALL_HOURS = 0.15;
const BREAKFAST = 0.7;

/**
 * The share of a home's evening lights on at `minute` of the day: all of them until ten, going out one by one till
 * half past midnight, a few left through the small hours, on again from half past five. In steps of a twentieth (the
 * buildings are written again at each).
 */
export function homeLights(minute: number): number {
  const h = (((minute / 60) % 24) + 24) % 24;
  const ramp = (a: number, b: number, x: number) => Math.min(1, Math.max(0, (x - a) / (b - a)));
  let k = 1;
  if (h >= 22) k = 1 - (1 - SMALL_HOURS) * ramp(22, 24.5, h);
  else if (h < 0.5) k = 1 - (1 - SMALL_HOURS) * ramp(22, 24.5, h + 24);
  else if (h < 5.5) k = SMALL_HOURS;
  else if (h < 12) k = SMALL_HOURS + (BREAKFAST - SMALL_HOURS) * ramp(5.5, 7, h);
  return Math.round(k * 20) / 20;
}

/**
 * The share of `b`'s windows lit at `minute` of the day on `weekday` (0 = Monday): a place with opening hours its own
 * while it is open and next to none when it is closed (but its flats', if it has flats over it: those by the hour, as
 * a flats building's), a home its own by the hour (`homeLights`), the rest their own. `part`: of a place with flats
 * over it, the place's own windows only (by its hours) or its flats' only (by the hour, as a flats building's); the
 * near street lights the two apart (`buildingDress.ts` `BuildingDress.panes`), the town view as one.
 */
export function litAt(b: Pick<TownBuilding, 'lit' | 'home' | 'hours' | 'flats'>, minute: number, weekday: number, part?: 'place' | 'flats'): number {
  if (part === 'flats') return STYLE.flats[1] * homeLights(minute);
  if (b.hours) {
    const { open, close, days } = b.hours;
    const m = ((minute % 1440) + 1440) % 1440;
    // (Hours past midnight when `close` isn't after `open`, as the tags read them: `ui/town/places.ts` `openState`.)
    const on = close > open ? m >= open && m < close && (days >> weekday) & 1 : (m >= open && (days >> weekday) & 1) || (m < close && (days >> ((weekday + 6) % 7)) & 1);
    if (on) return b.lit;
    return b.flats && part !== 'place' ? Math.max(b.lit * CLOSED, STYLE.flats[1] * homeLights(minute)) : b.lit * CLOSED;
  }
  return b.home ? b.lit * homeLights(minute) : b.lit;
}

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
  /** The share of its windows lit at night at most, and now (`litAt`; absent: at most). */
  lit: number;
  now?: number;
  /** A home (its lights go by the hour), or a place's opening hours (lit while open): `litAt`. */
  home?: boolean;
  hours?: { open: number; close: number; days: number };
  /** A place with flats over it (`lotLookBuilding`): its flats lit as homes while it is closed, its upper storeys dressed as flats'. */
  flats?: boolean;
  seed: number;
  /** A number of its own (0..1) for its shopfront's fascia. */
  accent: number;
  /**
   * Its lot's street chunk (`shared/chunks.ts`): where the game's street draws it near (dressed, `buildingDress.ts`,
   * or as the house it is) while that chunk is streamed in; the town view draws it only while it isn't.
   */
  chunk?: number;
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
  /** Its lot's pad (m): the level its floor stands at (`TownStreet.padOf`; 0 on flat land). */
  pad: number;
  /**
   * A lot with a look (`WorldMeta.lotLooks`: a named place's, a flats building's): how it looks from outside, its
   * place's hours, and whether it has flats over its place (`lotLookBuilding`).
   */
  look?: LotLook;
  hours?: TownPlace['hours'];
  flats?: boolean;
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
 * A number of a building's own, from its footprint's corners (its colours' and its windows' variation): the same
 * wherever its lot is in the town's list, so a lot added or taken away elsewhere leaves the rest as they were.
 */
function footprintSeed(poly: readonly (readonly [number, number])[]): number {
  let h = 0x811c9dc5;
  for (const [x, z] of poly) h = Math.imul(Math.imul(h ^ Math.round(x * 100), 16777619) ^ Math.round(z * 100), 16777619);
  return (h >>> 0) % 1_000_003;
}

// ---- a lot's building, closed ------------------------------------------------------------------

/**
 * The most storeys a building has: the world's (`build.storeys` in the content). A building closed looks exactly as
 * tall as its real storeys, never taller.
 */
export const MOST_STOREYS = 3;

/**
 * A lot's building closed (`WorldMeta.lotLooks`: a named place's, a flats building's), as drawn near
 * (`buildingDress.ts`) and far alike: on its walls' box (`box`, their outer faces), its floor its lot's pad, its walls
 * down to the lowest of `ground` round it, as many storeys as its real ones (`storeys`, the lot's own walls', at most
 * `MOST_STOREYS`), its roof's ridge along its longer side (a hip's always is). Its colours the palette's by its look,
 * a little different for each footprint. Its windows lit by its place's `hours` (`litAt`), or by the hour as a home's
 * (a flats building); `flats`: a place with flats over it, whose flats are lit by the hour while it is closed.
 */
export function lotLookBuilding(
  box: readonly [number, number, number, number],
  look: LotLook,
  storeys: number,
  pad: number,
  ground: (x: number, z: number) => number,
  hours?: TownPlace['hours'],
  flats = false,
): TownBuilding {
  const [x0, z0, x1, z1] = box;
  const poly: [number, number][] = [
    [x0, z0],
    [x0, z1],
    [x1, z1],
    [x1, z0],
  ];
  const index = footprintSeed(poly);
  // Along its longer side (along z when they are the same), its ridge and a hip's too.
  const along = x1 - x0 > z1 - z0;
  const [ux, uz] = along ? [1, 0] : [0, 1];
  const w = Math.max(1, along ? x1 - x0 : z1 - z0);
  const d = Math.max(1, along ? z1 - z0 : x1 - x0);
  const [cx, cz] = [(x0 + x1) / 2, (z0 + z1) / 2];
  let lowest = Infinity;
  for (const [x, z] of poly) lowest = Math.min(lowest, ground(x, z));
  const floor = pad;
  const base = Math.min(floor, lowest) - SINK;
  const real = Math.max(1, Math.min(MOST_STOREYS, Math.round(storeys)));
  const top = floor + real * STOREY;
  const roof = look.roof;
  const rise = roof === 'flat' ? PARAPET : roof === 'mansard' ? Math.min(MANSARD_RISE, d * 0.4) : (d / 2) * PITCH;
  // Its colours: the palette's (or its use's own), each building a little different.
  const t = hash2(index, look.look);
  const s = hash2(look.look, index + 7);
  let [wallHex, roofHex] = PALETTE[((look.look % PALETTE.length) + PALETTE.length) % PALETTE.length];
  if (look.use === 'church') [wallHex, roofHex] = CHURCH;
  let roofColor = hex(roofHex).map((c) => c * ROOF_TONE) as [number, number, number];
  if (roof === 'flat') {
    const g = hex(FLAT_ROOF);
    const r = hex(roofHex);
    roofColor = [g[0] * 0.8 + r[0] * 0.2, g[1] * 0.8 + r[1] * 0.2, g[2] * 0.8 + r[2] * 0.2];
  }
  const [style, lit] = STYLE[look.use] ?? STYLE.house;
  const home = HOMES.has(look.use);
  return {
    roof,
    cx,
    cz,
    ux,
    uz,
    w,
    d,
    base,
    top,
    rise,
    wall: vary(hex(wallHex), t, s),
    roofColor: vary(roofColor, s, t, 0.1),
    floor,
    style,
    lit,
    home,
    ...(hours ? { hours } : {}),
    ...(flats && !home && real > 1 ? { flats: true } : {}),
    seed: hash2(index * 31 + 5, real),
    accent: hash2(index * 17 + 3, look.look),
  };
}

/** sRGB 0..1 as a hex colour. */
const hexOf = (c: readonly number[]) => `#${c.map((v) => Math.round(Math.min(1, Math.max(0, v)) * 255).toString(16).padStart(2, '0')).join('')}`;

/**
 * Building `t` as a plain box and its roof into `geo`, in its colours (the street's far LOD of a lot with a look,
 * `TownStreet.farChunk`, as the town view's unit meshes have it: `unitMeshes`).
 */
export function farBuilding(geo: Geo, t: TownBuilding): void {
  const g = new Geo();
  const [W, D] = [t.w / 2, t.d / 2];
  const wall = hexOf(t.wall);
  const roof = hexOf(t.roofColor);
  const { top, rise } = t;
  g.color(wall, FINISH).box(-W, t.base, -D, W, top, D, 'ny py');
  if (t.roof === 'flat') {
    g.color(wall, FINISH).box(-W, top, -D, W, top + rise, D, 'ny py');
    g.color(roof, FINISH).poly([[-W, top + rise, -D], [W, top + rise, -D], [W, top + rise, D], [-W, top + rise, D]], [0, 1, 0]);
  } else {
    const [ex, ez] = [W + EAVE * t.w, D + EAVE * t.d];
    // (A gable's ridge the whole length, its ends walls; a hip's and a mansard's short, four slopes to it.)
    const r = t.roof === 'gable' ? ex : t.roof === 'hip' ? HIP_RIDGE * t.w : HIP_RIDGE * 0.6 * t.w;
    const y = top + rise;
    g.color(roof, FINISH);
    for (const s of [-1, 1]) {
      const q: V3[] = [[-ex, top, s * ez], [ex, top, s * ez], [r, y, 0], [-r, y, 0]];
      g.poly(q, facing(q[0], q[1], q[2], [0, 1, s]));
    }
    for (const s of [-1, 1]) {
      if (t.roof === 'gable') {
        g.color(wall, FINISH).poly([[s * W, top, -D], [s * W, top, D], [s * W, y, 0]], [s, 0, 0]);
        continue;
      }
      const q: V3[] = [[s * ex, top, -ez], [s * ex, top, ez], [s * r, y, 0]];
      g.color(roof, FINISH).poly(q, facing(q[0], q[1], q[2], [s, 1, 0]));
    }
  }
  geo.merge(g.place(t.ux, t.uz, t.cx, 0, t.cz));
}

/** The share of a lot's house's windows lit at night while someone is up in it. */
const LOT_LIT = 0.5;

/**
 * A far lot's house as drawn: its walls' box on its lot's pad, from under the lowest corner's ground (`ground`), the
 * street's far LOD's look. `up`: someone is up in it (its windows glow at night only then, as the neighbours' near the
 * camera do).
 */
export function lotBuilding(h: TownLotHouse, ground: (x: number, z: number) => number, up = true): TownBuilding {
  // (A lot with a look as its look has it, lit by its hours or as homes: `lotLookBuilding`.)
  if (h.look) return { ...lotLookBuilding(h.box, h.look, h.storeys, h.pad, ground, h.hours, h.flats), chunk: h.chunk };
  const [x0, z0, x1, z1] = h.box;
  const along = x1 - x0 >= z1 - z0;
  const [w, d] = along ? [x1 - x0, z1 - z0] : [z1 - z0, x1 - x0];
  const cx = (x0 + x1) / 2;
  const cz = (z0 + z1) / 2;
  const lowest = Math.min(ground(x0, z0), ground(x1, z0), ground(x0, z1), ground(x1, z1));
  const top = h.pad + h.storeys * STOREY;
  return {
    roof: h.shape,
    cx,
    cz,
    ux: along ? 1 : 0,
    uz: along ? 0 : 1,
    w,
    d,
    base: Math.min(h.pad, lowest) - SINK,
    top,
    rise: h.shape === 'flat' ? 0.25 : h.rise,
    wall: hex(h.wall),
    roofColor: hex(h.roof),
    floor: h.pad,
    style: TOWN_STYLE.house,
    lit: up ? LOT_LIT : 0,
    seed: hash2(h.id, 11),
    accent: 0,
  };
}

// ---- the unit meshes ------------------------------------------------------------------------

/** Vertex colour of a wall part (red 1) or a roof part (red 0), toned by `tone` (green; sRGB). */
const part = (wall: boolean, tone = 1) => `#${wall ? 'FF' : '00'}${Math.round(tone * 255).toString(16).padStart(2, '0')}00`;
/** The plain finish's rough, not metal (`farHouse`'s). */
export const FINISH = 0.4;
/** A roof's overhang (unit: a share of the footprint's side). */
export const EAVE = 0.035;
/** A hip's ridge end and a mansard's break, a share of the side in from the eaves. */
const HIP_RIDGE = 0.2;
export const MANSARD_IN = 0.13;
export const MANSARD_BREAK = 0.72;

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

