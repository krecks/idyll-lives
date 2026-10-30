/**
 * Neighbourhood generation and assembly. A town is a few roads winding through it (`roads.ts`)
 * with plots along them; each plot has a
 * small, medium or large house (or a farm) and gets a house layout (or a public place: the park,
 * a playground, a square, a shop) from `content/houses.json`. A generated house comes with its
 * lot, cut to fit it with a snug, roomy or big garden (`housegen.ts` `fitLot`); the fixed layouts
 * are drawn for a medium lot and sit centred on lots of the other sizes, with the front door
 * towards the street, a small lot only taking a house that fits. A farm is a large house with
 * fields beside it and a farmyard behind. Each plot is laid out with its street at z = 0 and
 * turned onto the town to face its road (`Facing`, `turner`), so every front door faces the road.
 * What kind of world the town is in (`world.ts`) and its seed decide the layout (`generateWorld`);
 * the households come after (`populate`). The result is one town file the simulation loads like any lot.
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import type { GameKind, LandscapeParams, Lifespan, WorldLook } from '../core/protocol';
import { fetchText } from '../content/content';
import { bedroomsNeeded, householdBonds, householdSpawns, randomHousehold, residentsOf, type HouseholdDraft, type SimSpawn } from './household';
import { homePrice } from './price';
import { generateHouse, objectInfos, random, residentsKey, weighted, type GardenSize, type HouseGenConfig, type ObjectInfos, type Residents } from './housegen';
import { cropRoads, inside, leftOf, lengths, nearest, pointAt, roadExits, roadIndex, roadMask, tangent, trimJoins, type Pt, type RoadPlan } from './roads';
import { householdFunds, savingsOf } from './wealth';
import { streetNames, WORLDS, worldName, worldTagline, type WorldKind } from './world';

export type Wall = [number, number, number, number];
export interface Door {
  x: number;
  z: number;
  axis: 'x' | 'z';
}
export interface Placement {
  def: string;
  x: number;
  z: number;
  rot: number;
  /** Degrees past `rot` (0..90), for things that turn freely (sim-core `check_turn`). */
  turn?: number;
  /** Off the middle of its footprint in hundredths of a tile ([x, z], up to ±50), for things that turn freely. */
  offset?: [number, number];
}

export interface HouseTemplate {
  id: string;
  name: string;
  description: string;
  bedrooms: number;
  walls: Wall[];
  doors: Door[];
  /** Windows in walls, placed like doors. */
  windows?: Door[];
  objects: Placement[];
  spawns: [number, number][];
  /** The storey above (generated two-storey houses), laid out like the ground. */
  upper?: { walls: Wall[]; doors: Door[]; windows?: Door[]; objects: Placement[]; rooms?: PlannedRoom[] };
  /** The basement (some generated houses), laid out like the ground under it: no windows, stairs standing in it come up inside the house. */
  basement?: { walls: Wall[]; doors: Door[]; windows?: Door[]; objects: Placement[]; rooms?: PlannedRoom[] };
  /** Generated houses: the rooms as planned (for checking them; the game finds rooms from the walls). */
  rooms?: PlannedRoom[];
  /** Generated houses: what kind of house it is (its class's label in housegen.json: cottage, villa...). */
  class?: string;
  /** Generated houses with a garden size: the lot cut to fit the house, the layout in its frame (see `housegen.ts` `fitLot`). */
  lot?: { width: number; depth: number };
}

/** A room of a generated house: what it's for and the rectangle it takes. */
export interface PlannedRoom {
  kind: string;
  x: number;
  z: number;
  w: number;
  d: number;
}

/**
 * A public place (`parks` in `content/houses.json`): a park or square laid out in the open, or a
 * shop, a small shell with walls, doors and windows like a house's. Everyone in town may use what
 * is there. The first one is the town park every town has.
 */
export interface PublicTemplate {
  id: string;
  name: string;
  description: string;
  /** `park` (default) or `shop` (labels, and which ones a town gets). */
  kind?: 'park' | 'shop';
  walls?: Wall[];
  doors?: Door[];
  windows?: Door[];
  objects: Placement[];
  /** Paved ground (plot coordinates, any size): walks, squares, terraces. Only drawn. */
  paths?: Rect[];
  /** Where visitors arrive (plot coordinates); default: the middle of the street front. */
  entry?: [number, number];
}

/** `farm`: a large lot's house with fields and a farmyard (farm worlds only; not offered to resize to). */
export type LotSize = 'small' | 'medium' | 'large' | 'farm';
export const LOT_SIZES: readonly LotSize[] = ['small', 'medium', 'large'];

/** The lot size a lot's house is made for (a farmhouse is a large lot's house). */
const houseLot = (size: LotSize | undefined): LotSize => (size === 'farm' ? 'large' : (size ?? 'medium'));

export interface Templates {
  /** The lot every layout is drawn for (a medium lot). */
  plot: { width: number; depth: number };
  /** Lot sizes (missing in older files: every lot is `plot`). */
  lotSizes?: Partial<Record<LotSize, { width: number; depth: number }>>;
  houses: HouseTemplate[];
  parks: PublicTemplate[];
  /** How houses are generated (`content/housegen.json`; absent: only the fixed layouts above). */
  gen?: HouseGenConfig;
  /** Object types from the game's content (for furnishing generated houses). */
  info?: ObjectInfos;
  /** Tiles one lamp lights at night (content `roomRules.lampTiles`): generated rooms get lights enough. */
  lampTiles?: number;
  /** Tiles one window lights by day (content `roomRules.windowTiles`): generated rooms get daylight enough. */
  windowTiles?: number;
}

/** Town sizes of drafts from older code (worlds have a `WorldKind` now). */
export type TownSize = 'small' | 'medium' | 'large';

export interface PlotSlot {
  index: number;
  /** `park` and `shop` are public places (see `PublicTemplate`). */
  kind: 'house' | 'park' | 'shop';
  template: string;
  /** Address or place name. */
  name: string;
  x: number;
  z: number;
  /** How it is turned onto the town to face its street (see `Facing`). */
  facing: Facing;
  /** The size of its house (and of its lot, unless `w` and `d` say otherwise). Missing in drafts from older code: medium. */
  lot?: LotSize;
  /** Its lot's own width and depth: a generated house's lot cut to fit it (and a farm's round that). Missing: `lot`'s. */
  w?: number;
  d?: number;
  /** How much garden a generated house's lot has round it (see `housegen.ts` `fitLot`). */
  garden?: GardenSize;
  /** The middle of its front along its road (x, or z for a plot turned a quarter), where its lot is cut again round it (`withGarden`). */
  anchor?: number;
  /** A farm lot's fields and farmyard, in plot coordinates like the house layout. */
  farm?: { objects: Placement[]; fields: Rect[] };
}

/**
 * Quarter turns a plot is laid onto the town with, as things are turned (`Placement.rot`: rot 0
 * faces +z, rot 1 +x): where its street is. 0: to the north (smaller z), 1: to the west (smaller
 * x), 2: to the south, 3: to the east. A plot turned 1 or 3 lies across, its depth along x.
 */
export type Facing = 0 | 1 | 2 | 3;

export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

export interface NeighbourhoodDraft {
  name: string;
  /** Drafts from older code; worlds say what kind they are in `world`. */
  size?: TownSize;
  width: number;
  depth: number;
  slots: PlotSlot[];
  /** The town's roads (`roads.ts`), each plot facing one. */
  roads: RoadPlan[];
  /** Greens the roads go round (polygons). */
  greens: Pt[][];
  /** Neighbour households and the slot each lives in. */
  households: { household: HouseholdDraft; slot: number }[];
  /**
   * Landscape seed (hills, woods, lawn detail). Kept while the draft is edited, so the 3D preview
   * in the menus and the game itself show the same valley. Missing in drafts from older code.
   * In a world it is also the seed its layout was made from.
   */
  seed?: number;
  /** The world the town is in (missing in drafts from older code: the classic valley). */
  world?: WorldInfo;
}

/** What kind of world a town is in, one line about it, and how its land looks. */
export interface WorldInfo {
  kind: WorldKind;
  tagline: string;
  landscape: LandscapeParams;
}

/** Presentation data the renderer reads back from the world (it survives save/load). */
export interface TownMeta {
  kind: 'town';
  name: string;
  /** Older towns' straight streets (newer ones have `roads`). */
  streets?: Rect[];
  roads?: RoadPlan[];
  greens?: Pt[][];
  paths: Rect[];
  /** Each plot's `Facing`, by plot index (missing in older saves: worked out from the streets). */
  facing?: Facing[];
  seed: number;
  world?: WorldLook;
  /** The menus' preview only (`previewWorld`): the plot drawn as the player's house. */
  home?: number | null;
}

/** Open land round a town's lots (m). */
const MARGIN = 8;
/** Tiles a town may measure each way (sim-core `Lot::from_file`). */
const MAX_TOWN = 256;
/** Grass between a road's sidewalk and the lots along it (m). */
const VERGE = 0.5;

let templates: Promise<Templates> | null = null;
let giveContent: (content: Content) => void = () => {};
const townContent = new Promise<Content>((resolve) => (giveContent = resolve));

/** The game's content, once loaded (generated houses are furnished from it). */
export function useTownContent(content: Content): void {
  giveContent(content);
}

export function loadTemplates(): Promise<Templates> {
  const base = import.meta.env.BASE_URL;
  templates ??= Promise.all([
    fetchText(`${base}content/houses.json`).then((t) => JSON.parse(t) as Templates),
    fetchText(`${base}content/housegen.json`)
      .then((t) => JSON.parse(t) as HouseGenConfig)
      .catch(() => undefined),
    townContent,
  ]).then(([t, gen, content]) => ({ ...t, gen, info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles }));
  return templates;
}

const generated = new Map<string, HouseTemplate | undefined>();

/** A generated house's id: its size, seed and garden (the same id always gives the same house and lot). */
export function generatedId(size: LotSize, seed: number, garden?: GardenSize): string {
  return `gen:${size}:${seed >>> 0}${garden ? `:${garden}` : ''}`;
}

type Rand = () => number;
const pick = <T>(list: readonly T[], rand: Rand = Math.random): T => list[Math.floor(rand() * list.length)];
/** A shuffled copy (Fisher-Yates). */
function shuffled<T>(list: readonly T[], rand: Rand = Math.random): T[] {
  const out = [...list];
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

/** A house layout by id; a generated one furnished for `residents` (the layout is the same for anyone; undefined: nobody lives there). */
export function houseTemplate(t: Templates, id: string, residents?: Residents): HouseTemplate | undefined {
  if (id.startsWith('gen:')) {
    const at = `${id}#${residentsKey(residents)}`;
    if (!generated.has(at)) {
      const [, size, seed, garden] = id.split(':');
      const house = t.gen && t.info ? generateHouse(t.gen, t.info, lotDims(t, size as LotSize), t.plot, size, Number(seed), t.lampTiles, t.windowTiles, residents, garden as GardenSize | undefined) : null;
      generated.set(at, house ?? undefined);
    }
    return generated.get(at);
  }
  return t.houses.find((h) => h.id === id);
}

/** Width and depth of a lot of this size. */
export function lotDims(t: Templates, size: LotSize | undefined): { width: number; depth: number } {
  return t.lotSizes?.[size ?? 'medium'] ?? t.plot;
}

/** Width and depth of a plot's lot: its own (cut to fit its house), else its size's. */
export function slotDims(t: Templates, s: Pick<PlotSlot, 'lot' | 'w' | 'd'>): { width: number; depth: number } {
  return s.w && s.d ? { width: s.w, depth: s.d } : lotDims(t, s.lot);
}

/** Plot `s` on the town: a lot turned a quarter lies across. */
export function plotRect(t: Templates, s: Pick<PlotSlot, 'lot' | 'w' | 'd' | 'x' | 'z' | 'facing'>): Rect {
  const { width, depth } = slotDims(t, s);
  return s.facing % 2 ? { x: s.x, z: s.z, w: depth, d: width } : { x: s.x, z: s.z, w: width, d: depth };
}

/**
 * Plot coordinates onto the town (street at z = 0) for a `width` × `depth` lot at (`x`, `z`)
 * turned `facing`: points, rectangles, walls, doors and windows (edges), and things.
 */
export function turner(x: number, z: number, width: number, depth: number, facing: Facing) {
  const point = ([px, pz]: [number, number]): [number, number] =>
    facing === 0 ? [x + px, z + pz] : facing === 1 ? [x + pz, z + width - px] : facing === 2 ? [x + width - px, z + depth - pz] : [x + depth - pz, z + px];
  const rect = (rc: Rect): Rect => {
    const [ax, az] = point([rc.x, rc.z]);
    const [bx, bz] = point([rc.x + rc.w, rc.z + rc.d]);
    // (Rounded: a driveway's 9.4 m stays 9.4 m turned round.)
    const r = (v: number) => Math.round(v * 1e6) / 1e6;
    return { x: r(Math.min(ax, bx)), z: r(Math.min(az, bz)), w: r(Math.abs(bx - ax)), d: r(Math.abs(bz - az)) };
  };
  /** `lift`: rows to the storey (see `TownLayout.storeys`). */
  const wall = ([x0, z0, x1, z1]: Wall, lift = 0): Wall => {
    const [a, b] = [point([x0, z0]), point([x1, z1])];
    return [a[0], a[1] + lift, b[0], b[1] + lift];
  };
  // Doors and windows sit on tile edges; turning the plot moves an edge's start corner (and turns its axis a quarter).
  const edge = (d: Door, lift = 0): Door => {
    const [ax, az] = point([d.x, d.z]);
    const [bx, bz] = point(d.axis === 'x' ? [d.x + 1, d.z] : [d.x, d.z + 1]);
    return az === bz ? { x: Math.min(ax, bx), z: az + lift, axis: 'x' } : { x: ax, z: Math.min(az, bz) + lift, axis: 'z' };
  };
  const place = (o: Placement, footprint: (def: string) => [number, number], lift = 0): Placement => {
    const [fw, fd] = footprint(o.def);
    const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
    const at = rect({ x: o.x, z: o.z, w, d });
    // Turned with the plot, a thing set off its tile's middle stands off the other way.
    const off = o.offset && (facing === 0 ? o.offset : facing === 1 ? [o.offset[1], -o.offset[0]] : facing === 2 ? [-o.offset[0], -o.offset[1]] : [-o.offset[1], o.offset[0]]);
    return { def: o.def, x: at.x, z: at.z + lift, rot: (o.rot + facing) % 4, ...(o.turn ? { turn: o.turn } : {}), ...(off ? { offset: off as [number, number] } : {}) };
  };
  return { point, rect, wall, edge, place };
}

/** Plot `s`'s coordinates onto the town (see `turner`). */
export function turnOnto(t: Templates, s: PlotSlot): ReturnType<typeof turner> {
  const { width, depth } = slotDims(t, s);
  return turner(s.x, s.z, width, depth, s.facing);
}

/** Farm lots: fields this wide beside the farmhouse's own lot, the farmyard this deep behind it. */
const FARM_FIELDS = 18;
const FARM_YARD = 16;

/**
 * A layout moved onto plot `s`'s lot: centred across it, front kept at the street (a farmhouse
 * stands at its west end, its fields beside it). Objects that would end up off the lot (garden
 * things on a small lot) are left out.
 */
export function onLot(t: Templates, h: HouseTemplate, s: Pick<PlotSlot, 'lot' | 'w' | 'd'>, footprint: (def: string) => [number, number]): HouseTemplate {
  const { width, depth } = slotDims(t, s);
  const frame = h.lot ?? t.plot;
  // A fixed layout's farmhouse stands where it would on a large lot.
  const dx = s.lot !== 'farm' ? Math.floor((width - frame.width) / 2) : h.lot ? 0 : Math.floor((lotDims(t, 'large').width - t.plot.width) / 2);
  if (dx === 0 && depth >= frame.depth) return h;
  const inside = (o: Placement) => {
    const [fw, fd] = footprint(o.def);
    const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
    return o.x >= 0 && o.z >= 0 && o.x + w <= width && o.z + d <= depth;
  };
  const move = <T extends { walls: Wall[]; doors: Door[]; windows?: Door[]; objects: Placement[] }>(f: T): T => ({
    ...f,
    walls: f.walls.map(([x0, z0, x1, z1]) => [x0 + dx, z0, x1 + dx, z1]),
    doors: f.doors.map((d) => ({ ...d, x: d.x + dx })),
    windows: f.windows?.map((d) => ({ ...d, x: d.x + dx })),
    objects: f.objects.map((o) => ({ ...o, x: o.x + dx })).filter(inside),
  });
  return { ...move(h), spawns: h.spawns.map(([x, z]) => [x + dx, z]), upper: h.upper && move(h.upper), basement: h.basement && move(h.basement) };
}

/** A public place moved onto plot `s`'s lot like a house layout (`onLot`): centred across it, front kept at the street. */
export function placeOnLot(t: Templates, p: PublicTemplate, s: Pick<PlotSlot, 'lot' | 'w' | 'd'>): PublicTemplate {
  const dx = Math.floor((slotDims(t, s).width - t.plot.width) / 2);
  if (dx === 0) return p;
  const door = (d: Door): Door => ({ ...d, x: d.x + dx });
  return {
    ...p,
    walls: p.walls?.map(([x0, z0, x1, z1]) => [x0 + dx, z0, x1 + dx, z1]),
    doors: p.doors?.map(door),
    windows: p.windows?.map(door),
    objects: p.objects.map((o) => ({ ...o, x: o.x + dx })),
    paths: p.paths?.map((r) => ({ ...r, x: r.x + dx })),
    entry: p.entry && [p.entry[0] + dx, p.entry[1]],
  };
}

/** Whether a layout's house fits on a lot of `size` (with a metre to spare around its walls). */
export function fitsLot(t: Templates, h: HouseTemplate, size: LotSize): boolean {
  const { width, depth } = lotDims(t, houseLot(size));
  const dx = Math.floor((width - t.plot.width) / 2);
  const xs = h.walls.flatMap(([x0, , x1]) => [x0 + dx, x1 + dx]);
  const zs = h.walls.flatMap(([, z0, , z1]) => [z0, z1]);
  return Math.min(...xs) >= 1 && Math.max(...xs) <= width - 1 && Math.max(...zs) <= depth - 1;
}

/**
 * A house for a new lot of `size` with a `garden` round it, and the lot's own width and depth: a
 * generated house (see `housegen.ts`) on a lot cut to fit it, else a fixed layout that fits.
 */
export function houseFor(t: Templates, size: LotSize, garden: GardenSize, rand: Rand = Math.random): Pick<PlotSlot, 'template' | 'w' | 'd' | 'garden'> {
  if (t.gen) {
    const id = generatedId(houseLot(size), (rand() * 2 ** 31) >>> 0, garden);
    const h = houseTemplate(t, id);
    if (h) return { template: id, garden, ...lotOf(h, size) };
  }
  const fitting = t.houses.filter((h) => fitsLot(t, h, size));
  return { template: pick(fitting.length ? fitting : t.houses, rand).id };
}

/** A generated house's lot (a farm's: its fields beside it, its farmyard behind). */
function lotOf(h: HouseTemplate, size: LotSize): Pick<PlotSlot, 'w' | 'd'> {
  if (!h.lot) return {};
  return size === 'farm' ? { w: h.lot.width + FARM_FIELDS, d: h.lot.depth + FARM_YARD } : { w: h.lot.width, d: h.lot.depth };
}

/**
 * The draft with the lot of `slot` cut for a `garden` of another size round its house (the same
 * house), its front kept where it was; unchanged when that lot doesn't fit there.
 */
export function withGarden(t: Templates, town: NeighbourhoodDraft, slot: number, garden: GardenSize): NeighbourhoodDraft {
  const old = town.slots[slot];
  if (!old || old.garden === garden || !old.template.startsWith('gen:')) return town;
  const [, size, seed] = old.template.split(':');
  const template = generatedId(size as LotSize, Number(seed), garden);
  const h = houseTemplate(t, template);
  if (!h) return town;
  const was = plotRect(t, old);
  const now = plotRect(t, { ...old, ...lotOf(h, old.lot ?? 'medium') });
  // The front stays on the street, its middle where it was cut round (`anchor`).
  const front = [was.z, was.x, was.z + was.d, was.x + was.w][old.facing];
  const anchor = old.anchor ?? (old.facing % 2 ? was.z + was.d / 2 : was.x + was.w / 2);
  const at = fronting(old.facing, front, anchor, now.w, now.d);
  const ground = new Ground(town.width, town.depth, town.roads, town.greens);
  for (const s of town.slots) if (s.index !== slot) ground.take(plotRect(t, s));
  if (!ground.free(at, 1)) return town;
  return { ...town, slots: town.slots.map((s) => (s.index === slot ? { ...s, x: at.x, z: at.z, template, garden, ...lotOf(h, s.lot ?? 'medium') } : s)) };
}

/** Each set of roads' land (see `Ground`): the tiles they take alone, and with their greens. */
const groundOf = new WeakMap<readonly RoadPlan[], { w: number; d: number; greens: readonly Pt[][]; roads: Uint8Array; base: Uint8Array }>();

/** A town's land as its lots go down: the tiles roads, greens and lots already take. */
class Ground {
  private readonly taken: Uint8Array;
  /** The tiles the roads alone take. */
  private readonly roads: Uint8Array;

  constructor(
    readonly w: number,
    readonly d: number,
    roads: readonly RoadPlan[],
    greens: readonly Pt[][] = [],
  ) {
    let had = groundOf.get(roads);
    if (!had || had.w !== w || had.d !== d || had.greens !== greens) {
      const base = roadMask(roads, w, d);
      const mask = base.slice();
      for (const g of greens) {
        const [x0, x1] = [Math.max(0, Math.floor(Math.min(...g.map((p) => p[0])))), Math.min(w, Math.ceil(Math.max(...g.map((p) => p[0]))))];
        const [z0, z1] = [Math.max(0, Math.floor(Math.min(...g.map((p) => p[1])))), Math.min(d, Math.ceil(Math.max(...g.map((p) => p[1]))))];
        for (let z = z0; z < z1; z++) for (let x = x0; x < x1; x++) if (inside(g, x + 0.5, z + 0.5)) mask[z * w + x] = 1;
      }
      groundOf.set(roads, (had = { w, d, greens, roads: base, base: mask }));
    }
    this.roads = had.roads;
    this.taken = had.base.slice();
  }

  /** Whether `r` is clear, `margin` m inside the edge of the land (`greens`: it may lie on a green, off the roads). */
  free(r: Rect, margin = MARGIN, greens = false): boolean {
    if (r.x < margin || r.z < margin || r.x + r.w > this.w - margin || r.z + r.d > this.d - margin) return false;
    const mask = greens ? this.roads : this.taken;
    for (let z = r.z; z < r.z + r.d; z++) for (let x = r.x; x < r.x + r.w; x++) if (mask[z * this.w + x]) return false;
    return true;
  }

  /** Takes `r` (and `grow` m round it). */
  take(r: Rect, grow = 0): void {
    for (let z = Math.max(0, r.z - grow); z < Math.min(this.d, r.z + r.d + grow); z++) {
      for (let x = Math.max(0, r.x - grow); x < Math.min(this.w, r.x + r.w + grow); x++) this.taken[z * this.w + x] = 1;
    }
  }

  /** Whether the tile under a point is taken. */
  takenAt(x: number, z: number): boolean {
    const [tx, tz] = [Math.floor(x), Math.floor(z)];
    return tx < 0 || tz < 0 || tx >= this.w || tz >= this.d || this.taken[tz * this.w + tx] === 1;
  }
}

/** A place along a road a lot could front onto: the road, where along it, which side (`n`: from the road towards the lot). */
interface Frontage {
  road: number;
  s: number;
  side: 1 | -1;
  p: Pt;
  n: Pt;
  /** How far from the town's middle (with a little noise: the town's edge is ragged). */
  far: number;
}

/** Which way a lot on the side of a road that `n` points to faces (towards the road). */
function facingFrom([nx, nz]: Pt): Facing {
  return Math.abs(nz) >= Math.abs(nx) ? (nz > 0 ? 0 : 2) : nx > 0 ? 1 : 3;
}

/**
 * Where a lot `w` wide along its street and `d` deep lies if its front is at the verge of road
 * `road` at frontage `f`: turned to face the road, set back a little more if it must (on a bend);
 * null if it doesn't fit there.
 */
function lotAt(ground: Ground, road: RoadPlan, f: Frontage, w: number, d: number): { x: number; z: number; facing: Facing; anchor: number } | null {
  const facing = facingFrom(f.n);
  const [W, D] = facing % 2 ? [d, w] : [w, d];
  const reach = road.width / 2 + VERGE;
  const [qx, qz] = [f.p[0] + f.n[0] * reach, f.p[1] + f.n[1] * reach];
  // (Quickly past frontage a lot already takes.)
  if (ground.takenAt(qx + f.n[0] * 3, qz + f.n[1] * 3)) return null;
  // On a bend or across a road running aslant, further back (a wedge of front garden before it).
  const [across, along] = facing % 2 ? [qx, qz] : [qz, qx];
  for (let back = 0; back <= Math.min(10, Math.ceil(w / 2)); back++) {
    // The front edge on the first tile line past the verge (further on, the further back).
    const front = facing < 2 ? Math.ceil(across) + back : Math.floor(across) - back;
    const r = fronting(facing, front, along, W, D);
    if (ground.free(r)) return { x: r.x, z: r.z, facing, anchor: along };
  }
  return null;
}

/**
 * A `W` × `D` rectangle (town tiles) facing `facing`'s way, its front edge on the tile line `front`
 * (z when it faces north or south, x when east or west) and its middle at `along` across it.
 */
function fronting(facing: Facing, front: number, along: number, W: number, D: number): Rect {
  const a = Math.round(along - (facing % 2 ? D : W) / 2);
  const at = [{ x: a, z: front }, { x: front, z: a }, { x: a, z: front - D }, { x: front - W, z: a }][facing];
  return { ...at, w: W, d: D };
}

/** Template id of a lot with nothing on it yet (no house template has it): build from the ground up. */
export const EMPTY_LOT = 'empty';

/** The draft with the house on `slot` cleared away (an empty lot), or put back (`template`). */
export function withLot(town: NeighbourhoodDraft, slot: number, template: string): NeighbourhoodDraft {
  if (town.slots[slot]?.template === template) return town;
  return { ...town, slots: town.slots.map((s) => (s.index === slot ? { ...s, template } : s)) };
}

/** Residents the neighbours number at most (sim-core holds 64 Sims: room for the player's household and babies). */
const MAX_NEIGHBOURS = 48;

/**
 * Lays out a world of `kind` from `seed` (the same kind and seed always give the same town): its
 * roads, lots, houses, public places and farms, its name, tagline and land. The roads come first
 * (`WorldRecipe.roads`), then the lots along them: the park on the green if there is one, the
 * public places round the town's middle, farms out on its edge, the houses from the middle out,
 * each turned to face its road, with a gap or a stretch of meadow between some. The town is cut
 * to the land its lots and roads take. Nobody lives there yet (see `populate`).
 */
export function generateWorld(content: Content, t: Templates, kind: WorldKind, seed = (Math.random() * 2 ** 31) >>> 0, name?: string): NeighbourhoodDraft {
  const recipe = WORLDS[kind];
  const rand = random(seed);
  // Rolled whether used or not, so a name typed in leaves the rest of the world as it was.
  const rolledName = worldName(content, kind, rand);
  const tagline = worldTagline(content, kind, rand);
  const { box } = recipe;
  const layout = recipe.roads(rand, box);
  // Dead ends stop short of the edge of the land (the roads out of town go on past it); lanes
  // begin at the kerb of the road they join.
  const within = ([x, z]: Pt) => x > 14 && z > 14 && x < box.w - 14 && z < box.d - 14;
  const roads = trimJoins(
    layout.roads.map((r) => {
      const out = r.ends[1] === 'dead' ? r.points.findIndex((p, i) => i >= 2 && !within(p)) : -1;
      return out < 0 ? r : { ...r, points: r.points.slice(0, out) };
    }),
  );
  const names = streetNames(content, kind, rand, roads.length);
  const greens = layout.greens ?? [];
  const centre = layout.centre ?? roads[0].points[Math.floor(roads[0].points.length / 2)];

  // What goes on the lots: the park and the other public places, farms, houses.
  const [fewest, most] = recipe.places;
  const count = fewest + Math.floor(rand() * (most - fewest + 1));
  // The places this kind of world would rather have first, in no particular order, then the rest.
  const prefer = recipe.prefer ?? [];
  const others = shuffled(t.parks.slice(1), rand)
    .sort((a, b) => Number(!prefer.includes(a.id)) - Number(!prefer.includes(b.id)))
    .slice(0, count);
  const [fewFarms, mostFarms] = recipe.farms ?? [0, 0];
  const farmCount = t.lotSizes?.farm ? fewFarms + Math.floor(rand() * (mostFarms - fewFarms + 1)) : 0;
  const farmWords = shuffled(content.names.worlds?.farm?.start ?? ['Meadow', 'Barley', 'Clover'], rand);
  const lotSize = (): LotSize => {
    if (!t.lotSizes) return 'medium';
    const r = rand();
    return r < recipe.small ? 'small' : r < 1 - recipe.large ? 'medium' : 'large';
  };
  const gardenSize = () => weighted(rand, recipe.gardens as Record<string, number>) as GardenSize;
  type Unplaced = Omit<PlotSlot, 'index' | 'x' | 'z' | 'facing'>;
  const wanted: Unplaced[] = [
    // On a lot of the frame they are drawn for.
    ...[t.parks[0], ...others].map((p): Unplaced => ({ kind: p.kind ?? 'park', template: p.id, name: p.name, lot: 'medium', w: t.plot.width, d: t.plot.depth })),
    ...Array.from({ length: farmCount }, (_, i): Unplaced => ({ kind: 'house', name: `${farmWords[i % farmWords.length]} Farm`, lot: 'farm', ...houseFor(t, 'farm', gardenSize(), rand) })),
  ];
  while (wanted.length < recipe.lots) {
    const lot = lotSize();
    wanted.push({ kind: 'house', name: '', lot, ...houseFor(t, lot, gardenSize(), rand) });
  }

  // Every place along every road a lot could front onto, nearest the town's middle first.
  const ground = new Ground(box.w, box.d, roads, greens);
  const frontage: Frontage[] = roads.flatMap((road, r) => {
    const ls = lengths(road.points);
    return (road.lots ? [road.lots] : ([1, -1] as const)).flatMap((side) =>
      road.points
        .map((p, i): Frontage => {
          const n = leftOf(tangent(road.points, i)).map((v) => v * side) as Pt;
          return { road: r, s: ls[i], side, p, n, far: Math.hypot(p[0] - centre[0], p[1] - centre[1]) + rand() * 10 };
        })
        // Not where it joins another road, nor right at its ends.
        .filter((f) => f.s > 4 && f.s < ls[ls.length - 1] - 4),
    );
  });
  frontage.sort((a, b) => a.far - b.far);
  const outskirts = [...frontage].reverse();

  const slots: PlotSlot[] = [];
  const placed = new Map<PlotSlot, Frontage>();
  const put = (u: Unplaced, at: { x: number; z: number; facing: Facing; anchor?: number }, f: Frontage | null) => {
    const slot: PlotSlot = { ...u, index: slots.length, ...at };
    if (u.lot === 'farm') slot.farm = farmstead(content, t, slot, rand);
    slots.push(slot);
    if (f) placed.set(slot, f);
    // Mostly next door to each other; now and then a gap, or a stretch of meadow.
    const [gap, meadow] = recipe.gaps;
    const r = rand();
    ground.take(plotRect(t, slot), r < meadow ? 5 + Math.floor(rand() * 6) : r < meadow + gap ? 1 + Math.floor(rand() * 2) : 0);
  };
  for (const u of wanted) {
    const { width, depth } = slotDims(t, u);
    // The park on the green, facing the road nearest it, if it fits there.
    if (u === wanted[0] && greens.length) {
      const [gx, gz] = centre;
      const near = roads.map((road, r) => ({ r, ...nearest(road.points, gx, gz) })).sort((a, b) => a.dist - b.dist)[0];
      const [qx, qz] = pointAt(roads[near.r].points, near.s).p;
      const facing = facingFrom([gx - qx, gz - qz]);
      const [W, D] = facing % 2 ? [depth, width] : [width, depth];
      const r = { x: Math.round(gx - W / 2), z: Math.round(gz - D / 2), w: W, d: D };
      const corners: Pt[] = [[r.x, r.z], [r.x + r.w, r.z], [r.x, r.z + r.d], [r.x + r.w, r.z + r.d]];
      if (greens.some((g) => corners.every(([x, z]) => inside(g, x, z))) && ground.free(r, MARGIN, true)) {
        put(u, { x: r.x, z: r.z, facing }, null);
        continue;
      }
    }
    for (const f of u.lot === 'farm' ? outskirts : frontage) {
      const at = lotAt(ground, roads[f.road], f, width, depth);
      if (!at) continue;
      put(u, at, f);
      break;
    }
  }

  // House numbers along each road: odd on its left, even on its right.
  for (const [r, road] of roads.entries()) {
    const along = [...placed].filter(([s, f]) => f.road === r && s.kind === 'house' && s.lot !== 'farm').sort((a, b) => a[1].s - b[1].s);
    for (const side of [1, -1] as const) along.filter(([, f]) => f.side === side).forEach(([s], i) => (s.name = `${i * 2 + (side > 0 ? 1 : 2)} ${names[r]}`));
    road.name = names[r];
  }

  // The town: as far as its lots, its greens and the roads that don't lead out of it go, and a margin.
  const xs: number[] = [];
  const zs: number[] = [];
  const add = ([x, z]: Pt) => (xs.push(x), zs.push(z));
  for (const s of slots) {
    const r = plotRect(t, s);
    add([r.x, r.z]);
    add([r.x + r.w, r.z + r.d]);
  }
  greens.flat().forEach(add);
  for (const road of roads) if (!road.ends.includes('out')) road.points.forEach(add);
  const [x0, z0] = [Math.max(0, Math.floor(Math.min(...xs) - MARGIN)), Math.max(0, Math.floor(Math.min(...zs) - MARGIN))];
  const width = Math.min(MAX_TOWN, Math.ceil(Math.min(box.w, Math.max(...xs) + MARGIN)) - x0);
  const depth = Math.min(MAX_TOWN, Math.ceil(Math.min(box.d, Math.max(...zs) + MARGIN)) - z0);
  const moved = slots.map((s) => ({ ...s, x: s.x - x0, z: s.z - z0, ...(s.anchor === undefined ? {} : { anchor: s.anchor - (s.facing % 2 === 0 ? x0 : z0) }) }));
  const kept = cropRoads(roads, -x0, -z0, width, depth);
  const landscape = recipe.landscape(rand, width, depth, layout.water);
  return {
    name: name ?? rolledName,
    width,
    depth,
    slots: moved,
    roads: kept,
    greens: greens.map((g) => g.map(([x, z]): Pt => [Math.round((x - x0) * 100) / 100, Math.round((z - z0) * 100) / 100])),
    households: [],
    seed,
    world: { kind, tagline, landscape },
  };
}

/** Tries at a household for a home: one that fits it and can buy it, not too rich for it. */
const HOUSEHOLD_TRIES = 36;
/** Savings beyond this many times a home's price make a household too well off for it (taken only when nothing closer turns up). */
const RICH_FOR = 4;

/**
 * Neighbours for a town: `count` households (default: all but two houses empty), each in a home
 * that has room for it and that it could buy (`wealth.ts` `savingsOf` covers `price.ts`
 * `homePrice`), no more than `MAX_NEIGHBOURS` residents in all. The houses are picked at random
 * and filled dearest first; one no household turns up for stays for sale.
 */
export function populate(content: Content, assets: AssetRegistry, t: Templates, town: NeighbourhoodDraft, count?: number): NeighbourhoodDraft {
  const houses = town.slots.filter((s) => s.kind === 'house');
  const wanted = Math.max(0, Math.min(count ?? houses.length - 2, houses.length - 1));
  const price = new Map(houses.map((s) => [s.index, homePrice(content, t, s)]));
  const dearestFirst = (list: PlotSlot[]) => [...list].sort((a, b) => price.get(b.index)! - price.get(a.index)!);
  // The houses picked first; the others stand in for any of them nobody could buy. The town's
  // cheapest home stays for sale (if it can): a starter home within reach of a young household.
  const cheapest = houses.reduce<PlotSlot | undefined>((a, s) => (!a || price.get(s.index)! < price.get(a.index)! ? s : a), undefined);
  const order = [...shuffled(houses.filter((s) => s !== cheapest)), ...(cheapest ? [cheapest] : [])];
  const queue = [...dearestFirst(order.slice(0, wanted)), ...dearestFirst(order.slice(wanted))];
  const taken = new Set<string>();
  const households: NeighbourhoodDraft['households'] = [];
  let residents = 0;
  for (const slot of queue) {
    if (households.length >= wanted || residents >= MAX_NEIGHBOURS) break;
    const bedrooms = houseTemplate(t, slot.template)?.bedrooms ?? 1;
    const household = householdFor(content, assets, bedrooms, price.get(slot.index)!, MAX_NEIGHBOURS - residents, [...taken]);
    if (!household) continue;
    taken.add(household.name);
    residents += household.members.length;
    households.push({ household, slot: slot.index });
  }
  return { ...town, households };
}

/**
 * A household for a home with `bedrooms` that costs `price`: it fits (a couple shares a bedroom,
 * two children may, anyone else needs one), has saved enough to buy it, and preferably isn't
 * many times richer than it needs to be. A dear home looks mostly among older households, who
 * have climbed higher and put more by. Null if none turns up.
 */
function householdFor(
  content: Content,
  assets: AssetRegistry,
  bedrooms: number,
  price: number,
  room: number,
  avoidNames: readonly string[],
): HouseholdDraft | null {
  let size = Math.max(1, Math.min(4, bedrooms + Math.floor(Math.random() * 2), room));
  let best: { household: HouseholdDraft; savings: number } | null = null;
  // A home dearer than two young grown-ups between jobs could buy goes mostly to an older household.
  const young = (content.adultStages[0]?.from ?? 18) + 7;
  const dear = price > savingsOf(content, { members: [{ age: young }, { age: young }] });
  for (let tries = 0, misfits = 0; tries < HOUSEHOLD_TRIES; tries++) {
    const household = randomHousehold(content, assets, size, avoidNames, { older: dear ? Math.random() < 0.8 : Math.random() < 0.2 });
    if (bedroomsNeeded(residentsOf(content, household)) > bedrooms) {
      if (++misfits % 6 === 0 && size > 1) size--;
      continue;
    }
    const savings = savingsOf(content, household);
    if (savings < price) continue;
    if (savings <= Math.max(price, 1) * RICH_FOR) return household;
    if (!best || savings < best.savings) best = { household, savings };
  }
  return best?.household ?? null;
}

/**
 * Another household for the home on `slot` (one that fits it and can buy it, as `populate` finds
 * them), with a name none of `avoidNames` has; any household of `size` if none turns up.
 */
export function rehouse(content: Content, assets: AssetRegistry, t: Templates, town: NeighbourhoodDraft, slot: number, size: number, avoidNames: readonly string[]): HouseholdDraft {
  const s = town.slots[slot];
  const bedrooms = (s && houseTemplate(t, s.template)?.bedrooms) ?? 1;
  return (s && householdFor(content, assets, bedrooms, homePrice(content, t, s), MAX_NEIGHBOURS, avoidNames)) ?? randomHousehold(content, assets, size, avoidNames);
}

/** A town of one of the older sizes with its neighbours (a large one is a dense town now). */
export function generateNeighbourhood(
  content: Content,
  assets: AssetRegistry,
  t: Templates,
  size: TownSize,
  name?: string,
  neighbours?: number,
  seed?: number,
): NeighbourhoodDraft {
  return populate(content, assets, t, generateWorld(content, t, size === 'large' ? 'dense' : size, seed, name), neighbours);
}

/** What a farm's fields grow: two or three of these, a pair of rows each in turn. */
const CROPS = ['garden.vegPatch', 'garden.pumpkins', 'garden.sunflowers', 'garden.strawberries', 'garden.tomatoes'];
/** The farmyard behind a farmhouse, biggest first: [object, fewest, most, turned (rot) or null: any way]. */
const FARMYARD: [string, number, number, number | null][] = [
  ['farm.barn', 1, 1, 2],
  ['farm.silo', 1, 1, 2],
  ['farm.tractor', 0, 1, null],
  ['farm.coop', 1, 1, 2],
  ['farm.trough', 1, 1, null],
  ['farm.hayStack', 1, 2, null],
  ['farm.hayBale', 2, 4, null],
];

/**
 * A farm lot's fields and farmyard (plot coordinates, street at z = 0), round a farmhouse that
 * stands as on a large lot: a field of crops in rows beside the house (a path between each pair
 * of rows), a fallow field with hay behind it, and the barn, silo, coop and the rest in a yard
 * behind the house. Clear of the house, its garden and its driveway.
 */
function farmstead(content: Content, t: Templates, slot: PlotSlot, rand: Rand): { objects: Placement[]; fields: Rect[] } {
  const footprint = (def: string): [number, number] => content.object(def)?.footprint ?? [1, 1];
  const has = (def: string) => !!content.object(def);
  const { width: pw, depth: pd } = slotDims(t, slot);
  const template = houseTemplate(t, slot.template);
  // The farmhouse's own lot (a fixed layout's: a large one).
  const large = template?.lot ?? lotDims(t, 'large');
  const drive = template ? driveway(onLot(t, template, slot, footprint), pw, pd, footprint) : null;
  const east = Math.ceil(Math.max(large.width, drive ? drive.x + drive.w : 0)) + 1;
  const objects: Placement[] = [];
  const taken: Rect[] = [];
  const free = (r: Rect) => !taken.some((o) => r.x < o.x + o.w + 1 && r.x + r.w + 1 > o.x && r.z < o.z + o.d + 1 && r.z + r.d + 1 > o.z);
  const put = (def: string, x: number, z: number, rot: number) => {
    const [fw, fd] = footprint(def);
    const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
    objects.push({ def, x, z, rot });
    taken.push({ x, z, w, d });
  };

  // Crops beside the house.
  const crops = shuffled(CROPS.filter(has), rand).slice(0, 2 + Math.floor(rand() * 2));
  // An odd number of tiles across, so its rows of crops line up with the furrows the renderer draws (street.ts `fieldLayers`).
  const across = pw - 1 - east;
  const cropField: Rect = { x: east + 1 - (across % 2), z: 2, w: across - 1 + (across % 2), d: Math.min(22, Math.floor(pd * 0.5)) };
  const fallow: Rect = { x: east, z: cropField.z + cropField.d + 2, w: cropField.w, d: pd - 2 - (cropField.z + cropField.d + 2) };
  if (crops.length && cropField.w >= 3) {
    // A scarecrow in one of the rows, halfway back (not on a path: those lead to the plants).
    const rows = Math.floor((cropField.w - 1) / 2);
    const scarecrow = has('farm.scarecrow') ? { x: cropField.x + 1 + 2 * Math.floor(rand() * rows), z: cropField.z + Math.floor(cropField.d / 2) } : null;
    for (let i = 0, x = cropField.x + 1; x < cropField.x + cropField.w - 1; i++, x += 2) {
      const crop = crops[Math.floor(i / 2) % crops.length];
      // Facing the paths either side (the next plant in the row stands in front otherwise).
      for (let z = cropField.z + 1; z < cropField.z + cropField.d - 1; z++) {
        if (scarecrow?.x === x && scarecrow.z === z) put('farm.scarecrow', x, z, 2);
        else put(crop, x, z, i % 2 === 0 ? 1 : 3);
      }
    }
  }
  // Hay on the fallow field.
  const [bw, bd] = footprint('farm.hayBale');
  if (has('farm.hayBale') && fallow.d >= bd + 2) {
    for (let k = 0, n = 3 + Math.floor(rand() * 4); k < n; k++) {
      const x = fallow.x + 1 + Math.floor(rand() * Math.max(1, fallow.w - 1 - bw));
      const z = fallow.z + 1 + Math.floor(rand() * Math.max(1, fallow.d - 1 - bd));
      if (free({ x, z, w: bw, d: bd })) put('farm.hayBale', x, z, Math.floor(rand() * 4));
    }
  }

  // The farmyard behind the house.
  const yard: Rect = { x: 1, z: large.depth + 1, w: east - 2, d: pd - large.depth - 2 };
  for (const [def, fewest, most, turned] of FARMYARD) {
    if (!has(def)) continue;
    for (let k = 0, n = fewest + Math.floor(rand() * (most - fewest + 1)); k < n; k++) {
      for (let tries = 0; tries < 40; tries++) {
        const rot = turned ?? Math.floor(rand() * 4);
        const [fw, fd] = footprint(def);
        const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
        if (w > yard.w || d > yard.d) break;
        const x = yard.x + Math.floor(rand() * (yard.w - w + 1));
        // Big buildings stand at the back of the yard, facing the house.
        const z = turned === 2 ? yard.z + yard.d - d : yard.z + Math.floor(rand() * (yard.d - d + 1));
        if (!free({ x, z, w, d })) continue;
        put(def, x, z, rot);
        break;
      }
    }
  }
  return { objects, fields: [cropField, ...(fallow.d >= 3 ? [fallow] : [])] };
}

/** Driveways are paths wider than this (garden paths are 1 m); see `driveway`. */
export const DRIVEWAY_MIN_WIDTH = 1.8;
/** Driveway length past the front wall of the house: room to park a car beside it. */
const DRIVEWAY_PAST_FRONT = 5.4;

/**
 * Driveway of a house layout in plot coordinates (front edge at z = 0): from the street to
 * beside the house, on the side with more room (away from the front door when even), clear
 * of garden objects. Null when neither side has room.
 */
export function driveway(h: HouseTemplate, pw: number, pd: number, footprint: (def: string) => [number, number]): Rect | null {
  const xs = h.walls.flatMap(([x0, , x1]) => [x0, x1]);
  const zs = h.walls.flatMap(([, z0, , z1]) => [z0, z1]);
  const [hx0, hx1, hz0] = [Math.min(...xs), Math.max(...xs), Math.min(...zs)];
  const door = h.doors[0]?.x ?? (hx0 + hx1) / 2;
  const d = Math.min(hz0 + DRIVEWAY_PAST_FRONT, pd - 1);
  const sides = [
    { gap: pw - hx1, far: door < (hx0 + hx1) / 2, rect: (w: number, gap: number) => ({ x: hx1 + Math.min(0.6, gap - w - 0.3), z: 0, w, d }) },
    { gap: hx0, far: door >= (hx0 + hx1) / 2, rect: (w: number, gap: number) => ({ x: hx0 - Math.min(0.6, gap - w - 0.3) - w, z: 0, w, d }) },
  ].sort((a, b) => b.gap - a.gap || Number(b.far) - Number(a.far));
  const blocked = (r: Rect) =>
    h.objects.some((o) => {
      const [fw, fd] = footprint(o.def);
      const [w, dd] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
      return o.x < r.x + r.w && o.x + w > r.x && o.z < r.z + r.d && o.z + dd > r.z;
    });
  for (const side of sides) {
    const w = Math.min(3, side.gap - 0.6);
    if (w < 2.3) continue;
    const r = side.rect(w, side.gap);
    if (!blocked(r)) return r;
  }
  return null;
}

export function vacantSlots(town: NeighbourhoodDraft): PlotSlot[] {
  const occupied = new Set(town.households.map((h) => h.slot));
  return town.slots.filter((s) => s.kind === 'house' && !occupied.has(s.index));
}

/** Town geometry in world tiles: what `assembleTown` writes and the menu preview draws. */
export interface TownLayout {
  walls: Wall[];
  doors: Door[];
  windows: Door[];
  objects: Placement[];
  /** Garden paths (1 m) and driveways (wider than `DRIVEWAY_MIN_WIDTH`). */
  paths: Rect[];
  /** Farm fields (see `WorldLook.fields`). */
  fields: Rect[];
  plots: { name: string; x: number; z: number; w: number; d: number; public: boolean; entry: [number, number]; storeys: number }[];
  /** Each plot's `Facing` (as `TownMeta.facing`). */
  facing: Facing[];
  /** Storeys of the tallest house: storey `k`'s walls and objects are on rows `k * depth ..` (see `WorldStructure.storeys`). */
  storeys: number;
  /** Whether a house has a basement: then basements are on the block of rows after the storeys', `storeys * depth ..`. */
  basement: boolean;
}

/**
 * Stamps every plot's house layout (or its public place: a park, a square, a shop) onto the town. `playerSlot` is left without a
 * driveway (it stays open for building); null gives every house one. Each generated house is
 * furnished for its household (`player`: a new one moving in at `playerSlot`).
 */
export function layoutTown(content: Content, t: Templates, town: NeighbourhoodDraft, playerSlot: number | null, player?: HouseholdDraft): TownLayout {
  const walls: Wall[] = [];
  const doors: Door[] = [];
  const windows: Door[] = [];
  const objects: Placement[] = [];
  const paths: Rect[] = [];
  const fields: Rect[] = [];
  const footprint = (def: string): [number, number] => content.object(def)?.footprint ?? [1, 1];

  /** Things of a layout onto the town, `lift` rows up (see `TownLayout.storeys`). */
  const placeAll = (s: PlotSlot, list: Placement[], lift = 0) => {
    const turn = turnOnto(t, s);
    for (const o of list) objects.push(turn.place(o, footprint, lift));
  };
  /** Walls, doors and windows of a layout (a house or a shop), `lift` rows up. */
  const shell = (s: PlotSlot, f: { walls?: Wall[]; doors?: Door[]; windows?: Door[] }, lift = 0) => {
    const turn = turnOnto(t, s);
    walls.push(...(f.walls ?? []).map((w) => turn.wall(w, lift)));
    doors.push(...(f.doors ?? []).map((d) => turn.edge(d, lift)));
    windows.push(...(f.windows ?? []).map((d) => turn.edge(d, lift)));
  };
  /**
   * A path's way on from the front of plot `s` to its road, across the verge (`r`: its stretch on the
   * plot, plot coordinates): as far as the sidewalk, longer where the plot stands back from a bend.
   */
  const roads = roadIndex(town.roads);
  const verge = (s: PlotSlot, r: Rect) => {
    const turn = turnOnto(t, s);
    for (let k = 1; k <= 12; k++) {
      const [x, z] = turn.point([r.x + r.w / 2, -k + 0.5]);
      if (!roads.onRoad(x, z)) continue;
      paths.push(turn.rect({ x: r.x, z: -k, w: r.w, d: k }));
      return;
    }
  };
  /** A garden path from a front door (plot coordinates) to the street, and on across the verge. */
  const pathTo = (s: PlotSlot, front: Door) => {
    const r = { x: front.x, z: 0, w: 1, d: front.z };
    paths.push(turnOnto(t, s).rect(r));
    verge(s, r);
  };
  const living = new Map(town.households.map((h) => [h.slot, h.household]));
  if (player && playerSlot !== null) living.set(playerSlot, player);
  const layoutOf = (s: PlotSlot) => {
    const who = living.get(s.index);
    const h = houseTemplate(t, s.template, who && residentsOf(content, who));
    return h && onLot(t, h, s, footprint);
  };
  const placeOf = (s: PlotSlot) => {
    const place = s.kind === 'house' ? undefined : t.parks.find((p) => p.id === s.template);
    return place && placeOnLot(t, place, s);
  };

  const plots = town.slots.map((s) => {
    // Visitors arrive in front of the door (the first spawn point of the house layout).
    const spawn = layoutOf(s)?.spawns[0] ?? placeOf(s)?.entry;
    const entry = turnOnto(t, s).point(spawn ?? [slotDims(t, s).width / 2, 1.5]);
    return { name: s.name, ...plotRect(t, s), public: s.kind !== 'house', entry, storeys: layoutOf(s)?.upper ? 2 : 1 };
  });
  const storeys = Math.max(1, ...plots.map((p) => p.storeys));
  let basement = false;
  for (const s of town.slots) {
    if (s.kind !== 'house') {
      const place = placeOf(s);
      if (!place) continue;
      shell(s, place);
      placeAll(s, place.objects);
      if (place.doors?.[0]) pathTo(s, place.doors[0]);
      paths.push(...(place.paths ?? []).map(turnOnto(t, s).rect));
      continue;
    }
    // A farm's fields and farmyard (an empty farm lot keeps them too).
    if (s.farm) {
      placeAll(s, s.farm.objects);
      fields.push(...s.farm.fields.map(turnOnto(t, s).rect));
    }
    const h = layoutOf(s);
    if (!h) continue;
    const { width: pw, depth: pd } = slotDims(t, s);
    shell(s, h);
    placeAll(s, h.objects);
    // The storey above, on its own rows of the lot.
    if (h.upper) {
      shell(s, h.upper, town.depth);
      placeAll(s, h.upper.objects, town.depth);
    }
    // The basement, on the rows after every storey's.
    if (h.basement) {
      shell(s, h.basement, storeys * town.depth);
      placeAll(s, h.basement.objects, storeys * town.depth);
      basement = true;
    }
    // A garden path from the front door to the street.
    if (h.doors[0]) pathTo(s, h.doors[0]);
    // Neighbours get a driveway beside the house (the renderer parks a car on it); the player's
    // plot stays open for building.
    const drive = s.index === playerSlot ? null : driveway(h, pw, pd, footprint);
    if (drive) {
      paths.push(turnOnto(t, s).rect(drive));
      verge(s, drive);
    }
  }
  return { walls, doors, windows, objects, paths, fields, plots, facing: town.slots.map((s) => s.facing), storeys, basement };
}

/** What the renderer needs to know about a town's world (it survives save/load in `meta`). */
export function worldLook(town: NeighbourhoodDraft, fields: Rect[]): WorldLook | undefined {
  return town.world && { kind: town.world.kind, landscape: town.world.landscape, ...(fields.length ? { fields } : {}) };
}

export interface AssembleOptions {
  /** `player` is the neighbour household already living at the player's slot. */
  existing?: boolean;
  mode?: GameKind;
  lifespan?: Lifespan;
}

/**
 * Builds the simulation's town file. A new player household moves in as household 0, before the
 * neighbours. With `existing`, `player` is the neighbour household already
 * living at `playerSlot`: it stays where it is in the list (with its jobs and history) and only
 * becomes the player's. A new household on an empty lot gets money to build with in Living games.
 */
export function assembleTown(
  content: Content,
  t: Templates,
  town: NeighbourhoodDraft,
  player: HouseholdDraft,
  playerSlot: number,
  { existing = false, mode = 'living', lifespan = 'normal' }: AssembleOptions = {},
): string {
  const { walls, doors, windows, objects, paths, fields, plots, facing, storeys, basement } = layoutTown(content, t, town, playerSlot, existing ? undefined : player);
  const footprint = (def: string): [number, number] => content.object(def)?.footprint ?? [1, 1];

  if (existing && !town.households.some((h) => h.slot === playerSlot)) throw new Error('Nobody lives in that house.');
  // A new household first, then the neighbours (one of whom may be the player's).
  const neighbours = town.households.map((h) => ({ ...h, player: existing && h.slot === playerSlot }));
  const home = town.slots[playerSlot];
  const residents: { household: HouseholdDraft; slot: number; player: boolean }[] = existing
    ? neighbours
    : [{ household: player, slot: playerSlot, player: true }, ...neighbours];
  const emptyLot = !houseTemplate(t, home.template);
  // What each household has left to spend once it has bought its home (a new household on an
  // empty lot: the lot, and money to build with besides). A new household in a Creative game
  // has the game's money.
  const funds = (r: (typeof residents)[number], i: number): number | undefined => {
    const fresh = !existing && i === 0;
    if (fresh && mode !== 'living') return undefined;
    const empty = fresh && emptyLot;
    return householdFunds(content, savingsOf(content, r.household), homePrice(content, t, town.slots[r.slot], empty), empty);
  };
  const households = residents.map((r, i) => {
    const money = funds(r, i);
    return { name: r.household.name.trim(), plot: r.slot, player: r.player, ...(money !== undefined ? { funds: money } : {}) };
  });
  const sims: (SimSpawn & { job?: { career: string; level: number } })[] = [];
  const relationships: { a: number; b: number; preset: string }[] = [];
  residents.forEach((r, index) => {
    const slot = town.slots[r.slot];
    const template = houseTemplate(t, slot.template);
    const spawns = ((template && onLot(t, template, slot, footprint).spawns) ?? [[slotDims(t, slot).width / 2, 1.5]]).map(turnOnto(t, slot).point);
    relationships.push(...householdBonds(r.household, sims.length));
    householdSpawns(r.household, index, spawns).forEach((spawn, m) => {
      // Their jobs as drafted (`wealth.ts` `jobFor`, or picked in Creation): the simulation gives
      // them the skills for their grade (a retired one's becomes a pension). Children have none.
      const { age, job } = r.household.members[m];
      sims.push(job && content.isAdult(age) ? { ...spawn, job: { career: job.career, level: job.level } } : spawn);
    });
  });

  // Neighbours have history with each other; a newly arrived player household doesn't.
  const newcomers = existing ? 0 : player.members.length;
  for (let a = newcomers; a < sims.length; a++) {
    for (let b = a + 1; b < sims.length; b++) {
      if (sims[a].household === sims[b].household) continue;
      const roll = Math.random();
      const preset = roll < 0.03 ? 'rivals' : roll < 0.09 ? 'friends' : roll < 0.3 ? 'acquaintances' : null;
      if (preset && content.bondPresets.includes(preset)) relationships.push({ a, b, preset });
    }
  }

  const meta: TownMeta = { kind: 'town', name: town.name, roads: town.roads, greens: town.greens, paths, facing, seed: town.seed ?? (Math.random() * 2 ** 31) >>> 0, world: worldLook(town, fields) };
  // Residents leave town (for work) where its roads lead out of it.
  const exits = roadExits(town.roads, town.width, town.depth);
  return JSON.stringify({ mode, lifespan, width: town.width, depth: town.depth, storeys, ...(basement ? { basement } : {}), walls, doors, windows, objects, plots, households, sims, relationships, exits, meta });
}
