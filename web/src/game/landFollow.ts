/**
 * A new place's land as a height field whose town follows its slopes (`Land.seeded` `'follow'`):
 * seeded bare, with no town on it, from the generator's own heights (`OpenWorldDraft.heights`:
 * `seedBare`, step 1); once a town is laid on it, gentle ground eased in under the town
 * (`gentleUnder`, the built land not flattened), then the roads, the lots and the land round them
 * graded into it (`render/shared/builtSurface.ts`), so the land a world stores agrees with its
 * town. Classic worlds and older saves have none (they stay flat); a land a player shaped keeps
 * its heights and is eased and graded the same way under the town laid on it.
 */

import type { CareerCategory } from '../content/content';
import { DEFAULT_LANDSCAPE, type RoadPlan, type TownPlace, type WorldStructure, type WorldZone } from '../core/protocol';
import { BRIDGE_CLEAR, builtSurfaceOf, CUT_MAX, CUT_MAX_MOUNTAIN_ROAD, GRADE, profileOf, RETAIN_MAX, surfacePlots, type BuiltSurface, type SurfaceWorld } from '../render/shared/builtSurface';
import { blankLand, COVER, COVER_CELL, coverSpan, heightAt, LAND_CELL, NONE, SEA_CM, type Land } from './land';
import { HOURS, hoursFor } from './backdrop';
import { APRON, builtOf, landZones, Protection, wetTiles } from './landEdit';
import { lotKey, removeLots } from './edit/lots';
import { rle, unrle } from './landMasks';
import { deckTiles } from './macro';
import { fbm, ridged, smoothstep } from './noise';
import type { NeighbourhoodDraft } from './town';
import { COURT, lengths, nearest, pointAt, RoadIndex, type Pt } from './roads';

/** The steepest a town's or a village's ground is tilted (rise per m: `gentler`). */
export const GENTLE_TILT = 0.06;

/** An area's lots this near its box and streets (m) are on its gentle ground (`gentler`); those further out have their own. */
const AREA_LOTS = 30;
/** How much more a town's or a village's plane has to say in the blend than a lone lot's (`gentler`). */
const TOWN_SAY = 1000;

/** Dry land stays this far over the sea's level (m) in a world with water: the sea is one sheet over the whole world. */
const DRY_OVER = 0.15;
/** The land beside water rises from this far over it (m), no steeper than the world's slope. */
const SHORE_OVER = 0.1;
/** A pond's level lies this far below the lowest of the land round it (m). */
const POND_BELOW = 0.1;
/** Water that reaches the world's edge or is bigger than this (m²) is at the sea's level (the sea, a lake, a river); smaller is a pond, at its own. */
const POND_MOST = 8000;
/** A pond whose level would come within this (m) of the sea's is at the sea's (one sheet). */
const POND_OVER = 0.2;
/** The deepest water (m), and its bed going down from the shore as the renderer's sheet's does (`nature.ts` `ground`). */
const DEPTH_MOST = 4;
const bedBelow = (e: number) => Math.min(DEPTH_MOST, e * 0.22 + e * e * 0.004);

/** What of a world the seeding and the grading read: its size, plots and meta (roads, paths, backdrop, look), as a menu's preview or a town file has them. */
export type FollowWorld = Pick<WorldStructure, 'width' | 'depth' | 'plots' | 'meta'> & SurfaceWorld;

/**
 * `draft` with its land (`open.land`) whose town follows its slopes: the land it was laid on
 * (`open.land`, seeded `'follow'`: step 1's, shaped or not), or else one seeded bare from its
 * heights (`seedBare`), given gentle ground under its towns (`gentleUnder`) and graded
 * (`gradeLand`). Anything else (a classic world, a draft from older code, a land drawn flat under
 * its town) as it was. `world`: the draft's preview (`previewWorld`), for its lots, their paths
 * and its roads. `seeded`: given the land before it is graded, its gentle ground in it (a copy).
 * `ease`: country roads whose land is eased to their own profile besides (`easeAlong`, `settleLand`).
 * Its areas' planes, fitted afresh, are kept with it (`open.gentle`: step 2's settles ease to them).
 */
export function withLand(draft: NeighbourhoodDraft, world: FollowWorld, seeded?: (land: Land) => void, ease: readonly RoadPlan[] = []): NeighbourhoodDraft {
  const open = draft.open;
  if (!open) return draft;
  let land: Land | null;
  if (open.land) land = open.land.seeded === 'follow' ? { ...open.land, h: open.land.h.slice() } : null;
  else land = open.heights ? seedBare(draft) : null;
  if (!land) return draft;
  const gentle = gentleUnder(land, world, draft.slots.map((s) => s.area ?? 0));
  if (ease.length) easeAlong(land, ease, { ...DEFAULT_LANDSCAPE, ...world.meta?.world?.landscape }.rise);
  seeded?.({ ...land, h: land.h.slice(), wl: land.wl.slice(), cover: land.cover.slice() });
  gradeLand(land, world);
  return { ...draft, open: { ...open, land, gentle } };
}

// ---- the limits checked, and the last resort -------------------------------------------------

/**
 * How a world's built land sits on its land (`measureLand`): the steepest of each kind of way
 * (rise per m), the tallest lot edges (m: over the land beside, another pad, a road), the deepest
 * cuts and fills (m: roads, lots), the least a bridge clears its water by (m), and how far the land
 * stored is from the surface worked out from it (m).
 */
export interface Slopes {
  grade: Record<string, number>;
  retain: { land: number; lot: number; road: number };
  cut: { road: number; lot: number };
  clear: number;
  drift: number;
}

/**
 * How far a road or a pad is over a slope limit (`measureLand`): it climbs too steeply (`climb`), is
 * cut into the land or stands over it too far (`cut`), or its edge stands too tall over the land
 * (`edge`) or the pad (`edgePad`) beside it; `by` how much (rise per m, or m) and the limit `most`.
 */
export interface Over {
  what: 'climb' | 'cut' | 'edge' | 'edgePad';
  by: number;
  most: number;
}

/**
 * What `measureLand` found over the limits: in words, and which pads (`surfacePlots` indices) and
 * roads break them; and for those over the limits themselves (not only within `margin` of them), how
 * (`over`: the first limit each breaks, keyed as `pads` and `roads` are).
 */
export interface LandMeasure {
  slopes: Slopes;
  problems: string[];
  pads: Set<number>;
  roads: Set<number>;
  over: { pads: Map<number, Over>; roads: Map<number, Over> };
}

/**
 * The slope limits checked on `world`'s built land as the renderer works it out from `land` (the
 * land stored: `builtSurfaceOf`), against `seeded` (the land before its town was graded in): no
 * road (not a trail, a footpath on the land as it lies) steeper than its kind's `GRADE` by more
 * than 1 %, or cut into the land or standing over it by more than `CUT_MAX` (a country road among
 * mountains `CUT_MAX_MOUNTAIN_ROAD`; not over a bridge's deck); no lot cut or filled by more than that; no lot's edge over the land or the pad beside it
 * by more than `RETAIN_MAX` (land narrower than the land's grain between two pads is the other
 * pad's edge; between a pad and a road, a verge: listed, not checked); every bridge's deck
 * `BRIDGE_CLEAR` over its water. `margin` (m) the limits are kept within besides (the last resort's);
 * `drift`: whether to measure how far the land stored is from the surface (`Slopes.drift`, else 0).
 */
export function measureLand(world: FollowWorld, land: Land, seeded: Land, margin = 0, drift = true): LandMeasure {
  const out: string[] = [];
  const pads = new Set<number>();
  const bad = new Set<number>();
  const surface = builtSurfaceOf(world, land);
  const roads = world.meta?.roads ?? [];
  const slopes: Slopes = { grade: {}, retain: { land: 0, lot: 0, road: 0 }, cut: { road: 0, lot: 0 }, clear: Infinity, drift: 0 };
  const over: LandMeasure['over'] = { pads: new Map(), roads: new Map() };
  const note = (into: Map<number, Over>, n: number, o: Over) => into.has(n) || into.set(n, o);
  const wet = (l: Land, k: number) => l.wl[k] !== NONE && l.wl[k] > l.h[k];
  const [cutMost, edgeMost] = [CUT_MAX - margin, RETAIN_MAX + 0.01 - margin];
  // (Among mountains a country road, a pass, may be cut deeper into the rock.)
  const mountains = (world.meta?.world as { options?: { hills?: string } } | undefined)?.options?.hills === 'mountains';
  const roadMost = (kind: string) => (mountains && kind === 'country' ? CUT_MAX_MOUNTAIN_ROAD : CUT_MAX);
  roads.forEach((r, i) => {
    const kind = r.kind ?? 'street';
    const { s, y } = surface.profile(i);
    let steepest = 0;
    for (let k = 0; k + 1 < s.length; k++) if (s[k + 1] - s[k] > 0.5) steepest = Math.max(steepest, Math.abs(y[k + 1] - y[k]) / (s[k + 1] - s[k]));
    slopes.grade[kind] = Math.max(slopes.grade[kind] ?? 0, steepest);
    if (kind === 'trail') return;
    if (steepest > GRADE[kind] + 0.01) {
      out.push(`${kind} ${r.name ?? i} climbs ${(steepest * 100).toFixed(1)} % (at most ${GRADE[kind] * 100} %)`);
      note(over.roads, i, { what: 'climb', by: steepest, most: GRADE[kind] });
      bad.add(i);
    }
    const span = r.bridge && r.bridge[1] > r.bridge[0] ? r.bridge : null;
    const ls = lengths(r.points);
    let [clear, cut] = [Infinity, 0];
    for (let k = 0; k < s.length; k++) {
      const { p } = pointAt(r.points, s[k], ls);
      const at = Math.min(seeded.rows - 1, Math.max(0, Math.round(p[1] / LAND_CELL))) * seeded.cols + Math.min(seeded.cols - 1, Math.max(0, Math.round(p[0] / LAND_CELL)));
      if (span && s[k] >= span[0] && s[k] <= span[1]) {
        if (wet(land, at)) clear = Math.min(clear, y[k] - land.wl[at] / 100);
        continue;
      }
      if (!wet(seeded, at)) cut = Math.max(cut, Math.abs(y[k] - heightAt(seeded, p[0], p[1])));
    }
    slopes.cut.road = Math.max(slopes.cut.road, cut);
    if (cut > roadMost(kind) - margin) {
      if (cut > roadMost(kind)) {
        out.push(`${kind} ${r.name ?? i} is cut ${cut.toFixed(1)} m into the land or stands over it (at most ${roadMost(kind)})`);
        note(over.roads, i, { what: 'cut', by: cut, most: roadMost(kind) });
      }
      bad.add(i);
    }
    if (span) {
      slopes.clear = Math.min(slopes.clear, clear);
      if (clear < BRIDGE_CLEAR - 0.01) out.push(`the bridge on ${r.name ?? i} clears its water by ${clear.toFixed(2)} m (at least ${BRIDGE_CLEAR})`);
    }
  });
  // The lots: cut and fill under each pad, and its edges.
  const index = new RoadIndex(roads);
  const all = surfacePlots(world);
  const cells = new Map<number, number[]>();
  all.forEach((q, n) => {
    for (let cz = Math.floor((q.z - LAND_CELL) / PAD_CELL); cz <= Math.floor((q.z + q.d + LAND_CELL) / PAD_CELL); cz++) {
      for (let cx = Math.floor((q.x - LAND_CELL) / PAD_CELL); cx <= Math.floor((q.x + q.w + LAND_CELL) / PAD_CELL); cx++) cells.set(cx * 65536 + cz, [...(cells.get(cx * 65536 + cz) ?? []), n]);
    }
  });
  /** The pad (a lot's or a backdrop building's) but `skip`'s within `by` m (no more than the land's grain) of (x, z), if any (the first). */
  const padNear = (x: number, z: number, skip: number, by = 0) => {
    for (const n of cells.get(Math.floor(x / PAD_CELL) * 65536 + Math.floor(z / PAD_CELL)) ?? []) {
      const q = all[n];
      if (n !== skip && x >= q.x - by && x <= q.x + q.w + by && z >= q.z - by && z <= q.z + q.d + by) return n;
    }
    return -1;
  };
  world.plots.forEach((p, n) => {
    const pad = surface.padY(n);
    let cut = 0;
    for (let j = Math.ceil(p.z / LAND_CELL); j <= Math.floor((p.z + p.d) / LAND_CELL); j++) {
      for (let i = Math.ceil(p.x / LAND_CELL); i <= Math.floor((p.x + p.w) / LAND_CELL); i++) if (i < seeded.cols && j < seeded.rows) cut = Math.max(cut, Math.abs(pad - seeded.h[j * seeded.cols + i] / 100));
    }
    slopes.cut.lot = Math.max(slopes.cut.lot, cut);
    if (cut > cutMost) {
      if (cut > CUT_MAX) {
        out.push(`lot ${n} (${p.w}×${p.d} m) is cut ${cut.toFixed(1)} m into the land or stands over it (at most ${CUT_MAX})`);
        note(over.pads, n, { what: 'cut', by: cut, most: CUT_MAX });
      }
      pads.add(n);
    }
    const edge = (x: number, z: number) => {
      if (x < 0 || z < 0 || x >= world.width - 0.01 || z >= world.depth - 0.01) return;
      const step = Math.abs(surface.at(x, z) - pad);
      const other = padNear(x, z, n, LAND_CELL);
      if (other >= 0) {
        slopes.retain.lot = Math.max(slopes.retain.lot, step);
        if (step > edgeMost) {
          // (The backdrop building, else the later lot, gives way; but a lot gives way to a building that is a place, a shop or the like.)
          const place = other >= world.plots.length && world.meta?.backdrop?.[other - world.plots.length]?.place !== undefined;
          const gives = place ? n : other >= world.plots.length ? other : Math.max(n, other);
          if (step > RETAIN_MAX + 0.01) {
            out.push(`lot ${n}'s edge stands ${step.toFixed(2)} m over the pad beside it (at most ${RETAIN_MAX})`);
            note(over.pads, gives, { what: 'edgePad', by: step, most: RETAIN_MAX });
          }
          pads.add(gives);
        }
      } else if (index.onRoad(x, z, LAND_CELL)) slopes.retain.road = Math.max(slopes.retain.road, step);
      else {
        slopes.retain.land = Math.max(slopes.retain.land, step);
        if (step > edgeMost) {
          if (step > RETAIN_MAX + 0.01) {
            out.push(`lot ${n}'s edge stands ${step.toFixed(2)} m over the land beside it (at most ${RETAIN_MAX})`);
            note(over.pads, n, { what: 'edge', by: step, most: RETAIN_MAX });
          }
          pads.add(n);
        }
      }
    };
    for (let t = 0.5; t < p.w; t += 1) {
      edge(p.x + t, p.z - 0.3);
      edge(p.x + t, p.z + p.d + 0.3);
    }
    for (let t = 0.5; t < p.d; t += 1) {
      edge(p.x - 0.3, p.z + t);
      edge(p.x + p.w + 0.3, p.z + t);
    }
  });
  // The land stored against the surface worked out from it, off what is built (unless not asked: the last resort).
  if (!drift) return { slopes, problems: out, pads, roads: bad, over };
  const lot = new Uint8Array(land.h.length);
  for (const p of world.plots) {
    for (let j = Math.ceil(p.z / LAND_CELL); j <= Math.min(land.rows - 1, Math.floor((p.z + p.d) / LAND_CELL)); j++) {
      for (let i = Math.ceil(p.x / LAND_CELL); i <= Math.min(land.cols - 1, Math.floor((p.x + p.w) / LAND_CELL)); i++) lot[j * land.cols + i] = 1;
    }
  }
  for (let k = 0; k < land.h.length; k++) {
    if (wet(land, k)) continue;
    const [x, z] = [(k % land.cols) * LAND_CELL, Math.floor(k / land.cols) * LAND_CELL];
    const on = lot[k] ? null : surface.roadAt(x, z);
    if (on && roads[on.road].bridge && on.s >= roads[on.road].bridge![0] && on.s <= roads[on.road].bridge![1]) continue;
    slopes.drift = Math.max(slopes.drift, Math.abs((lot[k] || on ? surface.at(x, z) : surface.conformed[k]) - land.h[k] / 100));
  }
  return { slopes, problems: out, pads, roads: bad, over };
}

/** Pads are filed for `measureLand` by cells this wide (m). */
const PAD_CELL = 16;

/** The last resort's rounds at most, and how far within the limits it keeps (m: the town file's surface may differ a little from its preview's). */
const SETTLE_ROUNDS = 5;
/** A backdrop building this near (m) a road left out goes with it (it faced it). */
const BY_ROAD = 40;
const SETTLE_MARGIN = 0.15;

/**
 * `withLand`, and the last resort: a place's world whose built land still breaks a slope limit
 * (`measureLand`, a little within it) gives up what breaks it and is graded again, round after
 * round (`SETTLE_ROUNDS` at most): a lot or a backdrop building cut too deep or whose edge stands
 * too tall (of two pads, the backdrop building, else the later lot; a lot before a building that is a place) is left out (`diagnostics.dropped`);
 * a country road cut too deep (a way out, a loop between places) is left out, and the lots and
 * backdrop buildings on it (those by it, and those it was the nearest road to), where the roads left are still one network with a way out; a dead end reaching past `COURT` m beyond the last road it meets is
 * cut back to that, and the lots past it left out; a country road cut too deep that may not be
 * left out has the land along it eased to its own profile the next round (`easeAlong`: once).
 * What it gave up: `diagnostics.slopes`. A career
 * that worked in the main town only at places given up works on there (`rehome`: `careers`, the
 * content's career categories; none given, it doesn't).
 * `preview`: the world as the menus' preview has it (its lots' paths: `previewWorld`; asked of the
 * draft without its land). The draft, that world, and what in it is still over a slope limit itself
 * (`marks`, as `settleEdit` keys them: what it couldn't give up).
 */
export function settleLand<W extends FollowWorld>(laid: NeighbourhoodDraft, preview: (draft: NeighbourhoodDraft) => W, seeded?: (land: Land) => void, careers: readonly CareerCategory[] = []): { draft: NeighbourhoodDraft; world: W; marks: string[] } {
  let draft = laid;
  const given = { lots: 0, backdrop: 0, roads: 0, courts: 0, backdropIn: {} as Record<number, number> };
  /** The country roads whose land is eased along them (`roadKey`). */
  const ease = new Set<string>();
  for (let round = 0; ; round++) {
    const world = preview(unlanded(draft));
    let first: Land | null = null;
    const graded = withLand(draft, world, (l) => (first = l), ease.size ? draft.roads.filter((r) => ease.has(roadKey(r))) : []);
    const land = graded.open?.land;
    if (!land || !first) return { draft: graded, world, marks: [] };
    const courts = longCourts(draft.roads);
    const found = round < SETTLE_ROUNDS ? measureLand(world, land, first, SETTLE_MARGIN, false) : null;
    const next = found ? giveUp(draft, world, found, courts, given, careers, ease) : null;
    if (!next) {
      seeded?.(first);
      const total = given.lots + given.backdrop + given.roads + given.courts;
      if (!given.backdrop) delete (given as { backdropIn?: unknown }).backdropIn;
      // (What is left over a limit itself: step 2's first settle's `before`. The margin changes what is given up, not that.)
      const marks = [...overOf(draft, world, found ?? measureLand(world, land, first, 0, false)).keys()];
      return { draft: total ? { ...graded, diagnostics: { ...(graded.diagnostics ?? { dropped: [] }), slopes: given } } : graded, world, marks };
    }
    draft = next;
  }
}

// ---- step 2's settle ---------------------------------------------------------------------------

/** What `settleEdit` gives: see there. */
export interface EditSettled<W> {
  draft: NeighbourhoodDraft;
  world: W;
  /** The edits refused (`op`: an edit's index in the batch, from 0), each with why: in words, as a clause ("Mill Lane would climb 21 %, …"). */
  refused: { op: number; why: string }[];
  /** What in the settled town is over a slope limit (`settleKeys`): the next settle's `before`. */
  marks: string[];
  /** The plan settled: its index in the batch's `steps` (the last, unless an edit was refused: then the one that edit was made on). */
  step: number;
}

/**
 * Step 2's settle: the town's plan draft (`plan`: its `open.land` the land as laid, its heights not
 * yet graded under the town; step 1's, with the town's woods and fields) with its land settled under
 * it as `settleLand` settles a new town's (gentle ground under the town: `gentleUnder`; the roads and
 * lots graded in: `gradeLand`), but giving nothing up (decision 11: an edit never makes something
 * else go). Its cover is opened within 2 m of anything built (as `macro.ts` `settleTown` opens it),
 * the woods, meadows and fields of its zones made again from that (the water, hills and orchards
 * kept), and its walkers' water (`open.water`) made again, the plan's (else the land's), with every
 * bridge's deck dried (`deckTiles` over `RoadPlan.bridge`) as the generator dries them. A plan with
 * no land of that kind (a classic town) is its own settle. `preview`: the world as the menus'
 * preview has it (asked of the plan without its land).
 *
 * What the batch of edits made over a slope limit (`measureLand`: a road too steep, a road or a lot
 * cut too deep or standing too high, a lot's edge too tall) is refused. `steps`: the plans the batch
 * went through, the first the plan last settled, the last `plan` (so its edits are `steps.length - 1`).
 * `before`: what was over a limit in the last settle (its `marks`); not given, the first step is
 * settled too to find it (its time again). What is over a limit now and wasn't then is the batch's:
 * each is laid to the edit that brought it in (a road or a lot that edit added or changed), else to
 * the edit whose changes are nearest it (removing a lot changes the ground under its neighbours).
 * Each refused edit is given the reason for the first of its; the plan the first refused edit was
 * made on is settled instead, and checked again in turn (the edits after it go with it: the plans
 * build on each other). With no `steps` nothing is refused.
 */
export function settleEdit<W extends FollowWorld>(plan: NeighbourhoodDraft, preview: (draft: NeighbourhoodDraft) => W, before?: Iterable<string>, steps?: readonly NeighbourhoodDraft[]): EditSettled<W> {
  const path = steps && steps.length > 1 ? [...steps.slice(0, -1), plan] : [plan];
  let at = path.length - 1;
  let settled = settleOnce(plan, preview);
  const refused: { op: number; why: string }[] = [];
  if (at > 0) {
    let first: Settle<W> | null = null;
    const had = new Set(before ?? (first = settleOnce(path[0], preview)).over.keys());
    // (Each step's things by their keys, made once.)
    const things = path.map(thingsOf);
    for (;;) {
      const fresh = [...settled.over].filter(([key]) => !had.has(key));
      if (!fresh.length || at === 0) break;
      // (Each edit's reason: the first of what it brought in itself, else the first of what it broke nearby.)
      const blamed = new Map<number, { why: string; own: boolean }>();
      for (const [key, o] of fresh) {
        const { op, own } = blame(key, o.at, things.slice(0, at + 1));
        if (!blamed.has(op) || (own && !blamed.get(op)!.own)) blamed.set(op, { why: o.why, own });
      }
      for (const [op, { why }] of blamed) if (!refused.some((r) => r.op === op)) refused.push({ op, why });
      at = Math.min(...blamed.keys());
      settled = at === 0 && first ? first : settleOnce(path[at], preview);
    }
    refused.sort((a, b) => a.op - b.op);
  }
  return { draft: settled.draft, world: settled.world, refused, marks: [...settled.over.keys()], step: at };
}

/** One settle of `settleEdit`'s: the plan's draft settled, its preview, and what is over a slope limit in it (by its key: where it is, and why in words). */
interface Settle<W> {
  draft: NeighbourhoodDraft;
  world: W;
  over: Map<string, { at: Pt[]; why: string }>;
}

/** `plan` settled as `settleEdit` settles it, giving nothing up: see there. */
function settleOnce<W extends FollowWorld>(plan: NeighbourhoodDraft, preview: (draft: NeighbourhoodDraft) => W): Settle<W> {
  const open = plan.open;
  const laid = open?.land;
  if (!open || !laid || laid.seeded !== 'follow') return { draft: plan, world: preview(plan), over: new Map() };
  const world = preview(unlanded(plan));
  const land: Land = { ...laid, h: laid.h.slice(), wl: laid.wl.slice(), cover: laid.cover.slice() };
  // The cover open under and by what is built (`macro.ts` `settleTown`), and the zones' woods, meadows and fields from it.
  const guard = new Protection(land.cols, land.rows, builtOf(world));
  const cw = coverSpan(land.cols);
  const c = COVER_CELL / LAND_CELL;
  for (let r = 0; r < coverSpan(land.rows); r++) for (let q = 0; q < cw; q++) if (guard.dist[Math.min(land.rows - 1, r * c + 1) * land.cols + Math.min(land.cols - 1, q * c + 1)] <= 2) land.cover[r * cw + q] = COVER.open;
  const cover = landZones(land, { cover: true });
  const zones = (['water', 'hill', 'fields', 'meadow', 'woods', 'orchard'] as const).flatMap((kind) => (kind === 'fields' || kind === 'meadow' || kind === 'woods' ? cover : open.zones).filter((z) => z.kind === kind));
  // The walkers' water: the plan's (else the land's), every bridge's deck dry.
  const [W, D] = [plan.width, plan.depth];
  const walk = open.water ? unrle(open.water, W * D) : wetTiles(land, W, D);
  for (const r of plan.roads) if (r.bridge && r.bridge[1] > r.bridge[0]) deckTiles(r, r.bridge, W, D, (tile) => (walk[tile] = 0));
  // The land eased and graded under the town, and measured (no margin: the limits themselves). Each
  // area's plane as the town was laid (`open.gentle`), so an edit changes the land only round itself;
  // a village put down or moved since gets one fitted afresh.
  const gentle = gentleUnder(land, world, plan.slots.map((s) => s.area ?? 0), open.gentle);
  const seeded: Land = { ...land, h: land.h.slice() };
  gradeLand(land, world);
  const over = overOf(plan, world, measureLand(world, land, seeded, 0, false));
  return { draft: { ...plan, open: { ...open, land, zones, water: rle(walk), gentle } }, world, over };
}

/** What `found` (measured on `plan`'s preview, `world`) says is over a slope limit itself, by its key (`roadKey`, `slotKey`, `buildingKey`): where it is, and why in words. */
function overOf(plan: NeighbourhoodDraft, world: FollowWorld, found: LandMeasure): Map<string, { at: Pt[]; why: string }> {
  const over = new Map<string, { at: Pt[]; why: string }>();
  for (const [i, o] of found.over.roads) {
    const r = plan.roads[i];
    over.set(roadKey(r), { at: r.points, why: overWords(o, r.name ?? `the ${r.kind ?? 'street'}`, r.kind ?? 'street') });
  }
  const plots = world.plots.length;
  const building = surfacePlots(world);
  for (const [n, o] of found.over.pads) {
    const p = building[n];
    const at: Pt[] = [[p.x, p.z], [p.x + p.w, p.z + p.d]];
    if (n < plots) over.set(slotKey(plan.slots[n]), { at, why: overWords(o, plan.slots[n].name || 'the lot', 'lot') });
    else over.set(buildingKey(plan.open!.backdrop![n - plots]), { at, why: overWords(o, 'a building there', 'building') });
  }
  return over;
}

/** How `o` breaks a slope limit, in words, of `name` (a road of `kind`, a lot or a building). */
function overWords(o: Over, name: string, kind: string): string {
  const m = (v: number) => `${v.toFixed(1)} m`;
  switch (o.what) {
    case 'climb':
      return `${name} would climb ${Math.round(o.by * 100)} %, more than a ${kind} may (${Math.round(o.most * 100)} %)`;
    case 'cut':
      return `the ground under ${name} would have to be dug out or built up ${m(o.by)} (${m(o.most)} at most)`;
    case 'edge':
      return `${name} would stand ${m(o.by)} over the land beside it (${m(o.most)} at most)`;
    case 'edgePad':
      return `${name} would stand ${m(o.by)} over the lot beside it (${m(o.most)} at most)`;
  }
}

/**
 * What is over a slope limit is known across edits by keys (`EditSettled.marks`): a road by where it
 * begins and ends, its points, kind and width (a road renamed is the same road; one drawn on further
 * another); a lot by where it stands (`lotKey`), the way it faces, its size and its house; a backdrop
 * building by its footprint. Whatever changes how the land is graded under it changes its key.
 */
const tenth = (v: number) => Math.round(v * 10) / 10;
function roadKey(r: RoadPlan): string {
  const [a, b] = [r.points[0], r.points[r.points.length - 1]];
  return `road ${tenth(a[0])},${tenth(a[1])} ${tenth(b[0])},${tenth(b[1])} ${r.points.length} ${r.kind ?? 'street'} ${r.width}`;
}
function slotKey(s: NeighbourhoodDraft['slots'][number]): string {
  return `lot ${lotKey(s)} ${s.facing} ${s.lot ?? ''} ${s.w ?? ''}x${s.d ?? ''} ${s.template}`;
}
function buildingKey(b: { poly: readonly Pt[] }): string {
  return `building ${b.poly.map(([x, z]) => `${tenth(x)},${tenth(z)}`).join(' ')}`;
}

/** A plan's roads, lots and backdrop buildings by their keys, and where each is (a lot by its corner). */
function thingsOf(draft: NeighbourhoodDraft): Map<string, readonly Pt[]> {
  const out = new Map<string, readonly Pt[]>();
  for (const r of draft.roads) out.set(roadKey(r), r.points);
  for (const s of draft.slots) out.set(slotKey(s), [[s.x, s.z]]);
  for (const b of draft.open?.backdrop ?? []) out.set(buildingKey(b), b.poly);
  return out;
}

/**
 * Which edit (its index among `steps.length - 1`) what is over a limit (`key`, at `at`) is laid to:
 * the last that brought it in (`own`), else the one whose changes (what it added or took away) are
 * nearest it; the last edit if none changed anything there is.
 */
function blame(key: string, at: readonly Pt[], steps: readonly Map<string, readonly Pt[]>[]): { op: number; own: boolean } {
  const ops = steps.length - 1;
  for (let i = ops - 1; i >= 0; i--) if (steps[i + 1].has(key) && !steps[i].has(key)) return { op: i, own: true };
  let [best, far] = [ops - 1, Infinity];
  for (let i = 0; i < ops; i++) {
    const [a, b] = [steps[i], steps[i + 1]];
    for (const [one, other] of [
      [b, a],
      [a, b],
    ]) {
      for (const [k, pts] of one) {
        if (other.has(k)) continue;
        const d = apart(at, pts);
        if (d <= far) [best, far] = [i, d];
      }
    }
  }
  return { op: best, own: false };
}

/** The least distance between two sets of points (m; every 4th of a long one, and its ends). */
function apart(a: readonly Pt[], b: readonly Pt[]): number {
  const thin = (p: readonly Pt[]) => (p.length > 16 ? p.filter((_, i) => i % 4 === 0 || i === p.length - 1) : p);
  let d = Infinity;
  for (const [x, z] of thin(a)) for (const [u, v] of thin(b)) d = Math.min(d, Math.hypot(x - u, z - v));
  return d;
}

/** `draft` without its land (`open.land`): what a settle round previews, its land not wanted there (nor coded into the preview). */
function unlanded(draft: NeighbourhoodDraft): NeighbourhoodDraft {
  if (!draft.open?.land) return draft;
  const { land: _land, ...open } = draft.open;
  return { ...draft, open };
}

/** The dead ends (not trails') reaching past `COURT` m beyond the last road they meet: each road, its end, and how far along it the last meeting is. */
function longCourts(roads: readonly RoadPlan[]): { road: number; end: 0 | 1; at: number }[] {
  const joins: number[][] = roads.map(() => []);
  roads.forEach((r, i) => {
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      roads.forEach((o, j) => {
        if (j === i) return;
        const near = nearest(o.points, end[0], end[1]);
        if (near.dist <= Math.max(o.width, r.width) / 2 + 1) joins[j].push(near.s);
      });
    }
  });
  const out: { road: number; end: 0 | 1; at: number }[] = [];
  roads.forEach((r, i) => {
    if (r.kind === 'trail') return;
    const L = lengths(r.points).at(-1)!;
    for (const e of [0, 1] as const) {
      if (r.ends[e] !== 'dead') continue;
      const met = joins[i].map((s) => (e === 1 ? s : L - s));
      const last = Math.max(0, ...met);
      if (L - last > COURT + 0.5) out.push({ road: i, end: e, at: e === 1 ? last : L - last });
    }
  });
  return out;
}

/**
 * `draft` without what `found` and `courts` say breaks the limits (see `settleLand`), counted in
 * `given`; null if it gives up nothing (all that breaks them is what it may not give up) and eases
 * no road it didn't already (`ease`, keys added to: a country road cut too deep that it may not
 * give up).
 */
function giveUp(draft: NeighbourhoodDraft, world: FollowWorld, found: LandMeasure, courts: ReturnType<typeof longCourts>, given: { lots: number; backdrop: number; roads: number; courts: number; backdropIn: Record<number, number> }, careers: readonly CareerCategory[], ease: Set<string>): NeighbourhoodDraft | null {
  const open = draft.open;
  if (!open) return null;
  const plots = world.plots.length;
  const entries = surfacePlots(world);
  const lots = new Set([...found.pads].filter((n) => n < plots));
  const backdrop = new Set([...found.pads].filter((n) => n >= plots).map((n) => n - plots));
  let roads = draft.roads.map((r) => ({ ...r, points: r.points.slice() }));
  const gone = new Set<number>();
  /** The dead ends cut back: per road, the stretch of it kept (m along it as it was). */
  const kept = new Map<number, [number, number]>();
  // Country roads cut too deep (a way out of the world, a loop between places), where the rest is
  // still one network with a way out. (One that must stay, over the cut limit itself: the land
  // along it eased to it next round.)
  let eased = false;
  for (const r of [...found.roads].sort((a, b) => a - b)) {
    if (roads[r].kind !== 'country') continue;
    const without = new Set([...gone, r]);
    if (!oneNetwork(roads, without)) {
      const key = roadKey(roads[r]);
      if (found.over.roads.get(r)?.what === 'cut' && !ease.has(key)) [eased] = [true, ease.add(key)];
      continue;
    }
    gone.add(r);
    given.roads++;
  }
  // Dead ends too long, cut back to `COURT` m past the last road they meet.
  for (const c of courts) {
    if (gone.has(c.road)) continue;
    const r = roads[c.road];
    const ls = lengths(r.points);
    const L = ls[ls.length - 1];
    const [from, to] = c.end === 1 ? [0, c.at + COURT] : [c.at - COURT, L];
    const pts: Pt[] = [pointAt(r.points, from, ls).p];
    for (let k = 0; k < r.points.length; k++) if (ls[k] > from && ls[k] < to) pts.push(r.points[k]);
    pts.push(pointAt(r.points, to, ls).p);
    const trimmed = roads.map((o, i) => (i === c.road ? { ...r, points: pts } : o));
    // (Never one that leaves the roads in two.)
    if (!oneNetwork(trimmed, gone)) continue;
    roads = trimmed;
    kept.set(c.road, [from, to]);
    given.courts++;
    // (The lots past the new end.)
    entries.forEach((p, n) => {
      if (n < plots && p.entry?.road === c.road && (p.entry.s < from - 0.5 || p.entry.s > to + 0.5)) lots.add(n);
    });
  }
  // (The lots on a road left out, and the backdrop buildings by it or facing it: no road of their
  // own left nearer. And the backdrop buildings facing a dead end past where it was cut back to, as
  // the lots there: the road they faced is gone from by them.)
  const facedGone = (x: number, z: number) => {
    let best = { road: -1, dist: Infinity, s: 0 };
    draft.roads.forEach((r, i) => {
      const { dist, s } = nearest(r.points, x, z);
      if (dist < best.dist) best = { road: i, dist, s };
    });
    const cut = kept.get(best.road);
    return gone.has(best.road) || (!!cut && (best.s < cut[0] - 0.5 || best.s > cut[1] + 0.5));
  };
  entries.forEach((p, n) => {
    const [x, z] = [p.x + p.w / 2, p.z + p.d / 2];
    if (n < plots ? p.entry && gone.has(p.entry.road) : (gone.size && [...gone].some((r) => nearest(draft.roads[r].points, x, z).dist < BY_ROAD)) || ((gone.size || kept.size) && facedGone(x, z))) (n < plots ? lots : backdrop).add(n < plots ? n : n - plots);
  });
  if (!eased && !gone.size && !lots.size && !backdrop.size && roads.every((r, i) => r === draft.roads[i] || r.points.length === draft.roads[i].points.length)) return null;
  roads = roads.filter((_, i) => !gone.has(i));
  given.lots += lots.size;
  given.backdrop += backdrop.size;
  for (const i of backdrop) {
    const area = open.backdrop?.[i]?.area ?? 0;
    given.backdropIn[area] = (given.backdropIn[area] ?? 0) + 1;
  }
  // The slots left, numbered again; the pads, places and backdrop buildings that name them with them (`edit/lots.ts`).
  const dropped = [...(draft.diagnostics?.dropped ?? []), ...draft.slots.filter((s) => lots.has(s.index)).map((s) => ({ name: s.name, kind: s.kind, ...(s.lot ? { lot: s.lot } : {}) }))];
  const removed = removeLots({ ...draft, roads }, lots, backdrop);
  const places = removed.open?.places && rehome(open.places ?? [], removed.open.places, careers);
  return { ...removed, ...(removed.open && places !== removed.open.places ? { open: { ...removed.open, places } } : {}), diagnostics: { ...(draft.diagnostics ?? { dropped: [] }), dropped } };
}

/**
 * The main town's places (`after`, area 0) given back the careers that worked there at places
 * given up (in `before`, not in `after`): each at a place of the kind it worked at (a garage's
 * mechanics at another workshop), else at the first backdrop office, as the generator gives an
 * office the careers no place took (`backdrop.ts` `places`); its hours widened to their shifts
 * (`hoursFor`). `after` as it was if no career lost its place (or none fits).
 */
function rehome(before: readonly TownPlace[], after: TownPlace[], careers: readonly CareerCategory[]): TownPlace[] {
  const working = (list: readonly TownPlace[]) => new Set(list.flatMap((p) => (p.area === 0 ? (p.work ?? []) : [])));
  const [had, still] = [working(before), working(after)];
  const lost = careers.map((c) => c.id).filter((id) => had.has(id) && !still.has(id));
  if (!lost.length) return after;
  const out = after.slice();
  for (const id of lost) {
    const kind = before.find((p) => p.area === 0 && p.work?.includes(id))!.kind;
    const at = [out.findIndex((p) => p.area === 0 && p.kind === kind), out.findIndex((p) => p.area === 0 && p.kind === 'office' && p.plot === undefined)].find((i) => i >= 0);
    if (at === undefined) continue;
    const p = out[at];
    const base = p.hours ?? (p.kind === 'park' || p.kind === 'outdoor' ? null : HOURS[p.kind]);
    out[at] = { ...p, work: [...(p.work ?? []), id], ...(base ? { hours: hoursFor(base, [id], careers) } : {}) };
  }
  return out;
}

/**
 * The land (`land`, its dry heights changed in place) along each of `roads` eased to that road's own
 * profile on it (`profileOf`: within its grade, as little off the land as it can be): wholly out to
 * the road's edges, fading back to the land's own height over `reach` m (the look's `rise`), as a
 * town's gentle ground fades (`gentler`). For a country road `settleLand` may not give up, cut too
 * deep where it climbs between two gentle planes further apart in height than its grade climbs.
 */
function easeAlong(land: Land, roads: readonly RoadPlan[], reach: number): void {
  const { cols, rows } = land;
  for (const r of roads) {
    const { s, y } = profileOf(r.points, 'country', land);
    if (!s.length) continue;
    const ls = lengths(r.points);
    const half = r.width / 2;
    const far = half + reach;
    const xs = r.points.map((p) => p[0]);
    const zs = r.points.map((p) => p[1]);
    const [i0, i1] = [Math.max(0, Math.floor((Math.min(...xs) - far) / LAND_CELL)), Math.min(cols - 1, Math.ceil((Math.max(...xs) + far) / LAND_CELL))];
    const [j0, j1] = [Math.max(0, Math.floor((Math.min(...zs) - far) / LAND_CELL)), Math.min(rows - 1, Math.ceil((Math.max(...zs) + far) / LAND_CELL))];
    // (The profile's height `at` m along the road, between its samples.)
    const heightAlong = (at: number) => {
      let k = 0;
      while (k + 1 < s.length && s[k + 1] < at) k++;
      if (k + 1 >= s.length) return y[s.length - 1];
      const f = Math.max(0, Math.min(1, (at - s[k]) / (s[k + 1] - s[k] || 1)));
      return y[k] + f * (y[k + 1] - y[k]);
    };
    for (let j = j0; j <= j1; j++) {
      for (let i = i0; i <= i1; i++) {
        const k = j * cols + i;
        if (land.wl[k] !== NONE && land.wl[k] > land.h[k]) continue;
        const near = nearest(r.points, i * LAND_CELL, j * LAND_CELL, ls);
        if (near.dist >= far) continue;
        const w = 1 - smoothstep(half, far, near.dist);
        const own = land.h[k] / 100;
        land.h[k] = Math.max(-32767, Math.min(32767, Math.round((own + w * (heightAlong(near.s) - own)) * 100)));
      }
    }
  }
}

/** Whether the roads but those `without` are one network (as the sim joins them) with a road out of the world on it. */
function oneNetwork(roads: readonly RoadPlan[], without: ReadonlySet<number>): boolean {
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  roads.forEach((r, i) => {
    if (without.has(i)) return;
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      roads.forEach((o, j) => {
        if (j === i || without.has(j)) return;
        if (nearest(o.points, end[0], end[1]).dist <= Math.max(o.width, r.width) / 2 + 1) parent[find(i)] = find(j);
      });
    }
  });
  const nets = new Set(roads.flatMap((_, i) => (without.has(i) ? [] : [find(i)])));
  return nets.size === 1 && roads.some((r, i) => !without.has(i) && r.ends.includes('out'));
}

/** The roads, lots and the land round them graded into `land` (its heights changed in place; water left as it is): the surface it was graded to. */
export function gradeLand(land: Land, world: SurfaceWorld): BuiltSurface {
  const surface = builtSurfaceOf(world, land);
  land.h.set(surface.graded());
  return surface;
}

/**
 * A place's bare land (step 1), from its draft with no town on it (`macro.ts` `landDraft`: its
 * `open.heights`, which a classic world has not). Every `LAND_CELL` m:
 * - the generator's heights between its cells' middles, over the level the town is built at
 *   (`LandPlan.level`), with the finer hills, some roughness, and among mountains their ridges
 *   everywhere (`gentleUnder` takes them out again under and round what is built);
 * - its water from the water zones: what reaches the world's edge or is bigger than `POND_MOST`
 *   at the sea's level (`SEA_CM`), each pond at its own, a little below the lowest land round it;
 *   the land beside water no higher than over it by the world's slope (`shoreClamp`), the bed
 *   going down from the shore as the renderer's does, and in a world with water the dry land over
 *   the sea's level;
 * - its cover from the woods, meadow and fields zones.
 */
export function seedBare(draft: NeighbourhoodDraft): Land {
  const open = draft.open;
  const heights = open?.heights;
  if (!open || !heights) throw new Error('seedBare: the draft has no heights');
  const land = blankLand(draft.width, draft.depth, 'follow');
  const { cols, rows, h: cm, wl } = land;
  const look = { ...DEFAULT_LANDSCAPE, ...draft.world?.landscape };
  const seed = draft.seed ?? 1;
  const h = new Float32Array(cols * rows);
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const [x, z] = [i * LAND_CELL, j * LAND_CELL];
      const base = between(heights, x, z);
      let detail = (fbm(x * 0.018, z * 0.018, seed) - 0.5) * look.hills + base * 0.3 * (fbm(x * 0.012, z * 0.012, seed + 517, 3) - 0.5);
      if (look.mountains && base > 15) detail += base * 0.35 * (ridged(x * 0.0075, z * 0.0075, seed + 211) - 0.5) * smoothstep(15, 25, base);
      h[j * cols + i] = base + detail;
    }
  }

  // ---- the water ----
  const wet = scan(
    open.zones.filter((z) => z.kind === 'water').map((z) => z.poly),
    cols,
    rows,
    LAND_CELL,
    0,
  );
  const sea = SEA_CM / 100;
  if (wet.includes(1)) {
    for (let k = 0; k < h.length; k++) if (!wet[k]) h[k] = Math.max(h[k], sea + DRY_OVER);
    // Each piece of water (side by side) at its level.
    const piece = new Int32Array(cols * rows).fill(-1);
    for (let start = 0; start < piece.length; start++) {
      if (!wet[start] || piece[start] >= 0) continue;
      const members = [start];
      piece[start] = start;
      let [edge, rim] = [false, Infinity];
      for (let n = 0; n < members.length; n++) {
        const k = members[n];
        const [i, j] = [k % cols, Math.floor(k / cols)];
        if (i === 0 || j === 0 || i === cols - 1 || j === rows - 1) edge = true;
        for (const [ni, nj] of [[i - 1, j], [i + 1, j], [i, j - 1], [i, j + 1]] as const) {
          if (ni < 0 || nj < 0 || ni >= cols || nj >= rows) continue;
          const q = nj * cols + ni;
          if (!wet[q]) rim = Math.min(rim, h[q]);
          else if (piece[q] < 0) {
            piece[q] = start;
            members.push(q);
          }
        }
      }
      const pond = !edge && members.length * LAND_CELL * LAND_CELL <= POND_MOST && rim - POND_BELOW > sea + POND_OVER;
      const level = pond ? Math.round((rim - POND_BELOW) * 100) : SEA_CM;
      for (const k of members) wl[k] = level;
    }
    // How far each wet sample is from dry land: its bed.
    const into = new Float32Array(cols * rows);
    for (let k = 0; k < h.length; k++) into[k] = wet[k] ? Infinity : 0;
    chamfer(cols, rows, (k, n, d) => {
      if (into[n] + d < into[k]) into[k] = into[n] + d;
    });
    for (let k = 0; k < h.length; k++) if (wet[k]) h[k] = wl[k] / 100 - bedBelow(into[k]);
    shoreClamp(h, wl, cols, rows, look.slope ?? 0.5);
  }
  for (let k = 0; k < h.length; k++) cm[k] = Math.max(-32767, Math.min(32767, Math.round(h[k] * 100)));

  // ---- the cover ----
  const [cw, cd] = [coverSpan(cols), coverSpan(rows)];
  for (const kind of ['fields', 'meadow', 'woods'] as const) {
    const mask = scan(
      open.zones.filter((z: WorldZone) => z.kind === kind).map((z) => z.poly),
      cw,
      cd,
      COVER_CELL,
      COVER_CELL / 2,
    );
    for (let c = 0; c < mask.length; c++) if (mask[c]) land.cover[c] = COVER[kind];
  }
  return land;
}

/**
 * The dry land beside water (`h`, m per sample; `wl` its water levels, cm) brought down to no
 * higher over the water than the world's slope (`bank`, rise per m) takes it from `SHORE_OVER`
 * over it at the shore, and in a world with water no lower than `DRY_OVER` over the sea's level.
 * The water's beds are left as they are.
 */
function shoreClamp(h: Float32Array, wl: Int16Array, cols: number, rows: number, bank: number): void {
  const sea = SEA_CM / 100;
  const wet = (k: number) => wl[k] !== NONE && wl[k] / 100 > h[k];
  const top = new Float32Array(cols * rows);
  let any = false;
  for (let k = 0; k < h.length; k++) {
    top[k] = wet(k) ? wl[k] / 100 + SHORE_OVER : Infinity;
    any ||= wet(k);
  }
  if (!any) return;
  chamfer(cols, rows, (k, n, d) => {
    if (top[n] + bank * d < top[k]) top[k] = top[n] + bank * d;
  });
  for (let k = 0; k < h.length; k++) if (!wet(k)) h[k] = Math.min(Math.max(h[k], sea + DRY_OVER), Math.max(top[k], sea + DRY_OVER));
}

/** How far the fine hills are smoothed out under and round what is built (`smoothHeights`: samples either way, and how many times). */
const SMOOTH_REACH = 8;
const SMOOTH_PASSES = 3;

/**
 * The land's dry heights (m) with its finer hills smoothed out (three box blurs of
 * `SMOOTH_REACH` samples either way, over the dry samples alone: a gaussian of about 17 m), its
 * wider hills and its mountains kept: what the land under and round a town eases to
 * (`gentleUnder`), and what the generator lays a town by (`macro.ts` `fillFrom`). Wet samples get
 * the dry land's round them.
 */
export function smoothHeights(land: Land): Float32Array {
  const { cols, rows } = land;
  const n = cols * rows;
  const sum = new Float32Array(n);
  const weight = new Float32Array(n);
  for (let k = 0; k < n; k++) {
    if (land.wl[k] !== NONE && land.wl[k] > land.h[k]) continue;
    sum[k] = land.h[k] / 100;
    weight[k] = 1;
  }
  const line = new Float64Array(Math.max(cols, rows) + 1);
  // (A box along each row, then each column, the edge's past the edge.)
  const blur = (a: Float32Array) => {
    const R = SMOOTH_REACH;
    const out = new Float32Array(n);
    for (const [count, length, stride, step] of [
      [rows, cols, cols, 1],
      [cols, rows, 1, cols],
    ] as const) {
      const src = step === 1 ? a : out.slice();
      const dst = step === 1 ? out : a;
      for (let r = 0; r < count; r++) {
        const at = (i: number) => src[r * stride + Math.min(length - 1, Math.max(0, i)) * step];
        line[0] = 0;
        for (let i = 0; i < length + 2 * R; i++) line[i + 1] = line[i] + at(i - R);
        // (Summed from -R: line[i] holds the samples -R .. i - R - 1.)
        for (let i = 0; i < length; i++) dst[r * stride + i * step] = (line[i + 2 * R + 1] - line[i]) / (2 * R + 1);
      }
    }
    return a;
  };
  for (let p = 0; p < SMOOTH_PASSES; p++) {
    blur(sum);
    blur(weight);
  }
  const out = new Float32Array(n);
  for (let k = 0; k < n; k++) out[k] = weight[k] > 1e-6 ? sum[k] / weight[k] : land.h[k] / 100;
  return out;
}

/**
 * Gentler ground under the towns, the villages and the lots out on their own of `world`, fitted
 * into `land` (its dry heights changed in place; `areaOf`: each plot's area; not given, a plot by a box counts as its area's), as
 * a place's land yields to its town: within the look's `rise` of anything built its finer hills
 * fade out (`smoothHeights`), as the renderer's hills fade in from a town's edge; then each area's
 * box, and each lot out of every box, gets a plane (`gentler`); then the land beside water is
 * kept to its shore again (`shoreClamp`). What `withLand` does before it grades the town in.
 * `kept`: planes fitted before (`open.gentle`), each used as it is for the area whose box it was
 * fitted to (an area with none, a village put down or moved since, gets one fitted afresh). The
 * areas' planes used, in their order (a plane per area that has built ground in its box).
 */
export function gentleUnder(land: Land, world: FollowWorld, areaOf: readonly number[] = [], kept: readonly GentlePlane[] = []): GentlePlane[] {
  const { cols, rows } = land;
  const look = { ...DEFAULT_LANDSCAPE, ...(world.meta?.world as { landscape?: Partial<typeof DEFAULT_LANDSCAPE> } | undefined)?.landscape };
  const dist = new Protection(cols, rows, builtOf(world)).dist;
  const smooth = smoothHeights(land);
  const h = new Float32Array(cols * rows);
  const dry = (k: number) => land.wl[k] === NONE || land.wl[k] <= land.h[k];
  for (let k = 0; k < h.length; k++) {
    const own = land.h[k] / 100;
    h[k] = dry(k) ? smooth[k] + smoothstep(APRON, APRON + look.rise, dist[k]) * (own - smooth[k]) : own;
  }
  const planes = gentler(h, cols, rows, dist, world, world.meta?.areas ?? [], world.meta?.roads ?? [], areaOf, look.rise, dry, kept);
  shoreClamp(h, land.wl, cols, rows, look.slope ?? 0.5);
  for (let k = 0; k < h.length; k++) if (dry(k)) land.h[k] = Math.max(-32767, Math.min(32767, Math.round(h[k] * 100)));
  return planes;
}

/**
 * A town's or a village's gentle ground as `gentleUnder` fitted it: the area's box as the draft has it
 * (`open.areas`: what it is matched by), the box the plane covers (the area's, grown by its own
 * streets and the lots by it), and the plane, `a + bx (x - cx) + bz (z - cz)` m. Kept with a laid
 * town (`open.gentle`; not in its town file nor its code), so step 2's edits ease the land to the
 * planes it was laid with: a lot removed refits nothing, and the land moves only round it.
 */
export interface GentlePlane {
  area: { x: number; z: number; w: number; d: number };
  box: { x0: number; z0: number; x1: number; z1: number };
  a: number;
  bx: number;
  bz: number;
  cx: number;
  cz: number;
}

/**
 * The land (`h`, m per sample) eased under what is built towards gentle ground: each area's box (a
 * town or a village, with its own streets and lanes where they reach out past it), and each lot or backdrop building out of every box, gets a plane fitted to
 * the land under what is built in it (least squares, tilted no more than `GENTLE_TILT`), the land
 * on it that plane, and the land round it eased back to its own height over `reach` m (the look's
 * `rise`). Where two come near each other, their planes are blended by how near each is (in its
 * own box, almost wholly its own; near a town or a village, its plane before a lone lot's). The hills
 * and mountains rise round the towns, not under them. `dist`: each sample's distance from what is
 * built (`Protection.dist`: 0 on it). The water (`dry` false) is left as it is. `kept`: an area's
 * box and plane fitted before, used as they are (`gentleUnder`). The areas' planes used.
 */
function gentler(h: Float32Array, cols: number, rows: number, dist: Float32Array, world: FollowWorld, areas: readonly { x: number; z: number; w: number; d: number }[], roads: readonly RoadPlan[], areaOf: readonly number[], reach: number, dry: (k: number) => boolean, kept: readonly GentlePlane[]): GentlePlane[] {
  const c = LAND_CELL;
  type Box = { x0: number; z0: number; x1: number; z1: number };
  const grow = (box: Box, x0: number, z0: number, x1: number, z1: number) => Object.assign(box, { x0: Math.min(box.x0, x0), z0: Math.min(box.z0, z0), x1: Math.max(box.x1, x1), z1: Math.max(box.z1, z1) });
  const keptFor = areas.map((a) => kept.find((g) => g.area.x === a.x && g.area.z === a.z && g.area.w === a.w && g.area.d === a.d));
  // (An area's box, its own streets and lanes, which may reach out past it, and its own lots by them.)
  const boxes: Box[] = areas.map((a, n) => {
    const known = keptFor[n];
    if (known) return { ...known.box };
    const box = { x0: a.x, z0: a.z, x1: a.x + a.w, z1: a.z + a.d };
    for (const r of roads) {
      if ((r.area ?? -1) !== n || r.kind === 'country' || r.kind === 'trail') continue;
      for (const [x, z] of r.points) grow(box, x, z, x, z);
    }
    const near = { ...box };
    world.plots.forEach((p, i) => {
      if ((areaOf[i] ?? n) === n && p.x < near.x1 + AREA_LOTS && p.x + p.w > near.x0 - AREA_LOTS && p.z < near.z1 + AREA_LOTS && p.z + p.d > near.z0 - AREA_LOTS) grow(box, p.x, p.z, p.x + p.w, p.z + p.d);
    });
    return box;
  });
  const within = (b: Box, x: number, z: number) => x >= b.x0 && x <= b.x1 && z >= b.z0 && z <= b.z1;
  const alone: Box[] = [];
  const footprints = [
    ...world.plots.map((p) => ({ x0: p.x, z0: p.z, x1: p.x + p.w, z1: p.z + p.d })),
    ...(world.meta?.backdrop ?? []).map((b) => {
      const xs = b.poly.map((q) => q[0]);
      const zs = b.poly.map((q) => q[1]);
      return { x0: Math.min(...xs), z0: Math.min(...zs), x1: Math.max(...xs), z1: Math.max(...zs) };
    }),
  ];
  for (const f of footprints) if (!boxes.some((b) => within(b, (f.x0 + f.x1) / 2, (f.z0 + f.z1) / 2))) alone.push(f);
  // Each one's plane: through the land under what is built in it (in a box: every built sample; a lot alone: its own).
  const planes: { box: Box; a: number; bx: number; bz: number; cx: number; cz: number; town: boolean }[] = [];
  const used: GentlePlane[] = [];
  for (const [n, box] of [...boxes, ...alone].entries()) {
    const own = n >= boxes.length;
    const known = own ? undefined : keptFor[n];
    if (known) {
      planes.push({ box, a: known.a, bx: known.bx, bz: known.bz, cx: known.cx, cz: known.cz, town: true });
      used.push(known);
      continue;
    }
    const [i0, i1] = [Math.max(0, Math.ceil(box.x0 / c)), Math.min(cols - 1, Math.floor(box.x1 / c))];
    const [j0, j1] = [Math.max(0, Math.ceil(box.z0 / c)), Math.min(rows - 1, Math.floor(box.z1 / c))];
    const pts: [number, number, number][] = [];
    for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) if (own || dist[j * cols + i] === 0) pts.push([i * c, j * c, h[j * cols + i]]);
    if (!pts.length) continue;
    const cx = pts.reduce((t, p) => t + p[0], 0) / pts.length;
    const cz = pts.reduce((t, p) => t + p[1], 0) / pts.length;
    const mean = pts.reduce((t, p) => t + p[2], 0) / pts.length;
    let [sxx, sxz, szz, sxh, szh] = [0, 0, 0, 0, 0];
    for (const [x, z, v] of pts) {
      const [dx, dz, dv] = [x - cx, z - cz, v - mean];
      sxx += dx * dx;
      sxz += dx * dz;
      szz += dz * dz;
      sxh += dx * dv;
      szh += dz * dv;
    }
    const det = sxx * szz - sxz * sxz;
    let [bx, bz] = Math.abs(det) > 1e-9 ? [(sxh * szz - szh * sxz) / det, (szh * sxx - sxh * sxz) / det] : [sxx > 0 ? sxh / sxx : 0, szz > 0 ? szh / szz : 0];
    const tilt = Math.hypot(bx, bz);
    if (tilt > GENTLE_TILT) [bx, bz] = [(bx * GENTLE_TILT) / tilt, (bz * GENTLE_TILT) / tilt];
    // (Tilted less, the plane through the land's mean again.)
    planes.push({ box, a: mean, bx, bz, cx, cz, town: !own });
    if (!own) {
      const a = areas[n];
      used.push({ area: { x: a.x, z: a.z, w: a.w, d: a.d }, box: { ...box }, a: mean, bx, bz, cx, cz });
    }
  }
  // Each sample eased towards the planes near it: the nearest's weight, their heights weighted by each one's.
  const sum = new Float64Array(cols * rows);
  const weight = new Float64Array(cols * rows);
  const most = new Float32Array(cols * rows);
  for (const p of planes) {
    const [i0, i1] = [Math.max(0, Math.ceil((p.box.x0 - reach) / c)), Math.min(cols - 1, Math.floor((p.box.x1 + reach) / c))];
    const [j0, j1] = [Math.max(0, Math.ceil((p.box.z0 - reach) / c)), Math.min(rows - 1, Math.floor((p.box.z1 + reach) / c))];
    for (let j = j0; j <= j1; j++) {
      for (let i = i0; i <= i1; i++) {
        const [x, z] = [i * c, j * c];
        const off = Math.hypot(Math.max(p.box.x0 - x, 0, x - p.box.x1), Math.max(p.box.z0 - z, 0, z - p.box.z1));
        const w = 1 - smoothstep(0, reach, off);
        if (w <= 0) continue;
        const k = j * cols + i;
        // (Each plane's say in the blend falls off far faster than its pull: in its own box, its own; a town's or a
        // village's over a lone lot's near it.)
        const say = w ** 8 * (p.town ? TOWN_SAY : 1);
        sum[k] += say * (p.a + p.bx * (x - p.cx) + p.bz * (z - p.cz));
        weight[k] += say;
        if (w > most[k]) most[k] = w;
      }
    }
  }
  for (let k = 0; k < h.length; k++) if (weight[k] > 0 && dry(k)) h[k] += most[k] * (sum[k] / weight[k] - h[k]);
  return used;
}

/** The generator's heights (m) at (`x`, `z`), between its cells' middles (the edge's past the edge). */
function between(g: NonNullable<NonNullable<NeighbourhoodDraft['open']>['heights']>, x: number, z: number): number {
  const gx = Math.min(g.cols - 1, Math.max(0, x / g.cell - 0.5));
  const gz = Math.min(g.rows - 1, Math.max(0, z / g.cell - 0.5));
  const [x0, z0] = [Math.floor(gx), Math.floor(gz)];
  const [x1, z1] = [Math.min(g.cols - 1, x0 + 1), Math.min(g.rows - 1, z0 + 1)];
  const [u, v] = [gx - x0, gz - z0];
  const at = (c: number, r: number) => g.h[r * g.cols + c];
  return (at(x0, z0) * (1 - u) + at(x1, z0) * u) * (1 - v) + (at(x0, z1) * (1 - u) + at(x1, z1) * u) * v;
}

/** The points of a `gw` × `gd` grid (point (i, j) at (i × `step` + `offset`, j × `step` + `offset`) m) inside `polys` (even-odd over them all, as `landMasks.ts` `rasterize`). */
function scan(polys: readonly Pt[][], gw: number, gd: number, step: number, offset: number): Uint8Array {
  const mask = new Uint8Array(gw * gd);
  for (const poly of polys) {
    let [lo, hi] = [Infinity, -Infinity];
    for (const p of poly) [lo, hi] = [Math.min(lo, p[1]), Math.max(hi, p[1])];
    for (let j = Math.max(0, Math.ceil((lo - offset) / step)); j < Math.min(gd, Math.floor((hi - offset) / step) + 1); j++) {
      const zc = j * step + offset;
      const xs: number[] = [];
      for (let a = 0, b = poly.length - 1; a < poly.length; b = a++) {
        const [[xa, za], [xb, zb]] = [poly[a], poly[b]];
        if (za > zc !== zb > zc) xs.push(((xb - xa) * (zc - za)) / (zb - za) + xa);
      }
      xs.sort((p, q) => p - q);
      for (let k = 0; k + 1 < xs.length; k += 2) {
        for (let i = Math.max(0, Math.ceil((xs[k] - offset) / step)); i < Math.min(gw, Math.ceil((xs[k + 1] - offset) / step)); i++) mask[j * gw + i] ^= 1;
      }
    }
  }
  return mask;
}

/** Two chamfer passes over a `cols` × `rows` grid of `LAND_CELL` m samples: `relax(k, n, d)` for each sample `k` and the neighbour `n` before it in each pass, `d` m apart. */
function chamfer(cols: number, rows: number, relax: (k: number, n: number, d: number) => void): void {
  const [side, diag] = [LAND_CELL, LAND_CELL * Math.SQRT2];
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const k = j * cols + i;
      if (i > 0) relax(k, k - 1, side);
      if (j > 0) {
        relax(k, k - cols, side);
        if (i > 0) relax(k, k - cols - 1, diag);
        if (i < cols - 1) relax(k, k - cols + 1, diag);
      }
    }
  }
  for (let j = rows - 1; j >= 0; j--) {
    for (let i = cols - 1; i >= 0; i--) {
      const k = j * cols + i;
      if (i < cols - 1) relax(k, k + 1, side);
      if (j < rows - 1) {
        relax(k, k + cols, side);
        if (i < cols - 1) relax(k, k + cols + 1, diag);
        if (i > 0) relax(k, k + cols - 1, diag);
      }
    }
  }
}
