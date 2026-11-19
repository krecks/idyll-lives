/**
 * The dense pass: what makes a town as built up as its size wants, every building a real lot on a
 * road's frontage (`LotPlacer`). It took the place of the backdrop, whose buildings nobody lived or
 * worked in. Along every street it lays, by the kinds its zone draws (`KINDS`: the centre's shops,
 * cafés, flats and offices shoulder to shoulder; the rest of a main town's homes, terraces, flats and
 * workshops with gaps; a hamlet's homes and workshops further apart):
 *
 * - a home on a `tight` garden (no driveway, so no car of its own): a small generated house;
 * - a terrace row of two to four lots (`macro.ts` `rowHouse`, `LotPlacer.placeRow`);
 * - a flats building (`flats.ts` `flatsFor`; a terrace row stands in where flats are not made yet);
 * - a named place from the compact layouts (`places.ts` `ZONE`'s `compact`: a shop or a café 8 m
 *   wide, a restaurant, an office or a workshop of 12 to 14).
 *
 * The generator runs it after the homes, the named places, the parking and the orchards, from the
 * `dense` stream (`macro.ts`); Build and the build-up tool make one from a draft (`DenseDraft`):
 * every lot standing is kept, the new ones are added after them.
 */

import type { Content } from '../content/content';
import type { BuildingUse, TownPlace, TownSize } from '../core/protocol';
import type { Centre } from './centre';
import { townGround } from './edit/ground';
import { flatsFor, upperFor } from './flats';
import { placeUnit, slotPlace } from './placeLots';
import { draftAreaSize, HOURS, hoursFor, namerFor, type Namer, type ZoneKind } from './places';
import { inside, leftOf, lengths, pointAt, RoadIndex, type Pt, type RoadPlan } from './roads';
import { houseFor, lotAt, LotPlacer, plotRect, slotDims, type Frontage, type NeighbourhoodDraft, type PlotSlot, type Rect, type Templates, type Unplaced } from './town';

type Rand = () => number;

/** Where a lot stands, which sets its kinds, gaps and the corner it keeps: a main town's centre, the rest of a main town, a hamlet. */
export type Zone = 'centre' | 'town' | 'hamlet';

/** What a pass over a town being generated is made from. */
export interface DenseInput {
  content: Content;
  t: Templates;
  roads: readonly RoadPlan[];
  /** The areas (0: the main town): name, box, middle, roads (indices into `roads`). */
  areas: readonly { name: string; rect: Rect; centre: Pt; roads: readonly number[] }[];
  /** The lots so far, and the ground they take: the pass adds its own to them. */
  lots: LotPlacer;
  centres: readonly Centre[];
  /** How much of a main town's land is built over, 0..1 (a dense town's most). */
  density: number;
  /** How big a town the main town is (its places' names: `Namer.size`). */
  civic: TownSize;
}

/** What a pass over a town laid already is made from (Build, the build-up tool): its draft, the house templates its lots are measured by, the content its places come from. */
export interface DenseDraft {
  draft: NeighbourhoodDraft;
  t: Templates;
  content: Content;
}

/**
 * Lots of a row along a road (`DensePass.rowsAlong`): their slots' indices, the road, how far along it
 * the row's middle is and on which side, how far back from the road its back is, its zone, and the
 * kinds the row was limited to (if it was).
 */
export interface Row {
  slots: number[];
  r: number;
  s: number;
  side: 1 | -1;
  back: number;
  zone: Zone;
  kinds?: readonly BuildingUse[];
}

/**
 * Where along its road a row goes (m, `from` to `to`; the whole road if not given) and what its lots may be
 * (`kinds`: any of its zone's if not given; `kindsAt`: those the land a metre along and 6 m back from the
 * sidewalk asks for, none there if null, in place of `kinds`, with no gaps but where it asks for none).
 */
export interface RowLimits {
  from?: number;
  to?: number;
  kinds?: readonly BuildingUse[];
  kindsAt?: (p: Pt) => readonly BuildingUse[] | null;
}

/**
 * The kinds by where they stand, and how often each is drawn: the centre's shops, cafés, flats and offices shoulder to
 * shoulder; the rest of a main town's homes, with a corner shop or a workshop now and then (the centre stays the
 * densest: `centre.ts` `DENSEST`); a hamlet's (a hall is an office's lot).
 */
const KINDS: Record<Zone, readonly (readonly [BuildingUse, number])[]> = {
  centre: [['shop', 40], ['flats', 16], ['cafe', 12], ['restaurant', 8], ['office', 16], ['terrace', 8]],
  town: [['house', 34], ['terrace', 30], ['flats', 16], ['shop', 3], ['workshop', 6], ['cafe', 1]],
  hamlet: [['house', 58], ['terrace', 16], ['workshop', 8], ['shop', 4], ['cafe', 2]],
};
/** What a named place of each use is (`ZONE`). */
const PLACE_OF: Partial<Record<BuildingUse, ZoneKind>> = { shop: 'shop', cafe: 'cafe', restaurant: 'restaurant', office: 'office', hall: 'office', workshop: 'workshop' };
/** A lot with no driveway (a home on a tight garden, a place, a row) stands with its middle this far from a junction at least (m; one with a driveway, `macro.ts` `CORNER`). */
export const NEAR_CORNER = 9;
/** How far into a stretch a lot's middle stands from a junction where a lot with a driveway may (m, `macro.ts` `CORNER`). */
const CORNER = 18;
/** Of a centre's shops, cafés and restaurants, the share with flats over them; elsewhere. */
const OVER_CENTRE = 0.8;
const OVER_TOWN = 0.3;
/** How far a lot set by hand looks for a road to front (m from the road's edge). */
const FILL_REACH = 30;

const range = (rand: Rand, lo: number, hi: number) => lo + (hi - lo) * rand();
/** How far a point is from a box (0 inside it). */
const offBox = (r: Rect, [x, z]: Pt) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
const between = (rand: Rand, lo: number, hi: number) => lo + Math.floor(rand() * (hi - lo + 1));

/** The dense pass (see the top of this file). */
export class DensePass {
  /** What else a lot must be to go down (by hand only: Build keeps each to the land painted its use); anything if unset. */
  allow?: (rect: Rect, use: BuildingUse) => boolean;
  /** Where rows go (the land 6 m back from a road's edge, in area `a`): anywhere if unset (`lay`'s `only`). */
  private within?: (a: number, p: Pt) => boolean;
  private readonly content: Content;
  private readonly t: Templates;
  private readonly roads: readonly RoadPlan[];
  private readonly areas: DenseInput['areas'];
  private readonly lots: LotPlacer;
  /** The centres' outlines, for which zone a lot is in. */
  private readonly outlines: readonly Pt[][];
  private readonly density: number;
  private readonly sizeOf: (a: number) => TownSize;
  private readonly index: RoadIndex;
  /** Where each road is met by another (m along it): a lot keeps its middle off them (`NEAR_CORNER`, `CORNER`). */
  private readonly meets: Map<number, number[]>;
  /** The lots there were before the pass (a draft's), and the places they had. */
  private readonly before: number;
  private readonly placesBefore: readonly TownPlace[];

  constructor(input: DenseInput | DenseDraft) {
    if ('draft' in input) {
      const { draft, t, content } = input;
      const open = draft.open;
      this.content = content;
      this.t = t;
      this.roads = draft.roads;
      this.areas = (open?.areas ?? []).map((a, i) => ({
        name: a.name,
        rect: { x: a.x, z: a.z, w: a.w, d: a.d },
        centre: [a.x + a.w / 2, a.z + a.d / 2] as Pt,
        roads: draft.roads.flatMap((r, k) => (r.area === i && r.kind !== 'trail' ? [k] : [])),
      }));
      // (Its lots standing taken, the new ones put down after them with no gap: Build lays them as the land is painted.)
      this.lots = new LotPlacer(content, t, townGround(draft, t), draft.roads, [0, 0], () => 0.5, true);
      this.lots.slots.push(...draft.slots.map((s) => ({ ...s })));
      this.outlines = (open?.centre ?? []).map((c) => c.poly);
      this.density = 0.6;
      this.sizeOf = (a) => draftAreaSize(draft, a);
      this.placesBefore = open?.places ?? [];
    } else {
      this.content = input.content;
      this.t = input.t;
      this.roads = input.roads;
      this.areas = input.areas;
      this.lots = input.lots;
      this.outlines = input.centres.map((c) => c.poly);
      this.density = input.density;
      const size = input.civic;
      this.sizeOf = (a) => (a === 0 ? size : size === 'hamlet' ? 'hamlet' : 'village');
      this.placesBefore = [];
    }
    this.before = this.lots.slots.length;
    this.index = new RoadIndex(this.roads);
    this.meets = junctions(this.roads);
  }

  /** How many lots there are so far (a draft's among them). */
  get count(): number {
    return this.lots.slots.length;
  }

  /** Whether a point is in a centre. */
  private centreAt(p: Pt): boolean {
    return this.outlines.some((poly) => inside(poly, p[0], p[1]));
  }

  /** The zone of a lot at `p` in area `a`. */
  private zoneAt(a: number, p: Pt): Zone {
    return this.centreAt(p) ? 'centre' : a === 0 ? 'town' : 'hamlet';
  }

  /** A use of `zone`'s drawn by its weight (`kinds`: of those only; null if the zone has none of them). */
  private pickUse(zone: Zone, rand: Rand, kinds?: readonly BuildingUse[]): BuildingUse | null {
    const list = kinds ? KINDS[zone].filter(([k]) => kinds.includes(k) || (k === 'office' && kinds.includes('hall'))) : KINDS[zone];
    if (!list.length) return null;
    let r = rand() * list.reduce((n, [, w]) => n + w, 0);
    for (const [k, w] of list) if ((r -= w) <= 0) return k;
    return list[0][0];
  }

  /**
   * What a lot of `use` in area `a` and `zone` is before it is placed: one lot, or a row of them (a terrace, and
   * flats' stand-in where none is made: `flats.ts` `flatsFor`). Drawn from `rand`.
   */
  private unitsOf(use: BuildingUse, a: number, zone: Zone, rand: Rand): Unplaced[] {
    const { t } = this;
    const row = (n: number) =>
      Array.from({ length: n }, () => rowHouse(t, rand))
        .filter((h) => !!h)
        .map((h): Unplaced => ({ kind: 'house', name: '', lot: 'terrace', area: a, ...h }));
    if (use === 'house') return [{ kind: 'house', name: '', lot: 'small', area: a, ...houseFor(t, 'small', 'tight', rand) }];
    if (use === 'terrace') return row(between(rand, 2, 4));
    if (use === 'flats') {
      const f = flatsFor(t, zone === 'centre' ? 'centre' : 'town', rand);
      // (Until flats are made: a terrace row stands in, its homes as many as a small building's flats.)
      return f ? [{ kind: 'flats', name: '', lot: 'medium', area: a, ...f }] : row(between(rand, 3, 4));
    }
    const what = PLACE_OF[use];
    if (!what) return [];
    const u: Unplaced = { ...placeUnit(t, what, this.sizeOf(a), rand, true), area: a };
    // Flats over most of a centre's shops, cafés and restaurants and some elsewhere; an office's own floors over it.
    const over = what === 'office' ? 'office' : what === 'shop' || what === 'cafe' || what === 'restaurant' ? 'flats' : null;
    if (over && rand() < (zone === 'centre' ? OVER_CENTRE : over === 'office' ? 0 : OVER_TOWN)) {
      const storeys = zone === 'centre' ? between(rand, 1, 2) : 1;
      const id = upperFor(t, u, over, storeys, rand);
      if (id && u.dress) return [{ ...u, upper: id, dress: { ...u.dress, storeys: 1 + storeys } }];
    }
    return [u];
  }

  /** Whether frontage at `s` along road `r` is `far` m from every junction along it. */
  private offJunctions(r: number, s: number, far: number): boolean {
    return (this.meets.get(r) ?? []).every((m) => Math.abs(s - m) >= far);
  }

  /** Whether the land 3 m past the verge of road `r` at `s` on `side` is taken already (a lot, a car park, the water...). */
  private takenAt(r: number, s: number, side: 1 | -1, ls: number[]): boolean {
    const f = this.frontage(r, s, side, ls);
    const reach = this.roads[r].width / 2 + 3.5;
    return this.lots.ground.takenAt(f.p[0] + f.n[0] * reach, f.p[1] + f.n[1] * reach);
  }

  /** Frontage of road `r` at `s` on `side` (`Frontage`), its middle on the road's middle line. */
  private frontage(r: number, s: number, side: 1 | -1, ls: number[]): Frontage {
    const { p, t } = pointAt(this.roads[r].points, s, ls);
    return { road: r, s, side, p, n: leftOf(t).map((v) => v * side) as Pt, far: 0 };
  }

  /**
   * Units `us` (one lot or a row) put down fronting road `r` with their middle `s` m along it on `side`, if they
   * fit there (`lotAt`; a row `LotPlacer.placeRow`), their middle off the junctions (`NEAR_CORNER`; `CORNER` for a
   * lot with a driveway) and wherever `allow` lets them: their slots' indices (none if they don't).
   */
  private put(us: readonly Unplaced[], r: number, s: number, side: 1 | -1, ls: number[], use: BuildingUse, grow: number): number[] {
    const { t, lots, roads } = this;
    const f = this.frontage(r, s, side, ls);
    const drives = us.length === 1 && us[0].kind === 'house' && us[0].garden !== 'tight';
    if (!this.offJunctions(r, s, drives ? CORNER : NEAR_CORNER)) return [];
    if (us.length > 1) {
      const allow = this.allow;
      if (allow) {
        const w = us.reduce((n, u) => n + slotDims(t, u).width, 0);
        const at = lotAt(lots.ground, roads[r], f, w, Math.max(...us.map((u) => slotDims(t, u).depth)), true);
        if (!at || !allow(plotRect(t, { lot: 'terrace', w, d: Math.max(...us.map((u) => slotDims(t, u).depth)), ...at }), use)) return [];
      }
      return lots.placeRow(us, [f], us[0].area).map((q) => q.index);
    }
    const u = us[0];
    const { width, depth } = slotDims(t, u);
    const at = lotAt(lots.ground, roads[r], f, width, depth, drives);
    if (!at || (drives && !lots.drivesOut(u, at))) return [];
    if (this.allow && !this.allow(plotRect(t, { ...u, ...at }), use)) return [];
    return [lots.put(u, at, f, grow).index];
  }

  /**
   * A row of lots along road `road` (of area `area`) on each of `sides`: the centre's shoulder to
   * shoulder, the rest with gaps, each of its zone's kinds (`zone`: that zone all along; else the
   * centre's where it is in one, the main town's in area 0, a hamlet's elsewhere). `limits`: see
   * `RowLimits`. The rows as laid.
   */
  rowsAlong(area: number, road: number, sides: readonly (1 | -1)[], rand: Rand, zone?: Zone, limits: RowLimits = {}): Row[] {
    const { roads, density } = this;
    const { from, to, kindsAt } = limits;
    const r = road;
    const ls = lengths(roads[r].points);
    const L = ls[ls.length - 1];
    const end = Math.min(L - 4, to ?? Infinity);
    const rows: Row[] = [];
    for (const side of sides) {
      let s = from === undefined ? 4 + rand() * 4 : Math.max(4, from + rand());
      // (The lots drawn and not yet placed: tried on along the road till they go down, drawn again after a few misses or where the land asks for other kinds.)
      let pending: { units: Unplaced[]; use: BuildingUse; w: number; key: string; misses: number } | null = null;
      while (s < end) {
        // (The land 6 m back from the road's edge, a metre on: what it asks for, and which zone it is.)
        const probe = (k: number) => {
          const f = this.frontage(r, Math.min(k, L), side, ls);
          const reach = roads[r].width / 2 + 6;
          return [f.p[0] + f.n[0] * reach, f.p[1] + f.n[1] * reach] as Pt;
        };
        if (this.within && !this.within(area, probe(s))) {
          s += 4;
          continue;
        }
        const kinds = kindsAt ? kindsAt(probe(s + 1)) : limits.kinds;
        if (kinds === null) {
          s += 2;
          continue;
        }
        // (Quickly past frontage a lot already takes.)
        if (this.takenAt(r, s + 1, side, ls)) {
          s += 2;
          continue;
        }
        const z: Zone = zone ?? this.zoneAt(area, probe(s));
        const key = `${z}|${kinds?.join(',') ?? ''}`;
        if (!pending || pending.key !== key || pending.misses > 4) {
          const use = this.pickUse(z, rand, kinds);
          const units = use ? this.unitsOf(use, area, z, rand) : [];
          if (!use || !units.length) {
            s += 3;
            pending = null;
            continue;
          }
          pending = { units, use, w: units.reduce((n, u) => n + slotDims(this.t, u).width, 0), key, misses: 0 };
        }
        const { units, use, w } = pending;
        if (s + w > end || this.takenAt(r, s + w - 1, side, ls)) {
          s += 2;
          pending.misses++;
          continue;
        }
        const laid = this.put(units, r, s + w / 2, side, ls, use, z === 'centre' ? 0 : 1);
        if (!laid.length) {
          s += 2;
          pending.misses++;
          continue;
        }
        pending = null;
        const depth = Math.max(...units.map((u) => slotDims(this.t, u).depth));
        rows.push({ slots: laid, r, s: s + w / 2, side, back: depth, zone: z, ...(kinds ? { kinds } : {}) });
        s += w + (z === 'centre' ? 0 : z === 'town' ? between(rand, 0, 2) : between(rand, 2, 6));
        // (Now and then a gap after it: the more the further out. Where the land asks for kinds, its gaps are where it asks for none.)
        const skip = z === 'centre' ? 0 : z === 'town' ? 0.16 * (1.2 - density) : 0.45;
        if (!kindsAt && rand() < skip) s += range(rand, 6, 14);
      }
    }
    return rows;
  }

  /**
   * A lot of area `area` near each of `points` (set by hand), on the frontage of the road nearest it,
   * of the kinds the land there asks for (`kindsAt`: none if null). The slots laid, by index.
   */
  fillPoints(area: number, points: readonly Pt[], rand: Rand, kindsAt: (p: Pt) => readonly BuildingUse[] | null): number[] {
    return points.flatMap((p) => {
      const kinds = kindsAt(p);
      if (kinds === null) return [];
      let best: { road: number; dist: number; s: number } | null = null;
      for (const [road, hit] of this.index.around(p[0], p[1], FILL_REACH + 10)) {
        if (this.roads[road].kind === 'trail' || (best && best.dist <= hit.dist)) continue;
        best = { road, dist: hit.dist, s: hit.s };
      }
      if (!best || best.dist - this.roads[best.road].width / 2 > FILL_REACH) return [];
      const ls = lengths(this.roads[best.road].points);
      const { p: q, t } = pointAt(this.roads[best.road].points, best.s, ls);
      const side: 1 | -1 = (p[0] - q[0]) * -t[1] + (p[1] - q[1]) * t[0] >= 0 ? 1 : -1;
      const zone = this.zoneAt(area, p);
      const use = this.pickUse(zone, rand, kinds);
      const units = use ? this.unitsOf(use, area, zone, rand) : [];
      return use && units.length ? this.put(units, best.road, best.s, side, ls, use, 0) : [];
    });
  }

  /**
   * The generator's pass: rows along every road of every area (`areas`: of those only), both sides, each zone's
   * kinds; `only`: where the land 6 m back from the road's edge is in it.
   */
  lay(rand: Rand, areas?: readonly number[], only?: (a: number, p: Pt) => boolean): void {
    this.within = only;
    this.areas.forEach((area, a) => {
      if (areas && !areas.includes(a)) return;
      for (const r of area.roads) if (this.roads[r].kind !== 'trail') this.rowsAlong(a, r, this.roads[r].lots ? [this.roads[r].lots] : [1, -1], rand);
    });
    this.within = undefined;
  }

  /**
   * Rows along the country roads (`roads`) where they come into an area: each stretch whose land 6 m back from the
   * road's edge `only` lets be (near enough the area's box, gentle, the road straight there), that area's, as its
   * outskirts are built along a road into town.
   */
  approaches(rand: Rand, roads: readonly number[], only: (a: number, p: Pt) => boolean): void {
    const { areas } = this;
    const near = (p: Pt) => areas.reduce((k, q, i) => (offBox(q.rect, p) < offBox(areas[k].rect, p) ? i : k), 0);
    for (const r of roads) {
      const ls = lengths(this.roads[r].points);
      // (Each road's area: the one nearest its middle; `only` keeps it to the land near that one.)
      const a = near(pointAt(this.roads[r].points, ls[ls.length - 1] / 2, ls).p);
      const ends = [near(this.roads[r].points[0]), near(this.roads[r].points[this.roads[r].points.length - 1])];
      for (const k of new Set([a, ...ends])) {
        this.within = (_, p) => near(p) === k && only(k, p);
        this.rowsAlong(k, r, [1, -1], rand, k === 0 ? 'town' : 'hamlet');
      }
    }
    this.within = undefined;
  }

  /** Every lot (a draft's first, then the new) and the places of the public ones (a draft's as they were, then the new lots'). */
  result(): { slots: PlotSlot[]; places: TownPlace[] } {
    const slots = [...this.lots.slots];
    const draft = { name: this.areas[0]?.name ?? '', open: { areas: this.areas } };
    const added = slots.slice(this.before).filter((s) => s.kind === 'shop').map((s) => slotPlace(draft, this.t, this.content, s));
    return { slots, places: [...this.placesBefore, ...added] };
  }
}

/** Where each road is met by another, m along it: its ends that join one, and where another runs by it close enough to touch. */
function junctions(roads: readonly RoadPlan[]): Map<number, number[]> {
  const meets = new Map<number, number[]>();
  const index = new RoadIndex(roads);
  roads.forEach((r, i) => {
    const ls = lengths(r.points);
    meets.set(i, r.ends.flatMap((e, k) => (e === 'join' ? [k ? ls[ls.length - 1] : 0] : [])));
  });
  roads.forEach((o, j) => {
    for (let k = 0; k < o.points.length; k++) {
      const [x, z] = o.points[k];
      for (const [r, hit] of index.around(x, z, 10)) if (r !== j && hit.dist <= (o.width + roads[r].width) / 2 + 2) meets.get(r)!.push(hit.s);
    }
  });
  return meets;
}

/**
 * A terraced house and its lot (`houseFor` on a terrace lot): one generated (a draw or two in a
 * hundred lays none out; `houseFor` would put a fixed layout, too wide for the lot, in its place).
 * Null: none in 40 draws.
 */
export function rowHouse(t: Templates, rand: Rand): ReturnType<typeof houseFor> | null {
  for (let k = 0; k < 40; k++) {
    const h = houseFor(t, 'terrace', 'snug', rand);
    if (h.template.startsWith('gen:')) return h;
  }
  return null;
}

/**
 * The places of a town's public lots (`placeLots.ts` `slotPlace`): a named place's (a civic place's, a shop's, an
 * office's...) as its lot is called, another's (the sim's own shops, the park) as its lot is unless a place has
 * that name already (then with its area's name before it, as a hamlet's café is called by its hamlet's). Then every
 * career not working in a main town yet goes to its first office. `areas`: the town's (name, roads); `centres`:
 * the main towns' areas; `sizeOf`: how big a place each area is for its names.
 */
export function townPlaces(content: Content, t: Templates, roads: readonly RoadPlan[], areas: readonly { name: string; roads: readonly number[] }[], slots: readonly PlotSlot[], centres: readonly number[], sizeOf: (a: number) => TownSize, rand: Rand): TownPlace[] {
  const categories = content.careerCategories;
  const lastNames = content.names.last.length ? content.names.last : ['Smith'];
  const draft = { name: areas[0]?.name ?? '', open: { areas } };
  const taken = new Set<string>(slots.flatMap((s) => (s.kind === 'shop' && s.dress ? [s.name] : [])));
  const namers = new Map<number, Namer>();
  const namer = (a: number) => {
    const streets = areas[a].roads.map((r) => roads[r].name).filter((n): n is string => !!n && !/Trail|Path/.test(n));
    return namers.get(a) ?? namers.set(a, namerFor(rand, areas[a].name, sizeOf(a), lastNames, streets, taken)).get(a)!;
  };
  const places: TownPlace[] = [];
  for (const s of slots) {
    if (s.kind === 'house' || s.kind === 'flats') continue;
    places.push(s.dress ? slotPlace(draft, t, content, s) : slotPlace(draft, t, content, s, namer(s.area ?? 0)));
  }
  // Every career a resident may hold works somewhere in each main town: any its places leave out
  // (content added since) go to its first office.
  for (const a of centres) {
    const here = places.filter((p) => p.area === a);
    const office = here.find((p) => p.kind === 'office');
    if (!office) continue;
    for (const c of categories) {
      if (here.some((p) => p.work?.includes(c.id))) continue;
      office.work = [...(office.work ?? []), c.id];
      office.hours = hoursFor(office.hours ?? HOURS.office, [c.id], categories);
    }
  }
  return places;
}
