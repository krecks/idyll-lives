/**
 * The card's choice as the card (`TownSide.svelte`) and its pin (`ChosenPin.svelte`) show it: its icon, where its pin
 * stands, what it is now (a place open or closed, who lives in a house) and who is inside.
 */

import { settings } from '../../settings/settings.svelte';
import { clock, WEEKDAYS } from '../format';
import { game } from '../state.svelte';
import { townMap } from '../townMap.svelte';
import { chosenAnchor, chosenIcon, placeDetail, whoIsIn, type ChosenTown } from './chosen';
import { liveTagWorld } from './liveWorld';
import { lotHours, townPlaces } from './places';
import { yourPeople } from './tags';
import { townHud } from './townHud.svelte';

class ChosenView {
  /** The card's choice, in Live. */
  readonly chosen = $derived(game.mode === 'live' ? townHud.chosen : null);
  private readonly meta = $derived(townMap.world?.meta ?? null);
  readonly town = $derived.by((): ChosenTown => ({ places: townPlaces(this.meta, game.plots), backdrop: this.meta?.backdrop ?? [], plots: game.plots, households: game.households }));
  readonly icon = $derived(this.chosen ? chosenIcon(this.chosen, this.town) : '');
  readonly anchor = $derived(this.chosen ? chosenAnchor(this.chosen, this.town) : null);
  private readonly world = $derived(this.chosen && (this.chosen.kind === 'place' || this.chosen.kind === 'home') ? liveTagWorld(this.town.places, this.meta) : null);
  /** The hours the choice keeps: its lot's (`lotHours`), else a backdrop place's own; undefined: always open. */
  readonly hours = $derived.by(() => {
    const c = this.chosen;
    if (!c) return undefined;
    if (c.plot !== null) return lotHours(c.plot, this.town.plots, this.town.places);
    return c.kind === 'place' ? this.town.places[c.place]?.hours : undefined;
  });
  /** Who is inside now; null where nobody is simulated (a backdrop building that is no place) or for a person. */
  readonly inside = $derived.by(() => {
    const w = this.world;
    if (!w || !this.chosen) return null;
    const members = w.sims.filter((s) => game.households[s.household]?.player);
    const ids = new Set(members.map((s) => s.id));
    const yours = new Map<number, string>(members.map((s) => [s.id, s.name]));
    for (const p of yourPeople(ids, w)) {
      const s = w.sims.find((x) => x.id === p.sim);
      if (s) yours.set(s.id, s.name);
    }
    return whoIsIn(this.chosen, w, yours);
  });
  /** What it is now: a place (a civic one's kind first: "Fire station · open till 17:00") open or closed and till when, who lives in a house; else what the click said. */
  readonly detail = $derived.by(() => {
    const c = this.chosen;
    if (!c) return '';
    if (c.kind === 'place') {
      const p = this.town.places[c.place];
      if (!p) return c.detail;
      return placeDetail(p, this.hours, game.minute, game.weekday, (m) => clock(m, settings.clock24h), WEEKDAYS);
    }
    if (c.kind === 'home') {
      const hh = game.households.findIndex((h) => h.plot === c.plot);
      if (hh < 0) return 'Nobody lives here';
      const names = game.roster.filter((s) => s.household === hh).map((s) => s.name);
      return names.length ? `Home of ${names.join(', ')}` : '';
    }
    return c.detail;
  });
}

export const chosenView = new ChosenView();
