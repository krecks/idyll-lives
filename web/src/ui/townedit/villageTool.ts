/**
 * The Village tool: puts a village down (a hamlet of the place's own kind, with its road to the
 * town and its homes: `villages.ts` `placeVillage`) where the pointer is. Its ghost is the village's
 * box, following the pointer: as the plan's new things where the land lets a village stand there,
 * red where it doesn't, with why under the tools (`canPlaceVillage`). A click puts it down and picks
 * it. A village dragged by its box (its name tag stands in it) is moved on release, its box's ghost
 * following (`moveVillage`); one clicked is picked. With a village to move from its card ("Move":
 * `villageChoice.moving`), the ghost is that village's box and a click puts it there.
 *
 * The main town is moved by step 2's "Move the town", which lays it again where it is put.
 */

import { canMoveVillage, canPlaceVillage, moveVillage, placeVillage, villageSize } from '../../game/edit/villages';
import type { TownGhosts } from '../../render/types';
import type { TownEditor, TownTool } from './tools';
import { applyVillage, villageChoice } from './villageChoice.svelte';
import VillageCard from './VillageCard.svelte';
import { sentence } from './words';

/** A press moves less than this (CSS px) and it is a click, not a drag. */
const CLICK = 4;

type Box = { x: number; z: number; w: number; d: number };

/** A village's box as a ghost: the plan's new things' tone where it may go, red where not. */
const boxGhost = (b: Box, ok: boolean): Partial<TownGhosts> => ({ boxes: [{ x: b.x, z: b.z, w: b.w, d: b.d, tone: ok ? 'new' : 'bad' }] });
/** A box `w` × `d` round a ground point. */
const around = (x: number, z: number, w: number, d: number): Box => ({ x: x - w / 2, z: z - d / 2, w, d });

export const villageTool: TownTool = {
  id: 'village',
  label: 'Village',
  icon: 'icon.ui.town',
  does: 'Put a village down where the land lets one stand; drag a village to move it.',
  stage(editor: TownEditor) {
    /** The next village's dice (its shape, name and homes); a new seed after each one put down. */
    let seed = editor.seed();
    /** A village being dragged: which, where the press was (on screen and on the ground), whether it has moved, and the move it would make. */
    let drag: { area: number; x0: number; y0: number; g0: { x: number; z: number }; moved: boolean; dx: number; dz: number; why: string | null } | null = null;
    let last: { x: number; y: number } | null = null;

    /** Village `area` moved by (`dx`, `dz`): its box there shown, blue or red, and the line saying so; why not, if it can't go. */
    const showMove = (area: number, dx: number, dz: number, verb: string): string | null => {
      const draft = editor.draft;
      const a = draft.open!.areas[area];
      const fit = canMoveVillage(draft, editor.t, area, dx, dz);
      if ('refused' in fit) {
        editor.preview(boxGhost({ ...a, x: a.x + dx, z: a.z + dz }, false));
        editor.hint(sentence(fit.refused));
        return sentence(fit.refused);
      }
      editor.preview(boxGhost(fit, true));
      editor.hint(`${verb} to move ${a.name} here.`);
      return null;
    };

    /** What the tool would do under (`x`, `y`): its ghost and line shown; the edit to make there, if one may be made. */
    const look = (x: number, y: number): { place: { x: number; z: number } } | { move: { area: number; dx: number; dz: number } } | null => {
      last = { x, y };
      const draft = editor.draft;
      const t = editor.t;
      const g = editor.ground(x, y);
      if (!g || !draft.open) {
        editor.preview(null);
        editor.hint(null);
        editor.hover(null);
        return null;
      }
      const moving = villageChoice.moving;
      if (moving !== null && moving > 0 && draft.open.areas[moving]) {
        const a = draft.open.areas[moving];
        const [dx, dz] = [Math.round(g.x - (a.x + a.w / 2)), Math.round(g.z - (a.z + a.d / 2))];
        editor.hover(null);
        return showMove(moving, dx, dz, 'Click') ? null : { move: { area: moving, dx, dz } };
      }
      const over = editor.pick(x, y, { areas: true });
      if (over?.kind === 'village') {
        editor.hover(over);
        editor.preview(null);
        editor.hint(`Drag ${draft.open.areas[over.area]?.name ?? 'the village'} to move it, or click it to see what can be done to it.`);
        return null;
      }
      editor.hover(null);
      const fit = canPlaceVillage(draft, t, { x: g.x, z: g.z, swap: villageChoice.swap });
      if ('refused' in fit) {
        const [w, d] = villageSize(draft, villageChoice.swap);
        editor.preview(boxGhost(around(g.x, g.z, w, d), false));
        editor.hint(over?.kind === 'town' ? `${sentence(fit.refused)} The main town moves with Move the town.` : sentence(fit.refused));
        return null;
      }
      editor.preview(boxGhost(fit, true));
      editor.hint('Click to put a village down here: its streets, a road to the town and its homes.');
      return { place: { x: g.x, z: g.z } };
    };

    /** A refusal for a click where `look` found nothing to do: what the hint says. */
    const refuseHere = (x: number, y: number) => {
      const g = editor.ground(x, y);
      const draft = editor.draft;
      const moving = villageChoice.moving;
      if (!g || !draft.open) return;
      if (moving !== null && draft.open.areas[moving]) {
        const a = draft.open.areas[moving];
        const fit = canMoveVillage(draft, editor.t, moving, g.x - (a.x + a.w / 2), g.z - (a.z + a.d / 2));
        if ('refused' in fit) editor.refuse(fit.refused);
        return;
      }
      const fit = canPlaceVillage(draft, editor.t, { x: g.x, z: g.z, swap: villageChoice.swap });
      if ('refused' in fit) editor.refuse(fit.refused);
    };

    return {
      down(x, y) {
        const g = editor.ground(x, y);
        if (!g) return false;
        const draft = editor.draft;
        // (A village being moved goes where the click is.)
        if (villageChoice.moving === null) {
          const over = editor.pick(x, y, { areas: true });
          if (over?.kind === 'village') {
            drag = { area: over.area, x0: x, y0: y, g0: g, moved: false, dx: 0, dz: 0, why: null };
            return true;
          }
        }
        const act = look(x, y);
        if (!act) {
          refuseHere(x, y);
          return true;
        }
        if ('move' in act) {
          const { area, dx, dz } = act.move;
          if (applyVillage(editor, moveVillage(draft, editor.t, area, dx, dz))) {
            villageChoice.moving = null;
            editor.select({ kind: 'village', area });
          }
        } else if (applyVillage(editor, placeVillage(draft, editor.t, editor.content, { ...act.place, swap: villageChoice.swap, seed }))) {
          seed = editor.seed();
          editor.select({ kind: 'village', area: editor.draft.open!.areas.length - 1 });
        }
        look(x, y);
        return true;
      },
      move(x, y) {
        if (!drag) return;
        if (!drag.moved && Math.hypot(x - drag.x0, y - drag.y0) < CLICK) return;
        drag.moved = true;
        editor.hover(null);
        const g = editor.ground(x, y);
        if (!g) return;
        [drag.dx, drag.dz] = [Math.round(g.x - drag.g0.x), Math.round(g.z - drag.g0.z)];
        drag.why = showMove(drag.area, drag.dx, drag.dz, 'Let go');
      },
      up() {
        const d = drag;
        drag = null;
        if (!d) return;
        if (!d.moved) editor.select({ kind: 'village', area: d.area });
        else if (!d.why && (d.dx || d.dz)) {
          if (applyVillage(editor, moveVillage(editor.draft, editor.t, d.area, d.dx, d.dz))) editor.select({ kind: 'village', area: d.area });
        } else if (d.why) editor.refuse(d.why);
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
  card: VillageCard,
};
