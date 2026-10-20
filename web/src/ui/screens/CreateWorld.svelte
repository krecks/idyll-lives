<script lang="ts">
  import { onMount } from 'svelte';
  import { menuScene } from '../../game/menuScene';
  import { generateWorld, loadTemplates, type NeighbourhoodDraft, type Templates } from '../../game/town';
  import { WORLD_KINDS, WORLDS, worldName, type WorldKind } from '../../game/world';
  import { app } from '../app.svelte';
  import { lifespanHint } from '../format';
  import Icon from '../Icon.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import TownStage, { type LotLabel } from '../kit/TownStage.svelte';
  import { services } from '../services';

  const { content } = services;
  let templates = $state.raw<Templates | null>(null);
  let world = $state.raw<NeighbourhoodDraft | null>(app.world);
  let kind = $state<WorldKind>(app.world?.world?.kind ?? 'medium');
  let name = $state(app.world?.name ?? '');
  /** The name was typed or rolled here (kept when the world is made again). */
  let named = $state(false);
  let hover = $state<number | null>(null);
  let focus = $state<number | null>(null);

  const homes = $derived(world ? world.slots.filter((s) => s.kind === 'house').length : 0);
  const farms = $derived(world ? world.slots.filter((s) => s.lot === 'farm').length : 0);
  const places = $derived(world ? world.slots.filter((s) => s.kind !== 'house').length : 0);

  // Public places and farms have their names on them; plain houses are left untagged (the town reads better).
  const labels = $derived<LotLabel[]>(
    world
      ? world.slots.flatMap((s): LotLabel[] => (s.kind !== 'house' ? [{ plot: s.index, title: s.name, kind: 'park' }] : s.lot === 'farm' ? [{ plot: s.index, title: s.name, kind: 'sale' }] : []))
      : [],
  );

  /** A new world of the chosen kind (a new seed: a different place every time). */
  function make() {
    if (!templates) return;
    world = generateWorld(content, templates, kind, undefined, named && name.trim() ? name.trim() : undefined);
    name = world.name;
    focus = null;
  }

  function choose(k: WorldKind) {
    if (k === kind && world) return;
    kind = k;
    make();
  }

  function rollName() {
    name = worldName(content, kind, Math.random);
    named = true;
  }

  function next() {
    if (!world) return;
    const chosen = { ...world, name: name.trim() || world.name };
    // The same world as before (back and forth): its neighbours stay.
    const same = app.town && app.town.seed === chosen.seed && app.town.world?.kind === chosen.world?.kind;
    app.town = same && app.town ? { ...app.town, name: chosen.name } : null;
    app.world = chosen;
    app.screen = 'neighbourhood';
  }

  onMount(async () => {
    templates = await loadTemplates();
    if (!world) {
      // Start from the town the main menu showed (the same valley, nobody living there yet).
      const showcase = await menuScene.showcaseDraft();
      world = { ...showcase, households: [] };
      kind = showcase.world?.kind ?? 'medium';
      name = world.name;
    }
  });

  $effect(() => {
    if (!world) return;
    void menuScene.show(world);
  });

  $effect(() => {
    menuScene.highlight({ hover, selected: focus, marked: [], outlines: true });
  });
  $effect(() => () => menuScene.highlight({}));

  /** Keeps lot tags clear of this element. */
  const avoid = (el: HTMLElement) => ({ destroy: menuScene.avoid(el) });
</script>

{#snippet glyph(k: WorldKind)}
  <svg viewBox="0 0 48 32" width="48" height="32" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round">
    {#if k === 'small'}
      <path d="M8 27V17l7-6 7 6v10M26 27V19l6-5 6 5v8M4 27h40" />
    {:else if k === 'medium'}
      <path d="M4 27V18l6-5 6 5v9M18 27V15l6-5 6 5v12M32 27V18l6-5 6 5v9M2 27h44" />
    {:else if k === 'dense'}
      <path d="M5 27V12h9v15M14 27V6h10v21M24 27V10h9v17M33 27V14h10v13M2 27h44M8 16h3M8 21h3M18 11h3M18 16h3M18 21h3M28 15h2M28 20h2M37 18h3M37 23h3" />
    {:else if k === 'farm'}
      <path d="M6 27V15l9-7 9 7v12M11 27v-6h8v6M30 27V9a4 4 0 0 1 8 0v18M2 27h44M38 18h6M41 15v12" />
    {:else if k === 'seaside'}
      <path d="M14 17V9l6-5 6 5v8M4 21c3-2 5-2 8 0s5 2 8 0 5-2 8 0 5 2 8 0 5-2 8 0M4 27c3-2 5-2 8 0s5 2 8 0 5-2 8 0 5 2 8 0 5-2 8 0" />
    {:else if k === 'mountain'}
      <path d="M2 27 16 7l8 11 6-7 16 16zM12 13l4 3 3-3M27 15l3 2 3-2" />
    {:else if k === 'lakeside'}
      <path d="M8 23c0-4 7-6 16-6s16 2 16 6-7 5-16 5-16-1-16-5zM14 23h8M28 25h4M38 15V7M38 7l-4 6h8z" />
    {:else}
      <path d="M10 27V21M10 21l-6 0 6-9 6 9zM10 14l-4 0 4-7 4 7zM24 27V19M24 19h-7l7-12 7 12zM38 27V21M38 21h-6l6-9 6 9zM38 14h-4l4-7 4 7zM2 27h44" />
    {/if}
  </svg>
{/snippet}

<div class="screen scaled">
  <div class="haze" aria-hidden="true"></div>
  <header>
    <button class="btn ghost back" onclick={() => app.toMenu()}><Icon name="icon.ui.back" size={18} /> Menu</button>
    <div class="title">
      <span class="eyebrow">Step 1 of 4 · A new world</span>
      <div class="name-row">
        <input class="input town-name" aria-label="Town name" bind:value={name} oninput={() => (named = true)} maxlength="24" />
        <button class="btn ghost" aria-label="Another name" title="Another name" onclick={rollName}><Icon name="icon.ui.dice" size={18} /></button>
      </div>
      {#if world?.world}
        {#key world.seed}<p class="lede">{world.world.tagline}</p>{/key}
      {/if}
    </div>
    <div class="actions">
      <button class="btn" disabled={!templates} onclick={make}><Icon name="icon.ui.dice" size={18} /> Another world</button>
      <button class="btn primary" disabled={!world} onclick={next}>Choose a household →</button>
    </div>
  </header>

  <main>
    <section class="view">
      {#if world}
        <TownStage {labels} {hover} selected={focus} {focus} onhover={(p) => (hover = p)} onselect={(p) => (focus = p === focus ? null : p)} />
      {:else}
        <div></div>
      {/if}
      {#if world}
        <div class="controls menu-glass" use:avoid>
          <div class="control">
            <span class="eyebrow">Game</span>
            <Segmented
              label="Game"
              bind:value={app.mode}
              options={[
                { value: 'living', label: 'Living' },
                { value: 'creative', label: 'Creative' },
              ]}
            />
            <span class="hint">{app.mode === 'creative' ? 'Building is free. No rent or bills at home.' : 'Your residents earn the money to build with.'}</span>
          </div>
          <div class="control">
            <span class="eyebrow">Lifespan</span>
            <Segmented
              label="Lifespan"
              bind:value={app.lifespan}
              options={[
                { value: 'off', label: 'Off' },
                { value: 'short', label: 'Short' },
                { value: 'normal', label: 'Normal' },
                { value: 'long', label: 'Long' },
              ]}
            />
            <span class="hint">{lifespanHint(app.lifespan)}</span>
          </div>
          <div class="stats" aria-label="World summary">
            <span><b>{world.streets.length}</b> {world.streets.length === 1 ? 'street' : 'streets'}</span>
            <span><b>{homes}</b> homes</span>
            {#if farms}<span><b>{farms}</b> {farms === 1 ? 'farm' : 'farms'}</span>{/if}
            <span><b>{places}</b> public {places === 1 ? 'place' : 'places'}</span>
          </div>
        </div>
      {/if}
    </section>

    <aside class="kinds menu-glass">
      <header class="list-head">
        <span class="eyebrow">Where it takes place</span>
      </header>
      <div class="list" role="radiogroup" aria-label="Kind of world">
        {#each WORLD_KINDS as k (k)}
          <button class="kind" role="radio" aria-checked={kind === k} class:on={kind === k} disabled={!templates} onclick={() => choose(k)}>
            <span class="glyph">{@render glyph(k)}</span>
            <span class="info">
              <b>{WORLDS[k].label}</b>
              <span>{WORLDS[k].blurb}</span>
            </span>
          </button>
        {/each}
      </div>
    </aside>
  </main>
</div>

<style>
  .screen {
    position: relative;
    /* (Viewport units count in the UI scale.) */
    height: calc(100vh / var(--ui-scale));
    display: grid;
    grid-template-rows: auto 1fr;
    padding: 20px 28px 24px;
    gap: 14px;
  }
  /* Light haze behind the header so its text reads on any part of the scene. */
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
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .title .eyebrow {
    color: #4a5163;
  }
  .name-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .town-name {
    max-width: 380px;
    font-size: 26px;
    font-weight: 750;
    letter-spacing: -0.025em;
    height: 46px;
    color: var(--text-strong);
    background: transparent;
    border-color: transparent;
    padding-left: 0;
  }
  .town-name:hover,
  .town-name:focus {
    background: var(--surface);
    padding-left: 12px;
  }
  .lede {
    margin: 0;
    font-size: 14px;
    font-weight: 550;
    font-style: italic;
    color: #3b4254;
    animation: rise 420ms var(--ease) both;
  }
  .actions {
    display: flex;
    gap: 10px;
  }
  main {
    position: relative;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr minmax(360px, 420px);
    /* One row as tall as the screen allows: a long side panel scrolls rather than stretching the town view. */
    grid-template-rows: minmax(0, 1fr);
    gap: 24px;
  }
  .view {
    position: relative;
    min-height: 0;
    display: grid;
    grid-template-rows: 1fr auto;
    gap: 14px;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px 28px;
    padding: 14px 20px;
    justify-self: start;
    animation: rise 520ms var(--ease) both;
  }
  .control {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 200px;
  }
  .hint {
    font-size: 12px;
    color: var(--text-muted);
    max-width: 210px;
  }
  .controls .eyebrow,
  .list-head .eyebrow {
    color: #4a5163;
  }
  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    color: #3b4254;
    font-size: 13px;
    font-weight: 550;
  }
  .stats span {
    padding: 5px 10px;
    border-radius: var(--radius-pill);
    background: rgba(29, 34, 48, 0.06);
  }
  .stats b {
    color: var(--text-strong);
    font-weight: 750;
  }
  .kinds {
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding: 16px 12px 12px 16px;
    align-self: start;
    max-height: 100%;
    animation: rise 520ms 80ms var(--ease) both;
  }
  .list-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 0 6px 10px 2px;
    border-bottom: 1px solid var(--hairline);
    margin-bottom: 8px;
  }
  .list {
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-right: 4px;
  }
  .kind {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 12px;
    text-align: left;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    color: var(--text-strong);
    transition:
      background var(--fast) var(--ease),
      border-color var(--fast) var(--ease);
  }
  .kind:hover {
    background: rgba(255, 255, 255, 0.75);
  }
  .kind.on {
    background: #fff;
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .glyph {
    flex: none;
    display: grid;
    place-items: center;
    width: 60px;
    height: 44px;
    border-radius: var(--radius-md);
    background: var(--surface-muted);
    color: #4a5163;
  }
  .kind.on .glyph {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .info span {
    color: var(--text-muted);
    font-size: 12.5px;
    font-weight: 550;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
