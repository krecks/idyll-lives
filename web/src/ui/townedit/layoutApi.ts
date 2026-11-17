/**
 * What step 2 (Streets & zones) asks of the layout's own edits and figures, in one place: painting
 * zones (`game/edit/uses.ts` `paintUses`), the live summary and the hatch (`game/proposal.ts`
 * `usesSummary`, `hatched`), and the edits the generation side has none of: the Village tool's village
 * as streets and zones (`villageAsLayout`, `villageZonesMoved`), and the panel's Clear all and Clear zones.
 */

import type { Edited } from '../../game/edit/edited';
import { removeLots } from '../../game/edit/lots';
import { noUseCells, usesOfDraft } from '../../game/edit/uses';
import { plotRect, type NeighbourhoodDraft, type PlotSlot, type Templates } from '../../game/town';
import { packUses, unpackUses, USE_CELL, USE_CODE, usesFor, type UseKind, type UseLayer } from '../../game/uses';
import type { UsesSummary } from '../../game/proposal';

export { needsSettle, noUseCells, paintUses, roadReach, type UseStamp } from '../../game/edit/uses';
export { BUILD_REACH, hatched, usesSummary, type UsesSummary } from '../../game/proposal';

/** "Beside streets only": the brush paints only this near a road (m). */
export const BESIDE_STREETS = 40;

/** Nothing cleared: a zones edit takes nothing out of the way. */
const NOTHING = { lots: [] as number[], backdrop: [] as number[] };

/** The layer a draft paints on: its own, else an empty one for its world. */
export const layerOf = usesOfDraft;

/** `draft` with `layer` as its uses. */
function setUses(draft: NeighbourhoodDraft, layer: UseLayer): NeighbourhoodDraft {
  if (!draft.open) return draft;
  return { ...draft, open: { ...draft.open, uses: layer } };
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
  const none = noUseCells(draft);
  for (const i of lots) {
    const use = useOfLot(draft.slots[i]);
    if (!use) continue;
    const r = plotRect(t, draft.slots[i]);
    for (let j = Math.floor(r.z / USE_CELL); j < Math.ceil((r.z + r.d) / USE_CELL); j++) {
      for (let k = Math.floor(r.x / USE_CELL); k < Math.ceil((r.x + r.w) / USE_CELL); k++) {
        if (k >= 0 && j >= 0 && k < layer.cols && j < layer.rows && !none[j * layer.cols + k]) codes[j * layer.cols + k] = USE_CODE[use];
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
  return { draft: setUses(draft, empty), cleared: NOTHING };
}

/**
 * Clear all: bare land to lay out from scratch. The main town's area is kept (where its centre is to
 * be); every road goes, every village, lot, building, place, parking space and zone. The first street
 * is then drawn in from the edge of the world (the Street tool says so while the land is bare).
 */
export function clearAll(draft: NeighbourhoodDraft): Edited {
  const open = draft.open;
  if (!open) return { refused: 'this town has no land to clear' };
  const lots = new Set(draft.slots.map((s) => s.index));
  const bare = removeLots(draft, lots, new Set((open.backdrop ?? []).map((_, i) => i)));
  const areas = open.areas.slice(0, 1);
  const cleared: NeighbourhoodDraft = {
    ...bare,
    roads: [],
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
  return { ...moved, draft: setUses(moved.draft, packUses(codes, cols, rows)) };
}
