/**
 * The zone brush (step 2): paints what the land is to be used for (`uses.ts`) with the card's use, or
 * erases it (`zoneChoice`). A drag is one stroke: its dabs are painted over the town as the pointer
 * goes (the overlay shows the stroke before it is taken), and the stroke is one edit, one step of
 * undo, taken when the button is let go. The brush's ring follows the pointer in the use's colour.
 *
 * Zones never go on the water, its shore, a road or off the world (`layoutApi.ts` `paintUses`); with
 * "Beside streets only" the brush paints only near a road. Painting needs no land settled under it.
 */

import type { NeighbourhoodDraft } from '../../game/town';
import { USE_LABELS } from '../../game/uses';
import { besideStreets, paintUses, type UseStamp } from './layoutApi';
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
    /** The stroke under way: the plan it is painted on, its dabs, where the last was, and the plan with it painted so far. */
    let stroke: { base: NeighbourhoodDraft; stamps: UseStamp[]; last: { x: number; z: number }; painted: NeighbourhoodDraft } | null = null;

    const radius = () => zoneChoice.size / 2;
    const tone = () => zoneChoice.kind ?? 'erase';

    /** The brush's ring at ground point `g` (none: off the land). */
    const ring = (g: { x: number; z: number } | null) => {
      editor.brush(g ? { x: g.x, z: g.z, radius: radius(), inner: radius() * INNER, tone: tone() } : null);
    };

    const say = () => {
      const what = zoneChoice.kind ? `paint ${USE_LABELS[zoneChoice.kind]}` : 'erase zones';
      editor.hint(`Drag to ${what}${zoneChoice.kind && zoneChoice.beside ? ' beside the streets' : ''}. One stroke is one undo.`);
    };

    /** Dabs from the last one to `g`, no further apart than `SPACING` of the radius, painted on the stroke so far. */
    const dab = (g: { x: number; z: number }) => {
      const s = stroke;
      if (!s) return;
      const r = radius();
      const gap = Math.max(1, r * SPACING);
      const far = Math.hypot(g.x - s.last.x, g.z - s.last.z);
      const n = Math.max(1, Math.ceil(far / gap));
      const fresh: UseStamp[] = [];
      for (let k = 1; k <= n; k++) fresh.push({ x: s.last.x + ((g.x - s.last.x) * k) / n, z: s.last.z + ((g.z - s.last.z) * k) / n, r });
      paint(fresh);
      s.last = g;
    };

    /** `fresh` dabs added to the stroke: the plan as painted so far shown on the overlay. */
    const paint = (fresh: UseStamp[]) => {
      const s = stroke;
      if (!s) return;
      s.stamps.push(...fresh);
      const made = paintUses(s.painted, fresh, zoneChoice.kind);
      if (!('draft' in made)) return;
      const kept = zoneChoice.kind && zoneChoice.beside ? besideStreets(s.base, made) : made;
      if (!('draft' in kept)) return;
      s.painted = kept.draft;
      editor.showUses(s.painted);
    };

    return {
      down(x, y) {
        const g = editor.ground(x, y);
        if (!g) return false;
        const base = editor.draft;
        stroke = { base, stamps: [], last: g, painted: base };
        paint([{ x: g.x, z: g.z, r: radius() }]);
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
        if (s.painted !== s.base && editor.draft === s.base) editor.apply({ draft: s.painted, cleared: { lots: [], backdrop: [] } });
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
