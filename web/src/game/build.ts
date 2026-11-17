/**
 * Step 3's Build (the `build` job): the town made from step 2's layout, its streets and the uses the
 * player painted (`uses.ts`), on the land.
 *
 * - A layout untouched (its `layoutHash` the proposal's) with the proposal's own seed (`PROPOSAL_SEED`)
 *   builds the generator's town as it is (`Proposal.built`): today's town and households.
 * - Else the town is built again where the layout changed: what stands on the changed cells (and 20 m
 *   round them) or fronts a road that is gone is cleared, and the frontage there is filled again road by
 *   road, side by side, stretch by stretch (`proposal.ts` `stretchesOf`: the use most of the land 6 to
 *   18 m behind the kerb is painted), each stretch from a stream of its own (the seed, the road, the use,
 *   the side and where the stretch begins), so a stretch the player didn't touch is filled the same way
 *   whatever else changed. With another seed ("Build again"), everything zoned is built again from it.
 *
 * What goes along a stretch, by its use:
 * - homes: houses of the place's lot mix at its gaps (`LotPlacer`, as `fillStreet` fills a street);
 * - terraces and flats: rows of terraced houses on about half of it (`terraceRows`), then the backdrop's
 *   houses, terraces and flats in what they leave (`BackdropPass.rowsAlong`), a row behind (`secondRow`);
 * - shops and work: the backdrop's shops, cafés, restaurants, offices and workshops (the centre's inside
 *   the town centre), a row behind, and a shop lot for an area that has none;
 * - parks and green: a park, then a playground; a green with no frontage near a town or village becomes
 *   an outdoor spot, a trail to it from the area's streets (`trailTo`);
 * - public places: the civic places the area lacks, the most needed first (`civicChoices`, its size as
 *   its homes now make it), each nearest the area's middle (`BackdropPass.putCivic`).
 * A lot or a building goes only where `SHARE` of the cells under it are painted its use and none is a
 * green; unpainted frontage stays empty, and so does land zoned more than `BUILD_REACH` m from a road.
 * The areas are those the layout has, a street drawn by hand given to the nearest within 30 m (its box
 * grown to it) or a village of its own (`areasFor`). Each changed area's parking is laid again, the
 * houses numbered along their streets, and the places named. The land is settled under it all, giving up
 * only lots and backdrop buildings (`settleLand`'s `keepRoads`: the streets were judged in step 2).
 */

import type { Content } from '../content/content';
import type { BackdropUse, RoadPlan, TownArea } from '../core/protocol';
import { BackdropPass, civicChoices, CIVIC, facesRoad } from './backdrop';
import { roadKey } from './edit/buildup';
import { frontOf, inCentre, townGround } from './edit/ground';
import { removeLots, renumber, withLotPlace } from './edit/lots';
import { roadReach, usesOfDraft } from './edit/uses';
import { encodeLand, heightAt } from './land';
import { openUnderBuilt, settleLand } from './landFollow';
import { unrle } from './landMasks';
import { AREA_APART, cells8, cornersOf, grown, hamletOf, parkingFor, terraceRows, trailTo, type AreaPlan } from './macro';
import { mulberry32 } from './noise';
import { BUILD_REACH, fronted, greenSpots, stretchesOf, type Stretch } from './proposal';
import { inside, lengths, leftOf, pointAt, RoadIndex, type Pt } from './roads';
import { boundsOf, frontageOf, generatedHouses, Ground, lotAt, LotPlacer, plotRect, slotDims, wantedLots, type Facing, type Frontage, type NeighbourhoodDraft, type PlotSlot, type Rect, type Templates, type Unplaced } from './town';
import { previewWorld } from './townPreview';
import type { LaidTown, Proposal } from './townJobs';
import { layoutHash, unpackUses, USE_CELL, USE_CODE, type UseKind } from './uses';
import { placeOf, specOf, type WorldSpec } from './world';

type Rand = () => number;

/** The build seed of the proposal's own town: an untouched layout built with it is the generator's town. "Build again" draws another. */
export const PROPOSAL_SEED = 0;

/** A lot or a building goes where this share of the cells under it at least are painted its use (and none a green, but a green's). */
export const SHARE = 0.6;
/** What stands within this many cells (20 m) of a changed cell is built again. */
const DIRTY_GROW = 5;
/** A street drawn by hand this near an area's box (m) is one of its streets; a stretch this near grows it. */
const NEAR_AREA = 30;
/** A painted cluster this big at least (cells: 1,600 m²), with frontage of its own and `AREA_APART` from every area, is a new village. */
const VILLAGE_CELLS = 100;
/** A village keeps this many lots at least (`townCheck.ts`); a new one with fewer is folded into the nearest area. */
const VILLAGE_LOTS = 4;
/** The share of a terraces-and-flats stretch its terraced rows take (the backdrop's rows the rest). */
const TERRACE_SHARE = 0.5;
/** A terraced house's frontage, about (m). */
const TERRACE_FRONT = 10;
/** The backdrop's kinds along terraces and flats, and along shops and work. */
const ROW_KINDS: readonly BackdropUse[] = ['house', 'terrace', 'flats'];
const SHOP_KINDS: readonly BackdropUse[] = ['shop', 'cafe', 'restaurant', 'office', 'workshop'];
/** The shop lots an area with shops and none of its own is given one of, and a green stretch's lots, in turn. */
const SHOP_LOTS = ['grocer', 'bakery', 'cafe'];
const GREEN_LOTS = ['park', 'playground'];
/** A civic place is tried at frontage this far apart along its stretch (m), and at so many places at most. */
const CIVIC_STEP = 4;
const CIVIC_TRIES = 200;

// ---- streams ----------------------------------------------------------------------------------------

/** One 32-bit seed from a seed and words (FNV-1a over them, scrambled with MurmurHash3's last step), as `edit/buildup.ts` draws its streams. */
function mix(seed: number, ...words: (string | number)[]): number {
  let h = 0x811c9dc5 ^ (seed >>> 0);
  for (const ch of words.join('|')) h = Math.imul(h ^ ch.charCodeAt(0), 0x01000193);
  h ^= h >>> 16;
  h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13;
  h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 16;
  return h >>> 0;
}

const range = (rand: Rand, lo: number, hi: number) => lo + (hi - lo) * rand();
const dist = (a: Pt, b: Pt) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const offRect = (r: Rect, [x, z]: Pt) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
const middle = (a: Rect): Pt => [a.x + a.w / 2, a.z + a.d / 2];
const centroidOf = (poly: readonly Pt[]): Pt => [poly.reduce((s, p) => s + p[0], 0) / poly.length, poly.reduce((s, p) => s + p[1], 0) / poly.length];

/** A road's identity between the proposal and the layout: its line, kind and width (not its name nor area). */
const roadId = (r: RoadPlan) => JSON.stringify([r.points, r.kind ?? 'street', r.width]);

/** `draft` without its uses (a town built carries none on: they never go into its file). */
function withoutUses(draft: NeighbourhoodDraft): NeighbourhoodDraft {
  if (!draft.open || !('uses' in draft.open)) return draft;
  const { uses: _uses, ...open } = draft.open;
  return { ...draft, open };
}

// ---- cells ------------------------------------------------------------------------------------------

/** The uses grid's cells whose middles a rectangle covers (the one under its middle if none). */
function rectCells(r: Rect, cols: number, rows: number): number[] {
  const out: number[] = [];
  const [i0, i1] = [Math.max(0, Math.ceil(r.x / USE_CELL - 0.5)), Math.min(cols - 1, Math.floor((r.x + r.w) / USE_CELL - 0.5))];
  const [j0, j1] = [Math.max(0, Math.ceil(r.z / USE_CELL - 0.5)), Math.min(rows - 1, Math.floor((r.z + r.d) / USE_CELL - 0.5))];
  for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) out.push(j * cols + i);
  if (!out.length) {
    const [ci, cj] = [Math.floor((r.x + r.w / 2) / USE_CELL), Math.floor((r.z + r.d / 2) / USE_CELL)];
    if (ci >= 0 && cj >= 0 && ci < cols && cj < rows) out.push(cj * cols + ci);
  }
  return out;
}

/** The uses grid's cells whose middles a polygon covers (the one under its middle if none). */
function polyCells(poly: readonly Pt[], cols: number, rows: number): number[] {
  const out: number[] = [];
  const xs = poly.map((p) => p[0]);
  const zs = poly.map((p) => p[1]);
  const [i0, i1] = [Math.max(0, Math.floor(Math.min(...xs) / USE_CELL)), Math.min(cols - 1, Math.floor(Math.max(...xs) / USE_CELL))];
  const [j0, j1] = [Math.max(0, Math.floor(Math.min(...zs) / USE_CELL)), Math.min(rows - 1, Math.floor(Math.max(...zs) / USE_CELL))];
  for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) if (inside(poly as Pt[], (i + 0.5) * USE_CELL, (j + 0.5) * USE_CELL)) out.push(j * cols + i);
  if (!out.length) {
    const [cx, cz] = centroidOf(poly);
    const [ci, cj] = [Math.floor(cx / USE_CELL), Math.floor(cz / USE_CELL)];
    if (ci >= 0 && cj >= 0 && ci < cols && cj < rows) out.push(cj * cols + ci);
  }
  return out;
}

/** Whether `cells` (of `codes`) are painted `kind` enough for a lot or a building of it: `SHARE` of them at least, none a green but a green's. */
function painted(codes: Uint8Array, cells: readonly number[], kind: UseKind): boolean {
  if (!cells.length) return false;
  const [code, green] = [USE_CODE[kind], USE_CODE.green];
  let n = 0;
  for (const c of cells) {
    if (codes[c] === code) n++;
    else if (codes[c] === green) return false;
  }
  return n >= SHARE * cells.length;
}

// ---- areas ------------------------------------------------------------------------------------------

/**
 * `draft`'s areas as Build makes them (area 0 always the main town; the layout's own kept in their order):
 * each street or lane of no area (drawn by hand) is given the nearest area within `NEAR_AREA` m, whose
 * box grows to take it in, and each stretch zoned to be built (`dirty`, `fresh`: as `stretchesOf` reads
 * them; all with neither) grows the box of its road's area (a country road's: the nearest within
 * `NEAR_AREA` m), where the box's middle stays `AREA_APART` from every other's. A painted cluster
 * (`VILLAGE_CELLS` or more, further than `NEAR_AREA` from every box) along streets of no area, its box's
 * middle `AREA_APART` from every area's, is a new village (a hamlet, named as `hamletOf` names one, from
 * `seed`); the rest of the roads of no area go with the nearest. The draft, and the new villages' indices.
 */
export function areasFor(draft: NeighbourhoodDraft, content: Content, { seed = PROPOSAL_SEED, dirty, fresh }: { seed?: number; dirty?: Uint8Array; fresh?: ReadonlySet<number> } = {}): { draft: NeighbourhoodDraft; villages: number[] } {
  const open = draft.open;
  if (!open?.areas.length) return { draft, villages: [] };
  const areas: TownArea[] = open.areas.map((a) => ({ ...a }));
  const roads = draft.roads.slice();
  const grow: Rect[][] = areas.map(() => []);
  const nearestArea = (pts: readonly Pt[]) => {
    let [best, far] = [0, Infinity];
    areas.forEach((a, k) => {
      for (const p of pts) {
        const d = offRect(a, p);
        if (d < far) [best, far] = [k, d];
      }
    });
    return { area: best, far };
  };
  const bounds = (pts: readonly Pt[]): Rect => {
    const xs = pts.map((p) => p[0]);
    const zs = pts.map((p) => p[1]);
    return { x: Math.min(...xs), z: Math.min(...zs), w: Math.max(...xs) - Math.min(...xs), d: Math.max(...zs) - Math.min(...zs) };
  };
  // Streets and lanes of no area: the nearest within reach takes them in.
  const pending: number[] = [];
  roads.forEach((r, i) => {
    if (r.area !== undefined || r.kind === 'country' || r.bridge) return;
    const { area, far } = nearestArea(r.points);
    if (r.kind !== 'trail' && far > NEAR_AREA) return void pending.push(i);
    roads[i] = { ...r, area };
    if (r.kind !== 'trail') grow[area].push(bounds(r.points));
  });

  // A painted cluster out of every area's reach, along streets of no area: a village of its own.
  const villages: number[] = [];
  const layer = usesOfDraft(draft);
  const { cols, rows } = layer;
  const codes = unpackUses(layer);
  if (pending.length) {
    const far = new Uint8Array(codes.length);
    for (let k = 0; k < codes.length; k++) {
      if (!codes[k]) continue;
      const c: Pt = [((k % cols) + 0.5) * USE_CELL, (Math.floor(k / cols) + 0.5) * USE_CELL];
      if (areas.every((a) => offRect(a, c) > NEAR_AREA)) far[k] = 1;
    }
    const seen = new Uint8Array(codes.length);
    const spec = specFor(draft);
    for (let k0 = 0; k0 < far.length; k0++) {
      if (!far[k0] || seen[k0]) continue;
      const cells: number[] = [];
      const stack = [k0];
      seen[k0] = 1;
      while (stack.length) {
        const k = stack.pop()!;
        cells.push(k);
        const [i, j] = [k % cols, Math.floor(k / cols)];
        for (let dj = -1; dj <= 1; dj++) {
          for (let di = -1; di <= 1; di++) {
            const [a, b] = [i + di, j + dj];
            if (a < 0 || b < 0 || a >= cols || b >= rows) continue;
            const n = b * cols + a;
            if (far[n] && !seen[n]) {
              seen[n] = 1;
              stack.push(n);
            }
          }
        }
      }
      if (cells.length < VILLAGE_CELLS || (dirty && !cells.some((k) => dirty[k]))) continue;
      // (Its streets: those of no area running by it, its frontage.)
      const near = new Uint8Array(codes.length);
      for (const k of cells) near[k] = 1;
      const by = grown(near, cols, rows, Math.ceil(24 / USE_CELL));
      const own = pending.filter((i) => roads[i].points.some(([x, z]) => {
        const [ci, cj] = [Math.floor(x / USE_CELL), Math.floor(z / USE_CELL)];
        return ci >= 0 && cj >= 0 && ci < cols && cj < rows && by[cj * cols + ci] === 1;
      }));
      if (!own.length) continue;
      const pts: Pt[] = [...cells.flatMap((k): Pt[] => [[(k % cols) * USE_CELL, Math.floor(k / cols) * USE_CELL], [((k % cols) + 1) * USE_CELL, (Math.floor(k / cols) + 1) * USE_CELL]]), ...own.flatMap((i) => roads[i].points)];
      const b = bounds(pts);
      const box = { x: Math.floor(b.x), z: Math.floor(b.z), w: Math.ceil(b.x + b.w) - Math.floor(b.x), d: Math.ceil(b.z + b.d) - Math.floor(b.z) };
      if (areas.some((a) => dist(middle(a), middle(box)) < AREA_APART)) continue;
      const name = hamletOf(content, spec, mulberry32(mix(seed, 'village', villages.length)), areas.length - 1, areas.map((a) => a.name)).name;
      const at = areas.push({ name, kind: 'hamlet', ...box }) - 1;
      grow.push([]);
      villages.push(at);
      for (const i of own) roads[i] = { ...roads[i], area: at };
      pending.splice(0, pending.length, ...pending.filter((i) => !own.includes(i)));
    }
  }
  // The rest of the roads of no area: the nearest area's, its box as it was.
  for (const i of pending) roads[i] = { ...roads[i], area: nearestArea(roads[i].points).area };

  // The stretches to be built grow their areas' boxes.
  const placed: NeighbourhoodDraft = { ...draft, roads, open: { ...open, areas } };
  for (const st of stretchesOf(placed, { dirty, fresh })) {
    if (dirty && !st.dirty) continue;
    const r = roads[st.road];
    let a = r.area;
    if (a === undefined) {
      const { area, far } = nearestArea([st.mid]);
      if (far > NEAR_AREA) continue;
      a = area;
    }
    const ls = lengths(r.points);
    const ends = [pointAt(r.points, st.s0, ls), pointAt(r.points, st.s1, ls)];
    const back = r.width / 2 + 18;
    grow[a].push(bounds(ends.flatMap(({ p, t }): Pt[] => {
      const n = leftOf(t).map((v) => v * st.side);
      return [p, [p[0] + n[0] * back, p[1] + n[1] * back]];
    })));
  }
  grow.forEach((list, k) => {
    if (!list.length) return;
    const a = areas[k];
    const b = bounds([[a.x, a.z], [a.x + a.w, a.z + a.d], ...list.flatMap((r): Pt[] => [[r.x, r.z], [r.x + r.w, r.z + r.d]])]);
    const box = { x: Math.max(0, Math.floor(b.x)), z: Math.max(0, Math.floor(b.z)), w: 0, d: 0 };
    [box.w, box.d] = [Math.min(draft.width, Math.ceil(b.x + b.w)) - box.x, Math.min(draft.depth, Math.ceil(b.z + b.d)) - box.z];
    if (areas.some((o, j) => j !== k && dist(middle(o), middle(box)) < AREA_APART)) return;
    areas[k] = { ...a, ...box };
  });
  return { draft: placed, villages };
}

/** The spec `draft`'s world was laid out from (its lot mix, gaps, terraces, hamlets). */
function specFor(draft: NeighbourhoodDraft): WorldSpec {
  return specOf(placeOf(draft.world?.kind ?? 'country'), draft.world?.options);
}

// ---- what of the proposal's town stays ---------------------------------------------------------------

/** The edge of the nearest road but a footpath to `p` (m, from its middle line less half its width), on `index`'s roads; Infinity if none within 60 m. */
function toWay(index: RoadIndex, p: Pt): number {
  let best = Infinity;
  for (const [r, hit] of index.around(p[0], p[1], 60)) if (index.roads[r].kind !== 'trail') best = Math.min(best, hit.dist - index.roads[r].width / 2);
  return best;
}

/** The footpath outdoor lot `s` is at the end of (its end within 6 m of it), or -1. */
function ownTrail(roads: readonly RoadPlan[], t: Templates, s: PlotSlot): number {
  const r = plotRect(t, s);
  return roads.findIndex((road) => road.kind === 'trail' && offRect(r, road.points[road.points.length - 1]) <= 6);
}

/**
 * What of the proposal's town (`town`, its land as laid) is cleared for the layout `plan`: the lots and
 * backdrop buildings over `dirty` cells (a farm and its buildings over cells `zoned` instead, if given:
 * built again only where painted), in an area the layout no longer has, on its roads, fronting a road
 * that is gone (the nearest road to their front further than it was) or turned from a new road nearer
 * (a building), and an outdoor spot whose trail is gone; a farm's buildings with it. Lots and buildings
 * by index, and the footpaths that led to the outdoor spots cleared (by their index in `plan`).
 */
function clearedFor(town: NeighbourhoodDraft, plan: NeighbourhoodDraft, t: Templates, dirty: Uint8Array, zoned: Uint8Array | null, areaMap: readonly number[]): { lots: Set<number>; backdrop: Set<number>; trails: Set<number> } {
  const { cols, rows } = usesOfDraft(plan);
  const lots = new Set<number>();
  const backdrop = new Set<number>();
  const trails = new Set<number>();
  const planIds = plan.roads.map(roadId);
  const ids = new Set(planIds);
  const same = town.roads.length === plan.roads.length && town.roads.every((r) => ids.has(roadId(r)));
  // (The roads but the footpaths: an outdoor spot's own runs a metre into it.)
  const ground = new Ground(plan.width, plan.depth, plan.roads.filter((r) => r.kind !== 'trail'), plan.greens);
  const [was, now] = [new RoadIndex(town.roads), new RoadIndex(plan.roads)];
  const over = (cells: readonly number[], mask: Uint8Array) => cells.some((c) => mask[c] === 1);
  const farms: Pt[] = [];
  for (const s of town.slots) {
    const r = plotRect(t, s);
    const farm = s.lot === 'farm';
    let gone = areaMap[s.area ?? 0] < 0 || over(rectCells(r, cols, rows), farm && zoned ? zoned : dirty) || !ground.free(r, 0, true);
    if (!gone && s.kind === 'outdoor') {
      const trail = ownTrail(town.roads, t, s);
      gone = trail < 0 || !ids.has(roadId(town.roads[trail]));
    } else if (!gone && !same) {
      const f = frontOf(t, s);
      gone = toWay(now, f) > toWay(was, f) + 0.5;
    }
    if (!gone) continue;
    lots.add(s.index);
    if (farm) farms.push(middle(r));
    if (s.kind === 'outdoor') {
      const trail = ownTrail(plan.roads, t, s);
      if (trail >= 0) trails.add(trail);
    }
  }
  (town.open?.backdrop ?? []).forEach((b, i) => {
    const c = centroidOf(b.poly);
    const farmish = b.use === 'farm' || b.use === 'barn';
    let gone = areaMap[b.area] < 0 || over(polyCells(b.poly, cols, rows), farmish && zoned ? zoned : dirty) || (farmish && farms.some((m) => dist(m, c) < 70));
    if (!gone && !same) {
      const [before, after] = [toWay(was, c), toWay(now, c)];
      gone = after > before + 0.5 || (after < before - 0.5 && !facesRoad(b.poly, plan.roads, now)) || b.poly.some((p, k) => {
        const q = b.poly[(k + 1) % b.poly.length];
        return [0, 0.25, 0.5, 0.75].some((u) => now.onRoad(p[0] + (q[0] - p[0]) * u, p[1] + (q[1] - p[1]) * u, 0.5));
      });
    }
    if (gone) backdrop.add(i);
  });
  return { lots, backdrop, trails };
}

// ---- the fill ---------------------------------------------------------------------------------------

/** A public lot of `template` (a park, a playground, a shop) as `wantedLots` makes one, in area `area`. */
function publicLot(t: Templates, template: string, area: number): Unplaced | null {
  const p = t.parks.find((q) => q.id === template);
  if (!p) return null;
  return { kind: p.kind === 'shop' ? 'shop' : 'park', template: p.id, name: p.name, lot: 'medium', w: t.plot.width, d: t.plot.depth, area };
}

/**
 * Puts each of `wanted` along `frontage` as `LotPlacer.place` puts a lot (its first place where it fits,
 * snug to its road, a car able to drive out), but only where the cells under it are painted `kind`
 * (`painted`). How many went down.
 */
function placePainted(placer: LotPlacer, t: Templates, roads: readonly RoadPlan[], wanted: readonly Unplaced[], frontage: readonly Frontage[], codes: Uint8Array, cols: number, rows: number, kind: UseKind): number {
  let n = 0;
  for (const u of wanted) {
    const { width, depth } = slotDims(t, u);
    for (const f of frontage) {
      const at = lotAt(placer.ground, roads[f.road], f, width, depth, true);
      if (!at || !placer.drivesOut(u, at)) continue;
      if (!painted(codes, rectCells(plotRect(t, { ...u, ...at }), cols, rows), kind)) continue;
      placer.put(u, at, f);
      n++;
      break;
    }
  }
  return n;
}

/** The smallest home's lot (m along its road, m deep), for `roomAlong`. */
const SMALLEST: readonly [number, number] = [12, 16];

/** Whether any of `frontage` has room for the smallest home's lot (`SMALLEST`) on land painted homes: no house is drawn where none could go. */
function roomAlong(placer: LotPlacer, roads: readonly RoadPlan[], frontage: readonly Frontage[], codes: Uint8Array, cols: number, rows: number): boolean {
  const [w, d] = SMALLEST;
  return frontage.some((f) => {
    const at = lotAt(placer.ground, roads[f.road], f, w, d, true);
    return !!at && painted(codes, rectCells(at.facing % 2 ? { x: at.x, z: at.z, w: d, d: w } : { x: at.x, z: at.z, w, d }, cols, rows), 'homes');
  });
}

/** The outdoor places by the land they want (`PublicTemplate.site`). */
function outdoorFor(t: Templates, site: 'water' | 'woods' | 'hill' | 'meadow', rand: Rand) {
  const list = t.parks.filter((p) => p.kind === 'outdoor' && (p.site ?? 'meadow') === site);
  const all = list.length ? list : t.parks.filter((p) => p.kind === 'outdoor');
  return all.length ? all[Math.floor(rand() * all.length)] : null;
}

/**
 * The town `plan` (step 2's layout) builds, before its land is settled: see the top of this file.
 * `base`: the proposal it was laid out from (none: everything zoned is built from the uses).
 */
export function layOutUses(base: Proposal | null, plan: NeighbourhoodDraft, t: Templates, content: Content, seed: number): NeighbourhoodDraft {
  const open = plan.open;
  if (!open?.areas.length) return plan;
  const [W, D] = [plan.width, plan.depth];
  const layer = usesOfDraft(plan);
  const { cols, rows } = layer;
  const codes = unpackUses(layer);
  const full = !base || seed !== PROPOSAL_SEED;
  const town = base ? withoutUses(base.builtPlan ?? base.built) : null;

  // ---- what is built again: the changed cells and 20 m round them; everything, built again from another seed ----
  let dirty: Uint8Array = new Uint8Array(codes.length);
  let zoned: Uint8Array | null = null;
  if (full) {
    dirty.fill(1);
    zoned = new Uint8Array(codes.length);
    for (let k = 0; k < codes.length; k++) zoned[k] = codes[k] ? 1 : 0;
  } else {
    const was = unpackUses(usesOfDraft(base!.plan));
    for (let k = 0; k < codes.length; k++) if (codes[k] !== was[k]) dirty[k] = 1;
    dirty = grown(dirty, cols, rows, DIRTY_GROW);
  }

  // ---- what of the proposal's town stays ----
  let draft: NeighbourhoodDraft = { ...plan, slots: [], households: [], open: { ...open, backdrop: [], places: [], parking: [], pads: {} }, diagnostics: { dropped: [], ...(plan.diagnostics?.roads ? { roads: plan.diagnostics.roads } : {}) } };
  const touched = new Set<number>();
  let roads = plan.roads;
  if (town?.open) {
    const areaMap = town.open.areas.map((a) => open.areas.findIndex((b) => b.name === a.name));
    const gone = clearedFor(town, plan, t, dirty, zoned, areaMap);
    for (const i of gone.lots) touched.add(areaMap[town.slots[i].area ?? 0]);
    for (const i of gone.backdrop) touched.add(areaMap[town.open.backdrop![i].area]);
    const left = removeLots(town, gone.lots, gone.backdrop);
    const lo = left.open!;
    const ground = new Ground(W, D, plan.roads, plan.greens);
    // (A space as the generator laid it: 6 m along the verge, 3 m deep.)
    const space = (p: { x: number; z: number; rot: number }) => (p.rot % 2 ? { x: Math.round(p.x - 3), z: Math.round(p.z - 1.5), w: 6, d: 3 } : { x: Math.round(p.x - 1.5), z: Math.round(p.z - 3), w: 3, d: 6 });
    const parking = lo.parking.filter((p) => areaMap[p.area] >= 0 && ground.free(space(p), 0)).map((p) => ({ ...p, area: areaMap[p.area] }));
    roads = plan.roads.filter((_, i) => !gone.trails.has(i));
    draft = {
      ...draft,
      roads,
      slots: left.slots.map((s) => ({ ...s, area: areaMap[s.area ?? 0] })),
      open: {
        ...draft.open!,
        backdrop: (lo.backdrop ?? []).map((b) => ({ ...b, area: areaMap[b.area] })),
        places: (lo.places ?? []).map((p) => ({ ...p, area: areaMap[p.area] })),
        pads: lo.pads,
        parking,
      },
    };
  }
  touched.delete(-1);

  // ---- the areas, and the stretches to fill ----
  const baseIds = new Set(town?.roads.map(roadId) ?? []);
  const fresh = new Set(roads.flatMap((r, i) => (baseIds.has(roadId(r)) ? [] : [i])));
  const made = areasFor(draft, content, { seed, dirty, fresh });
  draft = made.draft;
  roads = draft.roads;
  const areas = draft.open!.areas;
  const nearestArea = (p: Pt) => areas.reduce((k, a, i) => (offRect(a, p) < offRect(areas[k], p) ? i : k), 0);
  const areaOf = (st: Stretch) => roads[st.road].area ?? nearestArea(st.mid);
  const stretches = stretchesOf(draft, { dirty, fresh })
    .filter((st) => st.dirty)
    .map((st) => ({ ...st, area: areaOf(st) }))
    .sort((a, b) => a.area - b.area || dist(a.mid, middle(areas[a.area])) - dist(b.mid, middle(areas[b.area])));
  for (const st of stretches) touched.add(st.area);
  for (const i of fresh) if (roads[i]?.area !== undefined) touched.add(roads[i].area!);
  for (const v of made.villages) touched.add(v);
  if (full) areas.forEach((_, a) => touched.add(a));

  // ---- the lots ----
  // (Each changed area's parking is laid again after: out of the lots' way till then.)
  draft = { ...draft, open: { ...draft.open!, parking: draft.open!.parking.filter((q) => !touched.has(q.area)) } };
  const spec = specFor(draft);
  const ground = townGround(draft, t);
  const corners = cornersOf(roads, roads.flatMap((r, i) => (fronted(r) ? [i] : [])));
  const placers: LotPlacer[] = [];
  const stream = (st: Stretch, ...more: string[]) => mulberry32(mix(seed, roadKey(roads[st.road]), st.kind, st.side, Math.round(st.s0), ...more));
  const frontageAlong = (st: Stretch & { area: number }, rand: Rand) => frontageOf(roads, [st.road], middle(areas[st.area]), rand).filter((f) => f.side === st.side && f.s >= st.s0 && f.s <= st.s1 && corners(f));
  // A park on each green the roads go round that is painted green, changed and has none, as the generator puts one there.
  for (const [g, poly] of draft.greens.entries()) {
    const cells = polyCells(poly, cols, rows);
    if (!cells.some((k) => dirty[k]) || !painted(codes, cells, 'green')) continue;
    const box = boundsOf(poly);
    const taken = [...draft.slots, ...placers.flatMap((p) => p.slots)].some((s) => {
      const r = plotRect(t, s);
      return r.x < box.x + box.w && box.x < r.x + r.w && r.z < box.z + box.d && box.z < r.z + r.d;
    });
    if (taken) continue;
    const c = centroidOf(poly);
    const area = nearestArea(c);
    const u = publicLot(t, GREEN_LOTS[0], area);
    if (!u) continue;
    const placer = new LotPlacer(content, t, ground, roads, [0, 0], mulberry32(mix(seed, 'green', g)), true);
    placer.place([u], [], [poly], c, area, true);
    placers.push(placer);
  }
  // A shop lot for each area with shops and work painted and none of its own.
  const hasShop = new Set(draft.slots.flatMap((s) => (s.kind === 'shop' ? [s.area ?? 0] : [])));
  for (const st of stretches) {
    if (st.kind !== 'shops' || hasShop.has(st.area)) continue;
    const rand = stream(st, 'shop lot');
    const u = publicLot(t, SHOP_LOTS[Math.floor(rand() * SHOP_LOTS.length)], st.area);
    if (!u) continue;
    const placer = new LotPlacer(content, t, ground, roads, [0, 0], rand, true);
    placers.push(placer);
    if (placePainted(placer, t, roads, [u], frontageAlong(st, rand), codes, cols, rows, 'shops')) hasShop.add(st.area);
  }
  for (const st of stretches) {
    if (st.kind === 'shops' || st.kind === 'public') continue;
    const rand = stream(st);
    const frontage = frontageAlong(st, rand);
    if (!frontage.length) continue;
    const placer = new LotPlacer(content, t, ground, roads, spec.gaps, rand, true);
    placers.push(placer);
    if (st.kind === 'homes') {
      // (No home in a town centre: its frontage is kept for the centre's buildings.)
      const homes = frontage.filter((f) => !inCentre(draft, [f.p[0] + f.n[0] * 10, f.p[1] + f.n[1] * 10]));
      // (A home at a time, while there is room for the smallest: a house is drawn only to be put down.)
      const one = { ...spec.town, lots: 1, places: [0, 0] as [number, number], farms: [0, 0] as [number, number] };
      for (let k = 0, missed = 0; k < 60 && missed < 4 && roomAlong(placer, roads, homes, codes, cols, rows); k++) {
        const wanted = wantedLots(content, t, spec, one, rand, { park: false }).map((u) => ({ ...u, area: st.area }));
        missed = placePainted(placer, t, roads, wanted, homes, codes, cols, rows, 'homes') ? 0 : missed + 1;
      }
    } else if (st.kind === 'rows') {
      const k = Math.max(2, Math.round(((st.s1 - st.s0) * TERRACE_SHARE) / TERRACE_FRONT));
      const laid = terraceRows(t, placer, [frontage], { terraces: [k, k], rows: spec.rows[1] >= 2 ? spec.rows : [2, 4] }, rand);
      // (A row only where it is painted terraces and flats, every house of it.)
      const bad = new Set(laid.filter((row) => row.some((s) => !painted(codes, rectCells(plotRect(t, s), cols, rows), 'rows'))).flat());
      placer.slots.splice(0, placer.slots.length, ...placer.slots.filter((s) => !bad.has(s)));
      for (const s of bad) placer.placed.delete(s);
      for (const s of placer.slots) s.area = st.area;
    } else if (st.kind === 'green') {
      for (const id of GREEN_LOTS) {
        const u = publicLot(t, id, st.area);
        if (u) placePainted(placer, t, roads, [u], frontage, codes, cols, rows, 'green');
      }
    }
  }
  // Each green with no frontage near a town or village with no outdoor spot on it: an outdoor spot, a trail to it from the area's streets.
  let land: ReturnType<typeof cells8> | null = null;
  let water: Uint8Array | null = null;
  const trails: RoadPlan[] = [];
  const names = new Set(roads.map((r) => r.name));
  const reach = roadReach(draft);
  for (const spot of greenSpots(draft)) {
    const kept = draft.slots.some((s) => s.kind === 'outdoor' && rectCells(plotRect(t, s), cols, rows).some((k) => spot.cells.includes(k)));
    if (kept) continue;
    const rand = mulberry32(mix(seed, 'spot', Math.round(spot.centre[0]), Math.round(spot.centre[1])));
    water ??= open.water ? unrle(open.water, W * D) : new Uint8Array(W * D);
    land ??= open.land ? cells8(open.land) : null;
    if (!land) break;
    const wet = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < D && water![Math.floor(z) * W + Math.floor(x)] === 1;
    const [cx, cz] = spot.centre;
    const site = [0, 1, 2, 3, 4, 5, 6, 7].some((a) => wet(cx + Math.cos((a * Math.PI) / 4) * 24, cz + Math.sin((a * Math.PI) / 4) * 24)) ? 'water' : land.woods[land.cell(cx, cz)] ? 'woods' : land.at(cx, cz) > land.at(...middle(areas[spot.area])) + 4 ? 'hill' : 'meadow';
    const place = outdoorFor(t, site, rand);
    if (!place) break;
    const frame = place.lot ?? t.plot;
    const home = middle(areas[spot.area]);
    // (Its cells nearest the green's middle first; facing the area, its trail comes from there.)
    const tries = [...spot.cells].sort((a, b) => dist(cellMid(a, cols), spot.centre) - dist(cellMid(b, cols), spot.centre));
    let at: { x: number; z: number; facing: Facing } | null = null;
    for (const k of tries.slice(0, 40)) {
      const c = cellMid(k, cols);
      const v: Pt = [c[0] - home[0], c[1] - home[1]];
      const facing = (Math.abs(v[1]) >= Math.abs(v[0]) ? (v[1] > 0 ? 0 : 2) : v[0] > 0 ? 1 : 3) as Facing;
      const [w, d] = facing % 2 ? [frame.depth, frame.width] : [frame.width, frame.depth];
      const r = { x: Math.round(c[0] - w / 2), z: Math.round(c[1] - d / 2), w, d };
      if (!ground.free({ x: r.x - 3, z: r.z - 3, w: r.w + 6, d: r.d + 6 }, 8)) continue;
      if ([0, 0.5, 1].some((u) => [0, 0.5, 1].some((q) => wet(r.x - 2 + (r.w + 4) * u, r.z - 2 + (r.d + 4) * q)))) continue;
      if (!painted(codes, rectCells(r, cols, rows), 'green') || rectCells(r, cols, rows).some((q) => reach[q] <= BUILD_REACH - 8)) continue;
      at = { x: r.x, z: r.z, facing };
      break;
    }
    if (!at) continue;
    const placer = new LotPlacer(content, t, ground, roads, [0, 0], rand, true);
    const area = areas[spot.area];
    const slot = placer.put({ kind: 'outdoor', template: place.id, name: `${area.name} ${place.name}`, lot: 'medium', w: frame.width, d: frame.depth, area: spot.area }, at, null);
    const areaRoads = roads.flatMap((r, i) => (r.area === spot.area && r.kind !== 'trail' && r.kind !== 'country' ? [i] : []));
    const plan: AreaPlan = { name: area.name, rect: { x: area.x, z: area.z, w: area.w, d: area.d }, centre: home, roads: areaRoads, greens: [], spec: spec.town, ports: [] };
    const others = [...draft.slots, ...placers.flatMap((p) => p.slots), slot];
    const trail = areaRoads.length ? trailTo(slot, plan, [...roads, ...trails], others, water, land, t, W, D) : null;
    if (!trail) continue;
    let name = `${slot.name} Trail`;
    if (names.has(name)) name = `${slot.name} Path`;
    for (let k = 2; names.has(name); k++) name = `${slot.name} Trail ${k}`;
    names.add(name);
    trails.push({ ...trail, name, area: spot.area });
    for (const [x, z] of trail.points) ground.take({ x: Math.floor(x) - 1, z: Math.floor(z) - 1, w: 3, d: 3 });
    placers.push(placer);
  }
  const added = placers.flatMap((p) => p.slots).map((s, k): PlotSlot => ({ ...s, index: draft.slots.length + k }));
  roads = [...roads, ...trails];
  const pads = { ...draft.open!.pads };
  for (const s of added) {
    if (s.kind !== 'outdoor' || !open.land) continue;
    const r = plotRect(t, s);
    pads[s.index] = Math.round(heightAt(open.land, r.x + r.w / 2, r.z + r.d / 2) * 10) / 10;
  }
  draft = { ...draft, roads, slots: [...draft.slots, ...added], open: { ...draft.open!, pads } };
  for (const s of added) if (s.kind !== 'house') draft = withLotPlace(draft, t, content, s.index);

  // ---- the backdrop: rows along terraces and flats, and along shops and work; the civic places ----
  const pass = new BackdropPass({ draft, t, content });
  const from = pass.count;
  const meant = new Map<number, UseKind>();
  const since = (n0: number, kind: UseKind) => {
    for (let i = n0; i < pass.count; i++) meant.set(i, kind);
  };
  for (const st of stretches) {
    if (st.kind !== 'rows' && st.kind !== 'shops') continue;
    const n0 = pass.count;
    const rand = stream(st, 'rows');
    const rows = pass.rowsAlong(st.area, st.road, [st.side], rand, undefined, { from: st.s0, to: st.s1, kinds: st.kind === 'rows' ? ROW_KINDS : SHOP_KINDS });
    pass.secondRow(rows, rand);
    since(n0, st.kind);
  }
  const publicAreas = [...new Set(stretches.filter((st) => st.kind === 'public').map((st) => st.area))];
  for (const a of publicAreas) {
    const rand = mulberry32(mix(seed, 'civic', a));
    const mid = middle(areas[a]);
    const fronts: { road: number; side: 1 | -1; s: number; far: number }[] = [];
    for (const st of stretches) {
      if (st.kind !== 'public' || st.area !== a) continue;
      const r = roads[st.road];
      const ls = lengths(r.points);
      for (let s = st.s0 + 4; s <= st.s1 - 4; s += CIVIC_STEP) fronts.push({ road: st.road, side: st.side, s, far: dist(pointAt(r.points, s, ls).p, mid) });
    }
    fronts.sort((x, y) => x.far - y.far);
    const taken: { road: number; side: number; s0: number; s1: number }[] = [];
    for (const c of civicChoices(draft, a)) {
      const kind = CIVIC[c.index].kind;
      let tries = 0;
      for (const f of fronts) {
        if (taken.some((q) => q.road === f.road && q.side === f.side && f.s > q.s0 && f.s < q.s1)) continue;
        if (++tries > CIVIC_TRIES) break;
        const [w, d] = [range(rand, ...kind.w), range(rand, ...kind.d)];
        if (!painted(codes, polyCells(civicBox(roads[f.road], f.s, f.side, w, d), cols, rows), 'public')) continue;
        const n0 = pass.count;
        if (pass.putCivic(c.index, f.road, f.s, f.side, rand, { w, d }) < 0) continue;
        since(n0, 'public');
        taken.push({ road: f.road, side: f.side, s0: f.s - w / 2 - 2, s1: f.s + w / 2 + 2 });
        break;
      }
    }
  }
  const laid = pass.result().backdrop;
  const keep: number[] = [];
  for (let i = from; i < laid.length; i++) if (painted(codes, polyCells(laid[i].poly, cols, rows), meant.get(i) ?? 'shops')) keep.push(i);
  const places = pass.placesFor(keep, mulberry32(mix(seed, 'places')));
  draft = { ...draft, open: { ...draft.open!, backdrop: [...(draft.open!.backdrop ?? []), ...keep.map((i) => laid[i])], places: [...(draft.open!.places ?? []), ...places] } };

  // ---- parking in each area changed; a new village with too few lots goes with the nearest area ----
  draft = foldSmallVillages(parkedAgain(draft, t, touched, seed), made.villages);

  // ---- the houses numbered along every street ----
  return renumber(draft, t, roads.map((_, i) => i));
}

/** The middle of cell `k` of a grid `cols` wide (m). */
const cellMid = (k: number, cols: number): Pt => [((k % cols) + 0.5) * USE_CELL, (Math.floor(k / cols) + 0.5) * USE_CELL];

/** A civic building's ground fronting `road` at `s` on `side`, `w` along and `d` deep, set back as `putCivic` sets one (about 3 m past the road's edge). */
function civicBox(road: RoadPlan, s: number, side: 1 | -1, w: number, d: number): Pt[] {
  const { p, t } = pointAt(road.points, s);
  const n = leftOf(t).map((v) => v * side);
  const back = road.width / 2 + 3;
  const at = (u: number, v: number): Pt => [p[0] + t[0] * u + n[0] * v, p[1] + t[1] * u + n[1] * v];
  return [at(-w / 2, back), at(w / 2, back), at(w / 2, back + d), at(-w / 2, back + d)];
}

/** `draft` with each of `villages` (new areas, by index) that has fewer than `VILLAGE_LOTS` lots folded into the nearest other area (its streets, lots, buildings, places and parking), and gone. */
function foldSmallVillages(draft: NeighbourhoodDraft, villages: readonly number[]): NeighbourhoodDraft {
  const open = draft.open;
  if (!open || !villages.length) return draft;
  const small = villages.filter((v) => draft.slots.filter((s) => (s.area ?? 0) === v && s.kind !== 'outdoor').length < VILLAGE_LOTS);
  if (!small.length) return draft;
  const areas = open.areas;
  const keep = areas.flatMap((_, i) => (small.includes(i) ? [] : [i]));
  const index = new Map(keep.map((a, k) => [a, k]));
  const to = (a: number): number => {
    if (index.has(a)) return index.get(a)!;
    const m = middle(areas[a]);
    const best = keep.reduce((k, b) => (dist(middle(areas[b]), m) < dist(middle(areas[k]), m) ? b : k), keep[0]);
    return index.get(best)!;
  };
  return {
    ...draft,
    roads: draft.roads.map((r) => (r.area === undefined ? r : { ...r, area: to(r.area) })),
    slots: draft.slots.map((s) => ({ ...s, area: to(s.area ?? 0) })),
    open: {
      ...open,
      areas: keep.map((a) => areas[a]),
      backdrop: (open.backdrop ?? []).map((b) => ({ ...b, area: to(b.area) })),
      places: (open.places ?? []).map((p) => ({ ...p, area: to(p.area) })),
      parking: open.parking.map((p) => ({ ...p, area: to(p.area) })),
    },
  };
}

/** `draft` with the parking of each area of `areas` laid again as the generator lays it (`parkingFor` its homes, by the verge nearest its middle). */
function parkedAgain(draft: NeighbourhoodDraft, t: Templates, areas: ReadonlySet<number>, seed: number): NeighbourhoodDraft {
  const open = draft.open;
  if (!open || !areas.size) return draft;
  const parking = open.parking.filter((p) => !areas.has(p.area));
  const ground = townGround({ ...draft, open: { ...open, parking } }, t);
  const { roads } = draft;
  for (const a of [...areas].sort((x, y) => x - y)) {
    const area = open.areas[a];
    if (!area) continue;
    const own = roads.flatMap((r, i) => (r.area === a && fronted(r) ? [i] : []));
    if (!own.length) continue;
    const homes = draft.slots.filter((s) => s.kind === 'house' && (s.area ?? 0) === a).length;
    const spaces = parkingFor(homes, a === 0);
    let n = 0;
    for (const f of frontageOf(roads, own, middle(area), mulberry32(mix(seed, 'parking', a)))) {
      if (n >= spaces) break;
      const facing = (Math.abs(f.n[1]) >= Math.abs(f.n[0]) ? (f.n[1] > 0 ? 0 : 2) : f.n[0] > 0 ? 1 : 3) as Facing;
      const reach = roads[f.road].width / 2 + 0.5;
      const [qx, qz] = [f.p[0] + f.n[0] * (reach + 3), f.p[1] + f.n[1] * (reach + 3)];
      const r = facing % 2 ? { x: Math.round(qx - 3), z: Math.round(qz - 1.5), w: 6, d: 3 } : { x: Math.round(qx - 1.5), z: Math.round(qz - 3), w: 3, d: 6 };
      if (!ground.free(r, 8) || parking.some((p) => Math.hypot(p.x - r.x, p.z - r.z) < 12)) continue;
      ground.take(r, 1);
      parking.push({ x: r.x + r.w / 2, z: r.z + r.d / 2, rot: facing, area: a });
      n++;
    }
  }
  return { ...draft, open: { ...open, parking } };
}

// ---- the job ----------------------------------------------------------------------------------------

/**
 * The town step 2's layout `plan` builds (see the top of this file), from `base` (the proposal it was laid
 * out from; none: everything zoned built from its uses) and the build's `seed`: its draft (no uses), its
 * preview, the houses its lots use, and (a land whose town follows its slopes) its plan (the land as laid,
 * step 3's edits start from it) and what in it is over a slope limit (`marks`).
 */
export function buildFromUses(base: Proposal | null, plan: NeighbourhoodDraft, t: Templates, content: Content, seed: number): LaidTown {
  const preview = (d: NeighbourhoodDraft) => previewWorld(content, t, d, null);
  if (base && seed === PROPOSAL_SEED && layoutHash(plan) === base.hash) {
    const draft = withoutUses(base.built);
    const built = base.builtPlan ? { plan: withoutUses(base.builtPlan), marks: base.builtMarks ?? [] } : {};
    return { draft, preview: preview(draft), houses: generatedHouses(t, draft.slots.map((s) => s.template)), ...built };
  }
  const laid = withoutUses(layOutUses(base, plan, t, content, seed));
  const settled = settleLand(laid, preview, undefined, content.careerCategories, { keepRoads: true });
  let draft = settled.draft;
  const world = settled.world;
  const land = draft.open?.land;
  if (draft.open && land && draft !== laid) {
    // The cover opened under what was built since the land was laid, and the woods, meadows and fields made again from it (as step 2's settle does).
    const opened = { ...land, cover: land.cover.slice() };
    const { zones } = openUnderBuilt(opened, world, draft.open.zones);
    draft = { ...draft, open: { ...draft.open, land: opened, zones } };
    world.meta = { ...world.meta, zones, land: encodeLand(opened) };
  }
  const asLaid = laid.open?.land;
  const own = draft.open && asLaid?.seeded === 'follow' && draft.open.land !== asLaid ? { plan: { ...draft, open: { ...draft.open, land: asLaid } }, marks: settled.marks } : {};
  return { draft, preview: world, houses: generatedHouses(t, draft.slots.map((s) => s.template)), ...own };
}

