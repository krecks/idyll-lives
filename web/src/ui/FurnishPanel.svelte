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
   * Build mode's Furnish tab: the furniture catalog in the dock. A row of categories over a grid
   * of items; what an item is worth shows on pointing. The owned object picked on the lot gets a
   * card of its own above the dock.
   */
  const content = services.content;
  const ALL = '';
  const OTHER = '·other';
  const known = new Set(content.buyCategories.map((c) => c.id));

  /** Content categories, plus "Other" for items whose category isn't listed. */
  const categories = (() => {
    const list = [...content.buyCategories];
    if (content.shop.some((o) => !known.has(o.category ?? ''))) list.push({ id: OTHER, label: 'Other' });
    return list;
  })();
  /** Each category's backdrop hue for its pictures (golden-angle steps from a warm start). */
  const hue = (id: string) => Math.round((Math.max(0, categories.findIndex((c) => c.id === id)) * 137.5 + 28) % 360);
  const categoryOf = (def: ObjectDef) => {
    const id = def.category;
    return id && known.has(id) ? id : OTHER;
  };

  let category = $state(ALL);
  let query = $state('');
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
  /** Items passing the search and the wish, across all categories. */
  const matching = $derived(
    content.shop.filter((def) => {
      // Stairs are built with the Construct tab's Stairs tool.
      if (def.stairs) return false;
      if (wanted && !content.offers(def, wanted, game.buyFilter?.skill)) return false;
      if (game.buyFilter?.defs && !game.buyFilter.defs.has(def.id)) return false;
      if (!terms.length) return true;
      const text = summarize(content, def).haystack;
      return terms.every((t) => text.includes(t));
    }),
  );
  const items = $derived(category === ALL ? matching : matching.filter((def) => categoryOf(def) === category));

  const folded = $derived(!!game.placing && !over);
  const selected = $derived(game.buySelection === null ? null : (game.objects.find((o) => o.id === game.buySelection) ?? null));
  /** Something already owned, in hand, else the owned object picked on the lot: its card recolours, upgrades or sells it; a new purchase in hand that takes a colour, its colour. */
  const moving = $derived.by(() => {
    const placing = game.placing;
    if (!placing || placing.objectId == null) return null;
    return game.objects.find((o) => o.id === placing.objectId) ?? null;
  });
  const pinned = $derived.by(() => {
    const obj = moving ?? selected;
    const def = obj && content.object(obj.def);
    if (obj && def) return { def, owned: obj };
    const buying = game.placing?.objectId === null ? content.object(game.placing.def) : undefined;
    return buying && paintable(services.assets, buying) ? { def: buying, owned: null } : null;
  });

  /** The category row scrolls sideways when the dock is narrow; only then does its edge fade. */
  let catsEl = $state<HTMLElement>();
  let catsOverflow = $state(false);
  $effect(() => {
    const el = catsEl;
    if (!el) return;
    const updateOverflow = () => {
      catsOverflow = el.scrollWidth > el.clientWidth + 1;
    };
    updateOverflow();
    const observer = new ResizeObserver(updateOverflow);
    observer.observe(el);
    return () => observer.disconnect();
  });
  const hoveredDef = $derived(hovered && !folded ? content.object(hovered.id) : undefined);

  function pickCategory(id: string) {
    if (category !== id) play('tab');
    category = id;
    hovered = null;
  }

  function point(id: string, card: HTMLElement) {
    const dock = card.closest('[data-dock]')?.getBoundingClientRect();
    const r = card.getBoundingClientRect();
    hovered = { id, at: dock && dock.width ? (r.left + r.width / 2 - dock.left) / dock.width : 0.5 };
  }

  function clearFilters() {
    query = '';
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
</script>

{#if game.furnishing}
  <BuildDock label="Furnish" bind:over tipId="furnish">
    {#snippet tip()}
      <b>Make it home.</b> Pick something to buy, then click on your lot to place it. Point at an item to see what it does for your household;
      <span class="heart">♥</span> marks what suits someone's personality. Click something you own to move, recolour, upgrade or sell it.
    {/snippet}

    {#snippet head()}
      {#if game.buyFilter}
        <button class="chip active wish" title="Show everything" onclick={() => (game.buyFilter = null)}>
          {game.buyFilter.label} <Icon name="icon.ui.close" size={10} />
        </button>
      {/if}
      <label class="search" title="Search by name, need, skill or personality">
        <Icon name="icon.ui.search" size={13} />
        <input type="search" placeholder="Search furniture" bind:value={query} onkeydown={onSearchKey} />
      </label>
    {/snippet}

    {#snippet body()}
      <div class="body">
        <div class="cats" class:overflow={catsOverflow} bind:this={catsEl} role="tablist" aria-label="Furniture category">
          <button class="cat" role="tab" aria-selected={category === ALL} class:active={category === ALL} onclick={() => pickCategory(ALL)}>
            <span class="cat-icon all"><Icon name="icon.ui.buy" size={14} /></span>Everything
          </button>
          {#each categories as c (c.id)}
            <button
              class="cat"
              role="tab"
              aria-selected={category === c.id}
              class:active={category === c.id}
              style="--hue:{hue(c.id)}"
              title={c.label}
              onclick={() => pickCategory(c.id)}
            >
              <span class="cat-icon"><Icon name={c.icon ?? 'icon.ui.buy'} size={14} /></span>{c.label}
            </button>
          {/each}
        </div>
        {#key category}
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
                  onblur={() => (hovered = null)}
                />
              </li>
            {:else}
              <li class="none muted">
                {#if terms.length || game.buyFilter}
                  Nothing matches. <button class="link" onclick={clearFilters}>{game.buyFilter ? 'Clear filters' : 'Clear search'}</button>
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
  .chip {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 11px;
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
  .search {
    flex: 0 1 240px;
    min-width: 120px;
    margin-left: auto;
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
  .body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    height: var(--dock-body);
  }
  .cats {
    flex: none;
    display: flex;
    gap: 3px;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .cats.overflow {
    mask-image: linear-gradient(90deg, #000 calc(100% - 28px), transparent);
  }
  .cat {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 7px 0 3px;
    border-radius: var(--radius-pill);
    font-size: 11.5px;
    font-weight: 650;
    color: var(--text-muted);
    white-space: nowrap;
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  .cat:hover {
    color: var(--text);
    background: var(--hairline);
  }
  .cat.active {
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: inset 0 0 0 1px rgba(91, 124, 250, 0.22);
  }
  .cat-icon {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 6px;
    background: hsl(var(--hue) 38% 92%);
    color: hsl(var(--hue) 32% 42%);
  }
  .cat-icon.all {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .cat.active .cat-icon {
    background: var(--accent);
    color: var(--text-inverse);
  }
  .items {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(104px, 1fr));
    grid-auto-rows: 96px;
    gap: 8px;
    overflow-y: auto;
    overscroll-behavior: contain;
    margin: 0;
    padding: 2px 4px 2px 2px;
    list-style: none;
  }
  .items li {
    animation: deal 380ms var(--ease) backwards;
    animation-delay: calc(var(--i) * 16ms);
  }
  @keyframes deal {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.96);
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
