<script lang="ts" module>
  /** A part of making a step's world, as the progress panel lists it (the job's `WorldStep`s, then the renderer's `TownBuildStep`s). */
  export interface MakingStep {
    key: string;
    label: string;
  }

  /** A world being made: the panel's title, its parts in order, and those done so far (they may tick off in any order). */
  export interface Making {
    title: string;
    steps: readonly MakingStep[];
    done: ReadonlySet<string>;
  }
</script>

<script lang="ts">
  /**
   * The new game's first three steps (Land, Streets & zones, Build) share this frame: one slim panel down the left (a
   * head with the way back, the step and what names the world, then its choices, scrolling when
   * they don't fit), the world in the rest of the screen, the way on at its top right, the world in
   * figures at its top left, and the making panel over it while a world is made.
   *
   * What goes in the panel comes from the step (`head`, `panel`); so do the classes it may use from
   * here: `.group` (a section of the panel), `.hint`, `.eyebrow.sub`, `.options` and `.option` (a
   * row of choices), `.lede`, `.name-field` (a field with a button at its end).
   */
  import type { Snippet } from 'svelte';
  import { menuScene } from '../../game/menuScene';
  import Icon from '../Icon.svelte';

  let {
    label,
    step,
    back,
    head,
    panel,
    stage,
    stats,
    actions,
    making = null,
  }: {
    /** The panel's name for assistive technology ("A new world"). */
    label: string;
    /** "Step 1 of 5 · Land" (`newGame.ts` `stepLabel`). */
    step: string;
    back: { label: string; go: () => void };
    head?: Snippet;
    panel: Snippet;
    /** The world (a `TownStage`), or a stand-in while there is none. */
    stage: Snippet;
    stats?: Snippet;
    actions: Snippet;
    making?: Making | null;
  } = $props();

  /** The first part not done yet (the one under way). */
  const current = $derived(making?.steps.find((s) => !making.done.has(s.key))?.key ?? null);
  const progress = $derived(making ? making.steps.filter((s) => making.done.has(s.key)).length / Math.max(1, making.steps.length) : 0);

  /** Keeps lot tags clear of this element. */
  const avoid = (el: HTMLElement) => ({ destroy: menuScene.avoid(el) });
</script>

<div class="screen scaled setup">
  <div class="layout">
    <aside class="side menu-glass" use:avoid aria-label={label}>
      <div class="head" use:avoid>
        <button class="btn ghost back" onclick={back.go}><Icon name="icon.ui.back" size={16} /> {back.label}</button>
        <span class="eyebrow">{step}</span>
        {@render head?.()}
      </div>
      <div class="panel" use:avoid>
        {@render panel()}
      </div>
    </aside>

    <section class="view">
      {@render stage()}
      {#if stats}
        <div class="stats" use:avoid aria-label="In figures">{@render stats()}</div>
      {/if}
      <div class="actions" use:avoid>{@render actions()}</div>
      {#if making}
        <div class="making menu-glass" role="status" aria-live="polite" aria-label={making.title}>
          <span class="eyebrow">{making.title}</span>
          <div class="bar"><div class="fill" style:transform="scaleX({progress})"></div></div>
          <ol>
            {#each making.steps as s (s.key)}
              <li class:done={making.done.has(s.key)} class:now={s.key === current}>
                <span class="mark" aria-hidden="true">{#if making.done.has(s.key)}✓{:else if s.key === current}<span class="spinner"></span>{/if}</span>
                {s.label}
              </li>
            {/each}
          </ol>
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  .screen {
    --pad: 18px;
    position: relative;
    /* (Viewport units count in the UI scale.) */
    height: calc(100vh / var(--ui-scale));
    container: setup / inline-size;
  }
  .layout {
    height: 100%;
    display: grid;
    grid-template-columns: clamp(300px, 24%, 340px) minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr);
    grid-template-areas: 'side view';
    padding: 20px 28px 24px;
    gap: 24px;
  }
  /* (As tall as its choices, and no taller than the screen: then they scroll.) */
  .side {
    grid-area: side;
    align-self: start;
    max-height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: rise 520ms var(--ease) both;
  }
  .head {
    grid-area: head;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 5px;
    padding: 10px var(--pad) 14px;
    border-bottom: 1px solid var(--hairline);
  }
  .back {
    height: 32px;
    padding: 0 10px;
    margin: 0 0 2px -10px;
    gap: 6px;
    font-size: 13px;
    color: var(--text-strong);
  }
  .setup :global(.eyebrow) {
    margin: 0;
    color: #4a5163;
  }
  .setup :global(.eyebrow.sub) {
    font-size: 10.5px;
    color: var(--text-muted);
  }
  /* A field you can type in, not a heading: a surface and a hairline of its own at rest, a solid
     field under the pointer, the accent ring while it has the caret; its buttons sit in its end. */
  .setup :global(.name-field) {
    align-self: stretch;
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 4px 0 12px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--surface) 62%, transparent);
    transition:
      background var(--fast) var(--ease),
      border-color var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
  }
  .setup :global(.name-field:hover) {
    background: var(--surface);
  }
  .setup :global(.name-field:focus-within) {
    background: var(--surface);
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .setup :global(.name-field input) {
    flex: 1;
    min-width: 0;
    height: 42px;
    padding: 0;
    font-size: 22px;
    font-weight: 750;
    letter-spacing: -0.02em;
    color: var(--text-strong);
    background: transparent;
    border: none;
    box-shadow: none;
  }
  .setup :global(.name-field input:focus) {
    border: none;
    box-shadow: none;
  }
  .setup :global(.name-field .roll) {
    flex: none;
    width: 34px;
    height: 34px;
    padding: 0;
    color: #4a5163;
  }
  /* (Two lines kept for it: the choices under it stay put from world to world.) */
  .setup :global(.lede) {
    margin: 2px 0 0;
    min-height: 2lh;
    font-size: 13px;
    font-weight: 550;
    font-style: italic;
    line-height: 1.4;
    color: #3b4254;
    animation: rise 420ms var(--ease) both;
  }
  /* The choices: they scroll under the head when the window is too short for them all. */
  .panel {
    grid-area: panel;
    flex: 1;
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
  }
  .setup :global(.group) {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px var(--pad) 14px;
  }
  .setup :global(.group + .group) {
    border-top: 1px solid var(--hairline);
  }
  .setup :global(.group .segmented) {
    font-size: 13px;
  }
  .setup :global(.hint) {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
    color: var(--text-muted);
  }
  .setup :global(.options) {
    display: grid;
    gap: 8px;
  }
  .setup :global(.option) {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 13px;
  }
  .setup :global(.btn.small) {
    height: 30px;
    padding: 0 11px;
    font-size: 12.5px;
  }
  .view {
    grid-area: view;
    position: relative;
    min-height: 0;
    display: grid;
    grid-template-rows: minmax(0, 1fr);
  }
  /* The way on, at the top right over the world, a short haze behind it so it reads on roofs and roads. */
  .actions {
    position: absolute;
    top: 0;
    right: 0;
    z-index: 1;
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
    animation: rise 520ms 80ms var(--ease) both;
  }
  .actions::before {
    content: '';
    position: absolute;
    inset: -60px -60px -50px -90px;
    z-index: -1;
    background: radial-gradient(closest-side, rgba(252, 249, 243, 0.7), rgba(252, 249, 243, 0.35) 55%, rgba(252, 249, 243, 0));
    pointer-events: none;
  }
  /* A question asked at the way on: under it, at its right. */
  .actions :global(.confirm) {
    position: absolute;
    top: calc(100% + 10px);
    right: 0;
    width: min(340px, 100%);
    margin: 0;
  }
  /* The world in figures, small at the top left of the world (clear of the stream pill at the bottom). */
  .stats {
    position: absolute;
    top: 6px;
    left: 0;
    z-index: 1;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    max-width: calc(100% - 420px);
    color: #3b4254;
    font-size: 12px;
    font-weight: 550;
    pointer-events: none;
    animation: rise 520ms 120ms var(--ease) both;
  }
  .stats :global(span) {
    padding: 5px 10px;
    border-radius: var(--radius-pill);
    background: var(--glass-menu);
    border: 1px solid var(--glass-menu-border);
    box-shadow: var(--shadow-sm);
    backdrop-filter: blur(12px) saturate(1.4);
    -webkit-backdrop-filter: blur(12px) saturate(1.4);
  }
  .stats :global(b) {
    color: var(--text-strong);
    font-weight: 750;
  }
  /* The world's making, part by part: in the middle of the view (over the last world, if any, which stays in view meanwhile). */
  .making {
    position: absolute;
    inset: 0;
    margin: auto;
    width: min(320px, 100%);
    height: max-content;
    z-index: 2;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px 24px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-strong);
    pointer-events: none;
    animation: rise 260ms var(--ease) both;
  }
  .making .bar {
    height: 6px;
    border-radius: 3px;
    overflow: hidden;
    background: color-mix(in srgb, var(--text-strong) 12%, transparent);
  }
  .making .fill {
    height: 100%;
    background: var(--accent);
    transform-origin: left;
    transition: transform 300ms var(--ease);
  }
  .making ol {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 6px;
  }
  .making li {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-muted);
    transition: color var(--fast) var(--ease);
  }
  .making li.now,
  .making li.done {
    color: var(--text-strong);
  }
  .making .mark {
    flex: none;
    width: 16px;
    display: grid;
    place-items: center;
    color: var(--accent);
  }
  .making .mark .spinner {
    width: 12px;
    height: 12px;
  }
  /* A window not much wider than the panel and the world: the stream pill, centred at the window's
     bottom edge, would reach the panel's end; the panel stops short of it (its choices scroll sooner),
     and the figures make way for the buttons. */
  @container setup (max-width: 1000px) {
    .side {
      max-height: calc(100% - 72px / var(--ui-scale));
    }
  }
  @container setup (max-width: 1120px) {
    .stats {
      display: none;
    }
  }
  /* Too narrow for the world beside the panel (a small window or a large UI scale): the head on top,
     the way on under it, the world, then the choices in a panel of their own. The figures are left out. */
  @container setup (max-width: 760px) {
    .layout {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: auto minmax(0, 1fr) minmax(0, 1fr);
      grid-template-areas: 'head' 'view' 'panel';
      gap: 12px;
      /* (The stream pill, centred at the window's bottom edge, gets a strip of world of its own under the panel.) */
      padding: 16px 20px calc(70px / var(--ui-scale) + 26px);
    }
    .side {
      display: contents;
    }
    .head,
    .panel {
      background: var(--glass-menu);
      border: 1px solid var(--glass-menu-border);
      border-radius: var(--radius-lg);
      box-shadow: var(--glass-menu-shadow);
      backdrop-filter: blur(22px) saturate(1.5);
      -webkit-backdrop-filter: blur(22px) saturate(1.5);
    }
    .head {
      display: grid;
      grid-template-columns: auto minmax(0, 1fr);
      align-items: center;
      gap: 6px 12px;
      padding: 10px 14px 12px;
    }
    .back {
      margin: 0 0 0 -6px;
    }
    .head > :global(:not(.back)) {
      grid-column: 1 / -1;
    }
    .head > :global(.eyebrow:nth-child(2)) {
      grid-column: 2;
      grid-row: 1;
    }
    .view {
      grid-template-rows: auto minmax(0, 1fr);
      gap: 10px;
    }
    /* (First in the view, over the world: before it, or its stand-in while the first world is made.) */
    .actions {
      order: -1;
      position: relative;
      justify-content: flex-start;
    }
    .actions::before {
      display: none;
    }
    .setup :global(.lede) {
      min-height: 0;
    }
    .setup :global(.options) {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px 16px;
    }
  }
  /* Narrower still (a phone's width): one option to a row. */
  @container setup (max-width: 560px) {
    .setup :global(.options) {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
