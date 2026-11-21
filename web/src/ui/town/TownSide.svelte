<script lang="ts">
  import Icon from '../Icon.svelte';
  import { clock, money, WEEKDAYS } from '../format';
  import { services } from '../services';
  import { settings } from '../../settings/settings.svelte';
  import { game } from '../state.svelte';
  import { townMap } from '../townMap.svelte';
  import { TAG_ORDER, type TagKind } from './kinds';
  import { lotInSentence, plotInfo } from '../place';
  import { areaAt, closedWhy } from './places';
  import { eventText, nextEvents, waitText, type TodayMember } from './today';
  import { townHud } from './townHud.svelte';
  import { chosenView } from './chosenView.svelte';
  import { insideText } from './chosen';

  /**
   * The town view's left column: the tag the player clicked (one of your people, to
   * visit or call; a place; a building, its flats listed, each to look inside or visit), "Today in <town>" with the
   * household's next events and who is away out of sight, and the tag menu. The card stays until closed, also back
   * down in the play view; the rest shows in the town view.
   */
  const LABELS: Record<TagKind, string> = { household: 'Household', people: 'People', places: 'Places', sale: 'For sale', names: 'Names' };
  const time = (m: number) => clock(m, settings.clock24h);

  const meta = $derived(townMap.world?.meta ?? null);
  /** The home (a lot, or a flat as a lot of its own). */
  const home = $derived(plotInfo(game, game.households.find((h) => h.player)?.plot) ?? null);
  const town = $derived.by(() => {
    const areas = meta?.areas ?? [];
    if (home && areas.length) return areas[areaAt(home.x + home.w / 2, home.z + home.d / 2, areas)]?.name ?? meta?.name;
    return meta?.name ?? 'town';
  });

  const inTown = $derived(new Set(game.roster.map((s) => s.id)));
  const members = $derived(game.sims.filter((s) => inTown.has(s.id) && game.households[s.household]?.player));
  const content = services.content;
  const visits = new Set(content.activities.filter((a) => a.visit).map((a) => a.id));
  const events = $derived.by(() => {
    if (!townHud.on) return [];
    const list: TodayMember[] = members.map((s) => ({ id: s.id, name: s.name, job: s.job, pupil: !!content.lifeStages.find((st) => st.id === s.stage)?.school, plan: s.plan }));
    return nextEvents(list, { school: content.school, visits, householdRoutines: game.householdRoutines }, game.minute, game.weekday).slice(0, 3);
  });

  // ---- the card of the tag clicked ----
  const chosen = $derived(game.mode === 'live' ? townHud.chosen : null);
  const me = $derived(game.selectedSim);
  const call = content.phoneActions.findIndex((a) => a.id === 'call');
  /** Why the one selected can't visit there now (a place shut: "Closed · opens 08:00"), or null. */
  const visitWhy = $derived.by(() => {
    const plot = chosen?.plot ?? null;
    if (!me) return 'Nobody selected';
    if (plot === null) return 'Nowhere to visit';
    if (me.plot === plot) return `${me.name} is there`;
    return closedWhy(chosenView.hours, game.minute, game.weekday, time, WEEKDAYS) ?? me.homebound ?? null;
  });
  const callWhy = $derived.by(() => {
    if (chosen?.kind !== 'person') return null;
    if (!me) return 'Nobody selected';
    if (call < 0 || !me.phone) return 'No phones in this town';
    if (me.phone.off) return me.phone.off;
    const c = me.phone.contacts.find((x) => x.id === chosen.sim);
    if (!c) return `Not in ${me.name}'s phone`;
    return c.why[call] ?? null;
  });
  /** Where a visit goes ("the Smiths", "Town Park"): the lot they're on, else their home. */
  const visitTo = $derived.by(() => {
    const p = plotInfo(game, chosen?.plot);
    return p ? lotInSentence(p, game.households) : '';
  });
  /** Why the buttons are greyed out, in a line. */
  const why = $derived([chosen?.plot != null && chosen.kind !== 'flats' && visitWhy ? `Visit: ${visitWhy}` : '', callWhy ? `Call: ${callWhy}` : ''].filter(Boolean).join(' · '));
  /** A building's flats (`chosenView.flats`): who lives there, or for sale and its price. */
  const flats = $derived(chosenView.flats);
  /** Why the one selected can't visit flat `plot` now, or null. */
  const flatVisitWhy = (plot: number) => (!me ? 'Nobody selected' : me.plot === plot ? `${me.name} is there` : (me.homebound ?? null));
  function visitFlat(plot: number) {
    services.controls.visit(plot);
    townHud.chosen = null;
  }
  /** The lot of the choice, to look inside (a house, a public place's lot); and whether it is shown open now, by "Look inside". */
  const lookPlot = $derived(chosen?.kind === 'home' || chosen?.kind === 'place' ? chosen.plot : null);
  const looking = $derived(lookPlot !== null && game.lookInside === lookPlot && game.viewPlot === lookPlot);
  const detail = $derived(chosenView.detail);
  const inside = $derived(chosenView.inside ? insideText(chosenView.inside) : '');
  function close() {
    townHud.chosen = null;
    if (game.lookInside !== null) services.controls.lookInside(null);
  }
  function visit() {
    if (chosen?.plot != null) services.controls.visit(chosen.plot);
    townHud.chosen = null;
  }
  function phone() {
    if (chosen?.kind === 'person') services.controls.phone(call, chosen.sim);
    townHud.chosen = null;
  }
</script>

{#if game.mode === 'live' && (chosen || townHud.on)}
  <div class="side" style:--fade={townHud.on ? Math.max(townHud.fade, 0.001) : 1}>
    {#if chosen}
      <section class="card glass chosen" aria-label={chosen.title}>
        <header>
          {#if chosenView.icon && chosen.kind !== 'person'}<span class="kind"><Icon name={chosenView.icon} size={15} /></span>{/if}
          <b>{chosen.title}</b>
          <button class="close" aria-label="Close" onclick={close}><Icon name="icon.ui.close" size={13} /></button>
        </header>
        {#if detail}<p class="muted">{detail}</p>{/if}
        {#if inside}<p class="inside"><Icon name="icon.need.social" size={13} />{inside}</p>{/if}
        {#if flats.length}
          <ul class="flats" aria-label="Flats">
            {#each flats as f (f.plot)}
              {@const why = flatVisitWhy(f.plot)}
              <li>
                <span class="flat"><b>{f.name}</b><span class="muted">{f.plot === home?.id ? 'Your home' : (f.who ?? (f.price !== null && !game.creative ? `For sale · ${money(f.price)}+` : 'For sale'))}</span></span>
                {#if f.plot !== home?.id}
                  {#if f.who !== null}
                    <button class="btn small icon" disabled={!!why} title={why ?? `${me?.name} visits ${f.who.replace(/^The /, 'the ')}`} aria-label="Visit {f.name}" onclick={() => visitFlat(f.plot)}>
                      <Icon name="icon.ui.home" size={14} />
                    </button>
                  {/if}
                  {#if game.lookInside === f.plot}
                    <button class="btn small icon" title="Close {f.name} up again" aria-label="Close {f.name} up" onclick={() => services.controls.lookInside(null)}><Icon name="icon.ui.wallsUp" size={14} /></button>
                  {:else}
                    <button class="btn small icon" title="Look inside {f.name}" aria-label="Look inside {f.name}" onclick={() => services.controls.lookInside(f.plot)}><Icon name="icon.ui.wallsDown" size={14} /></button>
                  {/if}
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
        <div class="actions">
          {#if chosen.plot !== null && chosen.plot !== home?.id && chosen.kind !== 'flats'}
            <button class="btn small primary" disabled={!!visitWhy} title={visitWhy ?? `${me?.name} goes to ${visitTo}`} onclick={visit}>
              <Icon name="icon.ui.home" size={14} />{me ? `${me.name}: visit` : 'Visit'}
            </button>
          {/if}
          {#if looking}
            <button class="btn small" title="Close the house up again" onclick={() => services.controls.lookInside(null)}>
              <Icon name="icon.ui.wallsUp" size={14} />Close up
            </button>
          {:else if lookPlot !== null && game.viewPlot !== lookPlot}
            <button class="btn small" title="Show the rooms inside" onclick={() => services.controls.lookInside(lookPlot)}>
              <Icon name="icon.ui.wallsDown" size={14} />Look inside
            </button>
          {/if}
          {#if chosen.kind === 'person'}
            <button class="btn small" disabled={!!callWhy} title={callWhy ?? `${me?.name} calls ${chosen.title}`} onclick={phone}>
              <Icon name="icon.ui.phone" size={14} />Call
            </button>
          {/if}
        </div>
        {#if why}<p class="why muted">{why}</p>{/if}
      </section>
    {/if}

    {#if townHud.on}
      <section class="card glass today fades" aria-label="Today in {town}">
        <span class="eyebrow">Today in {town}</span>
        {#if events.length}
          <ul>
            {#each events as e (`${e.sim}:${e.kind}`)}
              <li>
                <span class="when tabular">{time(e.minute)}</span>
                <span class="what"><b>{e.name}</b> · {eventText(e.kind)}</span>
                <span class="wait muted">{waitText(e, game.weekday, WEEKDAYS)}</span>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="muted">Nothing planned.</p>
        {/if}
        {#if townHud.away.length}
          <ul class="away">
            {#each townHud.away as a (a.id)}
              <li><b>{a.name}</b> <span class="muted">· {a.text}</span></li>
            {/each}
          </ul>
        {/if}
      </section>

      <div class="filters glass fades" role="group" aria-label="Tags">
        {#each TAG_ORDER as kind (kind)}
          <button class="chip" class:on={townHud.filters[kind]} aria-pressed={townHud.filters[kind]} onclick={() => townHud.toggle(kind)}>{LABELS[kind]}</button>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .side {
    position: absolute;
    top: var(--top);
    left: var(--edge);
    width: 280px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  }
  .fades {
    opacity: var(--fade);
  }
  .card {
    padding: 10px 12px;
    border-radius: var(--radius-md);
    pointer-events: auto;
    animation: rise var(--slow) var(--ease);
  }
  .chosen header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .chosen header b {
    flex: 1;
    font-size: 14px;
  }
  .kind {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    flex: none;
    background: var(--accent);
    color: #fff;
  }
  .inside {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .close {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    color: var(--text-muted);
  }
  .close:hover {
    background: var(--hairline);
  }
  p {
    margin: 4px 0 0;
    font-size: 12px;
  }
  .muted {
    color: var(--text-muted);
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 8px;
  }
  .small {
    height: 30px;
    padding: 0 10px;
    font-size: 12px;
    gap: 6px;
  }
  .why {
    font-size: 11px;
  }
  ul.flats {
    margin-top: 8px;
    gap: 2px;
  }
  .flats li {
    align-items: center;
    gap: 4px;
  }
  .flat {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .flat .muted {
    font-size: 11px;
  }
  .btn.icon {
    width: 30px;
    padding: 0;
    justify-content: center;
  }
  .eyebrow {
    font-size: 10.5px;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }
  ul {
    list-style: none;
    margin: 6px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12.5px;
  }
  li {
    display: flex;
    align-items: baseline;
    gap: 8px;
    white-space: nowrap;
  }
  .when {
    font-weight: 700;
    min-width: 38px;
  }
  .what {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .wait {
    font-size: 11px;
  }
  .away {
    margin-top: 8px;
    padding-top: 6px;
    border-top: 1px solid var(--hairline);
    font-size: 12px;
  }
  .away li {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
    padding: 4px;
    border-radius: var(--radius-md);
    pointer-events: auto;
    align-self: flex-start;
  }
  .chip {
    height: 24px;
    padding: 0 9px;
    border-radius: var(--radius-pill);
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-muted);
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  .chip:hover {
    color: var(--text);
  }
  .chip.on {
    background: var(--surface);
    color: var(--accent);
    box-shadow: var(--shadow-sm);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
