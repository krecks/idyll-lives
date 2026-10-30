<script lang="ts">
  import { onMount } from 'svelte';
  import { menuScene } from '../../game/menuScene';
  import { houseTemplate, loadTemplates, slotDims, vacantSlots, withGarden, type PlotSlot, type Templates } from '../../game/town';
  import type { GardenSize } from '../../game/housegen';
  import { residentsOf, sleepersOf } from '../../game/household';
  import { homePrice, priceParts } from '../../game/price';
  import { fundsAfter, savingsOf } from '../../game/wealth';
  import { app } from '../app.svelte';
  import { money } from '../format';
  import Icon from '../Icon.svelte';
  import Modal from '../kit/Modal.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import TownStage, { type LotLabel } from '../kit/TownStage.svelte';
  import { services } from '../services';

  let templates = $state.raw<Templates | null>(null);
  /** Changes when a lot's garden is made bigger or smaller (the lots along the street move). */
  const town = $derived(app.town!);
  const household = app.household!;
  const economy = services.content.economy;
  const vacant = $derived(vacantSlots(town));
  const vacantIds = $derived(vacant.map((v) => v.index));
  let selectedIndex = $state<number | null>(vacantSlots(app.town!).find((v) => v.index === app.homeSlot)?.index ?? vacantSlots(app.town!)[0]?.index ?? null);
  const selected = $derived(selectedIndex === null ? null : (town.slots[selectedIndex] ?? null));
  const garden = $derived<GardenSize | undefined>(selected?.garden);
  const dims = $derived(templates && selected ? slotDims(templates, selected) : null);
  function setGarden(size: GardenSize) {
    if (templates && selected) app.town = withGarden(templates, town, selected.index, size);
  }
  /** The garden sizes to choose from: one that doesn't fit beside the neighbours can't be picked. */
  const gardens = $derived(
    (['snug', 'roomy', 'big'] as const).map((value) => ({
      value,
      label: value[0].toUpperCase() + value.slice(1),
      disabled: !!templates && !!selected && value !== garden && withGarden(templates, town, selected.index, value) === town,
    })),
  );
  let hover = $state<number | null>(null);
  /** The camera is on the chosen lot (false: the whole town). */
  let close = $state(true);
  $effect(() => {
    app.homeSlot = selected?.index ?? null;
  });
  /** The house as it stands, or an empty lot to build on. */
  let lot = $state<'house' | 'empty'>(app.emptyLot ? 'empty' : 'house');
  $effect(() => {
    app.emptyLot = lot === 'empty';
  });
  const house = $derived(templates && selected ? houseTemplate(templates, selected.template) : undefined);
  const residents = $derived(residentsOf(services.content, household));
  /**
   * Sleeping places in the house as it would be furnished for this household (empty lot: unknown),
   * counted as the simulation counts them (sim-core `lifecycle::beds`): a slot on anything with a sleep interaction, but a
   * couple's bed (one where partners try for a baby) is a place for one unless one of the household's couples sleeps in it.
   */
  const bedPlaces = $derived.by(() => {
    const sleep = services.content.sleepTags;
    if (!templates || !selected || lot === 'empty' || !sleep.length) return null;
    const fitted = houseTemplate(templates, selected.template, residents);
    if (!fitted) return null;
    let [places, couples] = [0, 0];
    for (const o of [fitted, fitted.upper, fitted.basement].flatMap((floor) => floor?.objects ?? [])) {
      const def = services.content.object(o.def);
      if (!def || !services.content.offersTags(def, sleep)) continue;
      const slots = def.slots ?? 1;
      if (slots > 1 && def.interactions.some((i) => i.conceive)) {
        places += 1;
        couples += slots - 1;
      } else places += slots;
    }
    return places + Math.min(couples, residents.couples);
  });
  const sleepers = $derived(sleepersOf(residents));
  const shortage = $derived(bedPlaces !== null && sleepers > bedPlaces ? { beds: bedPlaces, sleepers } : null);
  /** Asking whether to move in anyway (`shortage` can't change while the dialog is open). */
  let confirming = $state(false);
  const neighbours = $derived(
    selected
      ? town.households
          .map((h) => ({ ...h, distance: Math.hypot(town.slots[h.slot].x - selected!.x, town.slots[h.slot].z - selected!.z) }))
          .sort((a, b) => a.distance - b.distance)
          .slice(0, 3)
      : [],
  );
  /** A Creative game: homes are free, money is no object. */
  const creative = $derived(app.mode === 'creative');
  /** What the household has saved: what it can pay for a home (Living). */
  const savings = savingsOf(services.content, household);
  /** The chosen home's price (an empty lot: the land alone), and what it's made of. */
  const parts = $derived(templates && selected ? priceParts(services.content, templates, selected, lot === 'empty' || !house) : null);
  /** The chosen home costs more than the household has saved: it can't move in (Living). */
  const tooDear = $derived(!creative && !!parts && parts.total > savings);
  /** Whether the household can buy plot `s`'s home as it stands (for sale tags and chips; Creative: any). */
  const affordable = (s: PlotSlot) => creative || !templates || homePrice(services.content, templates, s) <= savings;
  /** What the household has to build with once it has bought its home (Living), or "free" (Creative). */
  const budget = $derived(
    creative
      ? 'Free: this is a Creative game'
      : money(fundsAfter(services.content, savings, parts?.total ?? 0) + (lot === 'empty' ? (economy.emptyLotFunds ?? 0) : 0)),
  );
  const mine = `The ${household.name}s`;

  const labels = $derived<LotLabel[]>(
    town.slots.map((s) => {
      const h = town.households.find((x) => x.slot === s.index);
      if (s.kind !== 'house') return { plot: s.index, title: s.name, kind: 'park' };
      if (h) return { plot: s.index, title: `The ${h.household.name}s`, kind: 'home' };
      if (selected?.index === s.index) return { plot: s.index, title: mine, sub: lot === 'empty' ? 'An empty lot' : 'Your new home', kind: 'mine' };
      const price = creative || !templates ? 0 : homePrice(services.content, templates, s);
      return { plot: s.index, title: price ? `For sale · ${money(price)}` : 'For sale', sub: bedrooms(s), kind: 'sale' };
    }),
  );

  function bedrooms(s: PlotSlot): string | undefined {
    const n = templates ? houseTemplate(templates, s.template)?.bedrooms : undefined;
    return n ? `${n} bedroom${n === 1 ? '' : 's'}` : undefined;
  }

  function choose(plot: number | null) {
    const slot = vacant.find((v) => v.index === plot);
    if (!slot) {
      close = false;
      return;
    }
    selectedIndex = slot.index;
    close = true;
  }

  function moveIn() {
    if (tooDear) return;
    if (shortage) {
      confirming = true;
      return;
    }
    start();
  }

  function start() {
    const request = selected && app.homeRequest(selected.index);
    if (!request) return;
    confirming = false;
    menuScene.highlight({});
    app.start(request);
  }

  function back() {
    app.screen = 'create';
  }

  onMount(async () => {
    templates = await loadTemplates();
  });

  // The 3D town shows the chosen lot as the game will (cleared when starting on an empty lot).
  $effect(() => {
    const shown = selected ? app.homeTown(selected.index) : town;
    if (shown) void menuScene.show(shown, selected?.index ?? null);
  });
  $effect(() => {
    menuScene.highlight({ hover, selected: selected?.index ?? null, marked: vacantIds, outlines: false });
  });

  /** Keeps lot tags clear of this element. */
  const avoid = (el: HTMLElement) => ({ destroy: menuScene.avoid(el) });
</script>

<div class="screen scaled">
  <div class="haze" aria-hidden="true"></div>
  <header>
    <button class="btn ghost back" onclick={back}><Icon name="icon.ui.back" size={18} /> Household</button>
    <div class="title">
      <span class="eyebrow">Step 4 of 4 · Choose a home</span>
      <h1>Where will the {household.name}s live?</h1>
    </div>
    <button class="btn primary large move" disabled={!selected || tooDear} title={tooDear ? `The ${household.name}s can't afford this home` : undefined} onclick={moveIn}>Move in →</button>
  </header>

  <main>
    <section class="view">
      <TownStage
        {labels}
        {hover}
        selected={selected?.index ?? null}
        focus={close ? (selected?.index ?? null) : null}
        selectable={(p) => vacantIds.includes(p)}
        onhover={(p) => (hover = p)}
        onselect={choose}
      />
      <div class="hint menu-glass" aria-hidden="true" use:avoid>
        <span class="swatch sale"></span> For sale
        <span class="sep"></span>
        Click a home to look closer · drag to turn · scroll to zoom
      </div>
      {#if close && vacant.length > 1}
        <button class="btn whole" use:avoid onclick={() => (close = false)}><Icon name="icon.ui.town" size={16} /> All homes</button>
      {/if}
    </section>

    <aside class="details menu-glass">
      {#if selected}
        {#if house}
          <Segmented
            label="Start with"
            bind:value={lot}
            options={[
              { value: 'house', label: 'The house' },
              { value: 'empty', label: 'An empty lot' },
            ]}
          />
        {/if}
        <!-- A generated house's lot is cut to fit it: its garden can be made bigger or smaller. A farm keeps its size (its fields and farmyard come with it). -->
        {#if garden && selected.lot !== 'farm'}
          <div class="lot-size">
            <span class="eyebrow">Garden</span>
            <Segmented label="Garden" bind:value={() => garden, setGarden} options={gardens} />
          </div>
        {/if}
        {#key `${selected.index}:${lot}:${selected.template}`}
          <div class="card">
            <span class="eyebrow">{town.name}</span>
            <h2>{selected.name}</h2>
            {#if lot === 'empty' || !house}
              <b class="house">An empty lot</b>
              <p>The house is cleared away: build your own from the ground up.</p>
            {:else}
              <b class="house">{house.name}</b>
              <p>{house.description}</p>
            {/if}
            <dl>
              {#if dims}
                <dt>Lot</dt>
                <dd>{dims.width} × {dims.depth} m{economy.rent ? ` · rent ${money(economy.rent.base + Math.round(economy.rent.perTile * dims.width * dims.depth))} a week` : ''}</dd>
              {/if}
              {#if house && lot === 'house'}
                <dt>Bedrooms</dt>
                <dd>{house.bedrooms}</dd>
              {/if}
              {#if parts && !creative}
                <dt>Price</dt>
                <dd>
                  {money(parts.total)}
                  {#if parts.house || parts.furniture}
                    <small class="parts">land {money(parts.land)} · house {money(parts.house)} · furniture {money(parts.furniture)}</small>
                  {/if}
                </dd>
                <dt>Saved</dt>
                <dd>{money(savings)}</dd>
              {/if}
              <dt>To build with</dt>
              <dd>{budget}</dd>
              <dt>Your household</dt>
              <dd>{household.members.map((m) => m.name).join(', ')}</dd>
            </dl>
            {#if tooDear && parts}
              <p class="shortage" role="status">
                Too expensive: the {household.name}s have {money(savings)} saved, {money(parts.total - savings)} short.{lot === 'empty' ? '' : ' Start on an empty lot, or choose a home that costs less.'}
              </p>
            {/if}
            {#if shortage}
              <p class="shortage" role="status">
                Only {shortage.beds} bed place{shortage.beds === 1 ? '' : 's'} for {shortage.sleepers} sleepers. You can still move in: they'll share a bed or sleep on the floor until you add beds.
              </p>
            {/if}
            {#if neighbours.length}
              <span class="eyebrow">Closest neighbours</span>
              <ul>
                {#each neighbours as n (n.slot)}
                  <li><b>The {n.household.name}s</b> · {town.slots[n.slot].name}</li>
                {/each}
              </ul>
            {/if}
          </div>
        {/key}
        {#if vacant.length > 1}
          <div class="others">
            <span class="eyebrow">Also for sale</span>
            <div class="chips">
              {#each vacant as v (v.index)}
                <button
                  class="chip"
                  class:on={v.index === selected.index}
                  class:dear={!affordable(v)}
                  title={affordable(v) ? undefined : 'Too expensive for them'}
                  onmouseenter={() => (hover = v.index)}
                  onmouseleave={() => (hover = null)}
                  onclick={() => choose(v.index)}
                >
                  {v.name}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      {:else}
        <p>Every house is taken. Go back and remove a neighbour household.</p>
      {/if}
    </aside>
  </main>
</div>

{#if confirming && shortage}
  <Modal title="Not enough beds" onclose={() => (confirming = false)} width={460}>
    <p>
      {shortage.sleepers} people are moving in, but this home has {shortage.beds} bed place{shortage.beds === 1 ? '' : 's'}. Extra residents will share a bed or sleep on the floor until you add beds.
    </p>
    <div class="actions">
      <button class="btn ghost" onclick={() => (confirming = false)}>Cancel</button>
      <button class="btn primary" onclick={start}>Move in anyway</button>
    </div>
  </Modal>
{/if}

<style>
  .screen {
    position: relative;
    /* (Viewport units count in the UI scale.) */
    height: calc(100vh / var(--ui-scale));
    container: setup / inline-size;
    display: grid;
    /* As wide as the screen, whatever the header's buttons would like. */
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto 1fr;
    padding: 20px 28px 24px;
    gap: 14px;
  }
  .haze {
    position: absolute;
    inset: 0 0 auto 0;
    height: 150px;
    background: var(--scene-haze);
    pointer-events: none;
  }
  header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 20px;
  }
  .back {
    color: var(--text-strong);
  }
  .title {
    flex: 1;
  }
  .title .eyebrow {
    color: #4a5163;
  }
  h1 {
    margin: 4px 0 0;
    font-size: 26px;
    font-weight: 750;
    letter-spacing: -0.025em;
    color: var(--text-strong);
  }
  .move {
    padding: 0 28px;
    box-shadow: 0 10px 28px rgba(60, 90, 220, 0.4);
  }
  main {
    position: relative;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr minmax(340px, 420px);
    /* One row as tall as the screen allows: a long side panel scrolls rather than stretching the town view. */
    grid-template-rows: minmax(0, 1fr);
    gap: 24px;
  }
  /* Too narrow for the town beside the panel (a small window or a large UI scale): the town on
     top, the panel under it. */
  @container setup (max-width: 760px) {
    .screen > header {
      flex-wrap: wrap;
      row-gap: 10px;
    }
    /* The title keeps a readable width; the buttons take the next line when they must. */
    .screen > header .title {
      flex: 1 1 300px;
    }
    main {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: minmax(0, 1.2fr) minmax(0, 1fr);
      gap: 14px;
    }
    .details {
      align-self: stretch;
    }
  }
  .view {
    position: relative;
    min-height: 0;
    display: grid;
    grid-template-rows: 1fr auto;
    gap: 14px;
  }
  .hint {
    justify-self: start;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 16px;
    border-radius: var(--radius-pill);
    font-size: 13px;
    font-weight: 550;
    color: #3b4254;
  }
  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--tag-sale);
  }
  .sep {
    width: 1px;
    height: 14px;
    margin: 0 4px;
    background: var(--hairline);
  }
  .whole {
    position: absolute;
    top: 0;
    left: 0;
    animation: rise 320ms var(--ease) both;
  }
  .details {
    min-height: 0;
    max-height: 100%;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 24px;
    align-self: start;
    color: var(--text-strong);
    animation: rise 520ms 80ms var(--ease) both;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    animation: swap 360ms var(--ease) both;
  }
  .details .eyebrow {
    color: #4a5163;
  }
  h2 {
    margin: 0;
    font-size: 26px;
    letter-spacing: -0.02em;
  }
  .house {
    color: var(--accent);
    font-size: 15px;
  }
  p {
    margin: 0;
    color: #4a5163;
    line-height: 1.5;
  }
  .shortage {
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--bad) 12%, transparent);
    color: var(--bad);
    font-weight: 600;
    font-size: 13px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 18px;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 16px;
    margin: 8px 0;
    padding: 12px 14px;
    border-radius: var(--radius-md);
    background: rgba(29, 34, 48, 0.05);
  }
  dt {
    color: var(--text-muted);
  }
  dd {
    margin: 0;
    font-weight: 650;
  }
  ul {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .lot-size {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .others {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 14px;
    border-top: 1px solid var(--hairline);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    padding: 7px 12px;
    border-radius: var(--radius-pill);
    background: rgba(29, 34, 48, 0.06);
    font-weight: 600;
    font-size: 13px;
    transition: background var(--fast) var(--ease);
  }
  .chip:hover {
    background: rgba(29, 34, 48, 0.12);
  }
  .chip.on {
    background: var(--accent);
    color: #fff;
  }
  /* More than the household has saved: still there to look at. */
  .chip.dear {
    opacity: 0.45;
  }
  .parts {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
  @keyframes swap {
    from {
      opacity: 0;
      transform: translateX(10px);
    }
  }
</style>
