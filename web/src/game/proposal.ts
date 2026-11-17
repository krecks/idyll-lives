/**
 * Step 2's proposed layout (the `layout` job): the generator's town as the `town` job lays and settles
 * it, and the layout step 2 starts from, its streets with the land painted for what the town built
 * there (`usesOf`: its uses, "zones" to the player). Build (`build.ts`) makes the town from that layout
 * again, and an untouched layout gives the generator's town as it is (`Proposal.built`).
 *
 * Also what step 2 and Build read of a layout's uses: its frontage, stretch by stretch, by the use most
 * of the land 6 to 18 m behind the kerb is painted (`stretchesOf`); the greens far from any street that
 * become an outdoor spot (`greenSpots`); the cells zoned too far from a street to be built on, which
 * step 2 hatches (`hatched`); and how much a layout would build (`usesSummary`, step 2's live line).
 */

import { civicChoices, civicOf } from './backdrop';
import { inCentre } from './edit/ground';
import { noUseCells, roadReach, usesOfDraft } from './edit/uses';
import { cornersOf } from './macro';
import { inside, lengths, leftOf, pointAt, tangent, type Pt, type RoadPlan } from './roads';
import { plotRect, type Frontage, type NeighbourhoodDraft, type Rect, type Templates } from './town';
import type { LaidTown, Proposal } from './townJobs';
import { layoutHash, NO_USE, packUses, unpackUses, USE_CELL, USE_CODE, useOfCode, type UseKind, type UseLayer } from './uses';

/** How far behind the kerb a road's frontage is read for its use (m): the land a lot or a building fronting it stands on. */
export const FRONT_PROBES = [6, 10, 14, 18] as const;
/** Zoned cells further than this from a road (m) build nothing (step 2 hatches them), but a green that becomes an outdoor spot. */
export const BUILD_REACH = 36;
/** A green with no frontage this near an area's box (m) becomes an outdoor spot, a trail to it (`townCheck.ts` wants 150). */
export const SPOT_REACH = 150;
/** The least of a green with no frontage that holds an outdoor spot (cells: the smallest spot, 12 × 12 m, and a little round it). */
const SPOT_CELLS = 12;
/** Frontage is read every this many metres along a road. */
const STEP = 2;
/** A stretch shorter than this (m) goes with the one before it. */
const SHORTEST = 8;
/** Gaps of this many cells or fewer between cells of one use are closed (`usesOf`). */
const CLOSE = 2;

// ---- what a town built is painted as -----------------------------------------------------------------

/** Which use each backdrop building is (null: none, a farm's barn). */
function backdropUse(use: string, civic: boolean): UseKind | null {
  if (civic) return 'public';
  switch (use) {
    case 'house':
      return 'homes';
    case 'terrace':
    case 'flats':
      return 'rows';
    case 'shop':
    case 'cafe':
    case 'restaurant':
    case 'office':
    case 'workshop':
      return 'shops';
    case 'church':
    case 'school':
    case 'hall':
      return 'public';
    default:
      return null;
  }
}

/** The order uses are painted in, the later over the earlier where two meet. */
const PAINT_ORDER: readonly UseKind[] = ['green', 'homes', 'rows', 'shops', 'public'];

/**
 * The uses a town built shows (`draft`, a big world's): its homes (house lots, the backdrop's houses),
 * its terraces and flats (terraced lots, the backdrop's terraces and flats), its shops and work (shop,
 * café and market lots, the backdrop's shops, offices and workshops), its parks and greens (parks,
 * playgrounds, outdoor spots and the greens the roads go round) and its public places (the civic places
 * and the backdrop's halls, churches and schools), each painted over the cells whose middles it covers;
 * farms none. Gaps of `CLOSE` cells or fewer between cells of one use are closed, the cells no use goes
 * on cleared (`noUseCells`: under roads and water), and so are those too far from a road to build on
 * (`BUILD_REACH`), but the greens that are outdoor spots (`greenSpots`).
 */
export function usesOf(draft: NeighbourhoodDraft, t: Templates): UseLayer {
  const { cols, rows } = usesOfDraft(draft);
  const codes = new Uint8Array(cols * rows);
  const open = draft.open;
  const places = open?.places ?? [];
  const shapes: { kind: UseKind; rect?: Rect; poly?: readonly Pt[] }[] = [];
  for (const g of draft.greens) shapes.push({ kind: 'green', poly: g });
  for (const s of draft.slots) {
    const kind: UseKind | null = s.kind === 'house' ? (s.lot === 'farm' ? null : s.lot === 'terrace' ? 'rows' : 'homes') : s.kind === 'shop' || s.template === 'market' ? 'shops' : 'green';
    if (kind) shapes.push({ kind, rect: plotRect(t, s) });
  }
  for (const b of open?.backdrop ?? []) {
    const kind = backdropUse(b.use, b.place !== undefined && civicOf(places[b.place]) >= 0);
    if (kind) shapes.push({ kind, poly: b.poly });
  }
  for (const kind of PAINT_ORDER) {
    const code = USE_CODE[kind];
    for (const sh of shapes) {
      if (sh.kind !== kind) continue;
      if (sh.rect) paintRect(codes, cols, rows, sh.rect, code);
      else paintPoly(codes, cols, rows, sh.poly!, code);
    }
  }
  closeGaps(codes, cols, rows);
  const none = noUseCells(draft);
  for (let k = 0; k < codes.length; k++) if (none[k]) codes[k] = NO_USE;
  // (Too far from a road to build on: none, but a green that is an outdoor spot.)
  const reach = roadReach(draft);
  const spots = new Uint8Array(codes.length);
  for (const spot of spotsOf(codes, reach, cols, rows, areasOf(draft))) for (const k of spot.cells) spots[k] = 1;
  for (let k = 0; k < codes.length; k++) if (codes[k] && !spots[k] && !(reach[k] <= BUILD_REACH)) codes[k] = NO_USE;
  return packUses(codes, cols, rows);
}

/** Paints the cells whose middles are in `r`, or the cell under its middle if none is. */
function paintRect(codes: Uint8Array, cols: number, rows: number, r: Rect, code: number): void {
  const [i0, i1] = [Math.max(0, Math.ceil(r.x / USE_CELL - 0.5)), Math.min(cols - 1, Math.floor((r.x + r.w) / USE_CELL - 0.5))];
  const [j0, j1] = [Math.max(0, Math.ceil(r.z / USE_CELL - 0.5)), Math.min(rows - 1, Math.floor((r.z + r.d) / USE_CELL - 0.5))];
  let any = false;
  for (let j = j0; j <= j1; j++) {
    for (let i = i0; i <= i1; i++) {
      codes[j * cols + i] = code;
      any = true;
    }
  }
  const [ci, cj] = [Math.floor((r.x + r.w / 2) / USE_CELL), Math.floor((r.z + r.d / 2) / USE_CELL)];
  if (!any && ci >= 0 && cj >= 0 && ci < cols && cj < rows) codes[cj * cols + ci] = code;
}

/** Paints the cells whose middles are in `poly`, or the cell under its middle if none is. */
function paintPoly(codes: Uint8Array, cols: number, rows: number, poly: readonly Pt[], code: number): void {
  const xs = poly.map((p) => p[0]);
  const zs = poly.map((p) => p[1]);
  const [i0, i1] = [Math.max(0, Math.floor(Math.min(...xs) / USE_CELL)), Math.min(cols - 1, Math.floor(Math.max(...xs) / USE_CELL))];
  const [j0, j1] = [Math.max(0, Math.floor(Math.min(...zs) / USE_CELL)), Math.min(rows - 1, Math.floor(Math.max(...zs) / USE_CELL))];
  let any = false;
  for (let j = j0; j <= j1; j++) {
    for (let i = i0; i <= i1; i++) {
      if (!inside(poly as Pt[], (i + 0.5) * USE_CELL, (j + 0.5) * USE_CELL)) continue;
      codes[j * cols + i] = code;
      any = true;
    }
  }
  const [cx, cz] = [xs.reduce((a, b) => a + b, 0) / xs.length, zs.reduce((a, b) => a + b, 0) / zs.length];
  const [ci, cj] = [Math.floor(cx / USE_CELL), Math.floor(cz / USE_CELL)];
  if (!any && ci >= 0 && cj >= 0 && ci < cols && cj < rows) codes[cj * cols + ci] = code;
}

/** Closes runs of `CLOSE` empty cells or fewer between two cells of one use, across and down. */
function closeGaps(codes: Uint8Array, cols: number, rows: number): void {
  const line = (count: number, at: (n: number) => number) => {
    let last = -1;
    for (let n = 0; n < count; n++) {
      const c = codes[at(n)];
      if (!c) continue;
      if (last >= 0 && n - last - 1 > 0 && n - last - 1 <= CLOSE && codes[at(last)] === c) for (let m = last + 1; m < n; m++) codes[at(m)] = c;
      last = n;
    }
  };
  for (let j = 0; j < rows; j++) line(cols, (i) => j * cols + i);
  for (let i = 0; i < cols; i++) line(rows, (j) => j * cols + i);
}

// ---- the proposal ------------------------------------------------------------------------------------

/** `draft` with uses `layer` (`open.uses`). */
function withLayer(draft: NeighbourhoodDraft, layer: UseLayer): NeighbourhoodDraft {
  return draft.open ? { ...draft, open: { ...draft.open, uses: layer } } : draft;
}

/**
 * The proposal of a town the `town` job laid and settled (`town`: its `LaidTown`): the town built
 * (`built`: its draft with the uses it shows, `usesOf`; its own plan, `builtPlan`, and `marks` as
 * `builtMarks`), and the layout step 2 starts from (`plan`: the town's plan, its land as laid, with the
 * same roads, greens, areas, centre and squares, and the uses, but no lots, households, backdrop,
 * places, pads or parking), with the layout's `layoutHash`.
 */
export function proposalOf(town: Pick<LaidTown, 'draft' | 'plan' | 'marks'>, t: Templates): Proposal {
  const layer = usesOf(town.draft, t);
  const built = withLayer(town.draft, layer);
  const from = town.plan ?? town.draft;
  const roadsFailed = from.diagnostics?.roads;
  const plan: NeighbourhoodDraft = {
    ...from,
    slots: [],
    households: [],
    ...(from.open ? { open: { ...from.open, backdrop: [], places: [], parking: [], pads: {}, uses: layer } } : {}),
    diagnostics: { dropped: [], ...(roadsFailed ? { roads: roadsFailed } : {}) },
  };
  return { plan, built, hash: layoutHash(plan), ...(town.plan ? { builtPlan: town.plan, builtMarks: town.marks ?? [] } : {}) };
}

// ---- frontage, stretch by stretch --------------------------------------------------------------------

/** A stretch of a road's frontage on one side painted one use: the road, its side, from `s0` to `s1` m along it, the use, whether any of its land is `dirty`, and its middle. */
export interface Stretch {
  road: number;
  side: 1 | -1;
  s0: number;
  s1: number;
  kind: UseKind;
  dirty: boolean;
  mid: Pt;
}

/** Per road and side lots may front, where its frontage is read: how far along at each read (m), the point, and the cells `FRONT_PROBES` behind the kerb there (-1: off the grid). */
interface Reads {
  road: number;
  side: 1 | -1;
  s: Float32Array;
  p: Float32Array;
  cells: Int32Array;
}
const readsOf = new WeakMap<readonly RoadPlan[], { cols: number; rows: number; reads: Reads[] }>();

/** Whether lots may front road `r` at all: not a footpath, nor a bridge. */
export const fronted = (r: RoadPlan): boolean => r.kind !== 'trail' && !r.bridge;

/** `roads`' frontage reads (`Reads`) on a grid of `cols` × `rows` cells, made once per set of roads. */
function frontageReads(roads: readonly RoadPlan[], cols: number, rows: number): Reads[] {
  const had = readsOf.get(roads);
  if (had && had.cols === cols && had.rows === rows) return had.reads;
  const reads: Reads[] = [];
  const P = FRONT_PROBES.length;
  roads.forEach((r, road) => {
    if (!fronted(r) || r.points.length < 2) return;
    const ls = lengths(r.points);
    const L = ls[ls.length - 1];
    const n = Math.floor(L / STEP);
    if (n < 1) return;
    for (const side of r.lots ? [r.lots] : ([1, -1] as const)) {
      const s = new Float32Array(n);
      const p = new Float32Array(n * 2);
      const cells = new Int32Array(n * P);
      let i = 0;
      for (let k = 0; k < n; k++) {
        const at = (k + 0.5) * STEP;
        while (i < ls.length - 2 && ls[i + 1] < at) i++;
        const f = Math.max(0, Math.min(1, (at - ls[i]) / (ls[i + 1] - ls[i] || 1)));
        const [a, b] = [r.points[i], r.points[i + 1]];
        const [x, z] = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
        const [nx, nz] = leftOf(tangent(r.points, f < 0.5 ? i : i + 1)).map((v) => v * side);
        s[k] = at;
        p[k * 2] = x;
        p[k * 2 + 1] = z;
        FRONT_PROBES.forEach((d, q) => {
          const back = r.width / 2 + d;
          const [ci, cj] = [Math.floor((x + nx * back) / USE_CELL), Math.floor((z + nz * back) / USE_CELL)];
          cells[k * P + q] = ci >= 0 && cj >= 0 && ci < cols && cj < rows ? cj * cols + ci : -1;
        });
      }
      reads.push({ road, side, s, p, cells });
    }
  });
  readsOf.set(roads, { cols, rows, reads });
  return reads;
}

/**
 * `draft`'s frontage (every road lots may front, each side they may), stretch by stretch: at each
 * read along it, the use most of the land `FRONT_PROBES` behind the kerb is painted (two of the four
 * at least; none else), in runs of one use, a run shorter than `SHORTEST` going with the one before it.
 * `dirty`: a cell each, set where the land is to be built again (a run is dirty if any of its land is,
 * and runs are split where that changes); `fresh`: roads all of whose frontage is dirty (new ones).
 * The stretches painted a use (unpainted frontage has none).
 */
export function stretchesOf(draft: NeighbourhoodDraft, { dirty, fresh }: { dirty?: Uint8Array; fresh?: ReadonlySet<number> } = {}): Stretch[] {
  const layer = usesOfDraft(draft);
  const { cols, rows } = layer;
  const codes = unpackUses(layer);
  const P = FRONT_PROBES.length;
  const out: Stretch[] = [];
  const votes = new Uint8Array(8);
  for (const rd of frontageReads(draft.roads, cols, rows)) {
    const n = rd.s.length;
    const label = new Int32Array(n);
    const all = fresh?.has(rd.road) ?? false;
    for (let k = 0; k < n; k++) {
      votes.fill(0);
      let wet = all;
      for (let q = 0; q < P; q++) {
        const c = rd.cells[k * P + q];
        if (c < 0) continue;
        votes[codes[c]]++;
        if (dirty && dirty[c]) wet = true;
      }
      // (The most painted use, two reads at least; of two as many, the nearer the road.)
      let [best, most] = [0, 1];
      for (let q = 0; q < P; q++) {
        const c = rd.cells[k * P + q];
        const code = c < 0 ? 0 : codes[c];
        if (code && votes[code] > most) [best, most] = [code, votes[code]];
      }
      label[k] = best * 2 + (wet ? 1 : 0);
    }
    // Runs of one label; a short one goes with the one before it (the first, with the one after).
    const runs: { label: number; k0: number; k1: number }[] = [];
    for (let k = 0; k < n; k++) {
      const last = runs[runs.length - 1];
      if (last && last.label === label[k]) last.k1 = k;
      else runs.push({ label: label[k], k0: k, k1: k });
    }
    const merged: typeof runs = [];
    for (const [i, run] of runs.entries()) {
      const short = (run.k1 - run.k0 + 1) * STEP < SHORTEST;
      const prev = merged[merged.length - 1];
      if (short && prev) prev.k1 = run.k1;
      else if (short && runs[i + 1]) runs[i + 1].k0 = run.k0;
      else if (prev && prev.label === run.label) prev.k1 = run.k1;
      else merged.push({ ...run });
    }
    for (const run of merged) {
      const kind = useOfCode(run.label >> 1);
      if (!kind) continue;
      const m = (run.k0 + run.k1) >> 1;
      out.push({ road: rd.road, side: rd.side, s0: rd.s[run.k0] - STEP / 2, s1: rd.s[run.k1] + STEP / 2, kind, dirty: (run.label & 1) === 1, mid: [rd.p[m * 2], rd.p[m * 2 + 1]] });
    }
  }
  return out;
}

// ---- greens with no frontage: outdoor spots --------------------------------------------------------

/** A green with no frontage that becomes an outdoor spot: its cells, its middle and the area it is nearest. */
export interface GreenSpot {
  cells: number[];
  centre: Pt;
  area: number;
}

/** `draft`'s areas' boxes. */
function areasOf(draft: NeighbourhoodDraft): Rect[] {
  return (draft.open?.areas ?? []).map((a) => ({ x: a.x, z: a.z, w: a.w, d: a.d }));
}

const offRect = (r: Rect, [x, z]: Pt) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));

/** The greens of `codes` with no frontage (every cell further than `BUILD_REACH` from a road) of `SPOT_CELLS` or more, their middles within `SPOT_REACH` of an area's box. */
function spotsOf(codes: Uint8Array, reach: Float32Array, cols: number, rows: number, areas: readonly Rect[]): GreenSpot[] {
  const green = USE_CODE.green;
  const seen = new Uint8Array(codes.length);
  const out: GreenSpot[] = [];
  for (let k0 = 0; k0 < codes.length; k0++) {
    if (seen[k0] || codes[k0] !== green) continue;
    const cells: number[] = [];
    let near = false;
    const stack = [k0];
    seen[k0] = 1;
    while (stack.length) {
      const k = stack.pop()!;
      cells.push(k);
      if (reach[k] <= BUILD_REACH) near = true;
      const [i, j] = [k % cols, Math.floor(k / cols)];
      for (const [di, dj] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) {
        const [a, b] = [i + di, j + dj];
        if (a < 0 || b < 0 || a >= cols || b >= rows) continue;
        const n = b * cols + a;
        if (seen[n] || codes[n] !== green) continue;
        seen[n] = 1;
        stack.push(n);
      }
    }
    if (near || cells.length < SPOT_CELLS || !areas.length) continue;
    const centre: Pt = [0, 0];
    for (const k of cells) [centre[0], centre[1]] = [centre[0] + ((k % cols) + 0.5) * USE_CELL, centre[1] + (Math.floor(k / cols) + 0.5) * USE_CELL];
    [centre[0], centre[1]] = [centre[0] / cells.length, centre[1] / cells.length];
    let [area, far] = [0, Infinity];
    areas.forEach((a, i) => {
      const d = offRect(a, centre);
      if (d < far) [area, far] = [i, d];
    });
    if (far <= SPOT_REACH) out.push({ cells, centre, area });
  }
  return out;
}

/** The greens of `draft`'s uses with no frontage that become outdoor spots at Build (see `spotsOf`). */
export function greenSpots(draft: NeighbourhoodDraft): GreenSpot[] {
  const layer = usesOfDraft(draft);
  return spotsOf(unpackUses(layer), roadReach(draft), layer.cols, layer.rows, areasOf(draft));
}

/**
 * The cells of `draft`'s uses zoned too far from a road to be built on (`BUILD_REACH`; but the greens
 * that become outdoor spots, `greenSpots`): a byte a cell of its layer, row-major, set where step 2
 * draws the zone hatched (`Renderer.setTownUses`).
 */
export function hatched(draft: NeighbourhoodDraft): Uint8Array {
  const layer = usesOfDraft(draft);
  const codes = unpackUses(layer);
  const reach = roadReach(draft);
  const out = new Uint8Array(codes.length);
  for (let k = 0; k < codes.length; k++) if (codes[k] && !(reach[k] <= BUILD_REACH)) out[k] = 1;
  for (const spot of spotsOf(codes, reach, layer.cols, layer.rows, areasOf(draft))) for (const k of spot.cells) out[k] = 0;
  return out;
}

// ---- how much a layout builds ------------------------------------------------------------------------

/** What a layout would build, roughly (`usesSummary`): homes, shops and workplaces, parks (and outdoor spots), public places. */
export interface UsesSummary {
  homes: number;
  shops: number;
  parks: number;
  public: number;
}

/**
 * How much frontage each thing takes, as Build fills a stretch (measured over the places' worlds, their
 * proposals built again and streets painted whole): a home along homes, a terraced home along terraces
 * and flats (m of frontage a lot may front: off the corners and bends, a home's out of a town centre);
 * the first shop or workplace (with the gaps either side) and each after it; the first park and each
 * after it (a green stretch); the gap between two civic places.
 */
const HOME_FRONT = 23;
const ROW_HOME_FRONT = 20;
const FIRST_SHOP = 20;
const SHOP_FRONT = 32;
const FIRST_PARK = 18;
const PARK_FRONT = 120;
const CIVIC_GAP = 10;
/** A civic place is counted where its stretch has this many times its width of public frontage left (Build seldom fits one snugger). */
const CIVIC_ROOM = 1.3;

/** Each set of roads' lot rule (`macro.ts` `cornersOf`: off the corners and tight bends), made once. */
const cornersFor = new WeakMap<readonly RoadPlan[], (f: Frontage) => boolean>();

/**
 * How much `draft`'s layout would build, from its frontage (`stretchesOf`) and its greens with no
 * frontage (`greenSpots`): homes along its homes and its terraces and flats (by the frontage a lot may
 * front), shops and workplaces along its shops and work, parks and playgrounds along its greens and an
 * outdoor spot on each green spot, and as many of the civic places each town and village may have
 * (`civicChoices`, the most needed first) as fit along its public places. For step 2's live line
 * ("Homes ≈ 84 · Shops ≈ 12 · Parks 3 · Public 5"); within about a quarter of what Build makes.
 */
export function usesSummary(draft: NeighbourhoodDraft): UsesSummary {
  const { roads } = draft;
  let corners = cornersFor.get(roads);
  if (!corners) cornersFor.set(roads, (corners = cornersOf(roads, roads.flatMap((r, i) => (fronted(r) ? [i] : [])))));
  const areas = areasOf(draft);
  const nearest = (p: Pt) => areas.reduce((k, a, i) => (offRect(a, p) < offRect(areas[k], p) ? i : k), 0);
  let [homes, shops, parks] = [0, 0, 0];
  const civic = new Map<number, number[]>();
  for (const st of stretchesOf(draft)) {
    const len = Math.max(0, st.s1 - st.s0);
    const r = roads[st.road];
    if (st.kind === 'homes' || st.kind === 'rows' || st.kind === 'green') {
      // (The frontage a lot may front, read every 2 m.)
      const ls = lengths(r.points);
      let usable = 0;
      for (let s = st.s0 + 1; s < st.s1; s += STEP) {
        const { p, t } = pointAt(r.points, s, ls);
        const n = leftOf(t).map((v) => v * st.side) as Pt;
        if (!corners({ road: st.road, s, side: st.side, p, n, far: 0 })) continue;
        if (st.kind === 'homes' && inCentre(draft, [p[0] + n[0] * (r.width / 2 + 10), p[1] + n[1] * (r.width / 2 + 10)])) continue;
        usable += STEP;
      }
      if (st.kind === 'homes') homes += usable / HOME_FRONT;
      else if (st.kind === 'rows') homes += usable / ROW_HOME_FRONT;
      else if (usable >= FIRST_PARK) parks += 1 + (usable - FIRST_PARK) / PARK_FRONT;
    } else if (st.kind === 'shops') {
      if (len >= FIRST_SHOP) shops += 1 + (len - FIRST_SHOP) / SHOP_FRONT;
    } else {
      const a = r.area ?? nearest(st.mid);
      civic.set(a, [...(civic.get(a) ?? []), len]);
    }
  }
  parks += greenSpots(draft).length;
  // The civic places each town and village lacks, the most needed first, each as wide as its kind is on the whole, packed along its public frontage.
  let publics = 0;
  for (const [a, stretches] of civic) {
    const left = [...stretches];
    for (const c of civicChoices(draft, a)) {
      const w = (c.w[0] + c.w[1]) / 2;
      const at = left.findIndex((l) => l >= CIVIC_ROOM * w);
      if (at < 0) continue;
      left[at] -= w + CIVIC_GAP;
      publics++;
    }
  }
  return { homes: Math.round(homes), shops: Math.round(shops), parks: Math.round(parks), public: publics };
}
