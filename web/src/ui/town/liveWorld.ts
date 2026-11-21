/**
 * The game's views as the town tags read them (`TagWorld`): those still in town, the places and their lots, the
 * clock. The tag layer and the building card read them alike.
 */

import type { TownPlace, WorldMeta } from '../../core/protocol';
import { settings } from '../../settings/settings.svelte';
import { clock, WEEKDAYS } from '../format';
import { services } from '../services';
import { game } from '../state.svelte';
import type { TagWorld } from './tags';

export function liveTagWorld(places: readonly TownPlace[], meta: Pick<WorldMeta, 'areas'> | null | undefined): TagWorld {
  const inTown = new Set(game.roster.map((s) => s.id));
  const stages = services.content.lifeStages;
  return {
    sims: game.sims.filter((s) => inTown.has(s.id)),
    roster: game.roster,
    households: game.households,
    plots: game.plots,
    flats: game.flats,
    relationships: game.relationships,
    places,
    placeOfPlot: new Map(places.map((p, i) => [p.plot, i])),
    minute: game.minute,
    weekday: game.weekday,
    time: (m) => clock(m, settings.clock24h),
    days: WEEKDAYS,
    pupil: (stage) => !!stages.find((s) => s.id === stage)?.school,
    areas: meta?.areas ?? [],
  };
}
