/**
 * The named places for checks (`tools/bench/check.mjs places`), under `?debug` on the dev server only:
 * `window.__places()` lists every place with a lot of its own (a school, a hotel, a shop on the high
 * street: `game/placeLots.ts`), with the layout it was most likely stamped from (`houses.json` `parks`
 * entries of kind `place` of its lot's size, matched by the things on the lot; none for a lot without a
 * `meta.lotLooks` entry, as the park, the grocer and the bakery are), and
 * `window.__lookInside(name, storey?)` looks inside one as its card's Look inside does, on that
 * storey (0: the ground), handing back its plot (null: no such place). Absent from production builds.
 */

import type { PlotInfo, TownPlace, WorldStructure } from '../core/protocol';
import { loadTemplates, type PublicTemplate } from '../game/town';
import type { GameControls } from '../ui/services';

/** A place as `__places()` gives it. */
export interface PlaceSeen {
  name: string;
  kind: TownPlace['kind'];
  plot: number;
  /** The layout id (`houses.json`), null when no place layout has the lot's size. */
  template: string | null;
  /** The lot (tiles) and the storeys drawn. */
  w: number;
  d: number;
  storeys: number;
  hours?: TownPlace['hours'];
}

declare global {
  interface Window {
    __places?: () => Promise<PlaceSeen[]>;
    __lookInside?: (name: string, storey?: number) => number | null;
  }
}

/** How many of the layout's things are on the lot, less the lot's things the layout has none of. */
function likeness(t: PublicTemplate, defs: Map<string, number>): number {
  const want = new Map<string, number>();
  for (const o of [...t.objects, ...(t.upper?.objects ?? [])]) want.set(o.def, (want.get(o.def) ?? 0) + 1);
  let score = 0;
  for (const [def, n] of defs) score += Math.min(n, want.get(def) ?? 0) - Math.max(0, n - (want.get(def) ?? 0));
  return score;
}

function templateOf(templates: readonly PublicTemplate[], plot: PlotInfo, world: WorldStructure): string | null {
  // (Only a place's own lot has a look of its kind: the town's park, grocer and bakery are on the common frame.)
  if (!world.meta?.lotLooks?.[plot.id]) return null;
  const sized = templates.filter((t) => t.kind === 'place' && t.lot && ((t.lot.width === plot.w && t.lot.depth === plot.d) || (t.lot.width === plot.d && t.lot.depth === plot.w)));
  if (sized.length < 2) return sized[0]?.id ?? null;
  const defs = new Map<string, number>();
  for (const o of world.objects) if (o.x >= plot.x && o.x < plot.x + plot.w && o.z >= plot.z && o.z < plot.z + plot.d) defs.set(o.def, (defs.get(o.def) ?? 0) + 1);
  return sized.map((t) => ({ id: t.id, score: likeness(t, defs) })).sort((a, b) => b.score - a.score)[0].id;
}

/** Installs `__places` and `__lookInside` on a dev build; does nothing in production. */
export function installPlaceHooks(controls: GameControls, world: () => WorldStructure | null): void {
  if (!import.meta.env.DEV) return;
  const seen = (w: WorldStructure, templates: readonly PublicTemplate[]): PlaceSeen[] =>
    (w.meta?.places ?? []).flatMap((p) => {
      const plot = p.plot === undefined ? undefined : w.plots[p.plot];
      if (!plot) return [];
      return [{ name: p.name, kind: p.kind, plot: plot.id, template: templateOf(templates, plot, w), w: plot.w, d: plot.d, storeys: plot.storeys ?? 1, ...(p.hours ? { hours: p.hours } : {}) }];
    });
  window.__places = async () => {
    const [w, t] = [world(), await loadTemplates()];
    return w ? seen(w, t.parks) : [];
  };
  window.__lookInside = (name, storey = 0) => {
    const plot = world()?.meta?.places?.find((p) => p.name === name)?.plot;
    if (plot === undefined) return null;
    controls.lookInside(plot);
    controls.setStorey(storey);
    return plot;
  };
}
