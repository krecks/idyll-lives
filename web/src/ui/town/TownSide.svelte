<script lang="ts">
  import Icon from '../Icon.svelte';
  import { clock, WEEKDAYS } from '../format';
  import { services } from '../services';
  import { settings } from '../../settings/settings.svelte';
  import { game } from '../state.svelte';
  import { townMap } from '../townMap.svelte';
  import { TAG_ORDER, type TagKind } from './kinds';
  import { lotInSentence } from '../place';
  import { areaAt } from './places';
  import { eventText, nextEvents, waitText, type TodayMember } from './today';
  import { townHud } from './townHud.svelte';

  /**
   * The town view's left column: the tag the player clicked (one of your people, to
   * visit or call; a place), "Today in <town>" with the household's next events and who is away out of sight, and
   * the tag menu. The card stays until closed, also back down in the play view; the rest shows in the town view.
   */
  const LABELS: Record<TagKind, string> = { household: 'Household', people: 'People', places: 'Places', sale: 'For sale', names: 'Names' };
  const time = (m: number) => clock(m, settings.clock24h);

  const meta = $derived(townMap.world?.meta ?? null);
  const home = $derived.by(() => {
    const id = game.households.find((h) => h.player)?.plot;
    return id == null ? null : (game.plots[id] ?? null);
  });
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
  /** Why the one selected can't visit there now, or null. */
  const visitWhy = $derived.by(() => {
    const plot = chosen?.plot ?? null;
    if (!me) return 'Nobody selected';
    if (plot === null) return 'Nowhere to visit';
    if (me.plot === plot) return `${me.name} is there`;
    return me.homebound ?? null;
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
    const p = chosen?.plot == null ? null : game.plots[chosen.plot];
    return p ? lotInSentence(p, game.households) : '';
  });
  /** Why the buttons are greyed out, in a line. */
  const why = $derived([chosen?.plot != null && visitWhy ? `Visit: ${visitWhy}` : '', callWhy ? `Call: ${callWhy}` : ''].filter(Boolean).join(' · '));
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
          <b>{chosen.title}</b>
          <button class="close" aria-label="Close" onclick={() => (townHud.chosen = null)}><Icon name="icon.ui.close" size={13} /></button>
        </header>
        {#if chosen.detail}<p class="muted">{chosen.detail}</p>{/if}
        <div class="actions">
          {#if chosen.plot !== null}
            <button class="btn small primary" disabled={!!visitWhy} title={visitWhy ?? `${me?.name} goes to ${visitTo}`} onclick={visit}>
              <Icon name="icon.ui.home" size={14} />{me ? `${me.name}: visit` : 'Visit'}
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
