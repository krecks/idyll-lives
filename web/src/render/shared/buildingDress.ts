/**
 * A lot's building closed (`WorldMeta.lotLooks`: a named place's, a flats building's) dressed as the game draws it near
 * (the street's chunks streamed in, `shared/street.ts`), engine-neutral. Far off the town view draws the same building
 * as a box and a roof (`townBuildings.ts`); this one has the box's measures and colours (`lotLookBuilding`: its
 * rectangle, its walls from under its ground to its real storeys, its roof's rise), dressed:
 *
 * - **Homes** (`house`, `terrace`, `farm`): siding (brick where the palette's wall is brick), corner boards, framed
 *   windows with curtains and shutters on every storey, a panelled front door under a little canopy on the side its
 *   way to the road leaves by, a chimney on some roofs. A terrace is a row of homes (one door each, a stack on each
 *   party wall).
 * - **Shops, cafés, restaurants**: a shopfront on the ground floor (glass, a door, a fascia, an awning on most). With
 *   flats over it (`TownBuilding.flats`), the flats' windows above and their own street door beside the shopfront.
 * - **Flats, offices, schools, halls**: render (or brick), bands between storeys, closer and wider windows, a door
 *   under a canopy.
 * - **A church**: stone, tall windows, a tower with a spire at one end. **A workshop, a barn**: boards, a wide door,
 *   few windows.
 *
 * Roofs as the box's: a gable or a hip (the house's `addRoof`), a flat roof behind a parapet, a mansard. Built in a
 * frame of the building's own (x along its rectangle, z across) and put in place (`Geo.place`). Its parts are named
 * and drawn as the neighbours' silhouettes are (`HousePlanner.silhouettes`: merged into the same meshes), but render
 * and stone; its window glass is apart, in `PANE_SETS` sets lit at night by the town view's rule (`litAt`).
 */

import type { BuildingUse } from '../../core/protocol';
import { Geo, facing, type V3 } from './geometry';
import { addRoof, HOUSE_MATERIALS, type HousePart } from './house';
import { EAVE, MANSARD_BREAK, MANSARD_IN, PARAPET, STOREY, type TownBuilding } from './townBuildings';

/** A building's window glass is in this many sets, each lit at night once the share lit passes its turn (`paneSetLit`). */
export const PANE_SETS = 4;

/** Whether pane set `k` of a building is lit while `share` of its windows are (`litAt`). */
export const paneSetLit = (k: number, share: number): boolean => share > (k + 0.5) / PANE_SETS;

/** A building as dressed: its parts (by the silhouettes' names where they share a material) and its window glass by set. */
export interface BuildingDress {
  parts: HousePart[];
  panes: Geo[];
}

const M = HOUSE_MATERIALS;
const TRIM = '#F7F5F0';
const PLINTH = 0.32;
const ACCENT = ['#3E5A4A', '#2F4058', '#7A3B34', '#4A4A4A', '#5B6E86', '#6E5A3E'];
const CURTAINS = ['#E8DCC8', '#C9B79C', '#D8C3C3', '#B9C4CC', '#E2D6B8'];
const FASCIA = ['#2F4058', '#3E5A4A', '#7A3B34', '#1F2A33', '#6B4A3A', '#8C6A2E'];
const STRIPES = ['#B23A3A', '#2F6B5A', '#2F4E7A', '#C27B2A'];
/** How far a roof's eaves overhang near (m), as the neighbours' do. */
const OVERHANG = 0.35;
/** A terrace's homes are about this wide (m). */
const TERRACE_HOME = 5.5;
/** Flats' windows (and a hall's): bays this far apart (m), this wide, their sill and head over the storey's floor. */
const FLATS_WINDOWS = [2.4, 1.2, 0.9, 2.3] as const;

const HOMES: ReadonlySet<BuildingUse> = new Set<BuildingUse>(['house', 'terrace', 'farm']);
const SHOPS: ReadonlySet<BuildingUse> = new Set<BuildingUse>(['shop', 'cafe', 'restaurant']);
const BOARDED: ReadonlySet<BuildingUse> = new Set<BuildingUse>(['barn', 'workshop']);

/** A number from 0 to 1 for `a`, `b`, `c` (stable). */
function h(a: number, b: number, c = 0): number {
  let x = Math.imul((a | 0) ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul((b | 0) + 0x632be5ab, 0xc2b2ae35) ^ Math.imul((c | 0) + 0x27d4eb2f, 0x165667b1);
  x = Math.imul(x ^ (x >>> 15), 0x2c1b3c6d);
  x ^= x >>> 13;
  return (x >>> 0) / 4294967296;
}

const pick = <T>(list: readonly T[], t: number): T => list[Math.min(list.length - 1, Math.floor(t * list.length))];

/** sRGB 0..1 as a hex colour. */
function hexOf(c: readonly number[]): string {
  return `#${c.map((v) => Math.round(Math.min(1, Math.max(0, v)) * 255).toString(16).padStart(2, '0')).join('')}`;
}

/** `c` (hex) times `k`, clamped. */
function shade(c: string, k: number): string {
  const v = parseInt(c.slice(1), 16);
  return hexOf([(((v >> 16) & 255) / 255) * k, (((v >> 8) & 255) / 255) * k, ((v & 255) / 255) * k]);
}

/** `c` (sRGB 0..1) mixed `t` of the way to white. */
const paler = (c: readonly number[], t: number) => hexOf(c.map((v) => v + (1 - v) * t));

/** A wall of the palette that reads as brick: red-brown, darkish. */
function brickish(c: readonly number[]): boolean {
  const lum = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
  return c[0] - c[2] > 0.3 && lum < 0.6;
}

/**
 * The sides of a building's box in its own frame: 0 the +z side, 1 the −z one, 2 the +x one, 3 the −x one. Along a
 * side `a` runs along x (on ±z) or z (on ±x), from −half to half its length.
 */
type Side = 0 | 1 | 2 | 3;
const SIDES: readonly Side[] = [0, 1, 2, 3];

class Dresser {
  readonly siding = new Geo();
  readonly brick = new Geo();
  readonly render = new Geo();
  readonly stone = new Geo();
  readonly plinth = new Geo().color('#D8D2C8');
  readonly roof = new Geo();
  readonly trim = new Geo().color(TRIM);
  readonly panes = Array.from({ length: PANE_SETS }, () => new Geo());
  private paneCount = 0;

  constructor(
    readonly W: number,
    readonly D: number,
    readonly seed: number,
  ) {}

  /** A side's half length and its normal (x, z). */
  half(s: Side): number {
    return s < 2 ? this.W : this.D;
  }
  normal(s: Side): [number, number] {
    return s === 0 ? [0, 1] : s === 1 ? [0, -1] : s === 2 ? [1, 0] : [-1, 0];
  }

  /**
   * A box on side `s`: `a0`..`a1` along it, `y0`..`y1` up, `o0`..`o1` out from its outer face. Without its face against
   * the wall nor its underside (the camera looks down on it): four faces, not six.
   */
  box(g: Geo, s: Side, a0: number, a1: number, y0: number, y1: number, o0: number, o1: number): void {
    const { W, D } = this;
    const skip = `ny ${['nz', 'pz', 'nx', 'px'][s]}`;
    if (s === 0) g.box(a0, y0, D + o0, a1, y1, D + o1, skip);
    else if (s === 1) g.box(a0, y0, -D - o1, a1, y1, -D - o0, skip);
    else if (s === 2) g.box(W + o0, y0, a0, W + o1, y1, a1, skip);
    else g.box(-W - o1, y0, a0, -W - o0, y1, a1, skip);
  }

  /** A quad on side `s`, `out` from its face, facing out. */
  quad(g: Geo, s: Side, a0: number, a1: number, y0: number, y1: number, out: number): void {
    const [nx, nz] = this.normal(s);
    const P = (a: number, y: number): V3 => (s < 2 ? [a, y, nz * (this.D + out)] : [nx * (this.W + out), y, a]);
    g.poly([P(a0, y0), P(a1, y0), P(a1, y1), P(a0, y1)], [nx, 0, nz]);
  }

  /** A pane of glass on side `s`, in one of the building's pane sets. */
  pane(s: Side, a0: number, a1: number, y0: number, y1: number): void {
    const k = Math.floor(h(this.seed, this.paneCount++, 31) * PANE_SETS);
    this.quad(this.panes[k], s, a0, a1, y0, y1, 0.006);
  }

  /** Centres along side `s` of as many bays `apart` m apart as fit, `margin` m clear of its ends. */
  bays(s: Side, apart: number, margin: number): number[] {
    const L = this.half(s) * 2 - 2 * margin;
    const n = Math.floor(L / apart) + 1;
    if (L < 0 || n < 1) return [];
    const first = -((n - 1) * apart) / 2;
    return Array.from({ length: n }, (_, i) => first + i * apart);
  }

  /** A house's window: framed, a sill, a cross of muntins, curtains drawn to the sides, shutters if `shutters` (in `accent`). */
  houseWindow(s: Side, a: number, sill: number, head: number, curtain: string, shutters: string | null, wide = 0.76): void {
    const [g0, g1] = [a - wide / 2, a + wide / 2];
    const f = 0.07;
    this.pane(s, g0, g1, sill, head);
    const t = this.trim;
    // (Curtains, muntins and shutters flat on the glass and the wall: a quad each.)
    t.color(curtain);
    this.quad(t, s, g0, g0 + 0.17, sill + 0.04, head, 0.012);
    this.quad(t, s, g1 - 0.17, g1, sill + 0.04, head, 0.012);
    this.quad(t, s, g0, g1, head - 0.16, head, 0.014);
    t.color(TRIM);
    this.box(t, s, g0 - f, g0, sill, head, 0, 0.04);
    this.box(t, s, g1, g1 + f, sill, head, 0, 0.04);
    this.box(t, s, g0 - f, g1 + f, head, head + f, 0, 0.04);
    this.box(t, s, g0 - f - 0.04, g1 + f + 0.04, sill - 0.06, sill, 0, 0.07);
    const mid = (g0 + g1) / 2;
    this.quad(t, s, mid - 0.016, mid + 0.016, sill, head - 0.16, 0.024);
    this.quad(t, s, g0, g1, (sill + head) / 2 + 0.1, (sill + head) / 2 + 0.13, 0.024);
    if (shutters) {
      t.color(shutters);
      this.quad(t, s, g0 - f - 0.36, g0 - f - 0.02, sill - 0.02, head + 0.04, 0.035);
      this.quad(t, s, g1 + f + 0.02, g1 + f + 0.36, sill - 0.02, head + 0.04, 0.035);
      t.color(TRIM);
    }
  }

  /** A town building's window: a plain frame, a mullion, a stone sill. */
  plainWindow(s: Side, a: number, sill: number, head: number, wide: number): void {
    const [g0, g1] = [a - wide / 2, a + wide / 2];
    const f = 0.06;
    this.pane(s, g0, g1, sill, head);
    const t = this.trim.color(TRIM);
    this.box(t, s, g0 - f, g0, sill, head, 0, 0.035);
    this.box(t, s, g1, g1 + f, sill, head, 0, 0.035);
    this.box(t, s, g0 - f, g1 + f, head, head + f, 0, 0.035);
    this.quad(t, s, a - 0.025, a + 0.025, sill, head, 0.025);
    this.box(this.plinth, s, g0 - f - 0.05, g1 + f + 0.05, sill - 0.08, sill, 0, 0.08);
  }

  /** A panelled door (`colour`) `wide` m wide from `floor`, framed, a step under it and a little canopy over it. */
  door(s: Side, a: number, floor: number, base: number, colour: string, wide = 1, canopy = true): void {
    const t = this.trim;
    const top = floor + 2.1;
    const [d0, d1] = [a - wide / 2, a + wide / 2];
    t.color(TRIM);
    this.box(t, s, d0 - 0.08, d0, floor, top + 0.1, 0, 0.06);
    this.box(t, s, d1, d1 + 0.08, floor, top + 0.1, 0, 0.06);
    this.box(t, s, d0 - 0.1, d1 + 0.1, top, top + 0.14, 0, 0.07);
    t.color(colour);
    this.box(t, s, d0, d1, floor, top, 0, 0.045);
    t.color(shade(colour, 1.12));
    const leaves = wide > 1.3 ? 2 : 1;
    const lw = wide / leaves;
    for (let i = 0; i < leaves; i++) {
      const l0 = d0 + i * lw;
      this.quad(t, s, l0 + 0.1, l0 + lw / 2 - 0.04, floor + 1.15, floor + 1.95, 0.055);
      this.quad(t, s, l0 + lw / 2 + 0.04, l0 + lw - 0.1, floor + 1.15, floor + 1.95, 0.055);
      this.quad(t, s, l0 + 0.1, l0 + lw / 2 - 0.04, floor + 0.15, floor + 0.95, 0.055);
      this.quad(t, s, l0 + lw / 2 + 0.04, l0 + lw - 0.1, floor + 0.15, floor + 0.95, 0.055);
    }
    t.color('#C8A64A');
    this.box(t, s, d1 - 0.16, d1 - 0.1, floor + 1.0, floor + 1.06, 0, 0.07);
    t.color(TRIM);
    // A step up to it, down to the ground.
    this.box(this.plinth, s, d0 - 0.25, d1 + 0.25, base, floor + 0.02, 0, 0.45);
    if (!canopy) return;
    // A little flat canopy on two brackets.
    this.box(t, s, d0 - 0.35, d1 + 0.35, top + 0.3, top + 0.42, 0, 0.75);
    this.box(t, s, d0 - 0.3, d0 - 0.22, top - 0.1, top + 0.3, 0, 0.6);
    this.box(t, s, d1 + 0.22, d1 + 0.3, top - 0.1, top + 0.3, 0, 0.6);
  }

  /** Plinth, walls (`walls`, in `colour`) from `base` to `top`, a box `x0`..`x1` along x (the whole depth). */
  walls(walls: Geo, colour: string, x0: number, x1: number, floor: number, top: number, skip: string): void {
    walls.color(colour).box(x0, floor + PLINTH, -this.D, x1, top, this.D, `ny py ${skip}`);
  }

  parts(): HousePart[] {
    return (
      [
        [this.siding, 'silhouetteWalls', M.siding, true],
        [this.brick, 'silhouetteBrick', M.brick, true],
        [this.render, 'buildingRender', M.plaster, true],
        [this.stone, 'buildingStone', M.stone, true],
        [this.plinth, 'silhouettePlinth', M.stone, false],
        [this.roof, 'silhouetteRoofs', M.roof, true],
        [this.trim, 'silhouetteTrim', M.trim, true],
      ] as const
    )
      .filter(([geo]) => !geo.empty)
      .map(([geo, name, key, cast]): HousePart => ({ geo, name, surface: { kind: 'surface', key }, cut: true, cast, roof: false, shadowOnly: false, glass: false }));
  }

  /** Everything put in place: x along (`ux`, `uz`), round (`cx`, `cz`). */
  place(ux: number, uz: number, cx: number, cz: number): void {
    for (const g of [this.siding, this.brick, this.render, this.stone, this.plinth, this.roof, this.trim, ...this.panes]) g.place(ux, uz, cx, 0, cz);
  }
}

/**
 * Which side of `t`'s box faces `to` (a point out in front of it, on its street), in the box's own frame (`Side`);
 * none known: its long +z side.
 */
export function frontSide(t: Pick<TownBuilding, 'cx' | 'cz' | 'ux' | 'uz' | 'w' | 'd'>, to: readonly [number, number] | null): Side {
  if (!to) return 0;
  const [dx, dz] = [to[0] - t.cx, to[1] - t.cz];
  const lx = dx * t.ux + dz * t.uz;
  const lz = -dx * t.uz + dz * t.ux;
  // (Its sides weighed by their reach: a long building's way leaves by a long side unless it is well past an end.)
  if (Math.abs(lz) / (t.d / 2) >= Math.abs(lx) / (t.w / 2)) return lz >= 0 ? 0 : 1;
  return lx >= 0 ? 2 : 3;
}

/**
 * Building `t` (a lot's closed, as the town view draws it, `lotLookBuilding`; its `use`) dressed, in place. `to`: a
 * point out in front of it, on its street: the side its door is on. `door`: where its lot's own front door is (town
 * metres, on that side): the door drawn there, and a church's tower over it where the door is on an end. `flatsDoor`:
 * a place with flats over it (`TownBuilding.flats`), where the flats' own street door is (else at the front's other end
 * from the place's).
 */
export function dressBuilding(t: TownBuilding, use: BuildingUse, to: readonly [number, number] | null, door: readonly [number, number] | null = null, flatsDoor: readonly [number, number] | null = null): BuildingDress {
  const W = t.w / 2;
  const D = t.d / 2;
  const seed = t.seed;
  const d = new Dresser(W, D, seed);
  const { floor, base, top } = t;
  const storeys = Math.max(1, Math.round((top - floor) / STOREY));
  const front = frontSide(t, to);
  const wall = t.wall;
  const home = HOMES.has(use);
  const shop = SHOPS.has(use);
  const boarded = BOARDED.has(use);
  // The walls' material and colour: a church's stone, brick where the palette's wall is, boards on homes, barns and
  // workshops, render on the rest.
  const brick = use !== 'church' && brickish(wall);
  const walls = use === 'church' ? d.stone : brick ? d.brick : home || boarded ? d.siding : d.render;
  const colour = use === 'church' ? paler(wall, 0.35) : brick ? paler(wall, 0.82) : hexOf(wall);
  const accent = pick(ACCENT, h(seed, 7));
  const curtain = pick(CURTAINS, h(seed, 3));
  const shutters = home && h(seed, 6) < 0.6 ? accent : null;
  const roofHex = hexOf(t.roofColor);
  const flat = t.roof === 'flat';
  const wallTop = flat ? top + PARAPET : top;

  // ---- plinth and walls (a terrace's homes each a little different) ----
  d.plinth.box(-W - 0.03, base, -D - 0.03, W + 0.03, floor + PLINTH, D + 0.03, 'ny');
  const homesIn = use === 'terrace' ? Math.max(1, Math.round((2 * W) / TERRACE_HOME)) : 1;
  const unit = (2 * W) / homesIn;
  for (let i = 0; i < homesIn; i++) {
    const tone = homesIn > 1 ? 0.93 + h(seed, i, 41) * 0.12 : 1;
    const skip = `${i > 0 ? 'nx' : ''} ${i < homesIn - 1 ? 'px' : ''}`;
    d.walls(walls, homesIn > 1 ? shade(colour, tone) : colour, -W + i * unit, -W + (i + 1) * unit, floor, wallTop, skip);
  }
  walls.color(colour);

  // ---- trim: corner boards and a frieze on boards; bands between storeys and a cornice on render, brick and stone ----
  const trim = d.trim.color(TRIM);
  if (walls === d.siding) {
    for (const [sx, sz] of [[-1, -1], [1, -1], [-1, 1], [1, 1]] as const) {
      const [x, z] = [sx * W, sz * D];
      trim.box(Math.min(x, x - sx * 0.13), floor + PLINTH, Math.min(z, z + sz * 0.025), Math.max(x, x - sx * 0.13), top - 0.2, Math.max(z, z + sz * 0.025), 'ny');
      trim.box(Math.min(x, x + sx * 0.025), floor + PLINTH, Math.min(z, z - sz * 0.13), Math.max(x, x + sx * 0.025), top - 0.2, Math.max(z, z - sz * 0.13), 'ny');
    }
    if (!flat) trim.box(-W - 0.02, top - 0.2, -D - 0.02, W + 0.02, top, D + 0.02, 'ny');
  } else {
    for (let k = 1; k < storeys; k++) {
      const y = floor + k * STOREY;
      trim.box(-W - 0.04, y - 0.07, -D - 0.04, W + 0.04, y + 0.07, D + 0.04, 'ny');
    }
    if (!flat) trim.box(-W - 0.05, top - 0.22, -D - 0.05, W + 0.05, top, D + 0.05, 'ny');
  }
  // Pilasters between a terrace's homes.
  for (let i = 1; i < homesIn; i++) {
    const x = -W + i * unit;
    for (const s of [0, 1] as const) d.box(trim, s, x - 0.08, x + 0.08, floor + PLINTH, top - 0.2, 0, 0.03);
  }

  // ---- doors: where, then the windows round them ----
  const doors: { s: Side; a: number; wide: number }[] = [];
  const L = d.half(front);
  // (A lot's own door along the front side, in the building's frame, kept off its corners.)
  const along = (p: readonly [number, number]) => ((dx: number, dz: number) => (front < 2 ? dx * t.ux + dz * t.uz : -dx * t.uz + dz * t.ux))(p[0] - t.cx, p[1] - t.cz);
  const own = door && along(door);
  const at = (wide: number, fallback: number, a: number | null = own) => (a === null ? fallback : Math.max(-L + wide / 2 + 0.3, Math.min(L - wide / 2 - 0.3, a)));
  // (A place with flats over it: the flats' street door beside its own, apart from the shopfront.)
  const flats = !!t.flats && shop;
  if (use === 'terrace') {
    for (let i = 0; i < homesIn; i++) {
      // (Each home's door at the same end of its frontage, its window beside it; along x on the long sides.)
      const mid = -W + (i + 0.5) * unit;
      doors.push({ s: front < 2 ? front : 0, a: mid - unit * 0.22, wide: 1 });
    }
  } else if (shop) {
    const a = at(1.1, (h(seed, 13) - 0.5) * Math.max(0, L - 2.2));
    doors.push({ s: front, a, wide: 1.1 });
    if (flats) doors.push({ s: front, a: at(1.1, a > 0 ? -L + 1.2 : L - 1.2, flatsDoor && along(flatsDoor)), wide: 1.1 });
  } else if (boarded) {
    const wide = Math.min(3.2, L * 2 - 1.4);
    doors.push({ s: front, a: at(wide, (h(seed, 13) - 0.5) * Math.max(0, L - 3)), wide });
  } else {
    const wide = use === 'church' || use === 'school' || use === 'hall' ? 1.7 : use === 'flats' || use === 'office' ? 1.4 : 1;
    const off = home ? (h(seed, 13) - 0.5) * Math.max(0, L * 0.6 - wide) : 0;
    doors.push({ s: front, a: at(wide, off), wide });
  }
  const nearDoor = (s: Side, a: number, k: number, half: number) => k === 0 && doors.some((o) => o.s === s && Math.abs(o.a - a) < o.wide / 2 + half + 0.25);

  // ---- windows ----
  const windows: readonly [number, number, number, number] = home ? [2.6, 0.76, 0.9, 2.1] : use === 'office' ? [2.0, 1.5, 0.8, 2.4] : use === 'school' ? [2.4, 1.8, 0.9, 2.6] : use === 'flats' || use === 'hall' ? FLATS_WINDOWS : [2.6, 0.9, 1.0, 2.2];
  if (use === 'church') {
    const tall = Math.min(storeys * STOREY - 0.9, 5.2);
    for (const s of [0, 1] as const) for (const a of d.bays(s, 3, 2.2)) if (!nearDoor(s, a, 0, 0.5)) d.plainWindow(s, a, floor + 1.4, floor + Math.max(2.6, tall), 0.9);
  } else if (boarded) {
    for (const s of SIDES) for (const a of d.bays(s, 4, 1.5)) if (!nearDoor(s, a, 0, 0.5)) d.plainWindow(s, a, floor + 1.4, floor + 2.3, 0.8);
  } else {
    for (const s of SIDES) {
      // (A gable end's windows only on a home or a building with an end long enough.)
      if (s >= 2 && d.half(s) * 2 < 3.2) continue;
      for (let k = 0; k < storeys; k++) {
        // (The shopfront is the front's ground floor.)
        if (shop && k === 0 && s === front) continue;
        const y = floor + k * STOREY;
        // (Flats over a place have flats' windows.)
        const [apart, wide, sill, head] = flats && k > 0 ? FLATS_WINDOWS : windows;
        for (const a of d.bays(s, apart, 0.9)) {
          if (nearDoor(s, a, k, wide / 2)) continue;
          // (A terrace's party walls keep clear.)
          if (homesIn > 1 && s < 2 && Array.from({ length: homesIn - 1 }, (_, i) => -W + (i + 1) * unit).some((x) => Math.abs(x - a) < wide / 2 + 0.3)) continue;
          if (home) d.houseWindow(s, a, y + sill, y + head, curtain, shutters, wide);
          else d.plainWindow(s, a, y + sill, y + head, wide);
        }
      }
    }
  }

  // ---- doors ----
  for (const o of doors) {
    if (boarded) {
      // A wide plank door, its boards up and a brace across.
      const tr = d.trim.color(shade(hexOf(wall), 0.7));
      const [d0, d1] = [o.a - o.wide / 2, o.a + o.wide / 2];
      d.box(tr, o.s, d0, d1, floor, floor + 2.8, 0, 0.05);
      tr.color(shade(hexOf(wall), 0.55));
      for (let x = d0 + 0.3; x < d1 - 0.1; x += 0.3) d.quad(tr, o.s, x - 0.015, x + 0.015, floor, floor + 2.8, 0.06);
      d.box(tr, o.s, d0, d1, floor + 1.3, floor + 1.45, 0, 0.065);
      tr.color(TRIM);
      d.box(tr, o.s, d0 - 0.1, d1 + 0.1, floor + 2.8, floor + 2.95, 0, 0.07);
      d.box(d.plinth, o.s, d0 - 0.2, d1 + 0.2, base, floor + 0.02, 0, 0.5);
    } else d.door(o.s, o.a, floor, base, use === 'church' ? '#5A3F2A' : accent, o.wide, !shop);
  }

  // ---- a shopfront on the ground floor of the front ----
  if (shop) {
    const s = front;
    const span = L - 0.45;
    const tr = d.trim;
    const sign = pick(FASCIA, t.accent);
    const [g0, g1] = [floor + 0.5, floor + 2.55];
    // Glass between mullions, but where the doors stand (the shop's, and its flats' beside it).
    const cuts = [-span, ...doors.filter((o) => o.s === s).sort((a, b) => a.a - b.a).flatMap((o) => [o.a - o.wide / 2 - 0.08, o.a + o.wide / 2 + 0.08]), span];
    for (let i = 0; i < cuts.length; i += 2) {
      const [a0, a1] = [cuts[i], cuts[i + 1]];
      if (a1 - a0 < 0.4) continue;
      d.pane(s, a0, a1, g0, g1);
      tr.color(sign);
      d.box(tr, s, a0, a1, floor, g0, 0, 0.06);
      tr.color(TRIM);
      d.box(tr, s, a0 - 0.05, a1 + 0.05, g0 - 0.06, g0, 0, 0.09);
      const n = Math.max(1, Math.round((a1 - a0) / 1.6));
      for (let j = 0; j <= n; j++) {
        const a = a0 + ((a1 - a0) * j) / n;
        d.box(tr, s, a - 0.04, a + 0.04, g0, g1, 0, 0.05);
      }
      d.box(tr, s, a0, a1, g1, g1 + 0.06, 0, 0.05);
    }
    // Pilasters at its ends and the fascia over it, in the shop's colour.
    tr.color(TRIM);
    for (const a of [-span - 0.2, span + 0.2]) d.box(tr, s, a - 0.2, a + 0.2, floor, floor + 2.95, 0, 0.08);
    tr.color(sign);
    d.box(tr, s, -span - 0.4, span + 0.4, floor + 2.6, floor + 2.95, 0, 0.12);
    tr.color(TRIM);
    d.box(tr, s, -span - 0.45, span + 0.45, floor + 2.95, floor + 3.02, 0, 0.16);
    // An awning over the glass (a café's and a restaurant's always, most shops'): striped cloth sloping out.
    if (use !== 'shop' || h(seed, 19) < 0.6) {
      const stripe = pick(STRIPES, h(seed, 23));
      const [y0, y1, out] = [floor + 2.55, floor + 2.1, 1.3];
      const [nx, nz] = d.normal(s);
      const P = (a: number, y: number, o: number): V3 => (s < 2 ? [a, y, nz * (D + o)] : [nx * (W + o), y, a]);
      const n = Math.max(2, Math.round((2 * span) / 0.5));
      for (let j = 0; j < n; j++) {
        const [a0, a1] = [-span + (2 * span * j) / n, -span + (2 * span * (j + 1)) / n];
        tr.color(j % 2 ? '#F2EEE6' : stripe);
        const q: V3[] = [P(a0, y0, 0.02), P(a1, y0, 0.02), P(a1, y1, out), P(a0, y1, out)];
        const up = facing(q[0], q[1], q[2], [nx, 1, nz]);
        tr.poly(q, up);
        tr.poly(q, [-up[0], -up[1], -up[2]]);
        // Its valance.
        const v: V3[] = [P(a0, y1, out), P(a1, y1, out), P(a1, y1 - 0.22, out), P(a0, y1 - 0.22, out)];
        tr.poly(v, [nx, 0, nz]);
      }
      tr.color(TRIM);
    }
  }

  // ---- the roof ----
  const roof = d.roof.color(roofHex);
  walls.color(homesIn > 1 ? shade(colour, 0.97) : colour);
  if (flat) {
    // A deck behind the parapet, its inner faces, and a coping round its top.
    const i = 0.25;
    const deck = top + 0.12;
    roof.poly([[-W + i, deck, -D + i], [W - i, deck, -D + i], [W - i, deck, D - i], [-W + i, deck, D - i]], [0, 1, 0]);
    const y1 = wallTop;
    walls.poly([[-W + i, deck, -D + i], [W - i, deck, -D + i], [W - i, y1, -D + i], [-W + i, y1, -D + i]], [0, 0, 1]);
    walls.poly([[-W + i, deck, D - i], [W - i, deck, D - i], [W - i, y1, D - i], [-W + i, y1, D - i]], [0, 0, -1]);
    walls.poly([[-W + i, deck, -D + i], [-W + i, deck, D - i], [-W + i, y1, D - i], [-W + i, y1, -D + i]], [1, 0, 0]);
    walls.poly([[W - i, deck, -D + i], [W - i, deck, D - i], [W - i, y1, D - i], [W - i, y1, -D + i]], [-1, 0, 0]);
    const c = d.trim.color(shade(TRIM, 0.92));
    const [o, k] = [0.06, i + 0.04];
    c.box(-W - o, y1, -D - o, W + o, y1 + 0.1, -D + k, 'ny');
    c.box(-W - o, y1, D - k, W + o, y1 + 0.1, D + o, 'ny');
    c.box(-W - o, y1, -D + k, -W + k, y1 + 0.1, D - k, 'ny');
    c.box(W - k, y1, -D + k, W + o, y1 + 0.1, D - k, 'ny');
    d.trim.color(TRIM);
  } else if (t.roof === 'mansard') {
    mansard(roof, d.trim, W, D, top, t.rise);
  } else {
    // (A gable or a hip along x, as the town view's: the house's roof, its rise half the depth at its pitch.)
    addRoof({ x0: -W + WALL_T, z0: -D + WALL_T, x1: W - WALL_T, z1: D - WALL_T }, t.roof === 'hip', roof, d.trim.color(TRIM), walls, top, OVERHANG, t.rise / D, true);
  }

  // ---- chimneys: a home's on some roofs, a stack on each of a terrace's party walls ----
  if (!flat && (use === 'house' || use === 'farm') && h(seed, 8) < 0.6) {
    const x = (h(seed, 9) < 0.5 ? -1 : 1) * Math.max(0, W - 1.1);
    chimney(d, x, D * 0.3, top, top + t.rise * 0.7 + 0.8);
  } else if (!flat && use === 'terrace') {
    for (let i = 1; i < homesIn; i++) chimney(d, -W + i * unit, 0, top, top + t.rise + 0.6, 0.9);
  }

  // ---- a church's tower and spire at one end ----
  if (use === 'church') {
    const tw = Math.min(2 * D * 0.55, 4.2, 2 * W * 0.4);
    const x0 = door && front === 3 ? -W : door && front === 2 ? W - tw : h(seed, 29) < 0.5 ? -W : W - tw;
    const towerTop = top + t.rise + 3.5;
    d.stone.color(colour).box(x0, floor + PLINTH, -tw / 2, x0 + tw, towerTop, tw / 2, 'ny py');
    d.trim.color(TRIM).box(x0 - 0.06, towerTop - 0.25, -tw / 2 - 0.06, x0 + tw + 0.06, towerTop, tw / 2 + 0.06, 'ny');
    // A louvred opening on each face near its top.
    const cx = x0 + tw / 2;
    d.trim.color('#3A3632');
    for (const [nx, nz] of [[0, 1], [0, -1], [1, 0], [-1, 0]] as const) {
      const [px, pz] = [cx + nx * (tw / 2 + 0.01), nz * (tw / 2 + 0.01)];
      const y0 = towerTop - 2.4;
      if (nz) d.trim.poly([[cx - 0.45, y0, pz], [cx + 0.45, y0, pz], [cx + 0.45, y0 + 1.6, pz], [cx - 0.45, y0 + 1.6, pz]], [0, 0, nz]);
      else d.trim.poly([[px, y0, -0.45], [px, y0, 0.45], [px, y0 + 1.6, 0.45], [px, y0 + 1.6, -0.45]], [nx, 0, 0]);
    }
    d.trim.color(TRIM);
    spire(roof, cx, 0, tw / 2 + 0.1, towerTop, towerTop + tw * 1.6);
  }

  d.place(t.ux, t.uz, t.cx, t.cz);
  return { parts: d.parts(), panes: d.panes };
}

/** The wall's half thickness the house's roof adds round its rectangle (`addRoof`'s `T`). */
const WALL_T = 0.07;

/** A mansard over `W` × `D` (half sides) from `top`, `rise` high, as the town view's (`unitMeshes`): steep lower slopes to a break, then a low hip. */
function mansard(roof: Geo, trim: Geo, W: number, D: number, top: number, rise: number): void {
  const [ex, ez] = [W + EAVE * 2 * W, D + EAVE * 2 * D];
  const [bx, bz] = [W - MANSARD_IN * 2 * W, D - MANSARD_IN * 2 * D];
  const yb = top + MANSARD_BREAK * rise;
  const yt = top + rise;
  const r = 0.2 * 0.6 * 2 * W;
  for (const s of [-1, 1]) {
    const qz: V3[] = [[-ex, top, s * ez], [ex, top, s * ez], [bx, yb, s * bz], [-bx, yb, s * bz]];
    roof.poly(qz, facing(qz[0], qz[1], qz[2], [0, 0.4, s]));
    const qx: V3[] = [[s * ex, top, -ez], [s * ex, top, ez], [s * bx, yb, bz], [s * bx, yb, -bz]];
    roof.poly(qx, facing(qx[0], qx[1], qx[2], [s, 0.4, 0]));
    const q: V3[] = [[-bx, yb, s * bz], [bx, yb, s * bz], [r, yt, 0], [-r, yt, 0]];
    roof.poly(q, facing(q[0], q[1], q[2], [0, 1, s]));
    const e: V3[] = [[s * bx, yb, -bz], [s * bx, yb, bz], [s * r, yt, 0]];
    roof.poly(e, facing(e[0], e[1], e[2], [s, 1, 0]));
  }
  // Its eaves' soffit, and a moulding at the break.
  trim.color(TRIM);
  trim.poly([[-ex, top, -ez], [ex, top, -ez], [ex, top, ez], [-ex, top, ez]], [0, -1, 0]);
  trim.box(-bx - 0.05, yb - 0.04, -bz - 0.05, bx + 0.05, yb + 0.06, bz + 0.05, 'ny');
}

/** A spire: a square pyramid `half` m to a side's half round (`cx`, `cz`) from `y0` up to `y1`. */
function spire(roof: Geo, cx: number, cz: number, half: number, y0: number, y1: number): void {
  const tip: V3 = [cx, y1, cz];
  const c: V3[] = [[cx - half, y0, cz - half], [cx + half, y0, cz - half], [cx + half, y0, cz + half], [cx - half, y0, cz + half]];
  for (let i = 0; i < 4; i++) {
    const [a, b] = [c[i], c[(i + 1) % 4]];
    const mid: V3 = [(a[0] + b[0]) / 2 - cx, 0.5, (a[2] + b[2]) / 2 - cz];
    roof.poly([a, b, tip], facing(a, b, tip, mid));
  }
}

/** A brick chimney at (`x`, `z`) from `y0` up to `y1`, `wide` m along x, capped. */
function chimney(d: Dresser, x: number, z: number, y0: number, y1: number, wide = 0.6): void {
  d.brick.color('#E8DCD4').box(x - wide / 2, y0, z - 0.25, x + wide / 2, y1, z + 0.25, 'ny');
  d.plinth.box(x - wide / 2 - 0.06, y1, z - 0.31, x + wide / 2 + 0.06, y1 + 0.1, z + 0.31, 'ny');
}
