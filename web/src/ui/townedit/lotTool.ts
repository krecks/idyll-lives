/**
 * The Lot tool: puts down what its card chose (`lotChoice`) along the road nearest the pointer, as
 * the generator would put one there (`lots.ts` `canPlace`: off the corners, on free ground, a car
 * able to drive out). Its ghost follows the pointer, snapped to where it would go: blue where it
 * fits, red where it doesn't, with why under the tools. A click puts it down (`addLot`). A lot
 * dragged is moved (`moveLot`), keeping its house, garden and kind; one clicked is picked.
 *
 * A civic place is a building, not a lot: the one chosen of those the area under the pointer lacks
 * (`civicChosen`), its footprint snapped along the nearest road as a home's is, laid as the
 * generator lays one (`buildup.ts` `addCivic`). The ghost is its footprint's box.
 */

import type { RoadSpot } from '../../game/edit/ground';
import { addCivic, areaOfRoad } from '../../game/edit/buildup';
import type { Edited } from '../../game/edit/edited';
import { addLot, canPlace, CIVIC_PLACES, moveLot, type LotChoice } from '../../game/edit/lots';
import { civicLabel, draftAreaSize } from '../../game/backdrop';
import type { TownGhosts } from '../../render/types';
import { choiceOf, choiceSize, civicChosen, lotChoice } from './lotChoice.svelte';
import LotCard from './LotCard.svelte';
import { buildingBox, lotBox } from './ghosts';
import { lotAtPoint, spotNear } from './pick';
import type { TownEditor, TownTool } from './tools';
import { sentence } from './words';

/** A press moves less than this (CSS px) and it is a click, not a drag. */
const CLICK = 4;

export const lotTool: TownTool = {
  id: 'lot',
  label: 'Lot',
  icon: 'icon.ui.home',
  does: 'Put a home, a farm or a public place down along a road; drag a lot to move it.',
  stage(editor: TownEditor) {
    /** The house the next lot gets (the ghost shows it; a new one after each lot put down). */
    let seed = editor.seed();
    /** A lot being dragged: which, where the press was, and where it would go now. */
    let drag: { slot: number; x0: number; y0: number; moved: boolean; at: RoadSpot | null; why: string | null } | null = null;
    let last: { x: number; y: number } | null = null;
    /** Why the civic place can't go where the pointer last was. */
    let civicWhy: string | null = null;

    /** A red box about the size of what is put down, at a ground point (where it can't go). */
    const badAt = (x: number, z: number): Partial<TownGhosts> => {
      const { w, d } = choiceSize(editor.t);
      return { lots: [{ x: x - w / 2, z: z - d / 2, w, d, tone: 'bad' }] };
    };

    /** Where a civic place would go at ground point `g`: its ghost and line shown; the edit that puts it down, if it fits. */
    const civicLook = (g: { x: number; z: number }): Edited | null => {
      const draft = editor.draft;
      const no = (why: string) => {
        civicWhy = why;
        editor.preview(badAt(g.x, g.z));
        editor.hint(sentence(why));
        return null;
      };
      const at = spotNear(draft, g.x, g.z);
      if (!at) return no('a civic place goes along a street of the town or a village: come nearer one');
      const area = areaOfRoad(draft, at.road);
      if (area >= 0 && lotChoice.civicArea !== area) lotChoice.civicArea = area;
      const civic = area < 0 ? -1 : civicChosen(draft, area);
      if (civic < 0) return no(area < 0 ? 'a civic place goes along a street of the town or a village' : `${draft.open?.areas[area]?.name ?? 'the town'} has every civic place already`);
      const e = addCivic(draft, editor.t, editor.content, { civic, road: at.road, s: at.s, side: at.side, seed });
      if ('refused' in e) return no(e.refused);
      civicWhy = null;
      const b = e.draft.open!.backdrop!.at(-1)!;
      editor.preview({ lots: [{ ...buildingBox(b), tone: 'new' }] });
      editor.hint(`Click to put the ${civicLabel(CIVIC_PLACES[civic], draftAreaSize(draft, area)).toLowerCase()} down on ${draft.roads[at.road]?.name ?? 'this road'}.`);
      return e;
    };

    /** Where the new lot would go under (`x`, `y`): its ghost and line shown; what to put down where, if it fits. */
    const look = (x: number, y: number): { civic: Edited } | { at: RoadSpot; choice: LotChoice } | null => {
      last = { x, y };
      const draft = editor.draft;
      const t = editor.t;
      const g = editor.ground(x, y);
      if (!g) {
        editor.preview(null);
        editor.hint(null);
        editor.hover(null);
        return null;
      }
      const over = lotAtPoint(draft, t, g.x, g.z);
      if (over !== null) {
        editor.hover({ kind: 'lot', slot: over });
        editor.preview(null);
        editor.hint(`Drag ${draft.slots[over].name} to move it, or click it to see what can be done to it.`);
        return null;
      }
      editor.hover(null);
      if (lotChoice.group === 'civic') {
        const e = civicLook(g);
        return e ? { civic: e } : null;
      }
      const choice = choiceOf(t);
      if (!choice) {
        editor.preview(null);
        editor.hint(null);
        return null;
      }
      const at = spotNear(draft, g.x, g.z);
      if (!at) {
        editor.preview(badAt(g.x, g.z));
        editor.hint('A lot goes along a road: come nearer one.');
        return null;
      }
      const fit = canPlace(draft, t, at, choice, seed);
      if ('refused' in fit) {
        editor.preview(badAt(g.x, g.z));
        editor.hint(sentence(fit.refused));
        return null;
      }
      editor.preview({ lots: [{ x: fit.x, z: fit.z, w: fit.w, d: fit.d, tone: 'new' }] });
      editor.hint(`Click to put it down on ${draft.roads[at.road]?.name ?? 'this road'}.`);
      return { at, choice };
    };

    /** Where the lot being dragged would go under (`x`, `y`): its place now greyed, the new one blue or red. */
    const dragTo = (x: number, y: number) => {
      if (!drag) return;
      const draft = editor.draft;
      const t = editor.t;
      const s = draft.slots[drag.slot];
      const g = editor.ground(x, y);
      const at = g && spotNear(draft, g.x, g.z);
      const from = { ...lotBox(t, s), tone: 'gone' as const };
      if (!g || !at) {
        drag.at = null;
        drag.why = 'A lot goes along a road: come nearer one.';
        editor.preview({ lots: [from] });
        editor.hint(drag.why);
        return;
      }
      const fit = canPlace(draft, t, at, { slot: drag.slot });
      drag.at = at;
      drag.why = 'refused' in fit ? sentence(fit.refused) : null;
      const box = 'refused' in fit ? { ...lotBox(t, s), x: g.x - lotBox(t, s).w / 2, z: g.z - lotBox(t, s).d / 2, tone: 'bad' as const } : { x: fit.x, z: fit.z, w: fit.w, d: fit.d, tone: 'new' as const };
      editor.preview({ lots: [from, box] });
      editor.hint(drag.why ?? `Let go to move ${s.name} to ${draft.roads[at.road]?.name ?? 'this road'}.`);
    };

    return {
      down(x, y) {
        const g = editor.ground(x, y);
        if (!g) return false;
        const over = lotAtPoint(editor.draft, editor.t, g.x, g.z);
        if (over !== null) {
          drag = { slot: over, x0: x, y0: y, moved: false, at: null, why: null };
          return true;
        }
        const placing = look(x, y);
        if (placing) {
          if (editor.apply('civic' in placing ? placing.civic : addLot(editor.draft, editor.t, editor.content, placing.at, placing.choice, seed))) seed = editor.seed();
          look(x, y);
        } else if (lotChoice.group === 'civic') {
          editor.refuse(civicWhy ?? 'nothing to put down');
        } else {
          const choice = choiceOf(editor.t);
          const at = spotNear(editor.draft, g.x, g.z);
          const fit = choice && at ? canPlace(editor.draft, editor.t, at, choice, seed) : null;
          editor.refuse(fit && 'refused' in fit ? fit.refused : at ? 'nothing to put down' : 'a lot goes along a road: come nearer one');
        }
        return true;
      },
      move(x, y) {
        if (!drag) return;
        if (!drag.moved && Math.hypot(x - drag.x0, y - drag.y0) < CLICK) return;
        drag.moved = true;
        editor.hover(null);
        dragTo(x, y);
      },
      up() {
        const d = drag;
        drag = null;
        if (!d) return;
        if (!d.moved) editor.select({ kind: 'lot', slot: d.slot });
        else if (d.at && !d.why) editor.apply(moveLot(editor.draft, editor.t, d.slot, d.at));
        else editor.refuse(d.why ?? 'it can’t go there');
        editor.preview(null);
        if (last) look(last.x, last.y);
      },
      hover: (x, y) => void look(x, y),
      leave() {
        last = null;
        editor.preview(null);
        editor.hint(null);
        editor.hover(null);
      },
      cancel() {
        drag = null;
        editor.preview(null);
        editor.hint(null);
      },
    };
  },
  card: LotCard,
};
