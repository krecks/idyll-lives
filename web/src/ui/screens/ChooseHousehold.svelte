<script lang="ts">
  /**
   * New game, step 4: the household. The town's residents move in here (the first wave, from the
   * world code's people seed), and some of their households are offered as cards: choose one to play
   * (step 5 then opens on their home), or make your own in Creation. The game's choices (Living or
   * Creative, the lifespan) and how many neighbours live here are in the side panel.
   *
   * The town is lived in again only when step 3's built town or the people's seed changed (`peopleKey`):
   * back and forth, the same households and the same cards stay, and a household made in Creation
   * stays too.
   */
  import { onDestroy, onMount } from 'svelte';
  import { lookAtAge, type SimDraft } from '../../game/household';
  import { flatName } from '../../game/flats';
  import { reducedMotion } from '../../game/host';
  import { menuScene } from '../../game/menuScene';
  import { householdsAtMost, peopleStreams, populate } from '../../game/people';
  import { flatPrice, homePrice } from '../../game/price';
  import { flatsIdOf, loadTemplates, type NeighbourhoodDraft, type Templates } from '../../game/town';
  import { homeKey, homeOffers, sameHome, type HomeRef } from '../homes';
  import { app } from '../app.svelte';
  import { lifespanHint, money } from '../format';
  import Icon from '../Icon.svelte';
  import { offerPage, peopleKey, peopleSeedOf, startCode, stepLabel } from '../newGame';
  import Segmented from '../kit/Segmented.svelte';
  import SimPreview from '../kit/SimPreview.svelte';
  import TownStage, { type LotLabel } from '../kit/TownStage.svelte';
  import { services } from '../services';
  import SetupShell from './SetupShell.svelte';

  const { content, assets } = services;
  /** A new seed: other neighbours every time. */
  const roll = () => (Math.random() * 2 ** 31) >>> 0;

  let templates = $state.raw<Templates | null>(null);
  let town = $state.raw<NeighbourhoodDraft | null>(app.town);
  /** The lot under the pointer (3D, a tag or a card), and the card chosen (its home: a house's lot or a flat): the camera goes to its area. */
  let hover = $state<number | null>(null);
  let focus = $state<HomeRef | null>(app.play !== null && app.offer.some((o) => sameHome(o, app.play)) ? app.play : null);
  let hoverOff = 0;
  let cards = $state<HTMLElement>();

  const code = $derived(app.code ?? startCode('country', 0));
  const streams = $derived(peopleStreams(peopleSeedOf(code)));
  const offer = $derived(app.offer);
  /** Every home (`homes.ts` `homeOffers`: houses and flats), once the templates are known. */
  const homes = $derived(town && templates ? homeOffers(content, templates, town) : []);
  /** The homes as steps 2 and 3 count them, farms apart (a farm's house is lived in, so the slider above still counts it). */
  const homeCount = $derived(homes.filter((h) => h.lot.lot !== 'farm').length);
  /** The Neighbours slider's end: as many households as the first wave moves in at most (`populate` stops there too). */
  const mostHouseholds = $derived(householdsAtMost(homes.length));
  const forSale = $derived(town ? homes.length - new Set(town.households.map(homeKey)).size : 0);
  const population = $derived(town?.households.reduce((n, h) => n + h.household.members.length, 0) ?? 0);
  /** Every household is on the cards already: "Show others" has none to show. */
  const allShown = $derived(!town || town.households.length <= offer.length);

  /** The offered households, in the cards' order. */
  const offeredHouseholds = $derived.by(() => {
    const t = town;
    if (!t) return [];
    const byHome = new Map(t.households.map((h) => [homeKey(h), h]));
    return offer.flatMap((home) => byHome.get(homeKey(home)) ?? []);
  });
  const chosen = $derived(focus === null ? undefined : offeredHouseholds.find((h) => sameHome(h, focus)));
  /** The lots of the homes offered (a building's once for all its flats offered). */
  const offeredLots = $derived([...new Set(offer.map((o) => o.slot))]);

  /**
   * Each lived-in home's price band among the homes lived in: the cheapest third one coin, the
   * dearest three. (Among them, not all the town's: the first wave moves into the dearer homes, so
   * nearly all would be the dearest third of the town.) Worked out once a town.
   */
  const bands = $derived.by(() => {
    const t = town;
    const map = new Map<string, { band: number; price: number }>();
    if (!t || !templates) return map;
    const priceOf = (h: HomeRef) => (h.flat === undefined ? homePrice(content, templates!, t.slots[h.slot]) : flatPrice(content, templates!, { template: flatsIdOf(t.slots[h.slot]) }, h.flat));
    const prices = t.households.flatMap((h) => (t.slots[h.slot] ? [{ key: homeKey(h), price: priceOf(h) }] : []));
    const sorted = prices.map((p) => p.price).sort((a, b) => a - b);
    const at = (q: number) => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * q))] ?? 0;
    const [low, high] = [at(1 / 3), at(2 / 3)];
    for (const p of prices) map.set(p.key, { price: p.price, band: p.price < low ? 1 : p.price < high ? 2 : 3 });
    return map;
  });
  const BAND_LABEL = ['', 'Modest home', 'Family home', 'Fine home'];
  const currency = $derived(content.economy.currency ?? '$');

  /** Where a home is: its street (a flat: its name, "Flat 2, Mill Court"), and the town or the village it is in (a village says so). */
  function whereOf(home: HomeRef): { street: string; place: string; village: boolean } {
    const s = town?.slots[home.slot];
    const area = s?.area ?? 0;
    const name = area === 0 ? (town?.name ?? '') : (town?.open?.areas[area]?.name ?? '');
    return { street: s ? (home.flat === undefined ? s.name : flatName(s.name, home.flat)) : '', place: name, village: area !== 0 };
  }

  /** A resident in brief: "Ben, 34", and their first two traits. */
  function about(m: SimDraft): { who: string; traits: string; all: string } {
    const labels = m.traits.map((t) => content.trait(t)?.label).filter((l): l is string => !!l);
    return { who: `${m.name}, ${m.age}`, traits: labels.slice(0, 2).join(', '), all: labels.join(', ') };
  }

  /** The tags over the town: the households offered (what this step is for), and the main town's public places. */
  const labels = $derived.by((): LotLabel[] => {
    const t = town;
    if (!t) return [];
    // (One tag a lot: a building with several households offered names the one chosen, else the first, and how many more.)
    const offered = new Map<number, typeof offeredHouseholds>();
    for (const h of offeredHouseholds) offered.set(h.slot, [...(offered.get(h.slot) ?? []), h]);
    return t.slots.flatMap((s): LotLabel[] => {
      const here = offered.get(s.index);
      if (here) {
        const h = here.find((x) => sameHome(x, focus)) ?? here[0];
        const more = here.length > 1 ? ` · and ${here.length - 1} more` : '';
        return [{ plot: s.index, title: `The ${h.household.name}s`, sub: `${whereOf(h).street}${more}`, kind: 'home' }];
      }
      if (s.kind === 'house') return [];
      const area = s.area ?? 0;
      if (area === 0) return [{ plot: s.index, title: s.name, kind: 'park' }];
      return s.index === hover || s.index === focus?.slot ? [{ plot: s.index, title: s.name, sub: t.open?.areas[area]?.name, kind: 'park' }] : [];
    });
  });

  /**
   * A tag shown only while its lot is looked at floats above the lot a moment after the pointer
   * leaves: crossing a little empty ground on the way to it should not drop the tag at once.
   * Hovering something else changes at once.
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

  /** Step 3's built town, nobody living there yet. */
  const empty = (): NeighbourhoodDraft | null => app.world?.draft ?? (town ? { ...town, households: [] } : null);

  /** The town lived in from the people's seed (`count` households, else as many as the first wave takes), and its first offer. */
  function populateTown(count?: number): void {
    const bare = empty();
    if (!templates || !bare) return;
    // No loading screen: that is only for starting a game. The cards draw their portraits as they come.
    town = populate(content, assets, templates, bare, count, streams);
    app.offerPage = 0;
    app.offer = offerPage(town, streams, 0);
    focus = null;
    keep();
  }

  /** Other neighbours: a people's seed of their own, in the code (so step 1's Copy carries it). */
  function newNeighbours(): void {
    app.code = { ...code, peopleSeed: roll() };
    populateTown();
  }

  /** The next households from the offer's stream. */
  function showOthers(): void {
    if (!town) return;
    app.offerPage += 1;
    app.offer = offerPage(town, streams, app.offerPage);
    focus = null;
    cards?.scrollTo({ top: 0, behavior: reducedMotion() ? 'auto' : 'smooth' });
  }

  /** The town as the next steps should see it, and what it was lived in from. */
  function keep(): void {
    if (!town) return;
    app.town = town;
    if (app.world) app.townOf = peopleKey(app.world.key, peopleSeedOf(code));
    if (app.play !== null && !town.households.some((h) => sameHome(h, app.play))) app.play = null;
  }

  /** Play the household of `home`: on to step 5, which opens on their home. */
  function choose(home: HomeRef): void {
    if (!town) return;
    keep();
    app.play = home;
    menuScene.highlight({});
    app.screen = 'home';
  }

  /** Make (or change) the player's own household in Creation. */
  function create(): void {
    keep();
    app.play = null;
    app.screen = 'create';
  }

  /** The household made in Creation, on to step 5. */
  function chooseOwn(): void {
    if (!app.household) return;
    keep();
    app.play = null;
    menuScene.highlight({});
    app.screen = 'home';
  }

  /** A card picked (again: let go): the camera goes to its area, its card into view. */
  function select(home: HomeRef | null): void {
    focus = home !== null && !sameHome(home, focus) && offer.some((o) => sameHome(o, home)) ? home : null;
    if (focus === null) return;
    const card = cards?.querySelector(`[data-home="${homeKey(focus)}"]`);
    card?.scrollIntoView({ block: 'nearest', behavior: reducedMotion() ? 'auto' : 'smooth' });
  }

  /**
   * A home picked on the map: its lot's household offered (a building's: the next of those offered there, in the
   * cards' order; past the last, or a house's picked again, let go).
   */
  function selectLot(plot: number | null): void {
    const here = plot === null ? [] : offer.filter((o) => o.slot === plot);
    const at = here.findIndex((o) => sameHome(o, focus));
    select(here[at + 1] ?? focus);
  }

  /** What the camera frames: the area of the household chosen, else the whole world. */
  const shot = $derived.by((): 'world' | { area: number } => {
    const s = focus === null ? undefined : town?.slots[focus.slot];
    return s ? { area: s.area ?? 0 } : 'world';
  });

  onMount(async () => {
    templates = await loadTemplates();
    const laid = app.world;
    if (!laid) {
      // No town built yet: that is step 3 (or step 2 or 1, with no layout or land either).
      app.screen = app.layout ? 'town' : app.land ? 'streets' : 'land';
      return;
    }
    // The town its neighbours were made for, with the same people's seed: they stay (under the
    // name it has now), and so do the cards. Else (another town, or other neighbours): lived in again.
    if (town && app.townOf === peopleKey(laid.key, peopleSeedOf(code))) {
      if (town.name !== laid.draft.name) town = { ...town, name: laid.draft.name };
      if (!app.offer.length) app.offer = offerPage(town, streams, app.offerPage);
      keep();
    } else populateTown();
  });

  // The 3D town follows the draft. Only a new layout needs it rebuilt: other neighbours leave the
  // streets and houses as they are (the tags and the lit lots follow separately).
  let layout = '';
  $effect(() => {
    const key = town ? `${town.seed}:${town.width}x${town.depth}:${town.slots.map((s) => `${s.x},${s.z},${s.template}`).join(';')}` : '';
    if (key === layout) return;
    layout = key;
    hover = null;
    if (town) void menuScene.show(town);
  });

  // The offered households' homes lit; the one under the pointer and the one chosen more so.
  $effect(() => {
    menuScene.highlight({ hover, selected: focus?.slot ?? null, marked: offeredLots, outlines: true });
  });
  $effect(() => () => menuScene.highlight({}));

  /** Keeps lot tags clear of this element. */
  const avoid = (el: HTMLElement) => ({ destroy: menuScene.avoid(el) });
</script>

{#snippet head()}
  <h1 class="town-name">{town?.name ?? app.world?.draft.name ?? ''}</h1>
  <p class="lede">Play one of the households who live here, or make your own.</p>
{/snippet}

{#snippet panel()}
  <section class="group" aria-labelledby="game-head">
    <h2 class="eyebrow" id="game-head">Your game</h2>
    <div class="options">
      <div class="option">
        <span class="eyebrow sub">Game</span>
        <Segmented
          fill
          label="Game"
          bind:value={app.mode}
          options={[
            { value: 'living', label: 'Living' },
            { value: 'creative', label: 'Creative' },
          ]}
        />
        <p class="hint">{app.mode === 'creative' ? 'Building is free. No rates or bills at home.' : 'Your residents earn the money to build with.'}</p>
      </div>
      <div class="option">
        <span class="eyebrow sub">Lifespan</span>
        <Segmented
          fill
          label="Lifespan"
          bind:value={app.lifespan}
          options={[
            { value: 'off', label: 'Off' },
            { value: 'short', label: 'Short' },
            { value: 'normal', label: 'Normal' },
            { value: 'long', label: 'Long' },
          ]}
        />
        <p class="hint">{lifespanHint(app.lifespan)}</p>
      </div>
    </div>
  </section>
  <section class="group" aria-labelledby="neighbours-head">
    <h2 class="eyebrow" id="neighbours-head">Neighbours</h2>
    {#if town && templates}
      <label class="option">
        <span class="eyebrow sub">Households living here · {town.households.length}</span>
        <input
          type="range"
          min="0"
          max={mostHouseholds}
          value={town.households.length}
          aria-label="Households living here"
          onchange={(e) => populateTown(Number((e.currentTarget as HTMLInputElement).value))}
        />
      </label>
      <p class="hint">{population} residents in {town.households.length} households. More move into the homes left once you have chosen yours.</p>
    {/if}
    <div class="row">
      <button class="btn" disabled={!templates || !town} onclick={newNeighbours}><Icon name="icon.ui.dice" size={16} /> New neighbours</button>
    </div>
  </section>
{/snippet}

{#snippet stage()}
  <div class="split">
    {#if town}
      <TownStage {labels} {hover} selected={focus?.slot ?? null} focus={null} {shot} selectable={(p) => offeredLots.includes(p)} onhover={setHover} onselect={selectLot} />
    {:else}
      <div></div>
    {/if}
    {#if town}
      <section class="offer menu-glass" use:avoid aria-label="Households to play">
        <header class="offer-head">
          <span class="eyebrow">Who you could play</span>
          <button class="btn ghost small" disabled={allShown} title={allShown ? 'Every household here is shown' : undefined} onclick={showOthers}>
            <Icon name="icon.ui.dice" size={15} /> Show others
          </button>
        </header>
        <div class="cards" bind:this={cards}>
          <!-- The player's own household: made in Creation (or to be). -->
          <article class="card own" class:made={!!app.household}>
            {#if app.household}
              {@const own = app.household}
              <button class="pick" aria-label="Choose your own household, the {own.name}s" onclick={chooseOwn}>
                <span class="faces">
                  {#each own.members as m (m.uid)}
                    <span class="face" style:--skin={m.appearance.skin} title={m.name}><SimPreview appearance={lookAtAge(content, assets, m.appearance, m.age)} gender={m.gender} size={36} /></span>
                  {/each}
                </span>
                <span class="info">
                  <b>The {own.name}s</b>
                  <span class="where">Your own household</span>
                  <span class="members">{own.members.map((m) => m.name).join(', ')}</span>
                </span>
              </button>
              <div class="card-actions">
                <button class="btn ghost small" onclick={create}>Change</button>
                <button class="btn small go" onclick={chooseOwn}>Choose →</button>
              </div>
            {:else}
              <button class="pick create" onclick={create}>
                <span class="plus" aria-hidden="true"><Icon name="icon.ui.plus" size={22} /></span>
                <span class="info">
                  <b>Create your own</b>
                  <span class="where">Make a household in Creation, then choose its home.</span>
                </span>
              </button>
            {/if}
          </article>

          {#each offeredHouseholds as h (homeKey(h))}
            {@const where = whereOf(h)}
            {@const band = bands.get(homeKey(h))}
            <article
              class="card"
              data-home={homeKey(h)}
              class:hover={hover === h.slot}
              class:focus={sameHome(focus, h)}
              onmouseenter={() => setHover(h.slot)}
              onmouseleave={() => setHover(null)}
            >
              <button class="pick" aria-label="Look at the {h.household.name}s' home" aria-pressed={sameHome(focus, h)} onclick={() => select(h)}>
                <span class="top">
                  <span class="faces">
                    {#each h.household.members as m (m.uid)}
                      <span class="face" style:--skin={m.appearance.skin} title={m.name}><SimPreview appearance={lookAtAge(content, assets, m.appearance, m.age)} gender={m.gender} size={36} /></span>
                    {/each}
                  </span>
                  {#if band}
                    <span class="band" title="{BAND_LABEL[band.band]} · {money(band.price)}" aria-label="{BAND_LABEL[band.band]}, {money(band.price)}">
                      {#each [1, 2, 3] as n (n)}<span class:on={n <= band.band}>{currency}</span>{/each}
                    </span>
                  {/if}
                </span>
                <span class="info">
                  <b>The {h.household.name}s</b>
                  <span class="where">{where.street} · {where.place}{#if where.village}<em class="village">village</em>{/if}</span>
                  <span class="members">
                    {#each h.household.members as m (m.uid)}
                      {@const a = about(m)}
                      <span class="member" title={a.all ? `${a.who} · ${a.all}` : a.who}>{a.who}{#if a.traits}<em>{` · ${a.traits}`}</em>{/if}</span>
                    {/each}
                  </span>
                </span>
              </button>
              <div class="card-actions">
                <button class="btn small go" aria-label="Play the {h.household.name}s" onclick={() => choose(h)}>Play →</button>
              </div>
            </article>
          {:else}
            <p class="empty">Nobody lives here yet: you will be the first to move in.</p>
          {/each}
        </div>
      </section>
    {/if}
  </div>
{/snippet}

{#snippet figures()}
  {#if town}
    <span><b>{homeCount}</b> homes</span>
    <span><b>{town.households.length}</b> households</span>
    <span><b>{population}</b> residents</span>
    <span><b>{forSale}</b> for sale</span>
  {/if}
{/snippet}

{#snippet actions()}
  {#if chosen}
    {#key homeKey(chosen)}
      <button class="btn primary" onclick={() => choose(chosen)}>Play the {chosen.household.name}s →</button>
    {/key}
  {:else if app.household}
    <button class="btn primary" onclick={chooseOwn}>The {app.household.name}s: choose a home →</button>
  {:else}
    <button class="btn primary" disabled={!town} onclick={create}>Create your own →</button>
  {/if}
{/snippet}

<SetupShell
  label="Your household"
  step={stepLabel(4)}
  back={{
    label: 'Build',
    go: () => {
      keep();
      app.screen = 'town';
    },
  }}
  {head}
  {panel}
  {stage}
  stats={figures}
  {actions}
/>

<style>
  .town-name {
    margin: 2px 0 0;
    font-size: 22px;
    font-weight: 750;
    letter-spacing: -0.02em;
    line-height: 1.2;
    color: var(--text-strong);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 2px;
  }
  .row .btn {
    flex: 1 1 auto;
    justify-content: center;
    gap: 8px;
    height: 36px;
    font-size: 13px;
  }
  input[type='range'] {
    width: 100%;
  }
  /* The world beside the cards: the camera frames the world into the stage alone. */
  .split {
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) clamp(340px, 38%, 700px);
    grid-template-rows: minmax(0, 1fr);
    gap: 20px;
  }
  /* (Under the way on, which sits at the view's top right.) */
  .offer {
    margin-top: 56px;
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding: 12px 8px 8px 14px;
    animation: rise 520ms 80ms var(--ease) both;
  }
  .offer-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 0 6px 10px 2px;
  }
  .offer-head .eyebrow {
    color: #4a5163;
  }
  .offer-head .btn {
    gap: 6px;
  }
  .cards {
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    align-content: start;
    gap: 8px;
    padding: 2px 6px 6px 2px;
  }
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--surface) 55%, transparent);
    border: 1px solid var(--hairline);
    transition:
      background var(--fast) var(--ease),
      border-color var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
  }
  .card.hover {
    background: rgba(255, 255, 255, 0.85);
  }
  .card.focus {
    background: #fff;
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  /* "Create your own": first, and set apart by the accent. */
  .card.own {
    grid-column: 1 / -1;
    border-style: dashed;
    border-color: color-mix(in srgb, var(--accent) 45%, transparent);
    background: color-mix(in srgb, var(--accent-soft) 60%, transparent);
  }
  .card.own.made {
    flex-direction: row;
    align-items: center;
    border-style: solid;
  }
  .card.own.made .card-actions {
    padding: 0 10px 0 0;
  }
  .pick {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 10px 4px;
    text-align: left;
  }
  .card.own .pick {
    flex-direction: row;
    align-items: center;
    gap: 14px;
    padding: 12px;
  }
  .plus {
    flex: none;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: var(--text-inverse);
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .faces {
    display: flex;
    flex: none;
  }
  /* 40px across, 2px of it border: the portrait is drawn for the 36px hole it gets. */
  .face {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    overflow: hidden;
    /* A tint of the resident's own skin under the portrait while it is still being drawn. */
    background: color-mix(in srgb, var(--skin, #d9c3b0) 26%, var(--surface-muted));
    border: 2px solid var(--surface);
    margin-left: -12px;
    display: grid;
    place-items: start center;
    box-shadow: var(--shadow-sm);
  }
  .face:first-child {
    margin-left: 0;
  }
  .band {
    display: flex;
    gap: 1px;
    padding: 3px 8px;
    border-radius: var(--radius-pill);
    background: rgba(29, 34, 48, 0.06);
    font-size: 12px;
    font-weight: 750;
    color: var(--text-muted);
  }
  .band span:not(.on) {
    opacity: 0.28;
  }
  .band .on {
    color: #8a6a1f;
  }
  .info {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    color: var(--text-strong);
  }
  .where {
    color: var(--text-muted);
    font-size: 12px;
    font-weight: 550;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .card.own .where {
    white-space: normal;
  }
  .village {
    margin-left: 6px;
    padding: 0 6px;
    border-radius: var(--radius-pill);
    background: rgba(70, 140, 90, 0.14);
    color: #2f6a3e;
    font-style: normal;
    font-size: 11px;
  }
  .members {
    display: flex;
    flex-direction: column;
    margin-top: 2px;
    font-size: 12.5px;
  }
  /* One line per resident; long trait lists end in an ellipsis (all of them on hover). */
  .member {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .members em {
    color: var(--text-muted);
    font-style: normal;
  }
  .card.own .members {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    display: block;
  }
  .card-actions {
    display: flex;
    justify-content: flex-end;
    gap: 4px;
    padding: 0 8px 8px;
  }
  .btn.small {
    height: 30px;
    padding: 0 11px;
    font-size: 12.5px;
  }
  .go {
    border-color: transparent;
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: none;
    font-weight: 700;
  }
  .go:hover,
  .card.focus .go,
  .go:focus-visible {
    background: var(--accent);
    color: var(--text-inverse);
  }
  .empty {
    grid-column: 1 / -1;
    color: var(--text-muted);
    padding: 8px;
    margin: 0;
  }
  /* Too narrow for the cards beside the world: the world over them. */
  @container setup (max-width: 1000px) {
    .split {
      grid-template-columns: minmax(0, 1fr) clamp(280px, 46%, 420px);
    }
  }
  @container setup (max-width: 760px) {
    /* The cards are what this step is for: the world and its cards get more of the height than the
       game's choices under them (step 1's and 2's frame shares it out evenly). */
    :global(.setup .layout:has(> .view .split)) {
      grid-template-rows: auto minmax(0, 1.8fr) minmax(0, 1fr);
    }
    .split {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: minmax(110px, 0.7fr) minmax(0, 1.3fr);
      gap: 10px;
    }
    .offer {
      margin-top: 0;
    }
    .cards {
      grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
