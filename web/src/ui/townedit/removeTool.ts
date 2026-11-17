/**
 * The Remove tool: a click on a lot, a backdrop building or a street removes it; a street takes
 * with it what stands on it and the roads it leaves joined to nothing (`streets.ts` `whatGoes`).
 * As the pointer goes over one, what would go is greyed on the town and said under the tools; what
 * may not go (the town's last home, its last park, a street the roads need) is red, with why.
 */

import { removalRefused, removeLotsEdit } from '../../game/edit/lots';
import { removeRoad, whatGoes } from '../../game/edit/streets';
import type { NeighbourhoodDraft } from '../../game/town';
import type { TownGhosts } from '../../render/types';
import { buildingBox, lotBox, roadGhost } from './ghosts';
import type { Picked } from './pick';
import type { TownEditor, TownTool } from './tools';
import { listed, sentence, whatGoesLine } from './words';

/** What removing `p` would do: its ghosts, and a line saying it (or why it may not go: `refused`). */
function removal(editor: TownEditor, p: Picked): { ghosts: Partial<TownGhosts>; line: string; refused: boolean } | null {
  const draft = editor.draft;
  const t = editor.t;
  if (p.kind === 'lot') {
    const s = draft.slots[p.slot];
    if (!s) return null;
    const why = removalRefused(draft, new Set([p.slot]));
    return { ghosts: { lots: [{ ...lotBox(t, s), tone: why ? 'bad' : 'gone' }] }, line: why ?? `Removes ${s.name}.`, refused: !!why };
  }
  if (p.kind === 'building') {
    const b = draft.open?.backdrop?.[p.building];
    if (!b) return null;
    const place = b.place === undefined ? undefined : draft.open?.places?.[b.place];
    return { ghosts: { lots: [{ ...buildingBox(b), tone: 'gone' }] }, line: `Removes ${place ? place.name : 'this building'}.`, refused: false };
  }
  if (p.kind === 'road') {
    const r = draft.roads[p.road];
    if (!r) return null;
    const goes = goesOf(editor, p.road);
    if ('refused' in goes) return { ghosts: { roads: [roadGhost(r, 'bad')] }, line: sentence(goes.refused), refused: true };
    const also = whatGoesLine(draft, goes, p.road);
    return {
      ghosts: {
        roads: goes.roads.map((i) => roadGhost(draft.roads[i], 'bad')),
        lots: [...goes.lots.map((i) => ({ ...lotBox(t, draft.slots[i]), tone: 'gone' as const })), ...goes.backdrop.map((i) => ({ ...buildingBox(draft.open!.backdrop![i]), tone: 'gone' as const }))],
      },
      line: `Removes ${r.name ?? 'this road'}${also.length ? `, and with it ${listed(also)}` : ''}.`,
      refused: false,
    };
  }
  return null;
}

/** `whatGoes` for a road of the plan as it stands (kept: the pointer asks again and again). */
const goesCache = new WeakMap<NeighbourhoodDraft, Map<number, ReturnType<typeof whatGoes>>>();
function goesOf(editor: TownEditor, road: number): ReturnType<typeof whatGoes> {
  let byRoad = goesCache.get(editor.draft);
  if (!byRoad) goesCache.set(editor.draft, (byRoad = new Map()));
  let goes = byRoad.get(road);
  if (!goes) byRoad.set(road, (goes = whatGoes(editor.draft, editor.t, road)));
  return goes;
}

const WHAT = { lots: true, buildings: true, roads: true };

export const removeTool: TownTool = {
  id: 'remove',
  label: 'Remove',
  icon: 'icon.ui.trash',
  does: 'Click a lot, a building or a street to remove it (a street with what stands on it).',
  stage(editor: TownEditor) {
    /** What the pointer was last over, and where (looked at again after an edit). */
    let last: { x: number; y: number } | null = null;
    const look = (x: number, y: number) => {
      last = { x, y };
      const p = editor.pick(x, y, WHAT);
      const r = p && removal(editor, p);
      editor.preview(r?.ghosts ?? null);
      editor.hint(r?.line ?? null);
    };
    return {
      down(x, y) {
        const p = editor.pick(x, y, WHAT);
        if (!p) return false;
        const draft = editor.draft;
        if (p.kind === 'lot') editor.apply(removeLotsEdit(draft, new Set([p.slot])));
        else if (p.kind === 'building') editor.apply(removeLotsEdit(draft, new Set(), new Set([p.building])));
        else if (p.kind === 'road') editor.apply(removeRoad(draft, editor.t, p.road));
        look(x, y);
        return true;
      },
      move: () => {},
      up: () => {},
      hover: look,
      leave() {
        last = null;
        editor.preview(null);
        editor.hint(null);
      },
      cancel: () => void (last && look(last.x, last.y)),
    };
  },
};
