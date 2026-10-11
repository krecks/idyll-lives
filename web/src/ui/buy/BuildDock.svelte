<script lang="ts">
  import type { Snippet } from 'svelte';
  import { settings } from '../../settings/settings.svelte';
  import Icon from '../Icon.svelte';
  import { play } from '../sfx';
  import { game } from '../state.svelte';
  import BuildTabs from './BuildTabs.svelte';
  import FirstTip from './FirstTip.svelte';

  /**
   * Build mode's dock at the foot of the screen: the same frame for each tab (Furnish, Construct,
   * Home), so the tabs never move. The tabs and the tab's own header share one line; its body sits
   * below, one row high by default (the lot stays in view) or several when expanded. While
   * something is in hand the body folds away to the header line; pointing at that line opens it.
   */
  let {
    label,
    head,
    body,
    above,
    tip,
    tipId = '',
    tall = false,
    over = $bindable(false),
    style = '',
  }: {
    label: string;
    /** The tab's part of the header line, after the tabs. */
    head?: Snippet;
    body: Snippet;
    /** Floating over the lot, just above the dock (hover cards, the picked object). */
    above?: Snippet;
    /** A first-use tip, shown above the dock until dismissed. */
    tip?: Snippet;
    tipId?: string;
    /** Always expanded (blueprints); the size toggle hides. */
    tall?: boolean;
    /** The pointer came to the header line (and stays on the dock): the body is open while placing. */
    over?: boolean;
    style?: string;
  } = $props();

  const expanded = $derived(tall || settings.catalogSize === 'expanded');
  const folded = $derived(!!game.placing && !over);

  function toggleSize() {
    play('tab');
    settings.catalogSize = settings.catalogSize === 'expanded' ? 'compact' : 'expanded';
  }
</script>

<section class="dock glass" class:expanded class:folded aria-label={label} data-dock {style} onpointerleave={() => (over = false)}>
  <div class="above">
    {#if tip && tipId}<FirstTip id={tipId}>{@render tip()}</FirstTip>{/if}
    {@render above?.()}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <header onpointerenter={() => (over = true)}>
    <BuildTabs />
    <div class="head">{@render head?.()}</div>
    {#if !tall}
      <button class="size" title={expanded ? 'Smaller: one row, more of the lot in view' : 'Bigger: more rows at once'} aria-label={expanded ? 'Smaller' : 'Bigger'} aria-pressed={expanded} onclick={toggleSize}>
        <Icon name="icon.ui.chevron" size={15} />
      </button>
    {/if}
  </header>
  <div class="drawer">
    <div class="drawer-inner">{@render body()}</div>
  </div>
</section>

<style>
  .dock {
    --dock-body: 124px;
    position: absolute;
    left: 50%;
    bottom: var(--edge);
    transform: translateX(-50%);
    width: min(1400px, calc(100vw - 2 * var(--edge)));
    /* Never reach the top bar. */
    max-height: calc(100vh - 2 * var(--edge) - 64px);
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: auto;
    transition: gap var(--slow) var(--ease);
  }
  .dock.expanded {
    --dock-body: clamp(250px, 44vh, 470px);
  }
  .dock.folded {
    gap: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .head {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .size {
    flex: none;
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-pill);
    background: var(--hairline);
    color: var(--text-muted);
    transition: transform var(--slow) var(--ease), color var(--fast) var(--ease);
  }
  .size:hover {
    color: var(--text);
  }
  .expanded .size {
    transform: rotate(180deg);
  }
  /* While something is in hand, the body folds away unless the pointer comes back to the header. */
  .drawer {
    display: grid;
    grid-template-rows: 1fr;
    transition:
      grid-template-rows var(--slow) var(--ease),
      opacity var(--slow) var(--ease);
  }
  .folded .drawer {
    grid-template-rows: 0fr;
    opacity: 0;
  }
  .drawer-inner {
    min-height: 0;
    overflow: hidden;
  }
  .above {
    position: absolute;
    left: 0;
    right: 0;
    bottom: calc(100% + 8px);
    pointer-events: none;
  }
</style>
