<script lang="ts">
  import type { ObjectDef } from '../content/content';
  import Icon from './Icon.svelte';
  import BuildDock from './buy/BuildDock.svelte';
  import ItemCard from './buy/ItemCard.svelte';
  import ItemDetail from './buy/ItemDetail.svelte';
  import ItemHoverCard from './buy/ItemHoverCard.svelte';
  import { loversOf, paintable, summarize } from './buy/catalog';
  import { services } from './services';
  import { play } from './sfx';
  import { game } from './state.svelte';

  /**
   * Build mode's Furnish tab: the furniture catalog in the dock (categories as icons beside a row of
   * cards; what an item is worth shows on pointing), plus the owned object picked on the lot, in a
   * card of its own above the dock. (Walls, rooms, doors and windows are the Construct tab's.)
   */
  const content = services.content;
  const ALL = '';
  const OTHER = '·other';

  /** Content categories, plus "Other" for items whose category isn't listed. */
  const categories = (() => {
    const known = new Set(content.buyCategories.map((c) => c.id));
    const list = [...content.buyCategories];
    if (content.shop.some((o) => !known.has(o.category ?? ''))) list.push({ id: OTHER, label: 'Other' });
    return list;
  })();
  /** Each category's backdrop hue for its pictures (golden-angle steps from a warm start). */
  const hue = (id: string) => Math.round((Math.max(0, categories.findIndex((c) => c.id === id)) * 137.5 + 28) % 360);
  const known = new Set(content.buyCategories.map((c) => c.id));
  const categoryOf = (def: ObjectDef) => (known.has(def.category ?? '') ? def.category! : OTHER);
  const categoryLabel = (id: string) => (id === ALL ? 'Everything' : (categories.find((c) => c.id === id)?.label ?? ''));

  type Sort = 'default' | 'cheap' | 'pricey' | 'name';
  const sorts: { id: Sort; label: string }[] = [
    { id: 'default', label: 'Featured' },
    { id: 'cheap', label: 'Price ↑' },
    { id: 'pricey', label: 'Price ↓' },
    { id: 'name', label: 'Name' },
  ];

  let category = $state(ALL);
  /** Group within the open category ('' = all of it). */
  let group = $state('');
  let query = $state('');
  let sort = $state<Sort>('default');
  let affordableOnly = $state(false);
  let lovedOnly = $state(false);
  let filtersOpen = $state(false);
  let filtersEl = $state<HTMLElement>();
  /** The card pointed at, and where it is along the dock (0..1) for its hover card. */
  let hovered = $state<{ id: string; at: number } | null>(null);
  /** The pointer came to the dock's header line: the catalog is open while something is in hand. */
  let over = $state(false);

  /** The household's residents, and who of them loves each item (by their traits). */
  const residents = $derived(game.roster.filter((s) => game.households[s.household]?.player));
  const lovers = $derived.by(() => {
    const map = new Map<string, string[]>();
    for (const def of content.shop) {
      const names = loversOf(content, def, residents);
      if (names.length) map.set(def.id, names);
    }
    return map;
  });
  /** How many of each item stand on the home lot. */
  const owned = $derived.by(() => {
    const plot = game.households.find((h) => h.player)?.plot;
    const home = plot == null ? null : game.plots[plot];
    const map = new Map<string, number>();
    if (!home) return map;
    for (const o of game.objects) {
      if (o.x >= home.x && o.z >= home.z && o.x < home.x + home.w && o.z < home.z + home.d) map.set(o.def, (map.get(o.def) ?? 0) + 1);
    }
    return map;
  });

  const terms = $derived(query.trim().toLowerCase().split(/\s+/).filter(Boolean));
  /** A wish or a goal from the planner narrows the catalog to what offers its activity. */
  const wanted = $derived(game.buyFilter?.activity ? content.activity(game.buyFilter.activity) : undefined);
  /** Items passing the search, "Affordable" and "For us" filters, across all categories. */
  const matching = $derived(
    content.shop.filter((def) => {
      // Stairs are built with the Construct tab's Stairs tool.
      if (def.stairs) return false;
      if (affordableOnly && !game.affords(def.price ?? 0)) return false;
      if (lovedOnly && !lovers.has(def.id)) return false;
      if (wanted && !content.offers(def, wanted, game.buyFilter?.skill)) return false;
      if (game.buyFilter?.defs && !game.buyFilter.defs.has(def.id)) return false;
      if (!terms.length) return true;
      const text = summarize(content, def).haystack;
      return terms.every((t) => text.includes(t));
    }),
  );
  const counts = $derived.by(() => {
    const map = new Map<string, number>();
    for (const def of matching) map.set(categoryOf(def), (map.get(categoryOf(def)) ?? 0) + 1);
    return map;
  });
  /** The open category's groups, with how many matching items each holds. */
  const groups = $derived.by(() => {
    const defs = content.buyCategories.find((c) => c.id === category)?.groups ?? [];
    return defs.map((g) => ({ ...g, n: matching.filter((d) => categoryOf(d) === category && d.group === g.id).length }));
  });
  const groupRank = (def: ObjectDef) => {
    const i = groups.findIndex((g) => g.id === def.group);
    return i < 0 ? groups.length : i;
  };
  const items = $derived.by(() => {
    const list = category === ALL ? [...matching] : matching.filter((d) => categoryOf(d) === category && (!group || d.group === group));
    // Featured: a grouped category lists its groups in order (content order within each).
    if (sort === 'default' && groups.length) list.sort((a, b) => groupRank(a) - groupRank(b));
    if (sort === 'cheap') list.sort((a, b) => (a.price ?? 0) - (b.price ?? 0));
    else if (sort === 'pricey') list.sort((a, b) => (b.price ?? 0) - (a.price ?? 0));
    else if (sort === 'name') list.sort((a, b) => a.name.localeCompare(b.name));
    return list;
  });
  const filterCount = $derived(Number(affordableOnly) + Number(lovedOnly));

  const folded = $derived(!!game.placing && !over);
  const selected = $derived(game.buySelection === null ? null : (game.objects.find((o) => o.id === game.buySelection) ?? null));
  /** Something already owned, in hand, else the owned object picked on the lot: its card recolours, upgrades or sells it; a new purchase in hand that takes a colour, its colour. */
  const moving = $derived(game.placing?.objectId == null ? null : (game.objects.find((o) => o.id === game.placing!.objectId) ?? null));
  const pinned = $derived.by(() => {
    const obj = moving ?? selected;
    const def = obj && content.object(obj.def);
    if (obj && def) return { def, owned: obj };
    const buying = game.placing?.objectId === null ? content.object(game.placing.def) : undefined;
    return buying && paintable(services.assets, buying) ? { def: buying, owned: null } : null;
  });
  const hoveredDef = $derived(hovered && !folded ? content.object(hovered.id) : undefined);

  function pickCategory(id: string) {
    if (category !== id) play('tab');
    category = id;
    group = '';
    hovered = null;
  }

  function pickGroup(id: string) {
    if (group !== id) play('tab');
    group = id;
    hovered = null;
  }

  function point(id: string, card: HTMLElement) {
    const dock = card.closest('[data-dock]')?.getBoundingClientRect();
    const r = card.getBoundingClientRect();
    hovered = { id, at: dock && dock.width ? (r.left + r.width / 2 - dock.left) / dock.width : 0.5 };
  }

  function clearFilters() {
    query = '';
    affordableOnly = false;
    lovedOnly = false;
    game.buyFilter = null;
  }

  function onSearchKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (query) query = '';
      else (e.currentTarget as HTMLInputElement).blur();
      e.stopPropagation();
    } else if (e.key === 'Enter') {
      if (items.length === 1) services.controls.startPlacing(items[0].id);
      (e.currentTarget as HTMLInputElement).blur();
    }
  }

  function onWindowPointer(e: PointerEvent) {
    if (filtersOpen && filtersEl && !filtersEl.contains(e.target as Node)) filtersOpen = false;
  }
  function onFiltersKey(e: KeyboardEvent) {
    if (e.key !== 'Escape') return;
    filtersOpen = false;
    e.stopPropagation();
  }
</script>

<svelte:window onpointerdown={onWindowPointer} />

{#if game.furnishing}
  <BuildDock label="Furnish" bind:over tipId="furnish">
    {#snippet tip()}
      <b>Make it home.</b> Pick something to buy, then click on your lot to place it. Point at an item to see what it does for your household;
      <span class="heart">♥</span> marks what suits someone's personality. Click something you own to move, recolour, upgrade or sell it.
    {/snippet}

    {#snippet head()}
      <span class="title">{categoryLabel(category)} <small class="tabular">{items.length}</small></span>
      {#if game.buyFilter}
        <button class="chip active wish" title="Show everything" onclick={() => (game.buyFilter = null)}>
          {game.buyFilter.label} <Icon name="icon.ui.close" size={10} />
        </button>
      {/if}
      <div class="subgroups" role="tablist" aria-label="{categoryLabel(category)} groups">
        {#if groups.length}
          <button class="chip" role="tab" aria-selected={!group} class:active={!group} onclick={() => pickGroup('')}>All</button>
          {#each groups as g (g.id)}
            <button class="chip" role="tab" aria-selected={group === g.id} class:active={group === g.id} class:empty={g.n === 0} onclick={() => pickGroup(g.id)}>{g.label}</button>
          {/each}
        {/if}
      </div>
      <label class="search" title="Search by name, need, skill or personality">
        <Icon name="icon.ui.search" size={13} />
        <input type="search" placeholder="Search" bind:value={query} onkeydown={onSearchKey} />
      </label>
      <select class="sort" bind:value={sort} aria-label="Sort">
        {#each sorts as s (s.id)}<option value={s.id}>{s.label}</option>{/each}
      </select>
      <div class="filters" bind:this={filtersEl} onkeydown={onFiltersKey} role="presentation">
        <button class="chip" class:active={filterCount > 0} aria-expanded={filtersOpen} onclick={() => (filtersOpen = !filtersOpen)}>
          <Icon name="icon.ui.filters" size={13} />Filters{#if filterCount}<small class="tabular">{filterCount}</small>{/if}
        </button>
        {#if filtersOpen}
          <div class="popover glass" role="dialog" aria-label="Filters">
            <span class="eyebrow">Show</span>
            <div class="row">
              {#if !game.creative}
                <button class="chip" class:active={affordableOnly} aria-pressed={affordableOnly} onclick={() => (affordableOnly = !affordableOnly)}>Affordable</button>
              {/if}
              {#if lovers.size}
                <button class="chip love" class:active={lovedOnly} aria-pressed={lovedOnly} title="Things your household's personalities draw them to" onclick={() => (lovedOnly = !lovedOnly)}
                  >♥ For us <small class="tabular">{lovers.size}</small></button
                >
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {/snippet}

    {#snippet body()}
      <div class="body">
        <div class="cats" role="tablist" aria-label="Furniture">
          <button role="tab" aria-selected={category === ALL} aria-label="Everything" class:active={category === ALL} title="Everything · {matching.length}" onclick={() => pickCategory(ALL)}>
            <span class="cat-icon all"><Icon name="icon.ui.buy" size={14} /></span>
          </button>
          {#each categories as c (c.id)}
            {@const n = counts.get(c.id) ?? 0}
            <button
              role="tab"
              aria-selected={category === c.id}
              aria-label={c.label}
              class:active={category === c.id}
              class:empty={n === 0}
              style="--hue:{hue(c.id)}"
              title="{c.label} · {n}"
              onclick={() => pickCategory(c.id)}
            >
              <span class="cat-icon"><Icon name={c.icon ?? 'icon.ui.buy'} size={14} /></span>
            </button>
          {/each}
        </div>
        {#key `${category}/${group}`}
          <ul class="items" onpointerleave={() => (hovered = null)}>
            {#each items as def, i (def.id)}
              <li style="--i:{Math.min(i, 16)}">
                <ItemCard
                  {def}
                  model={def.model}
                  affordable={game.affords(def.price ?? 0)}
                  active={game.placing?.def === def.id && game.placing.objectId === null}
                  owned={owned.get(def.id) ?? 0}
                  lovers={lovers.get(def.id) ?? []}
                  tint={hue(categoryOf(def))}
                  onpick={() => ((over = false), (hovered = null), services.controls.startPlacing(def.id))}
                  onpoint={(card) => point(def.id, card)}
                />
              </li>
            {:else}
              <li class="none muted">
                {#if terms.length || affordableOnly || lovedOnly || game.buyFilter}
                  Nothing matches. <button class="link" onclick={clearFilters}>Clear filters</button>
                {:else}Nothing here yet.{/if}
              </li>
            {/each}
          </ul>
        {/key}
      </div>
    {/snippet}

    {#snippet above()}
      {#if hoveredDef && hovered}
        <ItemHoverCard def={hoveredDef} at={hovered.at} lovers={lovers.get(hoveredDef.id) ?? []} owned={owned.get(hoveredDef.id) ?? 0} />
      {/if}
      {#if pinned}
        <section class="pinned glass" aria-label={moving || !pinned.owned ? 'In hand' : 'Picked'}>
          {#if !moving && pinned.owned}
            <button class="close" aria-label="Put down" title="Done (Esc)" onclick={() => (game.buySelection = null)}><Icon name="icon.ui.close" size={12} /></button>
          {/if}
          <ItemDetail def={pinned.def} owned={pinned.owned} />
        </section>
      {/if}
    {/snippet}
  </BuildDock>
{/if}

<style>
  .title {
    flex: none;
    font-size: 13px;
    font-weight: 700;
    white-space: nowrap;
  }
  .title small {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
  }
  .subgroups {
    flex: 1;
    min-width: 0;
    display: flex;
    gap: 4px;
    overflow-x: auto;
    scrollbar-width: none;
    mask-image: linear-gradient(90deg, #000 calc(100% - 24px), transparent);
  }
  .chip {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 26px;
    padding: 0 10px;
    border-radius: var(--radius-pill);
    background: var(--hairline);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
    white-space: nowrap;
  }
  .chip.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .chip.empty {
    opacity: 0.45;
  }
  .chip small {
    font-size: 10px;
    opacity: 0.75;
  }
  .chip.love {
    color: #c2405f;
  }
  .chip.love.active {
    background: rgba(224, 96, 126, 0.16);
  }
  .search {
    flex: 0 1 220px;
    min-width: 110px;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 10px;
    border-radius: var(--radius-pill);
    background: var(--surface);
    border: 1px solid var(--hairline);
    color: var(--text-muted);
    transition: border-color var(--fast) var(--ease), box-shadow var(--fast) var(--ease);
  }
  .search:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: none;
    font: inherit;
    font-size: 12px;
    color: var(--text);
    user-select: text;
  }
  .sort {
    flex: none;
    height: 30px;
    padding: 0 8px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--hairline);
    background: var(--surface);
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }
  .filters {
    position: relative;
    flex: none;
  }
  .filters > .chip {
    height: 30px;
  }
  .popover {
    position: absolute;
    right: 0;
    bottom: calc(100% + 10px);
    width: 280px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: var(--glass-strong);
    z-index: 3;
  }
  .popover .row {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .body {
    display: flex;
    gap: 10px;
    height: var(--dock-body);
  }
  .cats {
    flex: none;
    display: grid;
    grid-template-columns: repeat(4, 28px);
    grid-auto-rows: 28px;
    gap: 3px;
    align-content: start;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--hairline);
    overflow-y: auto;
    scrollbar-width: none;
  }
  .cats button {
    display: grid;
    place-items: center;
    border-radius: var(--radius-sm);
    transition: background var(--fast) var(--ease);
  }
  .cats button:hover,
  .cats button.active {
    background: var(--surface);
  }
  .cats button.active {
    box-shadow: var(--shadow-sm);
  }
  .cats button.empty {
    opacity: 0.4;
  }
  .cat-icon {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 7px;
    background: hsl(var(--hue) 55% 90%);
    color: hsl(var(--hue) 45% 38%);
    transition: transform 260ms var(--ease);
  }
  .cat-icon.all {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .cats button:hover .cat-icon {
    transform: rotate(-8deg) scale(1.08);
  }
  .cats button.active .cat-icon {
    background: hsl(var(--hue) 60% 52%);
    color: #fff;
  }
  .cats button.active .cat-icon.all {
    background: var(--accent);
  }
  .items {
    flex: 1;
    min-width: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    grid-auto-rows: 120px;
    gap: 8px;
    overflow-y: auto;
    overscroll-behavior: contain;
    scroll-snap-type: y mandatory;
    margin: 0;
    padding: 2px 4px 2px 2px;
    list-style: none;
  }
  .items li {
    scroll-snap-align: start;
    animation: deal 420ms var(--ease) backwards;
    animation-delay: calc(var(--i) * 20ms);
  }
  @keyframes deal {
    from {
      opacity: 0;
      transform: translateY(10px) scale(0.94);
    }
  }
  .items li.none {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 4px;
  }
  /* The owned object picked or in hand: its card, at the right, out of the way of the lot. */
  .pinned {
    position: absolute;
    right: 0;
    bottom: 0;
    width: min(340px, 100%);
    max-height: min(52vh, 520px);
    overflow-y: auto;
    overscroll-behavior: contain;
    padding: 10px 12px;
    background: var(--glass-strong);
    pointer-events: auto;
  }
  .close {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 1;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--surface);
    box-shadow: var(--shadow-sm);
    color: var(--text-muted);
  }
  .heart {
    color: #e0607e;
  }
  .link {
    color: var(--accent);
    font-weight: 600;
  }
  .muted {
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
