<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from '../Icon.svelte';
  import { services } from '../services';
  import { game } from '../state.svelte';
  import BuildTabs from './BuildTabs.svelte';
  import FirstTip from './FirstTip.svelte';

  /**
   * Build mode's dock at the foot of the screen: one frame for each tab (Furnish, Construct, Home),
   * so the tabs and the actions beside them never move. The tab's own header shares the top line;
   * its body sits below. While something is in hand the body folds away to the header line;
   * pointing at that line opens it again.
   */
  let {
    label,
    head,
    body,
    above,
    tip,
    tipId = '',
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
    /** The pointer came to the header line (and stays on the dock): the body is open while placing. */
    over?: boolean;
    style?: string;
  } = $props();

  const folded = $derived(!!game.placing && !over);
</script>

<section
  class="dock glass"
  class:folded
  aria-label={label}
  data-dock
  {style}
  onpointerleave={(e) => {
    if (!e.currentTarget.contains(document.activeElement)) over = false;
  }}
  onfocusin={() => (over = true)}
  onfocusout={(e) => {
    const next = e.relatedTarget;
    if (!(next instanceof Node) || !e.currentTarget.contains(next)) over = false;
  }}
>
  <div class="above">
    {#if tip && tipId}<FirstTip id={tipId}>{@render tip()}</FirstTip>{/if}
    {@render above?.()}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <header onpointerenter={() => (over = true)}>
    <BuildTabs />
    <div class="head">{@render head?.()}</div>
    <div class="actions">
      <button
        class="action"
        disabled={game.undoSteps === 0}
        title={game.undoSteps ? 'Undo the last change (Ctrl+Z / ⌘Z)' : 'Nothing to undo. Changes can be undone until time moves on.'}
        aria-label="Undo"
        onclick={() => services.controls.undo()}
      >
        <Icon name="icon.ui.undo" size={15} />
      </button>
      <button class="action" disabled={game.redoSteps === 0} title={game.redoSteps ? 'Redo what you undid (Ctrl+Shift+Z / ⌘⇧Z)' : 'Nothing to redo'} aria-label="Redo" onclick={() => services.controls.redo()}>
        <span class="mirror"><Icon name="icon.ui.undo" size={15} /></span>
      </button>
      <button
        class="action"
        class:on={game.eyedropper}
        aria-pressed={game.eyedropper}
        title="Eyedropper (E, or hold Alt): click a wall, floor, door, window or object to build or buy more of the same"
        aria-label="Eyedropper"
        onclick={() => services.controls.toggleEyedropper()}
      >
        <Icon name="icon.ui.eyedropper" size={15} />
      </button>
    </div>
  </header>
  <div class="drawer" inert={folded}>
    <div class="drawer-inner">{@render body()}</div>
  </div>
</section>

<style>
  .dock {
    --dock-body: 240px;
    position: absolute;
    left: 50%;
    bottom: var(--edge);
    transform: translateX(-50%);
    /* (Of the HUD, not the viewport: viewport units ignore the interface scale.) */
    width: min(1400px, calc(100% - 2 * var(--edge)));
    container: dock / inline-size;
    /* Never reach the top bar. */
    max-height: calc(100% - var(--edge) - var(--top));
    padding: 8px 10px 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: auto;
    transition: gap var(--slow) var(--ease);
  }
  .dock.folded {
    gap: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    min-width: 0;
  }
  .head {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .actions {
    flex: none;
    display: flex;
    gap: 2px;
  }
  .action {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  .action:hover:not(:disabled),
  .action.on {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .action:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .mirror {
    display: inline-flex;
    transform: scaleX(-1);
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
