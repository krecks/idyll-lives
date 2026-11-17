/**
 * The zone brush (step 2): paints what the land is to be used for (`uses.ts`) with the card's use, or
 * erases it (`zoneChoice`). A drag is one stroke: its dabs are painted over the town as the pointer
 * goes (the overlay shows the stroke before it is taken), and the stroke is one edit, one step of
 * undo, taken when the button is let go. The brush's ring follows the pointer in the use's colour.
 *
 * Zones never go on the water, its shore, a road or off the world (`game/edit/uses.ts` `paintUses`);
 * with "Beside streets only" the brush paints only near a road. Painting needs no land settled under it
 * (`needsSettle`): the session shows the town as it is with the new zones.
 */

import type { NeighbourhoodDraft } from '../../game/town';
import { USE_LABELS } from '../../game/uses';
import { BESIDE_STREETS, paintUses, type UseStamp } from './layoutApi';
import type { TownEditor, TownTool } from './tools';
import ZoneCard from './ZoneCard.svelte';
import { ZONE_SIZES, zoneChoice } from './zoneChoice.svelte';

/** The ring's inner edge as a share of its radius (the brush paints its whole disc: the band only reads as a ring). */
const INNER = 0.82;
/** Dabs along a drag at most this share of the brush's radius apart, so a quick drag leaves no gaps. */
const SPACING = 0.4;

export const zoneTool: TownTool = {
  id: 'zones',
  label: 'Zones',
  icon: 'icon.ui.floor',
  does: 'Paint where homes, shops, parks and public places go.',
  stage(editor: TownEditor) {
    /** The stroke under way: the plan it is painted on, its dabs, and the plan with it painted so far. */
    let stroke: { base: NeighbourhoodDraft; stamps: UseStamp[]; painted: NeighbourhoodDraft } | null = null;

    const radius = () => zoneChoice.size / 2;
    /** Erasing goes anywhere; painting with "Beside streets only" only within reach of a road. */
    const besideOf = () => (zoneChoice.kind && zoneChoice.beside ? { beside: BESIDE_STREETS } : {});
    const tone = () => zoneChoice.kind ?? 'erase';

    /** The brush's ring at ground point `g` (none: off the land). */
    const ring = (g: { x: number; z: number } | null) => {
      editor.brush(g ? { x: g.x, z: g.z, radius: radius(), inner: radius() * INNER, tone: tone() } : null);
    };

    const say = () => {
      const what = zoneChoice.kind ? `paint ${USE_LABELS[zoneChoice.kind]}` : 'erase zones';
      editor.hint(`Drag to ${what}${zoneChoice.kind && zoneChoice.beside ? ' beside the streets' : ''}. One stroke is one undo.`);
    };

    /** A dab at `g`, painted on the stroke so far as the line on from the last (`paintUses` paints along it): shown on the overlay. */
    const dab = (g: { x: number; z: number }) => {
      const s = stroke;
      if (!s) return;
      const last = s.stamps[s.stamps.length - 1];
      if (last && Math.hypot(g.x - last.x, g.z - last.z) < Math.max(1, radius() * SPACING)) return;
      const stamp: UseStamp = { x: g.x, z: g.z, size: zoneChoice.size };
      s.stamps.push(stamp);
      const made = paintUses(s.painted, last ? [last, stamp] : [stamp], zoneChoice.kind, besideOf());
      // (A dab that changes nothing is refused: the stroke goes on.)
      if (!('draft' in made)) return;
      s.painted = made.draft;
      editor.showUses(s.painted);
    };

    return {
      down(x, y) {
        const g = editor.ground(x, y);
        if (!g) return false;
        const base = editor.draft;
        stroke = { base, stamps: [], painted: base };
        dab(g);
        ring(g);
        return true;
      },
      move(x, y) {
        const g = editor.ground(x, y);
        ring(g);
        if (g) dab(g);
      },
      up() {
        const s = stroke;
        stroke = null;
        if (!s) return;
        // (The stroke as one edit: every dab painted on the plan it began on, as one line.)
        const made = s.painted !== s.base ? paintUses(s.base, s.stamps, zoneChoice.kind, besideOf()) : null;
        if (made && 'draft' in made && editor.draft === s.base) editor.apply(made);
        editor.showUses(null);
      },
      hover(x, y) {
        const g = editor.ground(x, y);
        ring(g);
        if (g) say();
        else editor.hint(null);
      },
      leave() {
        ring(null);
        editor.hint(null);
      },
      cancel() {
        stroke = null;
        ring(null);
        editor.showUses(null);
        editor.hint(null);
      },
      key(e) {
        // ([ and ] step the brush's size down and up.)
        if (e.key !== '[' && e.key !== ']') return false;
        const at = ZONE_SIZES.indexOf(zoneChoice.size) + (e.key === ']' ? 1 : -1);
        zoneChoice.size = ZONE_SIZES[Math.max(0, Math.min(ZONE_SIZES.length - 1, at))];
        return true;
      },
    };
  },
  card: ZoneCard,
};
