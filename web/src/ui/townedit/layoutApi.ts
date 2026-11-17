/**
 * What step 2 (Streets & zones) asks of the layout's own edits and figures, in one place: painting
 * zones (`paintUses`), the live summary (`usesSummary`), the zoned cells too far from a street to be
 * built on (`hatchedUses`), the Village tool's village as streets and zones (`villageAsLayout`), and
 * the panel's Clear all and Clear zones.
 *
 * The generation side (`game/edit/uses.ts`, `game/build.ts`) owns how a layout becomes a town; until
 * it is in, the ones here stand in for it with the same shapes, so the screens and tools read only
 * this file and taking up the real ones changes nothing else.
 */

import type { Edited } from '../../game/edit/edited';
import { removeLots } from '../../game/edit/lots';
import { unrle } from '../../game/landMasks';
import type { RoadPlan } from '../../core/protocol';
import { lengths, pointAt } from '../../game/roads';
import { plotRect, type NeighbourhoodDraft, type PlotSlot, type Templates } from '../../game/town';
import { packUses, unpackUses, USE_CELL, USE_CODE, useOfCode, usesFor, type UseKind, type UseLayer } from '../../game/uses';

/** One dab of the zone brush: a disc of radius `r` m round town point (`x`, `z`). */
export interface UseStamp {
  x: number;
  z: number;
  r: number;
}

/** What a layout's zones are about to build, as step 2's summary line says it: homes and shops about, parks and public places counted. */
export interface UsesSummary {
  homes: number;
  shops: number;
  parks: number;
  public: number;
}

/** How far from a road's edge (m) a zoned cell still builds (beyond it, step 2 hatches it). */
export const BUILD_REACH = 36;
/** "Beside streets only": the brush paints only this near a road's edge (m). */
export const BESIDE_STREETS = 40;
/** How far from the water's edge (m) the brush keeps off. */
const SHORE_GAP = 4;
/** The road distances worked out this far (m); every cell further is at least this far. */
const FAR = 48;

/** Nothing cleared: a zones edit takes nothing out of the way. */
const NOTHING = { lots: [] as number[], backdrop: [] as number[] };

/** The layer a draft paints on: its own, else an empty one for its world. */
export function layerOf(draft: NeighbourhoodDraft): UseLayer {
  return draft.open?.uses ?? usesFor(draft);
}

/** `draft` with `layer` as its uses (the draft itself when the cells are the same). */
function withUses(draft: NeighbourhoodDraft, layer: UseLayer): NeighbourhoodDraft {
  if (!draft.open) return draft;
  return { ...draft, open: { ...draft.open, uses: layer } };
}

// ---- where the roads and the water are, per cell ---------------------------------------------------

const roadDistances = new WeakMap<readonly RoadPlan[], { cols: number; rows: number; d: Float32Array }>();

/**
 * Each use cell's distance (m) from its middle to the nearest road's edge (0 on a road), up to `FAR`;
 * trails don't count. Kept per roads array (an edit that leaves the roads keeps them).
 */
export function roadDistance(draft: NeighbourhoodDraft): Float32Array {
  const { cols, rows } = layerOf(draft);
  const had = roadDistances.get(draft.roads);
  if (had && had.cols === cols && had.rows === rows) return had.d;
  const d = new Float32Array(cols * rows).fill(FAR);
  for (const r of draft.roads) {
    if (r.kind === 'trail' || r.points.length < 2) continue;
    const ls = lengths(r.points);
    const total = ls[ls.length - 1];
    const half = r.width / 2;
    const reach = Math.ceil((FAR + half) / USE_CELL);
    for (let s = 0; s <= total + 1e-6; s += 2) {
      const [px, pz] = pointAt(r.points, Math.min(s, total), ls).p;
      const [ci, cj] = [Math.floor(px / USE_CELL), Math.floor(pz / USE_CELL)];
      for (let j = Math.max(0, cj - reach); j <= Math.min(rows - 1, cj + reach); j++) {
        for (let i = Math.max(0, ci - reach); i <= Math.min(cols - 1, ci + reach); i++) {
          const dist = Math.max(0, Math.hypot((i + 0.5) * USE_CELL - px, (j + 0.5) * USE_CELL - pz) - half);
          const k = j * cols + i;
          if (dist < d[k]) d[k] = dist;
        }
      }
    }
  }
  roadDistances.set(draft.roads, { cols, rows, d });
  return d;
}

const wetCells = new WeakMap<object, Uint8Array>();

/** The use cells the brush keeps off for the water: those with water within `SHORE_GAP` m of their middle. */
function wet(draft: NeighbourhoodDraft): Uint8Array | null {
  const water = draft.open?.water;
  if (!water) return null;
  const had = wetCells.get(water);
  if (had) return had;
  const { cols, rows } = layerOf(draft);
  const [W, D] = [draft.width, draft.depth];
  const tiles = unrle(water, W * D);
  const out = new Uint8Array(cols * rows);
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const [x0, z0] = [Math.max(0, i * USE_CELL - SHORE_GAP), Math.max(0, j * USE_CELL - SHORE_GAP)];
      const [x1, z1] = [Math.min(W - 1, (i + 1) * USE_CELL + SHORE_GAP), Math.min(D - 1, (j + 1) * USE_CELL + SHORE_GAP)];
      // (Every other metre is enough to find water nearer than the gap.)
      search: for (let z = z0; z <= z1; z += 2) for (let x = x0; x <= x1; x += 2) if (tiles[z * W + x]) {
        out[j * cols + i] = 1;
        break search;
      }
    }
  }
  wetCells.set(water, out);
  return out;
}

// ---- painting --------------------------------------------------------------------------------------

/**
 * The zone brush's stroke (`stamps`) painted with `kind` (null: erased) on `draft`: never on the water
 * (nor its shore), a road or off the world. A pure edit (no land to settle under it): the same draft
 * when no cell changed.
 */
export function paintUses(draft: NeighbourhoodDraft, stamps: readonly UseStamp[], kind: UseKind | null): Edited {
  if (!draft.open) return { refused: 'this town has no land to zone' };
  const layer = layerOf(draft);
  const { cols, rows } = layer;
  const before = unpackUses(layer);
  const codes = before.slice();
  const code = kind ? USE_CODE[kind] : 0;
  const roads = roadDistance(draft);
  const water = wet(draft);
  let changed = false;
  for (const s of stamps) {
    const [i0, i1] = [Math.max(0, Math.floor((s.x - s.r) / USE_CELL)), Math.min(cols - 1, Math.floor((s.x + s.r) / USE_CELL))];
    const [j0, j1] = [Math.max(0, Math.floor((s.z - s.r) / USE_CELL)), Math.min(rows - 1, Math.floor((s.z + s.r) / USE_CELL))];
    for (let j = j0; j <= j1; j++) {
      for (let i = i0; i <= i1; i++) {
        const [cx, cz] = [(i + 0.5) * USE_CELL, (j + 0.5) * USE_CELL];
        if (Math.hypot(cx - s.x, cz - s.z) > s.r || cx > draft.width || cz > draft.depth) continue;
        const k = j * cols + i;
        if (code && (roads[k] <= 0 || water?.[k])) continue;
        if (codes[k] !== code) [codes[k], changed] = [code, true];
      }
    }
  }
  if (!changed) return { draft, cleared: NOTHING };
  return { draft: withUses(draft, packUses(codes, cols, rows)), cleared: NOTHING };
}

/**
 * `edited` (a paint over `before`) kept to cells within `BESIDE_STREETS` m of a road (the card's
 * "Beside streets only"): what it painted further out put back as it was. Erasing goes anywhere.
 */
export function besideStreets(before: NeighbourhoodDraft, edited: Edited): Edited {
  if (!('draft' in edited) || edited.draft === before || !edited.draft.open?.uses) return edited;
  const was = unpackUses(layerOf(before));
  const now = unpackUses(edited.draft.open.uses);
  if (was.length !== now.length) return edited;
  const d = roadDistance(before);
  let codes: Uint8Array | null = null;
  for (let k = 0; k < now.length; k++) {
    if (now[k] === was[k] || now[k] === 0 || d[k] <= BESIDE_STREETS) continue;
    codes ??= now.slice();
    codes[k] = was[k];
  }
  if (!codes) return edited;
  if (codes.every((c, k) => c === was[k])) return { draft: before, cleared: NOTHING };
  const { cols, rows } = edited.draft.open.uses;
  return { ...edited, draft: withUses(edited.draft, packUses(codes, cols, rows)) };
}

// ---- what the layout would build ---------------------------------------------------------------------

/** A cell the zones have that is further than `BUILD_REACH` m from a road (nothing is built there): a byte each, as the overlay takes them. Null: none. */
export function hatchedUses(draft: NeighbourhoodDraft): Uint8Array | null {
  const layer = draft.open?.uses;
  if (!layer) return null;
  const codes = unpackUses(layer);
  const d = roadDistance(draft);
  let out: Uint8Array | null = null;
  for (let k = 0; k < codes.length; k++) {
    if (!codes[k] || d[k] <= BUILD_REACH) continue;
    out ??= new Uint8Array(codes.length);
    out[k] = 1;
  }
  return out;
}

/** The frontage (m of kerb) a home, a terraced home and a shop take about, along their road. */
const FRONT = { homes: 16, rows: 7, shops: 18 } as const;
/** How far behind the kerb (m) the frontage is read. */
const BEHIND = [6, 12, 18] as const;

/**
 * What `draft`'s zones would build, about (step 2's summary): homes and shops from the zoned frontage
 * along the roads (both sides, the majority use 6–18 m behind the kerb), parks and public places as
 * the patches of them that front a road.
 */
export function usesSummary(draft: NeighbourhoodDraft): UsesSummary {
  const layer = draft.open?.uses;
  const out: UsesSummary = { homes: 0, shops: 0, parks: 0, public: 0 };
  if (!layer) return out;
  const codes = unpackUses(layer);
  const { cols, rows } = layer;
  const at = (x: number, z: number) => {
    const [i, j] = [Math.floor(x / USE_CELL), Math.floor(z / USE_CELL)];
    return i < 0 || j < 0 || i >= cols || j >= rows ? 0 : codes[j * cols + i];
  };
  const front = { homes: 0, rows: 0, shops: 0 };
  for (const r of draft.roads) {
    if (r.kind === 'trail' || r.points.length < 2) continue;
    const ls = lengths(r.points);
    const total = ls[ls.length - 1];
    for (let s = USE_CELL / 2; s < total; s += USE_CELL) {
      const { p, t } = pointAt(r.points, s, ls);
      for (const side of [1, -1]) {
        const tally = new Map<number, number>();
        for (const b of BEHIND) {
          const off = r.width / 2 + b;
          const c = at(p[0] - t[1] * off * side, p[1] + t[0] * off * side);
          if (c) tally.set(c, (tally.get(c) ?? 0) + 1);
        }
        let best = 0;
        let most = 0;
        for (const [c, n] of tally) if (n > most) [best, most] = [c, n];
        const use = useOfCode(best);
        if (use === 'homes' || use === 'rows' || use === 'shops') front[use] += USE_CELL;
      }
    }
  }
  out.homes = Math.round(front.homes / FRONT.homes + front.rows / FRONT.rows);
  out.shops = Math.round(front.shops / FRONT.shops);
  const d = roadDistance(draft);
  out.parks = patches(codes, cols, rows, USE_CODE.green, d);
  out.public = patches(codes, cols, rows, USE_CODE.public, d);
  return out;
}

/** The patches of cells of `code` (joined side by side) with a cell within building reach of a road. */
function patches(codes: Uint8Array, cols: number, rows: number, code: number, d: Float32Array): number {
  const seen = new Uint8Array(codes.length);
  let n = 0;
  const queue: number[] = [];
  for (let k0 = 0; k0 < codes.length; k0++) {
    if (codes[k0] !== code || seen[k0]) continue;
    let fronts = false;
    seen[k0] = 1;
    queue.length = 0;
    queue.push(k0);
    while (queue.length) {
      const k = queue.pop()!;
      if (d[k] <= BUILD_REACH) fronts = true;
      const [i, j] = [k % cols, Math.floor(k / cols)];
      for (const [di, dj] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) {
        const [a, b] = [i + di, j + dj];
        if (a < 0 || b < 0 || a >= cols || b >= rows) continue;
        const q = b * cols + a;
        if (!seen[q] && codes[q] === code) {
          seen[q] = 1;
          queue.push(q);
        }
      }
    }
    if (fronts) n++;
  }
  return n;
}

/** The summary in a line: "Homes ≈ 84 · Shops ≈ 12 · Parks 3 · Public 5". */
export function summaryLine(s: UsesSummary): string {
  return `Homes ≈ ${s.homes} · Shops ≈ ${s.shops} · Parks ${s.parks} · Public ${s.public}`;
}

// ---- the panel's and the Village tool's edits --------------------------------------------------------

/** The use a lot of the generator's stands for (`proposal.ts` maps them the same way); null: none (a farm). */
function useOfLot(s: PlotSlot): UseKind | null {
  if (s.kind === 'house') return s.lot === 'farm' ? null : s.lot === 'terrace' ? 'rows' : 'homes';
  if (s.kind === 'shop') return 'shops';
  return 'green';
}

/**
 * The Village tool's village in step 2 (`placed`, a `placeVillage` on `before`): its streets kept, its
 * lots and buildings taken off and painted as the zones they stood for. A refusal as it was.
 */
export function villageAsLayout(before: NeighbourhoodDraft, placed: Edited, t: Templates): Edited {
  if (!('draft' in placed) || !placed.draft.open) return placed;
  const draft = placed.draft;
  const area = draft.open!.areas.length - 1;
  if (area <= 0 || area < (before.open?.areas.length ?? 0)) return placed;
  const lots = new Set(draft.slots.filter((s) => (s.area ?? 0) === area).map((s) => s.index));
  const backdrop = new Set((draft.open!.backdrop ?? []).flatMap((b, i) => (b.area === area ? [i] : [])));
  const layer = layerOf(draft);
  const codes = unpackUses(layer).slice();
  for (const i of lots) {
    const use = useOfLot(draft.slots[i]);
    if (!use) continue;
    const r = plotRect(t, draft.slots[i]);
    for (let j = Math.floor(r.z / USE_CELL); j < Math.ceil((r.z + r.d) / USE_CELL); j++) {
      for (let k = Math.floor(r.x / USE_CELL); k < Math.ceil((r.x + r.w) / USE_CELL); k++) {
        if (k >= 0 && j >= 0 && k < layer.cols && j < layer.rows) codes[j * layer.cols + k] = USE_CODE[use];
      }
    }
  }
  const bare = removeLots(draft, lots, backdrop);
  const open = bare.open!;
  const zoned = { ...open, parking: open.parking.filter((p) => p.area !== area), uses: packUses(codes, layer.cols, layer.rows) };
  return { ...placed, draft: { ...bare, open: zoned } };
}

/** Clear zones: every zone gone, the streets as they are. */
export function clearZones(draft: NeighbourhoodDraft): Edited {
  if (!draft.open) return { refused: 'this town has no zones' };
  const empty = usesFor(draft);
  const had = draft.open.uses;
  if (!had || had.cells.length <= 2) return { draft, cleared: NOTHING };
  return { draft: withUses(draft, empty), cleared: NOTHING };
}

/**
 * Clear all: bare land to lay out from scratch. The main town's area is kept (where its centre is to
 * be); every street, lane and trail goes, every village, lot, building, place, parking space and zone.
 * The roads out of the world are kept, ending where they used to meet the streets, for the first
 * street to run from one to another.
 */
export function clearAll(draft: NeighbourhoodDraft): Edited {
  const open = draft.open;
  if (!open) return { refused: 'this town has no land to clear' };
  const roads = draft.roads
    .filter((r) => r.kind === 'country' && r.ends.includes('out'))
    .map((r): RoadPlan => ({ ...r, ends: [r.ends[0] === 'out' ? 'out' : 'dead', r.ends[1] === 'out' ? 'out' : 'dead'], area: undefined }));
  const lots = new Set(draft.slots.map((s) => s.index));
  const bare = removeLots(draft, lots, new Set((open.backdrop ?? []).map((_, i) => i)));
  const areas = open.areas.slice(0, 1);
  const cleared: NeighbourhoodDraft = {
    ...bare,
    roads,
    greens: [],
    open: { ...bare.open!, areas, parking: [], pads: {}, backdrop: [], places: [], centre: [], squares: [], uses: usesFor(draft) },
  };
  return { draft: cleared, cleared: { lots: [...lots], backdrop: (open.backdrop ?? []).map((_, i) => i) } };
}

/**
 * A village moved in step 2 (`moved`, a `moveVillage` of `area` by `dx`, `dz` m on `before`): the zones in
 * its box go with it, by whole cells.
 */
export function villageZonesMoved(before: NeighbourhoodDraft, moved: Edited, area: number, dx: number, dz: number): Edited {
  const a = before.open?.areas[area];
  const layer = before.open?.uses;
  if (!('draft' in moved) || !a || !layer) return moved;
  const { cols, rows } = layer;
  const was = unpackUses(layer);
  const codes = was.slice();
  const [di, dj] = [Math.round(dx / USE_CELL), Math.round(dz / USE_CELL)];
  const [i0, i1] = [Math.max(0, Math.floor(a.x / USE_CELL)), Math.min(cols - 1, Math.ceil((a.x + a.w) / USE_CELL))];
  const [j0, j1] = [Math.max(0, Math.floor(a.z / USE_CELL)), Math.min(rows - 1, Math.ceil((a.z + a.d) / USE_CELL))];
  for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) codes[j * cols + i] = 0;
  for (let j = j0; j <= j1; j++) {
    for (let i = i0; i <= i1; i++) {
      const [ti, tj] = [i + di, j + dj];
      if (was[j * cols + i] && ti >= 0 && tj >= 0 && ti < cols && tj < rows) codes[tj * cols + ti] = was[j * cols + i];
    }
  }
  return { ...moved, draft: withUses(moved.draft, packUses(codes, cols, rows)) };
}
