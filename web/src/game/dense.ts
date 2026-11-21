/**
 * The dense pass: what makes a town as built up as its size wants, every building a real lot on a
 * road's frontage (`LotPlacer`). Along every street it lays, by the kinds its zone draws (`KINDS`:
 * the centre's shops, cafés, flats and offices shoulder to shoulder; the rest of a main town's homes, terraces, flats and
 * workshops with gaps; a hamlet's homes and workshops further apart):
 *
 * - a home on a `tight` garden (no driveway, so no car of its own): a small generated house;
 * - a terrace row of two to four lots (`rowHouse`, `LotPlacer.placeRow`);
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
import { flatsFor, flatsOverFor, floorsFor, storeysOfId } from './flats';
import { placeOfTemplate, placeUnit, slotPlace } from './placeLots';
import { between } from './noise';
import { draftAreaSize, HOURS, hoursFor, namerFor, type Namer, type ZoneKind } from './places';
import { inside, leftOf, pointAt, RoadIndex, roadLengths, roadMeets, type Pt, type RoadPlan } from './roads';
import { houseFor, lotAt, LotPlacer, plotRect, slotDims, type Frontage, type Ground, type NeighbourhoodDraft, type PlotSlot, type Rect, type Templates, type Unplaced } from './town';
import * as pmath from './pmath';

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

/**
 * What a pass over a town laid already is made from (Build, the build-up tool): its draft, the house templates its lots
 * are measured by, the content its places come from, and the ground its lots go down on (`ground`: one shared with lots
 * being laid beside it, Build's; else the draft's own, `edit/ground.ts` `townGround`).
 */
export interface DenseDraft {
  draft: NeighbourhoodDraft;
  t: Templates;
  content: Content;
  ground?: Ground;
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
  centre: [['shop', 38], ['flats', 24], ['cafe', 12], ['restaurant', 8], ['office', 14], ['terrace', 4]],
  town: [['house', 34], ['terrace', 38], ['flats', 8], ['shop', 3], ['workshop', 6], ['cafe', 1]],
  hamlet: [['house', 58], ['terrace', 16], ['workshop', 8], ['shop', 4], ['cafe', 2]],
};
/** What a named place of each use is (`ZONE`). */
const PLACE_OF: Partial<Record<BuildingUse, ZoneKind>> = { shop: 'shop', cafe: 'cafe', restaurant: 'restaurant', office: 'office', hall: 'office', workshop: 'workshop' };
/** A lot with no driveway (a home on a tight garden, a place, a row) stands with its middle this far from a junction at least (m; one with a driveway, `macro.ts` `CORNER`). */
export const NEAR_CORNER = 9;
/** A lot along a country road into town keeps this far from a junction (m): cars take the country junctions wide. */
const COUNTRY_CORNER = 40;
/** How far into a stretch a lot's middle stands from a junction where a lot with a driveway may (m, `macro.ts` `CORNER`). */
const CORNER = 18;
/** Of a centre's shops, cafés and restaurants, the share with flats over them; elsewhere (`raisePlaces`). */
const OVER_CENTRE = 1;
const OVER_TOWN = 0.6;
/** The designs of the homes on tight gardens and of the terraced houses a pass lays (`designs`). */
const DESIGNS = 16;
/** How far a lot set by hand looks for a road to front (m from the road's edge). */
const FILL_REACH = 30;
/** A centre's buildings of flats stand at least this far apart along each side of a street (m, `DensePass.centreFlats`). */
const FLATS_APART = 50;

const range = (rand: Rand, lo: number, hi: number) => lo + (hi - lo) * rand();
/** How far a point is from a box (0 inside it). */
const offBox = (r: Rect, [x, z]: Pt) => pmath.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));

/** The dense pass (see the top of this file). */
export class DensePass {
  /** What else a lot must be to go down (by hand only: Build keeps each to the land painted its use); anything if unset. */
  allow?: (rect: Rect, use: BuildingUse) => boolean;
  /** How far from a junction a lot's middle stands at least, besides its own rule (`approaches`: `COUNTRY_CORNER`). */
  private corner = 0;
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
      this.lots = new LotPlacer(content, t, input.ground ?? townGround(draft, t), draft.roads, [0, 0], () => 0.5, true);
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
    // (Every road's; the others are read at their points only, not every 2 m as `macro.ts` `cornersOf` reads them:
    // the generated towns were laid so.)
    this.meets = roadMeets(this.roads, this.roads.keys(), { step: 0 });
  }

  /** How many lots there are so far (a draft's among them). */
  get count(): number {
    return this.lots.slots.length;
  }

  /**
   * `rand`, but each draw one of `DESIGNS` numbers fixed for the pass: the homes it lays are of a few designs a
   * kind, as a street built at once is (and each design is generated once: `town.ts` `houseTemplate` keeps them).
   */
  private pooled(rand: Rand): Rand {
    this.designs ??= designs(rand);
    return pooled(this.designs, rand);
  }
  private designs?: number[];

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
    const row = (n: number) => terraceUnits(t, n, a, this.pooled(rand));
    if (use === 'house') return [{ kind: 'house', name: '', lot: 'small', area: a, ...houseFor(t, 'small', 'tight', this.pooled(rand)) }];
    if (use === 'terrace') return row(between(rand, [2, 4]));
    if (use === 'flats') {
      const f = flatsFor(t, zone === 'centre' ? 'centre' : 'town', rand);
      // (Until flats are made: a terrace row stands in, its homes as many as a small building's flats.)
      if (!f) return row(between(rand, [3, 4]));
      const { name, ...building } = f;
      return [{ kind: 'flats', name, lot: 'medium', area: a, ...building }];
    }
    const what = PLACE_OF[use];
    if (!what) return [];
    return [{ ...placeUnit(t, what, this.sizeOf(a), rand, true), area: a }];
  }

  /** Whether frontage at `s` along road `r` is `far` m from every junction along it. */
  private offJunctions(r: number, s: number, far: number): boolean {
    return (this.meets.get(r) ?? []).every((m) => Math.abs(s - m) >= far);
  }

  /** Whether the land 3 m past the verge of road `r` at `s` on `side` is taken already (a lot, a car park, the water...). */
  private takenAt(r: number, s: number, side: 1 | -1, ls: readonly number[]): boolean {
    return this.lots.ground.takenAt(...this.backAt(r, s, side, ls, 3.5));
  }

  /** The point `metres` back from the edge of road `r` at `s` on `side`. */
  private backAt(r: number, s: number, side: 1 | -1, ls: readonly number[], metres: number): Pt {
    const f = this.frontage(r, s, side, ls);
    const reach = this.roads[r].width / 2 + metres;
    return [f.p[0] + f.n[0] * reach, f.p[1] + f.n[1] * reach];
  }

  /** Frontage of road `r` at `s` on `side` (`Frontage`), its middle on the road's middle line. */
  private frontage(r: number, s: number, side: 1 | -1, ls: readonly number[]): Frontage {
    const { p, t } = pointAt(this.roads[r].points, s, ls);
    return { road: r, s, side, p, n: leftOf(t).map((v) => v * side) as Pt, far: 0 };
  }

  /**
   * Units `us` (one lot or a row) put down fronting road `r` with their middle `s` m along it on `side`, if they
   * fit there (`lotAt`; a row `LotPlacer.placeRow`), their middle off the junctions (`NEAR_CORNER`; `CORNER` for a
   * lot with a driveway) and wherever `allow` lets them: their slots' indices (none if they don't).
   */
  private put(us: readonly Unplaced[], r: number, s: number, side: 1 | -1, ls: readonly number[], use: BuildingUse, grow: number): number[] {
    const { t, lots, roads } = this;
    const f = this.frontage(r, s, side, ls);
    const drives = us.length === 1 && us[0].kind === 'house' && us[0].garden !== 'tight';
    if (!this.offJunctions(r, s, Math.max(drives ? CORNER : NEAR_CORNER, this.corner))) return [];
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
    const ls = roadLengths(roads[r]);
    const L = ls[ls.length - 1];
    const end = Math.min(L - 4, to ?? Infinity);
    const rows: Row[] = [];
    for (const side of sides) {
      let s = from === undefined ? 4 + rand() * 4 : Math.max(4, from + rand());
      // (The lots drawn and not yet placed: tried on along the road till they go down, drawn again after a few misses or where the land asks for other kinds.)
      let pending: { units: Unplaced[]; use: BuildingUse; w: number; key: string; misses: number } | null = null;
      while (s < end) {
        // (The land 6 m back from the road's edge, a metre on: what it asks for, and which zone it is.)
        const probe = (k: number) => this.backAt(r, Math.min(k, L), side, ls, 6);
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
        s += w + (z === 'centre' ? 0 : z === 'town' ? between(rand, [0, 2]) : between(rand, [1, 4]));
        // (Now and then a gap after it: the more the further out. Where the land asks for kinds, its gaps are where it asks for none.)
        const skip = z === 'centre' ? 0 : z === 'town' ? 0.12 * (1.2 - density) : 0.25;
        if (!kindsAt && rand() < skip) s += range(rand, 6, 14);
      }
    }
    return rows;
  }

  /**
   * Buildings of flats on the streets of area `area`'s centre, `most` at most, before its named places go down there:
   * each where a street has room for one in the centre (the whole lot in it), `FLATS_APART` m from the last along that
   * side at least, so they stand at the ends of its parades and its shops keep the middle. The centre's homes, most of
   * them otherwise over its shops (`raisePlaces`). Each building is drawn once and tried on along the street till it
   * goes down. How many went down.
   */
  centreFlats(rand: Rand, most: number, area = 0): number {
    const { roads, t } = this;
    if (!this.outlines.length || !this.areas[area]) return 0;
    let laid = 0;
    for (const r of this.areas[area].roads) {
      if (roads[r].kind === 'trail') continue;
      const ls = roadLengths(roads[r]);
      const L = ls[ls.length - 1];
      // (The land 6 m back from the road's edge: whether it is the centre's.)
      const inCentre = (s: number, side: 1 | -1) => this.centreAt(this.backAt(r, Math.min(s, L), side, ls, 6));
      for (const side of roads[r].lots ? [roads[r].lots] : ([1, -1] as const)) {
        let units: Unplaced[] | null = null;
        for (let s = 4, last = -Infinity; s < L - 4 && laid < most; s += 2) {
          if (s - last < FLATS_APART || !inCentre(s, side) || this.takenAt(r, s + 1, side, ls)) continue;
          units ??= this.unitsOf('flats', area, 'centre', rand);
          if (units[0]?.kind !== 'flats') return laid;
          const w = slotDims(t, units[0]).width;
          if (s + w > L - 4 || !inCentre(s + w, side) || this.takenAt(r, s + w - 1, side, ls)) continue;
          if (!this.put(units, r, s + w / 2, side, ls, 'flats', 0).length) continue;
          laid++;
          last = s;
          s += w;
          units = null;
        }
      }
    }
    return laid;
  }

  /**
   * A lot of area `area` near each of `points` (set by hand), of the kinds the land there asks for (`kindsAt`:
   * none if null), on the frontage of the road nearest it, else of the next nearest within reach (a point by a
   * corner is nearer the side street than the road its lot would front). The slots laid, by index.
   */
  fillPoints(area: number, points: readonly Pt[], rand: Rand, kindsAt: (p: Pt) => readonly BuildingUse[] | null): number[] {
    return points.flatMap((p) => {
      const kinds = kindsAt(p);
      if (kinds === null) return [];
      const near = [...this.index.around(p[0], p[1], FILL_REACH + 10)]
        .filter(([road, hit]) => this.roads[road].kind !== 'trail' && hit.dist - this.roads[road].width / 2 <= FILL_REACH)
        .sort((a, b) => a[1].dist - b[1].dist);
      if (!near.length) return [];
      const zone = this.zoneAt(area, p);
      const use = this.pickUse(zone, rand, kinds);
      const units = use ? this.unitsOf(use, area, zone, rand) : [];
      if (!use || !units.length) return [];
      for (const [road, hit] of near) {
        const ls = roadLengths(this.roads[road]);
        const { p: q, t } = pointAt(this.roads[road].points, hit.s, ls);
        // (The side the point is on: `leftOf` the road's way is side 1.)
        const [lx, lz] = leftOf(t);
        const side: 1 | -1 = (p[0] - q[0]) * lx + (p[1] - q[1]) * lz >= 0 ? 1 : -1;
        const laid = this.put(units, road, hit.s, side, ls, use, 0);
        if (laid.length) return laid;
      }
      return [];
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
      const ls = roadLengths(this.roads[r]);
      // (Each road's area: the one nearest its middle; `only` keeps it to the land near that one.)
      const a = near(pointAt(this.roads[r].points, ls[ls.length - 1] / 2, ls).p);
      const ends = [near(this.roads[r].points[0]), near(this.roads[r].points[this.roads[r].points.length - 1])];
      for (const k of new Set([a, ...ends])) {
        this.within = (_, p) => near(p) === k && only(k, p);
        this.corner = COUNTRY_CORNER;
        this.rowsAlong(k, r, [1, -1], rand, k === 0 ? 'town' : 'hamlet');
      }
    }
    this.within = undefined;
    this.corner = 0;
  }

  /** Every lot (a draft's first, then the new) and the places of the public ones (a draft's as they were, then the new lots'). */
  result(): { slots: PlotSlot[]; places: TownPlace[] } {
    const slots = [...this.lots.slots];
    const draft = { name: this.areas[0]?.name ?? '', open: { areas: this.areas } };
    const added = slots.slice(this.before).filter((s) => s.kind === 'shop').map((s) => slotPlace(draft, this.t, this.content, s));
    return { slots, places: [...this.placesBefore, ...added] };
  }
}

/** `DESIGNS` numbers drawn from `rand`: the designs a street's or a terrace's houses are of (`pooled`). */
export function designs(rand: Rand): number[] {
  return Array.from({ length: DESIGNS }, () => rand());
}

/**
 * `rand`, but each draw one of `pool` (`designs`): the homes laid with it are of a few designs, as a street built at
 * once is (and each design is generated once: `town.ts` `houseTemplate` keeps them).
 */
export function pooled(pool: readonly number[], rand: Rand): Rand {
  return () => pool[Math.floor(rand() * pool.length)];
}

/** A row of `n` terraced houses of area `area` (`rowHouse`, drawn from `rand`), before they are placed; fewer where none is made. */
export function terraceUnits(t: Templates, n: number, area: number, rand: Rand): Unplaced[] {
  return Array.from({ length: n }, () => rowHouse(t, rand))
    .filter((h) => !!h)
    .map((h): Unplaced => ({ kind: 'house', name: '', lot: 'terrace', area, ...h }));
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
 * What stands over the named places of `slots` (`PlotSlot.upper`, `flats.ts`), drawn from `rand`: flats over most of
 * a centre's shops, cafés and restaurants (`OVER_CENTRE`; `OVER_TOWN` elsewhere), two storeys of them in a centre
 * mostly, one elsewhere; an office's or a hall's own real floors over it (one or two storeys). Each look as tall as its
 * real storeys (`storeysOfId`); a place nothing fits over keeps its own. `centres`: the centres' outlines.
 */
export function raisePlaces(t: Templates, slots: readonly PlotSlot[], centres: readonly (readonly Pt[])[], rand: Rand): void {
  for (const s of slots) {
    const what = s.kind === 'shop' && s.dress && !s.upper ? placeOfTemplate(s.template) : null;
    if (!what || !s.dress) continue;
    const use = s.dress.use;
    const r = plotRect(t, s);
    const inCentre = centres.some((poly) => inside(poly, r.x + r.w / 2, r.z + r.d / 2));
    const storeys = inCentre && rand() < 0.7 ? 3 : 2;
    let id: string | null = null;
    if (use === 'shop' || use === 'cafe' || use === 'restaurant') {
      if (rand() < (inCentre ? OVER_CENTRE : OVER_TOWN)) id = flatsOverFor(t, s.template, storeys, rand);
    } else if (use === 'office' || use === 'hall') id = floorsFor(t, s.template, use, storeys, rand);
    if (!id) continue;
    s.upper = id;
    s.dress = { ...s.dress, storeys: storeysOfId(id) };
  }
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
