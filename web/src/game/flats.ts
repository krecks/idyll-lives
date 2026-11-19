/**
 * Flats: a building of several homes on one lot, each a plot of its own (`FlatInfo`, the town
 * file's `plots[].of`) on one storey. A walk-up of 2 or 3 storeys round a stair hall, its flats
 * generated from the house generator's `flat` class, its lot a front strip with a bay for some of
 * its flats and a yard behind. The same model makes the flats over a shop, a café or a restaurant,
 * and the same stairs the real storeys above an office or a hall.
 *
 * Three kinds of id, each seeded (the same id always gives the same building):
 * - `flats:<seed>:<storeys>:<wide>[:p]`: a building of flats (`flatsFor`, `flatsTemplate`). `wide`
 *   1: a flat each side of a stair hall three tiles wide (2 or 3 storeys: 4 or 6 flats); 0: one
 *   flat beside a hall two wide (2 or 3 flats). `p`: a front strip deep enough for parking bays.
 * - `over:<place>:<seed>:<storeys>`: a named place's layout (`houses.json` `parks`) with flats
 *   over it (`flatsOverFor`, `flatsTemplate`): a stair hall cut from one side of its ground floor,
 *   with a street door of its own, and a flat on each storey above.
 * - `floors:<place>:<seed>:<storeys>:<use>`: a named place's layout with real storeys above it
 *   furnished as its kind's (`floorsFor`, `placeFloors`: an office's offices, a hall's meeting
 *   rooms), up the stairs it gains along one side wall.
 *
 * Everything is in plot coordinates (its street at z = 0) on the ground's rows, as a house layout
 * is; a storey's things go `storey * town.depth` rows down when the town is laid out.
 */

import type { LotLook } from '../core/protocol';
import { generateFlat, placeFloor, random, residentsKey, stairHall, type FlatFrame, type FlatPlan, type FloorPlan, type Furnishing, type HouseGenConfig, type ObjectInfos } from './housegen';
import { frontTile } from './surfaces';
import { turner, type Door, type Placement, type PlannedRoom, type PlotSlot, type PublicTemplate, type Rect, type Templates, type Wall } from './town';

type Rand = () => number;

/** A thing a flat keeps though it stands outside it (`unit`: the flat's index in `FlatsTemplate.units`): its letterbox by the street door. */
export type KeptPlacement = Placement & { unit?: number };

/** One storey of a building's own (its stair hall's floor): walls, doors, windows, things and rooms as planned. */
export interface StoreyShell extends FloorPlan {
  objects: KeptPlacement[];
}

/** A flat's layout in its building (plot coordinates on the ground's rows): its front door first. */
export interface FlatLayout extends FloorPlan {
  class: string;
  description: string;
}

/**
 * A flat: on `storey`, its rectangle `rect`, `door` the hall's tile before its front door and
 * `inside` the middle of the tile behind it (where one who lives there is at home), its bedrooms
 * and its layout, furnished for who lives there.
 */
export interface FlatUnit {
  storey: number;
  rect: Rect;
  door: [number, number];
  inside: [number, number];
  bedrooms: number;
  layout: FlatLayout;
}

/**
 * A building of flats on its lot (`lot`), or the flats over a named place: its walls' box, its stair
 * hall and street door (`front`, the first of `shell[0]`'s doors; visitors arrive at `entry`), each
 * storey's stair hall (`shell`), its flats (`units`, by storey, then from the left), the bays in
 * its front strip (each a flat's: `unit`), what stands outside (`outside`: each flat's letterbox,
 * kept by it, and the yard's things), its paved ground (`paths`: from the street to the street
 * door) and how it looks from outside (`look`). Over a place, `place` is the place's layout id and
 * `ground` that layout with the hall cut from it (to stamp instead of the place's own).
 */
export interface FlatsTemplate {
  id: string;
  name: string;
  description: string;
  lot: { width: number; depth: number };
  storeys: number;
  box: Rect;
  hall: Rect;
  front: Door;
  entry: [number, number];
  shell: StoreyShell[];
  units: FlatUnit[];
  bays: { rect: Rect; unit: number }[];
  outside: KeptPlacement[];
  paths: Rect[];
  look: LotLook;
  place?: string;
  ground?: PublicTemplate;
}

/**
 * A named place with real storeys above its ground floor: its layout (`place`) as `ground`, the
 * stairs added along one side wall (and what stood there taken away), and each storey above
 * (`floors[0]` is storey 1). Plot coordinates on the ground's rows, as the place's layout.
 */
export interface PlaceFloors {
  id: string;
  place: string;
  storeys: number;
  ground: PublicTemplate;
  floors: FloorPlan[];
}

/** A flats building as a slot takes it: its template (`flats:<seed>:<storeys>:<wide>[:p]`), its lot's size and its look. */
export type FlatsSlot = Pick<PlotSlot, 'template' | 'w' | 'd' | 'dress'>;

/** Time spent generating buildings of flats and floors above places (ms), and how many. */
export const flatsStats = { ms: 0, made: 0 };

const between = (rng: Rand, [a, b]: readonly [number, number]) => a + Math.floor(rng() * (b - a + 1));
const pick = <T>(rng: Rand, list: readonly T[]): T => list[Math.floor(rng() * list.length)];
const tileKey = (x: number, z: number) => `${x}:${z}`;

/** The id of a building of flats (see the module's note). */
export function flatsId(seed: number, storeys: number, wide: boolean, bays: boolean): string {
  return `flats:${seed >>> 0}:${storeys}:${wide ? 1 : 0}${bays ? ':p' : ''}`;
}

/** The id of the flats over a named place of layout `place`. */
export function overId(place: string, seed: number, storeys: number): string {
  return `over:${place}:${seed >>> 0}:${storeys}`;
}

/** The id of a named place of layout `place` with real storeys above it furnished as a `use`'s (`housegen.json` `floors`). */
export function floorsId(place: string, seed: number, storeys: number, use: string): string {
  return `floors:${place}:${seed >>> 0}:${storeys}:${use}`;
}

/** The place layout an `over:` or `floors:` id is built on (null: neither). */
export function placeOfId(id: string): string | null {
  return id.startsWith('over:') || id.startsWith('floors:') ? id.split(':')[1] : null;
}

/** The storeys of a building of flats, the flats over a place or a place with floors above it, by its id (1: none of those). */
export function storeysOfId(id: string): number {
  const parts = id.split(':');
  if (parts[0] === 'flats') return Number(parts[2]) || 1;
  if (parts[0] === 'over' || parts[0] === 'floors') return Number(parts[3]) || 1;
  return 1;
}

/** What the flats of a building are called: `Flat 3, Mill Court`. */
export function flatName(building: string, unit: number): string {
  return `Flat ${unit + 1}, ${building}`;
}

/**
 * A building of flats for a lot in `zone` (the centre or the rest of a town), drawn from `rand`:
 * in the centre mostly three storeys with a flat each side of the hall and no bays, elsewhere
 * mostly two, some narrow, half with bays. Null if the content has no flats.
 */
export function flatsFor(t: Templates, zone: 'centre' | 'town', rand: Rand): FlatsSlot | null {
  const centre = zone === 'centre';
  const storeys = rand() < (centre ? 0.75 : 0.3) ? 3 : 2;
  const wide = rand() < (centre ? 0.85 : 0.65);
  const bays = rand() < (centre ? 0.2 : 0.5);
  const id = flatsId((rand() * 2 ** 31) >>> 0, storeys, wide, bays);
  const b = flatsTemplate(t, id);
  if (!b) return null;
  return { template: id, w: b.lot.width, d: b.lot.depth, dress: b.look };
}

/** The flats over a named place of layout `place`, `storeys` high in all (2 or 3), drawn from `rand`: its id, or null where none fit (a place too narrow, or one with a storey of its own above). */
export function flatsOverFor(t: Templates, place: string, storeys: number, rand: Rand): string | null {
  const id = overId(place, (rand() * 2 ** 31) >>> 0, storeys);
  return flatsTemplate(t, id) ? id : null;
}

/** A named place of layout `place` with real storeys above it, `storeys` high in all (2 or 3), furnished as a `use`'s, drawn from `rand`: its id, or null where none fit. */
export function floorsFor(t: Templates, place: string, use: string, storeys: number, rand: Rand): string | null {
  const id = floorsId(place, (rand() * 2 ** 31) >>> 0, storeys, use);
  return placeFloors(t, id) ? id : null;
}

const buildings = new Map<string, FlatsTemplate | null>();
const plans = new Map<string, FlatPlan | null>();
const withFloors = new Map<string, PlaceFloors | null>();

/**
 * A building of flats (`flats:`) or the flats over a place (`over:`) by its id, each flat furnished
 * for `residents[i]` (undefined: nobody lives there yet, furnished as for sale; `'shell'`: a shell);
 * the layout is the same for anyone. Undefined if the id makes none.
 */
export function flatsTemplate(t: Templates, id: string, residents: readonly (Furnishing | undefined)[] = []): FlatsTemplate | undefined {
  const at = `${id}#${residents.map(residentsKey).join('/')}`;
  if (!buildings.has(at)) {
    const t0 = performance.now();
    const parts = id.split(':');
    const made = parts[0] === 'flats' ? flatsBuilding(t, Number(parts[1]), Number(parts[2]), parts[3] === '1', parts[4] === 'p', residents) : parts[0] === 'over' ? flatsOver(t, parts[1], Number(parts[2]), Number(parts[3]), residents) : null;
    if (made) made.id = id;
    buildings.set(at, made);
    flatsStats.ms += performance.now() - t0;
    flatsStats.made++;
  }
  return buildings.get(at) ?? undefined;
}

/** A named place with real storeys above it (`floors:`) by its id; undefined if the id makes none. */
export function placeFloors(t: Templates, id: string): PlaceFloors | undefined {
  if (!withFloors.has(id)) {
    const t0 = performance.now();
    const [kind, place, seed, storeys, use] = id.split(':');
    withFloors.set(id, kind === 'floors' ? floorsAbove(t, id, place, Number(seed), Number(storeys), use ?? 'office') : null);
    flatsStats.ms += performance.now() - t0;
    flatsStats.made++;
  }
  return withFloors.get(id) ?? undefined;
}

/** A flat's plan in `frame` (cached: the same seed, frame and furnishing give the same flat). */
function flatPlan(t: Templates, frame: FlatFrame, seed: number, residents: Furnishing | undefined): FlatPlan | null {
  const at = `${seed}:${JSON.stringify(frame)}#${residentsKey(residents)}`;
  if (!plans.has(at)) plans.set(at, generateFlat(t.gen!, t.info!, frame, seed, t.lampTiles, t.windowTiles, residents));
  return plans.get(at) ?? null;
}

/** The stairs' length (tiles), and the first row past a stair hall's flights from `front`: a flat's door is on that row or behind it. */
function stairRows(cfg: HouseGenConfig, info: ObjectInfos, front: number): { len: number; doors: number } {
  const len = info(cfg.stairs)?.footprint[1] ?? 4;
  return { len, doors: front + len + 1 };
}

/**
 * A flat beside a stair hall, in its building: its rectangle `rect` (the hall on its +x side if
 * `left`, else on its -x side), from `plan` laid out in its own frame (see `FlatFrame`), turned
 * into the building; its front door's tile in the hall and the middle of the one behind it.
 */
function placeFlat(plan: FlatPlan, rect: Rect, left: boolean, footprint: (def: string) => [number, number]): { layout: FlatLayout; door: [number, number]; inside: [number, number] } {
  // Turned as a lot onto a town: its street (the hall) to the east (3) or to the west (1).
  const turn = turner(rect.x, rect.z, rect.d, rect.w, left ? 3 : 1);
  const layout: FlatLayout = {
    class: plan.class,
    description: plan.description,
    walls: plan.walls.map((w) => turn.wall(w)),
    doors: plan.doors.map((d) => turn.edge(d)),
    windows: plan.windows.map((d) => turn.edge(d)),
    objects: plan.objects.map((o) => turn.place(o, footprint)),
    rooms: plan.rooms.map((r): PlannedRoom => ({ kind: r.kind, ...turn.rect(r) })),
  };
  const front = layout.doors[0];
  const [hallX, homeX] = left ? [front.x, front.x - 1] : [front.x - 1, front.x];
  return { layout, door: [hallX, front.z], inside: [homeX + 0.5, front.z + 0.5] };
}

/** A furnishing per unit: `residents` as given, padded with undefined. */
const residentAt = (residents: readonly (Furnishing | undefined)[], i: number) => residents[i];

/**
 * The flats beside `hall` on `storeys` storeys (from `first` up), one each `sides` gives a storey:
 * the same plan on every storey (the door on the hall's floor past its flights), furnished per flat.
 * Null if a side has no plan.
 */
function flatsBeside(t: Templates, hall: Rect, sides: { x: number; w: number; left: boolean; seed: number; blind?: FlatFrame['blind'] }[], storeys: number, first: number, residents: readonly (Furnishing | undefined)[]): FlatUnit[] | null {
  const cfg = t.gen!;
  const info = t.info!;
  const footprint = (def: string): [number, number] => info(def)?.footprint ?? [1, 1];
  const { len } = stairRows(cfg, info, hall.z);
  // Its door on the hall's floor past the flights, in the flat's own frame (along the hall from the far end, or from the front).
  const frameOf = (side: (typeof sides)[number]): FlatFrame => ({ w: hall.d, d: side.w, door: side.left ? [len + 1, hall.d - 1] : [0, hall.d - len - 2], ...(side.blind ? { blind: side.blind } : {}) });
  // A side's plan: from its seed, else (a frame some seeds lay out no flat in) the next of a few.
  const seeds = sides.map((side) => [0, 1, 2, 3].map((k) => (side.seed + k * 0x9e3779b1) >>> 0).find((s) => flatPlan(t, frameOf(side), s, undefined)));
  if (seeds.some((s) => s === undefined)) return null;
  const units: FlatUnit[] = [];
  for (let k = first; k < storeys; k++) {
    for (const [i, side] of sides.entries()) {
      const rect: Rect = { x: side.x, z: hall.z, w: side.w, d: hall.d };
      const plan = flatPlan(t, frameOf(side), seeds[i]!, residentAt(residents, units.length));
      if (!plan) return null;
      const { layout, door, inside } = placeFlat(plan, rect, side.left, footprint);
      units.push({ storey: k, rect, door, inside, bedrooms: plan.bedrooms, layout });
    }
  }
  return units;
}

/**
 * Each flat's letterbox outside, by the street door at (`doorX`, `front`): in a row along the
 * front wall (from `from` to `to`), facing the street with a free tile before it to fetch the post
 * from (`free`), the nearest to the door first, never on the path to it.
 */
function letterboxes(cfg: HouseGenConfig, units: number, doorX: number, front: number, from: number, to: number, free: (x: number, z: number) => boolean): KeptPlacement[] {
  const z = front - 1;
  const spots: number[] = [];
  for (let x = from; x < to; x++) if (x !== doorX && z >= 1 && free(x, z) && free(x, z - 1)) spots.push(x);
  spots.sort((a, b) => Math.abs(a - doorX) - Math.abs(b - doorX) || a - b);
  return spots.slice(0, units).map((x, unit) => ({ def: cfg.flats!.letterbox, x, z, rot: 2, unit }));
}

/** A building of flats (see the module's note), furnished per flat for `residents`. */
function flatsBuilding(t: Templates, seed: number, storeys: number, wide: boolean, bays: boolean, residents: readonly (Furnishing | undefined)[]): FlatsTemplate | null {
  const cfg = t.gen;
  const info = t.info;
  const spec = cfg?.flats;
  const cls = cfg?.classes.flat;
  if (!cfg || !info || !spec || !cls || !(storeys >= 1 && storeys <= 3)) return null;
  const rng = random((seed ^ 0x2c1b3c6d) >>> 0);
  const depth = between(rng, cls.depth);
  const widths = wide ? [between(rng, cls.width), between(rng, cls.width)] : [between(rng, cls.width)];
  const hw = wide ? spec.hall : spec.narrowHall;
  const front = bays ? spec.frontBays : spec.front;
  const yard = between(rng, spec.yard);
  // A narrow building's hall on the left of its flat, or on the right.
  const hallFirst = !wide && rng() < 0.5;
  const bw = widths.reduce((a, b) => a + b, 0) + hw;
  const box: Rect = { x: spec.side, z: front, w: bw, d: depth };
  const hx = hallFirst ? box.x : box.x + widths[0];
  const hall: Rect = { x: hx, z: front, w: hw, d: depth };
  const lot = { width: bw + 2 * spec.side, depth: front + depth + yard };
  const sideSeed = (i: number) => (seed * 2 + i + 1) >>> 0;
  const sides = wide
    ? [
        { x: box.x, w: widths[0], left: true, seed: sideSeed(0) },
        { x: hx + hw, w: widths[1], left: false, seed: sideSeed(1) },
      ]
    : [hallFirst ? { x: hx + hw, w: widths[0], left: false, seed: sideSeed(0) } : { x: box.x, w: widths[0], left: true, seed: sideSeed(0) }];
  // The flights: from the ground against the gable (or the left wall of a wide hall), from storey 1 beside it; the doors between.
  const outer = wide || hallFirst ? hx : hx + hw - 1;
  const inner = wide ? hx + hw - 1 : hallFirst ? hx + 1 : hx;
  const doorX = wide ? hx + Math.floor(hw / 2) : inner;
  const units = flatsBeside(t, hall, sides, storeys, 0, residents);
  if (!units) return null;
  const shell = stairHall(cfg, info, { rect: hall, storeys, up: [outer, inner], door: doorX, back: true }, t.lampTiles);
  // Bays for some of the flats (at most half, as many as the front has room for beside the path).
  const bayRects: { rect: Rect; unit: number }[] = [];
  if (bays) {
    const most = Math.min(Math.floor(units.length / 2), Math.floor((lot.width - hw) / 3));
    const starts: number[] = [];
    for (let x = hx - 3; x >= 0; x -= 3) starts.push(x);
    for (let x = hx + hw; x + 3 <= lot.width; x += 3) starts.push(x);
    starts.sort((a, b) => Math.abs(a + 1.5 - (hx + hw / 2)) - Math.abs(b + 1.5 - (hx + hw / 2)) || a - b);
    for (const x of starts.slice(0, most)) bayRects.push({ rect: { x, z: 0, w: 3, d: front }, unit: bayRects.length });
  }
  const inBay = (x: number, z: number) => bayRects.some(({ rect: r }) => x >= r.x && x < r.x + r.w && z >= r.z && z < r.z + r.d);
  // (Off the bays, and off the posts of the porch over the street door, a tile either side of it: `town.ts` `frontPorchPosts`.)
  const outside: KeptPlacement[] = letterboxes(cfg, units.length, doorX, front, box.x, box.x + box.w, (x, z) => !inBay(x, z) && Math.abs(x - doorX) > 1);
  if (outside.length < units.length) return null;
  outside.push(...yardThings(cfg, info, rng, { x: box.x, z: front + depth, w: box.w, d: yard }, doorX));
  const name = `${pick(rng, cfg.names.first)} ${pick(rng, spec.names)}`;
  const look: LotLook = { use: 'flats', storeys, roof: pick(rng, ['flat', 'hip', 'mansard', 'gable'] as const), look: Math.floor(rng() * 1024) };
  const words = ['no', 'one', 'two', 'three', 'four', 'five', 'six'];
  return {
    id: '',
    name,
    description: `A ${words[storeys]}-storey building of ${words[units.length] ?? units.length} flats round a stair hall${bayRects.length ? `, with parking for ${words[bayRects.length]}` : ''}.`,
    lot,
    storeys,
    box,
    hall,
    front: shell[0].doors.find((d) => d.axis === 'x' && d.z === front)!,
    entry: [doorX + 0.5, front - 1.5],
    shell: shell.map((s, k) => (k === 0 ? { ...s, doors: frontFirst(s.doors, front) } : s)),
    units,
    bays: bayRects,
    outside,
    paths: [{ x: doorX, z: 0, w: 1, d: front }],
    look,
  };
}

/** Doors with the street door (on the street wall, row `front`) first. */
const frontFirst = (doors: Door[], front: number) => [...doors].sort((a, b) => Number(b.axis === 'x' && b.z === front) - Number(a.axis === 'x' && a.z === front));

/**
 * The yard's things behind a building (`yard`: from its back wall to the lot's back): each of
 * `yardThings` facing the building, a row clear along the back wall to reach them from, none in
 * line with the back door (`doorX`), the ones that don't fit left out.
 */
function yardThings(cfg: HouseGenConfig, info: ObjectInfos, rng: Rand, yard: Rect, doorX: number): Placement[] {
  const out: Placement[] = [];
  const taken = new Set<string>();
  for (const def of cfg.flats?.yardThings ?? []) {
    const fp = info(def)?.footprint;
    if (!fp) continue;
    const [w, d] = fp;
    const xs = Array.from({ length: Math.max(0, yard.w - w + 1) }, (_, i) => yard.x + i);
    for (let i = xs.length - 1; i > 0; i--) {
      const j = Math.floor(rng() * (i + 1));
      [xs[i], xs[j]] = [xs[j], xs[i]];
    }
    const z = yard.z + 1;
    if (z + d > yard.z + yard.d) continue;
    const x = xs.find((x) => {
      const tiles = Array.from({ length: (w + 2) * d }, (_, i) => [x - 1 + (i % (w + 2)), z + Math.floor(i / (w + 2))] as const);
      return !(doorX >= x - 1 && doorX <= x + w) && tiles.every(([tx, tz]) => !taken.has(tileKey(tx, tz)));
    });
    if (x === undefined) continue;
    for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) taken.add(tileKey(x + dx, z + dz));
    out.push({ def, x, z, rot: 2 });
  }
  return out;
}

// ---- over a place --------------------------------------------------------------------

/** A layout's walls as unit edges (`h:x:z` along x on line z, `v:x:z` along z on line x). */
function unitEdges(walls: readonly Wall[]): Set<string> {
  const out = new Set<string>();
  for (const [x0, z0, x1, z1] of walls) {
    if (z0 === z1) for (let x = Math.min(x0, x1); x < Math.max(x0, x1); x++) out.add(`h:${x}:${z0}`);
    else for (let z = Math.min(z0, z1); z < Math.max(z0, z1); z++) out.add(`v:${x0}:${z}`);
  }
  return out;
}

/** Unit edges merged into wall runs. */
function wallRuns(edges: Set<string>): Wall[] {
  const h = new Map<number, number[]>();
  const v = new Map<number, number[]>();
  for (const e of edges) {
    const [axis, xs, zs] = e.split(':');
    const [x, z] = [Number(xs), Number(zs)];
    if (axis === 'h') (h.get(z) ?? h.set(z, []).get(z)!).push(x);
    else (v.get(x) ?? v.set(x, []).get(x)!).push(z);
  }
  const walls: Wall[] = [];
  const runs = (list: number[], wall: (a: number, b: number) => Wall) => {
    list.sort((a, b) => a - b);
    let start = list[0];
    for (let i = 1; i <= list.length; i++) {
      if (i < list.length && list[i] === list[i - 1] + 1) continue;
      walls.push(wall(start, list[i - 1] + 1));
      start = list[i];
    }
  };
  for (const [z, xs] of [...h].sort((a, b) => a[0] - b[0])) runs(xs, (a, b) => [a, z, b, z]);
  for (const [x, zs] of [...v].sort((a, b) => a[0] - b[0])) runs(zs, (a, b) => [x, a, x, b]);
  return walls;
}

const edgeOf = (d: Door) => (d.axis === 'x' ? `h:${d.x}:${d.z}` : `v:${d.x}:${d.z}`);

/** The box a layout's walls enclose. */
function boxOf(walls: readonly Wall[]): Rect | null {
  if (!walls.length) return null;
  const xs = walls.flatMap(([x0, , x1]) => [x0, x1]);
  const zs = walls.flatMap(([, z0, , z1]) => [z0, z1]);
  const [x, z] = [Math.min(...xs), Math.min(...zs)];
  return { x, z, w: Math.max(...xs) - x, d: Math.max(...zs) - z };
}

/** The tiles a thing takes (its footprint turned). */
function thingTiles(o: Placement, info: ObjectInfos): [number, number][] {
  const [fw, fd] = info(o.def)?.footprint ?? [1, 1];
  const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
  const out: [number, number][] = [];
  for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) out.push([o.x + dx, o.z + dz]);
  return out;
}

const inRect = (r: Rect, x: number, z: number) => x >= r.x && z >= r.z && x < r.x + r.w && z < r.z + r.d;

/** The tiles beside (`x`, `z`) and the unit edge between it and each. */
const around = (x: number, z: number): [number, number, string][] => [
  [x + 1, z, `v:${x + 1}:${z}`],
  [x - 1, z, `v:${x}:${z}`],
  [x, z + 1, `h:${x}:${z + 1}`],
  [x, z - 1, `h:${x}:${z}`],
];

/** The floor tiles walked to from `starts` (`floor`: a tile of the floor; `open`: an edge one walks through). */
function walked(starts: [number, number][], floor: (x: number, z: number) => boolean, open: (e: string) => boolean): Set<string> {
  const seen = new Set(starts.filter(([x, z]) => floor(x, z)).map(([x, z]) => tileKey(x, z)));
  const queue = [...seen].map((k) => k.split(':').map(Number) as [number, number]);
  while (queue.length) {
    const [x, z] = queue.pop()!;
    for (const [nx, nz, e] of around(x, z)) {
      const k = tileKey(nx, nz);
      if (seen.has(k) || !floor(nx, nz) || !open(e)) continue;
      seen.add(k);
      queue.push([nx, nz]);
    }
  }
  return seen;
}

/**
 * Place layout `p` (its walls' box `box`) with `cut` taken out of its ground floor for a stair hall
 * (`cut` against one side wall, the whole depth): what stood there gone, every wall inside it
 * gone, a wall along its inner side, no door or window on its own walls (the hall gets its own);
 * `add` (stairs) put in, what stood where they go taken away; and what could be used from the
 * place's floor before and no longer can taken away too (with what stood on it). With the price of
 * what the place lost (the cheaper the better), or null if its front door is in the cut or floor
 * that was reached from it no longer is.
 */
function cutOut(p: PublicTemplate, box: Rect, cut: Rect | null, add: Placement[], info: ObjectInfos, objectPrice: (def: string) => number, clear: [number, number][] = []): { ground: PublicTemplate; lost: number } | null {
  const walls = unitEdges(p.walls ?? []);
  const doors = new Map((p.doors ?? []).map((d) => [edgeOf(d), d]));
  const windows = new Map((p.windows ?? []).map((d) => [edgeOf(d), d]));
  const front = p.doors?.[0];
  if (!front) return null;
  const starts: [number, number][] = front.axis === 'x' ? [[front.x, front.z], [front.x, front.z - 1]] : [[front.x, front.z], [front.x - 1, front.z]];
  const blocks = (o: Placement) => {
    const layer = info(o.def)?.layer;
    return layer !== 'wall' && layer !== 'ceiling' && layer !== 'rug' && layer !== 'surface';
  };
  const indoors = (o: Placement) => thingTiles(o, info).every(([x, z]) => inRect(box, x, z));
  const blockedBy = (objects: Placement[]) => new Set(objects.filter((o) => blocks(o) && indoors(o)).flatMap((o) => thingTiles(o, info).map(([x, z]) => tileKey(x, z))));
  // What could be used from the floor before: from a tile beside it, walked to from the front door.
  const usable = (o: Placement, seen: Set<string>, open: (e: string) => boolean) => {
    const tiles = thingTiles(o, info);
    const mine = new Set(tiles.map(([x, z]) => tileKey(x, z)));
    return tiles.some(([x, z]) => around(x, z).some(([nx, nz, e]) => !mine.has(tileKey(nx, nz)) && seen.has(tileKey(nx, nz)) && open(e)));
  };
  const was = blockedBy(p.objects);
  const openBefore = (e: string) => !walls.has(e) || doors.has(e);
  const before = walked(starts, (x, z) => inRect(box, x, z) && !was.has(tileKey(x, z)), openBefore);
  const usedBefore = new Set(p.objects.filter((o) => info(o.def)?.used && blocks(o) && indoors(o) && usable(o, before, openBefore)));
  if (cut) {
    const within = (e: string) => {
      const [axis, xs, zs] = e.split(':');
      const [x, z] = [Number(xs), Number(zs)];
      return axis === 'h' ? x >= cut.x && x < cut.x + cut.w && z > cut.z && z < cut.z + cut.d : z >= cut.z && z < cut.z + cut.d && x > cut.x && x < cut.x + cut.w;
    };
    const rim = (e: string) => {
      const [axis, xs, zs] = e.split(':');
      const [x, z] = [Number(xs), Number(zs)];
      return axis === 'h' ? x >= cut.x && x < cut.x + cut.w && (z === cut.z || z === cut.z + cut.d) : z >= cut.z && z < cut.z + cut.d && (x === cut.x || x === cut.x + cut.w);
    };
    if (within(edgeOf(front)) || rim(edgeOf(front))) return null;
    for (const e of [...walls]) if (within(e)) walls.delete(e);
    for (const m of [doors, windows]) for (const e of [...m.keys()]) if (within(e) || rim(e)) m.delete(e);
    const side = cut.x === box.x ? cut.x + cut.w : cut.x;
    for (let z = cut.z; z < cut.z + cut.d; z++) walls.add(`v:${side}:${z}`);
  }
  // What stood in the cut, where the stairs go or on `clear` goes, and what was used from there.
  const added = new Set([...add.flatMap((o) => thingTiles(o, info)), ...clear].map(([x, z]) => tileKey(x, z)));
  const gone = new Set<Placement>();
  const lostTile = (x: number, z: number) => (cut && inRect(cut, x, z)) || added.has(tileKey(x, z));
  for (const o of p.objects) {
    const [fw, fd] = info(o.def)?.footprint ?? [1, 1];
    const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
    if (thingTiles(o, info).some(([x, z]) => lostTile(x, z)) || (blocks(o) && lostTile(...frontTile(o.x, o.z, w, d, o.rot)))) gone.add(o);
  }
  const floor = (x: number, z: number) => inRect(box, x, z) && !(cut && inRect(cut, x, z));
  const blocked = new Set([...added, ...blockedBy(p.objects.filter((o) => !gone.has(o)))]);
  const open = (e: string) => !walls.has(e) || doors.has(e);
  const after = walked(starts, (x, z) => floor(x, z) && !blocked.has(tileKey(x, z)), open);
  // The floor reached before still is (but what the cut and the stairs took).
  for (const k of before) if (!after.has(k) && floor(...(k.split(':').map(Number) as [number, number])) && !blocked.has(k)) return null;
  for (const o of usedBefore) if (!gone.has(o) && !usable(o, after, open)) gone.add(o);
  // And the small things that stood on what went.
  const under = new Set([...gone].filter(blocks).flatMap((o) => thingTiles(o, info)).map(([x, z]) => tileKey(x, z)));
  for (const o of p.objects) if (!gone.has(o) && info(o.def)?.layer === 'surface' && under.has(tileKey(o.x, o.z))) gone.add(o);
  let lost = 0;
  for (const o of gone) lost += objectPrice(o.def) + (info(o.def)?.used ? 1000 : 0);
  return {
    ground: { ...p, walls: wallRuns(walls), doors: (p.doors ?? []).filter((d) => doors.has(edgeOf(d))), windows: (p.windows ?? []).filter((d) => windows.has(edgeOf(d))), objects: [...p.objects.filter((o) => !gone.has(o)), ...add] },
    lost,
  };
}

/** A named place's layout by id (`houses.json` `parks` of kind `place`) and its walls' box. */
function placeLayout(t: Templates, place: string): { p: PublicTemplate; box: Rect } | null {
  const p = t.parks.find((q) => q.id === place && q.kind === 'place');
  const box = p && boxOf(p.walls ?? []);
  return p && box && !p.upper ? { p, box } : null;
}

/**
 * The flats over place `place` (see the module's note): a stair hall `narrowHall` wide cut from
 * the side of its ground floor that costs it least, with a street door and a back door of its own,
 * and a flat beside the hall on every storey above, each with its letterbox in the front strip.
 */
function flatsOver(t: Templates, place: string, seed: number, storeys: number, residents: readonly (Furnishing | undefined)[]): FlatsTemplate | null {
  const cfg = t.gen;
  const info = t.info;
  const spec = cfg?.flats;
  const at = placeLayout(t, place);
  if (!cfg || !info || !spec || !at || !(storeys >= 2 && storeys <= 3)) return null;
  const { p, box } = at;
  const hw = spec.narrowHall;
  const { len } = stairRows(cfg, info, box.z);
  // Too narrow for a flat and a shop, or too shallow for the stairs and a door past them.
  if (box.w - hw < 6 || box.d < len + 3) return null;
  // A flat no wider than the class has them (narrower beside a deep hall): over a wide place, roof beside it.
  const across = Math.min(box.w - hw, box.d >= 12 ? 8 : 9);
  const price = (def: string) => (info(def) as { price?: number } | undefined)?.price ?? 0;
  const cuts = [true, false]
    .map((left) => {
      const hall: Rect = { x: left ? box.x : box.x + box.w - hw, z: box.z, w: hw, d: box.d };
      // (Nothing before its doors, nor on the path from its street door.)
      const door = left ? hall.x + hw - 1 : hall.x;
      const clear: [number, number][] = [...Array.from({ length: box.z }, (_, z): [number, number] => [door, z]), [door, box.z + box.d]];
      const cut = cutOut(p, box, hall, [], info, price, clear);
      return cut && { left, hall, ...cut };
    })
    .filter((c) => !!c);
  if (!cuts.length) return null;
  const rng = random((seed ^ 0x5be0cd19) >>> 0);
  const best = cuts.sort((a, b) => a.lost - b.lost || Number(rng() < 0.5) - 0.5)[0];
  const { hall, left } = best;
  const outer = left ? hall.x : hall.x + hw - 1;
  const inner = left ? hall.x + hw - 1 : hall.x;
  // The flat beside the hall (the hall on its left: it is right of it, `left` false).
  const side = { x: left ? hall.x + hw : hall.x - across, w: across, left: !left, seed: (seed * 2 + 1) >>> 0 };
  const units = flatsBeside(t, hall, [side], storeys, 1, residents);
  if (!units) return null;
  const shell = stairHall(cfg, info, { rect: hall, storeys, up: [outer, inner], door: inner, back: true }, t.lampTiles);
  // Letterboxes on the front strip's free tiles (not on the place's things, nor before its door or under the porch
  // over it, whose posts stand a tile either side of it: `town.ts` `frontPorchPosts`).
  const taken = new Set(best.ground.objects.flatMap((o) => thingTiles(o, info)).map(([x, z]) => tileKey(x, z)));
  const placeDoor = p.doors![0];
  const outside = letterboxes(cfg, units.length, inner, box.z, box.x, box.x + box.w, (x, z) => !taken.has(tileKey(x, z)) && Math.abs(x - placeDoor.x) > 1);
  if (outside.length < units.length) return null;
  const lot = p.lot ?? t.plot;
  const words = ['no', 'one', 'two', 'three'];
  return {
    id: '',
    name: p.name,
    description: `${p.name} with ${words[units.length] ?? units.length} flat${units.length === 1 ? '' : 's'} over it, up a stair hall of their own.`,
    lot: { width: lot.width, depth: lot.depth },
    storeys,
    box,
    hall,
    front: shell[0].doors.find((d) => d.axis === 'x' && d.z === box.z)!,
    entry: [inner + 0.5, box.z - 1.5],
    shell: shell.map((s, k) => (k === 0 ? { ...s, doors: frontFirst(s.doors, box.z) } : s)),
    units,
    bays: [],
    outside,
    paths: [{ x: inner, z: 0, w: 1, d: box.z }],
    look: { use: 'flats', storeys, roof: 'flat', look: Math.floor(rng() * 1024) },
    place,
    ground: best.ground,
  };
}

// ---- floors above a place ------------------------------------------------------------

/**
 * Place `place` with real storeys above it (see the module's note): stairs along the side wall
 * where they cost the ground floor least (climbing away from the street from a foot at the front
 * wall; a second flight beside them from storey 1 in a building of three), a hall two tiles wide
 * over them the whole depth on every storey above, and rooms off it furnished as a `use`'s.
 */
function floorsAbove(t: Templates, id: string, place: string, seed: number, storeys: number, use: string): PlaceFloors | null {
  const cfg = t.gen;
  const info = t.info;
  const at = placeLayout(t, place);
  if (!cfg || !info || !at || !(storeys >= 2 && storeys <= 3)) return null;
  const { p, box } = at;
  const { len } = stairRows(cfg, info, box.z);
  if (box.w < 7 || box.d < len + 3) return null;
  const price = (def: string) => (info(def) as { price?: number } | undefined)?.price ?? 0;
  const walls = unitEdges(p.walls ?? []);
  const options: { left: boolean; outer: number; inner: number; flight: Placement; ground: PublicTemplate; lost: number }[] = [];
  for (const left of [true, false]) {
    const outer = left ? box.x : box.x + box.w - 1;
    const inner = left ? box.x + 1 : box.x + box.w - 2;
    // The flight from row `a` (its foot before it), climbing away from the street, in one room (no wall across it or its foot).
    for (let a = box.z + 1; a + len < box.z + box.d; a++) {
      if (Array.from({ length: len }, (_, i) => a + i).some((z) => walls.has(`h:${outer}:${z}`))) continue;
      const flight: Placement = { def: cfg.stairs, x: outer, z: a, rot: 2 };
      const cut = cutOut(p, box, null, [flight], info, price);
      if (!cut) continue;
      // (Nor anything on its foot.)
      const onFoot = cut.ground.objects.some((o) => o !== flight && (info(o.def)?.layer ?? 'floor') === 'floor' && thingTiles(o, info).some(([x, z]) => x === outer && z === a - 1));
      if (!onFoot) options.push({ left, outer, inner, flight, ...cut });
    }
  }
  if (!options.length) return null;
  // The cheapest for the ground floor, nearest the front if no dearer.
  options.sort((a, b) => a.lost - b.lost || a.flight.z - b.flight.z || Number(b.left) - Number(a.left));
  for (const best of options.slice(0, 4)) {
    const hall: Rect = { x: best.left ? box.x : box.x + box.w - 2, z: box.z, w: 2, d: box.d };
    const second: Placement = { def: cfg.stairs, x: best.inner, z: best.flight.z, rot: 0 };
    const floors: FloorPlan[] = [];
    for (let k = 1; k < storeys; k++) {
      const here = k + 1 < storeys ? second : null;
      const below = k === 1 ? best.flight : second;
      const f = placeFloor(cfg, info, box, hall, here, below, use, (seed * 7 + k) >>> 0, t.lampTiles, t.windowTiles);
      if (!f) break;
      floors.push(f);
    }
    if (floors.length === storeys - 1) return { id, place, storeys, ground: best.ground, floors };
  }
  return null;
}
