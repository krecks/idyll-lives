<script lang="ts">
  /**
   * New game, step 5: the home, chosen on the whole world. The camera opens on the whole land; the
   * chips under it glide to the main town or a village, and "Whole world" back out. The homes for
   * sale the household can afford are lit; two chips beside the legend add the rest ("All for sale"),
   * as a faint stone ring, or leave them out ("Within savings", where a Living game opens). In a
   * Creative game every home for sale is lit. A home under the pointer says its price and bedrooms,
   * and a click chooses it and glides in. The panel says who is moving
   * and what the chosen home is; under it the homes for sale as a list, for the keyboard, and the
   * main control on a narrow screen.
   *
   * A household of the town's (chosen in step 4, `app.playSlot`) has its own home chosen already:
   * it may stay (the game takes it over as it is) or move to a home for sale, its old home then for
   * sale too. On Move in the rest of the homes for sale are filled (`newGame.ts` `moveInRequest`).
   */
  import { onDestroy, onMount } from 'svelte';
  import { menuScene } from '../../game/menuScene';
  import { houseTemplate, loadTemplates, slotDims, vacantSlots, withGarden, type PlotSlot, type Templates } from '../../game/town';
  import { GARDEN_SIZES, type GardenSize } from '../../game/housegen';
  import { lookAtAge, residentsOf, sleepersOf, type SimDraft } from '../../game/household';
  import { homePrice, priceParts } from '../../game/price';
  import { householdFunds, jobTitle, savingsOf } from '../../game/wealth';
  import { settings } from '../../settings/settings.svelte';
  import { tierFor } from '../../game/people';
  import { furnishAhead } from '../../game/townJobs';
  import { householdsAhead, stepLabel } from '../newGame';
  import { app } from '../app.svelte';
  import { money } from '../format';
  import Icon from '../Icon.svelte';
  import Modal from '../kit/Modal.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import SimPreview from '../kit/SimPreview.svelte';
  import TownStage, { type LotLabel } from '../kit/TownStage.svelte';
  import { services } from '../services';
  import SetupShell from './SetupShell.svelte';

  const { content, assets } = services;
  const economy = content.economy;
  let templates = $state.raw<Templates | null>(null);
  /** Changes when a lot's garden is made bigger or smaller (the lots along the street move). */
  const town = $derived(app.town!);
  /** The home of the town's household chosen in step 4 (null: a household made in Creation). */
  const from = app.residentSlot;
  const household = app.mover!;
  const name = household.name;

  const vacant = $derived(vacantSlots(town));
  const vacantIds = $derived(vacant.map((v) => v.index));
  /** What can be chosen: the homes for sale, and the household's own home if it has one. */
  const choosable = $derived(new Set(from === null ? vacantIds : [from, ...vacantIds]));

  /**
   * The home chosen: a household of the town's opens on its own; one made in Creation on the home
   * chosen last time, else (once the prices are known) the one it can afford nearest the town's middle.
   */
  let selectedIndex = $state<number | null>(from ?? (app.homeSlot !== null && vacantSlots(app.town!).some((v) => v.index === app.homeSlot) ? app.homeSlot : null));
  const selected = $derived(selectedIndex === null ? null : (town.slots[selectedIndex] ?? null));
  /** The household stays in its own home (the game takes it over as it is). */
  const staying = $derived(from !== null && selectedIndex === from);
  $effect(() => {
    app.homeSlot = selectedIndex;
  });

  /** The house as it stands, or an empty lot to build on (not for a household staying in its own home). */
  let lot = $state<'house' | 'empty'>(app.emptyLot && from === null ? 'empty' : 'house');
  $effect(() => {
    app.emptyLot = lot === 'empty' && !staying;
  });

  const garden = $derived<GardenSize | undefined>(selected?.garden);
  const dims = $derived(templates && selected ? slotDims(templates, selected) : null);
  function setGarden(size: GardenSize) {
    if (templates && selected) app.town = withGarden(templates, town, selected.index, size);
  }
  /** The garden sizes to choose from: one that doesn't fit beside the neighbours can't be picked. */
  const gardens = $derived(
    GARDEN_SIZES.map((value) => ({
      value,
      label: value[0].toUpperCase() + value.slice(1),
      disabled: !!templates && !!selected && value !== garden && withGarden(templates, town, selected.index, value) === town,
    })),
  );

  const house = $derived(templates && selected ? houseTemplate(templates, selected.template) : undefined);
  const residents = $derived(residentsOf(content, household));
  /**
   * Sleeping places in the house as it would be furnished for this household (empty lot, or their
   * own home: none asked), counted as the simulation counts them (sim-core `lifecycle::beds`): a slot
   * on anything with a sleep interaction, but a couple's bed (one where partners try for a baby) is
   * a place for one unless one of the household's couples sleeps in it.
   */
  const bedPlaces = $derived.by(() => {
    const sleep = content.sleepTags;
    if (!templates || !selected || staying || lot === 'empty' || !sleep.length) return null;
    const fitted = houseTemplate(templates, selected.template, residents);
    if (!fitted) return null;
    let [places, couples] = [0, 0];
    for (const o of [fitted, fitted.upper, fitted.basement].flatMap((floor) => floor?.objects ?? [])) {
      const def = content.object(o.def);
      if (!def || !content.offersTags(def, sleep)) continue;
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

  /** A Creative game: homes are free, money is no object. */
  const creative = $derived(app.mode === 'creative');
  /** What the household has saved: what it can pay for a home (Living). */
  const savings = savingsOf(content, household);
  /** Each home's price as it stands, by lot (their own home's too: what they paid). */
  const prices = $derived(templates ? new Map([...choosable].map((i) => [i, homePrice(content, templates!, town.slots[i])])) : null);
  /** More than the household has saved (Living): still there to look at, as a faint ring once "All for sale" is chosen. */
  const dear = (plot: number) => !creative && plot !== from && (prices?.get(plot) ?? 0) > savings;
  const dearIds = $derived(prices ? vacantIds.filter(dear) : []);
  /** Which homes for sale the map lights (Living, with some too dear): those within their savings, or all of them, the rest faint. */
  let lit = $state<'within' | 'all'>('within');
  /** There is a choice to make: a Living game, the prices known, and some homes beyond their savings. */
  const litChoice = $derived(!creative && dearIds.length > 0);
  const showAll = $derived(!litChoice || lit === 'all');
  const litIds = $derived(showAll ? vacantIds : vacantIds.filter((i) => !dear(i)));
  /** Bought as an empty lot: chosen so, or no house stands on it (as `assembleTown` sees it: money to build with besides). */
  const emptyLot = $derived(!staying && (lot === 'empty' || (!!templates && !!selected && !house)));
  /** The chosen home's price (an empty lot: the land alone), and what it's made of. */
  const parts = $derived(templates && selected && !staying ? priceParts(content, templates, selected, emptyLot) : null);
  /** The chosen home costs more than the household has saved: it can't move in (Living). */
  const tooDear = $derived(!creative && !!parts && parts.total > savings);
  /** What the household has to build with once it has bought its home (Living), or "free" (Creative). */
  const budget = $derived(
    creative ? 'Free: this is a Creative game' : money(staying ? householdFunds(content, savings, prices?.get(from!) ?? 0) : householdFunds(content, savings, parts?.total ?? 0, emptyLot)),
  );

  const neighbours = $derived(
    selected
      ? town.households
          .filter((h) => h.slot !== from && h.slot !== selected!.index)
          .map((h) => ({ ...h, distance: Math.hypot(town.slots[h.slot].x - selected!.x, town.slots[h.slot].z - selected!.z) }))
          .sort((a, b) => a.distance - b.distance)
          .slice(0, 3)
      : [],
  );

  // ---- Where the camera is ---------------------------------------------------------------------

  const areas = $derived(town.open?.areas ?? []);
  const areaName = (s: PlotSlot) => areas[s.area ?? 0]?.name || town.name;
  /** Homes lit in each area (for sale; within their savings while only those are lit). */
  const forSaleIn = $derived(areas.map((_, k) => vacant.filter((s) => (s.area ?? 0) === k && (showAll || !dear(s.index))).length));
  /** What the camera frames with no home in focus: the whole world, or one of its areas. */
  let shot = $state<'world' | { area: number }>('world');
  /** The camera is on the chosen home (false: on `shot`). */
  let close = $state(false);

  /** A chip: the camera glides to an area, or out to the whole world (the player's turns, zoom and pan let go). */
  function jump(to: 'world' | number) {
    close = false;
    shot = to === 'world' ? 'world' : { area: to };
    menuScene.reframe();
  }

  /** A home clicked (on the map, its tag, or the list): chosen, and the camera glides in. Empty ground: back to the shot. */
  function choose(plot: number | null) {
    if (plot === null || !choosable.has(plot)) {
      close = false;
      return;
    }
    selectedIndex = plot;
    if (plot === from) lot = 'house';
    close = true;
  }

  // ---- Tags --------------------------------------------------------------------------------------

  let hover = $state<number | null>(null);
  let hoverOff = 0;
  /**
   * Tags show only for the home looked at, the one chosen and the household's own: crossing a little
   * empty ground on the way to a tag should not drop it at once. Hovering something else changes at once.
   */
  function setHover(plot: number | null): void {
    clearTimeout(hoverOff);
    if (plot !== null || hover === null) {
      hover = plot;
      return;
    }
    hoverOff = setTimeout(() => (hover = null), 160);
  }
  onDestroy(() => clearTimeout(hoverOff));

  function bedrooms(s: PlotSlot): string {
    const n = templates ? houseTemplate(templates, s.template)?.bedrooms : undefined;
    return n ? `${n} bedroom${n === 1 ? '' : 's'}` : 'An empty lot';
  }

  /** The chosen home's tag wins room first, then the one looked at. */
  const ORDER: readonly LotLabel['kind'][] = ['mine', 'sale', 'home', 'park'];

  const labels = $derived.by((): LotLabel[] => {
    const shown = [...new Set([selected?.index ?? null, from, hover])].filter((p): p is number => p !== null && !!town.slots[p]);
    return shown.map((plot): LotLabel => {
      const s = town.slots[plot];
      if (plot === selected?.index) return { plot, title: `The ${name}s`, sub: staying ? 'Their home' : emptyLot ? 'An empty lot' : 'Your new home', kind: 'mine' };
      if (plot === from) return { plot, title: `The ${name}s`, sub: 'Their home now', kind: 'home' };
      const h = town.households.find((x) => x.slot === plot);
      if (h) return { plot, title: `The ${h.household.name}s`, sub: s.name, kind: 'home' };
      if (s.kind !== 'house') return { plot, title: s.name, sub: (s.area ?? 0) > 0 ? areaName(s) : undefined, kind: 'park' };
      const price = creative || !prices ? undefined : prices.get(plot);
      return { plot, title: price === undefined ? 'For sale' : `For sale · ${money(price)}`, sub: `${bedrooms(s)}${dear(plot) ? ' · more than they have' : ''}`, kind: 'sale' };
    });
  });

  // ---- The list ----------------------------------------------------------------------------------

  /** The homes to choose from as the list has them: their own first, then the main town's, then each village's, cheapest first. */
  const rows = $derived(
    [...choosable]
      .map((i) => town.slots[i])
      .sort((a, b) => Number(b.index === from) - Number(a.index === from) || (a.area ?? 0) - (b.area ?? 0) || (prices?.get(a.index) ?? 0) - (prices?.get(b.index) ?? 0) || a.index - b.index),
  );
  let innerWidth = $state(window.innerWidth);
  /** Too narrow for the world beside the panel (as `SetupShell` lays it out): the list is the main control. */
  const narrow = $derived(innerWidth / settings.uiScale <= 760);
  let listOpen = $state(window.innerWidth / settings.uiScale <= 760);
  $effect(() => {
    if (narrow) listOpen = true;
  });
  /** The row the keyboard is on (its home lit on the map, its tag shown). */
  let active = $state(-1);
  let listEl = $state<HTMLElement>();

  function moveActive(to: number) {
    if (!rows.length) return;
    active = Math.max(0, Math.min(rows.length - 1, to));
    const plot = rows[active].index;
    setHover(plot);
    listEl?.querySelector(`#home-${plot}`)?.scrollIntoView({ block: 'nearest' });
  }

  function listKey(e: KeyboardEvent) {
    const step: Record<string, number> = { ArrowDown: 1, ArrowUp: -1, PageDown: 8, PageUp: -8 };
    if (e.key in step) moveActive((active < 0 ? 0 : active) + step[e.key]);
    else if (e.key === 'Home') moveActive(0);
    else if (e.key === 'End') moveActive(rows.length - 1);
    else if ((e.key === 'Enter' || e.key === ' ') && active >= 0) choose(rows[active].index);
    else return;
    e.preventDefault();
  }

  // ---- Who is moving -----------------------------------------------------------------------------

  /** "34 · Junior Developer": a resident's age and what they do. */
  function about(m: SimDraft): string {
    const retired = m.age >= content.retireAt;
    const work = !content.isAdult(m.age)
      ? (content.stageOf(m.age)?.label ?? '')
      : m.job
        ? `${retired ? 'Retired ' : ''}${jobTitle(content, m.job)}`
        : retired
          ? 'Retired'
          : 'Between jobs';
    return `${m.age} · ${work}`;
  }

  /** "Partner of Ben", "Child of Ann and Ben", "Sibling of Joe": a member's bonds within the household. */
  function bondsOf(uid: string): string {
    const nameOf = (u: string) => household.members.find((m) => m.uid === u)?.name;
    const names = (list: (string | undefined)[]) => list.filter(Boolean).join(' and ');
    const others = (preset: string, as: 'a' | 'b' | 'either') =>
      household.bonds
        .filter((b) => b.preset === preset && (as === 'either' ? b.a === uid || b.b === uid : b[as] === uid))
        .map((b) => nameOf(b.a === uid ? b.b : b.a));
    const lines = [
      ['Partner of', others('partners', 'either')],
      // (A parent bond runs from the child, `a`, to the parent, `b`.)
      ['Child of', others('parent', 'a')],
      ['Parent of', others('parent', 'b')],
      ['Sibling of', others('siblings', 'either')],
    ] as const;
    return lines
      .filter(([, list]) => list.some(Boolean))
      .map(([label, list]) => `${label} ${names([...list])}`)
      .join(' · ');
  }

  // ---- Moving in ---------------------------------------------------------------------------------

  function moveIn() {
    if (!selected || tooDear) return;
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
    app.screen = from === null ? 'create' : 'household';
  }

  onMount(async () => {
    templates = await loadTemplates();
    // The houses Move in fills furnished ahead in the town worker while the player chooses (left
    // running past this screen: Move in's town file waits behind it and finds them made).
    const seed = app.peopleSeed;
    if (seed !== undefined) {
      const ahead = householdsAhead(content, assets, templates, town, seed, household, tierFor(settings.graphics));
      furnishAhead(content, town, ahead).catch(() => {});
    }
    if (selectedIndex !== null) return;
    const a = areas[0];
    const [cx, cz] = a ? [a.x + a.w / 2, a.z + a.d / 2] : [town.width / 2, town.depth / 2];
    const near = (s: PlotSlot) => ((s.area ?? 0) === 0 ? 0 : 1e6) + Math.hypot(s.x - cx, s.z - cz);
    selectedIndex = vacant.filter((s) => !dear(s.index)).sort((p, q) => near(p) - near(q))[0]?.index ?? null;
  });

  // The 3D town shows the chosen home as the game will (cleared when starting on an empty lot).
  $effect(() => {
    const plot = selected?.index ?? null;
    const shown = plot === null || plot === from ? town : app.homeTown(plot);
    if (shown) void menuScene.show(shown, plot ?? from);
  });
  $effect(() => {
    menuScene.highlight({ hover, selected: selected?.index ?? null, marked: litIds, dear: litChoice && showAll ? dearIds : [], outlines: false });
  });
  $effect(() => () => menuScene.highlight({}));

  /** Keeps lot tags clear of this element. */
  const avoid = (el: HTMLElement) => ({ destroy: menuScene.avoid(el) });
</script>

<svelte:window bind:innerWidth />

{#snippet head()}
  <h1 class="title">Where will the {name}s live?</h1>
{/snippet}

{#snippet panel()}
  {#if narrow}{@render homes()}{/if}
  <section class="group who" aria-labelledby="who-head">
    <h2 class="eyebrow" id="who-head">{from === null ? 'Moving in' : `Living at ${town.slots[from]?.name ?? ''}`}</h2>
    <span class="family">The {name}s</span>
    <ul class="members">
      {#each household.members as m (m.uid)}
        {@const bonds = bondsOf(m.uid)}
        <li>
          <span class="face"><SimPreview appearance={lookAtAge(content, assets, m.appearance, m.age)} gender={m.gender} size={44} /></span>
          <span class="person">
            <span><b>{m.name}</b> <span class="about">{about(m)}</span></span>
            {#if bonds}<span class="bond">{bonds}</span>{/if}
            {#if m.traits.length || m.perks.length}
              <span class="tags">
                {#each m.traits as t (t)}<span class="trait">{content.trait(t)?.label ?? t}</span>{/each}
                {#each m.perks as p (p)}<span class="perk">{content.perk(p)?.label ?? p}</span>{/each}
              </span>
            {/if}
          </span>
        </li>
      {/each}
    </ul>
    {#if from !== null}
      <p class="hint">Settled residents: most already have a job and know a few of the neighbours. They can stay, or move to a home for sale.</p>
    {/if}
  </section>

  <section class="group home" aria-labelledby="home-head">
    {#if selected}
      <h2 class="eyebrow" id="home-head">{staying ? 'Their home' : 'The home chosen'}</h2>
      {#if house && !staying}
        <Segmented
          fill
          label="Start with"
          bind:value={lot}
          options={[
            { value: 'house', label: 'The house' },
            { value: 'empty', label: 'An empty lot' },
          ]}
        />
      {/if}
      <!-- A generated house's lot is cut to fit it: its garden can be made bigger or smaller. A farm keeps its size (its fields and farmyard come with it). -->
      {#if garden && selected.lot !== 'farm' && !staying}
        <div class="lot-size">
          <span class="eyebrow sub">Garden</span>
          <Segmented label="Garden" bind:value={() => garden, setGarden} options={gardens} />
        </div>
      {/if}
      {#key `${selected.index}:${lot}:${selected.template}`}
        <div class="card">
          <span class="eyebrow sub">{areaName(selected)}</span>
          <h3>{selected.name}</h3>
          {#if emptyLot}
            <b class="house">An empty lot</b>
            <p>The house is cleared away: build your own from the ground up.</p>
          {:else if house}
            <b class="house">{house.name}</b>
            <p>{house.description}</p>
          {/if}
          <dl>
            {#if dims}
              <dt>Lot</dt>
              <dd>{dims.width} × {dims.depth} m{economy.rent ? ` · rent ${money(economy.rent.base + Math.round(economy.rent.perTile * dims.width * dims.depth))} a week` : ''}</dd>
            {/if}
            {#if house && !emptyLot}
              <dt>Bedrooms</dt>
              <dd>{house.bedrooms}</dd>
            {/if}
            {#if staying && !creative && prices?.has(from!)}
              <dt>Bought for</dt>
              <dd>{money(prices.get(from!)!)}</dd>
            {/if}
            {#if parts && !creative}
              <dt>Price</dt>
              <dd>
                {money(parts.total)}
                {#if parts.house || parts.furniture}
                  <small class="parts">land {money(parts.land)} · house {money(parts.house)} · furniture {money(parts.furniture)}</small>
                {/if}
              </dd>
            {/if}
            {#if !creative}
              <dt>Saved</dt>
              <dd>{money(savings)}</dd>
            {/if}
            <dt>{staying ? 'To spend' : 'To build with'}</dt>
            <dd>{budget}</dd>
          </dl>
          {#if tooDear && parts}
            <p class="warn" role="status">
              Too expensive: the {name}s have {money(savings)} saved, {money(parts.total - savings)} short.{lot === 'empty' ? '' : ' Start on an empty lot, or choose a home that costs less.'}
            </p>
          {/if}
          {#if shortage}
            <p class="warn" role="status">
              Only {shortage.beds} bed place{shortage.beds === 1 ? '' : 's'} for {shortage.sleepers} sleepers. You can still move in: they'll share a bed or sleep on the floor until you add beds.
            </p>
          {/if}
          {#if from !== null && !staying}
            <p class="hint">Their home at {town.slots[from]?.name} goes up for sale, and another household may move in.</p>
          {/if}
          {#if neighbours.length}
            <span class="eyebrow sub">Closest neighbours</span>
            <ul class="near">
              {#each neighbours as n (n.slot)}
                <li><b>The {n.household.name}s</b> · {town.slots[n.slot].name}</li>
              {/each}
            </ul>
          {/if}
        </div>
      {/key}
    {:else if vacant.length}
      <h2 class="eyebrow" id="home-head">A home</h2>
      <p class="hint">
        Choose one of the {vacant.length} homes for sale on the map{creative ? '' : `: ${vacant.length - dearIds.length} within the ${money(savings)} they've saved`}. The chips under the map take you to the town and each village.
      </p>
    {:else}
      <p class="hint">Every house is taken. Go back and remove a neighbour household.</p>
    {/if}
    <p class="hint after">Once you move in, the other homes for sale find households too, but for a few.</p>
  </section>
  {#if !narrow}{@render homes()}{/if}
{/snippet}

<!-- The homes as a list: the keyboard's way round them, and the main control on a narrow screen (first in the panel there). -->
{#snippet homes()}
  <section class="group list">
    <button class="toggle" aria-expanded={listOpen} aria-controls="home-list" onclick={() => (listOpen = !listOpen)}>
      <span class="chevron" class:open={listOpen} aria-hidden="true"><Icon name="icon.ui.chevron" size={14} /></span>
      <span class="eyebrow">{from === null ? 'Homes for sale' : 'Homes to choose from'}</span>
      <span class="count">{rows.length}</span>
    </button>
    {#if listOpen}
      <ul
        id="home-list"
        class="homes"
        role="listbox"
        tabindex="0"
        aria-label="Homes to choose from"
        aria-activedescendant={active >= 0 && rows[active] ? `home-${rows[active].index}` : undefined}
        bind:this={listEl}
        onkeydown={listKey}
        onfocus={() => active < 0 && moveActive(Math.max(0, rows.findIndex((r) => r.index === selected?.index)))}
        onblur={() => setHover(null)}
      >
        {#each rows as r, i (r.index)}
          <!-- (The keys are the list's: an option of a listbox is not a stop of its own.) -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li
            id="home-{r.index}"
            role="option"
            aria-selected={r.index === selected?.index}
            class:active={i === active}
            class:dear={dear(r.index)}
            class:own={r.index === from}
            onclick={() => ((active = i), choose(r.index))}
            onmouseenter={() => setHover(r.index)}
            onmouseleave={() => setHover(null)}
          >
            <span class="dot" aria-hidden="true"></span>
            <span class="addr">
              <b>{r.name}</b>
              <small>{r.index === from ? 'Their home' : `${areaName(r)} · ${bedrooms(r)}`}</small>
            </span>
            <span class="price">
              {#if r.index !== from && !creative && prices}{money(prices.get(r.index) ?? 0)}{/if}
              {#if dear(r.index)}<small>too dear</small>{/if}
            </span>
          </li>
        {/each}
      </ul>
      <p class="hint keys"><kbd>↑</kbd> <kbd>↓</kbd> look along the list · <kbd>Enter</kbd> choose</p>
    {/if}
  </section>
{/snippet}

{#snippet stage()}
  <TownStage
    {labels}
    order={ORDER}
    {hover}
    selected={selected?.index ?? null}
    focus={close ? (selected?.index ?? null) : null}
    {shot}
    selectable={(p) => choosable.has(p)}
    onhover={setHover}
    onselect={choose}
  />
  <div class="below" use:avoid>
    <div class="jump" role="toolbar" aria-label="Look at">
      <button class="chip" class:on={!close && shot === 'world'} aria-pressed={!close && shot === 'world'} onclick={() => jump('world')}><Icon name="icon.ui.town" size={14} /> Whole world</button>
      {#each areas as a, k (k)}
        {@const on = !close && typeof shot === 'object' && shot.area === k}
        <button class="chip" class:on aria-pressed={on} onclick={() => jump(k)}>
          {a.name || town.name}
          {#if forSaleIn[k]}<span class="n" title="{forSaleIn[k]} {showAll ? 'for sale' : 'within their savings'}">{forSaleIn[k]}</span>{/if}
        </button>
      {/each}
    </div>
    <div class="key">
      {#if litChoice}
        <div class="jump" role="radiogroup" aria-label="Homes lit on the map">
          <button class="chip" class:on={lit === 'within'} role="radio" aria-checked={lit === 'within'} onclick={() => (lit = 'within')}>
            Within savings <span class="n">{vacant.length - dearIds.length}</span>
          </button>
          <button class="chip" class:on={lit === 'all'} role="radio" aria-checked={lit === 'all'} onclick={() => (lit = 'all')}>
            All for sale <span class="n">{vacant.length}</span>
          </button>
        </div>
      {/if}
      <div class="legend menu-glass" aria-hidden="true">
        <span class="swatch sale"></span>
        {litChoice ? 'Within their savings' : 'For sale'}
        {#if litChoice && showAll}<span class="swatch dear"></span> More than they have{/if}
        <span class="sep"></span>
        Click a home to look closer · drag to turn · scroll to zoom · middle-drag to pan
      </div>
    </div>
  </div>
{/snippet}

{#snippet figures()}
  <span><b>{vacant.length}</b> for sale</span>
  {#if !creative && templates}<span><b>{vacant.length - dearIds.length}</b> within their savings</span>{/if}
  {#if areas.length > 1}<span><b>{areas.length - 1}</b> {areas.length === 2 ? 'village' : 'villages'}</span>{/if}
{/snippet}

{#snippet actions()}
  <button class="btn primary move-in" disabled={!selected || tooDear} title={!selected ? 'Choose a home first' : tooDear ? `The ${name}s can't afford this home` : undefined} onclick={moveIn}>
    {staying ? `Play the ${name}s →` : 'Move in →'}
  </button>
{/snippet}

<SetupShell label="The home" step={stepLabel(5)} back={{ label: 'Household', go: back }} {head} {panel} {stage} stats={figures} {actions} />

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
  .title {
    margin: 2px 0 0;
    font-size: 22px;
    font-weight: 750;
    letter-spacing: -0.02em;
    line-height: 1.2;
    color: var(--text-strong);
  }
  .family {
    font-size: 18px;
    font-weight: 750;
    letter-spacing: -0.01em;
    color: var(--text-strong);
  }
  .members {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .members li {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .face {
    flex: none;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    overflow: hidden;
    background: var(--surface-muted);
    border: 2px solid var(--surface);
    box-shadow: var(--shadow-sm);
    display: grid;
    place-items: start center;
  }
  .person {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 13px;
    color: var(--text-strong);
  }
  .about {
    font-size: 12px;
    color: var(--text-muted);
  }
  .bond {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
  }
  .trait,
  .perk {
    padding: 1px 7px;
    border-radius: var(--radius-pill);
    font-size: 11px;
    font-weight: 600;
    background: rgba(29, 34, 48, 0.06);
    color: #3b4254;
  }
  .perk {
    background: rgba(233, 162, 59, 0.16);
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    animation: swap 360ms var(--ease) both;
  }
  h3 {
    margin: 0;
    font-size: 20px;
    letter-spacing: -0.02em;
    color: var(--text-strong);
  }
  .house {
    color: var(--accent);
    font-size: 14px;
  }
  .card p {
    margin: 0;
    font-size: 13px;
    color: #4a5163;
    line-height: 1.45;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 5px 14px;
    margin: 6px 0;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: rgba(29, 34, 48, 0.05);
    font-size: 13px;
    color: var(--text-strong);
  }
  dt {
    color: var(--text-muted);
  }
  dd {
    margin: 0;
    font-weight: 650;
  }
  .parts {
    display: block;
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-muted);
  }
  .card .warn {
    padding: 8px 10px;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--bad) 12%, transparent);
    color: var(--bad);
    font-weight: 600;
    font-size: 12.5px;
  }
  .near {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 13px;
    color: var(--text-strong);
  }
  .lot-size {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .after {
    font-style: italic;
  }
  /* The list: a header that opens and closes it, then the homes, the keyboard's way round them. */
  .toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
    margin: -4px 0;
    background: none;
    border: none;
    text-align: left;
    color: var(--text-strong);
  }
  /* (The icon points up: right while the list is closed, down while it is open.) */
  .chevron {
    display: grid;
    transform: rotate(90deg);
    transition: transform var(--fast) var(--ease);
  }
  .chevron.open {
    transform: rotate(180deg);
  }
  .count {
    margin-left: auto;
    padding: 1px 8px;
    border-radius: var(--radius-pill);
    background: rgba(29, 34, 48, 0.07);
    font-size: 12px;
    font-weight: 700;
  }
  .homes {
    list-style: none;
    margin: 0;
    padding: 3px;
    max-height: 320px;
    overflow: auto;
    overscroll-behavior: contain;
    display: flex;
    flex-direction: column;
    gap: 1px;
    border-radius: var(--radius-md);
    background: rgba(29, 34, 48, 0.04);
    outline: none;
  }
  .homes:focus-visible {
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .homes li {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 6px 9px;
    border-radius: calc(var(--radius-md) - 3px);
    font-size: 13px;
    color: var(--text-strong);
    cursor: pointer;
  }
  .homes li:hover,
  .homes li.active {
    background: rgba(255, 255, 255, 0.75);
  }
  .homes:focus-visible li.active {
    box-shadow: inset 0 0 0 2px var(--accent);
  }
  .homes li[aria-selected='true'] {
    background: var(--accent);
    color: #fff;
  }
  .homes .dot {
    flex: none;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: rgb(255, 199, 97);
  }
  .homes li.dear .dot {
    background: none;
    box-shadow: inset 0 0 0 1.5px rgb(170, 164, 152);
  }
  .homes li.own .dot {
    background: var(--tag-home);
  }
  .addr {
    min-width: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
    line-height: 1.2;
  }
  .addr b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .addr small,
  .price small {
    font-size: 11px;
    color: var(--text-muted);
  }
  .price {
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
    line-height: 1.2;
  }
  .homes li.dear .price {
    color: var(--text-muted);
  }
  .homes li[aria-selected='true'] small,
  .homes li[aria-selected='true'] .price {
    color: rgba(255, 255, 255, 0.85);
  }
  .keys kbd {
    font-size: 11px;
  }
  /* Under the world: the chips that glide to the town and each village, and what the colours say. */
  .below {
    position: absolute;
    left: 0;
    bottom: 0;
    z-index: 1;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    animation: rise 520ms 120ms var(--ease) both;
  }
  .jump {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: var(--radius-pill);
    background: var(--glass-menu);
    border: 1px solid var(--glass-menu-border);
    box-shadow: var(--shadow-sm);
    backdrop-filter: blur(12px) saturate(1.4);
    -webkit-backdrop-filter: blur(12px) saturate(1.4);
    font-size: 13px;
    font-weight: 650;
    color: var(--text-strong);
    transition: background var(--fast) var(--ease);
  }
  .chip:hover {
    background: var(--surface);
  }
  .chip.on {
    background: var(--accent);
    border-color: transparent;
    color: #fff;
  }
  .chip .n {
    min-width: 18px;
    padding: 0 5px;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--tag-sale) 30%, transparent);
    font-size: 11px;
    font-weight: 750;
    text-align: center;
  }
  .chip.on .n {
    background: rgba(255, 255, 255, 0.25);
  }
  .key {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    border-radius: var(--radius-pill);
    font-size: 12px;
    font-weight: 550;
    color: #3b4254;
  }
  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: rgb(255, 199, 97);
  }
  /* The faint stone ring the map draws round a home they can't afford. */
  .swatch.dear {
    margin-left: 6px;
    background: none;
    box-shadow: inset 0 0 0 1.5px rgb(170, 164, 152);
  }
  .sep {
    width: 1px;
    height: 14px;
    margin: 0 4px;
    background: var(--hairline);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 18px;
  }
  /* Too narrow for the world beside the panel: the hint about the mouse goes, and the list (first in
     the panel) is as long as the panel lets it be. */
  @container setup (max-width: 760px) {
    .legend {
      display: none;
    }
    .homes {
      max-height: none;
    }
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
